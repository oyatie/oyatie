# G004 next wave brief — manifest-hygiene + cross-artifact malformed-input hardening

Baseline gate before writes: **do not edit until leader confirms `origin/dev` after PR #950/#951 has no failed post-merge suites/check-runs**. Current required check-runs are green; broader suites may be queued.

## Completed previous wave evidence
- PR #950 cargo-prefix: merged 2026-06-27T19:22:56Z, head `fca4752e05d1793ba439c8d0f76a33931c0fe8c1`, merge `ba4f6347905ac6c31cde941d5383741ddd0318a1`, PR checks 41/41 success.
- PR #951 BNF layer-suffix: merged 2026-06-27T19:22:44Z, head `4248c14163451dacc38bbeb985df759dee95bb76`, merge `6942ece1cf598da91d5f5f34778c6304654a9379`, PR checks 41/41 success.
- Disposable materialized gate checks passed for both, with generated side effects confined to throwaway worktrees.
- Prior team `g004-cloud-ci-false-g-1fb6d50c` cleaned up after all tasks completed.

## Write concurrency
- Max two writers + one read-only reviewer.
- One writer per app subtree.
- No shared producer, workflow, root policy, generated face, or `.omx/ultragoal` edits.
- No new dependencies.
- Rust + Buck2 authoritative; no Cargo evidence as merge authority.

## Worker A — manifest-hygiene lane
Owned path: `cloud/cloud-ci/gates/oya-cloud-ci-manifest-hygiene-app/**`

Finding from read-only map:
- `src/lib.rs:87-93`: `rows` missing/non-array becomes `[]` via `unwrap_or_default()`.
- `src/lib.rs:94-97`: rows without string `crate_name` are skipped.
- `src/lib.rs:214-217`: test currently locks `empty_corpus_is_green`.

Minimal fix shape:
- Add one public structural code, e.g. `manifest_hygiene_malformed_input`.
- Fail closed for missing rows, non-array rows, empty rows, malformed/non-object row or missing string `crate_name`.
- Keep existing per-field hygiene codes unchanged.
- Stable sentinel keys: `<missing-rows>`, `<non-array-rows>`, `<empty-rows>`, `<malformed-row-0>` or equivalent deterministic row-index key.

Targets:
- `//cloud/cloud-ci/gates/oya-cloud-ci-manifest-hygiene-app:oya-cloud-ci-manifest-hygiene-app-unittest`
- `//cloud/cloud-ci/gates/oya-cloud-ci-manifest-hygiene-app:oya-cloud-ci-manifest-hygiene-app`
- Gate test only after materializer in disposable/cleaned worktree: `//cloud/cloud-ci/gates/oya-cloud-ci-manifest-hygiene-app:oya-cloud-ci-manifest-hygiene-app-gate`

## Worker B — cross-artifact-agreement lane
Owned path: `cloud/cloud-ci/gates/oya-cloud-ci-cross-artifact-agreement-app/**`

Finding from read-only map:
- `src/lib.rs:156-160`: `decisions` missing/non-array becomes `[]`.
- `src/lib.rs:167-170`: malformed decision rows or missing/non-string `id` skipped.
- `src/lib.rs:233-244`: missing/non-object/empty `generated_face_axes` emits nothing.
- `src/lib.rs:251-257`, `293-312`: missing fields/arrays can become empty/false and go quiet.
- Caveat: code emits/test references `decision_id_mismatch`, but it is not listed in `VIOLATION_CODES`; do not broaden unless needed for the malformed-input fix.

Minimal fix shape:
- Add one public structural code, e.g. `malformed_cross_artifact_payload`.
- Add small local shape validation helper called at start of `evaluate_keyed()`; no schema crate.
- Fail closed for missing/non-array/empty `decisions`, malformed decision row/missing `id` or `status`, non-array supporting arrays, non-object generated faces/statuses.
- Keep existing valid-payload behavior unchanged.
- Adjust existing tests that assumed empty/partial payload is quiet by adding one valid decision row where needed.

Targets:
- `//cloud/cloud-ci/gates/oya-cloud-ci-cross-artifact-agreement-app:oya-cloud-ci-cross-artifact-agreement-app-unittest`
- `//cloud/cloud-ci/gates/oya-cloud-ci-cross-artifact-agreement-app:oya-cloud-ci-cross-artifact-agreement-app`
- Gate test only after materializer in disposable/cleaned worktree: `//cloud/cloud-ci/gates/oya-cloud-ci-cross-artifact-agreement-app:oya-cloud-ci-cross-artifact-agreement-app-gate`

## Worker C — read-only reviewer/verifier
- No source edits.
- Review Worker A/B diffs for fail-closed behavior, stable public violation codes, no generated/shared/.omx edits, simplicity, security/perf regressions, and PR body evidence.
- Verify targeted tests/builds when feasible.

## Done per lane
1. RED regression captured.
2. Minimal fix only.
3. `rustfmt --check` on touched Rust file(s).
4. Targeted Buck2 unittest + app build pass.
5. Disposable materialized gate pass or explicit generated-face caveat.
6. `git diff --check`; no tracked `*.generated.json` diff.
7. Independent review approval.
8. PR to `dev` with `## Issue`, `## Summary`, `## Verification`, `## Traceability`, `## Evidence`, `## Code Review`.
9. PR checks green, merge, post-merge evidence, cleanup.
