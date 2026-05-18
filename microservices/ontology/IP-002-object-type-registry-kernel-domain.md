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
  - oya-foundry-fitness-lean-a1
  - oya-foundry-fitness-port-location
  - oya-foundry-fitness-layer-correctness
  - oya-foundry-fitness-ontology-tier-enforcement
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-object-type-registry-kernel/
  - microservices/ontology/src/crates/oya-ontology-object-type-registry-domain/
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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
