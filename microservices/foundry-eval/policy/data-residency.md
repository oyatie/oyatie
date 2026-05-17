---
doc_class: Policy
title: Data Residency Policy (foundry-eval)
microservice: foundry-eval
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + council-privacy + axis-foundry
deciders: ops-security, council-privacy, axis-foundry, council-architecture
related_adrs: [ADR-0024, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/foundry-eval/threat-model.md
  - microservices/foundry-eval/dpia.md
  - microservices/foundry-eval/multi-region.md
  - microservices/foundry-eval/policy/tenant-isolation.md
review_cadence: quarterly + on every new pack activation
doc_status: published
---

# Data Residency Policy (foundry-eval µservice)

## Purpose

Define where foundry-eval data lives, how cross-border transfer is governed, and how pack-pinning is enforced at runtime. Every eval-set, golden-output, replay-trace, parity-report, and per-subject DEK inherits the pack's residency rules.

## Per-Pack Residency Matrix

| Pack | Primary region | DR pair | Allowed transfer destinations | Forbidden destinations |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 (KR) | none (single-region) | none | all non-KR |
| pack-eu | OCI eu-frankfurt-1 (EU) | eu-amsterdam-1 (EU) | EU-resident only (with SCC) | non-EU |
| pack-us | OCI us-ashburn-1 (US) | us-phoenix-1 (US) | US-resident (with tenant approval) | non-US |
| pack-us-healthcare | HIPAA-eligible US region | HIPAA-eligible US DR pair | HIPAA-BAA-eligible only | non-BAA |
| pack-jp | OCI ap-tokyo-1 (JP) | none | JP-resident | non-JP |
| pack-sg | OCI ap-singapore-1 (SG) | none | SG + MAS-approved | non-SG-non-MAS |
| pack-au | OCI ap-sydney-1 (AU) | au-melbourne-1 (AU) | AU-resident + APRA-approved | non-AU |
| pack-in | OCI ap-hyderabad-1 (IN) | ap-mumbai-1 (IN) | IN-resident | non-IN |
| pack-br | OCI sa-saopaulo-1 (BR) | sa-vinhedo-1 (BR) | BR-resident | non-BR |
| pack-ae | OCI me-dubai-1 (AE) | none | AE-resident | non-AE |
| pack-ksa | OCI me-jeddah-1 (KSA) | none | KSA-resident | non-KSA |

## Enforcement

### Pack-pinning at OTel emission

Source µservices emit OTel signal with `oya-pack-id` resource attribute; Alloy collector refuses cross-pack ingest at the receiver. Replay-engine ingress validates pack-tag matches the destination cluster's pack-id.

### S3 bucket residency

Per-pack golden-output + replay-trace buckets are region-locked at the cloud-provider level; bucket policy refuses cross-region replication unless `replication-target-allowed` tag is present (set only via tenant-executed SCC import).

### KMS keyring residency

Per-pack KMS keyring is region-locked; KEKs do not leave the pack region. Cross-region KEK reuse is structurally impossible.

### LEAN-check enforcement

| Lane | Validates |
|---|---|
| `oya-check-pack-routing-conformance` | OTel emission carries correct pack-id |
| `oya-check-s3-bucket-region-conformance` | Per-pack bucket region matches policy |
| `oya-check-kms-region-conformance` | Per-pack KMS keyring region matches policy |
| `oya-check-cross-pack-replication-allowed` | Replication only when SCC import recorded |

## Cross-Border Transfer

Cross-border transfer requires:

1. **Tenant-executed SCCs** (Standard Contractual Clauses; GDPR Arts. 44-46) for GDPR-scope packs; equivalent local provisions (KR PIPA Art. 28; APPI Art. 24; PDPA Part IV; etc.) for non-GDPR.
2. **Recorded in transfer register**: `microservices/foundry-eval/legal/transfer-register.md`.
3. **Approved by council-privacy + ops-security**.
4. **Pack-routing override flag** set at the tenant level in `tenancy` µservice.

Without all four, cross-border transfer is structurally refused at the pack-pinning layer.

## DR Pair Handling

For packs with DR pairs (pack-eu, pack-us, pack-au, pack-in, pack-br):
- Primary + DR-pair regions are both within the pack's residency boundary.
- Replication primary → DR-pair is allowed by default.
- DR failover within pack is automatic; cross-pack failover is forbidden.

## EU AI Act + Pack-Specific Overlays

### pack-eu

- GDPR Arts. 44-50 govern; SCC required for non-EU transfer.
- EU AI Act Art. 17 logging: per-eval-run §17 evidence resides in EU; not exported.

### pack-us-healthcare

- HIPAA covered: PHI never leaves HIPAA-eligible BAA region.
- HIPAA §164.316(b)(2) retention 6y; cold-tier in HIPAA-eligible storage.

### pack-kr

- KR PIPA Art. 28 + 23-2 govern cross-border.
- 신용정보법: when finance capability, FSS approval required for any cross-border (effectively none).

## Verification

- Quarterly residency audit: sample 100 eval-runs + replay-traces; verify all resided in expected pack region throughout lifecycle.
- Pen-test annually: attempt cross-pack data exfiltration; should fail at OTel, S3, KMS, and ClickHouse layers.
- DPIA review on new pack activation.

## References

- ADR-0117 (Cloud-native infrastructure + data residency).
- threat-model.md (T-I-01, T-I-05, R-11).
- dpia.md (R-11, §2.2).
- multi-region.md.
- policy/tenant-isolation.md.
