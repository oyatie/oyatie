---
doc_class: APIReference
microservice: governance
version: 1.0.0
status: Accepted
date: 2026-05-20
owner: axis-governance + council-risk + ops-platform
openapi_version: 3.2.0
asyncapi_version: 3.1.0
proto3: true
---

# governance API Reference

Canonical REST, gRPC, and AsyncAPI reference for the `governance`
microservice. The service owns lane registry, lane-run evidence, findings,
evidence blobs, admission verdicts, baseline pins, conformance posture,
aggregation indexes, policy packs, and exportable governance bundles.

Contract status legend:

- `contract-bound`: implemented in the current OpenAPI, AsyncAPI, or proto3 file.
- `reference-planned`: canonical API surface derived from the PRD and SDK plan, pending contract promotion.

## Quick Start

Named example: `RegisterLaneRunAndExportEvidence`.

1. Register or inspect a lane with `GET /lanes` or `POST /lanes:register`.
2. Query lane runs and findings with `GET /lane-runs` and `GET /findings`.
3. Export an evidence bundle with `POST /evidence/export` and verify admission posture.

Minimum headers:

- `Authorization: Bearer <oidc-token>`
- `X-Tenant-Id: <uuid-v7>`
- `X-Governance-Pack: <pack-slug>`
- `Idempotency-Key: <ulid>` on mutating requests
- `X-Request-Id: <ulid>` for evidence correlation
- `Content-Type: application/json`

Example:

```http
GET /lanes HTTP/2
Host: governance.oyatie.dev
Authorization: Bearer eyJ...
X-Tenant-Id: 018f7a54-3ef5-7c42-a111-a2c4ad7f88f0
X-Governance-Pack: korea-base
X-Request-Id: 01HYGOVLANE000000000000
```

## Authentication & Authorization

Authentication patterns:

- OIDC bearer for tenant governance users and platform operators.
- SPIFFE SVID mTLS for CI, admission, audit-chain, and runtime lanes.
- Signed evidence blob URLs for bounded evidence download.
- Audit-chain proof binding for conformance and admission evidence.
- Webhook HMAC for repository, CI, and policy-pack callbacks.

Principal types:

- `GovernanceViewer`: read-only tenant or platform posture viewer.
- `LaneOperator`: principal allowed to register and dispatch governance lanes.
- `FindingTriageOwner`: principal allowed to update finding state and suppressions.
- `AdmissionController`: service principal allowed to request verdicts.
- `EvidenceExporter`: principal allowed to package and export evidence.
- `BaselineMaintainer`: operator allowed to refresh baseline pins.
- `PolicyPackMaintainer`: operator allowed to publish governance policy packs.
- `GovernanceAuditor`: read-only auditor with evidence and chain proof access.

Named Cedar policy patterns:

- `governance::tenant_scope_match`: tenant in token, request, and lane-run must match.
- `governance::lane_runtime_dispatch`: dispatch requires lane ownership or platform scope.
- `governance::finding_triage`: finding state mutation requires assigned triage authority.
- `governance::evidence_blob_read`: evidence blob read requires purpose and retention grant.
- `governance::admission_verdict_read`: admission verdict visibility is scoped to resource owner.
- `governance::baseline_pin_refresh`: baseline refresh requires maintainer role.
- `governance::policy_pack_publish`: policy pack mutation requires four-eyes approval.
- `governance::export_minimization`: export includes only approved evidence classes.

Authorization failure shape:

```json
{
  "error": {
    "code": "GOVERNANCE_AUTHZ_DENIED",
    "message": "Cedar policy denied governance action",
    "request_id": "01HYREQ...",
    "details": [{"policy": "governance::export_minimization"}]
  }
}
```

## REST Endpoints

### Lanes

#### GET /lanes

- Status: `contract-bound`.
- Operation: `listLanes`.
- Query schema: `tenant_id`, `pack`, `state`, `cursor`, `limit`.
- Response schema: `ListLanesResponse`.
- Status codes: `200`, `400`, `401`, `403`, `429`, `500`.
- Error shape: `GOVERNANCE_LANE_QUERY_INVALID`.

