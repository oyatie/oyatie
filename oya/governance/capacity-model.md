---
doc_class: CapacityModel
title: Capacity Model + Sizing Posture
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry + ops-finops
deciders: ops-sre-reliability, axis-foundry, council-architecture
related_adrs: [ADR-0117, ADR-0131, ADR-0133]
related_artifacts:
  - microservices/governance/cost-budget.md
  - microservices/governance/multi-region.md
  - microservices/governance/failure-modes.md
review_cadence: quarterly + on every major lane addition (>5 new lanes)
doc_status: published
---

# Capacity Model: governance µservice

## Purpose

Model governance µservice load (PRs/min, lanes/PR, finding rate, evidence growth) and predict per-cell capacity, scale-out triggers, and headroom. Feeds `cost-budget.md` and `iac/helm/*` Helm values.

## Volumetric Model

### Per-PR load profile (median PR; XS tier)

| Dimension | Median | p95 | p99 | Notes |
|---|---|---|---|---|
| Files in PR | 5 | 50 | 200 | per `oya-check-pr-traceability` historical |
| Lines changed | 50 | 500 | 5000 | per same source |
| Lanes invoked | ~50 (full platform) | ~50 | ~50 | every PR runs the full platform |
| Per-lane duration | 3s | 10s | 30s | per `lane-execution.md` Invariant 3 |
| Total PR wall-clock | 15s | 45s | 90s | parallel matrix |
| Findings emitted | 0–2 (BLOCKER) | 5 | 20 | majority of PRs pass clean |
| Evidence blob size (per finding) | 4 KB | 50 KB | 500 KB | |
| Postgres inserts (finding + lane-run) | 100 (50 + 50) | 110 | 150 | bounded |

### Per-µservice load profile

| Dimension | XS | S | M | L |
|---|---|---|---|---|
| µservices tracked | 36 | 36 | 50 | 100 |
| Active µservices/day | ~10 | ~15 | ~25 | ~50 |
| PRs/µservice/day | 1–5 | 5–20 | 20–80 | 50–200 |
| Total PRs/day | 30 | 200 | 1500 | 5000 |
| Total PRs/month | 1k | 6k | 50k | 100k |

### Per-Tier Throughput

| Tier | Avg PRs/min | Peak PRs/min | Concurrent lane runs (peak) | Postgres write IOPS (peak) | Evidence S3 ingest (peak) |
|---|---|---|---|---|---|
| XS | 0.7 | 5 | 250 (5 PRs × 50 lanes) | 5k | 1 MB/s |
| S | 4 | 30 | 1500 | 30k | 6 MB/s |
| M | 35 | 200 | 10000 | 200k | 40 MB/s |
| L | 70 | 400 | 20000 | 400k | 80 MB/s |

Verify-at-deploy: peak factors assume working-hours bias of 4× over avg.

## Per-Cell Capacity Envelope

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| ARC runner pods | 8 (pre-warm) | 200 | queue-depth > 60s × cadence |
| Concurrent lane runs | 250 | 4000 | runner-pool saturation > 80% |
| Postgres write IOPS | 20k sustained | 100k burst | replica-lag > 5s |
| Postgres read replicas | 2 | 5 | replay-query p99 > 5s |
| S3 evidence ingest rate | 50 MB/s | 500 MB/s | object-count > 100M per bucket |
| Aggregation-indexer regen cadence | 15 min coalescing | 5 min | divergence lag > 5 min |
| Lane-runtime worker | 2 replicas | 10 replicas | dispatch queue depth > 1k |
| Evidence-emitter worker | 2 replicas | 10 replicas | seal queue depth > 100 |

## Scale-Out Policy

### Kubernetes HPA

- **ARC runner pool**: scale on `pending_lane_runs > 50`; min 4, max 200 replicas; pre-warmed pool of 8.
- **lane-runtime worker**: scale on CPU `>70%`; min 2, max 10 replicas.
- **policy-engine worker**: scale on CPU `>70%`; min 2, max 10 replicas.
- **evidence-emitter worker**: scale on `seal_queue_depth > 100`; min 2, max 10 replicas.
- **aggregation-indexer worker**: scale on CPU `>70%`; min 2, max 5 replicas (lower; less hot).

### Postgres

- Primary-only writes; 2 sync replicas at M01; 5 replicas at L tier.
- Per-Bominal-ADR-0019 posture: vertical-then-horizontal scaling (size up before adding replicas for writes).
- Read scaling: replicas for `ReplayQuery` and `QueryAdmissionVerdict`.

