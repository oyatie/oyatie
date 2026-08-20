---
doc_class: Runbook
title: Eval-Set Rollback (revert a regressed eval-set version)
microservice: foundry-eval
severity: "Sev-2 (operational regression in eval signal) / Sev-3 (planned revert)"
status: Accepted
owner_team: axis-foundry + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-01 eval-set authoring)
  - microservices/intelligence/incident-response.md
  - microservices/intelligence/policy/two-person-admin-ops.md
  - ADR-0024 §"Eval kernel"
doc_status: published
---

# Runbook: Eval-Set Rollback

## Trigger

ONE of:

1. **Automated**: a newly-promoted eval-set version produces a per-capability pass-rate drop ≥ 10 percentage points within the first nightly cadence; nightly orchestrator opens an incident.
2. **Manual**: a capability owner detects a regressed eval-set (over-fit, miscalibrated rubric, contaminated baseline) and elects to revert.

## Severity

- Automated regressed eval-set detected: **Sev-2** (eval signal compromised; gate decisions unreliable).
- Manual revert (no current regression observed): **Sev-3** (planned recovery).

## Pre-checks

1. Confirm the rollback target version: `kubectl exec -n foundry-eval deploy/eval-set-registry -- oya-intelligence-eval-eval-set-registry-rest list --capability <cap> --order-by version` returns the prior eligible version.
2. Confirm the prior version is itself signed via Cosign + has a valid Rekor inclusion proof.
3. Confirm the prior version's most recent nightly run was passing (pass-rate ≥ threshold).
4. If manual: capture the rollback reason for `EvalSetRolledBack` event.

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | Open `#inc-<id>` Slack channel; assign IC; declare severity | ≤ 5 min |
| 2 | Confirm pre-checks above | ≤ 2 min |
| 3 | Invoke rollback: `cargo run -p oya-dev-cli -- foundry-eval rollback --capability <cap> --to-version <prior-version> --reason "<rfc>"`. The CLI: (a) verifies Cosign + Rekor inclusion; (b) emits 2-person-rule approval flow per `policy/two-person-admin-ops.md` if `--mass-rollback` set; (c) updates eval-set-registry Postgres row to point capability latest at prior-version; (d) re-runs publish-gate against prior-version; (e) emits `EvalSetRolledBack` event to foundry-evidence. | ≤ 3 min |
| 4 | Verify registry advance: `cargo run -p oya-dev-cli -- foundry-eval show --capability <cap>` returns `<prior-version>` | ≤ 1 min |
| 5 | Trigger an ad-hoc eval-run against prior-version to confirm pass-rate restored | ≤ 15 min |
| 6 | Verify nightly cadence picks up new version on next tick | ≤ 24 h (wait for nightly OR force-trigger) |
| 7 | If automated: file Issue for regressed-version root-cause analysis (over-fit? contamination? miscalibrated rubric?) | per priority |
| 8 | Postmortem within 5 business days for Sev-2 cases | — |

## Rollback of the rollback

If reverting to the prior version surfaces a different regression:
1. Identify next-prior signed version via `oya-intelligence-eval-eval-set-registry-rest list`.
2. Repeat steps 3–5 against that version.
3. Escalate to ExecSponsor if repeated reverts indicate accumulated regression debt.

## Verification

After completion:
- Eval-set-registry latest pointer for `<cap>` = `<prior-version>`.
- `oya_foundry_eval_eval_set_pass_rate{capability="<cap>"} >= threshold` within 1 nightly cycle.
- `EvalSetRolledBack` event in audit-chain seal log.
- Per-changeset evidence at `microservices/intelligence/evidence/multispectrum/` updated.

## References

- ADR-0024 §"Eval kernel" + §"Publish-time eval gate".
- `microservices/intelligence/failure-modes.md` FM-01.
- `microservices/intelligence/incident-response.md`.
- `microservices/intelligence/policy/two-person-admin-ops.md`.