#### POST /lanes:register

- Status: `reference-planned`.
- Operation: `registerLane`.
- Request schema: `RegisterLaneRequest`.
- Required fields: `lane_id`, `owner`, `evidence_schema`, `policy_pack`.
- Response schema: `GovernanceLane`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `422`, `429`, `500`.
- Error shape: `GOVERNANCE_LANE_ALREADY_EXISTS`.

#### GET /lanes/{lane_id}

- Status: `reference-planned`.
- Operation: `getLane`.
- Path schema: `lane_id` as slug.
- Response schema: `GovernanceLane`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `GOVERNANCE_LANE_NOT_FOUND`.

#### POST /lanes/{lane_id}:dispatch

- Status: `reference-planned`.
- Operation: `dispatchLane`.
- Request schema: `DispatchLaneRequest`.
- Required fields: `subject_ref`, `trigger`, `policy_pack`.
- Response schema: `LaneRun`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `GOVERNANCE_LANE_DISPATCH_DENIED`.

### Lane Runs

#### GET /lane-runs

- Status: `contract-bound`.
- Operation: `listLaneRuns`.
- Query schema: `tenant_id`, `lane_id`, `state`, `from_time`, `to_time`, `cursor`, `limit`.
- Response schema: `ListLaneRunsResponse`.
- Status codes: `200`, `400`, `401`, `403`, `429`, `500`.
- Error shape: `GOVERNANCE_LANE_RUN_QUERY_INVALID`.

#### GET /lane-runs/{run_id}

- Status: `reference-planned`.
- Operation: `getLaneRun`.
- Path schema: `run_id` as UUID-v7.
- Response schema: `LaneRun`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `GOVERNANCE_LANE_RUN_NOT_FOUND`.

#### POST /lane-runs/{run_id}:rerun

- Status: `reference-planned`.
- Operation: `rerunLaneRun`.
- Request schema: `RerunLaneRunRequest`.
- Required fields: `reason`, `requested_by`, `preserve_evidence_refs`.
- Response schema: `LaneRun`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `429`, `500`.
- Error shape: `GOVERNANCE_LANE_RUN_NOT_RERUNNABLE`.

#### GET /lane-runs/{run_id}/events

- Status: `reference-planned`.
- Operation: `listLaneRunEvents`.
- Query schema: `cursor`, `limit`, `event_type`.
- Response schema: `ListLaneRunEventsResponse`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `GOVERNANCE_LANE_EVENT_CURSOR_INVALID`.

### Findings

#### GET /findings

- Status: `contract-bound`.
- Operation: `listFindings`.
- Query schema: `tenant_id`, `lane_id`, `severity`, `state`, `cursor`, `limit`.
- Response schema: `ListFindingsResponse`.
- Status codes: `200`, `400`, `401`, `403`, `429`, `500`.
- Error shape: `GOVERNANCE_FINDING_QUERY_INVALID`.

#### GET /findings/{finding_id}

- Status: `contract-bound`.
- Operation: `getFinding`.
- Path schema: `finding_id` as UUID-v7.
- Response schema: `GovernanceFinding`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `GOVERNANCE_FINDING_NOT_FOUND`.

#### POST /findings/{finding_id}:acknowledge

- Status: `reference-planned`.
- Operation: `acknowledgeFinding`.
- Request schema: `AcknowledgeFindingRequest`.
- Required fields: `acknowledged_by`, `reason`, `target_resolution_date`.
- Response schema: `GovernanceFinding`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `429`, `500`.
- Error shape: `GOVERNANCE_FINDING_STATE_CONFLICT`.

#### POST /findings/{finding_id}:suppress

- Status: `reference-planned`.
- Operation: `suppressFinding`.
- Request schema: `SuppressFindingRequest`.
- Required fields: `suppressed_by`, `reason`, `expires_at`.
- Response schema: `GovernanceFinding`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `GOVERNANCE_SUPPRESSION_DENIED`.

