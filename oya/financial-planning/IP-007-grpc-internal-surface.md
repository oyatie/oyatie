---
doc_class: IP
ip_id: IP-007
microservice: financial-planning
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0246
  - ADR-0253-amendment
  - ADR-0257
  - ADR-0258
  - ADR-0263
  - ADR-0294
  - ADR-0296
  - ADR-0297
  - ADR-0314
  - ADR-0321
journey_ref: J-FP-007-grpc-internal-surface
tenant_class: product-critical
status: implementation-ready
date: 2026-05-20
owner_team: axis-financial-planning + council-platform
---

# IP-007 Financial Planning grpc-internal-surface

Service: financial-planning
ChangeSet scope: microservices/financial-planning/IP-007-grpc-internal-surface.md
Benchmarks: Anaplan, Workday Adaptive Planning, OneStream, Vena, Pigment
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- grpc-internal-surface-objective 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- grpc-internal-surface-objective 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- grpc-internal-surface-objective 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- grpc-internal-surface-objective 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- grpc-internal-surface-objective 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- grpc-internal-surface-objective 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Prerequisites
- grpc-internal-surface-prerequisites 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- grpc-internal-surface-prerequisites 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- grpc-internal-surface-prerequisites 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- grpc-internal-surface-prerequisites 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- grpc-internal-surface-prerequisites 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- grpc-internal-surface-prerequisites 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Implementation steps
- grpc-internal-surface-implementation-steps 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- grpc-internal-surface-implementation-steps 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- grpc-internal-surface-implementation-steps 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- grpc-internal-surface-implementation-steps 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- grpc-internal-surface-implementation-steps 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- grpc-internal-surface-implementation-steps 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Tests and evidence
- grpc-internal-surface-tests-and-evidence 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- grpc-internal-surface-tests-and-evidence 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- grpc-internal-surface-tests-and-evidence 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- grpc-internal-surface-tests-and-evidence 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- grpc-internal-surface-tests-and-evidence 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- grpc-internal-surface-tests-and-evidence 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Rollback
- grpc-internal-surface-rollback 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- grpc-internal-surface-rollback 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- grpc-internal-surface-rollback 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- grpc-internal-surface-rollback 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- grpc-internal-surface-rollback 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- grpc-internal-surface-rollback 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Acceptance criteria
- grpc-internal-surface-acceptance-criteria 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- grpc-internal-surface-acceptance-criteria 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- grpc-internal-surface-acceptance-criteria 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- grpc-internal-surface-acceptance-criteria 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- grpc-internal-surface-acceptance-criteria 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- grpc-internal-surface-acceptance-criteria 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Context
- IP-007 owns internal proto3 calls for Financial Planning usecases.
- gRPC is service-to-service only; users and connector partners enter through REST or async import channels.
- Internal calls normalize commands after REST admission, execute workflow-bound usecases, publish outbox events, and read projection state.
- All requests must carry tenant, principal or service identity, Cedar decision id, audit-chain target, traceparent, home cell, and idempotency key.
- gRPC APIs must be stable enough for workflow-engine, ontology, analytics, audit-chain, and connector adapters.
- Vendor references remain provenance fields; gRPC method names stay Oyatie-native.
- Streaming is allowed only for migration progress and replay inspection, not for raw workbook or cube payload transfer.
- Deadlines are mandatory so close-cycle orchestration cannot hang a worker pool.
- Every method maps to a named REST command or AsyncAPI event for observability symmetry.
- Proto evolution uses additive fields and reserved tags for removed vendor-specific experiments.

## Data Model Deltas
```sql
CREATE TABLE financial_planning_grpc_call_ledger (
  call_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  caller_service TEXT NOT NULL,
  grpc_method TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  request_fingerprint BYTEA NOT NULL,
  cedar_decision_id UUID NOT NULL,
  audit_chain_event_id UUID NOT NULL,
  deadline_ms INTEGER NOT NULL,
  status_code TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, caller_service, grpc_method, idempotency_key)
);
CREATE INDEX fp_grpc_call_method_idx
  ON financial_planning_grpc_call_ledger (tenant_id, grpc_method, status_code);
CREATE INDEX fp_grpc_call_created_idx
  ON financial_planning_grpc_call_ledger (created_at);
```

