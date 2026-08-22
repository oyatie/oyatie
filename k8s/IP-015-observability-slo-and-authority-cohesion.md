---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-onprem-k8s-substrate
impl_plan_id: IP-015-observability-slo-and-authority-cohesion
status: pending
execution_unit: ChangeSet
owner: axis-cloud + axis-observability
acceptance_lanes: [openslo-schema, governance-authority-cohesion, governance-promotion-readiness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: OpenSLO authoring + authority-cohesion registration

## Intent

Author OpenSLO manifests at `microservices/cloud-k8s/slos/*.openslo.yaml` so cloud-k8s's release pointer can advance past `dev` per ADR-0139. Then register HG-CLOUD-K8S in the authority-cohesion gate so cross-microservice claims (e.g., "Cluster bootstrap p99 ≤ 30min") have a verifier.

## ChangeSet boundary

OpenSLO manifests for 4 SLIs + authority-cohesion registry update.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-k8s/slos/cluster-bootstrap-availability.openslo.yaml` | create |
| `microservices/cloud-k8s/slos/node-join-latency.openslo.yaml` | create |
| `microservices/cloud-k8s/slos/network-policy-propagation-latency.openslo.yaml` | create |
| `microservices/cloud-k8s/slos/api-proxy-decision-latency.openslo.yaml` | create |
| `registry/authority-cohesion.json` | update — register HG-CLOUD-K8S |

## Code Shape

```yaml
# slos/cluster-bootstrap-availability.openslo.yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: cluster-bootstrap-availability
  displayName: cloud-k8s — cluster bootstrap availability
  labels:
    microservice: cloud-k8s
    bc: cluster-bootstrap
    data_class: INTERNAL_ONLY
spec:
  service: cloud-k8s
  indicator:
    metadata:
      name: cluster-bootstrap-success-rate
    spec:
      ratioMetric:
        counter: true
        good:
          metricSource:
            metricSourceType: Prometheus
            spec:
              query: 'rate(cloud_k8s_cluster_bootstrap_total{outcome="success"}[60m])'
        total:
          metricSource:
            metricSourceType: Prometheus
            spec:
              query: 'rate(cloud_k8s_cluster_bootstrap_total[60m])'
  budgetingMethod: Occurrences
  timeWindow:
    - duration: 30d
      isRolling: true
  objectives:
    - displayName: 99.5% successful bootstrap
      target: 0.995
      op: gte
```

```yaml
# slos/api-proxy-decision-latency.openslo.yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: api-proxy-decision-latency
spec:
  service: cloud-k8s
  indicator:
    metadata:
      name: api-proxy-decision-p99-under-50ms
    spec:
      thresholdMetric:
        metricSource:
          metricSourceType: Prometheus
          spec:
            query: 'histogram_quantile(0.99, rate(kubernetes_api_proxy_request_duration_seconds_bucket[5m]))'
  budgetingMethod: Timeslices
  timeWindow:
    - duration: 30d
      isRolling: true
  objectives:
    - displayName: 99% of decisions p99 < 50ms
      target: 0.99
      op: lte
      value: 0.05
```

```json
// registry/authority-cohesion.json excerpt
{
  "authorities": [
    {
      "id": "HG-CLOUD-K8S",
      "owner_team": "axis-cloud",
      "registered_at": "microservices/cloud-k8s/",
      "slo_manifests": [
        "microservices/cloud-k8s/slos/cluster-bootstrap-availability.openslo.yaml",
        "microservices/cloud-k8s/slos/node-join-latency.openslo.yaml",
        "microservices/cloud-k8s/slos/network-policy-propagation-latency.openslo.yaml",
        "microservices/cloud-k8s/slos/api-proxy-decision-latency.openslo.yaml"
      ],
      "claim_doc": "microservices/cloud-k8s/competitor-parity-matrix.md"
    }
  ]
}
```

## Acceptance Gates

```bash
for slo in microservices/cloud-k8s/slos/*.openslo.yaml; do
  cargo run -p observability-slo-engine-rest -- validate "$slo"
done
cargo run -p dev-cli -- gate validate authority-cohesion
cargo run -p dev-cli -- gate validate governance-promotion-readiness --microservice cloud-k8s --sha <head-sha> --env staging
```

## Test Plan

- All 4 OpenSLO manifests validate per OpenSLO v1.0 schema (AC-01 of observability PRD)
- HG-CLOUD-K8S registered in authority-cohesion: `gate list --id HG-CLOUD-K8S` returns it
- governance-promotion-readiness lane: green for cloud-k8s at head SHA (cluster up, all 4 SLIs green)
- Burn-rate alarms wired to grafana-oncall

## Halt Conditions

- Any SLO target lower than industry-norm baselines (per `competitor-parity-matrix.md`) — escalate to council-architecture
- HG-CLOUD-K8S registers without all 10 criteria verifiers green — refuse merge

## Next IP

End of phase. Begin `exit_gate` validation per PHASE-01.

## References

- ADR-0139 (agentic SLO-gated promotion); ADR-0123 (HG gate); ADR-0121 (substrate).
- `docs/standards/observability-slo.md`.
- `microservices/observability/PRD.md` FR-01 (OpenSLO authoring).
- `microservices/cloud-k8s/PRD.md` AC table.
- OpenSLO v1.0 spec — `openslo.com`.
