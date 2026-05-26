---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P18-IP-007
title: Wave-C operational fitness lanes
status: scaffolded
tier: S
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions: []
source_adrs:
  - ../../../../../../docs/decisions/ADR-0110-changeset-state-machine.md
  - ../../../../../../docs/decisions/ADR-0112-webhook-driven-foundry-agent-invocation.md
  - ../../../../../../docs/decisions/ADR-0113-vcs-orchestrator-end-to-end.md
  - ../../../../../../docs/decisions/ADR-0114-canary-observability-rollback.md
depends_on:
  - M01-P18-IP-006
purpose: Land the operational fitness lanes that detect drift, alarm anomalies, and keep the agentic pipeline observable + correct in steady state.
---

# M01-P18-IP-007 — Wave-C operational fitness lanes

## Scope

Six new fitness lanes spread across the ADRs in this phase:

1. **`oya-governance-changeset-cost-budget-monthly`** — per
   ADR-0110/0113. Asserts cumulative monthly spend per team stays
   under the team-budget cap (sourced from
   `registry/teams/budgets.yaml`). Alarms when burn rate
   indicates the team will exceed before month-end.

2. **`oya-governance-override-justification`** — per ADR-0113.
   Asserts every `oya vcs override` event has a justification
   ≥40 chars AND a valid human signing-key signature. Refuses
   override events that fail either.

3. **`oya-governance-override-frequency-alarming`** — per
   ADR-0113. Alarms when monthly override count per
   `(team, product)` pair exceeds threshold (default 3/month).
   Signal that the agent verdicts are unreliable + needs human
   triage.

4. **`oya-governance-webhook-stuck`** — per ADR-0112. Alarms
   when `agent_invocation_failed` retries exceed `MAX_RETRIES = 3`
   for any delivery_id within 24h.

5. **`oya-governance-webhook-delivery-log-monotonic`** — per
   ADR-0112. Asserts no delivery_id appears twice with
   conflicting dedup outcomes (e.g., `accepted` then
   `deduplicated` is fine; `accepted` then `routing_failed` is
   an anomaly).

6. **`oya-governance-canary-thresholds-tuned`** — per
   ADR-0114. Asserts every product's `registry/canary/thresholds.yaml`
   entry has been reviewed in the last 90 days (`reviewed_at`
   field). Stale config = false-rollback risk = alarm.

7. **`oya-governance-canary-emergency-rewind-frequency`** —
   per ADR-0114. Alarms when emergency-rewind frequency exceeds
   2/30 days; signal that canary controller is unreliable.

## Dependencies

- M01-P18-IP-006 (wave-B integration) — these lanes consume the
  event log + rewind log produced by wave-B.

## Acceptance

- Each lane has a kernel + runner + dev-cli gate-validate arm
  per the existing patterns
  (`crates/oya-check-<lane>/` + `crates/oya-dev-cli/src/<lane>_gate.rs`).
- Each lane runs on the canonical schedule (per-pr CI for hard
  invariants; daily-scheduled workflow for trend alarms).
- Each lane's first run produces a baseline evidence row in
  `/evidence/agentic-vcs-pipeline/ip-007-baseline/<lane-id>.json`
  capturing the day-1 state.
- `registry/teams/budgets.yaml` is initialized (1 row per active
  team, default $50/team/month).
- All lanes wired in `registry/quality/lanes.yaml` as new rows.
- The new `oya-governance-protection-context-match` lane
  (landed in PR #4) is now required-checked alongside the 6
  new lanes once wave-C lanes have 7 days of clean operation.

## Symbols to grit-claim

- `crates/oya-check-changeset-cost-budget-monthly/src/lib.rs::*`
- `crates/oya-check-override-justification/src/lib.rs::*`
- `crates/oya-check-override-frequency-alarming/src/lib.rs::*`
- `crates/oya-check-webhook-stuck/src/lib.rs::*`
- `crates/oya-check-webhook-delivery-log-monotonic/src/lib.rs::*`
- `crates/oya-check-canary-thresholds-tuned/src/lib.rs::*`
- `crates/oya-check-canary-emergency-rewind-frequency/src/lib.rs::*`
- `crates/oya-dev-cli/src/{changeset_cost_budget_monthly_gate,override_justification_gate,override_frequency_alarming_gate,webhook_stuck_gate,webhook_delivery_log_monotonic_gate,canary_thresholds_tuned_gate,canary_emergency_rewind_frequency_gate}.rs::*`
- `registry/teams/budgets.yaml::*`
- `registry/quality/lanes.yaml::*` (6 new lane rows)

## Exit evidence

- `/evidence/agentic-vcs-pipeline/ip-007-baseline/*.json` (7
  baseline records, one per lane)
- `/evidence/agentic-vcs-pipeline/ip-007-7day-clean-operation.json`
  (recorded after 7 days of green operation)
