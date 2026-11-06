# MW-BAAS ↔ MW-ORM Integration Contract

**Status:** Core ORM source pinned to MW-ORM; module kernel exposed behind an opt-in feature; end-to-end CI validation pending.

## Decision

MW-BAAS consumes MW-ORM. MW-ORM remains independently usable and must never depend on MW-BAAS product code. The current integration pins the `sea-orm` package to the MW-ORM Git revision below. A Cargo `[patch.crates-io]` entry is used so `sea-orm-migration` and other transitive consumers resolve the same ORM source and do not create duplicate `DatabaseConnection` types.

Before this integration change, MW-BAAS resolved SeaORM 2.0 from crates.io. This branch now patches the `sea-orm` package to an immutable MW-ORM revision. MW-ORM remains a SeaORM 2.0-derived codebase; shared ancestry and version numbers do not by themselves prove API, macro, migration, driver, or runtime compatibility. CI evidence is still required.

The pinned revision is an integration candidate, not a production compatibility certification. The PR CI must pass the full feature matrix before this change is considered validated.

## Ownership

| Concern | Owner |
|---|---|
| Entity, column, relation metadata; query and persistence primitives | MW-ORM |
| Transaction primitives | MW-ORM |
| Generic module runtime and add-on contracts, when implemented | MW-ORM |
| HTTP routes, API DTOs, application-facing services | MW-BAAS |
| Authentication and authorization policy | MW-BAAS |
| Tenant identity and request context | MW-BAAS |
| Product workflows and external integrations | MW-BAAS / application modules |
| Schema migration history | One explicitly selected owner per application |

Never run two ORMs as competing owners of the same models, schema, migration history, or transaction boundary. ORM entities must not be serialized directly as public API responses.

## Current pinned integration revision

- Repository: `https://github.com/BRIDGE-ALLAH/MW-ORM`
- Revision: `cfeeba8eb4a3e19f5ff42afc7bb07ff9c9674ba0`
- ORM package: `sea-orm` version `2.0.4` (upstream-compatible package identity retained).
- Module kernel package: `moonwitness-module` version `0.1.0`, optional behind BAAS feature `mw-modules`.
- Cargo patches crates.io `sea-orm` globally so `sea-orm-migration` resolves the same source.
- The module kernel is re-exported as `loco_rs::modules` when `mw-modules` is enabled. It currently offers typed manifests, deterministic dependency planning, a static registry, and synchronous prepare/activate/deactivate hooks. It does **not** run module migrations, manage persistence, or provide tenant isolation.

## Migration ownership decision for MW-BAAS

For applications built on MW-BAAS, the existing Loco/SeaORM migration runner remains the **single schema-migration owner** during this integration. The application migration crate and its migration-history table are authoritative. MW-ORM supplies the ORM and migration primitives consumed by that runner; the experimental `moonwitness-module` kernel does not execute migrations and must not introduce a second migration-history table or independently apply module schema changes.

Consequences:

- A BAAS application runs migrations through its existing Loco migration entry point only.
- ORM and migration crates must resolve the same patched MW-ORM `sea-orm` source.
- Module activation must not mutate schema. A module that requires schema changes is not eligible for production activation until a reviewed migration is added to the application's canonical migration crate and deployed through the normal migration pipeline.
- Migration ordering, locking, transactional behavior, rollback policy, and recovery remain governed by the application's existing SeaORM migration runner.
- Any future module-owned migration feature requires a separate design that delegates to this single owner; it must not create a competing runner or history.

This is the initial safe integration decision, not a claim that migration execution has already been validated against the pinned MW-ORM revision. That still requires a real downstream fixture and database-backed CI.

## Staged integration

### Stage 0 — Baseline inventory
- Record exact commits, Rust toolchains, feature flags, and CI results for both repositories.
- Map workspace crates and all SeaORM, migration, and SQLx references.
- Compare public APIs, derive macros, entity behavior, drivers, runtime/TLS features, and migration APIs.
- Preserve inherited copyright and license notices.

**Exit:** reproducible baseline and compatibility report.

