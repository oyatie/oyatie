---
doc_class: DPIA
template_id: TPL-DPIA
microservice: recordings
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-recordings
deciders: council-privacy, ops-compliance, axis-recordings, gtm-customer-success
methodology: GDPR Art. 35 + ICO DPIA guidance + ISO/IEC 29134 + CNIL PIA + KR PIPC Privacy Impact Assessment
related_adrs: [ADR-0008, ADR-0117, ADR-0126, ADR-RECORDINGS-0001, ADR-RECORDINGS-0002, ADR-RECORDINGS-0003, ADR-RECORDINGS-0006]
review_cadence: annually + on every BC addition
doc_status: published
---

# DPIA: recordings µservice

## Purpose

A Data Protection Impact Assessment under GDPR Art. 35 + EU AI Act Art. 27
(FRIA where high-risk-AI-system) + KR PIPA Art. 33-2 (개인정보 영향평가) + APPI
+ HIPAA risk-assessment. The recordings µservice processes voice (special-
category-adjacent under GDPR Art. 9 in many contexts), produces transcripts,
runs speaker diarization (biometric per GDPR Art. 9(1) where used for unique
identification), and emits AI-generated summary + translation outputs — all
of which trigger DPIA + FRIA obligations.

## Processing Description

### Categories of data subject

- **Tenant employees** (professional-context recordings via meet / messenger
  huddles).
- **Tenant customers** (when professional recordings include external
  participants, e.g., sales calls).
- **End-users of personal-context** (B2C — personal recordings).
- **Healthcare patients** (pack-us-healthcare clinical recordings — BAA-
  covered).
- **Financial-firm customers** (pack-us-financial — recorded communications
  under SEC 17a-4(f) + FINRA 4511 + MiFID II 16(7)).
- **Minors** (refused at ingest unless tenant-admin opts in with parental-
  consent attestation).

### Categories of personal data

| Category | GDPR class | Examples |
|---|---|---|
| Voice recording (audio + video) | Art. 4(1) personal data; Art. 9(1) biometric where diarization-for-identification | source media |
| Transcript text | Art. 4(1) personal data; sometimes Art. 9 special-category (health, religion, racial-origin) | transcript JSON |
| Speaker cluster (diarization) | Art. 9(1) biometric (when used to uniquely identify) | speaker_id binding |
| Translated transcript | Art. 4(1) + Art. 9 same as source | translation output |
| Auto-generated summary | derivative of above | summary text |
| Per-viewer watermark | INTERNAL_ONLY (viewer identifier) | watermark key |
| Share-link recipient | Art. 4(1) personal data (when recipient identified by email) | share-link audit |
| Audit-chain event | minimised — principal-ref + action + timestamp | every action |
| Legal-hold engagement | as above + court-order ref | hold event |

### Purposes of processing

| Purpose | Legal basis (GDPR) | Pack-specific basis |
|---|---|---|
| Recording archival (durable storage) | Art. 6(1)(b) contract + Art. 6(1)(f) legitimate-interest | SEC 17a-4 / FINRA 4511 / MiFID II (legal-obligation Art. 6(1)(c) for pack-us-financial); HIPAA TPO (pack-us-healthcare); 전자문서법 (pack-kr) |
| Transcription | Art. 6(1)(a) consent + Art. 6(1)(b) | EU AI Act Art. 50 transparency |
| Speaker diarization | Art. 9(2)(a) explicit consent (biometric for identification) | tenant-admin opt-in + per-user opt-in |
| Translation | Art. 6(1)(a) consent | EU AI Act Art. 50 transparency |
| Redaction (auto + manual) | Art. 6(1)(c) legal-obligation + Art. 6(1)(f) | GDPR Art. 25 (data-protection by design) |
| Legal hold | Art. 6(1)(c) legal-obligation | FRCP Rule 26(f) / 34 (court order); ISO 27037 |
| eDiscovery export | Art. 6(1)(c) legal-obligation | Sedona Conference; ISO 27037 |
| Watermarking | Art. 6(1)(f) legitimate-interest (leak attribution) | balancing-test record per `legal/balancing-test-watermark.md` |
| Search-indexing | Art. 6(1)(b) contract | — |
| Auto-summary | Art. 6(1)(a) consent + Art. 22 (no fully-automated decision-making with legal effect on subject) | EU AI Act — limited-risk where summary; high-risk if used in employment/legal context per Annex III |

