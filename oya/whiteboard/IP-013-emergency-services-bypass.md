# IP-013 Whiteboard emergency-services-bypass

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-013-emergency-services-bypass.md
Benchmarks: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo-local references: microservices/whiteboard/PRD.md, microservices/whiteboard/ARCHITECTURE.md, microservices/whiteboard/capabilities/board-open.yaml, microservices/whiteboard/capabilities/canvas-op-append.yaml, microservices/whiteboard/capabilities/presence-sync.yaml, microservices/whiteboard/capabilities/history-snapshot.yaml, microservices/whiteboard/capabilities/export-render.yaml, microservices/whiteboard/capabilities/template-marketplace-install.yaml, microservices/whiteboard/incident-response.md, microservices/whiteboard/runbooks, microservices/whiteboard/policies, microservices/whiteboard/threat-model.md

## Objective
- Replace the generic emergency path with a narrow safety bypass for verified emergency-service boards.
- Preserve Whiteboard's tenant scope, Cedar gates, ontology projection, audit-chain evidence, and pack overlays from PRD-whiteboard.
- Preserve ADR-0321 anchor coverage: principals, Cedar gates, tenant scoping, audit evidence, pack overlay, rollback, and benchmark parity.
- Support active incidents where public-safety users need a board while ordinary collaboration friction is partially unavailable.
- Bypass only non-critical friction such as bot challenge, invite delay, or export queue throttling when the emergency policy allows it.
- Never bypass tenant isolation, Cedar default deny, region residency, legal hold, audit-chain write, or OpenBao credential binding.
- Treat Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard as displaced benchmarks for incident rooms and classroom/public-sector boards.
- Make the Oyatie differentiator explicit: emergency continuity is a governed policy route, not a hidden support override.
- Bind the route to capabilities rather than vendor labels: board-open, canvas-op-append, presence-sync, history-snapshot, export-render, and template-marketplace-install.
- Keep evidence inspectable by SRE, tenant admin, auditor, and post-incident reviewer.

## Non-goals
- Do not create a general "break glass" bypass for support operators.
- Do not permit anonymous public boards without tenant_id and principal_id.
- Do not disable Cedar or downgrade to allow-by-default during outages.
- Do not move incident logic into identity, messenger, meet, drive, or workflow-engine.
- Do not treat vendor emergency templates as canonical data objects.
- Do not change ADR-0321 or any ADR in this slice.
- Do not expand write scope beyond this IP.
- Do not define implementation code, migrations, or runtime config here.
- Do not promise external emergency-service integrations before contracts exist.
- Do not replace marketplace DealSet settlement where template acquisition remains commercial.

## Scenario definition
- Emergency-service bypass applies to a board-session declared with `purpose=emergency_coordination`.
- The request still carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, `data_class`, and `audit_event_class`.
- The principal must resolve to a tenant-approved emergency role, a delegated public-sector responder role, or a pre-registered incident bridge identity.
- The board must be attached to a workflow-engine incident run or a tenant incident declaration packet.
- The policy decision must name the bypassed friction control and the non-bypassed mandatory controls.
- The bypass expiry is measured in minutes and must be renewed through a fresh Cedar decision.
- The bypass state is visible to tenant administrators and audit exports.
- Emergency boards default to restricted sharing and no public indexing.
- Emergency templates can be installed only from allowlisted template families.
- Emergency exports are watermarked with incident id, tenant id, time range, and policy decision id.

## Benchmark displacement notes
- Miro Enterprise offers enterprise incident-room collaboration, but Oyatie must add tenant-scoped evidence for every bypassed friction control.
- Mural Enterprise supports facilitated workshops, but Oyatie must prove the emergency path is policy-bound and reversible.
- FigJam provides low-friction collaboration, but Oyatie must prevent low friction from becoming unaudited access.
- Lucidspark supports collaborative diagramming, but Oyatie must keep data residency and audit export first-class.
- Whiteboard.fi emphasizes classroom immediacy, but Oyatie must map classroom emergency use to tenant and jurisdiction controls.
- Microsoft Whiteboard is expected inside enterprise-market systems, but Oyatie must avoid platform-coupled hidden state and preserve flat whiteboard ownership.
- The displaced set pressures speed of board-open and presence-sync.
- The displaced set does not weaken Cedar, audit, or residency requirements.
- Benchmark parity is accepted only when the emergency path has stronger evidence than the displaced vendor path.
- Any benchmark feature that depends on anonymous convenience is reinterpreted as delegated identity plus short-lived policy grant.

