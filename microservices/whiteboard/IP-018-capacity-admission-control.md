# IP-018 Whiteboard capacity-admission-control

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-018-capacity-admission-control.md
Benchmarks: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo-local references: microservices/whiteboard/PRD.md, microservices/whiteboard/ARCHITECTURE.md, microservices/whiteboard/capacity-model.md, microservices/whiteboard/slos, microservices/whiteboard/dashboards, microservices/whiteboard/failure-modes.md, microservices/whiteboard/capabilities/board-open.yaml, microservices/whiteboard/capabilities/canvas-op-append.yaml, microservices/whiteboard/capabilities/presence-sync.yaml, microservices/whiteboard/capabilities/history-snapshot.yaml, microservices/whiteboard/capabilities/export-render.yaml, microservices/whiteboard/capabilities/template-marketplace-install.yaml

## Objective
- Define admission control for Whiteboard's realtime boards, async snapshots, exports, templates, and replay work.
- Preserve interactive collaboration latency while preventing one tenant, classroom, incident, or migration from exhausting a cell.
- Preserve ADR-0321 anchors for tenant scoping, Cedar, audit, rollback, pack overlay, benchmark parity, SLOs, and operational evidence.
- Treat Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard as displaced scale expectations.
- Apply admission before expensive execution, not after queues are saturated.
- Coordinate with IP-013 emergency priority, IP-016 replay workers, and IP-017 budget enforcement.
- Separate capacity admission from authorization and cost.
- Keep tenant-visible refusal reasons actionable.
- Keep SRE-visible signals low-cardinality and cell-focused.
- Ensure rollback can restore prior admission policy without deleting collaboration history.

## Non-goals
- Do not implement autoscaling or scheduler code in this IP.
- Do not edit capacity-model.md, SLO files, or dashboards.
- Do not use capacity admission to bypass Cedar.
- Do not use capacity admission to bypass residency.
- Do not use capacity admission to bypass DealSet settlement.
- Do not let async replay compete equally with live collaboration.
- Do not globally admit traffic based on vendor labels.
- Do not promise unlimited classroom or emergency traffic.
- Do not update ADR-0321.
- Do not touch files outside IP-018.

## Admission classes
- Class `interactive_open` covers board-open and initial board metadata load.
- Class `interactive_append` covers canvas-op-append and operation ordering.
- Class `interactive_presence` covers presence-sync fanout and membership events.
- Class `snapshot_async` covers history-snapshot creation and repair.
- Class `export_async` covers export-render and regeneration.
- Class `template_install` covers template-marketplace-install materialization.
- Class `replay_async` covers IP-016 replay categories.
- Class `emergency_priority` covers IP-013 emergency boards.
- Class `migration_burst` covers vendor import and backfill.
- Class `admin_audit` covers auditor snapshots and regulator export.
- Each class has queue, concurrency, token, and shed rules.
- Each class records tenant, cell, board family, data class, and pack overlay.
- Each class has a refusal reason.
- Each class has rollback and policy version.
- Each class has SLO mapping.

## Benchmark displacement notes
- Miro Enterprise sets expectations for large collaborative boards and enterprise control.
- Mural Enterprise sets expectations for facilitated sessions with many active users.
- FigJam sets expectations for rapid multiplayer edits and cursor presence.
- Lucidspark sets expectations for diagram-heavy boards and export jobs.
- Whiteboard.fi sets expectations for classroom fanout and teacher-led bursts.
- Microsoft Whiteboard sets expectations for suite-scale tenant availability.
- Oyatie displaces these with explicit admission classes and evidence.
- Benchmark parity requires keeping clean interactive traffic fast under load.
- Benchmark parity also requires refusing or queueing expensive async work predictably.
- Vendor scale claims are insufficient without tenant, cell, and capability evidence.

