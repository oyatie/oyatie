# IP-014 Performance Management marketplace DealSet settlement

Service: performance-management
ChangeSet scope: microservices/performance-management/IP-014-marketplace-dealset-settlement.md
Wave: 15-IP-substance conversion, 2026-05-21
Counterpart anchors: Lattice, 15Five, Workday Performance, Culture Amp, Glint
Binding doctrine: ADR-0324 anti-template-stamping; ADR-0328 D-20 Big-8 P0 elevation

## A. Problem
This IP closes the marketplace DealSet settlement gap for `performance-management`. The previous 55-line shell repeated the same objective/prerequisite/test/rollback labels without proving how `marketplace-dealset-settlement` works for reviews, goals, calibration, engagement pulses, succession evidence, and compensation handoff cannot collapse into generic HR records.
The service is in the HR/Payroll Big-8 P0 lane. A generic platform answer is not enough: `goal-cycle`, `review-cycle`, and `recognition` each carry tenant, principal, cell, data-class, pack, and audit consequences that differ from neighboring services.
The gap matters against Lattice and 15Five because those products make this capability feel native inside their workflow; Oyatie must match that usability while adding stronger Cedar, audit-chain, cell, and DealSet evidence.
The success condition is an implementation plan a cold engineer can trace from this IP to concrete files such as `microservices/performance-management/cost-budget.md` without inventing a hidden service boundary.

## B. Approach
Implement `marketplace-dealset-settlement` as a service-local slice, not as a shared platform facility. The technical mechanism is billable provider/template/plugin movement bound to `DealSet settlement metadata` and checked before user-visible promotion.
Use `goal-cycle` as the first fixture path, then prove the same envelope across `review-cycle` and `recognition` so the design is not a one-object shortcut.
Every command or event carries `tenant_id`, `principal_id`, `audience_type=HR_BUSINESS_PARTNER`, `home_cell`, `jurisdiction_code`, `data_class`, `traceparent`, `idempotency_key` for mutations, and an audit event class.
The domain layer stays pure in `microservices/performance-management/src/domain/mod.rs`; usecase orchestration lives in `microservices/performance-management/src/usecase/mod.rs`; transport or provider details stay behind adapter/config files.
Cedar fragments under `microservices/performance-management/policy/` and `microservices/performance-management/policies/` are the guard surface. A deny is a signed refusal with operator evidence, not an absent row or swallowed exception.
The approach explicitly covers settlement id, usage attribution, reversal bundle; those are the failure modes that a stamped IP did not name.

## C. Deliverables
- D01: `microservices/performance-management/PRD.md` — modify or bind for `marketplace-dealset-settlement` evidence and contract traceability.
- D02: `microservices/performance-management/ARCHITECTURE.md` — modify or bind for `marketplace-dealset-settlement` evidence and contract traceability.
- D03: `microservices/performance-management/manifest.json` — modify or bind for `marketplace-dealset-settlement` evidence and contract traceability.
- D04: `microservices/performance-management/competitor-parity-matrix.md` — modify or bind for `marketplace-dealset-settlement` evidence and contract traceability.
- D05: `microservices/performance-management/feature-parity-matrix-2026-05-20.md` — modify or bind for `marketplace-dealset-settlement` evidence and contract traceability.
- D06: `microservices/performance-management/cost-budget.md` — modify or bind for `marketplace-dealset-settlement` evidence and contract traceability.
- D07: `microservices/performance-management/contracts/openapi-v1.yaml` — modify or bind for `marketplace-dealset-settlement` evidence and contract traceability.
- D08: `microservices/performance-management/contracts/asyncapi-v1.yaml` — modify or bind for `marketplace-dealset-settlement` evidence and contract traceability.
- D09: `microservices/performance-management/contracts/performance-management-v1.proto` — modify or bind for `marketplace-dealset-settlement` evidence and contract traceability.
- D10: `microservices/performance-management/src/domain/mod.rs` — modify or bind for `marketplace-dealset-settlement` evidence and contract traceability.
- D11: `microservices/performance-management/src/usecase/mod.rs` — modify or bind for `marketplace-dealset-settlement` evidence and contract traceability.
- D12: `microservices/performance-management/src/adapter/mod.rs` — modify or bind for `marketplace-dealset-settlement` evidence and contract traceability.
- D13: `microservices/performance-management/src/config.rs` — modify or bind for `marketplace-dealset-settlement` evidence and contract traceability.
- D14: `microservices/performance-management/policies/local-review-cycle-scope.cedar` — modify or bind for `marketplace-dealset-settlement` evidence and contract traceability.
- D15: `microservices/performance-management/policies/local-calibration-lock-control.cedar` — modify or bind for `marketplace-dealset-settlement` evidence and contract traceability.
- D16: `microservices/performance-management/policies/local-engagement-pulse-anonymity.cedar` — modify or bind for `marketplace-dealset-settlement` evidence and contract traceability.
- D17: `microservices/performance-management/policy/review-calibration-authorization.cedar` — modify or bind for `marketplace-dealset-settlement` evidence and contract traceability.
- D18: `microservices/performance-management/capabilities/goal-cycle-open.yaml` — modify or bind for `marketplace-dealset-settlement` evidence and contract traceability.
- D26: `microservices/performance-management/REMEDIATION-NOTES-2026-05-21.md` or tier-scrub equivalent records this Wave 15 conversion outcome.

