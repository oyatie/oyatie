---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: ontology
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-ontology + ops-sre-reliability
deciders: ops-finops, axis-ontology, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/ontology/capacity-model.md
  - microservices/ontology/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (ontology µservice)

## Purpose

Track the ontology µservice's monthly cloud cost across infrastructure (compute + storage + network), per Layer-A + Layer-B component, per pack region; surface budget breach via the `oya-check-cost-budget` LEAN lane. Numbers cite OCI public pricing (2026-05-17) and Postgres/ClickHouse/Cedar reference architectures from `capacity-model.md`; verify-at-deploy markers called out where vendor pricing may have moved.

## Cost Categories

| Category | What | OCI pricing reference |
|---|---|---|
| Compute (VM.Standard / OKE node) | Postgres + Citus + ClickHouse + Cedar + Valkey + Kafka KRaft + Layer-B pods | `oracle.com/cloud/compute/pricing/` |
| Block storage (PV) | Postgres data volumes (Citus shards) + ClickHouse data + Kafka logs + Valkey RDB | `oracle.com/cloud/storage/block-volume/pricing/` |
| Object storage (S3-compatible) | ClickHouse cold-tier + Postgres PITR backups + Kafka tiered storage | `oracle.com/cloud/storage/object-storage/pricing/` |
| Network egress | Cross-µservice mTLS (intra-cluster); status-page; auditor JIT reads | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-pack KMS keyring for SSE + Ed25519 signing | `oracle.com/security/key-management/pricing/` |
| Load balancer | Per-pack Istio gateway | `oracle.com/cloud/networking/load-balancing/pricing/` |
| Cedar policy engine | In-process; no separate licence (Apache-2 Rust SDK) | n/a |

## Per-Component Monthly Cost (XS tier, single pack-kr region, M02b launch)

Per `capacity-model.md` §"Worked example: oyatie XS tier (M02b launch; 20 tenants pack-kr-only)".

| Component | Replicas × instance-type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| Postgres + Citus coordinator | 2 × VM.Standard.E4 8-core | $290 | $50 PV | $340 |
| Postgres + Citus worker | 8 × VM.Standard.E4 16-core | $2320 | $400 PV (shards × 50GB) | $2720 |
| Postgres logical replica (per shard) | 8 × VM.Standard.E4 8-core | $1160 | $400 PV | $1560 |
| Postgres PITR backup (object storage) | – | – | $80 (3.2 TB cold) | $80 |
| ClickHouse shard | 4 × VM.Standard.E4 16-core | $1160 | $300 PV (hot) | $1460 |
| ClickHouse cold-tier (object storage) | – | – | $150 (6 TB warm + cold) | $150 |
| Valkey schema registry cache | 3 × VM.Standard.E4 2-core | $108 | $30 PV | $138 |
| Kafka KRaft broker (outbox) | 3 × VM.Standard.E4 4-core | $216 | $90 PV | $306 |
| Cedar policy engine (in-process; sidecar evaluator option) | sidecar in each `*-rest` pod | $0 (subsumed) | – | $0 |
| `oya-ontology-object-type-registry-app` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-ontology-link-type-registry-app` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-ontology-action-type-registry-app` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-ontology-function-type-registry-app` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-ontology-entity-store-app` | 2 × VM.Standard.E4 4-core | $145 | – | $145 |
| `oya-ontology-link-store-app` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-ontology-function-engine-app` | 4 × VM.Standard.E4 4-core | $290 | – | $290 |
| `oya-ontology-action-engine-app` | 2 × VM.Standard.E4 4-core | $145 | – | $145 |
| `oya-ontology-query-engine-app` | 2 × VM.Standard.E4 4-core | $145 | – | $145 |
| `oya-ontology-agent-gateway-app` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-ontology-audit-chain-app` | 2 × VM.Standard.E4 2-core | $72 | $20 PV (Merkle journal) | $92 |
| Per-µservice rest pods (`*-rest`; ~6 of them) | 12 × VM.Standard.E4 2-core | $432 | – | $432 |
| KMS keyring (per-pack) | – | $5 | – | $5 |
| Load balancer (per-pack Istio gateway) | – | $20 | – | $20 |
| **XS tier total per pack region** | | **~$6810** | **~$1520** | **~$8330 / month** |

Verify-at-deploy: OCI pricing changes; reconfirm against `oracle.com/cloud/pricing/`. Buffer 15% for OCI rate increases + 20% for actual-vs-forecast.

## Per-Scale-Tier Forecast

