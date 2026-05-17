---
doc_class: CapacityModel
title: Capacity Sizing Model
microservice: cell
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-cell-substrate
deciders: ops-sre-reliability, axis-cell-substrate, council-architecture
related_adrs: [ADR-0117, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/cell/cost-budget.md
  - microservices/cell/multi-region.md
  - microservices/cell/policy/cell-boundary.md
review_cadence: quarterly + on every component-replica-set change
doc_status: published
---

# Capacity Sizing Model (cell µservice)

## Purpose

Sizing formulas + reference-architecture baselines for every Layer-A component (Kubernetes Cluster API management + workload cluster pools + Postgres registry + warm pool + SPIRE) and Layer-B component (`oya-cell-*` crates). Drives `cost-budget.md` and `multi-region.md`. Numbers cite Kubernetes + Cluster API + Postgres reference architectures.

## Inputs

| Input | Variable | Source |
|---|---|---|
| Active tenants per pack | `N_tenants_per_pack` | tenancy µservice |
| Cells per pack | `N_cells_per_pack` | `N_tenants_per_pack / mean_tenants_per_cell` (band [40%, 80%]) |
| Workload pods per cell | `P_pods_per_cell` | per-cell capacity envelope (default 50; varies by tenant_scope) |
| Per-cell Postgres connections | `C_pg_conns_per_cell` | PgBouncer pool config (default 100) |
| Warm-pool size per pack | `W_warm_per_pack` | 2 (M01 default) |
| Scheduler placement throughput | `Q_placements_per_sec` | 1 (M01) — onboarding is bursty but bounded |
| Migration concurrency per pack | `M_migrations_per_sec` | 0.1 (1 migration per 10s avg; per Bominal ADR-0009) |

## Cell-Registry Postgres Sizing

### Formulae

```
registry_rows_per_pack = N_tenants_per_pack + N_cells_per_pack + N_hosts_per_pack
                       + N_migration_history_per_pack (90d hot retention)

Postgres data size per pack:
  ≈ registry_rows_per_pack × 0.5 KB  + audit-tier rows × 1 KB
  ≈ negligible (< 1 GB for tier L)

Postgres connections:
  pgbouncer_pool_size = C_pg_conns_per_cell × N_cells_per_pack + 50 control plane

postgres_replicas = 1 primary + 2 replicas per pack (HA)
```

Reference: Postgres-on-Kubernetes scaling guides — `postgresoperator.com`, `cloudnative-pg.io`.

### Reference-architecture baselines

| Scale tier | N_tenants per pack | N_cells per pack | Postgres footprint per pack | pgbouncer pool |
|---|---|---|---|---|
| **XS** (M01; 5–20 tenants) | 20 | 4 | < 1 GB | 250 connections |
| **S** (~100 tenants) | 100 | 12 | < 5 GB | 750 |
| **M** (~1000 tenants) | 1000 | 100 | < 50 GB | 6000 |
| **L** (~10000 tenants) | 10000 | 800 | < 500 GB | 30000 (sharded) |

For tier L: shard Postgres by `(pack, region)` second-level partitioning per `policy/data-residency.md`.

## K8s Cluster API Sizing

### Formulae

```
mgmt_cluster_replicas = 3 per pack (HA control plane)
mgmt_cluster_node_count = 3-5 (operator pods only; not workload)

workload_cluster_replicas = 3 (control plane per workload cluster)
workload_cluster_node_count = ceil(N_cells_per_pack × P_pods_per_cell / 50 pods_per_node)
                            × replication_factor (2) × 1.2 buffer
```

References: Cluster API scaling guide — `cluster-api.sigs.k8s.io/user/quick-start.html` + `cluster-api.sigs.k8s.io/clusterctl/commands/move.html`.

### Reference-architecture baselines

| Scale tier | N_cells per pack | Workload cluster nodes per pack | Management cluster nodes per pack |
|---|---|---|---|
| **XS** (M01) | 4 | 24 | 3 |
| **S** | 12 | 72 | 3 |
| **M** | 100 | 600 | 5 |
| **L** | 800 | 4800 | 12 (sharded mgmt cluster) |

## Warm-Pool Sizing

```
W_warm_per_pack = ceil(0.1 × N_cells_per_pack) capped at [2, 20]

Defaults (M01): 2 standby nodes per pack; sufficient for ≤ 2 concurrent cell creates.
```

Trigger: scheduler decision queue depth > 5 fires page; warm-pool refill operator scales node-pool up.

## Cell-Substrate Operator Pod Sizing

| Component | Per-pack replicas | Per-pod CPU | Per-pod memory |
|---|---|---|---|
| `cell-registry-rest` | 2 | 1 core | 512 MB |
| `cell-registry-app` | 2 | 0.5 core | 256 MB |
| `tenant-assignment-rest` | 2 | 1 core | 512 MB |
| `tenant-assignment-worker` | 2 | 1 core | 1 GB (migration state) |
| `scheduler-worker` | 2 | 1 core | 1 GB (cluster-state cache) |
| `lifecycle-manager-worker` | 2 | 1 core | 512 MB |
| `host-pool-worker` | 2 | 1 core | 512 MB |

All operators scale via HPA on CPU + custom metric (queue depth).

## Per-Cell Envelope (workload-µservice resident; informational for cell substrate)

| Dimension | Default | Max | Scale-out trigger |
|---|---|---|---|
| Tenants per cell (`cell_scope: shared`) | 100 | 1000 | utilization band [40%, 80%] |
| Workload pods per cell | 50 | 200 | CPU > 70% across pods |
| Postgres connections per cell | 100 | 500 | PgBouncer pool |
| Object-storage prefix bytes per cell | 1 TB | 10 TB | retention compaction |

## Worked Example: oyatie XS tier (M01 launch; 20 tenants pack-kr; 4 cells)

```
N_tenants_per_pack = 20
mean_tenants_per_cell = 5 (within [40%, 80%] band when cell.max = 25 trial / 100 shared)
N_cells_per_pack = 4
P_pods_per_cell = 50
W_warm_per_pack = 2

Postgres registry size: < 1 GB
pgbouncer pool: 250 connections
Workload cluster nodes: 24 (4 cells × 6 nodes/cell)
Management cluster nodes: 3
Warm-pool nodes: 2

scheduler placement budget:
  - Cold placement (new tenant): p99 ≤ 500ms (binpack over 4 candidates)
  - Migration evaluation: p99 ≤ 1s (binpack over candidates considering drain cost)

lifecycle-manager budget:
  - Cell create (warm hit): ≤ 90s p50; ≤ 5min p99
  - Cell create (cold; new node provision): ≤ 5min p50; ≤ 15min p99
  - Cell decommission: ≤ 6h p99 (bounded by last-tenant-migration)
```

## Scale-out Triggers

- Cells in pack exceed [80%] utilization band → scheduler triggers new cell create.
- Warm pool drops below 2 nodes → host-pool provisions replacement node.
- Workload cluster node-pool > 80% CPU → cluster autoscaler scales node-pool up.
- Management cluster CPU > 70% → ops-sre-reliability paged; manual scale-up.

## Cross-Region Story

- M01: single-region per pack; pack-kr only.
- Post-M01: pack-eu + pack-us add DR pair within pack (per `multi-region.md`).
- Federation between packs: forbidden per `policy/data-residency.md`.

## References

- `microservices/cell/cost-budget.md`.
- `microservices/cell/multi-region.md`.
- `microservices/cell/policy/cell-boundary.md`.
- `microservices/cell/policy/data-residency.md`.
- Kubernetes Cluster API — `cluster-api.sigs.k8s.io`.
- CloudNativePG Postgres operator — `cloudnative-pg.io`.
- Bominal ADR-0019 (runtime catalog + cell sharding).
