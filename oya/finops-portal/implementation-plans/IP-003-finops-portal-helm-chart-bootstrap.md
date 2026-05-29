---
ip_id: IP-003
ip_status: ready
slice_owner: ops-finops
authored: 2026-05-18
slice: finops-portal/iac/helm-bootstrap
related_adrs: [ADR-0064, ADR-0131, ADR-0186, ADR-0199]
depends_on: []
follow_up_owner: evidence/storage-batch-followup-scope.json#finops-portal-ip-fanout
target_lines: 150
---

# IP-003 — finops-portal Helm chart bootstrap

## Why this slice

Stand up the per-µservice Helm chart at
`microservices/finops-portal/iac/helm/finops-portal/` per ADR-0064
canonical-base discipline. The chart depends on the `_oya-helpers`
library and consumes the canonical helpers including
`oya.tenantCostLabels` (per ADR-0199 D-2 cost-attribution canonical).
Without the chart bootstrap, no environment can render the µservice's
manifests, so this IP unblocks every subsequent runtime IP.

## Acceptance criteria

1. `microservices/finops-portal/iac/helm/finops-portal/Chart.yaml`
   declares dependency on `_oya-helpers` per the helm-chart-convention
   standards doc.
2. `values.yaml` declares:
   - `image.registry` + `image.tag` (placeholder until the app exists).
   - `microservice: finops-portal`,
     `boundedContext: tenant-billing-presentation`.
   - `costAttribution.costCenter: infra-finops-portal`.
   - `costAttribution.workloadClass: app`.
   - `costAttribution.regulatoryPack: generic` (overlay per pack).
   - `slo.tenantInvoiceRenderLatencyTarget: 0.95`
     (binds to `slos/tenant-invoice-render-latency.openslo.yaml`).
3. `templates/deployment.yaml` consumes `oya.labels`,
   `oya.tenantCostLabels`, `oya.securityContext.restricted`, canonical
   probes, and a securityContext that pins:
   - `runAsNonRoot: true`.
   - `readOnlyRootFilesystem: true`.
   - `capabilities.drop: [ALL]`.
4. `templates/service.yaml` exposes HTTP port `8080` named `http`.
5. `templates/networkpolicy.yaml` consumes
   `oya.networkPolicy.defaultDeny` +
   `oya.networkPolicy.allowEgressToSubstrate` (kept narrow:
   audit-chain + tenancy + opencost only; explicit egress to mimir
   query endpoint).
6. `templates/servicemonitor.yaml` declared for Prometheus scrape on
   `/metrics`; emits the canonical SLI metric families
   (`finops_portal_tenant_invoice_render_latency_ms_*`,
   `finops_portal_focus_export_*`,
   `finops_portal_quarterly_report_emit_total`).
7. `templates/prometheusrule.yaml` ships the burn-rate alerts cited in
   `slos/*.openslo.yaml#alertPolicies` (e.g.
   `finops-portal-render-burn-rate`,
   `finops-portal-quarterly-emit-page-on-miss`).
8. `templates/hpa.yaml` declares an HPA targeting CPU 70 % +
   custom-metric `finops_portal_invoice_query_inflight` for elastic
   scale.
9. `helm lint .` returns 0 warnings.
10. `helm template .` produces output that
   `oya-check-tenant-cost-labels-coverage` reports full coverage.

## File-level work plan

1. `iac/helm/finops-portal/Chart.yaml`.
2. `iac/helm/finops-portal/values.yaml`.
3. `iac/helm/finops-portal/templates/deployment.yaml`.
4. `iac/helm/finops-portal/templates/service.yaml`.
5. `iac/helm/finops-portal/templates/networkpolicy.yaml`.
6. `iac/helm/finops-portal/templates/servicemonitor.yaml`.
7. `iac/helm/finops-portal/templates/prometheusrule.yaml`.
8. `iac/helm/finops-portal/templates/hpa.yaml`.
9. `iac/helm/finops-portal/templates/_helpers.tpl` — local label helpers
   that delegate to `_oya-helpers`.
10. Per-pack overlays: `values-kr.yaml`, `values-eu.yaml`,
    `values-us-healthcare.yaml`.

## Cost-attribution wiring (ADR-0199 D-2)

Every workload manifest emitted by this chart MUST carry the
canonical cost-attribution label triple:

- `oya.io/cost-center=infra-finops-portal`.
- `oya.io/workload-class=app`.
- `oya.io/regulatory-pack=<pack>` (driven by overlay).

The OpenCost configmap consumes these labels; if they are missing the
gate `oya gate tenant-cost-labels-coverage` rejects the chart.

## Multi-region overlay strategy (binds to multi-region-strategy.md)

- `values-kr.yaml` — KR pack; pins region `kr-1`; image registry
  mirror `kr-registry.oya.internal`.
- `values-eu.yaml` — EU pack; pins region `eu-central-1`; OpenCost
  scrape endpoint uses EU residency endpoint; storage class
  encrypted-eu only.
- `values-us-healthcare.yaml` — US healthcare pack; pins region
  `us-east-1`; turns on PHI-redaction toggle for invoice rendering
  (placeholder; runtime enforcement in IP-007 Cedar policies).

## Risk + mitigation

- **Risk**: chart drifts from `_oya-helpers` interface. **Mitigation**:
  `oya gate helm-helpers-consumer-contract` CI check.
- **Risk**: ServiceMonitor scrape interval too aggressive blows out
  Mimir storage. **Mitigation**: default `interval: 30s`; per-pack
  overlay may relax to 60s where appropriate.

## Out-of-scope

- The application container itself (separate IP once IP-006 builds
  the app crate).
- Per-pack rollout sequencing (handled by the rollout coordinator
  µservice, not this chart).

## References

- ADR-0064 canonical-base.
- ADR-0186 observability backplane.
- ADR-0199 cost-attribution canonical.
- `docs/standards/helm-chart-convention.md`.

## Verification

- `helm lint microservices/finops-portal/iac/helm/finops-portal/`.
- `helm template microservices/finops-portal/iac/helm/finops-portal/
  -f values.yaml | oya gate tenant-cost-labels-coverage --stdin`.
- `helm template ... -f values-kr.yaml -f values-eu.yaml -f
  values-us-healthcare.yaml` each renders cleanly.
