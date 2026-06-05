---
doc_class: CapacityModel
title: Capacity Sizing Model
microservice: sheets
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-sheets
deciders: ops-sre-reliability, axis-sheets, council-architecture
related_adrs: [ADR-0065, ADR-0117, ADR-0135, ADR-0131]
related_artifacts:
  - microservices/sheets/cost-budget.md
  - microservices/sheets/multi-region.md
  - microservices/sheets/policy/data-residency.md
review_cadence: quarterly + on every component-replica-set change
doc_status: published
---

# Capacity Sizing Model (sheets µservice)

## Purpose

Sizing formulas + reference-architecture baseline numbers for every Layer-A (CDN + WAF + Postgres + Valkey + WebSocket gateway + Arrow/Parquet OCI Object Storage + S3) and Layer-B (cell-grid-rest + collab-crdt-worker + recalc-engine-worker + xlsx-export-worker + license-gate-cedar) component. Drives `cost-budget.md` and `multi-region.md`.

## Inputs

| Input | Variable | Source |
|---|---|---|
| Active tenants | `N_tenants` | OpenBao tenant-resolver |
| Per-tenant active workbook sessions | `S_active_per_tenant` | Sheets REST metrics |
| Per-tenant workbook sessions opened per day | `S_opens_per_day_per_tenant` | telemetry |
| Per-tenant avg saves per day | `Save_per_day_per_tenant` | telemetry |
| Per-tenant avg cell-edit ops per second | `Edit_per_sec_per_tenant` | WS gateway metrics |
| Per-tenant avg recalc invocations per minute | `Recalc_per_min_per_tenant` | recalc-engine metrics |
| Per-tenant avg AI-formula invocations per day | `AI_per_day_per_tenant` | telemetry |
| Per-tenant avg XLSX import jobs per day | `XlsxImport_per_day_per_tenant` | import worker metrics |
| Per-tenant avg XLSX export jobs per day | `XlsxExport_per_day_per_tenant` | export worker metrics |
| Per-tenant avg connected-sheets refresh per day | `ConnectedRefresh_per_day_per_tenant` | connected-sheets worker metrics |
| Per-tenant avg seats | `Seats_per_tenant` | tenancy SDK |
| Per-tenant avg workbooks | `Workbook_per_tenant` | telemetry |
| Per-tenant avg cells per workbook | `Cells_per_workbook` | telemetry |

## Core Formulae

### Throughput

```
total_active_sessions          = N_tenants × S_active_per_tenant
total_sessions_per_day         = N_tenants × S_opens_per_day_per_tenant
total_saves_per_sec            = N_tenants × Save_per_day_per_tenant / 86400
total_edits_per_sec            = N_tenants × Edit_per_sec_per_tenant
total_recalcs_per_sec          = N_tenants × Recalc_per_min_per_tenant / 60
total_ws_connections           = total_active_sessions × 1.2
total_ai_invocations_per_day   = N_tenants × AI_per_day_per_tenant
total_xlsx_import_per_day      = N_tenants × XlsxImport_per_day_per_tenant
total_xlsx_export_per_day      = N_tenants × XlsxExport_per_day_per_tenant
total_connected_refresh_per_day= N_tenants × ConnectedRefresh_per_day_per_tenant
```

### Storage