### Evidence

#### GET /evidence/{blob_id}

- Status: `contract-bound`.
- Operation: `getEvidenceBlob`.
- Path schema: `blob_id` as content-addressed id.
- Response schema: `EvidenceBlob`.
- Status codes: `200`, `401`, `403`, `404`, `410`, `429`, `500`.
- Error shape: `GOVERNANCE_EVIDENCE_NOT_FOUND`.

#### POST /evidence/export

- Status: `contract-bound`.
- Operation: `exportEvidence`.
- Request schema: `ExportEvidenceRequest`.
- Required fields: `tenant_id`, `purpose`, `filters`, `format`.
- Response schema: `EvidenceExportTicket`.
- Status codes: `202`, `400`, `401`, `403`, `409`, `422`, `429`, `500`.
- Error shape: `GOVERNANCE_EXPORT_DENIED`.

#### GET /evidence/export/{ticket_id}

- Status: `contract-bound`.
- Operation: `getEvidenceExport`.
- Path schema: `ticket_id` as UUID-v7.
- Response schema: `EvidenceExportStatus`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `GOVERNANCE_EXPORT_NOT_FOUND`.

#### POST /evidence/{blob_id}:verify

- Status: `reference-planned`.
- Operation: `verifyEvidenceBlob`.
- Request schema: `VerifyEvidenceBlobRequest`.
- Required fields: `expected_hash`, `audit_proof`.
- Response schema: `EvidenceVerificationReport`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `GOVERNANCE_EVIDENCE_VERIFICATION_FAILED`.

### Admission

#### GET /admission-verdict

- Status: `contract-bound`.
- Operation: `getAdmissionVerdict`.
- Query schema: `tenant_id`, `subject_ref`, `policy_pack`, `resource_kind`.
- Response schema: `AdmissionVerdict`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `GOVERNANCE_ADMISSION_VERDICT_NOT_FOUND`.

#### POST /admission-verdict:explain

- Status: `reference-planned`.
- Operation: `explainAdmissionVerdict`.
- Request schema: `ExplainAdmissionVerdictRequest`.
- Required fields: `subject_ref`, `policy_pack`, `decision_id`.
- Response schema: `AdmissionVerdictExplanation`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `GOVERNANCE_ADMISSION_EXPLANATION_DENIED`.

#### POST /admission-verdict:evaluate

- Status: `reference-planned`.
- Operation: `evaluateAdmissionVerdict`.
- Request schema: `EvaluateAdmissionVerdictRequest`.
- Required fields: `subject_ref`, `resource_kind`, `evidence_refs`, `policy_pack`.
- Response schema: `AdmissionVerdict`.
- Status codes: `200`, `400`, `401`, `403`, `409`, `422`, `429`, `500`.
- Error shape: `GOVERNANCE_ADMISSION_INPUT_INVALID`.

### Baseline Pins and Posture

#### GET /baseline-pins

- Status: `contract-bound`.
- Operation: `listBaselinePins`.
- Query schema: `tenant_id`, `pack`, `state`, `cursor`, `limit`.
- Response schema: `ListBaselinePinsResponse`.
- Status codes: `200`, `400`, `401`, `403`, `429`, `500`.
- Error shape: `GOVERNANCE_BASELINE_PIN_QUERY_INVALID`.

#### POST /baseline-pins/{pin_id}:refresh

- Status: `reference-planned`.
- Operation: `refreshBaselinePin`.
- Request schema: `RefreshBaselinePinRequest`.
- Required fields: `reason`, `source_evidence_refs`.
- Response schema: `BaselinePin`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `GOVERNANCE_BASELINE_REFRESH_DENIED`.

#### GET /conformance-posture

