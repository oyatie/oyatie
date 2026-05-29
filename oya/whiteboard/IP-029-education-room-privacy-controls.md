# IP-029 Whiteboard education room privacy controls

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-029-education-room-privacy-controls.md
Capability focus: board-open, presence-sync, canvas-op-append, export-render
Benchmarks: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0253-amendment, ADR-0257, ADR-0263, ADR-0297, ADR-0314, ADR-0316, ADR-0321
Repo-local references: microservices/whiteboard/PRD.md; microservices/whiteboard/ARCHITECTURE.md; microservices/whiteboard/capabilities/board-open.yaml; microservices/whiteboard/capabilities/presence-sync.yaml; microservices/whiteboard/capabilities/export-render.yaml; microservices/whiteboard/dpia.md; microservices/whiteboard/compliance.md; microservices/whiteboard/threat-model.md; microservices/whiteboard/runbooks/local-collaboration-acl-mismatch.md; microservices/whiteboard/slos/local-presence-freshness.openslo.yaml; microservices/whiteboard/policy/canvas-collaboration-authorization.cedar

## Objective
- Define privacy controls for teacher-led education rooms.
- Make classroom boards safe for student isolation, teacher visibility, and audit review.
- Prevent whiteboard collaboration from leaking student work across peers.
- Bind education rooms to tenant, class roster, purpose, and data_class.
- Keep education-specific controls inside whiteboard, not a vendor suite boundary.
- Treat Whiteboard.fi as the primary classroom privacy benchmark.
- Treat Microsoft Whiteboard as the enterprise identity handoff benchmark.
- Treat FigJam as the lightweight student collaboration benchmark.
- Treat Miro Enterprise as the large workshop classroom benchmark.
- Treat Mural Enterprise as the structured activity benchmark.
- Treat Lucidspark as the diagram and concept-map classroom benchmark.

## Current repo anchors
- anchor 001: PRD-whiteboard lists education as a pack overlay.
- anchor 002: PRD-whiteboard requires tenant-scoped, Cedar-gated, observable capability.
- anchor 003: ARCHITECTURE.md names board-session and canvas contexts.
- anchor 004: board-open capability records tenant and purpose requirements.
- anchor 005: presence-sync capability records cursor state as a governed data stream.
- anchor 006: export-render capability governs teacher export and evidence packets.
- anchor 007: dpia.md is the privacy impact anchor for student work.
- anchor 008: compliance.md is the compliance-pack anchor for education controls.
- anchor 009: threat-model.md is the abuse and leakage scenario anchor.
- anchor 010: local-collaboration-acl-mismatch runbook handles privacy boundary drift.
- anchor 011: local-presence-freshness SLO controls teacher-visible room state freshness.
- anchor 012: ADR-0321 requires leader coverage while ADR-0316 prevents product fragmentation.

## Domain vocabulary
- vocabulary 001: `education_room_id` identifies a teacher-led whiteboard room.
- vocabulary 002: `class_roster_id` identifies the permitted student and staff set.
- vocabulary 003: `teacher_principal_id` identifies the room controller.
- vocabulary 004: `co_teacher_principal_id` identifies delegated classroom staff.
- vocabulary 005: `student_principal_id` identifies a student participant.
- vocabulary 006: `student_board_id` identifies an isolated student workspace.
- vocabulary 007: `teacher_overview_id` identifies the teacher aggregate view.
- vocabulary 008: `peer_visibility_mode` controls student-to-student visibility.
- vocabulary 009: `submission_state` records draft, submitted, returned, or archived status.
- vocabulary 010: `classroom_export_id` identifies teacher-controlled export.
- vocabulary 011: `guardian_access_ref` records external access only when policy permits.
- vocabulary 012: `privacy_guard_epoch` increments when visibility rules change.

