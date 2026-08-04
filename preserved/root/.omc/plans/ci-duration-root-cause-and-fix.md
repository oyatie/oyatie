# CI duration: root cause, measured — and why it is ONE fix, not forty-four

Measured 2026-07-28 from run 30349698618 (`oya-ci-required`, success).

## Which tests run when, and how long

**All 56 jobs run on every PR. Nothing is gated or skipped.**

| min | job | class |
|---|---|---|
| **40.5** | `gate · affected-set` (ADR-0554, binding workspace coverage) | **LARGE** |
| **16.8** | `buck2` (hermetic build + affected gate tests) | **LARGE** |
| 8.0 | `gate-live-postgres-facades` | LARGE (external dep) |
| 8.0 | `freshness` (lock + generated faces) | MEDIUM |
| 6.9 | `registry-drift` (materialized == regenerated) | MEDIUM |
| 6.7 | `cloud-ci-firewall` (baseline ratchet + gate registration) | MEDIUM |
| 4.9 | `producer-regen` (accounting-registry) | MEDIUM |
| 3.9 | `gate-live-postgres-adapters` | MEDIUM (external dep) |
| 1.6–2.3 | **~40 gate legs** | **SMALL** |

Total job time 145 min; wall clock ~71 min; **longest single job 40.5 min**.

## The finding

**The 44 gate legs were never the problem.** Most run in 1.6–2.3 min — by
Google's test-size taxonomy they are already *small* tests, which is exactly
what you want. They should not be touched.

**The optimisation is the bottleneck.** `affected-set` exists to make CI faster
by scoping work, and it is the single slowest job at 40.5 min. It escalates to
**FULL** (`//...` across ~900 crates) when any buildfile is edited, any path
escapes the rdeps cone, or the seed query fails. Since every new gate ships a
BUCK file, structural PRs escalate routinely.

So 40.5 (FULL build) + 16.8 (hermetic build) ≈ **57 of the ~71 min is cold Rust
compilation on ephemeral runners.**

## The fix: one thing, already proven

**Put the NativeLink cache behind a self-hosted runner.** That is the canonical
hyperscaler answer to cold-build cost — it is why Blaze/Bazel have remote
caching at all — and it attacks the measured 57 min directly rather than the
1.6-min legs.

Evidence it works, measured today on this repo:

- Brand-new worktree: **497/502 actions cached (99%)**
- Deleted `buck-out`: **100% hits, 0.67s**
- Cache pays **only** for cold consumers — pooled warm cycles were 314.2s
  without it and 336.2s with it. GitHub-hosted runners are cold **every single
  run**, which is precisely why they take 71 min.

A warm FULL build is mostly cache hits. That is the whole mechanism.

## Hyperscaler patterns — which apply, which do not

| pattern | verdict here |
|---|---|
| **Remote build cache** | **THE fix.** Attacks the 57 min of cold compile. Proven locally. |
| **Test-size taxonomy** (small/medium/large) | Partly already achieved — ~40 legs are small. Use it to *classify*, not to rewrite. |
| **Large/external tests run separately** | `gate-live-postgres-*` (12 min combined) hit a live database — not hermetic, and Google would not run these per-PR. Candidate for a scheduled lane. |
| **Many small hermetic targets** | Already true for the gate legs. Do not "improve" them. |
| **Test sharding** (`shard_count`) | **Does not apply.** These are single repo-scans; sharding replicates the scan per shard. Mitigation for tests already too big — not our shape. |
| **Affected-set / target determination** | Already present, and it is the bottleneck. Fix its FULL-escalation *after* the cache, since a warm FULL may simply be cheap enough. |

## Sequencing

1. **Self-hosted runner + shared cache.** Attacks 57 of 71 min. Everything else
   is second-order until this lands.
   **Validate first:** buck2 action keys include the platform. colima is
   aarch64, GitHub runners are x86_64 — a cache populated by one **cannot**
   serve the other. Compare action digests before building anything else; a
   partial migration could yield a split cache with hits on neither side.
2. **Re-measure.** A warm FULL build may make the affected-set escalation
   irrelevant. Do not optimise it before knowing.
3. **Only then** consider FULL-escalation reduction (synthetic_dependencies /
   completeness), and moving the live-postgres jobs off the per-PR path.

## Explicitly NOT doing, with reasons

- **"Scan once, assert many"** — falsified. A gate already on the shared face
  showed `Cache hits: 0%` and re-ran; buck2 executes tests regardless of
  declared inputs. It also targeted the 1.6-min legs, i.e. under 5% of cost.
- **Test sharding** — wrong shape (see table).
- **Forcing hermeticity on 47 gates** — live-corpus gates are legitimately
  non-cacheable; declaring the whole repo as inputs gives zero cache value.
- **Building a CI scheduler** — buck2, GitHub Actions and (if ever needed)
  NativeLink's scheduler tier already cover it. A fourth would re-implement what
  ADR-0363 retired.
- **Touching the ~40 small gate legs** — they are already the shape the pattern
  recommends.
