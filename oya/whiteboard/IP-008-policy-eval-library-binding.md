# IP-008 Whiteboard policy-eval-library-binding

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-008-policy-eval-library-binding.md
Benchmarks displaced: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## 1. Outcome
- Bind whiteboard to the caller-side policy evaluation library declared by the six capability records.
- Keep Cedar evaluation before board metadata, canvas operations, presence fanout, snapshots, exports, and template installation.
- Make policy inputs a first-class data contract rather than incidental request metadata.
- Preserve default-deny behavior from microservices/whiteboard/policy/canvas-collaboration-authorization.cedar.
- Preserve local scope behavior from microservices/whiteboard/policies/local-board-open-scope.cedar.
- Preserve local append behavior from microservices/whiteboard/policies/local-stroke-persistence-guard.cedar.
- Preserve local shape-update behavior from microservices/whiteboard/policies/local-shape-update-acl.cedar.
- Preserve local export behavior from microservices/whiteboard/policies/local-board-export-egress.cedar.
- Preserve local cursor behavior from microservices/whiteboard/policies/local-cursor-broadcast-rate.cedar.
- Preserve local merge behavior from microservices/whiteboard/policies/local-crdt-merge-control.cedar.

## 2. Why This IP Exists
- Miro Enterprise parity creates pressure for broad participant sharing.
- Mural Enterprise parity creates pressure for facilitator override controls.
- FigJam parity creates pressure for fast lightweight edits and reactions.
- Lucidspark parity creates pressure for diagram-style export and snapshot permissions.
- Whiteboard.fi parity creates pressure for classroom owner and student-board modes.
- Microsoft Whiteboard parity creates pressure for guest-link and Microsoft 365-style sharing behavior.
- Oyatie cannot satisfy those pressures with ad hoc booleans in handlers.
- Oyatie must use Cedar actions, principals, resources, and context consistently.
- ADR-0321 requires vendor-specific Cedar verbs and failure modes rather than generic “authorize request” statements.
- This file defines the policy library binding for that requirement without editing the policy files.

## 3. Local Source Anchors
- microservices/whiteboard/PRD.md requires tenant scope, principal, purpose, data class, pack overlay, idempotency, trace context, and audit-chain target.
- microservices/whiteboard/ARCHITECTURE.md requires Cedar permit before storage/provider access.
- microservices/whiteboard/compliance.md defines pack and compliance evidence expectations.
- microservices/whiteboard/threat-model.md defines abuse, insider, cross-tenant, and edge threats.
- microservices/whiteboard/failure-modes.md defines degraded-mode and rollback cases.
- microservices/whiteboard/policy/abuse-defence.cedar supplies policy vocabulary for suspicious behavior.
- microservices/whiteboard/policy/auditor-scope.cedar supplies read evidence scope.
- microservices/whiteboard/policy/ci-scope.cedar supplies automation scope.
- microservices/whiteboard/policy/data-residency.md supplies residency policy context.
- microservices/whiteboard/dashboards/local-policy-decisions.json is the dashboard evidence target.
- microservices/whiteboard/dashboards/abuse-defence-outcomes.json is the abuse outcome target.
- microservices/whiteboard/runbooks/local-collaboration-acl-mismatch.md is the first authorization runbook.

## 4. Policy Library Contract
- The library exposes `authorizeBoardOpen`.
- The library exposes `authorizeCanvasOpAppend`.
- The library exposes `authorizePresenceSync`.
- The library exposes `authorizeHistorySnapshot`.
- The library exposes `authorizeExportRender`.
- The library exposes `authorizeTemplateMarketplaceInstall`.
- Each function accepts a typed principal.
- Each function accepts a typed resource.
- Each function accepts a typed action.
- Each function accepts typed context.
- Each function returns a policy decision id.
- Each function returns a permit or deny result.
- Each function returns an explanation class.
- Each function returns a policy context hash.
- Each function returns the Cedar policy set version.
- Each function returns the pack overlay version.
- Each function returns an audit evidence payload.
- Each function is side-effect free except evidence emission through the configured evidence adapter.
- Each function is deterministic for identical input and policy version.
- Each function fails closed when policy bundles are stale for mutation actions.

## 5. Principal Model
- `TenantMember` covers regular tenant users.
- `TenantGuest` covers invited external collaborators.
- `ClassroomOwner` covers Whiteboard.fi-style teacher or trainer authority.
- `ClassroomParticipant` covers short-lived student boards.
- `Facilitator` covers Mural Enterprise facilitation controls.
- `BoardOwner` covers board lifecycle authority.
- `TemplateInstaller` covers marketplace template installation.
- `Auditor` covers evidence access through auditor-scope policy.
- `SupportOperator` covers break-glass investigation with audit trails.
- `WorkflowWorker` covers workflow-engine invocations.
- `CiRobot` covers CI-scope checks.
- Every principal carries `tenant_id`.
- Every principal carries `principal_id`.
- Every principal carries `audience_type`.
- Every principal carries `roles`.
- Every principal carries `home_cell`.
- Every principal carries `jurisdiction_code`.
- Every principal carries `data_class_clearance`.
- Every principal carries `pack_overlay_ids`.
- Every principal carries `authn_strength`.