```rust
#[derive(Clone, Debug)]
pub struct GrpcUsecaseEnvelope<T> {
    pub tenant_id: uuid::Uuid,
    pub caller_service: String,
    pub principal_id: Option<uuid::Uuid>,
    pub idempotency_key: String,
    pub cedar_decision_id: uuid::Uuid,
    pub audit_chain_event_id: uuid::Uuid,
    pub traceparent: String,
    pub home_cell: String,
    pub deadline_ms: u32,
    pub payload: T,
}

#[derive(Clone, Debug)]
pub struct ExecuteScenarioRecalculate {
    pub plan_version_id: uuid::Uuid,
    pub scenario_ref: String,
    pub assumption_projection_ids: Vec<uuid::Uuid>,
    pub source_vendor: Option<String>,
}
```

## API Endpoints
- REST proxy source: `POST /v1/financial-planning/scenarios:recalculate` dispatches to gRPC after admission.
```json
{
  "plan_version_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f2007",
  "scenario_ref": "fy27-downside",
  "assumption_projection_ids": ["018f9a60-7b8d-7f11-a9f1-0c7f4b9f3007"],
  "source_vendor": "Board"
}
```
- gRPC service excerpt:
```proto
service FinancialPlanningInternal {
  rpc ExecuteForecastVersionOpen(ExecuteForecastVersionOpenRequest) returns (UsecaseAccepted);
  rpc ExecuteScenarioRecalculate(ExecuteScenarioRecalculateRequest) returns (UsecaseAccepted);
  rpc ExecuteConsolidationClose(ExecuteConsolidationCloseRequest) returns (UsecaseAccepted);
  rpc ExecuteVarianceExplain(ExecuteVarianceExplainRequest) returns (UsecaseAccepted);
  rpc StreamMigrationProgress(StreamMigrationProgressRequest) returns (stream MigrationProgressEvent);
}
```
- gRPC example body:
```json
{
  "tenant_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f0007",
  "caller_service": "workflow-engine",
  "idempotency_key": "fp-scenario-recalc-007",
  "plan_version_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f2007",
  "scenario_ref": "fy27-downside",
  "assumption_projection_ids": ["018f9a60-7b8d-7f11-a9f1-0c7f4b9f3007"],
  "source_vendor": "Board"
}
```
- AsyncAPI response event `financial-planning.grpc.usecase.accepted.v1`
```json
{
  "call_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f7007",
  "grpc_method": "ExecuteScenarioRecalculate",
  "tenant_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f0007",
  "status_code": "OK"
}
```

## Cedar Policy Hooks
```cedar
permit (
  principal == FinancialPlanning::Service::"workflow-engine",
  action == FinancialPlanning::Action::"grpc.usecase.execute",
  resource
) when {
  resource.tenant_id == context.tenant_id &&
  context.grpc_method in [
    "ExecuteForecastVersionOpen",
    "ExecuteScenarioRecalculate",
    "ExecuteConsolidationClose",
    "ExecuteVarianceExplain"
  ] &&
  context.audit_class == "ADR0263GrpcUsecaseAccepted" &&
  context.deadline_ms <= 30000
};
```
- Principal: internal service account for workflow-engine, ontology, analytics, connector adapter, or audit-chain.
- Action: `grpc.usecase.execute`, `grpc.progress.stream`, `grpc.projection.read`.
- Resource: `FinancialPlanning::InternalUsecase::<method>`.
- Context: tenant id, grpc method, deadline, caller service, audit class, home cell.

