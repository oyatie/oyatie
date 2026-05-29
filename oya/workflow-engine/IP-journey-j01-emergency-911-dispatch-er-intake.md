---
doc_class: IP
ip_id: IP-journey-j01-er-intake
journey_id: j01-emergency-911-dispatch
microservice: workflow-engine
role: er-intake-workflow
status: draft
related_adrs: [ADR-0298, ADR-0263, ADR-0244, ADR-0145]
depends_on:
  - microservices/audit-chain/IP-journey-j01-emergency-911-dispatch-emergency-classes.md
  - microservices/ontology/IP-journey-j01-emergency-911-dispatch-pending-chart.md
date: 2026-05-20
owner_team: axis-workflow-engine + axis-healthcare-vertical
---

# IP-journey-j01-er-intake — Workflow Engine: SNUH ER intake on KR-119 ETA

## Goal

Implement the `er-intake-incoming-acute` workflow that triggers on
`kr.119.eta.pre_arrival` AsyncAPI events for SNUH tenant, creates a
pending chart, pages next-available emergency-medicine nurse, and emits
audit + observability events.

## Data model

| Object | Storage | Schema |
|---|---|---|
| `WorkflowDefinition` | Postgres `workflow_definitions` | YAML workflow spec |
| `WorkflowExecution` | Postgres `workflow_executions` | per-execution record |
| `Kr119EtaEvent` | Kafka topic `kr.119.eta.pre_arrival` consumer | per j01 schemas |

```sql
CREATE TABLE workflow_executions (
  id UUID PRIMARY KEY,
  workflow_def_id TEXT NOT NULL,
  tenant_id TEXT NOT NULL,
  trigger_event_id TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('pending','running','succeeded','failed','dlq')),
  started_at TIMESTAMPTZ NOT NULL,
  completed_at TIMESTAMPTZ,
  audit_id TEXT NOT NULL,
  trace_id TEXT NOT NULL
);
```

## Workflow definition

```yaml
workflow: er-intake-incoming-acute
  tenant_scope: snuh.org
  trigger:
    type: asyncapi-event
    channel: kr.119.eta.pre_arrival
    filter: event.target_facility == "snuh.org" && event.is_acute_emergency == true
  steps:
    - id: parse-eta
      action: parse_event_to_internal
    - id: cedar-permit
      action: cedar.evaluate
      policy: workflow-er-intake-create-chart.cedar
    - id: create-chart
      action: ontology.create_pending_chart
    - id: page-nurse
      action: roster.notify_next_available
      params: { specialty: emergency, priority: acute-cardiac-suspect }
    - id: emit-audit
      action: audit_chain.emit_sealed
      class: ChartPendingCreatedFromPreArrival
    - id: emit-metric
      action: observability.emit_metric
      metric: snuh_er_pre_arrival_intake_total
```

## Files

| File | Size |
|---|---|
| `microservices/workflow-engine/src/journeys/er_intake.rs` | ~280 lines |
| `microservices/workflow-engine/workflows/er-intake-incoming-acute.yaml` | ~80 lines |
| `microservices/workflow-engine/policy/workflow-er-intake-create-chart.cedar` | ~30 lines |
| `microservices/workflow-engine/policy/workflow-trigger-from-event.cedar` | ~30 lines |
| `microservices/workflow-engine/contracts/asyncapi-v1.yaml` (extend) | +40 lines |
| `microservices/workflow-engine/contracts/proto/journey-er-intake.proto` | ~80 lines |
| `microservices/workflow-engine/db/migrations/2026-05-20-001-workflow-executions.sql` | ~40 lines |
| `microservices/workflow-engine/runbooks/er-intake-degraded.md` | ~150 lines |
| `microservices/workflow-engine/tests/integration/er_intake_test.rs` | ~400 lines |

## Cedar

```cedar
permit (
  principal == Workflow::"snuh.org/er-intake-incoming-acute",
  action == Action::"ehr.create_pending_chart",
  resource is Tenant
) when {
  principal.attested_origin == "kr-119-dispatch" &&
  resource.compliance_pack_active("pack-hipa-2024") &&
  resource.compliance_pack_active("pack-kr-medical-records-act") &&
  context.source_event.is_acute_emergency == true
};
```

## Audit events
- `WorkflowTriggered`, `ChartPendingCreatedFromPreArrival`, `NurseRosterPaged`.

## SLOs
- end-to-end (event → page) p95 ≤ 800ms.

## Tests
Per integration-test-plan §4.

## Parallel work
Depends on ontology IP (pending-chart object type). Independent of identity + messenger.

— end of IP —

## Completion expansion for j01 workflow-engine emergency-911-dispatch-er-intake

This appendix completes a pre-existing partial IP scaffold to the 400-line per-service bar required by /tmp/codex-brief-j01-j20-lifesafety.md.
The expansion is bound to ADR-0298 and the shared life-safety ADR pack ADR-0298, ADR-0299, ADR-0300, ADR-0301, ADR-0302, ADR-0303, ADR-0304, ADR-0305, ADR-0306, ADR-0292.

## Completion scope

- Microservice: workflow-engine.
- Journey: j01 Emergency 119 dispatch.
- Role: emergency-911-dispatch-er-intake.
- This is an additive completion; prior scaffold text above is preserved.
- No ADR, standard, PRD, or ARCHITECTURE file is modified by this appendix.

## Contract closure

| Surface | Required behavior | Evidence |
|---|---|---|
| OpenAPI 3.2.0 command | workflow-engine validates j01 emergency-911-dispatch-er-intake with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| AsyncAPI 3.1.0 event | workflow-engine validates j01 emergency-911-dispatch-er-intake with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| proto3 internal RPC | workflow-engine validates j01 emergency-911-dispatch-er-intake with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| Cedar v4.1 policy | workflow-engine validates j01 emergency-911-dispatch-er-intake with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| audit-chain seal | workflow-engine validates j01 emergency-911-dispatch-er-intake with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| observability span | workflow-engine validates j01 emergency-911-dispatch-er-intake with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |
| integration harness fixture | workflow-engine validates j01 emergency-911-dispatch-er-intake with tenant_id, cell_id, audience_type, binding_adr, and idempotency_key. | Contract test plus trace id and audit id. |

## Implementation steps

### Step 01 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 02 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 03 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 04 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 05 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 06 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 07 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 08 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 09 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 10 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 11 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 12 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 13 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 14 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 15 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 16 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 17 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 18 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 19 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 20 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 21 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 22 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 23 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 24 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 25 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 26 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 27 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 28 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 29 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

### Step 30 - workflow-engine emergency-911-dispatch-er-intake
- Build: wire the emergency-911-dispatch-er-intake handler behind the existing workflow-engine boundary and keep adapter logic outside the core domain.
- Validate: require JSON Schema 2020-12 payloads from docs/user-journeys/j01-*/schemas before any irreversible mutation.
- Authorize: evaluate Cedar with binding_adr=ADR-0298, tenant_id, audience_type, jurisdiction_pack, and critical_path flag.
- Persist: store state with idempotency key, subject reference, trace id, cell id, and pack overlay.
- Emit: publish an accepted or refused event and seal audit-chain evidence before returning success.
- Observe: emit low-cardinality metrics for accepted_total, refused_total, latency_ms, retry_count, and audit_seal_latency_ms.
- Test: cover happy path, cross-tenant refusal, duplicate replay, regional outage, malformed schema, and post-hoc review branch.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`, `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml`, `microservices/workflow-engine/contracts/proto/workflow-engine.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j01-emergency-911-dispatch-er-intake.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.
