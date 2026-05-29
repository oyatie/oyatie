# IP-005 Whiteboard REST Contract Surface

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-005-rest-contract-surface.md
Planning lane: B2B-leader IP substance deepening pass
Primary concern: OpenAPI-facing command and query surface for tenant-scoped whiteboard collaboration
Contract references: microservices/whiteboard/contracts/openapi-v1.yaml; microservices/whiteboard/contracts/local-openapi-v1.yaml
Policy references: microservices/whiteboard/policy/canvas-collaboration-authorization.cedar; microservices/whiteboard/policies/local-board-open-scope.cedar; microservices/whiteboard/policies/local-board-export-egress.cedar
SLO references: microservices/whiteboard/slos/local-board-load-time.openslo.yaml; microservices/whiteboard/slos/read-latency.openslo.yaml; microservices/whiteboard/slos/write-latency.openslo.yaml; microservices/whiteboard/slos/local-export-render-latency.openslo.yaml
Capability references: whiteboard-board-open; whiteboard-canvas-op-append; whiteboard-presence-sync; whiteboard-history-snapshot; whiteboard-export-render; whiteboard-template-marketplace-install
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Benchmark displacement set: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard

## Executive Intent
- The REST surface is the stable external contract for board opening, board metadata, session starts, snapshots, exports, and template installs.
- High-rate canvas operations and presence may rely on async or streaming surfaces, but REST still owns initiation, status, idempotency, and evidence retrieval.
- The contract must make Oyatie's tenant, policy, DealSet, ontology, and audit model visible rather than copying vendor API assumptions.
- ADR-0321 requires credible B2B leader displacement; that means the REST API must model enterprise admin, education, diagramming, migration, and meeting-linked workflows explicitly.
- ADR-0253-amendment requires the REST surface to remain compatible with HTTP/3, ECH, and PQC transport posture.
- ADR-0244 requires contract-first implementation with OpenAPI 3.2.0.
- ADR-0314 requires marketplace template install endpoints to enforce DealSet settlement.
- ADR-0105 requires REST handlers to remain adapters and avoid owning domain decisions.
- The REST surface must return deterministic problem codes for scope, policy, residency, retention, idempotency, and settlement failures.
- The surface must be narrow enough for implementation but complete enough for SDK generation and contract tests.

## Route Inventory
- POST /v1/whiteboard/boards opens or creates a tenant-scoped board.
- GET /v1/whiteboard/boards/{board_id} reads board metadata after policy authorization.
- POST /v1/whiteboard/boards/{board_id}/sessions starts or joins a board session.
- GET /v1/whiteboard/boards/{board_id}/sessions/{session_id} reads session metadata.
- POST /v1/whiteboard/boards/{board_id}/operations appends a low-volume or fallback canvas operation.
- GET /v1/whiteboard/boards/{board_id}/operations/{operation_id} reads operation status.
- POST /v1/whiteboard/boards/{board_id}/snapshots requests a history snapshot.
- GET /v1/whiteboard/boards/{board_id}/snapshots/{snapshot_id} reads snapshot status.
- POST /v1/whiteboard/boards/{board_id}/exports requests an export render.
- GET /v1/whiteboard/boards/{board_id}/exports/{export_id} reads export status and authorized artifact metadata.
- POST /v1/whiteboard/boards/{board_id}/templates installs a template into a board.
- GET /v1/whiteboard/boards/{board_id}/templates/{template_install_id} reads template install status.
- POST /v1/whiteboard/imports starts benchmark or vendor board import.
- GET /v1/whiteboard/imports/{import_id} reads import status.
- POST /v1/whiteboard/replays starts a board replay or repair workflow.
- GET /v1/whiteboard/replays/{replay_id} reads replay status.
- GET /v1/whiteboard/catalog/templates lists allowed workflow templates.
- GET /v1/whiteboard/evidence/{evidence_id} reads audit-safe evidence bundles.
- GET /v1/whiteboard/health reads service health without exposing tenant data.
- GET /v1/whiteboard/capabilities reads capability metadata safe for SDK discovery.

