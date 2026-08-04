# Owned CI on the colima Talos VM, against the shared cache

## Problem Statement

**How might we** make verification fast and owned — CI executing on our own
infrastructure against the shared NativeLink cache — without discarding what the
PR pipeline currently enforces?

## Why now (measured today, not asserted)

- `oya-ci-required` is **~71 min** (40/41/71/73/86) and its cost is **constant
  per PR**: a static 44-leg matrix, `fail-fast: false`, no affected-set filter.
- The cause is not leg count. **GitHub-hosted runners are ephemeral and cold.**
  Every leg rebuilds from nothing, every time.
- The cache now works. A worktree whose `buck-out` was deleted rebuilt at
  **100% cache hits, 53/53 cached, 0.67s**.
- The unblock was `buck2.default_allow_cache_upload = true` (found in a code
  comment at `prelude/rust/build.bzl:1617`, absent from the docs). **No prelude
  fork is required** — the 27MB / 2,216-file fork I was asked to cost is
  unnecessary.
- The colima VM (**12 CPU / 64 GiB / 600 GiB, aarch64**) was provisioned for
  Talos and is idle.
- **Zero self-hosted runners** are registered today.

The repo already designed this. `infra/ci/buckconfig/warm-cache-rw.buckconfig`
says the warm endpoints are "reachable only from owned in-cluster runners
(ADR-0515 D5 destination)". This is the documented roadmap, not a detour.

## Recommended Direction

Stand up Talos on the already-provisioned colima VM. Run **NativeLink CAS+AC as
in-cluster pods** (the ADR-0560 tier split — CAS separate from any future
scheduler, because CAS is on every build's path and must not share a blast
radius). Register a **self-hosted runner in that cluster** and let it execute
`oya-ci-required` against the warm cache.

**Keep the PR surface unchanged for now.** Moving *where verification executes*
and changing *how changes are admitted* are separable, and only the first is
required to collapse 71 minutes. The PR keeps reviewer-APPROVE, the audit trail,
and branch protection as an enforcement point; `oya-ci-required` remains the
single protected context per ADR-0515. Revisit dropping it on evidence once
owned CI is proven — the same sequencing that just resolved the cache question,
where the design premise turned out to be false and only measurement caught it.

The specific reason not to swap PRs for git hooks today: a hook is bypassable
with `--no-verify`. The repo's own enforcement doctrine treats flag-only
enforcement as incomplete and asks "does it hold with hooks off?" A hook is a
fine *accelerator* and a poor *gate*.

## Key Assumptions to Validate

- [ ] **CI legs are cold-build dominated.** Test: run one gate leg locally
      against the warm cache and compare to its GitHub duration. If a leg is
      dominated by checkout or non-cacheable work, the ceiling is much lower
      than 2 min.
- [ ] **ARCHITECTURE MATCH — the highest-risk assumption.** buck2 action keys
      include the platform. The colima VM is **aarch64**; GitHub runners are
      **x86_64**. A cache populated by one **cannot serve** the other. If the
      self-hosted runner is aarch64 while any leg still runs on GitHub x86,
      those legs get **zero hits**. Test: compare action digests for one target
      across both before building anything else.
- [ ] **Runner reaches the cache in-cluster** over gRPC, and `instance_name`
      matches (`main`) end to end.
- [ ] **A laptop-populated cache serves the runner** (and vice versa) — same
      toolchain, so keys should match, but this is the whole premise and is
      cheap to check.
- [ ] **The VM absorbs build load** without starving the agent fleet that shares
      the same physical machine.

## MVP Scope

**In:** Talos on colima → NativeLink CAS+AC as in-cluster pods → one self-hosted
runner → execute **one** gate leg end to end → measure warm vs cold, and confirm
the digest match against a GitHub run.

**Out:** all 44 legs, remote execution, autoscaling, the OCI cluster, any change
to PR/governance.

One leg proves or kills the idea. If a single leg does not get cache hits on the
runner, nothing downstream matters.

## Not Doing (and Why)

- **Dropping the PR** — deferred, not rejected. Execution and governance are
  separable; only execution needs to move to hit the target. Decide later on
  evidence.
- **Git hooks as the gate** — `--no-verify` bypasses them. Accelerator, not gate.
- **An owned admission gate / merge queue** — that is rebuilding a VCS control
  plane, which ADR-0363 explicitly retired in favour of intelligence-on-GitHub.
- **Remote execution** — I previously argued RE was needed for cache population.
  **That was wrong**: `default_allow_cache_upload` populates from local
  execution. RE is now a scaling option, not a precondition.
- **Vendoring the prelude** — a supported config key exists. Zero cost.
- **The OCI Talos cluster** — dev-DB only, shared, non-destructive-use.
- **Flipping `/specs/cache-warm-license.json`** — requires a GREEN cold
  integrity-canary against a warm substrate. The substrate is only hours old.

## Open Questions

1. **If the runner is aarch64 and GitHub legs are x86, is the plan all-or-
   nothing?** Partial migration may yield a split cache with hits on neither
   side. This decides whether legs can move incrementally.
2. **What is CI still for**, once preflight and CI run the identical cached
   graph? If they converge, CI's value becomes the *independent environment*,
   not the checks — which is an argument for keeping it off-laptop.
3. **Does a laptop-dependent CI meet "productized, owned"?** colima is fast to
   prove but ties verification to one machine being awake. Talos-on-OCI is the
   always-on end state.
4. **Cache trust boundary.** Today it is unauthenticated on loopback. In-cluster
   with runners writing, the founder ruling is keyed auth, never anonymous.
