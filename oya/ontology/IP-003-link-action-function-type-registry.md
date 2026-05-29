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
  - oya-governance-lean-a1
  - oya-governance-port-location
  - oya-governance-layer-correctness
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
- ADR-0140 (retired per ADR-0145) (Cedar policy enforcement).
- `microservices/ontology/PRD.md` §"Bounded Contexts".


## A. Problem
`IP-003: link-type-registry + action-type-registry + function-type-registry` is not a generic implementation packet; it closes the `003 link action function type registry` gap for `ontology` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Object Type, Link Type, Action Type, Function Type, tenant-scoped entity store, Cedar fragment, read-path library, Merkle audit chain.

## B. Approach
Cedar-first authorization with action/function schemas, default-deny fragments, and signed denial evidence before any entity mutation or function read. The implementation must keep the µservice boundary intact: contracts remain under `microservices/ontology/contracts/openapi/ontology.yaml` / `microservices/ontology/contracts/proto/ontology.proto`, policy decisions remain in `microservices/ontology/policy/tenant-scope.cedar`, operational proof remains in `microservices/ontology/slos/read-path-library-freshness.openslo.yaml`, and the parity claim is checked against `microservices/ontology/competitor-parity-matrix.md`.

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
- `microservices/ontology/policy/cross-tenant-refusal.cedar` — verify/update as the authoritative artifact for this IP.
- `microservices/ontology/capabilities/cedar-evaluate.yaml` — verify/update as the authoritative artifact for this IP.
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
| Palantir Foundry Ontology / Palantir AIP, AWS Cedar, Neo4j, AWS Neptune, Apache TinkerPop, Stardog, and Salesforce object model | Palantir Foundry Ontology supplies the product bar for object/link/action/function types; AWS Cedar supplies the policy bar; Neo4j/AWS Neptune/Stardog supply graph traversal and virtual graph pressure; Salesforce object model supplies admin-facing object semantics. This IP closes the relevant gap by binding `003 link action function type registry` to concrete `ontology` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
