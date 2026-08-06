---
id: ADR-0333
status: Accepted
planning_impact: true
date: 2026-05-21
owner_team:
  - council-architecture
  - axis-tenancy
  - axis-cloud-iac
  - axis-observability
  - axis-audit-chain
  - axis-api-gateway
deciders:
  - user-directive-2026-05-21
  - council-architecture
  - axis-tenancy
  - axis-cloud-iac
  - axis-observability
  - axis-audit-chain
  - axis-api-gateway
supersedes:
  - microservices/cell/PRD.md
  - microservices/cell/ARCHITECTURE.md
amends:
  - ADR-0248
  - ADR-0138
  - ADR-0131
amended_by:
  - ADR-0351
related:
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0138-intelligence-six-path-deprecation.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0263-audit-event-class-registry.md
related_sources:
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_cell_standalone_network_merges_community_2026_05_21.md
  - microservices/cell/PRD.md
  - microservices/cell/coherence-audit-2026-05-20.md
  - docs/decisions/ADR-0248-amazon-shape-cellular-architecture.md
  - docs/decisions/ADR-0138-intelligence-six-path-deprecation.md
  - docs/decisions/ADR-0131-per-microservice-flat-layout.md
doc_class: Architecture-Decision-Record
purpose: >
  Retire the standalone cell microservice while preserving cellular
  architecture as a mandatory pattern implemented by adjacent owners.
---

# ADR-0333: Cell µservice retired; cellular architecture is a pattern, not a service

## Status

Accepted — 2026-05-21. **Amendment 2026-05-21 per ADR-0351**: the absorption decision below stands for cell **identity**, routing, audit, placement, and telemetry. Two additional bounded contexts — **tenant rebalancing across cells** (workflow) and **cell lifecycle state machine** (Registered → Activated → Promoted → Drained → Decommissioned) — are carved out into dedicated µservices `cell-rebalancer` and `cell-lifecycle` per ADR-0351. See ADR-0351 §D-2 + §D-3.

This ADR executes the 2026-05-21 user directive recorded as Option A:
`cell` retires as a standalone µservice.

Cellular architecture remains mandatory.

The retirement removes a service boundary.

The retirement does not remove cells.

The retirement does not remove shuffle sharding.

The retirement does not remove blast-radius isolation.

The retirement does not remove per-cell SLOs.

The retirement does not remove per-cell audit context.

The retirement does not remove cell-aware routing.

The retirement does not weaken ADR-0248.

This ADR codifies where every retired `cell` responsibility now lives.

It also establishes the redirect contract for old `microservices/cell/`
references.

## Context

The prior `cell` PRD described a rich internal substrate.

It owned tenant-to-cell assignment.

It owned scheduling.

It owned lifecycle management.

It owned host-pool management.

It owned registry reads.

It exposed OpenAPI, AsyncAPI, and protobuf contracts.

It carried SLOs for assignment, migration, decommission, and boundary enforcement.

It carried runbooks for decommission, rebalance, split-brain, and migration.

It carried policy fragments for tenant scope, data residency, CI scope, public read, and auditor scope.

It carried a capacity model for cells and warm pools.

It carried journey implementation plans for home-cell pinning and sovereign placement.

The 2026-05-20 coherence audit found the artifacts substantive.

The issue is not lack of substance.

The issue is ownership shape.

The user revisited the service boundary on 2026-05-21.

The confirmed decision is Option A.

Option A retires `cell` as a µservice.

The reasoning is direct.

AWS does not run a general "cell service" that owns cell topology for every product.

Cellular architecture is a service-team topology pattern.

Each service that needs cells implements the pattern in its natural ownership boundary.

A central service would become a meta-orchestrator.

A meta-orchestrator would duplicate tenancy lifecycle ownership.

A meta-orchestrator would duplicate infrastructure lifecycle ownership.

A meta-orchestrator would duplicate observability health ownership.

A meta-orchestrator would duplicate audit scoping ownership.

A meta-orchestrator would make api-gateway depend on a new hot-path lookup.

The retired service had obvious adjacent homes for every responsibility.

Tenancy already owns tenant lifecycle.

Cloud-iac already owns OpenTofu provisioning.

Observability already owns telemetry and SLO burn.

