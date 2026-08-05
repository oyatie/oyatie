# CI-heavy Talos + ARC capacity profile (host ≥ 96–128 GiB RAM)

**Host observed (2026-08-05):** 128 GiB RAM, 18 logical CPUs, multi-Ti free disk.  
**Live QEMU cluster (`oya-talos`) was undersized relative to that host:**

| Node | Live vCPU | Live RAM | Live data disk | Notes |
|------|-----------|----------|----------------|-------|
| controlplane-1 | 3 | ~8 GiB | 50 GiB system | lightly loaded (~18% CPU requests) |
| worker-1 | **5** | **~30 GiB** | 50 + 150 GiB | general runners |
| worker-2 | **5** | **~30 GiB** | 50 + 150 GiB | general + live-postgres; **~95% CPU requests** under load |
| **Total** | **13** | **~68 GiB** | | host spare ≈ 5 CPU + 60 GiB |

**Bottleneck evidence:** concurrency is limited by (1) ARC `maxRunners`, (2) workspace FS size, and (3) **worker CPU** — not host RAM. Raising only `maxRunners` without more vCPU starves cargo/buck2 jobs that already request 2 CPU each.

---

## Target live topology (merge-authority laptop cell)

| Node | vCPU | RAM | System disk | Data disk (`vdb`) | Role |
|------|------|-----|-------------|-------------------|------|
| controlplane-1 | **4** | **12 GiB** | 50 GiB | — | API/etcd (+ headroom) |
| worker-1 | **8** | **48 GiB** | 80 GiB | **150–200 GiB** | general CI; UserVolume general **120 GiB** |
| worker-2 | **8** | **48 GiB** | 80 GiB | **150–200 GiB** | general CI + live-postgres cell (48 GiB) |

**Host budget (guest):** 4+8+8 = **20 vCPU** (mild overcommit on 18 physical is acceptable for HVF; dial workers to 7 if host feels contended) and 12+48+48 = **108 GiB** guest RAM on a 128 GiB host (≈20 GiB for macOS + other processes).

### CPU math (why 8 vCPU workers)

| Concurrent load | CPU requests | Fits on 5 vCPU live? | Fits on 8 vCPU target? |
|-----------------|--------------|----------------------|------------------------|
| 2 general @ 2 CPU | 4 | tight (+system) | yes |
| 2 general + 1 live-postgres @ 2 | 6 | **no** (worker-2 already 95%) | yes with headroom |
| 2 general @ 2 on one node | 4 | marginal | yes |
| 2 general @ 3 (optional raise) | 6 | no | yes |

Runner template today: `requests.cpu: "2"`, `memory: "4Gi"` / `limits.memory: "8Gi"`. Keep 2 CPU request until workers are reprovisioned; optional follow-up is 3 CPU / 6 Gi request after 8 vCPU is live.

### RAM math (why 48 GiB workers)

| Load | Memory limits | On 30 GiB live | On 48 GiB target |
|------|---------------|----------------|------------------|
| 2 general @ 8 Gi limit | 16 Gi | fits, thin | comfortable |
| + live-postgres + system + nativelink co-tenancy | higher | risk OOM / throttle | intended headroom |

---

## ARC general set (git declaration)

| Knob | Value | Rationale |
|------|-------|-----------|
| `maxRunners` | **4** | 2 workers × ≤2 runners (topology spread) |
| workspace PVC | 44 GiB | per claim |
| general user volume | **120 GiB** | 2×44 + 4 Gi reserve ≤ 120 per node |
| spread | preferred anti-affinity + **topologySpread DoNotSchedule maxSkew=1** | hard cap ~2 general runners per hostname |
| live-postgres | maxRunners=1 | unchanged disk-heavy cell |

---

## Apply path (ordered)

### A. Immediate concurrency (no VM recreate)

Works on **current** 5 vCPU / 30 GiB workers if ≤2 general pods/node (topology spread). Expect **CPU contention** until step B.

```bash
# From repo root, committed values only — never a scratchpad copy.
helm upgrade oya-arm64 oci://ghcr.io/actions/actions-runner-controller-charts/gha-runner-scale-set \
  --namespace arc-runners --version 0.14.2 \
  -f infra/arc/runner-scale-set-arm64-values.yaml
```

### B. Grow general UserVolume to 120 GiB (disk already 150 GiB)

Patches: `infra/talos/local/patches/ci-workspace-worker-{1,2}.yaml` (`minSize`/`maxSize` 120GiB).  
**Human apply:** Talos machine config patch + grow XFS / re-provision UserVolume (may need brief worker drain). Do **not** shrink below live claims.

### C. Full CPU/RAM bump (best outcome) — QEMU recreate

`talosctl` QEMU VMs do **not** hot-resize vCPU/RAM. Capture kubeconfig, registry mirror, machine patches, then recreate:

```text
talosctl cluster create oya-talos \
  --workers 2 \
  --cpus 4 --memory 12288 \
  --cpus-workers 8 --memory-workers 49152 \
  --disk-workers … \
  + second blank data disk 150–200G per worker \
  + existing config patches (CNI none, registry mirror, workspace volumes)
```

Exact flags must match the original `oya-talos` bootstrap (see live `~/.talos/clusters/oya-talos/state.yaml`). Prefer a planned maintenance window: this is **destructive** to the live merge-authority cell.

vfkit multinode alternative: `infra/talos/local/MULTINODE-RUNBOOK.md` CI specialty sizes are smaller; this profile supersedes them for the **CI-heavy laptop** merge cell.

---

## Rollback

1. `maxRunners: 1` or `2` via committed values + helm/Argo.  
2. Do not wipe UserVolumes as part of scale-down.  
3. CPU/RAM rollback only by recreating smaller VMs (same destroy/create path).

## Explicit non-goals

- Does not activate warm CAS / Remote Execution.  
- Does not raise live-postgres `maxRunners`.  
- Does not claim local-path enforces PVC size.  
- Does not auto-destroy the live QEMU cluster from an agent.