## Common Headers
- X-Oyatie-Tenant-Id is required for tenant-scoped routes.
- X-Oyatie-Principal-Id is required for caller-scoped routes.
- X-Oyatie-Audience-Type is required and normally COLLABORATION_USER.
- X-Oyatie-Purpose is required for commands and policy-visible reads.
- X-Oyatie-Data-Class is required for commands and policy-visible reads.
- Idempotency-Key is required for POST commands that mutate state.
- Traceparent is accepted and propagated.
- Tracestate is accepted and propagated.
- X-Oyatie-Request-Cell is required when cell routing is not implicit.
- X-Oyatie-Policy-Pack-Set is optional when derived from tenant configuration.
- X-Oyatie-Source-Vendor is required for import routes and optional for migration status reads.
- X-Oyatie-DealSet-Id is required for marketplace template material.
- X-Oyatie-Residency-Zone is required for exports when not derivable.
- X-Oyatie-SDK-Version is optional for client compatibility telemetry.
- X-Oyatie-Contract-Version is optional but must be validated when present.
- Authorization is required unless a local system route explicitly allows mTLS-only worker access.
- Accept-Language is optional display preference and never policy authority.
- User-Agent is telemetry only and never policy authority.
- X-Forwarded-For is not trusted for policy.
- X-Oyatie-Emergency-Bypass is rejected unless emergency-services policy owns the route.

## Request Schema Requirements
- BoardCreateRequest includes title.
- BoardCreateRequest includes initial_template_id when creating from a template.
- BoardCreateRequest includes policy_pack_set when overriding tenant defaults.
- BoardCreateRequest includes source_vendor and source_object_id when created from import.
- BoardCreateRequest includes audit_chain_target.
- BoardSessionRequest includes board_id.
- BoardSessionRequest includes requested_role.
- BoardSessionRequest includes meeting_binding when replacing Microsoft Whiteboard meeting-linked behavior.
- BoardSessionRequest includes roster_binding when replacing Whiteboard.fi classroom behavior.
- CanvasOperationRequest includes operation_id.
- CanvasOperationRequest includes operation_kind.
- CanvasOperationRequest includes object_id when mutating existing object.
- CanvasOperationRequest includes idempotency_key or relies on Idempotency-Key header.
- CanvasOperationRequest includes operation_payload with size and data-class limits.
- SnapshotRequest includes snapshot_reason.
- SnapshotRequest includes operation_range or latest marker.
- SnapshotRequest includes retention_class.
- ExportRenderRequest includes export_format.
- ExportRenderRequest includes snapshot_id or board_id.
- ExportRenderRequest includes residency_zone.
- ExportRenderRequest includes retention_class.
- TemplateInstallRequest includes template_source.
- TemplateInstallRequest includes template_id.
- TemplateInstallRequest includes marketplace_dealset_id when source is marketplace.
- ImportRequest includes source_vendor.
- ImportRequest includes source_object_id.
- ImportRequest includes migration_mode.
- ImportRequest includes provenance_preservation=true by default.

## Response Schema Requirements
- Every accepted command response includes request_id.
- Every accepted command response includes workflow_instance_id when workflow-backed.
- Every accepted command response includes scope_hash.
- Every accepted command response includes policy_decision_ref.
- Every accepted command response includes audit_event_ref.
- Every accepted command response includes ontology_projection_ref when projection exists.
- Every accepted command response includes status.
- Every accepted command response includes links for status polling.
- Every status response includes terminal_state when complete.
- Every status response includes retry_after when work is pending.
- Every status response includes runbook_ref when operator action is needed.
- Every status response includes source_vendor for imports.
- Every status response includes source_object_id for imports.
- Every export status response includes artifact metadata only after policy authorization.
- Every export status response includes residency and retention evidence refs.
- Every template status response includes DealSet evidence refs when applicable.
- Every replay status response includes replay_mode.
- Every replay status response includes scope_version.
- Every error response uses application/problem+json.
- Every error response includes stable code.
- Every error response includes trace_id.
- Every error response excludes raw canvas payloads.

## Problem Code Families
- whiteboard.scope.missing_tenant maps to 422.
- whiteboard.scope.missing_principal maps to 422.
- whiteboard.scope.missing_purpose maps to 422.
- whiteboard.scope.missing_data_class maps to 422.
- whiteboard.scope.cell_mismatch maps to 403.
- whiteboard.scope.board_tenant_mismatch maps to 404 or 403 according to information disclosure policy.
- whiteboard.policy.denied maps to 403.
- whiteboard.policy.bundle_unavailable maps to 503.
- whiteboard.idempotency.conflict maps to 409.
- whiteboard.session.closed maps to 409.
- whiteboard.operation.duplicate maps to 409.
- whiteboard.operation.payload_too_large maps to 413.
- whiteboard.export.residency_denied maps to 403.
- whiteboard.export.retention_conflict maps to 409.
- whiteboard.template.dealset_missing maps to 403.
- whiteboard.template.dealset_pending maps to 409.
- whiteboard.import.source_vendor_unknown maps to 422.
- whiteboard.import.source_object_missing maps to 422.
- whiteboard.replay.scope_hash_mismatch maps to 409.
- whiteboard.rate_limited maps to 429.
- whiteboard.internal maps to 500.

