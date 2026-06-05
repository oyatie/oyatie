---
doc_class: PolicySpec
title: Data Residency Contract
microservice: translate
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-translate + ops-security
deciders: council-privacy, ops-security, axis-translate, council-architecture
related_adrs: [ADR-0117, ADR-0135, ADR-0131, ADR-TRANSLATE-0001, ADR-TRANSLATE-0004]
related_artifacts:
  - microservices/translate/dpia.md
  - microservices/translate/threat-model.md
  - microservices/translate/policy/translate-tenant-scope.cedar
  - microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md
  - microservices/translate/multi-region.md
review_cadence: annually + on every new pack activation + on every new vendor adapter
doc_status: published
---

# Data Residency Contract — translate µservice

## Purpose

This is the **canonical** per-pack engine-residency matrix referenced by ADR-TRANSLATE-0004. It tells:

- Tenants which engines may process their content for translation.
- Engineers what `policy.residency.permitted_vendors` is configured to per pack.
- Auditors which regulator citations underpin each row.

Per ADR-TRANSLATE-0004, this matrix is **default-deny**: any (pack × vendor × region) tuple NOT in the table is forbidden.

## Per-Pack Engine Matrix (M01)

### pack-kr (KR PIPA + PIPC + ISMS-P + KR-FSS)

| Engine | Region | Permitted? | Conditions / citations |
|---|---|---|---|
| in-house (foundry-runtime) | OCI ap-seoul-1 | YES | Default; KR-resident |
| Anthropic Claude (API) | KR-region via SCC + ZDR | YES | Tenant DPA + Anthropic ZDR (Zero Data Retention) attestation; PIPA Art. 28 satisfied |
| Google Cloud Translation | KR-region (asia-northeast3) | YES | Per-tenant DPA + Google's PIPA-compliant DPA; Anthropic Art. 28 satisfied |
| DeepL Pro | DE-EU (frankfurt) | CONDITIONAL | Only with explicit tenant PIPA Art. 28 consent on file (per-tenant; recorded in OpenBao) |
| OpenAI | n/a | NO | Not in M01 for pack-kr |
| Microsoft Translator | n/a | NO | Not in M01 |
| Amazon Translate | n/a | NO | Not in M01 |

Citations: PIPA Art. 17 (use within purpose) + Art. 22-2 (sensitive data minimisation) + Art. 23 (sensitive data; explicit consent) + Art. 28 (cross-border transfer; consent + DPA + adequate safeguards) + PIPC Notice 2020-7 + KR-FSS sector audit guidance.

### pack-eu (GDPR + EU AI Act + NIS2 + eIDAS)

| Engine | Region | Permitted? | Conditions / citations |
|---|---|---|---|
| in-house | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | YES | EU-resident; intra-EU DR |
| Anthropic Claude | EU-region | YES | Anthropic SCC + DPA; Schrems-II-supplementary measures via pseudonymisation + EU KMS |
| OpenAI | EU-region | YES | OpenAI SCC + DPA (post-SCC); Schrems-II-supplementary measures |
| Google Cloud Translation | EU-region (europe-west3 / europe-west4) | YES | Google EU DPA |
| DeepL Pro | DE-EU (frankfurt native) | YES | DE-EU native; GDPR-compliant by default |
| Microsoft Translator | EU-region | TRACKED | M02 |
| Amazon Translate | EU-region | TRACKED | M02 |

Citations: GDPR Arts. 5/6/9/25/28/32/35/44–50 + EDPB Recommendations 01/2020 + Schrems I + II + EU AI Act Arts. 9–15 + 50 + NIS2 (NIS Directive 2022/2555) + eIDAS Reg. (EU) 910/2014.

### pack-us (state PII + CCPA + sector laws)

| Engine | Region | Permitted? | Conditions |
|---|---|---|---|
| in-house | OCI us-ashburn-1 | YES | US-resident |
| Anthropic Claude | US-region | YES | Anthropic US DPA |
| OpenAI | US-region | YES | OpenAI US DPA |
| Google Cloud Translation | US-region | YES | Google US DPA |
| DeepL Pro | DE-EU (default) or US-region (when DeepL US available) | YES | Tenant DPA |
| Microsoft Translator | US-region | TRACKED | M02 |
| Amazon Translate | US-region | TRACKED | M02 |

Citations: CCPA (Cal. Civ. Code §1798.100 et seq.) + state-by-state PII laws + sector-specific (CFPB / FTC / etc.) per tenant DPA.

