---
doc_class: PolicySpec
title: Data Residency Contract (audit-chain)
microservice: audit-chain
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-audit-chain
deciders: council-privacy, ops-security, axis-audit-chain, gtm-customer-success
related_adrs: [ADR-0117, ADR-0028, ADR-0131]
related_artifacts:
  - microservices/audit-chain/threat-model.md (T-T-02, T-D-03; cross-pack threats)
  - microservices/audit-chain/dpia.md (R-06)
  - microservices/audit-chain/policy/seal-integrity.md
  - microservices/audit-chain/multi-region.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (audit-chain µservice)

## Purpose

Define where each tenant's chain lives, the cross-pack replication policy (forbidden by default; tenant-controlled export is the only exception), the cryptographic-continuity rationale, and the per-pack retention windows mapped to local law.

This document is the canonical residency artifact for SOC 2 + ISO 27001 + GDPR Arts. 44–50 + KR PIPA Art. 23-2 + HIPAA BAA + equivalent supervisory-authority review.

## Residency Model

### Default: per-pack chain locality

**Each pack has its own audit-chain.** This is stronger than the observability residency model. Reason: cryptographic continuity. A chain is constructed by walking signed roots from genesis forward; replicating a chain across packs would either require cross-pack signing (forbidden by HSM partition isolation) or break the chain (two parallel forks). Per Bominal ADR-0028 §"Chain locality": one chain per `(pack, tenant_partition)`; chains never merge.

| Pack | Primary region | HSM partition | Postgres + S3 footprint | Activated? |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-audit-hsm-1 | kr-audit-pg-1 + kr-audit-s3 | YES (M01 launch) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-audit-hsm-{1,2} | per-region | Conditional |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 (DR pair) | us-audit-hsm-{1,2} | per-region | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 + us-phoenix-1 (HIPAA-eligible) | us-hc-audit-hsm-{1,2} | per-region | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | jp-audit-hsm-1 | per-region | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-audit-hsm-1 | per-region | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-audit-hsm-{1,2} | per-region | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-audit-hsm-{1,2} | per-region | Conditional (DPDPA) |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-audit-hsm-{1,2} | per-region | Conditional (LGPD) |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-audit-hsm-{1,2} | per-region | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-audit-hsm-{1,2} | per-region | Conditional (KSA NCA cloud-residency) |

### Cross-pack replication: STRICTLY FORBIDDEN by default

No exception for "DR within pack different region" applies to cross-pack — DR is strictly intra-pack.

Specifically forbidden:
- Mimir root-publication metric series cross-pack replication.
- Postgres replica cross-pack.
- S3 cross-pack copy.
- HSM partition sharing across packs.

The forbid is **mechanically enforced**:
- Cedar policy at the emission-rest layer (`policy/data-residency-enforcement.cedar` referenced by `tenant-scope.cedar`) refuses emit() calls whose tenant's bound pack does not match the receiving emission-rest cluster's pack tag.
- HSM PKCS#11 access policy: sealing-worker in pack-X cannot reach HSM partition in pack-Y.
- S3 bucket policy: cross-pack `s3:GetObject` denied; cross-pack `s3:PutObject` denied.

CI lane: `oya-check-cross-pack-replication-forbidden` validates emission-rest, sealing-worker, Postgres replica, S3 bucket-policy configurations.

### Exception: tenant-initiated, tenant-controlled export bundle

