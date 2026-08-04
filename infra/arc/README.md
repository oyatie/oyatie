# ARC workspace capacity lane

Issue #1504 isolates disposable ARC build work from the Talos system filesystem.
The repository declaration has four parts:

1. The worker-1/worker-2 Talos patches allocate fixed 48 GiB XFS general
   workspaces on each exact blank 150 GiB `/dev/vdb`; worker-2 also carries the
   dedicated 48 GiB PostgreSQL workspace.
2. `ci-workspace-storage.yaml` runs a separate local-path provisioner identity and
   StorageClasses rooted only at `/var/mnt/ci-workspace-general` and
   `/var/mnt/ci-workspace-live-postgres` on `oya-talos-worker-2`, with no default
   path for any unlisted node.
3. Both runner scale sets mount a 44 GiB generic ephemeral PVC at
   `/home/runner/_work`. The general set is capped at two and uses required
   hostname anti-affinity, so each runner gets a different node-local 48 GiB
   filesystem. The PostgreSQL set is capped at one and selects
   `oya.io/ci-capacity=pg`.
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

Before applying either Talos patch, an authorized operator records:

```sh
talosctl -n <worker-1-ip>,<worker-2-ip> get disks -o yaml
talosctl -n <worker-1-ip>,<worker-2-ip> get discoveredvolumes -o yaml
kubectl get node oya-talos-worker-1 oya-talos-worker-2 -o yaml
kubectl get pv,pvc -A
```

`/dev/vdb` must still be a blank, non-system 150 GiB disk. CNPG, registry,
NativeLink, OpenBao, and their PVCs are out of scope and must not be modified.

## Post-apply readback

Record all of the following before allowing either scale set above zero:

```sh
talosctl -n <worker-2-ip> get volumestatus u-ci-workspace-general -o yaml
talosctl -n <worker-2-ip> get volumestatus u-ci-workspace-live-postgres -o yaml
talosctl -n <worker-2-ip> get mountstatus u-ci-workspace-general -o yaml
talosctl -n <worker-2-ip> get mountstatus u-ci-workspace-live-postgres -o yaml
kubectl get node oya-talos-worker-2 --show-labels
kubectl get storageclass oya-ci-workspace-general oya-ci-workspace-live-postgres -o yaml
kubectl -n oya-ci-workspace-storage get deploy,pods,configmap -o wide
kubectl -n arc-runners get pvc,pods -o wide
```

All three node-local volumes must be ready at their declared `/var/mnt/ci-workspace-*` paths, the provisioner
identity must be `oyatie.io/ci-workspace-local-path`, every runner/PVC must bind only to an admitted node, and
the existing `local-path` StorageClass/PVs must be byte-for-byte unchanged.

Admission is staged `maxRunners: 0 -> 1 -> 2`: at zero prove both mounts and
provisioner mappings; at one run the cold full fallback and prove cleanup; only
then admit two simultaneous general jobs and prove they land on different
hostnames without DiskPressure or eviction. A failed stage returns to zero.

## Safe rollback

Do not roll back to the earlier three-runner root-filesystem placement: that state
is the #1504 incident condition. First land a GitOps change setting both scale sets'
`maxRunners: 0`,
wait for all runner Pods and generic-ephemeral PVCs to disappear, and verify both
workspace directories are empty. Then revert the ARC storage/placement Applications.
Remove the two Talos `UserVolumeConfig` documents only after no Pod mounts either; Talos
leaves the data on disk, so disk wiping is a separate destructive action and is
never part of this rollback. Branch protection and `oya-ci-required` remain intact;
admission stays queued until safe runner capacity or hosted fallback exists.