## Capability binding
- `board-open` is the first gate because responders need to enter the incident board before editing.
- `board-open` requires an emergency declaration reference and emits `whiteboard.emergency_board_opened`.
- `canvas-op-append` allows rapid marks, cards, routes, and command notes while preserving operation ordering.
- `canvas-op-append` cannot bypass write authorization or operation schema validation.
- `presence-sync` keeps cursors and participants visible for responders and commanders.
- `presence-sync` may shed cosmetic cursor metadata but must retain participant join/leave evidence.
- `history-snapshot` captures minute-grain incident state for handover and after-action review.
- `history-snapshot` cannot omit redaction metadata, policy decision ids, or pack overlays.
- `export-render` produces official incident artifacts for auditors, regulators, and tenant records.
- `export-render` may receive emergency priority but cannot cross residency boundaries.
- `template-marketplace-install` supports pre-approved triage, evacuation, classroom lockdown, or outage coordination boards.
- `template-marketplace-install` must still settle DealSet obligations per ADR-0314 when a paid template is involved.
- Each capability record in microservices/whiteboard/capabilities must remain the source for capability names.
- Capability ownership stays inside the whiteboard microservice.
- Cross-service calls carry trace context and do not own bypass decisions.

## Policy model
- Cedar input includes `emergency_mode`, `incident_id`, `declaration_source`, `expiry_at`, and `bypass_controls`.
- Cedar resource includes board id, board classification, tenant home cell, pack overlays, and template family.
- Cedar action distinguishes `open_emergency_board`, `append_emergency_op`, `sync_emergency_presence`, `snapshot_emergency_history`, `render_emergency_export`, and `install_emergency_template`.
- Cedar denies if incident_id is absent.
- Cedar denies if expiry_at exceeds the tenant maximum.
- Cedar denies if requested bypass controls include tenant isolation, audit write, or residency.
- Cedar denies if principal has only ordinary collaboration authority.
- Cedar denies if the board data class conflicts with the declared pack.
- Cedar returns a policy decision id required by every downstream event.
- Cedar refusal evidence is user-visible as restricted emergency access, not a silent timeout.
- Policy fragments live under microservices/whiteboard/policies when implemented.
- ADR-0243 and ADR-0244 remain the authorization and ontology authority.
- ADR-0263 remains the audit and detection authority.
- ADR-0321 remains the documentation anchor authority.
- Policy review must include a negative test for every non-bypassable control.

## Data and ontology requirements
- Emergency board metadata adds `emergency_context`.
- `emergency_context.incident_id` is immutable after creation.
- `emergency_context.declaration_source` records workflow run, tenant admin declaration, or approved external bridge.
- `emergency_context.expiry_at` controls automatic reversion to ordinary friction.
- `emergency_context.bypass_controls` lists only allowed friction controls.
- `emergency_context.command_role` records responder, commander, observer, auditor, or tenant admin.
- Canvas operations add `emergency_sequence` for after-action timeline reconstruction.
- Presence events add `emergency_participant_state` without exposing unnecessary PII in metrics.
- Snapshots add `incident_time_range`.
- Exports add `emergency_watermark`.
- Template installs add `template_family_safety_class`.
- Ontology projection must classify the board as an incident coordination artifact.
- Ontology projection must not relabel the board as a document-file asset.
- Data class remains board_object, canvas_operation, presence_cursor, or export_snapshot.
- Pack overlays determine retention, redaction, and regulator export behavior.

## Implementation plan
- Step 1: Add the emergency bypass workflow template in a future scoped implementation slice.
- Step 2: Add a policy fragment for emergency role, incident id, expiry, and non-bypassable control checks.
- Step 3: Extend board-open request validation to require emergency_context when `purpose=emergency_coordination`.
- Step 4: Emit audit-chain events before board content is exposed.
- Step 5: Allow challenge bypass only after Cedar returns the explicit bypass_controls list.
- Step 6: Ensure `canvas-op-append` keeps idempotency keys and operation version monotonic.
- Step 7: Ensure `presence-sync` continues degraded-mode operation when cursor cosmetics are shed.
- Step 8: Ensure `history-snapshot` records policy decision ids for every covered operation range.
- Step 9: Ensure `export-render` verifies pack residency before priority rendering.
- Step 10: Ensure `template-marketplace-install` rejects templates outside emergency allowlists.
- Step 11: Add dashboard panels for active emergency boards and bypass expiry.
- Step 12: Add runbook entries for expiry extension, mistaken declaration, and post-incident freeze.
- Step 13: Add threat-model entries for fraudulent emergency declaration and overbroad bypass.
- Step 14: Add DPIA notes for public-sector and education pack handling.
- Step 15: Add marketplace settlement evidence for emergency template installation.
- Step 16: Add SLO exceptions only for bypassed friction, not for authorization or audit.
- Step 17: Add backfill replay behavior for emergency events after audit-chain degradation.
- Step 18: Add capacity admission tie-in so emergency boards can receive reserved lane priority.
- Step 19: Add cost-budget tags so emergency priority is visible and bounded.
- Step 20: Add acceptance evidence bundle paths under microservices/whiteboard/scorecards when the suite exists.

