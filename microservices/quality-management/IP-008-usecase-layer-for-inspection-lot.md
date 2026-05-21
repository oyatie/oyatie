---
doc_class: ImplementationPlan
ip_id: IP-008
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
journey_ref: j101-multi-tier-supply-chain-formation
sap_submodule: QM-IM Inspection Management
tenant_class: paid
billing_components:
  - per_usage
persona: Mateo Ruiz, receiving quality technician
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-008: Usecase layer for sample recording and usage decision

## Context

- SAP QM submodule: QM-IM Inspection Management.
- Topic: sample result recording and usage decision.
- Persona: Mateo Ruiz, receiving quality technician.
- Journey: j101 multi-tier supply-chain formation.
- Journey leg: received component is accepted, rejected, or sent to containment.
- SAP precedent: results recording, defects recording, sample valuation, and usage decision.
- Oyatie usecase: `RecordInspectionResults`.
- Boundary: orchestrates lot state, characteristic results, quality hold, and notification.
- ADR-0105 places orchestration in usecase.
- ADR-0131 keeps the plan with the microservice.
- ADR-0244 protects tenant-scoped result evidence.
- ADR-0263 governs result and usage-decision audit events.
- ADR-0297 requires policy before usage decision.
- ADR-0314 keeps supplier settlement outside this service.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires implementation-ready ERP detail.
- Usage decision must be impossible without complete required characteristics.
- Defect-triggered rejects must create containment handoffs.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.inspection_result (
  tenant_id UUID NOT NULL,
  result_id TEXT NOT NULL,
  inspection_lot_id TEXT NOT NULL,
  sample_no INTEGER NOT NULL,
  characteristic_code TEXT NOT NULL,
  measured_value NUMERIC(20,6),
  measured_text TEXT,
  unit TEXT,
  valuation TEXT NOT NULL,
  recorded_by_principal_id TEXT NOT NULL,
  recorded_hlc TEXT NOT NULL,
  evidence_ref TEXT,
  PRIMARY KEY (tenant_id, result_id)
);
CREATE TABLE quality_management.usage_decision (
  tenant_id UUID NOT NULL,
  usage_decision_id TEXT NOT NULL,
  inspection_lot_id TEXT NOT NULL,
  decision_code TEXT NOT NULL,
  stock_posting_code TEXT NOT NULL,
  defect_notification_id TEXT,
  quality_hold_id TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  decided_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, usage_decision_id),
  UNIQUE (tenant_id, inspection_lot_id)
);
```

### Rust Types

```rust
pub struct InspectionResult {
    pub tenant_id: TenantId,
    pub result_id: ResultId,
    pub inspection_lot_id: InspectionLotId,
    pub sample_no: u16,
    pub characteristic_code: CharacteristicCode,
    pub measured_value: Option<Decimal>,
    pub measured_text: Option<String>,
    pub unit: Option<UnitOfMeasure>,
    pub valuation: ResultValuation,
    pub evidence_ref: Option<EvidenceRef>,
}
pub struct UsageDecision {
    pub usage_decision_id: UsageDecisionId,
    pub inspection_lot_id: InspectionLotId,
    pub decision_code: UsageDecisionCode,
    pub stock_posting_code: StockPostingCode,
    pub defect_notification_id: Option<NotificationId>,
    pub quality_hold_id: Option<HoldId>,
}
pub enum ResultValuation { Pass, Fail, NotTested, Informational }
pub enum UsageDecisionCode { Accept, Reject, Rework, ReturnToSupplier, SkipAccepted }
pub enum ResultRecordingError {
    RequiredCharacteristicMissing,
    OutOfSpecWithoutDefect,
    DuplicateUsageDecision,
    StockPostingPolicyDenied,
    SampleNotAssigned,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/inspection-lots/{inspection_lot_id}/results`.
- Records sample characteristic results.
- `POST /v1/quality-management/inspection-lots/{inspection_lot_id}:complete-results`.
- Validates required result completeness.
- `POST /v1/quality-management/inspection-lots/{inspection_lot_id}:usage-decision`.
- Posts accept, reject, rework, return, or skip accepted decision.
- `GET /v1/quality-management/inspection-lots/{inspection_lot_id}/results`.
- Returns results and valuation state.

### gRPC

- Service: `quality_management.results.v1.InspectionResultService`.
- `rpc RecordResult(RecordResultRequest) returns (ResultReceipt)`.
- `rpc CompleteResults(CompleteResultsRequest) returns (LotResultSummary)`.
- `rpc MakeUsageDecision(MakeUsageDecisionRequest) returns (UsageDecisionReceipt)`.
- `rpc StreamResultEvents(StreamResultEventsRequest) returns (stream InspectionResultEvent)`.

### AsyncAPI

- Channel: `quality-management.inspection-result.recorded.v1`.
- Channel: `quality-management.usage-decision.made.v1`.
- Message: `InspectionResultRecorded`.
- Message: `UsageDecisionMade`.
- Payload carries `inspection_lot_id`, `sample_no`, `characteristic_code`, `valuation`, `decision_code`, `audit_event_class`.
- Consumers: warehouse, quality-hold, quality-notification, certificate-of-analysis, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::inspection_result::record`.
- Principal: `QualityTechnician`.
- Action: `inspection_result_record`.
- Resource: `InspectionLotSample`.
- Context: `assigned_to_principal_id`, `equipment_calibration_state`, `plant_code`, `pack_ids`.
- Policy: `quality_management::usage_decision::make`.
- Principal: `QualityManager`.
- Action: `usage_decision_make`.
- Resource: `InspectionLot`.
- Context: `result_completeness`, `failed_characteristic_count`, `stock_posting_code`, `authorized_plants`.
- Forbid: usage decision before required characteristics complete.
- Forbid: accept decision with uncontained failed characteristic.
- Forbid: stock posting code outside source plant.
- Permit: skip accepted only when IP-007 decision allows skip.

## Ontology Projection

- Vendor object: SAP QM results recording.
- Oyatie object: `quality_management.inspection_result`.
- `QAMR-PRUEFLOS` -> `inspection_lot_id`.
- `QAMR-VORGLFNR` -> `sample_no`.
- `QAMR-MERKNR` -> `characteristic_code`.
- `QAMR-MESSWERT` -> `measured_value`.
- `QAMR-MASSEINHSW` -> `unit`.
- `QAMR-BEWERTUNG` -> `valuation`.
- `QAVE-VCODE` -> `usage_decision.decision_code`.
- `QAVE-BUCHART` -> `stock_posting_code`.
- Defect record -> `defect_notification_id`.
- Blocked stock posting -> `quality_hold_id`.
- Technician id -> `recorded_by_principal_id`.
- Projection freshness floor: 2 seconds.
- Projection rule: accepted usage decision unlocks CoA and warehouse movement.
- Projection consumers: warehouse and certificate release.

## Workflow Steps

- Node `sample-assigned`: technician receives sample.
- Node `equipment-check`: calibration state is checked.
- Decision `equipment-overdue`: block result entry and open calibration task.
- Node `result-record`: characteristic result is recorded.
- Decision `value-out-of-spec`: require defect code.
- Node `valuation-calc`: pass/fail valuation computed.
- Node `result-audit`: result recorded event emitted.
- Node `completeness-check`: required plan characteristics verified.
- Decision `missing-required`: return list of missing characteristic codes.
- Node `usage-decision-start`: manager selects decision code.
- Decision `any-failed`: branch to reject, rework, or return.
- Decision `all-pass`: accept or skip accepted.
- Node `cedar-usage-decision`: evaluate decision policy.
- Node `hold-create`: request IP-005 for failed inventory.
- Node `notification-create`: request IP-004 for defect.
- Node `warehouse-post`: request stock posting.
- Node `coa-enable`: notify IP-003 if accepted.
- Node `audit-seal`: emit ADR-0263 class.
- Node `close`: lot cannot receive more results except correction workflow.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-INSPECTION_RESULT-RECORDED`.
- `EVT-QUALITY_MANAGEMENT-USAGE_DECISION-MADE`.
- `EVT-QUALITY_MANAGEMENT-USAGE_DECISION-POLICY_DENIED`.
- `EVT-QUALITY_MANAGEMENT-INSPECTION_LOT-CHANGED`.
- `EVT-QUALITY_MANAGEMENT-INSPECTION_LOT-IP_ACCEPTED`.
- ADR-0263 envelope stores `sample_no`.
- ADR-0263 envelope stores `characteristic_code`.
- ADR-0263 envelope stores `valuation`.
- ADR-0263 envelope stores `decision_code`.
- ADR-0263 envelope stores `stock_posting_code`.

## SLO Targets

- Result record latency p50: 50 ms.
- Result record latency p95: 180 ms.
- Result record latency p99: 400 ms.
- Usage decision p95: 450 ms excluding warehouse ACK.
- Throughput: 500 result records per second per cell.
- Availability: 99.95 percent monthly.
- Rationale: technicians enter many results; usage decisions block inventory movement.

## Failure Modes and Recovery

- Failure: equipment calibration is overdue.
- Recovery: `RESULT-CALIBRATION-GATE` blocks result and routes calibration task.
- Failure: out-of-spec value lacks defect code.
- Recovery: `RESULT-DEFECT-REQUIRED` rejects record until defect code is supplied.
- Failure: manager attempts accept with failed characteristic.
- Recovery: `UD-ACCEPT-DENIED` forces reject, rework, or containment branch.
- Failure: duplicate usage decision command arrives.
- Recovery: `UD-IDEMPOTENT-RETURN` returns existing decision.
- Failure: warehouse stock posting fails after decision.
- Recovery: `UD-WAREHOUSE-REPLAY` retries posting from outbox.
- Failure: CoA enable event fails.
- Recovery: `UD-COA-REPLAY` replays accepted decision event.

## Migration Notes

- Source vendor: SAP QM.
- Import result rows from `QAMR`.
- Import usage decision from `QAVE`.
- Preserve SAP valuation code as source valuation.
- Source vendor: TIPQA maps sample results into `inspection_result`.
- Source vendor: IQS-AQM maps test data into result rows.
- Source vendor: MasterControl maps batch disposition into usage decisions.
- Historical accepted decisions migrate as immutable snapshots.
- Failed lots with no defect become blocked migration exceptions.
- Rollback path: disable usage decision posting and keep result recording read-only.

## Cross-microservice Handoffs

- From inspection-lot: sample roster.
- To calibration: equipment gate when calibration overdue.
- To quality-notification: defect notification on failed characteristic.
- To quality-hold: containment on reject or rework.
- To warehouse: stock posting command.
- To certificate-of-analysis: accepted lot enables certificate draft.
- To ontology: result and usage decision projection.
- To compliance: regulated result evidence.

## Verification

- Unit: missing required characteristic blocks completion.
- Unit: out-of-spec result requires defect code.
- Unit: accept with failed characteristic denied.
- Contract: REST usage decision returns hold and notification ids.
- Contract: gRPC result stream preserves sample no.
- Event: usage decision event validates.
- Policy: Cedar denies overdue calibration result entry.
- Projection: SAP `QAMR` and `QAVE` fixtures map field-for-field.
- SLO: result record p95 under 180 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-INSPECTION_LOT-IP_ACCEPTED`.