```
workbook_per_row               ≈ 4KB (metadata; cells in separate columnar / hot OLTP tables)
cell_row_hot_per_row           ≈ 256B (cell ref + value + format hash; Postgres OLTP tier)
cell_block_columnar_per_block  ≈ 100KB Arrow block (1000 cells × 100B avg; cold analytical tier)
edit_log_per_row               ≈ 256B
share_acl_per_row              ≈ 256B
comment_per_row                ≈ 1KB
version_snapshot_per_row       ≈ 50KB (compressed workbook snapshot)
audit_seal_per_row             ≈ 256B
ai_formula_per_row             ≈ 5KB (prompt + completion + metadata)
xlsx_upload_quarantined        ≈ 1MB-200MB per upload (size cap 200MB)
xlsx_export_output             ≈ 100KB-50MB per export

postgres_storage_per_day       = (total_sessions_per_day × workbook_per_row)
                              + (total_edits_per_sec × 86400 × edit_log_per_row × sampling_rate)
                              + (total_saves_per_sec × 86400 × audit_seal_per_row)
                              + (total_ai_invocations_per_day × ai_formula_per_row)

cells_hot_per_tenant           = Workbook_per_tenant × min(Cells_per_workbook, 100_000)
cells_cold_per_tenant          = Workbook_per_tenant × max(0, Cells_per_workbook - 100_000)
                                  (per ADR-SHEETS-0003 100k-cell threshold)

arrow_parquet_storage          = N_tenants × cells_cold_per_tenant × 100B / 0.3
                                  (Parquet compression ratio ~30% of raw)

s3_snapshot_storage            = N_tenants × Workbook_per_tenant × version_snapshot_per_row × N_versions

valkey_state_per_session        ≈ 80KB (CRDT state + cursor + presence + recalc-progress)
valkey_total                    = total_active_sessions × valkey_state_per_session × 2 (HA replication)
```

### CDN

```
wasm_bundle_size_per_release   ≈ 10MB (gzip; Leptos WASM + design-system + cell-grid)
asset_chunks_per_release       ≈ 60 chunks
cdn_egress_per_session_open    ≈ 14MB (first load + spec schema)
cdn_egress_per_session_resume  ≈ 200KB (HMR-style delta)
total_cdn_egress_per_day       = (total_sessions_per_day × cdn_egress_per_session_open)
                              + (total_active_sessions × 24 × cdn_egress_per_session_resume / 4)
```

## Per-Component Replica Formulae

```
postgres_coordinator_replicas       = 2  (HA primary + standby; always)
postgres_worker_replicas            = ceil(total_active_sessions / 50_000) × 1.2 buffer
postgres_read_replica_replicas      = postgres_worker_replicas × 1
valkey_sentinel_replicas             = 3  (quorum)

cell_grid_rest_replicas             = max(2, ceil(qps_rest / 500)) × 1.2
collab_crdt_worker_replicas         = max(3, ceil(total_ws_connections / 30_000)) × 1.5
recalc_engine_worker_replicas       = max(2, ceil(total_recalcs_per_sec / 50)) × 1.3
xlsx_export_worker_replicas         = max(2, ceil(total_xlsx_export_per_day / 86400 × 60)) × 1.5
license_gate_cedar_replicas         = max(2, ceil(qps_license / 200)) × 1.2
cell_grid_app_replicas              = 2  (HA composition root)

clamav_sidecar_replicas             = max(1, ceil(total_xlsx_import_per_day / 86400 × 30))
opswat_sidecar_replicas             = max(1, ceil(total_xlsx_import_per_day / 86400 × 30))
```

## Reference-Architecture Baselines

| Scale tier | N_tenants | total_active_sessions | total_ws_connections | Postgres replica counts | WS gateway count | Recalc worker count | XLSX export worker count |
|---|---|---|---|---|---|---|---|
| **XS** (M03-launch; ~5-20 tenants) | 20 | 100 | 120 | coord=2, worker=4, replica=4 | 3 | 2 | 2 |
| **S** (~100 tenants) | 100 | 1,000 | 1,200 | coord=2, worker=4, replica=4 | 3 | 2 | 2 |
| **M** (~1k tenants) | 1000 | 10,000 | 12,000 | coord=2, worker=8, replica=8 | 6 | 5 | 4 |
| **L** (~10k tenants) | 10000 | 100,000 | 120,000 | coord=2, worker=40, replica=40 | 40 | 30 | 20 |
| **XL** (~100k tenants; hyperscaler) | 100000 | 1,000,000 | 1,200,000 | coord=2, worker=400, replica=400 | 400 | 300 | 200 |

