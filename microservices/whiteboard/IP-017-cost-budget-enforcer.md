# IP-017 Whiteboard cost-budget-enforcer

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-017-cost-budget-enforcer.md
Benchmarks: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo-local references: microservices/whiteboard/PRD.md, microservices/whiteboard/ARCHITECTURE.md, microservices/whiteboard/cost-budget.md, microservices/whiteboard/capacity-model.md, microservices/whiteboard/capabilities/board-open.yaml, microservices/whiteboard/capabilities/canvas-op-append.yaml, microservices/whiteboard/capabilities/presence-sync.yaml, microservices/whiteboard/capabilities/history-snapshot.yaml, microservices/whiteboard/capabilities/export-render.yaml, microservices/whiteboard/capabilities/template-marketplace-install.yaml, microservices/whiteboard/dashboards, microservices/whiteboard/scorecards

## Objective
- Define cost controls for Whiteboard's interactive and async workloads.
- Preserve B2B leader parity without letting collaboration spikes create unbounded tenant or platform spend.
- Preserve ADR-0321 anchors for tenant scoping, audit, rollback, pack overlay, benchmark parity, and operational evidence.
- Track cost by tenant, capability, source vendor, workflow template, cell, data class, board id, and replay category.
- Treat Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard as displaced cost and administration expectations.
- Enforce budget differently for live collaboration, emergency boards, marketplace templates, replay, and exports.
- Keep cost enforcement visible to tenant admins without exposing other tenants or raw internal metrics.
- Avoid surprise denial for clean interactive collaboration until configured soft limits are exceeded.
- Block expensive async or commercial actions when budget authority is missing.
- Preserve audit-chain evidence for every budget denial and override.

## Non-goals
- Do not build corporate billing or invoice systems in this IP.
- Do not edit cost-budget.md or capacity-model.md.
- Do not replace marketplace DealSet settlement from ADR-0314.
- Do not throttle emergency boards in a way that overrides IP-013 safety rules.
- Do not use raw tenant_id in high-cardinality metrics.
- Do not make cost controls a substitute for capacity admission.
- Do not permit budget override without Cedar and audit.
- Do not define code, queues, or dashboards here.
- Do not edit ADR-0321.
- Do not touch files outside IP-017.

## Cost dimensions
- Dimension `tenant` is present in signed audit and internal budget records.
- Dimension `tenant_hash` is used for metrics where raw tenant id would explode cardinality.
- Dimension `capability` records board-open, canvas-op-append, presence-sync, history-snapshot, export-render, template-marketplace-install, and replay categories.
- Dimension `cell` records home and execution cell.
- Dimension `data_class` records board_object, canvas_operation, presence_cursor, export_snapshot, and marketplace_asset.
- Dimension `source_vendor` records displaced import or template source where relevant.
- Dimension `workflow_template` records incident, migration, export, classroom, or collaboration workflow.
- Dimension `board_id` is used in audit and sampled cost analysis.
- Dimension `marketplace_pack_id` links paid template cost and settlement.
- Dimension `replay_id` links backfill work to cost.
- Dimension `export_profile` links render complexity.
- Dimension `presence_fanout_size` estimates realtime load.
- Dimension `operation_rate` estimates canvas append cost.
- Dimension `snapshot_range` estimates history and storage cost.
- Dimension `emergency_priority` identifies safety priority separately from ordinary spikes.

## Benchmark displacement notes
- Miro Enterprise and Mural Enterprise create expectations for admin-visible usage controls.
- FigJam creates expectations for low-friction creation and many lightweight boards.
- Lucidspark creates expectations for diagram-heavy boards and export workloads.
- Whiteboard.fi creates expectations for short-lived classroom boards with many participants.
- Microsoft Whiteboard creates expectations for suite-admin quotas and tenant policy.
- Oyatie displaces these expectations with capability-specific budget evidence.
- Benchmark parity requires admin clarity and predictable guardrails.
- Benchmark parity does not require unlimited free replay, export, or marketplace materialization.
- Cost controls should be invisible to normal clean collaboration until budget policy says otherwise.
- Every denial must explain the budget dimension and remediation path.

