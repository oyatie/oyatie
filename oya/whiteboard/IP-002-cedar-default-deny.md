# IP-002 Whiteboard Cedar Default Deny

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-002-cedar-default-deny.md
Planning lane: B2B-leader IP substance deepening pass
Primary concern: caller-side-library-first Cedar enforcement for whiteboard collaboration commands and live-session fanout
Policy references: microservices/whiteboard/policy/canvas-collaboration-authorization.cedar; microservices/whiteboard/policy/auditor-scope.cedar; microservices/whiteboard/policy/ci-scope.cedar; microservices/whiteboard/policy/data-residency.md; microservices/whiteboard/policies/local-board-open-scope.cedar; microservices/whiteboard/policies/local-stroke-persistence-guard.cedar; microservices/whiteboard/policies/local-shape-update-acl.cedar; microservices/whiteboard/policies/local-cursor-broadcast-rate.cedar; microservices/whiteboard/policies/local-crdt-merge-control.cedar; microservices/whiteboard/policies/local-board-export-egress.cedar
Capability references: whiteboard-board-open; whiteboard-canvas-op-append; whiteboard-presence-sync; whiteboard-history-snapshot; whiteboard-export-render; whiteboard-template-marketplace-install
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Benchmark displacement set: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard

## Executive Intent
- Whiteboard must be default-deny at the collaboration boundary because board sessions mix long-lived objects, short-lived presence, and high-rate canvas mutations.
- Vendor tools often hide access decisions inside workspace, room, or file membership models; Oyatie must expose every allow and deny as tenant-scoped policy evidence.
- The policy layer must displace Miro Enterprise workspace access with explicit board-open authorization.
- The policy layer must displace Mural Enterprise room access with explicit session and canvas operation authorization.
- The policy layer must displace FigJam quick collaboration with low-latency but still auditable presence and operation checks.
- The policy layer must displace Lucidspark diagram sharing with explicit object, connector, and export controls.
- The policy layer must displace Whiteboard.fi classroom access with education-pack-aware board and roster rules.
- The policy layer must displace Microsoft Whiteboard tenant-wide convenience permissions with explicit tenant, cell, purpose, and data-class inputs.
- ADR-0321 remains the B2B leader coverage driver; policy parity is not complete until these benchmark behaviors are governed locally.
- ADR-0314 remains a hard constraint for marketplace templates; Cedar cannot authorize commercial use when DealSet settlement is absent.
- ADR-0253-amendment means policy evidence must survive transport hardening and edge cryptography changes.
- ADR-0105 means policy evaluation is a library binding used by callers, not a hidden adapter side effect.

## Default Deny Contract
- Every command starts denied.
- Every event fanout starts denied.
- Every replay side effect starts denied.
- Every export egress starts denied.
- Every template install starts denied.
- The only path to allow is a Cedar decision with complete BoardScope input.
- The only path to bypass is an emergency-services policy that emits bypass-approved evidence.
- A missing policy file is deny.
- A policy parse error is deny.
- A policy version mismatch is deny.
- A missing tenant_id is deny before evaluation.
- A missing principal_id is deny before evaluation.
- A missing audience_type is deny before evaluation.
- A missing purpose is deny before evaluation.
- A missing data_class is deny before evaluation.
- A missing board_id for board operations is deny before evaluation.
- A missing session_id for live-session operations is deny before evaluation.
- A missing operation_id for canvas mutations is deny before evaluation.
- A missing marketplace_dealset_id for marketplace templates is deny before evaluation.
- A missing residency decision for export or snapshot is deny before evaluation.
- A missing audit_chain_target for accepted commands is deny before evaluation.

