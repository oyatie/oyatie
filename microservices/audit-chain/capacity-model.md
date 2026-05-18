---
doc_class: CapacityModel
title: Capacity Sizing Model (audit-chain)
microservice: audit-chain
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-audit-chain
deciders: ops-sre-reliability, axis-audit-chain, council-architecture
related_adrs: [ADR-0117, ADR-0028, ADR-0131]
related_artifacts:
  - microservices/audit-chain/cost-budget.md
  - microservices/audit-chain/multi-region.md
  - microservices/audit-chain/policy/data-residency.md (retention)
  - /specs/audit-chain-merkle-ed25519.json §"emission_rate"
review_cadence: quarterly + on every component-replica-set change
doc_status: published
---

# Capacity Sizing Model (audit-chain µservice)

## Purpose

Sizing formulas for every component (emission-rest, sealing-worker, Postgres, S3 WORM, HSM utilisation, verification-rest, query-rest, retention-cascade-worker). Drives `cost-budget.md` + `multi-region.md`. Numbers verified at deploy; re-validate quarterly.

## Inputs

| Input | Variable | Source |
|---|---|---|
| Active tenants per pack | `N_tenants` | tenancy µservice tenant-resolver |
| Per-tenant emission rate (events/sec) | `E_events_per_sec_per_tenant` | tenant_scope-bound (trial: 10/s; production: 100/s; sandbox: 5/s; internal: 500/s) |
| Per-tenant active microservices emitting | `M_per_tenant` | typically 1.2× the catalog size — many µservices emit per tenant |
| Average event payload size (bytes) | `P_bytes` | 1024 default; 1 MB cap; tenant-tunable |
| Period bucket (seconds) | `T_period` | 1s default; per-tenant override possible |
| Retention window (days) | `R_days` | per-pack matrix (default 730d; HIPAA 2190d; SAMA-finance 3650d) |
| Number of tenant partitions per pack | `S_shards` | `ceil(N_tenants × E_events_per_sec_per_tenant / 50000)` for per-shard 50k events/s cap |

## Emission-Rest Sizing

```
total_events_per_sec = N_tenants × E_events_per_sec_per_tenant

emission_rest_replicas = max(3, ceil(total_events_per_sec / 10000) × 1.3 buffer)
# Each replica handles 10k events/s sustained at p99 100ms.
```

## Sealing-Worker Sizing

```
sealing_worker_replicas_per_shard = 2  # 1 leader + 1 warm standby per tenant_partition shard

total_sealing_replicas = S_shards × 2
```

Each shard's sealing-worker computes one Merkle tree per period (1s) over up to ~50k events. Merkle tree build is O(n log n); for n = 50k, ≤ 200ms CPU on a 4-core node (per benchmark; verified at deploy). HSM signing call adds ~50ms latency.

## Postgres Sizing

```
events_index_row_size = 200 bytes  # (event_id, tenant_id, source_microservice, principal_hash, period_id, payload_sha, emitted_at, data_class)
events_index_per_day_bytes = total_events_per_sec × 86400 × events_index_row_size

storage_index_R_days = events_index_per_day_bytes × R_days

seal_record_per_day_bytes = N_tenants × (86400 / T_period) × 500 bytes  # SealRecord size: root_hash + signature + signer + period_id + chain_link
storage_seal_R_days = seal_record_per_day_bytes × R_days
```

Postgres index total = `storage_index + storage_seal`; for XS tier (20 tenants × 50 events/s × 730d retention) ≈ ~600 GB; HA primary + replica = 2× = 1.2 TB.

## S3 WORM Sizing

```
raw_blob_per_day_bytes = total_events_per_sec × 86400 × P_bytes  # raw event payload
merkle_blob_per_day_bytes = N_tenants × (86400 / T_period) × 100KB  # Merkle tree blob size

storage_s3_hot = (raw_blob + merkle_blob) × 30  # 30d hot
storage_s3_warm = (raw_blob + merkle_blob) × 180 × 0.5  # 6mo IA tier (50% smaller; chunk-merged)
storage_s3_cold = (raw_blob + merkle_blob) × (R_days - 30 - 180) × 0.3  # archive
```

For XS tier (20 tenants, 50 events/s avg, 1KB payload, 730d retention):
- raw_blob = 50 × 86400 × 1024 ≈ 4.4 GB/day; over 730d ≈ 3.2 TB.
- merkle_blob = 20 × 86400 × 100KB ≈ 173 GB/day; over 730d ≈ 126 TB.
- **Note: Merkle blob is the dominant storage line item** because every period generates a tree even if zero events emit. Mitigation: trial-tier tenants can opt into larger periods (5s default for trial vs 1s for production).

After tier mixing (30d hot, 6mo IA, rest archive):
- Hot: 30 × ~178 GB ≈ 5.3 TB.
- IA: 180 × ~89 GB ≈ 16 TB.
- Cold: 520 × ~53 GB ≈ 28 TB.
- Total: ~50 TB per pack region.