- Status: `contract-bound`.
- Operation: `getConformancePosture`.
- Query schema: `tenant_id`, `pack`, `resource_scope`.
- Response schema: `ConformancePosture`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `GOVERNANCE_POSTURE_NOT_FOUND`.

### Aggregation Indexes and Policy Packs

#### POST /aggregation-indexes:regenerate

- Status: `reference-planned`.
- Operation: `regenerateAggregationIndex`.
- Request schema: `RegenerateAggregationIndexRequest`.
- Required fields: `tenant_id`, `index_kind`, `reason`.
- Response schema: `AggregationIndexJob`.
- Status codes: `202`, `400`, `401`, `403`, `409`, `422`, `429`, `500`.
- Error shape: `GOVERNANCE_INDEX_REGENERATION_DENIED`.

#### GET /aggregation-indexes/{index_id}/diff

- Status: `reference-planned`.
- Operation: `getAggregationIndexDiff`.
- Path schema: `index_id` as UUID-v7.
- Query schema: `from_version`, `to_version`.
- Response schema: `AggregationIndexDiff`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `429`, `500`.
- Error shape: `GOVERNANCE_INDEX_DIFF_INVALID`.

#### GET /policy-packs

- Status: `reference-planned`.
- Operation: `listPolicyPacks`.
- Query schema: `jurisdiction`, `state`, `cursor`, `limit`.
- Response schema: `ListPolicyPacksResponse`.
- Status codes: `200`, `400`, `401`, `403`, `429`, `500`.
- Error shape: `GOVERNANCE_POLICY_PACK_QUERY_INVALID`.

#### GET /policy-packs/{pack_id}

- Status: `reference-planned`.
- Operation: `getPolicyPack`.
- Path schema: `pack_id` as slug.
- Response schema: `PolicyPack`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `GOVERNANCE_POLICY_PACK_NOT_FOUND`.

## gRPC Methods

### service GovernanceQuery

```proto
rpc ListLanes(ListLanesRequest) returns (ListLanesResponse);
```

- Status: `contract-bound`.
- Semantics: lists governance lanes by tenant and pack.
- Auth: `governance::tenant_scope_match`.
- Errors: `INVALID_ARGUMENT`, `PERMISSION_DENIED`.

```proto
rpc GetLaneRun(GetLaneRunRequest) returns (LaneRun);
```

- Status: `reference-planned`.
- Semantics: returns one lane-run record and evidence refs.
- Auth: `governance::tenant_scope_match`.
- Errors: `NOT_FOUND`, `PERMISSION_DENIED`.

```proto
rpc QueryLaneRuns(QueryLaneRunsRequest) returns (ListLaneRunsResponse);
```

- Status: `contract-bound`.
- Semantics: searches lane runs by lane, state, and time.
- Auth: `governance::tenant_scope_match`.
- Errors: `INVALID_ARGUMENT`, `PERMISSION_DENIED`.

```proto
rpc GetFinding(GetFindingRequest) returns (GovernanceFinding);
```

- Status: `contract-bound`.
- Semantics: returns one finding.
- Auth: `governance::finding_triage`.
- Errors: `NOT_FOUND`, `PERMISSION_DENIED`.

```proto
rpc QueryFindings(QueryFindingsRequest) returns (ListFindingsResponse);
```

- Status: `contract-bound`.
- Semantics: lists findings by severity, state, and lane.
- Auth: `governance::finding_triage`.
- Errors: `INVALID_ARGUMENT`, `PERMISSION_DENIED`.

```proto
rpc QueryAdmissionVerdict(QueryAdmissionVerdictRequest) returns (AdmissionVerdict);
```

- Status: `contract-bound`.
- Semantics: returns admission decision for subject and policy pack.
- Auth: `governance::admission_verdict_read`.
- Errors: `NOT_FOUND`, `FAILED_PRECONDITION`.

```proto
rpc GetConformancePosture(GetConformancePostureRequest) returns (ConformancePosture);
```

- Status: `contract-bound`.
- Semantics: returns posture across lanes and policy packs.
- Auth: `governance::tenant_scope_match`.
- Errors: `NOT_FOUND`, `PERMISSION_DENIED`.

