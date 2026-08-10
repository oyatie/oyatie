---
doc_class: Runbook
microservice: feature-flags
runbook_id: RB-FF-006
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0159
  - ADR-0294
companion_docs:
  - flags/runbooks/flag-mutation-cascade.md
  - flags/runbooks/killswitch-engaged.md
  - microservices/feature-flags/incident-response.md
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# Runbook: Stale Targeting Rule

## A. Trigger conditions

- `StaleTargetingRuleDetected` metric fires: targeting rule Cedar fragment has not been refreshed in >60s (ADR-0294 soak window expired without refresh).
- Flag evaluation returning incorrect variant for known cohort (targeting rule not matching expected users).
- Cedar fragment version mismatch between cells (one cell has newer fragment than another).
- `sunset_at` on a `release_toggle` or `experiment` flag is past due; CI lane red.

## B. Pre-checks (≤3 minutes)

1. Check which flag has the stale rule:
   ```bash
   oya metrics query "oya_feature_flag_stale_targeting_rule_total" --since 10m
   ```
2. Check Cedar fragment version in each cell:
   ```bash
   oya flags fragment-version <flag_key> --tenant <tenant_id> --all-cells
   ```
3. Check flag `sunset_at`:
   ```bash
   oya flags get <flag_key> --tenant <tenant_id>
   # Check: sunset_at vs current time
   ```
4. Check Cedar soak window status (ADR-0294): new fragments require ≥60s before activation.
   ```bash
   oya flags cedar-soak-status <flag_key> --tenant <tenant_id>
   ```

## C. Procedure

### Case A — Cedar fragment soak delay (normal; not an incident)

New targeting rule was applied; fragment is within 60s soak window. Expected: evaluation uses previous fragment during soak. Action: wait for soak window to expire (≤60s). No SRE action needed.

### Case B — Fragment version mismatch across cells

```bash
# Force fragment refresh on lagging cell
oya flags refresh-fragment <flag_key> \
  --tenant <tenant_id> \
  --cell <lagging_cell_id>

# Verify consistency
oya flags fragment-version <flag_key> --tenant <tenant_id> --all-cells
# All cells should show same fragment_version
```

Timing budget: ≤5 minutes.

### Case C — Stale cohort data (Ontology read stale)

Cohort membership used in targeting rule is stale (> `freshness_floor` of 60s):

```bash
# Force Ontology cache refresh for the tenant
oya ontology refresh-cohort --tenant <tenant_id> --cohort-id <cohort_id>
```

Note: kill-switch flags use `freshness_floor: 0s` (synchronous Ontology read). If a kill-switch flag is showing stale cohort data, this is SEV-1.

### Case D — Flag past `sunset_at`

```bash
# Archive the stale flag (step-up Class B)
oya auth step-up --class B
oya flags archive <flag_key> \
  --tenant <tenant_id> \
  --step-up-token $STEP_UP_TOKEN
```

This emits `FlagArchived` audit event. CI lane `oya-governance-flag-lifecycle` will go green.

### Case E — Targeting rule logic error

If the rule is incorrectly matching or not matching users:

1. Get current rule: `oya flags get <flag_key> --tenant <tenant_id>`.
2. Test rule evaluation: `oya cedar eval --predicate '<cedar_predicate>' --context '<eval_context_json>'`.
3. Fix rule via `FlagUpdate` (step-up Class B).
4. Verify with probe evaluation.

## D. Verification

- `StaleTargetingRuleDetected` metric returns to 0.
- Fragment versions consistent across all cells.
- Test evaluation returns expected variant for known-good evaluation context.

## E. Rollback

If updated targeting rule causes issues: use 15-second undo window or `FlagUpdate` to restore previous rule.

## F. Post-incident

- Was the stale rule causing incorrect access control (e.g., exposing beta feature to non-beta users)? If yes: audit review required.
- `sunset_at` management: add automated alerting 7 days before `sunset_at` to prevent surprise stale flags.

## G. References

- ADR-0294 — Cedar fragment soak window.
- `runbooks/flag-mutation-cascade.md` — if targeting rule change caused cascade.
- `policy/flag-mutation-authorization.cedar` — Cedar policy for targeting rule updates.