### pack-us-healthcare (HIPAA + HITECH)

| Engine | Region | Permitted? | Conditions / citations |
|---|---|---|---|
| in-house | OCI us-ashburn-1 + us-phoenix-1 (HIPAA-eligible regions) | YES | Internal; BAA n/a |
| Anthropic Claude | HIPAA-eligible US-region + ZDR | YES | Anthropic BAA + Zero Data Retention attestation required |
| OpenAI | per-tenant BAA | CONDITIONAL | Only if tenant executes BAA with OpenAI; off by default |
| Google Cloud Translation | per-tenant BAA + Google Healthcare-eligible region | CONDITIONAL | Per Google Cloud Healthcare API + tenant BAA |
| DeepL Pro | n/a | NO | DeepL does not offer HIPAA BAA at present |
| Microsoft Translator | per-tenant BAA + Azure HIPAA-eligible region | TRACKED | M02 |
| Amazon Translate | per-tenant BAA + AWS HIPAA-eligible region | TRACKED | M02 |

Citations: HIPAA 45 CFR Part 160 + 164; HITECH; §164.502(e) BAA requirements; §164.530(j) records-retention; OCR breach-notification.

### pack-jp (APPI)

| Engine | Region | Permitted? | Conditions |
|---|---|---|---|
| in-house | OCI ap-tokyo-1 | YES | JP-resident |
| Anthropic Claude | JP-region (Anthropic Tokyo) | YES | Anthropic APPI-compliant DPA |
| Google Cloud Translation | JP-region (asia-northeast1) | YES | Google JP DPA |
| DeepL Pro | JP-region (DeepL Tokyo) | YES | DeepL APPI-compliant DPA |
| OpenAI | n/a | NO | Not in M01 |

Citations: APPI Art. 24 (cross-border consent + adequate safeguards) + PPC guidelines.

### pack-sg (PDPA + MAS-TRM)

| Engine | Region | Permitted? | Conditions |
|---|---|---|---|
| in-house | OCI ap-singapore-1 | YES | SG-resident |
| Anthropic Claude | SG-region | YES | DPA + PDPA notification |
| Google Cloud Translation | SG-region | YES | Google APAC DPA |
| DeepL Pro | DE-EU + per-tenant consent | CONDITIONAL | PDPA cross-border notification |
| OpenAI | n/a | NO | M01 |

Citations: PDPA (Personal Data Protection Act 2012, as amended) + PDPC guidelines + MAS-TRM (Technology Risk Management Guidelines).

### pack-au (Privacy Act + APP 8 + APRA-CPS 234)

| Engine | Region | Permitted? | Conditions |
|---|---|---|---|
| in-house | OCI ap-sydney-1 + ap-melbourne-1 (DR pair) | YES | AU-resident |
| Anthropic Claude | AU-region | YES | DPA + APP 8 cross-border accountability |
| Google Cloud Translation | AU-region | YES | Google APAC DPA |
| DeepL Pro | DE-EU + tenant consent | CONDITIONAL | APP 8 disclosure |
| OpenAI | n/a | NO | M01 |

Citations: Privacy Act 1988 (Cth) + APPs (Australian Privacy Principles); APP 8 cross-border disclosure accountability; APRA-CPS 234.

### pack-in (DPDPA 2023)

| Engine | Region | Permitted? | Conditions |
|---|---|---|---|
| in-house | OCI ap-hyderabad-1 + ap-mumbai-1 (DR pair) | YES | IN-resident |
| Anthropic Claude | IN-region (when available) or DPDPA-permitted region | CONDITIONAL | Per DPDPA §16; Data Protection Board notification |
| Google Cloud Translation | IN-region (asia-south1 / asia-south2) | YES | Per DPDPA-compliant DPA |
| DeepL Pro | n/a | NO | M01 |
| OpenAI | n/a | NO | M01 |

Citations: DPDPA 2023 §6 (lawful purpose) + §8 (storage limitation) + §16 (cross-border transfer; notified countries).

### pack-br (LGPD + BACEN)

| Engine | Region | Permitted? | Conditions |
|---|---|---|---|
| in-house | OCI sa-saopaulo-1 + sa-vinhedo-1 (DR pair) | YES | BR-resident |
| Anthropic Claude | per-tenant DPA | CONDITIONAL | LGPD Art. 33 cross-border |
| Google Cloud Translation | BR-region (southamerica-east1) | YES | Google LGPD DPA |
| DeepL Pro | n/a | NO | M01 |
| OpenAI | n/a | NO | M01 |

