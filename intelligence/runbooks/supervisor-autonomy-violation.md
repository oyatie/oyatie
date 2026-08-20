---
doc_class: Runbook
title: Autonomy violation (Cedar denial flood / tier-escalation refusal)
microservice: foundry-supervisor
severity: "Sev-2 (autonomy-denial flood) / Sev-1 (autonomy fail-open or undetected violation)"
status: Accepted
owner_team: ops-security + axis-foundry-control-plane
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-07, FM-09)
  - microservices/intelligence/policy/tenant-scope.cedar
  - microservices/intelligence/threat-model.md (T-E-01, T-T-05)
doc_status: published
---

# Runbook: Autonomy violation

## Trigger

ONE of:

1. **AutonomyViolated event flood** — `oya_supervisor_autonomy_violation_total` rate spike (FM-07).
2. **Cedar latency spike** — `oya_supervisor_cedar_eval_p99 > 50 ms` (FM-09).
3. **Tier-escalation alarm** — `oya_supervisor_autonomy_level_escalation_attempt_total > 0` (T-E-01).
4. **Fail-open alarm** — autonomy-precondition returned `permit` when it should have denied (rare; pen-test or post-mortem).

## Severity

- Denial flood (false negatives — legitimate invocations refused): Sev-2.
- Cedar latency: Sev-3 unless spread + ≥ 100 ms p99 (Sev-2).
- Tier-escalation alarm: Sev-2.
- Fail-open suspected: **Sev-1**.

## Pre-checks

1. Identify scope: per-tenant vs cluster-wide.
2. Check OpenBao tenant-resolver health: `bao operator status` + recent token-renewal logs.
3. Check recently-merged Cedar fragments: `git log --since="48 hours ago" policy/`.

## Steps — Denial flood (FM-07)

| Step | Action | Time |
|---|---|---|
| 1 | Open `#inc-<id>`; assign IC | ≤ 5 min |
| 2 | Identify offending tenant scope from metric labels | ≤ 5 min |
| 3 | If recent Cedar change: rollback Helm release for `policy/` ConfigMap; verify denials clear | ≤ 15 min |
| 4 | If OpenBao stale entitlements: force-refresh tenant-resolver cache; verify entitlements current | ≤ 10 min |
| 5 | If tenant-specific (legitimate operation refused): apply manual override with 2-person rule for the affected scope: `cargo run -p oya-dev-cli -- supervisor autonomy-override --tenant <id> --capability <id> --tier <T> --reason "<rfc>" --duration 1h --signature-bundle <openbao-jit-token>` | ≤ 1 h |
| 6 | Audit-chain emission for override; tenant notified | ≤ 30 min |
| 7 | Root-cause: was Cedar correct + tenant config wrong? | varies |
| 8 | Postmortem within 5 business days | – |

## Steps — Cedar latency spike (FM-09)

| Step | Action |
|---|---|
| 1 | Inspect recent Cedar fragment changes; rollback if recent |
| 2 | Apply field-length bounds at REST layer if not present |
| 3 | Profile Cedar evaluator: `cargo flamegraph -p oya-intelligence-supervisor-autonomy-policy-enforcement-worker -- --bench` |
| 4 | If pathological input from a specific tenant: rate-limit; engage tenant |
| 5 | Postmortem + Cedar-fragment-coverage lane update |

## Steps — Fail-open suspected (Sev-1)

| Step | Action |
|---|---|
| 1 | Sev-1 declared; engage ops-security director + council-privacy |
| 2 | Engage fleet-wide kill-switch with 2-person rule (refuse all invocations until cause known) |
| 3 | Forensic: trace event chain; verify Ed25519 signatures; check Cedar fragment integrity |
| 4 | Confirm scope of unauthorized invocations: replay event log + cross-check with audit-chain |
| 5 | Breach-notification chain begins (GDPR Art. 33 72h clock starts) if data-subject impact suspected |
| 6 | EU AI Act Art. 73 serious-incident report to EU AI Office if high-risk Annex III tenant affected |
| 7 | Fix + redeploy + disengage kill-switch via 2-person rule |
| 8 | Post-mortem within 24 h; council-architecture + ops-security + council-privacy review |

## Verification

- `oya_supervisor_autonomy_violation_total` rate returns to baseline.
- `oya_supervisor_cedar_eval_p99` ≤ 15 ms.
- `oya_supervisor_autonomy_level_escalation_attempt_total` returns to 0.
- All overrides logged in audit-chain with reason + duration + signatures.

## References

- `failure-modes.md` FM-07, FM-09.
- `threat-model.md` T-E-01, T-T-05.
- `policy/tenant-scope.cedar` PERMIT 4 (autonomy precondition).
- `incident-response.md` §"Sev-1/2 response".
- Cedar v4 — `cedarpolicy.com`.