## Capability binding
- `board-open` checks interactive_open tokens before loading board state.
- `board-open` can admit degraded read-only state when append capacity is exhausted.
- `canvas-op-append` checks interactive_append tokens and board shard health.
- `canvas-op-append` can reject non-critical bulk edits before individual critical edits.
- `presence-sync` checks interactive_presence fanout and sheds cursor cosmetics first.
- `presence-sync` preserves participant membership before pointer detail.
- `history-snapshot` uses snapshot_async queue and does not block live appends.
- `history-snapshot` can be delayed unless required for emergency or regulator workflow.
- `export-render` uses export_async queue and capacity estimate.
- `export-render` can require approval or queue repositioning for large renders.
- `template-marketplace-install` uses template_install capacity after policy, budget, and settlement preflight.
- `template-marketplace-install` cannot materialize assets when capacity token is denied.
- IP-016 replay uses replay_async and lower priority than live collaboration.
- IP-017 cost budget can deny work before admission, but admission can also deny when capacity is scarce.
- IP-013 emergency priority can reserve capacity but still records admission evidence.

## Admission signals
- Signal `active_participant_count` estimates presence fanout.
- Signal `append_rate_per_board` estimates write pressure.
- Signal `board_object_count` estimates load and render pressure.
- Signal `operation_backlog` estimates projection lag.
- Signal `snapshot_queue_depth` estimates history delay.
- Signal `export_queue_depth` estimates render delay.
- Signal `template_materialization_queue_depth` estimates install delay.
- Signal `replay_lag` estimates backfill pressure.
- Signal `cell_cpu_pressure` estimates compute scarcity.
- Signal `cell_memory_pressure` estimates board working-set scarcity.
- Signal `cell_storage_pressure` estimates snapshot and export scarcity.
- Signal `egress_pressure` estimates presence and export transfer pressure.
- Signal `audit_chain_backpressure` prevents high-risk mutation admission.
- Signal `policy_latency` prevents unsafe admission during control-plane slowness.
- Signal `pack_overlay_constraint` prevents cross-cell capacity shortcuts.

## Admission policy model
- Admission policy evaluates tenant tier, pack overlays, board size, participant count, action class, and cell health.
- Admission returns admit, admit-degraded, queue, shed-cosmetic, require-approval, or deny.
- Admit-degraded is allowed for board-open read-only state and presence cosmetic shedding.
- Queue is allowed for snapshots, exports, templates, and replay.
- Require-approval is allowed for large exports and migration bursts.
- Deny is used for unsafe, unauthorized, or impossible capacity states.
- Emergency priority can admit ahead of ordinary async work.
- Emergency priority cannot move content across forbidden residency boundaries.
- Admin audit can preempt non-critical replay.
- Replay async yields to interactive classes.
- Template install yields to board-open and canvas-op-append.
- Export async yields to live collaboration unless regulator workflow requires priority.
- Admission decision includes policy_version, capacity_snapshot_id, and audit_event_id.
- Admission policy changes are staged by cell.
- Rollback restores previous policy version.

## Data requirements
- `admission_decision_id` identifies each decision.
- `capacity_class` records the admission class.
- `tenant_id` is present in audit evidence.
- `tenant_hash` is used in high-cardinality metrics.
- `board_id` is present for board-scoped actions.
- `cell_id` records execution cell.
- `pack_overlay_result` records residency constraints.
- `budget_decision_id` links IP-017 where applicable.
- `policy_decision_id` links Cedar authority.
- `capacity_snapshot_id` records measured load.
- `admission_result` records admit, degraded, queue, shed, approval, or deny.
- `refusal_reason` records user-facing explanation.
- `queue_position` records async queue status where safe to expose.
- `shed_detail` records which cosmetic details were dropped.
- `audit_event_id` seals the decision.

