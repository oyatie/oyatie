---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j79-cadence-orchestrator
journey_id: j79-eu-dsa-transparency-semi-annual-report
microservice: workflow-engine
role: cadence-orchestrator
status: draft
date: 2026-05-20
pack_overlay: EU-DSA
jurisdiction: EU
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0251, ADR-0263, ADR-0311, ADR-0313]
contracts: [OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3]
layer_enum: ADR-0105 13-layer canonical enum
layout: ADR-0131 flat per-microservice layout
audit_contract: ADR-0263 event classes required
cedar_contract: ADR-0243 deny-wins authorization
---

# IP-journey-j79-cadence-orchestrator - workflow-engine

## Goal

Implement the `cadence-orchestrator` slice for `workflow-engine` so j79 can satisfy `EU-DSA` without leaking tenant data, collapsing provider-BYOK and encryption-BYOK meanings, or bypassing Cedar.

## PRD row alignment

- PRD anchor: microservices/workflow-engine/PRD.md when present, otherwise the service manifest and architecture surface for that microservice.
- Journey anchor: docs/user-journeys/j79-eu-dsa-transparency-semi-annual-report/.
- Regulator article focus: DSA Art 14 terms and conditions.
- Rigor row: documentation-rigor.md section 2 IP row; one service, one single-PR-sized implementation plan.

## Files to author in the implementation PR

| File | Purpose | Notes |
|---|---|---|
| `microservices/workflow-engine/contracts/openapi/j79-cadence-orchestrator-v1.yaml` | OpenAPI 3.2.0 REST surface | External read/write or admin action |
| `microservices/workflow-engine/contracts/asyncapi/j79-cadence-orchestrator-events-v1.yaml` | AsyncAPI 3.1.0 event surface | Emits ADR-0263 events |
| `microservices/workflow-engine/contracts/proto/j79-cadence-orchestrator-v1.proto` | proto3 internal RPC | Service-to-service call path |
| `microservices/workflow-engine/policy/j79-cadence-orchestrator.cedar` | Cedar permit/forbid bundle | Deny-wins gate |
| `microservices/workflow-engine/runbooks/j79-cadence-orchestrator-rollback.md` | Rollback and incident path | Includes regulator deadline handling |
| `microservices/workflow-engine/tests/j79_cadence_orchestrator_test.rs` | Integration tests | Positive, negative, rollback, audit |

## Data model

Primary object: `dsa-transparency-report` with tenant_id, subject_id, pack_id, jurisdiction_code, purpose, data_class, deadline_at, byok_provider_ref, byok_encryption_ref, and prior_seal_ref.
The service may store a local projection only when it can be rebuilt from the audit-chain and source-of-truth service. Mutable state must carry tenant_id and data_class.

## Cedar fragment

```cedar
permit (principal is Principal, action == Action::"j79.workflow-engine.cadence-orchestrator", resource is JourneyObject) when {
  principal.tenant_id == resource.tenant_id &&
  resource.pack_overlay == "EU-DSA" &&
  context.jurisdiction == "EU" &&
  context.data_class_allowed == true &&
  context.audit_chain_required == true
};
```

## Audit event classes

- `J79WorkflowEngineCadenceOrchestratorStarted`.
- `J79WorkflowEngineCadenceOrchestratorPermitted`.
- `J79WorkflowEngineCadenceOrchestratorDenied`.
- `J79WorkflowEngineCadenceOrchestratorCommitted`.
- `J79WorkflowEngineCadenceOrchestratorRolledBack`.

## Bespoke implementation rows

The prior 24-row implementation loop repeated the same Scope/Contract/Authorization/State/Observability text. These rows bind the cadence to the shared workflow-engine contract and the specific regulator journey.

| Row | Trigger | Workflow-engine action | Evidence touch | Counterpart equivalence |
|---|---|---|---|---|
| 01 report period open | DSA reporting window opens. | `POST /runs` / `ExecutionEngine.StartWorkflowRun`; Cedar uses `policy/tenant-scope.cedar` and spec reads pin `version_sha`. | WorkflowStarted with report_period. | matches compliance calendar workflow. |
| 02 metric pull | moderation metrics are pulled from source services. | `POST /runs/{run_id}/signal` / `ExecutionEngine.SignalWorkflowRun`; Cedar uses `policy/tenant-scope.cedar` and spec reads pin `version_sha`. | StepCompleted per metric bundle hash. | matches Airflow DAG task. |
| 03 legal review | legal reviewer signs report draft. | `POST /runs/{run_id}/signal` / `ExecutionEngine.SignalWorkflowRun`; Cedar uses `policy/tenant-scope.cedar` and spec reads pin `version_sha`. | StepCompleted with reviewer permit id. | matches Camunda approval task. |
| 04 publication deadline | deadline slack enters risk window. | `POST /runs/{run_id}/pause` / `ExecutionEngine.PauseWorkflowRun`; Cedar uses `policy/tenant-scope.cedar` and spec reads pin `version_sha`. | WorkflowPaused with escalation reason. | matches ServiceNow compliance escalation. |
| 05 report publish | transparency report is published and sealed. | Workflow completion through `WorkflowCompleted` AsyncAPI event; Cedar uses `policy/tenant-scope.cedar` and spec reads pin `version_sha`. | audit hash and event seal. | matches Temporal workflow completion. |

Rows deleted as un-grounded: 19 prior rows repeated the same evidence and BYOK language without additional service artifacts. Provider credential handling and encryption-key rotation are not authored here unless a concrete workflow-engine contract row exists.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`, `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml`, `microservices/workflow-engine/contracts/proto/workflow-engine.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.
