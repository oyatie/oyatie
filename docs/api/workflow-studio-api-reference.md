---
doc_class: APIReference
microservice: workflow-studio
version: 1.0.0
status: Accepted
date: 2026-05-20
owner: axis-saas + central-governance + ops-runtime
openapi_version: 3.2.0
asyncapi_version: 3.1.0
proto3: true
---

Tenant class model: `tenant_class` is `controlled_evaluation` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# workflow-studio API Reference

Canonical REST, gRPC, and AsyncAPI reference for the `workflow-studio`
microservice. The service owns visual workflow editing, CRDT collaboration,
jurisdiction overlays, node-library browsing, LLM assist drafts, debugger
sessions, and editor-to-runtime handoff.

Contract status legend:

- `contract-bound`: implemented in the current OpenAPI, AsyncAPI, or proto3 file.
- `reference-planned`: canonical planning-closed API surface derived from the SaaS PRD and architecture notes; live readiness remains `target_non_claim` until promoted into contracts and proven by `presubmit`.

## Quick Start

Named example: `EditValidatePublishAndDebug`.

1. Open an editor session with `POST /editor-sessions`.
2. Save and validate the definition with `POST /workflow-definitions/{definition_id}:save` and `POST /workflow-definitions/{definition_id}:validate`.
3. Open a debugger session with `POST /debugger-sessions` and subscribe to `workflow-studio.debugger.frame-streamed`.

Minimum headers:

- `Authorization: Bearer <oidc-token>`
- `X-Tenant-Id: <uuid-v7>`
- `X-Context-Kind: Personal | Professional`
- `Idempotency-Key: <ulid>` on mutating requests
- `X-Request-Id: <ulid>` for trace correlation
- `Content-Type: application/json`

Example:

```http
POST /editor-sessions HTTP/2
Host: workflow-studio.oyatie.com
Authorization: Bearer eyJ...
X-Tenant-Id: 018f7a54-3ef5-7c42-a111-a2c4ad7f88f0
Idempotency-Key: 01HYWFOPEN00000000000000
Content-Type: application/json
```

## Authentication & Authorization

Authentication patterns:

- OIDC bearer for browser and SDK clients.
- SPIFFE SVID mTLS for workflow-runtime, ontology, messenger, and governance callers.
- Signed CRDT op tokens for collaborative editing streams.
- Cedar decision envelopes for every definition mutation and publish action.

Principal types:

- `WorkflowDesigner`: tenant member who can open editor sessions and save drafts.
- `WorkflowPublisher`: principal allowed to publish and bind runtime channels.
- `WorkflowDebugger`: runtime-support principal with debugger frame access.
- `NodeLibraryMaintainer`: service owner for curated node libraries.
- `JurisdictionAdmin`: policy operator for locale and compliance overlays.
- `LlmAssistOperator`: user or service principal allowed to request drafting assistance.
- `WorkflowRuntimeBridge`: internal runtime service that consumes published definitions.
- `GovernanceAuditor`: read-only principal for conformance evidence.

Named Cedar policy patterns:

- `workflow_studio::tenant_scope_match`: tenant in token, request, and definition must match.
- `workflow_studio::context_isolation`: Personal and Professional workflows cannot share definitions.
- `workflow_studio::definition_author_write`: draft save requires author or delegated editor.
- `workflow_studio::publish_requires_validation`: publish requires a successful validation result.
- `workflow_studio::jurisdiction_overlay_switch`: switch requires allowed jurisdiction pack.
- `workflow_studio::llm_assist_data_boundary`: assist prompts must redact blocked data classes.
- `workflow_studio::debugger_frame_read`: debugger frames require runtime support scope.
- `workflow_studio::node_library_admin`: node-library maintenance requires service owner role.

Authorization failure shape:

```json
{
  "error": {
    "code": "WORKFLOW_STUDIO_AUTHZ_DENIED",
    "message": "Cedar policy denied workflow-studio action",
    "request_id": "01HYREQ...",
    "details": [{"policy": "workflow_studio::publish_requires_validation"}]
  }
}
```

