---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-ci-fitness-consolidation
impl_plan_id: IP-014-observability-slo-authoring
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry + axis-observability
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, slo-coverage]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014: Author OpenSLO manifests for governance (self-observability)

## Intent

Author OpenSLO manifests for governance µservice's own SLIs per ADR-0139's SLO gate + ADR-0131 §"slos/" mandatory subfolder. Self-observability of the substrate that gates other µservices.

## ChangeSet boundary

OpenSLO manifests + Grafana dashboard refs + alert routing.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/governance/slos/lane-runtime-availability.openslo.yaml` | create | 99.95% monthly for per-PR gate decision |
| `microservices/governance/slos/lane-runtime-latency.openslo.yaml` | create | p99 ≤ 60s for full ~50-lane set |
| `microservices/governance/slos/evidence-emitter-seal-latency.openslo.yaml` | create | p99 ≤ 1s for seal |
| `microservices/governance/slos/aggregation-indexer-regen-latency.openslo.yaml` | create | p99 ≤ 5min for full repo |
| `microservices/governance/slos/finding-emit-availability.openslo.yaml` | create | 99.95% monthly for finding write path |
| `microservices/governance/dashboards/governance-self-slo.json` | create | per-SLO burn-rate panels |

## Code Shape

```yaml
# slos/lane-runtime-availability.openslo.yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: governance-lane-runtime-availability
  displayName: Governance Lane-Runtime Availability
  labels:
    microservice: governance
    bc: lane-runtime
    pack: pack-kr
spec:
  service: governance
  description: |
    Availability of the per-PR gate decision path. A request is "successful"
    when an AdmissionVerdict is computed within p99 ≤ 200ms.
  indicator:
    metricSource:
      type: Prometheus
      spec:
        query: |
          sum(rate(oya_governance_admission_verdict_total{result="success"}[5m]))
          /
          sum(rate(oya_governance_admission_verdict_total[5m]))
  timeWindow:
    - duration: 30d
      isRolling: true
  budgetingMethod: Occurrences
  objectives:
    - displayName: 99.95% availability
      target: 0.9995
  alertPolicies:
    - kind: AlertPolicy
      apiVersion: openslo/v1
      metadata: { name: governance-lane-runtime-availability-burn-rate }
      spec:
        description: Fast-burn (14.4×) + slow-burn (6×) per Google SRE Workbook
        conditions:
          - kind: AlertCondition
            apiVersion: openslo/v1
            metadata: { name: fast-burn-1h }
            spec:
              op: ">"
              threshold: 14.4
              lookbackWindow: 1h
          - kind: AlertCondition
            apiVersion: openslo/v1
            metadata: { name: slow-burn-6h }
            spec:
              op: ">"
              threshold: 6
              lookbackWindow: 6h
        notificationTargets:
          - kind: AlertNotificationTarget
            apiVersion: openslo/v1
            metadata: { name: governance-oncall }
            spec:
              target: grafana-oncall
              spec: { integrationId: governance-oncall-integration }
```

## Acceptance Gates

```bash
# OpenSLO schema validation
cargo run -p oya-observability-slo-engine-rest -- validate microservices/governance/slos/
# SLO coverage lane (every µservice must have ≥ 1 SLO)
cargo run -p oya-dev-cli -- gate validate slo-coverage --microservice governance
# Bootstrap: governance's own SLO engine eligibility check (using synthetic probes pre-deploy)
cargo run -p oya-observability-slo-engine-worker -- evaluate --microservice governance --env staging
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_openslo_schemas_valid` | OpenSLO v1.0 conformance |
| `test_slo_coverage_lane_green` | governance has ≥ 1 SLO |
| `test_burn_rate_alerts_route_to_governance_oncall` | OnCall routing |

## Halt Conditions

- OpenSLO schema invalid → halt; fix manifest.
- SLO bootstrap-paradox (governance's SLO engine cannot evaluate governance until governance is up) → use synthetic probe per `microservices/observability/PRD.md` Open Q4 fallback.

## Next IP

[`IP-015-runbooks-iac-finalization.md`](IP-015-runbooks-iac-finalization.md)

## References

- ADR-0139 (agentic SLO-gated promotion).
- ADR-0131 §"slos/" mandatory subfolder.
- `microservices/observability/PRD.md` §"OpenSLO v1.0 native".
- Google SRE Workbook ch. 5 (alerting on SLOs).

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the explicit counterpart hook required by ADR-0328 D-20. Governance parity is evaluated against GitHub Advanced Security, SonarQube, Snyk, Trivy, Open Policy Agent, Backstage TechDocs, and Renovate. The implementation must state which of those controls it closes or deliberately does not target before promotion.
