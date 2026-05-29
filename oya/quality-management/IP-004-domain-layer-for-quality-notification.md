---
doc_class: ImplementationPlan
ip_id: IP-004
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

# IP-004: Domain layer for defect notification routing

## Context

- SAP QM submodule: QM-QN Quality Notifications.
- Topic: defect notification routing.
- Persona: Aisha Morgan, supplier quality engineer.
- Journey: j118 supplier defect containment.
- Journey leg: incoming defect is routed to the accountable supplier and internal owner.
- SAP precedent: Q notification type, defect item, task, partner, and priority.
- Oyatie aggregate: `QualityNotification`.
- Boundary: domain state, routing intent, and defect classification.
- ADR-0105 places notification invariants in the domain layer.
- ADR-0131 keeps the IP local to the quality-management microservice.
- ADR-0244 protects tenant and customer complaint scope.
- ADR-0263 governs defect notification events.
- ADR-0297 requires Cedar checks before routing.
- ADR-0314 prevents marketplace settlement side effects.
- ADR-0315 requires SAP QM notification parity.
- ADR-0329/0330/0331 requires enough detail for implementation.
- Routing must be deterministic and explainable.
- Customer-facing complaint mirrors must not expose internal supplier scoring by default.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.quality_notification (
  tenant_id UUID NOT NULL,
  notification_id TEXT NOT NULL,
  notification_type TEXT NOT NULL,
  defect_code TEXT NOT NULL,
  material_id TEXT,
  batch_id TEXT,
  supplier_id TEXT,
  customer_id TEXT,
  source_lot_id TEXT,
  priority TEXT NOT NULL,
  severity TEXT NOT NULL,
  state TEXT NOT NULL,
  routing_queue TEXT NOT NULL,
  owner_principal_id TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, notification_id)
);
CREATE TABLE quality_management.quality_notification_item (
  tenant_id UUID NOT NULL,
  notification_id TEXT NOT NULL,
  item_no INTEGER NOT NULL,
  defect_code TEXT NOT NULL,
  quantity_affected NUMERIC(20,6),
  containment_required BOOLEAN NOT NULL,
  evidence_ref TEXT,
  PRIMARY KEY (tenant_id, notification_id, item_no)
);
```

### Rust Types

```rust
pub struct QualityNotification {
    pub tenant_id: TenantId,
    pub notification_id: NotificationId,
    pub notification_type: NotificationType,
    pub defect_code: DefectCode,
    pub material_id: Option<MaterialId>,
    pub batch_id: Option<BatchId>,
    pub supplier_id: Option<SupplierId>,
    pub customer_id: Option<CustomerId>,
    pub source_lot_id: Option<InspectionLotId>,
    pub priority: NotificationPriority,
    pub severity: DefectSeverity,
    pub state: NotificationState,
    pub routing_queue: RoutingQueue,
    pub owner_principal_id: Option<PrincipalId>,
    pub items: Vec<NotificationItem>,
}
pub enum NotificationType { SupplierDefect, InternalDefect, CustomerComplaint, AuditFinding, SafetyIssue }
pub enum NotificationPriority { Low, Normal, High, Critical }
pub enum NotificationState { Open, Routed, Contained, Investigating, WaitingSupplier, Closed, Cancelled }
pub enum NotificationError {
    MissingDefectItem,
    InvalidPriorityForSeverity,
    CustomerMirrorPolicyDenied,
    OwnerOutsideTenant,
    SupplierRouteUnavailable,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/quality-notifications`.
- Creates a defect notification from lot, audit, return, or manual source.
- `POST /v1/quality-management/quality-notifications/{notification_id}:route`.
- Computes and stores routing queue plus owner.
- `POST /v1/quality-management/quality-notifications/{notification_id}:mirror-customer`.
- Publishes a customer-safe complaint mirror.
- `GET /v1/quality-management/quality-notifications/{notification_id}`.
- Returns routing, items, state, and evidence refs.

### gRPC

- Service: `quality_management.notification.v1.QualityNotificationService`.
- `rpc CreateNotification(CreateNotificationRequest) returns (NotificationReceipt)`.
- `rpc RouteNotification(RouteNotificationRequest) returns (NotificationRouteDecision)`.
- `rpc MirrorCustomerComplaint(MirrorCustomerComplaintRequest) returns (ComplaintMirrorReceipt)`.
- `rpc StreamNotificationEvents(StreamNotificationEventsRequest) returns (stream NotificationEvent)`.

### AsyncAPI

- Channel: `quality-management.quality-notification.created.v1`.
- Channel: `quality-management.quality-notification.routed.v1`.
- Channel: `quality-management.quality-notification.customer-mirrored.v1`.
- Message: `QualityNotificationCreated`.
- Message: `QualityNotificationRouted`.
- Payload carries `defect_code`, `priority`, `severity`, `routing_queue`, `supplier_id`, `customer_id`, `audit_event_class`.
- Consumer groups: workflow-engine, supplier-portal, customer-portal, quality-hold, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::quality_notification::create`.
- Principal: `QualityEngineer`, `CustomerSupportAgent`, or `InspectionLotWorker`.
- Action: `quality_notification_create`.
- Resource: `DefectReport`.
- Context: `tenant_id`, `source_system`, `defect_code`, `customer_visibility`, `pack_ids`.
- Policy: `quality_management::quality_notification::route`.
- Principal: `NotificationRouterWorker`.
- Action: `quality_notification_route`.
- Resource: `QualityNotification`.
- Context: `supplier_id`, `authorized_supplier_programs`, `severity`, `regulatory_pack`.
- Forbid: critical severity with low priority.
- Forbid: customer mirror includes internal supplier score.
- Forbid: owner principal tenant differs from notification tenant.
- Permit: supplier route only if supplier portal trust binding exists.

## Ontology Projection

- Vendor object: SAP QM `QMEL` notification header.
- Oyatie object: `quality_management.quality_notification`.
- `QMEL-QMNUM` -> `notification_id`.
- `QMEL-QMART` -> `notification_type`.
- `QMEL-MATNR` -> `material_id`.
- `QMEL-CHARG` -> `batch_id`.
- `QMEL-LIFNUM` -> `supplier_id`.
- `QMEL-KUNUM` -> `customer_id`.
- `QMEL-PRIOK` -> `priority`.
- `QMEL-QMCOD` -> `defect_code`.
- SAP item table `QMFE` -> `quality_notification_item`.
- SAP task table `QMSM` -> workflow tasks.
- SAP partner function -> `routing_queue`.
- SAP status profile -> `state`.
- Customer complaint text -> complaint mirror source.
- Projection freshness floor: 5 seconds.
- Projection redaction: customer mirrors exclude internal supplier risk fields.

## Workflow Steps

- Node `defect-intake`: defect report or complaint enters.
- Node `source-normalize`: lot, return, audit, or manual source is normalized.
- Node `defect-classify`: defect code and severity are assigned.
- Decision `critical-safety`: escalate to safety queue and quality hold.
- Decision `customer-visible`: evaluate mirror policy.
- Node `priority-derive`: priority derived from severity, customer, and pack.
- Decision `priority-conflict`: block until quality manager corrects.
- Node `owner-resolve`: choose owner by material, supplier, and plant.
- Decision `owner-missing`: route to triage queue.
- Node `supplier-route-check`: verify supplier portal trust binding.
- Decision `supplier-route-unavailable`: route to internal supplier-quality owner.
- Node `cedar-route`: evaluate routing policy.
- Node `state-routed`: state `Open` -> `Routed`.
- Node `customer-mirror-create`: publish safe complaint mirror.
- Node `quality-hold-request`: ask IP-005 when containment required.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish notification read model.
- Node `close`: notification is ready for CAPA or containment.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-QUALITY_NOTIFICATION-CHANGED`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_NOTIFICATION-CREATED`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_NOTIFICATION-ROUTED`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_NOTIFICATION-CUSTOMER_MIRRORED`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_NOTIFICATION-IP_ACCEPTED`.
- ADR-0263 envelope stores `notification_type`.
- ADR-0263 envelope stores `defect_code`.
- ADR-0263 envelope stores `priority`.
- ADR-0263 envelope stores `routing_queue`.
- ADR-0263 envelope stores `customer_visibility`.

## SLO Targets

- Create latency p50: 70 ms.
- Route latency p95: 300 ms.
- Route latency p99: 750 ms.
- Customer mirror p95: 500 ms.
- Throughput: 150 notifications per second per cell.
- Availability: 99.95 percent monthly.
- Rationale: routing is operationally urgent when defect severity is critical.

## Failure Modes and Recovery

- Failure: defect report has no defect item.
- Recovery: `QN-ITEM-REQUIRED` rejects creation and returns missing field details.
- Failure: critical safety defect is marked low priority.
- Recovery: `QN-PRIORITY-ESCALATE` overrides to critical and audits the correction.
- Failure: supplier portal route is unavailable.
- Recovery: `QN-SUPPLIER-FALLBACK` routes to internal supplier quality owner.
- Failure: customer mirror would expose internal supplier score.
- Recovery: `QN-MIRROR-REDACT` blocks field and emits policy denied event.
- Failure: owner leaves tenant membership.
- Recovery: `QN-OWNER-REASSIGN` re-runs owner resolver.
- Failure: notification event dispatch stalls.
- Recovery: `QN-OUTBOX-REPLAY` replays routed event with same notification id.

## Migration Notes

- Source vendor: SAP QM.
- Import notification header from `QMEL`.
- Import defect items from `QMFE`.
- Import tasks from `QMSM`.
- Preserve SAP notification type as `notification_type`.
- Source vendor: Sparta Systems TrackWise maps deviation records into notifications.
- Source vendor: ETQ Reliance maps complaints into customer-visible notifications.
- Source vendor: MasterControl maps nonconformance records into internal defect notifications.
- Customer complaint attachments migrate as evidence refs, not inline blobs.
- Rollback path: freeze customer mirror and keep internal notification route active.

## Cross-microservice Handoffs

- From inspection-lot: failed result creates defect notification.
- From customer-portal: customer complaint creates customer notification.
- To quality-hold: containment request for critical defects.
- To workflow-engine: investigation and task routing.
- To supplier-portal: supplier corrective action request.
- To compliance: regulated complaint evidence.
- To ontology: defect and route projection.
- To marketplace: read-only supplier quality context.

## Verification

- Unit: critical severity cannot keep low priority.
- Unit: customer mirror redacts internal risk.
- Unit: missing defect item rejected.
- Contract: REST route returns owner and queue.
- Contract: gRPC stream preserves notification id.
- Event: routed event validates.
- Policy: Cedar denies cross-tenant owner.
- Projection: SAP `QMEL` fixture maps field-for-field.
- SLO: routing p95 under 300 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-QUALITY_NOTIFICATION-IP_ACCEPTED`.
