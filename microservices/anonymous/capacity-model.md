---
doc_class: CapacityModel
template_id: TPL-CAPACITY-MODEL
microservice: anonymous
status: Accepted
date: 2026-05-17
owner_team: axis-anonymous + ops-platform
related_adrs: [ADR-0139]
review_cadence: monthly
doc_status: published
---

# Capacity Model: anonymous µservice

## Sizing inputs

| Input | Source | Default |
|---|---|---|
| Active users / cell (MAU) | Tenant-onboarding form + observed growth | 200k baseline; 2M max per cell |
| Average posts / user / day | Industry benchmarks (Sidechat / YikYak proxy data) | 1.2 |
| Average comments / post | Internal forecast | 4.0 |
| Average vote events / post (up + down) | Internal forecast | 12.0 |
| Hashtag adoption ratio | Internal forecast | 0.3 (30% of posts) |
| Affinity communities / tenant | Tenant-config | 100 baseline; 50k max |
| Push notification fanout per event | follower graph estimate | ≤ 5k members per affinity |
| T2 attachment adoption ratio | Tenant opt-in | 0.0 default; up to 0.05 when enabled |
| BBS+ verify calls / session | Per-session ~1 (cached) | 1 |

## Sustained workload (per cell)

| Component | RPS | Peak RPS | p99 latency target |
|---|---|---|---|
| Post-create | 100 | 1k | ≤ 250ms |
| Comment-create | 400 | 4k | ≤ 250ms |
| Vote-action | 2k | 50k | ≤ 50ms |
| Feed-render | 2k | 30k | ≤ 500ms |
| Search hashtag | 100 | 1k | ≤ 600ms |
| Affinity-attestation verify | 50 | 500 | ≤ 1s |
| Abuse-classifier inference | 100 (batched 100 per call → 1 RPS to foundry-runtime) | 1k posts/s | ≤ 400ms |
| Hard-delete worker (steady) | 50 ops/s | 500 ops/s | n/a (background) |
| Legal-process disclosure | sporadic (operational) | ≤ 10/day per cell | n/a |
| Notification push | 1k | 30k | ≤ 1s |
| Anonymous-DM message | 50 | 500 | ≤ 100ms |

## Postgres sizing

| Resource | Baseline | Max per cell | Scale trigger |
|---|---|---|---|
| Connection pool | 500 | 2000 | pgBouncer pool saturation |
| Storage | 200 GB | 5 TB | provisioned IOPS saturation |
| Write IOPS | 5k | 50k | sustained > 70% |
| Read IOPS | 20k | 200k | sustained > 70% |
| Replication slots | 4 | 16 | DR + read-replica |

Partition strategy:
- `anonymous.post`: partitioned by `(tenant_id, affinity_id, posted_at year-month)`.
- `anonymous.vote`: partitioned by `(tenant_id, post_id mod 64)`.
- `anonymous.blinded_credential`: partitioned by `(tenant_id, issued_at year-month)`.
- `anonymous.affinity_attestation_binding`: partitioned by `(tenant_id, affinity_id)`.
- `anonymous.legal_process_disclosure`: partitioned by `(tenant_id, received_at year)` — small table.

## Valkey sizing

| Cluster | Memory | Nodes | Purpose |
|---|---|---|---|
| anonymous-redis-cache | 32 GB | 6 (3 primary + 3 replica) | Feed cache, BBS+ verify cache, rate-limit |
| anonymous-redis-streams | 16 GB | 6 (3 primary + 3 replica) | Vote counter, fanout |

## Foundry-runtime classifier

| Tier | Batched 100/call | Calls/s | Capacity per pod |
|---|---|---|---|
| T2 classifier (limited risk) | 100 verdicts | 10–100 calls/s | 1k verdicts/s/pod |

## OPSWAT MetaDefender (T2 attachments only; off by default)

| Capacity | Per attachment |
|---|---|
| Scan latency | ≤ 2s |
| Throughput | 100 scans/s/instance |
| Tenant opt-in default | off |

## Scale-out triggers

| Metric | Threshold | Action |
|---|---|---|
| Feed-render p99 latency | > 250ms 5min | Add 2 feed-timeline pods |
| Post-create p99 latency | > 100ms 5min | Add 2 post-thread-rest pods |
| Postgres write IOPS | > 70% sustained 10min | Scale Postgres class one tier |
| Valkey memory | > 75% | Evict + scale cluster |
| Foundry-runtime classifier queue depth | > 5k | Scale classifier inference pods |
| Hard-delete worker lag | > 10s | Scale retention-policy-worker pods |

## Capacity headroom

- 30% headroom maintained against forecasted peak.
- Black-friday-class spike absorbed via HPA + Valkey ENI prewarm.
- Per-tenant rate-limit prevents runaway tenant.

## Tail-latency budget

Per Hyrum's-law of dependencies:

| Path | Component-by-component p99 budget |
|---|---|
| Post-create p99 = 250ms | Cedar 10ms + Postgres write 80ms + audit-chain seal 50ms + fanout queue 20ms + buffer 90ms |
| Feed-render p99 = 500ms | Cedar 10ms + Valkey cache 50ms + Postgres backfill 200ms + Cedar filter 100ms + buffer 140ms |
| Vote-action p99 = 50ms | Cedar 10ms + Valkey increment 5ms + buffer 35ms |
| Affinity-attestation verify p99 = 1s | BBS+ verify (CPU-bound) 600ms + cache miss path + buffer |
| Abuse-classifier p99 = 400ms | foundry-runtime RTT 50ms + inference 200ms + buffer 150ms |