```proto
rpc ListBaselinePins(ListBaselinePinsRequest) returns (ListBaselinePinsResponse);
```

- Status: `contract-bound`.
- Semantics: lists baseline pins for pack and tenant.
- Auth: `governance::baseline_pin_refresh`.
- Errors: `INVALID_ARGUMENT`, `PERMISSION_DENIED`.

### service GovernanceEvidence

```proto
rpc GetEvidenceBlob(GetEvidenceBlobRequest) returns (EvidenceBlob);
```

- Status: `contract-bound`.
- Semantics: returns or redirects to evidence blob content.
- Auth: `governance::evidence_blob_read`.
- Errors: `NOT_FOUND`, `PERMISSION_DENIED`.

```proto
rpc ExportEvidenceBundle(ExportEvidenceRequest) returns (EvidenceExportTicket);
```

- Status: `contract-bound`.
- Semantics: starts minimized export bundle generation.
- Auth: `governance::export_minimization`.
- Errors: `FAILED_PRECONDITION`, `RESOURCE_EXHAUSTED`.

```proto
rpc GetEvidenceExport(GetEvidenceExportRequest) returns (EvidenceExportStatus);
```

- Status: `contract-bound`.
- Semantics: returns export job status.
- Auth: `governance::export_minimization`.
- Errors: `NOT_FOUND`, `PERMISSION_DENIED`.

```proto
rpc VerifySignature(VerifyEvidenceBlobRequest) returns (EvidenceVerificationReport);
```

- Status: `reference-planned`.
- Semantics: verifies evidence hash, signature, and audit proof.
- Auth: `governance::evidence_blob_read`.
- Errors: `INVALID_ARGUMENT`, `FAILED_PRECONDITION`.

### service GovernanceLaneRuntime

```proto
rpc RegisterLane(RegisterLaneRequest) returns (GovernanceLane);
```

- Status: `reference-planned`.
- Semantics: registers a governance lane and evidence schema.
- Auth: `governance::lane_runtime_dispatch`.
- Errors: `ALREADY_EXISTS`, `INVALID_ARGUMENT`.

```proto
rpc DispatchLane(DispatchLaneRequest) returns (LaneRun);
```

- Status: `reference-planned`.
- Semantics: schedules lane evaluation for a subject.
- Auth: `governance::lane_runtime_dispatch`.
- Errors: `FAILED_PRECONDITION`, `RESOURCE_EXHAUSTED`.

### service GovernancePolicy

```proto
rpc ExplainAdmissionVerdict(ExplainAdmissionVerdictRequest) returns (AdmissionVerdictExplanation);
```

- Status: `reference-planned`.
- Semantics: explains one admission verdict and evidence inputs.
- Auth: `governance::admission_verdict_read`.
- Errors: `NOT_FOUND`, `PERMISSION_DENIED`.

## AsyncAPI Channels

### governance.lane-failed

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `LaneFailed`.
- Delivery semantics: at-least-once, partitioned by `tenant_id`.
- Consumers: messenger, audit-chain, platform operations.

### governance.finding-emitted

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `FindingEmitted`.
- Delivery semantics: at-least-once with duplicate suppression by `finding_id`.
- Consumers: messenger, audit-chain, issue triage.

### governance.audit-completed

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `AuditCompleted`.
- Delivery semantics: ordered per `lane_id` and `run_id`.
- Consumers: audit-chain, compliance dashboards.

### governance.baseline-pin-updated

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `BaselinePinUpdated`.
- Delivery semantics: compacted by `pin_id`.
- Consumers: admission controller, audit-chain.

### governance.aggregation-index-regenerated

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `AggregationIndexRegenerated`.
- Delivery semantics: at-least-once.
- Consumers: conformance dashboard, admission controller.

### governance.pull-request-opened