## Benchmark Contract Behavior
- Miro Enterprise imports require source_vendor=Miro Enterprise.
- Miro Enterprise imports require source_workspace_id when available.
- Miro Enterprise board links are normalized to source_object_id and never trusted as tenant authority.
- Mural Enterprise imports require source_vendor=Mural Enterprise.
- Mural Enterprise facilitator metadata maps to session role claims in response evidence.
- Mural Enterprise room identifiers are provenance, not authorization.
- FigJam imports require source_vendor=FigJam.
- FigJam widget imports require template or app provenance fields.
- FigJam live collaboration should prefer async or streaming for high-rate operations while REST remains fallback.
- Lucidspark imports require source_vendor=Lucidspark.
- Lucidspark connector mutations must include endpoint object ids.
- Lucidspark export requests must declare diagram export format and residency zone.
- Whiteboard.fi flows require source_vendor=Whiteboard.fi for imports.
- Whiteboard.fi classroom setup requires roster_binding or education pack evidence.
- Whiteboard.fi student board export requires retention_class.
- Microsoft Whiteboard flows require source_vendor=Microsoft Whiteboard for imports.
- Microsoft Whiteboard meeting-linked setup requires meeting_binding.
- Microsoft Whiteboard tenant export cannot omit selected board ids.
- All benchmark imports preserve source_object_id in status responses.
- All benchmark routes return benchmark_name only as provenance, never as service boundary.

## Idempotency And Concurrency
- POST board creation uses Idempotency-Key scoped by tenant_id and principal_id.
- POST session start uses Idempotency-Key scoped by tenant_id, board_id, and principal_id.
- POST operation append uses operation_id plus Idempotency-Key.
- POST snapshot request uses Idempotency-Key scoped by board_id and requested operation range.
- POST export request uses Idempotency-Key scoped by snapshot_id or board_id plus export format.
- POST template install uses Idempotency-Key scoped by board_id and template_id.
- POST import uses Idempotency-Key scoped by source_vendor and source_object_id.
- Duplicate idempotency with identical body returns prior accepted response.
- Duplicate idempotency with different body returns whiteboard.idempotency.conflict.
- Concurrent session joins must be safe and return existing membership when equivalent.
- Concurrent canvas operations must not reorder accepted operation ids.
- Concurrent template installs must not duplicate template material.
- Concurrent exports must reuse existing pending export when identical.
- Concurrent imports must deduplicate by source_vendor and source_object_id.
- Retryable 503 responses include Retry-After.
- Rate-limited 429 responses include Retry-After.
- Client cancellation never deletes accepted audit evidence.
- Server timeout does not imply rollback unless terminal failure is emitted.
- Status polling is the canonical resolution path after ambiguous client timeout.
- SDKs must expose idempotency behavior in typed clients.

## Security And Transport
- REST endpoints require TLS posture aligned with HTTP/3 h3-alt-svc, ECH, and PQC rollout.
- REST authorization never trusts transport identity alone for tenant collaboration.
- REST handlers validate content-type.
- REST handlers enforce payload size limits by route.
- REST handlers reject unknown source_vendor values.
- REST handlers reject missing DealSet identifiers for marketplace installs.
- REST handlers reject export artifact reads without policy authorization.
- REST handlers redact raw canvas payloads from logs.
- REST handlers propagate trace context.
- REST handlers emit audit events for accepted commands.
- REST handlers emit audit events for policy denials.
- REST handlers emit audit events for export egress decisions.
- REST handlers separate health routes from tenant routes.
- REST handlers do not expose raw policy internals.
- REST handlers do not expose raw vendor tokens.
- REST handlers do not expose raw import payloads by default.
- REST handlers do not mutate state after response failure without outbox evidence.
- REST handlers prefer async status for long-running import, replay, snapshot, and export.
- REST handlers require SDK-generated clients to preserve header names.
- REST handlers support contract version negotiation without silent downgrade.
- REST handlers keep OpenAPI examples aligned with displaced benchmark names.

