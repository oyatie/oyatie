---
doc_class: PolicySpec
title: Data Residency Contract
microservice: observability
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-observability
deciders: council-privacy, ops-security, axis-observability, gtm-customer-success
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/observability/threat-model.md (T-I-01, T-T-02; cross-region replication threats)
  - microservices/observability/dpia.md (R-11; cross-border-misroute risk)
  - microservices/observability/policy/tenant-isolation.md
  - microservices/observability/multi-region.md (Slice B)
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (observability µservice)

## Purpose

Define which jurisdictions' tenant data lives in which Mimir / Loki / Tempo / Pyroscope cluster, the cross-pack replication policy, and the legal-transfer mechanisms that gate any exception. This document is the canonical residency artifact reviewed by EU DPAs (per GDPR Arts. 44–50), the Korean PIPC (per PIPA Art. 28 + Art. 23-2), HIPAA tenants' Covered Entity counsel (per BAA), and equivalent supervisory authorities in every active pack.

## Residency Model

### Default: pack-pinning

Every tenant is assigned a primary pack at onboarding. The tenant's telemetry is stored in the pack's region-pinned Mimir / Loki / Tempo / Pyroscope cluster. Cross-pack movement is **forbidden by default**.

| Pack | Primary region(s) | Cluster footprint | Activated? |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-mimir-1, kr-loki-1, kr-tempo-1, kr-pyroscope-1 | YES (M01 launch tenant) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-mimir-{1,2}, eu-loki-{1,2}, eu-tempo-{1,2}, eu-pyroscope-{1,2} | Conditional (activated when first EU tenant signs SCC) |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 (DR pair) | us-mimir-{1,2}, us-loki-{1,2}, us-tempo-{1,2}, us-pyroscope-{1,2} | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | us-hc-mimir-1, us-hc-loki-1, us-hc-tempo-1; isolated from non-HC pack-us | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | jp-mimir-1, … | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-mimir-1, … | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-mimir-{1,2}, … | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-mimir-{1,2}, … | Conditional (DPDPA 2023) |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-mimir-{1,2}, … | Conditional (LGPD) |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-mimir-{1,2}, … | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-mimir-{1,2}, … | Conditional (KSA NCA cloud-residency requirements) |

The "Activated?" column is updated at first-tenant onboarding per pack; activation triggers re-review of this document + the per-pack threat-model overlay + DPIA overlay.

### Pack-assignment routing

```text
Tenant onboarding
    ↓
gtm-customer-success: collects tenant's HQ jurisdiction + regulated-data declarations
    ↓
Pack-router (Cedar policy in cloud-iac):
    - HQ jurisdiction → primary pack
    - Regulated-data flag (PHI, KR-FSS, EU-resident, etc.) → may force secondary pack
    - Conflict: ops-legal escalation (rare)
    ↓
OpenBao assigns tenant → pack
    ↓
Workload µservice's OTel collector (Grafana Alloy) is configured with pack-pinned endpoints
    ↓
All telemetry flows to the pack's clusters; never cross-pack
```

Routing is encoded as a Cedar policy at `microservices/observability/policy/pack-routing.cedar` (Slice D).

### Per-tenant tenant_scope influences capacity, not residency

Pack-pinning is invariant per tenant; `tenant_scope` (trial / production / sandbox / internal) affects capacity allocation within the pack but does not move data across packs.

## Cross-Pack Replication Policy

### Default: forbidden

Cross-pack replication of any tenant data is forbidden by default. Specifically:

