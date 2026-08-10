---
doc_class: Runbook
title: Moderation classifier rollback / false-positive recovery
microservice: messenger
severity: "Sev-2 (drift) / Sev-1 (mass false-positive event)"
status: Accepted
owner_team: axis-messenger + ops-security + council-privacy + axis-foundry-runtime
date: 2026-05-17
related_artifacts:
  - comms/messenger/dashboards/moderation-and-safety.json
  - comms/messenger/capabilities/T2-auto.yaml
  - microservices/messenger/threat-model.md (T-T-06)
  - microservices/messenger/compliance.md (EU AI Act)
doc_status: published
---

# Runbook: moderation classifier rollback / false-positive event

## Trigger

Any of:
- `oya_messenger_moderation_verdict_total{verdict="abuse"|"csam-suspect"|"hate-speech"}`
  rate > 5× baseline for ≥ 10 min (suspected drift or attack).
- Tenant-admin escalation: > 100 tenant-reported false-positives in 24h on
  the same classifier version.
- Council-privacy review identifies a regulatory non-compliance (e.g., EU
  AI Act Art. 50 transparency obligation unmet).

## Severity

- Drift: Sev-2.
- Mass false-positive: Sev-1 (free-speech + regulatory implications).

## Immediate Mitigation (≤ 15 min)

| Step | Action | Time |
|---|---|---|
| 1 | Confirm trigger via dashboards/moderation-and-safety.json | ≤ 2 min |
| 2 | Identify classifier version active in pack(s) via T2-auto.yaml + helm release manifest | ≤ 3 min |
| 3 | Roll back classifier version: helm rollback messenger-moderation-classifier <prior> | ≤ 5 min |
| 4 | Or: pause T2 auto-categorize per-pack via Cedar entitlement revoke | ≤ 5 min |
| 5 | Restore any messages auto-hidden by the rolled-back version (last 24h) | ≤ 15 min |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Classifier drift after retrain | timing matches deploy; macro-F1 regression on reference set | rerun eval reference set; bisect retrain |
| Adversarial prompt-injection attack | clustering of "abuse" verdicts on benign content | sample 100 verdicts; inspect; engage ops-security |
| Locale gap (new language, low coverage) | clustering on `language_code` label | re-train with affected-locale data or disable for that locale |
| EU AI Act labeling gap | tenant complaint; legal-counsel review | restore transparency label; tenant comms |

## False-Positive Recovery Procedure

1. Identify the set of auto-hidden / auto-muted messages from the affected
   classifier version + time window.
2. Restore visibility (the rollback already restores future-visible state;
   this restores past).
3. Notify the sender + recipients of each affected message (per pack
   notification template; see `compliance.md` §"Tenant notification").
4. Emit audit-chain `ModerationVerdictReversed` event per restored item.
5. Council-privacy reviews + signs off; ops-security closes incident.

## Postmortem Triggers

- Within 5 business days; council-privacy + axis-foundry-runtime + ops-security.
- If EU AI Act non-compliance: regulator notification within 7 days
  (NIS2 timeline; DPA timeline if personal data involved).
- If KR PIPA Art. 29-2 issue: KISA notification within 24h if user-rights
  impact.

## Pack-Specific Considerations

| Pack | Note |
|---|---|
| pack-eu | EU AI Act Art. 50 transparency obligation; misclassification with user-rights impact triggers DPA notification |
| pack-kr | KR PIPA Art. 29-2 + KISA messenger guidance; misclassification triggers KISA review if affects services to KR users |
| pack-us-healthcare | HIPAA — auto-moderation on PHI channels disabled by default; if classifier accidentally engaged, triggers HHS OCR review |
| pack-cn (future) | content-moderation regulatory requirements differ; out of scope for current packs |

## Classifier Versioning + Audit

Per `capabilities/T2-auto.yaml`:
- Every classifier version tagged with: training-dataset SHA + commit + reference-set verdict macro-F1.
- Per-version evidence_topic record: `oya.messenger.capability.t2_auto.evidence`.
- Rollback record sealed via audit-chain Ed25519.

## References

- EU AI Act Art. 50 (transparency obligation).
- KR PIPA Art. 29-2 (automated decision-making rights).
- HIPAA 45 CFR §164.502(b) (minimum necessary).
- NIST AI RMF (Risk Management Framework).
- `microservices/messenger/threat-model.md` T-T-06.
- `comms/messenger/capabilities/T2-auto.yaml`.