## Implementation Steps
- Update openapi-v1.yaml with route inventory from this IP.
- Update local-openapi-v1.yaml with local operations and status routes.
- Define shared BoardScopeHeaders component.
- Define shared TraceHeaders component.
- Define shared IdempotencyHeaders component.
- Define BoardCreateRequest schema.
- Define BoardSessionRequest schema.
- Define CanvasOperationRequest schema.
- Define SnapshotRequest schema.
- Define ExportRenderRequest schema.
- Define TemplateInstallRequest schema.
- Define ImportRequest schema.
- Define ReplayRequest schema.
- Define CommandAcceptedResponse schema.
- Define StatusResponse schema.
- Define EvidenceResponse schema.
- Define WhiteboardProblem schema.
- Add examples for Miro Enterprise import.
- Add examples for Mural Enterprise workshop session.
- Add examples for FigJam brainstorm import.
- Add examples for Lucidspark diagram export.
- Add examples for Whiteboard.fi classroom setup.
- Add examples for Microsoft Whiteboard meeting-linked board.
- Generate SDK plan updates without adding dependencies in this IP.
- Add contract tests for required headers.
- Add contract tests for stable problem codes.
- Add contract tests for benchmark examples.
- Add contract tests for DealSet requirement.
- Add contract tests for export residency requirement.
- Add route-to-capability mapping tests.
- Add audit event expectation tests.

## Observability
- Emit whiteboard_rest_request_total by route, method, status, capability, and source_vendor.
- Emit whiteboard_rest_request_seconds by route, method, status, and contract_version.
- Emit whiteboard_rest_problem_total by problem_code and route.
- Emit whiteboard_rest_idempotency_total by route and result.
- Emit whiteboard_rest_benchmark_import_total by source_vendor.
- Emit trace span whiteboard.rest.parse_headers.
- Emit trace span whiteboard.rest.validate_body.
- Emit trace span whiteboard.rest.build_scope.
- Emit trace span whiteboard.rest.invoke_usecase.
- Emit trace span whiteboard.rest.write_response.
- Emit audit event whiteboard.rest.command_accepted.
- Emit audit event whiteboard.rest.command_rejected.
- Emit audit event whiteboard.rest.status_read.
- Emit dashboard fields compatible with local-slo-burn and local-domain-throughput dashboards.
- Attach request_id to every log line.
- Attach trace_id to every problem response.
- Attach scope_hash to accepted mutation logs.
- Attach source_vendor only when present.
- Attach contract_version when provided.
- Attach SDK version when provided.
- Avoid logging raw canvas operation payloads.

## Test Plan
- Contract test all mutation routes require X-Oyatie-Tenant-Id.
- Contract test all mutation routes require X-Oyatie-Principal-Id.
- Contract test all mutation routes require X-Oyatie-Purpose.
- Contract test all mutation routes require X-Oyatie-Data-Class.
- Contract test mutation routes require Idempotency-Key.
- Contract test import route requires source_vendor.
- Contract test import route accepts Miro Enterprise.
- Contract test import route accepts Mural Enterprise.
- Contract test import route accepts FigJam.
- Contract test import route accepts Lucidspark.
- Contract test import route accepts Whiteboard.fi.
- Contract test import route accepts Microsoft Whiteboard.
- Contract test import route rejects generic Miro.
- Contract test import route rejects generic Lucid.
- Contract test template install requires DealSet when marketplace source is used.
- Contract test export request requires residency_zone.
- Contract test export status hides artifact metadata without policy allow.
- Contract test problem responses use application/problem+json.
- Contract test problem codes are stable.
- Contract test status response includes workflow_instance_id.
- Contract test accepted response includes scope_hash.
- Contract test accepted response includes policy_decision_ref.
- Contract test accepted response includes audit_event_ref.
- Contract test examples validate against OpenAPI 3.2.0.
- Integration test board-open route maps to board-open capability.
- Integration test canvas-op fallback maps to canvas-op-append capability.
- Integration test snapshot route maps to history-snapshot capability.
- Integration test export route maps to export-render capability.
- Integration test template route maps to template-marketplace-install capability.
- Observability test emits REST metrics and audit events.
- SLO test simple board-open remains inside read latency budget.

## Acceptance Criteria
- OpenAPI routes cover board, session, operation fallback, snapshot, export, template, import, replay, evidence, health, and capability discovery.
- Every mutation route requires tenant, principal, purpose, data class, and idempotency.
- Every benchmark import uses the displaced benchmark names.
- Every marketplace template route requires DealSet evidence.
- Every export route enforces residency and retention inputs.
- Every accepted command response includes scope, policy, audit, and status evidence.
- Every problem response uses stable problem codes.
- Every route maps to the capability records or workflow templates named in this IP.
- Every route has contract tests.
- Every route has observability dimensions.
- No REST handler owns domain authorization directly.
- No REST route trusts source vendor workspace, room, file, class, or meeting as tenant authority.
- No REST contract edit requires changing ADR-0321.