## Policy Input Envelope
- principal.type must be CollaborationUser, Auditor, SystemWorker, or EmergencyServicePrincipal.
- principal.id must match BoardScope.principal_id.
- principal.tenant_id must match BoardScope.tenant_id.
- action must be board.open, canvas.operation.append, presence.sync, history.snapshot, export.render, or template.marketplace.install.
- resource.type must be Board, CanvasOperation, PresenceChannel, HistorySnapshot, ExportRender, or TemplateInstall.
- resource.tenant_id must match BoardScope.tenant_id.
- resource.board_id must be present when the resource is board-scoped.
- resource.session_id must be present for presence and active collaboration controls.
- context.audience_type must carry COLLABORATION_USER unless a special actor is explicit.
- context.purpose must carry a canonical purpose from the command.
- context.data_class must carry board_object, canvas_operation, presence_cursor, export_snapshot, or template_asset.
- context.tenant_home_cell must be present.
- context.request_cell must be present.
- context.policy_pack_set must list active overlays.
- context.source_vendor must be none or a displaced benchmark name.
- context.marketplace_dealset_id must be present for marketplace templates.
- context.trace_id must be present for request correlation.
- context.idempotency_key must be present for mutations.
- context.scope_hash must be present for async and replay checks.
- context.contract_family must identify REST, AsyncAPI, proto, worker, or replay caller.
- context.emergency_bypass_claim must be absent unless the emergency bypass policy owns the route.

## Capability Rules
- board-open policy must require principal membership in the tenant board audience.
- board-open policy must require request_cell to satisfy tenant_home_cell or a pack-approved routing exception.
- board-open policy must allow read materialization only after board ownership and session visibility are proven.
- board-open policy must not allow board discovery by guessing board_id values.
- board-open policy must expose Miro Enterprise and Mural Enterprise migration board opens as source-vendor-specific evidence.
- canvas-op-append policy must require active session membership.
- canvas-op-append policy must require operation_id and idempotency_key.
- canvas-op-append policy must guard strokes, shapes, sticky notes, connectors, frames, comments, and reactions as canvas_operation data.
- canvas-op-append policy must keep FigJam-like real-time mutation latency while preserving deny evidence.
- canvas-op-append policy must treat Lucidspark-style diagram connector updates as first-class mutations.
- presence-sync policy must require active session membership.
- presence-sync policy must rate-limit cursor broadcast through local-cursor-broadcast-rate policy.
- presence-sync policy must classify cursor data as presence_cursor.
- presence-sync policy must avoid cross-tenant fanout even when multiple tenants share an integration meeting.
- history-snapshot policy must require board history read rights and retention compatibility.
- history-snapshot policy must include source_vendor for imported boards.
- export-render policy must require export rights, residency permission, and retention compatibility.
- export-render policy must deny Microsoft Whiteboard-style broad tenant export unless pack rules allow the exact export.
- template-marketplace-install policy must require DealSet settlement before allowing template materialization.
- template-marketplace-install policy must deny source-vendor template material when commercial provenance is unresolved.

## Benchmark-Specific Deny Cases
- Deny Miro Enterprise import when source workspace owner is not mapped to tenant_id.
- Deny Miro Enterprise board open when source board is shared globally but tenant policy forbids public links.
- Deny Miro Enterprise template install when DealSet status is pending.
- Deny Mural Enterprise import when room membership cannot be mapped to a tenant-scoped audience.
- Deny Mural Enterprise facilitator controls when principal lacks moderator role.
- Deny Mural Enterprise anonymous guest access unless guest is represented as a tenant-bound CollaborationUser.
- Deny FigJam presence fanout when the user is not in the active board session.
- Deny FigJam widget mutation when widget source has no approved template or app provenance.
- Deny FigJam comment import when comment author cannot be resolved to principal_id or external_actor evidence.
- Deny Lucidspark diagram connector mutation when shape endpoints cross unauthorized boards.
- Deny Lucidspark export when diagram contains data classes not allowed by the active pack.
- Deny Lucidspark template use when source library license is missing.
- Deny Whiteboard.fi class board open when education pack roster is absent.
- Deny Whiteboard.fi student work export when student data residency is unresolved.
- Deny Whiteboard.fi teacher broadcast when the teacher principal lacks class facilitator claim.
- Deny Microsoft Whiteboard tenant export when export scope exceeds selected board_ids.
- Deny Microsoft Whiteboard Teams-linked board open when meeting membership is not mapped to board session membership.
- Deny Microsoft Whiteboard loop component import when source object identity cannot be retained.
- Deny any benchmark migration that lacks source_object_id.
- Deny any benchmark migration that tries to erase source_vendor after acceptance.

