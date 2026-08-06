---
doc_class: CrossMicroserviceHandoffMatrix
title: "Cloud IaC Cross-Microservice Handoff Matrix"
status: Draft
date: 2026-05-20
microservice: cloud-iac
owner_team: axis-cloud-iac
---

# Cloud IaC Cross-Microservice Handoff Matrix

This matrix records concrete handoffs for the `cloud-iac` microservice.
REST shapes are from `iac/contracts/openapi/cloud-iac.yaml`.
Async shapes are from `iac/contracts/asyncapi/cloud-iac-events.yaml`.
gRPC names are from `iac/contracts/proto/cloud_iac.proto`.
Cedar policies are from `iac/policies/`.
Audit-chain emission follows ADR-0263 with `source_microservice=cloud-iac`.
The service owns render, plan preview, apply, rollback, provenance, and drift reporting handoffs.

## Inbound Callers

| # | Calling microservice | Named API on `cloud-iac` | Data shape | Cedar permit required | Audit event emitted |
|---|---|---|---|---|---|
| 1 | `application` | `GET /charts/{digest}/provenance` `getProvenance` | `Provenance` with `artifact_digest`, `slsa_level`, `sigstore_signature`, `builder_id` | `public-read.cedar` action `CloudIac::read_slsa_attestation_public` | `CloudIacProvenanceRead` |
| 2 | `api-gateway` | `GET /microservices/api-gateway/apply-state/{environment}` `getApplyState` | `ApplyStateRecord` | `ci-scope.cedar` action `CloudIac::read_gateway_apply_state` | `CloudIacApplyStateRead` |
| 3 | `cell` | `POST /microservices/cell/plan-preview` `planPreview` | `PlanPreview` | `ci-scope.cedar` action `CloudIac::preview_cell_plan` | `CloudIacPlanPreviewed` |
| 4 | `cell` | `POST /microservices/cell/apply` `triggerApply` | `ApplyJob` | `ci-scope.cedar` action `CloudIac::apply_cell_manifest` | `CloudIacApplyStarted` |
| 5 | `cloud-k8s` | `GET /microservices/{microservice}/apply-state/{environment}` `getApplyState` | `ApplyStateRecord` | `ci-scope.cedar` action `CloudIac::read_apply_state` | `CloudIacApplyStateReadByK8s` |
| 6 | `cloud-secrets` | `GET /charts/{digest}/provenance` `getProvenance` | `Provenance` for secret controller chart | `ci-scope.cedar` action `CloudIac::read_secret_chart_provenance` | `CloudIacSecretChartProvenanceRead` |
| 7 | `developer-sdk` | `POST /charts/validate` `validateChartSignature` | `ChartSignatureValidationRequest` | `public-read.cedar` action `CloudIac::validate_chart_signature` | `CloudIacChartSignatureValidated` |
| 8 | `audit-chain` | `GET /microservices/{microservice}/drift-report/{pack}/{environment}` `getDriftReport` | `DriftReport` | `auditor-scope.cedar` action `CloudIac::read_drift_report_for_audit` | `CloudIacDriftReportReadForAudit` |
| 9 | `ops-dashboard-control-center` | `GET /microservices/{microservice}/apply/{apply_id}` `getApplyJob` | `ApplyJob` | `auditor-scope.cedar` action `CloudIac::read_apply_job_operator` | `CloudIacApplyJobRead` |
| 10 | `observability` | `GET /microservices/{microservice}/drift-report/{pack}/{environment}` `getDriftReport` | `DriftReport` projection | `auditor-scope.cedar` action `CloudIac::read_drift_report_metric` | `CloudIacDriftReportReadForMetrics` |
| 11 | `payments` | `POST /microservices/payments/render` `triggerRender` | `RenderedManifest` | `ci-scope.cedar` action `CloudIac::render_payments_chart` | `CloudIacPaymentsRenderRequested` |
| 12 | `audit-chain` | `GET /roots/{pack}/{period_id}` delegated through apply provenance | `SignedRootReference` inline | `auditor-scope.cedar` action `CloudIac::read_signed_root_reference` | `CloudIacSignedRootReferenceRead` |

## Outbound Callees

