---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-sites-foundation
impl_plan_id: IP-001-iac-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-sites + ops-sre-reliability
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: IaC bootstrap — Helm + Kustomize for Postgres + Valkey + Meilisearch + cert-manager + libvips-worker

## Intent

Author Helm + Kustomize manifests for the sites µservice substrate.
Postgres 16 LTS for site/page/cms-collection event store (RLS per-tenant
per ADR-0117); Valkey 8.1 (RESP3 wire-compatible) for the page-render + CMS-collection
cache; Meilisearch 0.10.0 LTS for per-tenant site search;
cert-manager 1.16 LTS for ACME cert reconciliation (per ADR-SITES-0004);
libvips 8.16 worker pods for image pipeline (per ADR-SITES-0007);
OpenBao for per-tenant DEK envelope encryption + ACME private-key
storage. Pack-aware overlays for the 11 packs.

## ChangeSet boundary

12 Helm template files + Kustomize base + per-pack overlays (pack-kr
first; eu/us/us-healthcare/jp/sg/au/in/br/ae/ksa follow). No Rust
code; pure IaC + values. All secrets via
`${openbao:secret/sites/...}` SecretReferences.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/sites/iac/helm/Chart.yaml` | created |
| `microservices/sites/iac/helm/values.yaml` | created |
| `microservices/sites/iac/helm/templates/deployment.yaml` | created (per-BC) |
| `microservices/sites/iac/helm/templates/service.yaml` | created (per-BC) |
| `microservices/sites/iac/helm/templates/hpa.yaml` | created |
| `microservices/sites/iac/helm/templates/pdb.yaml` | created |
| `microservices/sites/iac/helm/templates/networkpolicy.yaml` | created |
| `microservices/sites/iac/helm/templates/servicemonitor.yaml` | created |
| `microservices/sites/iac/helm/templates/prometheusrule.yaml` | created |
| `microservices/sites/iac/helm/templates/cronjob.yaml` | created (cert-renew CronJob; sitemap-regen CronJob) |
| `microservices/sites/iac/kustomize/base/kustomization.yaml` | created |
| `microservices/sites/iac/kustomize/base/namespace.yaml` | created |
| `microservices/sites/iac/kustomize/base/serviceaccount.yaml` | created |
| `microservices/sites/iac/kustomize/overlays/pack-kr/kustomization.yaml` | created |
| `microservices/sites/iac/kustomize/overlays/pack-eu/kustomization.yaml` | created |
| `microservices/sites/iac/kustomize/overlays/pack-us/kustomization.yaml` | created |
| `microservices/sites/iac/kustomize/overlays/pack-us-healthcare/kustomization.yaml` | created |

## Crate Naming

n/a — IaC only.

## Acceptance Gates

```bash
helm lint microservices/sites/iac/helm
kubectl --dry-run=client apply -k microservices/sites/iac/kustomize/overlays/pack-kr
buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice sites
buck2 build //:quality-lane-registry-authority-check # lane=version-pinning-conformance
```

## Test Plan

- helm lint per chart against kind/k3d cluster.
- E2E smoke: spin kind cluster; apply pack-kr overlay; verify all 11
  BC deployments + Postgres + Valkey + Meilisearch + cert-manager
  reach Ready within 10 min.
- ACME smoke: bind `acme-test.test.oyatie.dev`; verify cert issuance
  against Let's Encrypt staging.

## Halt Conditions

- Upstream chart version drifts past LTS pin — escalate per
  `docs/standards/observability-slo.md`.
- OpenBao secret-reference resolution fails — block.
- Helm chart fails kubectl-dry-run — root-cause; do not mask.

## Next IP

[`IP-002-site-bc-kernel.md`](IP-002-site-bc-kernel.md)

## References

- ADR-0117 (data residency); ADR-0131 (per-µservice flat layout); ADR-0133.
- ADR-SITES-0001 (Loro CRDT); ADR-SITES-0004 (ACME); ADR-SITES-0007 (image pipeline).
- Postgres CloudNativePG operator — `cloudnative-pg.io`.
- Valkey cluster mode — `valkey.io/topics/cluster-tutorial/`.
- Meilisearch — `meilisearch.com/docs`.
- cert-manager — `cert-manager.io/docs`.
- libvips — `libvips.github.io/libvips`.