## REST Endpoints

### Editor Sessions

#### POST /editor-sessions

- Status: `contract-bound`.
- Operation: `openEditorSession`.
- Request schema: `OpenEditorSessionRequest`.
- Required fields: `tenant_id`, `definition_id`, `context_kind`, `client_clock`.
- Optional fields: `base_version`, `requested_locale`, `presence_profile`.
- Response schema: `EditorSession`.
- Status codes: `201`, `400`, `401`, `403`, `404`, `409`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_SESSION_CONFLICT` when a lock lease is active.

#### GET /editor-sessions

- Status: `contract-bound`.
- Operation: `listEditorSessions`.
- Request query: `tenant_id`, `definition_id`, `state`, `cursor`, `limit`.
- Response schema: `ListEditorSessionsResponse`.
- Sort order: newest `last_seen_at` first.
- Status codes: `200`, `400`, `401`, `403`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_CURSOR_INVALID`.

#### GET /editor-sessions/{session_id}

- Status: `contract-bound`.
- Operation: `getEditorSession`.
- Path schema: `session_id` as UUID-v7.
- Response schema: `EditorSession`.
- Includes: participants, lease, current version, and active jurisdiction.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_SESSION_NOT_FOUND`.

#### DELETE /editor-sessions/{session_id}

- Status: `contract-bound`.
- Operation: `closeEditorSession`.
- Path schema: `session_id` as UUID-v7.
- Request schema: `CloseEditorSessionRequest`.
- Response schema: `EditorSessionClosed`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_UNSAVED_OPS_EXIST`.

### Workflow Definitions

#### POST /workflow-definitions/{definition_id}:save

- Status: `contract-bound`.
- Operation: `saveWorkflowDefinition`.
- Path schema: `definition_id` as UUID-v7.
- Request schema: `SaveWorkflowDefinitionRequest`.
- Required fields: `expected_version`, `graph`, `layout`, `change_summary`.
- Response schema: `WorkflowDefinitionVersion`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_VERSION_CONFLICT`.

#### GET /workflow-definitions/{definition_id}:load

- Status: `contract-bound`.
- Operation: `loadWorkflowDefinition`.
- Path schema: `definition_id` as UUID-v7.
- Query schema: `version`, `jurisdiction`, `include_layout`.
- Response schema: `WorkflowDefinitionDocument`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_DEFINITION_NOT_FOUND`.

#### POST /workflow-definitions/{definition_id}/jurisdiction:switch

- Status: `contract-bound`.
- Operation: `switchJurisdictionOverlay`.
- Path schema: `definition_id` as UUID-v7.
- Request schema: `SwitchJurisdictionOverlayRequest`.
- Response schema: `JurisdictionOverlayResult`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_JURISDICTION_DENIED`.

#### POST /workflow-definitions/{definition_id}:validate

- Status: `reference-planned`.
- Operation: `validateWorkflowDefinition`.
- Request schema: `ValidateWorkflowDefinitionRequest`.
- Required fields: `graph`, `jurisdiction`, `runtime_pack`.
- Response schema: `ValidationReport`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `422`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_VALIDATION_FAILED`.

#### POST /workflow-definitions/{definition_id}:diff

- Status: `reference-planned`.
- Operation: `diffWorkflowDefinition`.
- Request schema: `DiffWorkflowDefinitionRequest`.
- Required fields: `from_version`, `to_version`.
- Response schema: `WorkflowDefinitionDiff`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_DIFF_RANGE_INVALID`.

#### POST /workflow-definitions/{definition_id}:publish

- Status: `reference-planned`.
- Operation: `publishWorkflowDefinition`.
- Request schema: `PublishWorkflowDefinitionRequest`.
- Required fields: `validated_version`, `runtime_target`, `release_notes`.
- Response schema: `PublishReceipt`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_VALIDATION_REQUIRED`.

#### GET /workflow-definitions/{definition_id}/versions