| # | Callee microservice | Named API called by `cloud-iac` | Data shape sent or received | Cedar permit required | Audit event consumed |
|---|---|---|---|---|---|
| 1 | `audit-chain` | `POST /emit` `emitEvent` | `AuditEvent` with `source_microservice=cloud-iac` | `tenant-scope.cedar` action `AuditChain::emit_iac_event` | consumes `CloudIacAuditReceiptAccepted` |
| 2 | `cloud-secrets` | `GET /secrets/{tenant}/cloud-iac/gitops-signer/reference` | `SecretReference` | `secret-isolation.md` guard `cloud_iac_signer_read` | consumes `SecretReferenceRead` |
| 3 | `cloud-secrets` | `GET /secrets/{tenant}/cloud-iac/kubeconfig/reference` | `SecretReference` | `secret-isolation.md` guard `cloud_iac_kubeconfig_read` | consumes `SecretReferenceRead` |
| 4 | `cloud-k8s` | `POST /clusters/{pack}/{environment}/apply` | inline `KubernetesApplyRequest {manifest_digest, apply_id, environment}` | `ci-scope.cedar` action `CloudK8s::apply_rendered_manifest` | consumes `K8sApplyAccepted` |
| 5 | `cloud-k8s` | `POST /clusters/{pack}/{environment}/rollback` | inline `KubernetesRollbackRequest {rollback_id, target_sha}` | `ci-scope.cedar` action `CloudK8s::rollback_rendered_manifest` | consumes `K8sRollbackAccepted` |
| 6 | `cell` | `GET /cells` `listCells` | `Cell[]` topology | `ci-scope.cedar` action `Cell::list_for_iac` | consumes `CellInventoryReadByIac` |
| 7 | `api-gateway` | `POST /edge/cell-route-refresh` | inline `CellRouteRefresh {cell_id, route_epoch}` after gateway deploy | `route-authorization.cedar` action `Gateway::refresh_cell_route` | consumes `ApiGatewayCellRouteRefreshAccepted` |
| 8 | `application` | `POST /modules/{module}/manifest:publish` | `ModuleManifestPublishRequest` | `ci-scope.cedar` action `Application::publish_module_manifest` | consumes `ApplicationModuleManifestPublished` |
| 9 | `observability` | `POST /metrics/iac-drift` | inline `IacDriftMetric {microservice, pack, environment, drift_score}` | `public-read.cedar` action `Observability::write_iac_metric` | consumes `MetricAccepted` |
| 10 | `ops-dashboard-control-center` | `POST /incidents/iac-drift-detected` | inline `IacDriftIncident {microservice, pack, environment, drift_items}` | `auditor-scope.cedar` action `OpsDashboard::open_iac_incident` | consumes `OpsIncidentOpened` |
| 11 | `developer-sdk` | `POST /submissions/{submissionId}/status` | inline `VettingStageEvent {stage=iac_render, status}` | `developer-scope.cedar` action `DeveloperSdk::update_submission_stage` | consumes `DeveloperSubmissionStageUpdated` |
| 12 | `compliance` | `POST /residency/evaluate` | inline `ResidencyEvaluationRequest {pack, environment, artifact_digest}` | `data-residency.md` guard `iac_pack_jurisdiction_allowed` | consumes `ResidencyDecisionRecorded` |

## Event Subscriptions

| # | AsyncAPI channel subscribed | Event class | Handler behavior | Dead-letter policy |
|---|---|---|---|---|
| 1 | `workflow-events/render.requested` | `RenderRequestedPayload` | renders Helm/Kustomize manifest and stores `RenderedManifest` digest | retry 5 times, then `cloud-iac.dlq.render_requested` |
| 2 | `workflow-events/microservice.registered` | `MicroserviceRegisteredPayload` | creates chart source registry entry and policy gate | retry 6 times, then `cloud-iac.dlq.microservice_registered` |
| 3 | `cloud-secrets.secret.rotated` | `SecretLifecyclePayload` | reloads gitops signer and kubeconfig references by version | retry 10 times, then `cloud-iac.dlq.secret_rotated` |
| 4 | `cloud-secrets.secret.revoked` | `SecretRevokedPayload` | blocks apply jobs using revoked credential version | retry 10 times, then `cloud-iac.dlq.secret_revoked` |
| 5 | `cell.workflow-events/cell.lifecycle.transition` | `CellLifecycleTransitionPayload` | regenerates placement-aware manifests for affected cell | retry 5 times, then `cloud-iac.dlq.cell_lifecycle_transition` |
| 6 | `cell.workflow-events/cell.decommissioned` | `CellDecommissionedPayload` | removes cell resources from desired state | retry 5 times, then `cloud-iac.dlq.cell_decommissioned` |
| 7 | `developer-sdk.oya.developer-sdk.submission` | `VettingStageEvent` | runs chart validation for plugin submissions | retry 4 times, then `cloud-iac.dlq.developer_submission` |
| 8 | `audit-chain.audit.seal.minted` | `SealMintedPayload` | seals apply and rollback audit receipts | retry 10 times, then `cloud-iac.dlq.audit_seal_minted` |
| 9 | `api-gateway.upstream.circuit-open` | `GatewayCircuitOpen` | pauses gateway apply if upstream outage is active | retry 4 times, then `cloud-iac.dlq.gateway_circuit_open` |
| 10 | `payments.payment-events.{tenant_id}` | `PayoutFailed` | blocks payment infrastructure rollout only when payment pack is degraded | retry 3 times, then `cloud-iac.dlq.payment_payout_failed` |

