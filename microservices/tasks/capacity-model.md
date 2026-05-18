---
doc_class: CapacityModel
template_id: TPL-CAPACITY-MODEL
microservice: tasks
status: Accepted
date: 2026-05-17
owner_team: axis-tasks + ops-sre-reliability
related_adrs: [ADR-0117, ADR-0131]
doc_status: published
---

# Capacity Model — tasks µservice

## Purpose

Model per-cell capacity envelope, scale-out triggers, and headroom posture for tasks. Drives Helm `replicas` / HPA / Postgres / Redis / Meilisearch configuration + finops cost budget.

## Demand model

### Per-tenant demand (medium tenant, 1k active users)

| Workload | Rate | Notes |
|---|---|---|
| Task-list render | 30 RPS | board + list + table views |
| Task create | 5 RPS | new tasks + subtasks |
| Task update | 20 RPS | field changes; status transitions |
| Cross-project search | 5 RPS | Meilisearch query |
| Board render with DnD | 10 RPS (active board sessions) | client DnD + server commit |
| Bulk-update 100 tasks | 0.1 RPS | rare but spiky |
| Recurring materialisation | 0.5 / s | background |
| Webhook fire | 25 RPS (5× event rate × subscribers) | fanout |
| Dependency-edge add | 1 RPS | edge writes per project |
| Time-tracking tick (M02+) | 5 / s (5 concurrent timers per tenant) | append-only |
| Importer job | 1 / day per tenant | migration-tier; bursty |
| Comment add | 10 RPS | per active project |

### Per-cell aggregate (10M active tasks baseline, 500k projects)

| Workload | Aggregate rate (steady-state p50) |
|---|---|
| Task fetches | 30k RPS |
| Task writes | 5k RPS |
| Task updates | 20k RPS |
| Cross-project search | 5k RPS |
| Webhook fire | 25k RPS |
| Recurring materialisation | 500 / s |
| Active board sessions | 50k |
| Time-tracking ticks | 5k / s |

## Capacity envelope (per cell)

| Dimension | Baseline | Max | Scale-out trigger |
|---|---|---|---|
| Active projects | 500k | 5M | Postgres connection pool > 70% |
| Active tasks | 10M | 100M | task-store rest p99 > 200ms |
| Tasks/s write | 2k | 20k | task-store rest p99 > 200ms |
| Cross-project search/s | 500 | 5k | search-index rest p99 > 300ms |
| Recurrence materialisation/s | 200 | 2k | recurrence worker queue depth > 60s of cadence |
| Webhook fire/s | 5k | 50k | webhook worker queue > 1000 |
| Active board sessions (DnD presence) | 50k | 500k | view-engine rest CPU > 70% |
| Active CSV/JSON imports concurrent | 10 | 100 | importer worker queue > 5min |
| Dependency edges/s | 100 | 1k | cycle-check p99 > 50ms |
| Time-tracking ticks/s | 5k | 50k | time-track worker queue depth > 5s |
| Bulk-edit operations/s | 1 | 10 | tasks-per-bulk-op > 10k requires second confirmation |

## Substrate sizing

### Postgres (task-store + project-list + dependency-edge; 3-replica HA + per-tenant RLS)

| Param | Baseline | Max |
|---|---|---|
| OCPUs (primary) | 16 | 64 |
| OCPUs (each replica) | 8 | 32 |
| Memory (primary) | 128 GB | 512 GB |
| Persistent block | 4 TB | 32 TB |
| max_connections | 200 per replica | 1000 |
| WAL retention | 30 GB | 200 GB |

Scale-out triggers:
- Connection pool > 70% → scale rest pods.
- CPU > 70% sustained → vertical scale primary; replicas take horizontal read scale.
- Storage > 70% → expand persistent block; consider per-tenant partition pruning.

### Redis (view-cache + presence)

| Param | Baseline | Max |
|---|---|---|
| Shards | 3 | 15 |
| Per-shard memory | 8 GB | 32 GB |
| Per-tenant key prefix | yes | yes |
| Eviction policy | allkeys-lru | allkeys-lru |
| TTL | 60s ± 5s jitter | 60s ± 5s jitter |

