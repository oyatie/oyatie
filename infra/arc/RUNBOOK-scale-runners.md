# Runbook: scale general ARC runners (CI-heavy dual-worker)

**Audience:** authorized human operator with cluster and Talos credentials.
**Agents do not apply** Talos patches, Helm upgrades, or Argo syncs. This file is
the apply checklist for the git declarations under `infra/arc/`.

**Goal of this declaration:** unlock `maxRunners: 4` on `oya-arm64` across two
workers with **120 GiB** general UserVolumes and hard topology spread
(≤2 general runners per hostname). Companion profile:
[`CAPACITY-PROFILE-CI-HEAVY.md`](./CAPACITY-PROFILE-CI-HEAVY.md) (CPU/RAM target
and host budget).

**Out of scope:** CAS warm pools, Remote Execution, live-postgres `maxRunners`
(stays 1), credentials, and any change that raises general `maxRunners` above 4
without a new capacity plan.

## Why not just raise maxRunners

Local-path does not enforce PVC size. Each general claim requests 44 GiB.
With only 48 GiB volumes, two concurrent general runners on one volume would
overcommit disk. This slice grows each general volume to **120 GiB** and uses
**DoNotSchedule topology spread maxSkew=1** so each node hosts at most ~2 claims
(2×44 + 4 reserve ≤ 120).

Separately, live workers today are **5 vCPU / ~30 GiB**. That starves CPU when
two general runners (2 CPU request each) share a node with live-postgres or
system pods. Target workers are **8 vCPU / 48 GiB** (see capacity profile).
QEMU does not hot-resize vCPU/RAM — that step is a planned recreate.

## Prerequisites (read-only)

```sh
talosctl -n <worker-1-ip> get volumestatus u-ci-workspace-general -o yaml
talosctl -n <worker-1-ip> get mountstatus u-ci-workspace-general -o yaml
talosctl -n <worker-2-ip> get volumestatus u-ci-workspace-general -o yaml
talosctl -n <worker-2-ip> get mountstatus u-ci-workspace-general -o yaml
talosctl -n <worker-2-ip> get volumestatus u-ci-workspace-live-postgres -o yaml
talosctl -n <worker-2-ip> get mountstatus u-ci-workspace-live-postgres -o yaml
kubectl get nodes -o custom-columns=NAME:.metadata.name,CPU:.status.capacity.cpu,MEM:.status.capacity.memory
kubectl get node oya-talos-worker-1 oya-talos-worker-2 --show-labels
```

Both workers must show Ready and arm64. General volumes should report **120 GiB**
after step 1; until then keep `maxRunners` ≤ number of 48 GiB-safe concurrent
slots (historically 2 with required one-per-node anti-affinity).

## Apply order (human)

GitOps (Argo) is preferred when the root app tracks these files. Otherwise:

### 1. Workspace provisioner / nodePathMap

Desired `config.json` nodePathMap (from `ci-workspace-storage.yaml`):

- `oya-talos-worker-1` → `/var/mnt/ci-workspace-general`
- `oya-talos-worker-2` → `/var/mnt/ci-workspace-general`, `/var/mnt/ci-workspace-live-postgres`

```sh
# Prefer Argo sync of the Application that owns infra/arc/ci-workspace-storage.yaml.
# Fallback (only if that Application is not yet live):
kubectl apply -f infra/arc/ci-workspace-storage.yaml

kubectl -n oya-ci-workspace-storage get configmap local-path-config \
  -o jsonpath='{.data.config\.json}{"\n"}'
kubectl -n oya-ci-workspace-storage rollout status deploy/oya-ci-workspace-local-path
```

Readback must show general on **both** workers. Do not introduce
`DEFAULT_PATH_FOR_NON_LISTED_NODES` for the CI workspace provisioner.

### 2. Grow general UserVolume to 120 GiB (if still 48 GiB)

Source: `infra/talos/local/patches/ci-workspace-worker-{1,2}.yaml`.

Data disk is already 150 GiB on the live QEMU cell; only the UserVolume bound
needs to grow. This is a **Talos machine-config** change — drain the worker if
required by your patch procedure; do not destroy volumes with active claims.

### 3. General runner scale set values

Source of truth: `infra/arc/runner-scale-set-arm64-values.yaml`

Expected shape:

- `maxRunners: 4`
- `nodeSelector`: only `kubernetes.io/arch: arm64` (no hostname pin)
- preferred hostname anti-affinity + `topologySpreadConstraints` DoNotSchedule
  maxSkew=1 on `oya.io/ci-cell: general`
- workspace StorageClass: `oya-ci-workspace-general`, request `44Gi`

```sh
# Prefer Argo sync of the oya-arm64 / gha-runner-scale-set Application.
#
# Helm fallback uses the same committed values file — never a scratchpad copy:
helm upgrade --install oya-arm64 \
  oci://ghcr.io/actions/actions-runner-controller-charts/gha-runner-scale-set \
  -n arc-runners --version 0.14.2 \
  -f infra/arc/runner-scale-set-arm64-values.yaml
```

Do **not** change `runner-scale-set-live-postgres-arm64-values.yaml` in this slice
(`maxRunners: 1`, hostname `oya-talos-worker-2`).

### 4. Verify concurrent general runners

```sh
kubectl -n arc-runners get pods -o wide
kubectl -n arc-runners get pvc -o wide
# Under load: up to 4 general pods, at most 2 per kubernetes.io/hostname
kubectl -n arc-runners get pods -l oya.io/ci-cell=general -o wide
kubectl describe nodes | rg -A8 'Allocated resources'
```

Confirm:

1. At most two general runner Pods per `kubernetes.io/hostname` under max=4.
2. Each general PVC binds on the same node as its Pod (WaitForFirstConsumer).
3. Paths under `/var/mnt/ci-workspace-general` do not overcommit past 120 GiB.
4. Live-postgres set remains max one runner on worker-2.
5. No DiskPressure / unexpected eviction; watch CPU steal if workers still 5 vCPU.

### 5. Optional: reprovision Talos CPU/RAM (planned window)

See [`CAPACITY-PROFILE-CI-HEAVY.md`](./CAPACITY-PROFILE-CI-HEAVY.md) section C.
Target: CP **4 vCPU / 12 GiB**, workers **8 vCPU / 48 GiB** each.  
**Destructive** to the QEMU cell — not an agent default action.

## Safe scale-down / rollback

1. Land git change `maxRunners: 0`–`2` for the general set; keep live-postgres
   under its own plan.
2. Wait until surplus general Pods and ephemeral PVCs are gone.
3. Only then revert nodePathMap or Talos volume sizes if desired.
4. Never wipe disks as part of runner scale rollback.

## Explicit non-goals

- Does not enable CAS warm, Remote Execution, or workflow edits.
- Does not raise general `maxRunners` above 4 without a new plan.
- Does not claim XFS project quotas enforce PVC size without separate proof.
- Does not store or print credentials.
- Does not auto-destroy the live `oya-talos` QEMU cluster.