- Status: `reference-planned`.
- Operation: `listWorkflowDefinitionVersions`.
- Query schema: `cursor`, `limit`, `author_id`, `from_time`, `to_time`.
- Response schema: `ListWorkflowDefinitionVersionsResponse`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_VERSION_QUERY_INVALID`.

#### GET /workflow-definitions/{definition_id}/versions/{version_id}

- Status: `reference-planned`.
- Operation: `getWorkflowDefinitionVersion`.
- Path schema: `definition_id`, `version_id`.
- Response schema: `WorkflowDefinitionVersion`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_VERSION_NOT_FOUND`.

### Node Libraries

#### GET /node-libraries

- Status: `contract-bound`.
- Operation: `listNodeLibraries`.
- Query schema: `tenant_id`, `jurisdiction`, `capability`, `cursor`, `limit`.
- Response schema: `ListNodeLibrariesResponse`.
- Status codes: `200`, `400`, `401`, `403`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_LIBRARY_QUERY_INVALID`.

#### GET /node-libraries/{library_id}

- Status: `contract-bound`.
- Operation: `getNodeLibrary`.
- Path schema: `library_id` as slug or UUID-v7.
- Query schema: `version`, `include_examples`.
- Response schema: `NodeLibrary`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_LIBRARY_NOT_FOUND`.

#### POST /node-libraries/{library_id}:verify

- Status: `reference-planned`.
- Operation: `verifyNodeLibrary`.
- Request schema: `VerifyNodeLibraryRequest`.
- Required fields: `library_version`, `validation_bindings`.
- Response schema: `NodeLibraryVerificationReport`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `422`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_LIBRARY_VERIFICATION_FAILED`.

### LLM Assist

#### POST /llm-assist:draft

- Status: `contract-bound`.
- Operation: `llmAssistDraft`.
- Request schema: `LlmAssistDraftRequest`.
- Required fields: `tenant_id`, `definition_context`, `instruction`, `redaction_profile`.
- Response schema: `LlmAssistDraftResponse`.
- Status codes: `200`, `400`, `401`, `403`, `409`, `422`, `429`, `500`, `503`.
- Error shape: `WORKFLOW_STUDIO_ASSIST_REFUSED`.

#### POST /llm-assist:explain

- Status: `reference-planned`.
- Operation: `explainWorkflowChange`.
- Request schema: `ExplainWorkflowChangeRequest`.
- Required fields: `definition_id`, `version_id`, `audience`.
- Response schema: `WorkflowExplanation`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`, `503`.
- Error shape: `WORKFLOW_STUDIO_EXPLANATION_REFUSED`.

### Collaboration

#### POST /collab-sessions/{session_id}/ops

- Status: `reference-planned`.
- Operation: `appendCrdtOperation`.
- Request schema: `AppendCrdtOperationRequest`.
- Required fields: `op_id`, `actor_id`, `lamport_clock`, `patch`.
- Response schema: `CrdtOperationReceipt`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_CRDT_REJECTED`.

#### GET /collab-sessions/{session_id}/ops

- Status: `reference-planned`.
- Operation: `listCrdtOperations`.
- Query schema: `after_clock`, `cursor`, `limit`.
- Response schema: `ListCrdtOperationsResponse`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_OP_CURSOR_INVALID`.

### Debugger

#### POST /debugger-sessions

- Status: `contract-bound`.
- Operation: `openDebuggerSession`.
- Request schema: `OpenDebuggerSessionRequest`.
- Required fields: `definition_id`, `runtime_run_id`, `breakpoints`.
- Response schema: `DebuggerSession`.
- Status codes: `201`, `400`, `401`, `403`, `404`, `409`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_DEBUGGER_UNAVAILABLE`.

#### POST /debugger-sessions/{session_id}:resync

- Status: `contract-bound`.
- Operation: `resyncDebuggerSession`.
- Request schema: `ResyncDebuggerSessionRequest`.
- Required fields: `last_frame_id`, `client_clock`.
- Response schema: `DebuggerResyncResponse`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_DEBUGGER_RESYNC_REQUIRED`.

