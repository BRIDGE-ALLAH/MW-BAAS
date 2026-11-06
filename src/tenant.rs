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
//! This module currently defines the safe integration contract only. It does
//! not ship a tenant registry, secret-manager adapter, connection-pool manager,
//! request middleware, provisioning service, or migration orchestrator. Do not
//! treat enabling this module as enabling production tenant isolation.

use async_trait::async_trait;
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
    async fn resolve(
        &self,
        principal: &VerifiedTenantPrincipal,
    ) -> Result<TenantDatabase>;
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
        for value in ["", "../tenant", "tenant/a", "tenant a", "tenant?x=1", "租户"] {
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
}