## Risks Identified

### R-01 — Voice-biometric scope creep

**Risk**: diarization (biometric per Art. 9(1)) used outside the explicit-
consent scope.

**Mitigation**: diarization emits cluster labels only by default; speaker-
name binding requires explicit user opt-in; per-tenant config to disable
diarization entirely.

**Residual risk**: low.

### R-02 — Transcript leaks special-category data

**Risk**: a transcribed conversation reveals health / religion / racial-
origin / sexual-orientation data (Art. 9).

**Mitigation**: auto-PII redaction at transcription time per
ADR-RECORDINGS-0003; PHI-aware Whisper post-processing for pack-us-
healthcare; tenant-admin can configure auto-redact categories per pack;
right-to-erasure cascades to transcript + overlay + search index.

**Residual risk**: medium (mitigation reduces but cannot eliminate).

### R-03 — Cross-border transfer

**Risk**: recordings replicated to a non-EU region without SCC.

**Mitigation**: per-pack residency pinning per ADR-0117; cross-pack
replication forbidden by default; replication rules audited weekly via
`oya-check-recordings-pack-residency`.

**Residual risk**: low.

### R-04 — Right-to-erasure (Art. 17) conflict with retention floor

**Risk**: GDPR Art. 17 erasure request conflicts with pack-us-financial SEC
17a-4 36-month retention floor or HIPAA §164.530(j) 6-year floor.

**Mitigation**: documented in `policy/data-residency.md`; erasure-with-
retention-floor honours the retention floor for the body and erases the
identifier surface (handle replaced with `«erased»`); audit-chain notes the
redaction event.

**Residual risk**: low — conflict-resolution path is documented.

### R-05 — Automated decision-making

**Risk**: auto-summary used in employment decisions (e.g., performance
review based on recorded meeting summary).

**Mitigation**: EU AI Act Annex III — auto-summary IF used in employment-
related decisions is high-risk; tenant config refuses such use unless
tenant-admin attests + signs the high-risk FRIA addendum per
ADR-RECORDINGS-0006. Default: limited-risk + transparency label per Art. 50.

**Residual risk**: medium — depends on tenant attestation correctness.

### R-06 — Legal-hold integrity

**Risk**: legal-hold engagement fails / lags / is bypassed — court-order
violation + spoliation claim under FRCP 37(e).

**Mitigation**: load-bearing 100 % correctness SLO; pessimistic read-lock
between purge worker and hold engagement; Sev-1 page on any breach.

**Residual risk**: very low — load-bearing SLO + CI lane.

### R-07 — Watermark privacy paradox

**Risk**: per-viewer watermark identifies the viewer; viewer privacy concern.

**Mitigation**: watermark is a tenant-policy-controlled feature; tenant must
sign balancing-test record per `legal/balancing-test-watermark.md`; per-user
notice on share-link receipt.

**Residual risk**: medium — tenant-dependent.

### R-08 — eDiscovery export over-collection

**Risk**: export bundle includes more than the court order specified.

**Mitigation**: hold-scope strict matching per `policy/cedar/ediscovery-scope.cedar`;
four-eyes pair approval; counsel signs receipt.

**Residual risk**: low.

### R-09 — Whisper model leaks training data

**Risk**: Whisper output contains residue from model training.

**Mitigation**: only open-weights Whisper-large via foundry-runtime gVisor;
no cross-tenant fine-tuning; quarterly upstream-model review.

**Residual risk**: low.

### R-10 — Children / minor data

**Risk**: minor's recording ingested without parental consent.

**Mitigation**: ingest refuses minor-flagged recordings unless tenant-admin
opt-in with parental-consent attestation; pack-eu honors GDPR Art. 8.

**Residual risk**: low — refusal path is default.

### R-11 — Cross-µservice translate leakage

**Risk**: transcript sent to `translate` µservice leaks to a different
residency.

