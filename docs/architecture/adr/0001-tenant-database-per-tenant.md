# ADR: Tenant Isolation with a Separate Database per Tenant

**Status:** Accepted by the MW-BAAS product owner on 2026-11-06; implementation and production validation pending.

## Context

MW-BAAS needs a tenant isolation model that is understandable to operators and testable across request handling, persistence, background work, and recovery. A tenant identifier attached to a request is not itself a security boundary. Shared-database row scoping would require every query and access path to preserve scope, including raw SQL and administrative code.

## Decision

Use **one separately provisioned database per tenant** as the default isolation boundary.

The application must resolve an authenticated tenant identity through a trusted tenant registry/control plane and select that tenant's database configuration. Tenant identity supplied by an untrusted client must never directly select an arbitrary database or connection string.

## Required architecture

1. **Control plane / tenant registry**
   - Owns tenant lifecycle and maps an immutable tenant ID to an approved database target and secret reference.
   - Stores no database passwords in source control or ordinary application configuration.
   - Registry writes require privileged authorization and an audit event.
   - Unknown, suspended, deleted, or unresolved tenants fail closed.

2. **Data plane / tenant database**
   - Each tenant has a distinct database and distinct least-privilege database credential.
   - A tenant-scoped connection provider resolves only registry-approved targets.
   - A request or job receives a tenant-scoped database handle; it must not receive a global pool capable of selecting arbitrary tenant databases.
   - Pool identity is bound to the resolved tenant/database target. Connections must never be reassigned across tenant identities without a complete reset and verified target binding; the preferred implementation is distinct pools keyed by immutable tenant database identity.
   - No fallback to a default tenant database is permitted.

3. **Request and job context**
   - Tenant identity is derived from authenticated claims or a trusted routing layer and validated against the registry.
   - Background jobs, scheduled tasks, retries, and message consumers carry an immutable tenant ID and re-resolve it at execution time.
   - Missing, stale, or unauthorized tenant context fails closed.
   - Cross-tenant administration is a separate privileged workflow with explicit authorization, reason capture, and audit logging.

4. **Migrations and provisioning**
   - The application's canonical Loco/SeaORM migration runner remains the single schema-migration owner.
   - Provisioning creates a database and least-privilege credential, then applies the exact approved migration revision to that tenant database.
   - Migration orchestration records tenant ID, target identity, migration revision, operator/actor, outcome, and timestamps without exposing secrets.
   - Partial provisioning and migration failures leave the tenant unavailable until reconciled; they must not route traffic to another database.
   - Rollout, retry, rollback, and restore procedures must be rehearsed against isolated test databases.

5. **Backups, restore, and offboarding**
   - Backup, restore, deletion, retention, and offboarding operate per tenant database and any tenant-owned object storage.
   - Restore targets must be isolated and verified before traffic is switched.
   - Retention durations and deletion/legal-hold behavior remain application-owner policy decisions; this ADR does not invent them.

## Security invariants

- A tenant A principal cannot read, mutate, infer, export, or administer tenant B data.
- A database credential for tenant A cannot authenticate to tenant B's database.
- Tenant identity is never accepted solely from an untrusted request parameter.
- No shared global database connection is exposed to tenant-scoped application code.
- Tenant selection, registry changes, privileged access, provisioning, migrations, restore, and offboarding are auditable.
- Failures in tenant resolution, registry access, credential retrieval, connection establishment, or migration state fail closed.
- Logs, traces, metrics, cache keys, object keys, exports, and search indexes must preserve tenant boundaries and avoid secret or sensitive payload leakage.

## Required tests before enabling multi-tenant production

- Registry rejects unknown, suspended, deleted, and malformed tenant identities.
- Authenticated tenant A cannot request tenant B's database by changing headers, path, query, or message payload.
- A's credential cannot connect to B's database; database grants are least privilege.
- Concurrent A/B requests cannot exchange connections or leak results through pool reuse.
- Transactions and failures remain bound to the selected tenant database.
- Relations, joins, aggregates, pagination, raw SQL, and custom repositories operate only against the selected tenant database.
- Background jobs, retries, scheduled tasks, and message consumers re-resolve and enforce tenant identity.
- Cache, object-storage, export, search, and observability keys are tenant-qualified and access-controlled.
- Admin/support/break-glass paths require explicit privileged authorization and emit audit records.
- Provisioning, migration failure, retry, restore, and offboarding tests prove no cross-tenant routing or data exposure.
- Negative tests include missing context, stale context, malformed registry entries, unavailable secrets, connection failures, and pool saturation.

## Operational trade-offs

This model increases database provisioning, connection-pool, migration, backup, restore, monitoring, and cost-management complexity. The implementation must therefore include bounded pool management, idle pool eviction where safe, per-tenant quotas, lifecycle automation, and fleet-level health reporting. These optimizations must not weaken tenant-to-database binding.

## Scope and non-goals

This decision selects the isolation architecture; it does not claim that the tenant registry, database provisioning service, scoped connection provider, migration orchestrator, or production security controls already exist. It does not prescribe a universal database engine, cloud provider, retention duration, or consent policy. Supported database drivers and operational limits must be validated in a downstream reference application before production rollout.

## Rollout gates

- [ ] Implement trusted tenant registry and lifecycle state machine.
- [ ] Implement tenant-scoped connection provider and bounded pool strategy.
- [ ] Enforce distinct database credentials and least-privilege grants.
- [ ] Integrate canonical migration runner with per-tenant orchestration.
- [ ] Pass cross-tenant negative tests across all listed access surfaces.
- [ ] Validate per-tenant backup, restore, offboarding, and incident runbooks.
- [ ] Approve application-specific audit, consent, retention, and legal-hold policies.
- [ ] Pass full CI and a database-backed downstream reference application.
- [ ] Complete threat model and operational review before enabling production traffic.
