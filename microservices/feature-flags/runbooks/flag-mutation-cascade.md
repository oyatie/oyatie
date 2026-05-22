---
doc_class: Runbook
microservice: feature-flags
runbook_id: RB-FF-002
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0159
  - ADR-0263
companion_docs:
  - microservices/feature-flags/runbooks/killswitch-engaged.md
  - microservices/feature-flags/runbooks/experiment-rollback.md
  - microservices/feature-flags/incident-response.md
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# Runbook: Flag Mutation Cascade

## A. Trigger conditions

- A flag mutation (value or targeting rule change) causes unexpected behavior across multiple µservices.
- `oya_feature_flag_eval_total{reason="TARGETING_MATCH"}` spikes unexpectedly after a flag update.
- Multiple µservices report errors correlated with a flag state change.
- SLO error budget burn across >2 µservices simultaneously after a flag mutation.
- A pack overlay was applied to a flag that several µservices depend on.

## B. Pre-checks (≤3 minutes)

1. Identify the mutated flag: check `FlagUpdated` audit events in the last 30 minutes.
   ```bash
   oya audit query --event-class FlagUpdated --since 30m --tenant <tenant_id>
   ```
2. Identify affected µservices: search for µservices consuming the flag key.
   ```bash
   oya flags consumers <flag_key> --tenant <tenant_id>
   # Returns: list of µservices + SDK versions using this flag
   ```
3. Check replication status — is the new flag state consistent across all cells?
   ```bash
   oya flags propagation-status <flag_key> --tenant <tenant_id>
   ```
4. Check if undo window is still open (within 15 seconds of mutation).
   ```bash
   oya flags get <flag_key> --tenant <tenant_id>
   # Check: undo_window_open: true/false
   ```

## C. Procedure

### Step 1 — If within 15-second undo window (fastest path, ≤30s)

```bash
oya flags undo <flag_key> --tenant <tenant_id>
# Expected: FlagMutationUndone; flag restored to previous state
```

Verify undo propagated: `oya flags propagation-status <flag_key>` — all cells consistent.

### Step 2 — If undo window expired — revert via new mutation (≤5 minutes)

```bash
# Step-up Class B required
oya auth step-up --class B

# Get previous state from audit log
oya audit query --event-class FlagUpdated \
  --flag-key <flag_key> \
  --tenant <tenant_id> \
  --limit 2
# previous_state contains the value to restore to

# Apply corrective update
oya flags update <flag_key> \
  --tenant <tenant_id> \
  --targeting-rules '<previous_targeting_rules_json>' \
  --step-up-token $STEP_UP_TOKEN
```

### Step 3 — If cascade is severe — engage kill-switch (≤2 minutes)

If the flag cannot be safely reverted via update, engage kill-switch per `runbooks/killswitch-engaged.md`.

Kill-switch forces default variant for all evaluations, stopping the cascade immediately.

### Step 4 — Identify and triage affected µservices (≤15 minutes)

```bash
# Check each affected µservice's error rate
for ms in $(oya flags consumers <flag_key> --tenant <tenant_id>); do
  echo "=== $ms ===" 
  oya metrics query "rate(http_requests_total{job=\"$ms\",status=~\"5..\"}[5m])"
done
```

For each degraded µservice: check if they have a kill-switch for the affected feature (not the flag itself, but the µservice's own circuit-breaker).

### Step 5 — Verify cascade resolution (≤10 minutes)

After flag revert or kill-switch:
- Watch `oya_feature_flag_eval_total` — `reason` distribution should normalize.
- Watch affected µservice SLO metrics — error budget burn rate should drop.
- Confirm `FlagUpdated` (revert) or `KillSwitchEngaged` audit events are present and sealed.

## D. Verification

- Flag state consistent across all cells: `oya flags propagation-status <flag_key>`.
- Affected µservice error rates back to baseline.
- No new `FlagMutationAnomaly` detection signals in the last 5 minutes.

## E. Rollback

If revert itself causes issues (rare):
1. Restore to original mutated state (the one causing cascade was at least known; the pre-mutation state may also be problematic in some contexts).
2. Engage kill-switch (forces default variant; known safe state).
3. Schedule controlled re-introduction via rollout plan (Stage 1: 1% canary).

## F. Post-incident

- Root cause: what targeting rule or value change caused the cascade?
- SDK consumers: were µservices handling `KILL_SWITCH` reason correctly (falling back to safe defaults)?
- Prevention: add canary rollout to the flag mutation; enable SSE streaming in affected SDK consumers for faster propagation detection.

## G. References

- `runbooks/killswitch-engaged.md` — if kill-switch required.
- `runbooks/stale-targeting-rule.md` — if targeting rule was the root cause.
- `ARCHITECTURE.md §observability` — audit event classes for flag mutations.
- ADR-0159 — feature-flag substrate binding ADR.
