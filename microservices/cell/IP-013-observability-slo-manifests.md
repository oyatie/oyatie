---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-cell-substrate
impl_plan_id: IP-013-observability-slo-manifests
status: pending
owner: axis-cell-substrate + axis-observability
acceptance_lanes: [oya-check-openslo-conformance, oya-vcs-promotion-readiness]
---

# IP-013: OpenSLO manifests for cell µservice

## Intent

Author OpenSLO v1.0 manifests at `microservices/cell/slos/` covering the cell µservice's published SLOs. Per `docs/standards/observability-slo.md` (cross-cutting standard from observability µservice). The cell µservice's release pointer (`release/cell/*`) cannot advance past `dev` until these manifests validate.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cell/slos/cell-assignment-availability.openslo.yaml` | create |
| `microservices/cell/slos/cell-assignment-latency.openslo.yaml` | create |
| `microservices/cell/slos/scheduler-placement-availability.openslo.yaml` | create |
| `microservices/cell/slos/scheduler-placement-latency.openslo.yaml` | create |
| `microservices/cell/slos/migration-completion.openslo.yaml` | create |
| `microservices/cell/slos/cell-create-latency.openslo.yaml` | create |
| `microservices/cell/slos/cell-boundary-violations-zero.openslo.yaml` | create |

## Code Shape

```yaml
# slos/cell-assignment-availability.openslo.yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: cell-assignment-availability
  displayName: "Cell-Assignment Lookup Availability"
  labels:
    microservice: cell
    bounded_context: cell-registry
spec:
  service: cell
  indicator:
    metadata:
      name: assignment-lookup-success-ratio
    spec:
      thresholdMetric:
        metricSource:
          metricSourceRef: mimir
          spec:
            query: |
              sum(rate(cell_assignment_lookup_total{status="success"}[5m])) /
              sum(rate(cell_assignment_lookup_total[5m]))
  objectives:
    - target: 0.9999  # 99.99% availability
      displayName: cell-assignment-lookup
  timeWindow:
    - duration: 30d
      isRolling: true
  budgetingMethod: Occurrences
```

```yaml
# slos/cell-boundary-violations-zero.openslo.yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: cell-boundary-violations-zero
  displayName: "Cell-Boundary Violations Must Be Zero"
  labels:
    microservice: cell
    severity: sev-1-on-breach
spec:
  service: cell
  indicator:
    metadata:
      name: zero-violations
    spec:
      thresholdMetric:
        metricSource:
          metricSourceRef: mimir
          spec:
            query: |
              max_over_time(oya_cell_boundary_violation_total[5m]) == 0 OR vector(0)
  objectives:
    - target: 1.0  # MUST be zero violations
      displayName: zero-cross-cell-or-cross-pack
  timeWindow:
    - duration: 30d
      isRolling: true
  budgetingMethod: Occurrences
```

## Acceptance Gates

```bash
cargo run -p oya-observability-slo-engine-rest -- validate microservices/cell/slos/
cargo run -p oya-dev-cli -- gate validate openslo-conformance --microservice cell
cargo run -p oya-dev-cli -- gate validate vcs-promotion-readiness --sha <head> --env staging --microservice cell
```

## Test Plan

- Schema-validation: every manifest conforms to OpenSLO v1.0.
- Reachability: every PromQL expression is reachable against current Mimir state.
- Burn-rate alarm tuning: thresholds match the SRE workbook 14.4× / 6× / 3× / 1× recommendations.

## Halt Conditions

- Manifest schema-invalid — fix.
- PromQL unreachable — adjust metric emission in cell-substrate operator pods.

## Next IP

[`IP-014-branch-protection-gate-registration.md`](IP-014-branch-protection-gate-registration.md)

## References

- `docs/standards/observability-slo.md`.
- `microservices/observability/PRD.md` FR-01.
- ADR-0130 (SLO gate).
- OpenSLO v1.0 — `github.com/OpenSLO/OpenSLO`.