### Stage 1 — Isolated compatibility spike
- Pin one exact MW-ORM Git revision in a dedicated branch or fixture.
- Compile a minimal application with one entity, relation, query, and transaction.
- Test SQLite and PostgreSQL separately where supported.
- Verify macros, migrations, feature unification, and downstream compilation.
- Confirm Cargo resolves `sea-orm` and `sea-orm-migration` to one patched source using the dependency tree and build output.

**Exit:** compile and integration tests pass without duplicate ORM ownership.

### Stage 2 — Adapter and contracts
- Add a narrow MW-BAAS adapter; do not expose the entire ORM as a BAAS API.
- Define supported MW-ORM versions and feature combinations.
- Keep API DTOs separate from ORM entities.
- Select one migration runner and one migration history owner.
- Add compatibility tests to CI.

**Exit:** adapter, migration, and compatibility tests pass.

### Stage 3 — Module composition
Only after the experimental module API stabilizes and module migration semantics are implemented and tested:
- Register BAAS service modules through stable contracts.
- Validate dependencies and compatibility before activation.
- Make lifecycle ordering deterministic and observable.
- Define failure and migration semantics.
- Keep product policies and workflows in BAAS modules.

**Exit:** executable reference host with dependent modules and integration tests.

### Stage 4 — Cutover
- Migrate one bounded service at a time.
- Rehearse data/schema migration and recovery.
- Compare query behavior and performance.
- Remove the old persistence path only after consumers pass.
- Publish an upgrade guide and operator runbook.

**Exit:** one schema owner, no untested migration, CI green, runbook available.

## Tenant isolation architecture decision

MW-BAAS has selected **a separate database per tenant**. The accepted decision and implementation contract are recorded in [ADR 0001 — Tenant Database per Tenant](adr/0001-tenant-database-per-tenant.md).

The tenant registry/control plane and tenant-scoped connection provider belong to MW-BAAS, not MW-ORM. MW-ORM supplies database/ORM primitives; it must not become a second tenant registry or routing authority. The application must resolve tenant identity from trusted authenticated context, map it through the registry, and provide tenant-scoped connections to repositories and services. No global pool or default-tenant fallback may be exposed to tenant-scoped code.

The existing Loco/SeaORM migration runner remains the single schema-migration owner. A BAAS orchestration layer may iterate over tenant databases and invoke the canonical migration runner for each approved target, recording per-tenant outcomes; it must not introduce a competing migration engine or history table.

This decision does not make the current `multi-tenancy` feature alias a security boundary. Production use remains blocked until tenant provisioning, distinct least-privilege credentials, scoped pool management, migration orchestration, and cross-tenant negative tests are implemented and reviewed.

## Tenant database routing API (initial contract)

The `with-db` build now exposes `loco_rs::tenant` with:

- `TenantId`: syntax-validated stable tenant identifier. Parsing is not authentication and does not prove that a tenant is active.
- `VerifiedTenantPrincipal`: explicit trust-boundary marker for an authenticated subject and tenant claim. Construct it only after token verification and active membership validation against the trusted registry.
- `TenantDatabaseProvider`: application-owned async adapter that resolves a verified principal to a dedicated tenant database.
- `TenantDatabase`: a handle that carries the tenant ID alongside its SeaORM connection.

Example wiring for a tenant-only Axum router (the adapters are application-owned):

```rust,ignore
use axum::{middleware::from_fn_with_state, routing::get, Extension, Router};
use loco_rs::tenant::{
    tenant_database_middleware, TenantDatabase, TenantMiddlewareState,
};

let tenant_state = TenantMiddlewareState::new(authenticator, database_provider);
let tenant_routes = Router::new()
    .route("/records", get(list_records))
    .layer(from_fn_with_state(tenant_state, tenant_database_middleware));

async fn list_records(Extension(tenant_db): Extension<TenantDatabase>) {
    let db = tenant_db.connection();
    // All tenant-scoped queries use this connection, never AppContext::db.
}
```

The route handler must not accept a tenant ID from the request and use it to select a database itself. The authenticator must verify credentials and active tenant membership; the provider must resolve the trusted registry entry and return only that tenant's dedicated connection. Do not put bearer tokens or full database URIs in logs.

