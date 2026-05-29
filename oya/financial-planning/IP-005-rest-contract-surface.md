---
doc_class: IP
ip_id: IP-005
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
journey_ref: J-FP-005-rest-contract-surface
tenant_class: product-critical
status: implementation-ready
date: 2026-05-20
owner_team: axis-financial-planning + council-api
---

# IP-005 Financial Planning rest-contract-surface

Service: financial-planning
ChangeSet scope: microservices/financial-planning/IP-005-rest-contract-surface.md
Benchmarks: Anaplan, Workday Adaptive Planning, OneStream, Vena, Pigment
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- rest-contract-surface-objective 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- rest-contract-surface-objective 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- rest-contract-surface-objective 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- rest-contract-surface-objective 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- rest-contract-surface-objective 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- rest-contract-surface-objective 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Prerequisites
- rest-contract-surface-prerequisites 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- rest-contract-surface-prerequisites 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- rest-contract-surface-prerequisites 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- rest-contract-surface-prerequisites 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- rest-contract-surface-prerequisites 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- rest-contract-surface-prerequisites 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Implementation steps
- rest-contract-surface-implementation-steps 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- rest-contract-surface-implementation-steps 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- rest-contract-surface-implementation-steps 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- rest-contract-surface-implementation-steps 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- rest-contract-surface-implementation-steps 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- rest-contract-surface-implementation-steps 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Tests and evidence
- rest-contract-surface-tests-and-evidence 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- rest-contract-surface-tests-and-evidence 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- rest-contract-surface-tests-and-evidence 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- rest-contract-surface-tests-and-evidence 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- rest-contract-surface-tests-and-evidence 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- rest-contract-surface-tests-and-evidence 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Rollback
- rest-contract-surface-rollback 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- rest-contract-surface-rollback 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- rest-contract-surface-rollback 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- rest-contract-surface-rollback 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- rest-contract-surface-rollback 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- rest-contract-surface-rollback 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Acceptance criteria
- rest-contract-surface-acceptance-criteria 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- rest-contract-surface-acceptance-criteria 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- rest-contract-surface-acceptance-criteria 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- rest-contract-surface-acceptance-criteria 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- rest-contract-surface-acceptance-criteria 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- rest-contract-surface-acceptance-criteria 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Context
- IP-005 owns the public REST surface for Financial Planning operators, migration tools, and partner connectors.
- REST commands must expose Oyatie planning nouns instead of vendor labels, while request bodies preserve vendor provenance when migration data enters the system.
- The surface covers forecast versions, scenario assumptions, driver imports, consolidation close packets, variance explanations, board report seals, and projection reads.
- Every mutating endpoint requires `Idempotency-Key`, `X-Oyatie-Tenant`, `Traceparent`, Cedar decision id, and audit-chain target id.
- REST responses must be compatible with OpenAPI 3.2.0 and avoid leaking vendor credentials or raw workbook payloads.
- API shape must be understandable to teams migrating from Anaplan, Workday Adaptive Planning, Oracle EPM Cloud, OneStream, Vena, Pigment, Planful, IBM Planning Analytics, Board, and Jedox.
- The REST layer coordinates IP-003 projection, IP-004 templates, IP-006 events, IP-007 gRPC internal calls, IP-008 Cedar evaluation, IP-009 credentials, and IP-010 regional routing.
- REST endpoints are human-facing control surfaces and cannot bypass workflow templates for state transitions.
- Financial values use decimal strings with explicit currency and scale metadata.
- Bulk migration endpoints return accepted jobs; they do not synchronously import vendor files.

## Data Model Deltas
```sql
CREATE TABLE financial_planning_rest_command (
  command_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  principal_id UUID NOT NULL,
  idempotency_key TEXT NOT NULL,
  command_name TEXT NOT NULL,
  resource_type TEXT NOT NULL,
  resource_id UUID,
  request_body JSONB NOT NULL,
  cedar_decision_id UUID NOT NULL,
  audit_chain_event_id UUID NOT NULL,
  http_status INTEGER NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, principal_id, idempotency_key)
);
CREATE INDEX fp_rest_command_resource_idx
  ON financial_planning_rest_command (tenant_id, resource_type, resource_id);
CREATE INDEX fp_rest_command_body_gin
  ON financial_planning_rest_command USING gin (request_body jsonb_path_ops);
```

