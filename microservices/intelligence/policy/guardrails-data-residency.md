---
doc_class: PolicySpec
title: Data Residency Contract
microservice: foundry-guardrails
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-foundry-guardrails
deciders: council-privacy, ops-security, axis-foundry-guardrails, gtm-customer-success
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/intelligence/threat-model.md (T-I-01, FM-13 pack-misroute)
  - microservices/intelligence/dpia.md (R-11 cross-border-misroute)
  - microservices/intelligence/policy/tenant-isolation.md
  - microservices/intelligence/multi-region.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (foundry-guardrails µservice)

## Purpose

Define which jurisdictions' prompts / outputs / rules / classifier-models / Cedar overlays / decisions live in which pack region, the cross-pack replication policy, and the legal-transfer mechanisms gating any exception.

## Residency Model

### Default: pack-pinning

Every tenant assigned a primary pack at onboarding. The tenant's invocation traffic (prompt + output) traverses foundry-runtime → foundry-guardrails in the pack region; rules + Cedar overlays + classifier-models all pack-pinned. Cross-pack movement **forbidden by default**.

| Pack | Primary region(s) | Cluster footprint | Activated? |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-postgres-1, kr-classifier-1, kr-cedar-1 | YES (M01 launch) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR) | eu-postgres-{1,2}, eu-classifier-{1,2}, eu-cedar-{1,2} | Conditional (first EU SCC) |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 (DR) | us-postgres-{1,2}, us-classifier-{1,2}, us-cedar-{1,2} | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | us-hc-postgres-1, us-hc-classifier-1, us-hc-cedar-1; isolated from non-HC pack-us | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | jp-postgres-1, jp-classifier-1, jp-cedar-1 | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-* | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-*-{1,2} | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-*-{1,2} | Conditional (DPDPA 2023) |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-*-{1,2} | Conditional (LGPD) |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-*-{1,2} | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-*-{1,2} | Conditional (NCA cloud-residency) |

### Pack-assignment routing

```text
Tenant onboarding
    ↓
gtm-customer-success: HQ jurisdiction + regulated-data declarations
    ↓
Pack-router (Cedar policy):
    - HQ → primary pack
    - Regulated-data flag (PHI, KR-FSS, EU-resident, etc.) → secondary pack force
    - Conflict: ops-legal escalation
    ↓
OpenBao assigns tenant → pack
    ↓
foundry-runtime's invocation orchestrator is configured with pack-pinned guardrails endpoint
    ↓
All invocation classification + validation traffic flows to the pack's foundry-guardrails; never cross-pack
```

Routing encoded as Cedar policy at `policy/pack-routing.cedar` (sibling foundry-runtime µservice owns the router source-of-truth).

## Cross-Pack Replication Policy

### Default: forbidden

Cross-pack replication of any tenant data is forbidden by default:

- Postgres rule-store rows: replicate within-pack only (HA primary + RR within pack).
- Cedar policy bundles: per-pack bundle; cross-pack diff allowed at AUTHORING time (rules in git authored once, deployed to multiple packs) but RUNTIME state is per-pack.
- Classifier-model artifacts: per-pack S3 bucket; Cosign-signed; cross-pack replication forbidden by IAM bucket policy.
- Per-tenant Cedar overlay fragments: per-pack instance; never replicated cross-pack.
- Decision events: emitted to per-pack AsyncAPI bus; consumed by per-pack foundry-evidence + audit-chain.
- Recording rules + alert rules: per-pack Helm values; evaluator state per-pack.

### Exception: tenant-executed SCCs (GDPR transfer)

Cross-border transfer of EU-resident data permitted only with active SCC per GDPR Arts. 44-46:
1. Active SCC at `legal/transfer-register.md` (Slice D).
2. Receiving pack has adequacy-decision (Art. 45) or equivalent safeguard.
3. Transfer-purpose specifically named (e.g., "DR failover to pack-us").
4. Audit-chain-emitted SCC-acknowledgement at moment of transfer.

### Exception: HIPAA BAA + DR failover

Covered Entity tenants in pack-us-healthcare may DR-failover us-ashburn-1 ↔ us-phoenix-1; failover intra-region from HIPAA perspective. Cross-region (us → eu) failover NOT authorised without separate tenant agreement.

### Exception: BCDR drill (controlled, scheduled)

Intra-pack DR drill permitted per `multi-region.md` (pack-eu eu-frankfurt-1 → eu-amsterdam-1; pack-us us-ashburn-1 → us-phoenix-1; etc.). Cross-pack drills not authorised.

