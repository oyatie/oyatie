---
doc_class: IP
ip_id: IP-006
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
journey_ref: J-FP-006-async-event-surface
tenant_class: product-critical
status: implementation-ready
date: 2026-05-20
owner_team: axis-financial-planning + axis-eventing
---

# IP-006 Financial Planning async-event-surface

Service: financial-planning
ChangeSet scope: microservices/financial-planning/IP-006-async-event-surface.md
Benchmarks: Anaplan, Workday Adaptive Planning, OneStream, Vena, Pigment
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- async-event-surface-objective 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- async-event-surface-objective 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- async-event-surface-objective 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- async-event-surface-objective 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- async-event-surface-objective 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- async-event-surface-objective 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Prerequisites
- async-event-surface-prerequisites 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- async-event-surface-prerequisites 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- async-event-surface-prerequisites 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- async-event-surface-prerequisites 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- async-event-surface-prerequisites 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- async-event-surface-prerequisites 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Implementation steps
- async-event-surface-implementation-steps 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- async-event-surface-implementation-steps 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- async-event-surface-implementation-steps 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- async-event-surface-implementation-steps 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- async-event-surface-implementation-steps 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- async-event-surface-implementation-steps 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Tests and evidence
- async-event-surface-tests-and-evidence 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- async-event-surface-tests-and-evidence 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- async-event-surface-tests-and-evidence 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- async-event-surface-tests-and-evidence 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- async-event-surface-tests-and-evidence 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- async-event-surface-tests-and-evidence 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Rollback
- async-event-surface-rollback 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- async-event-surface-rollback 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- async-event-surface-rollback 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- async-event-surface-rollback 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- async-event-surface-rollback 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- async-event-surface-rollback 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Acceptance criteria
- async-event-surface-acceptance-criteria 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- async-event-surface-acceptance-criteria 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- async-event-surface-acceptance-criteria 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- async-event-surface-acceptance-criteria 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- async-event-surface-acceptance-criteria 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- async-event-surface-acceptance-criteria 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Context
- IP-006 owns Financial Planning AsyncAPI topics and event payload semantics.
- Events are the durable integration boundary between REST admission, gRPC usecases, workflow execution, ontology projection, analytics, and audit-chain sealing.
- The event surface must represent vendor migration, forecast state transitions, close-cycle milestones, scenario recalculation, variance explanation, and board report sealing.
- No event may contain raw vendor secrets, full uploaded workbooks, or unredacted formulas marked restricted.
- Partition keys use `tenant_id` plus `planning_entity_id` so tenant workloads preserve ordering without global serialization.
- Event schemas must carry `audit_chain_event_id`, `cedar_decision_id`, `traceparent`, `schema_version`, and `home_cell`.
- Vendor references from Anaplan, Workday Adaptive Planning, Oracle EPM Cloud, OneStream, Vena, Pigment, Planful, IBM Planning Analytics, Board, and Jedox remain provenance fields.
- Consumers must tolerate additive fields and reject unknown event names.
- Async replay is the recovery lane for projection, workflow, and analytics lag.
- This surface is not a queue-specific implementation; Pulsar, Kafka, or NATS adapters must honor the same event contract.

## Data Model Deltas
```sql
CREATE TABLE financial_planning_event_outbox (
  outbox_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  planning_entity_id UUID NOT NULL,
  topic_name TEXT NOT NULL,
  event_name TEXT NOT NULL,
  event_version TEXT NOT NULL,
  partition_key TEXT NOT NULL,
  payload JSONB NOT NULL,
  audit_chain_event_id UUID NOT NULL,
  cedar_decision_id UUID,
  traceparent TEXT NOT NULL,
  home_cell TEXT NOT NULL,
  published_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX fp_event_outbox_unpublished_idx
  ON financial_planning_event_outbox (created_at)
  WHERE published_at IS NULL;
CREATE INDEX fp_event_outbox_topic_idx
  ON financial_planning_event_outbox (tenant_id, topic_name, event_name);
```

```rust
#[derive(Clone, Debug)]
pub struct FinancialPlanningEventEnvelope<T> {
    pub event_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub planning_entity_id: uuid::Uuid,
    pub topic_name: String,
    pub event_name: String,
    pub event_version: String,
    pub partition_key: String,
    pub audit_chain_event_id: uuid::Uuid,
    pub cedar_decision_id: Option<uuid::Uuid>,
    pub traceparent: String,
    pub home_cell: String,
    pub payload: T,
}

#[derive(Clone, Debug)]
pub struct ForecastVersionOpenedEvent {
    pub plan_version_id: uuid::Uuid,
    pub scenario_ref: String,
    pub fiscal_year: i32,
    pub source_vendor: Option<String>,
}
```

## API Endpoints
- REST `GET /v1/financial-planning/events/{event_id}` returns event envelope metadata for audit inspection.
- REST `POST /v1/financial-planning/events:replay`
```json
{
  "tenant_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f0006",
  "topic_name": "financial-planning.forecast.version-opened.v1",
  "from_event_time": "2026-05-20T00:00:00Z",
  "reason": "analytics_projection_repair"
}
```
- gRPC `PublishFinancialPlanningEvent(PublishFinancialPlanningEventRequest) returns (PublishFinancialPlanningEventResponse)` writes to outbox inside usecase transactions.
```json
{
  "tenant_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f0006",
  "topic_name": "financial-planning.forecast.version-opened.v1",
  "event_name": "ForecastVersionOpened",
  "partition_key": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f0006:fy27",
  "home_cell": "us-east-1-cell-a"
}
```
- AsyncAPI topic `financial-planning.forecast.version-opened.v1`
```json
{
  "event_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f6006",
  "tenant_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f0006",
  "planning_entity_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f1006",
  "plan_version_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f2006",
  "scenario_ref": "fy27-board-base",
  "source_vendor": "Pigment",
  "home_cell": "us-east-1-cell-a"
}
```

