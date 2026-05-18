---
doc_class: PolicySpec
title: Data Residency Contract
microservice: workflow-studio
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-workflow
deciders: council-privacy, ops-security, axis-workflow, gtm-customer-success
related_adrs: [ADR-0117, ADR-0131]
related_artifacts:
  - microservices/workflow-studio/threat-model.md
  - microservices/workflow-studio/dpia.md
  - microservices/workflow-studio/multi-region.md
  - microservices/workflow-studio/policy/editor-isolation.md
review_cadence: annually + on every regional-pack activation OR LLM-assist provider change
doc_status: published
---

# Data Residency Contract (workflow-studio µservice)

## Purpose

Define which jurisdictions' tenant editor sessions, drafts, collab state, LLM-assist prompts, and per-seat license attribution live in which cluster; the cross-pack replication policy for Studio assets; the CDN edge residency model; and the legal-transfer mechanisms gating any exception. Canonical residency artifact reviewed by EU DPAs (per GDPR Arts. 44-50), Korean PIPC (per PIPA Art. 28 + Art. 23-2), HIPAA Covered Entity counsel, and equivalent supervisory authorities.

## Residency Model

### Default: pack-pinning

Every tenant assigned a primary pack at onboarding. Editor sessions, drafts, collab CRDT state, per-seat license attribution, and LLM-assist invocations all live in the pack's region-pinned Studio cluster. Cross-pack movement **forbidden by default**.

| Pack | Primary region(s) | Studio cluster footprint | CDN edges | Activated? |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-studio-{pg,redis,ws-gateway}-1 | OCI CDN KR PoPs | YES (M03 launch tenant) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-studio-{pg,redis,ws-gateway}-{1,2} | OCI CDN EU PoPs | Conditional (first EU tenant SCC) |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 (DR pair) | us-studio-{pg,redis,ws-gateway}-{1,2} | OCI CDN US PoPs | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | us-hc-studio-{pg,redis,ws-gateway}-1; isolated from pack-us | OCI CDN HIPAA-eligible PoPs | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | jp-studio-{pg,redis,ws-gateway}-1 | OCI CDN JP PoPs | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-studio-{pg,redis,ws-gateway}-1 | OCI CDN SG PoPs | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-studio-{pg,redis,ws-gateway}-{1,2} | OCI CDN AU PoPs | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-studio-{pg,redis,ws-gateway}-{1,2} | OCI CDN IN PoPs | Conditional (DPDPA) |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-studio-{pg,redis,ws-gateway}-{1,2} | OCI CDN BR PoPs | Conditional (LGPD) |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-studio-{pg,redis,ws-gateway}-{1,2} | OCI CDN ME PoPs | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-studio-{pg,redis,ws-gateway}-{1,2} | OCI CDN KSA PoPs | Conditional (KSA NCA) |

### Pack-assignment routing

```text
Tenant onboarding
    ↓
gtm-customer-success collects HQ jurisdiction + regulated-data declarations
    ↓
Pack-router (Cedar policy in tenancy µservice) maps tenant → pack
    ↓
OpenBao assigns tenant → pack
    ↓
Browser DNS → pack-pinned Studio endpoint (studio-<pack>.oyatie.dev)
    ↓
CDN serves WASM bundle from pack-resident edge PoP
    ↓
All editor sessions + CRDT state + LLM-assist routes through pack cluster
```

### Pack determines LLM-assist routing

| Pack | LLM-assist routing target |
|---|---|
| pack-kr | KR-resident LLM provider via foundry-providers (e.g., Solar, KT-LLM) |
| pack-eu | EU-resident LLM provider (e.g., Mistral EU, Aleph Alpha) |
| pack-us / pack-us-healthcare | US-resident LLM provider (Anthropic Claude via AWS Bedrock US, OpenAI EU/US regions); HIPAA-BAA-eligible providers only for pack-us-healthcare |
| pack-jp | JP-resident LLM provider |
| (others) | Pack-resident provider chosen by foundry-providers routing policy |

