---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P01-application-shell-landing
impl_plan_id: IP-015-application-openslo-and-hg-app
status: pending
execution_unit: ChangeSet
owner: axis-application + axis-observability
acceptance_lanes: [openslo-validate, hyperscaler-maturity-claims, authority-cohesion, per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: OpenSLO authoring + HG-APP gate registration + branch-protection

## Intent

Wire the application µservice into the SLO substrate + branch-protection:

- Author OpenSLO manifests for TTI, route-resolve, sign-in, module-load,
  audit-seal SLIs at `microservices/application/slos/*.openslo.yaml`.
- Register HG-APP gate in `/specs/hyperscaler-gates.json` per ADR-0123.
- Add branch-protection required-check rules for
  `release/application/{staging,production}` to `.github/branch-protection.yaml`.
- Author Helm + Kustomize wiring for the application cluster.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/application/slos/tti.openslo.yaml` | create |
| `microservices/application/slos/route-resolve.openslo.yaml` | create |
| `microservices/application/slos/oidc-signin.openslo.yaml` | create |
| `microservices/application/slos/module-load.openslo.yaml` | create |
| `microservices/application/slos/audit-seal.openslo.yaml` | create |
| `/specs/hyperscaler-gates.json` | update — register HG-APP with claim set |
| `.github/branch-protection.yaml` | update — required-check rules for application |
| `microservices/application/iac/helm/shell-app/Chart.yaml` | create |
| `microservices/application/iac/helm/shell-app/values.yaml` | create |
| `microservices/application/iac/helm/cdn-controller/Chart.yaml` | create |
| `microservices/application/iac/helm/cdn-controller/values.yaml` | create |
| `microservices/application/iac/helm/postgres/Chart.yaml` | create |
| `microservices/application/iac/helm/postgres/values.yaml` | create |
| `microservices/application/iac/kustomize/base/kustomization.yaml` | create |
| `microservices/application/iac/kustomize/overlays/pack-kr/kustomization.yaml` | create |

## OpenSLO shape (example: TTI)

```yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: oya-application-tti
  microservice: application
spec:
  service: oya-application-shell-frontend
  description: Application Shell time-to-interactive
  indicator:
    metricSource:
      type: Prometheus
      spec:
        query: |
          histogram_quantile(0.99, sum by (le) (rate(oya_application_tti_seconds_bucket{pack="$pack", cache="warm"}[5m])))
  objectives:
    - target: 0.999  # 99.9% of windows under threshold
      threshold: 2.0  # 2 s
  timeWindow:
    - duration: 30d
      isRolling: true
```

## HG-APP claim set (`/specs/hyperscaler-gates.json`)

```json
{
  "HG-APP": {
    "owner": "axis-application",
    "claims": [
      {"claim": "tti_p99_under_2s_warm", "evidence_lane": "oya-application-tti-budget"},
      {"claim": "per_tenant_origin_dns", "evidence_lane": "oya-application-per-tenant-origin-lint"},
      {"claim": "ed25519_signed_module_manifest", "evidence_lane": "oya-application-module-signature-integration-test"},
      {"claim": "cedar_gated_default_deny_routing", "evidence_lane": "oya-application-cedar-default-deny"},
      {"claim": "pack_residency_forbid_default", "evidence_lane": "oya-application-residency-pin"},
      {"claim": "audit_seal_p99_under_1s", "evidence_lane": "oya-application-audit-seal-budget"},
      {"claim": "cdn_global_purge_under_60s", "evidence_lane": "oya-application-cdn-purge-drill"}
    ]
  }
}
```

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate openslo-validate --microservice application
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice application
cargo run -p oya-dev-cli -- gate validate authority-cohesion --microservice application
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice application
helm lint microservices/application/iac/helm/shell-app
kubectl apply --dry-run=client -k microservices/application/iac/kustomize/overlays/pack-kr
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_openslo_schema_validate` | every SLI manifest validates against OpenSLO v1 |
| `test_hg_app_claims_evidence_lane_exists` | every claim points to existing CI lane |
| `test_branch_protection_required_checks_present` | release/application/* protected |
| `test_helm_lint_clean` | all charts lint |
| `test_kustomize_overlay_pack_kr_renders` | overlay renders |

## Halt Conditions

- Any HG-APP claim points to a lane that does not exist
- Any OpenSLO manifest references a metric not emitted by the application crates

## References

- ADR-0123 hyperscaler maturity claim gate
- ADR-0130 SLO-gated promotion
- `/specs/agentic-slo-gated-promotion.json`
