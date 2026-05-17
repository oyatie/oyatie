---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: observability
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-observability + ops-sre-reliability
deciders: ops-finops, axis-observability, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/observability/capacity-model.md
  - microservices/observability/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (observability µservice)

## Purpose

Track the observability µservice's monthly cloud cost across infrastructure (compute + storage + network), per Layer-A + Layer-B component, per pack region; surface budget breach via the `oya-check-cost-budget` LEAN lane. Numbers cite OCI public pricing (2026-05-17) and Grafana stack reference architectures from `capacity-model.md`; verify-at-deploy markers called out where vendor pricing may have moved.

## Cost Categories

| Category | What | OCI pricing reference |
|---|---|---|
| Compute (VM.Standard / OKE node) | Mimir / Loki / Tempo / Pyroscope / Grafana / Alertmanager / Alloy / OnCall pods | `oracle.com/cloud/compute/pricing/` |
| Object storage (S3-compatible) | Mimir blocks + Loki chunks + Tempo blocks + Pyroscope profiles cold-tier | `oracle.com/cloud/storage/object-storage/pricing/` |
| Block storage (PV) | Postgres for Grafana + OnCall; ingester local cache | `oracle.com/cloud/storage/block-volume/pricing/` |
| Network egress | Cross-region replication; public-status-page; auditor JIT reads | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-pack KMS keyring for SSE + Ed25519 signing | `oracle.com/security/key-management/pricing/` |
| Load balancer | Per-pack Istio gateway + public ingress | `oracle.com/cloud/networking/load-balancing/pricing/` |
| Observability-on-observability | Self-monitoring SLI ingest cost (small) | recursive |

## Per-Component Monthly Cost (XS tier, single pack-kr region, M01 launch)

Per `capacity-model.md` §"Worked example: oyatie XS tier (M01 launch; 20 tenants pack-kr-only)".

| Component | Replicas × instance-type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| Mimir distributor | 4 × VM.Standard.E4 4-core | $290 | – | $290 |
| Mimir ingester | 12 × VM.Standard.E4 8-core | $1730 | $250 PV cache | $1980 |
| Mimir querier | 4 × VM.Standard.E4 4-core | $290 | – | $290 |
| Mimir query-frontend | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Mimir compactor | 2 × VM.Standard.E4 4-core | $145 | $50 PV cache | $195 |
| Mimir ruler | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Mimir store-gateway | 2 × VM.Standard.E4 4-core | $145 | – | $145 |
| Mimir object-storage | – | – | $170 hot (6.7 TB) + $200 cold (98 TB archive) | $370 |
| Loki distributor | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| Loki ingester | 8 × VM.Standard.E4 4-core | $580 | $100 PV cache | $680 |
| Loki querier | 4 × VM.Standard.E4 4-core | $290 | – | $290 |
| Loki index-gateway | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Loki object-storage | – | – | $920 hot (36 TB) + $710 cold (285 TB archive) | $1630 |
| Tempo distributor | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Tempo ingester | 2 × VM.Standard.E4 4-core | $145 | $30 PV cache | $175 |
| Tempo querier | 2 × VM.Standard.E4 4-core | $145 | – | $145 |
| Tempo object-storage | – | – | $30 hot (1.2 GB) + $0.08 cold (31 GB) | $30 |
| Pyroscope | 2 × VM.Standard.E4 2-core | $72 | $25 hot + ~$0 cold | $97 |
| Grafana UI | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Grafana Postgres (HA primary+replica) | 2 × VM.Standard.E4 2-core | $72 | $50 PV | $122 |
| Alertmanager | 3 × VM.Standard.E4 1-core | $54 | – | $54 |
| Grafana Alloy (per-µservice; ~36 µservices) | sidecar per-µservice; ~50m CPU per | $40 | – | $40 |
| Grafana OnCall | 2 × VM.Standard.E4 2-core | $72 | $30 PV (Postgres) | $102 |
| `oya-observability-slo-engine-worker` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-observability-slo-engine-rest` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-observability-slo-engine-app` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| KMS keyring (per-pack) | – | $5 | – | $5 |
| Load balancer (per-pack Istio gateway + public LB) | – | $20 | – | $20 |
| **XS tier total per pack region** | | **~$4960** | **~$2510** | **~$7470 / month** |