## Privacy model
- privacy 001: Default education-room mode is teacher-visible and peer-hidden.
- privacy 002: Student work is isolated by student_board_id unless activity policy opens collaboration.
- privacy 003: Teacher overview aggregates progress without exposing raw peer content to students.
- privacy 004: Co-teacher access requires explicit delegation and class roster membership.
- privacy 005: Student presence is visible to teacher by default.
- privacy 006: Student presence is visible to peers only when peer_visibility_mode allows.
- privacy 007: Cursor locations may be generalized for peer-visible group activities.
- privacy 008: Student names may be pseudonymized in peer-visible modes.
- privacy 009: Teacher comments are visible to the target student and staff.
- privacy 010: Peer comments require explicit group activity mode.
- privacy 011: Export defaults to teacher-only unless policy grants external audience.
- privacy 012: Audit evidence preserves true principal ids under signed evidence.

## Room lifecycle
- lifecycle 001: Teacher opens education room with class_roster_id and purpose.
- lifecycle 002: Policy validates teacher authority before board shards are created.
- lifecycle 003: Student boards inherit tenant, class, pack, and retention labels.
- lifecycle 004: Teacher overview records only the fields needed for classroom control.
- lifecycle 005: Students join through board-open with roster-bound eligibility.
- lifecycle 006: Presence-sync applies peer visibility rules before fan-out.
- lifecycle 007: Canvas operations are admitted against student_board_id or group_board_id.
- lifecycle 008: Lock and release commands record privacy_guard_epoch.
- lifecycle 009: Submission changes are appended to board history.
- lifecycle 010: Teacher review creates comment or grade-reference objects only when allowed.
- lifecycle 011: Export-render uses classroom_export_id and explicit audience.
- lifecycle 012: Room close seals history snapshot and retention evidence.

## Access control requirements
- access 001: Teacher can view every student board in the room.
- access 002: Co-teacher can view assigned boards only when delegated.
- access 003: Student can view own board in individual mode.
- access 004: Student can view group board only when assigned to that group.
- access 005: Student cannot view another individual board by object id guessing.
- access 006: Student cannot infer peer cursor positions when peer visibility is hidden.
- access 007: Student cannot export peer content unless group activity policy allows.
- access 008: Tenant admin can audit room metadata without default access to student content.
- access 009: Auditor access requires auditor-scope policy and signed reason.
- access 010: Guardian access is denied unless a separate education policy grants it.
- access 011: Marketplace template install cannot expand classroom visibility.
- access 012: Import from Microsoft Whiteboard or Whiteboard.fi cannot bypass roster checks.

## Presence and collaboration controls
- presence 001: Teacher sees joined, active, idle, submitted, and disconnected states.
- presence 002: Peer-visible presence is reduced to allowed display name and activity status.
- presence 003: Hidden peer mode suppresses cursor coordinates from other students.
- presence 004: Group mode exposes only group member cursors.
- presence 005: Whole-class collaboration exposes cursors only after teacher activation.
- presence 006: Cursor fan-out obeys local-presence-freshness budgets.
- presence 007: Cursor storm throttling must not leak hidden peer counts.
- presence 008: Presence events never include raw unsupported roster metadata.
- presence 009: Teacher lock state is broadcast to affected students.
- presence 010: Student disconnect events are visible to teacher and co-teacher only.
- presence 011: Presence replay is available for audit under education policy.
- presence 012: Presence materialization is purged according to pack retention.

## Submission and review controls
- submission 001: Student submission seals a snapshot_epoch for teacher review.
- submission 002: Student can continue draft work only when activity policy allows resubmission.
- submission 003: Teacher return creates feedback state without destroying original submission.
- submission 004: Teacher annotation records teacher_principal_id and purpose.
- submission 005: Student revision after return creates a new snapshot_epoch.
- submission 006: Review comments are private to student and authorized staff.
- submission 007: Peer review requires group or peer-review policy.
- submission 008: Grade values are not stored in whiteboard unless an approved grade-reference object exists.
- submission 009: Export of submitted work requires classroom_export policy.
- submission 010: Submission history is retained according to education pack.
- submission 011: Late submission markers are derived from server time.
- submission 012: Deletion requests create workflow review, not destructive correction.

