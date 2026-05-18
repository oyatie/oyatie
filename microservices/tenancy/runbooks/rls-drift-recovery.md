---
doc_class: Runbook
title: RLS drift recovery (catastrophic isolation failure)
microservice: tenancy
severity: "Sev-1 (security breach risk)"
status: Accepted
owner_team: axis-tenancy + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/tenancy/failure-modes.md (FM-02 RLS policy drift; FM-09 pack misroute)
  - microservices/tenancy/policy/rls-isolation.md (Invariant RLS-06 continuous validator)
  - microservices/tenancy/threat-model.md (T-T-01, T-I-01)
  - microservices/tenancy/incident-response.md
doc_status: published
---

# Runbook: RLS drift recovery

## Trigger

`oya-tenancy-rls-state-validator` 5min cron detects drift: declared YAML manifest at `microservices/tenancy/policy/rls/<table>.yaml` does NOT match live Postgres `pg_policies` + `pg_class.relrowsecurity` + `pg_class.relforcerowsecurity` for one or more tenant-bound tables.

Drift signals:
- Missing `tenant_isolation` policy on a tenant-bound table.
- `relrowsecurity = false` on a tenant-bound table.
- `relforcerowsecurity = false` on a tenant-bound table.
- Predicate mismatch (e.g., live policy references `current_setting('different_setting')`).
- Unexpected additional policy on a tenant-bound table (could be benign, e.g., auditor read view; could be malicious).

## Severity

**Sev-1** (catastrophic isolation failure risk) regardless of size; even a single-table drift can expose every tenant's rows for that table simultaneously.

## Impact

Potential cross-tenant data exposure during the drift window. The 5min validator cadence + auto-rollback caps the window at ≤ 5min in steady state; longer if validator itself is degraded.

## Pre-checks

1. Verify drift is real (not false positive): manually compare YAML vs live state:
   ```bash
   cargo run -p oya-tenancy-isolation-policy-app -- rls validate --pack <pack> --table <table>
   ```
2. Verify the validator is healthy: `oya_tenancy_rls_state_validator_alive == 1`. If validator down, drift may have been longer.
3. Check audit-chain for the most recent intentional RLS-policy mutation (could be a recent legitimate PR + Helm-apply mid-rollout).
4. If recent legitimate change: confirm the change is the expected drift (transient during rollout); allow 5min for ArgoCD to converge.

## Recovery Path A — Auto-rollback (ArgoCD)

If drift is unintended (no recent legitimate PR):

| Step | Action | Time |
|---|---|---|
| 1 | Open `#inc-sec-<id>` Slack; declare Sev-1; assign IC + ops-security + axis-tenancy SME + DBA | ≤ 3 min |
| 2 | Verify ArgoCD has already initiated auto-rollback (`argocd app history <tenancy-rls-state>` shows pending rollback to last green) | ≤ 2 min |
| 3 | If ArgoCD auto-rollback succeeds within 5min: confirm live state matches declared YAML; continue investigation | ≤ 5 min |
| 4 | Run a synthetic cross-tenant probe: tenant-A authenticates + queries tenant-B rows; expected: zero rows | ≤ 2 min |
| 5 | If probe shows zero rows: confirm RLS restored; close immediate containment | – |
| 6 | Continue investigation: how was drift introduced? (CI lane evasion? DBA-JIT misuse? Live DB mutation?) | hours-days |

## Recovery Path B — Manual rollback (ArgoCD failed)

If ArgoCD auto-rollback fails (e.g., DBA JIT was used to alter manifest in live DB, bypassing GitOps):

| Step | Action | Time |
|---|---|---|
| 1 | Engage DBA on-call + ops-security; 2-person rule for any manual DB intervention | ≤ 5 min |
| 2 | Determine the intended state from declared YAML at `microservices/tenancy/policy/rls/<table>.yaml`. | ≤ 5 min |
| 3 | DBA JIT-elevated session executes:<br>  - `ALTER TABLE <table> ENABLE ROW LEVEL SECURITY;`<br>  - `ALTER TABLE <table> FORCE ROW LEVEL SECURITY;`<br>  - `CREATE POLICY tenant_isolation ON <table> USING (tenant_id = current_setting('app.current_tenant_id')::text);`<br>  Or any DDL needed to converge to YAML state. | ≤ 10 min |
| 4 | Audit-chain seal the DBA intervention (operator OIDC subject + secondary operator + change set). | – |
| 5 | Run synthetic cross-tenant probe; verify zero rows. | ≤ 2 min |
| 6 | If probe still fails: SCREENSHOT EVERY ACTION + engage ExecSponsor; consider emergency tenant-suspension as containment of last resort. | – |