```rust
#[derive(Clone, Debug)]
pub struct RestCommandEnvelope<T> {
    pub tenant_id: uuid::Uuid,
    pub principal_id: uuid::Uuid,
    pub idempotency_key: String,
    pub traceparent: String,
    pub cedar_decision_id: uuid::Uuid,
    pub audit_chain_event_id: uuid::Uuid,
    pub body: T,
}

#[derive(Clone, Debug)]
pub struct ForecastVersionOpenRequest {
    pub planning_entity_id: uuid::Uuid,
    pub scenario_ref: String,
    pub fiscal_year: i32,
    pub source_vendor: Option<String>,
    pub baseline_projection_id: Option<uuid::Uuid>,
}

#[derive(Clone, Debug)]
pub struct VarianceExplainRequest {
    pub plan_version_id: uuid::Uuid,
    pub actuals_dataset_ref: String,
    pub materiality_threshold_bps: i32,
    pub requested_output: Vec<String>,
}
```

## API Endpoints
- REST `POST /v1/financial-planning/forecast-versions:open`
```json
{
  "planning_entity_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f1005",
  "scenario_ref": "fy27-board-base",
  "fiscal_year": 2027,
  "source_vendor": "Workday Adaptive Planning",
  "baseline_projection_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f2005"
}
```
- REST `POST /v1/financial-planning/driver-imports`
```json
{
  "source_vendor": "Planful",
  "source_object_ref": "driver:headcount-growth",
  "target_plan_version_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f3005",
  "reconciliation_mode": "review_required"
}
```
- REST `POST /v1/financial-planning/variance-explanations`
```json
{
  "plan_version_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f4005",
  "actuals_dataset_ref": "analytics:actuals:fy27-m01",
  "materiality_threshold_bps": 75,
  "requested_output": ["metric_delta", "driver_delta", "owner_commentary"]
}
```
- gRPC bridge `ExecuteFinancialPlanningCommand(ExecuteFinancialPlanningCommandRequest)` is called after REST admission to usecase internals.
```json
{
  "tenant_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f0005",
  "command_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f5005",
  "command_name": "variance-explanations.create",
  "caller_service": "financial-planning-rest",
  "deadline_ms": 10000
}
```
- AsyncAPI command accepted event `financial-planning.rest.command.accepted.v1`
```json
{
  "command_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f5005",
  "tenant_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f0005",
  "command_name": "variance-explanations.create",
  "http_status": 202
}
```

## Cedar Policy Hooks
```cedar
permit (
  principal in FinancialPlanning::Role::"finance-planning-owner",
  action == FinancialPlanning::Action::"rest.command.submit",
  resource
) when {
  resource.tenant_id == principal.tenant_id &&
  context.idempotency_key != "" &&
  context.traceparent != "" &&
  context.audit_class == "ADR0263RestCommandAccepted" &&
  context.requested_endpoint in [
    "/v1/financial-planning/forecast-versions:open",
    "/v1/financial-planning/driver-imports",
    "/v1/financial-planning/variance-explanations"
  ]
};
```
- Principal: authenticated tenant finance user or connector service account.
- Action: `rest.command.submit`, `rest.command.read`, `rest.command.replay`.
- Resource: `FinancialPlanning::RestCommand::<command_id>` or target planning resource.
- Context: endpoint, idempotency key, traceparent, source vendor, audit class, requested region.