## Operational controls
- Active emergency boards have a maximum lifetime configured per tenant pack.
- Bypass extension requires a fresh policy decision and a reason code.
- Misdeclared emergency boards are frozen and copied into a review workflow.
- Audit export is mandatory before deletion or archive.
- Incident commanders can remove participants but cannot erase prior operations.
- Support operators can observe only when the tenant grants an incident support role.
- Tenant admins receive notifications for start, extension, expiry, export, and freeze.
- SRE receives alerts for audit backpressure, policy outage, and capacity shedding.
- Auditor views include the complete bypass_controls list.
- Detection receives signals for repeated emergency declarations from the same principal.
- The edge WAF cannot create bypass state on its own.
- The credential sidecar cannot mint longer-lived credentials for emergency boards.
- The data residency pack can block emergency export priority.
- Marketplace terms can block paid emergency template installation.
- Rollback reverts friction bypass and freezes future appends; it does not delete evidence.

## Failure modes
- Policy service unavailable: deny emergency mutation, allow only previously authorized reads if evidence is intact.
- Audit-chain unavailable: stop new emergency writes and show degraded state to incident owners.
- Workflow-engine unavailable: accept only tenant-admin declarations with tighter expiry and emit missing-workflow evidence.
- Identity delegation stale: deny new participants and preserve current participant state until expiry.
- Region outage: follow microservices/whiteboard/multi-region.md and do not move restricted boards across packs.
- Template marketplace outage: continue board operations without template installation.
- Export renderer saturation: queue emergency exports with priority but preserve tenant capacity limits.
- Presence fanout overload: shed cursor cosmetics before participant membership.
- False emergency declaration: freeze board, retain operations, revoke bypass, and open review.
- Expired bypass: revert to ordinary collaboration gates without closing the board.
- DealSet settlement failure: block paid template installation but do not block board-open.
- OpenBao lease failure: deny operations requiring fresh credentials.
- Cedar mismatch: fail closed and emit refusal evidence.
- Pack conflict: apply higher-restriction-wins.
- Replay drift: quarantine replay result and require manual adjudication.

## Evidence and tests
- Evidence 1: Contract examples show emergency_context on board-open.
- Evidence 2: Cedar tests cover allow, deny, expiry, role mismatch, and forbidden bypass_controls.
- Evidence 3: Audit tests prove every bypassed friction control is named.
- Evidence 4: Presence tests prove degraded cursor shedding keeps membership evidence.
- Evidence 5: Snapshot tests prove incident_time_range and policy decision ids are retained.
- Evidence 6: Export tests prove watermark and residency enforcement.
- Evidence 7: Template tests prove allowlist and DealSet settlement behavior.
- Evidence 8: Capacity tests prove emergency priority is bounded and observable.
- Evidence 9: Cost tests prove emergency usage is tagged by tenant, cell, incident, and capability.
- Evidence 10: Runbook drill covers mistaken declaration and expiry extension.
- Evidence 11: Threat-model review covers fraudulent declaration.
- Evidence 12: DPIA review covers public-sector and education packs.
- Evidence 13: Benchmark parity review maps each displaced vendor feature to Oyatie controls.
- Evidence 14: ADR-0321 matrix confirms principals, Cedar gates, tenant scoping, audit, pack, rollback, and benchmark anchors.
- Evidence 15: Negative tests prove audit and residency cannot be bypassed.

