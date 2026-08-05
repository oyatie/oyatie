# Retire the cache tarball

## Problem Statement

**How might we** stop spending 40.5 minutes per dev-push building the FULL graph
to produce a `buck-out` tarball, when a content-addressed cache makes the same
work 6.6 seconds?

## The measurement that makes this obvious

| | |
|---|---|
| `gate · affected-set` (builds FULL `//...` to SAVE a tarball) | **40.5 min** |
| Full gate matrix, 1,283 commands, `buck-out` deleted, warm CAS | **6.6s, 100% hits, 314 MiB fetched** |

The 40.5-minute job is not calculating anything. Per the workflow's own comment,
*"that job builds the FULL //... graph on push, so the saved buck-out is the
full-graph superset."* It exists **solely to produce the artifact that lets PRs
skip rebuilding** — and a CAS makes that artifact unnecessary.

**If a warm FULL build is 6.6 seconds, the machinery built to avoid FULL builds
no longer earns its keep.**

## Recommended Direction

Replace GitHub Actions `buck-out` tarball save/restore with the persistent
NativeLink CAS, reached from a self-hosted runner.

The repo already chose the right *strategy* — its comment cites "the
Bazel/Google post-merge-fills-the-cache pattern," and trunk-fills / PRs-read is
correct. The gap is the *implementation*: a directory tarball is the pre-Bazel
approximation of a CAS.

| | tarball (today) | CAS (the pattern) |
|---|---|---|
| granularity | whole `buck-out` | **per action** |
| transfer | multi-GB, all-or-nothing | **on demand** (314 MiB measured) |
| addressing | key-based (`.buckconfig`+`Cargo.lock`+toolchain) | **content hash**; partial hits work |
| lifecycle | explicit SAVE + RESTORE jobs | **none** — always-on infrastructure |
| miss cost | key miss ⇒ full rebuild | miss ⇒ rebuild *that action only* |

Google has no save step because the CAS is infrastructure; incrementality falls
out of content-addressing for free. This also retires **FRIC-017** (the "No
space left on device" from multi-GB per-commit tarball churn) — a CAS fetches
blobs rather than writing a superset per run.

The workflow already names this as the destination: *"Interim warm-by-default
until the shared content-addressed remote cache (NativeLink/CAS, ADR-0560)
lands with a cold-canary integrity job proving cold==warm."* This is not a new
idea; it is finishing a documented one.

**Migration: pilot ONE leg end-to-end, then move all legs together.** buck2
action keys include platform constraints (`cpu:arm64`, `os:macos` are literally
in the configuration hash), so an aarch64 runner and x86_64 GitHub runners share
**nothing**. A leg-by-leg migration would create a lasting split cache — two
namespaces, each cold, hits on neither — for the whole window. The pilot
de-risks the big move without creating that split.

## Key Assumptions to Validate

- [ ] **A self-hosted runner can reach the CAS and get hits.** The pilot. If one
      leg does not hit, nothing downstream matters.
- [ ] **Cold == warm.** The `cache-warm-license.json` precondition: a GREEN cold
      integrity-canary proving a cache-served build is bit-identical to a cold
      one. This gate already exists in the repo and has never been satisfied.
      **It does not yet exist as a job — that is the real work item.**
- [ ] **Linux/x86_64 hit rates match what was measured on aarch64/macOS.** The
      6.6s figure is from a Mac. The *pattern* transfers; the *blobs* do not.
- [ ] **CAS capacity and eviction hold at CI volume.** Measured only at
      single-developer scale. AC and CAS evict independently; a 24h AC TTL plus
      `existence_cache` is a mitigation, not reference-aware GC.

## MVP Scope

**In:** one self-hosted Linux runner reaching NativeLink; ONE gate leg migrated;
before/after hit-rate and wall-clock evidence; the cold-integrity canary job.

**Out:** the other 43 legs, remote execution, HA/multi-node, any change to the
PR/governance surface, deleting the tarball path (keep it until the pilot proves
out).

## Not Doing (and Why)

- **Deleting the affected-set *calculation*** — the `uquery owner → rdeps`
  query is cheap and still useful for scoping tests. Only the FULL-build SAVE
  step is being retired.
- **Leg-by-leg migration** — platform-keyed caches make it a lasting split with
  hits on neither side.
- **Remote execution** — proven unnecessary for cache population
  (`default_allow_cache_upload` populates from local execution). Separate door,
  ADR-0525 D3.
- **"Scan once, assert many"** — falsified: a gate already on the shared face
  showed `Cache hits: 0%`, because buck2 executes tests regardless of declared
  inputs. It also targeted the 1.6-min legs, i.e. under 5% of cost.
- **Making the 47 non-hermetic gates hermetic** — live-corpus gates are
  legitimately non-cacheable; declaring the whole repo as inputs yields zero
  cache value.
- **Building a CI scheduler** — buck2, GitHub Actions and (if ever needed)
  NativeLink's scheduler tier already cover it. A fourth re-implements what
  ADR-0363 retired.
- **Flipping `cache-warm-license.json` on current evidence** — the canary does
  not exist yet. Flipping it without one is exactly the control this repo built
  the flag to prevent.

## Open Questions

1. **Where does the runner live?** colima is aarch64 and provisioned for Talos;
   GitHub's are x86_64. An aarch64 runner starts a fresh namespace — acceptable,
   but it means CI's first warm build must populate its own cache.
2. **Who writes to the CAS?** Today's policy names dev-push as the sole writer.
   With a CAS, does every green PR write, or only trunk? Writer breadth is a
   trust decision, not a performance one.
3. **What does the canary actually compare?** "cold == warm" needs a definition:
   identical action results, identical outputs, or identical verdicts?
4. **Does the 40.5-min job have a second purpose?** It is named a *gate*
   ("binding workspace coverage"). If it also asserts mapping completeness, that
   assertion must survive the SAVE step's removal.
