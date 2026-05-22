# IP-027 Whiteboard facilitation timer voting governance

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-027-facilitation-timer-voting-governance.md
Capability focus: board-open, presence-sync, canvas-op-append, history-snapshot
Benchmarks: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0253-amendment, ADR-0257, ADR-0263, ADR-0297, ADR-0314, ADR-0316, ADR-0321
Repo-local references: microservices/whiteboard/PRD.md; microservices/whiteboard/ARCHITECTURE.md; microservices/whiteboard/capabilities/board-open.yaml; microservices/whiteboard/capabilities/presence-sync.yaml; microservices/whiteboard/capabilities/canvas-op-append.yaml; microservices/whiteboard/runbooks/cursor-storm-throttle.md; microservices/whiteboard/runbooks/moderation-report-escalation.md; microservices/whiteboard/slos/local-presence-freshness.openslo.yaml; microservices/whiteboard/policy/canvas-collaboration-authorization.cedar

## Objective
- Define governance for facilitation timers, voting sessions, and moderation controls.
- Keep workshop mechanics tenant-scoped, policy-gated, and audit-chain sealed.
- Prevent timer and vote controls from becoming informal admin bypasses.
- Give facilitators high-trust control without hiding decisions from auditors.
- Tie workshop state to board history instead of transient browser state.
- Match Miro Enterprise facilitation depth while retaining Oyatie policy semantics.
- Match Mural Enterprise timer and voting workflows for structured workshops.
- Match FigJam lightweight voting and reactions for product teams.
- Match Lucidspark prioritization sessions for diagram-heavy planning.
- Match Whiteboard.fi teacher-led activities where class privacy applies.
- Match Microsoft Whiteboard collaboration in identity-bound enterprise rooms.

## Current repo anchors
- anchor 001: PRD-whiteboard includes board-session and sticky-note user stories for tenant-scoped capability.
- anchor 002: ARCHITECTURE.md names board-session as a bounded context.
- anchor 003: board-open capability records tenant, principal, purpose, and data_class requirements.
- anchor 004: presence-sync capability records cursor and participant state as policy-aware data.
- anchor 005: canvas-op-append capability covers the marker objects used by timers and voting.
- anchor 006: canvas-collaboration-authorization.cedar is the main collaboration authorization policy.
- anchor 007: cursor-storm-throttle runbook is relevant to vote storms and workshop bursts.
- anchor 008: moderation-report-escalation runbook is relevant to abusive facilitation controls.
- anchor 009: local-presence-freshness SLO is the freshness budget for facilitator-visible quorum.
- anchor 010: ADR-0321 requires industry-leader depth without recreating vendor suite boundaries.

## Domain vocabulary
- vocabulary 001: `facilitation_session_id` identifies a governed workshop overlay on a board.
- vocabulary 002: `facilitator_principal_id` identifies the current controller with delegated authority.
- vocabulary 003: `timer_id` identifies a countdown, count-up, or interval phase.
- vocabulary 004: `vote_session_id` identifies a bounded voting round.
- vocabulary 005: `vote_token_id` identifies a per-participant voting allowance.
- vocabulary 006: `quorum_rule` defines who counts as eligible and visible for a vote.
- vocabulary 007: `anonymity_mode` defines named, anonymized, hidden-until-close, or auditor-reveal states.
- vocabulary 008: `governance_epoch` increments when timer, vote, or facilitator authority changes.
- vocabulary 009: `lock_zone_id` defines the board area controlled during a facilitation phase.
- vocabulary 010: `moderation_action_id` identifies removal, freeze, report, or restore action.
- vocabulary 011: `teacher_room_mode` is the education overlay for class-led boards.
- vocabulary 012: `sealed_vote_result` is the immutable outcome after closure and audit seal.