Citations: LGPD Art. 7 (lawful basis) + Art. 16 (data minimisation) + Art. 33 (cross-border) + ANPD guidelines + BACEN Resolução 4.893/2021 (financial-sector tenants).

### pack-ae (UAE PDPL)

| Engine | Region | Permitted? | Conditions |
|---|---|---|---|
| in-house | OCI me-abudhabi-1 + me-dubai-1 (DR pair) | YES | AE-resident |
| Anthropic Claude | per-tenant DPA + UAE DOA notification | CONDITIONAL | Per UAE PDPL cross-border |
| Google Cloud Translation | UAE region (me-central2) | YES | Google PDPL DPA |
| OpenAI | n/a | NO | M01 |
| DeepL Pro | n/a | NO | M01 |

Citations: UAE PDPL (Federal Decree-Law 45/2021) + UAE DOA (Data Office) guidelines.

### pack-ksa (KSA PDPL + SAMA)

| Engine | Region | Permitted? | Conditions |
|---|---|---|---|
| in-house | OCI me-jeddah-1 + me-riyadh-1 (DR pair) | YES | KSA-resident |
| Anthropic Claude | per-tenant DPA + SDAIA notification | CONDITIONAL | Per KSA PDPL cross-border |
| Google Cloud Translation | KSA region (me-central1) | YES | Google PDPL DPA |
| OpenAI | n/a | NO | M01 |
| DeepL Pro | n/a | NO | M01 |

Citations: KSA PDPL (Royal Decree M/19) + SDAIA Implementing Regulation + SAMA (financial-sector tenants).

### pack-cn-stub (CN Cybersecurity Law + DSL + PIPL)

| Engine | Region | Permitted? | Conditions |
|---|---|---|---|
| in-house | CN-region (scaffolding only M01; no production) | **CONDITIONAL on production-activation prerequisites** | Per ADR-TRANSLATE-0004 §9 |
| Anthropic Claude | n/a | **FORBIDDEN** | PIPL Arts. 38–43 cross-border |
| OpenAI | n/a | **FORBIDDEN** | PIPL Arts. 38–43 cross-border |
| Google Cloud Translation | n/a | **FORBIDDEN** | PIPL Arts. 38–43 cross-border |
| DeepL Pro | n/a | **FORBIDDEN** | PIPL Arts. 38–43 cross-border |

Citations: CN Cybersecurity Law (2017) + Data Security Law (2021) + PIPL Arts. 38–43 (cross-border data transfer; SCC + CAC security assessment + certification + tenant consent); CAC Measures on Outbound Cross-Border Data Transfer.

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=data-residency-correctness --microservice translate` exits 0.
- `tests/integration/e2e/per_pack_engine_whitelist.rs` validates each row.
- Pack overlay's `residency.engineWhitelist` matches this matrix; drift detected by `oya-translate-data-residency-correctness` BLOCKER lane.
- Quarterly chaos drill: simulated cross-region routing attempt rejected at all 5 layers (per ADR-TRANSLATE-0004 defense-in-depth).
- Annual privacy-counsel review.

## DSR (Right-to-Erasure) Cascade

When tenant raises a DSR per GDPR Art. 17 / PIPA Art. 36 / DPDPA §12 / LGPD Art. 18(V):

1. DSR-runner walks `tm_units`, `bulk_jobs`, `documents`, `audit_chain_events` for the subject.
2. Soft-delete with 30 d grace; hard-delete after.
3. Meilisearch index re-syncs.
4. S3 artifacts deleted.
5. Audit-chain emits `DsrExecuted{tenant_id, subject_hash, removed_count, timestamp}`.
6. Tenant notified per pack SLA (KR 30 d, BR 15 d, EU 30 d, etc.; respect strictest).

## References

- ADR-0117 (pack residency model).
- ADR-0135 (connect super-app expansion).
- ADR-TRANSLATE-0004 (residency-bound inference).
- ALL legal citations in matrix above.
- `microservices/translate/threat-model.md` T-05 + FM-70 + FM-71 + FM-72 + FM-73.
- `microservices/translate/dpia.md` R-02 + R-11 + R-17.
- `microservices/translate/multi-region.md`.
- `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`.
- Per-vendor DPA + BAA + ZDR attestations on file at `microservices/translate/legal/` (Slice D; not in this M01 scaffold).