## Capability binding
- `board-open` performs cheap budget read and can display budget degraded banners.
- `board-open` cannot be blocked by async replay cost unless board access itself is over tenant hard limit.
- `canvas-op-append` tracks operation rate and write amplification.
- `canvas-op-append` uses soft friction before hard denial for ordinary collaboration.
- `presence-sync` tracks fanout size and cell egress.
- `presence-sync` sheds cursor cosmetics before participant membership.
- `history-snapshot` estimates storage and replayable version range cost.
- `history-snapshot` can be delayed when non-critical budget is exceeded.
- `export-render` checks render complexity, asset license, residency cell, and budget.
- `export-render` can require tenant admin approval above threshold.
- `template-marketplace-install` checks budget and DealSet settlement independently.
- `template-marketplace-install` blocks paid pack use when commercial or budget proof is missing.
- Replay cost is governed through IP-016 categories.
- Emergency priority is governed through IP-013 and still emits budget telemetry.
- Capacity decisions are coordinated with IP-018 but not replaced by cost controls.

## Budget policy model
- Budget policy includes soft limit, hard limit, approval threshold, emergency reserve, and async reserve.
- Soft limit emits warning and dashboard signal.
- Approval threshold routes to workflow-engine for tenant admin approval.
- Hard limit denies non-critical expensive actions.
- Emergency reserve is not consumed by ordinary collaboration.
- Async reserve is used by replay, snapshot repair, and export regeneration.
- Marketplace reserve is used for paid template installation and premium assets.
- Cost policy evaluates tenant pack, tier, board class, capability, and workflow template.
- Cedar decision records whether a budget override is allowed.
- Budget override has expiry, amount, principal, and reason.
- Budget override cannot bypass DealSet settlement.
- Budget override cannot bypass residency.
- Budget override cannot bypass audit.
- Budget policy is versioned and auditable.
- Budget policy changes affect future actions and do not rewrite past cost evidence.

## Data requirements
- `budget_decision_id` is required for denied or overridden actions.
- `cost_event_id` identifies emitted telemetry.
- `tenant_budget_period` records budget window.
- `capability_cost_class` records interactive, async, commercial, emergency, or export.
- `estimated_units` records expected cost before action.
- `actual_units` records consumed cost after action.
- `unit_basis` records operation, participant-minute, render-megapixel, snapshot-version, replay-object, template-install, or storage-gb-day.
- `soft_limit_state` records under, warning, approval-required, hard-denied, or emergency-reserve.
- `budget_override_id` records approved override.
- `budget_reason` records user-facing denial or warning reason.
- `cost_tags` match PRD cost dimensions.
- `audit_event_id` seals warnings, denials, and overrides.
- `marketplace_pack_id` links cost and settlement for template installs.
- `replay_id` links cost and worker batches.
- `pack_overlay_result` records pack-imposed cost behavior.

## Implementation plan
- Step 1: Define budget decision DTOs for all six capabilities.
- Step 2: Add Cedar budget policy actions for estimate, warn, approve, deny, and override.
- Step 3: Add cheap budget read path for board-open.
- Step 4: Add operation-rate cost emission for canvas-op-append.
- Step 5: Add fanout and egress cost emission for presence-sync.
- Step 6: Add snapshot range cost estimation for history-snapshot.
- Step 7: Add render complexity estimation for export-render.
- Step 8: Add template install cost and settlement linkage for template-marketplace-install.
- Step 9: Add replay batch cost linkage to IP-016 replay categories.
- Step 10: Add emergency priority budget telemetry to IP-013 without blocking safety path.
- Step 11: Add capacity coordination signals for IP-018 admission.
- Step 12: Add dashboard panels for soft limit, hard denial, override, and emergency reserve.
- Step 13: Add scorecard rows for budget denial evidence.
- Step 14: Add runbook entries for runaway board, export spike, replay cost spike, and marketplace cost dispute.
- Step 15: Add compliance notes for cost evidence that contains tenant-sensitive information.
- Step 16: Add benchmark parity rows for admin usage controls across all six displaced vendors.
- Step 17: Add tests for soft warning and hard denial.
- Step 18: Add tests for budget override expiry.
- Step 19: Add tests proving budget cannot bypass settlement, residency, or audit.
- Step 20: Add rollback bundle behavior for mistaken budget policy rollout.