#### GET /debugger-sessions/{session_id}/frames

- Status: `reference-planned`.
- Operation: `listDebuggerFrames`.
- Query schema: `after_frame_id`, `cursor`, `limit`.
- Response schema: `ListDebuggerFramesResponse`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_FRAME_CURSOR_INVALID`.

### Runtime Handoff

#### POST /runtime-handoffs

- Status: `reference-planned`.
- Operation: `createRuntimeHandoff`.
- Request schema: `CreateRuntimeHandoffRequest`.
- Required fields: `definition_id`, `published_version`, `target_runtime`.
- Response schema: `RuntimeHandoffReceipt`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_RUNTIME_HANDOFF_FAILED`.

#### GET /runtime-handoffs/{handoff_id}

- Status: `reference-planned`.
- Operation: `getRuntimeHandoff`.
- Path schema: `handoff_id` as UUID-v7.
- Response schema: `RuntimeHandoffReceipt`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `WORKFLOW_STUDIO_HANDOFF_NOT_FOUND`.

### Health

#### GET /health

- Status: `contract-bound`.
- Operation: `health`.
- Response schema: `HealthStatus`.
- Status codes: `200`, `500`.
- Error shape: standard health probe failure.

#### GET /ready

- Status: `contract-bound`.
- Operation: `ready`.
- Response schema: `ReadinessStatus`.
- Status codes: `200`, `503`.
- Error shape: `WORKFLOW_STUDIO_DEPENDENCY_UNREADY`.

## gRPC Methods

### service WorkflowStudioEditor

```proto
rpc OpenEditorSession(OpenEditorSessionRequest) returns (EditorSession);
```

- Status: `contract-bound`.
- Semantics: opens a collaborative editor session and issues a lease.
- Auth: `workflow_studio::definition_author_write`.
- Errors: `ALREADY_EXISTS`, `PERMISSION_DENIED`, `RESOURCE_EXHAUSTED`.

```proto
rpc GetEditorSession(GetEditorSessionRequest) returns (EditorSession);
```

- Status: `contract-bound`.
- Semantics: returns current editor session state.
- Auth: `workflow_studio::tenant_scope_match`.
- Errors: `NOT_FOUND`, `PERMISSION_DENIED`.

```proto
rpc CloseEditorSession(CloseEditorSessionRequest) returns (EditorSessionClosed);
```

- Status: `contract-bound`.
- Semantics: closes a session and releases the edit lease.
- Auth: `workflow_studio::definition_author_write`.
- Errors: `FAILED_PRECONDITION`, `NOT_FOUND`.

```proto
rpc ListEditorSessions(ListEditorSessionsRequest) returns (ListEditorSessionsResponse);
```

- Status: `contract-bound`.
- Semantics: lists active and recent editor sessions.
- Auth: `workflow_studio::tenant_scope_match`.
- Errors: `INVALID_ARGUMENT`, `PERMISSION_DENIED`.

### service WorkflowStudioDefinitions

```proto
rpc LoadWorkflowDefinition(LoadWorkflowDefinitionRequest) returns (WorkflowDefinitionDocument);
```

- Status: `contract-bound`.
- Semantics: loads graph, layout, metadata, and overlay selection.
- Auth: `workflow_studio::definition_author_write`.
- Errors: `NOT_FOUND`, `PERMISSION_DENIED`.

```proto
rpc SaveWorkflowDefinition(SaveWorkflowDefinitionRequest) returns (WorkflowDefinitionVersion);
```

- Status: `contract-bound`.
- Semantics: writes a versioned workflow definition draft.
- Auth: `workflow_studio::definition_author_write`.
- Errors: `ABORTED`, `INVALID_ARGUMENT`, `FAILED_PRECONDITION`.

```proto
rpc SwitchJurisdictionOverlay(SwitchJurisdictionOverlayRequest) returns (JurisdictionOverlayResult);
```

