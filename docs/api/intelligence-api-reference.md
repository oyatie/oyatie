---
doc_class: APIReference
microservice: intelligence
version: 1.0.0-mvp
status: Proposed
date: 2026-05-20
owner: axis-intelligence + council-safety + ops-model-routing
openapi_version: 3.2.0
asyncapi_version: 3.1.0
proto3: true
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# intelligence API Reference

Canonical REST, gRPC, and AsyncAPI reference for the `intelligence`
microservice. The service owns assist drafts, retrieval binding, provider
routing, guardrail evaluation, refusal records, attribution, evaluations,
multimodal processing, credential resolution, and audit-tap evidence.

Contract status legend:

- `contract-bound`: implemented in the current OpenAPI, AsyncAPI, or proto3 file.
- `reference-planned`: canonical API surface derived from the PRD, architecture, and SDK plan, pending contract promotion.

## Quick Start

Named example: `RetrieveDraftAndAudit`.

1. Bind retrieval context with `POST /retrievals`.
2. Request a safe draft with `POST /assist-drafts`.
3. Subscribe to `intelligence.assist-draft-completed` and verify audit tap sealing.

Minimum headers:

- `Authorization: Bearer <oidc-token>`
- `X-Tenant-Id: <uuid-v7>`
- `X-Context-Kind: Personal | Professional`
- `Idempotency-Key: <ulid>` on mutating requests
- `X-Request-Id: <ulid>` for trace and audit
- `Content-Type: application/json`

Example:

```http
POST /assist-drafts HTTP/2
Host: intelligence.oyatie.dev
Authorization: Bearer eyJ...
X-Tenant-Id: 018f7a54-3ef5-7c42-a111-a2c4ad7f88f0
X-Context-Kind: Professional
Idempotency-Key: 01HYINTDRAFT000000000000
Content-Type: application/json
```

## Authentication & Authorization

Authentication patterns:

- OIDC bearer for user and product initiated assist requests.
- SPIFFE SVID mTLS for internal workflow-studio, messenger, drive, and governance callers.
- OpenBao SecretReference for provider-BYOK credential access.
- Provider response signatures where the upstream model vendor supports them.
- Audit-chain signed evidence envelopes for high-risk prompts and refusals.

Principal types:

- `IntelligenceEndUser`: human principal requesting assist or retrieval.
- `WorkflowAssistCaller`: workflow-studio delegated caller for graph drafting.
- `MessengerAssistCaller`: messenger delegated caller for thread summaries and drafts.
- `DriveRetrievalCaller`: drive delegated caller for document-grounded context.
- `SafetyReviewer`: review principal for refusals, evals, and guardrail decisions.
- `ProviderRouter`: internal routing worker with provider-health visibility.
- `CredentialBroker`: service principal allowed to resolve provider secrets.
- `IntelligenceAuditor`: read-only principal for evidence, attribution, and audit tap.

Named Cedar policy patterns:

- `intelligence::tenant_scope_match`: tenant in token, request, and provider route must match.
- `intelligence::context_isolation`: Personal and Professional context cannot co-resolve.
- `intelligence::retrieval_data_boundary`: retrieval sources must match declared data class.
- `intelligence::assist_prompt_redaction`: blocked fields are redacted before provider dispatch.
- `intelligence::provider_route_allowed`: provider, model, region, and pack must be allowed.
- `intelligence::guardrail_refusal_required`: unsafe content must produce refusal evidence.
- `intelligence::credential_reference_use`: SecretReference can be resolved only by broker.
- `intelligence::human_review_high_risk`: high-risk output requires reviewer or hold.

Authorization failure shape:

```json
{
  "error": {
    "code": "INTELLIGENCE_AUTHZ_DENIED",
    "message": "Cedar policy denied intelligence action",
    "request_id": "01HYREQ...",
    "details": [{"policy": "intelligence::provider_route_allowed"}]
  }
}
```

## REST Endpoints

### Assist Drafts

#### POST /assist-drafts

