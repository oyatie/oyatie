---
doc_class: ImplementationPlan
ip_id: IP-011
microservice: quality-management
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0253
  - ADR-0263
  - ADR-0294
  - ADR-0297
  - ADR-0314
  - ADR-0315
  - ADR-0329
  - ADR-0330
  - ADR-0331
  - ADR-0320
journey_ref: j118-supplier-defect-containment
sap_submodule: QM-CA Corrective and Preventive Actions
tenant_class: paid
billing_components:
  - per_usage
persona: Elena Petrova, containment coordinator
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-011: Usecase layer for containment release and action effectiveness

## Context

- SAP QM submodule: QM-CA Corrective and Preventive Actions.
- Topic: action effectiveness verification before quality-hold release.
- Persona: Elena Petrova, containment coordinator.
- Journey: j118 supplier defect containment.
- Journey leg: held inventory can only release after corrective action evidence is effective.
- SAP precedent: quality notification task completion, usage decision, and follow-up action.
- Oyatie usecase: `ReleaseQualityHoldWithEffectivenessGate`.
- Boundary: orchestrates hold state, CAPA state, warehouse movement, and finance failure-cost handoff.
- ADR-0105 places orchestration in usecase.
- ADR-0131 keeps this plan in quality-management.
- ADR-0244 protects tenant inventory references.
- ADR-0263 binds release and effectiveness audit events.
- ADR-0297 requires Cedar before release.
- ADR-0314 keeps marketplace settlement outside hold release.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires implementation-ready ERP detail.
- Release is not a human checkbox.
- Release is a policy-backed state transition with verified effectiveness evidence.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.hold_release_gate (
  tenant_id UUID NOT NULL,
  gate_id TEXT NOT NULL,
  hold_id TEXT NOT NULL,
  capa_case_id TEXT,
  required_effectiveness_state TEXT NOT NULL,
  observed_effectiveness_state TEXT NOT NULL,
  release_decision TEXT NOT NULL,
  warehouse_movement_ref TEXT,
  finance_cost_ref TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, gate_id),
  UNIQUE (tenant_id, hold_id)
);
CREATE TABLE quality_management.hold_release_attempt (
  tenant_id UUID NOT NULL,
  release_attempt_id TEXT NOT NULL,
  hold_id TEXT NOT NULL,
  requested_by_principal_id TEXT NOT NULL,
  requested_disposition TEXT NOT NULL,
  attempt_state TEXT NOT NULL,
  denial_reason TEXT,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, release_attempt_id)
);
```

### Rust Types

```rust
pub struct HoldReleaseGate {
    pub tenant_id: TenantId,
    pub gate_id: GateId,
    pub hold_id: HoldId,
    pub capa_case_id: Option<CapaCaseId>,
    pub required_effectiveness_state: EffectivenessState,
    pub observed_effectiveness_state: EffectivenessState,
    pub release_decision: HoldReleaseDecision,
    pub warehouse_movement_ref: Option<WarehouseMovementRef>,
    pub finance_cost_ref: Option<FinanceCostRef>,
}
pub enum EffectivenessState { NotRequired, Required, EvidenceSubmitted, VerifiedEffective, Failed }
pub enum HoldReleaseDecision { Release, PartialRelease, Scrap, ReturnToSupplier, Deny }
pub enum HoldReleaseError {
    EffectivenessNotVerified,
    WarehouseMovementFailed,
    FinanceCostRequired,
    HoldAlreadyReleased,
    ReleasePolicyDenied,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/quality-holds/{hold_id}:request-release`.
- Creates release attempt and gate evaluation.
- `POST /v1/quality-management/quality-holds/{hold_id}:verify-effectiveness`.
- Binds CAPA effectiveness evidence.
- `POST /v1/quality-management/quality-holds/{hold_id}:post-release`.
- Posts warehouse movement after gate passes.
- `GET /v1/quality-management/quality-holds/{hold_id}/release-gate`.
- Returns release decision, CAPA status, and movement refs.

### gRPC

- Service: `quality_management.hold_release.v1.HoldReleaseService`.
- `rpc RequestRelease(RequestHoldReleaseRequest) returns (HoldReleaseGateView)`.
- `rpc VerifyEffectiveness(VerifyHoldEffectivenessRequest) returns (HoldReleaseGateView)`.
- `rpc PostRelease(PostHoldReleaseRequest) returns (HoldReleaseReceipt)`.
- `rpc StreamReleaseAttempts(StreamReleaseAttemptsRequest) returns (stream HoldReleaseEvent)`.

### AsyncAPI

- Channel: `quality-management.quality-hold.release-requested.v1`.
- Channel: `quality-management.quality-hold.effectiveness-verified.v1`.
- Channel: `quality-management.quality-hold.release-posted.v1`.
- Message: `QualityHoldReleaseRequested`.
- Message: `QualityHoldEffectivenessVerified`.
- Payload includes `hold_id`, `capa_case_id`, `release_decision`, `warehouse_movement_ref`, `audit_event_class`.
- Consumers: warehouse, CAPA, finance, ontology, workflow-engine.

## Cedar Policy Hooks

- Policy: `quality_management::quality_hold::request_release`.
- Principal: `ContainmentCoordinator`.
- Action: `quality_hold_request_release`.
- Resource: `QualityHold`.
- Context: `disposition_type`, `quantity`, `capa_case_id`, `authorized_plants`.
- Policy: `quality_management::quality_hold::post_release`.
- Principal: `QualityManager`.
- Action: `quality_hold_post_release`.
- Resource: `HoldReleaseGate`.
- Context: `observed_effectiveness_state`, `warehouse_movement_ref`, `finance_cost_ref`, `pack_ids`.
- Forbid: observed effectiveness below required state.
- Forbid: scrap disposition lacks finance cost reference.
- Forbid: warehouse movement plant outside principal scope.
- Forbid: hold state is already released.

## Ontology Projection

- Vendor object: SAP QM quality notification action and stock posting.
- Oyatie object: `quality_management.hold_release_gate`.
- SAP task completion -> `observed_effectiveness_state`.
- SAP usage decision stock posting -> `warehouse_movement_ref`.
- SAP scrap or return movement -> `requested_disposition`.
- SAP notification action id -> `capa_case_id`.
- SAP follow-up action status -> release gate state.
- ETQ Reliance CAPA effectiveness check -> `VerifiedEffective`.
- TrackWise action effectiveness -> `observed_effectiveness_state`.
- MasterControl deviation closure -> CAPA evidence ref.
- TIPQA MRB disposition -> `release_decision`.
- Projection freshness floor: 5 seconds.
- Projection consumer: warehouse release and finance cost ledger.
- Projection rule: finance owns cost, quality owns release gate.

## Workflow Steps

- Node `release-requested`: coordinator asks to release or dispose.
- Node `hold-load`: current hold and disposition are loaded.
- Decision `hold-already-closed`: return existing terminal state.
- Node `capa-state-load`: CAPA effectiveness state loaded.
- Decision `effectiveness-required`: require verified effective.
- Decision `effectiveness-failed`: deny release and keep hold open.
- Decision `scrap-disposition`: require finance cost reference.
- Node `cedar-release-request`: evaluate request policy.
- Node `gate-record`: persist gate outcome.
- Node `warehouse-movement-request`: ask warehouse for release, scrap, or return movement.
- Decision `warehouse-failed`: keep gate pending and retry.
- Node `finance-cost-capture`: send failure cost if scrap or return.
- Decision `finance-failed`: keep release posted but mark cost pending.
- Node `cedar-post-release`: evaluate final post policy.
- Node `release-posted`: hold transitions to terminal state.
- Node `source-notify`: notify notification, CAPA, and lot.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish gate projection.
- Node `close`: release gate becomes immutable.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-QUALITY_HOLD-RELEASE_REQUESTED`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_HOLD-EFFECTIVENESS_VERIFIED`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_HOLD-RELEASED`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_HOLD-RELEASE_DENIED`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_HOLD-IP_ACCEPTED`.
- ADR-0263 envelope stores `gate_id`.
- ADR-0263 envelope stores `capa_case_id`.
- ADR-0263 envelope stores `release_decision`.
- ADR-0263 envelope stores `warehouse_movement_ref`.
- ADR-0263 envelope stores `finance_cost_ref`.

## SLO Targets

- Release request p50: 70 ms.
- Release request p95: 250 ms.
- Release request p99: 700 ms.
- Warehouse movement ACK p95: 900 ms.
- Throughput: 80 release gates per second per cell.
- Availability: 99.95 percent monthly.
- Rationale: release is less frequent than hold open but operationally blocks inventory.

## Failure Modes and Recovery

- Failure: CAPA effectiveness not verified.
- Recovery: `HOLD-RELEASE-CAPA-DENY` denies release and routes CAPA verification task.
- Failure: warehouse movement fails.
- Recovery: `HOLD-RELEASE-WAREHOUSE-RETRY` keeps gate pending and retries idempotently.
- Failure: scrap disposition lacks finance cost reference.
- Recovery: `HOLD-RELEASE-COST-GATE` holds release until cost event is accepted.
- Failure: release request races with already released hold.
- Recovery: `HOLD-RELEASE-IDEMPOTENT-TERMINAL` returns existing terminal gate.
- Failure: CAPA service unavailable.
- Recovery: `HOLD-RELEASE-CAPA-CACHE-DENY` denies release unless cached state is verified effective.
- Failure: ontology projection misses release event.
- Recovery: `HOLD-RELEASE-PROJECTION-REPLAY` replays from release gate events.

## Migration Notes

- Source vendor: SAP QM.
- Migrate usage decision stock postings into release gates.
- Migrate notification follow-up action closure into effectiveness evidence.
- Source vendor: ETQ Reliance maps CAPA verification into observed effectiveness.
- Source vendor: TrackWise maps action effectiveness into gate state.
- Source vendor: TIPQA maps MRB release and scrap decisions into release attempts.
- Open historical holds migrate as pending gates.
- Terminal vendor holds migrate as immutable released or scrapped gates.
- Rollback path: disable post-release command and leave hold state unchanged.
- Finance cost references may be backfilled after migration.

## Cross-microservice Handoffs

- From quality-hold: open hold and disposition.
- From CAPA: effectiveness state.
- To warehouse: release, scrap, or return movement.
- To finance: failure cost capture.
- To quality-notification: containment resolution.
- To inspection-lot: usage decision completion.
- To ontology: release gate projection.
- To workflow-engine: gate review task.

## Verification

- Unit: release denied until effectiveness verified.
- Unit: scrap requires finance cost ref.
- Unit: released hold is idempotent terminal.
- Contract: REST release gate returns CAPA state.
- Contract: gRPC stream emits release requested and posted.
- Event: release posted event validates.
- Policy: Cedar denies plant scope mismatch.
- Projection: TrackWise effectiveness fixture maps field-for-field.
- SLO: release request p95 under 250 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-QUALITY_HOLD-IP_ACCEPTED`.
