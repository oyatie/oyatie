# NativeLink Remote-Cache-First on Talos

## Problem Statement
How might we make the buck2 CI gate (and the agent fleet) fast by **warming a shared build cache across ephemeral Talos pods** — without yet taking on full remote execution?

## REAPI architecture (buck2 → NativeLink, gRPC)
```
                          gRPC REAPI
┌──────────────┐   1. CAS (blobs by hash) ─────────────> ┌─ NativeLink CAS ──────┐ ┐
│    Buck2     │   2. Action Cache (action→result) ────> │  NativeLink AC        │ ├ CACHE-FIRST (MVP)
│ (gate pod / │                                          └───────────────────────┘ ┘  local exec
│  laptop)    │   3. Remote Execution ────────────────>  ┌─ NativeLink Scheduler ┐    RE PHASE (later)
└──────────────┘                                         │  + worker pool        │
                                                          └───────────────────────┘
```
Cache-first uses **CAS + AC only**; the Scheduler is dormant until the RE phase.

## Recommended Direction
Deploy **NativeLink cache-only on Talos** (CAS + Action Cache, backed by SeaweedFS S3 at `storage`). Point buck2 at it via `[buck2_re_client]` with a **cache-only** `CommandExecutorConfig`:
`remote_cache_enabled=true` + `allow_cache_uploads=true` + `local_enabled=true` + **`remote_enabled=false`**
(`remote_enabled` = remote *execution* — flipped on later for full RE). Actions execute locally; a cache hit from any prior pod/laptop skips the work; every local result uploads so the next pod builds it for free. 80/20: most of RE's speed, none of the Scheduler/worker ops.

## Key Assumptions to Validate
- [ ] **Cache hit crosses pods** — two PRs touching the same target on two fresh pods → 2nd shows `Commands: (cached: N)`. The whole point.
- [ ] **NativeLink ↔ SeaweedFS-S3** backing works for CAS/AC (anon or keyed, like the sccache bucket).
- [ ] **Action-key determinism across pods** — same pinned aarch64 image + `.buckconfig` + de-cargo'd env ⇒ stable keys ⇒ hits.
- [ ] **In-cluster gRPC + tls** pod → NativeLink reachable.

## MVP Scope
**In:** NativeLink CAS+AC (ci ns, SeaweedFS-backed) + `[buck2_re_client]` endpoint + cache-only `CommandExecutorConfig` in `toolchains//` + the 2-pod hit + cold→warm speedup measurement.
**Out:** Scheduler/RE, worker autoscaling, cross-arch RE.

## Not Doing (and why)
- **Full Remote Execution now** — Scheduler + worker pool is the big op lift; cache-first is the 80/20. Flip `remote_enabled=true` once cache is proven and the agent-fleet fan-out needs distributed exec.
- **NativeLink LRE (Nix) mode** — Nix was dropped; plain REAPI only.
- **Changing the green gate's local buck2** — cache augments it; no gate-logic change.

## How it compounds
Green gate (safe fanout) + NativeLink cache (fast fanout) + llm-gateway (capacity) = the agentic-dev infra trio. RE-later makes the **Talos cluster the build farm** — aarch64-native, hyperscaler-optimal endgame. See [[build-platform-optimization]], [[post-foundation-roadmap]].

## Decided (founder 2026-05-30)
- **Keyed auth** — NativeLink CAS/AC are AUTHENTICATED, not anonymous (unlike the sccache bucket). Keys from OpenBao/secret (not in-spec). Security-first; avoids the open-cache footgun.
- **Unified NativeLink binary, split into 3 K8s TIERS** (one image/release/version; role chosen by config — matches NativeLink's own production guidance: *"in production, CAS and scheduler services run in different processes"*):
  ```
  Buck2 clients ─gRPC REAPI─▶ nativelink-cas pods (CAS + AC)  ◀── critical path for EVERY build
        │                          │ blobs + action results
        │                          ▼
        │                     [ SeaweedFS S3 ]
        └─(Execute; RE only)─▶ nativelink-scheduler (Scheduler + Capabilities; stateful coordinator)
                                   │ gRPC work orders
                                   ▼
                              nativelink-worker pods (execute on Talos; aarch64 farm)
  ```
  - **`nativelink-cas`** — CAS + AC; SeaweedFS-S3-backed, keyed; horizontally scalable (stateless in front of S3); on the critical path for ALL builds incl. cache-only. **MVP = this tier ONLY.**
  - **`nativelink-scheduler`** — Scheduler + Execution + Capabilities; the stateful coordinator (few replicas — Buildbarn runs a *single* scheduler). On the path ONLY for remote execution. **Not deployed until the RE phase.**
  - **`nativelink-worker`** — executes actions on Talos; many pods, compute-scaled. **RE phase only**; flip `remote_enabled=true` → work orders → workers.
- **Why split CAS+AC from the Scheduler (consensus 2026-05-30):** three distinct scaling axes (CAS = storage/bandwidth; Scheduler = coordination state; Worker = compute) and three blast radii — **CAS is on every build's path; a Scheduler fault must NOT take down the cache.** That argument holds even at one cluster, so the earlier "co-locate, split only if scale demands" was wrong. It's the documented production pattern: NativeLink runs CAS/scheduler in separate processes; [Buildbarn](https://github.com/buildbarn/bb-deployments) decomposes `bb-storage` / `bb-scheduler` / `bb-worker` / `bb-runner`. **AC stays WITH CAS** (both are storage). The cache-only MVP has no Scheduler at all, so the split costs nothing now — build the CAS tier; stand up scheduler+worker only when `remote_enabled` flips.
- **Keyed, not anonymous** (above).
