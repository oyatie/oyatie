---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: tenancy
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-tenancy + ops-sre-reliability
deciders: ops-finops, axis-tenancy, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/tenancy/capacity-model.md
  - microservices/tenancy/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (tenancy µservice)

## Purpose

Track the tenancy µservice's monthly cloud cost across infrastructure (compute + storage + network), per Layer-A + Layer-B component, per pack region; surface budget breach via the `check-cost-budget` LEAN lane. Numbers cite OCI public pricing (2026-05-17); verify-at-deploy markers called out where vendor pricing may have moved.

## Cost Categories

| Category | What | OCI pricing reference |
|---|---|---|
| Compute (VM.Standard / OKE node) | Postgres primary + sync + async replicas; Citus coordinator + workers; Patroni; tenancy crates' worker / rest / app pods | `oracle.com/cloud/compute/pricing/` |
| Block storage (PV) | Postgres data + WAL; Citus worker storage; Patroni DCS | `oracle.com/cloud/storage/block-volume/pricing/` |
| Object storage (S3-compatible) | Postgres archive WAL; pg_dump snapshots; audit-chain seal archive | `oracle.com/cloud/storage/object-storage/pricing/` |
| Network egress | Cross-AZ Patroni replication; cross-region DR replication (intra-pack only); public-status-page; auditor JIT reads | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-pack KMS keyring for AES-256-GCM encryption-at-rest + Ed25519 audit-chain seal keys | `oracle.com/security/key-management/pricing/` |
| OpenBao (sub-processor) | Tenant-resolver service + JWT signing key management + DBA JIT elevation | upstream `cloud-secrets` µservice |
| Valkey | Tenant-validate cache + cell-assignment cache | OKE pod cost (sized in capacity-model) |
| Load balancer | Per-pack Istio gateway + public ingress | `oracle.com/cloud/networking/load-balancing/pricing/` |
| Observability-on-tenancy | tenancy self-SLO ingest into observability cluster (small) | inherited from observability µservice's cost-budget |

## Per-Component Monthly Cost (XS tier, single pack-kr region, M01 launch)

Per `capacity-model.md` §"Worked example: oyatie XS tier (M01 launch; 20 tenants pack-kr-only)".

| Component | Replicas × instance-type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| Postgres primary | 1 × VM.Standard.E4 8-core, 64 GB | $145 | $50 PV (data) + $20 PV (WAL) | $215 |
| Postgres sync replicas | 2 × VM.Standard.E4 8-core, 64 GB | $290 | $100 PV (data) + $40 PV (WAL) | $430 |
| Citus coordinator | 1 × VM.Standard.E4 8-core, 64 GB | $145 | $30 PV | $175 |
| Citus workers | 4 × VM.Standard.E4 4-core, 32 GB | $290 | $200 PV total | $490 |
| Patroni DCS (etcd cluster) | 3 × VM.Standard.E4 2-core, 16 GB | $108 | $15 PV total | $123 |
| Patroni REST controllers (sidecars) | (collocated with Postgres pods) | – | – | – |
| Valkey (tenant-validate cache) | 3 × VM.Standard.E4 2-core, 16 GB | $108 | – | $108 |
| Valkey (cell-assignment cache) | 2 × VM.Standard.E4 1-core, 8 GB | $36 | – | $36 |
| `tenancy-tenant-lifecycle-rest` | 3 × VM.Standard.E4 2-core, 4 GB | $108 | – | $108 |
| `tenancy-tenant-lifecycle-worker` | 2 × VM.Standard.E4 2-core, 4 GB | $72 | – | $72 |
| `tenancy-tenant-lifecycle-app` | 2 × VM.Standard.E4 2-core, 4 GB | $72 | – | $72 |
| `tenancy-isolation-policy-rest` | 3 × VM.Standard.E4 2-core, 4 GB | $108 | – | $108 |
| `tenancy-isolation-policy-worker` | 2 × VM.Standard.E4 2-core, 4 GB | $72 | – | $72 |
| `tenancy-isolation-policy-app` | 2 × VM.Standard.E4 2-core, 4 GB | $72 | – | $72 |
| `tenancy-cell-assignment-worker` | 2 × VM.Standard.E4 2-core, 4 GB | $72 | – | $72 |
| `tenancy-cell-assignment-app` | 2 × VM.Standard.E4 2-core, 4 GB | $72 | – | $72 |
| `tenancy-dsr-cascade-rest` | 2 × VM.Standard.E4 2-core, 4 GB | $72 | – | $72 |
| `tenancy-dsr-cascade-worker` | 2 × VM.Standard.E4 2-core, 4 GB | $72 | – | $72 |
| `tenancy-dsr-cascade-app` | 2 × VM.Standard.E4 2-core, 4 GB | $72 | – | $72 |
| Object-storage (Postgres WAL archive + audit-chain) | – | – | $50 (1.9 TB; 30d retention) | $50 |
| KMS keyring (per-pack) | – | $10 | – | $10 |
| Load balancer (per-pack Istio gateway + public LB) | – | $20 | – | $20 |
| **XS tier total per pack region** | | **~$2334** | **~$505** | **~$2839 / month** |