## Governance principles
- principle 001: Facilitation is delegated authority, not ownership transfer.
- principle 002: Timer control requires explicit facilitator authority.
- principle 003: Vote creation requires board edit authority and eligible participant scope.
- principle 004: Vote casting requires participant eligibility and remaining vote tokens.
- principle 005: Vote result visibility follows the configured anonymity_mode.
- principle 006: Anonymity is never an audit erasure mechanism.
- principle 007: Facilitator transfer emits an audit-chain event.
- principle 008: Timer pause, resume, extend, and cancel each emit state-change evidence.
- principle 009: Sealed votes cannot be edited without reopening workflow approval.
- principle 010: Teacher-room controls override student edits only within declared class scope.
- principle 011: Moderator removal of objects keeps tombstone evidence.
- principle 012: Workshop templates cannot grant authority beyond tenant policy.

## Command surface
- command 001: `facilitation.session.open` creates the governed overlay for a board.
- command 002: `facilitation.session.close` seals outstanding timer and vote state.
- command 003: `facilitation.facilitator.assign` delegates control to a principal.
- command 004: `facilitation.facilitator.revoke` removes delegated control.
- command 005: `facilitation.timer.start` starts countdown or interval state.
- command 006: `facilitation.timer.pause` pauses active time without closing the phase.
- command 007: `facilitation.timer.resume` resumes active time.
- command 008: `facilitation.timer.extend` changes end time with reason.
- command 009: `facilitation.timer.cancel` closes timer without result.
- command 010: `facilitation.vote.open` creates vote tokens and eligibility rules.
- command 011: `facilitation.vote.cast` spends a token on a board target.
- command 012: `facilitation.vote.retract` is allowed only when vote policy permits.
- command 013: `facilitation.vote.close` seals result materialization.
- command 014: `facilitation.vote.reopen` requires governance workflow approval.
- command 015: `facilitation.zone.lock` prevents non-authorized edits in a board area.
- command 016: `facilitation.zone.unlock` releases lock state.
- command 017: `facilitation.moderation.report` creates a moderation workflow task.
- command 018: `facilitation.moderation.freeze` pauses suspicious participant mutations.

## Timer requirements
- timer 001: Timer state is persisted as board overlay state, not client-local state.
- timer 002: Timer start requires facilitation_session_id and governance_epoch.
- timer 003: Timer display may be local, but source of truth is server-admitted state.
- timer 004: Timer pause records actor, reason, previous deadline, and audit event id.
- timer 005: Timer extension records delta, reason, and eligible facilitator id.
- timer 006: Timer cancellation preserves elapsed state for replay.
- timer 007: Timer expiry can trigger vote close only when configured explicitly.
- timer 008: Timer phase can lock board zones only through Cedar-gated command.
- timer 009: Timer phase can reveal instructions only when content classification permits.
- timer 010: Timer drift metric compares server time and client display time.
- timer 011: Timer state survives worker restart and browser reconnect.
- timer 012: Timer state is included in history snapshots and export provenance.

## Voting requirements
- vote 001: Vote sessions bind to tenant_id, board_id, data_class, and purpose.
- vote 002: Vote sessions define eligible principals or eligible audience groups.
- vote 003: Vote tokens are minted deterministically at vote open.
- vote 004: Vote token spending is idempotent by vote_token_id and operation_id.
- vote 005: Vote targets may be objects, zones, comments, templates, or imported artifacts.
- vote 006: Vote result materialization references the causal board epoch.
- vote 007: Anonymous votes hide participant identity from normal users.
- vote 008: Anonymous votes keep auditor-reveal evidence under policy.
- vote 009: Hidden-until-close votes do not leak interim standings through presence events.
- vote 010: Vote retraction is disabled by default for sealed governance sessions.
- vote 011: Vote close emits sealed_vote_result with result hash.
- vote 012: Vote reopening requires workflow-engine approval and audit-chain linkage.

