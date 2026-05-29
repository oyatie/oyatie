---
doc_class: ImplementationPlan
ip_id: IP-016
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
journey_ref: j120-statistical-quality-control
sap_submodule: QM-QC Quality Control
tenant_class: paid
billing_components:
  - per_usage
persona: Mei Tan, process quality statistician
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-016: ML-driven anomaly detection on SPC streams

## Context

- SAP QM submodule: QM-QC Quality Control.
- Topic: ML-driven anomaly detection on SPC streams.
- Persona: Mei Tan, process quality statistician.
- Journey: j120 statistical quality control.
- Journey leg: a live process stream deviates before classic control limits fire.
- SAP precedent: inspection characteristic results and quality control charts.
- Oyatie capability: `SpcAnomalyDetector`.
- Boundary: feature extraction, model decision record, human review, and hold suggestion.
- ADR-0105 places model orchestration in usecase and domain evidence in quality-control records.
- ADR-0131 keeps the plan with the quality-management microservice.
- ADR-0244 requires tenant-scoped process streams.
- ADR-0263 binds anomaly audit events.
- ADR-0297 requires Cedar before model-backed actions.
- ADR-0314 keeps marketplace settlement read-only.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires implementation-ready ERP depth.
- Model output can recommend but cannot silently disposition stock.
- Human review is required before automated hold unless pack explicitly permits.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.spc_stream (
  tenant_id UUID NOT NULL,
  spc_stream_id TEXT NOT NULL,
  material_id TEXT NOT NULL,
  plant_code TEXT NOT NULL,
  work_center_id TEXT NOT NULL,
  characteristic_code TEXT NOT NULL,
  sampling_interval_seconds INTEGER NOT NULL,
  state TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, spc_stream_id)
);
CREATE TABLE quality_management.spc_anomaly_decision (
  tenant_id UUID NOT NULL,
  anomaly_decision_id TEXT NOT NULL,
  spc_stream_id TEXT NOT NULL,
  model_version TEXT NOT NULL,
  feature_window_ref TEXT NOT NULL,
  anomaly_score NUMERIC(10,6) NOT NULL,
  threshold NUMERIC(10,6) NOT NULL,
  decision TEXT NOT NULL,
  review_state TEXT NOT NULL,
  PRIMARY KEY (tenant_id, anomaly_decision_id)
);
```

### Rust Types

```rust
pub struct SpcStream {
    pub tenant_id: TenantId,
    pub spc_stream_id: SpcStreamId,
    pub material_id: MaterialId,
    pub plant_code: PlantCode,
    pub work_center_id: WorkCenterId,
    pub characteristic_code: CharacteristicCode,
    pub sampling_interval_seconds: u32,
    pub state: StreamState,
}
pub struct SpcAnomalyDecision {
    pub anomaly_decision_id: DecisionId,
    pub spc_stream_id: SpcStreamId,
    pub model_version: ModelVersion,
    pub feature_window_ref: FeatureWindowRef,
    pub anomaly_score: Decimal,
    pub threshold: Decimal,
    pub decision: AnomalyDecision,
    pub review_state: ReviewState,
}
pub enum AnomalyDecision { Normal, Watch, Investigate, RecommendHold }
pub enum ReviewState { NotRequired, PendingHumanReview, Approved, Rejected }
pub enum SpcAnomalyError {
    FeatureWindowIncomplete,
    ModelVersionNotApproved,
    ScoreThresholdMissing,
    AutomatedHoldPolicyDenied,
    StreamTenantMismatch,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/spc-streams`.
- Creates an SPC stream binding for a characteristic.
- `POST /v1/quality-management/spc-streams/{spc_stream_id}:score-window`.
- Scores a feature window using approved model version.
- `POST /v1/quality-management/spc-anomaly-decisions/{anomaly_decision_id}:review`.
- Human reviewer approves or rejects recommended action.
- `GET /v1/quality-management/spc-streams/{spc_stream_id}/anomaly-decisions`.
- Lists anomaly scores and review state.

### gRPC

- Service: `quality_management.spc_anomaly.v1.SpcAnomalyService`.
- `rpc CreateSpcStream(CreateSpcStreamRequest) returns (SpcStreamReceipt)`.
- `rpc ScoreFeatureWindow(ScoreFeatureWindowRequest) returns (SpcAnomalyDecisionView)`.
- `rpc ReviewAnomalyDecision(ReviewAnomalyDecisionRequest) returns (SpcAnomalyDecisionView)`.
- `rpc StreamAnomalyDecisions(StreamAnomalyDecisionsRequest) returns (stream SpcAnomalyEvent)`.

### AsyncAPI

- Channel: `quality-management.spc-anomaly.scored.v1`.
- Channel: `quality-management.spc-anomaly.reviewed.v1`.
- Message: `SpcAnomalyScored`.
- Message: `SpcAnomalyReviewed`.
- Payload includes `spc_stream_id`, `model_version`, `anomaly_score`, `decision`, `review_state`, `audit_event_class`.
- Consumers: quality-hold, workflow-engine, production-planning, ontology, compliance.

## Cedar Policy Hooks

- Policy: `quality_management::spc_anomaly::score`.
- Principal: `SpcMonitorWorker`.
- Action: `spc_anomaly_score`.
- Resource: `SpcStream`.
- Context: `model_version`, `feature_window_ref`, `stream_tenant_id`, `pack_ids`.
- Policy: `quality_management::spc_anomaly::recommend_hold`.
- Principal: `SpcMonitorWorker`.
- Action: `quality_hold_recommend`.
- Resource: `SpcAnomalyDecision`.
- Context: `anomaly_score`, `threshold`, `human_review_required`, `material_criticality`.
- Forbid: model version not approved.
- Forbid: feature window crosses tenant boundary.
- Forbid: automated hold without human review where pack requires review.
- Forbid: anomaly score below configured threshold.

## Ontology Projection

- Vendor object: SAP QM inspection characteristic result time series.
- Oyatie object: `quality_management.spc_anomaly_decision`.
- SAP characteristic -> `characteristic_code`.
- SAP work center -> `work_center_id`.
- SAP material -> `material_id`.
- SAP plant -> `plant_code`.
- SAP result timestamp series -> `feature_window_ref`.
- SAP control chart signal -> comparison baseline.
- Model version -> `model_version`.
- Feature vector hash -> model input evidence.
- Anomaly score -> `anomaly_score`.
- Hold recommendation -> `decision`.
- Review status -> `review_state`.
- Projection freshness floor: 2 seconds.
- Projection consumer: production-planning and quality-hold.
- Projection rule: model features are evidence pointers, not raw model inputs.

## Workflow Steps

- Node `stream-create`: statistician binds material, plant, work center, and characteristic.
- Node `feature-window-open`: worker starts rolling window.
- Decision `window-incomplete`: no score emitted.
- Node `model-version-load`: approved model version resolved.
- Decision `model-not-approved`: block scoring.
- Node `score-window`: model returns score and explanation vector.
- Decision `score-normal`: persist normal decision.
- Decision `score-watch`: notify statistician.
- Decision `score-investigate`: create workflow investigation task.
- Decision `recommend-hold`: evaluate hold recommendation policy.
- Node `human-review`: reviewer approves or rejects action.
- Decision `review-approved`: request quality hold.
- Decision `review-rejected`: keep stream on watch.
- Node `spc-chart-crosscheck`: compare with classic control chart.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish anomaly projection.
- Node `close`: decision immutable with model version and feature hash.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-SPC_ANOMALY-SCORED`.
- `EVT-QUALITY_MANAGEMENT-SPC_ANOMALY-REVIEWED`.
- `EVT-QUALITY_MANAGEMENT-SPC_ANOMALY-HOLD_RECOMMENDED`.
- `EVT-QUALITY_MANAGEMENT-SPC_ANOMALY-POLICY_DENIED`.
- `EVT-QUALITY_MANAGEMENT-SPC_ANOMALY-IP_ACCEPTED`.
- ADR-0263 envelope stores `model_version`.
- ADR-0263 envelope stores `feature_window_ref`.
- ADR-0263 envelope stores `anomaly_score`.
- ADR-0263 envelope stores `threshold`.
- ADR-0263 envelope stores `review_state`.

## SLO Targets

- Score latency p50: 35 ms.
- Score latency p95: 120 ms.
- Score latency p99: 300 ms.
- Review command p95: 200 ms.
- Throughput: 600 scored windows per second per cell.
- Availability: 99.95 percent monthly.
- Rationale: SPC stream scoring is near-real-time but not stock-posting critical until hold recommendation.

## Failure Modes and Recovery

- Failure: feature window is incomplete.
- Recovery: `SPC-WINDOW-WAIT` records no decision and waits for next sample.
- Failure: model version loses approval.
- Recovery: `SPC-MODEL-FREEZE` stops scoring and routes model governance task.
- Failure: score service times out.
- Recovery: `SPC-SCORE-FALLBACK-CLASSIC` uses control-chart rules only.
- Failure: automated hold is policy denied.
- Recovery: `SPC-HOLD-HUMAN-REVIEW` creates manual review task.
- Failure: stream receives cross-tenant sample.
- Recovery: `SPC-TENANT-QUARANTINE` rejects sample and emits security event.
- Failure: anomaly event not delivered to hold service.
- Recovery: `SPC-ANOMALY-OUTBOX-REPLAY` replays scored event.

## Migration Notes

- Source vendor: SAP QM.
- Migrate inspection characteristic result streams from SAP result history.
- Source vendor: IQS-AQM maps SPC data exports into stream fixtures.
- Source vendor: TIPQA maps process capability exports into stream seeds.
- Source vendor: ETQ Reliance maps statistical alert history into reviewed decisions.
- Source vendor: MasterControl maps approved model governance documents into model version evidence.
- Historical model scores migrate as read-only decisions.
- No migrated model can auto-hold without Cedar release.
- Rollback path: disable ML scoring and keep classic SPC charting active.
- Feature windows are recreated from immutable result history where possible.

## Cross-microservice Handoffs

- From production-planning: work center and production order context.
- From inspection-result: characteristic measurements.
- To quality-hold: reviewed hold recommendation.
- To workflow-engine: investigation and review tasks.
- To compliance: model decision evidence.
- To ontology: anomaly projection.
- To supplier scorecard: repeated anomaly signal.
- To marketplace: read-only supplier process stability signal.

## Verification

- Unit: incomplete window emits no score.
- Unit: unapproved model denied.
- Unit: hold recommendation below threshold denied.
- Contract: REST score returns model version and score.
- Contract: gRPC stream emits reviewed event.
- Event: anomaly scored event validates.
- Policy: Cedar denies cross-tenant feature window.
- Projection: SAP result stream fixture maps field-for-field.
- SLO: score p95 under 120 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-SPC_ANOMALY-IP_ACCEPTED`.
