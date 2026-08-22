---
doc_class: Runbook
microservice: feature-flags
runbook_id: RB-FF-001
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0159
  - ADR-0243
  - ADR-0263
  - ADR-0298
companion_docs:
  - microservices/feature-flags/runbooks/flag-mutation-cascade.md
  - microservices/feature-flags/runbooks/experiment-rollback.md
  - microservices/feature-flags/incident-response.md
  - microservices/feature-flags/policy/safety-killswitch-authorization.cedar
planned_enforcement_ref: governance-adr-adherence-matrix
---

# Runbook: Kill-Switch Engaged

## A. Trigger conditions

- Production regression traced to a specific feature flag.
- SLO burn rate >5× normal on a surface controlled by a flag.
- Security vulnerability in a feature that can be disabled via flag.
- Compliance team requests immediate disable of a regulated feature.
- Automated rollout gate failure requiring immediate rollback.

## B. Pre-checks (≤2 minutes)

1. Verify the flag key: `oya flags get <flag_key> --tenant <tenant_id>` — confirm flag exists and is `lifecycle_state: active`. (≤10s)
2. Confirm `flag_type == "kill_switch"` or the flag is explicitly intended as an emergency disable. Do NOT kill-switch a percentage-rollout flag — use `RollbackRolloutStage` instead. (≤10s)
3. Check for existing kill-switch: `oya flags list-kill-switches --tenant <tenant_id>`. Avoid duplicate engagement. (≤5s)
4. Check `policy/safety-killswitch-authorization.cedar` — confirm the flag key is NOT in the life-safety FORBID list (`emergency-services-bypass-flag`, `healthcare-break-glass-enable`, `emergency-dispatch-routing-override`). Engaging a kill-switch on these flags is FORBIDDEN by Cedar policy. (≤5s)
5. Obtain step-up auth token: Class C (TOTP + passkey): `oya auth step-up --class C`. (≤30s)

## C. Procedure

### Step 1 — Engage kill-switch via CLI (≤30s)

```bash
oya flags kill-switch engage <flag_key> \
  --tenant <tenant_id> \
  --reason "<human-readable reason; min 10 chars>" \
  --incident-id <incident_id> \
  --step-up-token $STEP_UP_TOKEN
```

Expected output: `KillSwitchEngaged — propagating to N cells. Audit event ID: <uuid>`.

**If this step fails:** Check Cedar permit — are you in `sre-oncall` or `killswitch-operator` role? Check step-up token validity (≤5min TTL). Escalate to `axis-governance-security` if Cedar policy is blocking erroneously.

### Step 2 — Verify propagation to all cells (≤60s)

```bash
# Wait ≤1s for Kafka broadcast to reach all cells
oya flags propagation-status <flag_key> --tenant <tenant_id>
# Expected: all cells show lifecycle_state: kill_switch_active
```

If any cell shows stale state after 5s: check Kafka consumer lag (`oya kafka consumer-lag feature-flags-killswitch`). Emit `KillSwitchPropagationDelayed` alert. (Timing budget: ≤30s)

### Step 3 — Verify flag evaluation returns default variant (≤30s)

```bash
# Test evaluation from each major cell
oya flags evaluate <flag_key> \
  --tenant <tenant_id> \
  --principal test-probe \
  --cell us-east-cell-1

# Expected reason: KILL_SWITCH; value: <default_variant>
```

Repeat for `eu-west-cell-1` and `kr-cell-1` if tenant has data-residency there.

### Step 4 — Confirm audit event sealed (≤10s)

```bash
oya audit query --event-class KillSwitchEngaged \
  --flag-key <flag_key> \
  --tenant <tenant_id> \
  --since 5m
# Expected: 1 sealed event with audit_chain_id
```

### Step 5 — Notify stakeholders (≤5 minutes)

Post to `#incident-<timestamp>`:
```
KILL-SWITCH ENGAGED: <flag_key>
Tenant: <tenant_id> (or ALL)
Reason: <reason>
Incident: <incident_id>
All cells propagated: YES/NO
Next action: <monitoring / rollout resume after fix>
```

### Step 6 — Monitor flag evaluation metrics (ongoing)

Dashboard: `dashboards/flag-state-overview.json` → confirm `feature_flag_killswitch_active{flag_key="<flag_key>"}` = 1.

Watch: `feature_flag_eval_total{flag_key="<flag_key>",reason="KILL_SWITCH"}` — should see evaluation volume with KILL_SWITCH reason.

## D. Verification

- `feature_flag_killswitch_active{flag_key="<flag_key>"}` gauge = 1 in all cells.
- `KillSwitchEngaged` audit event present and sealed.
- No evaluation with `reason != KILL_SWITCH` for the affected flag.
- User reports of the affected feature showing disabled state.

## E. Rollback (disengaging kill-switch)

When root cause is fixed:

```bash
# Step-up Class B required
oya auth step-up --class B
oya flags kill-switch disengage <flag_key> \
  --tenant <tenant_id> \
  --step-up-token $STEP_UP_TOKEN
```

Verify: `feature_flag_killswitch_active{flag_key="<flag_key>"}` gauge returns to 0.

**IMPORTANT:** Do NOT re-enable immediately into 100% traffic. Use rollout orchestration (`RolloutPlan`) to ramp up gradually after fix.

## F. Post-incident

- Post-mortem: required for all kill-switch activations (SEV-2 minimum).
- Root cause: document in post-mortem why the kill-switch was necessary.
- Prevention: was there a canary rollout in place? If not, why? Add rollout plan before re-enabling.
- Flag lifecycle: after resolution, consider archiving the flag if it was a release_toggle past `sunset_at`.
- Audit: `KillSwitchEngaged` + `KillSwitchDisengaged` events retained 7 years per compliance.md §pack-overlay-roster.

## G. References

- `runbooks/flag-mutation-cascade.md` — if kill-switch triggered cascade across µservices.
- `policy/safety-killswitch-authorization.cedar` — Cedar policy governing kill-switch access.
- `docs/standards/step-up-auth-classes.md` — step-up auth classes reference.
- `ARCHITECTURE.md §cedar-gates` — kill-switch Cedar permit structure.
- ADR-0159 — feature-flag substrate binding ADR.
- ADR-0298 — emergency-services bypass (flags that CANNOT be kill-switched).