## Emergency-specific domain and contract deltas
- Domain aggregate: `emergency_board_session` wraps a normal `board_session_document` with incident declaration metadata.
- Domain invariant: `emergency_board_session.incident_id` cannot change after the first successful board-open.
- Domain invariant: a bypassed friction control must be listed in `emergency_board_session.bypass_controls`.
- Domain invariant: `emergency_board_session.expires_at` must be earlier than tenant pack maximum.
- Domain invariant: appends after expiry use ordinary collaboration authorization.
- Domain event `whiteboard.emergency_session.declared` records declaration source and commander principal.
- Domain event `whiteboard.emergency_session.bypass_granted` records exactly which friction controls were bypassed.
- Domain event `whiteboard.emergency_session.bypass_expired` records automatic reversion.
- Domain event `whiteboard.emergency_session.frozen_for_review` records suspected misuse.
- OpenAPI delta: board-open accepts `emergency_context` only when `purpose=emergency_coordination`.
- OpenAPI delta: emergency_context includes `incident_id`, `declaration_source`, `requested_bypass_controls`, and `expires_at`.
- OpenAPI delta: response includes `granted_bypass_controls`, `policy_decision_id`, and `mandatory_controls`.
- AsyncAPI delta: emit `whiteboard.emergency.board.opened.v1` before board content exposure.
- AsyncAPI delta: emit `whiteboard.emergency.bypass.expired.v1` on timer-driven expiry.
- Proto delta: internal command `OpenEmergencyBoard` carries `EmergencyContext` and `MandatoryControlSet`.
- Proto delta: internal event `EmergencyBoardOpened` carries `capacity_lane`, `pack_overlay_result`, and `audit_event_id`.
- Cedar fact: `principal.emergency_roles` must intersect `resource.allowed_emergency_roles`.
- Cedar fact: `context.requested_bypass_controls` must be subset of `resource.allowed_friction_controls`.
- Cedar fact: `context.requested_bypass_controls` must not contain `tenant_isolation`, `audit_chain`, `residency`, or `cedar_gate`.
- Cedar fact: `context.incident_id` must equal the workflow incident id when workflow-engine is available.
- Workflow decision: emergency declaration starts in workflow-engine unless the workflow plane is degraded.
- Workflow decision: degraded declaration requires tighter expiry and tenant-admin approval.
- Workflow decision: expiry extension is a separate workflow transition, not a field update.
- Workflow decision: false declaration routes to review, freeze, and tenant notification.
- SLO: emergency board-open p95 target is 150 ms after identity and policy cache warmup.
- SLO: emergency canvas append p95 target is 250 ms for admitted boards.
- SLO: emergency bypass expiry propagation target is 30 seconds.
- SLO: audit event emission target is before first content byte for board-open.
- Replay case: after audit-chain recovery, replay `bypass_granted` events before canvas operations.
- Replay case: after workflow outage, reconcile degraded declarations to workflow runs.
- Replay case: after capacity shedding, preserve participant membership before cursor detail.
- Rollback: revoke bypass token and keep board open under ordinary collaboration controls.
- Rollback: freeze appends when declaration is false, but leave history snapshot exportable.
- Rollback: remove emergency priority from capacity lanes without deleting audit evidence.
- Test case: responder with expired declaration receives ordinary collaboration denial.
- Test case: support operator cannot self-grant emergency role.
- Test case: residence-forbidden board cannot use emergency path to reroute export.
- Test case: DealSet-paid emergency template still requires settlement proof.
- Test case: duplicate emergency board-open returns same policy decision until expiry.
- Evidence field: `mandatory_controls` names Cedar, tenant isolation, audit-chain, residency, and pack overlay.
- Evidence field: `bypass_controls` names only bot challenge, invite delay, export priority, or rate-friction controls.
- Evidence field: `emergency_review_packet_id` links after-action review and regulator export.

## Acceptance criteria
- AC-001: Emergency bypass is represented as policy-scoped friction relief, not as global access override.
- AC-002: All six capability records remain traceable by exact name.
- AC-003: ADR-0321 remains listed in Binding ADRs and is not edited.
- AC-004: Benchmark names include Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
- AC-005: Repo-local references include PRD, Architecture, capabilities, runbooks, policies, incident response, and threat model.
- AC-006: Tenant id, principal id, audience type, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target remain mandatory.
- AC-007: Non-bypassable controls are named and tested.
- AC-008: Rollback preserves evidence and stops future bypassed operations.
- AC-009: DealSet settlement remains required for commercial template installation.
- AC-010: The IP is substantive enough for implementation planning without reading competitor docs.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
