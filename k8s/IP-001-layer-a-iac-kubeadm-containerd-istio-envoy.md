---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-onprem-k8s-substrate
impl_plan_id: IP-001-layer-a-iac-kubeadm-containerd-istio-envoy
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, terraform-validate, governance-per-microservice-layout, governance-version-pinning-conformance, check-cis-k8s-benchmark]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: Layer-A IaC — kubeadm + containerd + Istio + Envoy + Cilium + CSI

## Intent

Author Helm charts (`istio-base`, `istiod`, `envoy-gateway`, `cni-cilium`) + OpenTofu modules (`kubeadm-cluster`, `containerd-config`) + Kustomize base + pack-kr overlay + CSI driver Helm deployments per backend (block-volume, object, file) under `microservices/cloud-k8s/iac/`. Deploys to the on-prem KR primary cell per ADR-0121 and to OCI OKE peers subsequent-to-M03-completion.

Per ADR-0121, kubeadm + containerd are Terraform-applied (lifecycle-managed by the cloud-iac µservice's OpenTofu runner; cloud-k8s declares the module); Istio + Envoy + Cilium are Helm-applied. CSI drivers are Helm-applied with per-backend values.

## ChangeSet boundary

One cohesive ChangeSet: 4 Helm charts (istio-base, istiod, envoy-gateway, cni-cilium) + 2 OpenTofu modules (kubeadm-cluster, containerd-config) + 1 Kustomize base + per-pack overlays (pack-kr at M01 launch) + CSI driver Helm deployments per backend. No Rust code; pure IaC + values. Per-pack secrets via OpenBao (no raw secrets in repo).

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/cloud-k8s/iac/helm/istio-base/{Chart.yaml,values.yaml,templates/*}` | create | Istio base CRDs + namespaces; LTS pin 1.29.2 |
| `microservices/cloud-k8s/iac/helm/istiod/{Chart.yaml,values.yaml,templates/*}` | create | istiod control plane; PeerAuthentication STRICT mesh-wide |
| `microservices/cloud-k8s/iac/helm/envoy-gateway/{Chart.yaml,values.yaml,templates/*}` | create | Envoy ingress gateway; TLS 1.3 only; WAF + rate-limit filter |
| `microservices/cloud-k8s/iac/helm/cni-cilium/{Chart.yaml,values.yaml,templates/*}` | create | Cilium 1.16 LTS; eBPF dataplane; NetworkPolicy + Hubble enabled |
| `microservices/cloud-k8s/iac/terraform/kubeadm-cluster/main.tf` | create | kubeadm init + node-join orchestration; per-pack region binding |
| `microservices/cloud-k8s/iac/terraform/containerd-config/main.tf` | create | containerd 2.3.0 + runc 1.4.0 install; seccomp / AppArmor base |
| `microservices/cloud-k8s/iac/kustomize/base/kustomization.yaml` | create | Shared base referencing all Helm releases |
| `microservices/cloud-k8s/iac/kustomize/overlays/pack-kr/kustomization.yaml` | create | pack-kr overlay (initial active pack) |
| `microservices/cloud-k8s/iac/helm/csi-block-volume/{Chart.yaml,values.yaml}` | create | OCI Block Volume CSI driver |
| `microservices/cloud-k8s/iac/helm/csi-object/{Chart.yaml,values.yaml}` | create | OCI Object / SeaweedFS CSI driver |
| `microservices/cloud-k8s/iac/helm/csi-file/{Chart.yaml,values.yaml}` | create | OCI File / CephFS CSI driver |

## Crate Naming

n/a — IaC only.

## Code Shape

`istiod/values.yaml`:

```yaml
istiod:
  image:
    tag: "1.29.2"  # LTS pin per docs/standards/cloud-k8s-stack.md
  meshConfig:
    accessLogFile: "/dev/stdout"
    enableTracing: true
    defaultConfig:
      proxyMetadata:
        ISTIO_META_DNS_CAPTURE: "true"
  peerAuthentication:
    mode: STRICT  # mesh-wide; per policy/cluster-isolation.md CI-05
  pilot:
    autoscaleMin: 3
    autoscaleMax: 20
    resources:
      requests:
        cpu: 1
        memory: 1Gi
```

`cni-cilium/values.yaml`:

```yaml
cilium:
  image:
    tag: "1.16.0"
  ipam:
    mode: "kubernetes"
  kubeProxyReplacement: "strict"  # Cilium replaces kube-proxy
  bpf:
    masquerade: true
    hostLegacyRouting: false  # eBPF host routing
  hubble:
    enabled: true
    relay:
      enabled: true
  encryption:
    enabled: true
    type: wireguard
  loadBalancer:
    mode: dsr
  prometheus:
    enabled: true
  operator:
    replicas: 2
```

`terraform/kubeadm-cluster/main.tf`:

```hcl
variable "pack" { type = string }
variable "region" { type = string }
variable "kubeadm_version" { type = string default = "v1.35.0" }
variable "control_plane_count" { type = number default = 1 }  # 3 subsequent-to-M04-completion
variable "worker_count" { type = number default = 17 }

resource "null_resource" "kubeadm_init" {
  triggers = {
    pack = var.pack
    version = var.kubeadm_version
  }
  provisioner "local-exec" {
    command = "bash ${path.module}/scripts/kubeadm-init.sh ${var.kubeadm_version}"
  }
}

# ... (additional resources for node-join orchestration)
```

## Acceptance Gates

```bash
helm lint microservices/cloud-k8s/iac/helm/istio-base
helm lint microservices/cloud-k8s/iac/helm/istiod
helm lint microservices/cloud-k8s/iac/helm/envoy-gateway
helm lint microservices/cloud-k8s/iac/helm/cni-cilium
helm lint microservices/cloud-k8s/iac/helm/csi-block-volume
helm lint microservices/cloud-k8s/iac/helm/csi-object
helm lint microservices/cloud-k8s/iac/helm/csi-file
terraform -chdir=microservices/cloud-k8s/iac/terraform/kubeadm-cluster validate
terraform -chdir=microservices/cloud-k8s/iac/terraform/containerd-config validate
kubectl --dry-run=client apply -k microservices/cloud-k8s/iac/kustomize/overlays/pack-kr
cargo run -p dev-cli -- gate validate per-microservice-layout --microservice cloud-k8s
cargo run -p dev-cli -- gate validate version-pinning-conformance
cargo run -p dev-cli -- gate validate cis-k8s-benchmark --microservice cloud-k8s
```

## Test Plan

Per PHASE-01 §"Per-IP Test Coverage Threshold" IaC class:
- ≥ 1 helm-install + helm-test smoke per chart
- 1 against kind/k3d cluster: bootstrap full pack-kr overlay; verify all components reach Ready ≤ 30min p99
- Test files: `microservices/cloud-k8s/tests/iac/{istio,envoy,cilium,csi-block,csi-object,csi-file}.bats`

## Halt Conditions

- Chart upstream-version drift from LTS pin — escalate to `docs/standards/cloud-k8s-stack.md` PR.
- OpenBao secret-reference resolution failure — block; engage cloud-secrets.
- kind cluster smoke fails — root-cause; do not mask.
- CIS benchmark lane fails — fix Helm values; do not waive.

## Next IP

[`IP-002-onprem-k8s-stack-standard.md`](IP-002-onprem-k8s-stack-standard.md)

## References

- ADR-0121 §"Layer | Component"
- ADR-0117 §"Cloud-native infrastructure progression"
- `microservices/cloud-k8s/policy/cluster-isolation.md` §"Invariant CI-05"
- `microservices/cloud-k8s/multi-region.md`
- `microservices/cloud-k8s/capacity-model.md`