Audit-chain already owns evidence sealing.

Api-gateway already owns north-south routing.

A Rust crate is the right boundary for the pure algorithm.

Therefore `cell` is retired as a service and preserved as a pattern.

## Decision

D-1. `microservices/cell/` is retired as a standalone µservice.

D-2. `microservices/cell/` keeps only a `RETIRED.md` redirect marker.

D-3. Historical cell service content is not the live authority after this ADR.

D-4. ADR-0248 remains the canonical cellular architecture doctrine.

D-5. ADR-0248 is amended only where it names a central cell µservice as the enforcement substrate.

D-6. Cellular topology is still required for workloads that need blast-radius isolation.

D-7. Shuffle sharding is still required for tenant-to-cell mapping where multi-cell assignment applies.

D-8. Per-cell SLO burn is still required.

D-9. Per-cell audit context is still required.

D-10. Cell-aware routing is still required.

D-11. Cross-cell traffic remains forbidden unless policy explicitly permits it.

D-12. Cross-pack assignment remains forbidden unless a sealed migration exception exists.

D-13. The retired service's responsibilities are absorbed by adjacent owners.

D-14. Tenant-to-cell assignment moves to `microservices/tenancy/ARCHITECTURE.md#cell-assignment`.

D-15. Cell provisioning moves to `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning`.

D-16. Cell lifecycle moves to `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning`.

D-17. Cell registry moves to cloud-iac OpenTofu state.

D-18. OpenTofu state is the authoritative registry for which cells exist.

D-19. Cell health moves to `microservices/observability/ARCHITECTURE.md#cell-health`.

D-20. Blast-radius monitoring moves to observability.

D-21. Per-cell SLO burn moves to observability.

D-22. Cell capacity planning splits between cloud-iac and observability.

D-23. Cloud-iac owns planned capacity and infrastructure ceilings.

D-24. Observability owns live utilization and SLO burn evidence.

D-25. Shuffle-sharding algorithm moves to `crates/oya-shuffle-sharding`.

D-26. The shuffle-sharding crate has no service runtime.

D-27. The shuffle-sharding crate has no storage adapter.

D-28. The shuffle-sharding crate has no network client.

D-29. The shuffle-sharding crate is deterministic for tenant, salt, and candidate set.

D-30. The shuffle-sharding crate rejects duplicate cell ids.

D-31. The shuffle-sharding crate rejects insufficient eligible cells.

D-32. The shuffle-sharding crate filters by pack and region when supplied.

D-33. Per-cell audit scoping moves to `microservices/audit-chain/ARCHITECTURE.md#cell-scoped-audit`.

D-34. Cell-aware tenant routing moves to `microservices/api-gateway/ARCHITECTURE.md#cell-aware-routing`.

D-35. Api-gateway reads cell context from the signed principal.

D-36. Api-gateway does not query a retired cell service on the hot path.

D-37. Audit-chain reads cell context from principal claims and assignment events.

D-38. Audit-chain rejects missing cell context for cell-scoped evidence unless the event is pre-assignment onboarding.

D-39. Tenancy persists `tenant.cell`.

D-40. Tenancy persists `tenant.cell_epoch`.

D-41. Tenancy persists `tenant.assignment_width`.

D-42. Tenancy persists `tenant.assignment_salt`.

D-43. Tenancy emits `tenant.cell-assigned`.

D-44. Cloud-iac emits `cloud-iac.cell-lifecycle-transition`.

D-45. Observability emits `observability.cell-health-verdict`.

D-46. Api-gateway emits `api-gateway.cell-route-admitted`.

D-47. Api-gateway emits `api-gateway.cell-route-denied`.

D-48. Audit-chain seals all events above with `cell_id` and `cell_epoch`.

D-49. A cell can be selected only when cloud-iac marks it ready.

D-50. A cell can receive new tenants only when observability does not mark it isolated, degraded beyond threshold, draining, or unknown.

D-51. A cell can be retired only after tenancy proves zero active assignments.

D-52. A destructive cell retirement pauses if audit-chain cannot seal evidence.

D-53. Cell placement salt changes are migration events.

D-54. Salt changes are not routine load-balancing knobs.

D-55. The old cell contracts are historical after this ADR.

