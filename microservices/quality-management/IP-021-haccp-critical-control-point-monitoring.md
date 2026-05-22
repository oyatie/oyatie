---
doc_class: ImplementationPlan
ip_id: IP-021
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
journey_ref: j124-healthcare-haccp-quality-monitoring
sap_submodule: QM-QC Quality Control
tenant_class: paid
billing_components:
  - per_usage
persona: Mara Jensen, clinical manufacturing quality lead
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-021: HACCP critical-control-point monitoring

## Context

- SAP QM submodule: QM-QC Quality Control.
- Topic: HACCP critical-control-point monitoring with healthcare-integration handoff.
- Persona: Mara Jensen, clinical manufacturing quality lead.
- Journey: j124 healthcare HACCP quality monitoring.
- Journey leg: critical control point breach triggers quarantine and clinical compliance evidence.
- SAP precedent: QM inspection characteristics, control recipes, and batch release blocks.
- Oyatie capability: `HaccpCriticalControlPoint`.
- Boundary: CCP definition, monitoring result, breach decision, and regulated handoff.
- ADR-0105 places CCP orchestration in usecase and CCP rule in domain.
- ADR-0131 keeps this IP in the quality-management microservice.
- ADR-0244 protects tenant and patient-adjacent data boundaries.
- ADR-0263 binds HACCP audit events.
- ADR-0297 requires Cedar before breach action.
- ADR-0314 keeps marketplace settlement read-only.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires implementation-ready detail.
- HACCP breach must fail closed to hold.
- Healthcare-integration receives minimum necessary breach evidence only.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.haccp_critical_control_point (
  tenant_id UUID NOT NULL,
  ccp_id TEXT NOT NULL,
  process_step_id TEXT NOT NULL,
  material_id TEXT NOT NULL,
  hazard_type TEXT NOT NULL,
  critical_limit_low NUMERIC(20,6),
  critical_limit_high NUMERIC(20,6),
  unit TEXT NOT NULL,
  monitoring_frequency_seconds INTEGER NOT NULL,
  state TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, ccp_id)
);
CREATE TABLE quality_management.haccp_monitoring_result (
  tenant_id UUID NOT NULL,
  monitoring_result_id TEXT NOT NULL,
  ccp_id TEXT NOT NULL,
  measured_value NUMERIC(20,6) NOT NULL,
  unit TEXT NOT NULL,
  breach_state TEXT NOT NULL,
  source_device_id TEXT,
  quality_hold_id TEXT,
  healthcare_evidence_ref TEXT,
  recorded_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, monitoring_result_id)
);
```

### Rust Types

```rust
pub struct HaccpCriticalControlPoint {
    pub tenant_id: TenantId,
    pub ccp_id: CcpId,
    pub process_step_id: ProcessStepId,
    pub material_id: MaterialId,
    pub hazard_type: HazardType,
    pub critical_limit: CriticalLimit,
    pub unit: UnitOfMeasure,
    pub monitoring_frequency_seconds: u32,
    pub state: CcpState,
}
pub struct HaccpMonitoringResult {
    pub monitoring_result_id: MonitoringResultId,
    pub ccp_id: CcpId,
    pub measured_value: Decimal,
    pub unit: UnitOfMeasure,
    pub breach_state: BreachState,
    pub source_device_id: Option<DeviceId>,
    pub quality_hold_id: Option<HoldId>,
    pub healthcare_evidence_ref: Option<EvidenceRef>,
}
pub enum HazardType { Biological, Chemical, Physical, Allergen, Sterility, Temperature }
pub enum BreachState { InControl, Warning, CriticalBreach, Corrected, InvalidReading }
pub enum HaccpError {
    LimitMissing,
    UnitMismatch,
    DeviceUntrusted,
    BreachPolicyDenied,
    HealthcareEvidenceMinimizationFailed,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/haccp/critical-control-points`.
- Creates CCP with critical limits.
- `POST /v1/quality-management/haccp/critical-control-points/{ccp_id}/results`.
- Records monitoring result.
- `POST /v1/quality-management/haccp/monitoring-results/{monitoring_result_id}:evaluate`.
- Evaluates breach and required actions.
- `GET /v1/quality-management/haccp/critical-control-points/{ccp_id}/status`.
- Returns recent results, breach state, and hold link.

### gRPC

- Service: `quality_management.haccp.v1.HaccpService`.
- `rpc CreateCriticalControlPoint(CreateCcpRequest) returns (CcpReceipt)`.
- `rpc RecordMonitoringResult(RecordMonitoringResultRequest) returns (MonitoringResultReceipt)`.
- `rpc EvaluateMonitoringResult(EvaluateMonitoringResultRequest) returns (HaccpEvaluation)`.
- `rpc StreamCcpBreaches(StreamCcpBreachesRequest) returns (stream HaccpBreachEvent)`.

### AsyncAPI

- Channel: `quality-management.haccp.monitoring-recorded.v1`.
- Channel: `quality-management.haccp.breach-detected.v1`.
- Message: `HaccpMonitoringRecorded`.
- Message: `HaccpBreachDetected`.
- Payload includes `ccp_id`, `hazard_type`, `measured_value`, `critical_limit`, `breach_state`, `audit_event_class`.
- Consumers: quality-hold, healthcare-integration, compliance, workflow-engine, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::haccp::record_monitoring_result`.
- Principal: `HaccpDeviceAdapter` or `QualityTechnician`.
- Action: `haccp_monitoring_record`.
- Resource: `HaccpCriticalControlPoint`.
- Context: `source_device_id`, `device_trust_state`, `unit`, `tenant_id`, `pack_ids`.
- Policy: `quality_management::haccp::breach_action`.
- Principal: `HaccpMonitorWorker`.
- Action: `haccp_breach_action`.
- Resource: `HaccpMonitoringResult`.
- Context: `breach_state`, `hazard_type`, `healthcare_minimum_necessary`, `material_criticality`.
- Forbid: untrusted device records critical result.
- Forbid: unit mismatch with CCP limit.
- Forbid: healthcare evidence includes unnecessary patient data.
- Forbid: critical breach without quality hold request.

## Ontology Projection

- Vendor object: SAP QM process control inspection characteristic.
- Oyatie object: `quality_management.haccp_critical_control_point`.
- SAP operation -> `process_step_id`.
- SAP inspection characteristic -> `ccp_id`.
- SAP lower tolerance -> `critical_limit_low`.
- SAP upper tolerance -> `critical_limit_high`.
- SAP unit -> `unit`.
- SAP measured value -> `measured_value`.
- SAP batch -> healthcare evidence context.
- MasterControl HACCP plan -> CCP definition.
- ETQ Reliance food safety record -> monitoring result.
- TIPQA device measurement -> source device result.
- Projection freshness floor: 2 seconds.
- Projection consumer: healthcare-integration and compliance.
- Projection rule: healthcare handoff is minimized by policy.

## Workflow Steps

- Node `ccp-define`: quality lead defines critical control point.
- Node `limit-validate`: critical limit and unit are validated.
- Decision `limit-missing`: reject CCP release.
- Node `device-bind`: source device is registered.
- Decision `device-untrusted`: require manual result.
- Node `monitoring-record`: result is recorded.
- Decision `unit-mismatch`: reject reading.
- Node `breach-evaluate`: compare result to critical limit.
- Decision `in-control`: persist normal state.
- Decision `warning`: create workflow review task.
- Decision `critical-breach`: open quality hold.
- Node `healthcare-minimize`: prepare minimum necessary evidence.
- Node `cedar-breach`: evaluate breach policy.
- Node `healthcare-handoff`: send breach evidence.
- Node `corrective-action-request`: create CAPA task.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish CCP status.
- Node `close`: result immutable with breach state.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-HACCP-CCP_CREATED`.
- `EVT-QUALITY_MANAGEMENT-HACCP-MONITORING_RECORDED`.
- `EVT-QUALITY_MANAGEMENT-HACCP-BREACH_DETECTED`.
- `EVT-QUALITY_MANAGEMENT-HACCP-HEALTHCARE_HANDOFF`.
- `EVT-QUALITY_MANAGEMENT-HACCP-IP_ACCEPTED`.
- ADR-0263 envelope stores `ccp_id`.
- ADR-0263 envelope stores `hazard_type`.
- ADR-0263 envelope stores `breach_state`.
- ADR-0263 envelope stores `quality_hold_id`.
- ADR-0263 envelope stores `healthcare_evidence_ref`.

## SLO Targets

- Result record p50: 45 ms.
- Result record p95: 160 ms.
- Breach evaluation p95: 220 ms.
- Healthcare handoff p95: 1 second.
- Throughput: 400 monitoring results per second per cell.
- Availability: 99.97 percent monthly.
- Rationale: critical breaches must trigger hold before downstream release.

## Failure Modes and Recovery

- Failure: device is untrusted.
- Recovery: `HACCP-DEVICE-DENY` rejects automated result and creates manual sampling task.
- Failure: monitoring unit mismatches CCP unit.
- Recovery: `HACCP-UNIT-REJECT` rejects reading and requests calibrated unit mapping.
- Failure: critical breach hold request fails.
- Recovery: `HACCP-HOLD-REPLAY` retries quality-hold request.
- Failure: healthcare evidence minimization fails.
- Recovery: `HACCP-MINIMUM-NECESSARY-BLOCK` blocks healthcare handoff.
- Failure: monitoring frequency is missed.
- Recovery: `HACCP-MONITORING-MISSED` escalates workflow task and marks CCP warning.
- Failure: duplicate device reading arrives.
- Recovery: `HACCP-READING-IDEMPOTENT` returns existing monitoring result.

## Migration Notes

- Source vendor: SAP QM.
- Migrate process control characteristics as CCP definitions.
- Source vendor: MasterControl maps HACCP plan documents into CCP definitions.
- Source vendor: ETQ Reliance maps food safety records into monitoring results.
- Source vendor: TIPQA maps device measurements into result history.
- Source vendor: IQS-AQM maps HACCP audit questions into clause links.
- Historical breaches migrate as immutable monitoring results.
- Healthcare evidence refs must be minimized during import.
- Rollback path: disable healthcare handoff while retaining holds.
- CCP limits require review before release.

## Cross-microservice Handoffs

- From production-planning: process step and work center context.
- From device integration: monitoring result.
- To quality-hold: critical breach containment.
- To healthcare-integration: minimum necessary regulated evidence.
- To CAPA: corrective action request.
- To workflow-engine: missed monitoring and warning tasks.
- To compliance: HACCP evidence.
- To ontology: CCP status projection.

## Verification

- Unit: unit mismatch rejects monitoring result.
- Unit: untrusted device denied.
- Unit: critical breach requires hold request.
- Contract: REST status returns hold and breach state.
- Contract: gRPC stream emits breach event.
- Event: breach detected event validates.
- Policy: Cedar denies healthcare over-disclosure.
- Projection: MasterControl HACCP fixture maps field-for-field.
- SLO: breach evaluation p95 under 220 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-HACCP-IP_ACCEPTED`.
