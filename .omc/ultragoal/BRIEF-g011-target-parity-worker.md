# Worker brief — G011 target-parity gate lane (one worker, one PR)

THE SPEC IS LAW: read `/Users/jasonlee/Developer/oyatie/.omc/ultragoal/SPEC-G011-target-parity-gate.md` FIRST and follow it exactly.

Work ONLY in `/Users/jasonlee/oyatie-worktrees/g011-target-parity` (branch `agent/g011-target-parity`, base dev @ 2705d1c96). NEVER touch `/Users/jasonlee/Developer/oyatie` (main checkout). Never run omc orphan-cleanup.

Deliverables (one PR): producer `target-parity` face rows in oya-cloud-ci-accounting-registry-app; pure gate crate `cloud/cloud-ci/gates/oya-cloud-ci-target-parity-app` with codes `member_missing_buck` (born-blocking, empty baseline) and `member_test_code_without_rust_test_target` (baseline-block-on-new, freeze the ~634 current keys — derive the exact set mechanically from the face, never hand-curate); remediation text per spec; matrix line; BUCK; registrations (oya-ci.toml, oya-ci-config disposition + count, gate-catalog, firewall meta-test); ADR-0540; GREEN/RED/baseline fixtures; faces regenerated ONLY via `infra/ci/materialize-cloud-ci-generated-faces.sh .`.

Rules: buck2 build + buck2 test = the green signal (cargo supplementary); lock refresh ONLY via `cargo metadata >/dev/null`; zero root Cargo.toml edits (gates/* glob auto-members your crate); SSH-signed commits; `git push -u origin agent/g011-target-parity`; open PR to dev with `gh` citing ADR-0540 + FRIC-1781063357 + measured debt numbers + buck2 evidence.