## D. Implementation steps
1. Read `microservices/performance-management/manifest.json` and confirm the Big-8 family, audience, compliance packs, cell eligibility, and benchmark list before editing code.
2. Add or update the `marketplace-dealset-settlement` contract shape in `cost-budget.md` and keep request/event/proto field names aligned with ADR-0105 layer naming.
3. Add domain invariants in `microservices/performance-management/src/domain/mod.rs` for `goal_record` and `review_evidence` so tenant scope and immutable evidence are checked before adapters run.
4. Add usecase orchestration in `microservices/performance-management/src/usecase/mod.rs` with idempotency, trace context, and audit-chain emission before external side effects.
5. Bind adapter behavior through `DealSet settlement metadata` and `microservices/performance-management/src/adapter/mod.rs`, using typed errors from `microservices/performance-management/src/error.rs` instead of stringly provider failures.
6. Update the Cedar policy files listed above so `goal-cycle-open` and `review-evidence-seal` deny cross-tenant, stale-pack, and missing-purpose requests.
7. Update catalog rows under `microservices/performance-management/catalog/` so the layer registry names the owning crate, layer, capability id, and contract version.
8. Update dashboards/SLOs or the service operating bar to expose success, refusal, replay, latency, and audit completeness for `marketplace-dealset-settlement`.
9. Run focused contract/policy checks, then a service-level `cargo check` or equivalent if the crate graph is present.
10. Attach verification evidence to the remediation notes with the exact commands and changed IP list.

## E. Acceptance
- A reviewer can trace `marketplace-dealset-settlement` from this IP to at least one real contract, one real policy file, one real source file, and one capability/catalog artifact.
- The contract tests include accepted, duplicate-idempotency, Cedar-denied, stale-pack, wrong-tenant, and replay/backfill cases for `goal-cycle`.
- The policy tests prove default deny for missing `tenant_id`, missing `principal_id`, wrong `audience_type`, stale `home_cell`, and data-class mismatch.
- The observability evidence includes metric, trace, structured log, audit event id, policy decision id, and low-cardinality labels for `marketplace-dealset-settlement`.
- The counterpart row below explains what Oyatie displaces from Lattice / 15Five / Workday Performance without claiming suite ownership or hiding substrate dependencies.
- Rollback is documented as contract version retreat, Cedar fragment pointer rollback, adapter feature flag off, and replay of idempotent commands from the backfill ledger.