- Direction: subscribe.
- Status: `contract-bound`.
- Payload schema: `PullRequestOpened`.
- Delivery semantics: at-least-once from repository provider.
- Handler: dispatch relevant lanes.

### governance.microservice-registered

- Direction: subscribe.
- Status: `contract-bound`.
- Payload schema: `MicroserviceRegistered`.
- Delivery semantics: compacted by `microservice`.
- Handler: update lane registry and conformance scopes.

### governance.policy-pack-published

- Direction: publish.
- Status: `reference-planned`.
- Payload schema: `PolicyPackPublished`.
- Delivery semantics: ordered per `pack_id`.
- Consumers: admission controller, workflow-studio, intelligence.

### governance.evidence-export-completed

- Direction: publish.
- Status: `reference-planned`.
- Payload schema: `EvidenceExportCompleted`.
- Delivery semantics: at-least-once with signed manifest ref.
- Consumers: compliance portal, audit-chain.

### audit-chain.verification-failed

- Direction: subscribe.
- Status: `reference-planned`.
- Payload schema: `VerificationFailed`.
- Delivery semantics: at-least-once.
- Handler: emits critical governance finding.

### ci.workflow-completed

- Direction: subscribe.
- Status: `reference-planned`.
- Payload schema: `CiWorkflowCompleted`.
- Delivery semantics: at-least-once.
- Handler: updates lane-run evidence.

### repository.branch-protected

- Direction: subscribe.
- Status: `reference-planned`.
- Payload schema: `BranchProtected`.
- Delivery semantics: compacted by repository and branch.
- Handler: refreshes baseline pins.

## Webhooks Inbound

### webhook.repository.pull-request-opened

- Source: Git provider.
- Event: `pull_request.opened`.
- Payload schema: `PullRequestOpenedWebhook`.
- Semantics: dispatches configured PR governance lanes.

### webhook.repository.pull-request-updated

- Source: Git provider.
- Event: `pull_request.synchronize`.
- Payload schema: `PullRequestUpdatedWebhook`.
- Semantics: reruns stale lanes and invalidates admission verdicts.

### webhook.ci.workflow-completed

- Source: CI provider.
- Event: `workflow.completed`.
- Payload schema: `CiWorkflowCompletedWebhook`.
- Semantics: attaches CI evidence to lane-run records.

### webhook.audit-chain.seal-minted

- Source: audit-chain.
- Event: `audit_chain.seal_minted`.
- Payload schema: `SealMintedWebhook`.
- Semantics: updates evidence chain proof references.

### webhook.audit-chain.verification-failed

- Source: audit-chain.
- Event: `audit_chain.verification_failed`.
- Payload schema: `VerificationFailedWebhook`.
- Semantics: emits critical governance finding.

### webhook.policy-pack.published

- Source: policy-pack publisher.
- Event: `policy_pack.published`.
- Payload schema: `PolicyPackPublishedWebhook`.
- Semantics: refreshes policy pack and lane mappings.

### webhook.microservice.registered

- Source: service catalog.
- Event: `microservice.registered`.
- Payload schema: `MicroserviceRegisteredWebhook`.
- Semantics: creates governance scope for the service.

### webhook.baseline.changed

- Source: baseline controller.
- Event: `baseline.changed`.
- Payload schema: `BaselineChangedWebhook`.
- Semantics: refreshes affected baseline pins.

## SDK Quick Reference

### Rust

```rust
let lanes = governance::list_lanes(client, query).await?;
let runs = governance::list_lane_runs(client, run_query).await?;
let finding = governance::get_finding(client, finding_id).await?;
let verdict = governance::get_admission_verdict(client, verdict_query).await?;
let export = governance::export_evidence_bundle(client, export_request).await?;
```

Named functions:

- `list_lanes`
- `register_lane`
- `dispatch_lane`
- `list_lane_runs`
- `get_lane_run`
- `list_findings`
- `get_finding`
- `suppress_finding`
- `get_evidence_blob`
- `export_evidence_bundle`
- `get_admission_verdict`
- `get_conformance_posture`
- `list_baseline_pins`

