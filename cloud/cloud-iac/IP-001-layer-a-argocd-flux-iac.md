---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-meta-iac-pipeline-substrate
impl_plan_id: IP-001-layer-a-argocd-flux-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud-iac
acceptance_lanes: [buck2-build, helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: Layer-A ArgoCD + Flux + Helm-controller + Kustomize-controller IaC

## Intent

Author Helm + Kustomize manifests for the GitOps reconciler stack (ArgoCD primary; Flux alternative for tenant choice) plus the Flux Helm-controller + Kustomize-controller under `microservices/cloud-iac/iac/cue-krm-packages/`. Deploys to the dedicated cloud-iac control-plane Kubernetes cluster per PRD Layer-A. Versions pinned to LTS per `docs/standards/observability-slo.md` §"Supply-chain conformance".

## ChangeSet boundary

One cohesive ChangeSet: 4 CUE/KRM package bundles (ArgoCD + Flux + helm-controller + kustomize-controller) + shared Kustomize base + pack-kr overlay. No Rust code; pure IaC. Per-pack secret references via OpenBao.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/cloud-iac/iac/cue-krm-packages/argocd/Chart.yaml` | create | upstream dep on argoproj/argo-cd chart at pinned LTS |
| `microservices/cloud-iac/iac/cue-krm-packages/argocd/values.yaml` | create | HA replicas + Valkey Sentinel + admission webhook |
| `microservices/cloud-iac/iac/cue-krm-packages/flux/{Chart.yaml,values.yaml}` | create | source-controller + kustomize-controller + helm-controller |
| `microservices/cloud-iac/iac/cue-krm-packages/helm-controller/{Chart.yaml,values.yaml}` | create | Flux Helm-controller standalone deployment |
| `microservices/cloud-iac/iac/cue-krm-packages/kustomize-controller/{Chart.yaml,values.yaml}` | create | Flux Kustomize-controller standalone deployment |
| `microservices/cloud-iac/iac/kustomize/base/kustomization.yaml` | create | shared base referencing all 4 charts |
| `microservices/cloud-iac/iac/kustomize/overlays/pack-kr/kustomization.yaml` | create | pack-kr overlay (initial active pack) |

## Code Shape

```yaml
# argocd/values.yaml
argo-cd:
  global:
    image:
      registry: quay.io
      repository: argoproj/argocd
      tag: "v2.13.0"  # LTS pinned per docs/standards/observability-slo.md
  redis-ha: # counterpart-fact: upstream Argo CD CUE/KRM package key; backing substrate is Valkey.
    enabled: true
    replicas:
      servers: 3
      sentinels: 3
  controller:
    replicas: 3
    metrics:
      enabled: true
  repoServer:
    replicas: 3
  server:
    replicas: 2
    extensions:
      enabled: true
    config:
      admin.enabled: "false"  # admin disabled; SSO only
      url: "https://argocd-kr.oyatie.dev"
  applicationSet:
    replicas: 1
    webhook:
      enabled: true
```

## Acceptance Gates

```bash
helm lint microservices/cloud-iac/iac/cue-krm-packages/argocd
helm lint microservices/cloud-iac/iac/cue-krm-packages/flux
helm lint microservices/cloud-iac/iac/cue-krm-packages/helm-controller
helm lint microservices/cloud-iac/iac/cue-krm-packages/kustomize-controller
kubectl --dry-run=client apply -k microservices/cloud-iac/iac/kustomize/overlays/pack-kr
buck2 build //:repo-hygiene-automation-check # native Buck2/Prow gate evidence for per-microservice-layout --microservice cloud-iac
buck2 build //:repo-hygiene-automation-check # native Buck2/Prow gate evidence for version-pinning-conformance
```

## Test Plan

- Per Phase-01 IaC class: ≥ 1 helm-install + helm-test smoke per chart; 1 against kind/k3d.
- Test files: `microservices/cloud-iac/tests/iac/{argocd,flux,helm-controller,kustomize-controller}.bats`.
- E2E: spin up kind cluster; apply pack-kr overlay; verify all 4 components Ready within 10min.

## Halt Conditions

- Chart upstream-version drift from LTS pin — escalate to docs/standards PR.
- OpenBao secret-reference resolution failure — block.
- kind smoke fails — root-cause.

## Next IP

[`IP-002-layer-a-opentofu-iac.md`](IP-002-layer-a-opentofu-iac.md)

## References

- PRD-cloud-iac §"Layer-A".
- `microservices/cloud-iac/multi-region.md`.
- `microservices/cloud-iac/capacity-model.md`.
- `docs/standards/observability-slo.md` §"Supply-chain conformance".
- ArgoCD HA chart — `github.com/argoproj/argo-helm/tree/main/charts/argo-cd`.
- Flux helm-charts — `github.com/fluxcd-community/helm-charts`.

## DR posture (per ADR-0343)

- Target source: `microservices/cloud-iac/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `seaweedfs_replicated`, `postgres_wal_g`].
- Surface evidence: `microservices/cloud-iac/runbooks/restore-drill-quarterly.md`, `microservices/cloud-iac/runbooks/seaweedfs-volume-failover.md`, `microservices/cloud-iac/manifest.json`, `microservices/cloud-iac/IP-001-layer-a-argocd-flux-iac.md`.