D-56. New contracts bind through tenancy, cloud-iac, observability, audit-chain, and api-gateway.

D-57. New code must not import `microservices/cell/` artifacts.

D-58. New code must not generate `oya-cell-*` service crates.

D-59. Existing `oya-cell-*` crate references are transition debt unless explicitly retained as historical evidence.

D-60. The pure crate is named `oya-shuffle-sharding`, not `oya-cell-*`.

D-61. The old cell capability YAML records are retired.

D-62. The old cell runbooks are retired.

D-63. The old cell dashboards are retired.

D-64. Successor dashboards live under observability.

D-65. Successor lifecycle runbooks live under cloud-iac.

D-66. Successor assignment runbooks live under tenancy.

D-67. Successor audit evidence policy lives under audit-chain.

D-68. Successor routing behavior lives under api-gateway.

D-69. The ADR-0138 strangler discipline applies.

D-70. Because the service retires before launch, the retirement uses the zero-current-consumer variant.

D-71. The zero-current-consumer variant keeps a redirect marker and removes live authority.

D-72. The redirect marker is enough because no production caller is being migrated.

D-73. Cross-reference sweeps must route old paths to absorption targets.

D-74. Historical forensic mentions may survive only when clearly marked historical.

D-75. Machine-readable specs must not list cell as an active µservice after this ADR.

D-76. Counts that included cell as an active µservice must be corrected when touched.

D-77. ADR-0248 enforcement gates must be retargeted from a central service to adjacent owners.

D-78. The target enforcement shape is distributed ownership with common doctrine.

D-79. Doctrine lives centrally.

D-80. Runtime responsibility lives locally.

D-81. The cell pattern remains universal.

D-82. The cell service is terminally retired.

## Absorption Map

| Retired responsibility | Successor owner | Successor authority |
|---|---|---|
| Tenant to cell assignment | tenancy | `microservices/tenancy/ARCHITECTURE.md#cell-assignment` |
| Deterministic selection algorithm | Rust crate | `crates/oya-shuffle-sharding` |
| Candidate cell registry | cloud-iac | OpenTofu state |
| Cell provisioning | cloud-iac | `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning` |
| Cell lifecycle create, drain, retire | cloud-iac | OpenTofu modules and lifecycle events |
| Planned capacity model | cloud-iac | cell module capacity envelope |
| Live capacity and health | observability | `microservices/observability/ARCHITECTURE.md#cell-health` |
| Blast-radius monitoring | observability | per-cell SLO burn and isolation alerts |
| Per-cell audit scoping | audit-chain | `microservices/audit-chain/ARCHITECTURE.md#cell-scoped-audit` |
| Cell-aware tenant routing | api-gateway | `microservices/api-gateway/ARCHITECTURE.md#cell-aware-routing` |

## Successor Contract

C-1. Tenancy is the assignment writer.

C-2. Cloud-iac is the topology writer.

C-3. Observability is the health writer.

C-4. Audit-chain is the evidence writer.

C-5. Api-gateway is the routing reader.

C-6. Identity carries the signed principal context.

C-7. Policy-engine carries Cedar corpus publication but not cell assignment ownership.

C-8. Workload µservices consume cell context from principal, route, and tenancy events.

C-9. No workload µservice calls a retired cell endpoint.

C-10. No workload µservice infers assignment from local hash code.

C-11. The only approved algorithm surface is the pure crate.

C-12. The only approved topology registry is cloud-iac state.

C-13. The only approved live-health verdict source is observability.

C-14. The only approved audit seal source is audit-chain.

C-15. The only approved north-south routing decision point is api-gateway.

## Consequences

Retiring `microservices/cell/` as a standalone service while keeping cellular topology as an architectural pattern under ADR-0248 means workloads needing blast-radius isolation and shuffle-sharded tenant placement are served by the successor contract rather than a central cell service; the data-model, operational, and migration consequences are enumerated in the sections below.

## Data Model Consequences

M-1. `tenant.cell` becomes part of tenant principal context.

M-2. `tenant.cell_epoch` detects stale sessions.

M-3. `tenant.cell_set` represents shuffle-shard width greater than one.

M-4. `tenant.primary_cell` is the default routing cell.

M-5. `tenant.assignment_salt` identifies the assignment epoch input.