## Event Emissions

| # | AsyncAPI channel published | Event class | Payload schema | Downstream consumers |
|---|---|---|---|---|
| 1 | `workflow-events/render.requested` | `RenderRequestedPayload` | `cloud-iac-events.yaml#/components/schemas/RenderRequestedPayload` | `cloud-iac` render workers |
| 2 | `workflow-events/render.completed` | `RenderCompletedPayload` | `RenderCompletedPayload` | `cloud-k8s`, `audit-chain`, `observability` |
| 3 | `workflow-events/apply.started` | `ApplyStartedPayload` | `ApplyStartedPayload` | `cloud-k8s`, `ops-dashboard-control-center`, `audit-chain` |
| 4 | `workflow-events/apply.completed` | `ApplyCompletedPayload` | `ApplyCompletedPayload` | `api-gateway`, `application`, `cloud-secrets`, `observability` |
| 5 | `workflow-events/apply.rolled_back` | `ApplyRolledBackPayload` | `ApplyRolledBackPayload` | `api-gateway`, `application`, `ops-dashboard-control-center` |
| 6 | `workflow-events/drift.detected` | `DriftDetectedPayload` | `DriftDetectedPayload` | `cell`, `api-gateway`, `ops-dashboard-control-center`, `audit-chain` |
| 7 | `workflow-events/microservice.registered` | `MicroserviceRegisteredPayload` | `MicroserviceRegisteredPayload` | `developer-sdk`, `observability`, `audit-chain` |
| 8 | `audit-chain /emit` | `CloudIacRenderRequested` | `AuditEvent.payload` with `render_id`, `microservice`, `sha`, `pack` | `audit-chain` |
| 9 | `audit-chain /emit` | `CloudIacApplyStarted` | `AuditEvent.payload` with `apply_id`, `microservice`, `environment`, `sha` | `audit-chain` |
| 10 | `audit-chain /emit` | `CloudIacApplyCompleted` | `AuditEvent.payload` with `apply_id`, `state`, `artifact_digest` | `audit-chain`, `compliance` |
| 11 | `audit-chain /emit` | `CloudIacDriftDetected` | `AuditEvent.payload` with `microservice`, `pack`, `environment`, `drift_score` | `audit-chain`, `ops-dashboard-control-center` |
| 12 | `observability.iac-drift` | `CloudIacDriftMetricRecorded` | inline `IacDriftMetric` | `observability` |
| 13 | `developer-sdk.submission-stage` | `CloudIacSubmissionStageEvaluated` | inline `VettingStageEvent` | `developer-sdk` |

## Synchronous vs Asynchronous Boundaries