Per-pack-region multiplier: each pack has own cluster sized at active-tenants-in-pack tier. DR pair sized 1.0× primary + 0.6× warm-standby.

## Headroom + Burst

All replica counts include buffer multipliers (1.2-1.5×). In addition:

- **Pre-warmed pool**: 5 standby WS gateway pods + 3 standby recalc-worker pods + 3 standby xlsx-export-worker pods per cell; cold-start budget ≤ 1s for WS, ≤ 5s for recalc/export.
- **HPA**: scales on CPU > 70% OR WS connection count > 70% pod cap OR recalc queue depth > 100 OR export queue depth > 20; ratchets 2 replicas per scale-out event.
- **VPA**: vertical-pod-autoscaler for Postgres workers + Valkey.
- **Burst absorbing**: 60s of session-open backlog absorbed by Valkey ephemeral queue before back-pressure.

## Postgres + Citus Sizing

```
total_postgres_tables          = (workbooks + cells_hot + edit_logs + share_acls + comments + version_pointers + license_attributions + ai_formula_archive + audit_seals) × N_tenants_per_shard
citus_distributed_tables       = workbooks, cells_hot, edit_logs, share_acls, comments, version_pointers, license_attributions, ai_formula_archive (sharded on tenant_id)
citus_reference_tables         = pack_overlay_lookup (replicated to all workers)

per_shard_size_target          = ≤ 500 GB (Citus reference)
shard_count                    = ceil(total_postgres_storage / per_shard_size_target)
```

## Arrow / Parquet Large-Sheet Sizing (per ADR-SHEETS-0003)

```
arrow_block_size               = 1000 cells per block
arrow_blocks_per_workbook      = ceil(cells_cold_per_workbook / arrow_block_size)
parquet_compression_ratio      = 0.3 (typical for cell-dense workbooks)
oci_objectstorage_storage      = sum across all tenants × workbooks × blocks
hot_to_cold_promotion          = workbook idle > 24h AND cells > 100k → migrate to Parquet cold tier
cold_to_hot_promotion          = first-edit on cold tier triggers materialize back to Postgres hot for the touched range
```

## Valkey Sizing

```
total_valkey_keys               = total_active_sessions × 6 (CRDT state + cursor + presence + lease + recalc-progress + edit-buffer-tip)
                              + total_active_subscriptions
valkey_memory                   = total_valkey_keys × 80KB + 1GB Sentinel overhead
valkey_replicas                 = 3 (quorum)
```

## WebSocket Gateway Sizing

```
ws_connections_per_pod         = 10,000 (axum-ws benchmark on E4 4-core)
ws_pod_replicas                = ceil(total_ws_connections / ws_connections_per_pod) × 1.5 buffer
```

Per-pod memory budget: 4GB; CPU budget: 2-4 cores.

## Recalc Engine Sizing (per ADR-SHEETS-0004)

```
recalcs_per_worker_per_sec     = 50 (parallel-task-graph; bounded by rayon thread pool sized to pod CPU)
recalc_worker_replicas         = ceil(total_recalcs_per_sec / recalcs_per_worker_per_sec) × 1.3 buffer
```

Per-pod memory budget: 8GB (Arrow block working set); CPU budget: 4-8 cores.

## XLSX Export Worker Sizing

```
xlsx_export_jobs_per_worker_per_min  = 60 (typical 100k-cell workbook export ≤ 5s)
xlsx_export_worker_replicas          = ceil(total_xlsx_export_per_day / 86400 × 60 / xlsx_export_jobs_per_worker_per_min) × 1.5
```

Per-pod memory budget: 4GB; CPU budget: 4 cores (gVisor overhead ~15%).

## CDN Sizing

```
cdn_pop_count_per_pack         = OCI CDN PoP count per pack (KR: 1, EU: 3, US: 6, etc.)
cdn_origin_egress              = total_cdn_egress_per_day × 0.2 (assumes 80% cache hit)
cdn_egress_to_origin_budget    = $X per pack region per month
```

