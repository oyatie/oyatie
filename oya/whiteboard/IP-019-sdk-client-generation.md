# IP-019 Whiteboard SDK Client Generation

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-019-sdk-client-generation.md
Benchmarks: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- Replace hand-written whiteboard client drift with generated clients from repo-owned OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, and BNF v4.1 contract sources.
- Preserve ADR-0321 flat-service ownership: SDK packages expose `whiteboard` capability clients, not vendor-suite namespaces.
- Match Miro Enterprise board APIs, Mural Enterprise workspace controls, FigJam multiplayer expectations, Lucidspark diagram exports, Whiteboard.fi classroom sessions, and Microsoft Whiteboard tenant administration without adopting their object models.
- Generate tenant-aware clients for `board-open`, `canvas-op-append`, `presence-sync`, `history-snapshot`, `export-render`, and `template-marketplace-install`.
- Make SDK generation a promotion gate, not a convenience artifact, because client shape determines whether integrators can safely leave displaced vendor products.

## Repo-Local Inputs
- Source PRD: `microservices/whiteboard/PRD.md`.
- Capability records: `microservices/whiteboard/capabilities/board-open.yaml`.
- Capability records: `microservices/whiteboard/capabilities/canvas-op-append.yaml`.
- Capability records: `microservices/whiteboard/capabilities/presence-sync.yaml`.
- Capability records: `microservices/whiteboard/capabilities/history-snapshot.yaml`.
- Capability records: `microservices/whiteboard/capabilities/export-render.yaml`.
- Capability records: `microservices/whiteboard/capabilities/template-marketplace-install.yaml`.
- SDK planning anchor: `microservices/whiteboard/sdk-plan.md`.
- Contract directory anchor: `microservices/whiteboard/contracts/`.
- Catalog anchor: `microservices/whiteboard/catalog/`.
- Policy anchor: `microservices/whiteboard/policies/`.
- SLO anchor: `microservices/whiteboard/slos/`.
- Dashboard anchor: `microservices/whiteboard/dashboards/`.
- Audit input: `microservices/whiteboard/AUDIT-FINDINGS-2026-05-21.json`.

## Client Package Boundaries
- Package name must be capability-neutral: `whiteboard`, `whiteboard-admin`, or `whiteboard-replay`, never `miro`, `mural`, `figjam`, `lucidspark`, `whiteboard-fi`, or `microsoft-whiteboard`.
- Public namespaces must align with repo capabilities: `boards`, `operations`, `presence`, `history`, `exports`, and `templates`.
- The generated client must expose tenant identity as an explicit constructor dependency, not a hidden global.
- The generated client must expose principal identity per request when the operation can be delegated by workflow automation.
- The generated client must expose audience type for collaborative users, auditors, CI principals, and emergency-service paths.
- The generated client must preserve data class on every mutation and replay call.
- The generated client must carry purpose and pack overlay fields from the capability records.
- The generated client must include DealSet settlement references for template marketplace calls under ADR-0314.
- The generated client must include HTTP/3 h3-alt-svc, ECH/PQC capability flags from ADR-0253-amendment where transport metadata is modeled.
- The generated client must model Cedar refusal as a first-class typed result, not a generic exception.
- The generated client must model replay conflicts separately from transport failures.
- The generated client must model export rendering as an async job even when a small board completes immediately.

## Capability Mapping
- `board-open` generates `openBoard`, `getBoardEnvelope`, and `resolveBoardAccess`.
- `board-open` maps to board object lifecycle parity with Miro Enterprise and Mural Enterprise.
- `board-open` must not imply document-file semantics from Microsoft Whiteboard storage integrations.
- `canvas-op-append` generates `appendCanvasOperation`, `appendBatch`, and `previewAppend`.
- `canvas-op-append` maps to multiplayer editing expectations from FigJam and Miro Enterprise.
- `canvas-op-append` must expose idempotency keys and operation sequence guards.
- `presence-sync` generates `syncPresence`, `subscribePresence`, and `closePresenceLease`.
- `presence-sync` maps to cursor, selection, and participant-state parity from FigJam and Microsoft Whiteboard.
- `presence-sync` must allow volatile event handling without audit-chain pollution.
- `history-snapshot` generates `createHistorySnapshot`, `getSnapshot`, and `compareSnapshots`.
- `history-snapshot` maps to audit and version recovery expectations from Lucidspark and Miro Enterprise.
- `history-snapshot` must bind snapshot data to `export_snapshot`.
- `export-render` generates `requestExportRender`, `getExportStatus`, and `downloadExportArtifact`.
- `export-render` maps to PDF/image/board export parity from Mural Enterprise, Lucidspark, and Microsoft Whiteboard.
- `export-render` must separate artifact access from board mutation scope.
- `template-marketplace-install` generates `installTemplate`, `previewTemplate`, and `removeTemplateGrant`.
- `template-marketplace-install` maps to Miro Enterprise template libraries and Mural Enterprise facilitation templates.
- `template-marketplace-install` must carry DealSet settlement metadata before activation.