| # | Boundary | Mode | Reasoning |
|---|---|---|---|
| 1 | `triggerRender` | synchronous admission, asynchronous render | caller needs render id immediately; heavy render runs out of band |
| 2 | `getRenderResult` | synchronous | callers poll by `render_id` and need exact digest |
| 3 | `planPreview` | synchronous | reviewers need plan summary before approving apply |
| 4 | `triggerApply` | synchronous admission, asynchronous apply | apply job id must be returned before Kubernetes mutation begins |
| 5 | `getApplyJob` | synchronous | operators need current apply state |
| 6 | `getApplyState` | synchronous | upstream services gate deploys on current sha |
| 7 | `triggerRollback` | synchronous admission, asynchronous rollback | rollback id and audit receipt must exist before action begins |
| 8 | `getDriftReport` | synchronous | audit and observability consumers need current drift facts |
| 9 | `getProvenance` | synchronous | artifact users must validate SLSA and signature before use |
| 10 | `validateChartSignature` | synchronous | caller must block on signature verdict |
| 11 | `render.completed` emission | asynchronous | deployment consumers can react after render commit |
| 12 | `apply.completed` emission | asynchronous | service caches can converge after deploy |
| 13 | `drift.detected` emission | asynchronous urgent | incident consumers react but render path should not deadlock |
| 14 | `audit-chain emitEvent` for apply | synchronous | infrastructure mutation is not durable until ADR-0263 receipt exists |
| 15 | `cloud-secrets` credential read | synchronous | signer and kubeconfig references are required before render/apply |

## Failure Mode Cascade

| # | Failure in `cloud-iac` | Upstream impact | Circuit breaker | Retry policy |
|---|---|---|---|---|
| 1 | render worker unavailable | `application`, `payments`, and `developer-sdk` deploys pause | `iac-render` breaker blocks new render jobs | retry render by `render_id` with same source digest |
| 2 | plan preview failure | reviewers cannot approve apply | `iac-plan-preview` breaker fails closed | no apply allowed until plan preview succeeds |
| 3 | apply admission failure | deployment pipeline stops | `iac-apply` breaker blocks apply requests | retry with same `apply_id` idempotency key |
| 4 | rollback admission failure | emergency rollback delayed | `iac-rollback` breaker raises incident | retry with same `rollback_id` |
| 5 | audit emit failure | infrastructure mutation cannot proceed | `iac-audit` breaker fails closed | retry 10 times, then hold in `cloud_iac.audit_pending` |
| 6 | cloud-secrets signer unavailable | render signatures cannot be produced | `gitops-signer` breaker blocks render completion | refetch versioned reference |
| 7 | cloud-k8s apply unavailable | apply job remains `waiting_for_cluster` | `k8s-apply` breaker prevents duplicate apply | retry Kubernetes apply with digest idempotency |
| 8 | drift detector unavailable | topology drift may go unreported | `drift-detector` breaker emits operator incident | retry scan by microservice pack |
| 9 | provenance read fails | consumers cannot trust artifact | `provenance` breaker returns 503 | caller retries with same digest |
| 10 | event bus unavailable | deploy consumers lag | outbox breaker spools lifecycle events | replay by `event_id` |
| 11 | DLQ saturation | deploy state convergence delayed | `iac-dlq` breaker pauses noncritical renders | manual replay after policy fix |
| 12 | microservice registry event missing | new service lacks chart entry | registry guard rejects apply | replay `microservice.registered` |

## Cross-tenant Coordination

| # | Scenario | Cedar guard pattern | Audit-mirror requirement |
|---|---|---|---|
| 1 | conglomerate deploy targets child tenant pack | `ci-scope.cedar` with active parent grant and child tenant resource | mirror `ConglomerateParentReadAction` and `CloudIacApplyStarted` to both partitions |
| 2 | apply crosses jurisdiction | `data-residency.md` guard `iac_pack_jurisdiction_allowed` | mirror `ConglomerateCrossJurisdictionResidencyEnforced` |
| 3 | office-scoped deploy overlay | `OfficePackOverlayActivated` guard with `sub_scope_path` | mirror `OfficePackOverlayChanged` on apply |
| 4 | information-barrier resource tag changes | `ci-scope.cedar` carries `barrier_tags` into plan | mirror `InformationBarrierTaintDerived` |
| 5 | personal tenant attempts deploy action | `ci-scope.cedar` forbids personal context | mirror `ConglomeratePersonalTenantBoundaryRefused` |

## Data Shape Ledger