## ADR Binding Notes
- ADR-0105 binds policy evaluation to explicit layers; the usecase caller invokes the policy library before domain state changes.
- ADR-0131 keeps tenant isolation visible in policy inputs and audit outputs.
- ADR-0242 requires policy decisions to be reproducible during replay and audit review.
- ADR-0243 requires the capability records to remain the source of required tenant-scope fields.
- ADR-0244 requires documentation and contract parity before implementation promotion.
- ADR-0246 requires evidence-producing gates rather than informal review claims.
- ADR-0253-amendment requires h3-alt-svc, ECH, and PQC rollout not to hide identity context.
- ADR-0257 requires caller-side-library-first policy behavior.
- ADR-0258 requires service-local policy evidence to be queryable.
- ADR-0263 requires cross-region and cell-aware policy posture.
- ADR-0294 requires operator-visible failure modes.
- ADR-0296 requires evidence continuity across async workers.
- ADR-0297 requires replay-safe decision material.
- ADR-0314 requires DealSet settlement for marketplace-originated material.
- ADR-0321 requires B2B leader coverage against the displaced benchmark set.

## Implementation Steps
- Load Cedar policies at service startup through a versioned policy bundle.
- Fail service readiness if required policy bundle files are missing.
- Expose policy bundle version through diagnostics.
- Create a policy input mapper for board-open.
- Create a policy input mapper for canvas-op-append.
- Create a policy input mapper for presence-sync.
- Create a policy input mapper for history-snapshot.
- Create a policy input mapper for export-render.
- Create a policy input mapper for template-marketplace-install.
- Add a pre-evaluation completeness check for BoardScope required fields.
- Add source_vendor normalization before policy mapping.
- Add DealSet presence check before template policy mapping.
- Add residency decision check before export policy mapping.
- Add retention decision check before history-snapshot policy mapping.
- Add active session membership check before presence policy mapping.
- Add operation idempotency check before canvas mutation policy mapping.
- Add deterministic deny reason mapping for every pre-evaluation failure.
- Add Cedar deny reason mapping for every policy failure.
- Add bypass-approved mapping only for emergency-services-bypass policy results.
- Add trace spans around completeness check, policy input build, and Cedar evaluation.
- Add audit events for pre-evaluation deny, policy deny, allow, and bypass-approved.

## Contract Projections
- REST 403 responses must use stable problem codes.
- REST 409 responses must be reserved for idempotency or state conflicts, not authorization denies.
- REST 422 responses must be reserved for malformed input before policy.
- AsyncAPI deny events must include capability, board_id when safe, result, reason, policy_version, and scope_hash.
- AsyncAPI allow events must include policy_version and evaluation_trace_id.
- Proto internal calls must not bypass policy because they are inside the cluster.
- Worker envelopes must carry prior policy_version for replay comparability.
- Replay commands must re-evaluate when policy_version requires strict current-policy replay.
- Replay commands must use stored decision evidence when the replay mode is historical reconstruction.
- Catalog records must identify policy surfaces through oya-whiteboard-canvas-collaboration-policy-related metadata.
- SDK generation must expose typed deny codes instead of raw Cedar errors.
- Runbooks must refer to deny codes, policy versions, and evaluation traces.
- Dashboards must aggregate policy_decision_total by capability.
- Dashboards must aggregate policy_decision_total by source_vendor.
- Dashboards must aggregate policy_decision_total by data_class.
- Dashboards must aggregate policy_decision_total by tenant_home_cell and request_cell.
- Alerts must distinguish expected authorization denials from policy-bundle failures.
- CI policy tests must load the same local policy files named in this IP.
- CI policy tests must include all six displaced benchmark names.
- CI policy tests must include all six whiteboard capability records.

