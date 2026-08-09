---
doc_class: Onboarding
microservice: cloud-k8s
persona: sre + cluster-operator
related_adrs: [ADR-0121, ADR-0131, ADR-0316]
date: 2026-05-20
doc_status: published
---

# SRE onboarding — first 5 working days

Audience: a new oyatie SRE assigned to the `cloud-k8s` substrate rotation. By Day-5 they will have bootstrapped a demo_trial cluster, joined a node, applied a network policy, recovered an etcd quorum-loss in the drill environment, and shadowed one on-call escalation.

## Day 1 — Tour the substrate

1. Read `PRD.md` + `ARCHITECTURE.md` §principals + §cedar-gates (≤ 60 min).
2. Open the Grafana dashboard `cloud-k8s-overview`. Identify the four key signals — `apiserver_request_duration_seconds`, `kubelet_running_pods`, `etcd_disk_wal_fsync_duration_seconds`, `istiod_xds_push_time_seconds`.
3. Walk the runbook index `runbooks/README.md`. The on-call runbooks are: `etcd-quorum-recovery.md`, `control-plane-restore.md`, `csi-rebuild.md`, `envoy-sni-debug.md`, `ingress-ddos-throttle.md`, `istio-mtls-rotation.md`, `node-drain-cascade.md`, `pod-eviction-storm.md`.
4. Sit in on Wed's substrate-on-call handoff. Watch how the outgoing rotation reads the past-week incident ledger and hands the pager.

Acceptance: you can sketch the substrate's request path (kubectl → apiserver → etcd → CNI / CSI / kubelet → containerd) on a whiteboard from memory.

## Day 2 — demo_trial cluster bootstrap

Provision 4 VMs in the drill harness (`oya cloud-k8s drill-env provision --vms 4`). Wait until the harness reports `ready`.

Run:

```sh
cargo run -p oya-dev-cli -- cloud-k8s bootstrap \
    --profile demo-trial \
    --site drill-syd-1 \
    --control-plane-vms drill-cp-1,drill-cp-2,drill-cp-3 \
    --worker-vms drill-w-1
```

Expected runtime: ≤ 30 min. Watch the logs — you should see, in order: containerd installed, kubeadm init on cp-1, kubeadm join cp-2 + cp-3, kube-proxy DaemonSet applied, Cilium CNI applied, CoreDNS rolled, kubeadm join w-1.

After bootstrap completes:

```sh
kubectl --kubeconfig drill-syd-1.kubeconfig get nodes -o wide
```

Should show 3 control-plane + 1 worker, all `Ready`.

Acceptance: cluster is up, you can describe the role of each kubeadm phase from the logs.

## Day 3 — Node join + drain

Join 2 more workers:

```sh
cargo run -p oya-dev-cli -- cloud-k8s node join \
    --cluster drill-syd-1 \
    --vms drill-w-2,drill-w-3
```

Watch p99 join time on the Grafana panel `cloud-k8s-node-readiness`. Should be ≤ 5 min per node.

Now drain `drill-w-1` for simulated hardware replacement:

```sh
kubectl --kubeconfig drill-syd-1.kubeconfig cordon drill-w-1
kubectl --kubeconfig drill-syd-1.kubeconfig drain drill-w-1 \
    --ignore-daemonsets --delete-emptydir-data --timeout=300s
```

Observe pods migrating to `drill-w-2` + `drill-w-3`. The Cedar fragment at `policy/tenant-scope.cedar` enforces that the drain action emits an `audit-chain.node_drain_initiated` event — verify with `oya audit query --event node_drain_initiated --cluster drill-syd-1`.

Acceptance: drain completes within timeout; audit event lands; pods on the drained node are zero.

## Day 4 — Network policy + Cilium

Apply a tenant-isolation NetworkPolicy. Read `policy/cluster-isolation.md` first — it explains the per-tenant namespace + per-namespace default-deny pattern.

```yaml
# tenant-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: deny-all-ingress
  namespace: tenant-drill-acme
spec:
  podSelector: {}
  policyTypes: [Ingress]
```

```sh
kubectl --kubeconfig drill-syd-1.kubeconfig apply -f tenant-policy.yaml
```

Verify enforcement with the canary path:

```sh
oya cloud-k8s policy verify \
    --cluster drill-syd-1 \
    --source-namespace tenant-drill-foo \
    --dest-namespace tenant-drill-acme \
    --dest-port 80
```

Expected: `DENY` with policy attribution `deny-all-ingress`. The verify command sources from the `oya-governance-cluster-isolation` lane's canonical test matrix.

Acceptance: cross-tenant traffic denied; same-tenant traffic permitted; both audit-chain emitted.

## Day 5 — etcd quorum-loss drill

Read `runbooks/etcd-quorum-recovery.md` end-to-end before starting. The drill kills 2 of 3 etcd members to force a quorum loss; you recover from snapshot.

```sh
oya cloud-k8s drill etcd-quorum-loss --cluster drill-syd-1 --killed-members 2
```

Cluster goes read-only within ~ 5 s (apiserver `5xx`s mutating requests; reads from local cache succeed for the lease duration).

Walk the runbook step-by-step. The recovery path is:

1. Identify the surviving etcd member from the Grafana panel `etcd-member-health`.
2. Snapshot the surviving member (`etcdctl snapshot save`).
3. Stop the surviving etcd container.
4. Restore from snapshot to a new single-member etcd quorum.
5. Re-add the two replacement members via `etcdctl member add`.
6. Verify apiserver writes resume.

Target end-to-end recovery: ≤ 20 min for the drill (production target ≤ 60 min per `slos/control-plane-availability.openslo.yaml`).

Acceptance: cluster is back to writable, you can explain why we restore from the surviving member's snapshot rather than from the periodic backup (the periodic backup is ≥ 4 h stale; the surviving-member snapshot is ≤ 0 s stale).

## What you've learned

- The demo_trial bootstrap profile end-to-end.
- The Cedar-bound audit emission on cluster-mutating operations.
- The tenant-isolation NetworkPolicy pattern.
- The etcd quorum-loss recovery drill (the single most-likely page on this rotation).

Next week: paid tenant_class promotion drill (Istio control-plane upgrade), paid on-prem-connected-tier promotion drill (multi-cluster federation join), and your first production shadow.
