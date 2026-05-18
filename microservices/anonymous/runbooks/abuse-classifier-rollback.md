---
doc_class: Runbook
template_id: TPL-RUNBOOK
title: Abuse-Classifier Rollback
microservice: anonymous
severity: "Sev-2 (planned) / Sev-1 (model misclassification cascade)"
status: Accepted
owner_team: axis-anonymous + axis-foundry-runtime + ops-sre-reliability
date: 2026-05-17
related_adrs: [ADR-ANON-0005]
related_artifacts:
  - microservices/anonymous/PRD.md FR-19, FR-20
  - microservices/anonymous/failure-modes.md FM-09
doc_status: published
---

# Runbook: Abuse-Classifier Rollback

## Trigger

| Signal | Severity |
|---|---|
| Classifier verdict false-positive rate > 5% over 1h sliding window | Sev-1 |
| Classifier verdict false-negative rate > 5% (sampled appeals) | Sev-1 |
| Classifier latency p99 > 500ms over 10m (PRD SLO breach) | Sev-2 |
| Model version deployment caused unexpected verdict distribution shift > 20% | Sev-1 |
| EU AI Act Art. 50 transparency label missing on any verdict | Sev-1 (regulatory) |

## Severity

- Sev-1 if any anonymity-leak risk OR EU AI Act Art. 50 violation OR > 1000 misclassified posts/hour.
- Sev-2 otherwise.

## Pre-checks

1. Confirm the misclassification signal: query Mimir `anonymous_abuse_classifier_verdict_distribution` over the last 1h vs. baseline.
2. Confirm the active model version: `cargo run -p oya-dev-cli -- anonymous content-moderation classifier-version`
3. List the prior known-good model version: `cargo run -p oya-dev-cli -- anonymous content-moderation classifier-version-history --last 5`
4. Confirm the prior version is reachable in the foundry-runtime registry.

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | Open Sev incident channel | ≤ 5 min |
| 2 | Halt the current model: `cargo run -p oya-dev-cli -- anonymous content-moderation halt-classifier --version <current>` | ≤ 1 min |
| 3 | Per-tenant decision: do we revert to prior model or pause classifier entirely? | ≤ 5 min |
| 4a | Revert: `cargo run -p oya-dev-cli -- anonymous content-moderation rollback --to-version <prior>` | ≤ 5 min |
| 4b | Pause: `cargo run -p oya-dev-cli -- anonymous content-moderation pause --reason "<rfc>"`. Moderation queue routes to human review only | ≤ 5 min |
| 5 | Verify new state: `cargo run -p oya-dev-cli -- anonymous content-moderation status` shows the intended classifier version (or paused) | ≤ 1 min |
| 6 | Re-evaluate posts that were classified by the bad model (last 1h to 24h depending on detection lag) by re-classifying with the prior model | ≤ 2h (replay job) |
| 7 | Audit-chain seal each re-classification event (`ClassifierVerdictRevised`) | – |
| 8 | Notify affected users whose verdicts were reversed | within 24h |
| 9 | Post-mortem within 5 business days | – |

## Replay procedure (Step 6)

```bash
cargo run -p oya-dev-cli -- vcs replay \
  --microservice anonymous \
  --consumer content-moderation-classifier \
  --from-seq <seq-of-bad-deploy> \
  --to-seq <current> \
  --target-classifier-version <prior> \
  --invariant-check I1 \
  --rate-limit 200eps
```

Replay re-runs the prior model against historical post bodies, emits revised verdicts, seals each revision to audit-chain, and notifies affected users.

## Failure modes during rollback

| Failure | Mitigation | Severity |
|---|---|---|
| Prior model not available in foundry-runtime registry | escalate to axis-foundry-runtime; pause classifier as backup | Sev-1 |
| Replay rate exceeds foundry-runtime throughput | back off; persist replay queue | Sev-3 |
| Audit-chain seal fails during replay | halt replay; investigate audit-chain | Sev-1 |
| EU AI Act Art. 50 label still missing on revised verdicts | rebuild revised verdict path; do not exit Sev-1 until verified | Sev-1 |

## Cross-µservice coordination

- `foundry-runtime`: model rollback authority sits with foundry-runtime; this runbook is the µservice-side coordination procedure
- `audit-chain`: every classifier change + revised verdict is sealed
- `observability`: rollback emits `oya_anonymous_classifier_rollback_total{version_before, version_after, reason}` metric

## References

- ADR-ANON-0005 — abuse-classifier bounds + EU AI Act Art. 50
- ADR-COMM-0001 — moderation chain-of-responsibility (inherited)
- EU AI Act 2024/1689 Art. 50 (transparency obligation)
- EU DSA Art. 14 (right of appeal)
