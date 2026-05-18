---
doc_class: CapacityModel
template_id: TPL-CAPACITY-MODEL
microservice: drive
status: Accepted
date: 2026-05-17
owner_team: axis-drive + ops-sre-reliability + finops
related_adrs: [ADR-0117, ADR-0130, ADR-0131, ADR-0133, ADR-DRIVE-0001]
doc_status: published
---

# Capacity Model — drive µservice

## Purpose

Quantify per-cell + per-tenant capacity envelopes; drive HPA + autoscale + cost budget. Identifies the bottleneck dimension for each scale-out trigger.

## Per-tenant capacity (medium tenant baseline)

- 1,000 active users.
- 1M files stored.
- 50TB raw bytes.
- 10k file-list/s baseline; burst 50k/s.
- 200 uploads/s baseline; burst 1k/s.
- 2k downloads/s baseline; burst 20k/s.
- 100 sync delta sessions/s baseline; burst 1k/s.
- 50 preview renders/s baseline; burst 500/s.
- 30k virus scans/day baseline; burst 200k/day.

## Per-cell capacity

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active tenants (medium) | 50k | 500k | Postgres connection pool > 70% |
| Files stored | 1B | 10B | object-store list latency > 100ms |
| Bytes stored | 5PB | 50PB | object-store free-space < 30% |
| Uploads/s | 500 | 5k | upload-rest p99 > 100ms |
| Downloads/s | 5k | 50k | download-rest p99 > 200ms; CDN-miss > 30% |
| Search QPS | 500 | 5k | search-rest p99 > 200ms |
| Sync delta/s | 100 | 1k | sync worker queue depth > 60s |
| Share-link mints/s | 1k | 10k | share-link-rest p99 > 100ms |
| Share-link verifies/s | 5k | 50k | verify p99 > 50ms (Argon2id CPU) |
| Preview renders/s | 20 | 200 | preview worker queue > 60s |
| Virus scans/s | 50 | 500 | scan worker queue > 60s |
| OpenBao Transit calls/s | 1k | 10k | KMS rate-limit |

## Bottleneck identification

| Component | Bottleneck dimension | Mitigation when hit |
|---|---|---|
| Postgres metadata | connection pool + index hot rows | per-tenant logical shard; read replicas |
| Redis upload session + sync cache | memory + connection count | cluster scale-out; eviction tuning |
| Object store (Garage) | per-cell free-space + cross-cell rebuild backlog | cell scale-out + rebalance |
| Meilisearch | per-index size + query CPU | per-tenant index sharding |
| Preview workers | CPU (LibreOffice + ffmpeg are CPU-bound) | HPA on queue depth; spot instance burst |
| Virus-scan workers | CPU (ClamAV in-memory scan) | HPA + per-tenant quota |
| Share-link verify | CPU (Argon2id KDF; 50ms / verification) | reverse-proxy CPU + HPA |
| CDN egress | bandwidth + per-edge cache hit-ratio | multi-CDN failover; per-tenant pinning |

## Scale-out policy

- Kubernetes HPA: rest pods scale on CPU > 70%; min 3, max 100.
- HPA: worker pods scale on queue depth metric (60s lookback); min 2, max 50 per worker class.
- Postgres: per-tenant logical shard; cross-cell replication-factor 3 with Patroni.
- Redis: cluster mode; per-tenant key prefix; eviction policy `allkeys-lru`.
- Object store: per-cell deployment; per-tenant prefix; replication-factor 3.
- Pre-warmed pool: 10 standby pods; cold-start ≤ 700ms.

## Cross-region

- M02 launches in pack-kr.
- M03 expands to pack-eu + pack-us + pack-us-healthcare.
- M04 expands to pack-jp + pack-sg.
- M05 expands to pack-au + pack-in + pack-br + pack-ae + pack-ksa.

## Sharding

- Files partitioned by `(tenant_id, file_id_prefix_4)`.
- Folders partitioned by `tenant_id`.
- Sync sessions partitioned by `session_id`.
- Share-links partitioned by `link_id`.
- Audit records replicated to audit-chain µservice (out-of-scope here).

## Capacity headroom guardrails

| Component | Steady-state utilisation | Capacity alarm | Capacity scale-out |
|---|---|---|---|
| Object store free | ≤ 50% | > 70% | > 80% |
| Postgres connection pool | ≤ 50% | > 70% | > 80% |
| Redis memory | ≤ 60% | > 75% | > 85% |
| Worker pool depth | ≤ 30% | > 60% | > 80% |

## References

- ADR-0117 (cloud-native infrastructure).
- ADR-0130 (SLO-gated promotion).
- ADR-0131 (per-microservice flat layout).
- ADR-0133 (industry conformance).
- ADR-DRIVE-0001 (object-storage substrate selection).
- `microservices/drive/PRD.md` §"Horizontal Scalability".
- `microservices/drive/cost-budget.md`.
- `microservices/drive/multi-region.md`.
