---
doc_class: CapacityModel
title: Capacity Sizing Model (foundry-supervisor)
microservice: foundry-supervisor
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry-control-plane
deciders: ops-sre-reliability, axis-foundry-control-plane, council-architecture
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/intelligence-supervisor/cost-budget.md
  - microservices/intelligence-supervisor/multi-region.md
  - microservices/intelligence-supervisor/policy/supervisor-isolation.md
review_cadence: quarterly + on every component-replica-set change
doc_status: published
---

# Capacity Sizing Model (foundry-supervisor µservice)

## Purpose

Sizing formulas + reference-architecture baselines for every component (Postgres HA, Valkey Cluster, Kubernetes Operator, REST, worker, app). Drives `cost-budget.md` and `multi-region.md`.

## Inputs

| Input | Variable | Source |
|---|---|---|
| Active tenants | `N_tenants` | tenancy µservice |
| Capabilities per tenant (avg) | `C_capabilities_per_tenant` | onboarding survey + telemetry |
| Agents per tenant (avg, in-flight workers) | `A_agents_per_tenant` | runtime telemetry |
| Supervision events per agent-hour | `E_events_per_agent_hour` | supervision-event-bus telemetry |
| Autonomy precondition evaluations per second per agent | `P_precond_per_sec_per_agent` | runtime invocation rate |
| Kill-switch queries per second per pack | `K_kill_query_per_sec_pack` | runtime poll cadence |

## Postgres Sizing

### Formulae

```
total_fleet_rows           = N_tenants × (C_capabilities_per_tenant + A_agents_per_tenant)
postgres_write_iops        = (N_deployment_events_per_min + N_kill_switch_state_changes_per_min) / 60
postgres_read_iops         = N_tenants × precond_eval_rate / cache_hit_ratio   (cached in Valkey; Postgres tail-end)
postgres_storage_per_tenant = ~5 MB row data + ~50 MB deployment history (2y retention)
postgres_storage_total     = N_tenants × 55 MB × (1 + DR-replica-overhead)
```

### Per-component replica formulae (per PostgreSQL HA best-practice)

```
postgres_primary_cores  = max(8, ceil(write_iops / 5000))
postgres_replica_cores  = same (matched)
pgbouncer_replicas      = max(2, ceil(connection_count / 200))
```

References: PostgreSQL HA — `postgresql.org/docs/current/high-availability.html`; PgBouncer — `pgbouncer.org`. Verify-at-deploy.

### Reference baselines

| Tier | N_tenants | total_fleet_rows | Postgres replica config |
|---|---|---|---|
| **XS** (M01; 20 tenants) | 20 | ~6k rows | primary=1×8c + replica=1×8c (DR-pair packs); 1 TB PV; PgBouncer × 2 |
| **S** (~100) | 100 | ~30k rows | primary=1×16c + replica×2; 4 TB PV; PgBouncer × 4 |
| **M** (~1000) | 1000 | ~300k rows | primary=1×32c + replica×3 + read-replicas×2; 16 TB PV; PgBouncer × 8 |
| **L** (~10000) | 10000 | ~3M rows | primary=1×64c + replica×3 + read-replicas×4 + tenant-sharded; 100 TB PV; PgBouncer × 24 |

Sharding triggers at M-tier (tenant_hash MOD num_shards).

## Valkey Cluster Sizing

### Formulae

```
kill_switch_state_keys = N_tenants × (1 + C_capabilities_per_tenant + A_agents_per_tenant)
redis_memory           = kill_switch_state_keys × ~200 bytes + supervision_event_stream × ~500 bytes
redis_ops_per_sec      = K_kill_query_per_sec_pack + supervision_event_writes_per_sec
```

### Per-component formulae

```
redis_shards    = max(3, ceil(redis_memory / 4 GB))           (one shard per 4 GB)
redis_replicas  = 2 per shard (HA)
```

References: Valkey Cluster sizing — `redis.io/docs/management/scaling/`.

### Reference baselines

