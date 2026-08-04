# ARC workspace capacity lane

Issue #1504 isolates disposable ARC build work from the Talos system filesystem.
The repository declaration has four parts:

1. `infra/talos/local/patches/ci-workspace-worker-1.yaml` allocates the general
   runner's fixed 48 GiB XFS user volume on worker 1's exact blank 150 GiB `/dev/vdb`.
   `ci-workspace-worker-2.yaml` allocates the live-PostgreSQL volume and retains
   its earlier general volume as an unadmitted rollback reserve.
2. `ci-workspace-storage.yaml` runs a separate local-path provisioner identity and
   admits `/var/mnt/ci-workspace-general` only on `oya-talos-worker-1` and
   `/var/mnt/ci-workspace-live-postgres` only on `oya-talos-worker-2`.
3. Both runner scale sets mount a 44 GiB generic ephemeral PVC at
   `/home/runner/_work`, pin to their respective node, and cap each scale set at one.
   Local-path does not enforce each 44 GiB request. Safety comes from max one runner
   per scale set and a separate fixed 48 GiB filesystem for each set; no capacity headroom
   inside either admitted filesystem is claimed.
4. `ci-workspace-alerts.yaml` covers node pressure, PVC free space, runner writable
   layer growth, eviction, delayed PVC cleanup, and ARC startup/queue latency.

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

Record all of the following before allowing either scale set above zero:

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
kubectl -n arc-runners get pvc,pods -o wide
```

The admitted general and live-PostgreSQL volumes must be ready on workers 1 and 2 respectively,
the provisioner identity must be `oyatie.io/ci-workspace-local-path`, each runner/PVC must bind
to its declared worker, and the existing `local-path` StorageClass/PVs must be byte-for-byte
unchanged. Worker 2's `ci-workspace-general` volume is rollback reserve only: it must remain
unlisted in `nodePathMap` and unused by PVCs.

## Safe rollback

Do not roll back to the earlier three-runner root-filesystem placement: that state
is the #1504 incident condition. First land a GitOps change setting both scale sets'
`maxRunners: 0`,
wait for all runner Pods and generic-ephemeral PVCs to disappear, and verify both
workspace directories are empty. Then revert the ARC storage/placement Applications.
Remove an admitted Talos `UserVolumeConfig` only after no Pod mounts it. Preserve worker 2's
unadmitted general volume as rollback reserve unless a separately reviewed destructive change
retires it. Talos leaves data on disk, so disk wiping is never part of this rollback. Branch
protection and `oya-ci-required` remain intact;
admission stays queued until safe runner capacity or hosted fallback exists.
