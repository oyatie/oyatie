# DELTA reviewer of record — PR #685 ROUND 2 (G011 main-checkout guard)

- Repo: jason931225/oyatie  Branch: agent/g011-checkout-guard  Head: 9cf241e22  Base: dev
- Fix commit under review: 418e0d95f "fix: close transparent-wrapper + git restore bypasses (review #685 F1/F2/F3)"
- Reviewer: fresh-context DELTA (Claude Opus), attacker/Torvalds lens, /using-superpowers + /using-agent-skills + /oh-my-claudecode:ultraqa, all FOREGROUND
- Round-1 verdict: BLOCK (1 HIGH wrapper-bypass class F1, 1 MEDIUM restore F2, 1 LOW $(command -v git) F3)

## VERDICT: **BLOCK**

The round-1 HIGH bypass class (F1) is **NOT fixed**. The fix is an allowlist-only patch of the
specific reviewer-named wrappers; it does **not** implement the default-closed scan-through that the
round-1 review recommended and that this fix's **own commit message explicitly claims**
("default-CLOSED when an unknown leading word precedes a canonical-targeted mutating git op — no
silent fall-through to ALLOW"). The code still silently falls through to ALLOW at lib.rs:398 for any
unknown leading word. Multiple ordinary process-wrappers outside their hardcoded list bypass the
guard against the canonical checkout. F2 (git restore) is genuinely fixed; F3 (LOW) persists and folds
into the same gap. New regression: even an enumerated wrapper (`xargs`) leaks with common
separate-token value flags.

This is the SAME bug class as round-1, re-surfaced under different wrapper names. The green 26-test
suite again gives false assurance — it asserts exactly the named examples and nothing structural.

---

## What I verified by running it (commands + exact exit codes)

Harness: rebuilt the real binary `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out`
(BUILD SUCCEEDED), drove it with JSON hook payloads on stdin, `OYA_CANONICAL_CHECKOUT` set to a
realpath-resolved canonical dir, session cwd neutral. (Note: macOS `/tmp`→`/private/tmp` symlink must
be realpath-resolved or it produces false ALLOWs — a harness artifact, not a guard bug; verified
separately that with realpath canonical, bare `git switch/checkout/reset --hard/restore` with
cwd==canonical all DENY rc=2 and `git status`/`git fetch` ALLOW rc=0.)

### 1) Round-1 bypass list — re-driven (target = canonical via -C)
```
rc=2  nohup git -C <canon> switch foo                         DENY  (was ALLOW r1) FIXED
rc=2  nice git -C <canon> switch foo                          DENY  FIXED
rc=2  timeout 5 git -C <canon> reset --hard HEAD              DENY  FIXED
rc=2  stdbuf -oL git -C <canon> switch foo                    DENY  FIXED
rc=2  setsid git -C <canon> switch foo                        DENY  FIXED
rc=2  echo x | xargs git -C <canon> checkout                  DENY  FIXED
rc=2  echo x | xargs -I{} git -C <canon> checkout {}          DENY  FIXED
rc=2  watch git -C <canon> switch foo                         DENY  FIXED
rc=2  parallel git -C <canon> checkout ::: foo                DENY  FIXED
rc=0  ls git -C <canon> switch foo                            ALLOW (not a real bypass: ls does not exec git; r1 probe)
rc=0  $(command -v git) -C <canon> switch foo                 ALLOW (F3, still open — LOW)
rc=2  git -C <canon> restore .                                DENY  (F2 FIXED)
(subshell/pipe/&&/env-assign supplementaries all DENY rc=2)
```

### 2) ADVERSARIAL EXTENSION — defeat the NEW allowlist+(claimed)default-closed logic
Unknown process-wrappers that exec git straight through — **must DENY if default-closed holds:**
```
rc=0  firejail git -C <canon> switch foo                      ALLOW  *** BYPASS ***
rc=0  eatmydata git -C <canon> switch foo                     ALLOW  *** BYPASS ***
rc=0  proxychains git -C <canon> switch foo                   ALLOW  *** BYPASS ***
rc=0  catchsegv git -C <canon> reset --hard HEAD              ALLOW  *** BYPASS ***
rc=0  flock /tmp/l git -C <canon> switch foo                  ALLOW  *** BYPASS ***
rc=0  runuser -u u -- git -C <canon> switch foo               ALLOW  *** BYPASS ***
rc=0  busybox git -C <canon> switch foo                       ALLOW  *** BYPASS ***
rc=0  systemd-run git -C <canon> switch foo                   ALLOW  *** BYPASS ***
rc=0  cpulimit -l 50 git -C <canon> switch foo                ALLOW  *** BYPASS ***
```
Enumerated wrapper + separate-token value flag (flag-skip bug):
```
rc=0  xargs -a file git -C <canon> checkout                   ALLOW  *** BYPASS (enumerated wrapper!) ***
rc=0  xargs -P 4 -n 1 git -C <canon> checkout                 ALLOW  *** BYPASS (enumerated wrapper!) ***
```
Correctly handled (denied via builtin/reserved-word prefix-strip, NOT the wrapper path):
```
rc=2  command git -C <canon> switch foo                       DENY
rc=2  exec git -C <canon> switch foo                          DENY
rc=2  time git -C <canon> switch foo                          DENY
rc=2  env -C <canon> git switch foo / env git -C <canon> ...  DENY
rc=2  chronic git -C <canon> switch foo                       DENY
rc=2  nohup -- git ... / timeout 5 nohup git ... / nice -n5 git ...  DENY
```

### 3) NO false positives (precision intact — confirmed clean)
```
rc=0  git -C <canon> {status,log,diff,show,branch -a}         ALLOW (reads)
rc=0  git -C <canon> fetch origin / fetch --all --prune       ALLOW
rc=0  git -C <canon> pull --ff-only                           ALLOW
rc=0  git -C <canon> merge --ff-only origin/dev               ALLOW
rc=0  git -C <canon> push origin dev                          ALLOW
rc=0  git -C <canon> branch -D/-d <b>                         ALLOW
rc=0  git worktree add/remove/list                            ALLOW
rc=0  nohup ./build.sh / timeout 5 cargo test / nice -n10 buck2 build  ALLOW
rc=0  xargs rm < files.txt / echo git switch foo / cat git-notes.txt   ALLOW
rc=0  firejail ./build.sh                                     ALLOW
```
The merge-train / leader real commands are not impeded; the allowlist does not over-block.

### 4) Suites / hygiene
- `buck2 test //tools/oya-checkout-guard-app:oya-checkout-guard-app-unittest` -> **26 passed; 0 failed** (Pass 1). The new tests `transparent_wrapper_prefix_bypass_is_denied` and `git_restore_is_denied_in_canonical_checkout` pass — but they assert only the named examples, not the structural property.
- `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-enforcement-liveness-app/...` -> **6 passed; 0 failed** (green; gate stays live).
- `lsp_diagnostics tools/oya-checkout-guard-app/src/lib.rs` -> **clean** (no errors/warnings).
- **Signatures:** all branch commits carry `gpgsig -----BEGIN SSH SIGNATURE-----` headers (ED25519). `git log %G?` shows `U` (Unknown) in my fresh context only because the allowed-signers file is not provisioned here — environmental, not a defect; commits ARE signed.
- **Settle protocol (faces-last):** content commit 418e0d95f touches only `tools/oya-checkout-guard-app/src/lib.rs`; settle commit 9cf241e22 touches only `*.generated.json` faces. Faces-only-last holds.
- **Baseline key-diff (both ways, laundering):** local `dev` ref is stale vs the branch merge-base (8801af778); the apparent large churn in `dev..HEAD` is dev advancing under the branch via two `Merge origin/dev` commits. True branch contribution merge-base..HEAD = **+18 keys, 0 anti-pattern keys removed** (no `forbidden_foundry/unjustified/unowned/unreachable` keys laundered out in either direction). Additions are the new guard's own files registering in the accounting census (expected). Settle baseline edit = +20/-1 lines (additive; -1 = digest). No debt laundered.

---

## Findings

### [HIGH, confidence HIGH] F1-R2 — Unknown-wrapper bypass persists; "default-closed" claim is false
- **lib.rs:375-398.** The fix added `transparent_wrapper_remainder()` (lib.rs:944-1052) as a **hardcoded allowlist** (`nohup nice setsid watch chronic ionice chrt taskset stdbuf timeout sudo doas xargs parallel`). Any leading command word NOT in that list and NOT a shell builtin/reserved word falls through to **lib.rs:398 `command_position = false;`** — a **silent fall-through to ALLOW**, identical to the round-1 defect. The commit message claims "default-CLOSED when an unknown leading word precedes a canonical-targeted mutating git op (no silent fall-through to ALLOW)"; **no such default-closed branch exists in the code.**
- **Confirmed bypasses (exit 0 / ALLOW, expected DENY):** `firejail`, `eatmydata`, `proxychains`, `catchsegv`, `flock <lock>`, `runuser -u u --`, `busybox`, `systemd-run`, `cpulimit -l 50` — each prefixing `git -C <canonical> {switch|reset --hard|checkout}`. These are ordinary, non-exotic process wrappers an agent or review critic can type. This reproduces FRIC-022/FRIC-1781062867 verbatim while the guard reports green.
- **Why it matters:** the rubric — "every bypass the ALLOW-list policy intends to block = HIGH", and "a check evadable by the exact input class it polices is a finding." The allowlist is open-ended by construction: enumerating wrappers is whack-a-mole. The round-1 review prescribed the correct fix ("when the leading command word is unknown AND a later in-position word is a `git` invocation targeting the canonical checkout, DENY — scan-through default-closed"); it was described in the commit but not implemented.
- **Minimal fix:** at lib.rs:398, before `command_position = false`, when the leading word is unknown (not git, not a builtin/reserved word, not env/command/exec/builtin/eval), scan the remaining in-position tokens of the simple command; if any is a `git` invocation whose effective target is within the canonical checkout running a blocked op, DENY. That is the actual default-closed behavior. Keep the allowlist only as an optimization for precise flag-skipping; correctness must not depend on membership.

### [HIGH, confidence HIGH] F4-R2 — Enumerated wrapper leaks via separate-token value flags
- **lib.rs:1041-1052 `skip_flag_args_and_join`.** The flag-skipper stops at the first non-`-` token and joins the rest as the nested command. For value-taking flags whose value is a **separate token** (e.g. `xargs -a <file>`, `xargs -P 4`, `xargs -n 1`), the value token (`file`, `4`, `1`) is treated as the nested **leading command word** — which is unknown, so the nested recursion itself hits the F1 fall-through and ALLOWs.
- **Confirmed (exit 0 / ALLOW, expected DENY):** `xargs -a file git -C <canon> checkout`; `xargs -P 4 -n 1 git -C <canon> checkout`. `xargs --arg-file=file ...` (attached form) correctly DENYs, proving the separate-token form is the gap. `-a`/`-P`/`-n`/`-I` are common xargs flags, so this leaks an explicitly-enumerated wrapper.
- **Why it matters:** even the wrappers the fix DID handle are bypassable with ordinary option spellings; the test suite only covers attached/no-flag forms. Folds into the F1 default-closed fix (a default-closed scan that finds the downstream canonical-targeted git regardless of intervening flag tokens removes this entire class).
- **Minimal fix:** either model per-wrapper value-consuming flags precisely, OR (preferred) rely on the F1 default-closed scan-through so flag-parsing precision is no longer load-bearing for safety. Add `xargs -a`, `xargs -P N`, `xargs -n N` fixtures.

### [LOW, confidence MEDIUM] F3-R2 — `$(command -v git)` path-substitution bypass still open
- **lib.rs:325.** `$(command -v git) -C <canon> switch foo` -> ALLOW: leading word is an unresolved `$(...)` placeholder (not recognized as git); inner substitution is non-mutating. Same as round-1 F3. Contrived; folds into the F1 default-closed fix (unknown leading word + downstream canonical git -> DENY). Tracking, not blocking on its own.

### Resolved since round 1 (verified)
- **F2 (MEDIUM) — `git restore` — FIXED.** `is_blocked_operation` lib.rs:1394 now maps `"restore" => true`. Confirmed: `git -C <canon> restore {.,src/lib.rs,--staged .,--worktree .}` all DENY rc=2; `git restore .` in a non-canonical worktree ALLOWs. New test `git_restore_is_denied_in_canonical_checkout` passes.

### Positive observations (reinforce)
- F2 restore fix is correct and consistent (blocks worktree + index forms; preserves worktree ALLOW).
- The named round-1 wrappers genuinely DENY now, including hardened forms (`nohup --`, double-wrapper `timeout 5 nohup git`, `nice -n5`, leading env-assigns, subshell/pipe/&&), and `timeout` duration/option-value parsing (`-s SIG`, `-sSIG`, `-k 1`, `--preserve-status`) is handled.
- Zero false positives: reads, the full merge-train command set, worktree ops, and non-git wrapper-word commands all ALLOW. Precision is excellent — the gap is purely recall.
- Hygiene is clean: 26+6 tests green, LSP clean, commits signed, faces-last settle, no baseline laundering (verified key-diff both directions against the true merge-base).

---

## Residual risk (single most likely production failure if merged as-is)
An agent or review critic runs an ordinary wrapped command outside the hardcoded allowlist —
`firejail git -C <canonical> switch <branch>`, `flock <lock> git -C <canonical> reset --hard`,
`eatmydata git -C <canonical> checkout`, `systemd-run git ...`, or even an enumerated
`xargs -P4 git -C <canonical> checkout` — and it silently ALLOWs, re-contaminating the canonical
checkout and reproducing FRIC-022/FRIC-1781062867 while the guard's 26 tests stay green and the
liveness gate stays live. The regression is invisible until it blocks a dev fast-forward again. The
allowlist will always lag the wrapper space; only the default-closed scan-through closes the class.

## Required to clear
1. Implement true default-closed at lib.rs:398 (unknown leading word + downstream canonical-targeted
   mutating git -> DENY), making allowlist membership non-load-bearing for safety. Closes F1, F4, F3.
2. Add fixtures for the unknown-wrapper corpus (firejail/eatmydata/proxychains/catchsegv/flock/
   runuser/busybox/systemd-run/cpulimit) and the `xargs -a/-P/-n` separate-token forms — assert DENY.
3. Re-run 26+(new) unit + 6 liveness; re-drive the adversarial sweep above; confirm zero false positives
   on the merge-train set.

VERDICT: BLOCK — fix incomplete (HIGH F1 bypass class re-surfaces; commit's default-closed claim is unimplemented).