Verify-at-deploy: OCI pricing changes; reconfirm against `oracle.com/cloud/pricing/` at deploy time. Buffer 15% for OCI rate increases + 20% for actual-vs-forecast.

## Per-Scale-Tier Forecast

| Scale tier | N_tenants | Monthly cost per pack region | Notes |
|---|---|---|---|
| XS (M01 launch; 20 tenants) | 20 | ~$7500 | active: pack-kr |
| S (~100 tenants) | 100 | ~$25k | active: pack-kr + pack-eu + pack-us (3 packs) |
| M (~1000 tenants) | 1000 | ~$120k | typically 5 active packs |
| L (~10000 tenants) | 10000 | ~$1.1M | all 11 packs + multi-region per pack |

## Per-Pack Multipliers

- **DR pair packs** (pack-eu, pack-us, pack-au, pack-in, pack-br, pack-ae, pack-ksa): 1.0× primary + 0.6× warm-standby (per `multi-region.md`).
- **HIPAA pack** (pack-us-healthcare): 1.4× base (extended retention 6y per HIPAA §164.316(b)(2) + dedicated HIPAA-eligible region; isolated from non-HC pack-us).
- **KR-FSS-regulated** tenants in pack-kr: 1.2× base (retention 5y per KR commercial code; KMS-in-KR).
- **Single-region packs** (pack-kr, pack-jp, pack-sg): 1.0× base (no DR multiplier).

## Budget + Alert Thresholds

| Metric | Threshold | Action |
|---|---|---|
| Monthly cost (per pack region) | within 90% of forecast | normal |
| 90% < cost < 110% | yellow alert | FinOps + ops-sre-reliability review |
| 110% < cost < 130% | orange alert | FinOps + leadership review; review autoscale + capacity-model |
| cost > 130% | red alert; budget breach incident | engage ops-finops + axis-observability; consider per-tenant rate-limit tightening |
| Per-tenant cost projection (highest spender) | within 5× median tenant | normal |
| Per-tenant cost > 10× median | yellow; engage tenant on cardinality discipline | tenant-facing dashboard surfaces self-overage |

## FinOps SLI

| SLI | Target | Burn-rate alert |
|---|---|---|
| Monthly cost / N_tenants (unit-economic) | within 5% of forecast | 6× burn over 6h |
| Storage growth / day (avg over last 7d) | within forecast | 14.4× burn over 1h (catches runaway-cardinality) |
| Spot-vs-on-demand ratio | ≥ 70% spot for non-critical workloads | informational |

## Cost-Optimisation Levers

| Lever | Estimated saving | Trade-off |
|---|---|---|
| Increase Mimir compaction aggressiveness | 10–15% Mimir storage | More compactor CPU |
| Reduce per-tenant default sampling on traces | 20–40% Tempo storage | Fewer spans available |
| Spot-instance fleet for stateless components (querier, distributor) | 30–50% compute | Spot eviction recovery via HA |
| OCI committed-use discounts (1y / 3y) | 20–40% compute | Vendor lock-in window |
| Archive cold-tier earlier (30d → 14d threshold) | 5–10% storage | Slower historical queries |
| Per-tenant cardinality budget enforcement | 5–20% Mimir compute | Tenant disruption if too aggressive |
| Cross-region replication: async vs sync | varies | RPO trade-off |

## Verification

- `cargo run -p oya-dev-cli -- gate validate cost-budget --microservice observability` — exit 0; current spend within 110%.
- Monthly FinOps review: actual vs forecast; lever decisions logged.
- Quarterly: capacity-model + cost-budget refresh; re-run formulas with current data.

## References

- `microservices/observability/capacity-model.md`.
- `microservices/observability/multi-region.md`.
- `microservices/observability/policy/data-residency.md` (per-pack retention multipliers).
- OCI pricing — `oracle.com/cloud/pricing/`.
- Grafana stack capacity guides — `grafana.com/docs/`.
- FinOps Foundation framework — `finops.org`.
