---
doc_class: PolicySpec
title: Data Residency Contract
microservice: sheets
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-sheets
deciders: council-privacy, ops-security, axis-sheets, gtm-customer-success
related_adrs: [ADR-0117, ADR-0135, ADR-0131]
related_artifacts:
  - microservices/sheets/threat-model.md
  - microservices/sheets/dpia.md
  - microservices/sheets/multi-region.md
  - microservices/sheets/policy/editor-isolation.md
review_cadence: annually + on every regional-pack activation OR AI-formula provider change
doc_status: published
---

# Data Residency Contract (sheets µservice)

## Purpose

Define which jurisdictions' tenant workbook state, cell-edit logs, collab CRDT state, AI-formula prompts, connected-sheets external-query results, comments, version-history snapshots, and per-seat license attribution live in which cluster; the cross-pack replication policy; the CDN edge residency model; and the legal-transfer mechanisms gating any exception.

## Residency Model

### Default: pack-pinning

Every tenant assigned a primary pack at onboarding. Workbook state, cell-edit logs, collab CRDT state, comments, version-history, per-seat license attribution, AI-formula invocations, and connected-sheets external-query results all live in the pack's region-pinned Sheets cluster. Cross-pack movement **forbidden by default**.

| Pack | Primary region(s) | Sheets cluster footprint | CDN edges | Activated? |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-sheets-{pg,valkey,ws-gateway,recalc-worker,xlsx-export,object-storage}-1 | OCI CDN KR PoPs | YES (M03 launch) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-sheets-{...}-{1,2} | OCI CDN EU PoPs | Conditional (first EU tenant SCC) |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 (DR pair) | us-sheets-{...}-{1,2} | OCI CDN US PoPs | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | us-hc-sheets-{...}-1; isolated from pack-us | OCI CDN HIPAA-eligible PoPs | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | jp-sheets-{...}-1 | OCI CDN JP PoPs | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-sheets-{...}-1 | OCI CDN SG PoPs | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-sheets-{...}-{1,2} | OCI CDN AU PoPs | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-sheets-{...}-{1,2} | OCI CDN IN PoPs | Conditional (DPDPA) |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-sheets-{...}-{1,2} | OCI CDN BR PoPs | Conditional (LGPD) |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-sheets-{...}-{1,2} | OCI CDN ME PoPs | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-sheets-{...}-{1,2} | OCI CDN KSA PoPs | Conditional (KSA NCA) |

### Pack determines AI-formula routing

| Pack | AI-formula routing target |
|---|---|
| pack-kr | KR-resident LLM provider via foundry-runtime (e.g., Solar, KT-LLM) |
| pack-eu | EU-resident LLM provider (e.g., Mistral EU, Aleph Alpha) |
| pack-us / pack-us-healthcare | US-resident LLM provider (Anthropic Claude via AWS Bedrock US, OpenAI EU/US regions); HIPAA-BAA-eligible providers only for pack-us-healthcare |
| pack-jp | JP-resident LLM provider |
| (others) | Pack-resident provider chosen by foundry-runtime routing policy |

### Pack determines connected-sheets external-source routing

Connected-sheets external SQL sources must be reachable from the pack-resident network; cross-pack external-source connections require tenant-executed SCC.

## Cross-Pack Replication Policy

### Default: forbidden

- Postgres workbook + cell + edit-log + license-attribution + share-ACL + range-ACL + comments + version-pointers: replicate within-pack only.
- Valkey ephemeral CRDT: per-cell; not cross-region replicated (regenerable from Postgres).
- S3 workbook snapshots + version-history: replicate within-pack only.
- OCI Object Storage Arrow/Parquet large-sheet blocks: replicate within-pack only.
- AI-formula prompts + completions (90d retention): within-pack only.
- Connected-sheets external-query results: within-pack only.
- XLSX upload quarantine + XLSX export jobs: within-pack only.
- Sheets audit-chain seals: replicate within-pack only.
- Function-library descriptor catalog: **global** (compiled-into-binary; per-pack overlays via Cedar).
- Cedar policies: **global** (git-versioned).
- WASM bundles + design-system primitives: **global** (CDN edges; tenant-agnostic).

### Exception: tenant-executed SCCs

Cross-border transfer of EU-resident workbook state permitted only with active SCC per GDPR Arts. 44-46.

### Exception: HIPAA BAA + DR failover

Covered Entity tenants in pack-us-healthcare have DR pair us-ashburn-1 + us-phoenix-1; failover intra-region for HIPAA.

### Exception: BCDR exercise

Controlled cross-region restore drills permitted intra-pack only.

## Tenant Tagging by Jurisdiction

Sheets entities carry jurisdiction labels for routing + retention enforcement:

```text
metric_label / row_label:
  jurisdiction: kr | eu | us | us-hc | jp | sg | au | in | br | ae | ksa
  pack:         pack-kr | pack-eu | ...
  data_class:   one of class taxonomy values per Bominal ADR-0028
```

## Retention by Jurisdiction × Data Class

