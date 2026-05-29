# IP-019 — SLO Wiring

**microservice**: feature-flags
**bc**: observability
**layer**: slo
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0130, ADR-0131, ADR-0159, ADR-0248, ADR-0263
**companion_ips**: IP-002, IP-008, IP-010, IP-012

## Scope

Wire all 4 SLOs into the observability substrate per ADR-0130 (SLO-gated promotion). Ensure OTEL metric emission, OpenSLO v1 manifests, Prometheus recording rules, Grafana burn-rate panels, and alerting routes are all consistent with the SLO targets.

## Deliverables

| # | SLO | File | Target | Alert Threshold |
|---|-----|------|--------|----------------|
| 1 | flag-eval-latency | `slos/flag-eval-latency.openslo.yaml` | p99 ≤1ms, 99% | fast-burn 14× (1h/5m), slow-burn 5× (6h/30m) |
| 2 | flag-state-propagation | `slos/flag-state-propagation.openslo.yaml` | ≤5s cross-region, 99% | fast-burn 14×, slow-burn 3× |
| 3 | experiment-result-freshness | `slos/experiment-result-freshness.openslo.yaml` | ≤60s p95, 95% | 10× burn (6h/30m) |
| 4 | killswitch-fire-latency | `slos/killswitch-fire-latency.openslo.yaml` | ≤1s all cells, 99.9% | LIFE-SAFETY: 100× fast-burn (1m/5m); SEV-1 alert |

## OTEL Metric Names

| Metric | Type | Labels |
|--------|------|--------|
| `oya_feature_flag_eval_duration_ms` | Histogram | `tenant_id`, `flag_key`, `result`, `cache_hit` |
| `oya_feature_flag_propagation_lag_s` | Histogram | `tenant_id`, `cell_id`, `path` (wal\|kafka) |
| `oya_experiment_metric_freshness_s` | Gauge | `tenant_id`, `experiment_id` |
| `oya_killswitch_fire_latency_ms` | Histogram | `tenant_id`, `scope`, `cell_id` |
| `oya_feature_flag_eval_queue_depth` | Gauge | `cell_id` (HPA custom metric) |

## Prometheus Recording Rules

```yaml
# Eval latency p99 budget burn
- record: oya:feature_flag_eval_latency:p99:1h
  expr: histogram_quantile(0.99, rate(oya_feature_flag_eval_duration_ms_bucket[1h]))

# Kill-switch fire latency p99 per cell
- record: oya:killswitch_fire_latency:p99_ms:1m
  expr: histogram_quantile(0.99, rate(oya_killswitch_fire_latency_ms_bucket[1m]))
```

## SLO-Gated Rollout Integration

The `RolloutKernel` (IP-011) reads SLO burn-rate state via `SloGate`. If `flag-eval-latency` slow-burn >5× or `flag-state-propagation` slow-burn >3×, `RolloutAdvance` is denied.

## Definition of Done

- All 4 OpenSLO manifests valid against `openslo/spec v1`
- OTEL emission wired in `oya-feature-flags-flag-app` for all 5 metrics
- Grafana dashboards include burn-rate panels for all 4 SLOs
- `killswitch-fire-latency` alert routes to PagerDuty SEV-1 oncall
- ADR-0130 SLO-gated promotion gate: `lean-a4-slo-coverage` lane green
