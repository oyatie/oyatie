---
doc_class: Runbook
title: Rollback — production-tier release pointer revert
microservice: observability
severity: "Sev-1 (production breach) / Sev-2 (operational revert)"
status: Accepted
owner_team: axis-observability + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/observability/failure-modes.md (FM-10 release pointer)
  - microservices/observability/incident-response.md
  - /specs/agentic-slo-gated-promotion.json §"rollback_primitive"
  - ADR-0130 §"Automated rollback primitive"
doc_status: published
---

# Runbook: Rollback (production-tier release pointer revert)

## Trigger

ONE of:

1. **Automated**: `slo-engine-worker` detects a production-tier fast-burn alert within 1h of a promotion to `release/<microservice>/production`; the worker auto-invokes this runbook's procedure programmatically.
2. **Manual**: on-call IC declares a rollback after a Sev-1/2 incident; this runbook is executed by hand.

## Severity

- Auto-rollback triggered by burn-rate breach: **Sev-1** (production regression).
- Operational manual rollback (no breach yet): **Sev-2** (planned recovery).

## Pre-checks

1. Confirm the rollback target SHA: read `oya_promotion_release_pointer_prior{microservice="<ms>", environment="production"}` from Mimir. The label `sha` carries the prior pointer SHA.
2. Confirm the prior SHA is itself signed + linear + present in repo history (`git log --oneline <prior-sha>` returns non-empty).
3. Confirm the rolling-back SHA's eligibility verdict at staging tier — verify the prior SHA passed staging gating at the time it was promoted (look up its `oya_promotion_eligibility_verdict{verdict="eligible", target_env="production"}` historical record).
4. If manual: capture the rollback reason in a structured form for `RollbackExecuted` event.

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | Open `#inc-<id>` Slack channel; assign IC; declare severity | ≤ 5 min |
| 2 | Confirm pre-checks above | ≤ 2 min |
| 3 | Invoke rollback: `cargo run -p oya-dev-cli -- vcs rollback --microservice <ms> --env production --to-sha <prior-sha> --reason "<rfc>"`. The CLI: <br>  a. validates signature requirements;<br>  b. signs the PATCH with `slo-engine-worker` Ed25519 key (or 2-person-rule manual sign if CLI run on operator console);<br>  c. PATCHes `release/<ms>/production` ref to `<prior-sha>` (signed fast-forward);<br>  d. appends a `rollback` verdict to Mimir (`oya_promotion_eligibility_verdict{verdict="rollback"}`);<br>  e. emits `RollbackExecuted` event consumed by audit-chain. | ≤ 1 min |
| 4 | Verify ref advance: `git ls-remote origin release/<ms>/production` returns `<prior-sha>` | ≤ 1 min |
| 5 | Verify `release/<ms>/production` deployed successfully to production (deployment-controller picks up the new ref + reconciles) | ≤ 5 min for k8s reconcile |
| 6 | Verify SLI returns to green (fast-burn alert clears within ≤ 15 min) | ≤ 15 min |
| 7 | CommsLead: status-page update; tenant comms per `incident-response.md` template | ≤ 30 min |
| 8 | If manual: file an Issue for the regression — analyse why staging didn't catch it | per priority |
| 9 | Postmortem within 5 business days | – |

## Rollback (of the rollback — if rollback itself causes issues)

This is rare but documented for completeness. If reverting to the prior SHA introduces a new regression (e.g., a known-fixed bug recurs):
1. Identify the next-best SHA in `oya_promotion_release_pointer_*` history (i.e., the prior-prior pointer).
2. Repeat Steps 3–6 against that SHA.
3. Escalate to ExecSponsor — this is unusual and indicates the µservice has accumulated regression debt.

## Verification

After completion:
- `release/<ms>/production` ref points to `<prior-sha>` (verified via `git ls-remote`).
- `oya_promotion_eligibility_verdict{microservice="<ms>", target_env="production", verdict="rollback"} == 1` for the affected SHA.
- Mimir burn-rate alerts cleared.
- `RollbackExecuted` event in audit-chain seal log.
- Per-changeset evidence at `microservices/<ms>/evidence/multispectrum/<change_id>-<unix_ts>.json` updated with rollback record.
- Grafana OnCall incident closed.
- Status page reflects "Resolved" with rollback timestamp.

## Post-incident updates

- Postmortem published to `evidence/postmortems/<year>/<incident-id>.md`.
- Action items tracked: typically include "why didn't staging catch this?" and "should the OpenSLO threshold be tightened to detect this signature earlier?".
- Action items closed via PRs that themselves go through the SLO gate.
- This runbook updated if the rollback procedure missed a step.

## References

- ADR-0130 §"Automated rollback primitive".
- `microservices/observability/failure-modes.md` FM-10.
- `microservices/observability/incident-response.md` §"Severity-1 response".
- `/specs/agentic-slo-gated-promotion.json` §"rollback_primitive".