| Pack | Data class | Minimum statutory | Default applied |
|---|---|---|---|
| pack-kr | `BEHAVIORAL_TENANT_PRODUCT` (cell-edits) | KR commercial code: 5y; not required for cell-edit log | 30d hot; aggressive purge |
| pack-kr | `SENSITIVE_PIPA_ART23` | PIPA Art. 28: bounded; erasure on request | 1y default; honour erasure |
| pack-kr | `AUDIT` (cell-edit + license-gate seals) | PIPA Enforcement Decree Art. 30: ≥ 1y | 3y (KR-FSS sector) |
| pack-eu | `PII_IDENTIFYING` | GDPR Art. 17: bounded; right-to-erasure within 30d | bounded; honour erasure |
| pack-eu | `AUDIT` | bounded by purpose; in ROPA | 2y default |
| pack-eu | AI-formula prompts | GDPR Art. 5(1)(e) storage limitation | 90d hot; aggressive purge after |
| pack-us-healthcare | `PHI` (clinical workbooks) | HIPAA: state-dependent | MAX(HIPAA 6y, state, tenant DPA) |
| pack-us-healthcare | `AUDIT` | HIPAA §164.316(b)(2): 6y | 6y |
| pack-jp | `PII_IDENTIFYING` | APPI: bounded | bounded |
| pack-au | `PII_IDENTIFYING` | Privacy Act APP 11+12: bounded | bounded |
| pack-in | `PII_IDENTIFYING` | DPDPA 2023 §8(1)(g) storage limitation | bounded |
| pack-br | `PII_IDENTIFYING` | LGPD Art. 16 | bounded |
| (all) | `SECRET` | rotate per ISO 27001 A.5.17 | 30d API keys, 90d signing keys |
| (all) | Workbook drafts | n/a (transient) | 30d after last access; aggressive purge |
| (all) | XLSX upload quarantine | n/a | 7d; then delete |
| (all) | XLSX export output | n/a | 24h hot for download; then delete |
| (all) | Connected-sheets external-query results | per refresh | not retained beyond materialized range |
| (all) | Version-history snapshots | n/a | 90d hot; 7y cold |
| (all) | Arrow/Parquet large-sheet blocks | n/a | 30d hot; cold-tier per pack |

CI lane `oya-governance-retention-conformance` validates Sheets retention configs against this table.

## DSR Cascade

Right-to-erasure honoured via `oya-dsr-cascade-runner`:

1. Tenant raises DSR on behalf of end-user.
2. DSR runner identifies end-user identifiers in:
   - Workbook cells.
   - Cell-edit logs.
   - AI-formula prompts.
   - Connected-sheets external-query result materialized ranges.
   - Comments + threaded notes.
   - Version-history snapshots.
   - Per-seat license attribution.
   - Audit-chain seals.
3. Postgres + Valkey + S3 + Arrow/Parquet + audit-chain searched; per-row deletion with 30-day soft-delete grace; hard-delete after.
4. Audit-chain seal: `dsr_executed{tenant, subject_hash, removed_rows_count, timestamp}`.
5. Tenant notified within 30d per GDPR; per-pack SLAs.

## Per-Pack Overlay Sections

### pack-kr (PIPA + PIPC)

- PIPA Art. 28 (storage limitation).
- PIPA Art. 23-2 (cross-border sensitive): forbidden by default; AI-formula routes KR-resident.
- PIPC Notice 2020-7.
- KR-FSS sector guidance: audit log retention ≥ 5y; KMS keys in KR.

### pack-eu (GDPR + EDPB + Schrems II + AI Act)

- GDPR Arts. 44-46: SCC-only; AI-formula routes EU-resident.
- EDPB Recommendations 01/2020: supplementary measures at `legal/schrems-supplementary-measures.md`.
- GDPR Art. 32 + 25.
- EU AI Act 2024 Art. 12: AI-formula invocation log retention 6mo minimum when used in high-risk-classified workflow context per ADR-SHEETS-0005.

### pack-us-healthcare (HIPAA)

- 45 CFR §164.530(j): records retention ≥ 6y.
- HIPAA-eligible regions: OCI us-ashburn-1 + us-phoenix-1.
- BAA-required before pack-us-healthcare ingest enabled.
- AI-formula provider must be HIPAA BAA-eligible.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/sheets-data-residency-overlay.md`.

## Verification

- `oya gate validate retention-conformance --microservice sheets` — exit 0.
- `oya gate validate pack-routing-conformance --microservice sheets` — exit 0.
- `oya gate validate cross-region-transfer-allowed-only-with-scc --microservice sheets` — exit 0.
- `oya gate validate ai-formula-pack-resident-routing --microservice sheets` — exit 0.
- Annual residency audit.
- Quarterly chaos drill.

## References

- ADR-0117, ADR-0135, ADR-0131.
- ADR-SHEETS-0005 (AI-formula bounds + routing).
- `microservices/sheets/threat-model.md` T-I-01.
- `microservices/sheets/dpia.md` R-09 + R-11 + R-13.
- `microservices/sheets/multi-region.md`.
- `microservices/sheets/policy/editor-isolation.md`.
- `microservices/sheets/legal/{transfer-register, schrems-supplementary-measures, baa-template, dpa-template, sub-processors, ropa, ai-act-conformity}.md`.
- `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/sheets-data-residency-overlay.md`.
- GDPR Arts. 44-50.
- EDPB Recommendations 01/2020.
- KR PIPA Art. 23-2 + Art. 28 + PIPC Notice 2020-7.
- HIPAA 45 CFR §164.530(j).
- LGPD Art. 16 + Art. 33.
- DPDPA 2023 §8(1)(g).
- EU AI Act 2024 Art. 12.
