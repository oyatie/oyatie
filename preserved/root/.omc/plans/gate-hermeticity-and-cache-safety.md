# Gate hermeticity, cache safety, and where the cache actually pays

Every number here was measured on 2026-07-28. Two of my own conclusions were
falsified during the work and are marked.

## The census answers the question before the plan does

**47 of 48 gate crates with tests escape to the live corpus** (`current_dir()`
walk-up, `repo_root()`, or `CARGO_MANIFEST_DIR`). That is not 47 defects. It is
the architecture: these are live-corpus **policy gates**, and a gate asserting
"every declared scan root resolves" *must* read the real repository.

**(a) DECISION — live-corpus gates are legitimately non-cacheable. Mark them,
do not make them hermetic.** Declaring the whole repo as inputs would be both
impractical and pointless: any file change would invalidate all 44 legs, so the
cache value would be zero anyway. Bazel has `no-cache` / `no-remote-cache` tags
for exactly this class; the hyperscaler pattern is to *classify* the
non-hermetic action, not to pretend it is hermetic.

The anti-pattern is not "a test reads the repo". It is **an undeclared
dependency that is silently treated as cacheable**. Classification removes the
silence.

## (c) The cache is SAFE today — my alarm was mostly wrong

I raised a stale-pass risk: a governed file changes, `tests/**/*.rs` does not,
identical action key, cache serves a stale PASS. **Measurement contradicts it.**

buck2 caches BUILD ACTIONS; test execution is separate (tpx) and re-runs:

- Fresh worktree: `Commands: 502 (cached: 497)` — yet the test **executed** and
  failed on a missing generated face.
- Repeat run: `Cache hits: 100%` alongside `Tests finished: Pass 1` — executed,
  not served.

The non-hermetic part is precisely the part that is not cached. No kill switch
is needed. `/specs/cache-warm-license.json` should still stay `false` until a
GREEN cold canary, because that control covers cold-vs-warm divergence in
general — but the specific failure I feared is not live.

## (b) DECISION — sharding has no role here

`test-execution` is 10.77s of a 12.4s critical path (87%); compilation is ~1.6s.
That looks like a sharding target and is not one:

- The Google/Bazel first-order pattern is **many small hermetic targets**;
  `shard_count` is a mitigation for tests already too large.
- These gates are single repo-scans. Sharding by test function would
  **replicate the scan per shard** and make it worse.
- The real lever is **not running all 44 legs on every PR**. The repo already
  owns an affected-set (ADR-0554) that the static gate matrix ignores.

## The reframe that matters: the cache pays for COLD consumers only

Measured, pooled 3-lane cycles:

| | c1 | c2 | c3 | median |
|---|---|---|---|---|
| no cache | 446.5s | 314.2s | 310.9s | **314.2s** |
| with cache | 373.7s | 323.4s | 336.2s | **336.2s** |

**No improvement.** The 60s bar is not met and the cache cannot meet it: the
critical path is test execution, which does not cache.

But on a **cold** consumer the cache is transformative — a brand-new worktree
built at **497/502 (99%) cached**, and a deleted `buck-out` rebuilt at 100% /
0.67s.

So: **my local lane loop was never the right beneficiary** (slots are already
warm). The right beneficiary is anything ephemeral — which is exactly why
GitHub-hosted runners take 71 minutes. This validates the owned-CI direction and
explains why the local 60s target failed. Two different problems.

## (d) Sequencing

1. **Classify the non-hermetic gates** (`no-remote-cache`-equivalent) so the
   status is explicit rather than accidental. Cheap, and it is the honest
   precondition for any broader cache rollout.
2. **Owned CI on colima Talos** (`.omc/plans/owned-ci-on-colima-talos.md`) —
   this is where the cache pays, because runners are cold by construction.
   Validate the aarch64-vs-x86 action-key question FIRST; a split cache hits on
   neither side.
3. **Affected-set for the gate matrix** — the only lever that attacks the 71
   minutes at its root rather than making each leg faster. Largest win, largest
   governance surface, and independent of the cache.
4. **Do not pursue** local-loop cache optimisation. Measured to be worth nothing.

## Falsified along the way (recorded so it is not re-derived)

- *"RE is required to populate the cache."* Wrong —
  `buck2.default_allow_cache_upload=true` populates from local execution. No
  prelude fork needed (would have been 27 MB / 2,216 files owned forever).
- *"The cache will produce stale gate passes."* Mostly wrong — build actions
  cache, tests execute.
- *"Worktree pooling recovers the cold-build cost."* The first measurement said
  no; the experiment was broken (the driver deleted the pool each cycle). Fixed
  it: 446.5 → 314.2 → 310.9s, converged. **A measurement harness that destroys
  the thing under test yields a confident wrong number.**

## Open questions

1. Is there a supported buck2 tag for "never cache this action", and does it
   apply to `rust_test` targets specifically?
2. Should the live-corpus gates instead consume a **declared snapshot** of the
   corpus (the materialized faces) rather than reading it live? That would make
   them hermetic AND cacheable — but only if the snapshot itself is a declared
   build artifact, which `materialize_faces` (140.8s, undeclared) currently is
   not. This is the only path to genuinely cacheable gates.
3. Does the affected-set already know the gate→path mapping needed for (3)?