## Per-Pack Classifier Model Variants

Classifier-model artifacts may differ per-pack for two reasons:
1. **Language coverage**: pack-kr ships Korean-tuned PII / content-safety models; pack-jp Japanese-tuned; pack-eu multilingual.
2. **Regulatory threshold alignment**: per-pack threshold matrix in `guardrail-enforcement.md`.

Model artifacts are still signed via Cosign with pack-bound public key; pack-bucket S3 cross-region replication FORBIDDEN.

## DSR (Data Subject Request) Cascade

foundry-guardrails does NOT persist prompts / outputs. DSR for prompt / output content cascades to:
- `foundry-evidence` (per its own DSR cascade) — has the persisted decision history; erases on request.
- `observability` (per its DSR) — has classifier score histograms; erases.
- `audit-chain` (per its DSR) — has audit seals; per regulatory retention obligation, audit seals are NOT erased (audit integrity > Art. 17 right-to-erasure in this context; documented in tenant DPA).

foundry-guardrails DOES persist rule definitions + Cedar overlays + classifier-model versions. These are NOT personal data; not subject to Art. 17.

## Retention by Jurisdiction × Data Class

Retention windows are MAX of: asset class default + pack legal minimum + tenant-contracted retention.

| Pack | Data class | Minimum statutory | Default applied |
|---|---|---|---|
| pack-kr | `AUDIT` (decision seals) | PIPA Enforcement Decree Art. 30: ≥ 1y | 3y (KR-FSS sector guidance) |
| pack-eu | `AUDIT` | bounded by purpose; ROPA-documented | 2y default |
| pack-us-healthcare | `AUDIT` | HIPAA §164.316(b)(2): 6y | 6y |
| pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa | `AUDIT` | per-pack PII law | 1-2y default |

Per-row retention enforced by Postgres TTL trigger; Cedar fragments + rules retained indefinitely (git history is authoritative).

CI lane `oya-governance-retention-conformance` validates retention configs.

## Per-Pack Overlay Sections

### pack-kr (KR PIPA + PIPC)

- **PIPA Art. 28**: bounded; sensitive data minimal retention.
- **PIPA Art. 23-2 (sensitive cross-border)**: forbidden by default; sensitive prompt content stays in pack-kr.
- **PIPC Notice 2020-7 (overseas-transfer notification)**: pack-kr residency guarantee in tenant DPA.
- **KR-FSS** (financial-services): audit log ≥ 5y; KMS-in-KR.

### pack-eu (GDPR + EDPB + Schrems II + EU AI Act)

- **GDPR Arts. 44-46**: SCC-only; Schrems-II supplementary measures (pseudonymisation + EU-resident KMS).
- **EDPB Recommendations 01/2020**: supplementary measures at `legal/schrems-supplementary-measures.md`.
- **EU AI Act Art. 10 (data-governance)**: classifier training data provenance documented in model-cards; never includes tenant prompts.
- **EU AI Act Art. 12 (record-keeping)**: 5y retention for high-risk-AI logs (longer than GDPR baseline; controls).

### pack-us-healthcare (HIPAA)

- **45 CFR §164.530(j)**: ≥ 6y retention for audit-relevant data.
- **HIPAA-eligible regions only**: OCI us-ashburn-1 + us-phoenix-1.
- **BAA-required**: BAA on file before pack-us-healthcare ingest enabled.
- **§164.502(a) Permitted Uses**: Operations only.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlay at `regional-packs/<pack>/foundry-guardrails-data-residency-overlay.md`.

## Verification

- `oya gate validate retention-conformance` — exit 0.
- `oya gate validate pack-routing-conformance` — exit 0.
- `oya gate validate cross-region-transfer-allowed-only-with-scc` — exit 0.
- Annual residency audit.
- Quarterly chaos drill: induce cross-pack write attempt; verify rejection + alerting (FM-13).

## References

- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0139; ADR-0131; ADR-0140 (retired per ADR-0145).
- `microservices/intelligence/threat-model.md`.
- `microservices/intelligence/dpia.md`.
- `microservices/intelligence/policy/tenant-isolation.md`.
- `microservices/intelligence/multi-region.md`.
- `microservices/observability/policy/data-residency.md` (sibling shape).
- OCI region docs; GDPR Arts. 44-50; KR PIPA Art. 23-2 + Art. 28; HIPAA 45 CFR §164.530(j); LGPD Art. 16; DPDPA 2023 §8(1)(g).