| # | Shape | Source | Required handoff fields |
|---|---|---|---|
| 1 | `ChartSource` | `openapi/cloud-iac.yaml` | `microservice`, `chart_name`, `version`, `digest` |
| 2 | `RenderedManifest` | `openapi/cloud-iac.yaml` | `microservice`, `content_digest`, `rendered_bytes_count` |
| 3 | `PlanPreview` | `openapi/cloud-iac.yaml` | `microservice`, `plan_id`, `summary`, `changes` |
| 4 | `DriftReport` | `openapi/cloud-iac.yaml` | `microservice`, `pack`, `environment`, `drift_score`, `detected_at`, `drift_items` |
| 5 | `ApplyJob` | `openapi/cloud-iac.yaml` | `apply_id`, `microservice`, `pack`, `environment`, `sha`, `state` |
| 6 | `ApplyStateRecord` | `openapi/cloud-iac.yaml` | `microservice`, `pack`, `environment`, `current_sha`, `applied_at` |
| 7 | `Provenance` | `openapi/cloud-iac.yaml` | `artifact_digest`, `slsa_level`, `sigstore_signature`, `builder_id` |
| 8 | `RenderRequestedPayload` | `asyncapi/cloud-iac-events.yaml` | `render_id`, `microservice`, `sha`, `pack`, `environment`, `requested_at` |
| 9 | `ApplyStartedPayload` | `asyncapi/cloud-iac-events.yaml` | `apply_id`, `microservice`, `pack`, `environment`, `sha`, `started_at` |
| 10 | `SecretReference` | `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml` | `path`, `version`, `data_class`, `rotation_policy_id` |

## Cedar Guard Ledger

| # | Policy file | Principal | Action | Resource |
|---|---|---|---|---|
| 1 | `ci-scope.cedar` | `Service::cell` | `CloudIac::preview_cell_plan` | `Microservice::cell` |
| 2 | `ci-scope.cedar` | `Service::cell` | `CloudIac::apply_cell_manifest` | `Microservice::cell` |
| 3 | `ci-scope.cedar` | `Service::cloud-k8s` | `CloudIac::read_apply_state` | `ApplyState::{environment}` |
| 4 | `ci-scope.cedar` | `Service::cloud-iac` | `CloudK8s::apply_rendered_manifest` | `Cluster::{pack}/{environment}` |
| 5 | `ci-scope.cedar` | `Service::cloud-iac` | `CloudK8s::rollback_rendered_manifest` | `Cluster::{pack}/{environment}` |
| 6 | `public-read.cedar` | `Service::developer-sdk` | `CloudIac::validate_chart_signature` | `Chart::{digest}` |
| 7 | `public-read.cedar` | `Service::application` | `CloudIac::read_slsa_attestation_public` | `Artifact::{digest}` |
| 8 | `auditor-scope.cedar` | `Service::audit-chain` | `CloudIac::read_drift_report_for_audit` | `DriftReport::{pack}/{environment}` |
| 9 | `secret-isolation.md` | `Service::cloud-iac` | `CloudSecrets::read_reference` | `SecretReference::{tenant}/cloud-iac/*` |
| 10 | `iac-isolation.md` | `Service::cloud-iac` | `CloudIac::mutate_desired_state` | `Microservice::{microservice}` |

## Audit Event Class Ledger

| # | Audit class | Emitting handoff | ADR-0263 envelope fields that must be present |
|---|---|---|---|
| 1 | `CloudIacRenderRequested` | `triggerRender` | `render_id`, `microservice`, `sha`, `pack`, `audit_id` |
| 2 | `CloudIacRenderCompleted` | `render.completed` | `render_id`, `content_digest`, `slsa_level`, `audit_id` |
| 3 | `CloudIacPlanPreviewed` | `planPreview` | `plan_id`, `microservice`, `changes`, `audit_id` |
| 4 | `CloudIacApplyStarted` | `triggerApply` | `apply_id`, `microservice`, `environment`, `sha` |
| 5 | `CloudIacApplyCompleted` | `apply.completed` | `apply_id`, `state`, `artifact_digest`, `audit_id` |
| 6 | `CloudIacApplyRolledBack` | `apply.rolled_back` | `rollback_id`, `target_sha`, `environment`, `audit_id` |
| 7 | `CloudIacDriftDetected` | `drift.detected` | `microservice`, `pack`, `environment`, `drift_score` |
| 8 | `OfficePackOverlayActivated` | office scoped deploy overlay | `sub_scope_path`, `policy_pack`, `resource_ref`, `decision` |
| 9 | `InformationBarrierTaintDerived` | barrier tag propagation | `tenant_id`, `resource_ref`, `policy_fragment_id`, `audit_id` |

## Handoff Control Checklist

