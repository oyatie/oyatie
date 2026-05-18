---
doc_class: Runbook
title: Moderation classifier rollback
microservice: shorts
severity: "Sev-1 (mass false-positive event; EU AI Act Art. 73 clock)"
status: Accepted
owner_team: axis-shorts + axis-foundry-runtime + council-privacy + ops-legal
date: 2026-05-17
related_artifacts:
  - microservices/shorts/failure-modes.md (FM-09, FM-25)
  - microservices/shorts/capabilities/T2-auto.yaml (EU AI Act high-risk)
  - microservices/shorts/threat-model.md (T-T-06)
  - microservices/shorts/decisions/ADR-SHORTS-0003-content-moderation-classifier-bounds.md
doc_status: published
---

# Runbook: Moderation classifier rollback (FM-09 + FM-25)

## Trigger

- `oya_shorts_moderation_classifier_drift_total` rate > 5x baseline.
- `oya_shorts_moderation_auto_hide_rate` rate > 3x baseline sustained 15min.
- Tenant-reported: mass false-positive auto-hides.
- ops-legal page: EU DSA Art. 73 serious-incident threshold approaching.
- New classifier version golden-set eval regression > 5% on macro-F1.

## Severity

Sev-1 default for classifier-version rollback (EU AI Act Art. 73 clock starts).

## Immediate Mitigation (≤ 30 min)

| Step | Action | Time |
|---|---|---|
| 1 | Confirm classifier version: `kubectl -n shorts get deploy shorts-content-moderation-worker -o yaml \| grep classifier_version` | ≤ 2 min |
| 2 | Inspect verdict drift: `oya_shorts_moderation_verdict_total` by `verdict` last 1h vs 24h | ≤ 5 min |
| 3 | If drift confirmed > 5x: engage foundry-runtime + axis-shorts + council-privacy | ≤ 5 min |
| 4 | Pause classifier verdict emission (queue accumulates; manual review path) | ≤ 5 min |
| 5 | Roll back classifier version via foundry-runtime API: `oya foundry-runtime model promote --version X.Y.Z --rollback` | ≤ 10 min |
| 6 | Verify rolled-back version against golden-set: macro-F1 ≥ 0.95 + bias-audit pass | ≤ 15 min |
| 7 | Resume classifier emission with rolled-back version | ≤ 5 min |
| 8 | Engage ops-legal: EU AI Act Art. 73 15-day clock evaluation | ≤ 30 min |

## Restore False-Positive Auto-Hides

After rollback, restore content auto-hidden by faulty version:

| Step | Action | Time |
|---|---|---|
| 1 | Identify videos auto-hidden by classifier version X.Y.Z in the affected window | ≤ 10 min |
| 2 | Per-video: reverse `auto_hide` action; emit `ModerationVerdictReversed` audit-chain event | continuous |
| 3 | Re-classify with rolled-back version; only auto-hide if new verdict confirms | continuous |
| 4 | Per-affected-creator notification (via tenant-of-tenant): "Your video was incorrectly auto-hidden; restored" | per-creator basis |
| 5 | If creator has filed appeal: resolve appeal as "classifier error; content restored" | per-appeal basis |

Audit-chain seal per reversal: `BackfillModerationClassifierEmitted{video_id, classifier_version, replayer_id, action_taken: reversed}`.

## EU AI Act Art. 73 Notification

If incident qualifies as serious (mass false-positive harming fundamental rights):

1. Engage ops-legal + council-privacy.
2. Draft notification to market-surveillance authority within 15 days per Art. 73.
3. Notification includes: classifier-version chain (X-1 → X.Y.Z → X-1), affected user count, harm assessment, mitigation actions.
4. Per-affected-tenant transparency disclosure per EU DSA Art. 24 (next quarterly report).
5. Per-affected-creator individual notification.

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Newly-promoted classifier version mis-calibrated | drift correlates with version promotion timestamp | rollback to X-1; eval X.Y.Z against golden-set |
| Training-data drift (real-world content shifted) | gradual drift over weeks | re-train with newer corpus; not rollback path |
| Adversarial attack pattern (creators evading) | targeted drift on specific verdict categories | foundry-guardrails review; adversarial-robustness eval |
| Foundry-runtime endpoint outage | classifier endpoint unreachable | failover to manual review queue; fix infra; resume |
| Pack-specific bias detected | drift concentrated in single pack | per-pack overlay review; pack-specific golden-set eval |

## Recovery Verification

- `oya_shorts_moderation_classifier_drift_total` rate returns to baseline (< 1x) for ≥ 24h.
- Golden-set eval on rolled-back version: macro-F1 ≥ 0.95 + bias-audit pass (4/5 disparity ratio).
- All false-positive auto-hides reversed; affected creators notified.
- EU AI Act Art. 73 notification filed (if applicable).
- EU DSA Art. 24 transparency record updated.

## Postmortem (mandatory; ≤ 5 business days)

Required sections:
1. Classifier version chain (X-1 → X.Y.Z → X-1).
2. Drift detection timeline.
3. Affected user count + content count.
4. EU AI Act Art. 73 evaluation outcome.
5. Per-tenant transparency disclosure status.
6. Root cause (training data / hyperparameters / deployment / mis-calibration).
7. Action items: pre-deployment eval bar revisions; canary cohort enlargement; rollback automation.

## References

- `microservices/shorts/failure-modes.md` FM-09, FM-25.
- `microservices/shorts/capabilities/T2-auto.yaml` (EU AI Act high-risk pipeline).
- `microservices/shorts/threat-model.md` T-T-06.
- `microservices/shorts/decisions/ADR-SHORTS-0003`.
- EU AI Act Regulation 2024/1689 Arts. 9, 13, 14, 15, 50, 73.
- EU DSA Regulation 2022/2065 Arts. 17, 20, 24.
- NIST AI RMF.
- HELM benchmark.