| Tier | N_tenants | Valkey memory (per pack) | Valkey shards × replicas |
|---|---|---|---|
| XS | 20 | ~50 MB | 3 × 2 (minimum) |
| S | 100 | ~250 MB | 3 × 2 |
| M | 1000 | ~2.5 GB | 3 × 2 |
| L | 10000 | ~25 GB | 6 × 2 |

## Kubernetes Operator Sizing

```
operator_replicas = max(3, ceil(crd_object_count / 50_000))    (one shard per 50k CRDs)
```

For M01 launch (~6k CRDs), 3 replicas (HA-leader-elected). Per controller-runtime + kube-rs reference.

## REST + Worker + App Sizing

```
supervisor_rest_replicas    = max(3, ceil(qps / 100))         (HA min)
supervisor_worker_replicas  = max(2, ceil(reconcile_queue_depth / 60s_cadence))
supervisor_app_replicas     = 2 (HA composition root)
```

For M01 launch, REST=3, worker=2, app=2.

## Headroom + Burst

- Pre-warmed pool: 2 standby pods per critical component (REST, worker, controller). Cold-start budget ≤ 500 ms.
- HPA: CPU > 70% or queue-depth thresholds; ratchet 2 replicas per scale-out.
- VPA: vertical scale on non-critical components (worker).

## Storage Costs

| Tier | Standard hot (Postgres PV) | Archive cold (S3 WAL) | Notes |
|---|---|---|---|
| OCI block volume | ~$0.0255 / GB / month | – | provisioned PV |
| OCI object-storage standard | ~$0.0255 / GB / month | – | hot tier |
| OCI object-storage archive | ~$0.0025 / GB / month | – | cold tier (24mo+) |

Storage policy:
- 0–30 d: PV hot
- 30 d–24 mo: object-storage standard (WAL archive)
- 24 mo+: object-storage archive

### Worked example: XS tier (M01 launch; 20 tenants pack-kr-only)

```
total_fleet_rows      = 20 × (50 + 250) = ~6k rows
postgres_write_iops   = (200 deployments/min + 100 kill-switch changes/min) / 60 = ~5 IOPS
postgres_storage      = 20 × 55 MB ≈ 1.1 GB initial; 1 TB PV provisioned for 24mo growth
redis_memory          = ~50 MB
redis_ops_per_sec     = (~1k poll/s + ~100 event-writes/s) ≈ 1.1k ops/s/cluster

Postgres replica config: primary=1×8c + replica=1×8c; 1 TB PV each; PgBouncer × 2.
Valkey Cluster: 3 shards × 2 replicas (minimum HA); 4 GB allocated per shard (massively over-provisioned for XS; aligned to future scale).
Kubernetes Operator: 3 replicas.
REST: 3 replicas.
Worker: 2 replicas.
App: 2 replicas.

Total compute: ~813 USD/month per pack region.
Storage: ~180 USD/month (hot + warm + archive mix).
Total: ~$1k/month/pack-region XS tier.
```

Cost projections per scale tier in `cost-budget.md`.

## Verification

- `cargo run -p oya-dev-cli -- gate validate capacity-conformance --microservice foundry-supervisor` — exit 0; deployed replica counts ≥ formula minimums.
- Quarterly capacity review; recalibrate inputs.
- Annual reference-architecture refresh.

## References

- PostgreSQL HA — `postgresql.org/docs/current/high-availability.html`.
- PgBouncer — `pgbouncer.org`.
- Valkey Cluster — `redis.io/docs/management/scaling/`.
- Kubernetes Operator pattern — `kubernetes.io/docs/concepts/extend-kubernetes/operator/`.
- controller-runtime + kube-rs.
- OCI pricing — `oracle.com/cloud/storage/pricing/`.
- `microservices/intelligence-supervisor/cost-budget.md`.
- `microservices/intelligence-supervisor/multi-region.md`.
- `microservices/intelligence-supervisor/policy/supervisor-isolation.md`.