## Ontology Projection
- Anaplan REST import bodies map `modelId`, `moduleId`, and `lineItemId` into `baseline_projection_id`.
- Workday Adaptive Planning version payloads map `versionId` and `sheetId` into `forecast_version` REST bodies.
- Oracle EPM Cloud REST dimensions map into `projection_ref` fields before close commands are accepted.
- OneStream workflow profile references map into `consolidation_node_id`.
- Vena workbook and template ids map into `board_report_packet_id`.
- Pigment block ids map into `scenario_assumption_ref`.
- Planful driver ids map into `driver_import.source_object_ref`.
- IBM Planning Analytics TM1 cube and view ids map into `actuals_dataset_ref`.
- Board capsule ids map into `workflow_template_ref`.
- Jedox cube and integrator job ids map into `source_object_ref` and `reconciliation_mode`.

## Workflow Steps
- Node `receive-rest-command`: validate headers, JSON body, tenant, and idempotency.
- Node `authorize-command`: call Cedar with endpoint-specific action.
- Branch `forecast-open`: call workflow template `forecast-version-open`.
- Branch `driver-import`: call connector validation and workflow template `driver-model-import`.
- Branch `variance-explain`: call analytics actuals lookup and workflow template `variance-explain`.
- Branch `close-command`: require consolidation template and regional lock.
- Node `persist-command-envelope`: store request hash and decision id.
- Node `emit-accepted-event`: publish AsyncAPI command accepted.
- Node `return-problem-json`: return RFC 9457 error body on validation or policy failure.
- Node `handoff-usecase`: call gRPC usecase executor with normalized command.

## Audit Events
- `ADR0263RestCommandAccepted`: command accepted and stored.
- `ADR0263RestCommandRejected`: validation or policy rejection.
- `ADR0263RestCommandReplayed`: idempotent replay returned prior result.
- `ADR0263RestCommandDispatched`: command handed to internal usecase.
- `ADR0263RestProblemReturned`: problem response sent to caller.

## SLO Targets
- p50 REST command admission latency: 35 ms.
- p95 REST command admission latency: 140 ms.
- p99 REST command admission latency: 360 ms.
- Throughput: 600 mutating commands per tenant per minute.
- Availability: 99.95% for REST admission and query endpoints.
- Error budget: no more than 0.1% policy-evaluable commands lost before audit event emission.

## Failure Modes + Recovery
- Missing idempotency key: reject with 400, no state mutation, and emit `ADR0263RestCommandRejected`.
- Cedar decision timeout: fail closed with 503 and retry-after because financial planning commands mutate controlled evidence.
- Duplicate idempotency key: return prior command result and emit `ADR0263RestCommandReplayed`.
- Vendor object not projected: return 409 with required IP-003 projection reference.
- Regional mismatch: return 409 with IP-010 home-cell hint and no cross-region write.
- Downstream workflow unavailable: persist accepted command as pending dispatch, emit retry event, and surface 202 with operation id.

## Migration Notes
- Anaplan customers use REST driver-import endpoints after model/module projections exist.
- Workday Adaptive Planning customers use forecast-version open endpoints with version lineage.
- Oracle EPM Cloud customers use close command endpoints only after cube member projection.
- OneStream customers use consolidation endpoints with workflow profile lineage.
- Vena customers use board report seal endpoints with workbook provenance.
- Pigment customers use scenario endpoints with block lineage and dimension refs.
- Planful customers use driver-import endpoints with spread method metadata.
- IBM Planning Analytics customers use variance endpoints with TM1 view references.
- Board customers use workflow template refs in REST calls rather than executing capsule procedures.
- Jedox customers use reconciliation mode because cube rules need parser review.

## Cross-Microservice Handoffs
- To `api-gateway`: enforce HTTP/3, ECH/PQC headers, rate limits, and problem-json responses.
- To `policy-cedar`: evaluate endpoint and resource actions.
- To `workflow-engine`: instantiate published templates for accepted commands.
- To `audit-chain`: seal command accepted, rejected, replayed, and dispatched events.
- To `ontology`: resolve projection ids and planning object ids.
- To `analytics`: validate actuals dataset refs for variance endpoints.
- To `financial-planning` IP-006: publish command lifecycle events.
