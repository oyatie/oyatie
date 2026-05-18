---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: cell
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-cell-substrate + ops-sre-reliability
deciders: ops-finops, axis-cell-substrate, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/cell/capacity-model.md
  - microservices/cell/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (cell µservice)

## Purpose

Track the cell substrate's monthly cloud cost across infrastructure (K8s control plane + worker nodes + Postgres + warm pool + cell-substrate operator pods), per Layer-A + Layer-B component, per pack. Surface budget breach via `oya-check-cost-budget` LEAN lane. Numbers cite OCI public pricing (2026-05-17) + Kubernetes Cluster API reference architectures; verify-at-deploy markers called out where vendor pricing may move.

## Cost Categories

| Category | What | OCI pricing reference |
|---|---|---|
| Compute (OKE node) | K8s management cluster + workload cluster pools + warm pool + cell-substrate operator pods | `oracle.com/cloud/compute/pricing/` |
| Block storage (PV) | Postgres registry PV + per-cell PVs (workload-µservice-resident) + K8s etcd | `oracle.com/cloud/storage/block-volume/pricing/` |
| Object storage | Per-cell S3 prefixes (workload-resident; cell pays per-cell prefix accounting overhead only) | `oracle.com/cloud/storage/object-storage/pricing/` |
| Network egress | Cross-AZ replication; per-cell pod-to-pod (intra-cell only); audit-chain seal traffic | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-pack KMS keyring for cell credential signing + per-cell SVID issuance | `oracle.com/security/key-management/pricing/` |
| Cluster-API management cluster | per-pack management cluster running Cluster API control plane | OCI OKE pricing |
| Postgres (cell-registry) | per-pack HA Postgres for cell-registry + tenant-assignment + lifecycle-manager state | OCI MySQL HeatWave or self-hosted on OKE |
| SPIFFE / SPIRE | per-pack SPIRE server + agent footprint | self-hosted; minimal |

## Per-Component Monthly Cost (XS tier, single pack-kr region, M01 launch)

Per `capacity-model.md` §"Worked example: XS tier (M01 launch; 20 tenants pack-kr; 4 cells)".

