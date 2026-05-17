---
doc_class: CapacityModel
title: Capacity Model
microservice: foundry-eval
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry + ops-sre-reliability
deciders: axis-foundry, ops-sre-reliability, council-architecture
related_adrs: [ADR-0024, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/foundry-eval/cost-budget.md
  - microservices/foundry-eval/multi-region.md
review_cadence: quarterly + on every Sev-2 capacity incident
doc_status: published
---

# Capacity Model (foundry-eval µservice)

## Purpose

Forecast resource consumption per scale tier, validate cost-budget assumptions, identify scale-out triggers, and document the formulas behind per-pack capacity envelopes.

## Per-Capability Eval Load Formula

For each capability `c` with eval set version `v`:

```
cases_per_run(c, v)      = |EvalSet(c, v).cases|
runs_per_day(c)          = 1 (nightly) + 0.5 (publish-gate avg per release-cadence)
                          + 0.2 (ad-hoc avg) + 0.1 (A/B avg) + 0.5 (replay sample)
cases_per_day(c)         = cases_per_run(c, v) × runs_per_day(c)
                        ≈ 1000 × 2.3 = 2300 cases/day (steady state, medium capability)
provider_tokens_per_case ≈ 2000 tokens (input + output, medium prompt)
provider_tokens_per_day(c) ≈ 4.6M tokens
```

## XS Tier Worked Example (M01 launch; pack-kr-only; 20 capabilities)

| Metric | Per-capability | Aggregate (20 caps) |
|---|---|---|
| Cases / day | 2300 | 46k |
| Provider tokens / day | 4.6M | 92M |
| GPU pod-hours / day | 0.5 | 10 |
| S3 PUT / day (results + run aggregates) | 200 | 4000 |
| ClickHouse rows / day (per-case results) | 2300 | 46k |
| Postgres reads / day (eval-set metadata) | 100 | 2000 |
| KMS unwrap / day (per-subject DEK on replay) | 30 | 600 |
| Replay-trace fetches / day | 50 | 1000 |
| Audit-chain events / day | ~250 | ~5000 |

GPU pool baseline: 8 GPUs × 24h = 192 pod-hours/day; 10 hours consumed → 5% utilization steady-state, peak ~50% during publish-gate bursts. Comfortable headroom.

ClickHouse capacity:
- Daily inserts ≈ 46k rows.
- Week-partition size ≈ 322k rows × ~1KB = ~320MB / week.
- Annual storage ≈ 17 GB (well within XS budget).

S3 storage:
- Golden outputs (steady-state per capability): ~500 cases × ~2KB = 1MB; 20 caps × 1MB = 20MB (signed envelopes).
- Eval-run results: 46k cases/day × ~2KB = 92MB/day → ~33GB/year.
- Replay traces (sampled): 1000/day × ~10KB = 10MB/day → ~3.6GB/year (hot); 24mo archive cold ~7GB.
- Total ~50GB at XS scale; well within budget.

## Per-Scale-Tier Forecast

| Tier | N_capabilities | N_tenants | Cases/day | Tokens/day | GPU pod-hours/day | S3 hot (GB) | ClickHouse rows |
|---|---|---|---|---|---|---|---|
| XS (M01) | 20 | 20 | 46k | 92M | 10 | ~50 | 17 GB |
| S | 100 | 100 | 230k | 460M | 50 | ~250 | 85 GB |
| M | 500 | 1000 | 1.15M | 2.3B | 250 | ~1250 | 425 GB |
| L | 2000 | 10000 | 4.6M | 9.2B | 1000 | ~5000 | 1.7 TB |

## Scale-Out Triggers

| Component | Trigger | Action |
|---|---|---|
| eval-runner-worker | Queue depth > 60s of cadence | HPA scale +1 (up to 50 max) |
| eval-runner-rest | CPU > 70% or p99 latency > 2s | HPA scale +1 (up to 20 max) |
| GPU pool | Pending-case backlog > 5min | Cluster-autoscaler provisions +2 GPU nodes (up to 64 max) |
| Postgres | Replica CPU > 70% | Add read replica (manual; 2-person rule) |
| ClickHouse | Shard CPU > 70% sustained | Provision new shard (manual; 2-person rule) |
| MinIO / S3 | Storage > 70% of allocation | Provision new bucket OR archive cold-tier earlier |
| KMS | Approaching rate-limit quota | Pre-provision KMS keyring expansion (cloud-secrets ticket) |

## Per-Cell Capacity Envelope

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Eval-runner workers | 4 replicas | 50 replicas | queue depth > 60s |
| GPU pool | 8 GPUs | 64 GPUs | pending backlog > 5min |
| Postgres eval-set metadata QPS | 1 k | 10 k | replica CPU > 70% |
| ClickHouse parity-analytics QPS | 100 | 1 k | shard CPU > 70% |
| ClickHouse rows | – | 10 B (per cluster) | week-partition seal forced |
| S3 golden-output throughput | 1 GB/s | 10 GB/s | provider SLO breach |
| Replay traces ingest | 10⁶/day | 10⁸/day | S3 PUT rate-limit warning |
| KMS DEK ops/sec | 10 | 100 | KMS rate-limit warning |

## Bottleneck Analysis

At each scale tier, identify load-bearing constraint:

| Tier | Bottleneck | Notes |
|---|---|---|
| XS | Provider token cost (eval-time) | Budget-bounded; not capacity |
| S | GPU pool throughput | Multi-pack adds parallel pools |
| M | ClickHouse query latency | Shard-by-week + DP-noise pre-compute |
| L | Cross-region replication BW | Per-pack region-pinning naturally bounds |

## Pre-warmed Pool

- 2 GPU pods + 2 eval-runner-worker pods pre-warmed; cold-start budget ≤ 60s for GPU (CUDA init) and ≤ 500ms for worker.
- Postgres connection pool: 50 connections min; HikariCP-equivalent in eval-runner-worker.
- ClickHouse query budget: 100 QPS per source; bursts buffered at query-frontend.

## Cross-Region Considerations

- M01 launch: pack-kr only; single region; no cross-region replication.
- Post-M01: per-pack region pinning; cross-region replication of eval-sets (read-only) via S3 cross-region replication + Cosign signature integrity preserved across boundary.
- Cross-region golden-output / replay-trace replication: forbidden by default (residency); allowed only with tenant SCC.

## References

- `microservices/foundry-eval/cost-budget.md`.
- `microservices/foundry-eval/multi-region.md`.
- `microservices/foundry-eval/policy/data-residency.md`.
- ADR-0024 §"Nightly cadence".
- OCI capacity reference architectures.
