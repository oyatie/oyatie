# Spoke cluster definitions (CAPI Cluster CRs) — Helm chart

One Talos cluster per **cell** (ADR-0009 cell-per-region), provisioned declaratively by CAPI on the
control plane. This is a **Helm chart**: a cell is an entry in `values.cells`, not a copied file
(ADR-0375 D3). Pick a substrate per cell, fill its block, render, apply.

```sh
helm template spokes infra/capi/clusters -f my-cells.yaml | kubectl apply -f -
```

| Substrate | `substrate:` | Infra provider | Use |
|---|---|---|---|
| OCI | `oci` | CAPOCI | OCI-region cells (live today: chuncheon) |
| AWS | `aws` | CAPA (unmanaged EC2, not EKS) | AWS-region cells |
| Metal³ | `metal3` | CAPM3 + Ironic | on-prem/colo bare metal (needs BMC + BareMetalHost CRs) |

Each cell renders the same CR set — only the infra CRs differ by substrate:
`Cluster` → `<Infra>Cluster` + `TalosControlPlane` (+`<Infra>MachineTemplate`) + `MachineDeployment`
(+`TalosConfigTemplate` +`<Infra>MachineTemplate`). All set `cni:none` + `proxy.disabled` (Cilium is
installed by the CRS); workers carry the `katacontainers.io/kata-runtime` label + vhost modules
(Kata pool, ADR-0147/0338). Every Cluster is labelled `oya.io/bootstrap=true` → the CRS drops Cilium
+ Argo CD on it; the cell's Argo CD then pulls `infra/gitops` (override `cell` per spoke).

## The ~8 cells
Add one entry to `cells` per cell (kr, jp, eu, ksa, us, us-gov, uae, …), set `substrate` + the
substrate block (OCIDs / AMIs / image URL + VIP + sizing), commit. `values-example.yaml` shows one
worked cell per substrate. Each cell is an independent failure domain (INV-CELL-ISOLATION).

## apiVersions (verified against released CRDs)
Infra CRs: CAPOCI/CAPA `infrastructure.cluster.x-k8s.io/v1beta2`; Metal3 `…/v1beta1`. Talos providers:
`TalosControlPlane` = `controlplane.cluster.x-k8s.io/v1alpha3`, `TalosConfig*` =
`bootstrap.cluster.x-k8s.io/v1alpha3` — verified against the released CABPT v0.6.x / CACPPT v0.5.x
CRDs (this is the current stable line; v1beta2 / ClusterClass is alpha). The chart selects the right
infra apiVersion per substrate in `templates/_helpers.tpl`.

## Validation
`helm lint infra/capi/clusters` and `helm template … -f values-example.yaml` render offline. The
provider CRDs (OCICluster/AWSCluster/Metal3Cluster/TalosControlPlane) exist only after `clusterctl
init` (infra/capi/init.sh) on the control plane, so server-side validation
(`kubectl --dry-run=server`) is gated on the control plane + providers being up.
