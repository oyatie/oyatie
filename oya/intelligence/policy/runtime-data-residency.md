---
doc_class: PolicySpec
title: Data Residency Contract
microservice: foundry-runtime
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-foundry-runtime
deciders: council-privacy, ops-security, axis-foundry-runtime, gtm-customer-success
related_adrs: [ADR-0025, ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/intelligence/threat-model.md (T-I-01, T-T-02, cross-region threats)
  - microservices/intelligence/dpia.md (R-11)
  - microservices/intelligence/policy/runtime-isolation.md
  - microservices/intelligence/multi-region.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (foundry-runtime µservice)

## Purpose

Define which jurisdictions' tenant data (sessions, capability descriptors, invocation lifecycle records) lives in which runtime cluster, the cross-pack replication policy, and the legal-transfer mechanisms that gate any exception. Canonical residency artifact reviewed by EU DPAs (GDPR Arts. 44–50), KR PIPC (PIPA Art. 28 + Art. 23-2), HIPAA tenants' Covered Entity counsel, and EU AI Act notified bodies (data-governance under Art. 10) at first-tenant onboarding.

## Residency Model

### Default: pack-pinning

Every tenant is assigned a primary pack at onboarding. The tenant's sessions + invocation records + capability descriptor mirror live in the pack's region-pinned cluster. Cross-pack movement is **forbidden by default**.

| Pack | Primary region | Runtime cluster footprint | Activated? |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-runtime-1, kr-redis-cluster-1, kr-postgres-1 | YES (M01 launch) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-runtime-{1,2}, eu-redis-{1,2}, eu-postgres-{1,2} | Conditional |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 (DR pair) | us-runtime-{1,2}, ... | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | us-hc-runtime-1, ...; isolated from non-HC pack-us | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | jp-runtime-1, ... | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-runtime-1, ... | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-runtime-{1,2}, ... | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-runtime-{1,2}, ... | Conditional (DPDPA 2023) |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-runtime-{1,2}, ... | Conditional (LGPD) |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-runtime-{1,2}, ... | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-runtime-{1,2}, ... | Conditional (KSA NCA cloud-residency) |

### Pack-assignment routing

```text
Tenant onboarding
    ↓
gtm-customer-success collects HQ jurisdiction + regulated-data declarations
    ↓
Pack-router (Cedar policy in cloud-iac):
    - HQ jurisdiction → primary pack
    - Regulated-data flag (PHI, KR-FSS, EU-resident, etc.) → may force secondary pack
    - High-risk EU AI Act use case → pack-eu mandatory + notified body engagement
    - Conflict: ops-legal escalation
    ↓
OpenBao assigns tenant → pack
    ↓
Workflow Studio + workload µservice OTel configured with pack-pinned endpoints
    ↓
All session-state + capability mirror + invocation records flow to pack cluster; never cross-pack
```

### tenant_scope influences capacity, not residency

Pack-pinning is invariant per tenant; `tenant_scope` (trial / production / sandbox / internal) affects capacity allocation within the pack.

## Cross-Pack Replication Policy

### Default: forbidden

Cross-pack replication of any tenant data is forbidden by default. Specifically:

- Session-state (Valkey + Postgres): replicate within-pack only.
- Capability descriptor mirror (Postgres): pulled from foundry-supervisor with pack-pinned scope; cross-pack capability templates only for `tenant:oya-system` (oyatie-owned templates).
- Invocation lifecycle records (Postgres): replicate within-pack only.
- Audit-chain seals: replicate within-pack (each pack has its own audit-chain instance).
- Cedar policies + Helm values + recording rules: git-versioned, declarative (zero RPO replication).

### Exception: tenant-executed SCCs (GDPR transfer mechanism)

Cross-border transfer of EU-resident data permitted only with active SCC per GDPR Arts. 44–46 + EDPB-recommended supplementary technical measures. Requires:

1. Active SCC at `microservices/intelligence/legal/transfer-register.md`.
2. Receiving-pack jurisdiction has adequate decision (Art. 45) or equivalent safeguard.
3. Transfer-purpose limited to specifically-named processing (e.g., "DR failover to pack-us").
4. Audit-chain-emitted SCC-acknowledgement at moment of transfer.

### Exception: HIPAA BAA + DR failover (intra-pack)

Covered Entity tenants in pack-us-healthcare may have DR failover us-ashburn-1 ↔ us-phoenix-1 (both HIPAA-eligible OCI regions). Cross-pack failover is NOT authorised.

### Exception: BCDR exercise (controlled, scheduled, intra-pack only)

For BCDR validation, intra-pack DR drills are permitted. Cross-pack BCDR is not authorised.

## Capability Descriptor Routing

Beyond session/lifecycle data, capability descriptors observe:

- `tenant:oya-system` descriptors (oyatie-owned templates) replicate cross-pack as needed (they are not tenant data; they are oyatie's product surface).
- Tenant-authored descriptors (`tenant:<hashed-id>:*`) pack-pinned; no cross-pack mirroring.
- High-risk EU AI Act capability descriptors carry `eu_ai_act_annex_iii=true` flag; the runtime in non-EU packs refuses to instantiate them for EU-resident tenants (defence-in-depth on Annex III locus-of-control).

## Tenant Tagging by Jurisdiction

In addition to `X-Scope-OrgID`, runtime invocation records carry jurisdiction labels:

```text
record_label:
  jurisdiction: kr | eu | us | us-hc | jp | sg | au | in | br | ae | ksa
  pack:         pack-kr | pack-eu | pack-us | ...
  data_class:   one of class taxonomy per Bominal ADR-0028
  eu_ai_act_class: minimal_risk | limited_risk | high_risk_annex_iii | prohibited (when applicable)
```

- `jurisdiction` set by the dispatching workload µservice; tampering detected by OTel collector enforcement layer.
- Postgres per-tenant retention policy keys on `(tenant, jurisdiction, data_class)`.
- `pack` label redundant with jurisdiction for routing convenience.

## Retention by Jurisdiction × Data Class

| Pack | Data class | Minimum statutory | Default applied |
|---|---|---|---|
| pack-kr | `BEHAVIORAL_TENANT_PRODUCT` (sessions) | KR commercial 5y | 5y aligned |
| pack-kr | `SENSITIVE_PIPA_ART23` | PIPA Art. 28 bounded | 1y default; honour erasure |
| pack-kr | `AUDIT` (invocation lifecycle) | PIPA ED Art. 30 ≥1y | 3y aligned (KR-FSS) |
| pack-eu | `PII_IDENTIFYING` | GDPR Art. 17 bounded | bounded; honour erasure 30d |
| pack-eu | `AUDIT` | bounded by purpose | 2y default |
| pack-us-healthcare | `PHI` | HIPAA per state | MAX(HIPAA 6y, state, tenant DPA) |
| pack-us-healthcare | `AUDIT` | HIPAA §164.316(b)(2) 6y | 6y |
| pack-jp | `PII_IDENTIFYING` | APPI bounded | bounded |
| pack-au | `PII_IDENTIFYING` | Privacy Act APP 11 + 12 | bounded |
| pack-in | `PII_IDENTIFYING` | DPDPA §8(1)(g) | bounded |
| pack-br | `PII_IDENTIFYING` | LGPD Art. 16 | bounded |
| (all packs) | `SECRET` | ISO 27001 A.5.17 rotation | 30d API, 90d signing |

CI lane `oya-governance-retention-conformance` validates Postgres + Valkey retention configs against this table.

## DSR (Data Subject Request) Cascade

Right-to-erasure honoured via `oya-dsr-cascade-runner` skill:

1. Tenant raises DSR for end-user (joint controllership per Art. 26).
2. DSR runner identifies end-user identifiers (user-id hash, IP hash patterns).
3. Session-state worker:
   - Valkey: SCAN per-tenant prefix; identify affected session keys; delete.
   - Postgres: per-tenant session_mutation_log; soft-delete with 30d grace; hard-delete after grace.
4. Audit-chain seal: `dsr_executed{tenant, subject_hash, removed_session_count, removed_record_count, timestamp}`.
5. Tenant notified within 30d SLA per GDPR; pack-specific shorter SLAs (KR 30d, BR 15d, EU 30d) honoured.

Limitations (per DPIA R-08):
- Data older than retention window already deleted before DSR processed.
- Cross-pack data not in scope (cross-pack forbidden by default).

## Per-Pack Overlay

### pack-kr (KR PIPA + PIPC)

- PIPA Art. 28 storage limitation: bounded.
- PIPA Art. 23-2 sensitive cross-border: forbidden by default.
- PIPC Notice 2020-7 overseas-transfer notification: pack-kr residency in tenant DPA.
- KR-FSS sector: audit retention ≥5y; encrypted at rest with KR-resident KMS.

### pack-eu (GDPR + EU AI Act + EDPB + Schrems II)

- GDPR Arts. 44–46: SCC-only; Schrems-II-compliant supplementary measures.
- EDPB Recommendations 01/2020: `legal/schrems-supplementary-measures.md`.
- GDPR Arts. 32 + 25: pseudonymisation + EU-resident KMS.
- EU AI Act Art. 10 (data governance): pack-eu primary for EU-resident high-risk capabilities.

### pack-us-healthcare (HIPAA)

- 45 CFR §164.530(j): ≥6y retention.
- HIPAA-eligible OCI regions only.
- BAA-required pre-ingest.
- TPO scope only.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/foundry-runtime-residency-overlay.md`.

## Verification

- `cargo run -p oya-dev-cli -- gate validate retention-conformance --microservice foundry-runtime` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate pack-routing-conformance` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate cross-region-transfer-allowed-only-with-scc` — exit 0.
- Annual residency audit.
- Quarterly chaos drill: induce cross-pack write attempt; verify rejection + alerting.

## References

- ADR-0025; ADR-0117; ADR-0139; ADR-0131.
- `microservices/intelligence/threat-model.md` T-I-01 + T-T-02.
- `microservices/intelligence/dpia.md` R-11 + R-13 + R-15 + §2.2.
- `microservices/intelligence/policy/runtime-isolation.md`.
- `microservices/intelligence/multi-region.md`.
- `microservices/intelligence/legal/{transfer-register, schrems-supplementary-measures, baa-template, dpa-template, sub-processors, ropa}.md`.
- `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/foundry-runtime-residency-overlay.md`.
- OCI region documentation.
- GDPR Arts. 44–50; EDPB Recommendations 01/2020.
- EU AI Act Art. 10.
- KR PIPA Art. 23-2 + Art. 28 + PIPC Notice 2020-7.
- HIPAA 45 CFR §164.530(j).
- LGPD Art. 16 + Art. 33; DPDPA 2023 §8(1)(g).