## 6. Resource Model
- `Board` carries `board_id`, `tenant_id`, `home_cell`, `region_affinity`, and `classification`.
- `CanvasOperation` carries `op_id`, `board_id`, `operation_kind`, `target_shape_ref`, and `data_class`.
- `PresenceChannel` carries `board_id`, `connection_id`, `participant_id`, and `fanout_partition`.
- `HistorySnapshot` carries `snapshot_id`, `board_id`, `revision_window`, and `retention_class`.
- `ExportArtifact` carries `export_id`, `board_id`, `artifact_class`, and `residency_target`.
- `TemplatePackage` carries `template_id`, `deal_set_id`, `license_scope`, and `publisher_ref`.
- Every resource carries `tenant_id`.
- Every resource carries `owner_principal_id` where ownership applies.
- Every resource carries `pack_overlay_id`.
- Every resource carries `source_system_ref` when imported.
- Every resource carries `ontology_object_ref` after projection.
- Every resource carries `audit_chain_ref` after accepted mutation.

## 7. Action Names
- `whiteboard.board.open` gates board open.
- `whiteboard.board.invite_guest` gates share-link and guest invitation translation.
- `whiteboard.board.facilitator_lock` gates Mural Enterprise-style facilitation locks.
- `whiteboard.canvas.append_shape` gates shape creation.
- `whiteboard.canvas.move_shape` gates shape movement.
- `whiteboard.canvas.delete_shape` gates deletion or tombstone writes.
- `whiteboard.canvas.add_sticky` gates sticky-note creation.
- `whiteboard.canvas.react` gates low-risk FigJam-style reactions.
- `whiteboard.presence.publish_cursor` gates cursor fanout.
- `whiteboard.presence.view_participant` gates participant visibility.
- `whiteboard.history.snapshot_create` gates snapshot creation.
- `whiteboard.history.replay_view` gates replay access.
- `whiteboard.export.render` gates export rendering.
- `whiteboard.export.download` gates artifact download.
- `whiteboard.template.install` gates marketplace template install.
- `whiteboard.template.publish` gates template publication.
- `whiteboard.template.license_apply` gates DealSet-backed licensing.
- `whiteboard.classroom.spawn_student_board` gates Whiteboard.fi-style ephemeral boards.
- `whiteboard.classroom.close_student_board` gates classroom cleanup.
- `whiteboard.support.inspect_evidence` gates support inspection.

## 8. Context Fields
- `purpose` is mandatory.
- `data_class` is mandatory.
- `home_cell` is mandatory.
- `request_cell` is mandatory.
- `jurisdiction_code` is mandatory.
- `pack_overlay_id` is mandatory.
- `pack_overlay_hash` is mandatory for mutations.
- `idempotency_key` is mandatory for mutations.
- `trace_id` is mandatory.
- `audit_event_class` is mandatory for accepted mutations.
- `workflow_run_id` is mandatory for workflow invocations.
- `deal_set_id` is mandatory for template installation and licensed exports.
- `source_vendor_benchmark` is optional benchmark provenance.
- `classroom_session_id` is mandatory for classroom mode.
- `facilitation_mode` is mandatory for facilitator lock actions.
- `guest_invite_ref` is mandatory for guest authority.
- `residency_target` is mandatory for exports.
- `client_risk_score` is accepted only as signal, never as final authority.

## 9. Capability Binding
- Board open calls `authorizeBoardOpen`.
- Board open maps to `whiteboard.board.open`.
- Board open uses `Board` as resource.
- Board open denies before returning board title, participants, or presence endpoint.
- Canvas op append calls `authorizeCanvasOpAppend`.
- Canvas op append maps operation kind to a Cedar action.
- Canvas op append uses `CanvasOperation` as resource.
- Canvas op append denies before CRDT merge or storage.
- Presence sync calls `authorizePresenceSync`.
- Presence sync maps cursor publish to `whiteboard.presence.publish_cursor`.
- Presence sync denies before fanout.
- History snapshot calls `authorizeHistorySnapshot`.
- History snapshot maps replay view and snapshot create separately.
- Export render calls `authorizeExportRender`.
- Export render maps render and download separately.
- Template install calls `authorizeTemplateMarketplaceInstall`.
- Template install requires DealSet context per ADR-0314.

