---
doc_class: CapacityModel
title: Capacity Sizing Model
microservice: cloud-iac
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-cloud-iac
deciders: ops-sre-reliability, axis-cloud-iac, architecture-governance
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/cloud-iac/cost-budget.md
  - microservices/cloud-iac/multi-region.md
  - microservices/cloud-iac/policy/iac-isolation.md (per-µservice limits)
review_cadence: quarterly + on every component-replica-set change
doc_status: published
---

# Capacity Sizing Model (cloud-iac µservice)

## Purpose

Sizing formulas + reference-architecture baselines for every Layer-A component (ArgoCD + Flux + OpenTofu + Helm-controller + Kustomize-controller + Postgres iac-state-index) and Layer-B component (`cloud-iac-iac-{renderer,validator,applier,rollback,registry}-*`). Drives `cost-budget.md` and `multi-region.md`. Numbers cite ArgoCD + Flux + OpenTofu published references; verify-against-current-docs markers called out.

## Inputs

| Input | Variable | Source |
|---|---|---|
| Active microservices in oyatie catalog | `N_microservices` | `/specs/per-microservice-flat-layout.json` migration table |
| Concurrent applies per µservice (rate-limited) | `C_applies_per_ms` | `policy/iac-isolation.md` §"Invariant ISO-06" (max 6/h) |
| Average apply duration | `D_apply_seconds` | observed: ~90s p50, 5min p99 |
| Drift-detection cycle period | `T_drift_seconds` | ≤ 3600s (≤1h) |
| Cluster count per pack | `K_clusters_per_pack` | typically 1–3 (workload + control-plane + observability) |
| Average chart size | `S_chart_bytes` | ~50KB |
| iac-state-index growth rate | `G_index_rows_per_day` | ~10 rows/µservice/day = 360 rows/day for XS tier |

## ArgoCD Sizing

### Formulae (cites ArgoCD scaling docs)

```
argocd_app_controller_replicas = ceil(N_microservices / 200) × 1.5 buffer (min 3)
argocd_repo_server_replicas    = ceil(N_microservices / 100) × 1.5 buffer (min 3)
argocd_server_replicas         = ceil(active_ui_users / 100) × 2 HA (min 2)
argocd_application_set_controller = 1 (singleton; HA via leader election)
argocd_valkey_replicas         = 3 (sentinel cluster)

reconcile_interval = 180s (default; honoured by drift-detection ≤1h gate)
sync_throughput    = N_microservices / reconcile_interval applications/sec
```

References: `argo-cd.readthedocs.io/en/stable/operator-manual/high_availability/`. Verify-at-deploy: numbers above match ArgoCD docs as of 2026-05-17; re-validate quarterly.

### Reference-architecture baselines

| Scale tier | N_microservices | argocd_replicas | reconcile_interval |
|---|---|---|---|
| **XS** (M01 launch; ~36 µservices) | 36 | app-controller=3, repo-server=3, server=2, valkey=3 | 180s |
| **S** (~100 µservices) | 100 | app-controller=3, repo-server=3, server=2, valkey=3 | 180s |
| **M** (~500 µservices) | 500 | app-controller=6, repo-server=6, server=4, valkey=3 | 180s |
| **L** (~2000 µservices) | 2000 | app-controller=20, repo-server=20, server=8, valkey=3 | 240s |

## Flux Sizing

Flux runs alongside ArgoCD for tenant-choice GitOps reconciler. Sized at lower replica counts since ArgoCD is the M01 primary.

```
flux_source_controller_replicas    = 2 (HA min)
flux_kustomize_controller_replicas = 2
flux_helm_controller_replicas      = 2
flux_image_automation_controller   = 0 (scheduled-for-distinct-tracked-work to subsequent-to-M01-completion)
```

References: `fluxcd.io/docs/installation/configuration/sharding/`.

## OpenTofu Runner Sizing

```
opentofu_runner_replicas = ceil(C_concurrent_plans / 5) × 1.3 buffer (min 3)
plan_timeout             = 10min p99
state_lock_timeout       = 10min (per Postgres advisory lock)

For XS tier: ~20 concurrent plans → 6 replicas. We size at min 3 for HA;
HPA scales to 6+ on load.
```

References: `opentofu.org/docs/internals/`.

## Helm-controller + Kustomize-controller Sizing

These are typically deployed via Flux:

```
helm_controller_replicas       = 2 (HA min)
kustomize_controller_replicas  = 2 (HA min)
```

References: `fluxcd.io/docs/components/helm/` and `fluxcd.io/docs/components/kustomize/`.

## Postgres iac-state-index Sizing

### Schema-shape

- 1 row per (microservice, pack, environment, applied_at) tuple → expected growth ~10 rows/µservice/day.
- Indexes on (microservice, pack, environment) for read-heavy queries.
- Partitioning by month for retention enforcement.

### Formulae

