---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-network-foundation
impl_plan_id: IP-001-iac-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-network + ops-sre-reliability
acceptance_lanes: [helm-lint, kustomize-build, oya-governance-version-pinning-conformance, oya-governance-cedar-policy-spec]
---

# IP-001: IaC bootstrap — Helm + Kustomize + Terraform for the network cluster

## Intent

Land the IaC substrate for the `network` µservice cluster per ADR-0131:

- Helm chart `iac/helm/network/` with deployment + service + HPA + PDB + NetworkPolicy + PrometheusRule + ServiceMonitor templates.
- Kustomize base + per-pack overlays (pack-kr + pack-eu in P01; remaining packs follow).
- Terraform module references for OKE node pools, OCI Object Storage, OCI KMS keyring (per ADR-NET-0005 endorsement-chain), pack-aware Postgres + Redis + Meilisearch provisioning.
- Per-pack secret references in OpenBao.
- gVisor runtime class for media/document transcode worker per threat-model T-E-05.
- LTS pins: Postgres 16, Redis 7.2, Meilisearch 0.10.0, Cedar v4.2, ImageMagick 7.1, ffmpeg 7.x, ClamAV 1.x, OPSWAT 5.x.

## ChangeSet boundary

`iac/helm/network/` + `iac/kustomize/{base,overlays/pack-kr,overlays/pack-eu}/`.

## Concrete File Targets

| Path | Action |
|---|---|
| `iac/helm/network/Chart.yaml` | already created |
| `iac/helm/network/values.yaml` | already created |
| `iac/helm/network/templates/deployment.yaml` | already created |
| `iac/helm/network/templates/service.yaml` | already created |
| `iac/helm/network/templates/hpa.yaml` | already created |
| `iac/helm/network/templates/pdb.yaml` | already created |
| `iac/helm/network/templates/networkpolicy.yaml` | already created |
| `iac/helm/network/templates/prometheusrule.yaml` | already created |
| `iac/helm/network/templates/servicemonitor.yaml` | already created |
| `iac/kustomize/base/kustomization.yaml` | already created |
| `iac/kustomize/base/namespace.yaml` | create — Namespace `network` + `network-kr` per pack |
| `iac/kustomize/base/openbao-secret-references.yaml` | create — OpenBao secret ExternalSecret CRs |
| `iac/kustomize/overlays/pack-kr/kustomization.yaml` | already created |
| `iac/kustomize/overlays/pack-eu/kustomization.yaml` | already created |
| `iac/kustomize/overlays/pack-us/kustomization.yaml` | create (P01-scheduled-for-distinct-tracked-work to successor-IP IP) |
| `iac/terraform/network-pack-kr.tf` | reference cloud-iac module |

## Acceptance Gates

```bash
helm lint iac/helm/network
kustomize build iac/kustomize/overlays/pack-kr | kubeval --strict
kustomize build iac/kustomize/overlays/pack-eu | kubeval --strict
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance --microservice network
cargo run -p oya-dev-cli -- gate validate cedar-policy-spec --microservice network
```

## Test Plan

- `helm lint` exit 0.
- `kustomize build` exit 0 on every pack overlay.
- Smoke deploy to ephemeral cluster (kind): verify all Deployments + Services come up healthy.
- gVisor runtimeClass functional on media-transcode-worker pod (verified via runtimeClass field present).
- LTS pins verified by `oya-gate validate version-pinning-conformance`.

## Halt Conditions

- LTS pin missing or version skew — fix pinning.
- gVisor runtime class not available in target cluster — escalate to cloud-k8s.
- NetworkPolicy default-deny prevents legitimate cross-µservice traffic — investigate.

## Next IP

[`IP-002-cargo-workspace-bootstrap.md`](IP-002-cargo-workspace-bootstrap.md)

## References

- `microservices/network/iac/helm/network/values.yaml`.
- `microservices/network/multi-region.md`.
- `microservices/network/threat-model.md` (T-E-05 sandboxed transcode).
- `microservices/observability/IP-001-layer-a-grafana-stack-iac.md` (precedent shape).
- ADR-0131 (per-µservice flat layout); ADR-0140 (Cedar v4.2 default-deny).