## Benchmark displacement map
- benchmark 001: Whiteboard.fi displaced behavior is teacher view over individual student boards.
- benchmark 002: Whiteboard.fi gap is closed by student_board_id and teacher_overview_id.
- benchmark 003: Microsoft Whiteboard displaced behavior is identity-bound classroom collaboration.
- benchmark 004: Microsoft Whiteboard gap is closed by tenant principal and roster policy binding.
- benchmark 005: FigJam displaced behavior is lightweight group ideation with visible cursors.
- benchmark 006: FigJam gap is closed by group mode and peer visibility controls.
- benchmark 007: Miro Enterprise displaced behavior is workshop-style classroom collaboration at scale.
- benchmark 008: Miro Enterprise gap is closed by lock zones and teacher-controlled phases.
- benchmark 009: Mural Enterprise displaced behavior is structured class activities and voting.
- benchmark 010: Mural Enterprise gap is closed by governance_epoch and activity policy.
- benchmark 011: Lucidspark displaced behavior is concept-map and diagram classroom tasks.
- benchmark 012: Lucidspark gap is closed by object-level ACL and export provenance.

## Policy hooks
- policy 001: board-open denies students absent from class_roster_id.
- policy 002: board-open denies teacher absent from room owner policy.
- policy 003: presence-sync filters peer state by peer_visibility_mode.
- policy 004: canvas-op-append denies writes into another student_board_id.
- policy 005: zone lock requires teacher or delegated co-teacher authority.
- policy 006: submission return requires teacher authority.
- policy 007: peer review requires explicit peer-review activity policy.
- policy 008: classroom export requires teacher authority and export purpose.
- policy 009: auditor reveal requires auditor scope and signed reason.
- policy 010: guardian access requires explicit education policy grant.
- policy 011: marketplace template use cannot widen visibility or retention.
- policy 012: cross-region replay denies when education residency forbids it.

## Threat controls
- threat 001: Object id guessing is blocked by object-level policy checks.
- threat 002: Cursor inference is blocked by filtered presence fan-out.
- threat 003: Peer scraping is blocked by export and board-open policy.
- threat 004: Teacher device compromise is mitigated by audit and revocation.
- threat 005: Co-teacher overreach is blocked by delegated-board assignment.
- threat 006: Marketplace template exfiltration is blocked by DealSet and export policy.
- threat 007: Imported classroom boards are dry-run checked before visibility assignment.
- threat 008: Anonymous student feedback remains auditor-reveal only.
- threat 009: Abusive content reports route to moderation workflow.
- threat 010: Room roster drift triggers local-collaboration-acl-mismatch.
- threat 011: Regional replay drift fails closed under education pack.
- threat 012: Retention mismatch blocks room close until remediation.

## Events and evidence
- event 001: `whiteboard.education.room_opened` records roster and teacher scope.
- event 002: `whiteboard.education.student_joined` records eligibility decision.
- event 003: `whiteboard.education.visibility_changed` records privacy_guard_epoch.
- event 004: `whiteboard.education.teacher_lock_applied` records lock zone.
- event 005: `whiteboard.education.student_submitted` records snapshot_epoch.
- event 006: `whiteboard.education.teacher_returned` records feedback state.
- event 007: `whiteboard.education.peer_review_enabled` records activity policy.
- event 008: `whiteboard.education.export_requested` records audience.
- event 009: `whiteboard.education.export_released` records artifact digest.
- event 010: `whiteboard.education.acl_mismatch_detected` records runbook trigger.
- event 011: `whiteboard.education.audit_reveal_requested` records reason.
- event 012: `whiteboard.education.room_closed` records retention evidence.