## Cedar Policy Hooks
```cedar
permit (
  principal == FinancialPlanning::Service::"event-outbox-publisher",
  action == FinancialPlanning::Action::"event.publish",
  resource
) when {
  resource.tenant_id == context.tenant_id &&
  context.audit_class == "ADR0263AsyncEventPublished" &&
  context.topic_name like "financial-planning.*" &&
  context.home_cell == resource.home_cell
};
```
- Principal: event-outbox-publisher service, workflow-engine service, or replay operator.
- Action: `event.publish`, `event.replay`, `event.inspect`.
- Resource: `FinancialPlanning::Event::<event_id>` or topic resource.
- Context: topic name, home cell, replay reason, audit class, traceparent.

## Ontology Projection
- Anaplan model events map `modelId` and `revisionTag` into `source_object_ref` and `source_object_hash`.
- Workday Adaptive Planning version events map `versionId` into `plan_version_id`.
- Oracle EPM Cloud calculation events map `jobId` into `consolidation_close_job_ref`.
- OneStream workflow events map `workflowProfile` into `consolidation_node_id`.
- Vena workbook events map `workbookId` into `board_report_packet_id`.
- Pigment scenario events map `blockId` and `scenarioId` into `scenario_ref`.
- Planful load events map `loadProcessId` into `driver_import_id`.
- IBM Planning Analytics chore events map `choreName` into `source_process_ref`.
- Board capsule events map `capsuleId` into `workflow_template_ref`.
- Jedox integrator events map `jobId` into `driver_import_id`.

## Workflow Steps
- Node `stage-outbox-event`: write event in same transaction as domain mutation.
- Node `validate-topic-contract`: check event name, schema version, and partition key.
- Branch `forecast-event`: publish version-opened and scenario-recalculated events.
- Branch `close-event`: publish consolidation closed and board packet sealed events.
- Branch `migration-event`: publish projection, driver import, and reconciliation events.
- Node `publish-to-broker`: adapter-specific delivery with idempotent event id.
- Node `mark-published`: update `published_at`.
- Node `seal-audit-event`: ensure ADR-0263 audit event has event id.
- Node `replay-requested`: rebuild publish set from outbox rows.
- Node `consumer-lag-alert`: hand off to observability when lag breaches SLO.

## Audit Events
- `ADR0263AsyncEventStaged`: outbox row staged.
- `ADR0263AsyncEventPublished`: event published to broker.
- `ADR0263AsyncEventReplayRequested`: replay requested by operator or repair job.
- `ADR0263AsyncEventReplayCompleted`: replay completed with count and topic.
- `ADR0263AsyncEventConsumerLagBreach`: consumer lag exceeds SLO.
- `ADR0263AsyncEventPublishFailed`: broker publish failed after retries.

## SLO Targets
- p50 outbox staging latency: 15 ms.
- p95 broker publish latency from commit: 750 ms.
- p99 broker publish latency from commit: 2 seconds.
- Throughput: 5,000 events per tenant per minute during migration bursts.
- Availability: 99.95% for event staging; 99.9% for replay API.
- Consumer lag: p95 below 5 seconds for audit-chain and workflow-engine consumers.

## Failure Modes + Recovery
- Broker unavailable: keep outbox row unpublished, retry with exponential backoff, and alert after 3 minutes.
- Event schema rejected: mark row publish failed, block replay, and require schema migration review.
- Consumer lag breach: emit lag breach audit event and pause low-priority migration events.
- Duplicate publish attempt: broker adapter uses event id as idempotency key and treats duplicate as success.
- Cross-cell publish request: deny via Cedar and redirect to IP-010 home-cell route.
- Replay range too broad: require operator-specified topic and time range; refuse tenant-wide unbounded replay.

## Migration Notes
- Anaplan migration batches emit projection and driver import events separately.
- Workday Adaptive Planning cycles emit forecast-version-opened before budget-cycle transitions.
- Oracle EPM Cloud close jobs emit calculation and consolidation events with job references.
- OneStream workflow migrations emit certification-node events without executing OneStream logic.
- Vena workbook migrations emit board-report packet events after workbook range projection.
- Pigment scenario migrations emit scenario-recalculated after block graph normalization.
- Planful driver migrations emit driver-imported and reconciliation-required events.
- IBM Planning Analytics migration emits dimension-projected events before cube view actuals.
- Board migrations emit template-published events for capsule-derived procedures.
- Jedox migrations emit import-reconciliation events for integrator jobs and cube rules.

## Cross-Microservice Handoffs
- To `audit-chain`: consume every event class for sealing and evidence queries.
- To `workflow-engine`: consume command accepted, template published, and scenario events.
- To `analytics`: consume forecast, variance, and actuals-linked events.
- To `ontology`: consume projection events for graph update confirmation.
- To `observability`: consume lag, failure, and throughput metrics.
- To `cell`: enforce home-cell publish routing and metadata-only replication.
- To `financial-planning` IP-007: call gRPC publish hooks from internal usecases.
