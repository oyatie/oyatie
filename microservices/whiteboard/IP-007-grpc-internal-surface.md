# IP-007 Whiteboard grpc-internal-surface

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-007-grpc-internal-surface.md
Benchmarks displaced: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## 1. Outcome
- Build the internal gRPC surface that lets whiteboard usecases call each other without leaking HTTP DTOs into the application, usecase, domain, or kernel layers defined in microservices/whiteboard/ARCHITECTURE.md.
- Keep the public ingress contract in microservices/whiteboard/contracts/openapi-v1.yaml separate from internal service-to-service calls.
- Keep the event contract in microservices/whiteboard/contracts/asyncapi-v1.yaml separate from synchronous usecase decisions.
- Extend the existing proto anchor in microservices/whiteboard/contracts/whiteboard-v1.proto without changing this IP in isolation.
- Treat `WhiteboardService.InvokeAction` as the first coarse compatibility shim, not as the final internal surface.
- Split final RPCs around the six capability records under microservices/whiteboard/capabilities/.
- Preserve the PRD requirement that every command carries tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- Make internal transport boring: gRPC, proto3, unary for decision calls, streaming only where cursor/presence or replay windows demand it.
- Prevent B2B parity pressure from producing vendor-shaped RPCs.
- Name Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard only as displaced benchmarks.

## 2. Local Source Anchors
- microservices/whiteboard/PRD.md defines `canvas`, `board-session`, `sticky-note`, `template`, and `export` as the bounded contexts.
- microservices/whiteboard/ARCHITECTURE.md maps ADR-0105 layers and states that adapters, storage, queues, and evidence stay outside domain/core.
- microservices/whiteboard/contracts/whiteboard-v1.proto currently exposes `WhiteboardActionRequest`, `WhiteboardActionAccepted`, and `WhiteboardService.InvokeAction`.
- microservices/whiteboard/capabilities/board-open.yaml declares `whiteboard-board-open`.
- microservices/whiteboard/capabilities/canvas-op-append.yaml declares `whiteboard-canvas-op-append`.
- microservices/whiteboard/capabilities/presence-sync.yaml declares `whiteboard-presence-sync`.
- microservices/whiteboard/capabilities/history-snapshot.yaml declares `whiteboard-history-snapshot`.
- microservices/whiteboard/capabilities/export-render.yaml declares `whiteboard-export-render`.
- microservices/whiteboard/capabilities/template-marketplace-install.yaml declares `whiteboard-template-marketplace-install`.
- microservices/whiteboard/policy/canvas-collaboration-authorization.cedar is the authorization companion.
- microservices/whiteboard/policies/local-board-open-scope.cedar is the local board-open scope guard.
- microservices/whiteboard/slos/local-board-load-time.openslo.yaml supplies board load expectations.
- microservices/whiteboard/slos/local-stroke-persistence-latency.openslo.yaml supplies append latency expectations.
- microservices/whiteboard/slos/local-cursor-latency.openslo.yaml supplies cursor latency expectations.
- microservices/whiteboard/slos/replay-freshness.openslo.yaml supplies replay freshness expectations.
- microservices/whiteboard/dashboards/local-domain-throughput.json is the primary internal throughput evidence surface.
- microservices/whiteboard/runbooks/local-collaboration-acl-mismatch.md is the first runbook for policy/context mismatches.

## 3. Benchmark Displacement
- Miro Enterprise pressure: multiplayer canvas actions must feel low-latency while retaining tenant and audit scope.
- Mural Enterprise pressure: facilitation sessions need explicit board membership, guest scopes, and template provenance.
- FigJam pressure: cursors, reactions, and lightweight sticky operations must not round-trip through heavyweight document semantics.
- Lucidspark pressure: exports and board snapshots need deterministic rendering references and replayable state.
- Whiteboard.fi pressure: classroom-style ephemeral boards need short-lived, bounded participant authority.
- Microsoft Whiteboard pressure: Microsoft 365-style sharing links must be translated into Oyatie principal and tenant grants.
- The internal surface does not copy any vendor object model.
- The internal surface maps vendor expectations into Oyatie capabilities, Cedar actions, ontology objects, audit events, and workflow runs.
- ADR-0321 requires vendor-specific substance; this IP supplies that by naming concrete benchmark pressures and rejecting vendor-shaped services.
- ADR-0316 keeps product labels out of service boundaries.

