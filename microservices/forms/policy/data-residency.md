---
doc_class: PolicySpec
title: Data Residency Contract
microservice: forms
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-forms
deciders: council-privacy, ops-security, axis-forms, gtm-customer-success
related_adrs: [ADR-0117, ADR-0131, ADR-FORMS-0003]
related_artifacts:
  - microservices/forms/threat-model.md
  - microservices/forms/dpia.md
  - microservices/forms/multi-region.md
review_cadence: annually + on every regional-pack activation OR LLM-provider change OR captcha-provider change
doc_status: published
---

# Data Residency Contract (forms µservice)

## Purpose

Define which jurisdictions' tenant form definitions, response data, file uploads, AI-form-build prompts, signatures, and submitter identifiers live in which cluster; the cross-pack replication policy for Forms assets; the CDN edge residency model; the captcha + LLM provider residency model; and the legal-transfer mechanisms gating any exception.

This document is the canonical residency artifact reviewed by EU DPAs (per GDPR Arts. 44-50), Korean PIPC (per PIPA Art. 28 + Art. 23-2), HIPAA Covered Entity counsel, ANPD (LGPD), DPB (DPDPA 2023), NCA (KSA PDPL), and equivalent supervisory authorities.

## Residency Model

### Default: pack-pinning

Every tenant assigned a primary pack at onboarding. Forms definitions, response data, file uploads, AI-form-build prompts + completions, signatures, and submitter identifiers all live in the pack's region-pinned Forms cluster. Cross-pack movement **forbidden by default**.

| Pack | Primary region(s) | Forms cluster footprint | CDN edges | Activated? |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-forms-{pg,redis,ms,clamav,rest,worker}-1 | OCI CDN KR PoPs | YES (M03 launch tenant) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-forms-{pg,redis,ms,clamav,rest,worker}-{1,2} | OCI CDN EU PoPs | Conditional (first EU tenant SCC) |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 (DR pair) | us-forms-{pg,redis,ms,clamav,rest,worker}-{1,2} | OCI CDN US PoPs | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 + us-phoenix-1 (HIPAA-eligible; isolated from pack-us) | us-hc-forms-{pg,redis,ms,clamav,rest,worker}-{1,2} | OCI CDN HIPAA-eligible PoPs | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | jp-forms-{pg,redis,ms,clamav,rest,worker}-1 | OCI CDN JP PoPs | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-forms-{pg,redis,ms,clamav,rest,worker}-1 | OCI CDN SG PoPs | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-forms-{pg,redis,ms,clamav,rest,worker}-{1,2} | OCI CDN AU PoPs | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-forms-{pg,redis,ms,clamav,rest,worker}-{1,2} | OCI CDN IN PoPs | Conditional (DPDPA) |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-forms-{pg,redis,ms,clamav,rest,worker}-{1,2} | OCI CDN BR PoPs | Conditional (LGPD) |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-forms-{pg,redis,ms,clamav,rest,worker}-{1,2} | OCI CDN ME PoPs | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-forms-{pg,redis,ms,clamav,rest,worker}-{1,2} | OCI CDN KSA PoPs | Conditional (KSA NCA) |

### Pack determines captcha provider

| Pack | Primary captcha | Fallback |
|---|---|---|
| pack-kr | hCaptcha (KR-routed) | Friendly Captcha |
| pack-eu | hCaptcha (EU-routed) | Friendly Captcha |
| pack-us | Cloudflare Turnstile | hCaptcha |
| pack-us-healthcare | hCaptcha (HIPAA BAA-eligible) | Friendly Captcha |
| pack-jp | hCaptcha | Friendly Captcha |
| pack-sg / pack-au | hCaptcha / Turnstile | Friendly Captcha |
| pack-in | hCaptcha | Friendly Captcha |
| pack-br | hCaptcha | Friendly Captcha |
| pack-ae / pack-ksa | hCaptcha | Friendly Captcha |

**Forbidden in pack-eu / pack-kr / pack-us-healthcare**: Google reCAPTCHA (per ADR-FORMS-0002 privacy posture; Schrems II + PIPA Art. 23-2 + HIPAA BAA risk).

### Pack determines LLM-assist routing (AI-form-build)

| Pack | LLM-assist routing target |
|---|---|
| pack-kr | KR-resident LLM provider via foundry-providers (e.g., Solar, KT-LLM, Naver HyperCLOVA) |
| pack-eu | EU-resident LLM provider (e.g., Mistral, Aleph Alpha, Anthropic Claude via AWS Bedrock EU) |
| pack-us | Anthropic Claude / OpenAI GPT-4 (US-resident); BAA-eligible only for pack-us-healthcare |
| pack-us-healthcare | HIPAA-BAA LLM provider (AWS Bedrock with BAA, Azure OpenAI with BAA) |
| pack-jp | JP-resident LLM provider |
| (others) | Pack-resident provider per foundry-providers routing policy |