- Status: `contract-bound`.
- Semantics: evaluates and applies jurisdiction-specific graph overlays.
- Auth: `workflow_studio::jurisdiction_overlay_switch`.
- Errors: `PERMISSION_DENIED`, `FAILED_PRECONDITION`.

```proto
rpc ValidateWorkflowDefinition(ValidateWorkflowDefinitionRequest) returns (ValidationReport);
```

- Status: `reference-planned`.
- Semantics: checks graph correctness and runtime deployability.
- Auth: `workflow_studio::publish_requires_validation`.
- Errors: `INVALID_ARGUMENT`, `FAILED_PRECONDITION`.

### service WorkflowStudioCollaboration

```proto
rpc StreamCrdtOps(StreamCrdtOpsRequest) returns (stream CrdtOperation);
```

- Status: `contract-bound`.
- Semantics: streams merged editor operations to collaborators.
- Auth: `workflow_studio::definition_author_write`.
- Errors: `UNAVAILABLE`, `OUT_OF_RANGE`, `RESOURCE_EXHAUSTED`.

### service WorkflowStudioLibraries

```proto
rpc ListNodeLibraries(ListNodeLibrariesRequest) returns (ListNodeLibrariesResponse);
```

- Status: `contract-bound`.
- Semantics: lists node libraries visible to the tenant and jurisdiction.
- Auth: `workflow_studio::tenant_scope_match`.
- Errors: `INVALID_ARGUMENT`, `PERMISSION_DENIED`.

```proto
rpc GetNodeLibrary(GetNodeLibraryRequest) returns (NodeLibrary);
```

- Status: `contract-bound`.
- Semantics: returns one node library and optional examples.
- Auth: `workflow_studio::tenant_scope_match`.
- Errors: `NOT_FOUND`, `PERMISSION_DENIED`.

### service WorkflowStudioAssistAndDebugger

```proto
rpc LlmAssistDraft(LlmAssistDraftRequest) returns (LlmAssistDraftResponse);
```

- Status: `contract-bound`.
- Semantics: generates a safe draft or explanation.
- Auth: `workflow_studio::llm_assist_data_boundary`.
- Errors: `FAILED_PRECONDITION`, `RESOURCE_EXHAUSTED`, `UNAVAILABLE`.

```proto
rpc OpenDebuggerSession(OpenDebuggerSessionRequest) returns (DebuggerSession);
```

- Status: `contract-bound`.
- Semantics: opens a debugger view onto a runtime run.
- Auth: `workflow_studio::debugger_frame_read`.
- Errors: `NOT_FOUND`, `FAILED_PRECONDITION`.

```proto
rpc StreamDebuggerFrames(StreamDebuggerFramesRequest) returns (stream DebuggerFrame);
```

- Status: `contract-bound`.
- Semantics: streams runtime frames for visual inspection.
- Auth: `workflow_studio::debugger_frame_read`.
- Errors: `UNAVAILABLE`, `OUT_OF_RANGE`.

```proto
rpc ResyncDebuggerSession(ResyncDebuggerSessionRequest) returns (DebuggerResyncResponse);
```

- Status: `contract-bound`.
- Semantics: reconciles missed debugger frames after client reconnect.
- Auth: `workflow_studio::debugger_frame_read`.
- Errors: `ABORTED`, `NOT_FOUND`.

## AsyncAPI Channels

### workflow-studio.editor-session-opened

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `EditorSessionOpened`.
- Delivery semantics: at-least-once, partitioned by `tenant_id`.
- Consumers: audit-chain, governance, presence, analytics.

### workflow-studio.editor-session-closed

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `EditorSessionClosed`.
- Delivery semantics: at-least-once with duplicate suppression by `event_id`.
- Consumers: audit-chain, governance, presence.

### workflow-studio.definition-saved

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `WorkflowDefinitionSaved`.
- Delivery semantics: ordered per `definition_id`.
- Consumers: workflow-runtime, governance, audit-chain.

