---
doc_class: Tutorial
microservice: cloud-k8s
persona: sre
date: 2026-05-20
doc_status: published
---

# Tutorial — Bootstrap a demo_trial tenant_class cluster end-to-end

You will: provision 4 VMs, run the bootstrap, verify the cluster, schedule a workload, and tear it down cleanly. Total time ≤ 90 minutes.

## Pre-requisites

- `dev-cli` ≥ 1.42.0.
- The drill-env harness running (`oya cloud-k8s drill-env status` shows green) OR access to 4 bare-metal / VM nodes with: 8 vCPU, 32 GiB RAM, 200 GiB NVMe, dual NIC, reachable on a flat L2 segment, hostnames resolving via DNS or `/etc/hosts`.
- An SSH keypair where the public key is in `~/.ssh/oyatie_drill.pub` and authorized on all 4 nodes for the `oyatie-bootstrap` user with `NOPASSWD` sudo.

## Step 1 — Pre-flight checks (≤ 10 min)

Run the prerequisite linter:

```sh
cargo run -p dev-cli -- cloud-k8s preflight \
    --vms cp-1.drill,cp-2.drill,cp-3.drill,w-1.drill \
    --ssh-key ~/.ssh/oyatie_drill \
    --ssh-user oyatie-bootstrap
```

The preflight checks 38 invariants from `iac/prerequisite-checklist.yaml`. The most-failed ones:

- Kernel version ≥ 6.6 (we use 6.6 LTS for cgroup-v2 + io_uring stability).
- `br_netfilter` + `overlay` kernel modules loaded.
- swap disabled (kubeadm rejects nodes with active swap).
- Time-sync via chrony (clock-skew > 100ms breaks etcd quorum-elect).
- TCP/IP forwarding enabled (`net.ipv4.ip_forward=1`, `net.ipv6.conf.all.forwarding=1`).

If preflight fails, fix in-place — the cluster cannot survive failures here.

## Step 2 — Bootstrap (≤ 30 min)

```sh
cargo run -p dev-cli -- cloud-k8s bootstrap \
    --profile demo-trial \
    --site drill-syd-1 \
    --control-plane-vms cp-1.drill,cp-2.drill,cp-3.drill \
    --worker-vms w-1.drill \
    --pod-cidr 10.244.0.0/16 \
    --service-cidr 10.96.0.0/12 \
    --kubeconfig-out ./drill-syd-1.kubeconfig
```

Watch the bootstrap output. The phases in order (and approximate elapsed wall-clock):

| Phase | Elapsed | What it does |
|---|---|---|
| Containerd install | 0:00–0:04 | apt install containerd; configure CRI cgroup driver to systemd. |
| Kubeadm init on cp-1 | 0:04–0:10 | First control-plane node; etcd starts on this node; certificates generated. |
| Cilium CNI apply | 0:10–0:13 | Cilium 1.18 DaemonSet rolled; vxlan tunnels established. |
| Kubeadm join cp-2 | 0:13–0:18 | Second control-plane joins; etcd member-add; certificate distribution. |
| Kubeadm join cp-3 | 0:18–0:23 | Third control-plane joins; etcd quorum at 3. |
| CoreDNS rollout | 0:23–0:25 | CoreDNS Deployment with 2 replicas applied. |
| Kubeadm join w-1 | 0:25–0:29 | Worker joins; kubelet enrolls; CNI plumbs on the worker. |

If any phase exceeds the elapsed budget by >50%, abort with Ctrl-C and read `runbooks/bootstrap-stuck.md`.

## Step 3 — Verify (≤ 5 min)

```sh
export KUBECONFIG=./drill-syd-1.kubeconfig
kubectl get nodes -o wide
```

Expected output: 4 nodes, all `Ready`. Master-roled: cp-1, cp-2, cp-3. Worker-roled: w-1.

```sh
kubectl get pods -A | grep -v Running
```

Expected: zero lines except the header. If any pod is not Running, read the `Status` column:

- `Init:0/1` for > 60 s → pod's init container is stuck. Check `kubectl describe pod`.
- `CrashLoopBackOff` → application-level crash. Probably image pull failed (check `kubectl describe pod` events).
- `Pending` → scheduler couldn't place. See FAQ §"A tenant says their pod isn't scheduling".

Run the conformance probe:

```sh
cargo run -p dev-cli -- cloud-k8s conformance --kubeconfig ./drill-syd-1.kubeconfig --profile demo-trial
```

Should output `PASS` for: kubeadm-cluster-shape, cilium-cni-installed, etcd-quorum-healthy, coredns-rolled, no-pending-pods.

## Step 4 — Schedule a workload (≤ 5 min)

Apply a sample tenant namespace + a stateless workload:

```sh
kubectl create namespace tenant-drill-tutorial
kubectl label namespace tenant-drill-tutorial \
    oyatie.io/tenant-id=drill-tutorial \
    oyatie.io/pack-id=us-default

kubectl -n tenant-drill-tutorial create deployment hello \
    --image=registry.oyatie.local/oss/hello-app:1.0 \
    --replicas=3

kubectl -n tenant-drill-tutorial expose deployment hello \
    --port 80 --target-port 8080
```

Watch the Pods schedule:

```sh
watch kubectl -n tenant-drill-tutorial get pods -o wide
```

Within ~ 30 s all 3 pods should be Running on `w-1.drill`. Test the service:

```sh
kubectl -n tenant-drill-tutorial port-forward svc/hello 8080:80 &
curl http://localhost:8080/
# Expected: "Hello from oyatie drill cluster, pod=hello-xxxxx"
```

## Step 5 — Audit-chain verification (≤ 5 min)

Every cluster mutation emitted an audit event. Query the chain:

```sh
oya audit query --cluster drill-syd-1 --since 1h --event-class cluster_*
```

Expected events, in order:

- `cluster_bootstrap_started` (1)
- `node_joined` (4 — one per node)
- `cilium_cni_applied` (1)
- `coredns_deployed` (1)
- `cluster_bootstrap_completed` (1)
- `namespace_created` (1 — your `tenant-drill-tutorial`)
- `deployment_applied` (1 — your `hello` deployment)
- `service_applied` (1 — your `hello` service)

All events signed Ed25519 against the `oyatie.cloud-k8s.runtime` key. Verify:

```sh
oya audit verify-chain --cluster drill-syd-1 --since 1h
```

Output: `chain verified, 10 events, no signature gaps`.

## Step 6 — Tear down (≤ 5 min)

```sh
cargo run -p dev-cli -- cloud-k8s teardown --cluster drill-syd-1 --confirm-i-mean-it
```

The `--confirm-i-mean-it` flag is intentional friction; teardown is destructive. The command writes `cluster_teardown_executed` to the audit-chain, drains all nodes, removes etcd state, and resets the VMs.

If you want to keep the cluster for further tutorials, skip Step 6 — it'll cost roughly $4/h of drill-env compute as long as the VMs are running.

## What you've learned

- The bootstrap phase sequence and elapsed budget per phase.
- The conformance + audit-chain verification pattern.
- The tenant namespace labelling conventions (`tenant-id`, `pack-id`).
- The teardown discipline.

Next tutorial: `tutorials/promote-to-paid-dedicated-cloud-istio.md` — adds Istio service mesh to this demo_trial cluster without re-bootstrapping.