M-6. `tenant.assignment_width` records the requested shard width.

M-7. `tenant.assignment_source` records `oya-shuffle-sharding`.

M-8. `cell_id` is infrastructure topology, not tenant identity.

M-9. `cell_id` is internal data.

M-10. `cell_id` may become quasi-identifying for small tenants.

M-11. Audit evidence may carry `cell_id`.

M-12. Metrics should aggregate by cell without raw tenant identifiers.

M-13. Dashboards may show cell health.

M-14. Public APIs must not expose raw topology unless a tenant-facing diagnostic contract explicitly permits it.

M-15. OpenTofu state includes cell lifecycle and capacity envelope.

M-16. OpenTofu state is not queried on the request hot path.

M-17. Api-gateway routing cache includes `tenant_id`, `cell_epoch`, route id, and primary cell.

M-18. Observability labels include `cell_id`, `pack`, `region`, service, and dependency class.

M-19. Audit-chain event classes include assignment, lifecycle, health, routing, isolation, migration, drain, and retirement.

M-20. Cedar context includes `home_cell` or `cell_id` for cell-scoped actions.

## Operational Consequences

O-1. Cell creation is an infrastructure change.

O-2. Cell creation follows cloud-iac review, OpenTofu plan, apply, and evidence.

O-3. Cell drain is an infrastructure lifecycle state.

O-4. Cell drain starts with `accepts_new_tenants=false`.

O-5. Cell drain completes only after tenancy proves zero active primary assignments.

O-6. Cell retirement is destructive and evidence-gated.

O-7. Cell health does not belong in a topology service.

O-8. Cell health belongs in telemetry and SLO evaluation.

O-9. Cell incident response starts with observability blast-radius classification.

O-10. Cell routing failover starts with api-gateway circuit breakers.

O-11. Cell migration starts with tenancy assignment planning.

O-12. Cell migration infrastructure support starts with cloud-iac capacity planning.

O-13. Cell migration evidence starts with audit-chain pre-migration seal.

O-14. Cell migration success ends with tenant principal context refresh.

O-15. Cell policy denial emits evidence instead of silently falling through.

O-16. Cell registry drift is an OpenTofu drift issue.

O-17. Cell health drift is an observability issue.

O-18. Cell assignment drift is a tenancy issue.

O-19. Cell route drift is an api-gateway issue.

O-20. Cell evidence drift is an audit-chain issue.

## ADR-0248 Preservation

P-1. ADR-0248 remains active.

P-2. The cell remains the blast-radius primitive.

P-3. Shuffle sharding remains the tenant distribution primitive.

P-4. Static stability remains a design goal.

P-5. Constant-work control plane sizing remains a design goal.

P-6. K8s-first workload placement remains the default.

P-7. Cloud Hypervisor and Kata remain the isolation direction where stronger sandboxing is needed.

P-8. Per-cell SLOs remain required.

P-9. Per-cell dashboards remain required.

P-10. Per-cell audit context remains required.

P-11. Cell-aware ingress remains required.

P-12. Cross-cell traffic remains policy-gated.

P-13. Cross-pack movement remains migration-gated.

P-14. Compliance pack certification remains compatible with cells.

P-15. Certified cells remain a concept.

P-16. Certified cells are cloud-iac and observability objects, not service-owned records.

P-17. Cell topology remains visible in architecture maps.

P-18. Cell topology is not an active µservice row.

P-19. The pattern is stronger because ownership follows natural boundaries.

P-20. The service boundary is weaker because it centralizes unrelated control loops.

## Rejected Alternatives

R-1. Keep `cell` standalone.

R-2. Rejected because it duplicates tenancy lifecycle ownership.

R-3. Rejected because it duplicates cloud-iac lifecycle ownership.

R-4. Rejected because it duplicates observability health ownership.

R-5. Rejected because it creates an api-gateway hot-path dependency.

R-6. Rejected because AWS-style cellular architecture is a pattern, not a meta-service.

R-7. Convert `cell` into a thin registry service.

R-8. Rejected because OpenTofu state is already the registry.

R-9. Convert `cell` into a scheduler service only.

R-10. Rejected because tenant assignment belongs with tenant lifecycle.

R-11. Convert `cell` into a policy service only.

