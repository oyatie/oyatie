---
doc_class: Runbook
title: Transcription classifier (Whisper) rollback / quality regression recovery
microservice: meet
severity: "Sev-2 (drift) / Sev-1 (mass-mistranscription affecting compliance)"
status: Accepted
owner_team: axis-meet + axis-foundry-runtime + council-privacy
date: 2026-05-17
last_drill_date: 2026-05-17
related_artifacts:
  - microservices/meet/failure-modes.md (FM-04)
  - comms/meet/dashboards/ai-features-quality.json
  - comms/meet/capabilities/T1-assist.yaml
  - microservices/meet/threat-model.md (T-T-02)
  - microservices/meet/compliance.md (EU AI Act)
doc_status: published
---

# Runbook: Whisper transcription rollback / quality regression (meet)

## Trigger

Any of:
- `oya_meet_transcription_quality_score` regression > 5 % vs prior 7-day baseline.
- WER (Word Error Rate) on baseline set > Whisper-large published baseline + 5 %.
- Tenant-admin escalation: > 50 tenant-reported transcript-quality complaints on a single model version in 24h.
- Council-privacy review identifies a regulatory non-compliance (EU AI Act Art. 50 mislabel; per-pack policy violation).
- For pack-us-financial / pack-us-healthcare: any false-transcription event affecting recorded supervised-comms.

## Severity

- Quality drift: Sev-2.
- Mass mistranscription affecting compliance: Sev-1 (FINRA 4511 + HIPAA + EU AI Act implications).

## Immediate Mitigation (≤ 15 min)

| Step | Action | Time |
|---|---|---|
| 1 | Confirm trigger via `dashboards/ai-features-quality.json` panel "transcription quality score" | ≤ 2 min |
| 2 | Identify Whisper model version active in pack(s) via `capabilities/T1-assist.yaml` + helm release manifest | ≤ 3 min |
| 3 | Roll back Whisper model version: `helm rollback meet-transcription <prior>` | ≤ 5 min |
| 4 | Or: pause live captions + transcription via Cedar entitlement revoke (emergency policy push) | ≤ 5 min |
| 5 | Re-process affected transcripts (post-meeting) with rolled-back model; emit corrected transcript-seal event | ≤ 15 min per meeting |
| 6 | Notify affected tenants of transcript regeneration | ≤ 30 min |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Whisper model retraining drift | timing matches model deploy; baseline-set WER regression | rerun eval baseline set; bisect retrain |
| Audio quality degraded upstream (LiveKit codec change) | upstream LiveKit deploy timing | inspect audio bitrate + codec mix |
| GPU degradation (faster-whisper acceleration unhealthy) | per-batch latency increased | inspect GPU node health |
| Locale gap (low coverage on new language) | clustering on `language_code` label | re-train with affected-locale data or disable for that locale |
| EU AI Act labeling gap | tenant complaint; legal-counsel review | restore transparency label; tenant comms |
| Adversarial input (synthetic audio designed to fool ASR) | sparse, anomalous transcripts | sample 100 outputs; inspect; engage ops-security |

## Mistranscription Recovery Procedure

1. Identify the set of transcripts produced by the regressed model version + time window.
2. Mark affected transcripts `regenerated=true` in manifest.
3. Re-run batch transcription with rolled-back Whisper model on the same source audio.
4. Emit `TranscriptionRegenerated` audit-chain event per affected meeting.
5. Update Meilisearch index with corrected transcripts.
6. Notify the meeting hosts + participants of regeneration (per pack notification template).

## Postmortem Triggers

- Within 5 business days; council-privacy + axis-foundry-runtime + ops-security.
- If EU AI Act non-compliance: regulator notification within 7 days.
- If pack-us-financial mistranscription of supervised-comms: FINRA supervisor notification.
- If pack-us-healthcare mistranscription of PHI content: BAA review.

## Pack-Specific Considerations

| Pack | Note |
|---|---|
| pack-eu | EU AI Act Art. 50 transparency obligation; misclassification with user-rights impact triggers DPA notification |
| pack-kr | KR PIPA Art. 29-2 + KISA guidance; misclassification triggers KISA review if affects services to KR users |
| pack-us-healthcare | HIPAA 45 CFR §164.502(b) minimum-necessary; clinical mistranscription could affect patient safety; tenant clinical-review process engaged |
| pack-us-financial | FINRA Rule 3110 supervisory review; mistranscription requires re-archiving |
| pack-eu (MiFID II) | recorded-comms integrity per RTS 6 |

## Whisper Versioning + Audit

Per `capabilities/T1-assist.yaml`:
- Every Whisper model version tagged with: model SHA + training-dataset SHA + commit + baseline-set WER baseline.
- Per-version evidence_topic record: `oya.meet.capability.t1_assist.evidence`.
- Rollback record sealed via audit-chain Ed25519.

## Translation Model Rollback (Sub-Path)

If the issue is with translation overlay (live cross-language captions):
1. Per ADR-MEET-0006, live translation is limited-risk + medium-impact UX flagged.
2. Roll back translation model via same procedure.
3. Re-run translation on affected transcripts.

## References

- EU AI Act Regulation 2024/1689 Art. 50 (transparency obligation).
- KR PIPA Art. 29-2 (automated decision-making rights).
- HIPAA 45 CFR §164.502(b) (minimum necessary).
- FINRA Rule 3110 + 4511 (supervisory review).
- NIST AI RMF.
- OpenAI Whisper paper `arxiv.org/abs/2212.04356`.
- HELM eval framework `crfm.stanford.edu/helm/`.
- ADR-MEET-0002; ADR-MEET-0006.
- `microservices/meet/threat-model.md` T-T-02.
- `comms/meet/capabilities/T1-assist.yaml`.
- `comms/messenger/runbooks/moderation-classifier-rollback.md` (analogous pattern).