**Mitigation**: `translate` µservice has the same pack-pinning constraint;
LEAN-A2 lane forbids cross-pack cross-product calls.

**Residual risk**: low.

## Compliance Mapping

| Framework | Article / Section | Recordings control |
|---|---|---|
| GDPR Art. 5(1) | data-minimisation + storage-limitation | redaction overlay + retention purge per ADR-RECORDINGS-0002 |
| GDPR Art. 9(1) | biometric category | diarization opt-in per R-01 |
| GDPR Art. 13/14 | transparency | recording-consent banner emitted by producing µservice |
| GDPR Art. 17 | right-to-erasure | DSR cascade |
| GDPR Art. 22 | automated decision-making | auto-summary Annex III gating per ADR-RECORDINGS-0006 |
| GDPR Art. 25 | data-protection by design | redaction overlay (no source mutation) per ADR-RECORDINGS-0003 |
| GDPR Art. 30 | record of processing | ROP entry per pack |
| GDPR Art. 32 | security | encryption-at-rest + at-transit; KMS-shred |
| GDPR Art. 35 | DPIA | this document |
| GDPR Arts. 44-50 | transfer | residency per ADR-0117 |
| HIPAA §164.502 | uses + disclosures | BAA + minimum-necessary |
| HIPAA §164.514 | de-identification (Safe Harbor) | redaction overlay matches Safe Harbor 18 identifiers |
| HIPAA §164.530(j) | 6-yr retention | retention floor per pack |
| SEC 17a-4(f) | WORM + 36mo retention | S3 object-lock + retention floor pack-us-financial |
| FINRA 4511 | books + records | export bundle includes timeline |
| MiFID II Art. 16(7) | recording-of-communications 5y | retention default 5y pack-us-financial |
| EU AI Act Art. 13 | technical documentation (high-risk) | per-capability `evidence_topic` |
| EU AI Act Art. 27 | FRIA | this document satisfies FRIA for high-risk uses per ADR-RECORDINGS-0006 |
| EU AI Act Art. 50 | transparency (AI-generated) | every transcription/summary/translate output labelled |
| ePrivacy Art. 5(3) | recording-consent | banner emitted by producing µservice |
| ISO 27037:2012 | digital-evidence handling | export bundle Merkle seal |
| FRCP Rule 26(f)/34 | e-discovery | export bundle workflow |
| KR PIPA Art. 22-2 | DPIA | satisfied |
| KR 통신비밀보호법 | recording-consent | banner-confirmed flag at ingest |
| APPI Art. 17/18 | purpose-limitation | per-pack overlay |
| LGPD Arts. 6/7 | legitimate-interest record | balancing-test |
| UAE PDPL / KSA PDPL | residency + erasure | pack overlay |

## Verification

- `oya gate validate ai-feature-bounds --microservice recordings` per
  ADR-RECORDINGS-0006.
- `oya gate validate retention-policy-correctness --microservice recordings`.
- `oya gate validate legal-hold-chain-of-custody-correctness --microservice recordings`.
- `oya gate validate cross-product-refusal --microservice recordings`.

## DPIA Outcome

Pre-mitigation: **medium-to-high**.

Post-mitigation: **low-to-medium** with two medium residual risks (R-02
transcript special-category + R-05 employment-use + R-07 watermark privacy
paradox) requiring tenant attestation + per-pack overlay enforcement.

DPIA is **approved for production deployment** with the residual-risk
tracking above. Annual review + on every BC addition.

## References

- GDPR Art. 35; EU AI Act Arts. 13/27/50/Annex III; HIPAA 45 CFR §§164.308-
  316/502/514/530; SEC 17a-4(f); FINRA 4511; MiFID II Art. 16(7); FRCP
  Rule 26(f)/34; ISO 27037:2012; KR PIPA Arts. 22-2/28/29; KR 전자문서법;
  KR 통신비밀보호법; APPI; PDPA; DPDPA; LGPD; UAE PDPL; KSA PDPL.
- ADR-RECORDINGS-0001..0007.
- `policy/data-residency.md`.
- `legal/balancing-test-watermark.md`.
- `threat-model.md`.
