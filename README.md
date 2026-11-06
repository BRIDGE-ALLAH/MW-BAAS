# MW-BAAS

### The Infrastructure Behind Every Witness.
**A modular backend platform for the MoonWitness ecosystem.**

*One Foundation. Many Applications. Accountable by Design.*

MW-BAAS is the backend platform intended to power products across the MoonWitness ecosystem. It provides application-facing services—HTTP APIs, identity and access, background work, storage, caching, notifications, integrations, and operations—while relying on **MoonWitness ORM (MW-ORM)** for the reusable relational and modular data foundation.

MW-BAAS helps applications share reliable infrastructure without forcing them to share product logic.

> The Moon Watches. The Record Remembers. The Evidence Remains Inspectable.

This principle comes from the MoonWitness vision: preserve original material, trace claims to their sources, distinguish evidence from interpretation, respect consent and rights, and keep consequential decisions accountable to people.

## Product identity

| Layer | Responsibility |
|---|---|
| **Empire of the Light** | Parent vision and ecosystem context |
| **MoonWitness** | Public-facing knowledge, witness, research, and storytelling ecosystem |
| **MW-BAAS** | Shared backend platform and application-facing services |
| **MW-ORM** | Reusable relational engine and modular data foundation |
| **Applications** | Independent products composed on shared contracts |

Applications remain responsible for their own user experience, policies, and domain decisions.

## Architecture and boundaries

```text
MoonWitness applications
          |
          v
       MW-BAAS
  APIs · Identity · Access
  Jobs · Storage · Cache
  Notifications · Operations
          |
          v
   MW-ORM integration
  Relational core · Entities
  Relations · Queries · Transactions
  Module contracts (as implemented)
          |
          v
 PostgreSQL · MySQL · SQLite
```

**Dependency direction:** MW-BAAS consumes MW-ORM. MW-ORM remains independently usable and must not depend on BAAS product logic.

- **MW-ORM owns** relational primitives, entity/relation metadata, persistence, transactions, and generic module/add-on contracts as implemented.
- **MW-BAAS owns** backend product capabilities, API composition, identity and authorization policy, tenant workflows, integrations, and deployment experience.
- **Applications own** domain rules and product-specific data policies.

There must be one authoritative owner for each schema, migration history, and transaction boundary. Do not run two ORMs as competing owners of the same database state.

## Relationship with MW-ORM

MW-ORM is evolving independently from a SeaORM 2.0 codebase. Its architecture describes a target modular platform composed of an ORM Core, Module Kernel, and optional Add-ons. The module manifest, resolver, lifecycle coordinator, and add-on ecosystem must not be treated as shipped capabilities until implementation and tests land.

MW-BAAS adopts an incremental integration policy:

1. Preserve the currently working database adapter while auditing compatibility.
2. Define a narrow, versioned integration boundary; keep BAAS product behavior out of the ORM core.
3. Integrate MW-ORM through an explicit adapter and compatibility tests.
4. Move model and migration ownership only after an end-to-end proof.
5. Remove duplicate persistence paths only after applications pass migration and compatibility gates.

See [MW-ORM Integration Contract](docs/architecture/MW-ORM-INTEGRATION.md) and [MW-BAAS Brand Narrative](docs/brand/BRAND.md).

## Current implementation status

This repository currently contains a Loco-derived Rust framework codebase. Existing package metadata and database integration still identify Loco and SeaORM. The MW-BAAS identity and MW-ORM integration described here are the intended direction—not proof that package renaming or a production-proven MW-ORM adapter is complete.

| Area | Status |
|---|---|
| Product narrative and responsibility boundaries | Documented |
| MW-ORM integration contract | Documented; implementation pending |
| Existing HTTP/backend foundation | Present; requires MW-BAAS-specific audit |
| Package/crate identity migration | Not started; compatibility-sensitive |
| MW-ORM adapter and compatibility tests | Not yet proven |
| Production readiness | Not claimed |

## Design principles

1. **One data authority** — one canonical owner for schemas, migrations, and transaction semantics.
2. **Explicit contracts** — versioned APIs between ORM, backend, and applications.
3. **Modularity by composition** — optional capabilities do not become mandatory dependencies.
4. **Human accountability** — preserve provenance and auditability where records or research require it.
5. **Privacy and consent** — collect and expose only authorized data.
6. **Secure by default** — deny by default, validate inputs, isolate tenants, and test bypass paths.
7. **Observable operations** — logs, metrics, traces, health checks, actionable diagnostics.
8. **Honest capability claims** — distinguish implementation from proposals and roadmap items.

## Roadmap and acceptance gates

- **Foundation:** repository, license, dependency, and CI audit.
- **Identity:** consistent MW-BAAS naming, documentation, package migration plan.
- **ORM bridge:** pinned MW-ORM revision, adapter, supported feature matrix, compatibility tests.
- **Service modules:** API, identity/access, storage, jobs, events, notifications, tenant policies.
- **Reference application:** demonstrate a real application using MW-BAAS and MW-ORM together.
- **Release readiness:** security review, migration rehearsal, operational docs, reproducible CI.

A milestone is complete only when acceptance tests and evidence are available. A README or architecture diagram alone is not proof of implementation.

## Development

This repository is transitioning from its Loco-derived foundation toward MW-BAAS. Until package migration is complete, consult the existing Rust workspace and development guidance before changing framework internals. Do not rename crates, replace the ORM dependency, or alter migration ownership as an isolated branding change.

## References

- [MoonWitness XV vision](https://github.com/bjo163/moonwitnessxv/blob/main/docs/vision/MASTER_VISION_NARRATIVE_XV.md)
- [MW-ORM repository](https://github.com/BRIDGE-ALLAH/MW-ORM)
- [MW-ORM architecture](https://github.com/BRIDGE-ALLAH/MW-ORM/blob/master/docs/ARCHITECTURE.md)
- [MW-ORM implementation blueprint](https://github.com/BRIDGE-ALLAH/MW-ORM/blob/master/docs/BLUEPRINT.md)
- [Integration contract](docs/architecture/MW-ORM-INTEGRATION.md)
- [Brand narrative](docs/brand/BRAND.md)

---
**MW-BAAS — One Foundation. Many Applications. Accountable by Design.**
