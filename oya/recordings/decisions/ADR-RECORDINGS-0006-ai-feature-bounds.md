---
id: ADR-RECORDINGS-0006
status: Accepted
date: 2026-05-17
microservice: recordings
deciders: council-privacy, ops-compliance, axis-recordings, axis-foundry-runtime, council-architecture
owner: council-privacy
supersedes: []
superseded_by: []
related: [ADR-0022, ADR-RECORDINGS-0001, ADR-RECORDINGS-0002, ADR-RECORDINGS-0003]
related_artifacts:
  - microservices/recordings/PRD.md (Open Question 6 — EU AI Act high-risk classification)
  - microservices/recordings/capabilities/T0-suggest.yaml
  - microservices/recordings/capabilities/T1-assist.yaml
  - microservices/recordings/capabilities/T2-auto.yaml
  - microservices/recordings/dpia.md (R-05)
purpose: |
  Fix the AI-capability autonomy + risk bounds for transcription, summary,
  translation, and auto-publish capabilities. Aligned with EU AI Act
  Art. 50 (transparency) + Annex III (high-risk classification when used
  in employment / law-enforcement / administration-of-justice context).
  Aligned with messenger ADR-MSGR-T1, meet ADR-MEET-AI, translate ADR-TR-AI.
---

# ADR-RECORDINGS-0006: AI feature bounds — EU AI Act Art. 50 transparency + Annex III high-risk gate

## Status

Accepted — 2026-05-17.

## Context

PRD-recordings capabilities span T0 (chapter-suggest) + T1 (transcription +
diarization + auto-redact-PII + auto-summary) + T2 (auto-translate + auto-
publish). The EU AI Act Art. 50 mandates transparency for AI-generated
content; Annex III §4(a) classifies AI in employment contexts as high-risk;
Annex III §6 + §8 classify law-enforcement + administration-of-justice
as high-risk. Auto-summary or auto-translate output that downstream feeds
employment decisions, court proceedings, or law-enforcement workflows
must be treated as high-risk.

The challenge: recordings µservice cannot a priori know the *downstream
use* of a transcript or summary. The downstream use depends on tenant
context (HR-tooling tenant? law-firm tenant? law-enforcement tenant?).

Options:

- **Refuse all auto-publish in any tenant that might be Annex-III-scoped** —
  over-conservative; blocks legitimate use.
- **Per-tenant high-risk attestation** — tenant signs an attestation when
  enabling T2 auto-publish in Annex-III contexts; provides traceable
  compliance posture.
- **Per-capability classification + per-tenant attestation gate** — sensible
  middle ground.

## Decision

oyatie recordings ships a **per-capability autonomy-tier classification +
per-pack EU-AI-Act risk classification + per-tenant high-risk attestation
gate**:

### 1. Per-capability classification

| Capability | Autonomy tier | EU AI Act default classification | High-risk attestation required when |
|---|---|---|---|
| Chapter-suggest (T0) | T0 | minimal_risk | n/a |
| Transcription (T1) | T1 | limited_risk (transparency only) | downstream-use = Annex III §4/6/8 |
| Diarization (T1) | T1 | limited_risk + Art. 9(1) biometric | downstream-use = Annex III §4/6/8 OR speaker-identification-for-identity-binding |
| PII-auto-redact (T1) | T1 | limited_risk | n/a |
| Auto-summary (T1) | T1 | limited_risk | downstream-use = Annex III §4/6/8 (employment review, legal-proceeding-summary) |
| Auto-translate (T2) | T2 | limited_risk | downstream-use = Annex III §4/6/8 |
| Auto-publish to Workflow (T2) | T2 | limited_risk | downstream-use = Annex III §4/6/8 |

### 2. Tenant high-risk attestation

When a tenant enables T2 auto-publish or T1 capability in Annex III context,
the tenant-admin signs an attestation in the ops portal:

```
I attest that the recordings µservice's <capability> output may be used in
the following Annex III contexts: [employment_review / law_enforcement /
administration_of_justice / none]. I commit to:
  - Posting AI-generated transparency labels per EU AI Act Art. 50.
  - Maintaining a Fundamental-Rights Impact Assessment per Art. 27 when
    high-risk.
  - Refusing automated decision-making with legal effect per GDPR Art. 22.
```

The attestation is stored under `${openbao:secret/recordings/<tenant>/ai-act-attestation}`
with a cryptographic signature. CI lane `ai-feature-bounds-attestation`
verifies attestation is present before T2 features activate.

### 3. EU AI Act Art. 50 transparency labelling

