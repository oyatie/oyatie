# CI trusted dev baseline and exact cache-hit plan

Status: design handoff for implementation
Date: 2026-06-26
Owner lane: worker-3 cache/baseline design
Allowed write scope for this lane: `docs/issues/**` only

## Problem

The current CI design has the right trust direction, but the next code change must
separate two concepts that are easy to conflate:

1. **Trusted dev build-health baseline artifact.** A baseline report may be reused
   only when it was produced by the trusted `dev` push pipeline for the exact
   merge-base commit that the pull request is comparing against.
2. **Exact cache hit.** A restored multi-GB `buck-out` blob is not evidence of
   Buck2 warmth by itself. A prefix fallback restore can download gigabytes and
   still yield `0%` action-cache hits. Treat that state as a cold/mis-keyed run
   for SLO and warm-read decisions.

The design goal is faster CI without weakening gates: a missing, stale, or
ambiguous baseline/cache signal must fall back to the current safe work, not make
`oya-ci-required` green with less evidence.

## Current evidence

- `.github/workflows/oya-ci-required.yml:396-402` defines the intended split:
  trusted `dev` push is the sole full-graph `buck-out` writer; PRs restore
  read-only.
- `.github/workflows/oya-ci-required.yml:441-448` and `:576-583` restore
  `buck-out` by stable dependency/toolchain key with a restore prefix. The prefix
  fallback is useful for disk/locality experiments but is not an exact-hit proof.
- `.github/workflows/oya-ci-required.yml:595-613` computes the current PR
  merge-base build-health baseline in the same working tree and explicitly keeps
  the candidate tree out of the baseline source.
- `.github/workflows/oya-ci-required.yml:658-682` passes that baseline only to
  the affected-set binary's PR `auto` mode; integration tiers run hard full mode.
- `.github/workflows/oya-ci-required.yml:683-700` saves `buck-out` only on
  `push` to `refs/heads/dev` after the full affected-set job has populated it.
- `.github/workflows/oya-ci-required.yml:701-717` already uploads
  `build-health-baseline-${{ github.sha }}` from trusted push-to-dev runs as a
  producer-only artifact.
- `.github/workflows/oya-ci-required.yml:931-977` keeps `oya-ci-required` as pure
  fan-in: every constituent gate must pass.
- `specs/cache-warmth-policy.json:14-24` says warm reads are licensed only when
  the class is warm-eligible and the cold integrity canary is green; red canary
  suspends all warm reads.
- `specs/cache-warmth-policy.json:30-35` makes unlisted build classes cold and
  non-cacheable by default.
- `specs/cache-warmth-policy.json:67-72` classifies the trusted affected cone as
  warm-eligible, and `:85-90` identifies post-merge `dev` as the canonical cache
  populator.
- `specs/cache-warm-license.json:6-8` currently keeps warm reads unlicensed
  because no live CAS endpoint has produced a green canary verdict.
- `cloud/cloud-ci/gates/oya-cloud-ci-cache-wiring-app/src/lib.rs:293-310`
  defines the structured cache-hit report fields. `:313-403` already contains the
  important fail-closed guard for warm modes: non-success build result, missing
  record fields, `0%` hit rate, or zero action-cache hits are failures.
- `.github/workflows/cache-integrity-canary.yml:1-20` states the canary is the
  no-cache trust anchor; `:86-97` builds from empty and hashes outputs; `:118-125`
  emits the canary verdict.
- `docs/decisions/ADR-0563-rename-aware-path-keyed-ci-baseline-relabel.md:39-45`
  records that some frozen CI baselines are path-keyed and do not follow renames by
  default; `:78-82` requires relabel failures to stay fail-closed.
- `docs/decisions/ADR-0565-zero-graphql-in-the-owned-api-surface.md:145-158`
  is a counterexample to over-generalizing baseline reuse: that gate evaluates the
  candidate tree directly with an empty frozen baseline, so this plan is only for
  affected-set FULL-tier build-health ratchets.

## Trust model

### Producers

A reusable build-health baseline artifact is trusted only when all of these are
true:

1. The producer run event is `push`.
2. The producer branch is exactly `refs/heads/dev`.
3. The producer run's head SHA is exactly the PR merge-base SHA being evaluated.
4. The producer run reached the same `oya-ci-required` fan-in contract; artifact
   provenance is tied to the successful run identity, not just to an artifact
   name string.
5. The artifact path is the build-health admission report emitted by the
   affected-set full mode, currently
   `${RUNNER_TEMP}/build-health-admission-report.json` before upload.
6. The artifact is immutable for the consuming run: consumer records producer run
   id, producer head SHA, artifact id/name, downloaded digest, and report digest.

### Consumers

A PR consumer may use the trusted dev baseline artifact only for the FULL-tier
build-health ratchet, and only after it has decided that FULL-tier evidence is
needed. It must not fetch or build a baseline before the affected-set plan is
known.

Consumer decision order:

0. Confirm the consuming gate is the affected-set FULL-tier build-health ratchet;
   candidate-tree gates must keep evaluating the candidate directly.
1. Derive the affected-set plan from the PR diff.
2. If the plan stays cone-binding, run the cone build/test and skip baseline
   artifact fetch/build.
3. If the plan escalates to FULL:
   - compute the exact merge-base SHA;
   - look up a trusted `dev` push artifact for that exact SHA;
   - verify the producer provenance fields above;
   - if verified, pass the downloaded report to `--baseline-report`;
   - if absent, expired, malformed, or provenance-mismatched, fall back to the
     current same-root merge-base baseline build.

This fallback preserves today's safety and only changes latency when an exact
trusted artifact is available.

## Exact cache-hit rules

### GitHub `buck-out` cache bridge