- Mimir blocks (metrics): replicate within-pack only.
- Loki chunks (logs): replicate within-pack only.
- Tempo blocks (traces): replicate within-pack only.
- Pyroscope profiles: replicate within-pack only.
- Audit-chain seals: replicate within-pack only (each pack has its own audit-chain instance per `audit-chain` µservice's residency contract).
- Recording rules + alert rules: configuration is global (per-pack Helm values); evaluator state is per-pack.

### Exception: tenant-executed SCCs (GDPR transfer mechanism)

Cross-border transfer of EU-resident data is permitted only when the tenant has executed an active Standard Contractual Clause (SCC) or equivalent transfer mechanism per GDPR Arts. 44–46. The exception requires:

1. Active SCC on file at `microservices/observability/legal/transfer-register.md` (Slice D).
2. Receiving-pack jurisdiction has adequate-decision (GDPR Art. 45) or equivalent safeguard.
3. Transfer-purpose limited to specifically-named processing (e.g., "DR failover to pack-us"); ad-hoc transfer not authorised.
4. Audit-chain-emitted SCC-acknowledgement at the moment of transfer (every transfer event sealed).

### Exception: HIPAA BAA + DR failover

Covered Entity tenants in pack-us-healthcare may have DR pair us-ashburn-1 + us-phoenix-1; failover between the pair is intra-region from a HIPAA perspective (both are HIPAA-eligible OCI regions). Cross-region (us → eu) failover is NOT authorised without separate tenant agreement.

### Exception: BCDR exercise (controlled, scheduled)

For BCDR validation, controlled cross-region restore drills are permitted in pack-eu (eu-frankfurt-1 → eu-amsterdam-1) and pack-us (us-ashburn-1 → us-phoenix-1) — intra-pack only. Cross-pack BCDR is not authorised.

## Mimir Tenant Tagging by Jurisdiction

In addition to `X-Scope-OrgID = tenant:<hashed-id>`, Mimir samples carry jurisdiction labels for routing + retention enforcement:

```text
metric_label:
  jurisdiction: kr | eu | us | us-hc | jp | sg | au | in | br | ae | ksa
  pack:         pack-kr | pack-eu | pack-us | ... (mirrors jurisdiction with pack-prefix)
  data_class:   one of the class taxonomy values per Bominal ADR-0028
```

Properties:
- The `jurisdiction` label is set by the workload µservice's OTel SDK based on the tenant's pack assignment; tampering attempts (label-injection from untrusted code paths) are detected by the Alloy collector's enforcement layer.
- Mimir's per-tenant retention policy keys on `(tenant, jurisdiction, data_class)` to apply correct retention windows.
- The `pack` label is redundant with `jurisdiction` for routing convenience.

## Retention by Jurisdiction × Data Class

Retention windows are the MAX of:
- Asset class default (per `threat-model.md` §"Assets & Data Classification").
- Pack legal minimum (statutory retention).
- Tenant-contracted retention (DPA-declared).

| Pack | Data class | Minimum statutory | Default applied |
|---|---|---|---|
| pack-kr | `BEHAVIORAL_TENANT_PRODUCT` | KR commercial code: 5 years | 5y aligned |
| pack-kr | `SENSITIVE_PIPA_ART23` | PIPA Art. 28: bounded; erasure on request | 1y default; honour erasure |
| pack-kr | `AUDIT` | PIPA Enforcement Decree Art. 30: ≥ 1y | 3y aligned (KR-FSS sector guidance) |
| pack-eu | `PII_IDENTIFYING` | GDPR Art. 17: bounded; right-to-erasure within 30d | bounded; honour erasure |
| pack-eu | `AUDIT` | bounded by purpose; documented in ROPA | 2y default |
| pack-us-healthcare | `PHI` | HIPAA: depends on tenant's state Medical Records Retention law | use MAX(HIPAA 6y, state, tenant DPA) |
| pack-us-healthcare | `AUDIT` | HIPAA §164.316(b)(2): 6y | 6y |
| pack-jp | `PII_IDENTIFYING` | APPI: bounded; honour deletion request | bounded |
| pack-au | `PII_IDENTIFYING` | Privacy Act APP 11 + APP 12: bounded | bounded |
| pack-in | `PII_IDENTIFYING` | DPDPA 2023 §8(1)(g) (storage limitation) | bounded |
| pack-br | `PII_IDENTIFYING` | LGPD Art. 16 | bounded |
| (all packs) | `SECRET` | rotate per ISO 27001 A.5.17 cadence | 30d API keys, 90d signing keys |

The CI lane `oya-governance-retention-conformance` (NEW; Slice D) validates Mimir / Loki / Tempo retention configs against this table.

## DSR (Data Subject Request) Cascade

Right-to-erasure (GDPR Art. 17 / PIPA Art. 36 / DPDPA §12 / LGPD Art. 18(V)-(VI)) honoured via the `oya-dsr-cascade-runner` skill:

1. Tenant raises DSR on behalf of their end-user (joint controllership per Art. 26).
2. DSR runner identifies the end-user's identifiers (user-id hash, IP hash patterns, span attributes).
3. Mimir / Loki / Tempo per-series deletion API invoked; soft-delete with 30-day grace; hard-delete after grace.
4. Audit-chain seal: `dsr_executed{tenant, subject_hash, removed_series_count, timestamp}`.
5. Tenant notified within 30d SLA per GDPR; some packs (KR 30d, BR 15d, EU 30d) have shorter; tenant SLA respects the strictest of the per-pack legal SLAs applicable.

Limitations (documented in DPIA R-08):
- Data older than retention window may already be deleted before DSR processed.
- Trace data sampled at 1% has only partial coverage; redaction is best-effort.

## Per-Pack Overlay Sections

### pack-kr (KR PIPA + PIPC)

- **PIPA Art. 28 (storage period limitation)**: bounded; sensitive data minimal retention.
- **PIPA Art. 23-2 (sensitive data cross-border)**: forbidden by default; requires consent from data subject (sensitive tenant data covered by tenant DPA).
- **PIPC Notice 2020-7 (overseas-transfer notification)**: oyatie's pack-kr residency guarantee acknowledged in tenant DPA.
- **KR-FSS sector guidance** (financial-services tenants): audit log retention ≥ 5y; encrypted at rest with KMS keys in KR-resident KMS.

### pack-eu (GDPR + EDPB + Schrems II)

- **GDPR Arts. 44–46 transfer mechanisms**: SCC-only; Adequacy decision via EU-list-of-adequate-countries; Schrems-II-compliant supplementary technical measures (pseudonymisation + encryption-at-rest with EU-controlled KMS keys).
- **EDPB Recommendations 01/2020 (post-Schrems-II)**: supplementary measures documented in `microservices/observability/legal/schrems-supplementary-measures.md` (Slice D).
- **GDPR Art. 32 + 25**: pseudonymisation + EU-resident-key encryption considered "appropriate technical measures" for sensitive data at rest in pack-eu.

### pack-us-healthcare (HIPAA)

- **45 CFR §164.530(j) (Records retention)**: ≥ 6y from creation or last effective date.
- **HIPAA-eligible regions only**: OCI us-ashburn-1 + us-phoenix-1 per Oracle HIPAA-compliance attestation.
- **BAA-required**: tenant must sign BAA before pack-us-healthcare ingest enabled.
- **Permitted Uses + Disclosures**: TPO (treatment + payment + operations); operations scope covers observability.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Each pack's overlay at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/data-residency-overlay.md` carries the local data-residency law's citations. Pack-pinning + cross-pack-replication-forbidden invariants apply universally.

## Verification

- `cargo run -p oya-dev-cli -- gate validate retention-conformance` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate pack-routing-conformance` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate cross-region-transfer-allowed-only-with-scc` — exit 0.
- Annual residency audit: confirm each tenant's data location matches its assigned pack.
- Quarterly chaos drill: induce a cross-pack write attempt; verify rejection + alerting.

## References

- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0139: Agentic SLO-gated promotion (per-component release pointers; per-pack pinning).
- ADR-0131: Per-microservice flat layout.
- `microservices/observability/threat-model.md` T-I-01 + T-T-02.
- `microservices/observability/dpia.md` R-11 + R-13 + R-15 + §2.2.
- `microservices/observability/policy/tenant-isolation.md`.
- `microservices/observability/multi-region.md` (Slice B).
- `microservices/observability/legal/{transfer-register, schrems-supplementary-measures, baa-template, dpa-template, sub-processors, ropa}.md` (Slice D).
- `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/data-residency-overlay.md` (per-pack).
- Oracle Cloud Infrastructure region documentation.
- GDPR Arts. 44–50.
- EDPB Recommendations 01/2020.
- KR PIPA Art. 23-2 + Art. 28 + PIPC Notice 2020-7.
- HIPAA 45 CFR §164.530(j).
- LGPD Art. 16 + Art. 33.
- DPDPA 2023 §8(1)(g).
