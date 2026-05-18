# IP-095 — Optional GPU Acceleration for Milvus

**Phase:** PHASE-02-FOUNDRY-DATA-SUBSTRATE
**Owner:** infra (axis-foundry + capacity-planning + ops-sre-reliability)
**Authority ADRs:** ADR-0192 §"GPU acceleration", ADR-0026 AI substrate, ADR-0027 robotics-vision-speech shared GPU pool, ADR-0001 cohesion authority (cost), ADR-0145 inter-microservice communication
**Depends on:** IP-091, IP-094
**Status:** Planned (opt-in per cell)
**Phase trace:** PHASE-02 §"GPU acceleration overlay" (addendum lines 52-58, gated opt-in).

## Scope

Per-cell opt-in GPU node pool that hosts Milvus **query** and **index** nodes when CPU index-build capacity is exceeded or recall+latency targets demand GPU-accelerated indices. Uses the GPU_CAGRA index type via NVIDIA RAPIDS RAFT (CUDA 12.x).

GPU adoption is **gated** by cost — GPU nodes are 5-10x more expensive than CPU equivalents. The cell-meta capacity planner blocks `gpu.enabled=true` rollout until the cell's `gpu_budget_usd_monthly` line is ratified. Per ADR-0027 the GPU pool is shared with robotics-vision-speech sub-substrates; Foundry Milvus is one tenant among several.

## File targets

| Path | Action | Line range | Notes |
|---|---|---|---|
| `microservices/foundry/capacity-model-milvus.md` | edit | append §"GPU pool sizing" (lines ~80-130) | infra+capacity |
| `microservices/foundry/iac/helm/milvus/values-gpu-overlay.yaml` | create | 1-90 | enables queryNode.gpu.enabled + indexNode.gpu.enabled |
| `microservices/foundry/iac/kustomize/components/milvus-gpu/nvidia-device-plugin.yaml` | create | 1-60 | NVIDIA k8s device plugin DaemonSet |
| `microservices/foundry/iac/kustomize/components/milvus-gpu/gpu-node-pool.yaml.tpl` | create | 1-50 | template for per-cluster GPU NodePool (provider-specific) |
| `microservices/foundry/iac/kustomize/overlays/gpu-enabled/kustomization.yaml` | create | 1-25 | composition switch |
| `microservices/foundry/iac/kustomize/overlays/gpu-enabled/cost-budget-pin.yaml` | create | 1-40 | mandatory cost-budget pin per ADR-0001 |
| `microservices/foundry/cost-budget-gpu-overlay.md` | create | 1-180 | per-tier GPU cost forecast + breakeven analysis |
| `microservices/foundry/runbooks/milvus-gpu-rollout.md` | create | 1-160 | step-by-step gated rollout |
| `microservices/foundry/runbooks/milvus-gpu-rollback.md` | create | 1-120 | revert to CPU path |
| `crates/oya-shared-vector-store-milvus-adapter/src/index.rs` | edit | extend IndexType::GpuCagra branch (lines 80-120) | backend |
| `crates/oya-shared-vector-store-milvus-adapter/tests/integration/gpu_cagra_build.rs` | create | 1-180 | gated test (skipped on non-GPU cells) |

## Pre-conditions for opt-in

