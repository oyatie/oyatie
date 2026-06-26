# CI velocity backlog from triage evidence

Status: backlog / implementation handoff
Date: 2026-06-26
Owner lane: worker-3 CI velocity triage follow-up
Allowed write scope for this lane: this `docs/issues/**` artifact only

## Problem

The active CI stack is correctly fail-closed, but task 7 triage showed that even
small pull requests still pay the full slow-path cost. The goal is to reduce CI
latency and wasted runner work without weakening any gate: every optimization must
preserve universal, hermetic, productized, cloud-native CI semantics while GitHub
Actions remains the transitional execution substrate and owned CI remains the
north star.

## Evidence from task 7

Snapshot: 2026-06-26T08:42:13Z UTC.

- PR #889 was green after the generated-artifact false-green fix, but still spent
  10m20s in `buck2`, 8m32s in `gate-live-postgres`, 5m27s in `freshness`, 5m12s
  in `affected-set`, and 3m24s in `registry-drift`.
- PR #894 touched one network file and was green, but still spent 9m59s in
  `buck2`, 8m17s in `gate-live-postgres`, 4m55s in `affected-set`, 4m27s in
  `freshness`, and 3m30s in `registry-drift`.
- PR #891 was fail-closed by `gate · generated-artifact-control-plane` because
  `docs/architecture/product-graph.html` was declared but not tracked; this is a
  correctness block, not a velocity problem to bypass.
- A superseded PR #893 run spent about 9m49s materializing a merge-base
  build-health baseline for affected-set FULL mode before cancellation, with 966
  actions still waiting. The same superseded run showed firewall/total-accounting
  failures for `.claude/BUCK`, `.codex/BUCK`, and `tools/hooks/BUCK` ownership and
  reachability, which were real fail-closed findings.
- Buck2 warm restore is not enough evidence of warmth: the green PR #894 run
  restored an approximately 5.95GB `buck-out` cache but still took about 10m, and
  multiple Buck2/materialization steps logged `Cache hits: 0%`.
- Generated volatile faces such as `scm-facts.generated.json`,
  `scm-volatile-facts.generated.json`, and reorg move manifests changed during
  materialization, then broad cloud-ci build/test work still proceeded.
- `gate-live-postgres` passing logs include expected PostgreSQL RLS violation
  `ERROR` lines; failure classifiers must not treat those expected test probes as
  red by string match alone.

## Non-negotiable constraints

- Do not weaken `oya-ci-required`; it remains fan-in over constituent gates.
- Do not make a narrower, path-filtered, or cached check a substitute for a gate
  that is required today.
- Missing, stale, malformed, or provenance-mismatched cache/baseline evidence
  falls back to the current safe work or goes red; it never produces green.
- Generated artifacts remain controller/materializer-owned; no hand edits to
  `*.generated.json` and no manual generated merge surfaces.
- CI must stay universal and hermetic: skip decisions require machine-readable
  proof artifacts, not unreviewed path globs or human assertions.
- GitHub Actions is only the transitional runner substrate; designs should keep
  the owned CI control plane and cloud-native execution model as the target.

## Backlog slices

### Slice A — cache telemetry and hit-rate authority

Problem: large `buck-out` restores can coexist with `0%` action-cache hits, so
cache downloads are not reliable warmth evidence.

Deliverables:

1. Emit one stable cache telemetry artifact per Buck2 job with restore state,
   primary key, matched key if available, invocation-record counters, hit rate,
   action-cache hit count, and build result.
2. Classify restore state as exact primary-key hit, prefix fallback, or miss.
3. Treat `restored buck-out + 0% action hits` as cold/mis-keyed for velocity SLOs;
   keep build correctness separate from warmth claims.
4. Surface the classification in job summaries so future triage does not require
   manually grepping logs.

Acceptance criteria:

- A restored cache blob with `cache_hit_rate == 0.0` cannot satisfy a warm-cache
  claim.
- Missing invocation-record counters are red for warm-mode assertions.
- Prefix fallback restores are reported separately from exact primary-key hits.
- Green builds may still pass cold, but they cannot be counted as warm successes.

### Slice B — affected-set baseline acceleration