| Component | Replicas × instance-type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| K8s management cluster (Cluster API) | 3 × VM.Standard.E4 4-core | $217 | $30 etcd PV | $247 |
| K8s workload cluster control plane | 3 × VM.Standard.E4 4-core | $217 | $30 etcd PV | $247 |
| Workload cluster node pool (4 cells × ~6 nodes/cell) | 24 × VM.Standard.E4 4-core | $1738 | $400 PV | $2138 |
| Warm pool (2 standby nodes) | 2 × VM.Standard.E4 4-core | $145 | $30 PV | $175 |
| Postgres cell-registry HA (primary + 2 replicas) | 3 × VM.Standard.E4 4-core | $217 | $100 PV | $317 |
| PgBouncer (per-pack pool) | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-cell-cell-registry-rest` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-cell-cell-registry-app` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-cell-tenant-assignment-rest` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-cell-tenant-assignment-worker` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-cell-scheduler-worker` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-cell-lifecycle-manager-worker` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-cell-host-pool-worker` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| SPIRE server (per-pack) | 3 × VM.Standard.E4 1-core | $54 | $10 PV | $64 |
| KMS keyring (per-pack) | – | $5 | – | $5 |
| Load balancer (per-pack ingress) | – | $20 | – | $20 |
| **XS tier total per pack region** | | **~$3187** | **~$600** | **~$3787 / month** |

Verify-at-deploy: OCI pricing changes; reconfirm against `oracle.com/cloud/pricing/` at deploy time. Buffer 15% for OCI rate increases + 20% for actual-vs-forecast.

Cells themselves don't carry direct workload-compute cost on the cell-substrate balance sheet — that cost belongs to workload µservices (each workload µservice pays for its compute inside the cell). The cell substrate pays for the *substrate cost* (registry + scheduler + lifecycle + host pool + management cluster + warm pool).

## Per-Scale-Tier Forecast

| Scale tier | N_tenants | N_cells | Monthly cost per pack region | Notes |
|---|---|---|---|---|
| XS (M01; 20 tenants) | 20 | 4 | ~$3800 | active: pack-kr |
| S (~100 tenants) | 100 | 12 | ~$10k | active: pack-kr + pack-eu + pack-us (3 packs) |
| M (~1000 tenants) | 1000 | 100 | ~$60k | typically 5 active packs |
| L (~10000 tenants) | 10000 | 800 | ~$500k | all 11 packs + multi-region per pack |

## Per-Pack Multipliers

- **DR pair packs** (pack-eu, pack-us, pack-au, pack-in, pack-br, pack-ae, pack-ksa): 1.0× primary + 0.5× warm-standby (cell-substrate has lower DR-warm cost than observability because management state is small).
- **HIPAA pack** (pack-us-healthcare): 1.3× base (extended audit retention 6y per HIPAA §164.316(b)(2); dedicated HIPAA-eligible region).
- **KR-FSS-regulated** tenants in pack-kr: 1.15× base (extended audit retention 5y per KR commercial code).
- **Single-region packs** (pack-kr, pack-jp, pack-sg): 1.0× base (no DR multiplier).

## Budget + Alert Thresholds

| Metric | Threshold | Action |
|---|---|---|
| Monthly cost (per pack region) | within 90% of forecast | normal |
| 90% < cost < 110% | yellow | FinOps + ops-sre-reliability review |
| 110% < cost < 130% | orange | FinOps + leadership review; review autoscale + capacity-model |
| cost > 130% | red; budget breach incident | engage ops-finops + axis-cell-substrate |
| Cells in pack utilization band | [40%, 80%] | normal |
| Cells outside band | yellow | trigger rebalance evaluation |
| Warm-pool nodes per pack | ≥ 2 | normal; < 2 fires page (T-D-01 in threat-model) |
| Per-cell compute cost projection (highest spender cell) | within 5× median cell | normal |
| Per-cell cost > 10× median | yellow; engage scheduler on imbalance | tenant-facing dashboard surfaces self-overage |

## FinOps SLI

| SLI | Target | Burn-rate alert |
|---|---|---|
| Monthly cost / N_cells (unit-economic) | within 5% of forecast | 6× burn over 6h |
| Warm pool size (per pack) | ≥ 2 | 14.4× burn over 1h (catches pool depletion) |
| Cell utilization band compliance | [40%, 80%] for ≥ 80% of cells per pack | hourly |
| Spot-vs-on-demand ratio | ≥ 60% spot for warm-pool + scheduler workers | informational |

## Cost-Optimisation Levers

| Lever | Estimated saving | Trade-off |
|---|---|---|
| Pack-cells together at higher density (raise band to [50%, 90%]) | 10–15% compute | More aggressive rebalance churn |
| Spot-instance fleet for warm-pool nodes | 30–50% compute | Spot eviction → HA fallback to on-demand |
| OCI committed-use discounts (1y / 3y) | 20–40% compute | Vendor lock-in window |
| Postgres horizontal partitioning per pack | reduces per-pack PV cost as scale grows | More operational complexity |
| Cell decommission soft-delete window 30d → 14d | 5–10% cold-cell cost | Tighter recovery window |
| Cluster API management cluster sharing across DR pair | reduces management overhead | More blast-radius on management cluster outage |

## Verification

- `cargo run -p oya-dev-cli -- gate validate cost-budget --microservice cell` — exit 0; current spend within 110%.
- Monthly FinOps review: actual vs forecast; lever decisions logged.
- Quarterly: capacity-model + cost-budget refresh.

## References

- `microservices/cell/capacity-model.md`.
- `microservices/cell/multi-region.md`.
- `microservices/cell/policy/data-residency.md` (per-pack retention multipliers).
- OCI pricing — `oracle.com/cloud/pricing/`.
- Kubernetes Cluster API + OKE scaling guides.
- FinOps Foundation framework — `finops.org`.