## Implementation plan
- Step 1: Define capacity classes and admission DTOs for all six capabilities.
- Step 2: Add admission preflight after Cedar and before expensive execution.
- Step 3: Add board-open degraded read-only response shape.
- Step 4: Add canvas append admission by board shard and operation rate.
- Step 5: Add presence fanout admission with cursor cosmetic shedding.
- Step 6: Add snapshot async queue admission and regulator priority.
- Step 7: Add export async admission with render complexity estimate.
- Step 8: Add template install admission after settlement and budget preflight.
- Step 9: Add replay async admission tied to IP-016 worker lease and cursor.
- Step 10: Add emergency priority reservation tied to IP-013.
- Step 11: Add cost-budget coordination tied to IP-017.
- Step 12: Add pack overlay guardrails tied to IP-015.
- Step 13: Add dashboards for admission results, queue depth, shed rate, and refusal reasons.
- Step 14: Add SLO annotations for interactive and async classes.
- Step 15: Add runbook entries for cell pressure, presence storm, append storm, export backlog, and replay pause.
- Step 16: Add scorecard rows for admission evidence and degraded behavior.
- Step 17: Add benchmark parity rows for scale expectations across all six displaced vendors.
- Step 18: Add failure-mode handling for stale capacity snapshots.
- Step 19: Add rollback bundle for bad admission policy rollout.
- Step 20: Add tests proving capacity cannot bypass policy, residency, audit, settlement, or budget.

## Operational controls
- Interactive classes have explicit token pools separate from async classes.
- Emergency priority has reserve capacity and tight audit.
- Replay async has a lower ceiling and must pause under live load.
- Export async has queue visibility and approval path for large renders.
- Snapshot async has regulator priority lane.
- Template install does not start materialization until capacity is admitted.
- Presence shedding drops cursor cosmetics before membership.
- Board-open can show read-only degraded state when append capacity is scarce.
- SRE dashboards aggregate by cell, capacity class, and tenant_hash.
- Tenant admins see board-level queue and denial reasons.
- Auditors can export capacity decisions for regulated boards.
- Admission policy versions are staged and rollbackable.
- Capacity snapshots are bounded in age.
- Admission refusals are idempotent for duplicate commands.
- Audit-chain backpressure blocks high-risk admissions.

## Failure modes
- Capacity snapshot stale: deny expensive work and allow only safe low-cost reads.
- Admission service unavailable: fail closed for mutations and async work, allow cached board-open read only if policy allows.
- Cell CPU pressure: queue async, shed presence cosmetics, and protect append.
- Cell memory pressure: deny large board-open expansions and queue exports.
- Egress pressure: reduce presence detail and queue exports.
- Audit-chain backpressure: stop high-risk appends and materialization.
- Policy latency spike: queue non-critical work until Cedar is healthy.
- Pack overlay forbids alternate cell: do not reroute content.
- Emergency reserve exhausted: alert and apply IP-013 safety path with evidence.
- Replay starving live traffic: pause replay and preserve cursor.
- Export backlog grows: require approval for new large exports.
- Template materialization backlog grows: block new paid materialization after reservation expiry.
- Budget denial and capacity denial conflict: report budget denial first when it occurred earlier.
- Duplicate command after queue: return existing queue state.
- Bad policy rollout: rollback to previous admission policy and preserve decisions.

## Evidence and tests
- Evidence 1: Board-open can admit degraded read-only state.
- Evidence 2: Canvas append is protected from replay and export backlog.
- Evidence 3: Presence sync sheds cursor cosmetics before membership.
- Evidence 4: Snapshot async queues without blocking live collaboration.
- Evidence 5: Export async queues or requires approval based on render complexity.
- Evidence 6: Template install waits for settlement, budget, policy, and capacity.
- Evidence 7: Replay async pauses under live load.
- Evidence 8: Emergency priority uses reserve capacity and emits audit.
- Evidence 9: Pack overlay blocks forbidden cross-cell admission.
- Evidence 10: Budget and capacity decisions are linked but distinct.
- Evidence 11: Negative tests prove admission cannot bypass Cedar.
- Evidence 12: Negative tests prove admission cannot bypass residency.
- Evidence 13: Negative tests prove admission cannot bypass DealSet settlement.
- Evidence 14: Benchmark parity maps scale and admin controls for all six vendors.
- Evidence 15: ADR-0321 matrix covers SLO, tenant, Cedar, audit, pack, rollback, and benchmark anchors.

