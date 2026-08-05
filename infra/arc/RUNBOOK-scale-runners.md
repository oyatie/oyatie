# Runbook: scale general ARC runners (dual-worker)

**Audience:** authorized human operator with cluster and Talos credentials.
**Agents do not apply** Talos patches, Helm upgrades, or Argo syncs. This file is
the apply checklist for the git declarations under `infra/arc/`.

**Goal of the R1 declaration:** unlock `maxRunners: 2` on `oya-arm64` by admitting
the general workspace path on both workers and requiring hostname anti-affinity so
each concurrent general runner owns a distinct 48 GiB physical volume.

**Out of scope:** CAS warm pools, Remote Execution, live-postgres `maxRunners`
(stays 1), credentials, and any change that raises general `maxRunners` above 2.

## Why not just raise maxRunners on one node

Local-path does not enforce PVC size. Each general claim requests 44 GiB; each
Talos user volume is 48 GiB. Two concurrent general runners on one volume would
overcommit disk. Dual-worker admission + required anti-affinity is the storage-safe
path without enlarging a single volume.

## Prerequisites (read-only)

Confirm volumes already exist and are mounted (first-time Talos patches are a
separate, reviewed operation — see `README.md` pre-apply):

```sh
talosctl -n <worker-1-ip> get volumestatus u-ci-workspace-general -o yaml
talosctl -n <worker-1-ip> get mountstatus u-ci-workspace-general -o yaml
talosctl -n <worker-2-ip> get volumestatus u-ci-workspace-general -o yaml
talosctl -n <worker-2-ip> get mountstatus u-ci-workspace-general -o yaml
talosctl -n <worker-2-ip> get volumestatus u-ci-workspace-live-postgres -o yaml
talosctl -n <worker-2-ip> get mountstatus u-ci-workspace-live-postgres -o yaml
kubectl get node oya-talos-worker-1 oya-talos-worker-2 --show-labels
```

Both workers must show Ready and arm64. Worker-2 general volume must be present
even if it was previously only a rollback reserve.

## Apply order (human)

GitOps (Argo) is the preferred path when the root app already tracks these files.
Otherwise apply in this order so storage admits the second general cell before
runners can schedule onto it.

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

### 2. General runner scale set values

Source of truth: `infra/arc/runner-scale-set-arm64-values.yaml`

Expected shape:

- `maxRunners: 2`
- `nodeSelector`: only `kubernetes.io/arch: arm64` (no hostname pin)
- `affinity.podAntiAffinity.requiredDuringSchedulingIgnoredDuringExecution` with
  `topologyKey: kubernetes.io/hostname` and `oya.io/ci-cell: general`
- workspace StorageClass: `oya-ci-workspace-general`, request `44Gi`

```sh
# Prefer Argo sync of the oya-arm64 / gha-runner-scale-set Application
# (valueFiles: infra/arc/runner-scale-set-arm64-values.yaml per infra/gitops/values.yaml).
#
# Helm fallback uses the same committed values file — never a scratchpad copy:
# helm upgrade --install <release> <chart> -n arc-runners \
#   -f infra/arc/runner-scale-set-arm64-values.yaml
```

Do **not** change `runner-scale-set-live-postgres-arm64-values.yaml` in this slice
(`maxRunners: 1`, hostname `oya-talos-worker-2`).

### 3. Verify two general runners can register

```sh
kubectl -n arc-runners get pods -o wide
kubectl -n arc-runners get pvc -o wide
# When two general jobs are queued, expect two runner pods on different nodes:
# kubectl -n arc-runners get pods -l oya.io/ci-cell=general -o wide
```

GitHub UI / API (operator token, not stored in git): the `oya-arm64` scale set
should show up to two registered runners when two jobs demand capacity.

Confirm:

1. At most one general runner Pod per `kubernetes.io/hostname`.
2. Each general PVC binds on the same node as its Pod (WaitForFirstConsumer).
3. Paths under `/var/mnt/ci-workspace-general` on each worker hold at most one
   active claim directory while a runner is live.
4. Live-postgres set remains max one runner on worker-2.
5. No DiskPressure / unexpected eviction on either worker.

## Safe scale-down / rollback

1. Land git change `maxRunners: 0` (or 1 with hostname re-pin if intentionally
   returning to single-worker general) for the general set; keep live-postgres
   behavior under its own plan.
2. Wait until general runner Pods and ephemeral PVCs are gone.
3. Only then revert nodePathMap or Talos volume admission if desired.
4. Never wipe disks as part of runner scale rollback.

## Explicit non-goals

- Does not enable CAS warm, Remote Execution, or workflow edits.
- Does not raise general `maxRunners` above 2.
- Does not claim XFS project quotas enforce PVC size without separate proof.
- Does not store or print credentials.
