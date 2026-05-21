---
doc_class: ImplementationPlan
ip_id: IP-009
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
sap_submodule: EWM-WT (warehouse task)
tenant_class: paid
billing_components:
  - per_usage
persona: Hana Suzuki, stock placement analyst
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-009: Usecase layer for putaway task

## Context

- SAP submodule: EWM-WT warehouse task orchestration.
- Persona: Hana Suzuki, stock placement analyst.
- Journey leg: j102 accepted receipt becomes an executable putaway task with inventory posting on confirmation.
- SAP tables: `/SCWM/ORDIM_O`, `/SCWM/STORAGEBIN`, `/SCWM/QUANT`, `/SCWM/WAREHOUSEORDER`.
- Oyatie usecase: `ExecutePutawayTask`.
- Precedent: SAP EWM task confirmation plus transactional outbox for inventory movement.
- ADR-0105 keeps orchestration outside the domain aggregate and ADR-0263 binds task audit events.
- Boundary: creates, assigns, confirms, compensates, and replays putaway task commands.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.putaway_command (
  tenant_id UUID NOT NULL,
  command_id TEXT NOT NULL,
  putaway_task_id TEXT NOT NULL,
  command_kind TEXT NOT NULL CHECK (command_kind IN ('create','assign','confirm','exception','compensate')),
  idempotency_key TEXT NOT NULL,
  command_status TEXT NOT NULL,
  failure_code TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, command_id),
  UNIQUE (tenant_id, idempotency_key)
);
CREATE TABLE warehouse.putaway_inventory_movement (
  tenant_id UUID NOT NULL,
  movement_id TEXT NOT NULL,
  putaway_task_id TEXT NOT NULL,
  source_bin_id TEXT NOT NULL,
  destination_bin_id TEXT NOT NULL,
  quantity NUMERIC(18,6) NOT NULL,
  movement_state TEXT NOT NULL,
  PRIMARY KEY (tenant_id, movement_id)
);
```

### Rust Types

```rust
pub struct PutawayCommand {
    pub tenant_id: TenantId,
    pub command_id: CommandId,
    pub putaway_task_id: PutawayTaskId,
    pub command_kind: PutawayCommandKind,
    pub idempotency_key: IdempotencyKey,
    pub command_status: CommandStatus,
}
pub struct PutawayInventoryMovement {
    pub movement_id: MovementId,
    pub putaway_task_id: PutawayTaskId,
    pub source_bin_id: BinId,
    pub destination_bin_id: BinId,
    pub quantity: Decimal,
    pub movement_state: MovementState,
}
pub enum ExecutePutawayTaskError { DestinationUnavailable, MovementPostFailed, CommandDuplicate, ExceptionPolicyDenied, ConfirmationReplayConflict }
```

## API Endpoints

- REST `POST /v1/warehouse/putaway-tasks/{id}:execute` runs assign or confirm command.
- REST `POST /v1/warehouse/putaway-tasks/{id}:compensate` reverses failed movement state.
- REST `GET /v1/warehouse/putaway-commands/{command_id}` fetches command state.
- gRPC `warehouse.putaway_usecase.v1.ExecutePutawayTask`.
- gRPC `CompensatePutawayTask` and `GetPutawayCommand`.
- AsyncAPI channel `warehouse.putaway-command.succeeded.v1`.
- AsyncAPI channel `warehouse.putaway-command.failed.v1`.
- Consumers: inventory-ledger, labor-assignment, ontology, compliance.

## Cedar Policy Hooks

- Policy: `warehouse::putaway_command::execute`.
- Principal: `WarehouseRfOperator`.
- Action: `putaway_command_execute`.
- Resource: `PutawayTask`.
- Context: `tenant_id`, `command_kind`, `source_bin_id`, `destination_bin_id`, `confirmed_qty`, `resource_id`.
- Forbid when operator is not assigned, destination bin is blocked, or command replay conflicts with prior payload.

## Ontology Projection

- Vendor object: SAP EWM putaway confirmation usecase.
- Oyatie object: `warehouse.putaway_command`.
- `/SCWM/ORDIM_O-TANUM` -> `putaway_task_id`.
- `/SCWM/STORAGEBIN-LGPLA` -> source and destination bin identity.
- `/SCWM/QUANT-QUAN` -> movement quantity.
- `/SCWM/WAREHOUSEORDER-WHO` -> labor bundle.
- Command kind -> usecase operation.
- Inventory movement -> ledger handoff evidence.
- Projection freshness floor: 2 seconds.
- Projection rule: compensated commands retain original command ID and compensation link.

## Workflow Steps

- Node `command-dedupe`: reject conflicting idempotency payloads.
- Node `policy-evaluate`: check operator assignment and bin state.
- Decision `destination-unavailable`: branch to exception command.
- Node `domain-apply`: update task state.
- Node `movement-post`: request inventory movement.
- Decision `movement-post-failed`: compensate task confirmation.
- Node `labor-update`: update workload and productivity evidence.
- Decision `exception-policy-denied`: keep task open.
- Node `outbox-dispatch`: emit success or failure.
- Node `audit-seal`: close command trail.

## Audit Events

- `EVT-WAREHOUSE-PUTAWAY_COMMAND-ACCEPTED`.
- `EVT-WAREHOUSE-PUTAWAY_COMMAND-CONFIRMED`.
- `EVT-WAREHOUSE-PUTAWAY_COMMAND-MOVEMENT_POSTED`.
- `EVT-WAREHOUSE-PUTAWAY_COMMAND-COMPENSATED`.
- `EVT-WAREHOUSE-PUTAWAY_COMMAND-POLICY_DENIED`.
- `EVT-WAREHOUSE-PUTAWAY_COMMAND-IP_ACCEPTED`.
- ADR-0263 envelope stores `command_kind`, `movement_id`, `source_bin_id`, and `destination_bin_id`.

## SLO Targets

- Command accept p50: 28 ms.
- Command accept p95: 95 ms.
- Command accept p99: 260 ms.
- Movement post p95: 350 ms.
- Rationale: RF confirmation must be under human perception threshold, while durable inventory posting may use outbox latency.

## Failure Modes and Recovery

- Failure: `DESTINATION-UNAVAILABLE`; recovery: reopen task with alternate bin selection.
- Failure: `MOVEMENT-POST-FAILED`; recovery: compensate confirmation and retry ledger outbox.
- Failure: `COMMAND-DUPLICATE-CONFLICT`; recovery: reject with prior command evidence.
- Failure: `EXCEPTION-POLICY-DENIED`; recovery: keep task assigned and request supervisor review.
- Failure: `CONFIRMATION-REPLAY-CONFLICT`; recovery: block replay and emit audit event.
- Failure: `LABOR-HANDOFF-FAILED`; recovery: retry analytics update without rolling back stock movement.

## Migration Notes

- Import open SAP tasks into command queue only when executable state remains.
- Import completed confirmations as movement evidence with no replay.
- Preserve SAP task number and source/destination bins as lineage.
- Map SAP exception codes before migrating exception commands.
- Rollback path: disable command endpoint and keep domain task read-only.
- Backfill order: task, command, movement, exception, audit outbox.

## Cross-microservice Handoffs

- From inbound receipt: staging stock requiring putaway.
- From labor assignment: assigned operator.
- To inventory-ledger: movement post.
- To workflow-engine: exception and compensation approval.
- To analytics: productivity evidence.
- To ontology: command and movement projection.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The usecase remains bound to SAP EWM warehouse-task orchestration. |
| Persona specificity | Hana Suzuki owns stock-placement command, exception, and rollback acceptance language. |
| Journey specificity | The j102 accepted-receipt-to-putaway leg drives movement posting and exception routing. |
| DDL anchor | The putaway command, movement, and exception tables above are the normative usecase persistence model. |
| Rust anchor | The command, movement result, and error enum above are the implementation contract. |
| REST anchor | Confirm, exception, compensate, and replay endpoints are the tenant command surface. |
| gRPC anchor | The putaway usecase service is the worker and replay contract for movement posting. |
| AsyncAPI anchor | Movement-posted and exception channels carry inventory and workflow evidence. |
| Cedar anchor | Confirmation and exception commands are default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP EWM task and bin lineage projects to command and movement nodes. |
| ADR-0263 class binding | Putaway command checks emit `OfficeBoundaryAttemptEvaluated` plus allowed or denied outcome classes. |
| ADR-0263 pack binding | Storage-policy overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on putaway APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, task id, source bin, destination bin, movement id, and `cedar_decision_id`. |
| Metric | `oya_warehouse_putaway_usecase_commands_total{tenant_id,cell_id,command,status}` caps cardinality. |
| Latency histogram | `oya_warehouse_putaway_usecase_duration_seconds` tracks command-to-movement-post latency. |
| Trace span | `warehouse.putaway_task.post_movement` links inbound receipt, labor assignment, inventory-ledger, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `task_id`, `operator_id`, `exception_code`, and compensation state. |
| Capacity math | Movement concurrency is capped by bin-lock wait time; wait p95 above 250 ms triggers queue backpressure. |
| Multi-region | Movement writes are home-cell authoritative; DR cells serve read-only movement projections. |
| Sovereign cells | Material and supplier lineage remains in-region for regulated pack overlays. |
| Rollback | Disable command endpoint, keep domain task read-only, and replay from last sealed movement audit id. |
| Test evidence | Required tests cover movement conflict, exception approval, tenant mismatch, inventory timeout, and idempotent replay. |
| Rejected shortcut | A generic `StockMove` usecase is rejected because it loses EWM task, exception, and bin-lock semantics. |