Problem: affected-set FULL mode can spend many minutes locally materializing the
merge-base build-health baseline, and stale runs can burn that time after a newer
head supersedes them.

Deliverables:

1. Derive the affected-set plan before any merge-base baseline work.
2. For FULL mode only, look up a trusted `dev` build-health baseline artifact by
   exact merge-base SHA and toolchain identity.
3. Verify producer provenance: event is trusted `push`, branch is `dev`, producer
   SHA equals the PR merge-base, artifact/report digests are recorded, and the
   producing run satisfied the required CI contract.
4. Fall back to the current same-root merge-base baseline build when no exact
   trusted artifact exists.
5. Cancel or short-circuit stale affected-set baseline work when a newer commit
   supersedes the run before the baseline can affect fan-in.

Acceptance criteria:

- Non-FULL affected-set decisions do not fetch or build a merge-base baseline.
- FULL decisions with exact trusted artifacts use them and record provenance.
- Wrong-event, wrong-branch, wrong-SHA, expired, or malformed artifacts are
  refused and do not turn CI green.
- Stale-run cancellation is fail-closed and cannot hide a failing required gate on
  the latest head.

### Slice C — volatile generated-face isolation

Problem: volatile SCM/reorg generated faces can change during CI materialization
and invalidate broad cloud-ci work even for unrelated pull requests.

Deliverables:

1. Inventory volatile generated faces that change per run or per checkout rather
   than per source decision.
2. Separate volatile facts from stable generated control-plane inputs where the
   Buck graph allows it.
3. Key volatile inputs explicitly, or move them out of broad source dependency
   cones, so unrelated gates do not rebuild only because SCM facts changed.
4. Keep freshness and generated-artifact gates authoritative for any committed
   generated output.

Acceptance criteria:

- Volatile face isolation cannot mask stale committed generated artifacts.
- Freshness, registry-drift, and generated-artifact-control-plane checks remain
  required where they are required today.
- The optimization has a before/after metric showing fewer invalidated actions or
  lower runtime for an unrelated single-file PR.

### Slice D — path-aware live-postgres and shard proof

Problem: unrelated changes still run the full live Postgres durable suite, but the
suite is a required correctness gate for tenant/RLS/CDC/SCIM invariants.

Deliverables:

1. Split live-postgres work into named shards with explicit invariant ownership.
2. Produce a machine-readable skip-proof artifact when a shard is not run for a PR
   path set.
3. Keep a conservative full-suite fallback when the proof is missing, ambiguous,
   or touches shared durable abstractions.
4. Teach failure classifiers to distinguish expected RLS violation probes inside
   passing tests from actual job failures.

Acceptance criteria:

- A skipped shard has a proof artifact naming the inputs, invariant owner, and
  reason it is unaffected.
- Missing or ambiguous proof runs the shard instead of skipping it.
- Any durable adapter/facade/core change still runs the relevant live shard or the
  full suite.
- Expected Postgres `ERROR` probes are not treated as failures without the test
  process failing.

### Slice E — stale-run cancellation and queue hygiene

Problem: superseded runs can continue expensive baseline/build work after a newer
head exists, wasting runner time and delaying useful feedback.

Deliverables:

1. Add a stale-head guard before expensive CI phases: baseline materialization,
   broad Buck2 build/test, live-postgres, and generated-face materialization.
2. Mark stale cancellation distinctly from test failure in summaries and fan-in.
3. Ensure the latest head still receives every required gate; cancellation of an
   older head cannot contribute green status to the current PR.

Acceptance criteria:

- Superseded runs stop before expensive work when a newer head is visible.
- Canceled stale runs do not produce green required contexts for the current head.
- Latest-head required checks remain complete and fail-closed.

## Suggested disjoint team lanes

- Lane 1: cache telemetry artifact and summary renderer.
- Lane 2: trusted affected-set baseline lookup and provenance verifier.
- Lane 3: stale-run guard around expensive workflow phases.
- Lane 4: volatile generated-face dependency inventory and isolation plan.
- Lane 5: live-postgres shard/skip-proof design and classifier hardening.

Each lane can work independently if it preserves the no-gate-weakening constraints
above and reports explicit before/after timing evidence.
