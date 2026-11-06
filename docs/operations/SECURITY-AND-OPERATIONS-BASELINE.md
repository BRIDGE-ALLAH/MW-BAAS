# Security and Operations Baseline

**Status:** Engineering baseline; policy decisions and production validation are pending.

This document records minimum controls for applications built on MW-BAAS. It is not a security certification, compliance statement, or substitute for an application-specific threat model.

## Security invariants

- Authentication establishes an actor; authorization must be checked at every protected operation.
- Never trust tenant, actor, role, or ownership identifiers supplied only by a client.
- A tenant identifier in a request context is not an isolation boundary by itself.
- Apply authorization to reads, writes, joins, aggregate queries, raw SQL, background jobs, exports, scheduled tasks, and administrative interfaces.
- Treat logs, traces, backups, exports, and support tooling as data access surfaces.
- Secrets must be supplied through the deployment secret manager or equivalent protected runtime configuration; do not commit credentials.
- Production modules must be statically reviewed and version-pinned. The current module kernel does not load arbitrary dynamic libraries.
- Destructive schema changes require review, a tested backup, and an operator-approved execution plan.

## Tenant isolation: separate database per tenant

**Decision accepted:** MW-BAAS uses a separate database per tenant. See [ADR 0001 — Tenant Database per Tenant](../architecture/adr/0001-tenant-database-per-tenant.md).

This is an architecture decision, not evidence that tenant isolation is already implemented. The required control plane, tenant registry, scoped connection provider, per-tenant credentials, provisioning, and migration orchestration remain implementation gates.

Do not enable multi-tenant production workloads until tests prove isolation for:
- ordinary queries and mutations;
- eager/lazy relations, joins, aggregates, and pagination;
- raw SQL and custom repositories;
- transactions and connection-pool reuse;
- background jobs, retries, and scheduled work;
- exports, reports, caches, search indexes, and object storage;
- admin, support, impersonation, and break-glass paths;
- migrations, restore, and tenant offboarding.

For the selected database-per-tenant model, negative tests must demonstrate that an actor from tenant A cannot read, mutate, infer, export, or administer tenant B data; tenant A credentials cannot connect to tenant B's database; and concurrent requests cannot exchange tenant-bound connections. Tests must include missing or stale tenant context, failed registry/secret resolution, and pool reuse.

The `multi-tenancy` Cargo feature remains a compatibility alias only, not a security capability. Do not enable multi-tenant production workloads until the ADR rollout gates pass.

## Audit and data governance: policy decisions pending

Before production, each application owner must approve:
- auditable event classes and required actor/action/target/time/request/provenance fields;
- which events are immutable or tamper-evident and who can inspect them;
- retention and deletion schedules by data class and jurisdiction;
- consent and lawful-purpose requirements, where applicable;
- access review, privileged support, and incident escalation procedures;
- treatment of backups, exports, logs, and derived datasets during deletion requests.

Do not invent a universal retention period or consent model in the framework. Applications must provide their approved policy and tests.

## Deployment checklist

### Before deployment
- Pin application, MW-BAAS, MW-ORM, and module revisions.
- Review dependency and license changes; run vulnerability and secret scanning.
- Run formatting, lint, unit, integration, feature-matrix, and database-driver tests.
- Apply schema changes only through the application's canonical Loco/SeaORM migration runner.
- Confirm migration source resolution and migration-history ownership.
- Verify configuration schema and secret references without printing secret values.
- Verify TLS, network policy, least-privilege database credentials, and restricted administrative access.
- Confirm backup completion and restore-test evidence.

### During deployment
- Use a staged rollout or maintenance window appropriate to the application.
- Record release revision, migration revision, operator, and start/end timestamps.
- Monitor startup, health checks, database connectivity, queue lag, error rate, and resource saturation.
- Stop rollout on failed migrations, unexpected data changes, authorization errors, or elevated error rates.

### Recovery
- Prefer forward fixes for additive migrations where safe.
- Restore from backup only under an approved incident procedure; coordinate application and database consistency.
- Preserve incident evidence and migration logs.
- Verify data integrity, access controls, and background-job state before reopening traffic.
- Record the root cause and preventive actions.

## Backup and restore
- Define RPO/RTO per application; this framework does not prescribe values.
- Back up database and required object/blob data with encryption and access controls.
- Define backup retention according to approved policy.
- Test restore into an isolated environment on a recurring, documented cadence.
- Validate schema version, row counts or application invariants, object references, and access controls after restore.
- Document who can request, approve, execute, and verify a restore.

## Observability
At minimum, operators should monitor:
- service availability and latency;
- application and database error rates;
- connection-pool saturation and slow queries;
- worker queue depth, retries, and dead-letter items;
- migration status and schema revision;
- authentication failures and authorization denials;
- privileged actions and audit-pipeline health;
- backup freshness and restore-test status.

Logs must avoid passwords, tokens, secret values, and unnecessary sensitive payloads. Define redaction and access controls before enabling production log collection.

## Release gates

- [x] Tenant isolation architecture ADR approved: database per tenant.
- [ ] Tenant registry, scoped connection provider, and per-tenant credentials implemented and reviewed.
- [ ] Cross-tenant negative tests pass across all access surfaces.
- [ ] Audit, consent, and retention policies approved by application owners.
- [ ] Database migration and rollback/recovery rehearsal passes.
- [ ] Backup restore test meets application RPO/RTO.
- [ ] Deployment, incident, and operator ownership documented.
- [ ] Full CI and downstream application fixture pass.

## Current implementation boundary

The MW-BAAS ↔ MW-ORM integration is still under review. The database-per-tenant architecture is approved, but this document does not claim that tenant isolation, tenant provisioning, scoped connection routing, audit policy enforcement, migration compatibility, or production operations have already been implemented or validated.
