---
doc_class: ImplementationPlan
ip_id: IP-003
title: link-type-registry + action-type-registry + function-type-registry (kernel + domain + usecase)
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
  - oya-foundry-fitness-lean-a1
  - oya-foundry-fitness-port-location
  - oya-foundry-fitness-layer-correctness
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-link-type-registry-{kernel,domain,usecase}/
  - microservices/ontology/src/crates/oya-ontology-action-type-registry-{kernel,domain,usecase}/
  - microservices/ontology/src/crates/oya-ontology-function-type-registry-{kernel,domain,usecase}/
doc_status: published
---

# IP-003: link-type-registry + action-type-registry + function-type-registry

## Intent

Author the sibling registries — `oya-ontology-{link-type-registry, action-type-registry, function-type-registry}-{kernel, domain, usecase}` — that register Link Type schemas, Action Type schemas (with Cedar fragments + autonomy tier ceiling), and Function Type schemas (with JSON-IR DSL + result shape + cache TTL + max memory projection).

## Scope

In-scope (per BC):

| BC | kernel | domain | usecase |
|---|---|---|---|
| `link-type-registry` | `LinkTypeStore` port, `LinkTypeSchema`, `LinkCardinality`, `TraversalDirection`, `TenantScopeEnum` | pure cardinality + traversal-direction validation; cross-tenant scope check | orchestrator (read + emit `LinkTypeRegistered`) |
| `action-type-registry` | `ActionTypeStore` port, `ActionTypeSchema`, `EffectSpec`, `IdempotencyKind`, `CedarFragmentRef`, `AutonomyTierCeiling` | effect-validation + idempotency-kind check + Cedar fragment presence | orchestrator (read + emit `ActionTypeRegistered`) |
| `function-type-registry` | `FunctionTypeStore` port, `FunctionTypeSchema`, `FunctionDSL`, `ResultShape`, `CacheTtl`, `MaxMemoryProjection` | DSL validation + max-memory check + cache TTL bounds | orchestrator (read + emit `FunctionTypeRegistered`) |

Out-of-scope: adapter, rest, worker, sdk, app (IP-014 + IP-015).

## Implementation

For each of the three registries (3 BCs × 3 layers = 9 crates):

| Step | Action |
|---|---|
| 1 | Scaffold the kernel crate; add to workspace Cargo.toml |
| 2 | Author port trait + entities with `#[data_class]` |
| 3 | Author sealed-trait + tests |
| 4 | Scaffold the domain crate; author pure logic + property tests |
| 5 | Scaffold the usecase crate; orchestrator (no I/O — reads via port, emits via port) |
| 6 | LEAN lanes green |
| 7 | Register catalog records |

## Verification

- Per-crate `cargo nextest run` exit 0.
- Coverage thresholds met (90% kernel; 95% domain; 90% usecase).
- `oya gate validate port-location --microservice ontology` — exit 0.

## References

- ADR-0006 (Ontology typed-entity layer).
- Bominal ADR-0106 + ADR-0107 (Ontology + agent gateway).
- ADR-0140 (Cedar policy enforcement).
- `microservices/ontology/PRD.md` §"Bounded Contexts".