This is an integration seam, not a completed runtime. No default provider is shipped. The framework now supplies an opt-in Axum request middleware seam: an application-owned authenticator validates credentials and tenant membership, then an application-owned provider resolves a dedicated database and the middleware injects a TenantDatabase extension. The framework still does not supply a tenant registry, token/session verifier, secrets manager, bounded pool cache/provider implementation, automatic route wiring, background-job propagation, provisioning lifecycle, per-tenant migration orchestration, or tenant-aware backup/restore. The ordinary global `AppContext::db` remains available for existing single-tenant applications; tenant-aware handlers must not use it as a fallback.

Before using the API in an application, implement and test an adapter that:
1. verifies the authentication token and derives the subject and tenant claim from trusted claims;
2. checks tenant status and membership against a trusted registry;
3. maps the immutable tenant ID to a secret reference and approved database target (never accept a client-supplied URI);
4. creates or retrieves a dedicated pool using an immutable database identity as the cache key, with bounded pool count/idle eviction and distinct least-privilege credentials;
5. injects the resulting TenantDatabase into request/job context only after successful resolution, failing closed on all errors;
6. applies the canonical Loco/SeaORM migration runner to each tenant database and records per-tenant outcomes.

The new `src/tenant.rs` tests cover tenant-ID syntax, principal input validation, request-level database extension injection, fail-closed behavior when authentication fails, and a two-independent-in-memory-SQLite-database routing smoke test (SQLite-enabled test builds). They do not prove production Postgres/MySQL separation, bounded pool isolation under concurrency, or cross-tenant safety across all data-access surfaces. Multi-tenant production remains blocked until a real downstream fixture exercises two isolated databases, concurrent requests, pool reuse, transactions, raw SQL, background jobs, exports, admin access, migration failure/retry, backup/restore, and offboarding.

## Security and integrity requirements

- Tenant isolation must cover queries, joins, raw SQL, background jobs, exports, and administrative paths.
- Authentication and authorization remain BAAS policy decisions; database filtering is not a substitute.
- Never trust client-supplied tenant or actor identifiers without authenticated context.
- Define audit actor, action, target, timestamp, provenance, retention, and access policy.
- Destructive migrations require review, backups, and operator-controlled plans.
- Module removal preserves user data by default.
- Do not claim tenant isolation or audit readiness until bypass paths and event coverage are tested.

## Current non-claims

This contract does not claim that the MW-ORM module kernel is production-ready, that a BAAS adapter exists, that tenant isolation is complete, that crate names have migrated, or that production migrations have been executed.

MW-ORM architecture documents describe the module manifest, resolver, registry, lifecycle, and add-ons as planned until implementation and tests land.

## Engineering checklist

- [ ] Capture baseline CI for both repositories.
- [x] Pin a specific MW-ORM revision and route the SeaORM package through it.
- [ ] Pass the complete PR feature matrix and module-kernel integration test.
- [ ] Compile entity, macro, query, relation, and transaction fixture.
- [ ] Verify database drivers and runtime/TLS feature combinations.
- [ ] Select one migration owner and test migration history.
- [ ] Expand ORM/migration adapter and compatibility tests.
- [ ] Build a reference application.
- [ ] Complete security review before tenant isolation claims.
- [x] Add a security and operations release baseline; policy approval and production evidence remain pending.
- [x] Approve tenant-isolation architecture ADR: separate database per tenant.
- [ ] Implement tenant registry, scoped connection provider, per-tenant credentials, and provisioning lifecycle.
- [ ] Pass cross-tenant negative tests for requests, pool reuse, transactions, jobs, raw SQL, exports, admin paths, migration, restore, and offboarding.
- [ ] Migrate package identity only after compatibility gates.

## Security and operations companion

See [Security and Operations Baseline](../operations/SECURITY-AND-OPERATIONS-BASELINE.md) for release blockers, tenant-isolation coverage, audit-policy decisions, deployment, backup/restore, recovery, and observability requirements.

## References

- [MW-ORM repository](https://github.com/BRIDGE-ALLAH/MW-ORM)
- [MW-ORM architecture](https://github.com/BRIDGE-ALLAH/MW-ORM/blob/master/docs/ARCHITECTURE.md)
- [MW-ORM implementation blueprint](https://github.com/BRIDGE-ALLAH/MW-ORM/blob/master/docs/BLUEPRINT.md)
- [MW-BAAS Cargo manifest](../../Cargo.toml)
