---
doc_class: CostBudget
title: Cost Budget + FinOps Posture (foundry-supervisor)
microservice: foundry-supervisor
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-foundry-control-plane + ops-sre-reliability
deciders: ops-finops, axis-foundry-control-plane, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/intelligence-supervisor/capacity-model.md
  - microservices/intelligence-supervisor/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (foundry-supervisor µservice)

## Purpose

Track the foundry-supervisor µservice's monthly cloud cost across the substrate (Postgres HA, Valkey Cluster, Operator pods, REST + worker pods) per pack region; surface budget breach via the `oya-check-cost-budget` LEAN lane. Cites OCI public pricing (2026-05-17); verify-at-deploy markers where vendor pricing may have moved.

## Cost Categories

| Category | What | OCI pricing reference |
|---|---|---|
| Compute (VM.Standard / OKE node) | Postgres + Valkey Cluster + Operator + REST + worker pods | `oracle.com/cloud/compute/pricing/` |
| Block storage (PV) | Postgres data + WAL; Valkey AOF | `oracle.com/cloud/storage/block-volume/pricing/` |
| Object storage (S3-compatible) | Postgres WAL archive (long-term) | `oracle.com/cloud/storage/object-storage/pricing/` |
| Network egress | Cross-µservice mTLS (Mimir, evidence, runtime); cross-region replication intra-pack | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-pack KMS for SSE on Postgres + S3 archives; Ed25519 signing key | `oracle.com/security/key-management/pricing/` |
| Load balancer | Per-pack Istio gateway | `oracle.com/cloud/networking/load-balancing/pricing/` |
| OpenBao integration | per-pack OpenBao instance share (operated by cloud-secrets) | sub-processor cost |

## Per-Component Monthly Cost (XS tier, single pack-kr region, M01 launch)

Per `capacity-model.md` §"Worked example: XS tier (M01 launch; 20 tenants pack-kr-only; ~1000 capabilities, ~5000 agents)".

| Component | Replicas × instance-type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| Postgres primary | 1 × VM.Standard.E4 8-core | $145 | $80 PV (1 TB block) | $225 |
| Postgres DR-pair replica (pack-eu-style; pack-kr is single-region so this is intra-pack only) | 1 × VM.Standard.E4 8-core | $145 | $80 PV | $225 (n/a for pack-kr; included for DR-pair packs) |
| Postgres connection pooler (PgBouncer) | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Valkey Cluster (3 shards × 2 replicas) | 6 × VM.Standard.E4 2-core | $216 | $30 PV (AOF) | $246 |
| Kubernetes Operator pods | 3 × VM.Standard.E4 2-core | $108 | – | $108 |
| Supervisor REST | 3 × VM.Standard.E4 2-core | $108 | – | $108 |
| Supervisor worker (reconcile + drain) | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Supervisor app (composition root) | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Postgres WAL archive (S3 standard tier) | – | – | $40 (1.5 TB hot) | $40 |
| Postgres WAL archive (S3 archive tier; 24mo cold) | – | – | $30 (12 TB archive) | $30 |
| KMS (per-pack) | – | $5 | – | $5 |
| Load balancer (per-pack) | – | $10 | – | $10 |
| OpenBao integration share | – | $5 | – | $5 |
| **XS tier total per pack region (single-region)** | | **~$813** | **~$180** | **~$993 / month** |

For DR-pair packs (pack-eu, pack-us, pack-au, pack-in, pack-br, pack-ae, pack-ksa), add 0.6× warm-standby compute per `multi-region.md`. Verify-at-deploy: OCI pricing changes; reconfirm at deploy time; 15% buffer for vendor rate increases + 20% for actual-vs-forecast.

## Per-Scale-Tier Forecast

| Scale tier | N_tenants | Approx capabilities | Approx agents | Monthly cost per pack region |
|---|---|---|---|---|
| XS (M01 launch; 20 tenants) | 20 | ~1000 | ~5000 | ~$1k |
| S (~100 tenants) | 100 | ~5000 | ~25k | ~$3.5k |
| M (~1000 tenants) | 1000 | ~50k | ~250k | ~$18k |
| L (~10000 tenants) | 10000 | ~500k | ~2.5M | ~$140k |

## Per-Pack Multipliers

- DR-pair packs: 1.0× primary + 0.6× warm-standby.
- HIPAA pack (pack-us-healthcare): 1.4× base (6 y audit retention + dedicated HIPAA-eligible region + isolation).
- KR-FSS-regulated tenants in pack-kr: 1.2× base (5 y audit + KMS-in-KR).
- Single-region packs (pack-kr, pack-jp, pack-sg): 1.0× base (no DR multiplier).

## Budget + Alert Thresholds

| Metric | Threshold | Action |
|---|---|---|
| Monthly cost (per pack region) | within 90% of forecast | normal |
| 90% < cost < 110% | yellow | FinOps + ops-sre-reliability review |
| 110% < cost < 130% | orange | FinOps + leadership review; review autoscale + capacity-model |
| cost > 130% | red; budget breach incident | engage ops-finops + axis-foundry-control-plane |
| Per-tenant cost projection (highest spender) | within 5× median tenant | normal |
| Per-tenant cost > 10× median | yellow; engage tenant on capability + agent-count discipline | tenant-facing dashboard surfaces self-overage |

## FinOps SLI

| SLI | Target | Burn-rate alert |
|---|---|---|
| Monthly cost / N_tenants (unit-economic) | within 5% of forecast | 6× burn over 6h |
| Postgres storage growth / day (7d avg) | within forecast | 14.4× over 1h |
| Valkey memory % per shard | < 70% | 6× over 6h |
| Operator reconcile rate (anomaly = high; cost-runaway proxy) | within 2× baseline | 6× over 6h |

## Cost-Optimisation Levers

| Lever | Estimated saving | Trade-off |
|---|---|---|
| Postgres compression on cold partitions | 20–30% storage | Compute CPU on compactor |
| Reduce supervision-event retention in Valkey Streams (Redis wire-compat) (1h → 30min) | 5–10% Valkey | Replay-window shorter |
| Spot-instance fleet for stateless REST + worker | 30–50% compute | Spot eviction recovery via HA |
| OCI committed-use discounts (1y / 3y) | 20–40% compute | Vendor lock-in window |
| Postgres replica only (drop DR-pair) for non-critical packs | 30–40% pack compute | RPO degrades to single-region availability |
| Per-tenant agent-count budget enforcement | varies | Tenant disruption if too aggressive |

## Verification

- `cargo run -p oya-dev-cli -- gate validate cost-budget --microservice foundry-supervisor` — exit 0; current spend within 110%.
- Monthly FinOps review.
- Quarterly capacity-model + cost-budget refresh.

## References

- `microservices/intelligence-supervisor/capacity-model.md`.
- `microservices/intelligence-supervisor/multi-region.md`.
- `microservices/intelligence-supervisor/policy/data-residency.md` (retention multipliers).
- OCI pricing — `oracle.com/cloud/pricing/`.
- PostgreSQL HA reference — `postgresql.org/docs/current/high-availability.html`.
- Valkey Cluster reference — `redis.io/docs/management/scaling/`.
- FinOps Foundation — `finops.org`.