Tenants may request a signed export bundle (per Bominal ADR-0003 §"Tenant export contract"). The bundle:
- Is constructed by query-rest in the tenant's home pack.
- Is signed by the home pack's HSM key.
- Is written to a tenant-controlled receiving bucket attested by the tenant per `cloud-secrets` BAA-equivalent attestation.
- Is NOT part of oyatie's chain; the receiving bucket is the tenant's responsibility once delivered.
- Is recorded in the chain via a `BundleExported` event (so subsequent re-exports can verify the bundle's earlier delivery).

Cross-border export of EU-resident bundles requires tenant-executed SCC.

### Exception: pack-internal DR

For DR-pair packs (pack-eu, pack-us, pack-us-healthcare, pack-au, pack-in, pack-br, pack-ae, pack-ksa), failover between the pair is within-pack and is NOT a cross-pack replication. The DR-pair HSM partitions share key material via the OCI Cloud-HSM partition-replication feature (intra-pack).

## Retention by Jurisdiction × Data Class

Retention windows are the MAX of:
- Bominal ADR-0028 default minimum (1y for AUDIT class).
- Pack legal minimum (statutory).
- Tenant DPA-contracted retention.

| Pack | Class | Statutory minimum | Default applied |
|---|---|---|---|
| pack-kr | `AUDIT` | KR PIPA Enforcement Decree Art. 30: ≥ 1y | 3y (KR-FSS sector guidance 5y for financial-services tenants) |
| pack-kr | `SENSITIVE_PIPA_ART23` | bounded; erasure on request | 1y; DSR-honoured |
| pack-eu | `AUDIT` | bounded by purpose; per ROPA | 2y default |
| pack-eu | `PII_IDENTIFYING` | GDPR Art. 17: bounded; erasure 30d | bounded; DSR-honoured |
| pack-us-healthcare | `AUDIT` + `PHI` | HIPAA §164.316(b)(2): ≥ 6y | 6y (or MAX with state-level + tenant DPA) |
| pack-us | `AUDIT` | per tenant DPA; default 3y | 3y |
| pack-jp | `AUDIT` | APPI: per purpose; default 2y | 2y |
| pack-sg | `AUDIT` | PDPA: per purpose; MAS 644 for finance 5y | 2y (default); 5y (MAS-finance) |
| pack-au | `AUDIT` | per APP 11; APRA-CPS 234 ≥ 7y for finance | 2y (default); 7y (APRA-finance) |
| pack-in | `AUDIT` | DPDPA 2023: bounded; RBI 7y for finance | 2y (default); 7y (RBI-finance) |
| pack-br | `AUDIT` | LGPD: per purpose; BACEN 5y for finance | 2y (default); 5y (BACEN-finance) |
| pack-ae | `AUDIT` | UAE PDPL: per purpose | 2y (default) |
| pack-ksa | `AUDIT` | KSA PDPL + SAMA 10y for finance | 2y (default); 10y (SAMA-finance) |
| (all packs) | `SecretKey` | rotate 90d | 90d active; ≥ retention-of-chain for retired |

The CI lane `oya-check-audit-chain-retention-conformance` validates the live retention-cascade config matches `policy/retention-matrix.yaml`.

## DSR Cascade (Data Subject Request)

Right-to-erasure honoured via `retention-cascade` BC, coordinated with `tenancy` µservice's DSR runner:

1. Tenant raises DSR on behalf of end-user (joint controllership per Art. 26).
2. DSR runner identifies all events containing the subject's `subject_hash` (computed via per-deployment salt).
3. retention-cascade-worker:
   a. Marks affected events for redaction (writes a `RedactionToken` to the chain — itself sealed).
   b. After 30d grace (regulator-mandated review window for legal-claims-defence per recital 65), hard-redacts the payload (sets payload to `<redacted>`).
   c. Preserves the leaf hash + Merkle proof of the redaction; the chain remains verifiable.
4. Audit-chain emits `RetentionApplied{tenant, subject_hash, dsr_id, applied_at, mode}` event.
5. Tenant notified within per-pack SLA (GDPR 30d, KR PIPA 10d, BR LGPD 15d).

Limitations:
- Events in statutory retention window (HIPAA 6y) are NOT erasable until window expires; DSR result returns "retention-locked until <date>" with the locked-until date.
- The chain retains the proof of redaction; verifiers can confirm "this event existed and was redacted at <ts>" without recovering the payload.

Per Bominal ADR-0028 §"Right-to-erasure with chain preservation".

## Per-Pack Overlay Sections

### pack-kr (PIPA + PIPC)

- PIPA Art. 28 storage period limitation: 3y default; 5y KR-FSS-finance.
- PIPA Art. 23-2 sensitive data cross-border: forbidden; pack-kr-pinned.
- PIPA Art. 36 right to erasure: DSR cascade with 10d SLA.
- PIPC Notice 2020-7 (cross-border-transfer notification): tenant DPA acknowledges pack-kr residency guarantee.
- KR 전자문서법 Art. 5–7: chain integrity guarantee at residency boundary.

### pack-eu (GDPR + EDPB + Schrems II)

- Arts. 44–46 SCC mechanism: required for any cross-border export bundle.
- EDPB Recommendations 01/2020 (post-Schrems II): pseudonymisation + EU-controlled KMS keys; supplementary measures documented.
- GDPR Art. 32 + 25: pseudonymisation + per-pack residency = "appropriate technical measures".
- eIDAS 910/2014 Art. 26 (AdES): HSM-Ed25519 satisfies.

### pack-us-healthcare (HIPAA)

- §164.530(j) retention ≥ 6y.
- HIPAA-eligible OCI regions only.
- BAA-required before pack-us-healthcare emission enabled.
- TPO permitted-uses; operations covers audit-chain.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/audit-chain-residency-overlay.md`. Each follows the same residency model:
- Pack-pinning enforced at emission level.
- Cross-pack replication forbidden.
- Retention per local minimum.

## Verification

- `cargo run -p oya-dev-cli -- gate validate cross-pack-replication-forbidden --microservice audit-chain` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate retention-conformance --microservice audit-chain` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate pack-pinning-conformance --microservice audit-chain` — exit 0.
- Annual residency audit: confirm each tenant's chain location matches assigned pack.
- Quarterly chaos drill: induce a cross-pack write attempt; verify rejection + alerting.

## References

- ADR-0117 (cloud-native infra).
- ADR-0131 (per-microservice flat layout).
- Bominal ADR-0028 (chain locality §).
- `microservices/audit-chain/threat-model.md`.
- `microservices/audit-chain/dpia.md` R-06.
- `microservices/audit-chain/policy/seal-integrity.md`.
- `microservices/audit-chain/multi-region.md`.
- `regional-packs/<pack>/audit-chain-residency-overlay.md` (per-pack).
- OCI Cloud-HSM region docs.
- GDPR Arts. 44–50; EDPB Recommendations 01/2020.
- KR PIPA Art. 23-2 + Art. 28 + Art. 36.
- HIPAA 45 CFR §164.530(j) + §164.316(b)(2).