## Ontology Projection
- Anaplan module ids passed over gRPC resolve to `PlanningMetric` projection ids.
- Workday Adaptive Planning version ids resolve to `FinancialPlanVersion` ids before usecase execution.
- Oracle EPM Cloud cube members resolve to `PlanningDimension` ids for close calls.
- OneStream workflow profiles resolve to `ConsolidationNode` ids for certification calls.
- Vena workbook references resolve to `BoardReportPacket` ids for seal calls.
- Pigment block ids resolve to `ScenarioAssumption` ids for recalculation calls.
- Planful driver ids resolve to `PlanningMetric` ids for import calls.
- IBM Planning Analytics cube views resolve to actuals dataset refs for variance calls.
- Board capsule procedure ids resolve to workflow template refs.
- Jedox integrator job ids resolve to import reconciliation refs.

## Workflow Steps
- Node `receive-grpc-call`: validate mTLS service identity and method deadline.
- Node `load-call-ledger`: enforce idempotency across caller service and method.
- Node `authorize-internal-call`: call Cedar with method and tenant.
- Branch `workflow-engine-caller`: execute state transition usecase.
- Branch `ontology-caller`: return projected object deltas.
- Branch `analytics-caller`: return variance and actuals metadata.
- Node `execute-usecase`: run domain logic inside transaction.
- Node `stage-outbox-events`: write IP-006 event rows.
- Node `record-call-result`: persist status and request fingerprint.
- Node `return-usecase-accepted`: return operation id and audit event id.

## Audit Events
- `ADR0263GrpcUsecaseAccepted`: internal usecase accepted.
- `ADR0263GrpcUsecaseRejected`: policy, deadline, or validation rejection.
- `ADR0263GrpcUsecaseCompleted`: usecase completed and events staged.
- `ADR0263GrpcProgressStreamOpened`: migration progress stream opened.
- `ADR0263GrpcDeadlineExceeded`: method deadline exceeded.
- `ADR0263GrpcIdempotentReplay`: duplicate call returned prior response.

## SLO Targets
- p50 unary gRPC admission latency: 20 ms.
- p95 unary gRPC admission latency: 85 ms.
- p99 unary gRPC admission latency: 200 ms.
- Throughput: 2,000 unary usecase calls per tenant per minute.
- Availability: 99.95% for unary internal APIs.
- Streaming progress update interval: p95 below 3 seconds during migration.

## Failure Modes + Recovery
- Caller missing mTLS service identity: reject before payload deserialization and emit `ADR0263GrpcUsecaseRejected`.
- Deadline over maximum: reject with invalid argument and require caller to choose async workflow.
- Duplicate idempotency key: return stored result and emit idempotent replay.
- Usecase transaction conflict: return retryable aborted status and preserve no partial mutation.
- Outbox staging fails: rollback usecase transaction and return unavailable.
- Streaming consumer disconnects: close stream, retain migration progress in outbox, and allow resume from event id.

## Migration Notes
- Anaplan migration adapters call gRPC only after REST or connector admission approves batch scope.
- Workday Adaptive Planning version transitions use unary calls with strict deadlines.
- Oracle EPM Cloud close migrations use gRPC for close status reconciliation, not cube extraction.
- OneStream certification migration uses workflow-engine as caller, not direct connector execution.
- Vena workbook migration calls board report seal usecases after workbook range projection.
- Pigment scenario migration calls scenario recalculate usecase after block graph projection.
- Planful driver migration calls driver import usecase with source driver ref.
- IBM Planning Analytics migration streams progress for TM1 dimension and cube view imports.
- Board migration calls workflow-template refs rather than capsule procedure execution.
- Jedox migration uses import reconciliation usecases with parser-review status.

## Cross-Microservice Handoffs
- To `workflow-engine`: execute template-bound state transitions.
- To `ontology`: resolve projection ids and graph relationships.
- To `analytics`: request actuals and variance-ready datasets.
- To `audit-chain`: seal gRPC call lifecycle events.
- To `connector`: receive normalized vendor migration batches.
- To `observability`: export method latency, deadline, and status counters.
- To `financial-planning` IP-006: stage AsyncAPI outbox events from usecase transactions.
