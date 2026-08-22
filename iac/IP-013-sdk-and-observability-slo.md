---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-meta-iac-pipeline-substrate
impl_plan_id: IP-013-sdk-and-observability-slo
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud-iac
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, governance-openslo-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: SDK crates + OpenSLO manifests + observability self-SLOs

## Intent

Two parts:
1. SDK crates: `cloud-iac-iac-renderer-sdk` and `cloud-iac-iac-registry-sdk` for µservice + tenant consumption per `sdk-plan.md`.
2. OpenSLO manifests for cloud-iac's own self-SLOs (apply success rate; drift coverage; render p99; registry read latency) at `microservices/cloud-iac/slos/`.

## ChangeSet boundary

Two SDK crates + four OpenSLO manifests. Catalog rows.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-iac/src/crates/cloud-iac-iac-renderer-sdk/{Cargo.toml,src/lib.rs,src/client.rs}` | create |
| `microservices/cloud-iac/src/crates/cloud-iac-iac-registry-sdk/{Cargo.toml,src/lib.rs,src/client.rs}` | create |
| `microservices/cloud-iac/slos/availability.openslo.yaml` | create — apply success-rate SLO |
| `microservices/cloud-iac/slos/latency.openslo.yaml` | create — apply p99 latency SLO |
| `microservices/cloud-iac/slos/correctness.openslo.yaml` | create — render-determinism SLO |
| `microservices/cloud-iac/slos/freshness.openslo.yaml` | create — drift-detection coverage SLO |
| `microservices/cloud-iac/catalog/cloud-iac-iac-*-sdk.yaml` | create (2 rows) |

## Code Shape

```rust
// renderer-sdk/src/client.rs
pub struct RendererClient {
    base_url: Url,
    auth: Arc<dyn AuthProvider>,
    microservice_scope: String,
    http: reqwest::Client,
}

impl RendererClient {
    pub async fn trigger_render(&self, sha: &str, pack: &str, env: Environment) -> Result<RenderId, Error> {
        let token = self.auth.get_oidc_token().await?;
        let resp = self.http.post(format!("{}/microservices/{}/render", self.base_url, self.microservice_scope))
            .bearer_auth(token)
            .header("X-Microservice", &self.microservice_scope)
            .json(&TriggerRenderRequest { sha, pack, environment: env })
            .send().await?;
        // retry + circuit-breaker
        ...
    }
}
```

```yaml
# slos/availability.openslo.yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: cloud-iac-apply-success
  displayName: cloud-iac Apply Success Rate
  labels:
    microservice: cloud-iac
    sli_type: availability
spec:
  service: cloud-iac
  indicator:
    metadata:
      name: cloud-iac-apply-success-ratio
    spec:
      ratioMetric:
        good:
          metricSource:
            type: Prometheus
            spec:
              query: sum(rate(cloud_iac_apply_completed_total{state="completed"}[1m]))
        total:
          metricSource:
            type: Prometheus
            spec:
              query: sum(rate(cloud_iac_apply_completed_total[1m]))
  objectives:
    - displayName: "≥99.5% apply success over 30d"
      target: 0.995
  timeWindow:
    - duration: 30d
      isRolling: true
  budgetingMethod: Occurrences
```

```yaml
# slos/freshness.openslo.yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: cloud-iac-drift-coverage
  displayName: cloud-iac Drift Detection Coverage
  labels:
    microservice: cloud-iac
    sli_type: freshness
spec:
  service: cloud-iac
  indicator:
    metadata:
      name: cloud-iac-drift-coverage-ratio
    spec:
      ratioMetric:
        good:
          metricSource:
            type: Prometheus
            spec:
              query: sum(cloud_iac_drift_cycles_completed_within_1h)
        total:
          metricSource:
            type: Prometheus
            spec:
              query: sum(cloud_iac_drift_cycles_expected_within_1h)
  objectives:
    - displayName: "≥99.5% clusters polled per 1h cycle"
      target: 0.995
  timeWindow:
    - duration: 30d
      isRolling: true
  budgetingMethod: Occurrences
```

## Acceptance Gates

```bash
cargo check -p cloud-iac-iac-renderer-sdk -p cloud-iac-iac-registry-sdk --all-features
cargo nextest run -p cloud-iac-iac-renderer-sdk -p cloud-iac-iac-registry-sdk --all-features
cloud-ci/ci governance gate `openslo-conformance` for --microservice cloud-iac is green in the branch-protected `presubmit` context
```

## Test Plan

| Test | Layer | Verifies |
|---|---|---|
| `test_renderer_client_trigger_render` | sdk | Happy path against mocked HTTP |
| `test_renderer_client_retry` | sdk | Retry on transient 5xx |
| `test_registry_client_get_apply_state` | sdk | Read against mocked HTTP |
| `openslo_schema_validate_availability` | openslo | manifest validates against OpenSLO v1 schema |

## Halt Conditions

- SDK retry policy allows infinite retry — bounded.
- OpenSLO target < 99% (too lax) — escalate.

## Next IP

[`IP-014-per-pack-iac-overlays.md`](IP-014-per-pack-iac-overlays.md)

## References

- ADR-0105 §"sdk layer".
- ADR-0139 (observability SLO gate authority).
- OpenSLO v1.0 schema — `github.com/OpenSLO/OpenSLO`.
- `microservices/cloud-iac/sdk-plan.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/cloud-iac/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `seaweedfs_replicated`, `postgres_wal_g`].
- Surface evidence: `microservices/cloud-iac/runbooks/restore-drill-quarterly.md`, `microservices/cloud-iac/runbooks/seaweedfs-volume-failover.md`, `microservices/cloud-iac/manifest.json`, `microservices/cloud-iac/IP-013-sdk-and-observability-slo.md`.
