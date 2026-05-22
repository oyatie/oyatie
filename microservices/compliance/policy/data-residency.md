---
doc_class: PolicySpec
title: Data Residency Contract
microservice: compliance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: council-compliance + axis-compliance
deciders: council-compliance, council-privacy, ops-security, axis-compliance
related_adrs:
  - ADR-0242
  - ADR-0244
  - ADR-0245
  - ADR-0248
  - ADR-0251
  - ADR-0263
  - ADR-0276
  - ADR-0292
  - ADR-0297
related_artifacts:
  - microservices/compliance/threat-model.md
  - microservices/compliance/dpia.md
  - microservices/compliance/compliance.md
  - microservices/compliance/multi-region.md
  - microservices/tenancy/policy/data-residency.md
  - docs/standards/documentation-rigor.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (compliance µservice)

## Purpose

Define where the compliance µservice's own data (compliance-pack signatures, DPIA records, regulator-evidence cadence trackers, breach-notification workflow state, cell-certification attestations, control-mapping records) lives, the cross-pack replication policy for that data, and the legal-transfer mechanisms that gate any exception.

The compliance µservice is a **substrate µservice** per ADR-0245; it serves every tenant across every active compliance pack. Its data residency obligations are therefore the union of all per-pack residency floors — pack-pinning at the *tenant* level (tenancy µservice's concern per ADR-0244) is inherited and extended here for the compliance-substrate's own evidence storage.

## Residency model

### Substrate-level invariants

1. **Pack signatures** (the cryptographically signed manifests authored by `oyatie.platform-ops.compliance-office`) live in the **global control plane** (Tier 1 per ADR-0248). Pack signatures are public artifacts (signed + Rekor-anchored per ADR-0247); they do not carry tenant PII.
2. **Per-tenant DPIA records** live in the **tenant's home_cell** per ADR-0244 (Tier 3); DPIAs contain tenant + processing-context PII subject to GDPR Art. 35 / KR-PIPA Art. 33 / equivalent.
3. **Regulator-evidence cadence trackers** live in the **tenant's home_cell** unless the pack mandates a sovereign-pack cell (e.g., CN-PIPL → mainland-CN cells per ADR-0251 §D-2 `cn-pipl-eligible`).
4. **Breach-notification workflow state** lives in the **tenant's home_cell + DR-pair cell** per ADR-0241; replication is per-pack regulator-floor (e.g., EU NIS2 Art. 23 three-stage cadence → state must survive single-cell failure; cross-ref ADR-0251 §nis2_three_stage_cadence).
5. **Cell-certification attestations** live in the **certifying cell** + replicated to the global attestation registry under sigstore + Rekor per ADR-0247. Tier 0 ceremony.
6. **Compliance-control mapping records** live in the **global control plane** (Tier 1); these are pack-scoped, not tenant-scoped.

### Per-pack residency floors honored

| Pack | Tenant data | Pack data | Sovereign-cell required |
|---|---|---|---|
| **EU-GDPR-2018-baseline** | EU cells only unless SCCs + per-tenant opt-in | EU control plane | No (EU cells but global control plane permitted) |
| **KR-PIPA-2023-amendment** | KR cells; cross-border requires PIPC notification | KR control plane mirror | Yes — `kr-pipa-eligible` cells |
| **KR-CSAP-v3.1** | KR sovereign cells | KR sovereign control plane | Yes — `kr-csap-eligible` |
| **CN-PIPL-2021** | Mainland-CN cells only (PIPL Art. 40); cross-border requires CAC security assessment | Mainland-CN control plane mirror | Yes — `cn-pipl-eligible` |
| **HIPAA-2024** | HIPAA-eligible US cells | US control plane; PHI never crosses border without BAA | Yes — `hipaa-eligible` |
| **PCI-DSS-L1-v4** | PCI-eligible cells; PAN tokenized at L0 edge | Global control plane (no card data) | Yes — `pci-dss-l1-eligible` |
| **FedRAMP-High-v5** | FedRAMP-High-authorized GovCloud cells | GovCloud control plane | Yes — `fedramp-high-authorized` |
| **EU-AI-ACT-2024-HIGH-RISK** | EU cells for EU-residing data subjects | EU control plane | No (EU cells but global control plane permitted) |

Cross-pack conflict resolution: higher-restriction floor wins (per documentation-rigor.md §3.2.5 row 23). A tenant with both EU-GDPR + CN-PIPL packs active CANNOT have a single home_cell satisfying both — pack admission rejects this configuration at activation time.

## Cross-border transfer gating

Cross-border transfer of compliance-µservice data is permitted only via:

1. **GDPR Chapter V mechanisms** — Standard Contractual Clauses (SCCs 2021/914/EU), Binding Corporate Rules, Article 49 derogations (narrow), adequacy decisions.
2. **KR-PIPA Article 28 + 23-2** — PIPC notification + tenant consent + adequacy or contractual safeguards.
3. **CN-PIPL Article 38-40** — CAC security assessment OR Standard Contractual Clauses (CAC Measures 2023) OR Personal Information Protection Certification.
4. **HIPAA** — BAA required with every cross-border processor; PHI in transit MUST be encrypted per ADR-0251 §D-10.
5. **Per-pack DPA** signed before any cross-border processor onboarded.

Cross-border transfers are tagged in every audit event per ADR-0263 with the `cross_border_transfer_authority` field; the transparency report enumerates the active mechanisms per pack per ADR-0249.

## Enforcement

- Cedar fragment `policy/data-residency.cedar` (paired with this document) gates every compliance-µservice read/write against the resolved residency rule for the calling tenant.
- CI lane `oya-check-compliance-data-residency` enforces no compliance-µservice migration introduces a cross-pack residency violation.
- Per-pack regulator review cadence per ADR-0251 `regulator_evidence_cadence` field.
- Cross-ref documentation-rigor.md §3.2.1 row 23 (cross-jurisdiction conflict) + §3.2.4 Domain 14 (data classification + lineage).

## Out of scope

- Per-tenant home_cell assignment policy (lives in `microservices/tenancy/policy/data-residency.md`).
- Per-data-class encryption key management (lives in `microservices/cloud-secrets/`).
- Sovereign-cloud overlay primitives (lives in `docs/standards/sovereign-cloud-overlay.md`).

## References

- ADR-0242 — oyatie-is-a-tenant
- ADR-0244 — tenant scoping primitive
- ADR-0248 — Amazon-shape cellular architecture (Tier 0-4)
- ADR-0251 — Compliance Pack + Cell Certification Levels
- ADR-0276 — Backup portability GDPR Art. 20
- `microservices/tenancy/policy/data-residency.md` — the load-bearing tenant-side residency contract
- GDPR Chapter V (Arts. 44-50); KR-PIPA Arts. 28 + 23-2; CN-PIPL Arts. 38-40; HIPAA 45 CFR 164.504(e) BAA requirements
