---
doc_class: PolicySpec
title: Data Residency Contract
microservice: workflow-engine
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-workflow
deciders: council-privacy, ops-security, axis-workflow, gtm-customer-success
related_adrs: [ADR-0117, ADR-0131]
related_artifacts:
  - microservices/workflow-engine/threat-model.md
  - microservices/workflow-engine/dpia.md
  - microservices/workflow-engine/multi-region.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (workflow-engine µservice)

## Purpose

Define which jurisdictions' tenant workflow runs live in which Postgres + Citus / Redis / ClickHouse cluster, the cross-pack replication policy, and the legal-transfer mechanisms that gate any exception. Canonical residency artifact reviewed by EU DPAs (per GDPR Arts. 44-50), the Korean PIPC (per PIPA Art. 28 + Art. 23-2), HIPAA tenants' Covered Entity counsel, and equivalent supervisory authorities.

## Residency Model

### Default: pack-pinning

Every tenant is assigned a primary pack at onboarding. Workflow specs, run state, event log, and audit seals all live in the pack's region-pinned engine cluster. Cross-pack movement is **forbidden by default**.

| Pack | Primary region(s) | Cluster footprint | Activated? |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-engine-pg-1, kr-engine-redis-1, kr-engine-ch-1 | YES (M02b launch tenant) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-engine-{pg,redis,ch}-{1,2} | Conditional (activated on first EU tenant SCC) |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 (DR pair) | us-engine-{pg,redis,ch}-{1,2} | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | us-hc-engine-{pg,redis,ch}-1; isolated from pack-us | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | jp-engine-{pg,redis,ch}-1 | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-engine-{pg,redis,ch}-1 | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-engine-{pg,redis,ch}-{1,2} | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-engine-{pg,redis,ch}-{1,2} | Conditional (DPDPA) |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-engine-{pg,redis,ch}-{1,2} | Conditional (LGPD) |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-engine-{pg,redis,ch}-{1,2} | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-engine-{pg,redis,ch}-{1,2} | Conditional (KSA NCA) |

### Pack-assignment routing

```text
Tenant onboarding
    ↓
gtm-customer-success collects HQ jurisdiction + regulated-data declarations
    ↓
Pack-router (Cedar policy) maps:
  - HQ jurisdiction → primary pack
  - Regulated-data flag → may force secondary pack
    ↓
OpenBao assigns tenant → pack
    ↓
Workload SDK is configured with pack-pinned engine endpoints
    ↓
All workflow runs flow to the pack's engine cluster; never cross-pack
```

Routing is encoded as a Cedar policy fragment at `policy/pack-routing.cedar` (Slice D).

### Per-tenant tenant_scope influences capacity, not residency

Pack-pinning invariant; `tenant_scope` (trial/production/sandbox/internal) affects capacity allocation within the pack but does not move data across packs.

## Cross-Pack Replication Policy

### Default: forbidden

- Postgres run state: replicate within-pack only.
- Event log + outbox: replicate within-pack only.
- ClickHouse run history: replicate within-pack only.
- Audit-chain seals: replicate within-pack only.
- Spec versions: pack-pinned; cross-pack replication forbidden (specs may carry tenant-confidential business logic).
- Cedar policies + workflow event registry: configuration is global (git-versioned).

### Exception: tenant-executed SCCs

Cross-border transfer of EU-resident workflow data is permitted only when tenant has executed an active SCC per GDPR Arts. 44-46. Requires:
1. Active SCC on file at `legal/transfer-register.md`.
2. Receiving-pack jurisdiction has adequate-decision or equivalent.
3. Transfer-purpose limited to named processing (e.g., "DR failover").
4. Audit-chain emission at moment of transfer.

### Exception: HIPAA BAA + DR failover

Covered Entity tenants in pack-us-healthcare have DR pair us-ashburn-1 + us-phoenix-1; failover intra-region for HIPAA.

### Exception: BCDR exercise

Controlled cross-region restore drills are permitted intra-pack only (eu-frankfurt-1 → eu-amsterdam-1, us-ashburn-1 → us-phoenix-1, etc.). Cross-pack BCDR is not authorised.

## Tenant Tagging by Jurisdiction

Engine entities carry jurisdiction labels for routing + retention enforcement:

```text
metric_label / row_label:
  jurisdiction: kr | eu | us | us-hc | jp | sg | au | in | br | ae | ksa
  pack:         pack-kr | pack-eu | ... (mirrors jurisdiction)
  data_class:   one of the class taxonomy values per Bominal ADR-0028
```

## Retention by Jurisdiction × Data Class

