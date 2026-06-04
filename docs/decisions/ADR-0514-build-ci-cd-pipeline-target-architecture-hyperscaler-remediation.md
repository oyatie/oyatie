---
id: ADR-0514
status: Proposed
deciders: council-architecture, founder
date: 2026-05-31
owner: council-architecture
supersedes: []
superseded_by: []
related:
  - ADR-0513
  - ADR-0381
  - ADR-0146
  - ADR-0181
  - ADR-0111
  - ADR-0374
  - ADR-0130
  - ADR-0131
  - ADR-0134
related_specs:
  - /specs/masterplan.json
  - /specs/master-plan-sequencing.json
  - /docs/ideas/build-ci-pipeline-review.md
planning_impact: true
milestone: M-TOOLCHAIN
depends_on:
  - ADR-0392
  - ADR-0408
affected_surfaces:
  crates:
    - oya-ci-webhook-gateway
    - oya-ci-controller
    - oya-dev-cli
  microservices: []
  specs:
    - /specs/masterplan.json
deliverables:
  - id: ADR-0514-D1
    description: "Hermetic clang+lld+sysroot toolchain cell as default (replaces host /usr/bin/clang)."
    exit_criteria: "aws-lc-sys + reqwest closure builds cleanly on aarch64-linux + aarch64-darwin; per-crate LDFLAGS=-nostartfiles deleted; git diff clean."
    verified_by: "oya gate validate hermetic-toolchain-default"
  - id: ADR-0514-D2
    description: "Trunk-sourced gate security fix: deployed controller Job clones dev (trusted) + PR-ref as data."
    exit_criteria: "PR editing buck2-affected-gate.sh cannot weaken its own gate; controller runs dev's gate.sh; untrusted Job namespace + NetworkPolicy isolation present."
    verified_by: "oya gate validate controller-trunk-sourcing"
  - id: ADR-0514-D3
    description: "Idempotent buckify + durable DEP propagation (post-buckify patch step)."
    exit_criteria: "make third-party twice → git diff third-party/BUCK clean; per-OS select() + $(location) DEP env present; CI enforces clean diff."
    verified_by: "oya gate validate third-party-buckify-idempotent"
  - id: ADR-0514-D4
    description: "ControllerDispatcher wiring + cutover from Jenkins to deployed oya-ci-controller (P1 core only)."
    exit_criteria: "gateway dispatches to controller /gate-run; controller Job spawns per-PR; parallel-run both gates green; Jenkins gate path deleted."
    verified_by: "oya gate validate controller-dispatch-cutover"
  - id: ADR-0514-D5
    description: "Affected-gate rdeps depth-cap + presubmit/postsubmit two-tier + NativeLink CAS MVP."
    exit_criteria: "third-party/BUCK changes gate under 30min presubmit; postsubmit unbounded + attributed; CAS hit-rate measured >60%; wall-time <20min."
    verified_by: "oya gate validate rdeps-scope-capped"
  - id: ADR-0514-D6
    description: "Structured failure summary from buck2 event-log JSON (replaces fragile grep)."
    exit_criteria: "Controller harvests events; crier posts {target, error_type, first_stderr}[] to Forgejo; logs persisted to S3; no kubectl-exec required."
    verified_by: "oya gate validate structured-failure-summary"
---

# ADR-0514: Build/CI/CD Pipeline Target Architecture + Hyperscaler Remediation

## Status

Proposed — 2026-05-31.

## Context

oyatie's build system exhibits five mechanically distinct, compounding faults that drive a recurring "green-on-dev, red-on-CI, can't-diagnose-why" loop (`docs/ideas/build-ci-pipeline-review.md`):

1. **Hermetic toolchain gap.** The build uses the host's `/usr/bin/clang` (`toolchains/BUCK:22-29`), so dev (macOS/ld64) and CI (Linux/lld) diverge at the link layer. Every "works locally, breaks in CI" bug traces here.

2. **Non-durable third-party layer.** `reindeer buckify` regenerates `third-party/BUCK` from static TOML and wipes every hand-added per-OS `select()` and `$(location …)` DEP env (exactly 4 in a 19.7k-line file). Corrections survive only as prose warnings.