## Generation Pipeline
- Read contract sources only from repo-owned whiteboard contract paths.
- Fail generation if contract titles contain displaced vendor namespaces.
- Fail generation if any operation omits `tenant_id`.
- Fail generation if any mutation omits `principal_id`.
- Fail generation if any request omits `purpose`.
- Fail generation if any request omits `data_class`.
- Fail generation if any mutation omits idempotency.
- Fail generation if any async job omits progress state.
- Fail generation if any response omits audit-chain correlation where required.
- Fail generation if any generated endpoint lacks OpenAPI tags aligned to capability names.
- Fail generation if AsyncAPI channels use anonymous board identifiers.
- Fail generation if proto packages cross microservice boundaries without an explicit internal surface.
- Fail generation if BNF operation names diverge from capability records.
- Fail generation if marketplace settlement fields appear outside template installation flows.
- Fail generation if presence streams are treated as durable board history.
- Fail generation if export artifacts can be downloaded without tenant and principal checks.

## Language Targets
- TypeScript client must be generated first for internal console and workflow template usage.
- TypeScript client must expose abort signals for collaborative UX cancellation.
- TypeScript client must preserve discriminated unions for Cedar denial, validation rejection, quota rejection, and replay conflict.
- TypeScript client must avoid any direct local-storage assumption.
- TypeScript client must ship examples for board open, append, export, and template install.
- Rust client must be generated for internal service-to-service and replay tooling.
- Rust client must expose typed tenant scope rather than stringly request maps.
- Rust client must return structured errors compatible with audit-event capture.
- Rust client must not hide retry policy inside generated method bodies.
- Kotlin or JVM client remains optional until a repo-local consumer exists.
- Python client remains optional until a migration fixture or notebook consumer exists.
- Any optional client must pass the same contract parity gate before publication.

## Authentication And Authorization
- SDK construction requires tenant resolver injection.
- SDK calls require principal resolver injection or explicit principal arguments.
- SDK calls require Cedar decision correlation in metadata.
- SDK calls must preserve default-deny as the generated documentation baseline.
- SDK calls must expose `audience_type=COLLABORATION_USER` for ordinary collaborative users.
- SDK calls must expose auditor and CI audience paths only through separate method overloads or scoped factories.
- SDK calls must never infer tenant from board id.
- SDK calls must never infer principal from websocket identity alone.
- SDK calls must never downgrade data class during retry.
- SDK calls must never bypass policy evaluation for local previews.
- SDK examples must include denial handling beside success handling.
- SDK examples must include audit evidence retrieval beside mutation calls.

## Transport Semantics
- REST client methods map command paths from OpenAPI 3.2.0.
- Event subscribers map channels from AsyncAPI 3.1.0.
- Internal client stubs map proto3 service shapes only when synchronous internals exist.
- HTTP/3 support is represented as negotiated metadata, not as a required client transport for every caller.
- Websocket or event-stream presence must support reconnect with explicit lease renewal.
- Reconnect must not replay stale canvas operations without sequence verification.
- Retry must be disabled by default for non-idempotent operations.
- Retry may be enabled for idempotent append requests only when the idempotency key is supplied.
- Export polling must use server-provided backoff hints.
- Template installation must not retry DealSet settlement submission unless the settlement id is reused.
- Snapshot comparison calls must preserve source snapshot and target snapshot ids.
- Download calls must verify artifact tenant binding before streaming.

## Data Shapes
- Board envelope contains board id, tenant id, cell, data class, policy snapshot id, and version pointer.
- Canvas operation contains operation id, board id, actor principal, sequence number, operation body, and merge hint.
- Presence state contains ephemeral cursor, selection, viewport, participant lease, and expiry.
- Snapshot state contains snapshot id, board version, export class, retention pack, and audit event id.
- Export job contains job id, requested formats, status, progress, artifact ids, and refusal reason.
- Template install contains template id, source marketplace reference, DealSet settlement id, pack overlay, and rollback token.
- All generated models preserve unknown-safe extension fields where contracts allow forward compatibility.
- All generated models reject unscoped vendor import payloads before network submission.
- All generated models include serialization tests for tenant, principal, purpose, data class, and trace context.
- All generated models include deserialization tests for denial and partial-progress states.

