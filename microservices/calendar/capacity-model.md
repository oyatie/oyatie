---
doc_class: CapacityModel
template_id: TPL-CAPACITY-MODEL
microservice: calendar
status: Accepted
date: 2026-05-17
owner_team: axis-calendar + ops-sre-reliability
related_adrs: [ADR-0117, ADR-0131]
doc_status: published
---

# Capacity Model — calendar µservice

## Purpose

Model per-cell capacity envelope, scale-out triggers, and headroom posture. Drives Helm `replicas` / HPA / Postgres replica configuration + finops cost budget.

## Demand model

### Per-tenant demand (medium tenant, 1k active users)

| Workload | Rate | Notes |
|---|---|---|
| Event-fetch requests | 50 RPS | reads dominate (5:1 read:write) |
| Event writes | 10 RPS | new events + updates |
| Cross-tenant availability lookups | 30 RPS | interactive scheduling |
| Recurrence expansions | 1 / s | typically background expansion |
| Room bookings | 0.5 RPS | resource-graph writes |
| Invitation sends (fanout × attendees) | 5 RPS | event creates × avg 5 attendees |
| .ics imports (background) | 1 / day | migration-tier; bursty |
| CalDAV PROPFIND | 5 RPS | per active CalDAV client |
| CalDAV PUT/DELETE | 1 RPS | per active CalDAV client |

### Per-cell aggregate (100k active calendars baseline)

| Workload | Aggregate rate (steady-state p50) |
|---|---|
| Event fetches | 50k RPS |
| Event writes | 10k RPS |
| Cross-tenant availability | 30k RPS |
| Recurrence expansions | 1k / s |
| Room bookings | 500 RPS |
| Invitation sends | 5k RPS |
| CalDAV PROPFIND | 50k RPS (peak from sync clients) |

## Capacity envelope (per cell)

| Dimension | Baseline | Max | Scale-out trigger |
|---|---|---|---|
| Active calendars | 100k | 1M | Postgres connection pool > 70% |
| Active users | 1M | 10M | rest-pod CPU > 70% |
| Events/s write | 1k | 10k | event-store rest p99 > 200ms |
| Cross-tenant availability lookup/s | 5k | 50k | availability-resolver rest p99 > 400ms |
| Recurrence expansion/s | 100 | 1k | worker queue depth > 60s of cadence |
| Active CalDAV sessions | 10k | 100k | rest-pod CPU > 70% |
| Room bookings/s | 50 | 500 | Postgres FOR UPDATE wait time > 200ms |
| Invitation dispatch/s | 500 | 5k | event-bus producer queue > 1000 |
| .ics import jobs concurrent | 5 | 50 | worker queue depth > 5min |

## Substrate sizing

### Postgres (event-store; 3-replica HA + per-tenant RLS)

| Param | Baseline | Max |
|---|---|---|
| OCPUs (primary) | 16 | 64 |
| OCPUs (each replica) | 8 | 32 |
| Memory (primary) | 128 GB | 512 GB |
| Persistent block | 2 TB | 16 TB |
| max_connections | 200 per replica | 1000 |
| WAL retention | 30 GB | 200 GB |

Scale-out triggers:
- Connection pool > 70% → scale rest pods (more pool workers).
- CPU > 70% sustained → vertical scale primary; replicas can take horizontal-scale reads.
- Storage > 70% → expand persistent block; consider per-tenant partition pruning if pruning < 60%.

### Redis (availability cache)

| Param | Baseline | Max |
|---|---|---|
| Shards | 3 | 15 |
| Per-shard memory | 8 GB | 32 GB |
| Per-tenant key prefix | yes | yes |
| Eviction policy | allkeys-lru | allkeys-lru |
| TTL | 60s ± 5s jitter | 60s ± 5s jitter |

Scale-out: shard count grows by 2 when per-shard memory > 80%.

### Kubernetes pods (rest + worker)

| Service | Min replicas | Max replicas | HPA on |
|---|---|---|---|
| event-store-rest | 5 | 100 | CPU > 70% or p99 > 200ms |
| availability-resolver-rest | 5 | 100 | CPU > 70% or p99 > 400ms |
| room-booking-rest | 3 | 50 | CPU > 70% |
| ics-import-export-rest | 3 | 30 | CPU > 70% |
| event-store-worker | 3 | 30 | queue depth > 60s |
| availability-resolver-worker | 3 | 30 | queue depth > 60s |
| invitation-flow-worker | 3 | 50 | queue depth > 60s |

Pre-warmed pool: 5 standby pods; cold-start ≤ 700ms (Rust binary; minimal init).

## Saturation indicators (per service)

| Service | Saturation indicator | Threshold |
|---|---|---|
| event-store-rest | rest p99 / queue depth | p99 > 200ms |
| availability-resolver-rest | rest p99 + cache hit rate | p99 > 400ms or hit rate < 80% |
| room-booking-rest | Postgres FOR UPDATE wait time | > 200ms |
| ics-import-export-rest | concurrent job count | > 10 / pod |
| recurrence-engine worker | queue depth | > 60s of cadence |
| invitation-flow worker | event-bus producer queue | > 1000 |

## Growth projection

| Quarter | Tenants | Calendars | Events/day |
|---|---|---|---|
| Q3 2026 (M02 GA) | 1k | 100k | 1M |
| Q4 2026 | 5k | 500k | 5M |
| Q1 2027 | 20k | 2M | 20M |
| Q2 2027 | 50k | 5M | 50M |

Scale-out plan: at Q4 2026, expand from single-cell to 3-cell deployment (pack-kr + pack-eu + pack-us); at Q2 2027, evaluate pack-jp + pack-sg + pack-au.

## Disaster-recovery sizing

- RTO ≤ 15 min: requires hot standby replica with auto-promotion (Patroni).
- RPO ≤ 60s: requires synchronous replication to nearest standby; async to off-site.
- Backup retention: 30d hot + 12mo cold (S3-compatible WORM); pack-us-healthcare ≥ 6y.
- DR drill: quarterly; full restore from cold-tier; validate audit-chain seal continuity.

## References

- ADR-0117: cloud-native infrastructure.
- ADR-0131: per-microservice layout.
- `cost-budget.md`, `multi-region.md`, `incident-response.md`, `failure-modes.md`.
- Google SRE Workbook ch. 18 (load balancing) + ch. 21 (handling overload).
- AWS Well-Architected Framework Reliability Pillar.