The interim GitHub cache bridge may report three distinct states:

| State | Meaning | Gate implication |
|---|---|---|
| Exact primary-key restore | Restored key equals `buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-${{ hashFiles('.buckconfig', 'toolchains/BUCK', 'Cargo.lock') }}`. | May be labeled exact `buck-out` restore. Still verify Buck2 invocation counters before claiming warmth. |
| Prefix fallback restore | Restore matched only `buck-out-${{ runner.os }}-${{ hashFiles('rust-toolchain.toml') }}-`. | Useful diagnostic only. Must not count as cache warmth or latency success; expect possible `0%` Buck2 hits. |
| Miss | No `buck-out` restored. | Cold run. Green is allowed only if the actual build/test evidence passes. |

Implementation note for the future workflow/code lane: name the restore step and
record both the primary key and the restore output (`cache-hit` or equivalent).
If the API cannot expose the matched key, treat any non-exact signal as fallback.

### Buck2 action-cache warmth

A run is an exact warm-cache hit only when the Buck2 structured invocation record
proves all of the following for the relevant build class:

1. `exit_result_name == "SUCCESS"`.
2. `last_snapshot.re_action_cache_started > 0` when mode is warm.
3. `cache_hit_rate > 0.0` when mode is warm.
4. `run_action_cache_count > 0` when mode is warm.
5. Required counters are present; missing shape is a failure, not zero.
6. The run mode matches the resolver output for the class:
   - current GitHub bridge is `bypass` until the license flips;
   - future CAS reader is `warm-ro`;
   - trusted dev writer is `warm-rw`.

A downloaded cache blob with `cache_hit_rate == 0.0` is a mis-keyed or cold run,
not a success. It may pass the build, but it must not satisfy warm-cache SLOs or
justify removing cold fallback paths.

## Implementation slices

### Slice A — baseline artifact consumer

Scope for implementation lane: affected-set app plus narrowly required workflow
wiring, not this documentation-only lane.

1. Add a trusted-baseline lookup that takes `(base_ref, merge_base_sha)` and
   returns either a verified local report path plus provenance metadata, or a
   typed unavailable reason.
2. Bind lookup to successful `push` runs on `dev`; never trust artifacts from
   `pull_request`, `merge_group`, `workflow_dispatch`, forks, or artifact name
   alone.
3. Validate downloaded report shape before passing it to `--baseline-report`.
4. If the PR includes a rename-aware move plan or any path-keyed baseline surface,
   preserve the existing ADR-0563 fail-closed relabel rules; do not treat the
   build-health baseline artifact as a generic path-keyed debt baseline.
5. Preserve current same-root baseline as fail-closed fallback.
6. Emit provenance in the job summary/artifact: merge-base SHA, producer run id,
   artifact id/name, downloaded digest, and fallback reason if not used.

### Slice B — exact cache-hit classification

1. Capture the restore state for each `buck-out` restore step: exact primary-key
   hit, prefix fallback, or miss.
2. Extend the cache-hit report or adjacent summary with `restore_state`,
   `primary_key`, and `warmth_claim`.
3. Keep the existing Buck2 invocation-record checks as the source of truth for
   actual action-cache warmth.
4. When `CACHE_MODE=bypass`, report telemetry but do not require hits.
5. When mode becomes `warm-ro` or `warm-rw`, keep the current strict guard: zero
   hit rate, zero action-cache hits, missing counters, or non-success result are
   RED.

### Slice C — bring-up/license sequence

1. Keep `specs/cache-warm-license.json` false while no live CAS endpoint is
   reachable and no canary has produced a green verdict.
2. Populate CAS from a trusted `dev` writer run after `oya-ci-required` is green.
3. Run the cold integrity canary and require a GREEN verdict with at least one
   compared key and zero divergent keys.
4. Flip the license true only in a reviewed change that cites the green canary
   run and preserves the red response.
5. On any red canary, flip the license false before eviction/remediation work;
   do not keep serving non-divergent warm hits while red stands.

## Acceptance criteria

- A PR that does not need FULL affected-set escalation does not build or fetch a
  merge-base baseline.
- A FULL-tier PR with a matching trusted `dev` artifact uses it and records
  provenance.
- A FULL-tier PR with no exact trusted artifact falls back to the same-root
  merge-base baseline and remains safe.
- An artifact from the wrong event, branch, run conclusion, or SHA is refused.
- Prefix fallback `buck-out` restore is reported separately from exact primary
  key restore and never counted as warmth.
- A warm-mode run with `0%` hit rate or zero action-cache hits is red.
- Missing invocation-record counters are red in warm/cold assertions.
- The canary remains cold: no cache restore, no overlay, no action-cache hits, no
  remote execution, and no upload attempts.
- `oya-ci-required` remains fan-in only; no narrower substitute check can turn it
  green.
- Candidate-tree gates that intentionally do not consume merge-base baselines keep
  their direct evaluation semantics.

## Non-goals for this lane

- No edits to `.github/workflows/oya-ci-required.yml`.
- No edits to `cloud/cloud-ci/gates/oya-cloud-ci-affected-set-app/**`.
- No edits to cache-wiring code or policy JSON.
- No hand edits to any `*.generated.json`.
- No new dependencies.
- No attempt to make the build-health baseline artifact a universal baseline for
  path-keyed debt, generated-artifact, or candidate-tree recurrence gates.

## Handoff summary

The safe next implementation move is to make affected-set derive its plan before
baseline work, then consume a trusted `dev` baseline artifact only for exact
merge-base FULL-tier ratchets. In parallel, classify cache restores as exact,
fallback, or miss, and keep Buck2 invocation-record counters as the authority for
whether an actual action-cache hit happened.