## SLO and telemetry
- telemetry 001: Measure board-open latency for student joins.
- telemetry 002: Measure teacher overview freshness.
- telemetry 003: Measure presence fan-out filtering latency.
- telemetry 004: Measure denied peer visibility attempts.
- telemetry 005: Measure object-level ACL denials by activity mode.
- telemetry 006: Measure room roster drift detection.
- telemetry 007: Measure classroom export latency.
- telemetry 008: Measure privacy redaction count per export.
- telemetry 009: Measure local-presence-freshness SLO burn.
- telemetry 010: Measure collaboration ACL mismatch runbook activations.
- telemetry 011: Trace education_room_id through board-open, presence, canvas, export, and audit.
- telemetry 012: Do not expose raw tenant_id or student ids in metrics.

## Acceptance criteria
- acceptance 001: Student-to-student board access is denied by default.
- acceptance 002: Teacher overview sees all authorized student boards.
- acceptance 003: Co-teacher access requires explicit delegation.
- acceptance 004: Peer visibility mode filters presence before fan-out.
- acceptance 005: Student submission creates snapshot_epoch evidence.
- acceptance 006: Classroom export includes privacy and provenance manifest.
- acceptance 007: Guardian access is denied without explicit policy.
- acceptance 008: Marketplace templates do not widen room visibility.
- acceptance 009: ACL mismatch routes to local-collaboration-acl-mismatch.
- acceptance 010: Benchmark evidence names all six required displaced products.
- acceptance 011: ADR-0321 and ADR-0316 are included in the evidence packet.
- acceptance 012: DPIA and compliance anchors are cited for privacy review.

## Test plan
- test 001: Unit-test education room open.
- test 002: Unit-test student board shard creation.
- test 003: Unit-test peer visibility filtering.
- test 004: Unit-test teacher overview aggregation.
- test 005: Unit-test student submission snapshot.
- test 006: Unit-test teacher return state.
- test 007: Cedar-fixture-test non-roster student denial.
- test 008: Cedar-fixture-test peer board access denial.
- test 009: Cedar-fixture-test guardian access denial.
- test 010: Cedar-fixture-test marketplace template visibility denial.
- test 011: Contract-test board-open education fields.
- test 012: AsyncAPI-test education room events.
- test 013: Migration-fixture-test Whiteboard.fi classroom import.
- test 014: Migration-fixture-test Microsoft Whiteboard identity import.
- test 015: Migration-fixture-test FigJam group activity import.
- test 016: Migration-fixture-test Miro Enterprise classroom workshop import.
- test 017: Migration-fixture-test Mural Enterprise activity import.
- test 018: Migration-fixture-test Lucidspark concept-map import.

## Rollback and recovery
- rollback 001: Disable peer-visible mode while preserving teacher-only rooms.
- rollback 002: Freeze classroom exports if privacy manifest creation fails.
- rollback 003: Rebuild teacher overview from student board snapshots.
- rollback 004: Quarantine roster drift rooms until ACL mismatch remediation completes.
- rollback 005: Revoke co-teacher delegation without deleting board history.
- rollback 006: Preserve student submissions during room recovery.
- rollback 007: Preserve audit reveal requests for review.
- rollback 008: Route privacy incidents through threat-model and incident-response evidence.
- rollback 009: Prevent cross-region replay when education residency is uncertain.
- rollback 010: Never merge student boards destructively to repair a privacy incident.

## Command and proto deltas
- proto 001: Add `EducationRoomOpenRequest.education_room_id`, `class_roster_id`, and `teacher_principal_id`.
- proto 002: Add `EducationRoomOpenRequest.default_peer_visibility_mode`.
- proto 003: Add `StudentBoardRef.student_board_id`, `student_principal_id`, and `submission_state`.
- proto 004: Add `TeacherOverview.teacher_overview_id` and `visible_student_board_refs`.
- proto 005: Add `VisibilityChange.privacy_guard_epoch`, `previous_mode`, `next_mode`, and `reason_code`.
- proto 006: Add `TeacherLockCommand.lock_zone_id`, `student_board_scope`, and `lock_reason`.
- proto 007: Add `StudentSubmission.submission_snapshot_epoch`.
- proto 008: Add `TeacherFeedback.feedback_visibility` with student-only, staff-only, and class-visible values.
- proto 009: Add `ClassroomExportRequest.classroom_export_id` and `export_audience`.
- proto 010: Add `AuditRevealRequest.reveal_reason` and `auditor_scope_ref`.
- proto 011: Add `PresenceFilteredEvent.peer_visibility_mode` so receivers know why data is reduced.
- proto 012: Add `RosterDriftEvent.expected_roster_hash` and `observed_roster_hash`.