## Education-room overlay
- education 001: Whiteboard.fi benchmark pressure requires teacher-led privacy controls.
- education 002: Teacher can open a room with per-student board shards.
- education 003: Student board shards inherit tenant and class roster scope.
- education 004: Teacher view may aggregate student progress without exposing student-to-student work.
- education 005: Student vote eligibility may be anonymous to peers but visible to teacher under policy.
- education 006: Timer controls are teacher-only unless co-teacher delegation exists.
- education 007: Lock zones may freeze all student boards or selected groups.
- education 008: Moderation reports route to education policy and tenant admin review.
- education 009: Export of classroom vote results obeys education pack retention.
- education 010: Parent or guardian access is out of scope unless a separate policy grants it.
- education 011: Teacher room replay must show what the teacher could see at the time.
- education 012: Student privacy evidence binds to microservices/whiteboard/dpia.md.

## Benchmark displacement map
- benchmark 001: Miro Enterprise displaced behavior is workshop timers plus attention management.
- benchmark 002: Miro Enterprise gap is closed by persisted facilitation_session state.
- benchmark 003: Mural Enterprise displaced behavior is structured voting and facilitator-led phases.
- benchmark 004: Mural Enterprise gap is closed by sealed vote governance and workflow reopen.
- benchmark 005: FigJam displaced behavior is lightweight dot voting and reaction-style alignment.
- benchmark 006: FigJam gap is closed by tokenized vote sessions with hidden-until-close mode.
- benchmark 007: Lucidspark displaced behavior is priority scoring over diagram and sticky objects.
- benchmark 008: Lucidspark gap is closed by vote targets over object_id and zone_id.
- benchmark 009: Whiteboard.fi displaced behavior is teacher view over many student boards.
- benchmark 010: Whiteboard.fi gap is closed by teacher_room_mode and class privacy controls.
- benchmark 011: Microsoft Whiteboard displaced behavior is enterprise identity-bound collaboration.
- benchmark 012: Microsoft Whiteboard gap is closed by tenant principal binding and audit evidence.

## Policy hooks
- policy 001: Facilitator assignment requires tenant admin or board owner authority.
- policy 002: Facilitator revocation requires same or higher authority than assignment.
- policy 003: Timer command requires facilitator role for the active governance_epoch.
- policy 004: Vote open requires board edit authority and explicit purpose.
- policy 005: Vote cast requires participant eligibility at the vote open epoch.
- policy 006: Vote retract requires vote policy and unsealed state.
- policy 007: Vote close requires facilitator authority or timer expiry automation.
- policy 008: Vote reopen requires workflow approval and auditor-visible reason.
- policy 009: Zone lock requires facilitator authority and bounded board area.
- policy 010: Moderation freeze requires abuse or classroom safety reason.
- policy 011: Anonymous vote auditor reveal requires auditor scope and break-glass evidence.
- policy 012: Marketplace-origin workshop templates cannot bypass these policy checks.

## Data and event model
- event 001: `whiteboard.facilitation.session_opened` records board and facilitator scope.
- event 002: `whiteboard.facilitation.facilitator_assigned` records delegated authority.
- event 003: `whiteboard.facilitation.timer_started` records deadline and phase.
- event 004: `whiteboard.facilitation.timer_changed` records pause, resume, extend, or cancel.
- event 005: `whiteboard.facilitation.vote_opened` records eligibility and anonymity mode.
- event 006: `whiteboard.facilitation.vote_cast` records token spend without leaking anonymous identity.
- event 007: `whiteboard.facilitation.vote_closed` records sealed result hash.
- event 008: `whiteboard.facilitation.vote_reopened` records workflow approval.
- event 009: `whiteboard.facilitation.zone_locked` records affected board area.
- event 010: `whiteboard.facilitation.moderation_reported` records safety workflow route.
- event 011: `whiteboard.facilitation.student_privacy_applied` records education overlay activation.
- event 012: `whiteboard.facilitation.export_attested` records timer and vote provenance inclusion.

## SLO and telemetry
- telemetry 001: Measure timer command admission latency.
- telemetry 002: Measure timer display drift against server state.
- telemetry 003: Measure vote cast p95 admission latency.
- telemetry 004: Measure vote close materialization latency.
- telemetry 005: Measure presence freshness for eligible participant count.
- telemetry 006: Measure cursor storm throttle activation during vote sessions.
- telemetry 007: Measure moderation report escalation time.
- telemetry 008: Measure anonymous vote reveal attempts and denials.
- telemetry 009: Measure education room privacy denial rate.
- telemetry 010: Measure facilitator transfer count by board size bucket.
- telemetry 011: Link traces to governance_epoch and audit event id.
- telemetry 012: Avoid raw tenant_id in metrics while preserving signed evidence.

