---
doc_class: Runbook
title: Deployment rollback (capability rollout revert)
microservice: foundry-supervisor
severity: "Sev-1 (production breach with tenant impact) / Sev-2 (operational revert without breach)"
status: Accepted
owner_team: axis-foundry-control-plane + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-02, FM-12)
  - microservices/intelligence/incident-response.md
doc_status: published
---

# Runbook: Deployment rollback

## Trigger

ONE of:

1. **Automated** — observability `EligibilityChanged{verdict=rollback}` for the capability's release pointer; supervisor auto-rolls-back.
2. **Tenant-initiated** — tenant DPO requests rollback via REST.
3. **Operator** — ops on-call declares rollback during a Sev-1/2 incident.
4. **Stuck canary** (FM-02) — canary phase aged > 15 min without observability advance.

## Severity

- Automated rollback on breach: **Sev-1**.
- Operational manual rollback (no breach): Sev-2.

## Pre-checks

1. Confirm rollback target version: read `Deployment` rows in Postgres for the capability; identify prior `version_sha` that was last successfully at 100 % rollout.
2. Confirm prior version's eligibility was green at the time it was promoted.
3. If manual: capture rollback reason from enum: `fast_burn_breach | slow_burn_breach | manual_override | eval_regression | guardrail_violation | post_mortem_remediation`.

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | Open `#inc-<id>` Slack; assign IC; declare severity | ≤ 5 min |
| 2 | Confirm pre-checks | ≤ 2 min |
| 3 | Invoke rollback: `cargo run -p oya-dev-cli -- supervisor rollback-deployment --capability <id> --to-version <sha> --reason "<enum>"`. CLI: (a) signs the rollback; (b) updates `AgentDeployment` CRD to prior version; (c) Operator reconciles → foundry-runtime workers swap; (d) emits `DeploymentRolledBack` event Ed25519-signed | ≤ 1 min |
| 4 | Verify CRD reflects prior version: `kubectl get agentdeployment <id> -n foundry-tenant-<hashed-id> -o yaml` | ≤ 1 min |
| 5 | Verify foundry-runtime workers reconciled (drain old + spin up prior) | ≤ 5 min |
| 6 | Verify SLI returns to green (burn-rate alert clears within ≤ 15 min) | ≤ 15 min |
| 7 | CommsLead: status-page + tenant email | ≤ 30 min |
| 8 | If automated: file Issue for the regression — analyse why canary didn't catch it | per priority |
| 9 | Postmortem within 5 business days | – |

## Stuck canary (FM-02)

| Step | Action |
|---|---|
| 1 | Inspect `Deployment` row + phase + age |
| 2 | Confirm observability `EligibilityChanged` events present for the capability |
| 3 | Option A — advance: if eligibility green, manually advance `cargo run -p oya-dev-cli -- supervisor advance-phase --capability <id>` |
| 4 | Option B — rollback: invoke standard rollback (above) |
| 5 | If neither: escalate to axis-foundry-control-plane SME |

## Rollback of the rollback (rare)

If reverting introduces a known-fixed regression:
1. Identify next-best version in `Deployment` history.
2. Repeat Steps 3–6 against that version.
3. Escalate to ExecSponsor — indicates regression debt.

## Verification

- `Deployment.current_version == <prior-sha>` in Postgres.
- `AgentDeployment` CRD reflects.
- foundry-runtime workers running prior version (verified via runtime self-metrics).
- `DeploymentRolledBack` event sealed in audit-chain.
- Per-changeset evidence updated.
- OnCall incident closed.

## Post-incident updates

- Postmortem published.
- Action items closed via PRs that go through the SLO gate.
- This runbook updated if procedure missed a step.

## References

- ADR-0139 §"Automated rollback primitive" (precedent).
- `failure-modes.md` FM-02, FM-12.
- `incident-response.md` §"Sev-1 response".
- `/specs/foundry-supervisor-control-plane.json` §"rollback".