- Status: `contract-bound`.
- Operation: `createAssistDraft`.
- Request schema: `CreateAssistDraftRequest`.
- Required fields: `tenant_id`, `caller`, `instruction`, `context_refs`, `safety_profile`.
- Response schema: `AssistDraft`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `422`, `429`, `500`, `503`.
- Error shape: `INTELLIGENCE_ASSIST_REFUSED`.

#### GET /assist-drafts/{draft_id}

- Status: `reference-planned`.
- Operation: `getAssistDraft`.
- Path schema: `draft_id` as UUID-v7.
- Response schema: `AssistDraft`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `INTELLIGENCE_DRAFT_NOT_FOUND`.

#### POST /assist-drafts/{draft_id}:accept

- Status: `reference-planned`.
- Operation: `acceptAssistDraft`.
- Request schema: `AcceptAssistDraftRequest`.
- Required fields: `accepted_by`, `target_resource`, `edits_applied`.
- Response schema: `AssistAcceptanceReceipt`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `429`, `500`.
- Error shape: `INTELLIGENCE_ACCEPTANCE_REJECTED`.

#### POST /assist-drafts/{draft_id}:reject

- Status: `reference-planned`.
- Operation: `rejectAssistDraft`.
- Request schema: `RejectAssistDraftRequest`.
- Required fields: `rejected_by`, `reason_code`.
- Response schema: `AssistRejectionReceipt`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `429`, `500`.
- Error shape: `INTELLIGENCE_REJECTION_RECORDED`.

### Retrievals

#### POST /retrievals

- Status: `contract-bound`.
- Operation: `createRetrieval`.
- Request schema: `CreateRetrievalRequest`.
- Required fields: `tenant_id`, `query`, `source_refs`, `data_boundary`.
- Response schema: `RetrievalContext`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `422`, `429`, `500`, `503`.
- Error shape: `INTELLIGENCE_RETRIEVAL_BOUNDARY_DENIED`.

#### GET /retrievals/{retrieval_id}

- Status: `reference-planned`.
- Operation: `getRetrieval`.
- Path schema: `retrieval_id` as UUID-v7.
- Response schema: `RetrievalContext`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `INTELLIGENCE_RETRIEVAL_NOT_FOUND`.

#### POST /retrievals/{retrieval_id}:rebind

- Status: `reference-planned`.
- Operation: `rebindRetrieval`.
- Request schema: `RebindRetrievalRequest`.
- Required fields: `source_refs`, `expected_version`, `data_boundary`.
- Response schema: `RetrievalContext`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `INTELLIGENCE_RETRIEVAL_VERSION_CONFLICT`.

### Dispatches

#### POST /dispatches

- Status: `reference-planned`.
- Operation: `createDispatch`.
- Request schema: `CreateDispatchRequest`.
- Required fields: `tenant_id`, `prompt_envelope`, `route_constraints`, `safety_profile`.
- Response schema: `DispatchReceipt`.
- Status codes: `202`, `400`, `401`, `403`, `409`, `422`, `429`, `500`, `503`.
- Error shape: `INTELLIGENCE_PROVIDER_UNAVAILABLE`.

#### GET /dispatches/{dispatch_id}

- Status: `reference-planned`.
- Operation: `getDispatch`.
- Path schema: `dispatch_id` as UUID-v7.
- Response schema: `DispatchReceipt`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `INTELLIGENCE_DISPATCH_NOT_FOUND`.

#### GET /dispatches/{dispatch_id}/stream

- Status: `reference-planned`.
- Operation: `streamDispatch`.
- Path schema: `dispatch_id` as UUID-v7.
- Response schema: `text/event-stream` of `DispatchChunk`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `429`, `500`, `503`.
- Error shape: `INTELLIGENCE_STREAM_EXPIRED`.

#### POST /dispatches/{dispatch_id}:cancel

- Status: `reference-planned`.
- Operation: `cancelDispatch`.
- Request schema: `CancelDispatchRequest`.
- Required fields: `reason`, `cancelled_by`.
- Response schema: `DispatchCancellationReceipt`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `429`, `500`.
- Error shape: `INTELLIGENCE_DISPATCH_ALREADY_FINAL`.

