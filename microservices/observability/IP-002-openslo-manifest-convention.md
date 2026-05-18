---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agentic-slo-gated-promotion
impl_plan_id: IP-002-openslo-manifest-convention
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-observability
acceptance_lanes: [cargo-check, oya-governance-openslo-conformance, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: OpenSLO manifest convention + observability self-SLOs

## Intent

`docs/standards/observability-slo.md` is already authored (Slice D). This IP authors the first OpenSLO manifests at `microservices/observability/slos/*.openslo.yaml` for observability's self-SLOs (the substrate must observe its own SLOs to dogfood the gate).

## ChangeSet boundary

4 OpenSLO manifests (one per canonical SLI: availability, latency, correctness, freshness) for the observability µservice itself. Verified against OpenSLO v1.0 schema and the project's authoring rules.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/observability/slos/availability.openslo.yaml` | create | 99.95% / 30d rolling; covers slo-engine-rest 200 vs 5xx |
| `microservices/observability/slos/latency.openslo.yaml` | create | p99 ≤ 2s for verdict-emission path |
| `microservices/observability/slos/correctness.openslo.yaml` | create | burn-rate-computation conformance vs Google SRE Workbook reference values |
| `microservices/observability/slos/freshness.openslo.yaml` | create | evaluator cycle freshness — evaluated_at lag ≤ 90s |
| `microservices/observability/slos/waivers.md` | create | empty initial waiver register; future SLI deferrals recorded here |

## Code Shape

Example `availability.openslo.yaml`:

```yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: observability-availability
  displayName: "observability — REST 2xx availability"
  labels:
    microservice: observability
    sli: availability
    pack: pack-kr
    data_classes: BEHAVIORAL_TENANT_PRODUCT
spec:
  service: observability
  indicator:
    spec:
      ratioMetric:
        good:
          metricSource:
            type: Prometheus
            spec:
              query: |
                sum(rate(http_requests_total{job="oya-observability-slo-engine-rest",status!~"5.."}[5m]))
        total:
          metricSource:
            type: Prometheus
            spec:
              query: |
                sum(rate(http_requests_total{job="oya-observability-slo-engine-rest"}[5m]))
  objectives:
    - displayName: "99.95% over rolling 30d"
      target: 0.9995
  timeWindow:
    - duration: 30d
      isRolling: true
  budgetingMethod: Occurrences
```

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate openslo-conformance --microservice observability
cargo run -p oya-dev-cli -- gate validate openslo-promql-feasibility --microservice observability
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice observability
```

## Test Plan

- 4 unit tests: parse each manifest; assert schema-valid + targets within bounds.
- 1 integration test: hot-reload one manifest; verify slo-engine-worker picks it up within 3s.
- E2E scheduled-for-distinct-tracked-work to IP-008 (worker integration).

## Halt Conditions

- PromQL infeasibility for any manifest (returns empty for representative Mimir snapshot) — author non-empty seed metrics first.

## Next IP

[`IP-003-slo-engine-kernel.md`](IP-003-slo-engine-kernel.md)

## References

- `docs/standards/observability-slo.md`
- ADR-0130; ADR-0131
- OpenSLO v1.0 spec