3. **Double-CRT at the prelude layer.** The bundled prelude's `__ld_shim` re-invokes clang as the linker driver, re-adding C-runtime startfiles. Any build-script that links an executable (aws-lc-sys memcmp probe) gets `ld.lld: duplicate symbol _start/_init` on Linux.

4. **PR-sourced gate security hole.** The Jenkinsfile is trunk-sourced but runs `infra/ci/buck2-affected-gate.sh` from the PR workspace (`Jenkinsfile:53,76`), making the gate script PR-controllable — a half-open `pull_request_target`-class vulnerability.

5. **Full-closure scale crisis.** A one-line `third-party/BUCK` change owner-expands to ~1689 targets → rdeps closes to ~1919 near-whole-tree targets, built **and** tested serially under a hard 60-min timeout with no cache tier.

**The throughline:** the build is not hermetic, the third-party layer is not durable, and the gate is not isolated. Every other symptom is downstream of those three.

ADR-0513 already scopes the removal of the Jenkins+Groovy+cpsScm orchestration fragility as a multi-phase replacement with a bespoke-Rust `oya-ci` platform (hook/plank/crier/tide/deck/plugins). This ADR narrows the immediate scope to **ADR-0513 Phase 1 (core gate reliability)** and specifies the prerequisite fixes (hermetic toolchain, trunk-sourcing, durable buckify, rdeps scope-capping) that unblock the controller and close the critical failure modes.

## Decision

### 1. End goal: closed-loop dogfood

Build a **closed-loop, conflict-free merge** that enables the rest of oyatie to be automated autonomously:

> Parallel fan-out agents (across multiple subscriptions) work ADRs → open PRs → CI/CD gate (green/red) → review/fix loop → **merge to `dev` with zero merge conflicts** → repeat.

The structural enabler of "zero conflicts" is **Tide-style merge-queue** (ADR-0111 / ADR-0513). Historical wording in this ADR deferred Tide behind gate reliability; the 2026-06-02 correction below moves Tide admission ownership and automatic merge after CI into Phase 0 while leaving projected-state batching and larger Tide ergonomics phased.

### 2. Target architecture (minimal robust shape)

**Three hops, one language, isolated gate:**

```
Forgejo PR webhook
  → oya-ci-webhook-gateway (bespoke Rust, ADR-0374; HMAC fail-closed)
    → deployed oya-ci-controller (bespoke Rust, kube-rs; K8s-native)
      → K8s Job [git clone dev (TRUSTED) + fetch PR-ref as DATA
                  → run dev's gate.sh
                  → buck2 build/test with NativeLink CAS]
        → controller harvests buck2 event-log JSON
          → Forgejo commit-status + structured failure summary
```

**Properties:**
- One language (Rust); no Groovy/CPS.
- Hermetic toolchain default; dev == CI at the link layer.
- Gate logic trunk-sourced (pr-ref as data only); PR cannot weaken its own gate.
- Structured observability (buck2 event-log → failure taxonomy); no `kubectl exec` to diagnose.
- CAS-first for cache efficiency; defer RE tier until measured and justified.
- **Tide admission and auto-merge-after-CI are Phase-0 contracts** per the 2026-06-02 correction; defer projected-state batching, deck, and plugins behind explicit demand.

### 3. Six founder principles (non-negotiable)

1. **Hermetic + "just works"** — Fresh macOS/Linux → one bootstrap → dev ≡ CI ≡ deploy ≡ agent-env. Zero host dependencies.
2. **Hyperscaler shape** — Hermetic build → CAS/RE → trunk-sourced affected gate → introspectable controller → merge-queue → reviewer-agent → fan-out.
3. **Observability by DEFAULT** — OTel, persisted/queryable. Never `kubectl exec` to diagnose.
4. **Automate-DRY** — Did-it-twice → codify. No per-crate fixup whack-a-mole.
5. **Doubt-driven** — Audit for anti-patterns continuously. Evidence gates over aspirational enforcement.
6. **Conflict-free merge via Tide** — Structural enabler of the closed loop.

### 4. Anti-patterns being killed

- **Full-tree build per third-party change.** → rdeps depth-cap + presubmit/postsubmit two-tier.
- **PR-sourced gate.** → deployed controller + trunk-sourced Job command.
- **Per-crate host-hardcoded fixups.** → hermetic toolchain + durable buckify patch step.
- **Ephemeral-pod opaque logs.** → structured failure summary from buck2 event-log JSON.
- **Jenkins + Groovy + cpsScm fragility.** → bespoke-Rust deployed controller.
- **Dockerfile-based BuildKit image builds.** → buck2-native OCI images (transitory, post-gate-green).