## Capacity-specific domain and contract deltas
- Domain aggregate: `whiteboard_capacity_admission` owns capacity class, measured snapshot, result, and refusal reason.
- Domain invariant: `capacity_snapshot_id` must be newer than the policy-defined maximum age.
- Domain invariant: admission cannot create authority; it only gates already authorized work.
- Domain invariant: degraded board-open cannot expose content forbidden by Cedar or residency.
- Domain invariant: presence shedding cannot remove participant membership evidence.
- Domain event `whiteboard.capacity.admitted` records class, cell, and policy version.
- Domain event `whiteboard.capacity.degraded` records degraded feature and shed detail.
- Domain event `whiteboard.capacity.queued` records async queue class and position.
- Domain event `whiteboard.capacity.denied` records refusal reason and remediation.
- Domain event `whiteboard.capacity.policy_rolled_back` records prior and restored policy version.
- OpenAPI delta: commands receive `admission_context` after policy and budget preflight.
- OpenAPI delta: degraded board-open response includes `read_only_reason`, `append_admission_state`, and `presence_admission_state`.
- AsyncAPI delta: emit `whiteboard.capacity.decision.v1` for every non-admit result.
- AsyncAPI delta: emit `whiteboard.capacity.queue.updated.v1` for async queue movement.
- Proto delta: internal `CapacityAdmissionRequest` carries class, cost estimate, pack result, and cell snapshot.
- Proto delta: internal `CapacityAdmissionDecision` carries admit, degraded, queue, shed, approval, or deny.
- Cedar fact: `context.capacity_class` must match requested action.
- Cedar fact: `context.emergency_priority == true` requires emergency board evidence from IP-013.
- Cedar fact: `context.cross_cell_admission == true` requires pack overlay permission from IP-015.
- Cedar fact: `context.template_install == true` requires settlement and budget preflight ids.
- Workflow decision: admission queue for exports is visible to tenant admins.
- Workflow decision: admission queue for replay is visible to SRE and workflow owner.
- Workflow decision: append storms trigger tenant workflow only after soft mitigation.
- Workflow decision: bad policy rollout uses cell-staged rollback, not global reset.
- SLO: board-open admission p95 target is 20 ms from warmed cell snapshot.
- SLO: canvas append admission p95 target is 15 ms from shard-local counters.
- SLO: presence shed decision p95 target is 10 ms from fanout counters.
- SLO: async queue position update target is 5 seconds.
- Replay case: replay_async admission pauses IP-016 worker before live traffic is affected.
- Replay case: capacity policy replay reconstructs decision evidence after metrics outage.
- Replay case: queued export survives policy rollback with original queue evidence.
- Rollback: bad admission policy restores previous policy version per cell.
- Rollback: incorrect denial replays admission with corrected snapshot and preserves original denial event.
- Rollback: queue corruption freezes async queue and preserves admitted live traffic.
- Test case: stale capacity snapshot denies expensive async work.
- Test case: emergency priority cannot be used without IP-013 incident evidence.
- Test case: presence shed preserves membership and drops cursor cosmetics first.
- Test case: pack-forbidden alternate cell is denied even when local cell is saturated.
- Test case: template materialization is denied when admission lacks settlement preflight id.
- Evidence field: `capacity_snapshot_age_ms` proves freshness.
- Evidence field: `shed_detail` proves which non-critical details were dropped.
- Evidence field: `queue_reason` explains async delay or denial.

## Acceptance criteria
- AC-001: Admission classes are explicit and capability-bound.
- AC-002: Interactive and async traffic have separate treatment.
- AC-003: ADR-0321 remains listed and unmodified.
- AC-004: All six benchmark names are present exactly.
- AC-005: Admission, cost, emergency, replay, residency, and settlement interactions are concrete.
- AC-006: Degraded behavior is defined for board-open and presence-sync.
- AC-007: Failure modes cover capacity, policy, audit, pack, emergency, replay, export, and template paths.
- AC-008: Rollback restores policy versions without deleting evidence.
- AC-009: Repo-local references include capacity-model.md, SLOs, dashboards, and failure-modes.md.
- AC-010: Admission evidence records board class, active participant bucket, operation-rate bucket, selected shed mode, rejected capability, and user-visible retry guidance.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