## 4. RPC Families
- `BoardOpenService` owns synchronous checks for board session admission.
- `CanvasOpService` owns append admission and idempotent persistence acceptance.
- `PresenceService` owns bounded cursor and participant freshness calls.
- `HistoryService` owns history snapshot requests and replay window lookup.
- `ExportService` owns export render admission and status lookup.
- `TemplateInstallService` owns template marketplace install admission.
- Each family must keep command-specific messages instead of generic string maps.
- Each family must carry `tenant_id`.
- Each family must carry `principal_id`.
- Each family must carry `audience_type`.
- Each family must carry `purpose`.
- Each family must carry `data_class`.
- Each family must carry `home_cell`.
- Each family must carry `jurisdiction_code`.
- Each family must carry `pack_overlay_id`.
- Each family must carry `deal_set_id` when commercial rights or template licensing are involved.
- Each family must carry `trace_id`.
- Each family must carry `span_id` or parent trace reference.
- Each family must carry `idempotency_key` for mutation-like decisions.
- Each family must carry `cedar_decision_id` after policy evaluation.
- Each family must carry `audit_chain_ref`.
- Each family must carry `workflow_run_id` when invoked by workflow-engine.

## 5. Message Shape
- Prefer `BoardRef` with `board_id`, `tenant_id`, `home_cell`, and `region_affinity`.
- Prefer `CanvasOpRef` with `op_id`, `board_id`, `client_sequence`, and `operation_kind`.
- Prefer `PresenceRef` with `participant_id`, `cursor_epoch`, and `connection_id`.
- Prefer `HistoryWindowRef` with `board_id`, `from_revision`, `to_revision`, and `snapshot_id`.
- Prefer `ExportRef` with `export_id`, `render_profile`, and `artifact_class`.
- Prefer `TemplateInstallRef` with `template_id`, `deal_set_id`, and `license_scope`.
- Avoid opaque `action` strings except in the compatibility shim.
- Avoid embedding REST paths in proto messages.
- Avoid embedding AsyncAPI channel names in proto messages.
- Avoid accepting vendor ids without a parallel `source_system_ref`.
- Keep vendor ids in provenance fields, never as primary keys.
- Use `ontology_object_ref` for durable projection references.
- Use `audit_event_class` for evidence classification.
- Use `policy_context_hash` for decision reproducibility.
- Use `pack_overlay_hash` for pack resolver reproducibility.

## 6. Service Boundaries
- Internal gRPC callers are application/usecase adapters, not domain objects.
- Domain commands accept value objects derived from proto messages.
- Kernel functions never import generated gRPC types.
- REST handlers translate OpenAPI DTOs to application commands before any gRPC handoff.
- Worker handlers translate AsyncAPI events to application commands before any gRPC handoff.
- gRPC clients must run behind adapter interfaces in the ADR-0105 adapter layer.
- gRPC servers must call usecase handlers, not storage adapters directly.
- Storage failures become typed application errors before they cross the gRPC boundary.
- Policy failures become typed denial results before they cross the gRPC boundary.
- Audit-chain failures become typed paused/degraded results before they cross the gRPC boundary.
- Marketplace settlement failures become typed commercial-hold results before they cross the gRPC boundary.

## 7. Board Open RPC
- Board open supports the `whiteboard-board-open` capability.
- The request names `board_id`, `tenant_id`, `principal_id`, and `audience_type`.
- The request carries `entry_source` such as direct URL, workflow task, embedded meet session, or marketplace template preview.
- The request carries `guest_invite_ref` only when the participant is not a tenant member.
- Cedar must evaluate before board metadata is returned.
- The response returns `board_revision`.
- The response returns `allowed_capabilities`.
- The response returns `presence_endpoint_ref` only after authorization.
- The response returns `audit_chain_ref`.
- Denials return a refusal code usable by microservices/whiteboard/runbooks/local-collaboration-acl-mismatch.md.
- Miro Enterprise and Microsoft Whiteboard link-share parity is satisfied only when share links become explicit principal grants.
- Whiteboard.fi classroom parity is satisfied only when class roster grants expire without manual cleanup.

