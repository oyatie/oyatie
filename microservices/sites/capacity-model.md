---
doc_class: CapacityModel
template_id: TPL-CAPACITY-MODEL
microservice: sites
status: Accepted
date: 2026-05-17
owner_team: axis-sites + ops-sre-reliability
related_adrs: [ADR-0117, ADR-0131, ADR-SITES-0002, ADR-SITES-0003]
doc_status: published
---

# Capacity Model — sites µservice

## Purpose

Model per-cell capacity envelope, scale-out triggers, and headroom
posture. Drives Helm `replicas` / HPA / Postgres replica / Meilisearch
shard configuration + finops cost budget.

## Demand model

### Per-tenant demand (medium tenant, 1k pages, 100k monthly visitors)

| Workload | Rate | Notes |
|---|---|---|
| Anonymous page-render (CDN cache miss) | 5 RPS | mostly cache-hit; only misses reach origin |
| Anonymous page-render (CDN cache hit) | 100 RPS | CDN serves; origin not touched |
| Editor authoring requests | 5 RPS | writes via REST; CRDT log on background |
| CMS-collection queries | 20 RPS | indexed reads |
| Site-search queries | 2 RPS | per active visitor session |
| Image-optimize jobs | 1/min | per page-edit with image change |
| Page publishes | 1/hour | typical |
| ACME cert renewals (per-domain) | 1/60d | per-domain |
| Custom-domain DNS verifies | 0.1 RPS | per-domain at bind time |
| AI-page-build (T2) | 1/hour | when enabled |
| Loro CRDT op messages | 100/s | per active co-editing session (~10 editors) |

### Per-cell aggregate (50k active sites baseline)

| Workload | Aggregate rate (steady-state p50) |
|---|---|
| Anonymous page-render (cache miss to origin) | 50k RPS |
| Editor writes | 500 RPS |
| CMS-collection queries | 5k RPS |
| Site-search QPS | 5k QPS |
| Image-optimize jobs | 500/min |
| Page publishes | 500/hour |
| ACME cert renewals | 1k/day |
| AI-page-build (T2) | 1k/hour (enabled subset) |

## Capacity envelope (per cell)

| Dimension | Baseline | Max | Scale-out trigger |
|---|---|---|---|
| Active sites | 50k | 500k | Postgres connection pool > 70% |
| Active pages | 5M | 50M | Postgres event-store rest p99 > 200ms |
| Page renders/s (origin) | 5k | 50k | cdn-delivery-rest p95 > 200ms |
| CMS-collection writes/s | 500 | 5k | postgres FOR UPDATE wait > 200ms |
| Site-search QPS | 5k | 50k | Meilisearch CPU > 70% |
| ACME cert renewals/day | 1k | 10k | Let's Encrypt rate-limit (per ADR-SITES-0004 — multi-account pool) |
| Image-optimize jobs/min | 500 | 5k | libvips worker queue > 5min |
| Concurrent Loro CRDT sessions | 5k | 50k | crdt-relay pod CPU > 70% |
| Publish jobs/s | 5 | 50 | publish-worker queue > 60s |
| CDN cache invalidations/s | 5 | 50 | cdn-purge worker queue > 30s |

## Substrate sizing

### Postgres (site + page + cms-collection; 3-replica HA + per-tenant RLS)

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
- Storage > 70% → expand persistent block; consider per-tenant partition pruning.

### Redis (page-render cache + CMS-collection cache)

| Param | Baseline | Max |
|---|---|---|
| Shards | 3 | 15 |
| Per-shard memory | 16 GB | 64 GB |
| Per-tenant key prefix | yes | yes |
| Eviction policy | allkeys-lru | allkeys-lru |
| TTL | 60s ± 5s jitter (page-render) / 300s (cms-collection) | same |

Scale-out: shard count grows by 2 when per-shard memory > 80%.

### Meilisearch (per-tenant site-search)

| Param | Baseline | Max |
|---|---|---|
| Instances | 3 | 12 |
| Per-instance memory | 16 GB | 64 GB |
| Index per tenant | yes | yes |
| Cross-cell replication factor | 2 | 3 |