## Worked Example: oyatie XS Tier (M03 launch; 20 tenants pack-kr-only)

```
N_tenants                  = 20
S_active_per_tenant        = 5
S_opens_per_day_per_tenant = 30
Save_per_day_per_tenant    = 100
Edit_per_sec_per_tenant    = 3
Recalc_per_min_per_tenant  = 50
AI_per_day_per_tenant      = 5
XlsxImport_per_day_per_t   = 2
XlsxExport_per_day_per_t   = 3
ConnectedRefresh_per_day   = 5
Seats_per_tenant           = 10
Workbook_per_tenant        = 100
Cells_per_workbook         = 5000  (median)

total_active_sessions      = 20 × 5 = 100
total_sessions_per_day     = 20 × 30 = 600
total_saves_per_sec        ≈ 0.023
total_edits_per_sec        ≈ 60
total_recalcs_per_sec      ≈ 17
total_ws_connections       = 120
total_ai_invocations/day   = 100
total_xlsx_import/day      = 40
total_xlsx_export/day      = 60
total_connected_refresh/day= 100

postgres_storage_per_day   ≈ (600 × 4KB) + (60 × 86400 × 256B × 0.1 sampling) + (2000 × 256B) + (100 × 5KB)
                           ≈ 2.4MB + 130MB + 0.5MB + 0.5MB ≈ 134 MB/day
postgres_storage_30d       ≈ 4 GB

arrow_parquet_storage      ≈ ~0 (XS tier: most workbooks fit in 100k-cell hot threshold)

s3_snapshot_storage        ≈ 20 × 100 × 50KB × 10 versions ≈ 1 GB

valkey_memory               ≈ 100 × 80KB × 2 + 1 GB ≈ 1.02 GB

Replica counts:
  postgres_coordinator     = 2
  postgres_worker          = max(ceil(100 / 50_000), 4) → 4 (HA minimum)
  postgres_read_replica    = 4
  valkey_sentinel           = 3
  cell_grid_rest           = 2 (HA min)
  collab_crdt_worker       = max(3, ceil(120 / 30_000)) = 3
  recalc_engine_worker     = max(2, ceil(17 / 50)) = 2
  xlsx_export_worker       = max(2, ceil(60 / 86400 × 60)) = 2
  license_gate_cedar       = 2
  cell_grid_app            = 2
  clamav_sidecar           = 1
  opswat_sidecar           = 1

CDN egress (XS):
  cdn_egress_per_day       ≈ (600 × 14MB) + (100 × 24 × 200KB / 4) ≈ 8.4 GB + 0.12 GB ≈ 8.5 GB/day
  cdn_egress_per_month     ≈ 255 GB

Total Sheets storage (XS, M03 launch):
  ~30 GB Postgres hot + 10 GB Valkey + 1 GB S3 + 255 GB CDN egress
  ~$2880/month per pack region (per cost-budget.md)
```

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=capacity-conformance --microservice sheets` — exit 0.
- Quarterly capacity review: actual usage vs forecast; recalibrate per-tenant averages.
- Annual reference-architecture refresh.

## References

- Postgres + Citus docs — `docs.citusdata.com/`.
- Valkey Sentinel — `valkey.io/topics/sentinel`.
- axum WebSocket — `docs.rs/axum/latest/axum/extract/ws/`.
- Apache Arrow 18.x — `arrow.apache.org/docs/`.
- Apache Parquet 18.x — `parquet.apache.org/`.
- Loro 1.x — `loro.dev/docs`.
- OCI CDN — `oracle.com/cloud/cdn/`.
- OCI Object Storage — `oracle.com/cloud/storage/object-storage/`.
- gVisor — `gvisor.dev`.
- `microservices/sheets/cost-budget.md`.
- `microservices/sheets/multi-region.md`.
- `microservices/sheets/policy/data-residency.md`.
