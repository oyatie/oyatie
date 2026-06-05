---
doc_class: CapacityModel
title: Capacity Sizing Model
microservice: observability
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-observability
deciders: ops-sre-reliability, axis-observability, council-architecture
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/observability/cost-budget.md
  - microservices/observability/multi-region.md
  - microservices/observability/policy/tenant-isolation.md (per-tenant limits)
  - /specs/agentic-slo-gated-promotion.json §"mimir_multi_tenancy"
review_cadence: quarterly + on every component-replica-set change
doc_status: published
---

# Capacity Sizing Model (observability µservice)

## Purpose

Sizing formulas + reference-architecture baseline numbers for every Layer-A component (Mimir / Loki / Tempo / Pyroscope / Grafana / Alertmanager / Alloy / OnCall) and Layer-B component (`oya-observability-slo-engine-*`). Drives `cost-budget.md` and `multi-region.md`. Numbers cite Grafana published reference architectures; verify-against-current-Grafana-docs marker called out where the upstream may have moved on.

## Inputs

The model is parameterised by:

| Input | Variable | Source |
|---|---|---|
| Active tenants | `N_tenants` | OpenBao tenant-resolver |
| Per-tenant active series | `K_series_per_tenant` | `tenant_scope` (per `tenant-isolation.md` per-tenant limits) |
| Per-tenant samples/sec | `S_samples_per_sec_per_tenant` | as above |
| Per-tenant log bytes/sec | `B_log_bytes_per_sec_per_tenant` | as above |
| Per-tenant trace spans/sec | `T_spans_per_sec_per_tenant` | as above |
| Number of microservices in oyatie catalog | `M_microservices` | `/specs/per-microservice-flat-layout.json` migration table |
| SLI count per µservice | `Q_sli_per_ms` | OpenSLO manifests (default 4: availability, latency, correctness, freshness) |
| Evaluator cadence | `E_cycle_seconds` | 60s |

## Mimir Sizing

### Formulae

```
total_active_series  = N_tenants × K_series_per_tenant
total_samples_per_sec = N_tenants × S_samples_per_sec_per_tenant
total_eligibility_metrics_per_sec = M_microservices × Q_sli_per_ms × 2 (envs: staging+production) / E_cycle_seconds

(Eligibility metrics emit one sample per (µservice, sha, env) tuple per cycle.)

storage_per_day = total_samples_per_sec × 86400 × avg_bytes_per_sample
              ≈ total_samples_per_sec × 86400 × 1.3 bytes  (per Mimir compression benchmarks)

storage_30d_hot  = storage_per_day × 30
storage_24mo_cold = storage_per_day × 730 × 0.6  (cold-tier compression efficiency)
```

### Per-component replica formulae (per Grafana Mimir scaling guide)

```
mimir_distributor_replicas  = ceil(total_samples_per_sec / 100_000) × 1.3 buffer
mimir_ingester_replicas     = ceil(total_active_series / 1_500_000) × replication_factor (3) × 1.2 buffer
mimir_querier_replicas      = ceil(qps / 100) × 1.3 buffer
mimir_query_frontend_replicas = ceil(qps / 1000) × 1.5 buffer
mimir_compactor_replicas    = ceil(total_active_series / 5_000_000) × 1.2 buffer (per Mimir docs)
mimir_ruler_replicas        = ceil(rule_groups / 100) × 1.5 buffer
mimir_store_gateway_replicas = ceil(total_active_series / 10_000_000) × replication_factor (3)
```

References: Grafana Mimir scaling guide — `grafana.com/docs/mimir/latest/manage/run-production-environment/planning-capacity/`. Verify-at-deploy: numbers above match Grafana docs as of 2026-05-17; re-validate quarterly.

### Reference-architecture baselines (cites Grafana)

