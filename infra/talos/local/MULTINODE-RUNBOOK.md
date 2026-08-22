# Multi-node Talos local bring-up (ADR-0381 D2)

Operator runbook for the 7-VM topology (3 CP + 2 worker + 1 CI specialty + 1 storage specialty) on a single macOS host via vfkit + Apple Virtualization.framework. Replaces the single-node default in `talos-local.sh up --role single` once the host has the headroom (recommended: 32 GiB+ RAM, ~30+ GiB free disk).

## Target topology

| Node | Role | vCPU | RAM | Disk | Cell label | Taint |
|------|------|------|-----|------|-----------|-------|
| `cp-0` | control-plane | 2 | 2 GiB | 20 GiB | `oya.cell/foundation=true` | (default CP NoSchedule) |
| `cp-1` | control-plane | 2 | 2 GiB | 20 GiB | `oya.cell/foundation=true` | (default CP NoSchedule) |
| `cp-2` | control-plane | 2 | 2 GiB | 20 GiB | `oya.cell/foundation=true` | (default CP NoSchedule) |
| `worker-0` | worker (tenant) | 4 | 8 GiB | 20 GiB | `oya.cell/tenant=true` | (none) |
| `worker-1` | worker (tenant) | 4 | 8 GiB | 20 GiB | `oya.cell/tenant=true` | (none) |
| `ci-0` | worker (CI specialty) | 6 | 16 GiB | 40 GiB | `oya.cell/ci=true` | `dedicated=ci:NoSchedule` |
| `storage-0` | worker (storage specialty) | 2 | 8 GiB | 100 GiB | `oya.cell/storage=true` | `dedicated=storage:NoSchedule` |
| **Total** | **7 VMs** | **22 vCPU** | **46 GiB** | **220 GiB** | | |

**Dial-down (16 GiB host):** 1 CP + 1 worker + 1 CI specialty, SeaweedFS co-located on worker. Loses CP HA + storage-pool isolation; document the trade-off when running this profile.

**Hyperscaler-lens** (per memory `hyperscaler-lens-architectural-filter`): Talos is Apache 2, actively maintained (Sidero Labs quarterly), runs anywhere — the OSS analogue of GKE Container-Optimized OS / EKS Bottlerocket / AKS CBL-Mariner. Multi-pool topology IS the GKE/EKS/AKS node-pool topology. Passes (a)-(d).

## Bring-up

Each VM is brought up via `talos-local.sh up --role <role> --name <name>`, with a per-cell config patch applied via `talosctl apply-config --config-patch @<patch>.yaml`. The patches live in `infra/talos/local/patches/`.

### 1. First control-plane node — `cp-0`

Bootstraps etcd and generates the cluster config bundle:

```sh
infra/talos/local/talos-local.sh up --role control-plane --name local-cp-0 \
  --cpus 2 --ram-gb 2 --disk-gb 20
# After the VM boots and the IP shows in dhcpd_leases, apply the cell patch:
WORKDIR="${OYATIE_TALOS_WORKDIR:-$HOME/.oya/talos-local}"
CP0_IP=$(infra/talos/local/talos-local.sh status | awk '/local-cp-0/ {print $NF}')
talosctl apply-config --insecure --nodes "$CP0_IP" \
  --file "$WORKDIR/controlplane.yaml" \
  --config-patch @infra/talos/local/patches/cell-foundation.yaml
talosctl bootstrap --nodes "$CP0_IP" --talosconfig "$WORKDIR/talosconfig"
```

### 2. Additional control-plane nodes — `cp-1`, `cp-2`

Join the etcd quorum. Each gets the foundation cell patch:

```sh
for n in cp-1 cp-2; do
  infra/talos/local/talos-local.sh up --role control-plane --name local-$n \
    --cpus 2 --ram-gb 2 --disk-gb 20
  IP=$(infra/talos/local/talos-local.sh status | awk "/local-$n/ {print \$NF}")
  talosctl apply-config --insecure --nodes "$IP" \
    --file "$WORKDIR/controlplane.yaml" \
    --config-patch @infra/talos/local/patches/cell-foundation.yaml
done
```

