---
doc_class: ImplementationPlan
ip_id: IP-002
title: object-type-registry kernel + domain
microservice: ontology
phase: P01-typed-entity-substrate
status: pending
owner_team: axis-ontology
date: 2026-05-17
depends_on: [IP-001]
acceptance_lanes:
  - cargo-check
  - cargo-clippy
  - cargo-nextest
  - oya-governance-lean-a1
  - oya-governance-port-location
  - oya-governance-layer-correctness
  - oya-governance-ontology-tier-enforcement
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-object-type-registry-kernel/
  - microservices/ontology/src/crates/oya-ontology-object-type-registry-domain/
doc_status: published
---


# IP-002: object-type-registry kernel + domain

## Intent

Author the `oya-ontology-object-type-registry-{kernel,domain}` crates — the typed schema-registry primitive for Object Types. Per Bominal ADR-0106 + ADR-0006: every Object Type schema (Patient, Order, Payslip, etc.) is registered here with property descriptors, pillar kind, jurisdiction overlay, and tier classification.

## Scope

In-scope:
- `oya-ontology-object-type-registry-kernel` crate:
  - Port traits (sealed): `SchemaRegistry`, `PillarResolver`.
  - Entities: `ObjectTypeSchema`, `PropertyDescriptor`, `PropertyTier`, `PillarKind`, `JurisdictionOverlay`, `SchemaRevision`, `SchemaVersionedRef`.
  - Value objects: `ObjectTypeName` (newtype with BNF validation), `PropertyName`, `DataClass`.
  - Zero I/O; zero business logic; `#[data_class(...)]` annotations on every field.
  - Tests: 1 per public type + 1 per port trait; 90% line / 80% branch coverage.
- `oya-ontology-object-type-registry-domain` crate:
  - Pure schema-evolution logic: `evolve_schema(prev, next) -> SchemaEvolutionDecision`.
  - Pillar/tier inference helpers.
  - Jurisdiction overlay merge.
  - Property-tier monotonicity check (no tier loosening without 2-person rule signal).
  - 95% line / 90% branch coverage; property tests for invariants.

Out-of-scope:
- Adapter (`-adapter`) — IP-002 boundary; adapter delivered as part of IP-014's sweep.
- Usecase (`-usecase`) — see below in same IP if scope grows; else IP-003 covers.
- REST surface, worker, SDK, app — owned by IP-014 + IP-015.

## Implementation

| Step | Action |
|---|---|
| 1 | Scaffold `oya-ontology-object-type-registry-kernel` crate under `microservices/ontology/src/crates/`; add to workspace Cargo.toml |
| 2 | Author kernel ports + entities with `#[data_class]` annotations |
| 3 | Author sealed-trait pattern per Bominal ADR-0101 |
| 4 | Author kernel tests; cargo nextest pass |
| 5 | Scaffold `oya-ontology-object-type-registry-domain` crate |
| 6 | Author pure domain logic (schema-evolution decisions) |
| 7 | Author property tests for monotonic tier invariants |
| 8 | LEAN lanes validate dependency direction, port location, tier enforcement |
| 9 | Register catalog records in `microservices/ontology/catalog/oya-ontology-object-type-registry-{kernel,domain}.yaml` |

## Verification

- `cargo nextest run -p oya-ontology-object-type-registry-kernel` — exit 0.
- `cargo nextest run -p oya-ontology-object-type-registry-domain` — exit 0.
- `cargo llvm-cov --workspace -p oya-ontology-object-type-registry-kernel --fail-under-lines 90` — exit 0.
- `cargo llvm-cov --workspace -p oya-ontology-object-type-registry-domain --fail-under-lines 95` — exit 0.
- `oya gate validate port-location --microservice ontology` — exit 0.
- `oya gate validate ontology-tier-enforcement --microservice ontology` — exit 0.

## References

- ADR-0006 (Ontology typed-entity layer).
- ADR-0056 (BNF v4.1).
- ADR-0105 (13-layer enum).
- Bominal ADR-0106 (Ontology architecture); ADR-0101 (hexagonal sealed traits); ADR-0132 (pillars).
- `microservices/ontology/PRD.md` §"Bounded Contexts" §"object-type-registry".


## A. Problem
`IP-002: object-type-registry kernel + domain` is not a generic implementation packet; it closes the `002 object type registry kernel domain` gap for `ontology` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Object Type, Link Type, Action Type, Function Type, tenant-scoped entity store, Cedar fragment, read-path library, Merkle audit chain.

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
| Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model | Palantir Foundry Ontology supplies the product bar for object/link/action/function types; AWS Cedar supplies the policy bar; Neo4j/AWS Neptune/Stardog supply graph traversal and virtual graph pressure; Salesforce object model supplies admin-facing object semantics. This IP closes the relevant gap by binding `002 object type registry kernel domain` to concrete `ontology` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
