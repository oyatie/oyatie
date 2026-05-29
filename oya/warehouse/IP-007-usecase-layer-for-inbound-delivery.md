---
doc_class: ImplementationPlan
ip_id: IP-007
microservice: warehouse
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0245
  - ADR-0253
  - ADR-0263
  - ADR-0294
  - ADR-0297
  - ADR-0314
  - ADR-0315
  - ADR-0329
  - ADR-0330
  - ADR-0331
journey_ref: j102-raw-material-purchase-with-quality-attestation
sap_submodule: EWM-WIM (inventory)
tenant_class: paid
billing_components:
  - per_usage
persona: Nikhil Rao, receiving operations analyst
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-007: Usecase layer for inbound delivery

## Context

- SAP submodule: EWM-WIM inventory update after inbound receipt.
- Persona: Nikhil Rao, receiving operations analyst.
- Journey leg: j102 receipt must become stock evidence while preserving supplier quality lineage.
- SAP tables: `/SCWM/PRDI`, `/SCWM/QUANT`, `/SCWM/STORAGEBIN`, `/SCWM/ORDIM_O`.
- Oyatie usecase: `ReceiveInboundDelivery`.
- Precedent: SAP EWM goods receipt posting plus Stripe idempotency-key command execution.
- ADR-0105 places orchestration in usecase layer and ADR-0314 keeps marketplace settlement out of this flow.
- Boundary: orchestrates receive, quality gate, stock evidence, putaway task request, and audit outbox.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.inbound_receipt_command (
  tenant_id UUID NOT NULL,
  command_id TEXT NOT NULL,
  inbound_delivery_id TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  principal_id TEXT NOT NULL,
  command_status TEXT NOT NULL CHECK (command_status IN ('accepted','executing','succeeded','failed','compensated')),
  failure_code TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, command_id),
  UNIQUE (tenant_id, idempotency_key)
);
CREATE TABLE warehouse.inbound_receipt_outbox (
  tenant_id UUID NOT NULL,
  event_id TEXT NOT NULL,
  command_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  payload JSONB NOT NULL,
  dispatch_state TEXT NOT NULL DEFAULT 'pending',
  PRIMARY KEY (tenant_id, event_id)
);
```

### Rust Types

```rust
pub struct ReceiveInboundDeliveryCommand {
    pub tenant_id: TenantId,
    pub command_id: CommandId,
    pub inbound_delivery_id: InboundDeliveryId,
    pub idempotency_key: IdempotencyKey,
    pub principal_id: PrincipalId,
}
pub struct InboundReceiptOutboxEvent {
    pub event_id: EventId,
    pub command_id: CommandId,
    pub event_type: InboundReceiptEventType,
    pub payload: serde_json::Value,
}
pub enum ReceiveInboundDeliveryError { DuplicateCommand, QualityGateDenied, StockPostFailed, PutawayRequestFailed, AuditOutboxBackpressure }
```

## API Endpoints

- REST `POST /v1/warehouse/inbound-deliveries/{id}:receive` orchestrates item receipt.
- REST `GET /v1/warehouse/inbound-receipt-commands/{command_id}` returns command state.
- REST `POST /v1/warehouse/inbound-receipt-commands/{command_id}:compensate` reverses a failed partial receipt.
- gRPC `warehouse.inbound_usecase.v1.ReceiveInboundDelivery`.
- gRPC `GetInboundReceiptCommand` and `CompensateInboundReceipt`.
- AsyncAPI channel `warehouse.inbound-receipt.command-succeeded.v1`.
- AsyncAPI channel `warehouse.inbound-receipt.command-failed.v1`.
- Consumers: putaway-task, inventory-ledger, quality-management, audit-chain.

## Cedar Policy Hooks

- Policy: `warehouse::inbound_receipt::execute`.
- Principal: `WarehouseDockOperator`.
- Action: `receive_inbound_delivery`.
- Resource: `InboundDelivery`.
- Context: `tenant_id`, `idempotency_key`, `quality_gate_state`, `dock_door_id`, `principal_capabilities`.
- Forbid when quality gate denies receipt, duplicate idempotency key belongs to a different payload, or dock assignment is stale.

## Ontology Projection

- Vendor object: SAP EWM goods receipt usecase.
- Oyatie object: `warehouse.inbound_receipt_command`.
- `/SCWM/PRDI-DOCID` -> `inbound_delivery_id`.
- `/SCWM/QUANT-QUAN` -> posted stock quantity.
- `/SCWM/STORAGEBIN-LGPLA` -> destination or staging bin.
- `/SCWM/ORDIM_O-TANUM` -> requested putaway task.
- Command outbox -> event lineage.
- Cedar decision -> policy proof.
- Projection freshness floor: 3 seconds.
- Projection rule: failed commands project with failure code for replay analysis.

## Workflow Steps

- Node `command-accept`: dedupe by idempotency key.
- Node `policy-evaluate`: run Cedar receipt policy.
- Decision `quality-gate-denied`: fail command and emit blocked event.
- Node `domain-receive`: update inbound aggregate.
- Node `stock-post`: create inventory evidence.
- Decision `stock-post-failed`: compensate receipt state.
- Node `putaway-request`: request putaway task creation.
- Decision `putaway-request-failed`: keep stock in staging and queue retry.
- Node `outbox-dispatch`: emit success or failure event.
- Node `command-close`: seal command evidence.

## Audit Events

- `EVT-WAREHOUSE-INBOUND_RECEIPT-COMMAND_ACCEPTED`.
- `EVT-WAREHOUSE-INBOUND_RECEIPT-POLICY_DENIED`.
- `EVT-WAREHOUSE-INBOUND_RECEIPT-STOCK_POSTED`.
- `EVT-WAREHOUSE-INBOUND_RECEIPT-COMPENSATED`.
- `EVT-WAREHOUSE-INBOUND_RECEIPT-OUTBOX_DISPATCHED`.
- `EVT-WAREHOUSE-INBOUND_RECEIPT-IP_ACCEPTED`.
- ADR-0263 envelope stores `command_id`, `idempotency_key`, `quality_gate_state`, and `stock_post_ref`.

## SLO Targets

- Command accept p50: 30 ms.
- Command accept p95: 110 ms.
- Command accept p99: 300 ms.
- End-to-end receipt p95: 900 ms including stock and putaway request.
- Rationale: dock RF feedback must be fast, while downstream stock and task creation can complete under one second.

## Failure Modes and Recovery

- Failure: `DUPLICATE-COMMAND-PAYLOAD-MISMATCH`; recovery: reject with conflict and preserve first command.
- Failure: `QUALITY-GATE-DENIED`; recovery: create quality review task.
- Failure: `STOCK-POST-FAILED`; recovery: compensate receipt and retry inventory handoff.
- Failure: `PUTAWAY-REQUEST-FAILED`; recovery: hold stock in staging and retry durable outbox.
- Failure: `AUDIT-OUTBOX-BACKPRESSURE`; recovery: keep command succeeded but dispatch pending.
- Failure: `COMMAND-COMPENSATE-DENIED`; recovery: require supervisor policy override.

## Migration Notes

- Migrate open SAP receipt processes into command history only when replay is required.
- Closed SAP receipts become read-only stock movement evidence.
- Preserve SAP posting date and user as source lineage.
- Recompute idempotency keys from source document plus item for deterministic import.
- Rollback path: disable receipt command endpoint and use direct domain read-only projection.
- Backfill order: inbound delivery, receipt command, stock post evidence, outbox event.

## Cross-microservice Handoffs

- From quality-management: receipt gate status.
- To inventory-ledger: stock post command.
- To putaway-task: staging stock requiring putaway.
- To workflow-engine: compensation and quality exception tasks.
- To ontology: receipt command projection.
- To compliance: audit chain and command replay evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The usecase remains bound to SAP EWM inventory update after inbound receipt. |
| Persona specificity | Nikhil Rao owns receipt command replay, stock-post evidence, and rollback acceptance language. |
| Journey specificity | The j102 receipt-to-stock leg drives supplier-quality lineage and inventory handoff ordering. |
| DDL anchor | The receipt command and stock-post evidence tables above are the normative usecase persistence model. |
| Rust anchor | The receipt command type, result type, and error enum above are the implementation contract. |
| REST anchor | Receive, compensate, and replay endpoints are the tenant command surface. |
| gRPC anchor | The inbound receipt usecase service is the internal stock-post and replay contract. |
| AsyncAPI anchor | Receipt-posted and receipt-compensated channels carry inventory-ledger evidence. |
| Cedar anchor | Receipt operation is default-deny and must persist `cedar_decision_id` before stock posting. |
| Ontology anchor | SAP receipt lineage projects to command and stock evidence nodes without replacing domain delivery identity. |
| ADR-0263 class binding | Receipt command checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Quality or residency overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on receipt usecase APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, receipt command id, stock post id, material id, and `cedar_decision_id`. |
| Metric | `oya_warehouse_inbound_receipt_usecase_commands_total{tenant_id,cell_id,command,status}` caps cardinality. |
| Latency histogram | `oya_warehouse_inbound_receipt_usecase_duration_seconds` tracks command-to-stock-post latency. |
| Trace span | `warehouse.inbound_receipt.post_stock` links quality-management, inventory-ledger, ontology, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `delivery_id`, `material_id`, `stock_post_id`, and compensation state. |
| Capacity math | Receipt concurrency is capped by inventory-ledger write tokens; queue depth above 0.8 utilization triggers backpressure. |
| Multi-region | Home-cell stock post is authoritative; DR cells serve read-only receipt command projections. |
| Sovereign cells | Supplier-quality and material evidence remains in-region for regulated pack overlays. |
| Rollback | Disable receipt command endpoint, keep domain projection read-only, and replay from last sealed receipt audit id. |
| Test evidence | Required tests cover gate denial, stock-post timeout, compensation, tenant mismatch, and idempotent replay. |
| Rejected shortcut | A generic `ReceiveGoods` usecase is rejected because it loses SAP EWM receipt and quality-gate lineage. |
