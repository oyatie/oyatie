# `gate · affected-set` wall-clock profile (measured 2026-08-08)

Measurement record for bead `oyatie-zng`. Every number below was read off a completed
`oya-ci-required` run via `gh api .../jobs`, or off the job log; none is estimated.
**No code changed.** The conclusion is that the FULL-tier work is irreducible inside this
package, and the one lever that does not shrink coverage lives outside it.

## 1. The job is trimodal, not a single number

`gate · affected-set` duration over the 7 most recent completed successful runs
(2026-08-07T23:05Z .. 2026-08-08T11:27Z):

| run | event / branch | job | `Binding affected-set build + test` | decision |
|---|---|---|---|---|
| 31226011563 | PR `evidence/completion-packets` | 6.0m | 0s | NO-GRAPH-TARGETS |
| 31255030523 | PR `fix/modeled-crypto-not-callable` | 8.5m | 179s | cone (no escalation) |
| 31241020054 | push `dev` | 26.2m | 1331s | FULL (push tier) |
| 31254973893 | push `dev` | 27.9m | 1424s | FULL (push tier) |
| 31253758836 | PR `gate/adr-citation-closure` | 28.9m | 1392s | FULL (escalated) |
| 31249079674 | PR `gate/workflow-lane-preflight` | 29.1m | 1393s | FULL (escalated) |
| 31248740033 | PR `gate/adr-citation-closure` | 29.1m | 1409s | FULL (escalated) |

FULL runs (n=5): binding step median 1393s, range 1331–1424 (±3.3%). The bead's 29.1/29.4-minute
figure is correct **for a FULL run** and is not steady state across event classes.

## 2. Inside a FULL run, one command is 86% of it

Phase telemetry, two independent FULL runs:

| phase | 31248740033 | 31253758836 |
|---|---|---|
| `buck2 build //... --keep-going --build-report` | 1201s | 1193s |
| `buck2 test //... --keep-going` (test-health ratchet) | 207s | 198s |
| binding step total | 1409s | 1392s |

Both runs report the identical action count for the head build:

```
Cache hits: 0%
Commands: 14362 (cached: 0, remote: 0, local: 14362)
```

`cached: 0, remote: 0` on every buck2 invocation in this job — the derive preflight
(`Commands: 4`, `Commands: 2`, `Commands: 454`), the face materializer (`Commands: 220`),
and the head build (`Commands: 14362`). The job starts from an empty runner-local `buck-out`
by design (ADR-0554 D10, `.github/workflows/oya-ci-required.yml:554-560`).

## 3. Fixed floor, paid by every run including the 6-minute one

`Materialize cloud-ci generated faces (out-of-graph boundary)`: 200, 216, 217, 217, 219, 220,
227s across the same 7 runs (median 217s). It is a `buck2 run` of a single binary that executes
220–454 cold local actions. Plus checkout ~20s, rustup ~9s, merge-base baseline 0–98s. Floor
≈ 250–350s regardless of the decision — which is essentially the whole 6.0m docs-only run.

## 4. Five candidate causes, all refuted by the measurement

| candidate | verdict | evidence |
|---|---|---|
| full-corpus walk where incremental would do | refuted | `phase=derive-affected-set-tier ... elapsed_seconds=0` |
| recomputing a graph that could be cached | refuted | derive is 0s; the cost is executing actions, not deriving them |
| O(n·m) join over the target set | refuted | same; the gate binary's own runtime is sub-second |
| a second full test pass inside one job | refuted | `buck2 test //...` is 198–207s because it reuses the build already in `buck-out` |
| cold merge-base rebuild (11m12s–17m48s) | refuted | `build-health: trusted merge-base baseline pair REUSED from run 31246463235 ... the cold merge-base rebuild is skipped` — 86–98s on PRs, 0s on pushes |
| `[]` affected set doing FULL-tier work for the wrong reason | refuted | the fast runs resolve `decision=NO-GRAPH-TARGETS` naming all 7 changed files, licensed by `inert_selection_classes`; the anti-vacuity predicate is live (`affected-set-policy.json:252`) |
| **cold buck2 with no warm cache** | **confirmed** | `Cache hits: 0%`, `remote: 0` on 14362 actions |

## 5. Why FULL fires so often, and why that must not be narrowed

Run 31248740033 escalated for two reasons, both correct:

```
- buildfile `governance/check/adr-citation-closure/BUCK` changed (blast radius exceeds its own package)
- unowned path `Cargo.lock` has no buck2 owner and no synthetic-dependency declaration (derivation uncertainty)
```