## Canvas Domain Model For SDKs
- The SDK must expose a `BoardEnvelope` read model, not a generic file model.
- The SDK must expose a `CanvasOperation` append command, not a full-board save command.
- The SDK must expose operation-family discriminators for shape, connector, sticky note, frame, comment anchor, template insertion, and imported vendor object.
- The SDK must expose a CRDT-compatible operation id and sequence guard while leaving merge implementation server-owned.
- The SDK must expose vector or lamport-style version metadata only as contract fields defined by whiteboard contracts.
- The SDK must expose participant presence as expiring session state, not as board content.
- The SDK must expose board history as immutable snapshot pointers, not as undo-stack mutation calls.
- The SDK must expose export render jobs as artifact-producing workflows.
- The SDK must expose marketplace template installation as a settlement-backed grant.
- The SDK must expose migration replay as a preview-to-accept workflow.
- The SDK must never let clients synthesize authoritative board versions.
- The SDK must never let clients directly mutate snapshot state.

## Command, Event, And Proto Deltas
- OpenAPI command delta: add `POST /whiteboard/boards:open` for board envelope access.
- OpenAPI command delta: add `POST /whiteboard/boards/{board_id}/operations:append` for operation append.
- OpenAPI command delta: add `POST /whiteboard/boards/{board_id}/operations:preview` for non-mutating replay preview.
- OpenAPI command delta: add `POST /whiteboard/boards/{board_id}/history:snapshot` for accepted snapshot jobs.
- OpenAPI command delta: add `POST /whiteboard/boards/{board_id}/exports:render` for async export jobs.
- OpenAPI command delta: add `POST /whiteboard/templates:install` for settlement-backed templates.
- AsyncAPI event delta: publish `whiteboard.canvas_operation.appended`.
- AsyncAPI event delta: publish `whiteboard.canvas_operation.rejected`.
- AsyncAPI event delta: publish `whiteboard.presence.lease_renewed`.
- AsyncAPI event delta: publish `whiteboard.presence.lease_expired`.
- AsyncAPI event delta: publish `whiteboard.history_snapshot.completed`.
- AsyncAPI event delta: publish `whiteboard.export_render.completed`.
- AsyncAPI event delta: publish `whiteboard.template_install.settled`.
- Proto delta: internal append service carries tenant, principal, board, operation, and sequence facts.
- Proto delta: internal render service carries tenant, board, export job, artifact class, and retention facts.
- Proto delta: internal presence service carries lease id and expiry, not durable payload history.

## Cedar Facts Exposed To Clients
- SDK request metadata must include `tenant_id`.
- SDK request metadata must include `principal_id`.
- SDK request metadata must include `audience_type`.
- SDK request metadata must include `purpose`.
- SDK request metadata must include `data_class`.
- SDK request metadata must include `capability`.
- SDK request metadata must include `board_id` only after tenant scope is already present.
- SDK request metadata must include `operation_id` for append decisions.
- SDK request metadata must include `snapshot_id` for history access decisions.
- SDK request metadata must include `artifact_id` for export download decisions.
- SDK request metadata must include `template_id` and `dealset_id` for template install decisions.
- SDK denial types must preserve Cedar policy id and decision correlation id.

## Workflow Decisions For Generated Examples
- Example 1 opens a board, receives a Cedar allow, appends a sticky-note operation, then observes an append event.
- Example 2 opens a board, receives a Cedar deny, and records refusal evidence without retrying.
- Example 3 joins a presence session, renews a lease, drops stale cursor state, and exits cleanly.
- Example 4 creates a history snapshot, polls the accepted job, and compares the snapshot to a prior pointer.
- Example 5 requests a PDF export, polls render progress, and downloads only after artifact authorization.
- Example 6 previews a Mural Enterprise-style facilitation template, settles DealSet, installs the template, and stores rollback token.
- Example 7 imports a FigJam-style board fixture, previews CRDT operation mapping, and rejects unmapped vendor permissions.
- Example 8 imports a Microsoft Whiteboard export fixture and demonstrates retention-safe export reissue.

## Benchmark Displacement Requirements
- Miro Enterprise displacement requires typed board open, append, export, template, and history calls.
- Mural Enterprise displacement requires workspace-style facilitation templates without adopting workspace as a service boundary.
- FigJam displacement requires presence and multiplayer append semantics that survive reconnect.
- Lucidspark displacement requires diagram-grade export and history evidence paths.
- Whiteboard.fi displacement requires classroom-style board sessions with explicit audience and data class controls.
- Microsoft Whiteboard displacement requires tenant-admin-safe export and retention posture.
- Each benchmark must appear in generated SDK documentation as a parity target, not a dependency.
- Each benchmark mapping must cite repo capability names, not vendor endpoint names.
- Each benchmark mapping must show what Oyatie refuses to clone where vendor behavior conflicts with ADR-0321.
- Each benchmark mapping must include a migration-safe example path.

