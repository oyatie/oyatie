# Spec: G011 item 3 — cargo-buck2 target-parity gate (test-wiring false-green)

Status: ACTIVE · Story: G011 · Frictions: FRIC-1781063357 (PR #645 merged green with uncompilable tests), FRIC-008(b)
Verified facts as of dev @ 2705d1c96, 2026-06-10.

## Objective

CI never compiles the test code of most crates: the hermetic buck2 lane builds all *declared* targets and runs affected gate tests, and the cargo matrix tests only 9 gate crates — but a buck2 target that is never declared is never built. Measured on dev today (via oya-workspace-members-kernel expansion):

- 817 members; **all** have BUCK files (member_missing_buck debt = 0).
- **634 members have test code (`#[cfg(test)]`/`#[test]`/`tests/` dir) but NO `rust_test` target** → their tests never compile anywhere → uncompilable tests merge green (PR #645 sqlx-Debug class).
- 74 members have no test code and no rust_test (benign — do NOT flag).

After this lands: a born-blocking gate freezes today's 634 as baseline debt; any NEW crate (or newly added test code) without a compiled rust_test counterpart is impossible-to-ship. Burn-down of the 634 is explicitly OUT OF SCOPE (follow-on mass-wiring effort).

## Design (single-concern, mirror manifest-hygiene-app baseline-block-on-new)

New crate `cloud/cloud-ci/gates/oya-cloud-ci-target-parity-app`:

1. Enumeration (producer-side per the established split): extend `oya-cloud-ci-accounting-registry-app` with a `target-parity` face — rows `{member_path, has_buck, has_rust_test_target, has_test_code}` computed from tracked files: members via `oya-workspace-members-kernel` (REUSE), `has_test_code` = tracked `tests/` dir or any `src/**/*.rs` containing `#[cfg(test)]` or `#[test]`, `has_rust_test_target` = BUCK file contains a `rust_test` rule (textual detection is acceptable v1 — note it in the ADR; a buck2-uquery-grade check is a later hardening).
2. Gate = pure policy over rows. Violation codes:
   - `member_missing_buck` (born-blocking, baseline empty — debt is 0 today, keep it that way),
   - `member_test_code_without_rust_test_target` (baseline-block-on-new: freeze today's 634 keys; only NEW keys block).
   Baseline mechanism: copy `manifest-hygiene-app`'s `baseline-block-on-new` shape exactly (keys = member_path).
3. Remediation text in findings: "declare a rust_test target in <member>/BUCK (see any gates/* BUCK for the stanza shape) and ensure `buck2 test <target>` passes" — plus pointer to ADR-0540.
4. Wiring: one matrix line in `.github/workflows/oya-ci-required.yml` (cargo matrix leg, like the other pure gates) + BUCK targets + registrations: `oya-ci.toml`, `libs/oya-ci-config` (gate count + disposition.json with both codes), `docs/oya-ci/gate-catalog.md`, firewall gate_registration meta-test expectations, regenerate producer faces via `infra/ci/materialize-cloud-ci-generated-faces.sh .` ONLY.
5. `docs/decisions/ADR-0540-*.md`: cites FRIC-1781063357 + FRIC-008 + ADR-0538 (kernel reuse) + ADR-0539 precedent; states the 634-key baseline and the burn-down follow-on; one-way door.

## Tests

GREEN fixture (crate with tests + rust_test; crate without tests and without rust_test); RED fixtures per code (missing BUCK; test code without rust_test NOT in baseline); baseline fixture (in-baseline key does NOT block; same key with NEW debt elsewhere does). Mirror workspace-glob-coverage-app/manifest-hygiene-app test shape. Cited tests must exist.

## Boundaries / commands

Same as prior lanes: isolated worktree, buck2-first verify, `cargo metadata >/dev/null` for lock only (your crate is auto-membered by the gates/* glob), never hand-edit *.generated.json (the new freshness gate will catch staleness — run the materialize script after producer changes), SSH-signed commits, PR to dev citing ADR-0540 with the measured-debt numbers and buck2 evidence.

## Success criteria

1. Gate green on dev tree with the frozen baseline; RED fixtures prove both codes block.
2. A simulated new crate with test code and no rust_test target fails closed (fixture).
3. oya-ci-required green on rebased head; adversarial review APPROVE in code.
4. Baseline file = exactly the measured debt set (no padding, no omissions — reviewer will re-derive).