### workflow-studio.collab-merged

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `CrdtMerged`.
- Delivery semantics: ordered per `session_id`.
- Consumers: editor clients, audit-chain.

### workflow-studio.collab-conflict-surfaced

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `CrdtConflictSurfaced`.
- Delivery semantics: at-least-once.
- Consumers: messenger, governance, support.

### workflow-studio.license-gate-emitted

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `LicenseGateEmitted`.
- Delivery semantics: at-least-once.
- Consumers: governance, billing, tenant-admin.

### workflow-studio.jurisdiction-switched

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `JurisdictionSwitched`.
- Delivery semantics: ordered per `definition_id`.
- Consumers: governance, audit-chain.

### workflow-studio.llm-assist-draft-requested

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `LlmAssistDraftRequested`.
- Delivery semantics: at-least-once, redact blocked fields before publish.
- Consumers: intelligence, audit-chain.

### workflow-studio.llm-assist-draft-accepted

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `LlmAssistDraftAccepted`.
- Delivery semantics: at-least-once with acceptance receipt.
- Consumers: intelligence, governance, audit-chain.

### workflow-runtime.workflow-started

- Direction: subscribe.
- Status: `contract-bound`.
- Payload schema: `WorkflowStarted`.
- Delivery semantics: at-least-once.
- Handler: bind runtime run to debugger session.

### workflow-runtime.step-started

- Direction: subscribe.
- Status: `contract-bound`.
- Payload schema: `WorkflowStepStarted`.
- Delivery semantics: ordered per `run_id`.
- Handler: append visual debugger frame.

### workflow-runtime.step-completed

- Direction: subscribe.
- Status: `contract-bound`.
- Payload schema: `WorkflowStepCompleted`.
- Delivery semantics: ordered per `run_id`.
- Handler: update debugger frame state and validation hints.

### tenancy.tenant-seat-limit-updated

- Direction: subscribe.
- Status: `contract-bound`.
- Payload schema: `TenantSeatLimitUpdated`.
- Delivery semantics: compacted by `tenant_id`.
- Handler: enforce collaborative editor seat limits.

### ontology.type-descriptor-updated

- Direction: subscribe.
- Status: `contract-bound`.
- Payload schema: `OntologyTypeDescriptorUpdated`.
- Delivery semantics: at-least-once.
- Handler: refresh node input/output descriptors.

## Webhooks Inbound

### webhook.workflow-runtime.run-started

- Source: workflow-runtime.
- Event: `workflow.run.started`.
- Payload schema: `RuntimeRunStartedWebhook`.
- Semantics: opens debugger availability and emits audit evidence.

### webhook.workflow-runtime.step-frame

- Source: workflow-runtime.
- Event: `workflow.step.frame`.
- Payload schema: `RuntimeStepFrameWebhook`.
- Semantics: appends one debugger frame to the active session.

### webhook.tenant.seat-limit-updated

- Source: tenant-management.
- Event: `tenant.seat_limit.updated`.
- Payload schema: `TenantSeatLimitWebhook`.
- Semantics: recalculates active collaboration admission.

### webhook.license.entitlement-changed

- Source: billing or governance.
- Event: `license.entitlement.changed`.
- Payload schema: `LicenseEntitlementWebhook`.
- Semantics: gates premium nodes and LLM assist features.

### webhook.ontology.type-descriptor-updated

- Source: ontology.
- Event: `ontology.type_descriptor.updated`.
- Payload schema: `TypeDescriptorUpdatedWebhook`.
- Semantics: invalidates node schema caches.

### webhook.intelligence.assist-completed

- Source: intelligence.
- Event: `intelligence.assist.completed`.
- Payload schema: `AssistCompletedWebhook`.
- Semantics: attaches safe draft output to the editor session.

### webhook.governance.policy-pack-updated

- Source: governance.
- Event: `governance.policy_pack.updated`.
- Payload schema: `PolicyPackUpdatedWebhook`.
- Semantics: refreshes validation and publish gates.

