---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: tasks
runbook_id: RB-ai-assign-classifier-rollback
status: Accepted
date: 2026-05-17
owner_team: axis-tasks + council-privacy + foundry-runtime
severity_applicable: [Sev-1]
related_failure_modes: [FM-07]
related_dashboards: [automation-and-ai-quality]
doc_status: published
---

# Runbook — AI Auto-Assign Classifier Rollback

## When this runbook fires

- `tasks_auto_assign_fairness_score < 0.8` per protected class for > 1h, OR
- EU AI Act notified body raises serious-incident finding (Art. 73), OR
- EEOC bias-audit identifies disparate-impact, OR
- Tenant DPO raises GDPR Art. 22 automated-decision incident, OR
- Worker council (where applicable) raises objection to auto-assign decisions.

## Symptoms

- T2 auto-assign decisions in tenant + pack-eu/pack-us employment-context show statistical bias against protected class.
- Per-decision Ed25519 audit chain preserves all decisions for replay/rollback.
- Tenant operators report "auto-assign is picking the same employee repeatedly" or similar bias signals.

## Probable causes

1. Classifier model drift (concept drift from training-time distribution).
2. New training data introduced bias.
3. Feature engineering bug surfaced underrepresented class.
4. Production input distribution shifted (e.g., new tenant onboarded with different demographics).

## Triage (within 15 min, Sev-1)

1. Acknowledge OnCall page.
2. **Page council-privacy + ops-security + axis-tasks immediately**.
3. Identify scope:
   ```promql
   tasks_auto_assign_fairness_score{pack=~"pack-eu|pack-us",context_kind="employment"}
   ```
4. Identify model version:
   ```bash
   oya tasks foundry-bridge model-version --capability t2-auto-assign
   ```
5. Compute affected-decision count:
   ```bash
   oya tasks foundry-bridge audit-trail --capability t2-auto-assign --since "1h" --format json | wc -l
   ```

## Mitigation steps

### Step 1 — IMMEDIATE: Auto-rollback to prior model version (per ADR-TASKS-0006)

```bash
oya tasks foundry-bridge rollback --capability t2-auto-assign --to-version <prior-version-id> --audit-reason "RB-ai-assign-classifier-rollback" --2-person-approver <approver-id>
```

This requires 2-person rule per `threat-model.md` T-E-04 + ADR-TASKS-0006.

### Step 2 — Refuse further T2 auto-assign in affected scope (Cedar policy denial)

```bash
oya tasks policy deny --action task_t2_auto_assign --pack <pack> --context_kind employment --audit-reason "RB-ai-assign-classifier-rollback"
```

Until fairness-audit complete on rollback'd model, T2 is REFUSED at Cedar layer.

### Step 3 — Per-decision audit-chain replay

```bash
oya tasks foundry-bridge audit-replay --capability t2-auto-assign --since "<incident-start>" --output /tmp/affected-decisions.jsonl
```

For each affected decision:
- Identify the human assignee + the alternative assignee the unbiased model would have chosen.
- Surface to tenant DPO for tenant-level remediation.

### Step 4 — Within 24h: Notify affected tenants

```bash
oya tasks notify-tenants --reason "ai-bias-incident" --pack <pack> --capability t2-auto-assign --severity sev-1
```

Tenant DPO receives:
- Affected-decision list (anonymised aggregate).
- Recommended remediation (manual re-assignment of disputed tasks).
- Conformity-assessment status of rollback'd model.

### Step 5 — Within 15 days: Notify EU AI Act notified body + market surveillance (Art. 73)

```bash
oya tasks notify-eu-ai-act --capability t2-auto-assign --pack pack-eu --incident-kind serious --within-days 15
```

### Step 6 — Within state-AG windows: NY Local Law 144 AEDT + state notifications

For pack-us:
- NY: notify NY DCWP within Local Law 144 window.
- CO: notify per CO AI Act HB23-1041.
- CA: notify per AB-331 (when in force).

### Step 7 — Within 72h: GDPR Art. 22 automated-decision incident notification per affected pack-eu tenant

### Step 8 — Conduct bias-audit on rollback'd model + prior model

Engage external bias-audit firm; produce per-protected-class accept-rate analysis:
```bash
oya tasks fairness audit --capability t2-auto-assign --model-version <rollback-version> --output evidence/fairness-audit/<unix_ts>.json
```

### Step 9 — Conformity-assessment re-trigger (if necessary)

If the bias was systematic and not just drift:
- Re-trigger EU AI Act Annex III §4 conformity-assessment per ADR-TASKS-0006.
- Until conformity ADR re-confirms, T2 stays Cedar-refused for affected pack.

## Recovery validation

| Metric | Target | After mitigation |
|---|---|---|
| `tasks_auto_assign_fairness_score` per protected class | ≥ 0.8 | within 24h post-rollback |
| Cedar refusal active for affected pack+context_kind | enforced | within 5 min |
| Per-decision audit-chain seal coverage | 100% of affected period | post-replay |

## Post-incident review

- Was the fairness-monitoring trigger (0.8 threshold) appropriately sensitive?
- Should we add real-time per-decision fairness check (not just weekly audit)?
- Was the rollback timing within Sev-1 expectations (15 min triage + 1h mitigation)?
- Was the Art. 73 notification window achievable (15 days)?
- Update ADR-TASKS-0006 if conformity-assessment scope expands.

## Drills

- Bi-annual: simulated bias incident in synthetic tenant; verify rollback + notification flow.
- Annual: full bias-audit cycle (external firm) per pack-eu / pack-us-employment.

## References

- `failure-modes.md` FM-07.
- ADR-TASKS-0006 (AI auto-assign + EU AI Act Annex III §4 bounds).
- `policy/dual-context-isolation.cedar` (EU AI Act refusal section).
- `slos/auto-assign-fairness-correctness.openslo.yaml`.
- EU AI Act (EU) 2024/1689 — Art. 73 (serious-incident reporting) + Annex III §4.
- GDPR Art. 22 (automated decisions).
- EEOC UGESP 1978 (29 CFR §1607); Title VII; NY Local Law 144 AEDT; CO AI Act HB23-1041.
- `dashboards/automation-and-ai-quality.json`.
