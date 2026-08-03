# Fable review of record — PR #684 (G011 lane supervisor)

- Repo: jason931225/oyatie · Branch: agent/g011-lane-supervisor · Base: dev
- Head SHA reviewed: 90142e40ad2f3a15b7a429ec8d8f93d2d9f0d22d
- Reviewer: Fable (Claude), fresh-context, Torvalds + hyperscaler + owned-architecture lenses
- Harness: /using-superpowers + /using-agent-skills + /oh-my-claudecode:ultraqa
- Context: worker (codex) died at provider usage limit before self-review; PR body carries NO worker verdict. This review is the gating signal.

## VERDICT: APPROVE

Rationale: the commissioned fix (mechanical lane-liveness detector, FRIC-1781110000) is correctly built and correctly solves the right problem; the FRIC-1781111000 stdin-null dispatch fix is present; the decision kernel is pure with an injected clock and exhaustive fail-closed unit tests; baseline/faces are producer-mechanical (regenerated + byte-identical) and the firewall laundering gate is green-for-right-reasons (growth-blocked, still-red-on-violation). No CRITICAL/HIGH at HIGH confidence. Findings below are MEDIUM/LOW process/robustness notes that do not gate merge.

## Commands run + exact outputs (evidence)

1. buck2 test //tools/oya-lane-supervisor-app/...  -> EXIT 0
   - "test result: ok. 22 passed; 0 failed" (lib unittest), bin unittest pass; target-level "Pass 2. Fail 0".
2. buck2 build //tools/oya-lane-supervisor-app:oya-lane-supervisor  -> "BUILD SUCCEEDED" EXIT 0
3. Regenerate faces in worktree: bash infra/ci/materialize-cloud-ci-generated-faces.sh .
   - gate-baseline.generated.json: IDENTICAL (producer-mechanical)
   - accounting-registry.generated.json: IDENTICAL
   - scm-facts.generated.json: IDENTICAL
   - git status clean after regen (committed faces == fresh regeneration)
4. buck2 test firewall gate + gate-registration -> EXIT 0, "Pass 2", inner 4 tests:
   - firewall_is_green_on_the_live_corpus_with_the_baseline ... ok
   - firewall_blocks_baseline_growth_without_signoff ... ok
   - firewall_goes_red_on_a_synthetic_new_violation ... ok
   - firewall_fixtures_execute_red_green_cases ... ok
