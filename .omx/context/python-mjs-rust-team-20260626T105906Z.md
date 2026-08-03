# Python/MJS to Rust retirement team context

## Task statement
Replace, delete, or fence Python and MJS automation in Oyatie with Rust/Buck2/cloud-ci authority **only where the scripts are still valid and worth preserving**. Ponytail rule: deletion beats rewrite; do not port dead wrappers.

## Desired outcome
Multiple independent PRs against `dev`, each from an isolated worktree/branch, shrinking Python/MJS authority or preserving valid behavior in Rust with tests. Each teammate owns their slice end-to-end: plan, tests, build, review, fix, commit, push, PR. Leader integrates status only.

## Hard constraints
- Read `specs/root-hub-pointers.json` then `docs/AGENTS.md` in each worktree before editing.
- Use plain git on isolated worktree branches; PR target is `dev`.
- Never hand-edit `*.generated.json`.
- Final verification authority is Buck2/cloud-ci gates, not Cargo/Node/Python wrappers.
- CLI/local bridge surfaces are retirement-marked; do not promote `oya-dev-cli` or cargo-run wrappers as merge authority.
- No new dependencies without explicit user request.
- Ponytail: prefer deleting non-worth-preserving wrappers and removing policy exceptions over rewriting them.

## Current known work/evidence
- Existing branch/worktree to finish: `/Users/jasonlee/oyatie-worktrees/python-mjs-rust-20260626T105334Z` on `agent/python-mjs-rust-20260626T105334Z`.
- That slice deletes four root MJS shims already backed by Rust lint tests:
  - `scripts/asyncapi-lint.mjs`
  - `scripts/proto-lint.mjs`
  - `scripts/validate-adr-shape.mjs`
  - `scripts/validate-foundry-phase00-evidence.mjs`
- Verification already passed there before evidence JSON parse failed:
  - `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-rust-first-automation-hygiene-app:oya-cloud-ci-rust-first-automation-hygiene-app-gate` (9 passed)
  - `buck2 test //marketplace/facade/dev-cli:marketplace-dev-cli-lint-cli` (4 passed)
  - non-generated reference scan clean; policy scan clean; generated baselines untouched.
- Invalid evidence file was partially written in that worktree; fix JSON rather than recreating broad changes.

## Initial independent slices
1. Finish root MJS shim retirement branch above and open PR.
2. App-shell MJS scripts: inspect `oya/app-shell-frontend/scripts/*.mjs`; decide delete/fence/port only if still valid; likely larger frontend transition surface, avoid broad churn.
3. Root Python/Buck generators: inspect `scripts/emit_rust_tests.py`, `scripts/gen_first_party_buck.py`, `tools/buck2/gen-first-party-buck.py`, `tools/buck/apply-thirdparty-patches.py`; preserve only real generator behavior worth keeping.
4. Cloud Python validators: inspect `scripts/tests/cloud_*_check.py` and `cloud/cloud-k8s/tests/test_runtime_substrate_validation.py`; decide delete/fence/port with Buck2 tests.
5. Root doc-generator MJS scripts: inspect `scripts/generate-erp-second-pass-docs.mjs` and `scripts/generate-marketplace-workplace-doc-set.mjs`; delete if stale, otherwise design minimal Rust replacement with tests.
6. Integration/review lane: inventory remaining Python/MJS after worker PRs, verify no generated hand edits, review PRs for ponytail/deletion-first discipline.

## Unknowns/open questions
- Some remaining scripts may be vendor/bootstrap/local bridge only; do not rewrite unless they carry live valid behavior.
- Generated accounting baselines may reference deleted scripts; do not hand-edit them.
- Remote `oya-ci-required` and reviews will be pending after PR creation.