Wait for etcd quorum to converge: `talosctl etcd status --nodes $CP0_IP` should show 3 members.

### 3. Tenant workers — `worker-0`, `worker-1`

```sh
for n in worker-0 worker-1; do
  infra/talos/local/talos-local.sh up --role worker --name local-$n \
    --cpus 4 --ram-gb 8 --disk-gb 20
  IP=$(infra/talos/local/talos-local.sh status | awk "/local-$n/ {print \$NF}")
  talosctl apply-config --insecure --nodes "$IP" \
    --file "$WORKDIR/worker.yaml" \
    --config-patch @infra/talos/local/patches/cell-tenant.yaml
done
```

### 4. CI specialty — `ci-0`

```sh
infra/talos/local/talos-local.sh up --role worker --name local-ci-0 \
  --cpus 6 --ram-gb 16 --disk-gb 40
IP=$(infra/talos/local/talos-local.sh status | awk '/local-ci-0/ {print $NF}')
talosctl apply-config --insecure --nodes "$IP" \
  --file "$WORKDIR/worker.yaml" \
  --config-patch @infra/talos/local/patches/cell-ci.yaml
```

### 5. Storage specialty — `storage-0`

```sh
infra/talos/local/talos-local.sh up --role worker --name local-storage-0 \
  --cpus 2 --ram-gb 8 --disk-gb 100
IP=$(infra/talos/local/talos-local.sh status | awk '/local-storage-0/ {print $NF}')
talosctl apply-config --insecure --nodes "$IP" \
  --file "$WORKDIR/worker.yaml" \
  --config-patch @infra/talos/local/patches/cell-storage.yaml
```

### 6. Verify topology

```sh
kubectl --kubeconfig "$WORKDIR/kubeconfig" get nodes -o wide \
  --show-labels --selector='oya.cell/role'
kubectl --kubeconfig "$WORKDIR/kubeconfig" describe nodes \
  | grep -E '^Name:|oya\.cell/|Taints:'
```

Expected: 3 CP nodes labelled `oya.cell/foundation=true`, 2 worker `oya.cell/tenant=true`, 1 ci-specialty `oya.cell/ci=true` + taint `dedicated=ci`, 1 storage-specialty `oya.cell/storage=true` + taint `dedicated=storage`.

### 7. Pin existing workloads to their cells

Update pod templates to declare cell affinity (these edits land in follow-up PRs once the multi-node cluster is the canonical baseline):

- `infra/seaweedfs/seaweedfs.k8s.yaml` — add `nodeSelector: { oya.cell/storage: "true" }` + `tolerations: dedicated=storage:NoSchedule` to the Deployment spec.
- `infra/ci-webhook-gateway/buildkit-build.yaml` — add `nodeSelector: { oya.cell/ci: "true" }` + `tolerations: dedicated=ci:NoSchedule` to both the buildkitd Deployment and the Job.

Each PR re-runs the gate and verifies workloads continue to schedule on the intended cell.

## Tear-down

```sh
infra/talos/local/talos-local.sh down --all   # stops + deletes every tracked VM
```

## Notes

- vfkit NAT — leases come from `/var/db/dhcpd_leases`; if a node's IP doesn't show within ~5 minutes after `up`, see `talos-local.sh` header for the socket_vmnet fallback.
- ADR-0381 D3 cell-boundary NetworkPolicies are enforced by Cilium L3/L4 (ADR-0148); they evaluate per-pod regardless of node placement, so the policies continue to work unchanged once the multi-node topology lands.
- Production fleet (non-local) uses the same cell labels via CAPI/Cluster API per ADR-0375; the local layout matches production for fidelity.