| Scale tier | N_tenants | Monthly cost per pack region | Notes |
|---|---|---|---|
| XS (M02b launch; 20 tenants) | 20 | ~$8.5k | active: pack-kr |
| S (~100 tenants) | 100 | ~$30k | active: pack-kr + pack-eu + pack-us (3 packs) |
| M (~1000 tenants) | 1000 | ~$180k | typically 5 active packs |
| L (~10000 tenants) | 10000 | ~$1.8M | all 11 packs + multi-region per pack |

## Per-Pack Multipliers

- **DR pair packs** (pack-eu, pack-us, pack-au, pack-in, pack-br, pack-ae, pack-ksa): 1.0× primary + 0.6× warm-standby.
- **HIPAA pack** (pack-us-healthcare): 1.5× base (extended retention 6y per HIPAA §164.316(b)(2) + dedicated HIPAA-eligible region + HSM-backed Ed25519 signing + isolated from non-HC pack-us).
- **KR-FSS-regulated** tenants in pack-kr: 1.3× base (retention 5y per KR commercial code; KMS-in-KR; KR-FSS quarterly audit overhead).
- **Single-region packs** (pack-kr, pack-jp, pack-sg): 1.0× base (no DR multiplier).
- **High-LLM-traffic** tenants: variable; agent gateway scales linearly with LLM session count.

## Budget + Alert Thresholds

| Metric | Threshold | Action |
|---|---|---|
| Monthly cost (per pack region) | within 90 % of forecast | normal |
| 90 % < cost < 110 % | yellow alert | FinOps + ops-sre-reliability review |
| 110 % < cost < 130 % | orange alert | FinOps + leadership review; review autoscale + capacity-model |
| cost > 130 % | red alert; budget breach incident | engage ops-finops + axis-ontology; consider per-tenant rate-limit tightening |
| Per-tenant cost projection (highest spender) | within 5× median tenant | normal |
| Per-tenant cost > 10× median | yellow; engage tenant on Object Type cardinality + LLM tool-call discipline | tenant-facing dashboard surfaces self-overage |

## FinOps SLI

| SLI | Target | Burn-rate alert |
|---|---|---|
| Monthly cost / N_tenants (unit-economic) | within 5 % of forecast | 6× burn over 6h |
| Storage growth / day (avg over last 7d) | within forecast | 14.4× burn over 1h (catches runaway Object Type creation) |
| Spot-vs-on-demand ratio | ≥ 70 % spot for stateless components | informational |
| Per-µservice `cargo bloat` budget | per release | LEAN lane on binary size growth |

## Cost-Optimisation Levers

| Lever | Estimated saving | Trade-off |
|---|---|---|
| Increase ClickHouse compression aggressiveness (zstd vs lz4) | 15–25 % ClickHouse storage | More compute on compactor |
| Citus shard pre-split with smaller default size | 10–15 % I/O | More rebalance overhead |
| Spot-instance fleet for stateless Layer-B (function-engine, agent-gateway, *-rest) | 30–50 % compute | Spot eviction recovery via HA |
| OCI committed-use discounts (1y / 3y) | 20–40 % compute | Vendor lock-in window |
| Postgres archive cold-tier earlier (90d → 30d threshold) | 5–10 % storage | Slower historical queries |
| Per-tenant Object Type cardinality budget enforcement | 5–20 % Postgres compute | Tenant disruption if too aggressive |
| Cross-region replication: async vs sync | varies | RPO trade-off |
| Cache Function results in Valkey aggressively (5min TTL → 30min for non-volatile) | 10–20 % Function-engine compute | Stale reads if tenant edits |
| Agent gateway tool-call result memoisation (deterministic Functions only) | 20–40 % LLM agent loop cost | Cache invalidation complexity |

## Verification

- `cargo run -p oya-dev-cli -- gate validate cost-budget --microservice ontology` — exit 0; current spend within 110 %.
- Monthly FinOps review: actual vs forecast; lever decisions logged.
- Quarterly: capacity-model + cost-budget refresh; re-run formulas with current data.

## References

- `microservices/ontology/capacity-model.md`.
- `microservices/ontology/multi-region.md`.
- `microservices/ontology/policy/data-residency.md` (per-pack retention multipliers).
- OCI pricing — `oracle.com/cloud/pricing/`.
- Postgres + Citus capacity guides — `docs.citusdata.com`.
- ClickHouse sizing — `clickhouse.com/docs/en/operations/tips`.
- FinOps Foundation framework — `finops.org`.
