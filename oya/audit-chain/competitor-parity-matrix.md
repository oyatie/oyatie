---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix (audit-chain)
microservice: audit-chain
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-audit-chain + council-architecture
deciders: axis-audit-chain, council-architecture, gtm-customer-success
related_adrs: [ADR-0123, ADR-0028, ADR-0003]
related_artifacts:
  - microservices/audit-chain/PRD.md (§Competitive Benchmark)
  - /specs/hyperscaler-gates.json (HG-AUDIT gate)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (audit-chain µservice)

## Purpose

Parity comparison vs the industry-leading cloud-native audit-trail products. Drives `oya-governance-hyperscaler-maturity-claims` (ADR-0123 HG-AUDIT) and informs gtm-customer-success on permissible sales claims.

## Competitor Set

| Competitor | Product | Differentiator | Primary source |
|---|---|---|---|
| AWS | CloudTrail + CloudTrail Lake | Per-account; integrity-validated digest chain | `docs.aws.amazon.com/awscloudtrail/` |
| Google Cloud | Cloud Audit Logs | Admin/Data/Policy/System audit; Cloud KMS-tied integrity | `cloud.google.com/logging/docs/audit` |
| Azure | Activity Log + Microsoft Defender for Cloud | Resource + management plane | `learn.microsoft.com/en-us/azure/azure-monitor/essentials/activity-log` |
| IBM | IBM Cloud Activity Tracker + IBM Verify Trust | Identity audit | `cloud.ibm.com/docs/activity-tracker` |
| Splunk | Splunk Enterprise + Security Cloud | SIEM with hash-chain audit option | `docs.splunk.com` |
| Datadog | Audit Logs | User-action + resource | `docs.datadoghq.com/account_management/audit_logs/` |
| Sumo Logic | Audit Index | Retention + correlation | `help.sumologic.com` |
| Hashicorp Vault | Audit devices | Append-only audit log; HMAC integrity | `developer.hashicorp.com/vault/docs/audit` |

## Feature Parity Matrix

### Integrity model

| Capability | oyatie | AWS CT | GCP CAL | Azure | IBM | Splunk | Datadog | Vault |
|---|---|---|---|---|---|---|---|---|
| **Cryptographic per-event Merkle proof** | ✅ | partial (per-batch digest) | partial (Cloud KMS sign batch) | ❌ | partial | optional HMAC | ❌ | ❌ |
| **HSM-rooted Ed25519 signing** | ✅ | ❌ (SHA-256 chain) | partial (Cloud KMS) | partial | ❌ | ❌ | ❌ | ❌ |
| **Tenant-independent verification (offline-verifiable)** | ✅ | ❌ (AWS trust required) | ❌ (Google trust required) | ❌ | ❌ | partial | ❌ | partial (HMAC key shared) |
| **eIDAS AdES-compatible** | ✅ | ❌ | partial | ❌ | ❌ | ❌ | ❌ | ❌ |
| **RFC-6962 Merkle shape** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Three-channel root publication (S3 + Mimir + GitHub)** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

### Tenant isolation + residency

| Capability | oyatie | AWS CT | GCP CAL | Azure | IBM | Splunk | Datadog | Vault |
|---|---|---|---|---|---|---|---|---|
| **Per-pack chain locality (cross-pack forbidden)** | ✅ | partial (per-region) | partial (per-region) | partial | partial | n/a | partial | n/a |
| **Multi-region (11 packs)** | ✅ (conditional) | ✅ (AWS regions) | ✅ (GCP regions) | ✅ | ✅ | n/a | partial | n/a |
| **Tenant-controlled export bundle** | ✅ | partial (S3 export) | partial | partial | ❌ | ❌ | partial | ❌ |
| **Auditor JIT scoped token** | ✅ | ❌ | partial (IAM) | partial | ❌ | partial | ❌ | partial |

### Retention + compliance

| Capability | oyatie | AWS CT | GCP CAL | Azure | IBM | Splunk | Datadog | Vault |
|---|---|---|---|---|---|---|---|---|
| **WORM Object Lock (Compliance mode)** | ✅ | ✅ (S3) | partial | partial | partial | partial | ❌ | ❌ |
| **Per-pack retention matrix (HIPAA 6y / KR 3y / etc.)** | ✅ | ✅ (per-account) | partial | partial | partial | partial | partial | ❌ |
| **DSR cascade with Merkle proof of redaction** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | partial | ❌ |
| **HIPAA BAA** | conditional | ✅ | ✅ | ✅ | ✅ | partial | ✅ | partial |
| **KR PIPA + 전자문서법 compliance** | conditional | partial | partial | partial | partial | partial | partial | partial |

### Operations + integrations

