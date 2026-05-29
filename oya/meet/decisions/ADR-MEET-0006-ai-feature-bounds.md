---
id: ADR-MEET-0006
status: Accepted
date: 2026-05-17
microservice: meet
deciders: council-privacy, axis-meet, axis-foundry-runtime, ops-security, council-architecture
owner: council-privacy
supersedes: []
superseded_by: []
related:
  - ADR-0131
  - ADR-0132
  - ADR-MEET-0002
  - ADR-MEET-0003
related_artifacts:
  - microservices/meet/PRD.md (FR-07, FR-16)
  - microservices/meet/IP-009-transcription-pipeline.md
  - microservices/meet/capabilities/T0-suggest.yaml
  - microservices/meet/capabilities/T1-assist.yaml
  - microservices/meet/capabilities/T2-auto.yaml
  - microservices/meet/compliance.md (EU AI Act)
purpose: Classify each meet AI capability under EU AI Act risk tiers; define transparency, evidence, audit, and refresh policies. Aligned with mail ADR-MAIL-0004 + workflow-studio ADR-WS-0005 + sheets ADR-SHEETS-0005.
---

# ADR-MEET-0006: AI feature bounds — transcription low-risk; live-translate medium-risk; summary low-risk; aligned with mail / workflow-studio / sheets risk-class policy

## Status

Accepted — 2026-05-17.

## Context

The meet µservice ships three AI-capability families (per `capabilities/T0-suggest.yaml` + `T1-assist.yaml` + `T2-auto.yaml`):

- **T0 (suggest-only; no autonomous action)** — meeting topic hints, mention-resolution hints in chat-in-meeting (small-model; client-side or server-side).
- **T1 (user-invoked; human-in-loop)** — post-meeting transcript (Whisper-large); post-meeting AI summary + action-item extraction; live captions in user's language (Whisper-medium streaming); live translation overlay.
- **T2 (autonomous classification; bounded side-effects)** — auto-mute on background-noise detection; auto-translate caption overlay for users whose preferred-locale differs from the speaker's.

EU AI Act (Regulation 2024/1689; in force August 2024 with phased application through August 2026) classifies AI systems into four risk tiers:

- **Prohibited** (Art. 5): social scoring, real-time biometric in public spaces (with carve-outs), emotion recognition in workplace/education, etc.
- **High-risk** (Annex III): biometric identification, critical infrastructure, education, employment, essential services, law enforcement, migration, justice/democracy.
- **Limited-risk / Transparency** (Art. 50): AI-generated content + deepfakes + chatbots + emotion-detection in non-prohibited contexts; transparency obligation.
- **Minimal-risk**: all other AI systems; voluntary codes of conduct.

Speech-to-text (transcription) per EDPB Guidelines 4/2024 is generally **limited-risk** (transparency-only) unless deployed in a high-risk Annex III context (e.g., as part of a critical-infrastructure recording system, or law-enforcement context). Live cross-language translation is also limited-risk + transparency. AI summary is limited-risk + transparency.

Other oyatie µservices have already classified their AI features:
- **mail (ADR-MAIL-0004)** — smart-compose limited-risk; smart-reply limited-risk; summary limited-risk.
- **workflow-studio (ADR-WS-0005)** — workflow-suggest limited-risk; workflow-auto-execute T2 with bounded side-effects + EU AI Act Art. 50 banner.
- **sheets (ADR-SHEETS-0005)** — formula-suggest limited-risk; insight-extract limited-risk.

Aligning meet's risk classification with these sibling µservices keeps the regulatory + UX posture consistent.

## Decision

meet µservice classifies its AI capabilities under EU AI Act risk tiers as follows:

1. **Transcription (live captions + post-meeting transcript) — limited-risk (Art. 50 transparency)**
   - Whisper-medium streaming for live captions; Whisper-large batch for post-meeting transcript.
   - Art. 50 transparency: every transcript + caption labelled as **AI-generated**.
   - Per-capability metadata `eu_ai_act_classification: limited_risk` in `capabilities/T1-assist.yaml`.
   - Tenant onboarding notice + per-meeting consent banner discloses AI transcription.