### Provider Routing

#### POST /providers/routes:evaluate

- Status: `reference-planned`.
- Operation: `evaluateProviderRoute`.
- Request schema: `EvaluateProviderRouteRequest`.
- Required fields: `capability`, `jurisdiction`, `latency_class`, `data_class`.
- Response schema: `ProviderRouteDecision`.
- Status codes: `200`, `400`, `401`, `403`, `422`, `429`, `500`, `503`.
- Error shape: `INTELLIGENCE_NO_PROVIDER_ROUTE`.

#### GET /providers/{provider_id}/health

- Status: `reference-planned`.
- Operation: `getProviderHealth`.
- Path schema: `provider_id` as slug.
- Response schema: `ProviderHealth`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`, `503`.
- Error shape: `INTELLIGENCE_PROVIDER_NOT_FOUND`.

#### GET /providers

- Status: `reference-planned`.
- Operation: `listProviders`.
- Query schema: `capability`, `region`, `data_class`, `cursor`, `limit`.
- Response schema: `ListProvidersResponse`.
- Status codes: `200`, `400`, `401`, `403`, `429`, `500`.
- Error shape: `INTELLIGENCE_PROVIDER_QUERY_INVALID`.

### Guardrails

#### POST /guardrails:evaluate

- Status: `reference-planned`.
- Operation: `evaluateGuardrails`.
- Request schema: `EvaluateGuardrailsRequest`.
- Required fields: `input`, `output_candidate`, `policy_pack`, `context_kind`.
- Response schema: `GuardrailDecision`.
- Status codes: `200`, `400`, `401`, `403`, `422`, `429`, `500`.
- Error shape: `INTELLIGENCE_GUARDRAIL_INPUT_INVALID`.

#### GET /guardrails/refusals

- Status: `reference-planned`.
- Operation: `listRefusals`.
- Query schema: `tenant_id`, `caller`, `reason_code`, `cursor`, `limit`.
- Response schema: `ListRefusalsResponse`.
- Status codes: `200`, `400`, `401`, `403`, `429`, `500`.
- Error shape: `INTELLIGENCE_REFUSAL_QUERY_INVALID`.

#### GET /guardrails/refusals/{refusal_id}

- Status: `reference-planned`.
- Operation: `getRefusal`.
- Path schema: `refusal_id` as UUID-v7.
- Response schema: `RefusalRecord`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `INTELLIGENCE_REFUSAL_NOT_FOUND`.

### Attribution and Evaluations

#### POST /attributions

- Status: `reference-planned`.
- Operation: `createAttribution`.
- Request schema: `CreateAttributionRequest`.
- Required fields: `draft_id`, `source_refs`, `claim_spans`.
- Response schema: `AttributionReport`.
- Status codes: `201`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `INTELLIGENCE_ATTRIBUTION_FAILED`.

#### POST /eval-runs

- Status: `reference-planned`.
- Operation: `createEvalRun`.
- Request schema: `CreateEvalRunRequest`.
- Required fields: `eval_suite`, `candidate_route`, `sample_set`.
- Response schema: `EvalRun`.
- Status codes: `202`, `400`, `401`, `403`, `409`, `422`, `429`, `500`.
- Error shape: `INTELLIGENCE_EVAL_SUITE_INVALID`.

#### GET /eval-runs/{eval_run_id}

- Status: `reference-planned`.
- Operation: `getEvalRun`.
- Path schema: `eval_run_id` as UUID-v7.
- Response schema: `EvalRun`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `INTELLIGENCE_EVAL_RUN_NOT_FOUND`.

### Credentials

#### POST /credentials:resolve

- Status: `reference-planned`.
- Operation: `resolveCredential`.
- Request schema: `ResolveCredentialRequest`.
- Required fields: `provider_id`, `secret_reference`, `purpose`.
- Response schema: `ResolvedCredentialLease`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `429`, `500`.
- Error shape: `INTELLIGENCE_CREDENTIAL_DENIED`.

#### POST /credentials:rotate

- Status: `reference-planned`.
- Operation: `rotateCredential`.
- Request schema: `RotateCredentialRequest`.
- Required fields: `provider_id`, `secret_reference`, `rotation_reason`.
- Response schema: `CredentialRotationReceipt`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `429`, `500`.
- Error shape: `INTELLIGENCE_CREDENTIAL_ROTATION_FAILED`.

### Multimodal and Audit Tap

#### POST /multimodal/transcriptions

- Status: `reference-planned`.
- Operation: `createTranscription`.
- Request schema: `CreateTranscriptionRequest`.
- Required fields: `media_ref`, `language_hint`, `safety_profile`.
- Response schema: `TranscriptionJob`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`, `503`.
- Error shape: `INTELLIGENCE_TRANSCRIPTION_FAILED`.

