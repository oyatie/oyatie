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
Deploy **NativeLink cache-only on Talos** (CAS + Action Cache, backed by SeaweedFS S3 at `oya-storage`). Point buck2 at it via `[buck2_re_client]` with a **cache-only** `CommandExecutorConfig`:
`remote_cache_enabled=true` + `allow_cache_uploads=true` + `local_enabled=true` + **`remote_enabled=false`**
(`remote_enabled` = remote *execution* — flipped on later for full RE). Actions execute locally; a cache hit from any prior pod/laptop skips the work; every local result uploads so the next pod builds it for free. 80/20: most of RE's speed, none of the Scheduler/worker ops.

## Key Assumptions to Validate
- [ ] **Cache hit crosses pods** — two PRs touching the same target on two fresh pods → 2nd shows `Commands: (cached: N)`. The whole point.
- [ ] **NativeLink ↔ SeaweedFS-S3** backing works for CAS/AC (anon or keyed, like the sccache bucket).
- [ ] **Action-key determinism across pods** — same pinned aarch64 image + `.buckconfig` + de-cargo'd env ⇒ stable keys ⇒ hits.
- [ ] **In-cluster gRPC + tls** pod → NativeLink reachable.

## MVP Scope
**In:** NativeLink CAS+AC (oya-ci ns, SeaweedFS-backed) + `[buck2_re_client]` endpoint + cache-only `CommandExecutorConfig` in `toolchains//` + the 2-pod hit + cold→warm speedup measurement.
**Out:** Scheduler/RE, worker autoscaling, cross-arch RE.

## Not Doing (and why)
- **Full Remote Execution now** — Scheduler + worker pool is the big op lift; cache-first is the 80/20. Flip `remote_enabled=true` once cache is proven and the agent-fleet fan-out needs distributed exec.
- **NativeLink LRE (Nix) mode** — Nix was dropped; plain REAPI only.
- **Changing the green gate's local buck2** — cache augments it; no gate-logic change.

## How it compounds
Green gate (safe fanout) + NativeLink cache (fast fanout) + llm-gateway (capacity) = the agentic-dev infra trio. RE-later makes the **Talos cluster the build farm** — aarch64-native, hyperscaler-optimal endgame. See [[build-platform-optimization]], [[post-foundation-roadmap]].

## Decided (founder 2026-05-30)
- **Keyed auth** — NativeLink CAS/AC are AUTHENTICATED, not anonymous (unlike the sccache bucket). Keys from OpenBao/secret (not in-spec). Security-first; avoids the open-cache footgun.
- **Unified NativeLink binary, split into 2 K8s TIERS** (NativeLink-idiomatic: one image/release/version, role chosen by config):
  ```
  Buck2 clients ──gRPC REAPI──▶ nativelink-frontend pods (CAS + AC [+ Scheduler])
                                   │ S3 assets          │ gRPC work orders
                                   ▼                    ▼
                              [ SeaweedFS S3 ]    nativelink-worker pods (execute on Talos)
  ```
  - **`nativelink-frontend`**: CAS + AC + Scheduler services; SeaweedFS-S3-backed, keyed. **MVP = frontend with CAS+AC only** (Scheduler dormant), **no workers** → buck2 CAS+AC, local exec.
  - **`nativelink-worker`**: executes actions on Talos. **RE phase only** — enable the frontend's Scheduler service + deploy workers (scalable aarch64 build farm); flip `remote_enabled=true` → work orders → workers.
  - Frontend co-locates CAS+AC+Scheduler (right-sized for one cluster; split CAS-from-Scheduler only if scale demands). Worker is the independently-scaled tier.
- **Keyed, not anonymous** (above).
