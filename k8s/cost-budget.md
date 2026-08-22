---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: cloud-k8s
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-cloud + ops-sre-reliability
deciders: ops-finops, axis-cloud, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0121, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/cloud-k8s/capacity-model.md
  - microservices/cloud-k8s/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (cloud-k8s µservice)

## Purpose

Track the cloud-k8s µservice's monthly cloud cost across compute + storage + network + KMS + load balancer + IP allocation, per cluster component, per pack region; surface budget breach via the `check-cost-budget` LEAN lane.

## Cost Categories

| Category | What | OCI pricing reference |
|---|---|---|
| Compute (VM.Standard.E4 Flex / OKE nodes) | control-plane + worker nodes; component pods | `oracle.com/cloud/compute/pricing/` |
| Block storage (PV) | etcd volume; CSI block-volume PVCs; container images cache | `oracle.com/cloud/storage/block-volume/pricing/` |
| Object storage | etcd snapshots; CSI object PVCs; Harbor registry mirror | `oracle.com/cloud/storage/object-storage/pricing/` |
| File storage | CSI file PVCs (NFS-compatible) | `oracle.com/cloud/storage/file-storage/pricing/` |
| Network egress | Cross-cluster Istio mesh; status-page; auditor reads | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-pack KMS keyring for etcd envelope + Cosign verification | `oracle.com/security/key-management/pricing/` |
| Load balancer | Per-pack Envoy gateway + public LB + bare-metal LB | `oracle.com/cloud/networking/load-balancing/pricing/` |
| IP allocation | Public IP per ingress gateway; private IP pool per cluster | per OCI VCN pricing |

## Per-Component Monthly Cost (XS tier, single pack-kr region, M01 launch)

Per `capacity-model.md` §"Worked example".