## Operational controls
- Interactive clean traffic gets warnings before denial where tenant policy allows.
- Hard-denial applies immediately to expensive async work when budget is exhausted.
- Emergency boards use emergency reserve and emit high-priority cost evidence.
- Replay jobs pause before consuming live collaboration budget.
- Export jobs can be queued for approval.
- Marketplace installs require both budget and DealSet proof.
- Tenant admins see budget state by capability and board family.
- SRE sees cell-level spend and runaway workloads without raw tenant id metrics.
- Auditors can trace override principal, reason, expiry, and audit event.
- Support operators cannot grant budget overrides without tenant authority.
- Cost events are idempotent by action id.
- Actual cost reconciliation updates the same cost event rather than emitting conflicting records.
- Budget policy changes have staged rollout and rollback bundle.
- Anomaly detection receives operation-rate and fanout spikes.
- Budget denial copy avoids leaking internal cost formulas.

## Failure modes
- Budget service unavailable: allow low-cost reads, deny expensive async and commercial actions unless existing approval covers them.
- Cost telemetry queue unavailable: buffer bounded telemetry and stop high-cost work before loss.
- Cedar unavailable: deny new override and expensive actions.
- Marketplace settlement available but budget unavailable: block paid install until budget proof exists.
- Budget available but DealSet unavailable: block paid install until settlement proof exists.
- Emergency reserve exhausted: alert tenant admin and SRE, continue safety path only within IP-013 policy.
- Export estimate too low: emit reconciliation event and adjust future estimates.
- Replay cost spike: pause replay and preserve cursor.
- Presence fanout spike: shed cursor cosmetics before membership.
- Canvas operation storm: rate-limit non-critical appends after warnings.
- Dashboard outage: continue enforcement and emit audit events.
- Cross-tenant budget id: deny and emit refusal.
- Override expired: revert to current budget policy.
- Pack overlay changes budget class: apply stricter class and emit policy-change evidence.
- Cost rollback needed: restore prior budget policy and keep all cost events.

## Evidence and tests
- Evidence 1: Board-open exposes budget state without heavy cost computation.
- Evidence 2: Canvas append emits operation-rate cost events.
- Evidence 3: Presence sync emits fanout cost and sheds cosmetics under pressure.
- Evidence 4: Snapshot and export estimate cost before execution.
- Evidence 5: Template install requires both budget and DealSet proof.
- Evidence 6: Replay pauses when async reserve is exhausted.
- Evidence 7: Emergency boards emit reserve telemetry and preserve safety policy.
- Evidence 8: Budget override requires Cedar, expiry, reason, and audit.
- Evidence 9: Negative tests prove budget override cannot bypass residency.
- Evidence 10: Negative tests prove budget override cannot bypass settlement.
- Evidence 11: Negative tests prove budget override cannot bypass audit.
- Evidence 12: Benchmark parity maps admin cost controls for all six vendors.
- Evidence 13: ADR-0321 matrix covers tenant, Cedar, audit, pack, rollback, and benchmark anchors.
- Evidence 14: Runbook drill covers runaway export and replay cost spike.
- Evidence 15: Dashboard checks show soft limit, hard denial, override, and emergency reserve.

