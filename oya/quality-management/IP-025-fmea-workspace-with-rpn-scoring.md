---
doc_class: ImplementationPlan
ip_id: IP-025
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
journey_ref: j128-fmea-risk-reduction
sap_submodule: QM-CA Corrective and Preventive Actions
tenant_class: paid
billing_components:
  - per_usage
persona: Ravi Menon, reliability quality engineer
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-025: FMEA workspace with RPN scoring

## Context

- SAP QM submodule: QM-CA Corrective and Preventive Actions.
- Topic: failure-mode-effects-analysis workspace with RPN scoring.
- Persona: Ravi Menon, reliability quality engineer.
- Journey: j128 FMEA risk reduction.
- Journey leg: engineering and quality rank failure modes, launch actions, and verify risk reduction.
- SAP precedent: quality issue analysis, defect catalogs, and preventive actions.
- Oyatie aggregate: `FmeaWorkspace`.
- Boundary: failure mode, effects, causes, controls, RPN scoring, and action linkage.
- ADR-0105 separates FMEA domain model from action workflow orchestration.
- ADR-0131 keeps the IP inside quality-management.
- ADR-0244 protects tenant product and supplier risk data.
- ADR-0263 binds FMEA audit event classes.
- ADR-0297 requires Cedar before risk acceptance.
- ADR-0314 keeps marketplace settlement read-only.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires implementation-ready ERP detail.
- RPN changes must be tied to action evidence.
- Risk acceptance must be explicit and reversible.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.fmea_workspace (
  tenant_id UUID NOT NULL,
  fmea_workspace_id TEXT NOT NULL,
  workspace_type TEXT NOT NULL,
  material_id TEXT,
  process_step_id TEXT,
  supplier_id TEXT,
  state TEXT NOT NULL,
  owner_principal_id TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, fmea_workspace_id)
);
CREATE TABLE quality_management.fmea_failure_mode (
  tenant_id UUID NOT NULL,
  failure_mode_id TEXT NOT NULL,
  fmea_workspace_id TEXT NOT NULL,
  failure_mode TEXT NOT NULL,
  effect TEXT NOT NULL,
  cause TEXT NOT NULL,
  current_control TEXT NOT NULL,
  severity INTEGER NOT NULL,
  occurrence INTEGER NOT NULL,
  detection INTEGER NOT NULL,
  rpn INTEGER NOT NULL,
  action_ref TEXT,
  PRIMARY KEY (tenant_id, failure_mode_id)
);
```

### Rust Types

```rust
pub struct FmeaWorkspace {
    pub tenant_id: TenantId,
    pub fmea_workspace_id: FmeaWorkspaceId,
    pub workspace_type: FmeaWorkspaceType,
    pub material_id: Option<MaterialId>,
    pub process_step_id: Option<ProcessStepId>,
    pub supplier_id: Option<SupplierId>,
    pub state: FmeaWorkspaceState,
    pub owner_principal_id: PrincipalId,
    pub failure_modes: Vec<FmeaFailureMode>,
}
pub struct FmeaFailureMode {
    pub failure_mode_id: FailureModeId,
    pub failure_mode: String,
    pub effect: String,
    pub cause: String,
    pub current_control: String,
    pub severity: RiskScore,
    pub occurrence: RiskScore,
    pub detection: RiskScore,
    pub rpn: u16,
    pub action_ref: Option<ActionRef>,
}
pub enum FmeaWorkspaceType { Design, Process, Supplier, ControlPlan }
pub enum FmeaWorkspaceState { Draft, Review, Active, RiskAccepted, Closed }
pub enum FmeaError {
    ScoreOutOfRange,
    RpnMismatch,
    HighRiskWithoutAction,
    RiskAcceptancePolicyDenied,
    WorkspaceScopeMismatch,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/fmea-workspaces`.
- Creates FMEA workspace.
- `POST /v1/quality-management/fmea-workspaces/{workspace_id}/failure-modes`.
- Adds or updates failure mode.
- `POST /v1/quality-management/fmea-workspaces/{workspace_id}:recalculate-rpn`.
- Recalculates RPN for all failure modes.
- `POST /v1/quality-management/fmea-workspaces/{workspace_id}:accept-risk`.
- Accepts residual risk under Cedar gate.
- `GET /v1/quality-management/fmea-workspaces/{workspace_id}`.
- Returns workspace, RPNs, action refs, and audit trail.

### gRPC

- Service: `quality_management.fmea.v1.FmeaService`.
- `rpc CreateFmeaWorkspace(CreateFmeaWorkspaceRequest) returns (FmeaWorkspaceView)`.
- `rpc UpsertFailureMode(UpsertFailureModeRequest) returns (FmeaWorkspaceView)`.
- `rpc RecalculateRpn(RecalculateRpnRequest) returns (FmeaWorkspaceView)`.
- `rpc AcceptRisk(AcceptRiskRequest) returns (FmeaWorkspaceView)`.
- `rpc StreamFmeaEvents(StreamFmeaEventsRequest) returns (stream FmeaEvent)`.

### AsyncAPI

- Channel: `quality-management.fmea.failure-mode-upserted.v1`.
- Channel: `quality-management.fmea.rpn-recalculated.v1`.
- Channel: `quality-management.fmea.risk-accepted.v1`.
- Message: `FmeaFailureModeUpserted`.
- Message: `FmeaRpnRecalculated`.
- Payload includes `fmea_workspace_id`, `failure_mode_id`, `severity`, `occurrence`, `detection`, `rpn`, `audit_event_class`.
- Consumers: CAPA, production-planning, supplier scorecard, audit evidence, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::fmea::upsert_failure_mode`.
- Principal: `ReliabilityQualityEngineer`.
- Action: `fmea_failure_mode_upsert`.
- Resource: `FmeaWorkspace`.
- Context: `workspace_type`, `material_id`, `supplier_id`, `authorized_scope`, `pack_ids`.
- Policy: `quality_management::fmea::accept_risk`.
- Principal: `QualityRiskApprover`.
- Action: `fmea_risk_accept`.
- Resource: `FmeaFailureMode`.
- Context: `rpn`, `action_ref`, `residual_risk_reason`, `approval_level`.
- Forbid: score outside 1..10.
- Forbid: RPN does not equal severity * occurrence * detection.
- Forbid: RPN above threshold without action ref.
- Forbid: risk acceptance without approver level.

## Ontology Projection

- Vendor object: SAP QM defect catalog and preventive action analysis.
- Oyatie object: `quality_management.fmea_workspace`.
- SAP defect code -> `failure_mode`.
- SAP effect text -> `effect`.
- SAP cause code -> `cause`.
- SAP inspection characteristic -> `current_control`.
- SAP quality notification task -> `action_ref`.
- Severity ranking -> `severity`.
- Occurrence ranking -> `occurrence`.
- Detection ranking -> `detection`.
- RPN score -> `rpn`.
- TrackWise FMEA/deviation analysis -> failure mode.
- ETQ Reliance risk register -> FMEA workspace.
- MasterControl risk file -> workspace evidence.
- Projection freshness floor: 10 seconds.
- Projection consumer: CAPA and production-planning.

## Workflow Steps

- Node `workspace-create`: engineer defines scope.
- Node `scope-validate`: material, process, or supplier scope checked.
- Decision `scope-mismatch`: reject workspace.
- Node `failure-mode-add`: failure mode, effect, cause, and control recorded.
- Node `score-enter`: severity, occurrence, and detection entered.
- Decision `score-out-of-range`: reject row.
- Node `rpn-calc`: compute severity * occurrence * detection.
- Decision `rpn-mismatch`: reject supplied RPN.
- Decision `high-risk`: require action ref.
- Node `action-link`: link CAPA or preventive action.
- Node `rpn-recalculate`: recalc after action evidence.
- Decision `risk-reduced`: update residual RPN.
- Decision `risk-accept-request`: evaluate acceptance policy.
- Node `cedar-risk-accept`: Cedar approval gate.
- Node `risk-accepted`: state `RiskAccepted`.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish FMEA risk.
- Node `close`: workspace active or closed with residual risk evidence.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-FMEA-WORKSPACE_CREATED`.
- `EVT-QUALITY_MANAGEMENT-FMEA-FAILURE_MODE_UPSERTED`.
- `EVT-QUALITY_MANAGEMENT-FMEA-RPN_RECALCULATED`.
- `EVT-QUALITY_MANAGEMENT-FMEA-RISK_ACCEPTED`.
- `EVT-QUALITY_MANAGEMENT-FMEA-IP_ACCEPTED`.
- ADR-0263 envelope stores `fmea_workspace_id`.
- ADR-0263 envelope stores `failure_mode_id`.
- ADR-0263 envelope stores `rpn`.
- ADR-0263 envelope stores `action_ref`.
- ADR-0263 envelope stores `residual_risk_reason`.

## SLO Targets

- Failure mode upsert p50: 70 ms.
- Failure mode upsert p95: 240 ms.
- RPN recalc p95: 500 ms for 1,000 modes.
- Workspace read p95: 250 ms.
- Throughput: 150 failure-mode updates per second per cell.
- Availability: 99.9 percent monthly.
- Rationale: FMEA is collaborative planning work, but updates should feel immediate.

## Failure Modes and Recovery

- Failure: RPN score mismatch.
- Recovery: `FMEA-RPN-MISMATCH-REJECT` rejects row and returns expected value.
- Failure: high RPN lacks action ref.
- Recovery: `FMEA-HIGH-RISK-ACTION-GATE` blocks risk acceptance.
- Failure: risk approver lacks approval level.
- Recovery: `FMEA-APPROVER-DENY` routes approval to senior quality owner.
- Failure: CAPA action link is missing.
- Recovery: `FMEA-CAPA-LINK-RETRY` searches action refs and keeps risk open.
- Failure: workspace scope conflicts with supplier.
- Recovery: `FMEA-SCOPE-REJECT` rejects workspace mutation.
- Failure: RPN recalculation event fails.
- Recovery: `FMEA-RPN-OUTBOX-REPLAY` replays recalculated event.

## Migration Notes

- Source vendor: SAP QM.
- Migrate defect catalogs and preventive action analyses into FMEA workspaces.
- Source vendor: TrackWise maps risk analysis and deviations.
- Source vendor: ETQ Reliance maps risk register rows.
- Source vendor: MasterControl maps risk file content and approvals.
- Source vendor: IQS-AQM maps process risk worksheets.
- Historical FMEA rows with missing score migrate as Draft.
- Accepted high-risk rows require approver evidence before Active.
- Rollback path: freeze FMEA updates and retain read-only workspace.
- RPN scoring remains deterministic and recalculable.

## Cross-microservice Handoffs

- To CAPA: preventive and corrective action refs.
- To production-planning: process risk controls.
- To inspection-plan: new characteristic or control requirement.
- To supplier scorecard: supplier risk signal.
- To audit-evidence: risk file evidence.
- To workflow-engine: action and approval tasks.
- To ontology: FMEA risk projection.
- To compliance: residual risk acceptance evidence.

## Verification

- Unit: score outside 1..10 rejected.
- Unit: RPN mismatch rejected.
- Unit: high risk requires action ref.
- Contract: REST recalc returns updated RPNs.
- Contract: gRPC stream emits risk accepted event.
- Event: RPN recalculated event validates.
- Policy: Cedar denies acceptance without approval level.
- Projection: TrackWise FMEA fixture maps field-for-field.
- SLO: recalc 1,000 modes p95 under 500 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-FMEA-IP_ACCEPTED`.