### S3 / Object Storage

- Multi-AZ by construction (OCI Object Storage).
- Per-pack bucket; bucket-count tracked per pack.
- Object-count cap per bucket: 100M; partition by prefix `<microservice>/<year>/<month>/<lane-id>/`.

## Worked example: oyatie XS tier (M01 launch; 1k PRs/month)

| Component | Sizing | Rationale |
|---|---|---|
| ARC runner pool | 8 standbys + 50 burst | Peak 5 PRs/min × 50 lanes = 250 concurrent → 50 runners @ 4 cores each |
| Postgres primary | VM.Standard.E4 4-core | 5k write IOPS peak; lots of headroom |
| Postgres replicas (2) | VM.Standard.E4 4-core each | sync replication; read-scaling for replay |
| Evidence S3 hot tier | 10 TB capacity | 1 MB/s × 30d × 60% retention = ~2.5 TB; 4× headroom |
| Evidence S3 cold tier | 3 TB | older blobs archive after 90d |
| Lane-runtime worker | 2 × VM.Standard.E4 2-core | bounded by Postgres |
| Policy-engine worker | 2 × VM.Standard.E4 2-core | bounded by rule-pack reads (in-memory cache) |
| Evidence-emitter worker | 2 × VM.Standard.E4 2-core | bounded by S3 ingest |
| Aggregation-indexer worker | 2 × VM.Standard.E4 2-core | full repo regen ≤5 min |
| KMS keyring | per-pack | per-pack signing key |

## Cross-Region Story

### M01 launch

- Single KR region (OCI ap-seoul-1).
- Per-pack residency lock per ADR-0117 + `policy/data-residency.md`.
- No cross-region failover at M01; RTO bounded by intra-region recovery.

### Post-M01 expansion

- Per-pack region per ADR-0117; per `multi-region.md` posture.
- Postgres: per-region primary; logical replication for cross-region read-scaling (optional).
- S3: per-pack bucket; cross-region replication refused by default; per-tenant override available.

## Sharding

- **Postgres `findings` table**: partition by `microservice` + `month`; per-µservice retention scaling.
- **Postgres `lane-runs` table**: partition by `month`; lane-runs retention 90d hot + 2y cold.
- **S3 evidence bucket**: per-µservice key prefix; auto-tier from hot (Standard) to cold (Archive) at 90d.
- `oya-check-shardability-cli` lane verifies partition-key presence on the governance µservice itself.

## Forecast vs Actual

| Quarter | Forecast PRs/month | Forecast cost / month | Actual PRs/month | Actual cost | Notes |
|---|---|---|---|---|---|
| 2026-Q2 (M01) | 1000 | $2000 | populated by `oya-dev-cli capacity quarterly-actual --quarter 2026-Q2` on quarter close (writes Actual PRs/month from `gh api graphql` PR-history + Actual cost from `gcloud billing` aggregated query) | populated by same command (cost dimension) | first measurement |
| 2026-Q3 | 2000 | $3500 | — | — | |
| 2026-Q4 | 4000 | $7000 | — | — | |
| 2027-Q1 | 8000 | $12500 | — | — | |

## Capacity Triggers + Actions

| Trigger | Action | Owner |
|---|---|---|
| PR rate > 80% of cell max | Scale ARC pool up; review per-µservice fairness | ops-sre-reliability |
| Postgres write IOPS > 70% | Vertical scale primary; review partition strategy | ops-sre-reliability |
| Postgres replica lag > 5s sustained | Add replica; tune replication config | ops-sre-reliability |
| Evidence S3 bucket object-count > 80M | Add prefix-shard; archive cold-tier earlier | ops-sre-reliability |
| Aggregation-indexer p99 > 5 min | Increase replicas; tune incremental regen | axis-foundry |
| Per-µservice PR-bomb (T-D-01) | Engage per-author rate-limit; alert ops-finops | ops-security |

## Verification

- `cargo run -p oya-dev-cli -- gate validate capacity-model --microservice governance` — exit 0.
- Quarterly capacity-model review by ops-sre-reliability + ops-finops.
- Game-day load test: simulate L-tier PR rate against XS cell; record degradation curve.

## References

- `microservices/governance/cost-budget.md`.
- `microservices/governance/multi-region.md`.
- `microservices/governance/failure-modes.md`.
- Bominal ADR-0019 (state-strategy enum).
- Google SRE Workbook ch. 11 (capacity planning).
- AWS Well-Architected Framework — Performance Efficiency + Reliability.
- `microservices/observability/capacity-model.md` (shape reference).
