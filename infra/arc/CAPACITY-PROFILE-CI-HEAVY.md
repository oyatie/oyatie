# CI-heavy capacity profile — GitHub-standard runner units

**Host (2026-08-05):** 128 GiB RAM, 18 logical CPUs, multi-Ti free disk.  
**Unit of capacity = one GitHub-standard Linux runner:**

| Surface | vCPU | RAM | Notes |
|---------|------|-----|--------|
| **GitHub-hosted Linux (Ubuntu)** | **2** | **7–8 GiB** | baseline we match |
| GitHub-hosted Windows | 2 | 7–8 GiB | N/A here |
| GitHub-hosted macOS | 3–4 | 14 GiB | N/A here |
| GitHub slim Linux | 1 | 5 GiB | optional “pack” mode only |
| **Oyatie general ARC (this profile)** | **2 request, no CPU limit** | **7 GiB request / 8 GiB limit** | same class as GH Linux |

Matching GH dimensions means wall-clock and job density are comparable to hosted; concurrency is then pure packing math on the laptop cell.

---

## Live Talos (today) vs target

| Node | Live vCPU | Live RAM | Target vCPU | Target RAM | Data disk |
|------|-----------|----------|-------------|------------|-----------|
| controlplane-1 | 3 | ~8 GiB | **4** | **12 GiB** | 50 GiB system |
| worker-1 | **5** | **~30 GiB** | **10** | **56 GiB** | 150→**250 GiB** |
| worker-2 | **5** | **~30 GiB** | **10** | **56 GiB** | 150→**250 GiB** |
| **Guest total** | 13 | ~68 GiB | **24** | **124 GiB** | mild vCPU overcommit on 18 host cores; RAM near full host |

**Why 10 vCPU workers:** each GH-standard runner wants **2** CPU. System + buildkitd + nativelink + postgres cell eat ~2–3 CPU/node.  
10 − 3 = **7 usable → 3 concurrent GH-units per worker** without starving co-tenants. Two workers → **6** concurrent general runners at GH size (comfortable). Path to **8–12** below.

---

## How many runners fit? (answer: more than 4)

### Disk packing (local-path does not enforce PVC size)

| General UserVolume | Max 44 Gi claims/node | maxRunners on 2 workers (topology maxSkew=1) |
|--------------------|------------------------|-----------------------------------------------|
| 48 GiB (legacy) | **1** | **2** |
| 120 GiB | 2 (2×44+4) | **4** |
| **140 GiB** (fits live **150 GiB** `vdb`) | **3** (3×44+4=136) | **6** |
| 200 GiB (after `qemu-img` grow) | **4** (4×44+4=180) | **8** |
| 3rd worker + 140 GiB each | 3 × 3 nodes | **9** |

### CPU packing (GH-standard 2 vCPU unit)

| Worker vCPU | System reserve | GH-units @ 2 CPU | 2 workers |
|-------------|----------------|------------------|-----------|
| 5 (live) | ~2.5 | **1** hard / 2 soft | 2–4 (CPU-starved at 2) |
| 8 | ~2.5 | **2–3** | 4–6 |
| **10** | ~2.5 | **3** | **6** |
| 12 (aggressive) | ~2.5 | **4** | **8** |

### RAM packing (8 GiB limit per GH unit)

| Worker RAM | System ~8 GiB | GH-units @ 8 GiB | 2 workers |
|------------|---------------|------------------|-----------|
| 30 GiB live | ~8 | **2** | 4 |
| 48 GiB | ~8 | **5** | 10 (CPU/disk bind first) |
| **56 GiB** | ~8 | **6** | 12 (CPU/disk bind first) |

**Conclusion:** On this host, **6 concurrent GH-standard general runners** is the sweet spot on **current 150 GiB data disks** (volume 140 GiB, 3/node). **8–12** is realistic after disk grow + 10 vCPU / 56 GiB workers (or a third worker). **4 was never a hardware ceiling** — it was a conservative dual-worker + 120 GiB planning step.

---

## ARC general set (git declaration)

| Knob | Value | Rationale |
|------|-------|-----------|
| `maxRunners` | **6** | 2 workers × 3 GH-units; topology DoNotSchedule maxSkew=1 |
| Runner CPU | **2** request, no limit | GitHub Linux standard |
| Runner memory | **7 GiB** req / **8 GiB** limit | GitHub Linux standard |
| workspace PVC | 44 GiB | unchanged claim size |
| general UserVolume | **140 GiB** | 3×44 + 4 reserve on 150 GiB `vdb` |
| live-postgres | maxRunners=1 | separate cell / GH-sized unit on worker-2 |

Optional **slim pack** (not default): 1 CPU / 5 GiB like GH slim — can raise maxRunners toward 8–10 on same disks if builds remain correct under contention; measure first.

---

## Stretch: 8 or 12 general runners

| Profile | Workers | vCPU/RAM each | Data disk | maxRunners | Host fit |
|---------|---------|---------------|-----------|------------|----------|
| **A (now)** | 2 | 10 / 56 GiB | 150 GiB → vol 140 | **6** | 20 vCPU guest, ~124 GiB |
| **B** | 2 | 12 / 56 GiB | 250 GiB → vol 200 | **8** | 28 vCPU overcommit, same RAM |
| **C** | 3 | 6 / 40 GiB | 150 GiB → vol 140 | **9** | 22 vCPU, ~132 GiB |

Prefer **A → B** (fewer nodes, simpler topology). **C** only if dual-worker disk contention dominates.

---

## Live apply order

### 1. Immediate concurrency (no VM recreate)

```bash
# GH-standard resources + maxRunners=6 from committed values
helm upgrade oya-arm64 \
  oci://ghcr.io/actions/actions-runner-controller-charts/gha-runner-scale-set \
  -n arc-runners --version 0.14.2 \
  -f infra/arc/runner-scale-set-arm64-values.yaml
```

Until UserVolumes are **≥140 GiB**, scheduler may only place **1 claim/node** if still on 48 GiB FS (disk pressure). Grow volumes (step 2) before expecting 3/node.

### 2. UserVolume → 140 GiB (fits existing 150 GiB data disks)

`infra/talos/local/patches/ci-workspace-worker-{1,2}.yaml` — human Talos machine-config apply.

### 3. QEMU CPU/RAM bump (destructive window)

```text
workers: --cpus-workers 10 --memory-workers 57344   # 56 GiB
control-plane: --cpus 4 --memory 12288
# optional disk grow: qemu-img resize worker-*-1.disk 250G before 8-runner profile
```

### 4. Stretch to 8

Grow data disks to ≥220 GiB, UserVolume 200 GiB, `maxRunners: 8`, optional worker vCPU 12.

---

## CAS note (cache hits, not runner count)

NativeLink is live but **mTLS client certs are not mounted on runners** (logs: `NoCertificatesPresented`). Raising maxRunners without mounting `nativelink-client-reader`/`writer` keeps **0% remote cache hits**. Wire certs + `OYA_CI_RE_CACHE_MODE=rw` on cache-writer in parallel.

---

## Explicit non-goals

- Does not activate warm CAS/RE by itself.  
- Does not admin-merge product PRs.  
- Does not raise live-postgres above 1.  
- Does not claim local-path enforces PVC size.