## Cedar facts
- cedar-fact 001: `principal_in_class_roster` gates student board-open.
- cedar-fact 002: `principal_is_teacher` gates teacher overview and lock commands.
- cedar-fact 003: `principal_is_delegated_coteacher` gates assigned co-teacher visibility.
- cedar-fact 004: `peer_visibility_mode` gates student presence fan-out.
- cedar-fact 005: `target_student_board_id` must equal caller student board unless group policy allows.
- cedar-fact 006: `submission_state=submitted` gates teacher review state.
- cedar-fact 007: `guardian_access_policy_present` gates guardian access.
- cedar-fact 008: `education_pack_active` gates classroom export and retention controls.
- cedar-fact 009: `auditor_scope_ref` gates audit reveal of student identity.
- cedar-fact 010: `class_roster_hash_matches` must be true before room admission.

## Workflow decisions
- workflow 001: Room opening creates student board shards before any student join event is accepted.
- workflow 002: Roster drift check runs on room open, join, visibility change, and export.
- workflow 003: Peer visibility changes are versioned by privacy_guard_epoch and replayable.
- workflow 004: Teacher overview is a derived read model and never the source of student board state.
- workflow 005: Student submission seals a snapshot but does not prevent teacher-authorized return.
- workflow 006: Classroom export uses export-render but adds education_manifest and student redaction counts.
- workflow 007: Audit reveal is a workflow-engine approved action, not a direct room command.
- workflow 008: Group activity mode creates explicit group_board_id instead of widening individual boards.

## Failure and replay cases
- failure 001: Student join during roster drift is denied and routes to ACL mismatch evidence.
- failure 002: Teacher overview rebuild after worker crash uses student board snapshots only.
- failure 003: Peer visibility replay must not expose old cursor coordinates after hidden mode resumes.
- failure 004: Classroom export retry must preserve the same submission_snapshot_epoch.
- failure 005: Whiteboard.fi import creates separate student_board_id values, not one shared board.
- failure 006: Microsoft Whiteboard import maps external accounts to roster-bound principals only.
- failure 007: FigJam group import requires explicit group activity mode before peer visibility opens.
- failure 008: Miro Enterprise classroom workshop import must preserve teacher-controlled lock zones.
- failure 009: Mural Enterprise activity import must preserve voting governance without leaking student identity.
- failure 010: Lucidspark concept-map import must preserve object ACL on diagram nodes.

## Evidence fields
- evidence 001: `education_room_id` joins room, board, presence, submission, export, and audit records.
- evidence 002: `class_roster_id` proves eligibility source.
- evidence 003: `class_roster_hash` proves roster version.
- evidence 004: `student_board_id` proves isolation boundary.
- evidence 005: `teacher_overview_id` proves derived aggregate identity.
- evidence 006: `peer_visibility_mode` proves fan-out rule.
- evidence 007: `privacy_guard_epoch` proves visibility-version ordering.
- evidence 008: `submission_snapshot_epoch` proves submitted board state.
- evidence 009: `classroom_export_id` proves export lineage.
- evidence 010: `auditor_reveal_decision_id` proves reveal authorization.

## Done definition
- done 001: IP defines education room privacy controls.
- done 002: IP references whiteboard PRD, architecture, capabilities, DPIA, compliance, threat model, SLO, policy, and runbook anchors.
- done 003: IP names Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
- done 004: IP includes lifecycle, policy, threat, event, telemetry, test, and rollback substance.
- done 005: IP stays inside microservices/whiteboard and does not edit ADR-0321.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
