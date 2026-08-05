# ARC workspace capacity lane

Issue #1504 isolates disposable ARC build work from the Talos system filesystem.
The repository declaration has four parts:

1. `infra/talos/local/patches/ci-workspace-worker-1.yaml` allocates the general
   runner's fixed **120 GiB** XFS user volume on worker 1's blank 150 GiB `/dev/vdb`.
   `ci-workspace-worker-2.yaml` allocates both a general volume (dual-worker general
   cell, **120 GiB**) and the live-PostgreSQL volume (**48 GiB**) on worker 2's
   blank 150 GiB `/dev/vdb`.
2. `ci-workspace-storage.yaml` runs a separate local-path provisioner identity and
   admits `/var/mnt/ci-workspace-general` on **both** `oya-talos-worker-1` and
   `oya-talos-worker-2`, and `/var/mnt/ci-workspace-live-postgres` only on
   `oya-talos-worker-2`.
3. The general scale set (`runner-scale-set-arm64-values.yaml`) mounts a 44 GiB
   generic ephemeral PVC at `/home/runner/_work`, pins only `kubernetes.io/arch:
   arm64` (no hostname pin), prefers hostname anti-affinity, hard-caps spread with
   topologySpread DoNotSchedule maxSkew=1 on `oya.io/ci-cell: general`, and sets
   `maxRunners: 4` (≤2 general runners per node on 120 GiB volumes). Local-path
   does not enforce each 44 GiB request; topology spread is the packing guard.
   The live-PostgreSQL scale set stays `maxRunners: 1` and hostname-pinned to
   worker 2 on its own 48 GiB volume.
4. `ci-workspace-alerts.yaml` covers node pressure, PVC free space, runner writable
   layer growth, eviction, delayed PVC cleanup, and ARC startup/queue latency.

**CPU/RAM** for the Talos VMs themselves (live was 5 vCPU / ~30 GiB workers on a
128 GiB host) is documented in
[`CAPACITY-PROFILE-CI-HEAVY.md`](./CAPACITY-PROFILE-CI-HEAVY.md) — target workers
**8 vCPU / 48 GiB**; QEMU recreate is a human maintenance window.

Human apply steps for raising concurrency live under
[`RUNBOOK-scale-runners.md`](./RUNBOOK-scale-runners.md). This declaration slice
does **not** activate CAS warm pools or Remote Execution.

The current local cluster has no `monitoring.coreos.com` CRDs or `observability`
namespace. The PrometheusRule is therefore a reviewed telemetry contract but is
not registered as an Argo Application in this slice: doing so would make the root
Application fail. ARC metric exposition is enabled now; rule rollout is blocked
on the separately governed observability substrate and must include CRD/scrape/
alert-routing readback.

This is desired state, not rollout or closure evidence. Issue #1504 remains open
until an exact-SHA cold maximum-supported-concurrency exercise completes without
DiskPressure or eviction and shows automatic PVC/directory cleanup.

## Pre-apply readback

Before applying the Talos patch, an authorized operator records:

```sh
talosctl -n <worker-1-ip> get disks -o yaml
talosctl -n <worker-1-ip> get discoveredvolumes -o yaml
talosctl -n <worker-2-ip> get disks -o yaml
talosctl -n <worker-2-ip> get discoveredvolumes -o yaml
kubectl get node oya-talos-worker-1 oya-talos-worker-2 -o yaml
kubectl get pv,pvc -A
```

`/dev/vdb` on both workers must still be a blank, non-system 150 GiB disk before
the respective patch is first applied. CNPG, registry,
NativeLink, OpenBao, and their PVCs are out of scope and must not be modified.

## Post-apply readback

Record all of the following before allowing the general scale set above one
runner, or the live-PostgreSQL scale set above zero:

```sh
talosctl -n <worker-1-ip> get volumestatus u-ci-workspace-general -o yaml
talosctl -n <worker-1-ip> get mountstatus u-ci-workspace-general -o yaml
talosctl -n <worker-2-ip> get volumestatus u-ci-workspace-general -o yaml
talosctl -n <worker-2-ip> get volumestatus u-ci-workspace-live-postgres -o yaml
talosctl -n <worker-2-ip> get mountstatus u-ci-workspace-general -o yaml
talosctl -n <worker-2-ip> get mountstatus u-ci-workspace-live-postgres -o yaml
kubectl get node oya-talos-worker-1 oya-talos-worker-2 --show-labels
kubectl get storageclass oya-ci-workspace-general oya-ci-workspace-live-postgres -o yaml
kubectl -n oya-ci-workspace-storage get deploy,pods,configmap -o wide
kubectl -n oya-ci-workspace-storage get configmap local-path-config -o jsonpath='{.data.config\.json}{"\n"}'
kubectl -n arc-runners get pvc,pods -o wide
```

Both workers' general volumes and worker 2's live-PostgreSQL volume must be ready,
the provisioner identity must be `oyatie.io/ci-workspace-local-path`, the
`nodePathMap` must list `/var/mnt/ci-workspace-general` on both workers, general
runner PVCs must bind one-per-node under anti-affinity, live-postgres must stay
on worker 2, and the existing `local-path` StorageClass/PVs must be byte-for-byte
unchanged.

## Safe rollback

Do not roll back to the earlier three-runner root-filesystem placement: that state
is the #1504 incident condition. First land a GitOps change setting both scale sets'
`maxRunners: 0`,
wait for all runner Pods and generic-ephemeral PVCs to disappear, and verify both
workspace directories are empty. Then revert the ARC storage/placement Applications.
Remove an admitted Talos `UserVolumeConfig` only after no Pod mounts it. Worker 2's
general volume is part of dual-worker capacity; retiring it requires first lowering
general `maxRunners` to 1 and re-pinning general to worker 1 (or an equivalent
reviewed plan). Talos leaves data on disk, so disk wiping is never part of this
rollback. Branch protection and `oya-ci-required` remain intact;
admission stays queued until safe runner capacity or hosted fallback exists.