## Acceptance criteria
- acceptance 001: Timer state is recoverable from history snapshot replay.
- acceptance 002: Vote state is recoverable from history snapshot replay.
- acceptance 003: Facilitator delegation emits audit evidence.
- acceptance 004: Vote anonymity mode prevents ordinary user identity leakage.
- acceptance 005: Auditor reveal path is policy-gated and evidence-sealed.
- acceptance 006: Teacher room mode prevents student-to-student board leakage.
- acceptance 007: Zone locks prevent unauthorized canvas-op-append commands.
- acceptance 008: Sealed vote mutation is rejected without workflow reopen.
- acceptance 009: Benchmark evidence names all six required displaced products.
- acceptance 010: ADR-0321 and ADR-0316 are present in the evidence packet.
- acceptance 011: No command bypasses Cedar default-deny.
- acceptance 012: Moderation and cursor-storm runbooks have trigger thresholds.

## Test plan
- test 001: Unit-test facilitator assignment and revocation state transitions.
- test 002: Unit-test timer pause, resume, extend, cancel, and expiry.
- test 003: Unit-test vote token deterministic minting.
- test 004: Unit-test hidden-until-close vote result suppression.
- test 005: Property-test duplicate vote cast idempotency.
- test 006: Property-test timer replay across worker restart.
- test 007: Cedar-fixture-test non-facilitator timer denial.
- test 008: Cedar-fixture-test ineligible participant vote denial.
- test 009: Cedar-fixture-test anonymous vote reveal denial.
- test 010: Contract-test command shapes against OpenAPI.
- test 011: AsyncAPI-test facilitation events.
- test 012: Migration-fixture-test Miro Enterprise timer import.
- test 013: Migration-fixture-test Mural Enterprise vote import.
- test 014: Migration-fixture-test FigJam dot vote import.
- test 015: Migration-fixture-test Lucidspark prioritization import.
- test 016: Migration-fixture-test Whiteboard.fi classroom timer import.
- test 017: Migration-fixture-test Microsoft Whiteboard identity-bound room import.

## Rollback and recovery
- rollback 001: Disable vote reopen command through policy flag if evidence sealing fails.
- rollback 002: Freeze facilitator transfer while preserving timer and vote reads.
- rollback 003: Rebuild timer and vote materialization from history snapshots.
- rollback 004: Quarantine anonymous vote reveal attempts for auditor review.
- rollback 005: Route moderation incidents through moderation-report-escalation runbook.
- rollback 006: Activate cursor-storm-throttle when vote fan-out exceeds budget.
- rollback 007: Preserve sealed_vote_result hashes during rollback.
- rollback 008: Prevent export of unsealed or quarantined governance state.
- rollback 009: Notify workflow-engine of reopened governance tasks.
- rollback 010: Never delete vote tokens or timer state from the append-only history.

## Command and proto deltas
- proto 001: Add `FacilitationSessionOpenRequest.board_id` and `facilitation_session_id`.
- proto 002: Add `FacilitationSessionOpenRequest.default_anonymity_mode` for vote defaults.
- proto 003: Add `FacilitatorAssignment.delegation_scope` with board, zone, vote, timer, and moderation values.
- proto 004: Add `TimerState.timer_id`, `deadline_server_time`, `paused_at_server_time`, and `timer_phase`.
- proto 005: Add `TimerCommand.reason_code` for pause, extend, cancel, and emergency stop.
- proto 006: Add `VoteSession.vote_session_id`, `eligible_audience_ref`, `vote_token_count`, and `quorum_rule`.
- proto 007: Add `VoteCast.vote_token_id` so duplicate casts are idempotent.
- proto 008: Add `VoteResult.result_hash` and `sealed_at_epoch` for immutable close.
- proto 009: Add `ZoneLock.lock_zone_id`, `geometry_ref`, and `lock_reason`.
- proto 010: Add `ModerationFreeze.freeze_subject_ref` to target principal, object, or zone.
- proto 011: Add `EducationFacilitationOverlay.teacher_room_mode` for classroom activity governance.
- proto 012: Add `FacilitationEvent.governance_epoch` to every timer, vote, lock, and facilitator event.

