---
doc_class: DPIA
title: Data Protection Impact Assessment — translate µservice
microservice: translate
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-security + axis-translate
deciders: council-privacy, ops-security, axis-translate, council-architecture
related_adrs: [ADR-0117, ADR-0126, ADR-0131, ADR-TRANSLATE-0003, ADR-TRANSLATE-0004]
related_artifacts:
  - microservices/translate/PRD.md
  - microservices/translate/threat-model.md
  - microservices/translate/compliance.md
  - microservices/translate/policy/data-residency.md
  - microservices/translate/policy/ai-act-overlay.md
review_cadence: annually + on every new pack activation + on every new vendor adapter + on every QE-model update
doc_status: published
---

# DPIA — translate µservice

## 1. Description of Processing

### 1.1 Nature

The `translate` µservice processes:
- Source text (user-authored content in mail/messenger/social/docs/sheets/slides/meet/shorts/workflow) for translation into a target language.
- Real-time audio-derived caption text (from `meet` Whisper STT pipeline) for caption translation.
- Whole documents (DOCX/PPTX/XLSX/PDF/HTML/Markdown/PO/XLIFF/ARB/.strings/.resx/.properties) for format-preserving round-trip translation.
- Per-tenant Translation Memory units (source + target segment pairs) accumulating over time as a tenant-owned asset.
- Per-tenant termbase entries.
- QE scores (model-predicted edit-distance without a reference).

### 1.2 Scope

Per-pack scope (M01 launch):
- `pack-kr` (PIPC + PIPA Art. 28 storage limitation + Art. 23 sensitive data).
- `pack-eu` (GDPR + EU AI Act).
- `pack-us` (state-by-state PII laws; CCPA where applicable).
- `pack-us-healthcare` (HIPAA-eligible; PHI processing).
- `pack-jp` (APPI Art. 24 cross-border).
- `pack-sg` / `pack-au` / `pack-in` / `pack-br` / `pack-ae` / `pack-ksa` per pack overlays.
- `pack-cn-stub` (PIPL Art. 38–43; scaffolding only — no production activation in M01).

### 1.3 Context

- Tenant base: SaaS B2B; multi-tenant per pack.
- Processing role: oyatie is processor (Art. 28 GDPR); tenant is controller. Each engine vendor (DeepL, Google, Anthropic, OpenAI) is sub-processor.
- Special categories (GDPR Art. 9) may be present when tenants translate medical, legal, employment-related content. ADR-TRANSLATE-0003 §"EU AI Act bounds" restricts QE deployment for these classes.

### 1.4 Purposes

- Provide translation as a paid SaaS feature to enterprise tenants.
- Enable in-product translation for sibling oyatie µservices (mail / messenger / social / docs / sheets / slides / meet / shorts / workflow-studio).
- Power i18n/localization workflows for tenant product/marketing content.
- Compound Translation Memory value over time as a tenant-owned asset.

## 2. Necessity + Proportionality

### 2.1 Lawful basis

| Pack | Lawful basis | Tenant covers it via |
|---|---|---|
| pack-kr | Tenant contract performance (PIPA Art. 17(1)(2)) + Art. 23 explicit consent for sensitive data | tenant DPA + (if sensitive) per-end-user consent |
| pack-eu | GDPR Art. 6(1)(b) contract + (if sensitive) Art. 9(2)(a) explicit consent | tenant DPA + tenant's controller-level Art. 9 basis |
| pack-us-healthcare | HIPAA TPO (Treatment/Payment/Operations) | tenant BAA |
| pack-jp | APPI Art. 17 (user consent or contract) | tenant DPA |
| pack-in | DPDPA 2023 §6 (lawful purpose) | tenant DPA |
| pack-br | LGPD Art. 7 (contract or consent) | tenant DPA |
| (all packs) | Sub-processor contracts (Art. 28 GDPR equivalents) with vendors | per-vendor DPA |

### 2.2 Data minimization (GDPR Art. 5(1)(c))

