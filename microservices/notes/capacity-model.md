---
doc_class: CapacityModel
title: notes µservice — Capacity Model + Cell Sizing
microservice: notes
status: Accepted
date: 2026-05-17
owner_team: axis-notes + ops-sre-reliability
related_artifacts:
  - microservices/notes/PRD.md
  - microservices/notes/multi-region.md
  - microservices/notes/cost-budget.md
doc_status: published
---

# Capacity Model — notes µservice

## Workload Shape

Note workloads are bimodal:

| Mode | Profile | Notes |
|---|---|---|
| Capture-burst | spike on first-cup-of-coffee + meeting boundaries | small notes (<5KB); high write rate; low read rate |
| Read-heavy | sustained read for graph-navigation + search | moderate read rate; very low write rate |
| Loro-collab | spike on shared-note sessions | small ops (<1KB) at 5-20 Hz per active session |
| Web-clipper | clip-on-discovery; episodic | larger payloads (50KB-500KB raw HTML); per-installation rate |

## Per-Cell Baseline + Max

| Dimension | XS-baseline | M-target | XL-max | Scale-out trigger |
|---|---|---|---|---|
| Active notes accounts | 50k | 200k | 500k | API CPU > 70 % |
| Notes/sec creates | 1k | 5k | 20k | Postgres write IOPS > 70 % |
| Notes/sec edits | 2k | 10k | 50k | Postgres write IOPS > 70 % |
| Notes/sec opens (warm) | 5k | 25k | 100k | Redis cache hit ratio < 80 % |
| Tags per tenant | 100k | 1M | 10M | tag-graph adjacency cardinality |
| Backlinks per note | 200 baseline | 1k baseline | 50k cap | per-note fan-in cap |
| Web-clipper installations | 50k | 200k | 500k | per-installation rate-limit cap |
| Active Loro collab sessions | 100 | 500 | 5k | Loro op-broker queue depth |
| Search index size (Professional only) | 50GB | 500GB | 2TB | shard count exceeded |
| Attachments/day (via drive µservice) | 100k | 500k | 2M | drive-µservice rate-limit |
| Daily-notes/day | = active-accounts | = active-accounts | = active-accounts | scale linear with accounts |

## Per-BC Compute Sizing (XS-baseline)

| BC | Pod replicas | CPU request | Memory request | Notes |
|---|---|---|---|---|
| `note-store-rest` | 6 | 500m | 512Mi | HPA min 6 max 100 |
| `note-store-worker` | 4 | 500m | 1Gi | HPA min 4 max 50 |
| `tag-graph` | 3 | 250m | 256Mi | HPA min 3 max 30 |
| `backlink-graph` | 3 | 500m | 1Gi | HPA min 3 max 50; worker-heavy |
| `daily-note` | 2 | 100m | 256Mi | static |
| `template-gallery` | 2 | 100m | 256Mi | static |
| `web-clipper-bridge` | 4 | 250m | 512Mi | HPA min 4 max 50 |
| `share-link` | 3 | 250m | 256Mi | HPA min 3 max 30 |
| `embed` | 2 | 100m | 256Mi | drive client only |
| `checklist-worker` | 2 | 250m | 512Mi | HPA min 2 max 20 |
| `version-history-worker` | 3 | 250m | 512Mi | HPA min 3 max 30 |
| `search-index-worker` | 4 | 500m | 1Gi | HPA min 4 max 30 |
| `graph-view-data` | 2 | 250m | 512Mi | static |
| `collab-edit-broker` | 3 | 500m | 1Gi | HPA min 3 max 30; Loro broker |
| `import-pipeline-worker` | 2 | 1000m | 2Gi | HPA min 2 max 20; CPU-heavy parse |
| `export-pipeline-worker` | 2 | 1000m | 2Gi | HPA min 2 max 20 |
| `ai-assist-worker` | 3 | 250m | 512Mi | thin client; foundry-runtime does heavy lift |
| `e2e-key-management` | 3 | 100m | 256Mi | thin signing-cert distribution |

## Storage Sizing

| Store | Per-cell XS | M-target | XL-max | Notes |
|---|---|---|---|---|
| Postgres (notes metadata + Professional body) | 500GB | 5TB | 50TB | per-tenant-shard at M+ |
| Postgres (Personal-tier ciphertext) | 200GB | 2TB | 20TB | append + tombstone |
| Postgres (tag adjacency + backlink) | 50GB | 500GB | 5TB | indexed heavily |
| Postgres (version-history) | 1TB | 10TB | 100TB | compacted at 90d for inactive |
| Redis (sync + hot cache) | 16GB | 64GB | 256GB | per-pack |
| Meilisearch (Professional search) | 50GB | 500GB | 2TB | per-tenant namespace |
| S3 (attachments via drive µservice) | n/a (drive-µservice owns) | — | — | — |
| S3 (Personal-tier ciphertext blobs > 100KB) | 1TB | 10TB | 100TB | tenant-pinned bucket |
| OpenBao / KMS | per-tenant escrow per epoch | — | — | tenant-isolated |

## HPA Policies

```yaml
# Standard per-pod HPA shape — applied to each BC
hpa:
  minReplicas: <per-BC>
  maxReplicas: <per-BC>
  metrics:
    - type: Resource
      resource: {name: cpu, target: {type: Utilization, averageUtilization: 70}}
    - type: Resource
      resource: {name: memory, target: {type: Utilization, averageUtilization: 80}}
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies: [{type: Percent, value: 20, periodSeconds: 60}]
    scaleUp:
      stabilizationWindowSeconds: 0
      policies: [{type: Percent, value: 100, periodSeconds: 30}, {type: Pods, value: 4, periodSeconds: 30}]
```

## Tenant-Tier Sizing (Reference Customers)

| Tier | Active users | Notes total | Avg notes/user | Tags/tenant | Storage | Compute (vCPU) |
|---|---|---|---|---|---|---|
| Starter | 50 | 10k | 200 | 1k | 5GB | 4 |
| Team | 500 | 250k | 500 | 10k | 100GB | 32 |
| Business | 5k | 5M | 1k | 100k | 1TB | 256 |
| Enterprise | 50k | 100M | 2k | 1M | 20TB | 2,048 |
| Heavy power-user vault (per-user) | 1 | 100k | 100k | 5k (personal) | 5GB | per-cell |

## Capacity Headroom Policy

- Target 40 % headroom on CPU + 40 % on memory at p99 over 7d.
- Target 50 % headroom on Postgres write IOPS over 7d.
- Capacity review monthly; pre-emptive scale-up if any dimension within 20 % of XL-max.

## Growth Assumptions

- Year-1: 10× user growth quarter-over-quarter for first 4Q subsequent-to-launch.
- Personal-tier ratio: 70 % Personal / 30 % Professional notes (initial).
- Power-user vault upper-bound: 100k notes per user.
- Loro collab penetration: 5 % of Professional notes have ≥ 1 collab session/month.

## References

- ADR-0130 (SLO-gated promotion).
- ADR-0131 (per-microservice flat layout).
- `multi-region.md`.
- `cost-budget.md`.