R-12. Rejected because Cedar and policy-engine already own policy publication and evaluation conventions.

R-13. Keep old docs as live references.

R-14. Rejected because live old docs would preserve an incorrect service boundary.

R-15. Delete all evidence without a redirect.

R-16. Rejected because future agents need a deterministic retirement pointer.

R-17. Put the shuffle-sharding algorithm in tenancy directly.

R-18. Rejected because other services may need the pure algorithm for verification, simulation, or planning.

R-19. Put the shuffle-sharding algorithm in cloud-iac.

R-20. Rejected because cloud-iac owns topology state, not tenant lifecycle decisions.

## Migration Plan

S-1. Author this ADR.

S-2. Create `crates/oya-shuffle-sharding`.

S-3. Add deterministic selection tests.

S-4. Generate cargo documentation for the crate.

S-5. Add tenancy `§cell-assignment`.

S-6. Add cloud-iac `§cell-provisioning`.

S-7. Add observability `§cell-health`.

S-8. Add audit-chain `§cell-scoped-audit`.

S-9. Add api-gateway `§cell-aware-routing`.

S-10. Replace active `microservices/cell/` content with `RETIRED.md`.

S-11. Update docs and specs references to absorption targets.

S-12. Verify no active docs/specs references still route readers to `microservices/cell/`.

S-13. Verify the crate tests pass.

S-14. Verify cargo doc succeeds.

S-15. Report any remaining historical references or validation gaps.

## Verification

V-1. `RUSTC_WRAPPER= cargo test --manifest-path crates/oya-shuffle-sharding/Cargo.toml`.

V-2. The crate test set proves deterministic selection.

V-3. The crate test set proves tenant-sensitive selection changes.

V-4. The crate test set proves pack and region filtering.

V-5. The crate test set proves inactive cells are not selected.

V-6. The crate test set proves duplicate cell ids are rejected.

V-7. The crate test set proves insufficient eligible cells are rejected.

V-8. `RUSTC_WRAPPER= cargo doc --manifest-path crates/oya-shuffle-sharding/Cargo.toml --no-deps`.

V-9. `microservices/tenancy/ARCHITECTURE.md` contains `§cell-assignment`.

V-10. `microservices/cloud-iac/ARCHITECTURE.md` contains `§cell-provisioning`.

V-11. `microservices/observability/ARCHITECTURE.md` contains `§cell-health`.

V-12. `microservices/audit-chain/ARCHITECTURE.md` contains `§cell-scoped-audit`.

V-13. `microservices/api-gateway/ARCHITECTURE.md` contains `§cell-aware-routing`.

V-14. `microservices/cell/RETIRED.md` exists.

V-15. `microservices/cell/` has no live service artifacts after retirement.

V-16. Docs and specs cross-reference sweep points to successor owners.

V-17. ADR-0248 doctrine remains in force.

V-18. No commit is created by this wave.

## Completion Report

The completion report is embedded as an HTML comment so automated readers can
parse the ADR without changing the visible decision text.

<!--
wave: 15L
status: completed-locally
decision: cell microservice retired; cellular architecture preserved as pattern
crate: crates/oya-shuffle-sharding
assignment_owner: microservices/tenancy/ARCHITECTURE.md#cell-assignment
provisioning_owner: microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning
health_owner: microservices/observability/ARCHITECTURE.md#cell-health
audit_owner: microservices/audit-chain/ARCHITECTURE.md#cell-scoped-audit
routing_owner: microservices/api-gateway/ARCHITECTURE.md#cell-aware-routing
retired_marker: microservices/cell/RETIRED.md
tests: RUSTC_WRAPPER= cargo test --manifest-path crates/oya-shuffle-sharding/Cargo.toml
docs: RUSTC_WRAPPER= cargo doc --manifest-path crates/oya-shuffle-sharding/Cargo.toml --no-deps
json_validation: jq empty specs/microservices/manifests-index.json docs/architecture/transition-classification-2026-05-21.json docs/user-journeys/j01-j20-lifesafety-deliverable-report.json
active_crossrefs: rg -n "microservices/cell/" docs specs | rg -v "ADR-0333" returned no active docs/spec references
retired_tree_shape: find microservices/cell -type f returned only RETIRED.md
line_count: 620
commits: none
-->
