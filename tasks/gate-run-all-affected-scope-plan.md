# gate-run-all-affected-scope — ordered task plan

## Objective

Add `--affected [--base <ref>]` to `oya gate run-all` so that presubmit runs
execute only the governance lanes that are transitively triggered by the diff,
reusing the already-landed `verify_affected::changed_files()` +
`oya_governance_gate_catalog_domain::lanes_for_changed()` infrastructure.

`--ci-required` remains the authoritative whole-set trunk backstop; it always
runs the full `AGGREGATED_VALIDATE_LANES` set regardless of `--affected`.

## Edge cases / acceptance criteria

1. `--affected` without `--base` defaults base to `origin/dev`.
2. `--affected` with `--base <ref>` uses the supplied ref.
3. `--ci-required` is mutually non-conflicting with `--affected`; when both are
   supplied, `--ci-required` forces the full lane set (trunk backstop wins).
4. `--affected` with zero changed files (clean branch vs base) runs the full
   catalog (conservative: `lanes_for_changed(&[])` already does this).
5. Unknown flags remain rejected with an error.
6. The narrowed run emits `[gate run-all] affected mode: N lanes selected (M total)`.
7. Unit tests cover:
   - `--ci-required` selects the full `AGGREGATED_VALIDATE_LANES` set.
   - `--affected` with a sample diff narrows to the expected subset.
   - `--affected --base custom-ref` parses `base` correctly.

## Subtasks (ordered)

1. [x] Write `tasks/gate-run-all-affected-scope-plan.md` (this file).
2. [x] Write `docs/specs/task-gate-run-all-affected-scope.md`.
3. [ ] Extend `RunAllArgs` + `parse_run_all_args` with `affected: bool` + `base: String`.
4. [ ] Thread `--affected` logic into `run_all_gates`: call `changed_files()` +
       `lanes_for_changed()` to compute the active lane list; skip this when
       `--ci-required`.
5. [ ] Add unit tests (red → green cycle).
6. [ ] `cargo check -p oya-dev-cli --all-targets` → clean.
7. [ ] `cargo nextest run -p oya-dev-cli` → green.
8. [ ] Self-review (correctness / arch / security / perf / cloud-native).
9. [ ] Simplify pass; re-run nextest.