Scale-out: instance count grows when per-instance CPU > 70%.

### S3 (published artifacts)

| Param | Baseline | Max |
|---|---|---|
| Per-tenant prefix bucketing | yes | yes |
| Storage per cell | 10 TB | 1 PB |
| WORM (object-lock) | enabled for published-public | enabled |
| Cross-region replication | within-pack only | per-DR policy |

### Kubernetes pods (rest + worker)

| Service | Min replicas | Max replicas | HPA on |
|---|---|---|---|
| site-rest | 3 | 50 | CPU > 70% |
| page-rest | 5 | 100 | CPU > 70% or p99 > 200ms |
| url-routing-rest | 5 | 100 | CPU > 70% |
| domain-binding-rest | 3 | 30 | CPU > 70% |
| domain-binding-worker (ACME renewer) | 3 | 30 | queue depth > 60s |
| cms-collection-rest | 5 | 50 | CPU > 70% |
| search-rest | 5 | 50 | CPU > 70% |
| cdn-delivery-rest | 5 | 100 | CPU > 70% or p95 > 200ms |
| cdn-delivery-worker (publish-pipeline) | 5 | 50 | queue depth > 60s |
| cdn-delivery-worker (image-optimize) | 3 | 30 | queue depth > 5min |

Pre-warmed pool: 5 standby pods; cold-start ≤ 700ms.

## Saturation indicators (per service)

| Service | Saturation indicator | Threshold |
|---|---|---|
| page-rest | rest p99 / queue depth | p99 > 200ms |
| cdn-delivery-rest | rest p95 / cache hit ratio | p95 > 200ms or cache-hit < 80% |
| cms-collection-rest | postgres FOR UPDATE wait | > 200ms |
| search-rest | Meilisearch CPU / query-time | CPU > 70% or p95 > 300ms |
| domain-binding-worker | ACME challenges-queued | > 60s |
| publish-pipeline-worker | publish-queue depth | > 60s |
| image-optimize-worker | optimization queue | > 5min |
| crdt-relay (Loro) | op-message rate / pod CPU | CPU > 70% |

## Growth projection

| Quarter | Tenants | Sites | Pages | Visitors/mo |
|---|---|---|---|---|
| Q3 2026 (M03 GA) | 500 | 5k | 500k | 10M |
| Q4 2026 | 2k | 20k | 2M | 50M |
| Q1 2027 | 10k | 100k | 10M | 200M |
| Q2 2027 | 50k | 500k | 50M | 1B |

Scale-out plan: at Q4 2026, expand from single-cell to 3-cell (pack-kr
+ pack-eu + pack-us); at Q2 2027, evaluate additional packs (jp, sg,
au, in, br, ae, ksa, us-healthcare).

## Disaster-recovery sizing

- RTO ≤ 15 min: requires hot standby replica with auto-promotion (Patroni).
- RPO ≤ 60s: requires synchronous replication to nearest standby; async to off-site.
- Backup retention: 30d hot + 12mo cold (S3-compatible WORM);
  pack-us-healthcare ≥ 6y; pack-kr financial ≥ 5y.
- DR drill: quarterly; full restore from cold-tier; validate
  audit-chain seal continuity.

## CDN considerations (per ADR-SITES-0003)

- Edge cache survives origin outage for ≥ 24h via
  `stale-while-revalidate=86400`.
- Cache-key includes version-hash (fixes legacy version-blind bug);
  invalidation pattern is `tenant-id|site-id|version-hash|route-path`.
- Per-pack edge nodes only; pack-eu uses EU edges; pack-kr uses KR edges.

## References

- ADR-0117: cloud-native infrastructure.
- ADR-0131: per-microservice layout.
- ADR-SITES-0002: rendering strategy.
- ADR-SITES-0003: CDN substrate.
- `cost-budget.md`, `multi-region.md`, `incident-response.md`,
  `failure-modes.md`.
- Google SRE Workbook ch. 18 (load balancing) + ch. 21 (handling overload).
- AWS Well-Architected Framework Reliability Pillar.
