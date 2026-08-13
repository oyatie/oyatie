---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-onprem-k8s-substrate
impl_plan_id: IP-015-observability-slo-and-authority-cohesion
status: pending
execution_unit: ChangeSet
owner: axis-cloud + axis-observability
acceptance_lanes: [openslo-schema, oya-governance-authority-cohesion, oya-governance-promotion-readiness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: OpenSLO authoring + authority-cohesion registration

## Intent

Author four planned OpenSLO manifests at these exact future destinations — `k8s/slos/cluster-bootstrap-availability.openslo.yaml`, `k8s/slos/node-join-latency.openslo.yaml`, `k8s/slos/network-policy-propagation-latency.openslo.yaml`, and `k8s/slos/api-proxy-decision-latency.openslo.yaml` — so the cloud-k8s release pointer can advance past `dev`. These four files are planned outputs of this ChangeSet and are not present in the tree today. Existing files under `k8s/slos/` measure other signals and must not be treated as substitutes. Then register HG-CLOUD-K8S in the authority-cohesion gate so cross-microservice claims (for example, cluster bootstrap p99 within 30 minutes) have a verifier.

## ChangeSet boundary

OpenSLO manifests for the four named SLIs below (future outputs at the exact `k8s/slos/` destinations) plus an authority-cohesion registry update.

## Concrete File Targets

| Path | Action |
|---|---|
| `k8s/slos/cluster-bootstrap-availability.openslo.yaml` | create — planned; not present today |
| `k8s/slos/node-join-latency.openslo.yaml` | create — planned; not present today |
| `k8s/slos/network-policy-propagation-latency.openslo.yaml` | create — planned; not present today |
| `k8s/slos/api-proxy-decision-latency.openslo.yaml` | create — planned; not present today |
| `registry/authority-cohesion.json` | update — register HG-CLOUD-K8S |

## Code Shape

```yaml
# k8s/slos/cluster-bootstrap-availability.openslo.yaml
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
              query: 'rate(oya_cloud_k8s_cluster_bootstrap_total{outcome="success"}[60m])'
        total:
          metricSource:
            metricSourceType: Prometheus
            spec:
              query: 'rate(oya_cloud_k8s_cluster_bootstrap_total[60m])'
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
# k8s/slos/api-proxy-decision-latency.openslo.yaml
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
            query: 'histogram_quantile(0.99, rate(oya_kubernetes_api_proxy_request_duration_seconds_bucket[5m]))'
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
      "registered_at": "k8s/",
      "slo_manifests": [
        "k8s/slos/cluster-bootstrap-availability.openslo.yaml",
        "k8s/slos/node-join-latency.openslo.yaml",
        "k8s/slos/network-policy-propagation-latency.openslo.yaml",
        "k8s/slos/api-proxy-decision-latency.openslo.yaml"
      ],
      "claim_doc": "k8s/competitor-parity-matrix.md"
    }
  ]
}
```

## Acceptance Gates

```bash
# Precondition: the four planned destinations must exist as this
# ChangeSet's outputs. Do not glob k8s/slos/*.openslo.yaml — those
# existing files are unrelated and must not satisfy this check.
planned_slos=(
  k8s/slos/cluster-bootstrap-availability.openslo.yaml
  k8s/slos/node-join-latency.openslo.yaml
  k8s/slos/network-policy-propagation-latency.openslo.yaml
  k8s/slos/api-proxy-decision-latency.openslo.yaml
)
for slo in "${planned_slos[@]}"; do
  test -f "$slo" || { echo "missing planned SLO $slo" >&2; exit 1; }
  cargo run -p oya-observability-slo-engine-rest -- validate "$slo"
done
cargo run -p oya-dev-cli -- gate validate authority-cohesion
cargo run -p oya-dev-cli -- gate validate oya-governance-promotion-readiness --microservice cloud-k8s --sha <head-sha> --env staging
```

## Test Plan

- The four planned OpenSLO files exist at the exact destinations above and validate per OpenSLO v1.0 schema (cluster bootstrap availability, node join latency, network-policy propagation latency, API-proxy decision latency). Existing `k8s/slos/*` files are not substitutes.
- HG-CLOUD-K8S registered in authority-cohesion: `gate list --id HG-CLOUD-K8S` returns it
- oya-governance-promotion-readiness lane: green for cloud-k8s at head SHA (cluster up, all 4 SLIs green)
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
- `k8s/PRD.md` AC table.
- OpenSLO v1.0 spec — `openslo.com`.
