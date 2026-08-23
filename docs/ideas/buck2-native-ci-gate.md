# Buck2-Native CI Gate (Hyperscaler Structure)

## Problem Statement
How might we gate every PR to `dev` with hyperscaler-grade correctness — fast, hermetic, and without cargo, a fat agent image, or the retiring `oya` CLI?

## Recommended Direction
Pivot the gate from the cargo-interim (`cargo build dev-cli` + `oya verify --affected`) to **buck2-native**. Every cargo-interim blocker (sccache backend, CARGO_HOME perms, clang+mold, cargo-nextest, custom merge-base affected bugs, dev-not-gate-clean) is a symptom of running per-language tools + a custom affected-detector inside a hand-built agent image — exactly what a build-graph tool replaces. buck2 `//...` is now green (darwin), so this is viable.

The end-state is a **stack**, sequenced over several landings:
1. **buck2-native gate body** — `buck2 build/test` driven by a **BXL affected query** (replaces `oya verify --affected`; no merge-base CLI bugs, no oyatie-CLI dep).
2. **Presubmit-affected + postsubmit-full** — fast affected build/test on the PR; full `buck2 build/test //...` on `dev` after merge. Canonical Google/Meta split.
3. **NativeLink remote cache** (Rust, Apache-2, S3/SeaweedFS-backed — passes hyperscaler + bespoke lens) → warm cross-build cache; replaces sccache entirely.
4. **Hermetic toolchains** (`download_toolchain` rust+clang as build inputs) → agent image collapses to *just buck2*; kills image-drift whack-a-mole permanently.
5. **Remote Execution** (NativeLink RE) → distributed, the 10× finish.

Reuse verbatim the proven, build-system-agnostic chain: GitHub webhook → ci-webhook-gateway → Jenkins `ci-gate` (genericTrigger flat paths) → authed checkout → commit-status. Only the ~3-line gate **body** changes.

## Key Assumptions to Validate
- [ ] **buck2 `//...` AND `buck2 test //...` green on LINUX** (only verified darwin). The native fixups (psm/blake3/ring/aws-lc/openssl) hardcode `/usr/bin/clang` + `*-apple-darwin` triples — Linux-correctness is the #1 risk. *Test: in-cluster buildkit/Job runs `buck2 build //... && buck2 test //...` on the rust-ci(linux) image.*
- [ ] **buck2 test targets exist and pass** (we verified build, not test). *Test: `buck2 targets //... --type rust_test` count; `buck2 test //...`.*
- [ ] **BXL affected from a changed-file list is feasible** (bxl `cquery rdeps`). *Test: write a small `affected.bxl`, feed PR changed files, confirm target set.*
- [ ] **NativeLink ↔ buck2 ↔ SeaweedFS-S3** integration works. *Test: deploy NativeLink (cache-only), point buck2 `[buck2_re_client]`/cache at it, confirm cache hits.*
- [ ] **Governance-lanes disposition** — do `buck2 build/test` replace the 99 retired CLI lanes, or do select lanes run as buck2 test targets? (per cli-retirement-blueprint).

## MVP Scope (first landable increment, P0+P1)
- **P0 (gating risk):** prove `buck2 build //... && buck2 test //...` green on Linux in-cluster. Fix any Linux fixup gaps (clang/triple). NOTHING else proceeds until this is green.
- **P1:** gate body → `affected.bxl` → `buck2 build+test` affected closure; thin rust-ci image (buck2 + clang only; drop sccache/nextest/CARGO_HOME); enforce **presubmit-affected** on `dev` + branch protection (retire relax-merge). dev's latent dirt only blocks PRs that touch dirty targets (correct — fix-what-you-touch).
- Out of MVP: NativeLink, postsubmit-full, hermetic toolchains, RE.

## Not Doing (and why)
- **Finish cargo-interim** — dead: throwaway + whack-a-mole + gated on dirty dev.
- **Full Remote Execution now** — biggest infra; not needed for correctness; the finish, not the start.
- **Bespoke CI executor** — Jenkins is a fine executor/trigger; not the bottleneck.
- **Replace Cedar / remove psm** — psm is legitimate (see psm-cedar-dependency-rationale); out of scope.
- **One-time full dev cleanup up front** — affected-mode tolerates latent dirt; clean incrementally (fix-what-you-touch) + flag via postsubmit.

## Open Questions
- Governance lanes (99 retired CLI): retire into buck2 test targets, or keep a thin subset?
- dev-cleanliness ratchet: warning-only period before hard-enforce, or hard from day 1 (affected-only makes hard-from-day-1 safe)?
- NativeLink vs bazel-remote vs Buildbarn — confirm via best-practice-research before adopting (bespoke lens favors NativeLink/Rust).
