# MW-BAAS ↔ MW-ORM Integration Contract

**Status:** Architecture contract; implementation not yet proven.

## Decision

MW-BAAS consumes MW-ORM. MW-ORM remains independently usable and must never depend on MW-BAAS product code.

The current MW-BAAS repository is a Loco-derived Rust framework whose database feature uses SeaORM 2.0 from crates.io. MW-ORM is an independently evolving SeaORM 2.0-derived repository. Shared ancestry and version numbers do not prove API, macro, migration, driver, or runtime compatibility.

**Do not switch the production dependency to a Git revision until an isolated compatibility spike and CI matrix pass.**

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
- Keep the current crates.io dependency available until the spike passes.

**Exit:** compile and integration tests pass without duplicate ORM ownership.

### Stage 2 — Adapter and contracts
- Add a narrow MW-BAAS adapter; do not expose the entire ORM as a BAAS API.
- Define supported MW-ORM versions and feature combinations.
- Keep API DTOs separate from ORM entities.
- Select one migration runner and one migration history owner.
- Add compatibility tests to CI.

**Exit:** adapter, migration, and compatibility tests pass.

### Stage 3 — Module composition
Only after MW-ORM implements and tests its module kernel:
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
- [ ] Pin and test a specific MW-ORM revision.
- [ ] Compile entity, macro, query, relation, and transaction fixture.
- [ ] Verify database drivers and runtime/TLS feature combinations.
- [ ] Select one migration owner and test migration history.
- [ ] Add adapter and compatibility tests.
- [ ] Build a reference application.
- [ ] Complete security review before tenant isolation claims.
- [ ] Migrate package identity only after compatibility gates.

## References

- [MW-ORM repository](https://github.com/BRIDGE-ALLAH/MW-ORM)
- [MW-ORM architecture](https://github.com/BRIDGE-ALLAH/MW-ORM/blob/master/docs/ARCHITECTURE.md)
- [MW-ORM implementation blueprint](https://github.com/BRIDGE-ALLAH/MW-ORM/blob/master/docs/BLUEPRINT.md)
- [MW-BAAS Cargo manifest](../../Cargo.toml)
