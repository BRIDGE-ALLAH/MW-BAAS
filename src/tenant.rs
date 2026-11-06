//! Tenant identity and database-routing contracts for database-per-tenant apps.
//!
//! This module deliberately does not infer tenant identity from request headers,
//! query parameters, or host names. An application must authenticate the caller,
//! validate the tenant claim against its trusted tenant registry, and only then
//! construct a VerifiedTenantPrincipal for its database provider.
//!
//! The framework's global AppContext::db remains the ordinary application
//! database. Tenant-aware handlers must use the database returned by a
//! TenantDatabaseProvider and must not fall back to the global pool.
//!
//! This module provides tenant identity types, a provider contract, and opt-in
//! Axum request middleware. It does not ship a tenant registry, token/session
//! verifier, secret-manager adapter, connection-pool manager/provider,
//! provisioning service, background-job propagation, or migration orchestrator.
//! Do not treat enabling this module as enabling production tenant isolation.

use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Method, Request, Uri},
    middleware::Next,
    response::Response,
};
use sea_orm::DatabaseConnection;

use crate::{Error, Result};

/// Stable tenant identifier used by the trusted tenant registry.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TenantId(String);

impl TenantId {
    /// Parse a tenant identifier.
    ///
    /// IDs are intentionally restricted to ASCII letters, digits, hyphens, and
    /// underscores to prevent ambiguous routing and unsafe identifier reuse.
    ///
    /// # Errors
    ///
    /// Returns an error when the ID is empty, too long, or contains unsupported
    /// characters. Parsing validates syntax only; it does not authenticate a
    /// tenant or prove that the tenant is active.
    pub fn parse(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.len() <= 128
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');

        if !valid {
            return Err(Error::Unauthorized("invalid tenant identifier".to_owned()));
        }

        Ok(Self(value))
    }

    /// Return the stable identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}


/// Authenticated tenant and subject identity passed to a tenant DB provider.
///
/// Construct this value only after validating the caller's authentication and
/// tenant membership against a trusted identity/tenant registry. The type is
/// an explicit trust-boundary marker; it does not verify tokens itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedTenantPrincipal {
    tenant_id: TenantId,
    subject: String,
}

impl VerifiedTenantPrincipal {
    /// Construct a principal from identity claims already verified by the app.
    ///
    /// Callers must not pass raw client-supplied tenant IDs here without
    /// authenticating the subject and confirming active tenant membership.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed tenant IDs or empty/oversized subjects.
    pub fn from_verified_claims(
        tenant_id: impl Into<String>,
        subject: impl Into<String>,
    ) -> Result<Self> {
        let tenant_id = TenantId::parse(tenant_id)?;
        let subject = subject.into();
        if subject.trim().is_empty() || subject.len() > 256 {
            return Err(Error::Unauthorized("invalid tenant subject".to_owned()));
        }

        Ok(Self { tenant_id, subject })
    }

    /// Return the verified tenant identifier.
    #[must_use]
    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Return the authenticated subject identifier.
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }
}

/// Database handle bound to the tenant identity used to resolve it.
#[derive(Clone)]
pub struct TenantDatabase {
    tenant_id: TenantId,
    connection: DatabaseConnection,
}

impl TenantDatabase {
    /// Create a tenant database handle for use by a provider implementation.
    ///
    /// This constructor is public so downstream secret-manager/pool adapters
    /// can implement TenantDatabaseProvider. Only pass a connection whose
    /// database target was selected from the trusted tenant registry.
    #[must_use]
    pub fn new(tenant_id: TenantId, connection: DatabaseConnection) -> Self {
        Self {
            tenant_id,
            connection,
        }
    }

    /// Return the tenant identity associated with this connection.
    #[must_use]
    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Borrow the tenant-scoped SeaORM connection.
    #[must_use]
    pub fn connection(&self) -> &DatabaseConnection {
        &self.connection
    }
}

/// Request metadata made available to the application's authentication adapter.
#[derive(Clone, Debug)]
pub struct TenantRequestContext {
    headers: HeaderMap,
    method: Method,
    uri: Uri,
}

impl TenantRequestContext {
    /// Borrow request headers for token/session verification.
    #[must_use]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Borrow the HTTP method.
    #[must_use]
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Borrow the request URI.
    #[must_use]
    pub fn uri(&self) -> &Uri {
        &self.uri
    }
}

/// Authenticates the request and validates its tenant membership.
///
/// Implementations must derive identity from verified credentials, not from an
/// untrusted tenant header, path segment, query parameter, or host name alone.
#[async_trait]
pub trait TenantRequestAuthenticator: Send + Sync {
    /// Authenticate a request and return its verified tenant principal.
    ///
    /// # Errors
    ///
    /// Returns an authorization error when credentials or tenant membership
    /// are missing, invalid, expired, or inactive.
    async fn authenticate(&self, request: &TenantRequestContext)
        -> Result<VerifiedTenantPrincipal>;
}