Verify-at-deploy: OCI pricing changes; reconfirm against `oracle.com/cloud/pricing/` at deploy time. Buffer 15% for rate increases + 20% for actual-vs-forecast.

## Per-Scale-Tier Forecast

| Scale tier | N_tenants | Monthly cost per pack region | Notes |
|---|---|---|---|
| XS (M01 launch; 20 tenants) | 20 | ~$2900 | active: pack-kr |
| S (~100 tenants) | 100 | ~$8k | active: pack-kr + pack-eu + pack-us (3 packs) |
| M (~1000 tenants) | 1000 | ~$35k | typically 5 active packs |
| L (~10000 tenants) | 10000 | ~$300k | all 11 packs + DR per pack |

## Per-Pack Multipliers

- **DR pair packs** (pack-eu, pack-us, pack-au, pack-in, pack-br, pack-ae, pack-ksa): 1.0× primary + 0.6× warm-standby per `multi-region.md`.
- **HIPAA pack** (pack-us-healthcare): 1.4× base (extended retention 6y per HIPAA §164.316(b)(2); dedicated HIPAA-eligible region; isolated from non-HC pack-us).
- **KR-FSS-regulated** tenants in pack-kr: 1.2× base (retention 5y per KR commercial code; KMS-in-KR).
- **Single-region packs** (pack-kr, pack-jp, pack-sg): 1.0× base (no DR multiplier).

## Budget + Alert Thresholds

| Metric | Threshold | Action |
|---|---|---|
| Monthly cost (per pack region) | within 90% of forecast | normal |
| 90% < cost < 110% | yellow alert | FinOps + ops-sre-reliability review |
| 110% < cost < 130% | orange alert | FinOps + leadership review; review autoscale + capacity-model |
| cost > 130% | red alert; budget breach incident | engage ops-finops + axis-tenancy; consider per-tenant rate-limit tightening |
| Per-tenant cost (highest spender) | within 5× median tenant | normal |
| Per-tenant cost > 10× median | yellow; engage tenant on excess-cardinality discipline | tenant-facing dashboard surfaces self-overage |
| Patroni replication lag | > 5s | warning; verify network |
| Patroni replication lag | > 60s | Sev-2; verify Patroni health + cluster topology |

## FinOps SLI

| SLI | Target | Burn-rate alert |
|---|---|---|
| Monthly cost / N_tenants (unit-economic) | within 5% of forecast | 6× burn over 6h |
| Storage growth / day (avg over last 7d) | within forecast | 14.4× burn over 1h (catches runaway-cardinality on shard placement) |
| Spot-vs-on-demand ratio | tenancy run on-demand only (HA-critical) | informational |

## Cost-Optimisation Levers

| Lever | Estimated saving | Trade-off |
|---|---|---|
| OCI committed-use discounts (1y / 3y) | 20–40% compute | Vendor lock-in window |
| Citus worker right-sizing per tenant load | 5–15% compute | Requires per-pack benchmark refresh |
| Postgres async replicas off for non-DR packs | 20–25% compute | RPO degradation on primary loss |
| Valkey memory cap optimization | 5–10% cache cost | Higher cache miss → Postgres load |
| Archive WAL aggressively (lower retention beyond DR window) | 10–20% storage | DR window matches PR-restore window |
| Patroni DCS via shared cluster (etcd in cloud-k8s) | reduce 3 etcd pods | Tighter coupling to cloud-k8s |

## Verification

- `cargo run -p dev-cli -- gate validate cost-budget --microservice tenancy` — exit 0; current spend within 110%.
- Monthly FinOps review: actual vs forecast; lever decisions logged.
- Quarterly: capacity-model + cost-budget refresh; re-run formulas with current data.

## References

- `microservices/tenancy/capacity-model.md`.
- `microservices/tenancy/multi-region.md`.
- `microservices/tenancy/policy/data-residency.md` (per-pack retention multipliers).
- OCI pricing — `oracle.com/cloud/pricing/`.
- Postgres + Citus + Patroni reference architectures — vendor docs.
- FinOps Foundation framework — `finops.org`.