Every transcription / summary / translation / auto-published output carries
an `ai-generated` label:
- Transcript JSON has `model_version` + `ai_generated_label: true`.
- Summary text carries an inline tag and metadata field.
- Auto-translate output carries provenance ("translated by oyatie translate µservice
  from <source_lang>").
- Auto-published events to Workflow include `ai_act_classification` field.

### 4. Annex III high-risk additional obligations

When tenant attests high-risk context:
- FRIA appended to tenant's account per Art. 27.
- Per-capability quality monitoring (WER for transcript; BLEU for translate;
  faithfulness for summary) reported to ops-compliance + DPA on quarterly
  cadence.
- Human-in-the-loop on every auto-publish (T2 in high-risk context is
  effectively T1 — refuses silent auto-publish; requires human review +
  approval per output).
- Per-output explainability evidence emitted to `evidence/`.

### 5. Refusals

- pack-us-healthcare default: T2 auto-publish refused without BAA + tenant
  high-risk attestation.
- pack-us-financial default: T2 auto-publish refused (SEC 17a-4 + FINRA
  4511 — recorded comms cannot be auto-published outside audit-chained
  archive).
- pack-kr / pack-au: producer-side `consent_banner_confirmed` required;
  recordings refuses transcript / translation / summary processing without.

### 6. Cross-µservice alignment

- meet ADR-MEET-AI: meet recordings emit to recordings µservice with the
  same EU AI Act classification baked into the ingest contract.
- messenger ADR-MSGR-T1: huddle recordings same.
- translate ADR-TR-AI: when recordings calls translate, the high-risk
  attestation propagates.

## Alternatives Considered

### A. Refuse all auto-publish globally

- Pros: most-conservative; zero-EU-AI-Act-risk.
- Cons: blocks legitimate use; competitors offer auto-publish (Otter / Fireflies);
  hero-product positioning hurt.
- Rejected.

### B. No tenant attestation gate (transparency-only)

- Pros: low friction.
- Cons: EU AI Act Annex III requires more than transparency for high-risk;
  oyatie cannot defend posture at audit.
- Rejected.

### C. Per-tenant attestation with no per-capability override

- Pros: simpler.
- Cons: tenant cannot opt-in only for low-risk capabilities; over-applies.
- Rejected; per-capability classification is the right granularity.

### D. Cross-µservice attestation propagation handled by translate / mail / social

- Pros: each µservice owns its own attestation.
- Cons: tenant has to attest multiple times; UX bad; risk of drift.
- Rejected; recordings is the source-of-truth attestation, propagates via
  ingest + cross-µservice event metadata.

## Consequences

### Positive

- EU AI Act Art. 50 transparency satisfied at every output.
- Annex III high-risk obligation satisfied via tenant attestation.
- GDPR Art. 22 (no automated-decision-with-legal-effect) honoured.
- Cross-µservice alignment (meet / messenger / translate) gives unified
  oyatie AI posture.

### Negative

- Tenant attestation adds onboarding friction (mitigated by self-service
  ops portal flow).
- Per-output transparency label adds bytes (negligible).
- Quarterly DPA reporting on Annex III tenants adds ops-compliance load.

### Operational

- Cargo workspace adds attestation surface in `oya-recordings-recording-
  kernel` + `oya-recordings-recording-ingest-kernel`.
- CI lane `ai-feature-bounds-attestation` validates attestation presence.
- DPIA section R-05 references this ADR.

### Regulatory

- **EU AI Act Art. 13** — technical documentation per high-risk capability
  via `evidence_topic` (per capabilities/*.yaml).
- **EU AI Act Art. 27** — FRIA when tenant attests high-risk.
- **EU AI Act Art. 50** — transparency labels.
- **EU AI Act Annex III §4(a)** — employment; tenant attestation gate.
- **EU AI Act Annex III §6** — law-enforcement; tenant attestation gate.
- **EU AI Act Annex III §8** — administration-of-justice; tenant attestation
  gate.
- **GDPR Art. 22** — no automated-decision-with-legal-effect by default.
- **GDPR Art. 9(1)** — biometric (speaker-identification) opt-in.

## References

- EU AI Act Regulation (EU) 2024/1689 Arts. 13/27/50/Annex III.
- GDPR Arts. 9(1), 22, 35.
- HIPAA 45 CFR §164.502(b) minimum-necessary.
- SEC Rule 17a-4 + FINRA Rule 4511 + MiFID II Art. 16(7).
- KR PIPA Art. 22-2; KR 통신비밀보호법.
- ADR-0022 (autonomy tiers).
- ADR-RECORDINGS-0001 (transcription pipeline).
- ADR-RECORDINGS-0002 (retention + legal hold).
- ADR-RECORDINGS-0003 (redaction overlay).
- microservices/recordings/capabilities/T0-suggest.yaml.
- microservices/recordings/capabilities/T1-assist.yaml.
- microservices/recordings/capabilities/T2-auto.yaml.
- microservices/recordings/dpia.md R-05.