/// Dependencies required by the tenant database request middleware.
#[derive(Clone)]
pub struct TenantMiddlewareState {
    authenticator: Arc<dyn TenantRequestAuthenticator>,
    provider: Arc<dyn TenantDatabaseProvider>,
}

impl TenantMiddlewareState {
    /// Create middleware state from application-owned auth and database adapters.
    #[must_use]
    pub fn new(
        authenticator: Arc<dyn TenantRequestAuthenticator>,
        provider: Arc<dyn TenantDatabaseProvider>,
    ) -> Self {
        Self {
            authenticator,
            provider,
        }
    }
}

/// Authenticate, resolve, and inject a tenant-scoped database for one request.
///
/// Wire this middleware onto tenant-only routes using Axum's
/// from_fn_with_state. Routes outside that layer remain ordinary single-tenant
/// routes and must not be assumed to have a tenant database extension.
pub async fn tenant_database_middleware(
    State(state): State<TenantMiddlewareState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response> {
    let context = TenantRequestContext {
        headers: request.headers().clone(),
        method: request.method().clone(),
        uri: request.uri().clone(),
    };
    let principal = state.authenticator.authenticate(&context).await?;
    let database = state.provider.resolve(&principal).await?;
    request.extensions_mut().insert(database);
    Ok(next.run(request).await)
}

/// Resolves an authenticated tenant principal to its dedicated database.
///
/// Implementations belong to the application/control-plane integration and
/// must fail closed for unknown, suspended, deleted, or unresolved tenants.
/// They must use a tenant-specific database target and least-privilege
/// credential; returning a shared/default application pool violates this
/// contract. Pool caches must be keyed by immutable tenant database identity.
#[async_trait]
pub trait TenantDatabaseProvider: Send + Sync {
    /// Resolve a verified tenant principal to its dedicated database.
    ///
    /// # Errors
    ///
    /// Returns an error when the tenant is not active, registry or secret
    /// resolution fails, or its dedicated database cannot be reached.
    async fn resolve(&self, principal: &VerifiedTenantPrincipal) -> Result<TenantDatabase>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenant_id_accepts_safe_identifiers() {
        let id = TenantId::parse("tenant-01_prod").expect("valid tenant ID");
        assert_eq!(id.as_str(), "tenant-01_prod");
    }

    #[test]
    fn tenant_id_rejects_empty_or_unsafe_identifiers() {
        for value in [
            "",
            "../tenant",
            "tenant/a",
            "tenant a",
            "tenant?x=1",
            "租户",
        ] {
            assert!(
                TenantId::parse(value).is_err(),
                "expected rejection for {value:?}"
            );
        }
    }

    #[test]
    fn verified_principal_requires_subject() {
        assert!(VerifiedTenantPrincipal::from_verified_claims("tenant-a", "").is_err());
        assert!(VerifiedTenantPrincipal::from_verified_claims("tenant-a", "user-1").is_ok());
    }

    #[cfg(feature = "db-sqlite")]
    #[tokio::test]
    async fn middleware_injects_the_database_resolved_for_the_verified_tenant() {
        use axum::{
            body::to_bytes, http::StatusCode, middleware::from_fn_with_state, routing::get,
            Extension, Router,
        };
        use sea_orm::Database;
        use tower::ServiceExt;

        struct TestAuthenticator;

        #[async_trait]
        impl TenantRequestAuthenticator for TestAuthenticator {
            async fn authenticate(
                &self,
                _request: &TenantRequestContext,
            ) -> Result<VerifiedTenantPrincipal> {
                VerifiedTenantPrincipal::from_verified_claims("tenant-a", "subject-1")
            }
        }

        struct TestProvider {
            connection: DatabaseConnection,
        }

        #[async_trait]
        impl TenantDatabaseProvider for TestProvider {
            async fn resolve(&self, principal: &VerifiedTenantPrincipal) -> Result<TenantDatabase> {
                let tenant_id = TenantId::parse(principal.tenant_id().as_str())?;
                Ok(TenantDatabase::new(tenant_id, self.connection.clone()))
            }
        }

        let connection = Database::connect("sqlite::memory:")
            .await
            .expect("connect test database");
        let state = TenantMiddlewareState::new(
            Arc::new(TestAuthenticator),
            Arc::new(TestProvider { connection }),
        );
        let app = Router::new()
            .route(
                "/tenant",
                get(
                    |Extension(database): Extension<TenantDatabase>| async move {
                        database.tenant_id().as_str().to_owned()
                    },
                ),
            )
            .layer(from_fn_with_state(state, tenant_database_middleware));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/tenant")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("request should complete");
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read response body");
        assert_eq!(&body[..], b"tenant-a");
    }

    #[cfg(feature = "db-sqlite")]
    #[tokio::test]
    async fn middleware_fails_closed_when_authentication_fails() {
        use axum::{http::StatusCode, middleware::from_fn_with_state, routing::get, Router};
        use std::sync::atomic::{AtomicUsize, Ordering};
        use tower::ServiceExt;

        struct DenyAuthenticator;

        #[async_trait]
        impl TenantRequestAuthenticator for DenyAuthenticator {
            async fn authenticate(
                &self,
                _request: &TenantRequestContext,
            ) -> Result<VerifiedTenantPrincipal> {
                Err(Error::Unauthorized("authentication denied".to_owned()))
            }
        }

        struct MustNotResolve(Arc<AtomicUsize>);

        #[async_trait]
        impl TenantDatabaseProvider for MustNotResolve {
            async fn resolve(
                &self,
                _principal: &VerifiedTenantPrincipal,
            ) -> Result<TenantDatabase> {
                self.0.fetch_add(1, Ordering::SeqCst);
                Err(Error::Unauthorized(
                    "provider must not be called".to_owned(),
                ))
            }
        }

        let provider_calls = Arc::new(AtomicUsize::new(0));
        let state = TenantMiddlewareState::new(
            Arc::new(DenyAuthenticator),
            Arc::new(MustNotResolve(Arc::clone(&provider_calls))),
        );
        let app = Router::new()
            .route("/tenant", get(|| async { "should not reach handler" }))
            .layer(from_fn_with_state(state, tenant_database_middleware));

        let response = app
            .oneshot(Request::builder().uri("/tenant").body(Body::empty()).unwrap())
            .await
            .expect("request should complete");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(provider_calls.load(Ordering::SeqCst), 0);
    }



    #[cfg(feature = "db-sqlite")]
    #[tokio::test]
    async fn separate_tenant_databases_do_not_share_rows() {
        use std::collections::HashMap;

        use axum::{
            body::to_bytes, http::StatusCode, middleware::from_fn_with_state, routing::get,
            Extension, Router,
        };
        use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseBackend, Statement};
        use tower::ServiceExt;

        struct HeaderTestAuthenticator;

        #[async_trait]
        impl TenantRequestAuthenticator for HeaderTestAuthenticator {
            async fn authenticate(
                &self,
                request: &TenantRequestContext,
            ) -> Result<VerifiedTenantPrincipal> {
                let tenant_id = request
                    .headers()
                    .get("x-test-tenant")
                    .and_then(|value| value.to_str().ok())
                    .ok_or_else(|| Error::Unauthorized("missing test tenant".to_owned()))?;
                VerifiedTenantPrincipal::from_verified_claims(tenant_id, "test-subject")
            }
        }

        struct MapProvider(HashMap<String, DatabaseConnection>);

        #[async_trait]
        impl TenantDatabaseProvider for MapProvider {
            async fn resolve(
                &self,
                principal: &VerifiedTenantPrincipal,
            ) -> Result<TenantDatabase> {
                let tenant_id = principal.tenant_id().as_str();
                let connection = self
                    .0
                    .get(tenant_id)
                    .ok_or_else(|| Error::Unauthorized("tenant not registered".to_owned()))?;
                Ok(TenantDatabase::new(
                    TenantId::parse(tenant_id)?,
                    connection.clone(),
                ))
            }
        }

        async fn isolated_db(marker: &str) -> DatabaseConnection {
            let mut options = ConnectOptions::new("sqlite::memory:");
            options.max_connections(1);
            let db = Database::connect(options).await.expect("connect sqlite");
            db.execute_unprepared("CREATE TABLE tenant_marker (value TEXT NOT NULL)")
                .await
                .expect("create marker table");
            db.execute_unprepared(&format!(
                "INSERT INTO tenant_marker (value) VALUES ('{marker}')"
            ))
            .await
            .expect("insert marker");
            db
        }

        let mut databases = HashMap::new();
        databases.insert("tenant-a".to_owned(), isolated_db("only-a").await);
        databases.insert("tenant-b".to_owned(), isolated_db("only-b").await);
        let state = TenantMiddlewareState::new(
            Arc::new(HeaderTestAuthenticator),
            Arc::new(MapProvider(databases)),
        );
        let app = Router::new()
            .route(
                "/tenant",
                get(
                    |Extension(database): Extension<TenantDatabase>| async move {
                        let row = database
                            .connection()
                            .query_one(&Statement::from_string(
                                DatabaseBackend::Sqlite,
                                "SELECT value FROM tenant_marker",
                            ))
                            .await
                            .expect("query tenant database")
                            .expect("tenant marker exists");
                        row.try_get::<String>("", "value")
                            .expect("read tenant marker")
                    },
                ),
            )
            .layer(from_fn_with_state(state, tenant_database_middleware));

        for (tenant, expected) in [("tenant-a", "only-a"), ("tenant-b", "only-b")] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/tenant")
                        .header("x-test-tenant", tenant)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .expect("request should complete");
            assert_eq!(response.status(), StatusCode::OK);
            let body = to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("read response body");
            assert_eq!(&body[..], expected.as_bytes());
        }
    }
}

