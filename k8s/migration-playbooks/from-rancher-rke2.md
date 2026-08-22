---
doc_class: MigrationPlaybook
microservice: cloud-k8s
vendor: Rancher RKE2 / SUSE
date: 2026-05-20
doc_status: published
---

# Migration playbook — Rancher RKE2 → oyatie cloud-k8s

Audience: an oyatie tenant or internal cell-owner moving an existing Rancher RKE2 cluster to oyatie's vanilla kubeadm substrate without taking workload downtime > 5 minutes per workload.

## Why this migration is non-trivial

Rancher RKE2 ships:

- A bundled containerd (1.7.x) — oyatie pins 2.3.0 LTS.
- A bundled Canal CNI by default — oyatie uses Cilium 1.18.
- Rancher's cluster-controller in the `cattle-system` namespace — has no oyatie equivalent (we use GitOps via Flux + cluster-api directly).
- RKE2's bundled Helm chart packager — oyatie uses Flux's Kustomize-first reconciler.
- Rancher Fleet for multi-cluster GitOps — oyatie uses Flux v2.

Naive `kubectl apply` of RKE2 workloads against oyatie produces 70 % success out-of-box; the 30 % failure modes are: CNI annotation deltas, Helm chart references that resolve through `cattle-system` proxies, PV references through Longhorn (RKE2-bundled) that need re-binding to Ceph RBD.

## Step 1 — Inventory the source cluster (≤ 60 min)

```sh
oya cloud-k8s migrate inventory \
    --source-kubeconfig ~/.kube/rke2-prod.kubeconfig \
    --out inventory/rke2-prod.yaml
```

The inventory enumerates: namespaces, workloads (Deployment + StatefulSet + DaemonSet), Services, Ingresses, PVCs, NetworkPolicies, ServiceAccounts + RBAC bindings, Helm releases, Rancher-specific CRDs (ClusterRoleTemplateBinding, ProjectAlertRule, etc.).

Manually classify each Rancher CRD:

- `cattle.io/*` Rancher-management CRDs: drop. oyatie GitOps replaces these.
- `helm.cattle.io/*` (HelmChart, HelmChartConfig): convert to Flux `HelmRelease`.
- `k3s.cattle.io/*`: not in RKE2 (k3s-only); should not appear.
- `longhorn.io/*`: requires PV migration step (Step 4); flag with `human-review`.

## Step 2 — Stand up the target oyatie cluster (≤ 90 min)

Per `tutorials/bootstrap-demo-trial-cluster.md` (or paid dedicated-cloud / paid on-prem-connected per the target tier).

Provision target hardware sized at least equal to source — RKE2's bundled overhead is ~ 8 % of cluster CPU; oyatie's Istio + Cilium overhead is ~ 11 % at paid dedicated-cloud. Plan +3 % capacity headroom.

## Step 3 — Workload conversion (≤ 2 days for typical 200-Deployment cluster)

The converter:

```sh
oya cloud-k8s migrate convert \
    --inventory inventory/rke2-prod.yaml \
    --target-cell-topology dedicated-cloud \
    --out manifests/oyatie-prod/
```

The converter handles automatically:

- Cilium NetworkPolicy from Canal NetworkPolicy: 1:1 mostly; the exception is Canal-specific `egress.cidrBlock.except` semantics — Cilium uses `except[]` not nested-block; the converter rewrites.
- HelmChart CRDs → Flux HelmReleases: maps the chart repo, version, values block; the chart repo URL is preserved (if the chart is hosted on a Rancher-proxied URL, you'll need to rehost or use Flux's `HelmRepository` pointing at the upstream).
- ServiceAccount RBAC: preserved as-is; Rancher's `ProjectRoleTemplateBinding` is dropped (replace with explicit RBAC in the GitOps repo).
- Annotations `field.cattle.io/*`: dropped (decorative metadata; no functional impact).

The converter flags for human review:

- `cattle.io/creator-principal-name` annotations on Deployments: usually decorative but sometimes consumed by tenant tooling.
- Longhorn-backed PVCs (see Step 4).
- Pod-Security-Policy → Pod-Security-Standards conversion (RKE2's PSP enforcement is implicit; oyatie's PSS is explicit per namespace; converter sets `pod-security.kubernetes.io/enforce=restricted` by default, requires tenant sign-off if the tenant runs privileged sidecars).