| Capability | oyatie | AWS CT | GCP CAL | Azure | IBM | Splunk | Datadog | Vault |
|---|---|---|---|---|---|---|---|---|
| **Multi-language SDK (Rust/TS/Py/Go/JVM/.NET)** | M01: Rust + TS; M01+1: Py; M02: Go/JVM; M03: .NET | ✅ (AWS SDKs) | ✅ (GCP) | ✅ | ✅ | ✅ | ✅ | partial |
| **Cedar / Rego / OPA policy** | ✅ Cedar | ❌ (IAM) | partial (IAM) | partial (RBAC) | ❌ | ❌ | ❌ | ❌ |
| **SPIFFE identity-bound emission** | ✅ | partial (IAM) | partial | partial | ❌ | ❌ | ❌ | ❌ |
| **Idempotency-key dedup** | ✅ | partial | partial | partial | ❌ | partial | ❌ | ❌ |
| **gRPC + REST + SDK** | ✅ | partial (REST) | ✅ | partial | partial | partial | partial | partial |
| **Self-hosted (no vendor lock)** | ✅ | ❌ (SaaS) | ❌ (SaaS) | ❌ (SaaS) | ❌ (SaaS) | ✅ | ❌ | ✅ |

## Quantitative Performance Parity

| Metric | oyatie target | AWS CT reference | GCP CAL | Azure | Notes |
|---|---|---|---|---|---|
| Emission latency p99 | ≤ 100ms | ~50ms (CloudTrail) | ~100ms | ~150ms | parity |
| Seal latency p99 | ≤ 1s | ~5min (batched digest) | ~5min | n/a | **oyatie advantage** (1s vs 5min) |
| Verification latency p99 | ≤ 200ms | n/a (AWS-mediated) | n/a | n/a | **oyatie unique** (tenant-side offline) |
| Per-event Merkle proof retrievable | ✅ | ❌ | ❌ | ❌ | **oyatie unique** |
| Sustained emission per cluster | ≥ 50k events/s | ~100k events/s/region (managed) | ~50k events/s | ~30k events/s | parity |

## Key Parity Gaps to Close (oyatie → industry leader)

| # | Gap | Owner | Target close |
|---|---|---|---|
| 1 | SaaS-vs-self-hosted operational maturity (AWS / GCP run audit-trail as a managed service; we self-host) | ops-sre-reliability | M02–M03 (operational maturity); permanent self-host model |
| 2 | Multi-language SDK breadth (Py / Go / JVM not until M01+1 to M02) | axis-audit-chain | M02 |
| 3 | Native cloud-provider integration (CloudTrail capturing AWS API calls automatically) | n/a | not a goal — oyatie owns its own emission contract via Bominal ADR-0003 |
| 4 | Long-history search performance at scale (Splunk excels at SIEM-shape queries) | axis-audit-chain | M03 (forensic-query optimisation) |

## Key oyatie Differentiators (NOT in any competitor)

1. **Per-event Merkle proof + offline tenant verification**: tenants verify without trusting oyatie.
2. **HSM-rooted Ed25519 + eIDAS AdES posture**: load-bearing for EU + KR document-integrity laws; competitors use SHA-256 digest chains.
3. **Three-channel root publication (S3 + Mimir + GitHub)**: transparency-log substitute; tampering one channel without the others is deterministically detected.
4. **Per-pack chain locality with cryptographic continuity**: chain doesn't fork or merge across packs; clean residency model.
5. **DSR cascade with Merkle proof of redaction**: GDPR Art. 17 + chain integrity simultaneously.
6. **Cedar policy + SPIFFE identity binding**: granular policy that competitors' IAM cannot express.

## Claim-Boundary Rules

Permitted (citation-bounded):
- ✅ "Per-event Merkle inclusion proofs returned synchronously" (unique).
- ✅ "HSM-rooted Ed25519 + eIDAS AdES" (unique on AdES + HSM combination).
- ✅ "Tenant-independent offline verification" (unique).
- ✅ "Per-pack chain locality with no cross-pack replication" (unique).

Forbidden (per ADR-0123):
- ❌ "Faster than CloudTrail" (parity claim without published benchmark).
- ❌ "Cheaper than CloudTrail" (depends on workload; HSM cost is real).
- ❌ "More secure than AWS" (broad-spectrum claim; not a comparable measurement).

## Bi-Annual Refresh

Same shape as observability — `gtm-customer-success` surveys; `axis-audit-chain` updates with citations; `ops-sre-reliability` re-runs quantitative benchmarks in staging; `council-architecture` reviews claim-boundary rules; gtm publishes.

## References

- `microservices/audit-chain/PRD.md` §Competitive Benchmark.
- `/specs/hyperscaler-gates.json` HG-AUDIT.
- ADR-0123; ADR-0028; ADR-0003.
- Competitor docs as cited inline.
- RFC 6962 (Certificate Transparency) — reference for Merkle-tree-shape comparison.
- eIDAS 910/2014 (AdES) — reference for cryptographic-signature-grade comparison.
- KR 전자문서법 — reference for electronic-document-integrity comparison.