## Evidence Gates
- Generated code must be reproducible from contract inputs.
- Generated code must be reviewed for vendor namespace leakage.
- Generated code must be reviewed for missing tenant scope.
- Generated code must be reviewed for missing principal scope.
- Generated code must be reviewed for missing data class.
- Generated code must be reviewed for missing purpose.
- Generated code must be reviewed for missing idempotency.
- Generated code must be reviewed for hidden retry behavior.
- Generated code must be reviewed for hidden global auth state.
- Generated code must be reviewed for board-id tenant inference.
- Generated code must be reviewed for untyped Cedar denial.
- Generated code must be reviewed for untyped replay conflict.
- Generated code must be reviewed for export artifact authorization.
- Generated code must be reviewed for template DealSet settlement handling.
- Generated code must be reviewed for presence lease expiry handling.
- Generated code must be reviewed for snapshot retention handling.

## Test Matrix
- Contract generation test: OpenAPI command methods exist for all six capabilities.
- Contract generation test: AsyncAPI event clients exist for append, presence, snapshot, and export progress.
- Contract generation test: proto3 internal clients exist only where the whiteboard architecture declares synchronous internal calls.
- Serialization test: `tenant_id` survives every request model.
- Serialization test: `principal_id` survives every mutation model.
- Serialization test: `purpose` survives every operation.
- Serialization test: `data_class` survives every operation.
- Serialization test: `audience_type` survives collaborative and auditor paths.
- Serialization test: DealSet settlement survives template install.
- Serialization test: trace context survives retries.
- Behavioral test: idempotent append retry reuses the same operation key.
- Behavioral test: non-idempotent operation refuses automatic retry.
- Behavioral test: presence reconnect renews lease before publishing cursor state.
- Behavioral test: export polling observes server backoff.
- Behavioral test: denied calls return typed Cedar refusal.
- Behavioral test: replay conflict returns typed conflict, not transport failure.
- Regression test: generated names contain no `Miro`, `Mural`, `FigJam`, `Lucidspark`, `WhiteboardFi`, or `MicrosoftWhiteboard` package identifiers.
- Regression test: benchmark names remain documentation strings only.

## Publication Rules
- SDK artifacts remain internal until all six capability paths pass generation and tests.
- SDK artifacts must include generated source hash references in release notes.
- SDK artifacts must include contract input versions in metadata.
- SDK artifacts must include ADR list in generated docs.
- SDK artifacts must include migration examples for displaced vendors.
- SDK artifacts must include a known refusal example for each public mutation family.
- SDK artifacts must include rollback guidance for generated breaking changes.
- SDK artifacts must include preview labels if any language target is incomplete.
- SDK artifacts must include no bundled credentials.
- SDK artifacts must include no tenant defaults.
- SDK artifacts must include no sample real tenant identifiers.
- SDK artifacts must include no vendor SDK transitive dependency unless explicitly approved elsewhere.

## Rollback
- Roll back generated SDK publication by withdrawing package metadata before deleting generated source.
- Roll back generated source by returning to the previous source hash and preserving contract inputs.
- Roll back examples independently when only documentation parity is wrong.
- Roll back language targets independently when one generator emits bad types.
- Roll back transport helpers if HTTP/3 or event-stream behavior drifts from contracts.
- Roll back template install methods if DealSet settlement metadata is missing.
- Roll back export helpers if artifact authorization cannot be proven.
- Roll back presence helpers if lease expiry behavior is ambiguous.
- Roll back append helpers if idempotency is not preserved.
- Roll back board-open helpers if tenant inference appears.

## Acceptance Criteria
- The SDK plan names Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard as displacement benchmarks.
- The SDK plan preserves ADR-0321 and the existing ADR binding set.
- Generated client scope is explicitly flat-service `whiteboard`.
- Generated client methods cover all six capability records.
- Generated client data models carry tenant, principal, audience, purpose, data class, trace context, and policy evidence.
- Generated client errors distinguish validation, Cedar refusal, quota refusal, replay conflict, transport failure, and export-artifact denial.
- Generated client examples show success and refusal paths.
- Generated client publication is blocked by contract drift, tenant drift, or vendor namespace leakage.
- Generated client rollback is described at package, language-target, and method-family levels.
- Evidence can be reviewed without editing ADR-0321.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