## Title-Specific Command, Event, And Proto Deltas
- BoardOpenCommand is the REST command for POST /v1/whiteboard/boards when opening or creating a board.
- BoardSessionCommand is the REST command for POST /v1/whiteboard/boards/{board_id}/sessions.
- CanvasOperationCommand is the REST fallback command for POST /v1/whiteboard/boards/{board_id}/operations.
- HistorySnapshotCommand is the REST command for POST /v1/whiteboard/boards/{board_id}/snapshots.
- ExportRenderCommand is the REST command for POST /v1/whiteboard/boards/{board_id}/exports.
- TemplateInstallCommand is the REST command for POST /v1/whiteboard/boards/{board_id}/templates.
- VendorImportCommand is the REST command for POST /v1/whiteboard/imports.
- BoardReplayCommand is the REST command for POST /v1/whiteboard/replays.
- BoardOpenCommand emits board.session.requested when a session starts.
- CanvasOperationCommand emits canvas.operation.accepted or canvas.operation.rejected.
- HistorySnapshotCommand emits history.snapshot.requested.
- ExportRenderCommand emits export.render.requested.
- TemplateInstallCommand emits template.install.requested.
- VendorImportCommand emits vendor.import.requested.
- BoardReplayCommand emits board.replay.requested.
- whiteboard-v1.proto mirrors REST command ids so internal workers can correlate command, event, and status.
- local-operations-v1.proto receives worker commands only after REST acceptance and outbox persistence.
- Proto status messages must map back to REST StatusResponse without lossy conversion.
- REST evidence links must include async event ids when command completion is asynchronous.
- REST examples must show command-to-event correlation for at least one operation, export, import, and replay.

## Title-Specific Canvas, CRDT, And Session Facts
- REST board-open does not materialize CRDT state unless the board metadata read is authorized.
- REST session start validates facilitator and participant intent before presence fanout.
- REST canvas operation fallback carries CRDT operation_kind, operation_id, object_id, and idempotency_key.
- REST canvas operation fallback rejects connector mutations without endpoint ids.
- REST canvas operation fallback rejects frame membership changes without parent object ids.
- REST snapshot request specifies operation range so CRDT replay can be bounded.
- REST export request declares snapshot_id or board-current rendering mode.
- REST replay request declares replay_mode historical or current-policy.
- REST import request declares whether source material is board, template, diagram, class board, or meeting board.
- REST Miro Enterprise import carries source workspace and board provenance.
- REST Mural Enterprise import carries room and facilitator provenance.
- REST FigJam import carries file, section, widget, and comment provenance when present.
- REST Lucidspark import carries diagram shape and connector provenance.
- REST Whiteboard.fi import carries class and roster provenance.
- REST Microsoft Whiteboard import carries meeting and tenant-board provenance.
- REST status reads expose CRDT merge state without exposing raw operation payloads.
- REST status reads expose session state without exposing unauthorized participant details.
- REST status reads expose export artifact metadata only after policy allow.
- REST status reads expose replay progress with operation counts and failure runbook refs.
- REST status reads expose template install materialization counts and DealSet evidence.

## Title-Specific Cedar, SLO, And Evidence Gates
- Every REST mutation builds BoardScope before policy.
- Every REST mutation maps policy denial to stable problem+json.
- Every REST read that exposes tenant data requires policy allow.
- Every REST export artifact read requires export egress policy allow.
- Every REST template marketplace install requires DealSet evidence before policy allow.
- Board-open REST latency is measured against local-board-load-time.
- Canvas operation fallback latency is measured against write-latency and local-stroke-persistence-latency.
- Session start latency is measured against read-latency and local-presence-freshness setup.
- Snapshot request latency is measured separately from snapshot materialization.
- Export request latency is measured separately from local-export-render-latency.
- REST audit emission is measured against audit-emission-lag.
- Evidence fields include route_id, method, command_id, request_id, scope_hash, policy_decision_ref, audit_event_ref, workflow_instance_id, and async_event_id.
- Evidence fields include source_vendor and source_object_id for import routes.
- Evidence fields include marketplace_dealset_id for template routes.
- Evidence fields include residency_zone and retention_class for export and snapshot routes.

## Rollback
- Roll back route publication before changing shared schemas.
- Keep problem codes stable once SDKs consume them.
- Keep benchmark examples once published.
- Keep idempotency semantics once clients rely on them.
- Keep accepted response evidence fields once audit workflows consume them.
- Disable routes through capability gating rather than removing schemas.
- Route export failures to export-render-failure.
- Route import failures to template-import-rollback or local-regional-board-replay.
- Treat removal of DealSet requirements as a governance escalation.
- Treat broad tenant export without board scoping as a security escalation.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
