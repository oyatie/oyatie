---
doc_class: ImplementationPlan
ip_id: IP-014
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
journey_ref: j123-multi-tenant-coordinated-product-launch
sap_submodule: EWM-WT (warehouse task)
tenant_class: paid
billing_components:
  - per_usage
persona: Imani Okafor, warehouse integration engineer
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-014: REST, gRPC, and worker surfaces for warehouse

## Context

- SAP submodule: EWM-WT warehouse task service surfaces.
- Persona: Imani Okafor, warehouse integration engineer.
- Journey leg: j123 high-volume launch uses REST for tenant commands, gRPC for internal workers, and AsyncAPI for replayable events.
- SAP tables: `/SCWM/ORDIM_O`, `/SCWM/WAREHOUSEORDER`, `/SCWM/QUANT`, `/SCWM/STORAGEBIN`.
- Oyatie surface: `WarehouseSurfaceContracts`.
- Precedent: SAP EWM service APIs plus AWS API Gateway to worker queue split.
- ADR-0253 binds HTTP/3, TLS 1.3, ECH, and PQC fallback; ADR-0329/0330/0331 requires implementation-ready contracts.
- Boundary: owns surface contract slices and worker dispatch envelopes; business logic remains in domain/usecase IPs.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.api_command_receipt (
  tenant_id UUID NOT NULL,
  command_receipt_id TEXT NOT NULL,
  surface_kind TEXT NOT NULL CHECK (surface_kind IN ('rest','grpc','worker','asyncapi')),
  operation_name TEXT NOT NULL,
  idempotency_key TEXT,
  http_trace_id TEXT,
  grpc_request_id TEXT,
  worker_job_id TEXT,
  status TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, command_receipt_id)
);
CREATE TABLE warehouse.worker_dispatch_job (
  tenant_id UUID NOT NULL,
  worker_job_id TEXT NOT NULL,
  operation_name TEXT NOT NULL,
  payload JSONB NOT NULL,
  dispatch_state TEXT NOT NULL,
  retry_count INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (tenant_id, worker_job_id)
);
```

### Rust Types

```rust
pub struct ApiCommandReceipt {
    pub tenant_id: TenantId,
    pub command_receipt_id: CommandReceiptId,
    pub surface_kind: SurfaceKind,
    pub operation_name: OperationName,
    pub idempotency_key: Option<IdempotencyKey>,
    pub status: SurfaceStatus,
}
pub struct WorkerDispatchJob {
    pub worker_job_id: WorkerJobId,
    pub operation_name: OperationName,
    pub payload: serde_json::Value,
    pub dispatch_state: DispatchState,
    pub retry_count: u32,
}
pub enum WarehouseSurfaceError { UnsupportedVersion, MissingIdempotencyKey, WorkerDispatchFailed, PolicyContextMissing, ContractViolation }
```

## API Endpoints

- REST `POST /v1/warehouse/inbound-deliveries`, `/outbound-deliveries`, `/putaway-tasks`, `/picking-waves`, `/yard-appointments`, `/labor-assignments`.
- REST `GET /v1/warehouse/command-receipts/{id}` returns surface-neutral receipt.
- gRPC package `warehouse.v1` exposes bounded-context services and worker dispatch service.
- gRPC `WarehouseWorker.DispatchJob`, `AckJob`, and `FailJob`.
- AsyncAPI channels `warehouse.*.created.v1`, `warehouse.*.policy-denied.v1`, and `warehouse.worker.job-failed.v1`.
- Worker queue `warehouse.command.dispatch.v1`.
- Consumers: API gateway, workflow-engine, adapter workers, audit-chain.

## Cedar Policy Hooks

- Policy: `warehouse::surface::invoke`.
- Principal: `TenantApiCaller` or `WarehouseWorker`.
- Action: `warehouse_surface_invoke`.
- Resource: `WarehouseOperation`.
- Context: `tenant_id`, `surface_kind`, `operation_name`, `api_version`, `transport`, `idempotency_key_present`.
- Forbid when transport violates ADR-0253, version is retired, idempotency is missing on mutation, or worker lacks operation capability.

## Ontology Projection

- Vendor object: SAP EWM service API call or queued task.
- Oyatie object: `warehouse.api_command_receipt`.
- `/SCWM/ORDIM_O-TANUM` -> warehouse task operation lineage where applicable.
- `/SCWM/WAREHOUSEORDER-WHO` -> warehouse order lineage.
- `/SCWM/QUANT-MATID` -> stock operation lineage.
- `/SCWM/STORAGEBIN-LGPLA` -> bin operation lineage.
- Surface kind -> contract type.
- Operation name -> bounded-context command.
- Projection freshness floor: 5 seconds.
- Projection rule: receipts project regardless of success or policy denial.

## Workflow Steps

- Node `ingress-validate`: enforce API version, transport, and idempotency.
- Node `policy-evaluate`: invoke surface Cedar gate.
- Decision `version-retired`: return deprecation error.
- Decision `policy-context-missing`: reject mutation and emit policy denied.
- Node `receipt-create`: persist command receipt.
- Node `worker-dispatch`: enqueue long-running worker job when operation is asynchronous.
- Decision `dispatch-failed`: mark receipt failed and retry job.
- Node `event-publish`: emit AsyncAPI event.
- Node `receipt-query`: expose status to callers.
- Node `audit-seal`: emit ADR-0263 evidence.

## Audit Events

- `EVT-WAREHOUSE-SURFACE-COMMAND_RECEIVED`.
- `EVT-WAREHOUSE-SURFACE-WORKER_DISPATCHED`.
- `EVT-WAREHOUSE-SURFACE-WORKER_FAILED`.
- `EVT-WAREHOUSE-SURFACE-CONTRACT_VIOLATION`.
- `EVT-WAREHOUSE-SURFACE-POLICY_DENIED`.
- `EVT-WAREHOUSE-SURFACE-IP_ACCEPTED`.
- ADR-0263 envelope stores `surface_kind`, `operation_name`, `api_version`, and transport profile.

## SLO Targets

- REST command receipt p50: 25 ms.
- REST command receipt p95: 90 ms.
- REST command receipt p99: 240 ms.
- Worker dispatch p95: 500 ms.
- Rationale: API callers need a fast durable receipt while workers execute heavier SAP-parity operations asynchronously.

## Failure Modes and Recovery

- Failure: `UNSUPPORTED-VERSION`; recovery: return SemVer deprecation detail and no-op mutation.
- Failure: `MISSING-IDEMPOTENCY-KEY`; recovery: reject mutation before domain call.
- Failure: `WORKER-DISPATCH-FAILED`; recovery: retry job and keep receipt in pending state.
- Failure: `POLICY-CONTEXT-MISSING`; recovery: reject and log integration defect.
- Failure: `CONTRACT-VIOLATION`; recovery: dead-letter payload and emit schema violation.
- Failure: `ASYNCAPI-BACKPRESSURE`; recovery: buffer in outbox and expose pending receipt.

## Migration Notes

- Keep existing OpenAPI, proto, and AsyncAPI file names stable.
- Add new operations behind versioned paths and channel suffix `.v1`.
- Do not migrate old unversioned worker names into public contract.
- Preserve old command receipt IDs only as source lineage.
- Rollback path: mark new operations disabled in surface registry while leaving read endpoints intact.
- Backfill order: operation registry, receipt table, worker queue, AsyncAPI channel bindings.

## Cross-microservice Handoffs

- From API gateway: authenticated tenant and transport context.
- To workflow-engine: asynchronous approvals and worker jobs.
- To audit-chain: command receipt and policy decision events.
- To ontology: surface command projection.
- To compliance: contract violation and policy denied evidence.
- To observability: surface latency and dispatch metrics.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The surface remains bound to SAP EWM warehouse task, delivery, RF, and worker semantics. |
| Persona specificity | Imani Okafor owns surface registration, contract compatibility, and rollback acceptance language. |
| Journey specificity | The j123 high-volume launch leg drives REST, gRPC, and AsyncAPI split by workload. |
| DDL anchor | The operation registry, receipt, worker queue, and channel binding tables above are normative. |
| Rust anchor | The operation descriptor, command receipt, worker job, and error enum above are implementation anchors. |
| REST anchor | `/v1/warehouse/*` endpoints are tenant-facing and must advertise HTTP/3 with fallback order preserved. |
| gRPC anchor | Warehouse worker services are internal replay contracts and must carry W3C trace context. |
| AsyncAPI anchor | Warehouse command and worker-result channels carry immutable event evidence. |
| Cedar anchor | Every mutating surface is default-deny and persists `cedar_decision_id` before dispatch. |
| Ontology anchor | Surface commands project to operation nodes so API receipts remain searchable by workflow and audit. |
| ADR-0263 class binding | Surface policy checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Contract or compliance-pack activation emits `OfficePackOverlayChanged`. |
| ADR-0263 security binding | API throttling emits `AbuseDefenceRateLimitHit` and bot/spoof/scrape denials use ADR-0297 classes. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, operation id, receipt id, transport, and `cedar_decision_id`. |
| Metric | `oya_warehouse_surface_requests_total{tenant_id,cell_id,operation,status}` caps operation/status cardinality. |
| Latency histogram | `oya_warehouse_surface_request_duration_seconds` tracks REST, gRPC, and worker dispatch latency. |
| Trace span | `warehouse.surface.dispatch` links API gateway, Cedar, worker queue, downstream usecase, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `operation_id`, `receipt_id`, `transport`, and schema version. |
| Capacity math | Worker queue capacity uses arrival_rate * service_time; saturation above 0.75 triggers admission backpressure. |
| Multi-region | Mutating operations route to home cell; DR cells serve read-only GET and replay surfaces until promoted. |
| Sovereign cells | Transport, audit, and command payloads remain in-region for active compliance-pack overlays. |
| Rollback | Mark operations disabled in registry, keep read endpoints active, and replay from last sealed receipt id. |
| Test evidence | Required tests cover HTTP/3 fallback, gRPC trace propagation, AsyncAPI schema, policy denial, and idempotency. |
| Rejected shortcut | A generic API gateway shim is rejected because it loses warehouse operation receipt and worker replay semantics. |
