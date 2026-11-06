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
- [ ] Migrate package identity only after compatibility gates.

## References

- [MW-ORM repository](https://github.com/BRIDGE-ALLAH/MW-ORM)
- [MW-ORM architecture](https://github.com/BRIDGE-ALLAH/MW-ORM/blob/master/docs/ARCHITECTURE.md)
- [MW-ORM implementation blueprint](https://github.com/BRIDGE-ALLAH/MW-ORM/blob/master/docs/BLUEPRINT.md)
- [MW-BAAS Cargo manifest](../../Cargo.toml)