### TypeScript

```ts
const governance = new GovernanceClient({ tenantId, token });
const lanes = await governance.listLanes({ pack });
const runs = await governance.listLaneRuns({ laneId, state: "failed" });
const verdict = await governance.getAdmissionVerdict({ subjectRef, policyPack });
await governance.exportEvidenceBundle({ purpose, filters, format: "zip" });
```

Named functions:

- `listLanes`
- `registerLane`
- `dispatchLane`
- `listLaneRuns`
- `getLaneRun`
- `listFindings`
- `getFinding`
- `suppressFinding`
- `getEvidenceBlob`
- `exportEvidenceBundle`
- `getAdmissionVerdict`
- `getConformancePosture`
- `listBaselinePins`

### Python

```python
governance = GovernanceClient(tenant_id=tenant_id, token=token)
lanes = governance.list_lanes(pack=pack)
runs = governance.list_lane_runs(lane_id=lane_id, state="failed")
finding = governance.get_finding(finding_id)
verdict = governance.get_admission_verdict(subject_ref=subject_ref, policy_pack=pack)
ticket = governance.export_evidence_bundle(purpose=purpose, filters=filters)
```

Named functions:

- `list_lanes`
- `register_lane`
- `dispatch_lane`
- `list_lane_runs`
- `get_lane_run`
- `list_findings`
- `get_finding`
- `suppress_finding`
- `get_evidence_blob`
- `export_evidence_bundle`
- `get_admission_verdict`
- `get_conformance_posture`
- `list_baseline_pins`

## Error Catalogue

### GOVERNANCE_AUTHZ_DENIED

- Meaning: Cedar denied governance operation.
- Retry policy: do not retry without changing scope or authority.
- HTTP mapping: `403`.
- gRPC mapping: `PERMISSION_DENIED`.

### GOVERNANCE_LANE_ALREADY_EXISTS

- Meaning: lane id already exists in the registry.
- Retry policy: fetch existing lane or choose a new id.
- HTTP mapping: `409`.
- gRPC mapping: `ALREADY_EXISTS`.

### GOVERNANCE_LANE_DISPATCH_DENIED

- Meaning: lane cannot be dispatched for subject or policy pack.
- Retry policy: do not retry until scope, pack, or evidence changes.
- HTTP mapping: `403`.
- gRPC mapping: `PERMISSION_DENIED`.

### GOVERNANCE_LANE_RUN_NOT_RERUNNABLE

- Meaning: lane run is not in a state that supports rerun.
- Retry policy: create a new dispatch or wait for terminal state.
- HTTP mapping: `409`.
- gRPC mapping: `FAILED_PRECONDITION`.

### GOVERNANCE_FINDING_STATE_CONFLICT

- Meaning: finding state changed since caller read it.
- Retry policy: reload and retry with current state token.
- HTTP mapping: `409`.
- gRPC mapping: `ABORTED`.

### GOVERNANCE_SUPPRESSION_DENIED

- Meaning: suppression violates policy, expiry, or approval requirement.
- Retry policy: retry only after approval or policy changes.
- HTTP mapping: `403`.
- gRPC mapping: `PERMISSION_DENIED`.

### GOVERNANCE_EVIDENCE_VERIFICATION_FAILED

- Meaning: evidence hash, signature, or audit proof failed verification.
- Retry policy: do not retry; escalate as evidence integrity issue.
- HTTP mapping: `422`.
- gRPC mapping: `FAILED_PRECONDITION`.

### GOVERNANCE_ADMISSION_INPUT_INVALID

- Meaning: admission request lacks required subject, evidence, or pack data.
- Retry policy: correct request and retry.
- HTTP mapping: `400`.
- gRPC mapping: `INVALID_ARGUMENT`.

### GOVERNANCE_EXPORT_DENIED

