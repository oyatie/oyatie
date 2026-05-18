---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: foundry-runtime
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-foundry-runtime + ops-sre-reliability
deciders: ops-finops, axis-foundry-runtime, ops-sre-reliability, council-architecture
related_adrs: [ADR-0025, ADR-0117, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/foundry-runtime/capacity-model.md
  - microservices/foundry-runtime/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (foundry-runtime µservice)

## Purpose

Track the foundry-runtime µservice's monthly cloud cost across compute + Redis + Postgres + KMS + network egress; surface budget breach via the `oya-check-cost-budget` LEAN lane. Numbers cite OCI public pricing (2026-05-17) and Redis 7.4 + Postgres 16 reference architectures from `capacity-model.md`; verify-at-deploy markers called out.

## Cost Categories

| Category | What | OCI pricing reference |
|---|---|---|
| Compute (VM.Standard / OKE node) | Runtime pool pods + capability-executor pods + invocation-orchestrator pods + session-state app + cache app | `oracle.com/cloud/compute/pricing/` |
| Redis-as-service (or self-hosted Redis on OKE) | Session-state hot tier | self-hosted on VM.Standard.E4 + persistent volumes |
| Postgres-as-service (or self-hosted Postgres on OKE) | Session cold restore + capability mirror + invocation lifecycle records | self-hosted on VM.Standard.E4 + persistent volumes |
| Network egress | Sibling µservice mTLS traffic (intra-region; minimal); ingress for tenant traffic | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-pack keyring for Redis-AUTH + Postgres-TDE + audit-chain signing | `oracle.com/security/key-management/pricing/` |
| Load balancer | Istio gateway per pack | `oracle.com/cloud/networking/load-balancing/pricing/` |
| Object storage | Postgres WAL archives + Redis AOF snapshots for cold-tier session restore | `oracle.com/cloud/storage/object-storage/pricing/` |
| Observability cost | Self-monitoring SLI emission cost (small) | shared with observability µservice cost |

## Per-Component Monthly Cost (XS tier, single pack-kr region, M01 launch)

Per `capacity-model.md` §"Worked example: XS tier (M01 launch; 20 tenants pack-kr-only)".

| Component | Replicas × instance-type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| capability-executor-app | 4 × VM.Standard.E4 4-core | $290 | – | $290 |
| invocation-orchestrator-app | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| invocation-orchestrator-worker | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| runtime-pool-app + worker | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| runtime pool warm pods | 8 × VM.Standard.E4 4-core | $580 | – | $580 |
| session-state-app | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| capability-registry-cache-app + worker | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Redis primary nodes (6 shards × 1 primary) | 6 × VM.Standard.E4 4-core | $435 | $200 PV (AOF) | $635 |
| Redis replicas (6 shards × 1 replica) | 6 × VM.Standard.E4 4-core | $435 | $200 PV | $635 |
| Postgres primary (mirror + cold + lifecycle) | 1 × VM.Standard.E4 8-core | $145 | $100 PV (16 LTS data) | $245 |
| Postgres read replica | 1 × VM.Standard.E4 8-core | $145 | $100 PV | $245 |
| Postgres WAL archive (object storage) | – | – | $30 | $30 |
| KMS keyring (per-pack) | – | $5 | – | $5 |
| Load balancer (Istio gateway + public LB) | – | $20 | – | $20 |
| **XS tier total per pack region** | | **~$2,415** | **~$630** | **~$3,045 / month** |

Verify-at-deploy: OCI pricing changes; reconfirm against `oracle.com/cloud/pricing/`. Buffer 15% for OCI rate increases + 20% for actual-vs-forecast.

## Per-Scale-Tier Forecast

| Scale tier | N_tenants | Concurrent invocations | Active sessions | Monthly cost per pack region |
|---|---|---|---|---|
| XS (M01 launch; 20 tenants) | 20 | 1,000 peak | 50,000 | ~$3,000 |
| S (~100 tenants) | 100 | 10,000 peak | 500,000 | ~$15,000 |
| M (~1,000 tenants) | 1,000 | 100,000 peak | 5,000,000 | ~$95,000 |
| L (~10,000 tenants) | 10,000 | 1,000,000 peak | 50,000,000 | ~$850,000 |

## Per-Pack Multipliers

- **DR pair packs** (pack-eu, pack-us, pack-au, pack-in, pack-br, pack-ae, pack-ksa): 1.0× primary + 0.6× warm-standby (per `multi-region.md`).
- **HIPAA pack** (pack-us-healthcare): 1.4× base (6y retention per HIPAA §164.316(b)(2); dedicated HIPAA-eligible region; isolated from non-HC pack-us).
- **KR-FSS-regulated** tenants in pack-kr: 1.2× base (5y retention; KR-resident KMS).
- **Single-region packs** (pack-kr, pack-jp, pack-sg): 1.0× base.
- **High-risk EU AI Act capabilities active** (pack-eu): 1.1× base (extended record-keeping per EU AI Act Art. 12).

## Budget + Alert Thresholds

| Metric | Threshold | Action |
|---|---|---|
| Monthly cost (per pack region) | within 90% of forecast | normal |
| 90% < cost < 110% | yellow alert | FinOps + ops-sre-reliability review |
| 110% < cost < 130% | orange alert | FinOps + leadership review; review autoscale + capacity-model |
| cost > 130% | red alert; budget breach incident | engage ops-finops + axis-foundry-runtime; consider per-tenant rate-limit tightening |
| Per-tenant cost projection (highest spender) | within 5× median | normal |
| Per-tenant cost > 10× median | yellow; engage tenant on capability concurrency discipline | tenant-facing dashboard surfaces overage |
| Per-capability cost projection (highest spender) | within 10× median | normal |
| Provider invocation cost vs runtime overhead | runtime overhead ≤ 5% of total dispatch cost | optimisation target |

## FinOps SLI

| SLI | Target | Burn-rate alert |
|---|---|---|
| Monthly cost / N_tenants (unit-economic) | within 5% of forecast | 6× burn over 6h |
| Cost per 1M invocations | within forecast | 14.4× burn over 1h |
| Pool warm-pod utilisation | ≥ 60% steady-state | informational |
| Spot-vs-on-demand ratio (non-critical components) | ≥ 70% spot | informational |

## Cost-Optimisation Levers

| Lever | Estimated saving | Trade-off |
|---|---|---|
| Increase pool warm-pod TTL | 5–10% compute | Cold-start uptick on low-utilisation tenants |
| Reduce session-state hot-tier retention (14d → 7d) | 30–40% Redis | More frequent cold restores |
| Spot-instance fleet for runtime-pool warm pods | 30–50% compute | Spot eviction → cold-start uptick |
| OCI committed-use discounts (1y / 3y) | 20–40% compute | Vendor lock-in window |
| Postgres connection pooling tighter | 5–10% Postgres | Fewer concurrent connections; latency sensitivity |
| Per-tenant cardinality budget on session-state | 5–20% Redis | Tenant disruption if too aggressive |
| Capability cache TTL extension | 5% compute | Slightly slower descriptor freshness |

## Verification

- `cargo run -p oya-dev-cli -- gate validate cost-budget --microservice foundry-runtime` — exit 0; current spend within 110%.
- Monthly FinOps review.
- Quarterly capacity-model + cost-budget refresh.

## References

- `microservices/foundry-runtime/capacity-model.md`.
- `microservices/foundry-runtime/multi-region.md`.
- `microservices/foundry-runtime/policy/data-residency.md` (per-pack retention multipliers).
- OCI pricing — `oracle.com/cloud/pricing/`.
- Redis 7.4 LTS — `redis.io/docs/about/releases/`.
- Postgres 16 LTS — `postgresql.org/about/news/postgresql-160-released/`.
- FinOps Foundation framework — `finops.org`.