## HSM Sizing

```
hsm_ops_per_sec = N_tenants × (1 / T_period)  # one sign per (tenant, period)
```

For XS tier: 20 tenants × 1/s = 20 ops/s. OCI Cloud-HSM partition baseline: ≥ 500 ops/s. Headroom: 25× at XS tier.

At L tier (10k tenants): 10000 ops/s ⇒ partition saturated; multi-partition fanout required or partition upsize.

| Tier | hsm_ops_per_sec | Partition strategy |
|---|---|---|
| XS (20 tenants) | 20 | 1 partition; 25× headroom |
| S (100 tenants) | 100 | 1 partition; 5× headroom |
| M (1000 tenants) | 1000 | 1 partition; 50% utilisation |
| L (10000 tenants) | 10000 | 4 partitions per pack; fanout by shard |

## Verification-Rest Sizing

```
verify_qps = expected_external_verifications_per_sec  # tenant + auditor reads
            + audit_query_internal_qps  # CI lane reads

verification_rest_replicas = max(2, ceil(verify_qps / 200) × 1.3 buffer)
```

verify() is pure-function over published roots + Merkle proofs; CPU-bound at ~5ms per call; ≤ 200 qps per pod sustainable.

## Query-Rest Sizing

```
query_qps = tenant_forensic_qps + auditor_export_qps

query_rest_replicas = max(2, ceil(query_qps / 50) × 1.3 buffer)
```

Query is Postgres-indexed range scan; bounded latency depends on index health.

## Retention-Cascade-Worker Sizing

```
retention_worker_replicas = max(2, ceil(N_tenants / 1000))
```

Worker runs daily sweep per tenant; per-tenant sweep is ≤ 1min on healthy index.

## Reference Baselines

| Scale tier | N_tenants | E/s/tenant | Total events/s | emission-rest | sealing-worker | HSM partitions | Storage / pack |
|---|---|---|---|---|---|---|---|
| XS (M01 launch) | 20 | 50 | 1k | 3 | 8 (4 shards × 2) | 1 | ~50 TB |
| S | 100 | 100 | 10k | 4 | 16 (8 shards × 2) | 1 | ~250 TB |
| M | 1000 | 100 | 100k | 13 | 40 (20 shards × 2) | 1 | ~2.5 PB |
| L | 10000 | 100 | 1M | 130 | 200 (100 shards × 2) | 4 per pack | ~25 PB |

Per-pack-region multiplier: each pack independent. DR-pair pack: 1.0× primary + 0.6× warm-standby.

## Headroom + Burst

- Pre-warmed pool: 2 standby emission-rest pods + 1 standby per-shard sealing-worker.
- HPA: emission-rest on CPU > 70% OR queue depth > 100 events.
- Sustained burst absorption: 5× normal rate for ≤ 60s (HPA scales in window).

## Storage Costs (per pack region; cites OCI 2026-05-17)

```
OCI object-storage standard: $0.0255 / GB / month (hot)
OCI object-storage IA: $0.01 / GB / month (warm)
OCI archive: $0.0025 / GB / month (cold)
OCI Object Lock Compliance mode: same per-tier; lock-fee absorbed in tier rate
```

Per XS tier (~50 TB total mix): ~$560/month (matches `cost-budget.md`).

## Worked Example: XS tier (M01 launch, pack-kr, 20 tenants)

```
N_tenants = 20
E_events_per_sec_per_tenant = 50  # production tenants mostly
total_events_per_sec = 1000
M_per_tenant = ~30 (oyatie µservice catalog × per-tenant emission ratio)

emission_rest_replicas = max(3, ceil(1000/10000) × 1.3) = 3 (rounded up from formula minimum)
S_shards = ceil(20 × 50 / 50000) = 1, but 4 minimum for early-stage isolation
sealing_worker_replicas = 4 × 2 = 8
hsm_ops_per_sec = 20
verification_rest_replicas = 3 (provisioned higher than formula for hot-availability)
query_rest_replicas = 3
retention_cascade_worker_replicas = 2

Postgres index: ~600 GB (HA = 1.2 TB)
S3: ~50 TB across hot/IA/cold tiers
HSM: 1 partition at ~4% utilization
```

## Verification

- `cargo run -p oya-dev-cli -- gate validate capacity-conformance --microservice audit-chain` — exit 0; deployed replicas ≥ formula minimums.
- Quarterly capacity review.
- Annual upstream-doc refresh.

## References

- `microservices/audit-chain/cost-budget.md`.
- `microservices/audit-chain/multi-region.md`.
- `microservices/audit-chain/policy/data-residency.md` (retention).
- `/specs/audit-chain-merkle-ed25519.json` §"emission_rate".
- OCI Cloud-HSM published throughput — `docs.oracle.com/en-us/iaas/Content/KeyManagement/`.
- OCI object-storage pricing.
- RFC 6962 (Merkle-tree-build computational complexity).