Tenant may opt-out of LLM-assist entirely (no foundry-providers invocation). Tenant may BYO-LLM (their own provider; foundry-providers routes through tenant's egress).

## Cross-Pack Replication Policy

### Default: forbidden

- Postgres editor session state: replicate within-pack only.
- Valkey ephemeral CRDT: per-cell; not cross-region replicated (regenerable from Postgres).
- Per-seat license attribution: within-pack only.
- LLM-assist prompts + completions (90d retention): within-pack only.
- Studio audit-chain seals: replicate within-pack only.
- Node library descriptors + signatures: **global** (git-versioned + per-pack signed); content is tenant-agnostic.
- Cedar policies: **global** (git-versioned).
- WASM bundles + design-system primitives: **global** (CDN edges; tenant-agnostic).

### Exception: tenant-executed SCCs

Cross-border transfer of EU-resident editor state permitted only with active SCC per GDPR Arts. 44-46. Requires:
1. Active SCC on file at `legal/transfer-register.md`.
2. Receiving-pack jurisdiction has adequate-decision or equivalent.
3. Transfer-purpose limited to named processing (e.g., "DR failover").
4. Audit-chain emission at moment of transfer.

### Exception: HIPAA BAA + DR failover

Covered Entity tenants in pack-us-healthcare have DR pair us-ashburn-1 + us-phoenix-1; failover intra-region for HIPAA.

### Exception: BCDR exercise

Controlled cross-region restore drills permitted intra-pack only (eu-frankfurt-1 → eu-amsterdam-1, etc.). Cross-pack BCDR NOT authorised.

## Tenant Tagging by Jurisdiction

Studio entities carry jurisdiction labels for routing + retention enforcement:

```text
metric_label / row_label:
  jurisdiction: kr | eu | us | us-hc | jp | sg | au | in | br | ae | ksa
  pack:         pack-kr | pack-eu | ... (mirrors jurisdiction)
  data_class:   one of class taxonomy values per Bominal ADR-0028
```

## Retention by Jurisdiction × Data Class

| Pack | Data class | Minimum statutory | Default applied |
|---|---|---|---|
| pack-kr | `BEHAVIORAL_TENANT_PRODUCT` (sessions) | KR commercial code: 5y; not required for editor drafts | 30d hot; aggressive purge |
| pack-kr | `SENSITIVE_PIPA_ART23` | PIPA Art. 28: bounded; erasure on request | 1y default; honour erasure |
| pack-kr | `AUDIT` (save events) | PIPA Enforcement Decree Art. 30: ≥ 1y | 3y aligned (KR-FSS sector) |
| pack-eu | `PII_IDENTIFYING` | GDPR Art. 17: bounded; right-to-erasure within 30d | bounded; honour erasure |
| pack-eu | `AUDIT` | bounded by purpose; in ROPA | 2y default |
| pack-eu | LLM-assist prompts | GDPR Art. 5(1)(e) storage limitation | 90d hot; aggressive purge after |
| pack-us-healthcare | `PHI` (spec drafts mentioning patients) | HIPAA: state-dependent | MAX(HIPAA 6y, state, tenant DPA) |
| pack-us-healthcare | `AUDIT` | HIPAA §164.316(b)(2): 6y | 6y |
| pack-jp | `PII_IDENTIFYING` | APPI: bounded; deletion request | bounded |
| pack-au | `PII_IDENTIFYING` | Privacy Act APP 11+12: bounded | bounded |
| pack-in | `PII_IDENTIFYING` | DPDPA 2023 §8(1)(g) storage limitation | bounded |
| pack-br | `PII_IDENTIFYING` | LGPD Art. 16 | bounded |
| (all) | `SECRET` | rotate per ISO 27001 A.5.17 | 30d API keys, 90d signing keys |
| (all) | Editor session drafts | n/a (transient) | 30d after last access; aggressive purge |
| (all) | LLM-assist prompts | varies | 90d hot for audit; purge after |

CI lane `oya-governance-retention-conformance` validates Studio retention configs against this table.

## DSR Cascade

Right-to-erasure (GDPR Art. 17 / PIPA Art. 36 / DPDPA §12 / LGPD Art. 18) honoured via `oya-dsr-cascade-runner`:

1. Tenant raises DSR on behalf of end-user (joint controllership per Art. 26).
2. DSR runner identifies end-user identifiers in:
   - Editor session metadata (author OIDC sub).
   - Spec draft contents (user-id field patterns).
   - LLM-assist prompts (prose mentioning end-user).
   - Per-seat license attribution.
3. Postgres + Valkey + audit-chain searched; per-row deletion with 30-day soft-delete grace; hard-delete after.
4. Audit-chain seal: `dsr_executed{tenant, subject_hash, removed_rows_count, timestamp}`.
5. Tenant notified within 30d per GDPR; per-pack SLAs (KR 30d, BR 15d, EU 30d) respect strictest applicable.

Limitations (DPIA R-07):
- Data older than retention may be deleted before DSR processed.
- LLM-assist prompts at LLM provider may persist beyond DSR window (mitigated by zero-retention provider selection).

## Per-Pack Overlay Sections

### pack-kr (PIPA + PIPC)

- PIPA Art. 28 (storage limitation): bounded; sensitive data minimal retention.
- PIPA Art. 23-2 (cross-border sensitive): forbidden by default; LLM-assist routes KR-resident only.
- PIPC Notice 2020-7 (overseas-transfer notification): pack-kr residency in tenant DPA.
- KR-FSS sector guidance: audit log retention ≥ 5y; KMS keys in KR.

### pack-eu (GDPR + EDPB + Schrems II + AI Act)

- GDPR Arts. 44-46 transfer mechanisms: SCC-only; LLM-assist routes EU-resident.
- EDPB Recommendations 01/2020: supplementary measures at `legal/schrems-supplementary-measures.md`.
- GDPR Art. 32 + 25: pseudonymisation + EU-resident KMS + Studio assets cached at EU-resident CDN PoPs.
- EU AI Act 2024 Art. 12 (record-keeping): LLM-assist invocation log retention 6mo minimum when used in high-risk-classified workflow context.

### pack-us-healthcare (HIPAA)

- 45 CFR §164.530(j): records retention ≥ 6y.
- HIPAA-eligible regions: OCI us-ashburn-1 + us-phoenix-1.
- BAA-required before pack-us-healthcare ingest enabled.
- LLM-assist provider must be HIPAA BAA-eligible (e.g., AWS Bedrock with BAA, Azure OpenAI with BAA).

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/workflow-studio-data-residency-overlay.md`.

## Verification

- `oya gate validate retention-conformance` — exit 0.
- `oya gate validate pack-routing-conformance` — exit 0.
- `oya gate validate cross-region-transfer-allowed-only-with-scc` — exit 0.
- `oya gate validate llm-assist-pack-resident-routing` — exit 0.
- Annual residency audit.
- Quarterly chaos drill: induce cross-pack write attempt; verify rejection + alerting.

## References

- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0131: Per-microservice flat layout + workflow unbundle.
- `microservices/workflow-studio/threat-model.md` T-I-01.
- `microservices/workflow-studio/dpia.md` R-09 + R-11 + R-13.
- `microservices/workflow-studio/multi-region.md`.
- `microservices/workflow-studio/policy/editor-isolation.md`.
- `microservices/workflow-studio/legal/{transfer-register, schrems-supplementary-measures, baa-template, dpa-template, sub-processors, ropa, ai-act-conformity}.md`.
- `regional-packs/<pack>/workflow-studio-data-residency-overlay.md`.
- OCI region documentation.
- GDPR Arts. 44-50.
- EDPB Recommendations 01/2020.
- KR PIPA Art. 23-2 + Art. 28 + PIPC Notice 2020-7.
- HIPAA 45 CFR §164.530(j).
- LGPD Art. 16 + Art. 33.
- DPDPA 2023 §8(1)(g).
- EU AI Act 2024 Art. 12.