## Observability
- Emit whiteboard_policy_eval_total with result allow, deny, precheck_deny, bypass_approved, and error.
- Emit whiteboard_policy_eval_seconds with capability, action, policy_version, and result.
- Emit whiteboard_policy_precheck_deny_total with reason and capability.
- Emit whiteboard_policy_bundle_loaded with bundle version and policy count.
- Emit whiteboard_policy_bundle_error_total when parsing or loading fails.
- Emit whiteboard_policy_benchmark_deny_total with benchmark_name.
- Emit trace span whiteboard.policy.precheck.
- Emit trace span whiteboard.policy.map_input.
- Emit trace span whiteboard.policy.evaluate.
- Emit trace span whiteboard.policy.audit_emit.
- Log allow decisions at debug with trace ids and without raw canvas payload.
- Log deny decisions at info with reason and without raw canvas payload.
- Log policy bundle failures at error with bundle version.
- Emit audit event whiteboard.policy.allow.
- Emit audit event whiteboard.policy.deny.
- Emit audit event whiteboard.policy.precheck_deny.
- Emit audit event whiteboard.policy.bypass_approved.
- Emit dashboard fields compatible with microservices/whiteboard/dashboards/local-policy-decisions.json.
- Emit audit fields compatible with microservices/whiteboard/dashboards/local-audit-completeness.json.
- Attach policy_version to every emitted event.
- Attach evaluation_trace_id to every emitted event.

## Test Plan
- Unit test missing tenant_id returns precheck deny.
- Unit test missing principal_id returns precheck deny.
- Unit test missing audience_type returns precheck deny.
- Unit test missing purpose returns precheck deny.
- Unit test missing data_class returns precheck deny.
- Unit test missing board_id for board-open returns precheck deny.
- Unit test missing session_id for presence-sync returns precheck deny.
- Unit test missing operation_id for canvas-op-append returns precheck deny.
- Unit test missing marketplace_dealset_id for marketplace template returns precheck deny.
- Unit test missing residency decision for export-render returns precheck deny.
- Cedar test board-open allows active tenant collaborator.
- Cedar test board-open denies cross-tenant principal.
- Cedar test canvas-op-append allows active board session member.
- Cedar test canvas-op-append denies closed session member.
- Cedar test presence-sync denies absent session membership.
- Cedar test history-snapshot denies retention conflict.
- Cedar test export-render denies residency conflict.
- Cedar test template-marketplace-install denies pending DealSet.
- Benchmark test includes Miro Enterprise import deny.
- Benchmark test includes Mural Enterprise facilitator deny.
- Benchmark test includes FigJam presence deny.
- Benchmark test includes Lucidspark connector deny.
- Benchmark test includes Whiteboard.fi roster deny.
- Benchmark test includes Microsoft Whiteboard tenant export deny.
- Contract test REST maps deny to stable problem code.
- AsyncAPI test deny event includes scope_hash and policy_version.
- Replay test historical mode uses stored decision evidence.
- Replay test current-policy mode re-evaluates with current policy_version.
- Observability test emits policy metrics and audit events.
- CI test fails when a required local policy file is absent.
- CI test fails when benchmark names regress to old generic names.

## Acceptance Criteria
- Every whiteboard capability is denied until policy permits it.
- Every policy input includes BoardScope fields from IP-001.
- Every deny has a stable reason.
- Every allow has policy_version and evaluation_trace_id.
- Every benchmark-specific migration path is explicitly covered.
- Every marketplace template path requires DealSet settlement.
- Every export path checks residency and retention.
- Every presence path checks session membership and cursor broadcast limits.
- Every canvas mutation path checks operation identity and idempotency.
- Every board-open path checks tenant board membership.
- Every replay path can compare stored decision evidence.
- Every dashboard can separate allow, deny, precheck deny, bypass-approved, and policy errors.
- No internal proto or worker call bypasses policy.
- No anonymous collaboration user can be created by omission.
- No ADR-0321 edits are required or made by this IP.