## 8. Canvas Op Append RPC
- Canvas op append supports the `whiteboard-canvas-op-append` capability.
- The request names `board_id`, `op_id`, `client_sequence`, and `operation_kind`.
- The request carries an operation payload reference, not arbitrary unbounded inline JSON.
- The request carries `conflict_basis_revision`.
- The request carries `client_clock_ms` for skew analysis, not authority.
- The response returns `accepted_revision`.
- The response returns `merge_strategy`.
- The response returns `replay_cursor`.
- The response returns `stroke_persistence_budget_ms`.
- Denials distinguish stale revision, policy denial, abuse throttle, and storage backpressure.
- FigJam and Miro Enterprise parity requires fluid appends, but Oyatie still fails closed on missing tenant scope.
- Mural Enterprise parity requires facilitator locks, but locks are policy contexts rather than hidden board state.

## 9. Presence RPC
- Presence supports the `whiteboard-presence-sync` capability.
- The request names `connection_id`, `participant_id`, and `cursor_epoch`.
- The request carries viewport bounds only at coarse granularity.
- The request carries tool mode only when needed for collaboration affordances.
- The request never carries raw text selection content.
- The response returns freshness status.
- The response returns `fanout_partition`.
- The response returns `throttle_tier`.
- The response returns `presence_audit_sampled` when audit sampling applies.
- Streaming presence must have explicit server-side max duration.
- FigJam parity requires cursor smoothness.
- Microsoft Whiteboard parity requires tenant guest state.
- Whiteboard.fi parity requires classroom owner visibility into transient participants.

## 10. History Snapshot RPC
- History snapshot supports the `whiteboard-history-snapshot` capability.
- The request names `board_id`, `from_revision`, `to_revision`, and `snapshot_reason`.
- The request carries `requested_by_workflow_run_id` when invoked by workflow-engine.
- The request carries `retention_pack_id` when pack overlays alter retention.
- The response returns `snapshot_id`.
- The response returns `snapshot_revision`.
- The response returns `replay_freshness_budget_ms`.
- The response returns `rollback_bundle_ref`.
- Lucidspark and Miro Enterprise parity requires replayable board history, not vendor-copied revision graphs.
- Snapshot reads must stay in the tenant home cell unless microservices/whiteboard/multi-region.md permits metadata-only behavior.

## 11. Export Render RPC
- Export render supports the `whiteboard-export-render` capability.
- The request names `export_id`, `board_id`, `render_profile`, and `artifact_class`.
- The request carries `watermark_policy_ref`.
- The request carries `data_residency_pack_id`.
- The request carries `deal_set_id` for commercially licensed template content.
- The response returns `artifact_ref`.
- The response returns `render_revision`.
- The response returns `audit_event_class`.
- The response returns `export_render_budget_ms`.
- Lucidspark export parity requires deterministic diagram snapshots.
- Mural Enterprise export parity requires facilitation artifact provenance.
- Microsoft Whiteboard export parity requires sharing-source preservation.

## 12. Template Install RPC
- Template install supports the `whiteboard-template-marketplace-install` capability.
- The request names `template_id`, `tenant_id`, `principal_id`, and `deal_set_id`.
- The request carries `source_vendor_benchmark` only as benchmark provenance.
- The request carries `license_scope`.
- The request carries `install_target_board_id` when installation happens into an existing board.
- The response returns `installed_template_ref`.
- The response returns `settlement_ref`.
- The response returns `workflow_run_id`.
- Miro Enterprise and Mural Enterprise template parity requires reusable facilitation templates.
- Oyatie acceptance requires marketplace DealSet settlement per ADR-0314.