2. **Live translation (caption overlay in user's preferred language) — limited-risk + medium-impact UX**
   - Per-language live translation via foundry-runtime LLM atop the Whisper transcript.
   - Art. 50 transparency: every translated caption labelled as **AI-translated**.
   - Per-capability metadata `eu_ai_act_classification: limited_risk` + UX flag `live_translation_disclaimer: "Translation may not be perfectly accurate; please verify critical decisions"` in `capabilities/T1-assist.yaml`.
   - Rationale for elevated "medium-impact UX" flag: real-time translation drives interpretation decisions in real meetings; mistranslation has higher consequence than batch transcript. Not high-risk under EU AI Act (translation is not an Annex III category) but warrants extra UX clarity.

3. **Post-meeting AI summary + action-item extraction — limited-risk (Art. 50 transparency)**
   - Foundry-runtime LLM atop the Whisper-large transcript.
   - Art. 50 transparency: summary labelled as **AI-generated; review before action**.
   - Per-capability metadata `eu_ai_act_classification: limited_risk` in `capabilities/T1-assist.yaml`.
   - GDPR Art. 22 (automated decision-making) concern: summary is non-binding; user retains decision authority; per-participant opt-out at meeting-join.

4. **T2 auto-mute on background-noise detection — limited-risk**
   - Classifier (small open-weights) detects sustained background noise; auto-mutes that participant's mic with a banner notification.
   - Reversible by participant in one click.
   - Per-capability metadata `eu_ai_act_classification: limited_risk` in `capabilities/T2-auto.yaml`.
   - Bounded side-effect: only affects the participant's own mute state; no broader impact.

5. **T2 auto-translate caption overlay — limited-risk**
   - Inherits classification from live-translation; sub-feature of T1.
   - Per-pack overlay: pack-us-healthcare DISABLES auto-translate by default (third-party translation providers not BAA-covered; only on-prem local model permitted).

6. **T0 meeting-topic hint suggestions — minimal-risk**
   - Small-model client-side or server-side suggestion of meeting topic from calendar event + invitee list.
   - No autonomous action; user picks.
   - Per-capability metadata `eu_ai_act_classification: minimal_risk` in `capabilities/T0-suggest.yaml`.

7. **Per-pack overlays**
   - **pack-eu**: every capability surfaces the Art. 50 transparency banner in the UI; tenant-admin must accept Art. 13 transparency at first enable.
   - **pack-us-healthcare**: T0 minimum-necessary verify per HIPAA §164.502(b); T2 auto-translate DISABLED for PHI channels per Safe Harbour; T1 summary DISABLED on PHI channels by default (opt-in only).
   - **pack-kr**: KR PIPA Art. 28 tenant-admin opt-in + per-user end-user notice for all AI capabilities.
   - **pack-us-financial**: T1 summary captured into SEC 17a-4 supervisory archive alongside recording; supervisor entitlement Cedar-gated.

8. **High-risk uplift triggers**
   - If a future feature crosses into Annex III (e.g., emotion recognition in workplace meetings; biometric identification from face appearance in recordings; participant-screening before meeting): immediate re-classification + conformity assessment under EU AI Act Art. 43 + Annex IV.
   - **Biometric identification from face appearance**: DISABLED by default; opt-in requires GDPR Art. 9(2)(a) consent + EU AI Act high-risk conformity assessment + per-pack regulatory review.

9. **Evidence + audit**
   - Every AI capability output emits to its `evidence_topic` per `capabilities/*.yaml`.
   - Audit-chain seal on every output_hash + capability_version.
   - Per-version eval-set baseline inputs at `capabilities/eval/`.

10. **Refresh cadence**
    - This ADR re-reviewed annually by council-privacy + axis-meet + axis-foundry-runtime.
    - Re-reviewed sooner if EU AI Act guidance changes (EU AI Office publishes annual technical guidance).

## Alternatives Considered

### A. Classify everything as high-risk (cautious posture)
- Pros: maximum compliance margin.
- Cons: triggers EU AI Act Annex III conformity assessment + technical documentation + record-keeping + accuracy/robustness/cybersecurity obligations + post-market monitoring + serious-incident reporting — disproportionate for transcription/summary which are not Annex III categories; competitive disadvantage vs Zoom/Teams/Webex which classify their equivalents as limited-risk.
- Rejected: regulatory misalignment + competitive disadvantage.

### B. Classify everything as minimal-risk (lenient posture)
- Pros: minimum compliance burden.
- Cons: misclassifies live-translation + summary which are AI-generated-content under Art. 50 (limited-risk by definition); under-discloses to users; regulatory risk if EU AI Office investigates.
- Rejected: under-classification.

### C. Tier per capability per Annex III conformity assessment heuristic (this ADR's choice)
- Pros: each capability classified per Art. 5 (prohibited check) + Annex III (high-risk check) + Art. 50 (transparency check) + minimal-risk default; aligns with sibling µservice ADRs; aligns with EDPB Guidelines 4/2024; aligns with industry consensus.
- Accepted.

### D. Disable all AI capabilities until EU AI Office publishes definitive guidance
- Pros: zero AI regulatory risk.
- Cons: feature gap; competitive disadvantage; tenants expect AI in 2026-era meeting platforms.
- Rejected: business-impossible.

### E. Per-tenant AI on/off, no per-capability classification
- Pros: simplest UX.
- Cons: misses per-capability transparency obligation; tenants can't selectively use transcription without summary; granularity needed for HIPAA + SEC overlays.
- Rejected: granularity needed.

## Consequences

### Positive

- Each capability has explicit EU AI Act classification + Art. 50 transparency obligation; auditable.
- Aligns with mail (ADR-MAIL-0004) + workflow-studio (ADR-WS-0005) + sheets (ADR-SHEETS-0005) for cross-µservice regulatory consistency.
- Tenant-admin can enable/disable per-capability per-pack; granularity for HIPAA + KR PIPA + SEC overlays.
- Evidence + audit + eval-set policy makes capability quality continuously monitored.

### Negative

- Per-capability classification + Art. 50 banner is UX overhead; mitigated by capability-output watermark/label (small "AI-generated" badge).
- High-risk uplift triggers require explicit review; council-privacy + axis-foundry-runtime engagement on every new capability.
- Re-classification on EU AI Office guidance updates; planned annual review.

### Operational

- `capabilities/T0-suggest.yaml`, `capabilities/T1-assist.yaml`, `capabilities/T2-auto.yaml` declare per-capability `eu_ai_act_classification`.
- LEAN-lane `oya-check-ai-act-classification-coverage` asserts every capability has a classification.
- Cedar policy `policy/meeting-scope.cedar` includes pack overlays for HIPAA/SEC/KR PIPA conditional disables.
- Dashboards: meet ai-features-quality dashboard surfaces per-capability output rate + eval-set score.
- Runbook `runbooks/transcription-classifier-rollback.md` covers Whisper model version rollback (extends to translation + summary models).

### Regulatory

- **EU AI Act (Regulation 2024/1689) Art. 5** (prohibited): meet capabilities reviewed; none prohibited.
- **EU AI Act Annex III** (high-risk): meet capabilities reviewed; none in Annex III categories (no biometric ID, no critical infra, no education-grading, no employment-screening, no law-enforcement, no justice/democracy).
- **EU AI Act Art. 50** (limited-risk transparency): every AI-generated transcript/caption/summary/translation labelled.
- **EU AI Act Art. 13** (transparency by deployer): tenant-admin attests at first-enable.
- **GDPR Art. 22** (automated decision-making): no capability makes a binding decision; user retains authority.
- **KR PIPA Art. 28**: tenant-admin opt-in required; per-user end-user notice.
- **HIPAA 45 CFR §164.502(b)** (minimum-necessary): T1/T2 disabled on PHI channels by default.
- **SEC Rule 17a-4(f)**: T1 summary in pack-us-financial captured into supervisory archive.

## References

- EU AI Act (Regulation 2024/1689) — `eur-lex.europa.eu/eli/reg/2024/1689`
- EDPB Guidelines 4/2024 (AI Act + GDPR interplay)
- EU AI Office technical guidance (annual)
- OpenAI Whisper paper — Radford et al.
- HELM (Holistic Evaluation of Language Models) — `crfm.stanford.edu/helm/`
- NIST AI Risk Management Framework — `nist.gov/itl/ai-risk-management-framework`
- ROUGE-Lsum (summary evaluation metric)
- BLEU + chrF++ + COMET (translation evaluation metrics)
- ADR-MAIL-0004 (mail AI feature bounds)
- ADR-WS-0005 (workflow-studio AI feature bounds)
- ADR-SHEETS-0005 (sheets AI feature bounds)
- ADR-MEET-0002 (recording + transcription pipeline)
- ADR-MEET-0003 (E2E mode disables AI)
- ADR-0131; ADR-0132
- GDPR Arts. 13, 22; KR PIPA Art. 28; HIPAA 45 CFR §164.502(b); SEC Rule 17a-4(f)