## Step 4 — PV migration (Longhorn → Ceph RBD)

The hardest step. Longhorn snapshots are not Ceph snapshots — there is no in-place rename.

For each PVC:

1. Take a Longhorn snapshot via `kubectl annotate pvc <name> longhorn.io/snapshot-now=true`.
2. Export the snapshot via Longhorn's backup-target (S3-compatible). The export takes ~ 1 min per GiB of used data.
3. Provision a Ceph RBD PVC of equivalent size in the target oyatie cluster.
4. Restore the data via a one-shot Job that mounts both the new RBD PVC and an S3 client; copies bytes; verifies SHA-256.
5. Cut workload over to the new PVC; verify health; delete old PVC.

For workloads that cannot tolerate the cutover gap (database PVs typically can; cache PVs typically need a warm-start period), use the `oya cloud-k8s migrate pvc-dual-write` mode — the workload is restarted twice, writing to both PVs until the lag is < 5 s, then a coordinated cutover.

Field-level deltas:

| Longhorn field | Ceph RBD field | Notes |
|---|---|---|
| `volume.longhorn.io/numberOfReplicas: 3` | (n/a; Ceph CRUSH rule encodes replication) | Ceph pool's `size 3` replaces. |
| `volume.longhorn.io/staleReplicaTimeout: 30` | (n/a) | Ceph's RBD lock recovery is automatic. |
| Longhorn snapshot-on-schedule label | `csi-snapshot-class: ceph-rbd-snap` + `VolumeSnapshotSchedule` CRD (snapshot-controller-shipped) | Different CRD, equivalent functionality. |
| Longhorn `BackingImage` | Ceph RBD `--image-feature` clone-from-snapshot | Different shape; rehydrate from snapshot manually. |

## Step 5 — Ingress migration (Traefik default in RKE2 → Envoy via Istio Gateway)

The converter maps Traefik IngressRoute → Istio Gateway + VirtualService. The deltas:

- Traefik middlewares (`Middleware` CRD) → Envoy filters + Istio EnvoyFilter (the converter emits draft EnvoyFilters but flags for human review — EnvoyFilter is the wire format, not high-level).
- Traefik's `Resilience.RateLimit` → Istio's `EnvoyFilter` with `local_rate_limit` filter (different config shape; tested but always read the generated config before applying).
- Traefik's `auth.basicAuth` and `auth.forwardAuth` → Istio AuthorizationPolicy + RequestAuthentication. The forwardAuth pattern usually needs an extauth setup change.

## Step 6 — Shadow cutover (≤ 7 days)

The target oyatie cluster shadow-traffics the source RKE2 cluster:

- DNS round-robin 5 % traffic to oyatie at hour 0.
- Watch `cloud-k8s-shadow-delta` Grafana board — should show < 0.1 % per-endpoint error-rate delta vs RKE2.
- Scale shadow to 25 % at hour 24, 50 % at hour 72, 100 % at hour 168 if all deltas hold.
- Decommission RKE2 cluster only after 7 days of 100 % oyatie traffic with the Rancher Fleet `clusterregistration` deleted and the RKE2 control-plane shutdown evidence captured.

## Step 7 — Sunset evidence

```sh
oya cloud-k8s migrate sunset-evidence \
    --source-cluster rke2-prod \
    --target-cluster oyatie-prod \
    --out evidence/migrations/rke2-prod-to-oyatie-prod.json
```

The evidence file enumerates: inventory diff, conversion log, PV migration receipts, ingress cutover timeline, RKE2 cluster final-state ledger. Required by the `governance-migration-evidence` lane.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Longhorn snapshot export takes longer than expected | Medium | Test on a representative PVC first; budget 4× the calculated time. |
| Helm chart references resolve through Rancher proxy | Medium | Rehost charts or point Flux `HelmRepository` at upstream. |
| Traefik middleware semantics don't 1:1 to Envoy | High | Manual filter review per ingress; do not ship without a test. |
| Pod-Security-Standards stricter than implicit RKE2 PSP | Medium | Tenant sign-off before tightening; phased rollout with `enforce=warn` first. |
| Rancher Fleet GitOps drift during the migration | Low | Freeze Fleet syncs during the migration window. |
