# Worker brief — G011 freshness-gate lane (one worker, one PR)

THE SPEC IS LAW: read `/Users/jasonlee/Developer/oyatie/.omc/ultragoal/SPEC-G011-freshness-gate.md` FIRST and follow it exactly. This brief is the executable summary.

Work ONLY in `/Users/jasonlee/oyatie-worktrees/g011-freshness-gate` (branch `agent/g011-freshness-gate`, base dev @ 5aaa68ab4). NEVER touch `/Users/jasonlee/Developer/oyatie` (main checkout). Never run omc orphan-cleanup.

## Deliverables (one PR)
1. New crate `cloud/cloud-ci/gates/oya-cloud-ci-freshness-app`:
   - Lock-freshness (pure Rust, no cargo, no network): resolve members via `libs/oya-workspace-members-kernel`; compare member `[package]` name+version vs root `Cargo.lock` `[[package]]` entries. Violation codes: `lock_missing_member_package`, `lock_stale_member_version`, `lock_orphan_path_package`.
   - Face-freshness: rematerialize via the SAME buck2 targets `infra/ci/materialize-cloud-ci-generated-faces.sh` uses; byte-diff the 4 committed faces. Violation code: `generated_face_stale`.
   - Failure output includes the remediation commands verbatim: `cargo metadata >/dev/null` (lock) and `infra/ci/materialize-cloud-ci-generated-faces.sh .` (faces).
2. dev-cli: new `freshness_gate.rs` registered in the `gate run-all` ci-required aggregator (mirror an existing gate module's shape; keep `libs/oya-check-pre-push` contract green).
3. CI: one fast job in `.github/workflows/oya-ci-required.yml` (no needs-edges; add to the `oya-ci-required` rollup needs list like the other jobs).
4. `docs/decisions/ADR-0539-*.md` citing FRIC-1781082000 + FRIC-1781062100 + ADR-0538 + enforcement layering (local = bridge, CI = canonical).
5. Registrations: `oya-ci.toml`, `libs/oya-ci-config/src/bundled/gate-disposition.json`, `docs/oya-ci/gate-catalog.md`, BUCK targets.
6. TDD: GREEN fixture + RED fixture per violation code (mirror workspace-glob-coverage-app test shape).

## Rules
- buck2 build + buck2 test on every affected target = the green signal; cargo supplementary only; refresh lock ONLY via `cargo metadata >/dev/null`. Your new crate is auto-membered by the `cloud/cloud-ci/gates/*` glob — zero root Cargo.toml edits.
- Never hand-edit `*.generated.json` — if the gate registry changes them, regenerate via the materialize script.
- Pre-existing local buck2 REDs are FRIC-009 class — re-verify against your clean base before attributing.
- SSH-signed commits; `git push -u origin agent/g011-freshness-gate`; open PR to dev with `gh`, body citing ADR-0539 + FRIC-1781082000 and listing buck2 evidence.
