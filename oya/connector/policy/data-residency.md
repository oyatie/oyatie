---
doc_class: PolicySpec
title: Data Residency Contract
microservice: connector
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: council-privacy + axis-integration
deciders: council-privacy, ops-security, axis-integration, gtm-customer-success
related_adrs: [ADR-0117, ADR-0244, ADR-0248, ADR-0273]
related_artifacts:
  - microservices/connector/threat-model.md
  - microservices/connector/dpia.md
  - microservices/connector/multi-region.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract — connector

## Purpose

Define which jurisdictions' tenant data (OAuth grants, webhook payloads, DLQ entries, audit chains, telemetry) lives in which region. Canonical residency artifact reviewed by EU DPAs (GDPR Arts. 44–50), KR PIPC (PIPA Art. 28 + 23-2), HIPAA Covered Entity counsel (BAA), and equivalent supervisory authorities.

## Residency model — default pack-pinning

Every tenant assigned a primary pack at onboarding. All connector-owned data for that tenant lives in the pack's region-pinned infrastructure. **Cross-pack movement forbidden by default.**

| Pack | Primary region | DR pair | Activated? |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | ap-tokyo-1 (read-replica only) | YES (M01 launch tenant) |
| pack-eu | eu-frankfurt-1 | eu-amsterdam-1 | Conditional (first EU tenant + SCC signed) |
| pack-us | us-ashburn-1 | us-phoenix-1 | Conditional |
| pack-us-healthcare | us-ashburn-1 (HIPAA-eligible) | isolated DR | Conditional (post-BAA) |
| pack-jp | ap-tokyo-1 | ap-osaka-1 | Conditional |
| pack-sg | ap-singapore-1 | ap-melbourne-1 | Conditional (PDPA) |
| pack-au | ap-sydney-1 | ap-melbourne-1 | Conditional |
| pack-in | ap-hyderabad-1 | ap-mumbai-1 | Conditional (DPDPA 2023) |
| pack-br | sa-saopaulo-1 | sa-vinhedo-1 | Conditional (LGPD) |
| pack-ae | me-abudhabi-1 | me-dubai-1 | Conditional |
| pack-ksa | me-jeddah-1 | me-riyadh-1 | Conditional (NCA cloud-residency) |
| pack-cn | cn-shanghai-1 (Alibaba Cloud) | cn-beijing-1 | Conditional (PIPL-2021) |

Activation triggers re-review of this doc + per-pack threat-model overlay + DPIA overlay.

## Data classes stored in connector

| Data class | Storage | Residency rule |
|---|---|---|
| OAuth grant metadata (no raw tokens) | Postgres `connector.oauth_grants` | Pinned to tenant's pack region |
| OAuth refresh tokens | OpenBao (cloud-secrets µservice) | Pinned to pack region; KMS HSM-backed |
| Webhook signing secrets | OpenBao | Pinned |
| Webhook payload digests | Postgres `connector.audit_events` | Pinned |
| Webhook full payloads (DLQ only) | Encrypted PG blob; 7d default TTL | Pinned |
| Connector action audit events | Audit chain (Merkle-sealed) | Pinned + replicated to DR pair |
| Catalog records (vendor-owned, not tenant-data) | Global ElasticSearch | NOT subject to residency (vendor metadata) |
| Telemetry (traces, metrics, logs) | Mimir / Loki / Tempo (observability µservice) | Per ADR-0117 jurisdiction routing |

## Cross-border transfer mechanisms

Cross-pack transfers are forbidden by default. Exceptions require:

| Pack pair | Mechanism | Status |
|---|---|---|
| EU ↔ US | EU-US Data Privacy Framework (Schrems II remedy) | Conditional |
| KR ↔ JP | PIPA Art. 28 transfer-equivalent assessment | Conditional |
| KR ↔ US | PIPA Art. 23-2 explicit consent + adequacy | Conditional |
| CN ↔ anything | PIPL Art. 38-40 specific contract + filing | Restricted |

All cross-pack transfers emit `CrossPackTransferRequested` + `CrossPackTransferApproved` (or `Denied`) audit events.

## Webhook URL DNS placement

Per ADR-0273, per-tenant DNS for webhook endpoints:
- Pack-kr tenant: `hooks.<tenant-id>.kr.oyatie.com` → resolves to ap-seoul-1 ingress
- Pack-eu tenant: `hooks.<tenant-id>.eu.oyatie.com` → eu-frankfurt-1
- Pack-us tenant: `hooks.<tenant-id>.us.oyatie.com` → us-ashburn-1

ECH config-id per ADR-0253 served on every Tier-0 ingress; HTTPS RR includes `ech=`.

## Vendor egress endpoints

When connector adapter calls a vendor (e.g., Salesforce), the call originates from the tenant's pack region. Vendor receives traffic from the pack-region IP range.

## Retention

| Data | Retention | Driver |
|---|---|---|
| OAuth grant metadata | Until grant revoked + 30d | Audit |
| OAuth refresh tokens | Until grant revoked | Operational |
| Webhook signing secrets | Until wiring deleted | Operational |
| Audit chain | Per pack (KR 5yr; EU 6yr; US-HC 6yr) | Compliance |
| DLQ entries | 7d default; max 30d | Tenant policy |
| Telemetry | Per observability residency.md | Per ADR-0117 |

## References

- ADR-0117 jurisdiction-code routing
- ADR-0244 tenant as universal scoping primitive
- ADR-0248 cellular architecture
- ADR-0273 per-tenant DKIM/SPF/DMARC + DNS
- ADR-0276 backup portability (GDPR Art. 20)