1. `triggerRender` must require a chart digest.
2. `triggerRender` must require a source sha.
3. `triggerRender` must create a stable `render_id`.
4. `triggerRender` must emit `CloudIacRenderRequested`.
5. Render workers must write `RenderedManifest.content_digest`.
6. Render workers must record `rendered_bytes_count`.
7. Render workers must not apply manifests directly.
8. `planPreview` must run before `triggerApply`.
9. `planPreview` must include drift-sensitive changes.
10. `planPreview` must include residency-sensitive changes.
11. `triggerApply` must require an approved plan id.
12. `triggerApply` must create an `apply_id`.
13. `triggerApply` must emit audit before Kubernetes mutation.
14. `triggerApply` must reject unsigned charts.
15. `triggerApply` must reject provenance below required SLSA level.
16. `getApplyJob` must be protected by `auditor-scope.cedar` or `ci-scope.cedar`.
17. `getApplyState` must never expose secret values.
18. `triggerRollback` must require a target sha.
19. `triggerRollback` must emit `CloudIacApplyRolledBack`.
20. `getDriftReport` must include `detected_at`.
21. `getDriftReport` must include `drift_items`.
22. Drift reports must be PII-free.
23. `getProvenance` must return `sigstore_signature`.
24. `getProvenance` must return `builder_id`.
25. `validateChartSignature` must block on signature verdict.
26. `cloud-secrets.secret.rotated` must reload signer version.
27. `cloud-secrets.secret.revoked` must block affected apply jobs.
28. `cell.lifecycle.transition` must refresh topology inputs.
29. `cell.decommissioned` must remove desired resources.
30. Developer submission events must be sandbox-scoped.
31. Audit seal minted handling must close apply receipts.
32. Gateway circuit events must pause affected gateway deploys.
33. Payment degradation events must affect only payments pack deploys.
34. `render.completed` must publish after content digest persists.
35. `apply.started` must publish after audit receipt.
36. `apply.completed` must publish after cluster acknowledgement.
37. `apply.rolled_back` must publish after rollback acknowledgement.
38. `drift.detected` must publish for nonzero drift score.
39. Microservice registry events must include chart name and digest.
40. `CloudIacRenderRequested` audit must include `source_microservice=cloud-iac`.
41. `CloudIacApplyStarted` audit must include `apply_id`.
42. `CloudIacDriftDetected` audit must include `drift_score`.
43. The render outbox must replay by `render_id`.
44. The apply outbox must replay by `apply_id`.
45. Rollback retries must preserve `rollback_id`.
46. Kubernetes apply retries must preserve manifest digest.
47. Kubernetes rollback retries must preserve target sha.
48. Drift detector retries must preserve scan window.
49. DLQ names must be channel-specific.
50. DLQ replay must not reorder apply lifecycle events.
51. Apply jobs must fail closed when audit-chain is unavailable.
52. Apply jobs must fail closed when signer secret is revoked.
53. Apply jobs must fail closed when residency check denies.
54. Public provenance reads must not reveal private deploy context.
55. Public signature validation must not reveal tenant ids.
56. Operator incidents must include microservice, pack, and environment.
57. Observability metrics must hash tenant references.
58. Cross-tenant deploys must mirror audit to parent and child tenants.
59. Office overlays must include `sub_scope_path`.
60. Information barrier taints must propagate to rendered manifests.
61. Personal context deploy requests must be refused.
62. Chart validation must reject unknown builder ids.
63. Chart validation must reject mismatched digest.
64. Apply state reads must include `current_sha`.
65. Apply state reads must include `applied_at`.
66. Drift report reads must include `environment`.
67. `cloud-iac` must attach `trace_id` to every audit event.
68. `cloud-iac` must attach `audit_id` to every mutating response.
69. `cloud-iac` must update this matrix when `cloud-iac.yaml` changes.
70. `cloud-iac` must update this matrix when `cloud-iac-events.yaml` changes.

## Checkpoint

- Authored for `cloud-iac` on 2026-05-20.
- Source contracts checked: `cloud-iac.yaml`, `cloud-iac-events.yaml`, and cloud-iac proto.
- Source policies checked: `iac-isolation.md`, `public-read.cedar`, `tenant-scope.cedar`, `ci-scope.cedar`, `auditor-scope.cedar`.
- No in-flight microservice directories were edited.
- GitOps change-bundle scope: `microservices`.