- Only translatable segments extracted from documents; non-text formatting metadata NOT sent to vendors.
- Source segments hashed (BLAKE3) for audit; original plaintext not retained outside the in-flight call (unless tenant opts into TM persistence).
- QE scores are model probabilities, not personal data.
- TM units retained only with tenant-explicit opt-in per project.

### 2.3 Proportionality

The processing is necessary to deliver the translation feature; less-invasive alternatives:
- All-in-house models — would prevent cross-vendor parity for less common language pairs; partial alternative deployed per ADR-TRANSLATE-0001 §"fallback".
- On-device translation — feasible for Apple Translate / iOS; not feasible for enterprise i18n workflows on documents.

## 3. Risks to Data Subjects

| # | Risk | Likelihood | Severity | Risk score | Mitigation |
|---|---|---|---|---|---|
| R-01 | Source segment containing PII processed by external vendor (DeepL/Google/OpenAI/Anthropic) | High | Medium | Medium-High | DLP scan + ZDR negotiation + per-pack residency + tenant DPA chains to vendor |
| R-02 | Sovereign tenant's content crosses border to non-resident vendor | Medium (without controls) → Very Low (with ADR-TRANSLATE-0004) | Critical | Critical → Low | ADR-TRANSLATE-0004 default-deny on cross-region; per-pack engine whitelist; BLOCKER lane `oya-translate-data-residency-correctness` |
| R-03 | TM accumulates PII over time; right-to-erasure requests must propagate | Medium | High | High | DSR cascade via `oya-dsr-cascade-runner` skill; per-segment soft-delete + 30 d grace + hard-delete |
| R-04 | Real-time caption translation processes audio-derived text containing potentially sensitive speech | High | Medium | Medium | Audio source is consented per meet µservice DPIA; transient processing only; not persisted |
| R-05 | Document round-trip drops markup but exposes whole document to vendor when single segment desired | Medium | Medium | Medium | Segment-level extraction; whole document NOT sent to vendor |
| R-06 | QE score deployed as high-risk AI per EU AI Act inadvertently | Low (default low-risk) | High | Medium | ADR-TRANSLATE-0003 explicit bounds + per-tenant opt-in for high-risk content classes |
| R-07 | Vendor model swap silently changes quality + behaviour | Medium | Medium | Medium | Per-tenant adapter pin + canary cohort weighting + QE monitoring |
| R-08 | Cross-tenant TM leverage (Tenant A's TM returned to Tenant B) | Low | Critical | Medium | Per-tenant RLS + Cedar default-deny + per-tenant Meilisearch index |
| R-09 | Vendor credential leak via logs/error → impersonation | Medium | High | High | OpenBao SecretReference + zeroize-on-drop + `oya-translate-credential-isolation` lane |
| R-10 | Bulk-job upload contains malicious DOCX/PDF/PPTX → exploitation of LibreOffice/Pandoc | Medium | Critical | High | gVisor + seccomp + no-network + read-only-rootfs sandbox per ADR-TRANSLATE-0005 |
| R-11 | Cross-border-misroute (region misconfiguration during deploy) | Low | Critical | Medium | Per-pack residency BLOCKER lane + chaos drill quarterly |
| R-12 | Termbase entry containing PII (a name term) leaked across projects | Low | Medium | Low-Medium | Per-project termbase scope + Cedar policy |
| R-13 | EU AI Act disclosure suppression → regulatory exposure | Low | Medium | Low-Medium | Mandatory disclosure emission at adapter; LEAN-lane verifies |
| R-14 | Real-time stream session hijack (attacker injects mistranslation into live meeting) | Low | High | Medium | Per-session nonce + Ed25519 signed chunks |
| R-15 | Tenant's TM exported to insecure backup channel | Low (per RLS) | High | Medium | Export via signed-URL bulk-export only; audit-chain emits ExportRequested |
| R-16 | Multi-tenant inference engine sees cross-tenant content in same batch | Low | High | Medium | foundry-runtime per-tenant cell isolation (cell µservice posture); single-tenant batch in pack-us-healthcare |
| R-17 | Document containing PHI translated outside HIPAA-eligible region | Low (per AC-04) | Critical | Low | pack-us-healthcare engine whitelist enforced; BAA on file |
| R-18 | Argos/MarianNMT open-source model adapter (future) leaks tenant content through model-introspection attack | Future risk | High | Future | Out-of-scope for M01; tracked under ADR-0026 Phase 4 |

## 4. Mitigations Summary

- ADR-TRANSLATE-0001 — engine routing + fallback (residency-aware).
- ADR-TRANSLATE-0002 — TM leverage model (per-tenant scoped).
- ADR-TRANSLATE-0003 — QE bounds (EU AI Act low-risk default).
- ADR-TRANSLATE-0004 — Data-residency-bound inference (default-deny + per-pack whitelist).
- ADR-TRANSLATE-0005 — Document round-trip fidelity (gVisor sandboxed).
- ADR-TRANSLATE-0006 — Real-time stream (per-session nonce + signed chunks).
- `policy/credential-isolation.md` — credential-handling invariants.
- `policy/data-residency.md` — per-pack residency matrix.
- `policy/ai-act-overlay.md` — EU AI Act Art. 50 + Art. 13 emission rules.
- `runbooks/sovereign-tenant-cross-region-leak-incident-p0.md` — incident protocol for residency breach.
- DSR cascade runner — automated per-DPA SLA.
- `oya-translate-credential-isolation` LEAN BLOCKER lane.
- `oya-translate-data-residency-correctness` LEAN BLOCKER lane.
- `oya-governance-eu-ai-act-disclosure` LEAN lane.

## 5. Residual Risk

After mitigations:
- R-02 (cross-border leak): Very Low (critical mitigation + BLOCKER lane + chaos drill).
- R-06 (EU AI Act high-risk classification): Low (default-low-risk + bounds + per-tenant opt-in).
- R-10 (malicious document): Low (gVisor + CVE refresh).
- Other risks: Low or Low-Medium.

Council-privacy verdict: **Acceptable** for M01 launch (pack-kr, pack-eu pending tenant); annual review.

## 6. Consultation Requirements

- **DPO consultation**: required pre-launch per pack-eu and pack-kr; recorded in `council-privacy/dpo-consult-translate-2026-05-17.md`.
- **DPA notification (GDPR Art. 36)**: not triggered (no high-risk processing identified post-mitigation per Art. 35(3)(b) test).
- **PIPC pre-notification (PIPA Art. 33)**: not triggered (no automated decision-making with legal effect; translation is informational).
- **Stakeholder consultation**: customer-success + ops-security + council-architecture + axis-translate + axis-meet (real-time caption use case) signed off.

## 7. Ongoing Monitoring

- Quarterly DPIA refresh per `review_cadence` above.
- Per-vendor adapter addition triggers DPIA addendum.
- Per-pack activation triggers DPIA per-pack overlay.
- QE-model update triggers ADR-TRANSLATE-0003 re-affirmation.

## 8. References

- ADR-TRANSLATE-0001..0006.
- `microservices/translate/threat-model.md`.
- `microservices/translate/policy/data-residency.md`.
- `microservices/translate/policy/ai-act-overlay.md`.
- `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`.
- Bominal ADR-0028 (data class taxonomy).
- GDPR Arts. 5/6/9/22/25/32/35/36/44–50.
- EU AI Act (Reg. (EU) 2024/1689) Arts. 9–15 + 27 + 50.
- KR PIPA Arts. 17/22-2/23/28/33; PIPC Notice 2020-7.
- HIPAA 45 CFR §164.502/§164.530.
- APPI Art. 24.
- DPDPA 2023 §§6/8/16.
- LGPD Art. 7/16/33.
- ICO Sample DPIA template — `ico.org.uk/for-organisations/uk-gdpr-guidance-and-resources/accountability-and-governance/data-protection-impact-assessments-dpias/`.
- EDPB Guidelines 4/2019 on Art. 25.