5. SSH signatures: all 12 branch commits report "G" (good ED25519) via git log --format='%G?'.
6. Production unwrap/expect/panic scan (excluding #[cfg(test)]): lib.rs 0, main.rs 0.
7. forbid(unsafe_code): present in both lib.rs:1 and main.rs:1.
8. Append-only invariants: friction-ledger.jsonl 0 removed lines; dispatch-ledger.jsonl gitignored (local runtime), not rewritten.

## Brief compliance matrix
- dispatch (null stdin / detached / append-only ledger): VERIFIED. main.rs:197 + main.rs:370 stdin(Stdio::null); detach via process_group(0) main.rs:605-610; append_row uses OpenOptions::append(true) main.rs:532-542.
- reap (PID liveness / log mtime with injected clock / gh pr presence; exited/stalled/dead rows; exit 1): VERIFIED. lib.rs evaluate_reap; main.rs:289-293 exit 1 on unhealthy.
- status (human/JSON): VERIFIED. main.rs:296-334.
- pure decision kernel + injected Clock + exhaustive unit tests: VERIFIED. lib.rs Clock trait, no Date::now in lib; 22 tests cover unknown-field preservation, terminal lattice, stall thresholds, fail-closed.
- no unwrap/expect/panic in production; forbid(unsafe_code); BUCK + catalog conventions: VERIFIED.
- TEAMMATE-PREAMBLE commit-early amendment: VERIFIED (§2.1).
- FRIC-1781110000 fix-delivered ledger row: VERIFIED (status_update="fix-delivered", id FRIC-1781110000-G011-lane-supervisor).
- SSH-signed commits / settle (faces-only last commit): VERIFIED last commit faces-only; see Finding 1 for ordering nit.
- CLI retirement-marking per cli_surface_policy: VERIFIED (binary about-string, catalog notes, ADR-0363 amendment).

## Numbered findings

1. [MEDIUM, HIGH confidence] Settle-protocol ordering deviation.
   commit 9f700981e (faces settle) precedes 34ba476ce (content: premise.txt/.gitignore/ADR-0363/root-hub-pointers), then 90142e40a re-settles. Protocol = all content FIRST, faces-only LAST. Net final state is correct (last commit faces-only, byte-clean) so no gate fails; this is a process nit, not a weakening. Minimal fix: none required for merge; for hygiene, future lanes should land all content before the first settle.

2. [LOW, HIGH confidence] Unrelated accounting change bundled into a lane PR.
   New registry/catalog/OWNERS (cloud-ci-platform) reassigns 883 registry/catalog/*.yaml from unowned -> owned, and the producer dot-path fix (accounting-registry-app/src/main.rs resolve_justifications: stop stripping leading '.') reclassifies dotfiles; +97 member_test_code_without_rust_test_target entries are pre-existing untested members baselined as known debt (consistent with dev G011 burndown slices). All producer-mechanical and admitted by the firewall sign-off gate (verified green). Single-concern (ADR-0132) would prefer these in a dedicated accounting lane, but they are correct and non-laundering. Minimal fix: none blocking; note for scope discipline.

3. [LOW, MEDIUM confidence] PID-reuse false-positive window.
   reap liveness uses kill -0 on the supervisor-wrapper PID (main.rs:463-489). If the wrapper is SIGKILL'd (never writes wait-file) and the OS later reuses that PID for an unrelated process, process_alive could report a false "alive". Backstopped by the log-mtime stall check (lib.rs:232-238) which still flags Stalled once the log goes cold. Acceptable for a local bridge; the durable cloud-ci substrate should key on a run_id+start-time tuple rather than bare PID. Minimal fix (follow-on): compare process start-time against the recorded started_at before trusting kill -0.

4. [LOW, HIGH confidence] FRIC-1781111000 row not advanced.
   The dedicated FRIC-1781110000-G011-lane-supervisor row marks fix-delivered, but FRIC-1781111000 itself remains "fix-in-flight" though its dispatch-stdin-null fix actually landed. Cosmetic ledger bookkeeping; append-only respected. Minimal fix: append a fix-delivered status_update row for FRIC-1781111000 (not required for merge).

## Cited-test reality
All 22 lib tests and 2 bin tests exist in the diff and assert what their names claim; ran green under buck2. No phantom tests. Fail-closed coverage is real: exited_without_pr_fails_closed_even_when_worker_exit_was_zero, dispatched_without_pid_fails_closed_before_stall_threshold, dispatching_without_parseable_timestamp_fails_closed, pr_lookup_error_is_non_terminal_but_unhealthy.

## Hyperscaler + owned-architecture lens
- Liveness detector = standard supervisor/heartbeat pattern (Borg-class lane reconciliation), reimplemented Rust-native; PR state treated as ground truth (reconciliation from durable ledger). Sound precedent fit.
- Pure-kernel/effects split (Clock trait injected; process/fs/gh effects confined to main.rs) directly satisfies the 2026-06-10 founder structure directive: zero process/filesystem effects in lib decision paths.
- Crate docs + catalog + about-string explicitly mark the tool as a retirement-marked LOCAL BRIDGE with zero merge authority; durable home named as cloud-ci lane state. The trait shapes (LaneObservation, ReapDecision, Clock) model the destination; gh/kill/codex are adapter-absorbed in main.rs and would not force the kernel to change at cutover. No deepening of coupling to a retirement-marked surface beyond the explicitly-bridged, ledger-tracked boundary.

## Hand-trace of the riskiest path (the exact FRIC-1781110000 failure)
dispatched + pid recorded + process dead + no PR + no wait-file -> evaluate_reap falls to lib.rs:242 (process_alive==Some(false) || pid.is_some()) -> ReapDecision::Dead -> is_unhealthy -> reap exits 1. The previously-silent death is now mechanically detected and gate-hookable. Correct.

## Residual risk
Most likely production failure even if merged: a wrapper that is hard-killed (SIGKILL) at the same instant a fresh, unrelated process inherits its PID, during a window where the lane log is also still fresh (< stall threshold) — reap would momentarily read the lane as alive/healthy and defer the dead verdict until the log goes cold (default 30 min). Bounded, self-correcting via the mtime stall backstop, and fully retired once cloud-ci owns durable lane state keyed on run_id rather than bare PID.