### 5. Sequencing (MVP first, defer non-blocking)

#### P0 — OPEN (gate-reliability spine; unblocked today)

| Seq | ID | Title | Status | Anti-pattern(s) |
|---|---|---|---|---|
| 1 | #96 | `cloud-intelligence-app` final-link failure (aarch64-linux repro) | in-progress (repro) | green-on-dev/red-on-CI; link divergence |
| 2 | #83 | Hermetic clang+lld+sysroot toolchain cell as DEFAULT | open | host-toolchain divergence; double-CRT; per-crate fixups |
| 3 | #95 | Trunk-source the gate (SECURITY: controller Job clones dev + PR-ref as DATA) | open | PR-sourced gate |
| 4 | post-buckify patch + CI clean-diff | Idempotent `reindeer buckify` + DEP env durability | open | non-durable third-party |
| 5 | #88 oya-ci controller P1 | Add `ControllerDispatcher` + cutover (carry scale-cap + log-harvest) | open | Jenkins fragility; self-deadlock; opaque logs |
| 6 | #94-secondary rdeps depth-cap | Depth-limited presubmit + postsubmit tier + CAS-only-first | open | full-tree build per third-party change |

#### DONE — landed (record as resolved in ADR-0134)

| ID | Title | Status |
|---|---|---|
| #93 | aws-lc-sys `LDFLAGS=-nostartfiles` | done (bundled PR#23, blocked by #96) |
| #91 | openssl `DEP_OPENSSL_*` platform `select()` | done (bundled PR#23, blocked by #96) |
| #94 | gate rdeps `@argfile`/`%Ss` | done (landed dev) |
| PR#25 | gate root-cause summary (Jenkins-side precursor) | done (landed dev) |

#### LATER — post-gate-green (ADR-0513 Phases 2-4; sequence-after)

| ID | Title | Status | Role |
|---|---|---|---|
| #89 | Tide / merge-queue | open now for P0.0 admission + auto-merge-after-CI; projected-state batching later | conflict-free auto-merge |
| #90 | reviewer-agent + auto-fix loop | open (deferred) | closes review/fix arc |
| oya-ci-deck | SolidJS CI-visibility surface | open (deferred) | founder + agent introspection |
| oya-ci-plugins | ChatOps / governance pipeline | open (deferred) | ops ergonomics |
| buck2-native OCI | Retire BuildKit/Dockerfile | open (deferred) | Dockerfile non-hermetic |
| bespoke NativeLink RE | Scheduler + workers (measurement-gated) | open (deferred) | parallelism (after CAS hit-rate validated) |

### 6. Key assumptions to validate (acceptance-test criteria)

1. **Double-CRT ≠ final-link failure.** Test: on aarch64-linux, build aws-lc-sys ±`-nostartfiles` (expect fix) AND cloud-intelligence final link with workaround in place (expect still-fails → different mechanism).
2. **Hermetic toolchain unblocks all OS-divergence.** Test: swap toolchain, remove all per-crate workarounds, build aws-lc-sys + reqwest closure on both aarch64-linux + aarch64-darwin.
3. **Post-buckify patch is idempotent.** Test: `make third-party` twice → `git diff third-party/BUCK` clean both times + `select()`/`$(location)` present.
4. **CAS hit-rate alone brings 1919-closure under timeout.** Test: warm CAS on dev, change one third-party crate, measure presubmit wall-time + cache hit %.
5. **Controller trunk-sourcing resists malicious gate.sh.** Test: open PR editing `buck2-affected-gate.sh` to `exit 0`; confirm controller still runs *dev*'s gate.sh.
6. **Depth-capped presubmit + postsubmit catches regressions.** Test: seed a known third-party regression; confirm depth-N presubmit catches it OR postsubmit attributes within one cycle.

## Consequences