## Recovery Path C — Validator outage (drift visibility lost)

If the validator itself is down for an extended period:

| Step | Action | Time |
|---|---|---|
| 1 | Engage axis-tenancy on-call; restart validator pods. | ≤ 5 min |
| 2 | Once validator alive, run an immediate full-table validation (not just 5min cadence): expected: all tables match declared YAML. | ≤ 5 min |
| 3 | If validator alive + reports drift: switch to Path A. |
| 4 | If validator alive + reports clean: declare drift-window-end at validator-recovery time; document the visibility gap; postmortem assigns action to harden validator HA. |

## Recovery Path D — Pack misroute (FM-09 cross-pollination)

Cause: in a multi-pack tenancy fleet, a tenant from pack-eu was misrouted to pack-us cluster's tables (data has the wrong jurisdiction). Distinct from RLS drift (which is single-pack), but triggers similar containment.

| Step | Action |
|---|---|
| 1 | Identify misrouted rows via integration-test detector `oya_tenancy_pack_misroute_total`. |
| 2 | Quarantine: suspend the affected tenants temporarily; engage council-privacy for breach assessment. |
| 3 | Correct adapter / OTel collector config; redeploy. |
| 4 | Move misrouted rows back via the schema-export → re-import path (NOT via cross-pack replication — that's exactly what we forbid). |
| 5 | Breach-notification chain: GDPR Art. 33 72h to lead DPA. |
| 6 | Post-incident: harden integration tests + runtime detector. |

## Forensic / Post-Incident Investigation

Once containment confirmed:

| Question | Answer source |
|---|---|
| When did drift begin? | `oya_tenancy_rls_drift_total` time series + audit-chain seal log |
| Who introduced the drift? | DB audit log (pg_event_trigger on `ALTER TABLE`/`CREATE POLICY`/`ALTER POLICY`/`DROP POLICY`) + OpenBao JIT-issuance log + git blame (if YAML-side) |
| Did any cross-tenant query succeed during the window? | Cross-tenant query attempt metric + post-hoc query-log replay against affected tables |
| Was data exfiltrated? | Network egress audit + customer reports + breach-detection alarms |
| Was the drift accidental or adversarial? | Patterns of access + secondary observations |

## Verification

After containment:
- Live `pg_policies` matches declared YAML for every tenant-bound table.
- `relrowsecurity = true` AND `relforcerowsecurity = true` for every tenant-bound table.
- Synthetic cross-tenant probe returns zero rows from every tenant.
- ArgoCD reports rollback successful + last-applied matches declared.
- `oya_tenancy_rls_drift_total == 0` for ≥ 30 min.
- Audit-chain seal log captures the recovery action.

## Post-incident updates

- Postmortem within 5 business days (always; Sev-1).
- Breach-notification chain if cross-tenant exposure confirmed (GDPR 72h; KR PIPA 72h; HIPAA 60d).
- Action items typically include:
  - Tighten CI lane `oya-governance-rls-no-superuser-bypass` if a code-path enabled the drift.
  - Tighten DBA JIT issuance criteria (additional approval / scope-narrowing).
  - Add pg_event_trigger real-time alert (in addition to 5min cron) to shrink detection window.
  - Add forensic-query tooling if data-exfil determination was blocked by gaps.

## References

- `microservices/tenancy/policy/rls-isolation.md` (Invariant RLS-06 continuous validator + Audit Trail).
- `microservices/tenancy/threat-model.md` T-T-01, T-I-01, T-E-01.
- `microservices/tenancy/failure-modes.md` FM-02 + FM-09.
- `microservices/tenancy/incident-response.md`.
- ArgoCD docs — `argo-cd.readthedocs.io`.
- Postgres `pg_event_trigger` — `postgresql.org/docs/16/event-triggers.html`.
- GDPR Art. 33 breach notification.
- KR PIPA Art. 34 breach notification.
- HIPAA §164.404 breach notification.
