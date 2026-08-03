# Worker brief — G011 rust_test wiring generator + batch-1 (one worker, one PR)

Context: ADR-0540 (merged) froze a 634-key baseline of workspace members that have Rust test code but NO `rust_test` target in their BUCK file — their tests never compile in CI (the PR #645 false-green class). This lane builds the burn-down AUTOMATION (founder doctrine: manual-twice = write the automation) and proves it on one batch. The mass campaign comes later; do NOT attempt all 634.

Work ONLY in `/Users/jasonlee/oyatie-worktrees/g011-test-wiring-gen` (branch `agent/g011-test-wiring-gen`, base = current origin/dev @ 16f2e3b54). NEVER touch `/Users/jasonlee/Developer/oyatie` (main checkout).

## Deliverables (one PR)
1. **Generator tool** `tools/oya-buck-test-wiring-app` (Rust, single-concern, local-bridge — retirement-marked like all CLI per repo policy, zero merge authority):
   - Enumerate members via `libs/oya-workspace-members-kernel` (REUSE — never re-parse the members array).
   - For each member with test code (tracked `tests/` dir or `#[cfg(test)]`/`#[test]` in `src/**/*.rs`) and no `rust_test` in BUCK: emit the missing `rust_test` stanza(s) APPENDED to the member's BUCK file — unit-test target for in-crate `#[cfg(test)]` (named `<crate>-unittest`) and one per `tests/*.rs` integration file if present (follow the naming + attrs conventions of existing BUCK files — STUDY `cloud/cloud-ci/gates/oya-cloud-ci-manifest-hygiene-app/BUCK` and 2-3 others FIRST and mirror exactly: deps from the existing rust_library target, srcs globs, env wiring only where genuinely needed).
   - Modes: `--list` (print candidates), `--apply --limit N [--root <subtree>]` (edit BUCK files), `--check` (exit 1 if candidates exist — future gate hook).
   - No unwrap/expect/panic in production, `#![forbid(unsafe_code)]`, BUCK + manifest hygiene for the tool itself, unit tests for stanza generation (golden-file fixtures).
2. **Batch-1 application:** run `--apply --limit 20 --root libs/` (the 20 alphabetically-first libs/ candidates). Then `buck2 test` every generated target. EXPECT some to fail — that is the point (latent uncompilable/broken tests):
   - Trivially fixable (missing dev-dep in BUCK deps, path issue): fix the BUCK stanza, not the test.
   - Genuinely broken test code: do NOT fix the production/test code in this PR — REVERT that member's BUCK change, record it in the PR body under "deferred-broken" with the compile error one-liner. Keep batch-1 = only members whose tests now COMPILE AND PASS via buck2.
3. **Baseline shrink:** regenerate the target-parity face so the wired members leave the frozen baseline (the gate's baseline-block-on-new must show the baseline SHRINKING — verify the gate stays green; if the baseline file requires explicit regeneration follow the producer path, never hand-edit).
4. PR body: cite ADR-0540 + FRIC-1781063357, list batch-1 members wired (N compiled+passing), deferred-broken list with errors, generator usage line, buck2 evidence.

## Rules
- buck2 build + buck2 test = the green signal; cargo supplementary only; lock refresh ONLY via `cargo metadata >/dev/null` (the tool crate is auto-membered by tools/oya-* glob; lock gains one package).
- SETTLE PROTOCOL (mandatory): all content commits FIRST → `git add` everything → run `infra/ci/materialize-cloud-ci-generated-faces.sh .` → FACES-ONLY settle commit. Never hand-edit `*.generated.json`.
- SSH-signed commits; push -u origin agent/g011-test-wiring-gen; open PR to dev with `gh`.
