---
doc_class: CapacityModel
template_id: TPL-CAPACITY-MODEL
microservice: community
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-community + ops-sre
related_adrs: [ADR-0056, ADR-0105, ADR-0135, ADR-0131]
doc_status: published
---

# Capacity Model: community µservice

## Workload Characterisation

### Per-tenant baseline (M tier)

- Members: 2 000 active / 8 000 total
- Posts / day: 5 000
- Replies / post: 2.3 (avg)
- Votes / day: 25 000
- KB articles / month: 60 new + 200 edits
- KB attachment volume: 200 GB
- Search QPS: 200
- Subscriptions / member: 12

### Read:write ratio

- Feed read : post create ≈ 200:1
- Vote cast : vote read (tally) ≈ 1:30
- KB article view : KB article edit ≈ 300:1

## Per-Component Sizing

### Postgres (Citus + Patroni)

| Tier | Coordinator | Workers | Storage / worker | WAL retention |
|---|---|---|---|---|
| XS | 4 vCPU / 16 GB | 2 × 4 vCPU / 16 GB | 200 GB | 7 d |
| S | 8 vCPU / 32 GB | 4 × 8 vCPU / 32 GB | 500 GB | 7 d |
| M | 16 vCPU / 64 GB | 8 × 16 vCPU / 64 GB | 1 TB | 7 d |
| L | 32 vCPU / 128 GB | 16 × 32 vCPU / 128 GB | 4 TB | 14 d |
| XL | 64 vCPU / 256 GB | 32 × 64 vCPU / 256 GB | 8 TB | 14 d |

Distribution column: `tenant_id`. Replication factor: 2.

### Elasticsearch

| Tier | Data nodes | Master | Storage / data | Heap |
|---|---|---|---|---|
| XS | 3 × 4 vCPU / 16 GB | 3 × 2 vCPU / 8 GB | 100 GB | 8 GB |
| S | 3 × 8 vCPU / 32 GB | 3 × 2 vCPU / 8 GB | 200 GB | 16 GB |
| M | 6 × 16 vCPU / 64 GB | 3 × 4 vCPU / 16 GB | 500 GB | 32 GB |
| L | 12 × 32 vCPU / 128 GB | 3 × 4 vCPU / 16 GB | 1 TB | 64 GB |
| XL | 24 × 64 vCPU / 256 GB | 3 × 8 vCPU / 32 GB | 2 TB | 128 GB |

Index pattern: `community-<tenant_id_short>-<bc>`. Replica count: 1.

### Valkey

| Tier | Nodes | RAM / node | Mode |
|---|---|---|---|
| XS | 3 | 8 GB | Cluster (sharded) |
| S | 6 | 16 GB | Cluster |
| M | 12 | 32 GB | Cluster |
| L | 24 | 64 GB | Cluster |
| XL | 48 | 128 GB | Cluster |

LFU eviction. Per-tenant memory quota: tier / tenants.

### S3 (KB attachments)

- Per-tenant prefix: `s3://oya-community-attachments-<region>/<tenant_id>/`.
- Object lock: governance, 6 y default.
- Lifecycle: 90 d → IA; 365 d → Glacier (opt-in).

## Headroom

- Postgres CPU target: 60 % p95.
- ES heap target: 60 % p95.
- Valkey memory target: 70 % p95.
- Worker pool: 50 % HPA min; 90 % HPA scale-out.

## Scaling Triggers

| Component | Metric | Threshold | Action |
|---|---|---|---|
| Postgres | Workers CPU > 80 % for 10 min | warning | add worker pair |
| Postgres | Coordinator CPU > 80 % for 10 min | critical | scale up coordinator |
| ES | Heap > 80 % | warning | scale out data nodes |
| ES | Index latency p99 > 500 ms | critical | scale + reindex |
| Valkey | Memory > 80 % | warning | scale out cluster |
| Worker | Lag > 10 min | critical | scale out worker fleet |
| REST | p99 > 300 ms | warning | scale out gateway |

## Load Drill (quarterly)

- 10× nominal traffic for 30 min.
- Expected: SLOs hold; no error budget burn > 10 %.
- Failure: ADR + capacity revision before next drill.

## Multi-Region Capacity

Per region: full stack. Cross-region: opt-in tenant replication only. No global cluster (data residency).
