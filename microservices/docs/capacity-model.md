---
doc_class: CapacityModel
template_id: TPL-CAPACITY-MODEL
microservice: docs
status: Accepted
date: 2026-05-17
owner_team: axis-docs + ops-sre-reliability
related_adrs: [ADR-0117, ADR-0131, ADR-DOCS-0001, ADR-DOCS-0003]
doc_status: published
---

# Capacity Model — docs µservice

## Purpose

Model per-cell capacity envelope, scale-out triggers, and headroom posture. Drives Helm `replicas` / HPA / Postgres replica configuration + gVisor pool sizing + finops cost budget.

## Demand model

### Per-tenant demand (medium tenant, 1k active users)

| Workload | Rate | Notes |
|---|---|---|
| Document opens | 100 RPS | reads dominate (10:1 read:write) |
| Edit-ops | 50 RPS | CRDT ops; bursty during collab |
| Concurrent editor sessions | 100 | open documents per tenant baseline |
| Comments | 10 RPS | indexed insert |
| Suggestions | 5 RPS | state-machine insert |
| Share grants | 1 RPS | low-frequency |
| Exports (PDF/DOCX) | 2 / s | bursty |
| Imports (DOCX) | 0.5 / s | migration-tier; bursty |
| Attachment uploads | 5 / s | 100KB-10MB |
| Embed-refreshes | 20 / s | per-doc-open fan-out average |
| AI writing-assist (T1) | 5 / s | tenant-policy bound |
| Auto-summary (T2) | 0.1 / s | nightly batches |

### Per-cell aggregate (1M active documents baseline)

| Workload | Aggregate rate (steady-state p50) |
|---|---|
| Document opens | 100k RPS |
| Edit-ops | 50k RPS |
| Concurrent editor sessions | 50k |
| Comments | 10k RPS |
| Suggestions | 5k RPS |
| Share grants | 1k RPS |
| Exports | 2k / s |
| Imports | 500 / s |
| Attachment uploads | 5k / s |
| Embed-refreshes | 20k / s |
| AI writing-assist | 5k / s |

## Capacity envelope (per cell)

| Dimension | Baseline | Max | Scale-out trigger |
|---|---|---|---|
| Active documents | 1M | 10M | Postgres connection pool > 70% |
| Concurrent editor sessions | 50k | 500k | WS gateway lease pressure |
| Edits/s | 5k | 50k | document-store rest p99 > 100ms |
| Comments/s | 500 | 5k | comments-rest p99 > 100ms |
| Exports concurrent | 50 | 500 | gVisor worker queue > 5min |
| Imports concurrent | 20 | 200 | worker queue > 5min |
| Attachment uploads/s | 100 | 1k | S3 PUT p99 > 1s OR ClamAV queue > 1min |
| Embed-refresh/s | 500 | 5k | embed-resolver worker queue > 60s |
| AI writing-assist/s | 100 | 1k | foundry-runtime rate-limit |
| Postgres replica lag | < 5s | 30s | scale-out |
| Valkey CRDT spool memory | < 60% | 80% | shard split |

## Substrate sizing

### Postgres (document-metadata; 3-replica HA + per-tenant RLS)

| Param | Baseline | Max |
|---|---|---|
| OCPUs (primary) | 16 | 64 |
| OCPUs (each replica) | 8 | 32 |
| Memory (primary) | 128 GB | 512 GB |
| Persistent block | 4 TB | 32 TB |
| max_connections | 300 per replica | 1500 |
| WAL retention | 60 GB | 400 GB |

### S3 (content blobs + attachments; per-pack bucket)

| Param | Baseline | Max |
|---|---|---|
| Storage tier | standard | standard + cold |
| Object Lock | enabled for held docs | enabled |
| Lifecycle: archive after 90d | enabled | enabled |
| Per-tenant prefix | yes | yes |
| Cross-AZ replication | yes | yes |
| Cross-region (intra-pack DR) | yes | yes |
| Cross-region (cross-pack) | NO | NO |

### Valkey (collab presence + CRDT op spool + cache)

| Param | Baseline | Max |
|---|---|---|
| Shards | 5 | 25 |
| Per-shard memory | 16 GB | 64 GB |
| Per-tenant key prefix | yes | yes |
| Eviction policy | volatile-lru | volatile-lru |
| Persistence (CRDT spool) | AOF every-sec | AOF every-sec |
| Presence TTL | 60s | 60s |
| Cache TTL | 5min ± 30s jitter | 5min ± 30s jitter |

### Kubernetes pods (rest + worker + gVisor pool)

| Service | Min replicas | Max replicas | HPA on |
|---|---|---|---|
| document-store-rest | 5 | 100 | CPU > 70% or p99 > 100ms |
| collab-crdt-worker (WS gateway) | 5 | 200 | CPU > 70% or lease pressure |
| comments-and-suggestions-rest | 3 | 50 | CPU > 70% |
| sharing-and-permissions-rest | 3 | 30 | CPU > 70% |
| export-import-rest | 3 | 30 | CPU > 70% |
| export-import-worker (gVisor pool) | 10 | 200 | queue depth > 5min |
| embed-resolver-rest | 3 | 50 | CPU > 70% |
| embed-resolver-worker | 5 | 50 | queue depth > 60s |
| version-history-worker | 3 | 30 | queue depth > 60s |
| document-store-worker | 3 | 30 | queue depth > 60s |

Pre-warmed pool: 10 standby gVisor sandboxes; cold-start ≤ 800ms.

## Saturation indicators (per service)

| Service | Saturation indicator | Threshold |
|---|---|---|
| document-store-rest | rest p99 / queue depth | p99 > 100ms |
| collab-crdt-worker (WS) | concurrent leases | > 80% of cell max |
| comments-and-suggestions-rest | rest p99 + comment insert lat | p99 > 100ms |
| sharing-and-permissions-rest | Cedar eval p99 | p99 > 50ms |
| export-import-worker | concurrent gVisor jobs | > 80% of pool |
| embed-resolver-rest | cross-µservice mTLS p99 | p99 > 500ms |
| ClamAV scanner | queue depth | > 1min |

## Growth projection

| Quarter | Tenants | Docs | Edits/day |
|---|---|---|---|
| Q3 2026 (M03 GA) | 1k | 1M | 10M |
| Q4 2026 | 5k | 5M | 50M |
| Q1 2027 | 20k | 20M | 200M |
| Q2 2027 | 50k | 50M | 500M |

Scale-out plan: at Q4 2026, expand from single-cell to 3-cell deployment (pack-kr + pack-eu + pack-us); at Q2 2027, evaluate pack-jp + pack-sg + pack-au.

## Disaster-recovery sizing

- RTO ≤ 15 min: requires hot standby replica with auto-promotion (Patroni).
- RPO ≤ 60s: requires synchronous replication to nearest standby; async to off-site.
- Backup retention: 30d hot + 12mo cold; pack-us-healthcare ≥ 6y.
- DR drill: quarterly; full restore from cold-tier; validate audit-chain seal continuity + CRDT state restoration.

## References

- ADR-0117: cloud-native infrastructure.
- ADR-0131: per-microservice layout.
- ADR-DOCS-0001 (Loro CRDT); ADR-DOCS-0003 (export backend).
- `cost-budget.md`, `multi-region.md`, `incident-response.md`, `failure-modes.md`.
- Google SRE Workbook ch. 18 (load balancing) + ch. 21 (handling overload).
- AWS Well-Architected Framework Reliability Pillar.