Tenant may opt-out of AI-form-build entirely (no foundry-providers invocation). Tenant may BYO-LLM (their own provider; foundry-providers routes through tenant's egress).

### Pack determines file-upload bridge target

| Pack | drive µservice cluster |
|---|---|
| pack-kr | kr-drive cluster |
| (mirrors Forms pack list per drive's residency contract) | – |

## Cross-Pack Replication Policy

### Default: forbidden

- Postgres response store: replicate within-pack only.
- Per-tenant DEK: stored in OpenBao at pack-resident endpoint; never replicated cross-pack.
- AI-form-build prompts + completions (90d retention): within-pack only.
- File uploads: within-pack only (drive µservice handles its own residency).
- Forms audit-chain seals: replicate within-pack only.
- Form template signatures: **global** (git-versioned + per-pack signed); content is tenant-agnostic.
- Cedar policies: **global** (git-versioned).
- WASM bundles + design-system primitives: **global** (CDN edges; tenant-agnostic).

### Exception: tenant-executed SCCs

Cross-border transfer of EU-resident response data permitted only with active SCC per GDPR Arts. 44-46. Requires:
1. Active SCC on file at `legal/transfer-register.md`.
2. Receiving-pack jurisdiction has adequate-decision or equivalent.
3. Transfer-purpose limited to named processing (e.g., "DR failover" or "explicit tenant analytics export to non-EU sister-pack").
4. Audit-chain emission at moment of transfer.

### Exception: HIPAA BAA + DR failover

Covered Entity tenants in pack-us-healthcare have DR pair us-ashburn-1 + us-phoenix-1; failover intra-region for HIPAA.

### Exception: BCDR exercise

Controlled cross-region restore drills permitted intra-pack only (eu-frankfurt-1 → eu-amsterdam-1, etc.). Cross-pack BCDR NOT authorised.

## Tenant Tagging by Jurisdiction

Forms entities carry jurisdiction labels for routing + retention enforcement:

```text
metric_label / row_label:
  jurisdiction: kr | eu | us | us-hc | jp | sg | au | in | br | ae | ksa
  pack:         pack-kr | pack-eu | ... (mirrors jurisdiction)
  data_class:   per class taxonomy
```

## Retention by Jurisdiction × Data Class

| Pack | Data class | Minimum statutory | Default applied |
|---|---|---|---|
| pack-kr | `BEHAVIORAL_TENANT_PRODUCT` (analytics) | KR commercial code: 5y; not required for response data | 30d hot; aggregate roll-up retained |
| pack-kr | `SENSITIVE_PIPA_ART23` | PIPA Art. 28: bounded; erasure on request | 1y default; honour erasure |
| pack-kr | `PII_IDENTIFYING` | PIPA Art. 36: erasure ≤ 30d | bounded by tenant DPA; honour erasure |
| pack-kr | `AUDIT` (publish + submit + DSR seals) | PIPA Enforcement Decree Art. 30: ≥ 1y | 3y aligned (KR-FSS sector); 5y for finance vertical |
| pack-eu | `PII_IDENTIFYING` | GDPR Art. 17: bounded; right-to-erasure within 30d | bounded; honour erasure |
| pack-eu | `SENSITIVE_GDPR_ART9` | GDPR Art. 9: explicit consent + minimum retention | bounded by purpose; auto-purge at TTL |
| pack-eu | `AUDIT` | bounded by purpose; in ROPA | 2y default |
| pack-eu | AI-form-build prompts | GDPR Art. 5(1)(e) storage limitation | 90d hot; aggressive purge after |
| pack-us-healthcare | `PHI` (responses with patient identifiers) | HIPAA: state-dependent | MAX(HIPAA 6y, state, tenant DPA) |
| pack-us-healthcare | `AUDIT` | HIPAA §164.316(b)(2): 6y | 6y |
| pack-us-healthcare | E-signature envelopes | HIPAA + state notary | MAX(state notary law, 7y) |
| pack-jp | `PII_IDENTIFYING` | APPI: bounded; deletion request | bounded |
| pack-au | `PII_IDENTIFYING` | Privacy Act APP 11+12: bounded | bounded |
| pack-in | `PII_IDENTIFYING` | DPDPA 2023 §8(1)(g) storage limitation | bounded |
| pack-br | `PII_IDENTIFYING` | LGPD Art. 16 | bounded |
| pack-ae | `PII_IDENTIFYING` | UAE PDPL | bounded |
| pack-ksa | `PII_IDENTIFYING` | KSA PDPL | bounded |
| (all) | `SECRET` | rotate per ISO 27001 A.5.17 | 30d API keys, 90d signing keys |
| (all) | File uploads | n/a (delegated to drive) | per drive contract |
| (all) | AI-form-build prompts | varies | 90d hot for audit; purge after |
| (all) | Webhook delivery logs | bounded by purpose | 30d |

CI lane `oya-governance-retention-conformance` validates Forms retention configs against this table.

## DSR Cascade

Right-to-erasure (GDPR Art. 17 / PIPA Art. 36 / DPDPA §12 / LGPD Art. 18) honoured via `oya-dsr-cascade-runner`:

1. Submitter raises DSR; OR tenant raises on behalf of submitter (joint-controllership per Art. 26).
2. DSR runner identifies subject identifiers in:
   - Response store (Postgres + Citus).
   - Submitter hash table.
   - AI-form-build prompts mentioning subject.
   - Webhook delivery logs.
   - Bulk-distribute recipient logs.
   - File uploads (cascade to drive µservice DSR).
3. Postgres + Valkey + Meilisearch + audit-chain searched; per-row deletion with 30-day soft-delete grace; hard-delete after.
4. Audit-chain seal: `forms_dsr_executed{tenant, subject_hash, removed_response_count, timestamp}`.
5. Submitter / tenant notified within 30d per GDPR; per-pack SLAs (KR 30d, BR 15d, EU 30d) respect strictest applicable.

Limitations (DPIA R-09):
- Data older than retention may be deleted before DSR processed.
- AI-form-build prompts at LLM provider may persist beyond DSR window (mitigated by zero-retention provider selection).
- Derivative aggregates (analytics roll-ups) cascade-purged or k-anonymised.

## Per-Pack Overlay Sections

### pack-kr (PIPA + PIPC)

- PIPA Art. 28 (storage limitation): bounded; sensitive data minimal retention.
- PIPA Art. 23-2 (cross-border sensitive): forbidden by default; AI-form-build routes KR-resident only.
- PIPA Art. 22-2 (RRN handling): RRN field-type explicitly forbidden in builder; alternate-ID field offered.
- PIPC Notice 2020-7 (overseas-transfer notification): pack-kr residency in tenant DPA.
- KR-FSS sector guidance: audit log retention ≥ 5y; KMS keys in KR.
- reCAPTCHA forbidden (Schrems-II-equivalent posture under PIPA Art. 23-2).

### pack-eu (GDPR + EDPB + Schrems II + AI Act)

- GDPR Arts. 44-46 transfer mechanisms: SCC-only; AI-form-build routes EU-resident.
- EDPB Recommendations 01/2020: supplementary measures at `legal/schrems-supplementary-measures.md`.
- GDPR Art. 32 + 25: pseudonymisation + EU-resident KMS + Forms assets cached at EU-resident CDN PoPs.
- EU AI Act 2024 Art. 12 (record-keeping): AI-form-build invocation log retention 6mo minimum when used in high-risk-classified form context (Annex III §4).
- reCAPTCHA forbidden (Schrems II posture).

### pack-us-healthcare (HIPAA)

- 45 CFR §164.530(j): records retention ≥ 6y.
- HIPAA-eligible regions: OCI us-ashburn-1 + us-phoenix-1.
- BAA-required before pack-us-healthcare ingest enabled.
- AI-form-build provider must be HIPAA BAA-eligible (e.g., AWS Bedrock with BAA, Azure OpenAI with BAA).
- reCAPTCHA forbidden (BAA risk).
- ClamAV/OPSWAT scan retained 6y per HIPAA audit.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/forms-data-residency-overlay.md`.

## Verification

- `oya gate validate retention-conformance` — exit 0.
- `oya gate validate pack-routing-conformance` — exit 0.
- `oya gate validate cross-region-transfer-allowed-only-with-scc` — exit 0.
- `oya gate validate forms-llm-provider-pack-resident-routing` — exit 0.
- `oya gate validate forms-captcha-pack-resident-routing` — exit 0.
- `oya gate validate forms-recaptcha-forbidden-pack-eu-kr-us-hc` — exit 0.
- Annual residency audit.
- Quarterly chaos drill: induce cross-pack write attempt; verify rejection + alerting.

## References

- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0131: Per-microservice flat layout.
- ADR-FORMS-0003: Per-tenant DEK + envelope encryption.
- `microservices/forms/threat-model.md`.
- `microservices/forms/dpia.md`.
- `microservices/forms/multi-region.md`.
- `microservices/forms/policy/dual-context.md`.
- `legal/{transfer-register, schrems-supplementary-measures, baa-template, dpa-template, sub-processors, ropa, ai-act-conformity}.md`.
- `regional-packs/<pack>/forms-data-residency-overlay.md`.
- OCI region documentation.
- GDPR Arts. 44-50.
- EDPB Recommendations 01/2020.
- KR PIPA Art. 22-2 + Art. 23-2 + Art. 28 + Art. 36 + PIPC Notice 2020-7.
- HIPAA 45 CFR §164.530(j).
- LGPD Art. 16 + Art. 33.
- DPDPA 2023 §8(1)(g).
- EU AI Act 2024 Art. 12.
