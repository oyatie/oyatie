---
doc_class: Runbook
title: Content moderation classifier rollback / false-positive recovery
microservice: social
severity: "Sev-2 (drift) / Sev-1 (mass false-positive event)"
status: Accepted
owner_team: axis-social + ops-security + council-privacy + axis-foundry-runtime
date: 2026-05-17
related_artifacts:
  - microservices/social/dashboards/moderation-and-safety.json
  - microservices/social/capabilities/T2-auto.yaml
  - microservices/social/threat-model.md (T-T-06)
  - microservices/social/compliance.md (EU AI Act + EU DSA)
  - microservices/social/decisions/ADR-SOC-0003-content-moderation-classifier-bounds.md
doc_status: published
---

# Runbook: Content moderation classifier rollback / false-positive event

## Trigger

Any of:
- `oya_social_moderation_verdict_total{verdict="abuse"|"csam-suspect"|"hate-speech"|"spam"|"harassment"|"self-harm-suspect"}` rate > 5× baseline for ≥ 10 min (suspected drift or attack).
- Tenant-admin escalation: > 100 tenant-reported false-positives in 24h on the same classifier version.
- Council-privacy review identifies a regulatory non-compliance (e.g., EU AI Act Art. 50 transparency obligation unmet, EU DSA Art. 17 Statement-of-Reasons malformed).
- Appeal-resolution rate spike (> 50% of appeals overturning verdicts) on a single classifier version.

## Severity

- Drift: Sev-2.
- Mass false-positive: Sev-1 (free-speech + regulatory implications).

## Immediate Mitigation (≤ 15 min)

| Step | Action | Time |
|---|---|---|
| 1 | Confirm trigger via dashboards/moderation-and-safety.json | ≤ 2 min |
| 2 | Identify classifier version active in pack(s) via T2-auto.yaml + helm release manifest | ≤ 3 min |
| 3 | Roll back classifier version: helm rollback social-moderation-classifier <prior> | ≤ 5 min |
| 4 | Or: pause T2 auto-moderation per-pack via Cedar entitlement revoke | ≤ 5 min |
| 5 | Restore any posts auto-hidden by the rolled-back version (last 24h) | ≤ 15 min |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Classifier drift after retrain | timing matches deploy; macro-F1 regression on reference set | rerun eval reference set; bisect retrain |
| Adversarial prompt-injection attack | clustering of "abuse" verdicts on benign content | sample 100 verdicts; inspect; engage ops-security |
| Locale gap (new language, low coverage) | clustering on `language_code` label | re-train with affected-locale data or disable for that locale |
| EU AI Act labeling gap | tenant complaint; legal-counsel review | restore transparency label; tenant comms |
| EU DSA Statement-of-Reasons format gap | DSA Coordinator notification or tenant report | fix SoR template; resubmit Art. 24 transparency record |
| Sybil-coordinated false-flag campaign | clustering on reporter IPs | engage foundry-guardrails sybil detector |

## False-Positive Recovery Procedure

1. Identify the set of auto-hidden / auto-flagged posts from the affected classifier version + time window.
2. Restore visibility (the rollback already restores future-visible state; this restores past).
3. Notify the author + recipients of each affected post (per pack notification template; see `compliance.md` §"Tenant notification").
4. Emit audit-chain `ModerationVerdictReversed` event per restored item.
5. Emit EU DSA Art. 24 transparency-update record.
6. Council-privacy reviews + signs off; ops-security closes incident.

## Postmortem Triggers

- Within 5 business days; council-privacy + axis-foundry-runtime + ops-security + axis-social.
- If EU AI Act non-compliance: regulator notification within 7 days (Art. 73 serious-incident; market-surveillance authority).
- If EU DSA Art. 24 transparency gap: immediate update to transparency report.
- If KR PIPA Art. 29-2 issue: KISA notification within 24h if user-rights impact.

## Pack-Specific Considerations

| Pack | Note |
|---|---|
| pack-eu | EU AI Act Art. 50 transparency obligation; misclassification with user-rights impact triggers DPA notification + Art. 73 serious-incident path |
| pack-kr | KR PIPA Art. 29-2 + KISA social-platform guidance; misclassification triggers KISA review if affects services to KR users |
| pack-us-healthcare | HIPAA — auto-moderation on PHI posts disabled by default; if classifier accidentally engaged on PHI, triggers HHS OCR review |
| pack-us | Section 230 safe-harbor; tenant-publisher liability; coordinate with tenant counsel |
| pack-uk | UK Online Safety Act 2023 illegal-content duty; Ofcom notification per significance |
| pack-au | AU Online Safety Act 2021 BOSE; eSafety Commissioner |

## Classifier Versioning + Audit

Per `capabilities/T2-auto.yaml`:
- Every classifier version tagged with: training-dataset SHA + commit + reference-set verdict macro-F1 + bias-audit record.
- Per-version evidence_topic record: `oya.social.capability.t2_auto.evidence`.
- Rollback record sealed via audit-chain Ed25519.
- EU AI Act Art. 50 transparency label maintained per version; rollback restores prior label.

## References

- EU AI Act Art. 50 (transparency obligation), Art. 73 (serious-incident reporting).
- EU DSA Arts. 17 (Statement of Reasons), 20 (internal complaint), 24 (transparency report).
- UK Online Safety Act 2023 illegal-content duty.
- KR PIPA Art. 29-2 (automated decision-making rights).
- HIPAA 45 CFR §164.502(b) (minimum necessary).
- NIST AI RMF (Risk Management Framework).
- `microservices/social/threat-model.md` T-T-06.
- `microservices/social/capabilities/T2-auto.yaml`.
- `microservices/social/decisions/ADR-SOC-0003-content-moderation-classifier-bounds.md`.