## F. Evidence
- `microservices/performance-management/PRD.md`
- `microservices/performance-management/ARCHITECTURE.md`
- `microservices/performance-management/manifest.json`
- `microservices/performance-management/competitor-parity-matrix.md`
- `microservices/performance-management/feature-parity-matrix-2026-05-20.md`
- `microservices/performance-management/cost-budget.md`
- `microservices/performance-management/contracts/openapi-v1.yaml`
- `microservices/performance-management/contracts/asyncapi-v1.yaml`
- `microservices/performance-management/contracts/performance-management-v1.proto`
- `microservices/performance-management/src/domain/mod.rs`
- `microservices/performance-management/src/usecase/mod.rs`
- `microservices/performance-management/src/adapter/mod.rs`
- `microservices/performance-management/src/config.rs`
- `microservices/performance-management/policies/local-review-cycle-scope.cedar`
- `microservices/performance-management/policies/local-calibration-lock-control.cedar`
- `microservices/performance-management/policies/local-engagement-pulse-anonymity.cedar`
- `microservices/performance-management/policy/review-calibration-authorization.cedar`
- `microservices/performance-management/capabilities/goal-cycle-open.yaml`
- `microservices/performance-management/capabilities/review-evidence-seal.yaml`
- `microservices/performance-management/capabilities/calibration-run.yaml`
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`

## G. Counterparts
| Counterpart | Gap closed by this IP | Oyatie substance requirement |
|---|---|---|
| Lattice | Provides a native `marketplace DealSet settlement` experience inside its product boundary. | Oyatie keeps the same operator fluency while binding `marketplace-dealset-settlement` to tenant scope, Cedar deny evidence, audit-chain records, and flat `performance-management` ownership. |
| 15Five | Competes on workflow speed and admin ergonomics. | Oyatie must prove the workflow through `goal-cycle-open` / `review-evidence-seal` with explicit policy and replay paths. |
| Workday Performance | Sets buyer expectation for enterprise reporting and integration. | Oyatie closes the gap through contracts, catalog rows, SLOs, and remediation evidence instead of a stamped parity claim. |
| Culture Amp | Pressures adjacent analytics or collaboration expectations. | Oyatie accepts the benchmark only where it maps to `recognition` and does not weaken residency, audit, or data-class controls. |

## H. Non-goals and deletion check
- Do not move `marketplace-dealset-settlement` into a sibling service such as `people-records` or `compensation` unless a later ADR changes ownership.
- Do not add Terraform, Cedar, or SDK claims for files that are not present; missing IaC remains a follow-up rather than fake HCL in this IP.
- No duplicative IP was deleted in this pass because this slice has a distinct contract/policy/evidence concern from its siblings.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/performance-management/contracts/asyncapi-v1.yaml`, `microservices/performance-management/contracts/hr-handoff-compensation.asyncapi.yaml`, `microservices/performance-management/contracts/hr-handoff-learning-management.asyncapi.yaml`, `microservices/performance-management/contracts/hr-handoff-people-records.asyncapi.yaml`, `microservices/performance-management/contracts/hr-handoff-recruiting.asyncapi.yaml`, `microservices/performance-management/contracts/hr-handoff-time-tracking.asyncapi.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`, `asyncapi`, `.proto`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/performance-management/IP-014-marketplace-dealset-settlement.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/performance-management/IP-014-marketplace-dealset-settlement.md`, `microservices/performance-management/manifest.json`, `microservices/performance-management/ARCHITECTURE.md`, `microservices/performance-management/PRD.md`, `microservices/performance-management/multi-region.md`, `microservices/performance-management/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/performance-management/IP-014-marketplace-dealset-settlement.md` matched [`cost`, `attribution`, `emission`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/performance-management/IP-014-marketplace-dealset-settlement.md`, `microservices/performance-management/manifest.json`, `microservices/performance-management/capacity-model.md`, `microservices/performance-management/compliance.md`, `microservices/performance-management/ARCHITECTURE.md`].

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`plugin`].
- surface_evidence_paths: [`microservices/performance-management/IP-014-marketplace-dealset-settlement.md`, `microservices/performance-management/manifest.json`, `microservices/performance-management/contracts/performance-management-v1.proto`, `microservices/performance-management/PRD.md`].
