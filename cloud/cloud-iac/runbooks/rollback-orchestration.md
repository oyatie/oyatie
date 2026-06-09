---
doc_class: Runbook
title: Rollback orchestration
microservice: cloud-iac
severity: "Sev-1 (production breach) / Sev-2 (operational revert)"
status: Accepted
owner_team: axis-cloud-iac + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/cloud-iac/failure-modes.md (FM-09)
  - microservices/cloud-iac/incident-response.md
  - microservices/observability/runbooks/rollback.md (parent SLO-gate rollback)
doc_status: published
---

# Rollback orchestration (cloud-iac state-revert)

## Trigger

ONE of:

1. **Automated**: observability's SLO gate emits `RollbackExecuted` for `release/<ms>/production`; cloud-iac's rollback-watcher consumes and reverts the µservice's IaC state to the prior apply.
2. **Manual**: on-call IC declares a rollback after a Sev-1/2 incident in cloud-iac itself (e.g., supply-chain attempt FM-08; apply-elevation escape FM-06).
3. **Drift-driven**: drift-remediation runbook Path B determines that a security-suspect mutation requires reverting to known-good state.

## Severity

- Auto-rollback triggered by SLO burn-rate: **Sev-1** (production regression).
- Operational manual rollback (no breach yet): **Sev-2** (planned recovery).
- Security-suspect rollback: **Sev-1** (breach response).

## Pre-checks

1. Identify the rollback target apply: cloud-native IaC controller/API `history --microservice` workflow.
2. Verify the prior apply was eligible at promote-time: cross-reference observability's `oya_promotion_eligibility_verdict{verdict="eligible"}` historical record.
3. Verify the prior apply's content digest is still verifiable via Sigstore / SLSA L3.
4. If manual: capture rollback reason as structured enum (`fast_burn_breach | drift_remediation | supply_chain_response | manual_override | post_mortem_remediation`).

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | Open `#inc-<id>` Slack channel; assign IC; declare severity | ≤ 5min |
| 2 | Confirm pre-checks | ≤ 2min |
| 3 | Invoke rollback: cloud-native IaC controller/API `rollback --microservice` workflow. The CLI: <br> a. validates signature requirements; <br> b. signs the rollback payload with iac-rollback-worker Ed25519 key (or 2-person-rule manual sign if from operator console); <br> c. orchestrates the revert: ArgoCD re-syncs the prior apply's manifest set; <br> d. updates iac-state-index with the rollback row (signed); <br> e. emits `ApplyRolledBack` event consumed by audit-chain + observability + grafana-oncall | ≤ 1min |
| 4 | Verify cluster state reverts: ArgoCD app shows `Synced` at prior apply's content digest within ≤ 2min | ≤ 2min |
| 5 | Verify apply-state index reflects rollback: cloud-native IaC controller/API `status --microservice` workflow returns prior apply id + `rolled_back_at` timestamp | ≤ 1min |
| 6 | Verify SLI returns to green (burn-rate clears within ≤ 15min) | ≤ 15min |
| 7 | CommsLead: status-page update; tenant comms per `incident-response.md` template | ≤ 30min |
| 8 | If manual: file an Issue for the regression — analyse why staging didn't catch it | per priority |
| 9 | Postmortem within 5 business days | – |

## Rollback chain depth > 1 (rollback-of-rollback; FM-09)

Rare but documented. If reverting to the prior apply introduces a new regression:

| Step | Action |
|---|---|
| 1 | Escalate to ExecSponsor; declare Sev-2 if not already |
| 2 | Identify the next-best apply in cloud-native IaC controller/API `history --limit` workflow |
| 3 | Verify that apply was eligible + SLSA-L3-verified at promote-time |
| 4 | Repeat rollback Steps 3–6 against that apply |
| 5 | If chain depth > 2: this µservice has accumulated regression debt; engage µservice owner; consider freezing deploys |
| 6 | Document the chain in audit-chain; per-changeset evidence regenerated |

## Verification

After completion:
- cloud-native IaC controller/API `status --microservice` workflow returns the rollback target apply-id.
- ArgoCD app shows `Synced` at expected content digest.
- `oya_cloud_iac_rollback_executed_total{microservice="<ms>"}` incremented.
- `ApplyRolledBack` event in audit-chain seal log.
- Per-changeset evidence at `microservices/<ms>/evidence/multispectrum/<change_id>-<unix_ts>.json` updated with rollback record.
- Grafana OnCall incident closed.
- Status page reflects "Resolved".

## Post-incident updates

- Postmortem published to `evidence/postmortems/<year>/<incident-id>.md`.
- Action items: typically "why didn't staging catch this?" + "should the SLO threshold tighten?".
- This runbook updated if rollback procedure missed a step.

## References

- `microservices/cloud-iac/failure-modes.md` FM-09.
- `microservices/cloud-iac/incident-response.md` §"Severity-1 response".
- `microservices/observability/runbooks/rollback.md` (parent SLO-gate rollback flow).
- ADR-0139 §"Automated rollback primitive".
- `/specs/agentic-slo-gated-promotion.json` §"rollback_primitive".