## 10. Vendor-Specific Policy Cases
- Miro Enterprise imported boards often include wide team sharing; translate team sharing into explicit Cedar principal sets.
- Miro Enterprise anonymous edit links are not accepted as anonymous authority.
- Mural Enterprise facilitator permissions become explicit `Facilitator` role grants.
- Mural Enterprise timer/voting controls become board-session context, not hidden policy state.
- FigJam reactions map to lower-risk `whiteboard.canvas.react`, still tenant-scoped.
- FigJam audio/cursor collaboration metadata must not grant content access by itself.
- Lucidspark diagram exports require `whiteboard.export.render` plus residency context.
- Lucidspark shared diagram comments map to canvas operations with comment data class.
- Whiteboard.fi student-board spawn uses `whiteboard.classroom.spawn_student_board`.
- Whiteboard.fi board close uses `whiteboard.classroom.close_student_board`.
- Microsoft Whiteboard shared links map to `TenantGuest` or `TenantMember` grants.
- Microsoft Whiteboard Teams/365 context maps to source provenance, not policy bypass.

## 11. Failure Modes
- Missing tenant id produces `TENANT_SCOPE_MISSING`.
- Missing principal id produces `PRINCIPAL_SCOPE_MISSING`.
- Missing purpose produces `PURPOSE_MISSING`.
- Missing data class produces `DATA_CLASS_MISSING`.
- Missing pack overlay produces `PACK_OVERLAY_MISSING`.
- Stale policy bundle produces `POLICY_BUNDLE_STALE`.
- Policy hash mismatch produces `POLICY_CONTEXT_MISMATCH`.
- Cross-cell request produces `HOME_CELL_MISMATCH`.
- Guest invite expired produces `GUEST_INVITE_EXPIRED`.
- Classroom session expired produces `CLASSROOM_SESSION_EXPIRED`.
- Facilitator lock conflict produces `FACILITATOR_LOCK_CONFLICT`.
- DealSet hold produces `DEAL_SET_HOLD`.
- Residency conflict produces `RESIDENCY_CONFLICT`.
- Audit-chain pause produces `AUDIT_CHAIN_PAUSED`.
- Abuse throttle produces `ABUSE_THROTTLED`.

## 12. Evidence
- Every permit emits a decision id.
- Every deny emits a refusal evidence event.
- Every mutation permit carries an audit event payload.
- Every export permit carries residency and artifact evidence.
- Every template permit carries DealSet evidence.
- Every classroom permit carries session lifetime evidence.
- Every guest permit carries invite provenance.
- Every facilitator lock permit carries lock owner and expiry evidence.
- Every support inspection permit carries support case id.
- Every CI permit carries run id and code ref.
- Dashboard evidence lands in local-policy-decisions.
- Abuse evidence lands in abuse-defence-outcomes.
- Audit completeness lands in local-audit-completeness.
- SLO burn links to slo-and-error-budget.

## 13. Implementation Steps
- Replace generic policy calls with capability-specific library calls.
- Define typed principal builders at the API/adapter boundary.
- Define typed resource builders at the API/adapter boundary.
- Define typed context builders at the API/adapter boundary.
- Add validators for mandatory policy fields.
- Add policy bundle version checks.
- Add policy context hashing.
- Add pack overlay hashing.
- Add denial-to-runbook mapping.
- Add decision-id propagation to gRPC and event envelopes.
- Add audit evidence payload generation after permit.
- Add refusal evidence payload generation after deny.
- Add tenant-safe metric labels.
- Add replay fixtures for policy decisions.
- Keep Cedar files authoritative; do not hardcode policy outcomes in application code.

## 14. Tests
- Unit tests cover each principal builder.
- Unit tests cover each resource builder.
- Unit tests cover mandatory context validation.
- Cedar contract tests cover board open permit and deny.
- Cedar contract tests cover canvas append permit and deny.
- Cedar contract tests cover presence fanout permit and deny.
- Cedar contract tests cover history snapshot permit and deny.
- Cedar contract tests cover export render permit and deny.
- Cedar contract tests cover template install permit and deny.
- Replay tests assert identical inputs produce identical decision hashes.
- Failure tests assert stale bundles fail closed for mutations.
- Failure tests assert reads can degrade only where architecture permits.
- Benchmark tests name Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.

## 15. Acceptance Criteria
- All six capability records use caller-side library policy evaluation.
- All mutation paths fail closed when policy evaluation is unavailable.
- All read paths either deny or degrade according to documented mode.
- All decisions carry tenant, principal, action, resource, context, decision id, and policy version.
- All DealSet-sensitive decisions carry marketplace context.
- All pack-sensitive decisions carry pack overlay context.
- All benchmark pressures are mapped to Oyatie actions rather than vendor folders.
- ADR-0321 remains cited and satisfied with vendor-specific Cedar verbs, objects, and failure modes.

## 16. Proto Propagation Notes
- Proto requests carry policy decision references as opaque ids, never embedded policy bundles.
- Proto responses return denial class and remediation hint for internal workers.
- Proto replay envelopes carry policy input hash so deterministic re-evaluation can compare old and new bundles.
- Proto worker commands reject missing tenant, principal, action, resource, and context digest fields.
- Proto benchmark imports preserve source vendor names for Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
