---
doc_class: ImplementationPlan
ip_id: IP-017
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
journey_ref: j121-mes-to-erp-quality-feedback
sap_submodule: QM-QC Quality Control
tenant_class: paid
billing_components:
  - per_usage
persona: Daniel Weber, manufacturing quality lead
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-017: Closed-loop quality from MES to ERP feedback

## Context

- SAP QM submodule: QM-QC Quality Control.
- Topic: closed-loop quality from MES to ERP feedback.
- Persona: Daniel Weber, manufacturing quality lead.
- Journey: j121 MES to ERP quality feedback.
- Journey leg: shop-floor nonconformance adjusts ERP quality status and future planning.
- SAP precedent: QM result recording, production order confirmation, and quality notification feedback.
- Oyatie capability: `MesQualityFeedbackLoop`.
- Boundary: feedback event normalization, ERP status update command, and downstream handoffs.
- ADR-0105 separates adapter ingestion, usecase orchestration, and domain state.
- ADR-0131 keeps this plan in the microservice.
- ADR-0244 requires tenant-scoped MES lines.
- ADR-0263 binds feedback loop audit events.
- ADR-0297 requires Cedar before MES events mutate ERP quality state.
- ADR-0314 keeps marketplace settlement read-only.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires implementation-ready ERP detail.
- MES feedback must not bypass inspection result rules.
- ERP feedback must be idempotent because shop-floor events can replay.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.mes_quality_feedback (
  tenant_id UUID NOT NULL,
  feedback_id TEXT NOT NULL,
  mes_system_id TEXT NOT NULL,
  production_order_id TEXT NOT NULL,
  work_center_id TEXT NOT NULL,
  material_id TEXT NOT NULL,
  batch_id TEXT,
  feedback_type TEXT NOT NULL,
  severity TEXT NOT NULL,
  feedback_state TEXT NOT NULL,
  source_event_id TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, feedback_id),
  UNIQUE (tenant_id, mes_system_id, source_event_id)
);
CREATE TABLE quality_management.erp_quality_feedback_action (
  tenant_id UUID NOT NULL,
  action_id TEXT NOT NULL,
  feedback_id TEXT NOT NULL,
  action_type TEXT NOT NULL,
  target_microservice TEXT NOT NULL,
  action_state TEXT NOT NULL,
  target_ref TEXT,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, action_id)
);
```

### Rust Types

```rust
pub struct MesQualityFeedback {
    pub tenant_id: TenantId,
    pub feedback_id: FeedbackId,
    pub mes_system_id: SourceSystemId,
    pub production_order_id: ProductionOrderId,
    pub work_center_id: WorkCenterId,
    pub material_id: MaterialId,
    pub batch_id: Option<BatchId>,
    pub feedback_type: FeedbackType,
    pub severity: DefectSeverity,
    pub state: FeedbackState,
    pub source_event_id: SourceEventId,
}
pub enum FeedbackType { Nonconformance, MeasurementDrift, ReworkComplete, ScrapReported, LineStop }
pub enum FeedbackState { Received, Normalized, Routed, Applied, Rejected, Replayed }
pub enum FeedbackActionType { CreateNotification, OpenHold, UpdateProductionOrder, AdjustInspectionPlan, RecordCost }
pub enum MesFeedbackError {
    DuplicateSourceEvent,
    MesTenantUnbound,
    FeedbackPolicyDenied,
    ProductionOrderUnknown,
    ActionDispatchFailed,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/mes-quality-feedback`.
- Receives MES feedback event through adapter boundary.
- `POST /v1/quality-management/mes-quality-feedback/{feedback_id}:route`.
- Computes ERP actions.
- `POST /v1/quality-management/mes-quality-feedback/{feedback_id}:apply-actions`.
- Dispatches quality notification, hold, planning, and finance actions.
- `GET /v1/quality-management/mes-quality-feedback/{feedback_id}`.
- Returns feedback state and action outcomes.

### gRPC

- Service: `quality_management.mes_feedback.v1.MesQualityFeedbackService`.
- `rpc ReceiveFeedback(ReceiveFeedbackRequest) returns (FeedbackReceipt)`.
- `rpc RouteFeedback(RouteFeedbackRequest) returns (FeedbackRoutePlan)`.
- `rpc ApplyFeedbackActions(ApplyFeedbackActionsRequest) returns (FeedbackActionSummary)`.
- `rpc StreamFeedback(StreamFeedbackRequest) returns (stream MesFeedbackEvent)`.

### AsyncAPI

- Channel: `quality-management.mes-feedback.received.v1`.
- Channel: `quality-management.mes-feedback.applied.v1`.
- Message: `MesQualityFeedbackReceived`.
- Message: `MesQualityFeedbackApplied`.
- Payload includes `mes_system_id`, `production_order_id`, `work_center_id`, `feedback_type`, `severity`, `audit_event_class`.
- Consumers: production-planning, quality-notification, quality-hold, finance, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::mes_feedback::receive`.
- Principal: `MesAdapter`.
- Action: `mes_feedback_receive`.
- Resource: `MesQualityFeedback`.
- Context: `mes_system_id`, `source_event_id`, `line_binding`, `tenant_id`, `pack_ids`.
- Policy: `quality_management::mes_feedback::apply_actions`.
- Principal: `FeedbackLoopWorker`.
- Action: `mes_feedback_apply_actions`.
- Resource: `FeedbackRoutePlan`.
- Context: `production_order_id`, `severity`, `action_types`, `authorized_plants`.
- Forbid: MES system not bound to tenant.
- Forbid: source event duplicates with different payload hash.
- Forbid: line stop event without production-planning notification.
- Forbid: scrap action without finance cost capture.

## Ontology Projection

- Vendor object: SAP ME or SAP Digital Manufacturing shop-floor quality event.
- Oyatie object: `quality_management.mes_quality_feedback`.
- MES event id -> `source_event_id`.
- MES line -> `work_center_id`.
- MES order -> `production_order_id`.
- MES material -> `material_id`.
- MES batch -> `batch_id`.
- MES defect type -> `feedback_type`.
- MES severity -> `severity`.
- SAP QM notification created from feedback -> `target_ref`.
- SAP production order confirmation -> production-planning action.
- TIPQA shop-floor defect -> feedback event.
- TrackWise production deviation -> quality notification action.
- Projection freshness floor: 3 seconds.
- Projection consumer: production-planning and ontology.
- Projection rule: MES raw event stays in adapter storage.

## Workflow Steps

- Node `mes-event-received`: adapter receives signed feedback.
- Node `source-idempotency`: duplicate source event check.
- Decision `duplicate-same-hash`: return existing feedback.
- Decision `duplicate-different-hash`: reject as conflict.
- Node `mes-tenant-bind`: verify MES line belongs to tenant.
- Decision `tenant-unbound`: reject feedback.
- Node `feedback-normalize`: map MES code to quality feedback type.
- Decision `line-stop`: mandatory production-planning action.
- Decision `scrap-reported`: mandatory finance cost action.
- Decision `nonconformance`: create quality notification.
- Decision `measurement-drift`: update SPC and inspection plan signal.
- Node `cedar-apply`: evaluate action application policy.
- Node `action-dispatch`: dispatch actions idempotently.
- Node `source-ack`: ACK MES source.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish feedback projection.
- Node `close`: feedback state is terminal.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-MES_FEEDBACK-RECEIVED`.
- `EVT-QUALITY_MANAGEMENT-MES_FEEDBACK-ROUTED`.
- `EVT-QUALITY_MANAGEMENT-MES_FEEDBACK-APPLIED`.
- `EVT-QUALITY_MANAGEMENT-MES_FEEDBACK-POLICY_DENIED`.
- `EVT-QUALITY_MANAGEMENT-MES_FEEDBACK-IP_ACCEPTED`.
- ADR-0263 envelope stores `mes_system_id`.
- ADR-0263 envelope stores `source_event_id`.
- ADR-0263 envelope stores `feedback_type`.
- ADR-0263 envelope stores `production_order_id`.
- ADR-0263 envelope stores `action_types`.

## SLO Targets

- Feedback receive p50: 55 ms.
- Feedback receive p95: 200 ms.
- Feedback receive p99: 550 ms.
- Action dispatch p95: 1 second.
- Throughput: 250 feedback events per second per cell.
- Availability: 99.95 percent monthly.
- Rationale: MES line-stop feedback must reach ERP quickly enough to prevent bad downstream movement.

## Failure Modes and Recovery

- Failure: MES system is not tenant-bound.
- Recovery: `MES-FEEDBACK-TENANT-DENY` rejects and alerts integration owner.
- Failure: duplicate source event has different payload hash.
- Recovery: `MES-FEEDBACK-CONFLICT` rejects and requires source-system reconciliation.
- Failure: production order is unknown.
- Recovery: `MES-FEEDBACK-ORDER-HOLD` creates manual routing task.
- Failure: action dispatch to production-planning fails.
- Recovery: `MES-FEEDBACK-ACTION-REPLAY` retries route plan action.
- Failure: scrap feedback lacks cost classification.
- Recovery: `MES-FEEDBACK-COST-GATE` blocks action application.
- Failure: source ACK fails.
- Recovery: `MES-FEEDBACK-ACK-RETRY` retries with same source event id.

## Migration Notes

- Source vendor: SAP QM with SAP Digital Manufacturing.
- Source vendor: TIPQA maps shop-floor inspections into feedback events.
- Source vendor: TrackWise maps production deviations into nonconformance feedback.
- Source vendor: ETQ Reliance maps line quality alerts into feedback route plans.
- Source vendor: MasterControl maps approved rework completion records into feedback actions.
- Historical feedback migrates as applied or rejected snapshots.
- MES line binding must be created before migration.
- No migrated MES event can mutate production-planning without Cedar policy.
- Rollback path: disable action dispatch and retain feedback receipt.
- Source event ids remain immutable.

## Cross-microservice Handoffs

- From MES adapter: signed quality feedback.
- To production-planning: line stop and order adjustment.
- To quality-notification: nonconformance notification.
- To quality-hold: containment request.
- To finance: scrap or failure cost capture.
- To inspection-plan: drift signal for dynamic modification review.
- To ontology: feedback projection.
- To workflow-engine: unresolved action task.

## Verification

- Unit: duplicate source event returns existing feedback.
- Unit: unbound MES tenant denied.
- Unit: line stop requires production-planning action.
- Contract: REST apply returns action summary.
- Contract: gRPC stream emits applied event.
- Event: feedback applied event validates.
- Policy: Cedar denies scrap without finance action.
- Projection: SAP Digital Manufacturing fixture maps field-for-field.
- SLO: receive p95 under 200 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-MES_FEEDBACK-IP_ACCEPTED`.
