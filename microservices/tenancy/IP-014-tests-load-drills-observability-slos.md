---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-tenancy-substrate-stable
impl_plan_id: IP-014-tests-load-drills-observability-slos
status: pending
owner: ops-sre-reliability + axis-tenancy
acceptance_lanes: [cargo-nextest, oya-governance-openslo-conformance]
---

# IP-014: Tests + load drills + OpenSLO manifests for tenancy

## Intent

Author k6 load tests; Patroni-failover availability drill; quarterly synthetic cross-tenant probe; tenancy OpenSLO manifests at `microservices/tenancy/slos/{availability,latency,correctness,freshness}.openslo.yaml`.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/tenancy/tests/load/tenant-validate-100krps.js` | create — k6 100k RPS sustained |
| `microservices/tenancy/tests/load/patroni-failover-availability.sh` | create — induce Patroni primary loss; measure validate availability |
| `microservices/tenancy/tests/integration/synthetic_cross_tenant_probe.rs` | create — quarterly drill |
| `microservices/tenancy/tests/e2e/dsr_cascade_proof.rs` | create — full DSR drill across all M01 µservices |
| `microservices/tenancy/slos/availability.openslo.yaml` | create |
| `microservices/tenancy/slos/latency.openslo.yaml` | create |
| `microservices/tenancy/slos/correctness.openslo.yaml` | create — RLS-no-cross-tenant probe success rate ≥ 100% |
| `microservices/tenancy/slos/freshness.openslo.yaml` | create — RLS drift detection within 5min |
| `microservices/tenancy/slos/waivers.md` | create — empty register |

## Code Shape

`slos/availability.openslo.yaml`:

```yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: tenancy-availability
  labels:
    microservice: tenancy
    sli: availability
    pack: pack-kr
spec:
  service: tenancy
  indicator:
    spec:
      ratioMetric:
        good:
          metricSource:
            type: Prometheus
            spec:
              query: 'sum(rate(http_requests_total{job="oya-tenancy-tenant-lifecycle-rest",status!~"5.."}[5m]))'
        total:
          metricSource:
            type: Prometheus
            spec:
              query: 'sum(rate(http_requests_total{job="oya-tenancy-tenant-lifecycle-rest"}[5m]))'
  objectives:
    - displayName: "99.99% over rolling 30d (tightest in catalog)"
      target: 0.9999
  timeWindow:
    - duration: 30d
      isRolling: true
  budgetingMethod: Occurrences
```

`slos/correctness.openslo.yaml`:

```yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: tenancy-correctness-rls-probe
spec:
  service: tenancy
  indicator:
    spec:
      ratioMetric:
        good:
          metricSource:
            type: Prometheus
            spec:
              query: 'sum(rate(oya_tenancy_synthetic_cross_tenant_probe_success_total[5m]))'
        total:
          metricSource:
            type: Prometheus
            spec:
              query: 'sum(rate(oya_tenancy_synthetic_cross_tenant_probe_total[5m]))'
  objectives:
    - displayName: "100% — RLS MUST refuse cross-tenant; any failure = Sev-1"
      target: 1.0
  timeWindow: [{duration: 7d, isRolling: true}]
  budgetingMethod: Occurrences
```

## Acceptance Gates

```bash
k6 run microservices/tenancy/tests/load/tenant-validate-100krps.js
bash microservices/tenancy/tests/load/patroni-failover-availability.sh
cargo nextest run -p oya-tenancy-tenant-lifecycle-adapter-postgres --test synthetic_cross_tenant_probe
cargo run -p oya-dev-cli -- gate validate openslo-conformance --microservice tenancy
```

## Test Plan

- 100k RPS sustained 10min; p99 ≤ 5ms; error rate ≤ 0.01%.
- Patroni failover: 10s blip; overall availability ≥ 99.99% during 10min window with one induced primary loss.
- Cross-tenant probe: zero rows returned across all paths.
- DSR cascade: end-to-end proof-of-erasure across all M01 µservices.

## Next IP

[`IP-015-legacy-crates-migration.md`](IP-015-legacy-crates-migration.md)
