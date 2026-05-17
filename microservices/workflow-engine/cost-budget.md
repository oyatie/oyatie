---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: workflow-engine
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-workflow + ops-sre-reliability
deciders: ops-finops, axis-workflow, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0131]
related_artifacts:
  - microservices/workflow-engine/capacity-model.md
  - microservices/workflow-engine/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (workflow-engine µservice)

## Purpose

Track the workflow-engine µservice's monthly cloud cost across infrastructure (compute + storage + network) per Layer-A + Layer-B component per pack region; surface budget breach via the `oya-check-cost-budget` LEAN lane. Numbers cite OCI public pricing (2026-05-17); verify-at-deploy markers called out.

## Cost Categories

| Category | What | OCI pricing reference |
|---|---|---|
| Compute (VM.Standard / OKE node) | Engine workers, REST, outbox-relay, replay workers, Postgres / Citus / Redis / ClickHouse pods | `oracle.com/cloud/compute/pricing/` |
| Block storage (PV) | Postgres + Citus data; Redis AOF; ClickHouse local cache | `oracle.com/cloud/storage/block-volume/pricing/` |
| Object storage | Large step payloads; ClickHouse cold tier | `oracle.com/cloud/storage/object-storage/pricing/` |
| Network egress | Cross-region replication; Studio + SDK clients | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-pack signing keys (spec + audit chain) | `oracle.com/security/key-management/pricing/` |
| Load balancer | Per-pack Istio gateway + public LB | `oracle.com/cloud/networking/load-balancing/pricing/` |
| Engine-on-observability | Self-monitoring SLI ingest cost (small) | recursive |

## Per-Component Monthly Cost (XS tier, single pack-kr region, M02b launch)

Per `capacity-model.md` §"Worked example: oyatie XS tier (M02b launch; 20 tenants pack-kr-only)".

| Component | Replicas × instance-type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| Postgres coordinator (Citus head) | 2 × VM.Standard.E4 8-core (HA) | $580 | $200 PV | $780 |
| Postgres worker nodes (Citus shards) | 4 × VM.Standard.E4 8-core | $1160 | $800 PV | $1960 |
| Postgres read-replica (per worker) | 4 × VM.Standard.E4 4-core | $580 | $400 PV | $980 |
| Redis Sentinel HA cluster | 3 × VM.Standard.E4 2-core | $108 | $30 PV (AOF) | $138 |
| ClickHouse replica | 2 × VM.Standard.E4 4-core | $290 | $300 PV + $50 cold | $640 |
| `execution-engine-worker` | 3 × VM.Standard.E4 4-core | $217 | – | $217 |
| `execution-engine-rest` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `event-bus-worker` (outbox relay) | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `event-bus-rest` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `spec-store-rest` | 2 × VM.Standard.E4 1-core | $36 | – | $36 |
| `replay-debugger-backend-rest` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `execution-engine-app` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `event-bus-app` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `spec-store-app` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `replay-debugger-backend-app` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Object storage (large step payloads) | – | – | $200 hot (8 TB) | $200 |
| KMS keyring | – | $5 | – | $5 |
| Load balancer | – | $20 | – | $20 |
| **XS tier total per pack region** | | **~$3700** | **~$1980** | **~$5680 / month** |

Verify-at-deploy: OCI pricing changes; reconfirm against `oracle.com/cloud/pricing/` at deploy time. Buffer 15% for OCI rate increases + 20% for actual-vs-forecast.

## Per-Scale-Tier Forecast

| Scale tier | N_tenants | Monthly cost per pack region | Notes |
|---|---|---|---|
| XS (M02b launch; 20 tenants) | 20 | ~$5700 | active: pack-kr |
| S (~100 tenants) | 100 | ~$22k | active: pack-kr + pack-eu + pack-us |
| M (~1000 tenants) | 1000 | ~$110k | typically 5 active packs |
| L (~10000 tenants) | 10000 | ~$1.0M | all 11 packs + multi-region per pack |

## Per-Pack Multipliers

- **DR pair packs** (pack-eu, pack-us, pack-au, pack-in, pack-br, pack-ae, pack-ksa): 1.0× primary + 0.6× warm-standby.
- **HIPAA pack** (pack-us-healthcare): 1.4× base (extended retention 6y per HIPAA §164.316(b)(2)).
- **KR-FSS-regulated** tenants in pack-kr: 1.2× base (retention 5y per KR commercial code).
- **Single-region packs** (pack-kr, pack-jp, pack-sg): 1.0× base.

## Budget + Alert Thresholds

| Metric | Threshold | Action |
|---|---|---|
| Monthly cost (per pack region) | within 90% of forecast | normal |
| 90% < cost < 110% | yellow alert | FinOps + ops-sre-reliability review |
| 110% < cost < 130% | orange alert | FinOps + leadership review |
| cost > 130% | red alert; budget breach incident | engage ops-finops |
| Per-tenant cost projection (highest spender) | within 5× median | normal |
| Per-tenant cost > 10× median | yellow; engage tenant on run discipline | tenant-facing dashboard surfaces self-overage |

## FinOps SLI

| SLI | Target | Burn-rate alert |
|---|---|---|
| Monthly cost / N_tenants (unit-economic) | within 5% of forecast | 6× burn over 6h |
| Per-run cost (avg run cost) | within forecast | 14.4× burn over 1h |
| Spot-vs-on-demand ratio | ≥ 70% spot for non-critical workloads | informational |

## Cost-Optimisation Levers

| Lever | Estimated saving | Trade-off |
|---|---|---|
| ClickHouse cold-tier archive earlier (30d → 14d) | 5–10% storage | Slower historical queries |
| Spot-instance fleet for stateless components (REST, replay workers) | 30–50% compute | Spot eviction recovery via HA |
| OCI committed-use discounts (1y / 3y) | 20–40% compute | Vendor lock-in window |
| Per-tenant run budget enforcement | 10–20% engine compute | Tenant disruption if too aggressive |
| Citus shard rebalance off-hours | minor latency win + cost neutral | scheduling complexity |
| Reduce default outbox event retention (24mo → 12mo) | 10% Postgres storage | Replay window shorter |
| Object-storage lifecycle: large step payloads → archive after 7d | 15% object storage | Slower replay for archived |

## Verification

- `cargo run -p oya-dev-cli -- gate validate cost-budget --microservice workflow-engine` — exit 0; current spend within 110%.
- Monthly FinOps review.
- Quarterly: capacity-model + cost-budget refresh.

## References

- `microservices/workflow-engine/capacity-model.md`.
- `microservices/workflow-engine/multi-region.md`.
- `microservices/workflow-engine/policy/data-residency.md`.
- OCI pricing — `oracle.com/cloud/pricing/`.
- FinOps Foundation framework — `finops.org`.
