---
scorecard_id: finops-portal/adr-0186-observability-backplane
authored: 2026-05-18
authority: ADR-0186 observability backplane (Mimir + Grafana + OTel)
status: ready
---

# Scorecard — ADR-0186 Observability backplane

ADR-0186 mandates every µservice export logs (JSON to stdout), metrics
(Prometheus + Mimir), and traces (OTLP) following the canonical
shape, and that dashboards live in-repo per ADR-0131 flat layout.

## Compliance evidence

| Criterion                                              | Status | Evidence                                                  |
|--------------------------------------------------------|--------|-----------------------------------------------------------|
| Logs emitted as JSON to stdout                         | ✓      | IP-006 app crate uses `tracing-subscriber` JSON layer     |
| Metrics on `/metrics` (Prometheus scrape)              | ✓      | IP-006 + `templates/servicemonitor.yaml`                  |
| Traces emitted via OTLP                                | ✓      | IP-006 `OTEL_TRACES_EXPORTER=otlp`                        |
| ServiceMonitor declared                                | ✓      | `templates/servicemonitor.yaml`                           |
| PrometheusRule declared with SLO burn-rate alerts      | ✓      | `templates/prometheusrule.yaml`                           |
| SLOs authored in-repo (OpenSLO v1)                     | ✓      | 9 SLOs at `slos/*.openslo.yaml`                            |
| Grafana dashboards in-repo (JSON)                      | ✓      | 3 dashboards at `dashboards/*.grafana.json`                |
| Dashboards reference `$tenant_id` for tenant scope     | ✓      | `tenant-cost-drilldown.grafana.json` templating list      |
| Cost-attribution labels propagated to metrics          | ✓      | ServiceMonitor relabelings + `oya.tenantCostLabels`       |
| Audit-chain event emit on key events                   | ✓      | `manifest.json#audit_chain.seal_events` lists 5 classes   |

## Cited SLOs

1. `tenant-invoice-render-latency.openslo.yaml` (p95 ≤ 2s)
2. `tenant-invoice-pdf-render-availability.openslo.yaml` (99.5%)
3. `drilldown-query-latency-p99.openslo.yaml` (p99 ≤ 1s)
4. `cost-allocation-policy-change-latency.openslo.yaml` (p95 ≤ 60s)
5. `focus-export-availability.openslo.yaml` (99.9%)
6. `credit-application-correctness.openslo.yaml` (100%)
7. `anomaly-explanation-latency.openslo.yaml` (p95 ≤ 30s)
8. `regulator-emit-availability.openslo.yaml` (100% on-time 4Q)
9. `quarterly-regulator-evidence-emit-correctness.openslo.yaml` (100%)

## Gaps + remediation

- **Gap**: Grafana dashboard import smoke test gate (`dashboards-
  import-smoke`) is referenced in IP-008 but not yet wired in CI.
  **Remediation**: tracked as follow-up.

## Verdict

**PASS**.

## References

- ADR-0186 observability backplane.
- IP-006 app observability wiring.
- IP-008 dashboards.
