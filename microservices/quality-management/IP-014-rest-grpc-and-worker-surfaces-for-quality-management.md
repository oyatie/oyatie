---
doc_class: ImplementationPlan
ip_id: IP-014
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
journey_ref: j101-multi-tier-supply-chain-formation
sap_submodule: QM-IM/QM-QC/QM-QN/QM-AU API Surface
tenant_class: paid
billing_components:
  - per_usage
persona: Sofia Blake, platform API owner
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-014: REST, gRPC, AsyncAPI, and worker surfaces

## Context

- SAP QM submodule: all quality-management submodules.
- Topic: API and worker surface consistency.
- Persona: Sofia Blake, platform API owner.
- Journey: j101 multi-tier supply-chain formation.
- Journey leg: production, warehouse, supplier, and customer systems call quality-management through stable contracts.
- SAP precedent: QM BAPIs, IDocs, workflow tasks, and output events.
- Oyatie surface: REST, gRPC, AsyncAPI, and workers.
- Boundary: contract shell and worker ingress, not domain behavior.
- ADR-0105 governs layer naming and API boundary.
- ADR-0131 keeps surfaces inside the flat microservice folder.
- ADR-0244 requires tenant-scoped request metadata.
- ADR-0263 requires audit event classes.
- ADR-0297 requires Cedar policy hooks.
- ADR-0314 prevents marketplace settlement mutation.
- ADR-0315 requires SAP QM parity coverage.
- ADR-0329/0330/0331 requires implementation-ready depth.
- Surface consistency matters because every upstream service calls these APIs.
- Worker commands must be idempotent and auditable.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.api_idempotency_key (
  tenant_id UUID NOT NULL,
  idempotency_key TEXT NOT NULL,
  operation_name TEXT NOT NULL,
  request_hash TEXT NOT NULL,
  response_pointer TEXT,
  state TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (tenant_id, idempotency_key)
);
CREATE TABLE quality_management.worker_command_inbox (
  tenant_id UUID NOT NULL,
  command_id TEXT NOT NULL,
  worker_name TEXT NOT NULL,
  command_type TEXT NOT NULL,
  payload_hash TEXT NOT NULL,
  command_state TEXT NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, command_id)
);
```

### Rust Types

```rust
pub struct QualityApiRequestContext {
    pub tenant_id: TenantId,
    pub principal_id: PrincipalId,
    pub idempotency_key: IdempotencyKey,
    pub correlation_id: CorrelationId,
    pub policy_bundle_version: PolicyBundleVersion,
    pub transport: TransportAttestation,
}
pub struct WorkerCommandEnvelope<T> {
    pub tenant_id: TenantId,
    pub command_id: CommandId,
    pub worker_name: WorkerName,
    pub command_type: CommandType,
    pub payload: T,
    pub payload_hash: PayloadHash,
}
pub enum SurfaceError {
    MissingTenantHeader,
    IdempotencyConflict,
    PolicyDenied,
    UnsupportedApiVersion,
    WorkerPayloadHashMismatch,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/inspection-plans`.
- `POST /v1/quality-management/inspection-lots`.
- `POST /v1/quality-management/inspection-lots/{id}/results`.
- `POST /v1/quality-management/certificates-of-analysis/{id}:publish`.
- `POST /v1/quality-management/quality-notifications`.
- `POST /v1/quality-management/quality-holds`.
- `POST /v1/quality-management/audit-findings/{id}:close`.
- Every mutating endpoint requires `X-Oya-Tenant-Id`.
- Every mutating endpoint requires `Idempotency-Key`.
- Every mutating endpoint returns audit event class and correlation id.

### gRPC

- Package: `quality_management.v1`.
- Service: `InspectionPlanService`.
- Service: `InspectionLotService`.
- Service: `CertificateOfAnalysisService`.
- Service: `QualityNotificationService`.
- Service: `QualityHoldService`.
- Service: `AuditEvidenceService`.
- Service: `VendorImportService`.
- Metadata: `tenant-id`, `principal-id`, `idempotency-key`, `policy-bundle-version`.
- Streaming methods use server-side heartbeats and retry tokens.

### AsyncAPI

- Channel prefix: `quality-management.*.v1`.
- Required message fields: `event_id`, `tenant_id`, `correlation_id`, `causation_id`.
- Required policy fields: `cedar_decision_id`, `policy_bundle_version`.
- Required audit field: `audit_event_class`.
- Required transport field: `source_cell`.
- Dead-letter prefix: `dlq.quality-management.*.v1`.
- Worker inbox consumes only signed command envelopes.
- Outbox emits only after domain/usecase commit succeeds.

## Cedar Policy Hooks

- Policy: `quality_management::surface::mutate`.
- Principal: any authenticated service or user principal.
- Action: REST or gRPC mutating operation.
- Resource: `QualityManagementSurface`.
- Context: `tenant_id`, `principal_id`, `operation_name`, `api_version`, `transport_attestation`.
- Policy: `quality_management::worker::consume`.
- Principal: `QualityManagementWorker`.
- Action: `worker_command_consume`.
- Resource: `WorkerCommandEnvelope`.
- Context: `worker_name`, `command_type`, `payload_hash`, `source_topic`, `pack_ids`.
- Forbid: missing tenant header.
- Forbid: unsupported API version.
- Forbid: idempotency conflict with different request hash.
- Forbid: worker payload hash mismatch.

## Ontology Projection

- Vendor object: SAP QM BAPI and IDoc call surface.
- Oyatie object: `quality_management.api_surface`.
- SAP BAPI name -> REST operation id.
- SAP IDoc message type -> AsyncAPI channel.
- SAP workflow task -> worker command type.
- SAP logical system -> `source_system_id`.
- SAP client -> `tenant_id` mapping.
- SAP transaction user -> `principal_id`.
- IQS-AQM API operation -> adapter command type.
- TIPQA webhook -> worker command envelope.
- TrackWise event -> AsyncAPI source topic.
- MasterControl export -> vendor import batch.
- ETQ Reliance webhook -> notification or audit worker.
- Projection freshness floor: contract release only.
- Projection rule: surface projection describes contract, not runtime domain state.
- Projection consumer: developer portal and API governance.

## Workflow Steps

- Node `request-received`: REST or gRPC request arrives.
- Node `context-extract`: tenant, principal, policy bundle, and idempotency key parsed.
- Decision `missing-context`: reject before usecase.
- Node `idempotency-check`: compare request hash.
- Decision `duplicate-same-hash`: return cached receipt.
- Decision `duplicate-different-hash`: reject conflict.
- Node `cedar-surface`: evaluate operation policy.
- Node `usecase-dispatch`: call exact usecase port.
- Node `response-cache`: store receipt pointer.
- Node `outbox-write`: write event after commit.
- Node `worker-command-received`: worker inbox accepts signed command.
- Decision `payload-hash-mismatch`: reject command.
- Node `cedar-worker`: evaluate worker consume policy.
- Node `worker-usecase-dispatch`: execute command idempotently.
- Node `dlq-route`: send exhausted command to dead letter.
- Node `audit-seal`: emit ADR-0263 class.
- Node `close`: return deterministic receipt.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-SURFACE-MUTATION_ACCEPTED`.
- `EVT-QUALITY_MANAGEMENT-SURFACE-MUTATION_DENIED`.
- `EVT-QUALITY_MANAGEMENT-WORKER-COMMAND_ACCEPTED`.
- `EVT-QUALITY_MANAGEMENT-WORKER-COMMAND_DLQ`.
- `EVT-QUALITY_MANAGEMENT-SURFACE-IP_ACCEPTED`.
- ADR-0263 envelope stores `operation_name`.
- ADR-0263 envelope stores `api_version`.
- ADR-0263 envelope stores `idempotency_key`.
- ADR-0263 envelope stores `worker_name`.
- ADR-0263 envelope stores `transport_attestation`.

## SLO Targets

- REST context validation p95: 20 ms.
- REST mutation wrapper p95 overhead: 40 ms.
- gRPC wrapper p95 overhead: 25 ms.
- Worker inbox ACK p95: 80 ms.
- Throughput: 1,000 surface requests per second per cell.
- Availability: 99.97 percent monthly.
- Rationale: surfaces are shared by all hot-path quality workflows.

## Failure Modes and Recovery

- Failure: missing tenant header.
- Recovery: `SURFACE-MISSING-TENANT-REJECT` returns 400 and emits deny event.
- Failure: idempotency conflict.
- Recovery: `SURFACE-IDEMPOTENCY-CONFLICT` returns 409 with original operation name.
- Failure: unsupported API version.
- Recovery: `SURFACE-VERSION-DENY` returns compatibility error.
- Failure: worker command hash mismatch.
- Recovery: `WORKER-HASH-DENY` dead-letters command.
- Failure: outbox write succeeds but response cache fails.
- Recovery: `SURFACE-RECEIPT-REBUILD` rebuilds receipt from operation event.
- Failure: worker exhausts retries.
- Recovery: `WORKER-DLQ-RUNBOOK` emits DLQ event and opens workflow task.

## Migration Notes

- Source vendor: SAP QM.
- Map SAP BAPIs to REST and gRPC operations.
- Map SAP IDocs to AsyncAPI channels.
- Source vendor: IQS-AQM webhook maps to worker command envelope.
- Source vendor: TIPQA export maps to vendor import REST endpoint.
- Source vendor: TrackWise event subscription maps to AsyncAPI ingest.
- Source vendor: MasterControl export maps to import batch.
- Source vendor: ETQ Reliance complaint webhook maps to notification create.
- No vendor API bypasses Cedar or idempotency.
- Rollback path: disable worker subscriptions while leaving read endpoints available.

## Cross-microservice Handoffs

- From production-planning: production inspection obligations.
- From warehouse: goods receipt inspection obligations.
- From customer-portal: complaint and certificate reads.
- To workflow-engine: worker tasks and human review.
- To ontology: event and contract projections.
- To compliance: audit event receipts.
- To marketplace: read-only trust signals.
- To identity/tenancy: principal and tenant validation.

## Verification

- Unit: idempotency same hash returns cached receipt.
- Unit: idempotency different hash rejects.
- Unit: missing tenant header rejects.
- Contract: OpenAPI includes every mutating surface.
- Contract: proto metadata includes required context.
- Event: AsyncAPI required fields validate.
- Policy: Cedar denies unsupported API version.
- Projection: SAP BAPI mapping fixture maps operation ids.
- SLO: wrapper overhead p95 under 40 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-SURFACE-IP_ACCEPTED`.