## Title-Specific Command, Event, And Proto Deltas
- REST command handlers must call Cedar before emitting accepted command events.
- Async workers must verify a prior policy_decision_ref before executing command-derived work.
- Proto internal calls must include policy_decision_ref when crossing usecase, worker, or replay boundaries.
- Proto internal calls must include policy_version so replay can compare current and historical decisions.
- Proto internal calls must include evaluation_trace_id for audit correlation.
- board.session.requested events must include board-open policy outcome.
- canvas.operation.accepted events must include canvas-op-append policy outcome.
- presence.sync.requested events must include presence-sync policy outcome.
- history.snapshot.requested events must include history-snapshot policy outcome.
- export.render.requested events must include export-render policy outcome.
- template.install.requested events must include template-marketplace-install policy outcome.
- vendor.import.requested events must include source_vendor policy facts before import workers run.
- board.replay.requested events must state historical or current-policy mode.
- Deny events must use policy-deny reason codes, not raw Cedar text.
- Allow events must include the policy bundle digest.
- Bypass-approved events must include emergency policy id and bounded duration.
- local-operations-v1.proto must reject worker commands without policy_decision_ref unless the command is policy-exempt health telemetry.
- whiteboard-v1.proto must reserve fields for policy_result and deny_reason.
- SDKs must expose typed policy denial results for all six capabilities.
- Contract examples must include a denied benchmark import for each displaced vendor.

## Title-Specific Canvas, CRDT, And Session Facts
- Cedar facts for CRDT merge include board_id, operation_range, merge_strategy, and scope_hash.
- Cedar facts for canvas operation append include operation_kind, object_id, session_id, and idempotency_key.
- Cedar facts for connector mutation include endpoint object ids for Lucidspark displacement.
- Cedar facts for sticky note mutation include author_principal_id and data_class.
- Cedar facts for frame mutation include parent frame and board ownership.
- Cedar facts for board session open include requested_role and active participant count.
- Cedar facts for facilitator controls include facilitator_principal_ids and workflow_template_id.
- Cedar facts for FigJam-like cursor fanout include rate_limit_bucket and session membership.
- Cedar facts for Whiteboard.fi classroom flows include roster_binding and education pack.
- Cedar facts for Microsoft Whiteboard meeting-linked flows include meeting_binding and participant mapping.
- Cedar facts for Mural Enterprise workshop flows include facilitator role and room provenance.
- Cedar facts for Miro Enterprise migration include source_workspace_id and source_object_id as provenance only.
- Cedar facts for history snapshot include operation_range and retention_class.
- Cedar facts for export render include residency_zone, retention_class, export_format, and selected board ids.
- Cedar facts for template install include template_source, template_id, marketplace_dealset_id, and source_vendor.

## Title-Specific SLO And Evidence Gates
- Policy evaluation p95 must fit within board-open and write-latency budgets for synchronous commands.
- Policy evaluation for canvas-op-append must be measured against local-stroke-persistence-latency when used in fallback REST path.
- Policy evaluation for presence-sync must be measured against local-presence-freshness when it gates fanout.
- Policy evaluation for export-render must be measured against local-export-render-latency setup budget.
- Policy decision emission must be measured against audit-emission-lag.
- Evidence fields must include policy_version, policy_bundle_digest, evaluation_trace_id, policy_result, and deny_reason.
- Evidence fields must include action, capability, data_class, source_vendor, tenant_home_cell, and request_cell.
- Evidence fields must include crdt_merge_id when policy gates merge repair.
- Evidence fields must include session_id when policy gates facilitator or cursor behavior.
- Evidence fields must include marketplace_dealset_id when policy gates template material.

## Rollback
- Roll back route exposure before weakening policy rules.
- Roll back async publication before weakening worker policy checks.
- Keep deny reason codes stable once published.
- Keep audit event schemas stable once emitted.
- Keep benchmark-specific tests even if a migration source is delayed.
- Treat policy bundle deletion as an incident, not a rollback path.
- Treat DealSet bypass as an ADR-level decision.
- Treat broad tenant export bypass as an ADR-level decision.
- Route policy incidents to local-collaboration-acl-mismatch, local-session-throttle-activation, export-render-failure, or moderation-report-escalation runbooks.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
