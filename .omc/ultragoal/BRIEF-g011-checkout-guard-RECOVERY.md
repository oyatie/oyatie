# Recovery brief — finish the G011 main-checkout-guard lane (worker died at context compaction)

You are resuming a lane whose previous worker process died mid-iteration (context compaction at 2026-06-10 12:32). ALL prior context is in the worktree state; do not re-derive the design.

Read FIRST: `/Users/jasonlee/Developer/oyatie/.omc/ultragoal/BRIEF-g011-main-checkout-guard.md` (the commissioning brief — its deliverables and rules are unchanged and binding).

## Verified state at recovery dispatch
- Worktree: `/Users/jasonlee/oyatie-worktrees/g011-checkout-guard` (branch `agent/g011-checkout-guard`, base `16f2e3b54`). Work ONLY here. NEVER touch `/Users/jasonlee/Developer/oyatie`.
- 4 commits landed: `1560f662f` (feat: guard kernel+bin+shim+wiring), `b3b58fe40` (hook liveness census), `dacf443da` (evidence record), `a6d9484df` (faces-only settle, currently HEAD).
- DIRTY uncommitted edit in `tools/oya-checkout-guard-app/src/lib.rs`: the previous worker was mid-way through hardening fixes — `cd` tracking for effective-target resolution, `git branch -f/--force` deny, `--git-common-dir` fallback for `default_canonical_checkout`, plus new unit tests. The edit may be INCOMPLETE — treat it as a draft, not finished work.

## Your job (one PR at the end)
1. Inspect `git status` + `git diff`. Review the dirty edit for completeness and correctness (does `parse_cd_target` handle `cd -`? quoting? does the `cd` tracking respect command separators `;`/`&&`/`|`? do all new tests pass?). Finish or fix it; keep the decision logic a pure function with exhaustive unit tests; no unwrap/expect/panic in production paths, `#![forbid(unsafe_code)]`.
2. `buck2 build` + `buck2 test` the crate's targets until green. Measure warm shim invocation latency as the original brief requires (<100ms) and record it in the evidence file if not already recorded.
3. Re-check the dogfood proof: enforcement-liveness gate test still green (dual wiring `.claude/settings.json` + `.codex/hooks.json` intact).
4. Commit the content fix (SSH-signed), then SETTLE PROTOCOL: `git add` everything → `infra/ci/materialize-cloud-ci-generated-faces.sh .` → faces-only settle commit LAST. Never hand-edit `*.generated.json`.
5. MANDATORY pre-PR adversarial self-review per the original brief §"MANDATORY": fresh `codex exec` with `/Users/jasonlee/Developer/oyatie/.omc/ultragoal/RUBRIC-torvalds-review.md`, fix all CRITICAL/HIGH, include verdict + findings-fixed in the PR body.
6. Push `-u origin agent/g011-checkout-guard`, open PR to `dev` with `gh`, citing FRIC-022 + FRIC-1781062867 + rust-hooks doctrine + the recovery (previous worker death) in the body. Report the PR number as your final output line in the form `PR_OPENED: <number>`.
