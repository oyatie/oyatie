---
doc_class: ImplementationPlan
ip_id: IP-004
title: entity-store (Postgres + Citus + RLS)
microservice: ontology
phase: P01-typed-entity-substrate
status: pending
owner_team: axis-ontology
date: 2026-05-17
depends_on: [IP-002]
acceptance_lanes:
  - cargo-check
  - cargo-clippy
  - cargo-nextest
  - oya-foundry-fitness-ontology-tenancy-isolation
  - oya-foundry-fitness-shardability
  - oya-foundry-fitness-no-raw-sql-cross-tenant
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-entity-store-{kernel,domain,usecase,adapter,adapter-postgres}/
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: entity-store (Postgres + Citus + RLS)

## Intent

Author the canonical Object Type instance persistence layer with Postgres `FORCE ROW LEVEL SECURITY` + Citus `tenant_id` shard key + audit-chain outbox emit per Bominal ADR-0050.

## Scope

In-scope:
- `oya-ontology-entity-store-kernel`: `ObjectTypeStore` port, `ObjectInstance`, `ObjectId`, `WriteReceipt` entities.
- `oya-ontology-entity-store-domain`: pure invariant logic (tier monotonicity at write; pillar consistency).
- `oya-ontology-entity-store-usecase`: orchestrator (Cedar gate → write via port → emit ObjectInstanceMutated).
- `oya-ontology-entity-store-adapter`: protocol-neutral persistence trait impl skeleton.
- `oya-ontology-entity-store-adapter-postgres`: Postgres + Citus implementation with:
  - `FORCE ROW LEVEL SECURITY` on every Object Type table.
  - `app.tenant_id` session variable bound from JWT claim.
  - Citus `multi_shard_modify_mode = strict`.
  - Outbox table emit per ADR-0050.
  - PITR via WAL-archiving.

## Implementation

| Step | Action |
|---|---|
| 1 | Scaffold kernel crate; ports + entities |
| 2 | Scaffold domain crate; pure invariant logic |
| 3 | Scaffold usecase crate; orchestrator with mocked ports |
| 4 | Scaffold adapter crate; protocol-neutral wrapper |
| 5 | Scaffold adapter-postgres crate; sqlx-based impl |
| 6 | Author RLS-enabling migration SQL: `ALTER TABLE ... FORCE ROW LEVEL SECURITY` |
| 7 | Author Citus shard config (multi_shard_modify_mode strict) in migration |
| 8 | Tests: synthetic cross-tenant query attempt → 0 rows; tier-mismatch write → refused |
| 9 | LEAN runtime probe: cross-tenant write attempt fails the lane if it returns non-zero rows |

## Verification

- `cargo nextest run -p oya-ontology-entity-store-adapter-postgres` against test Postgres container — exit 0.
- LEAN runtime probe: synthetic cross-tenant query returns 0 rows; passes.
- `oya gate validate ontology-tenancy-isolation --microservice ontology` — exit 0.
- `oya gate validate shardability --microservice ontology` — exit 0 (Citus shard key declared in kernel).
- `oya gate validate no-raw-sql-cross-tenant --microservice ontology` — exit 0.

## References

- ADR-0006 (Ontology typed-entity layer).
- Bominal ADR-0018 (RLS posture); ADR-0050 (outbox); ADR-0106 (Ontology).
- `microservices/ontology/policy/type-isolation.md` TI-01..TI-13.
- Postgres RLS — `postgresql.org/docs/16/ddl-rowsecurity.html`.
- Citus — `docs.citusdata.com`.