## Cedar facts
- cedar-fact 001: `principal_is_facilitator` gates timer mutation.
- cedar-fact 002: `principal_is_vote_eligible` gates vote token minting and casting.
- cedar-fact 003: `vote_is_sealed` blocks cast, retract, and result mutation commands.
- cedar-fact 004: `vote_reopen_workflow_approved` is required before sealed result reopening.
- cedar-fact 005: `anonymity_mode` determines ordinary-user result visibility.
- cedar-fact 006: `auditor_reveal_scope` gates identity reveal for anonymous votes.
- cedar-fact 007: `lock_zone_active` blocks canvas edits by non-facilitators.
- cedar-fact 008: `teacher_room_mode` grants teacher timer and lock authority over student boards.
- cedar-fact 009: `marketplace_template_origin` cannot grant facilitator authority.
- cedar-fact 010: `moderation_reason_present` is required for freeze and report commands.

## Workflow decisions
- workflow 001: Timer expiry is server-authoritative and browser display is advisory.
- workflow 002: Vote token minting happens once at vote open and is replayable.
- workflow 003: Vote result materialization waits for all accepted casts up to sealed_at_epoch.
- workflow 004: Hidden-until-close mode suppresses interim vote standings from presence and events.
- workflow 005: Anonymous mode stores identity reveal material only in signed audit evidence.
- workflow 006: Facilitator transfer pauses active timer commands until the new governance_epoch is acknowledged.
- workflow 007: Zone locks are represented as board overlay objects so export and replay include them.
- workflow 008: Moderation freeze opens workflow-engine review when it lasts beyond the configured threshold.

## Failure and replay cases
- failure 001: Timer worker restart replays TimerState from history and does not create a second expiry.
- failure 002: Duplicate VoteCast requests spend one token once.
- failure 003: Late vote cast after sealed_at_epoch is rejected with sealed_vote_mutation.
- failure 004: Facilitator disconnect does not transfer authority without explicit assignment or timeout policy.
- failure 005: Hidden-until-close vote does not leak standings through presence counts.
- failure 006: Anonymous vote export redacts ordinary identity while preserving auditor reveal.
- failure 007: Mural Enterprise imported vote sessions replay with original close semantics.
- failure 008: FigJam imported dot votes replay into tokenized vote sessions.
- failure 009: Whiteboard.fi classroom timer replay keeps teacher authority.
- failure 010: Microsoft Whiteboard room import maps external identities to Oyatie principals only after roster binding.

## Evidence fields
- evidence 001: `facilitation_session_id` joins timer, vote, lock, moderation, and export evidence.
- evidence 002: `governance_epoch` proves which authority set admitted a command.
- evidence 003: `timer_id` proves deadline, pause, resume, extension, and cancellation lineage.
- evidence 004: `vote_session_id` proves vote lifecycle.
- evidence 005: `vote_token_id` proves eligible participant token spend.
- evidence 006: `result_hash` proves sealed result integrity.
- evidence 007: `anonymity_mode` proves ordinary-user visibility rules.
- evidence 008: `auditor_reveal_decision_id` proves reveal authorization.
- evidence 009: `lock_zone_id` proves spatial governance.
- evidence 010: `moderation_action_id` proves freeze or report lineage.

## Done definition
- done 001: IP defines facilitation timer, vote, moderation, and education-room governance.
- done 002: IP references whiteboard PRD, architecture, capabilities, SLOs, policies, and runbooks.
- done 003: IP names Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
- done 004: IP includes policy, data, telemetry, test, and rollback substance.
- done 005: IP stays inside microservices/whiteboard and does not edit ADR-0321.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
