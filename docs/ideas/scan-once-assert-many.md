# Scan once, assert many

## VERDICT: FALSIFIED — do not build this

The falsifier this document itself named (Open Question 4) was run and fired:

```
ci-scm-facts-snapshot-gate  ->  Cache hits: 0%
                                Commands: 2 (cached: 0, local: 2)
                                Tests finished: Pass 1
```

A gate **already consuming the shared scm-facts face** gets ZERO cache hits and
re-executes. Migrating the 23 walkers onto a face therefore CANNOT produce the
success measure ("a gate leg that is a genuine cache no-op"), because buck2
executes tests regardless of how well their inputs are declared.

Three adversarial challenges, in descending severity:

1. **THE PREMISE IS WRONG.** The "test-execution = 87% of critical path" figure
   was measured on a WARM build with compilation already cached. On a cold
   runner that inverts: cold builds here were 200-550s of compile against ~10s
   of test, and the 44 legs run in PARALLEL. If scanning were the bottleneck,
   44 parallel ~10s legs would total ~10s, not 71 min. **71 min is cold Rust
   compilation plus queueing** — which the NativeLink cache already fixes (99%
   hits in a fresh worktree). This idea attacks well under 5% of the real cost.

2. **THE CURE HAS THE DISEASE.** A universal face has maximum fan-in: every gate
   depends on it, so any repo change regenerates it and invalidates every gate.
   That is exactly the FULL-tier degeneracy this document accuses affected-set
   of. Repairing it needs SHARDED per-crate faces — ~900 of them, multiplying
   the freshness failure mode that is already live here.

3. **BAZEL WOULD NOT BUILD A FACE.** The build graph already knows every source
   file and its metadata; the idiomatic answer is a query/aspect over the graph
   (`uquery`/`cquery`/`aquery`), not a snapshot artifact. A face is a second
   source of truth that can go stale. The graph cannot.

**What survives:** deduplicating 23 redundant repo walks is a real but small
win, and declaring each gate's own policy JSON is cheap and correct. Neither
justifies the migration on its own.

**Correct sequencing instead:** get the cache to CI runners first (it fixes cold
compile, the actual 71 min), re-measure the critical path on a warm runner, and
only then decide whether scan deduplication is worth anything. Optimising
before that re-measurement is optimising a term we have not yet observed to be
dominant.

The analysis below is retained because the census and the two-class split are
sound and reusable — only the conclusion drawn from them was wrong.

---

## Problem Statement

**How might we** make global repo gates fast, hermetic, and cacheable at once —
by having them consume a declared corpus snapshot instead of each independently
walking a 19,010-file repository?

## Why this is the root cause, not a symptom

Measured 2026-07-28:

- `oya-ci-required` is **~71 min**, a **static 44-leg** matrix with **no
  affected-set filter**.
- **23 gate crates each run their own filesystem walk.** The same repository,
  scanned 23 times.
- **47 of 48 gate crates with tests escape to the live corpus**
  (`current_dir()` walk-up / `repo_root()`), declaring only
  `srcs = glob(["tests/**/*.rs"])`.
- `test-execution` is **10.77s of a 12.4s critical path (87%)**; compilation is
  ~1.6s.

The static matrix is not an oversight. A gate asserting "every declared scan
root resolves" is affected by *any* change, so affected-set scoping degenerates
to FULL for this whole class. **You cannot scope global gates by path while
their input is "everything".**

## Recommended Direction

**The pattern already exists here and is ~1/3 adopted.** 14 gates consume a
shared `scm-facts` precomputed face (1.2 MB); 23 still walk. Migrating walkers
onto a declared face collapses four problems into one move:

| problem | why a declared face fixes it |
|---|---|
| 71 min CI | one scan, not 23 |
| non-hermetic (47/48) | a face is a **declared input** |
| uncacheable | action key covers the face → real cache hits |
| affected-set → FULL | the face is a *file*, so gates become path-scopable |

That last row is the point: **the face is the precondition that makes
affected-set possible for global gates.** It is not an alternative to scoping,
it is what unlocks it.

