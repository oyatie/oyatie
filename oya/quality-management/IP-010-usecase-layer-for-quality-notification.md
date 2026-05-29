---
doc_class: ImplementationPlan
ip_id: IP-010
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
sap_submodule: QM-QN Quality Notifications
tenant_class: paid
billing_components:
  - per_usage
persona: Aisha Morgan, supplier quality engineer
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-010: Usecase layer for priority escalation and complaint mirror

## Context

- SAP QM submodule: QM-QN Quality Notifications.
- Topic: priority escalation and customer-facing complaint mirror.
- Persona: Aisha Morgan, supplier quality engineer.
- Journey: j118 supplier defect containment.
- Journey leg: defect priority changes as severity, customer impact, and containment age change.
- SAP precedent: notification priority, task deadlines, partner functions, and escalation.
- Oyatie usecase: `EscalateQualityNotification`.
- Boundary: escalation orchestration, timers, customer mirror sync, and supplier route.
- ADR-0105 places orchestration in usecase.
- ADR-0131 keeps the implementation plan local.
- ADR-0244 protects tenant and customer complaint boundaries.
- ADR-0263 binds escalation events.
- ADR-0297 requires Cedar for customer mirror visibility.
- ADR-0314 prevents marketplace settlement side effects.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires implementation-ready detail.
- Escalation must not depend on manual inbox vigilance.
- Customer mirror must trail internal truth without leaking internal analysis.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.notification_escalation (
  tenant_id UUID NOT NULL,
  escalation_id TEXT NOT NULL,
  notification_id TEXT NOT NULL,
  prior_priority TEXT NOT NULL,
  next_priority TEXT NOT NULL,
  escalation_reason TEXT NOT NULL,
  due_at TIMESTAMPTZ NOT NULL,
  assigned_queue TEXT NOT NULL,
  customer_mirror_required BOOLEAN NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, escalation_id)
);
CREATE TABLE quality_management.customer_complaint_mirror (
  tenant_id UUID NOT NULL,
  mirror_id TEXT NOT NULL,
  notification_id TEXT NOT NULL,
  customer_id TEXT NOT NULL,
  mirror_state TEXT NOT NULL,
  visible_summary TEXT NOT NULL,
  last_synced_hlc TEXT NOT NULL,
  redaction_profile_id TEXT NOT NULL,
  PRIMARY KEY (tenant_id, mirror_id),
  UNIQUE (tenant_id, notification_id, customer_id)
);
```

### Rust Types

```rust
pub struct NotificationEscalation {
    pub tenant_id: TenantId,
    pub escalation_id: EscalationId,
    pub notification_id: NotificationId,
    pub prior_priority: NotificationPriority,
    pub next_priority: NotificationPriority,
    pub escalation_reason: EscalationReason,
    pub due_at: DateTime<Utc>,
    pub assigned_queue: RoutingQueue,
    pub customer_mirror_required: bool,
}
pub struct CustomerComplaintMirror {
    pub tenant_id: TenantId,
    pub mirror_id: MirrorId,
    pub notification_id: NotificationId,
    pub customer_id: CustomerId,
    pub state: MirrorState,
    pub visible_summary: String,
    pub redaction_profile_id: RedactionProfileId,
}
pub enum EscalationReason { SeverityRaised, DeadlineBreached, CustomerImpact, SafetyRisk, SupplierNonresponse }
pub enum MirrorState { Draft, Published, Updated, Revoked }
pub enum EscalationError {
    PriorityWouldDecrease,
    MirrorRedactionFailed,
    SupplierRouteMissing,
    DeadlinePolicyDenied,
    NotificationClosed,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/quality-notifications/{notification_id}:escalate`.
- Raises priority and schedules deadlines.
- `POST /v1/quality-management/quality-notifications/{notification_id}:sync-customer-mirror`.
- Publishes or updates complaint mirror.
- `POST /v1/quality-management/quality-notifications/{notification_id}:supplier-nonresponse`.
- Escalates after supplier deadline breach.
- `GET /v1/quality-management/customer-complaint-mirrors/{mirror_id}`.
- Returns customer-safe complaint state.

### gRPC

- Service: `quality_management.notification_escalation.v1.NotificationEscalationService`.
- `rpc EscalateNotification(EscalateNotificationRequest) returns (EscalationReceipt)`.
- `rpc SyncCustomerMirror(SyncCustomerMirrorRequest) returns (ComplaintMirrorReceipt)`.
- `rpc RecordSupplierNonresponse(SupplierNonresponseRequest) returns (EscalationReceipt)`.
- `rpc StreamEscalations(StreamEscalationsRequest) returns (stream NotificationEscalationEvent)`.

### AsyncAPI

- Channel: `quality-management.notification.escalated.v1`.
- Channel: `quality-management.customer-complaint-mirror.synced.v1`.
- Message: `QualityNotificationEscalated`.
- Message: `CustomerComplaintMirrorSynced`.
- Payload carries `notification_id`, `prior_priority`, `next_priority`, `escalation_reason`, `mirror_state`, `audit_event_class`.
- Consumers: workflow-engine, supplier-portal, customer-portal, quality-hold, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::notification_escalation::escalate`.
- Principal: `SupplierQualityEngineer` or `EscalationTimerWorker`.
- Action: `quality_notification_escalate`.
- Resource: `QualityNotification`.
- Context: `severity`, `age_minutes`, `customer_impact`, `supplier_response_state`, `pack_ids`.
- Policy: `quality_management::customer_complaint_mirror::sync`.
- Principal: `ComplaintMirrorWorker`.
- Action: `customer_complaint_mirror_sync`.
- Resource: `CustomerComplaintMirror`.
- Context: `redaction_profile_id`, `customer_id`, `visibility_fields`, `internal_fields_removed`.
- Forbid: next priority lower than prior priority.
- Forbid: notification closed.
- Forbid: mirror contains internal supplier score.
- Forbid: supplier route without trust binding.

## Ontology Projection

- Vendor object: SAP QM notification priority and partner task.
- Oyatie object: `quality_management.notification_escalation`.
- `QMEL-PRIOK` -> priority.
- SAP deadline profile -> `due_at`.
- SAP task partner -> `assigned_queue`.
- SAP customer complaint indicator -> `customer_mirror_required`.
- SAP partner customer -> `customer_id`.
- SAP long text customer-safe summary -> `visible_summary`.
- SAP internal text -> excluded from mirror.
- ETQ Reliance complaint priority -> `next_priority`.
- TrackWise deviation escalation -> `escalation_reason`.
- MasterControl complaint packet -> mirror evidence link.
- Projection freshness floor: 5 seconds.
- Projection consumer: workflow-engine for deadline timers.
- Projection rule: mirror projection is redacted before customer portal receives it.

## Workflow Steps

- Node `notification-open`: notification exists and is not closed.
- Node `timer-check`: age and deadline state are evaluated.
- Decision `deadline-breached`: escalate priority.
- Decision `supplier-nonresponse`: assign supplier escalation queue.
- Decision `customer-impact`: require customer mirror sync.
- Decision `safety-risk`: request immediate quality hold.
- Node `priority-calc`: compute next priority.
- Decision `priority-decrease`: reject command.
- Node `cedar-escalate`: evaluate escalation policy.
- Node `escalation-record`: persist escalation.
- Node `workflow-deadline-update`: update timers.
- Node `mirror-redact`: create customer-safe summary.
- Decision `redaction-failed`: block mirror.
- Node `cedar-mirror`: evaluate mirror policy.
- Node `mirror-sync`: publish or update mirror.
- Node `supplier-notify`: publish supplier escalation.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish escalation projection.
- Node `close`: escalation is visible to owners.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-QUALITY_NOTIFICATION-ESCALATED`.
- `EVT-QUALITY_MANAGEMENT-CUSTOMER_COMPLAINT_MIRROR-SYNCED`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_NOTIFICATION-SUPPLIER_NONRESPONSE`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_NOTIFICATION-CHANGED`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_NOTIFICATION-IP_ACCEPTED`.
- ADR-0263 envelope stores `escalation_reason`.
- ADR-0263 envelope stores `prior_priority`.
- ADR-0263 envelope stores `next_priority`.
- ADR-0263 envelope stores `mirror_state`.
- ADR-0263 envelope stores `redaction_profile_id`.

## SLO Targets

- Escalation command p50: 60 ms.
- Escalation command p95: 220 ms.
- Escalation command p99: 600 ms.
- Timer-to-escalation p95: 30 seconds.
- Throughput: 120 escalations per second per cell.
- Availability: 99.95 percent monthly.
- Rationale: SLA breach timers tolerate seconds, but manual escalation should feel immediate.

## Failure Modes and Recovery

- Failure: escalation would lower priority.
- Recovery: `QN-ESCALATION-MONOTONIC-DENY` rejects command.
- Failure: supplier trust binding is missing.
- Recovery: `QN-SUPPLIER-TRUST-FALLBACK` routes internally and creates supplier portal task.
- Failure: customer mirror redaction fails.
- Recovery: `QN-MIRROR-BLOCK` keeps internal notification active and prevents customer sync.
- Failure: timer worker fires duplicate escalation.
- Recovery: `QN-ESCALATION-IDEMPOTENT` returns existing escalation by reason and due time.
- Failure: notification closes while escalation command is pending.
- Recovery: `QN-ESCALATION-CLOSED-DROP` audits stale command and noops.
- Failure: workflow deadline update fails.
- Recovery: `QN-WORKFLOW-TIMER-REPLAY` replays escalation event.

## Migration Notes

- Source vendor: SAP QM.
- Migrate notification priority and task deadlines from notification records.
- Preserve SAP priority code as `source_priority_code`.
- Source vendor: ETQ Reliance maps complaint priority and customer status into mirror.
- Source vendor: Sparta Systems TrackWise maps supplier nonresponse to escalation reason.
- Source vendor: MasterControl maps complaint packet visible text into mirror summary.
- Internal notes migrate only to internal notification, never customer mirror.
- Closed vendor complaints migrate as immutable mirror snapshots.
- Rollback path: pause mirror sync and keep internal escalation timers active.
- Supplier nonresponse history can be replayed into supplier scorecard.

## Cross-microservice Handoffs

- From quality-notification: open notification state.
- To workflow-engine: deadline timer and escalation tasks.
- To supplier-portal: nonresponse and escalation notices.
- To customer-portal: redacted complaint mirror.
- To quality-hold: safety risk containment request.
- To supplier scorecard: nonresponse signal.
- To ontology: escalation projection.
- To compliance: customer complaint evidence.

## Verification

- Unit: priority cannot decrease.
- Unit: closed notification cannot escalate.
- Unit: customer mirror excludes internal supplier score.
- Contract: REST escalate returns next priority and due date.
- Contract: gRPC stream emits escalation event.
- Event: mirror synced event validates.
- Policy: Cedar denies mirror with internal fields.
- Projection: SAP priority fixture maps field-for-field.
- SLO: escalation p95 under 220 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-QUALITY_NOTIFICATION-IP_ACCEPTED`.