- Meaning: evidence export violates minimization, purpose, or approval policy.
- Retry policy: do not retry without changing export scope or approval state.
- HTTP mapping: `403`.
- gRPC mapping: `PERMISSION_DENIED`.

### GOVERNANCE_RATE_LIMITED

- Meaning: capability-tier quota was exceeded.
- Retry policy: honor `Retry-After`; clients should use token bucket pacing.
- HTTP mapping: `429`.
- gRPC mapping: `RESOURCE_EXHAUSTED`.

## Pagination

Cursor pattern name: `governance_evidence_cursor`.

Cursor fields:

- `tenant_id`
- `pack`
- `resource_kind`
- `lane_id`
- `state_partition`
- `sort_key`
- `last_seen_id`
- `issued_at`
- `signature`

Rules:

- Cursor values are opaque and signed.
- Lane runs sort by `started_at` descending.
- Findings sort by severity, state, then `created_at`.
- Evidence export views sort by `created_at`.
- Cursor TTL is 15 minutes for live views and 24 hours for export status views.
- Invalid cursors return `GOVERNANCE_CURSOR_INVALID`.

Max page-size limits:

- Lanes: `200`.
- Lane runs: `500`.
- Lane-run events: `500`.
- Findings: `500`.
- Baseline pins: `200`.
- Policy packs: `200`.
- Aggregation diffs: `500`.
- Default page size: `100`.

## Rate Limits per Tier

Per ADR-0316, governance uses capability-tier throttles rather than
product-fragmented limits.

| Tier | REST requests per second | gRPC requests per second | Async publishes per second | Burst |
| --- | ---: | ---: | ---: | ---: |
| Bronze | 30 | 60 | 50 | 2x for 10s |
| Silver | 100 | 200 | 200 | 2x for 20s |
| Gold | 300 | 600 | 800 | 3x for 30s |
| Platinum | 900 | 1800 | 3000 | 3x for 60s |

Special limits:

- Lane dispatch: Bronze `5 rps`, Silver `20`, Gold `80`, Platinum `250`.
- Evidence export: Bronze `1 rps`, Silver `3`, Gold `10`, Platinum `30`.
- Admission evaluation: Bronze `25 rps`, Silver `100`, Gold `400`, Platinum `1200`.
- Aggregation regeneration: Bronze `1 concurrent`, Silver `3`, Gold `10`, Platinum `30`.

## OpenAPI 3.2.0 Schema

Actual contracts file:

- [governance.yaml](../../microservices/governance/contracts/openapi/governance.yaml)

Design references:

- [governance PRD](../../microservices/governance/PRD.md)
- [governance SDK plan](../../microservices/governance/sdk-plan.md)
- [API design standard](../standards/api-design.md)

## AsyncAPI 3.1.0 Schema

Actual contracts file:

- [governance-events.yaml](../../microservices/governance/contracts/asyncapi/governance-events.yaml)

Delivery notes:

- Finding events are at-least-once and deduplicated by `finding_id`.
- Baseline pin events may be compacted.
- Lane events are ordered per `run_id`.
- Consumers must verify evidence hashes before acting on export events.

## proto3 Schema

Actual contracts file:

- [governance.proto](../../microservices/governance/contracts/proto/governance.proto)

Proto package expectations:

- Use proto3 syntax.
- Map denied verdict access to `PERMISSION_DENIED`.
- Map stale finding state changes to `ABORTED`.
- Evidence payloads should use references for large blobs.

## Cross-References

- [governance PRD](../../microservices/governance/PRD.md)
- [governance SDK plan](../../microservices/governance/sdk-plan.md)
- [ADR-0316 capability tier over product fragmentation](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md)
- [API design standard](../standards/api-design.md)
- [Throttling tiers](../standards/throttling-tiers.md)
- [Audit-chain API reference](audit-chain-api-reference.md)
- [Workflow Studio API reference](workflow-studio-api-reference.md)
- [Intelligence API reference](intelligence-api-reference.md)
- [Ontology API reference](ontology-api-reference.md)