**Positive:**
- Kills all six failure modes identified in the current Jenkins gate (parse fragility, self-deadlock, opaque logs, cold-pod no-completion, PR-sourced gate, full-tree scale).
- One pure-Rust, Forgejo-native platform (gateway + controller) replaces Jenkins + Groovy + JCasC + separate merge-queue + bespoke auto-merge glue.
- Hermetic toolchain aligns with the `kubers`/`source` Rust-K8s ambition and single-bootstrap/zero-drift doctrine.
- Durable buckify + scope-capped rdeps + CAS-first strategy unblocks parallel fan-out agents (the closed-loop enabler).
- Trunk-sourced gate closes a half-open security hole.

**Negative/cost:**
- Owning a multi-phase CI platform increases reinvention risk; mitigated by lifting Prow's proven plank state-machine and adversarial review.
- Hermetic toolchain requires prebuilt clang+lld+sysroot archives per OS; fallback is vendoring + patching prelude (manual upgrade tracking).
- Post-buckify patch step is external machinery; future work may generate per-OS DEP `select()`s, eliminating it.

**Neutral:**
- The gate *logic* and branch-protection unchanged; the gateway (ADR-0374) is retained.
- Deck/plugins and large-scale Tide batching still land incrementally; Tide admission ownership and auto-merge-after-CI are no longer deferred per the 2026-06-02 correction.

## Verification

Per-deliverable `verified_by` gates above. Net:
- `oya gate validate hermetic-toolchain-default` (toolchain builds both hosts, per-crate workarounds deleted)
- `oya gate validate controller-trunk-sourcing` (PR-sourced gate cannot execute)
- `oya gate validate third-party-buckify-idempotent` (buckify + patch → clean diff)
- `oya gate validate controller-dispatch-cutover` (both gates green in parallel, Jenkins path deleted)
- `oya gate validate rdeps-scope-capped` (presubmit <30min, postsubmit attributed, CAS >60% hit)
- `oya gate validate structured-failure-summary` (events harvested, crier posts summary, S3 persisted)

Local testing per §6 assumptions validates the root causes before full cutover.

## References

- `/docs/ideas/build-ci-pipeline-review.md` — full analysis, 279 lines, every claim file:line-cited.
- ADR-0513 (oya-ci bespoke-Rust Prow platform, Phases 0-4).
- ADR-0111 (merge-queue projected-state).
- ADR-0374 (CI webhook gateway, Forgejo → Jenkins).
- ADR-0381 (Kaniko → BuildKit + Talos topology).
- ADR-0392 (Buck2 canonical build graph).
- ADR-0408 (Buck2-driven CI/CD).
- docs/research/mold-linker-impl-2026-05-28.md (mold linker via clang — the linker/toolchain-routing decision; currently a research note + a `.cargo/config.toml` comment citing the not-yet-authored "ADR-0488"; promote to a real ADR before this ADR is Accepted).
- ADR-0146 (distroless base image).
- ADR-0181 (image promotion + cosign).
- ADR-0130 (observability SLOs).
- ADR-0131 (per-microservice flat layout).
- ADR-0134 (portfolio hyperscaler remediation backlog — append P0 + LATER work items).

## Open Questions

1. Cloud-intelligence final-link failure (#96): static-crypto duplicate-symbol, mold-vs-lld drift, or other? (Gates the double-CRT vs. final-link mechanism split.)
2. Hermetic toolchain (#83): is a maintained prebuilt clang+lld+**sysroot** archive available for aarch64-linux + aarch64-darwin, or must we build/host it?
3. Post-hermetic-toolchain: can remaining per-OS DEP `select()`s be *generated* (eliminating the patch step), or is the patch permanent?
4. What depth N for the rdeps cap balances catch-rate vs. timeout?
5. Does JCasC re-seed an existing live job on restart, or must it be manually deleted+recreated?
6. Does cutover need parallel double-CI-cost during migration, or can the controller take over atomically?

## 2026-06-02 Tide deferral correction

Founder directive on 2026-06-02 supersedes this ADR's wording that deferred Tide
behind explicit demand. Deck and plugin expansion can remain phased, but Tide /
merge-queue ownership and automatic merge after CI belong in cloud-ci/oya-ci now.
The P0.0 admission contract requires `oya-ci-required`, Buck2-owned gate
execution, PR-head pinning, and mergeability/conflict checks before Forgejo or
GitHub auto-merge is armed.

This correction is target/contract evidence only until live branch protection and
the trusted cloud-ci/oya-ci producer converge on the candidate SHA.
