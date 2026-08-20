# RE (R6) sandbox readiness — plan-only

**Date:** 2026-08-05  
**Lane:** R6-re-sandbox / ultragoal **G043**  
**Issue:** #1549 (RE sandbox)  
**Authority:** CAS plan SHA-256 `8833df33de2600f0bd960518f2402dce20b27ef828cb3cbf27878b4caeaebae5`  
  (`evidence/approved-plan-cas-re-20260805.md`); AGENTS + ADR-0515 merge floor.  
**Explicit non-authority:** **ADR-0612 Proposed does not authorize RE.**  
  It is design input only. No scheduler, workers, or `remote_enabled=true` from this ADR.

**This document is plan-only readiness.** It does **not** implement RE, open #1549 implementation PRs, flip flags, deploy workers, or activate product surfaces.

---

## 1. Authority posture

| Source | Status | RE consequence |
|--------|--------|----------------|
| Approved CAS/RE plan (#1560) | Consensus plan | Sequencing + measured reopen criteria only |
| ADR-0612 Buck2 RE / NativeLink scheduler | **Proposed** | **Cannot** authorize RE activation |
| ADR-0560 NativeLink CAS slice | **Proposed** | Does not authorize warm reads |
| ADR-0515 / `oya-ci-required` | Accepted | Merge admission remains singleton |
| ADR-0613–0616, 0619, 0624 | Accepted | Face de-commit / brand / census constraints on any future reorg PR |
| Option D (plan §4) | Plan disposition | **Owned CI + CAS/AC is a valid terminal state** without RE |

**Default disposition:** stop at owned CI + cache-only CAS/AC unless every quantitative reopen trigger in §3 is measured and an **Accepted** RE decision exists. Plan-only G043 prep may run in parallel as docs/evidence; activation may not.

---

## 2. Prerequisites checklist (must all be true before #1549 implementation)

| # | Prerequisite | Source | Current session state (2026-08-05) | Blocks RE? |
|---|--------------|--------|-------------------------------------|------------|
| P1 | **G039 packet terminal** — #1558 representative storage pilot promoted; post-merge `oya-ci-required` green on promoted SHA; completion packet filled (not draft-only claim) | Plan G039; SESSION / DRIVE-STATUS | #1558 **MERGED** `a1bd1f14a`; G039 packet **DRAFT** awaiting trunk green | Yes — no fan-out while pilot incomplete |
| P2 | **Lane 3A** promoted + post-merge proven: NativeLink manifest/OWNERS → `storage/adapters/nativelink/` under `specs/reorg/nativelink-storage-move-plan.json`; source deleted; no alias; population parity | Plan Lane 3A | **Not started** (after G039 terminal) | Yes |
| P3 | **Lane 3B** starts from **exact promoted 3A head**: complete Buck2 cache package + both warm overlays → `build/buck2/cache/`; all active consumers rewritten; old sources deleted; root `.buckconfig` dark; `specs/reorg/buck2-cache-move-plan.json` | Plan Lane 3B | **Not started** | Yes |
| P4 | **Lane 3C** starts from **exact promoted 3B head**: behavior-only CI cache resolver/policy on canonical paths; pre-edit old-path scan hard-RED if 3B incomplete; **no** move plan | Plan Lane 3C | **Not started** | Yes |
| P5 | **#1541** closed (credential rotation/rebuild + rejection proof + closeout packet). No reuse of any implicated Talos/OpenBao/ARC/NativeLink/cache credential | Plan P0 anti-patterns; `1541-status-20260805.json` | **OPEN** — rotation_complete=false; blocks live CAS / proof-cell / new creds | Yes |
| P6 | **G041** cache-only CAS/AC proof (#1534 path): cold no-participation; distinct reader/writer; `remote=0` / `remote_enabled=false`; digest parity; §5.5 matrix negatives; license-off rollback; **Accepted** warm/CAS authority or plan-scoped waiver as required by CAS Go Gate | Plan Lane 4–6, CAS Go Gate, G041 | **Pending** after #1541 + Lanes 0–3 | Yes |
| P7 | Sustained owned-CI-plus-CAS evidence window (reopen §3.1) and production CAS qualification where plan requires it before Lane 7 | Plan Lane 7 deps; Option D | **Not measured** | Yes |
| P8 | **Accepted RE decision** (or plan-bound Accepted successor to ADR-0612) before activation PR | RE Go Gate | ADR-0612 still **Proposed** | Yes for activation (not for this readiness note) |

### Sequential ownership (hard)

```
G039 terminal
  → 3A promoted+proven
    → 3B from 3A promoted head only
      → 3C from 3B promoted head only
        → #1541 closed
          → G041 cache-only proof (remote=0)
            → measured reopen criteria (§3)
              → #1549 dark design/impl with remote_enabled=false
                → separate reviewed activation PR only if RE Go Gate green
```

No concurrent path ownership across 3A/3B/3C. No consumption of unpromoted predecessors.

---

## 3. Measured reopen criteria (from plan Option D)

Reopen RE design/activation **only when all** of the following are measured on a **preregistered corpus** (plan §4 Option D + §8.1). If any trigger is absent, **stop at owned CI + CAS/AC** and record the next measurement watermark.

| ID | Criterion | Threshold / method |
|----|-----------|-------------------|
| R1 | Sustained green owned-CI-plus-CAS | ≥ **20 consecutive** green runs across ≥ **7 days** |
| R2 | Cache-only end-to-end gate still latency-bound | **p95 wall time > 15 minutes** |
| R3 | Local execution dominates wall time | Local action execution ≥ **60%** of wall time |
| R4 | Queue pressure at proven concurrency cap | Admitted-load queue age **p95 > 120 s** at the proven physical concurrency cap in **three separate 15-minute** windows |
| R5 | Hermetic qualification of proposed canary set | ≥ **90%** of proposed RE canary actions pass hermeticity + deterministic-output qualification |
| R6 | Predicted benefit after full cost model | Model predicts ≥ **30%** p95 wall-time reduction **or** ≥ **20%** cost-per-successful-gate reduction, including scheduler, worker, storage, and control-plane cost |

### Related proof budgets (not substitutes for R1–R6)

Plan §8.1 freezes corpus (control target, medium CI graph, native-build set, affected-set SHA, cache classes, ARM64 then AMD64, ≥30 steady-state reps after warm-up). CAS/queue/scheduling p95 budgets and Mac concurrency step 2→4→6 apply to **CAS/owned-CI proof**, not as RE authorization.

### Corpus freeze rule

Evidence packet freezes corpus, commit, image/toolchain digests, repetitions, concurrency, metrics, and thresholds **before** the run. Post-hoc threshold tuning requires a reviewed revision and a new watermark.

---

## 4. Explicit **not authorized** list

Until Accepted RE authority + RE Go Gate + closed #1549 proof exist, the following are **forbidden** (this readiness note does not schedule them):

| Forbidden | Why |
|-----------|-----|
| `remote_enabled=true` (or any RE client enablement / Buck2 remote execution flip) | Plan cache-only platform; ADR-0612 Proposed; RE Stop/Go gates |
| RE **scheduler** pods / deployments / services (cell-local or otherwise) | Lane 7 keeps scheduler dark; activation only after reopen + Accepted decision |
| RE **worker** pods / node pools / hermetic worker fleets (ARM64/AMD64) | Same; workers only after dark impl + canary sequence |
| **ARC-as-RE-worker** (ARC runner/coordinator pods doubling as RE workers) | Plan P1 anti-pattern #10; ARC is temporary coordinator bridge only |
| Shared identities/cache instances across trust tiers or architectures | Shared-nothing / zero-trust |
| Mounting worker SVIDs, tokens, control sockets, or authorized egress into the action sandbox | P0 anti-pattern |
| Caller-controlled trust headers as policy evidence | §5.5; P0 |
| Merging/copying archived RE/PDP/Envoy prototype | Plan non-goal / P0 |
| Warm CAS reads without #1541 close + CAS Go Gate | G041 / #1534 |
| Treating Proposed ADR-0612 as activation authority | Explicit plan + this program policy |
| Stretching K8s control plane Mac↔OCI WAN for RE | Topology Option B rejected for normal proof |

**Allowed now (docs / process only):** this readiness note; measurement instrumentation design; pre-registration of future canary action lists **as documents**; dual-critic of future #1549 **plans** without code activation.

---

## 5. Suggested future PR shape for #1549 (only after G041)

Do **not** open until: G039 terminal · 3A→3B→3C proven · #1541 closed · **G041 cache-only proof green** · R1–R6 measured or an explicit “stop at CAS” watermark recorded.

### 5.1 Split (minimum three sequential concerns)

| PR | Scope | Hard stops |
|----|-------|------------|
| **#1549-A — Dark contracts (docs + ports only)** | `ci/ports/` execution submission/scheduling contracts; §5.5 fixture specs; sandbox contract (vendor-neutral properties); **no** live scheduler/worker; **no** `remote_enabled` | No cluster apply; no identity mint for RE workers |
| **#1549-B — Dark implementation** | `ci/core/` queue/policy stubs; `ci/adapters/` NativeLink scheduler/worker **provider adapters** behind feature-off; `build/` execution-platform declarations still `remote_enabled=false`; identity outside sandbox; negative authz tests | Keep `remote_enabled=false`; no canary production path; fail closed on PDP outage |
| **#1549-C — Activation (separate reviewed PR)** | Only after RE Go Gate + Accepted RE ADR + R1–R6; single hermetic ARM64 target → bounded set → AMD64 → digest parity → failure injection → small trusted canary | Rollback rehearsed: drain workers, stop scheduler, preserve CAS, cold/local execution |

### 5.2 Canonical ownership (from plan Lane 7)

- `ci/ports/` — execution submission/scheduling contracts  
- `ci/core/` — owned RE scheduling policy and queue behavior  
- `ci/adapters/` — NativeLink scheduler/worker provider adapters while used  
- `build/` — crate-free Buck2 execution-platform and toolchain declarations  
- `os/` / `k8s/` — sandbox/runtime integration  
- `network/` — Cilium policies  
- `secrets/` — worker/coordinator identity projection  
- `iac/` — GitOps reconciliation mechanics  

Sandbox is vendor-neutral (isolation, identity separation, controlled mounts, bounded network, resource accounting, teardown, attestation). Kata is first candidate, not permanent authority.

### 5.3 Activation sequence (post #1549 close, plan Lane 8)

1. Single hermetic target on ARM64  
2. Bounded target set on ARM64  
3. Same set on AMD64  
4. Local-versus-remote digest parity  
5. Controlled failure injection  
6. Small trusted build-class canary  
7. Separate reviewed activation PR  

Concurrency benchmark only at 2 → 4 → 6 with stop on steal/memory/IO/queue/cache/thermal; never infer from host CPU count alone.

### 5.4 RE Go Gate (activation checklist excerpt)

All must be true: CAS production qualification + sustained healthy evidence; **Accepted RE decision**; #1549 closed; scheduler/worker separated from CAS and coordinators; exact URI-SAN + full §5.5 fail-closed; accepted sandbox; ARM64+AMD64 selection; local/remote digest parity; scheduler/worker failure and `remote_enabled=false` rollback rehearsed; independent security, ops, and exact-head code review.

---

## 6. Lane mapping

| Lane | Role for RE |
|------|-------------|
| R5 | G039 → G041 cache-only; **blocks** R6 activation |
| R6 | This readiness + later #1549 only after R5 terminal + criteria |
| R9 | #1541 human security; blocks warm CAS and proof-cell |
| R1 | Runner capacity ≠ RE authorization (queue ops only) |
| R3 | Post-merge packets for 3A/3B/3C/G041; never claim RE from PR-head green |

---

## 7. Honesty statement

- **RE is not authorized** by this file, by ADR-0612 Proposed, by #1558 green/merge, or by runner scale-out.  
- **G043** remains pending until evidence-gated decision.  
- Stopping without RE is success under plan Option D if R1–R6 do not fire.  
- Next agent action on R6: re-query G039 packet, 3A/3B/3C, #1541, G041 status; update this checklist only — **do not implement RE**.
