# Open questions

## ci-gate-matrix-contention-durable-fix - 2026-08-03

- [ ] **CENTRAL RISK.** Can `p3_history_fixture` + `copy_materialized_p2_parent_receipt` + a
  canonical-path emission drive `validate_census_gate_for_event` to `Ok(())`? — Every ingredient
  exists (`snapshot_integration.rs:1526-1570` performs each operation), but no existing arm combines
  them, and no existing green arm emits to the canonical in-repo path. If no, §8's fallback applies:
  keep M1, land the two negative arms only, do not revert the extraction.
- [ ] Can the liveness invariant ("every registered gate lane has at least one negative-control arm")
  be derived from the build graph, or does it need a registry? — The offered precedent
  (`gate_registration.rs:1218-1220` / `:677`) derives *pattern → package directory*, not test names.
  A test-name → gate-lane map would be a second hand-kept list, which P4 forbids. Blocks follow-up (b).
- [ ] Should the 19 never-executed `rust_binary` targets under `ci/` be deleted? — 19 of 30 (63%) are
  executed by nothing: no `$(exe)` in any BUCK file, no non-comment workflow line. Promoted to the
  top follow-up; should be the next plan, not a trailing bullet.
- [ ] Is `//ci/facade/baseline-ratchet:oya-cloud-ci-run-terminal-state-bin` intended to stay
  comment-only? — It is the single binary named in raw YAML but not in any executable line (11 raw
  vs 10 non-comment). A deliberate parking spot with no expiry date is indistinguishable from rot.
- [ ] AC#1 — assert the bin's `validate_gate_from_event` stays ≤ 2 statements: schedule the diff-based
  check in this PR, or drop AC#1 and rely on follow-up (a). Must be one or the other, not implicit.

## ci-gate-matrix-contention-durable-fix (iteration 4) - 2026-08-03

- [ ] RESOLVED, recorded here for audit: AC#1 (the "≤ 2 statements" check) is DROPPED. Its `grep -c ';'`
  counted the rewritten import block, not the function body. It is superseded by follow-up (a).
- [ ] Follow-up (a) landing condition CORRECTED: a `#[test]` in the bin's existing `mod tests`, already
  compiled by `ci-scm-facts-snapshot-gate` (`BUCK:61-67`), not a `srcs` change at `BUCK:82` — that target
  is rooted at `src/lib.rs` and cannot compile the bin's test module.
- [ ] G1: the bin's `validate_gate_from_event` — env read, `?` chain as written there, and the choice of
  `repo_root.join(ADR_CENSUS_EPOCH_RECEIPT_PATH)` (`bin:136`) — stays covered only by matrix leg 1.
- [ ] Will the appended line be green on the `:1553` host? — Not run; this worktree is a stale preserve
  branch. Confirm on a clean `origin/dev` checkout. A red result is a finding, not a workaround trigger.
- [ ] Are the 19 direct-executor-less `rust_binary` targets under `ci/` covered by their sibling gate
  tests? — UNMEASURED. 18 of 19 sit in a crate declaring a `*-gate` `rust_test`; that is presence, not
  coverage. Entry point for follow-up (d).