#### POST /multimodal/summaries

- Status: `reference-planned`.
- Operation: `createMultimodalSummary`.
- Request schema: `CreateMultimodalSummaryRequest`.
- Required fields: `media_refs`, `summary_profile`, `data_boundary`.
- Response schema: `MultimodalSummary`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`, `503`.
- Error shape: `INTELLIGENCE_SUMMARY_FAILED`.

#### GET /audit-tap/records

- Status: `reference-planned`.
- Operation: `listAuditTapRecords`.
- Query schema: `tenant_id`, `caller`, `risk_level`, `cursor`, `limit`.
- Response schema: `ListAuditTapRecordsResponse`.
- Status codes: `200`, `400`, `401`, `403`, `429`, `500`.
- Error shape: `INTELLIGENCE_AUDIT_TAP_QUERY_INVALID`.

### Health

#### GET /health

- Status: `reference-planned`.
- Operation: `health`.
- Response schema: `HealthStatus`.
- Status codes: `200`, `500`.
- Error shape: standard health probe failure.

#### GET /ready

- Status: `reference-planned`.
- Operation: `ready`.
- Response schema: `ReadinessStatus`.
- Status codes: `200`, `503`.
- Error shape: `INTELLIGENCE_PROVIDER_POOL_UNREADY`.

## gRPC Methods

### service IntelligenceAssist

```proto
rpc CreateAssistDraft(CreateAssistDraftRequest) returns (AssistDraft);
```

- Status: `contract-bound`.
- Semantics: generates a safe draft from instruction and context refs.
- Auth: `intelligence::assist_prompt_redaction`.
- Errors: `FAILED_PRECONDITION`, `RESOURCE_EXHAUSTED`, `UNAVAILABLE`.

```proto
rpc GetAssistDraft(GetAssistDraftRequest) returns (AssistDraft);
```

- Status: `reference-planned`.
- Semantics: returns a draft and safety metadata.
- Auth: `intelligence::tenant_scope_match`.
- Errors: `NOT_FOUND`, `PERMISSION_DENIED`.

```proto
rpc AcceptAssistDraft(AcceptAssistDraftRequest) returns (AssistAcceptanceReceipt);
```

- Status: `reference-planned`.
- Semantics: records human or workflow acceptance.
- Auth: `intelligence::human_review_high_risk`.
- Errors: `FAILED_PRECONDITION`, `NOT_FOUND`.

### service IntelligenceRetrieval

```proto
rpc CreateRetrieval(CreateRetrievalRequest) returns (RetrievalContext);
```

- Status: `contract-bound`.
- Semantics: binds retrieved context to a data boundary and source set.
- Auth: `intelligence::retrieval_data_boundary`.
- Errors: `INVALID_ARGUMENT`, `PERMISSION_DENIED`, `UNAVAILABLE`.

```proto
rpc GetRetrieval(GetRetrievalRequest) returns (RetrievalContext);
```

- Status: `reference-planned`.
- Semantics: returns retrieval metadata and evidence references.
- Auth: `intelligence::retrieval_data_boundary`.
- Errors: `NOT_FOUND`, `PERMISSION_DENIED`.

### service IntelligenceDispatch

```proto
rpc CreateDispatch(CreateDispatchRequest) returns (DispatchReceipt);
```

- Status: `reference-planned`.
- Semantics: dispatches a prompt envelope to an allowed provider route.
- Auth: `intelligence::provider_route_allowed`.
- Errors: `FAILED_PRECONDITION`, `UNAVAILABLE`, `RESOURCE_EXHAUSTED`.

```proto
rpc StreamDispatch(StreamDispatchRequest) returns (stream DispatchChunk);
```

- Status: `reference-planned`.
- Semantics: streams provider output chunks through guardrails.
- Auth: `intelligence::provider_route_allowed`.
- Errors: `UNAVAILABLE`, `OUT_OF_RANGE`.

```proto
rpc CancelDispatch(CancelDispatchRequest) returns (DispatchCancellationReceipt);
```

- Status: `reference-planned`.
- Semantics: cancels in-flight provider work and records evidence.
- Auth: `intelligence::tenant_scope_match`.
- Errors: `FAILED_PRECONDITION`, `NOT_FOUND`.

### service IntelligencePolicy

```proto
rpc EvaluateGuardrails(EvaluateGuardrailsRequest) returns (GuardrailDecision);
```

- Status: `reference-planned`.
- Semantics: evaluates input and output candidate against policy packs.
- Auth: `intelligence::guardrail_refusal_required`.
- Errors: `INVALID_ARGUMENT`, `PERMISSION_DENIED`.

```proto
rpc ListRefusals(ListRefusalsRequest) returns (ListRefusalsResponse);
```

- Status: `reference-planned`.
- Semantics: lists refusal records for review and analytics.
- Auth: `intelligence::human_review_high_risk`.
- Errors: `INVALID_ARGUMENT`, `PERMISSION_DENIED`.

### service IntelligenceProvider

```proto
rpc EvaluateProviderRoute(EvaluateProviderRouteRequest) returns (ProviderRouteDecision);
```

- Status: `reference-planned`.
- Semantics: chooses provider, model, region, and fallback path.
- Auth: `intelligence::provider_route_allowed`.
- Errors: `FAILED_PRECONDITION`, `UNAVAILABLE`.

```proto
rpc GetProviderHealth(GetProviderHealthRequest) returns (ProviderHealth);
```

- Status: `reference-planned`.
- Semantics: returns health, saturation, and circuit-breaker state.
- Auth: `intelligence::tenant_scope_match`.
- Errors: `NOT_FOUND`, `UNAVAILABLE`.

### service IntelligenceEvidence

```proto
rpc CreateAttribution(CreateAttributionRequest) returns (AttributionReport);
```

- Status: `reference-planned`.
- Semantics: maps generated claims to source spans.
- Auth: `intelligence::retrieval_data_boundary`.
- Errors: `INVALID_ARGUMENT`, `FAILED_PRECONDITION`.

```proto
rpc CreateEvalRun(CreateEvalRunRequest) returns (EvalRun);
```

- Status: `reference-planned`.
- Semantics: starts model or route evaluation against a suite.
- Auth: `intelligence::human_review_high_risk`.
- Errors: `INVALID_ARGUMENT`, `RESOURCE_EXHAUSTED`.

```proto
rpc ResolveCredential(ResolveCredentialRequest) returns (ResolvedCredentialLease);
```

- Status: `reference-planned`.
- Semantics: resolves a provider credential lease through OpenBao.
- Auth: `intelligence::credential_reference_use`.
- Errors: `PERMISSION_DENIED`, `NOT_FOUND`, `UNAVAILABLE`.

## AsyncAPI Channels

### intelligence.assist-draft-completed

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `AssistDraftCompleted`.
- Delivery semantics: at-least-once, deduplicate by `event_id`.
- Consumers: workflow-studio, messenger, audit-chain, governance.

### intelligence.policy-refused

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `PolicyRefused`.
- Delivery semantics: at-least-once with durable refusal id.
- Consumers: governance, audit-chain, safety review.

### intelligence.retrieval-context-bound

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `RetrievalContextBound`.
- Delivery semantics: ordered per `retrieval_id`.
- Consumers: audit-chain, governance, assist callers.

### intelligence.dispatch-completed

- Direction: publish.
- Status: `reference-planned`.
- Payload schema: `DispatchCompleted`.
- Delivery semantics: at-least-once, partitioned by `tenant_id`.
- Consumers: caller service, governance, audit-chain.

### intelligence.dispatch-failed

- Direction: publish.
- Status: `reference-planned`.
- Payload schema: `DispatchFailed`.
- Delivery semantics: at-least-once with terminal status.
- Consumers: caller service, provider router, governance.

### intelligence.provider-saturated

- Direction: publish.
- Status: `reference-planned`.
- Payload schema: `ProviderSaturated`.
- Delivery semantics: compacted by `provider_id` and `region`.
- Consumers: provider router, operations, governance.

### intelligence.guardrail-refusal-recorded

- Direction: publish.
- Status: `reference-planned`.
- Payload schema: `GuardrailRefusalRecorded`.
- Delivery semantics: at-least-once.
- Consumers: safety review, audit-chain, governance.

### intelligence.eval-completed

- Direction: publish.
- Status: `reference-planned`.
- Payload schema: `EvalCompleted`.
- Delivery semantics: at-least-once, ordered per `eval_run_id`.
- Consumers: governance, model routing, release gates.

### intelligence.credential-rotated

- Direction: publish.
- Status: `reference-planned`.
- Payload schema: `CredentialRotated`.
- Delivery semantics: compacted by `secret_reference`.
- Consumers: provider router, audit-chain, security.

### workflow-studio.llm-assist-draft-requested

- Direction: subscribe.
- Status: `contract-bound`.
- Payload schema: `LlmAssistDraftRequested`.
- Delivery semantics: at-least-once.
- Handler: produce assist draft or refusal event.

### messenger.thread-summary-requested

- Direction: subscribe.
- Status: `reference-planned`.
- Payload schema: `ThreadSummaryRequested`.
- Delivery semantics: at-least-once.
- Handler: bind thread retrieval and generate summary.

### drive.document-grounding-requested

- Direction: subscribe.
- Status: `reference-planned`.
- Payload schema: `DocumentGroundingRequested`.
- Delivery semantics: at-least-once.
- Handler: bind document source spans to retrieval context.

## Webhooks Inbound

### webhook.provider.outage

- Source: model provider status bridge.
- Event: `provider.outage`.
- Payload schema: `ProviderOutageWebhook`.
- Semantics: trips route circuit breaker and publishes `provider-saturated`.

### webhook.provider.rate-limit

- Source: model provider status bridge.
- Event: `provider.rate_limit`.
- Payload schema: `ProviderRateLimitWebhook`.
- Semantics: reduces route weight for affected model and region.

### webhook.provider.invoice-usage

- Source: provider billing bridge.
- Event: `provider.invoice_usage`.
- Payload schema: `ProviderUsageWebhook`.
- Semantics: updates cost attribution and eval budget posture.

### webhook.governance.policy-pack-updated

- Source: governance.
- Event: `governance.policy_pack.updated`.
- Payload schema: `PolicyPackUpdatedWebhook`.
- Semantics: invalidates guardrail decision cache.

### webhook.audit-chain.seal-failed

- Source: audit-chain.
- Event: `audit_chain.seal.failed`.
- Payload schema: `AuditSealFailedWebhook`.
- Semantics: blocks high-risk dispatch until audit tap recovers.

### webhook.tenancy.pack-changed

- Source: tenant-management.
- Event: `tenant.capability_pack.changed`.
- Payload schema: `TenantPackChangedWebhook`.
- Semantics: recalculates provider and model entitlements.

### webhook.ontology.context-updated

- Source: ontology.
- Event: `ontology.context.updated`.
- Payload schema: `OntologyContextUpdatedWebhook`.
- Semantics: refreshes retrieval type descriptors.

### webhook.security.credential-rotation-required

- Source: security or OpenBao controller.
- Event: `credential.rotation_required`.
- Payload schema: `CredentialRotationRequiredWebhook`.
- Semantics: schedules provider credential rotation and evidence emission.

## SDK Quick Reference

### Rust

```rust
let retrieval = intelligence::create_retrieval(client, retrieval_request).await?;
let draft = intelligence::create_assist_draft(client, draft_request).await?;
let route = intelligence::evaluate_provider_route(client, route_request).await?;
let decision = intelligence::evaluate_guardrails(client, guardrail_request).await?;
let eval = intelligence::create_eval_run(client, eval_request).await?;
```

Named functions:

- `create_retrieval`
- `get_retrieval`
- `create_assist_draft`
- `get_assist_draft`
- `accept_assist_draft`
- `create_dispatch`
- `stream_dispatch`
- `evaluate_provider_route`
- `evaluate_guardrails`
- `create_attribution`
- `create_eval_run`
- `resolve_credential`

### TypeScript

```ts
const intelligence = new IntelligenceClient({ tenantId, token });
const retrieval = await intelligence.createRetrieval({ query, sourceRefs });
const draft = await intelligence.createAssistDraft({ instruction, contextRefs: [retrieval.id] });
const route = await intelligence.evaluateProviderRoute({ capability: "draft" });
for await (const chunk of intelligence.streamDispatch(draft.dispatchId)) render(chunk);
```

Named functions:

- `createRetrieval`
- `getRetrieval`
- `createAssistDraft`
- `getAssistDraft`
- `acceptAssistDraft`
- `createDispatch`
- `streamDispatch`
- `cancelDispatch`
- `evaluateProviderRoute`
- `evaluateGuardrails`
- `listRefusals`
- `createEvalRun`
- `resolveCredential`

### Python

```python
client = IntelligenceClient(tenant_id=tenant_id, token=token)
retrieval = client.create_retrieval(query=query, source_refs=source_refs)
draft = client.create_assist_draft(instruction=instruction, context_refs=[retrieval.id])
decision = client.evaluate_guardrails(input_text=instruction, output_candidate=draft.text)
client.create_attribution(draft_id=draft.id, source_refs=source_refs)
```

Named functions:

- `create_retrieval`
- `get_retrieval`
- `create_assist_draft`
- `get_assist_draft`
- `accept_assist_draft`
- `create_dispatch`
- `stream_dispatch`
- `evaluate_provider_route`
- `evaluate_guardrails`
- `list_refusals`
- `create_eval_run`
- `resolve_credential`

## Error Catalogue

### INTELLIGENCE_AUTHZ_DENIED

- Meaning: Cedar denied the requested operation.
- Retry policy: do not retry without changing scope or principal.
- HTTP mapping: `403`.
- gRPC mapping: `PERMISSION_DENIED`.

### INTELLIGENCE_ASSIST_REFUSED

- Meaning: guardrails refused the prompt, source, or draft candidate.
- Retry policy: retry only with safer input or different data boundary.
- HTTP mapping: `422`.
- gRPC mapping: `FAILED_PRECONDITION`.

### INTELLIGENCE_RETRIEVAL_BOUNDARY_DENIED

- Meaning: retrieval tried to cross an unauthorized data boundary.
- Retry policy: do not retry without changing source refs or authorization.
- HTTP mapping: `403`.
- gRPC mapping: `PERMISSION_DENIED`.

### INTELLIGENCE_PROVIDER_UNAVAILABLE

- Meaning: no allowed provider route is currently available.
- Retry policy: exponential backoff; client may downgrade capability if allowed.
- HTTP mapping: `503`.
- gRPC mapping: `UNAVAILABLE`.

### INTELLIGENCE_NO_PROVIDER_ROUTE

- Meaning: provider route constraints cannot be satisfied.
- Retry policy: retry only after changing model, region, or data constraints.
- HTTP mapping: `422`.
- gRPC mapping: `FAILED_PRECONDITION`.

### INTELLIGENCE_GUARDRAIL_INPUT_INVALID

- Meaning: guardrail evaluation input is missing required context.
- Retry policy: fix request and retry.
- HTTP mapping: `400`.
- gRPC mapping: `INVALID_ARGUMENT`.

### INTELLIGENCE_CREDENTIAL_DENIED

- Meaning: credential resolution failed policy or secret lookup.
- Retry policy: do not retry unless credential policy changes.
- HTTP mapping: `403`.
- gRPC mapping: `PERMISSION_DENIED`.

### INTELLIGENCE_ATTRIBUTION_FAILED

- Meaning: generated claims could not be mapped to source spans.
- Retry policy: retry after rebind or reduce output scope.
- HTTP mapping: `422`.
- gRPC mapping: `FAILED_PRECONDITION`.

### INTELLIGENCE_RATE_LIMITED

- Meaning: request exceeded capability-tier quota.
- Retry policy: honor `Retry-After`; stream clients should reconnect with backoff.
- HTTP mapping: `429`.
- gRPC mapping: `RESOURCE_EXHAUSTED`.

### INTELLIGENCE_AUDIT_TAP_UNAVAILABLE

- Meaning: high-risk output cannot be sealed to audit-chain.
- Retry policy: retry with backoff; fail closed for high-risk dispatch.
- HTTP mapping: `503`.
- gRPC mapping: `UNAVAILABLE`.

## Pagination

Cursor pattern name: `intelligence_audit_cursor`.

Cursor fields:

- `tenant_id`
- `resource_kind`
- `risk_partition`
- `sort_key`
- `last_seen_id`
- `issued_at`
- `signature`

Rules:

- Cursors are opaque and signed.
- Default sort is descending `created_at`.
- Refusal, eval, and audit-tap lists include risk partition in the cursor.
- Cursor TTL is 15 minutes for operational views and 24 hours for evidence export views.
- Invalid cursors return `INTELLIGENCE_CURSOR_INVALID`.

Max page-size limits:

- Assist drafts: `100`.
- Retrievals: `100`.
- Dispatches: `200`.
- Refusals: `200`.
- Eval runs: `100`.
- Audit tap records: `500`.
- Providers: `200`.
- Default page size: `50`.

## Rate Limits per Tier

Per ADR-0316, intelligence uses capability-tier throttles rather than
product-fragmented limits.

| Tier | REST requests per second | gRPC requests per second | Async publishes per second | Burst |
| --- | ---: | ---: | ---: | ---: |

Special limits:


## OpenAPI 3.2.0 Schema

Actual contracts file:

- [intelligence.yaml](../../microservices/intelligence/contracts/openapi/intelligence.yaml)

Design references:

- [intelligence architecture](../../microservices/intelligence/ARCHITECTURE.md)
- [intelligence SDK plan](../../microservices/intelligence/sdk-plan.md)
- [API design standard](../standards/api-design.md)

## AsyncAPI 3.1.0 Schema

Actual contracts file:

- [intelligence-events.yaml](../../microservices/intelligence/contracts/asyncapi/intelligence-events.yaml)

Delivery notes:

- Refusals are durable and must be audit-sealed.
- Dispatch terminal events are at-least-once.
- Provider health events may be compacted.
- Consumers deduplicate by `event_id`.

## proto3 Schema

Actual contracts file:

- [intelligence.proto](../../microservices/intelligence/contracts/proto/intelligence.proto)

Proto package expectations:

- Use proto3 syntax.
- Keep provider credentials out of request bodies; pass references only.
- Map safety refusals to `FAILED_PRECONDITION`.
- Map provider saturation to `UNAVAILABLE` or `RESOURCE_EXHAUSTED`.

## Cross-References

- [intelligence PRD](../../microservices/intelligence/PRD.md)
- [intelligence architecture](../../microservices/intelligence/ARCHITECTURE.md)
- [intelligence SDK plan](../../microservices/intelligence/sdk-plan.md)
- [ADR-0316 capability tier over product fragmentation](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md)
- [API design standard](../standards/api-design.md)
- [Throttling tiers](../standards/throttling-tiers.md)
- [Workflow Studio API reference](workflow-studio-api-reference.md)
- [Messenger API reference](messenger-api-reference.md)
- [Drive API reference](drive-api-reference.md)
- [Audit-chain API reference](audit-chain-api-reference.md)