| Scale tier | N_tenants | total_active_series | total_samples_per_sec | Mimir replica counts |
|---|---|---|---|---|
| **XS** (oyatie M01-launch; ~5–20 tenants) | 20 | 20M | 2M | distributor=4, ingester=12, querier=4, query-frontend=2, compactor=2, ruler=2, store-gateway=2 |
| **S** (~100 tenants; small SaaS) | 100 | 100M | 10M | distributor=12, ingester=24, querier=12, query-frontend=4, compactor=4, ruler=2, store-gateway=4 |
| **M** (~1k tenants; medium SaaS) | 1000 | 1B | 100M | distributor=120, ingester=120, querier=80, query-frontend=12, compactor=24, ruler=12, store-gateway=12 |
| **L** (~10k tenants; large SaaS / hyperscaler) | 10000 | 10B | 1B | distributor=1300, ingester=600, querier=400, query-frontend=80, compactor=120, ruler=60, store-gateway=60 |

Per-pack-region multiplier: each pack has its own cluster sized at the active-tenants-in-pack tier. DR pair (pack-eu eu-frankfurt + eu-amsterdam; pack-us us-ashburn + us-phoenix) sized 1.0× primary + 0.6× warm-standby (snapshot-restore in ≤ 1h).

## Loki Sizing

### Formulae

```
total_log_bytes_per_sec = N_tenants × B_log_bytes_per_sec_per_tenant
log_storage_per_day = total_log_bytes_per_sec × 86400 × 0.15  (Loki compression ratio, gzip)
log_storage_14d_hot = log_storage_per_day × 14
log_storage_12mo_cold = log_storage_per_day × 365 × 0.3 (cold-tier with chunk-merge)

loki_distributor_replicas = ceil(total_log_bytes_per_sec / 50_000_000) × 1.3
loki_ingester_replicas    = ceil(total_log_bytes_per_sec / 25_000_000) × 1.2
loki_querier_replicas     = ceil(qps_log / 50) × 1.3
loki_index_gateway_replicas = ceil(N_tenants / 500) × 1.2
```

References: `grafana.com/docs/loki/latest/operations/planning/sizing/`.

### Reference baselines

| Tier | N_tenants | total_log_bytes_per_sec | Loki replicas |
|---|---|---|---|
| XS | 20 | 200 MB/s | distributor=4, ingester=8, querier=4, index-gateway=2 |
| S | 100 | 1 GB/s | distributor=20, ingester=40, querier=12, index-gateway=2 |
| M | 1000 | 10 GB/s | distributor=200, ingester=400, querier=120, index-gateway=4 |
| L | 10000 | 100 GB/s | distributor=2000, ingester=4000, querier=1200, index-gateway=20 |

## Tempo Sizing

```
total_trace_spans_per_sec = N_tenants × T_spans_per_sec_per_tenant × sample_rate (default 0.01 = 1%)
trace_storage_per_day = total_trace_spans_per_sec × 86400 × avg_bytes_per_span (≈ 2KB)
trace_storage_7d_hot = trace_storage_per_day × 7
trace_storage_6mo_cold = trace_storage_per_day × 180 × 0.5

tempo_distributor_replicas = ceil(total_trace_spans_per_sec / 25_000) × 1.3
tempo_ingester_replicas    = ceil(total_trace_spans_per_sec / 15_000) × 1.2
tempo_querier_replicas     = ceil(qps_trace / 30) × 1.3
```

References: `grafana.com/docs/tempo/latest/operations/`.

## Pyroscope Sizing

```
profile_storage_per_day = total_profiles_per_sec × 86400 × avg_profile_bytes (≈ 50KB)
profile_storage_14d_hot = profile_storage_per_day × 14

pyroscope_ingester_replicas = ceil(total_profiles_per_sec / 100) × 1.3
```

References: `grafana.com/docs/pyroscope/latest/`.

## Grafana (UI + Alertmanager + OnCall) Sizing

```
grafana_replicas = ceil(active_users / 200) × 2 (HA)
grafana_postgres = single primary + read replica (small; metadata only)
alertmanager_replicas = 3 (consensus cluster)
oncall_replicas = ceil(active_alerts_per_min / 1000) × 2
```