Any PR that adds a crate produces both. The buildfile rule (`src/lib.rs:589-601`) is sound: a
new or edited BUCK file adds and removes targets that dependents resolve, so a head-only
`rdeps()` cone cannot bound it. Narrowing either trigger buys wall clock by checking less.
**Not doing it.** The tier decision is not the defect; the cold cache is.

## 6. The lever, and the gap nobody has closed

Filesystem snapshotting of `buck-out` is closed by decision — a 6.37 GB `actions/cache` archive
crossed the owned node's ephemeral-storage eviction threshold on 2026-08-01
(`oya-ci-required.yml:554-560`). The sanctioned route is a Buck2-aware remote action cache + CAS,
and most of it already exists:

- `infra/ci/buckconfig/warm-cache-ro.buckconfig` / `warm-cache-rw.buckconfig` carry complete
  NativeLink wiring (grpc endpoints, `instance_name = main`, `tls = true`,
  `remote_cache_enabled = true`, cache execution platform). Shipped dark.
- `specs/cache-warm-license.json` ships `warm_reads_licensed: false`; ADR-0556 D2 is an IFF —
  warm-eligible class AND the most recent scheduled cold integrity-canary GREEN. The resolver
  refuses every warm mode until then. **This is a blocking precondition, not a footnote.**
- **The gap:** `oya-cloud-ci-cache-wiring-bin` has 12 call sites — 9 in
  `.github/workflows/cache-integrity-canary.yml`, 3 in `oya-ci-required.yml` (:587, :604, :606),
  all three inside the `buck2` job (:502-626, `CACHE_BUILD_CLASS: untrusted-author-presubmit`).
  The `gate-affected-target-set` job (:650-1026) **never calls the resolver.** Licensing warm
  reads tomorrow would not move this job by one second.

Ordered work, none of it in this package. **Step 0 is an authorization gate, not engineering** —
steps 1–6 are the work a go-gate would authorize, never a substitute for it, and nothing below may
be switched on before it clears:

0. **Go-gate.** `docs/decisions/ADR-0700-ci-admission-live-apex.md` live hard norm 4 keeps warm
   CAS / RE activation **fail-closed** until an explicit go-gate: credentials (#1541), cache-only
   proof, and an **Accepted** activation ADR. That norm also says outright that apex gists
   mentioning `remote_enabled=true` are historical design, not activation authority — which is
   exactly what the dark `warm-cache-*.buckconfig` wiring quoted above is.
1. Stand up a NativeLink CAS reachable from the executing lanes.
2. Run the cold integrity canary to GREEN; record its run id in `cache-warm-license.json`.
3. **Issue and mount a cache identity for this lane.** `controlled_child` resolves its overlay
   through `effective_buckconfig`, which **rejects every warm mode** when
   `OYA_CACHE_TLS_CLIENT_CERT` is unset, empty or non-absolute
   (`ci/facade/build-cache-policy/src/lib.rs:230-245`), and `gate-affected-target-set` grants only
   `contents: read` / `actions: read` and mounts no secret (`oya-ci-required.yml:652-654`). Wiring
   the resolver in without this step either fails the required job or leaves it in bypass — no
   reuse either way. Fork PRs are handed no secret at all, so they must resolve to a declared
   cold/bypass class rather than to a broken warm one.
4. **Relicense the PR build class, as a reviewed policy edit.**
   `specs/cache-warmth-policy.json` pins `untrusted-author-presubmit` to `warmth: cold,
   cache_read: false, cache_write: false`, and its own reason text calls a read-only relaxation
   "a reviewed two-way policy edit". Read-only reuse on PRs is therefore not merely unwired, it is
   currently **prohibited by policy**. The write prohibition is one-way and stays.
5. Wire `gate-affected-target-set`'s build + test through `cache-wiring-bin` with its own build
   class (trusted-push for the dev-push producer, read-only untrusted-author for PRs, cold for
   forks). This is a `.github/workflows/oya-ci-required.yml` edit and is the piece that has never
   been written.
6. Re-measure. Do not quote a speedup before then.

What is measured about reuse, locally, on this package's own graph:

```
cold: Commands: 690 (cached: 0, remote: 0, local: 690)   Network: Down: 29MiB   Pass 7 Fail 0 Skip 0
warm: (no Commands line — zero actions executed)          Network: Down: 0B      Pass 7 Fail 0 Skip 0
```

That establishes the actions are stably keyed and re-usable across invocations — the
precondition for remote reuse. It does **not** establish the hit rate a remote CAS would reach
across runners, and no number for that is asserted here.
