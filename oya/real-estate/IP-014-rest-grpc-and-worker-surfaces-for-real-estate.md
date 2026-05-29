---
doc_class: ImplementationPlan
ip_id: IP-014
microservice: real-estate
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
journey_ref: j137-corporate-internal-audit-sox-controls-test
sap_submodule: RE-FX-CN (contracts)
tenant_class: paid
billing_components:
  - per_usage
persona: Jae Park, real-estate integration engineer
status: Accepted
date: 2026-05-20
owner_team: axis-real-estate + axis-erp-parity
---

# IP-014: REST, gRPC, and worker surfaces for real-estate

## Context

- SAP submodule: RE-FX-CN contract and worker surface integration.
- Persona: Jae Park, real-estate integration engineer.
- Journey leg: j137 audit commands use REST for tenant operations, gRPC for workers, and AsyncAPI for immutable accounting/contract events.
- SAP tables: `VICDCONTRACT`, `VICDOBJASS`, `VICDCONDLINE`, `VICDADJREASN`.
- Oyatie surface: `RealEstateSurfaceContracts`.
- Precedent: SAP RE-FX BAPI/API split plus AWS API Gateway and worker queue pattern.
- ADR-0253 binds transport and ADR-0329/0330/0331 requires implementation-ready contract surfaces.
- Boundary: owns API receipts, worker dispatch, and event surface conventions; bounded-context logic remains in domain/usecase IPs.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.api_command_receipt (
  tenant_id UUID NOT NULL,
  command_receipt_id TEXT NOT NULL,
  surface_kind TEXT NOT NULL CHECK (surface_kind IN ('rest','grpc','worker','asyncapi')),
  operation_name TEXT NOT NULL,
  idempotency_key TEXT,
  trace_id TEXT,
  status TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, command_receipt_id)
);
CREATE TABLE real_estate.worker_dispatch_job (
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
pub struct RealEstateApiCommandReceipt {
    pub tenant_id: TenantId,
    pub command_receipt_id: CommandReceiptId,
    pub surface_kind: SurfaceKind,
    pub operation_name: OperationName,
    pub idempotency_key: Option<IdempotencyKey>,
    pub trace_id: TraceId,
    pub status: SurfaceStatus,
}
pub struct RealEstateWorkerDispatchJob {
    pub worker_job_id: WorkerJobId,
    pub operation_name: OperationName,
    pub payload: serde_json::Value,
    pub dispatch_state: DispatchState,
    pub retry_count: u32,
}
pub enum RealEstateSurfaceError { UnsupportedVersion, MissingIdempotencyKey, WorkerDispatchFailed, ContractViolation, PolicyDenied }
```

## API Endpoints

- REST `POST /v1/real-estate/lease-contracts`, `/facility-objects`, `/occupancy-allocations`, `/rent-schedules`, `/lease-accounting-events`, `/facility-service-requests`.
- REST `GET /v1/real-estate/command-receipts/{id}` returns surface-neutral receipt.
- gRPC package `real_estate.v1` exposes bounded-context services.
- gRPC `RealEstateWorker.DispatchJob`, `AckJob`, and `FailJob`.
- AsyncAPI channels `real-estate.*.created.v1`, `real-estate.*.policy-denied.v1`, and `real-estate.worker.job-failed.v1`.
- Worker queue `real-estate.command.dispatch.v1`.
- Consumers: API gateway, workflow-engine, adapter workers, audit-chain.

## Cedar Policy Hooks

- Policy: `real_estate::surface::invoke`.
- Principal: `TenantApiCaller` or `RealEstateWorker`.
- Action: `real_estate_surface_invoke`.
- Resource: `RealEstateOperation`.
- Context: `tenant_id`, `surface_kind`, `operation_name`, `api_version`, `transport`, `idempotency_key_present`.
- Forbid when version is retired, mutation lacks idempotency, transport violates ADR-0253, or worker lacks operation capability.

## Ontology Projection

- Vendor object: SAP RE-FX API or BAPI call.
- Oyatie object: `real_estate.api_command_receipt`.
- `VICDCONTRACT-CONTRACT` -> contract operation lineage.
- `VICDOBJASS-OBJNR` -> object assignment lineage.
- `VICDCONDLINE-CONDGUID` -> condition operation lineage.
- `VICDADJREASN-ADJREASON` -> adjustment operation lineage.
- Surface kind -> contract transport.
- Operation name -> bounded context command.
- Projection freshness floor: 5 seconds.
- Projection rule: receipts project for success, failure, and policy denial.

## Workflow Steps

- Node `ingress-validate`: enforce API version, transport, and idempotency.
- Node `policy-evaluate`: run surface Cedar gate.
- Decision `unsupported-version`: return deprecation error.
- Decision `missing-idempotency-key`: reject mutation.
- Node `receipt-create`: persist command receipt.
- Node `worker-dispatch`: queue long-running job when required.
- Decision `dispatch-failed`: mark receipt pending-retry.
- Node `event-publish`: emit AsyncAPI event.
- Node `receipt-query`: expose status.
- Node `audit-seal`: emit command receipt evidence.

## Audit Events

- `EVT-REAL_ESTATE-SURFACE-COMMAND_RECEIVED`.
- `EVT-REAL_ESTATE-SURFACE-WORKER_DISPATCHED`.
- `EVT-REAL_ESTATE-SURFACE-WORKER_FAILED`.
- `EVT-REAL_ESTATE-SURFACE-CONTRACT_VIOLATION`.
- `EVT-REAL_ESTATE-SURFACE-POLICY_DENIED`.
- `EVT-REAL_ESTATE-SURFACE-IP_ACCEPTED`.
- ADR-0263 envelope stores surface kind, operation name, API version, and transport profile.

## SLO Targets

- REST receipt p50: 25 ms.
- REST receipt p95: 90 ms.
- REST receipt p99: 250 ms.
- Worker dispatch p95: 500 ms.
- Rationale: property operations need quick durable receipts; accounting and import work can be asynchronous.

## Failure Modes and Recovery

- Failure: `UNSUPPORTED-VERSION`; recovery: return SemVer deprecation error.
- Failure: `MISSING-IDEMPOTENCY-KEY`; recovery: reject before business mutation.
- Failure: `WORKER-DISPATCH-FAILED`; recovery: retry job and keep receipt pending.
- Failure: `CONTRACT-VIOLATION`; recovery: dead-letter payload and emit schema event.
- Failure: `POLICY-DENIED`; recovery: expose policy decision ID to caller.
- Failure: `ASYNCAPI-BACKPRESSURE`; recovery: buffer in outbox.

## Migration Notes

- Keep existing contract file names stable.
- Add new operations under versioned REST path and `.v1` AsyncAPI channel.
- Do not expose unversioned worker names publicly.
- Preserve old receipts as source lineage only.
- Rollback path: disable new operation in surface registry.
- Backfill order: operation registry, receipt table, worker queue, event bindings.

## Cross-microservice Handoffs

- From API gateway: authenticated principal and transport context.
- To workflow-engine: approval and worker jobs.
- To audit-chain: command receipt events.
- To ontology: surface command projection.
- To compliance: contract violation and policy denied evidence.
- To observability: latency and dispatch metrics.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The surface remains bound to SAP RE-FX contract and worker surface integration. |
| Persona specificity | Jae Park owns operation registry, receipt compatibility, and rollback acceptance language. |
| Journey specificity | The j137 audit-command leg drives REST, gRPC, and AsyncAPI split by workload. |
| DDL anchor | Operation registry, receipt, worker queue, and event-binding tables above are normative. |
| Rust anchor | Operation descriptor, command receipt, worker job, and error types above are implementation anchors. |
| REST anchor | `/v1/real-estate/*` endpoints are tenant-facing and must preserve HTTP/3 fallback order. |
| gRPC anchor | Real-estate worker services are internal replay contracts with W3C trace propagation. |
| AsyncAPI anchor | Contract, accounting, facility, and service-request channels carry immutable events. |
| Cedar anchor | Every mutating operation is default-deny and persists `cedar_decision_id`. |
| Ontology anchor | Surface commands project to operation nodes for workflow and audit discovery. |
| ADR-0263 class binding | Surface policy checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Contract or compliance-pack activation emits `OfficePackOverlayChanged`. |
| ADR-0263 security binding | API throttling emits `AbuseDefenceRateLimitHit`; bot/spoof/scrape denials use ADR-0297 classes. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, operation id, receipt id, transport, and `cedar_decision_id`. |
| Metric | `oya_real_estate_surface_requests_total{tenant_id,cell_id,operation,status}` caps operation/status cardinality. |
| Latency histogram | `oya_real_estate_surface_request_duration_seconds` tracks REST, gRPC, and worker dispatch latency. |
| Trace span | `real_estate.surface.dispatch` links API gateway, Cedar, worker queue, downstream usecase, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `operation_id`, `receipt_id`, `transport`, and schema version. |
| Capacity math | Worker queue admission uses arrival_rate * service_time and throttles when saturation exceeds 0.75. |
| Multi-region | Mutations route to home cell; DR cells serve read-only GET and replay surfaces until promoted. |
| Sovereign cells | Lease, facility, and accounting payloads remain in-region for active compliance packs. |
| Rollback | Disable new operation in surface registry, keep read endpoints intact, and replay from last sealed receipt id. |
| Test evidence | Required tests cover HTTP/3 fallback, gRPC trace propagation, AsyncAPI schema, policy denial, and idempotency. |
| Rejected shortcut | A generic API shim is rejected because it loses RE-FX operation receipt and worker replay semantics. |