## 13. Errors
- `TENANT_SCOPE_MISSING` means mandatory tenant fields were absent.
- `CEDAR_DENIED` means policy evaluation refused the action.
- `CEDAR_UNAVAILABLE_FAIL_CLOSED` means mutation cannot proceed.
- `PACK_OVERLAY_CONFLICT` means the pack resolver found incompatible obligations.
- `DEAL_SET_HOLD` means marketplace settlement blocks the action.
- `HOME_CELL_MISMATCH` means the request targeted the wrong cell.
- `AUDIT_CHAIN_PAUSED` means evidence cannot be sealed for a critical mutation.
- `IDEMPOTENCY_REPLAYED` means the previous accepted response was returned.
- `ABUSE_THROTTLED` means IP-012 controls blocked or slowed the action.
- `PRESENCE_STREAM_EXPIRED` means the bounded stream lifetime ended.
- `SNAPSHOT_WINDOW_INVALID` means the requested history interval cannot be proven.
- `EXPORT_RENDER_BLOCKED` means render policy, residency, or artifact controls refused output.

## 14. Security
- Require mTLS with SPIFFE identity between internal callers.
- Require TLS 1.3 floor under ADR-0253-amendment transport expectations.
- Prefer HTTP/3 externally, but keep internal gRPC deployment compatible with cell ingress policy.
- Do not pass OpenBao secret material through proto messages.
- Pass SecretReference ids only when a server-side adapter needs a credential.
- Bind every server method to Cedar decision context or a prior decision id.
- Do not accept caller-provided `cedar_decision_id` without verifying the policy context hash.
- Do not log raw canvas operation payloads.
- Do not log raw cursor payloads.
- Keep tenant id in audit evidence and use bounded cardinality labels in metrics.

## 15. Observability
- Emit `whiteboard.grpc.request.count` by method, result, cell, capability, and data class.
- Emit `whiteboard.grpc.request.duration` by method, result, and cell.
- Emit `whiteboard.grpc.denial.count` by denial code and capability.
- Emit `whiteboard.grpc.idempotency.replay.count` for duplicate mutation attempts.
- Emit `whiteboard.grpc.policy_context_mismatch.count` when decision hashes fail.
- Trace every internal call with parent request or workflow span.
- Link traces to audit events through `audit_chain_ref`.
- Link traces to runbooks through typed error codes.
- Link board open failures to microservices/whiteboard/runbooks/local-board-load-burn.md when SLO burn triggers.
- Link canvas append failures to microservices/whiteboard/runbooks/local-stroke-persistence-lag.md when latency burn triggers.
- Link cursor failures to microservices/whiteboard/runbooks/local-cursor-latency-burn.md when latency burn triggers.

## 16. Compatibility Plan
- Phase 1 keeps `WhiteboardService.InvokeAction` as a coarse shim.
- Phase 2 adds capability-specific RPCs beside the shim.
- Phase 3 migrates REST and worker adapters to capability-specific clients.
- Phase 4 marks the shim internal-deprecated.
- Phase 5 removes the shim only after replay, contract, and dashboard evidence prove no callers remain.
- Compatibility must be tracked through microservices/whiteboard/sdk-plan.md when client generation is in scope.
- Catalog records under microservices/whiteboard/catalog/ must name the final internal API owner when IP-020 lands.

## 17. Tests
- Proto lint validates package name `oyatie.whiteboard.v1`.
- Contract tests validate each mandatory tenant and trace field.
- Authorization tests prove Cedar denial occurs before usecase execution.
- Idempotency tests replay identical append and export requests.
- Replay tests rebuild a board from accepted `canvas-op-append` responses.
- Presence tests enforce stream duration and throttle behavior.
- Export tests assert deterministic artifact refs for identical board revisions.
- Template tests assert DealSet hold behavior.
- Cross-cell tests assert home-cell mismatch behavior.
- Audit tests assert `audit_chain_ref` on every accepted mutation.
- Benchmark parity tests name Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard as displaced sources.

## 18. Acceptance Criteria
- The gRPC design covers all six local capability records.
- No final RPC is vendor-named.
- No final message uses raw untyped action strings except the temporary shim.
- All mutation-like RPCs include idempotency.
- All policy-dependent RPCs bind a Cedar decision context.
- All commercial template/export paths bind DealSet settlement.
- All history/export paths bind audit and rollback evidence.
- All presence paths define bounded streaming behavior.
- All errors map to observable denial or degraded evidence.
- ADR-0321 remains cited and the IP contains vendor-specific displacement substance.


## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