The census shows two classes, and they should NOT be solved the same way:

- **Class A — path/metadata gates** (`crate-name-prefix`, `crate-layer-suffix`,
  `module-membership`, `port-placement`, `service-tier-metadata`,
  `facade-core-layering`, `layer-dependency-acyclicity`,
  `workspace-member-coverage`). They assert where files live and what they are
  named. A paths+metadata face fully serves them. **Migrate these.**
- **Class B — content-scanning gates** (`crypto-backend-policy`,
  `graphql-usage-policy`, `caller-supplied-authorization`,
  `endpoint-authorization-coverage`, `embedded-asset-hermeticity`,
  `core-dependency-isolation`). They need file *contents*. That is exactly the
  ADR-0580 corpus/AST substrate's purpose. **Do not hand-roll a content index**
  — the cache design was already built once and nearly redesigned by accident.

Every gate also reads its **own policy JSON**, which is declarable today in
either class and is the cheapest first win.

## Key Assumptions to Validate

- [ ] **A declared face actually yields a cache no-op.** Test: build a Class A
      gate twice with the face unchanged; the second must do zero work. This is
      the success measure and nothing else matters if it fails.
- [ ] **The face can be produced as a buck2 artifact with declared inputs.**
      Today `materialize_faces` is a separate 140.8s step whose inputs are
      undeclared — so the face itself is currently outside the graph. Until the
      producer is a proper target, its consumers cannot be hermetic either.
- [ ] **A paths+metadata face is sufficient for Class A.** Verify against two
      gates before generalising; a face that does not fit its consumers is worse
      than no face.
- [ ] **Gate semantics survive.** A gate reading a snapshot asserts about the
      snapshot, not the working tree. If the face is stale, the gate is
      confidently wrong — freshness becomes the new failure mode, and staleness
      is already a measured one in this repo.

## MVP Scope

**Pilot ONE Class A gate end to end** and demonstrate all four properties:
declared face input → hermetic → genuine cache no-op → path-scopable.

**In:** one gate, its policy JSON declared, its walk replaced by a face read,
before/after cache-hit evidence.
**Out:** the other 22, Class B entirely, the corpus/AST substrate, any change to
the gate matrix or CI.

One gate proves or kills the thesis. If a single migrated gate does not become a
cache no-op, none of the rest is worth doing.

## Not Doing (and Why)

- **Test sharding** — the Google/Bazel first-order pattern is many small
  hermetic targets; `shard_count` mitigates tests that are already too big.
  These are single repo-scans, so sharding would **replicate the scan per
  shard** and make it worse.
- **Forcing hermeticity on all 47** — live-corpus gates are legitimately
  non-cacheable. Declaring the whole repo as inputs means any change
  invalidates every leg: zero cache value, large complexity. Mark them
  uncacheable instead.
- **Hand-rolling a content index for Class B** — that is ADR-0580's job.
- **Affected-set on raw paths first** — it degenerates to FULL for global gates.
  The face has to come first.
- **Local-loop cache optimisation** — measured worth nothing: pooled cycles were
  314.2s without the cache and 336.2s with it. The cache pays for **cold**
  consumers (99% hits in a fresh worktree), which is CI runners, not warm slots.

## Open Questions

1. **Is the face producer itself declarable?** `materialize_faces` is 140.8s and
   outside the graph. If it cannot become a proper buck2 target with declared
   inputs, consumers inherit its non-hermeticity and the whole idea stalls.
2. **What does a stale face cost?** A snapshot-reading gate is confidently wrong
   when the snapshot lags. Does the freshness gate already cover this, or does
   this shift the risk rather than remove it?
3. **Does ADR-0580's corpus substrate already specify the Class A face too?** If
   so this is an implementation of existing design, not a new one — which is the
   repo's stated preference and would change how it lands.
4. **Do the 14 gates already on scm-facts get cache no-ops today?** If they do
   not, the thesis is already falsified and this is measurable in minutes.