```
index_total_rows_per_year = N_microservices × 10 × 365 = ~131k rows/yr for XS tier
storage_per_million_rows  ≈ 1 GB (with indexes + signatures)

postgres_replica_replicas = 1 primary + 1 read-replica (HA min)
postgres_replica_size     = VM.Standard.E4 4-core / 16GB
postgres_pv_size_initial  = 200 GB (handles ~5y XS-tier growth + WAL archive)
postgres_pgbouncer        = optional; scheduled-for-distinct-tracked-work until S tier
```

References: `postgresql.org/docs/current/high-availability.html`.

## Layer-B Sizing (`cloud-iac-iac-*-worker`)

```
renderer_worker_replicas = max(2, ceil(N_microservices / 200))
validator_worker_replicas = max(2, ceil(N_microservices / 200))
applier_worker_replicas  = max(2, ceil(C_concurrent_applies / 10))
rollback_worker_replicas = max(2, ceil(N_microservices / 1000))
registry_worker_replicas = max(2, ceil(N_microservices / 500))
```

For M01 launch (XS tier; N_microservices ≈ 36), each is at the HA minimum of 2 replicas.

## Headroom + Burst

- **Pre-warmed pool**: 2 standby renderer + 2 standby applier pods; cold-start ≤500ms per ADR-0020.
- **HPA**: scales on CPU > 70% OR queue-depth thresholds; ratchet up 2 replicas per scale-out.
- **VPA**: vertical-pod-autoscaler for non-critical components (rollback worker, registry worker).

## Drift-Detection Cycle Capacity

```
drift_detector_replicas    = ceil(K_clusters_per_pack / 2) × 1.5 buffer (min 2)
drift_diff_throughput      = ~100 resources/second per replica
clusters_per_cycle_per_pack = K_clusters_per_pack × N_microservices

For XS tier (3 clusters × 36 µservices = 108 cluster-µservice combos):
each cycle scans ~108 × avg-resource-count per µservice ~= ~50 resources = 5400 resources
at ~100 resources/sec = 54 sec per cycle (well within ≤1h target)
```

## Apply-Latency Budget

Per PRD §"Performance Targets":

| Step | p50 | p99 | p999 |
|---|---|---|---|
| Render | ≤1s | ≤5s | ≤10s |
| Validate (Cedar + plan-preview) | ≤2s | ≤10s | ≤30s |
| Apply (k8s reconcile or tofu apply) | ≤60s | ≤4min | ≤14min |
| Audit-chain seal | ≤500ms | ≤2s | ≤5s |
| **End-to-end** | **≤90s** | **≤5min** | **≤15min** |

## Storage Costs (per pack region)

### Object-storage (S3-compatible) at OCI rates

Storage tier policy:
- 0–30d: standard (hot).
- 30d–6mo: infrequent-access (warm).
- 6mo–6y: archive (cold; HIPAA pack uses full 6y).
- Beyond retention: deleted per `data-residency.md` matrix.

### Worked example: oyatie XS tier (M01 launch; 36 µservices pack-kr-only)

```
N_microservices = 36
N_packs = 1
K_clusters_per_pack = 3

iac_state_index_rows_per_day = 36 × 10 = 360
iac_state_index_storage_5y   = 360 × 365 × 5 × 1 KB/row ≈ 660 MB (negligible)

OpenTofu state per µservice ~= 50 KB
Total OpenTofu state         = 36 × 50 KB × 3 envs = ~5.4 MB live state per pack
State backups (versioned)    = 5.4 MB × 30 versions × 90d ≈ 15 GB

iac-state-index backups (Postgres dumps) = ~1 GB/day × 90d retention = 90 GB hot + 1 GB/day × 365 × 5 × 0.5 archive ≈ 900 GB cold

ArgoCD application records   = ~36 × 3 envs × ~2 KB = 216 KB (negligible)
Sigstore Rekor cache         = ~100 MB rolling cache

Total cloud-iac storage XS tier ≈ 1 TB / pack region all tiers
≈ $30 / month per pack region (mix of hot+warm+archive)
```

Cost projections per scale tier in `cost-budget.md`.

## Verification

- cloud-ci/ci governance gate `capacity-conformance` for --microservice cloud-iac is green in the branch-protected `presubmit` context — exit 0; deployed replica counts ≥ formula minimums.
- Quarterly capacity review: actual usage vs forecast; recalibrate `C_concurrent_applies` averages.
- Annual reference-architecture refresh: re-verify against current ArgoCD / Flux / OpenTofu published sizing guides.

## References

- ArgoCD HA + scaling — `argo-cd.readthedocs.io/en/stable/operator-manual/high_availability/`.
- Flux scaling — `fluxcd.io/docs/installation/configuration/sharding/`.
- OpenTofu internals — `opentofu.org/docs/internals/`.
- Postgres HA — `postgresql.org/docs/current/high-availability.html`.
- OCI compute + storage pricing — `oracle.com/cloud/pricing/`.
- `microservices/cloud-iac/cost-budget.md`.
- `microservices/cloud-iac/multi-region.md`.
- `microservices/cloud-iac/policy/iac-isolation.md` (per-µservice limits).