### Meilisearch (cross-project search; per-tenant index)

| Param | Baseline | Max |
|---|---|---|
| Cluster nodes | 3 | 9 |
| OCPUs per node | 4 | 16 |
| Memory per node | 32 GB | 128 GB |
| Storage per node | 500 GB | 5 TB |
| Per-tenant index name | yes | yes |
| Index version | Meilisearch 0.10.0 LTS | — |
| Rebuildable | yes (degraded mode falls back to Postgres trigram) | — |

### Kubernetes pods (rest + worker)

| Service | Min replicas | Max replicas | HPA on |
|---|---|---|---|
| task-store-rest | 5 | 100 | CPU > 70% or p99 > 200ms |
| project-list-rest | 3 | 50 | CPU > 70% |
| view-engine-rest | 5 | 100 | CPU > 70% (DnD presence) |
| dependency-graph-rest | 3 | 50 | CPU > 70% or cycle-check p99 > 50ms |
| search-index-rest | 5 | 100 | CPU > 70% or p99 > 300ms |
| importers-rest | 3 | 30 | CPU > 70% |
| task-store-worker | 3 | 30 | queue depth > 60s |
| recurrence-worker | 3 | 30 | queue depth > 60s |
| search-index-worker | 3 | 30 | queue depth > 60s (full-rebuild) |
| importers-worker | 3 | 30 | queue depth > 5min |
| webhook-fanout-worker | 5 | 100 | queue depth > 1000 |
| time-tracking-tick-worker (M02+) | 3 | 30 | queue depth > 5s |

Pre-warmed pool: 5 standby pods; cold-start ≤ 700ms (Rust binary; minimal init).

## Saturation indicators (per service)

| Service | Saturation indicator | Threshold |
|---|---|---|
| task-store-rest | rest p99 / queue depth | p99 > 200ms |
| view-engine-rest | rest p99 + DnD presence count | p99 > 100ms perceived |
| dependency-graph-rest | cycle-check p99 | > 50ms |
| search-index-rest | rest p99 + Meilisearch CPU | p99 > 300ms |
| importers-rest | concurrent job count | > 10 / pod |
| recurrence-worker | queue depth | > 60s of cadence |
| webhook-fanout-worker | event-bus producer queue | > 1000 |
| time-tracking-tick-worker | append-only queue depth | > 5s |

## Growth projection

| Quarter | Tenants | Active tasks | Events/day |
|---|---|---|---|
| Q3 2026 (M03 GA) | 1k | 1M | 100k |
| Q4 2026 | 5k | 10M | 1M |
| Q1 2027 | 20k | 50M | 5M |
| Q2 2027 | 50k | 200M | 20M |

Scale-out plan: at Q4 2026, expand from single-cell to 3-cell deployment (pack-kr + pack-eu + pack-us); at Q2 2027, evaluate pack-jp + pack-sg + pack-au.

## Disaster-recovery sizing

- RTO ≤ 15 min: requires hot standby replica with auto-promotion (Patroni for Postgres; Sentinel for Redis).
- RPO ≤ 60s: synchronous replication to nearest standby; async to off-site.
- Meilisearch search-index can be **rebuilt** from Postgres in ≤30 min (PRD AC-09); RPO for search-index is effectively infinite (rebuild from source).
- Backup retention: 30d hot + 12mo cold (S3-compatible WORM); pack-us-healthcare ≥ 6y.
- DR drill: quarterly; full restore from cold-tier; validate audit-chain seal continuity + Meilisearch rebuild.

## References

- ADR-0117: cloud-native infrastructure.
- ADR-0131: per-microservice layout.
- `cost-budget.md`, `multi-region.md`, `incident-response.md`, `failure-modes.md`.
- Google SRE Workbook ch. 18 (load balancing) + ch. 21 (handling overload).
- AWS Well-Architected Framework Reliability Pillar.
- `microservices/calendar/capacity-model.md` — sibling reference template.
