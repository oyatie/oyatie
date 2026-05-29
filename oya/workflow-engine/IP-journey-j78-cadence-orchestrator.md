---
doc_class: IP
template_id: TPL-IP-Journey
ip_id: IP-journey-j78-cadence-orchestrator
journey_id: j78-eu-nis2-breach-three-stage-cadence
microservice: workflow-engine
role: cadence-orchestrator
status: draft
date: 2026-05-20
pack_overlay: EU-NIS2
jurisdiction: EU
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0251, ADR-0263, ADR-0311, ADR-0313]
contracts: [OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3]
layer_enum: ADR-0105 13-layer canonical enum
layout: ADR-0131 flat per-microservice layout
audit_contract: ADR-0263 event classes required
cedar_contract: ADR-0243 deny-wins authorization
---

# IP-journey-j78-cadence-orchestrator - workflow-engine

## Goal

Implement the `cadence-orchestrator` slice for `workflow-engine` so j78 can satisfy `EU-NIS2` without leaking tenant data, collapsing provider-BYOK and encryption-BYOK meanings, or bypassing Cedar.

## PRD row alignment

- PRD anchor: microservices/workflow-engine/PRD.md when present, otherwise the service manifest and architecture surface for that microservice.
- Journey anchor: docs/user-journeys/j78-eu-nis2-breach-three-stage-cadence/.
- Regulator article focus: NIS2 Art 21 cybersecurity risk-management.
- Rigor row: documentation-rigor.md section 2 IP row; one service, one single-PR-sized implementation plan.

## Files to author in the implementation PR

| File | Purpose | Notes |
|---|---|---|
| `microservices/workflow-engine/contracts/openapi/j78-cadence-orchestrator-v1.yaml` | OpenAPI 3.2.0 REST surface | External read/write or admin action |
| `microservices/workflow-engine/contracts/asyncapi/j78-cadence-orchestrator-events-v1.yaml` | AsyncAPI 3.1.0 event surface | Emits ADR-0263 events |
| `microservices/workflow-engine/contracts/proto/j78-cadence-orchestrator-v1.proto` | proto3 internal RPC | Service-to-service call path |
| `microservices/workflow-engine/policy/j78-cadence-orchestrator.cedar` | Cedar permit/forbid bundle | Deny-wins gate |
| `microservices/workflow-engine/runbooks/j78-cadence-orchestrator-rollback.md` | Rollback and incident path | Includes regulator deadline handling |
| `microservices/workflow-engine/tests/j78_cadence_orchestrator_test.rs` | Integration tests | Positive, negative, rollback, audit |

## Data model

Primary object: `nis2-breach-cadence` with tenant_id, subject_id, pack_id, jurisdiction_code, purpose, data_class, deadline_at, byok_provider_ref, byok_encryption_ref, and prior_seal_ref.
The service may store a local projection only when it can be rebuilt from the audit-chain and source-of-truth service. Mutable state must carry tenant_id and data_class.

## Cedar fragment

```cedar
permit (principal is Principal, action == Action::"j78.workflow-engine.cadence-orchestrator", resource is JourneyObject) when {
  principal.tenant_id == resource.tenant_id &&
  resource.pack_overlay == "EU-NIS2" &&
  context.jurisdiction == "EU" &&
  context.data_class_allowed == true &&
  context.audit_chain_required == true
};
```

## Audit event classes

- `J78WorkflowEngineCadenceOrchestratorStarted`.
- `J78WorkflowEngineCadenceOrchestratorPermitted`.
- `J78WorkflowEngineCadenceOrchestratorDenied`.
- `J78WorkflowEngineCadenceOrchestratorCommitted`.
- `J78WorkflowEngineCadenceOrchestratorRolledBack`.

## Bespoke implementation rows

The prior 24-row implementation loop repeated the same Scope/Contract/Authorization/State/Observability text. These rows bind the cadence to the shared workflow-engine contract and the specific regulator journey.

| Row | Trigger | Workflow-engine action | Evidence touch | Counterpart equivalence |
|---|---|---|---|---|
| 01 incident start | security incident is classified as NIS2 reportable. | `POST /runs` / `ExecutionEngine.StartWorkflowRun`; Cedar uses `policy/tenant-scope.cedar` and spec reads pin `version_sha`. | WorkflowStarted with incident_id and classification. | matches PagerDuty incident workflow. |
| 02 24h early warning | early-warning timer fires. | `EventBus.PublishWorkflowEvent` and AsyncAPI lifecycle channel; Cedar uses `policy/tenant-scope.cedar` and spec reads pin `version_sha`. | StepStarted/StepCompleted with deadline slack. | matches Step Functions wait state. |
| 03 72h notification | authority notification is submitted. | `POST /runs/{run_id}/signal` / `ExecutionEngine.SignalWorkflowRun`; Cedar uses `policy/tenant-scope.cedar` and spec reads pin `version_sha`. | StepCompleted with notification_receipt_hash. | matches ServiceNow major incident comms. |
| 04 one-month final | final report evidence is requested. | `POST /runs/{run_id}/pause` / `ExecutionEngine.PauseWorkflowRun`; Cedar uses `policy/tenant-scope.cedar` and spec reads pin `version_sha`. | WorkflowPaused if report owner misses deadline. | matches Camunda escalation timer. |
| 05 closure seal | final report accepted. | Workflow completion through `WorkflowCompleted` AsyncAPI event; Cedar uses `policy/tenant-scope.cedar` and spec reads pin `version_sha`. | audit seal and completion SLO. | matches Temporal scheduled workflow close. |

Rows deleted as un-grounded: 19 prior rows repeated the same evidence and BYOK language without additional service artifacts. Provider credential handling and encryption-key rotation are not authored here unless a concrete workflow-engine contract row exists.

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`, `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml`, `microservices/workflow-engine/contracts/proto/workflow-engine.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j78-cadence-orchestrator.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.
