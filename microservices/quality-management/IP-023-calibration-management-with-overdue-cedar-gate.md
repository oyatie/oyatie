---
doc_class: ImplementationPlan
ip_id: IP-023
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
journey_ref: j126-calibrated-equipment-result-entry
sap_submodule: QM-QC Quality Control
tenant_class: paid
billing_components:
  - per_usage
persona: Mateo Ruiz, receiving quality technician
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-023: Calibration management with overdue Cedar gate

## Context

- SAP QM submodule: QM-QC Quality Control.
- Topic: calibration management with overdue Cedar gate.
- Persona: Mateo Ruiz, receiving quality technician.
- Journey: j126 calibrated equipment result entry.
- Journey leg: technician records measurement only if equipment calibration is current.
- SAP precedent: inspection equipment assignment and calibration status.
- Oyatie aggregate: `CalibrationAsset`.
- Boundary: calibration asset state, due schedule, result-entry gate, and overdue recovery.
- ADR-0105 keeps calibration domain separate from equipment adapter.
- ADR-0131 keeps this IP with quality-management.
- ADR-0244 protects tenant equipment boundaries.
- ADR-0263 binds calibration audit events.
- ADR-0297 requires Cedar denial for overdue equipment.
- ADR-0314 keeps marketplace settlement read-only.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires implementation-ready ERP detail.
- Overdue equipment must block regulated measurements.
- Calibration override must be rare, explicit, and audited.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.calibration_asset (
  tenant_id UUID NOT NULL,
  calibration_asset_id TEXT NOT NULL,
  equipment_id TEXT NOT NULL,
  equipment_class TEXT NOT NULL,
  plant_code TEXT NOT NULL,
  calibration_state TEXT NOT NULL,
  last_calibrated_at TIMESTAMPTZ,
  next_due_at TIMESTAMPTZ NOT NULL,
  tolerance_profile_id TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, calibration_asset_id),
  UNIQUE (tenant_id, equipment_id)
);
CREATE TABLE quality_management.calibration_event (
  tenant_id UUID NOT NULL,
  calibration_event_id TEXT NOT NULL,
  calibration_asset_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  performed_by_principal_id TEXT,
  certificate_ref TEXT,
  event_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, calibration_event_id)
);
```

### Rust Types

```rust
pub struct CalibrationAsset {
    pub tenant_id: TenantId,
    pub calibration_asset_id: CalibrationAssetId,
    pub equipment_id: EquipmentId,
    pub equipment_class: EquipmentClass,
    pub plant_code: PlantCode,
    pub calibration_state: CalibrationState,
    pub last_calibrated_at: Option<DateTime<Utc>>,
    pub next_due_at: DateTime<Utc>,
    pub tolerance_profile_id: ToleranceProfileId,
}
pub enum CalibrationState { Current, DueSoon, Overdue, OutOfService, OverrideApproved }
pub enum CalibrationEventType { Registered, Calibrated, DueSoonFlagged, OverdueFlagged, OverrideApproved, Removed }
pub enum CalibrationGateError {
    EquipmentUnknown,
    CalibrationOverdue,
    ToleranceProfileMissing,
    OverridePolicyDenied,
    CrossTenantEquipment,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/calibration-assets`.
- Registers equipment for calibration management.
- `POST /v1/quality-management/calibration-assets/{asset_id}:record-calibration`.
- Records calibration certificate and next due date.
- `POST /v1/quality-management/calibration-assets/{asset_id}:approve-override`.
- Approves temporary override for non-regulated result.
- `GET /v1/quality-management/calibration-assets/{asset_id}/gate-result-entry`.
- Returns permit or deny for result recording.

### gRPC

- Service: `quality_management.calibration.v1.CalibrationService`.
- `rpc RegisterCalibrationAsset(RegisterCalibrationAssetRequest) returns (CalibrationAssetView)`.
- `rpc RecordCalibration(RecordCalibrationRequest) returns (CalibrationAssetView)`.
- `rpc ApproveCalibrationOverride(ApproveCalibrationOverrideRequest) returns (CalibrationAssetView)`.
- `rpc GateResultEntry(GateResultEntryRequest) returns (CalibrationGateDecision)`.

### AsyncAPI

- Channel: `quality-management.calibration.overdue.v1`.
- Channel: `quality-management.calibration.updated.v1`.
- Message: `CalibrationOverdue`.
- Message: `CalibrationUpdated`.
- Payload includes `calibration_asset_id`, `equipment_id`, `calibration_state`, `next_due_at`, `audit_event_class`.
- Consumers: inspection-result, workflow-engine, compliance, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::calibration::gate_result_entry`.
- Principal: `QualityTechnician`.
- Action: `inspection_result_record_with_equipment`.
- Resource: `CalibrationAsset`.
- Context: `inspection_lot_id`, `characteristic_code`, `regulated_material`, `calibration_state`, `override_ref`.
- Policy: `quality_management::calibration::approve_override`.
- Principal: `QualityManager`.
- Action: `calibration_override_approve`.
- Resource: `CalibrationAsset`.
- Context: `override_reason`, `regulated_material`, `duration_minutes`, `authorized_plants`.
- Forbid: regulated measurement with overdue asset.
- Forbid: override duration exceeds pack limit.
- Forbid: asset out of service.
- Forbid: equipment tenant differs from lot tenant.

## Ontology Projection

- Vendor object: SAP QM inspection equipment calibration record.
- Oyatie object: `quality_management.calibration_asset`.
- SAP equipment number -> `equipment_id`.
- SAP plant -> `plant_code`.
- SAP calibration date -> `last_calibrated_at`.
- SAP next calibration date -> `next_due_at`.
- SAP equipment category -> `equipment_class`.
- SAP certificate -> `certificate_ref`.
- SAP status -> `calibration_state`.
- MasterControl calibration certificate -> calibration event.
- ETQ Reliance equipment record -> calibration asset.
- IQS-AQM gauge management -> calibration asset.
- Projection freshness floor: 5 seconds.
- Projection consumer: result recording and compliance.
- Projection rule: overdue state is computed at read and event time.

## Workflow Steps

- Node `asset-register`: equipment is registered.
- Node `tolerance-profile-bind`: tolerance profile attached.
- Decision `profile-missing`: block asset release.
- Node `calibration-record`: certificate and due date recorded.
- Decision `due-soon`: create reminder task.
- Decision `overdue`: emit overdue event.
- Node `result-entry-gate`: technician attempts measurement.
- Decision `asset-current`: permit result entry.
- Decision `asset-overdue-regulated`: deny result entry.
- Decision `asset-overdue-nonregulated`: require override review.
- Node `override-request`: manager reviews reason and duration.
- Node `cedar-override`: evaluate override policy.
- Node `override-approved`: temporary state set.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish calibration state.
- Node `close`: gate decision attached to result evidence.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-CALIBRATION-ASSET_REGISTERED`.
- `EVT-QUALITY_MANAGEMENT-CALIBRATION-CALIBRATED`.
- `EVT-QUALITY_MANAGEMENT-CALIBRATION-OVERDUE`.
- `EVT-QUALITY_MANAGEMENT-CALIBRATION-OVERRIDE_APPROVED`.
- `EVT-QUALITY_MANAGEMENT-CALIBRATION-IP_ACCEPTED`.
- ADR-0263 envelope stores `equipment_id`.
- ADR-0263 envelope stores `calibration_state`.
- ADR-0263 envelope stores `next_due_at`.
- ADR-0263 envelope stores `certificate_ref`.
- ADR-0263 envelope stores `override_ref`.

## SLO Targets

- Gate result entry p50: 20 ms.
- Gate result entry p95: 70 ms.
- Gate result entry p99: 160 ms.
- Calibration update p95: 250 ms.
- Throughput: 800 gate checks per second per cell.
- Availability: 99.97 percent monthly.
- Rationale: every measured result may call this gate, so latency budget is tight.

## Failure Modes and Recovery

- Failure: equipment unknown.
- Recovery: `CAL-EQUIPMENT-UNKNOWN-DENY` blocks result and opens registration task.
- Failure: calibration overdue for regulated measurement.
- Recovery: `CAL-OVERDUE-DENY` blocks result and escalates calibration.
- Failure: tolerance profile missing.
- Recovery: `CAL-PROFILE-BLOCK` prevents asset release.
- Failure: override policy denied.
- Recovery: `CAL-OVERRIDE-DENY` keeps result blocked.
- Failure: calibration certificate hash mismatch.
- Recovery: `CAL-CERT-HASH-REJECT` rejects calibration event.
- Failure: overdue event misses workflow.
- Recovery: `CAL-OVERDUE-REPLAY` replays overdue event.

## Migration Notes

- Source vendor: SAP QM.
- Migrate inspection equipment and calibration dates.
- Source vendor: MasterControl maps calibration certificates.
- Source vendor: ETQ Reliance maps equipment management records.
- Source vendor: IQS-AQM maps gauge records.
- Source vendor: TIPQA maps inspection equipment assignments.
- Historical overdue assets migrate as `Overdue`.
- Missing due dates migrate as blocked registration exceptions.
- Rollback path: default result-entry gate to deny for regulated measurements.
- Calibration override history migrates as evidence-only.

## Cross-microservice Handoffs

- To inspection-result: permit or deny result entry.
- To workflow-engine: calibration due and overdue tasks.
- To compliance: calibration certificate evidence.
- To asset-management: equipment master references.
- To quality-hold: invalid measurements can trigger hold review.
- To audit-evidence: calibration certificate link.
- To ontology: calibration asset projection.
- To procurement: external calibration vendor task.

## Verification

- Unit: overdue regulated equipment denied.
- Unit: override duration limit enforced.
- Unit: out-of-service asset denied.
- Contract: REST gate returns permit and deny reasons.
- Contract: gRPC gate returns Cedar decision id.
- Event: overdue event validates.
- Policy: Cedar denies cross-tenant equipment.
- Projection: MasterControl calibration fixture maps field-for-field.
- SLO: gate p95 under 70 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-CALIBRATION-IP_ACCEPTED`.