### webhook.audit-chain.seal-failed

- Source: audit-chain.
- Event: `audit_chain.seal.failed`.
- Payload schema: `AuditSealFailedWebhook`.
- Semantics: blocks publish until evidence sealing recovers.

## SDK Quick Reference

### Rust

```rust
let session = workflow_studio::open_editor_session(client, request).await?;
let version = workflow_studio::save_workflow_definition(client, definition_id, graph).await?;
let report = workflow_studio::validate_workflow_definition(client, definition_id, graph).await?;
let publish = workflow_studio::publish_workflow_definition(client, definition_id, version.id).await?;
let frames = workflow_studio::stream_debugger_frames(client, session.id).await?;
```

Named functions:

- `open_editor_session`
- `close_editor_session`
- `load_workflow_definition`
- `save_workflow_definition`
- `validate_workflow_definition`
- `publish_workflow_definition`
- `list_node_libraries`
- `llm_assist_draft`
- `open_debugger_session`
- `stream_debugger_frames`

### TypeScript

```ts
const studio = new WorkflowStudioClient({ tenantId, token });
const session = await studio.openEditorSession({ definitionId });
await studio.saveWorkflowDefinition(definitionId, graph);
await studio.validateWorkflowDefinition(definitionId, graph);
await studio.publishWorkflowDefinition(definitionId, { validatedVersion });
for await (const frame of studio.streamDebuggerFrames(session.id)) render(frame);
```

Named functions:

- `openEditorSession`
- `listEditorSessions`
- `loadWorkflowDefinition`
- `saveWorkflowDefinition`
- `switchJurisdictionOverlay`
- `validateWorkflowDefinition`
- `publishWorkflowDefinition`
- `listNodeLibraries`
- `llmAssistDraft`
- `streamDebuggerFrames`

### Python

```python
studio = WorkflowStudioClient(tenant_id=tenant_id, token=token)
session = studio.open_editor_session(definition_id=definition_id)
version = studio.save_workflow_definition(definition_id, graph)
report = studio.validate_workflow_definition(definition_id, graph)
studio.publish_workflow_definition(definition_id, validated_version=version.id)
for frame in studio.stream_debugger_frames(session.id):
    render(frame)
```

Named functions:

- `open_editor_session`
- `list_editor_sessions`
- `load_workflow_definition`
- `save_workflow_definition`
- `switch_jurisdiction_overlay`
- `validate_workflow_definition`
- `publish_workflow_definition`
- `list_node_libraries`
- `llm_assist_draft`
- `stream_debugger_frames`

## Error Catalogue

### WORKFLOW_STUDIO_AUTHZ_DENIED

- Meaning: Cedar denied the action.
- Retry policy: do not retry without changing principal or scope.
- HTTP mapping: `403`.
- gRPC mapping: `PERMISSION_DENIED`.

### WORKFLOW_STUDIO_SESSION_CONFLICT

- Meaning: another active editor lease blocks this open request.
- Retry policy: retry after lease expiry or close the existing session.
- HTTP mapping: `409`.
- gRPC mapping: `ALREADY_EXISTS`.

### WORKFLOW_STUDIO_VERSION_CONFLICT

- Meaning: expected version does not match the current definition version.
- Retry policy: reload, merge, then retry with a new idempotency key.
- HTTP mapping: `409`.
- gRPC mapping: `ABORTED`.

### WORKFLOW_STUDIO_VALIDATION_FAILED

- Meaning: graph is syntactically valid but not deployable.
- Retry policy: do not retry until validation findings are fixed.
- HTTP mapping: `422`.
- gRPC mapping: `FAILED_PRECONDITION`.

### WORKFLOW_STUDIO_VALIDATION_REQUIRED

- Meaning: publish was requested without a current validation receipt.
- Retry policy: run validation first.
- HTTP mapping: `422`.
- gRPC mapping: `FAILED_PRECONDITION`.

### WORKFLOW_STUDIO_JURISDICTION_DENIED

