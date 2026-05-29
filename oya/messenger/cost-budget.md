---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: messenger
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-messenger + ops-sre-reliability
deciders: ops-finops, axis-messenger, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0135, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/messenger/capacity-model.md
  - microservices/messenger/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (messenger µservice)

## Purpose

Track the messenger µservice's monthly cloud cost across Postgres + Valkey + S3 + WebSocket gateway + Tantivy/ES search + observability sidecars + Layer-B compute (Rust BC services), per pack region. Surface budget breach via the `oya-check-cost-budget` LEAN lane. Numbers cite OCI public pricing (2026-05-17); verify-at-deploy markers called out.

## Cost Categories

| Category | What | OCI pricing reference |
|---|---|---|
| Compute (OKE node) | Layer-B Rust services (rest + worker + app per BC) | `oracle.com/cloud/compute/pricing/` |
| Postgres (managed or self-hosted on PV) | Message + channel + thread + ACL + audit-write store | `oracle.com/database/pricing/` |
| Valkey (managed or self-hosted) | Presence + read-receipt + ephemeral cache | `oracle.com/cloud/cache/pricing/` |
| WebSocket gateway pods | Envoy + custom Rust gateway crate | bundled into compute |
| Object storage (S3-compatible) | Attachment blobs + previews + quarantine | `oracle.com/cloud/storage/object-storage/pricing/` |
| Search backend | Tantivy on PV, optional Elasticsearch cluster | `oracle.com/opensearch-service/` |
| Block storage (PV) | Postgres data + Tantivy indexes + Valkey AOF | `oracle.com/cloud/storage/block-volume/pricing/` |
| Network egress | WebSocket fanout to public clients; cross-AZ replication | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-tenant DEK envelope; attachment SSE-KMS | `oracle.com/security/key-management/pricing/` |
| Load balancer | Per-pack ingress (Envoy / Cloudflare) | `oracle.com/cloud/networking/load-balancing/pricing/` |
| Attachment scan | OPSWAT MetaDefender (SaaS) or ClamAV (self-hosted) | `metadefender.opswat.com/pricing` (SaaS path) |
| Observability sidecar | Alloy sidecar pushing to observability cluster | bundled into compute |

## Per-Component Monthly Cost (XS tier; pack-kr; M02 launch)

Per `capacity-model.md` "XS: 20 tenants, ~500k MAU, ~1k msg/sec sustained".

| Component | Replicas × type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| WebSocket gateway | 8 × VM.Standard.E4 4-core | $580 | – | $580 |
| message-stream-rest | 6 × VM.Standard.E4 2-core | $216 | – | $216 |
| message-stream-worker (search emitter) | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| channel-store-rest | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| thread-tree-rest | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| read-receipt-tracker-worker | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| file-attachment-rest | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| file-attachment-worker (scan + preview) | 6 × VM.Standard.E4 4-core | $435 | – | $435 |
| mention-router-worker | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| presence-worker | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| Postgres primary | 1 × VM.Standard.E4 8-core | $145 | $250 PV (5 TB) | $395 |
| Postgres replicas (2) | 2 × VM.Standard.E4 8-core | $290 | $500 PV | $790 |
| Valkey cluster (3 shards × primary+replica) | 6 × VM.Standard.E4 2-core | $216 | $30 PV | $246 |
| Tantivy indexers | 4 × VM.Standard.E4 4-core | $290 | $400 PV (8 TB) | $690 |
| Attachment S3 bucket | – | – | $400 hot (16 TB) + $200 cold (100 TB archive) | $600 |
| OPSWAT scan SaaS | – | $300 (5k scans/day) | – | $300 |
| KMS keyring | – | $5 | – | $5 |
| Load balancer (per-pack ingress) | – | $25 | – | $25 |
| Alloy sidecars (per pod) | absorbed | – | – | $30 |
| **XS tier total per pack region** | | **~$3300** | **~$1800** | **~$5100 / month** |

Verify-at-deploy: OCI pricing changes; reconfirm against `oracle.com/cloud/pricing/` at deploy time. Buffer 15 % for OCI rate increases + 20 % for actual-vs-forecast.

## Per-Scale-Tier Forecast

| Tier | Active connections | Msg/sec | Monthly per pack |
|---|---|---|---|
| XS (M02 launch; 20 tenants; 500k MAU) | 100k | 1k | ~$5100 |
| S (~100 tenants; 2.5M MAU) | 500k | 5k | ~$22k |
| M (~1k tenants; 25M MAU) | 5M | 50k | ~$140k |
| L (~10k tenants; 250M MAU) | 50M | 500k | ~$1.2M |

## Per-Tenant Unit Economics

| Tier | $/active user / month | $/msg | $/attachment-GB-month |
|---|---|---|---|
| XS | $0.010 | $0.000002 | $0.025 |
| S | $0.009 | $0.0000018 | $0.022 |
| M | $0.0056 | $0.0000012 | $0.018 |
| L | $0.0048 | $0.0000010 | $0.015 |

## Budget Breach Alerting

| Alert | Threshold | Action |
|---|---|---|
| Pack monthly burn > 110% forecast | sustained 7 days | FinOps review |
| Pack monthly burn > 130% forecast | sustained 3 days | engagement of council-architecture |
| Pack monthly burn > 150% forecast | sustained 1 day | Sev-3 incident |

CI lane `oya-check-cost-budget --microservice messenger` evaluates against this matrix every 24h.

## References

- `microservices/messenger/capacity-model.md`.
- `microservices/observability/cost-budget.md` (shape reference).
- OCI pricing pages (verify at deploy).
- OPSWAT MetaDefender pricing (verify at deploy).