## Cost-specific domain and contract deltas
- Domain aggregate: `whiteboard_budget_guard` owns budget policy version, decision, and reconciliation.
- Domain invariant: `whiteboard_budget_guard.budget_period` is immutable for a decision.
- Domain invariant: `budget_override` requires principal, expiry, reason, and max_units.
- Domain invariant: `actual_units` can reconcile estimates but cannot erase denial history.
- Domain invariant: emergency reserve is separate from async and marketplace reserves.
- Domain event `whiteboard.cost.estimated` records predicted units before expensive work.
- Domain event `whiteboard.cost.warning_emitted` records soft-limit notification.
- Domain event `whiteboard.cost.denied` records hard-limit refusal.
- Domain event `whiteboard.cost.override_granted` records authority, expiry, and max units.
- Domain event `whiteboard.cost.reconciled` records actual units and estimate delta.
- OpenAPI delta: expensive commands accept optional `budget_override_id`.
- OpenAPI delta: denial response includes `budget_reason`, `capability_cost_class`, and `remediation_workflow`.
- AsyncAPI delta: emit `whiteboard.cost.decision.v1` for estimate, warning, denial, and override.
- AsyncAPI delta: emit `whiteboard.cost.reconciled.v1` after export, replay, or template materialization completes.
- Proto delta: internal `BudgetDecision` carries soft limit, hard limit, approval threshold, and reserve class.
- Proto delta: internal `CostEvent` carries estimated_units, actual_units, unit_basis, and tenant_hash.
- Cedar fact: `context.budget_override_id` must refer to the same tenant, capability, and period.
- Cedar fact: `resource.capability_cost_class == "commercial"` cannot skip DealSet proof.
- Cedar fact: `resource.capability_cost_class == "emergency"` can use emergency reserve only for IP-013 boards.
- Cedar fact: `context.actual_units` reconciliation cannot authorize retroactive work.
- Workflow decision: soft limit opens notification workflow, not denial.
- Workflow decision: approval threshold opens tenant-admin approval workflow.
- Workflow decision: hard denial blocks non-critical async or commercial action.
- Workflow decision: cost dispute links to immutable cost_event_id and marketplace_install_id when applicable.
- SLO: budget estimate p95 target is 30 ms from warmed policy cache.
- SLO: warning event emission target is within 5 seconds of threshold crossing.
- SLO: hard denial response p95 target is 100 ms after budget service response.
- SLO: cost reconciliation target is 5 minutes after async work completion.
- Replay case: replay cost spike pauses IP-016 cursor before exceeding async reserve.
- Replay case: cost-event replay deduplicates by action id and budget period.
- Replay case: export reconciliation updates actual units without changing approval evidence.
- Rollback: bad budget policy rollout restores prior policy and keeps all cost events.
- Rollback: mistaken override is revoked for future actions and preserved in audit.
- Rollback: failed cost telemetry queue flush pauses high-cost work before evidence loss.
- Test case: budget override cannot be used after expiry.
- Test case: commercial template install with budget override still fails without DealSet.
- Test case: emergency reserve is rejected for ordinary collaboration boards.
- Test case: duplicate cost event reconciles instead of double charging.
- Test case: raw tenant_id is absent from high-cardinality metric dimensions.
- Evidence field: `unit_basis` explains operation, participant-minute, render-megapixel, snapshot-version, replay-object, template-install, or storage-gb-day.
- Evidence field: `reserve_class` explains interactive, async, commercial, emergency, or export budget.
- Evidence field: `reconciliation_delta` explains estimate-to-actual variance.

## Acceptance criteria
- AC-001: Cost dimensions match PRD-whiteboard cost language.
- AC-002: Capability-specific enforcement is documented.
- AC-003: ADR-0321 remains listed and unmodified.
- AC-004: All six benchmark names are present exactly.
- AC-005: Budget override is bounded and auditable.
- AC-006: Settlement, residency, audit, and emergency safety are non-bypassable.
- AC-007: Async work is treated differently from live collaboration.
- AC-008: Failure modes cover budget, telemetry, Cedar, settlement, emergency reserve, and replay.
- AC-009: Repo-local references include cost-budget.md and capacity-model.md.
- AC-010: Budget enforcement evidence records estimated units, actual units, budget period, override decision, tenant notification, and denial reason for expensive export, replay, and template jobs.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
