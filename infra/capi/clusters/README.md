# Spoke cluster definitions (CAPI Cluster CRs)

One Talos cluster per **cell** (ADR-0009 cell-per-region), provisioned declaratively by CAPI on the
hub. Three substrate templates — pick per cell, fill the `${PLACEHOLDERS}`, git-commit; CAPI reconciles.

| Substrate | Template | Infra provider | Use |
|---|---|---|---|
| OCI | `oci/cluster.yaml` | CAPOCI | OCI-region cells (live today: chuncheon) |
| AWS | `aws/cluster.yaml` | CAPA (unmanaged EC2, not EKS) | AWS-region cells |
| Metal³ | `metal3/cluster.yaml` | CAPM3 + Ironic | on-prem/colo bare metal (needs BMC + BareMetalHost CRs) |

Each template is the same shape — only the infra CRs differ:
`Cluster` → `<Infra>Cluster` + `TalosControlPlane` (+`<Infra>MachineTemplate`) + `MachineDeployment`
(+`TalosConfigTemplate` +`<Infra>MachineTemplate`). All set `cni:none` + `proxy.disabled` (Cilium is
installed by the CRS); workers carry the `katacontainers.io/kata-runtime` label + vhost modules
(Kata pool, ADR-0147/0338). Every Cluster is labelled `oya.io/bootstrap=true` → the CRS drops Cilium
+ Argo CD on it; the cell's Argo CD then pulls `infra/gitops` (override `cell` per spoke).

## The ~8 cells
Map cells (kr, jp, eu, ksa, us, us-gov, uae, …) to a substrate + region, copy the matching template
to e.g. `oci/us.yaml`, fill placeholders (OCIDs / AMIs / BMC + region + sizing), commit. Each = an
independent failure domain (INV-CELL-ISOLATION).

## Not validated offline
These reference provider CRDs (OCICluster, AWSCluster, Metal3Cluster, TalosControlPlane) that only
exist after `clusterctl init` (infra/capi/init.sh) on the hub. Validate with
`clusterctl generate cluster` / `kubectl --dry-run=server` once the hub + providers are up.
API-version pins (v1beta1/v1beta2, v1alpha3 for the Talos providers) follow the stable-line research
(CABPT v0.6.x / CACPPT v0.5.x) — reconcile if you adopt the v1beta2 alpha (ClusterClass).