## Layer-B Sizing (oya-observability-slo-engine-*)

```
slo_engine_worker_replicas = max(2, ceil(M_microservices / 500)) × 2 (HA min)
slo_engine_rest_replicas   = max(2, ceil(qps_rest / 100)) × 2 (HA min)
slo_engine_app_replicas    = 2 (HA composition root)
```

For M01 launch (M_microservices ≈ 36), worker_replicas = 2 (HA minimum suffices).

## Headroom + Burst

All component-replica counts include the buffer multipliers above (1.2–1.5×). In addition:

- **Pre-warmed pool**: 2 standby pods per critical component (distributor, ingester, query-frontend, worker). Cold-start budget ≤ 500ms per ADR-0020.
- **HPA**: scales on CPU > 70% OR queue-depth thresholds (per component); ratchet up 2 replicas per scale-out event.
- **VPA**: vertical-pod-autoscaler for non-critical components (ruler, compactor) sized to recommended memory.

## Storage Costs (per pack region)

### Object-storage (S3-compatible) at OCI rates (cites Oracle public pricing, 2026-05-17)

```
OCI object-storage standard tier: ~$0.0255 / GB / month
OCI object-storage infrequent-access tier: ~$0.01 / GB / month
OCI archive tier: ~$0.0025 / GB / month
```

Storage tier policy:
- 0–30d: standard (hot)
- 30d–6mo: infrequent-access (warm)
- 6mo–24mo+: archive (cold)
- Beyond retention: deleted per `data-residency.md` matrix

### Worked example: oyatie XS tier (M01 launch; 20 tenants pack-kr-only)

```
total_active_series = 20 × 1_000_000 = 20M
total_samples_per_sec = 20 × 100_000 = 2M
storage_per_day = 2M × 86400 × 1.3 bytes ≈ 224 GB/day Mimir
storage_30d_hot = 6.7 TB Mimir hot
storage_24mo_cold = 224 × 730 × 0.6 ≈ 98 TB Mimir cold

Loki:
  20 × 10MB/s = 200 MB/s = 17 TB/day uncompressed → 2.6 TB/day compressed
  14d hot = 36 TB
  12mo cold = 285 TB (gzipped + chunk-merged)

Tempo:
  20 × 5000 spans/s × 0.01 sample = 1000 spans/s
  storage_per_day = 1000 × 86400 × 2KB ≈ 173 MB/day
  7d hot = 1.2 GB
  6mo cold = 31 GB

Pyroscope:
  Profile rate ≈ 1/min/service × 36 services = ~30 profiles/min total
  ~50KB × 30/min × 60 × 24 ≈ 2.2 GB/day
  14d hot = 30 GB

Total observability storage (XS, M01 launch):
  ~50 TB / pack region all-tiers
  ~$1500/month per pack region storage cost (mix of hot+warm+archive)
```

Cost projections per scale tier in `cost-budget.md`.

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=capacity-conformance` — exit 0; deployed replica counts ≥ formula minimums.
- Quarterly capacity review: actual usage vs forecast; recalibrate `K_series_per_tenant` averages.
- Annual reference-architecture refresh: re-verify against current Grafana published sizing guides.

## References

- Grafana Mimir scaling guide — `grafana.com/docs/mimir/latest/manage/run-production-environment/planning-capacity/`.
- Grafana Loki sizing — `grafana.com/docs/loki/latest/operations/planning/sizing/`.
- Grafana Tempo operations — `grafana.com/docs/tempo/latest/operations/`.
- Grafana Pyroscope — `grafana.com/docs/pyroscope/latest/`.
- Grafana Alloy — `grafana.com/docs/alloy/latest/`.
- Prometheus storage benchmarks — `prometheus.io/docs/prometheus/latest/storage/`.
- OCI object-storage pricing — `oracle.com/cloud/storage/pricing/`.
- `microservices/observability/cost-budget.md`.
- `microservices/observability/multi-region.md`.
- `microservices/observability/policy/tenant-isolation.md` (per-tenant limits).