1. Cell-meta entry `cells[<cell-id>].gpu_pool.enabled = true`.
2. Cost-budget signed off by capacity-planning + finance (audit-chain `capacity.gpu.budget.approved`).
3. Per-cluster GPU node pool provisioned (cloud-provider specific — out of this IP's scope, see capacity µservice).
4. NVIDIA device plugin DaemonSet healthy on GPU nodes (`nvidia.com/gpu` resource reports > 0).
5. IP-094 adapter supports `IndexType::GpuCagra` (already in the index module).

## Deliverables

1. **Cell capacity-model entry** — declares GPU pool size + GPU SKU + per-month USD ceiling.
2. **Helm values overlay** — `values-gpu-overlay.yaml` enabling `queryNode.gpu.enabled` + `indexNode.gpu.enabled` + GPU resource requests.
3. **NVIDIA device plugin DaemonSet** — canonical version pinned.
4. **Per-collection GPU_CAGRA index option** — kernel already supports `IndexType::GpuCagra`; this IP wires the path end-to-end through the adapter.
5. **Cost-budget pin** — `cost-budget-gpu-overlay.md` with per-tier breakeven analysis. GPU nodes only justified when index-build CPU saturation > 60% for sustained 7-day window OR when GPU_CAGRA's 10x build speedup unlocks a recall+latency point unreachable on CPU.
6. **Rollout + rollback runbooks** — explicit step-by-step.
7. **Adapter integration test** — gated `gpu_cagra_build.rs` test, runs only on cells where the GPU pool is healthy.

## Acceptance criteria

- GPU-enabled cell builds an HNSW-equivalent index **10x faster** than CPU equivalent (per NVIDIA RAFT published numbers; verified by `gpu_cagra_build.rs`).
- Per-collection GPU index is selectable via the collection manifest (`index_type: GpuCagra`).
- NVIDIA device plugin reports `nvidia.com/gpu` resource > 0 on all GPU-pool nodes.
- Milvus query / index nodes scheduled exclusively on GPU pool (verified by node selector + toleration).
- Cost-budget pin is **mandatory** — the gpu-enabled overlay does not apply without a signed cost-budget audit-chain event in the last 90 days.
- GPU pool is shared with the robotics-vision-speech sub-substrate per ADR-0027 — verified by namespace-pinned ResourceQuota.
- Rollback to CPU path is non-destructive — index rebuild required, but data preserved.

## Test plan

| Test | Verifies |
|---|---|
| `test_gpu_device_plugin_healthy` | gated; verifies `nvidia.com/gpu` resource > 0 |
| `test_gpu_cagra_build_10x_speedup` | gated; CAGRA build 10x faster than CPU HNSW on 1M vector set |
| `test_gpu_cagra_recall_at_10` | gated; recall ≥ 0.95 |
| `test_gpu_node_selector_enforced` | query/index nodes scheduled on GPU nodes only |
| `test_cost_budget_pin_enforced` | overlay without signed budget pin fails apply |
| `test_rollback_to_cpu_no_data_loss` | gated; switch to CPU path; data preserved; index rebuilt |
| `test_shared_pool_robotics_quota` | namespace ResourceQuota enforced |
| `test_gpu_cagra_index_type_selectable` | per-collection manifest selects index type |

## Evidence emission

- **Audit chain (ADR-0145):** `capacity.gpu.budget.approved`, `milvus.gpu.overlay.applied`, `milvus.gpu.overlay.rolled_back` events to `oya.foundry.audit.milvus.gpu`.
- **Metrics:** `milvus_index_build_duration_seconds{index_type="GpuCagra"}` plus existing adapter metrics with `device=gpu` label.
- **Bench artifact:** `evidence/bench/milvus-gpu-cagra-<cell-id>-<commit>.json` per cell on every gpu-overlay apply.
- **Cost evidence:** monthly attribution rolled into `evidence/cost-attribution/foundry-milvus-<month>.json`.

## Rollback procedure

1. **Detection.** `MilvusGpuPoolDegraded` alert (≥ 1 GPU node unhealthy for 10min) or cost-budget breach detected by the per-cell budget watcher.
2. **Soft rollback (preferred).** Helm overlay flip: remove `values-gpu-overlay.yaml` from the cell's overlay composition; `helm upgrade foundry-milvus ...`; query/index nodes reschedule onto CPU pool; index rebuild begins automatically.
3. **Hard rollback (storage-loss recovery).** Restore from IP-096 backup if rebuild fails.
4. **Cost-overrun rollback.** If cost-budget breached mid-month, the cell-meta planner emits `capacity.gpu.budget.breach` event; auto-rollback policy disables the overlay until next budget cycle.
5. Per the runbook at `runbooks/milvus-gpu-rollback.md`, all rollbacks require 2-person approval (foundry-oncall + capacity-oncall).

## Cost-budget analysis (summary; full at `cost-budget-gpu-overlay.md`)

| Cell size | CPU baseline ($/mo) | GPU overlay ($/mo) | Breakeven trigger |
|---|---|---|---|
| Small | 1,200 | 6,500 | Index-build > 60% saturated 7d sustained |
| Medium | 3,500 | 18,000 | Above + recall+latency target unreachable on CPU |
| Large | 9,000 | 48,000 | Above + sustained > 1M vectors/hour ingest |

## Blocking deps

- IP-091, IP-094.
- Cell-meta `gpu_pool.enabled = true`.
- ADR-0027 GPU pool agreement with robotics-vision-speech.
- Cost-budget approval (audit-chain event).

## Exit criteria

GPU overlay applies cleanly in one canary cell; `test_gpu_cagra_build_10x_speedup` passes 3 consecutive runs; cost-budget pin enforces in CI; rollback runbook drilled by foundry-oncall.

## Out of scope

- GPU node pool provisioning at the cloud-provider layer (capacity µservice).
- Multi-cell GPU orchestration (post-M02 capability).
- Per-tenant GPU billing surcharge (billing µservice integration).

## Phased rollout plan

| Phase | Trigger | Action | Rollback signal |
|---|---|---|---|
| Phase 0 — Capacity proposal | Index-build CPU > 50% sustained 7d | Capacity µservice proposes GPU pool; cost-budget review begins | n/a |
| Phase 1 — Budget approval | Finance signs cost-budget pin | Audit-chain `capacity.gpu.budget.approved` emitted | Budget rejected → cancel proposal |
| Phase 2 — Pool provisioning | Budget approved | Capacity µservice provisions GPU node pool (cloud-provider specific) | Provisioning fails → cancel rollout |
| Phase 3 — Device plugin | Pool nodes Ready | NVIDIA device plugin DaemonSet applied; `nvidia.com/gpu` reports > 0 | Plugin unhealthy → revert step |
| Phase 4 — Overlay applied | Plugin healthy | `gpu-enabled` overlay applied via Kustomize; query/index pods reschedule onto GPU nodes | Pods fail to schedule → soft rollback (overlay removed) |
| Phase 5 — Burn-in | Overlay applied | 7-day burn-in; recall + latency tracked against pre-GPU baseline | Recall regression → hard rollback + investigation |
| Phase 6 — Accepted | Burn-in green | IP marked Accepted; per-collection GPU_CAGRA opt-in available | n/a |

## Security posture

- GPU pool is namespace-pinned; foundry-milvus query/index nodes have nodeSelector + toleration; no other workload can land on GPU nodes unless namespace-allow-listed.
- NVIDIA device plugin runs with `privileged: false` and the minimal `hostPath` mounts required by the upstream chart.
- GPU memory is not shared across tenants — every query is single-tenant scoped at the proxy layer.

## Observability mapping

| Signal | Metric | Alert |
|---|---|---|
| GPU memory used | `nvidia_gpu_memory_used_bytes` (DCGM exporter) | `GpuMemoryPressure` (> 90% sustained 15min) |
| GPU SM utilization | `nvidia_gpu_sm_utilization_ratio` | — (informational) |
| GPU temp | `nvidia_gpu_temperature_celsius` | `GpuOverheat` (> 85°C sustained 10min) |
| CAGRA build duration | `milvus_index_build_duration_seconds{index_type="GpuCagra"}` | `CagraBuildSlow` (> 10x CPU baseline) |
| Per-month GPU cost | `foundry_milvus_gpu_cost_usd_monthly` (computed by capacity µservice) | `GpuCostOverrun` (> +20% over budget) |
| Device plugin health | `kube_node_status_capacity{resource="nvidia.com/gpu"}` | `GpuDevicePluginUnhealthy` (any node reports 0 while overlay enabled) |

## Per-tier GPU budget (illustrative; canonical pin at `cost-budget-gpu-overlay.md`)

| Tier | GPUs per cell | SKU | Monthly $ ceiling | Auto-disable trigger |
|---|---|---|---|---|
| Small (canary only) | 1 | A10G | 800 | Budget breach |
| Medium | 4 | A100 | 18,000 | +20% over budget projection |
| Large | 8 | H100 | 56,000 | +15% over budget projection |

Auto-disable: when the budget watcher detects a projected month-end overrun > threshold, it emits `capacity.gpu.budget.breach` and the overlay flips to disabled within 1h.

## References

- ADR-0192 §"GPU acceleration".
- ADR-0026 — AI substrate.
- ADR-0027 — robotics-vision-speech shared GPU pool.
- ADR-0001 — cohesion authority (cost attribution).
- ADR-0145 — communication reform.
- Capacity model: `microservices/foundry/capacity-model-milvus.md` §"GPU pool sizing".
- Runbooks: `microservices/foundry/runbooks/milvus-gpu-rollout.md`, `milvus-gpu-rollback.md`.
