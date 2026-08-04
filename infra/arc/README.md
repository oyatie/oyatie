# ARC workspace capacity lane

Issue #1504 isolates disposable ARC build work from the Talos system filesystem.
The repository declaration has four parts:

1. `infra/talos/local/patches/ci-workspace-worker-2.yaml` allocates a bounded
   96 GiB XFS user volume on the exact blank 150 GiB `/dev/vdb` and labels the node.
2. `ci-workspace-storage.yaml` runs a separate local-path provisioner identity and
   StorageClass rooted only at `/var/mnt/ci-workspace` on `oya-talos-worker-2`.
3. Both runner scale sets mount a 44 GiB generic ephemeral PVC at
   `/home/runner/_work`, pin runners to that node, and cap each scale set at one.
   Two claims consume at most 88 GiB of the 96 GiB physical aggregate boundary.
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
talosctl -n <worker-2-ip> get disks -o yaml
talosctl -n <worker-2-ip> get discoveredvolumes -o yaml
kubectl get node oya-talos-worker-2 -o yaml
kubectl get pv,pvc -A
```

`/dev/vdb` must still be a blank, non-system 150 GiB disk. CNPG, registry,
NativeLink, OpenBao, and their PVCs are out of scope and must not be modified.

## Post-apply readback

Record all of the following before allowing either scale set above zero:

```sh
talosctl -n <worker-2-ip> get volumestatus u-ci-workspace -o yaml
talosctl -n <worker-2-ip> get mountstatus u-ci-workspace -o yaml
kubectl get node oya-talos-worker-2 --show-labels
kubectl get storageclass oya-ci-workspace -o yaml
kubectl -n oya-ci-workspace-storage get deploy,pods,configmap -o wide
kubectl -n arc-runners get pvc,pods -o wide
```

The volume must be ready at `/var/mnt/ci-workspace`, the provisioner identity must
be `oyatie.io/ci-workspace-local-path`, every runner/PVC must bind to worker 2, and
the existing `local-path` StorageClass/PVs must be byte-for-byte unchanged.

## Safe rollback

Do not roll back to the earlier three-runner root-filesystem placement: that state
is the #1504 incident condition. First land a GitOps change setting both scale sets'
`maxRunners: 0`,
wait for all runner Pods and generic-ephemeral PVCs to disappear, and verify the
workspace directory is empty. Then revert the ARC storage/placement Applications.
Remove the Talos `UserVolumeConfig` document only after no Pod mounts it; Talos
leaves the data on disk, so disk wiping is a separate destructive action and is
never part of this rollback. Branch protection and `oya-ci-required` remain intact;
admission stays queued until safe runner capacity or hosted fallback exists.