- Meaning: selected overlay is not available to the tenant or principal.
- Retry policy: do not retry; request entitlement or choose another overlay.
- HTTP mapping: `403`.
- gRPC mapping: `PERMISSION_DENIED`.

### WORKFLOW_STUDIO_CRDT_REJECTED

- Meaning: CRDT operation failed ordering, signature, or patch validation.
- Retry policy: resync session and replay from the returned clock.
- HTTP mapping: `409`.
- gRPC mapping: `ABORTED`.

### WORKFLOW_STUDIO_ASSIST_REFUSED

- Meaning: intelligence policy refused the draft request.
- Retry policy: retry only with a safer instruction or redaction profile.
- HTTP mapping: `422`.
- gRPC mapping: `FAILED_PRECONDITION`.

### WORKFLOW_STUDIO_DEBUGGER_UNAVAILABLE

- Meaning: runtime frames cannot be attached to the requested run.
- Retry policy: exponential backoff for transient runtime unavailability.
- HTTP mapping: `503`.
- gRPC mapping: `UNAVAILABLE`.

### WORKFLOW_STUDIO_RATE_LIMITED

- Meaning: request exceeded the tier limit.
- Retry policy: honor `Retry-After` and apply client-side token bucket.
- HTTP mapping: `429`.
- gRPC mapping: `RESOURCE_EXHAUSTED`.

## Pagination

Cursor pattern name: `workflow_studio_versioned_cursor`.

Cursor fields:

- `tenant_id`
- `resource_kind`
- `sort_key`
- `last_seen_id`
- `issued_at`
- `signature`

Rules:

- Cursor values are opaque to clients.
- Cursor TTL is 15 minutes for editor sessions and 60 minutes for version history.
- Sort is stable by `last_seen_at` or `created_at` plus UUID-v7 tiebreaker.
- Deleted sessions are skipped unless `include_closed=true`.
- Invalid cursors return `WORKFLOW_STUDIO_CURSOR_INVALID`.

Max page-size limits:

- Editor sessions: `100`.
- Definition versions: `100`.
- Node libraries: `200`.
- CRDT operations: `500`.
- Debugger frames: `500`.
- Default page size: `50`.

## Rate Limits per Tier

Per ADR-0316, workflow-studio uses capability-tier throttles rather than
product-fragmented limits.

| Tier | REST requests per second | gRPC requests per second | Async publishes per second | Burst |
| --- | ---: | ---: | ---: | ---: |

Special limits:


## OpenAPI 3.2.0 Schema

Actual contracts file:

- [workflow-studio.yaml](../../microservices/workflow-studio/contracts/openapi/workflow-studio.yaml)

Compatibility and design references:

- [API design standard](../standards/api-design.md)
- [Throttling tiers](../standards/throttling-tiers.md)

## AsyncAPI 3.1.0 Schema

Actual contracts file:

- [workflow-studio-events.yaml](../../microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml)

Delivery notes:

- Published workflow-studio events are at-least-once.
- Definition events are ordered per `definition_id`.
- Collaboration events are ordered per `session_id`.
- Consumers must deduplicate by `event_id`.

## proto3 Schema

Actual contracts file:

- [workflow-studio.proto](../../microservices/workflow-studio/contracts/proto/workflow-studio.proto)

Proto package expectations:

- Use proto3 syntax.
- Include request-scoped `tenant_id`.
- Include `request_id` or metadata equivalent on mutating calls.
- Map Cedar denials to `PERMISSION_DENIED`.

## Cross-References

- [workflow-studio PRD](../../microservices/workflow-studio/PRD.md)
- [ADR-0316 capability tier over product fragmentation](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md)
- [API design standard](../standards/api-design.md)
- [Throttling tiers](../standards/throttling-tiers.md)
- [Messenger API reference](messenger-api-reference.md)
- [Audit-chain API reference](audit-chain-api-reference.md)
- [Governance API reference](governance-api-reference.md)
- [Ontology API reference](ontology-api-reference.md)
