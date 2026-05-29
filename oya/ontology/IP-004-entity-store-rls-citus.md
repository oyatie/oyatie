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
  - oya-governance-ontology-tenancy-isolation
  - oya-governance-shardability
  - oya-governance-no-raw-sql-cross-tenant
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-entity-store-{kernel,domain,usecase,adapter,adapter-postgres}/
doc_status: published
---


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


## A. Problem
`IP-004: entity-store (Postgres + Citus + RLS)` is not a generic implementation packet; it closes the `004 entity store rls citus` gap for `ontology` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Object Type, Link Type, Action Type, Function Type, tenant-scoped entity store, Cedar fragment, read-path library, Merkle audit chain.

## B. Approach
Typed registry evolution with monotonic data-class/pillar rules, versioned object/link/action/function schemas, and migration receipts for caller-side read libraries. The implementation must keep the µservice boundary intact: contracts remain under `microservices/ontology/contracts/openapi/ontology.yaml` / `microservices/ontology/contracts/proto/ontology.proto`, policy decisions remain in `microservices/ontology/policy/tenant-scope.cedar`, operational proof remains in `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`, and the parity claim is checked against `microservices/ontology/competitor-parity-matrix.md`.

## C. Deliverables
- `microservices/ontology/PRD.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/ARCHITECTURE.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/contracts/openapi/ontology.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/contracts/proto/ontology.proto` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/contracts/asyncapi/ontology-events.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/policy/tenant-scope.cedar` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/slos/read-path-library-freshness.openslo.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/runbooks/type-registry-migration.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/catalog/oya-ontology-object-type-registry-kernel.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/competitor-parity-matrix.md` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/catalog/oya-ontology-object-type-registry-domain.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/capabilities/type-register.yaml` — verify/update as the authoritative artifact for this IP.
- Named code targets declared by this IP and `manifest.json` must be created only when the implementation PR actually adds the crates/types; this scrub does not pretend source files exist.

## D. Implementation Steps
1. Read `microservices/ontology/PRD.md` and `microservices/ontology/ARCHITECTURE.md` to confirm the bounded context, tenant class, and first-ship milestone for `ontology`.
2. Diff the declared contract in `microservices/ontology/contracts/openapi/ontology.yaml` and `microservices/ontology/contracts/proto/ontology.proto` against the IP title so every endpoint/message has a matching domain type or explicit backlog gap.
3. Check `microservices/ontology/policy/tenant-scope.cedar` plus adjacent Cedar/policy files before adding any mutation, share, webhook, agent, AI, or cross-tenant path.
4. Wire observability to `microservices/ontology/slos/read-path-library-freshness.openslo.yaml` and the relevant dashboard/runbook; no acceptance claim counts without a metric or sealed evidence path.
5. Update the catalog/capability record such as `microservices/ontology/catalog/oya-ontology-object-type-registry-kernel.yaml` so the service registry can discover the new boundary.
6. Run the IP-specific test/gate commands listed above; if a source crate is absent, record the absent crate as implementation debt rather than faking a green result.

## E. Acceptance
- Local artifact links resolve for `microservices/ontology/PRD.md`, `microservices/ontology/ARCHITECTURE.md`, `microservices/ontology/contracts/openapi/ontology.yaml`, `microservices/ontology/policy/tenant-scope.cedar`, `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`, and `microservices/ontology/competitor-parity-matrix.md`.
- The implementation exposes no cross-tenant, cross-pack, credential, E2E, or vendor-call path without the policy file cited in this IP.
- At least one targeted unit/contract/gate command verifies the named behavior, and any skipped command is documented with the missing artifact.
- The final PR includes evidence that counterpart parity is improved or explicitly marks the remaining gap.

## F. Evidence
- `microservices/ontology/PRD.md`
- `microservices/ontology/ARCHITECTURE.md`
- `microservices/ontology/contracts/openapi/ontology.yaml`
- `microservices/ontology/contracts/proto/ontology.proto`
- `microservices/ontology/contracts/asyncapi/ontology-events.yaml`
- `microservices/ontology/policy/tenant-scope.cedar`
- `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`
- `microservices/ontology/runbooks/type-registry-migration.md`
- `microservices/ontology/catalog/oya-ontology-object-type-registry-kernel.yaml`
- `microservices/ontology/competitor-parity-matrix.md`
- `microservices/ontology/competitor-parity-matrix.md` — counterpart gap table used for the comparison below.

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model | Palantir Foundry Ontology supplies the product bar for object/link/action/function types; AWS Cedar supplies the policy bar; Neo4j/AWS Neptune/Stardog supply graph traversal and virtual graph pressure; Salesforce object model supplies admin-facing object semantics. This IP closes the relevant gap by binding `004 entity store rls citus` to concrete `ontology` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