| Pack | Data class | Minimum statutory | Default applied |
|---|---|---|---|
| pack-kr | `BEHAVIORAL_TENANT_PRODUCT` | KR commercial code: 5y | 5y aligned |
| pack-kr | `SENSITIVE_PIPA_ART23` | PIPA Art. 28: bounded; erasure on request | 1y default; honour erasure |
| pack-kr | `AUDIT` (run-history seals) | PIPA Enforcement Decree Art. 30: ≥ 1y | 3y aligned (KR-FSS sector) |
| pack-eu | `PII_IDENTIFYING` | GDPR Art. 17: bounded; right-to-erasure within 30d | bounded; honour erasure |
| pack-eu | `AUDIT` | bounded by purpose; in ROPA | 2y default |
| pack-us-healthcare | `PHI` (step payloads) | HIPAA: state-dependent | MAX(HIPAA 6y, state, tenant DPA) |
| pack-us-healthcare | `AUDIT` | HIPAA §164.316(b)(2): 6y | 6y |
| pack-jp | `PII_IDENTIFYING` | APPI: bounded; deletion request | bounded |
| pack-au | `PII_IDENTIFYING` | Privacy Act APP 11+12: bounded | bounded |
| pack-in | `PII_IDENTIFYING` | DPDPA 2023 §8(1)(g) storage limitation | bounded |
| pack-br | `PII_IDENTIFYING` | LGPD Art. 16 | bounded |
| (all) | `SECRET` | rotate per ISO 27001 A.5.17 | 30d API keys, 90d signing keys |

CI lane `oya-governance-retention-conformance` validates engine retention configs against this table.

## DSR Cascade

Right-to-erasure (GDPR Art. 17 / PIPA Art. 36 / DPDPA §12 / LGPD Art. 18(V)-(VI)) honoured via `oya-dsr-cascade-runner`:

1. Tenant raises DSR on behalf of end-user (joint controllership per Art. 26).
2. DSR runner identifies end-user identifiers (user-id hash, payload-field patterns, span attributes).
3. Postgres + outbox + ClickHouse per-row deletion API invoked; soft-delete with 30-day grace; hard-delete after.
4. Audit-chain seal: `dsr_executed{tenant, subject_hash, removed_rows_count, timestamp}`.
5. Tenant notified within 30d per GDPR; per-pack SLAs (KR 30d, BR 15d, EU 30d) respect strictest applicable.

Limitations (DPIA R-07):
- Data older than retention may be deleted before DSR processed.
- Step payloads sampled — only partial coverage.

## Per-Pack Overlay Sections

### pack-kr (PIPA + PIPC)

- PIPA Art. 28 (storage limitation): bounded; sensitive data minimal retention.
- PIPA Art. 23-2 (cross-border sensitive): forbidden by default; consent at tenant DPA.
- PIPC Notice 2020-7 (overseas-transfer notification): pack-kr residency in tenant DPA.
- KR-FSS sector guidance (financial-services tenants): audit log retention ≥ 5y; KMS in KR.

### pack-eu (GDPR + EDPB + Schrems II)

- GDPR Arts. 44-46 transfer mechanisms: SCC-only; Adequacy via EU list; Schrems-II supplementary measures (pseudonymisation + EU-controlled KMS).
- EDPB Recommendations 01/2020: supplementary measures at `legal/schrems-supplementary-measures.md`.
- GDPR Art. 32 + 25: pseudonymisation + EU-resident-key encryption as appropriate technical measures.

### pack-us-healthcare (HIPAA)

- 45 CFR §164.530(j): records retention ≥ 6y.
- HIPAA-eligible regions: OCI us-ashburn-1 + us-phoenix-1.
- BAA-required before pack-us-healthcare ingest enabled.
- Permitted Uses: TPO; workflow execution under Operations.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Each pack's overlay at `regional-packs/<pack>/data-residency-overlay.md` carries local citations. Pack-pinning + cross-pack-replication-forbidden apply universally.

## Verification

- `oya gate validate retention-conformance` — exit 0.
- `oya gate validate pack-routing-conformance` — exit 0.
- `oya gate validate cross-region-transfer-allowed-only-with-scc` — exit 0.
- Annual residency audit.
- Quarterly chaos drill: induce cross-pack write attempt; verify rejection + alerting.

## References

- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0131: Per-microservice flat layout + workflow unbundle.
- `microservices/workflow-engine/threat-model.md` T-I-01.
- `microservices/workflow-engine/dpia.md` R-09 + R-11 + R-13.
- `microservices/workflow-engine/multi-region.md`.
- `microservices/workflow-engine/legal/{transfer-register, schrems-supplementary-measures, baa-template, dpa-template, sub-processors, ropa}.md`.
- `regional-packs/<pack>/data-residency-overlay.md`.
- OCI region documentation.
- GDPR Arts. 44-50.
- EDPB Recommendations 01/2020.
- KR PIPA Art. 23-2 + Art. 28 + PIPC Notice 2020-7.
- HIPAA 45 CFR §164.530(j).
- LGPD Art. 16 + Art. 33.
- DPDPA 2023 §8(1)(g).