| Component | Replicas × instance-type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| Control-plane node | 1 × VM.Standard.E4 Flex 8 OCPU / 64 GB | $580 | $40 (16 GB etcd PV + 100 GB root) | $620 |
| Worker nodes | 17 × VM.Standard.E4 Flex 4 OCPU / 32 GB | $4930 | $680 (40 GB root each) | $5610 |
| kube-apiserver | (on control-plane node) | — | — | — |
| etcd | (on control-plane node) | — | $30 (snapshot object storage 14d hot + 90d cold) | $30 |
| Cilium operator | 2 × VM.Standard.E4 1 OCPU / 4 GB | $72 | — | $72 |
| Cilium agent (DaemonSet) | 17 × 0.1 OCPU + 256 MB each | (included in node) | — | — |
| Istio istiod | 3 × VM.Standard.E4 1 OCPU / 4 GB | $108 | — | $108 |
| Istio Envoy ingress | 2 × VM.Standard.E4 2 OCPU / 8 GB | $144 | — | $144 |
| Envoy sidecars | (50m CPU + 128 MB per pod; ~1440 pods) | (included in worker nodes) | — | — |
| Kyverno admission | 3 × VM.Standard.E4 1 OCPU / 4 GB | $108 | — | $108 |
| Kyverno background | 2 × VM.Standard.E4 1 OCPU / 2 GB | $54 | — | $54 |
| CSI block-volume controller | 2 × VM.Standard.E4 1 OCPU / 2 GB | $54 | — | $54 |
| CSI object controller | 2 × VM.Standard.E4 1 OCPU / 2 GB | $54 | — | $54 |
| CSI file controller | 2 × VM.Standard.E4 1 OCPU / 2 GB | $54 | — | $54 |
| CSI node plugins (DaemonSet) | 17 × 0.1 OCPU each | (included in node) | — | — |
| kubernetes-api-proxy | 3 × VM.Standard.E4 1 OCPU / 2 GB | $108 | — | $108 |
| cluster-bootstrap worker | 2 × VM.Standard.E4 1 OCPU / 2 GB | $54 | — | $54 |
| Public LB (Envoy gateway frontend) | 1 LB-100 instance | $20 | — | $20 |
| Internal LB (kube-apiserver service) | 1 LB-10 instance | $5 | — | $5 |
| KMS keyring (per pack; etcd envelope key + Cosign verification key) | — | $5 | — | $5 |
| Audit log forwarding egress | — | $30 (to observability µservice's Mimir) | — | $30 |
| etcd snapshot retention object storage | — | — | $130 (hot 14d + 90d cold) | $130 |
| Worker node ephemeral storage (container images cache, etc.) | — | — | $50 | $50 |
| **XS tier total per pack region** | | **~$6300** | **~$830** | **~$7130 / month** |

Verify-at-deploy: OCI pricing changes; reconfirm against `oracle.com/cloud/pricing/`. Buffer 15% for OCI rate increases + 20% for actual-vs-forecast.

## Per-Scale-Tier Forecast

| Scale tier | N_tenants | Monthly cost per pack region | Notes |
|---|---|---|---|
| XS (M01 launch; 20 tenants) | 20 | ~$7100 | active: pack-kr |
| S (~100 tenants) | 100 | ~$30k | active: 3 packs |
| M (~1000 tenants) | 1000 | ~$200k | typically 5 active packs |
| L (~10000 tenants) | 10000 | ~$1.8M | all 11 packs; multi-cluster federation |

## Per-Pack Multipliers

- **DR pair packs** (pack-eu, pack-us, pack-au, pack-in, pack-br, pack-ae, pack-ksa): 1.0× primary + 0.6× warm-standby.
- **HIPAA pack** (pack-us-healthcare): 1.4× base (extended retention 6y; lower pods/node for isolation; dedicated HIPAA-eligible region).
- **KR-FSS-regulated tenants in pack-kr**: 1.2× base (audit log retention 5y; KMS-in-KR).
- **Single-region packs** (pack-kr, pack-jp, pack-sg): 1.0× base.

## Budget + Alert Thresholds

| Metric | Threshold | Action |
|---|---|---|
| Monthly cost per pack region | within 90% forecast | normal |
| 90% < cost < 110% | yellow | FinOps + ops-sre-reliability review |
| 110% < cost < 130% | orange | FinOps + leadership; review autoscale + capacity-model |
| cost > 130% | red; budget breach incident | engage ops-finops + axis-cloud; consider per-tenant rate-limit tightening |
| Per-tenant cost projection (highest spender) | within 5× median tenant | normal |
| Per-tenant cost > 10× median | yellow; engage tenant on resource discipline | tenant dashboard surfaces self-overage |

## FinOps SLI

| SLI | Target | Burn-rate alert |
|---|---|---|
| Monthly cost / N_tenants (unit-economic) | within 5% forecast | 6× burn over 6h |
| Worker node utilization | ≥ 50% target (CPU); ≥ 60% (memory) | informational (too low = waste; too high = noisy-neighbor) |
| Spot-vs-on-demand ratio | ≥ 30% spot for stateless workloads where workload allows | informational |
| Per-pack KMS API call rate | bounded | informational |

## Cost-Optimisation Levers

| Lever | Estimated saving | Trade-off |
|---|---|---|
| OCI committed-use discounts (1y / 3y) | 20–40% compute | Vendor lock-in window |
| Spot fleet for stateless components (api-proxy, istiod canary) | 30–50% compute | Spot eviction recovery via HA |
| Reduce worker node size (pack more pods per smaller node) | 5–15% | Tighter eviction storms |
| Aggressive etcd snapshot compaction | 5–10% etcd storage | More compactor CPU |
| Archive etcd snapshots earlier (14d → 7d hot threshold) | 5% storage | Faster cold-tier reads only |
| Per-tenant resource quota enforcement | 5–20% | Tenant disruption if too aggressive |
| Pre-pull common images on node-join | 1–3% cold-start latency reduction; no $ change | Bandwidth |
| Cluster-autoscaler aggressive scale-down (M02-onward) | 10–20% compute | Slower scale-up when needed |

## Verification

- `cargo run -p dev-cli -- gate validate cost-budget --microservice cloud-k8s` — exit 0; current spend within 110%.
- Monthly FinOps review: actual vs forecast; lever decisions logged.
- Quarterly: capacity-model + cost-budget refresh.

## References

- `microservices/cloud-k8s/capacity-model.md`.
- `microservices/cloud-k8s/multi-region.md`.
- `microservices/cloud-k8s/policy/data-residency.md` (per-pack retention multipliers).
- OCI pricing — `oracle.com/cloud/pricing/`.
- Kubernetes node sizing — `kubernetes.io/docs/concepts/configuration/manage-resources-containers/`.
- FinOps Foundation framework — `finops.org`.
