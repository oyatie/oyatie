---
doc_class: Benchmark
microservice: cloud-k8s
benchmark_date: 2026-05-20
related_adrs: [ADR-0121, ADR-0316]
doc_status: published
---

# Benchmarks — oyatie cloud-k8s vs managed-K8s vs Rancher RKE2 vs OpenShift

Workloads measured: (a) cluster bootstrap from zero to first-pod-scheduled, (b) cluster API p99 under steady load, (c) pod scheduling latency p99 for 1 000-pod burst, (d) annual TCO for a paid on-prem-connected cell_topology cluster at 50 worker nodes.

Hardware (for the on-prem comparators): 12× bare-metal nodes, each 32 vCPU AMD EPYC 7543P, 256 GiB DDR4, 2× 1.92 TB NVMe (RAID-1 for boot, separate for containerd), dual 25 GbE NIC. Network: 25 GbE leaf-spine, MTU 9000. Storage: shared Ceph 3-node cluster on identical hardware.

Cloud-managed comparators measured on equivalent compute: m6i.8xlarge (AWS), n2-standard-32 (GCP), Standard_D32s_v5 (Azure), VM.Standard.E5.Flex (OCI).

## Workload (a) — bootstrap from zero to first-pod-scheduled

| Cluster shape | Time (min) | Operator-action steps |
|---|---:|---:|
| oyatie cloud-k8s demo_trial (`--profile demo-trial`) | 28 | 1 (single CLI command) |
| oyatie cloud-k8s paid dedicated-cloud (`--profile paid-dedicated-cloud`) | 42 | 1 |
| oyatie cloud-k8s paid on-prem-connected (`--profile paid-onprem-connected`) | 84 | 2 (provision + bootstrap) |
| AWS EKS via Terraform module | 22 | 1 |
| GCP GKE via Terraform module | 19 | 1 |
| Azure AKS via Terraform module | 24 | 1 |
| Rancher RKE2 (3 CP + 9 W) | 38 | 4 (rancherd bootstrap + 3 worker enroll batches) |
| OpenShift 4.16 via assisted-installer | 71 | 6 (discovery ISO, host registration, network, ACM hub bind, cluster install, post-install operators) |

oyatie demo_trial beats Rancher RKE2 by ~ 10 min largely because we don't ship Rancher's cluster-controller chart on demo_trial. We are ~ 6 min behind EKS / GKE / AKS because those are hyperscaler-managed and parallel-provisioned at a level we can't match on-prem without a co-located warm node pool (which we have, but only at paid on-prem-connected cell_topology).

OpenShift's 71 min is consistent with industry reports; the assisted-installer's discovery-then-config phase serializes more than we do.

## Workload (b) — cluster API p99 under steady load (10 RPS sustained get/list/watch for 1 hour)

| Cluster shape | p50 (ms) | p99 (ms) |
|---|---:|---:|
| oyatie cloud-k8s demo_trial | 8 | 28 |
| oyatie cloud-k8s paid dedicated-cloud | 11 | 34 |
| oyatie cloud-k8s paid on-prem-connected | 13 | 41 |
| AWS EKS (1.30) | 14 | 38 |
| GCP GKE Standard (1.31) | 9 | 31 |
| Azure AKS (1.30) | 12 | 36 |
| Rancher RKE2 (1.31) | 10 | 33 |
| OpenShift 4.16 | 15 | 48 |

paid dedicated-cloud adds Istio's xDS push overhead at the apiserver (Istio's webhook intercepts pod creates for sidecar inject). paid on-prem-connected adds Velero's periodic backup pulls + multi-cluster federation's xDS cross-talk. The deltas are well within the SLO posture in `slos/cluster-api-availability.openslo.yaml`.

## Workload (c) — pod scheduling latency p99 for 1 000-pod burst

| Cluster shape | first-pod (s) | p99 (s) | p100 (s) |
|---|---:|---:|---:|
| oyatie cloud-k8s demo_trial (no sidecar inject) | 2 | 28 | 41 |
| oyatie cloud-k8s paid dedicated-cloud (sidecar inject ON) | 4 | 44 | 67 |
| oyatie cloud-k8s paid on-prem-connected (multi-AZ spread) | 6 | 58 | 91 |
| AWS EKS + AWS Load Balancer Controller | 3 | 38 | 56 |
| GCP GKE + Cloud Run sidecar | 2 | 32 | 49 |
| Azure AKS + AGIC ingress | 3 | 40 | 62 |
| Rancher RKE2 + canal CNI | 3 | 35 | 53 |
| OpenShift 4.16 + OVN-K8s | 4 | 47 | 71 |

The paid dedicated-cloud vs demo_trial gap (44 → 28 s p99) is Istio's sidecar pull. Cold images dominate; warming via the `containerd image pull --pre-pull-list iac/sidecar-prepull.yaml` step at kubeadm join cuts the paid dedicated-cloud p99 to ~ 36 s — close to demo_trial.

## Workload (d) — annual TCO for paid on-prem-connected cell_topology at 50 worker nodes

| Cluster shape | Hardware / instance (USD) | Ops time (USD) | Licence (USD) | Total (USD) |
|---:|---:|---:|---:|---:|
| oyatie cloud-k8s paid on-prem-connected on-prem | 184 000 | 248 000 (2 SRE × 0.4 FTE) | 0 | 432 000 |
| AWS EKS | 528 000 (EC2 + EKS control plane fee) | 124 000 (1 SRE × 0.2 FTE) | 0 | 652 000 |
| GCP GKE Standard | 492 000 | 124 000 | 0 | 616 000 |
| Azure AKS | 548 000 | 124 000 | 0 | 672 000 |
| Rancher RKE2 + bare-metal | 184 000 | 372 000 (3 SRE × 0.4 FTE; Rancher-specific ops overhead) | 32 000 (Rancher subscription) | 588 000 |
| OpenShift 4.16 on bare-metal | 184 000 | 372 000 | 215 000 (Red Hat subscription per node) | 771 000 |

oyatie's edge vs hyperscaler-managed is the absence of the control-plane fee + lower per-pod compute markup. The edge vs Rancher RKE2 is the simpler operational model (fewer Rancher-specific upgrade dances). The edge vs OpenShift is the licence (Red Hat charges ~ $215 / node / year for OpenShift 4.x at scale).

Caveats:

- These numbers assume 24×7 utilisation. Bursty workloads tilt the hyperscalers favourably because they can scale to zero.
- The ops-time number is from our 2026-Q1 internal ops survey. Other organisations should re-baseline.
- The hardware number assumes 5-year amortisation. At 3-year amortisation the on-prem numbers rise to ~ 232 k.

## Reproducibility

The benchmark harness is at `benchmarks/k8sbench/`. Run with:

```sh
cargo run -p dev-cli -- benchmarks cloud-k8s \
    --workload bootstrap \
    --shape oyatie-paid-onprem-connected \
    --output ./benchmark-results.json
```

Hyperscaler comparators require valid `--cloud-credentials`. The results live at `benchmarks/results/cloud-k8s/<date>.csv` and are re-run weekly in CI to detect drift.
