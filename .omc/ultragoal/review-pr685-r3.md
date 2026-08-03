# DELTA reviewer of record — PR #685 ROUND 3 (G011 main-checkout guard)

- Repo: jason931225/oyatie  Branch: agent/g011-checkout-guard  Head: 096b3a2e8  Base: dev
- Fix under review: 993a8b529 "fix(checkout-guard): true default-closed scan-through (review #685 r2 BLOCK)"
  (+ settle 096b3a2e8). Content delta = 1 file, +106 lines on tools/oya-checkout-guard-app/src/lib.rs.
- Reviewer: fresh-context DELTA (Claude Opus), attacker/Torvalds lens, /using-superpowers +
  /using-agent-skills + security-and-hardening, /oh-my-claudecode:ultraqa, all FOREGROUND.
- r1 BLOCK (transparent-wrapper bypass) -> r2 BLOCK (allowlist-only, default-closed claim false) -> r3.

## VERDICT: **BLOCK**

r3 is a large, genuine improvement: the entire r2 adversarial corpus now DENYs, including the
never-enumerated wrappers (firejail/flock/systemd-run/cpulimit/runuser/eatmydata/proxychains/
catchsegv/busybox) and the F4-R2 xargs separate-token flag leak. The default-closed scan-through is
real and the allowlist is now non-load-bearing for the *bare* `<wrapper> git -C <canon> <mut>` shape.

But the r3 commit's "TRUE default-closed scan-through" claim is **still not fully true**. Two ordinary
command shapes that target the canonical checkout with a mutating git op silently ALLOW. Both are
typed daily by agents and review critics; both re-contaminate the canonical checkout exactly like
FRIC-022/FRIC-1781062867; both pass the green 28-test suite because the new test only asserts the
`<wrapper> git -C <canon>` shape and not these two. This is the SAME meta-pattern as r1/r2: the fix
closes the named corpus, the structural property still leaks one layer deeper.

The default-closed scan-through (`unmodelled_command_git_remainder`, lib.rs:448-461) returns the
remainder *starting at the git token*, discarding everything before it and re-tokenising the rest. That
(a) drops a `GIT_DIR=`/`env GIT_DIR=` env prefix that establishes the canonical target, and
(b) destroys the shell quoting around an `sh -c '<script>'` body, so the nested `sh -c` receives only
the first word of its script. The modeled-wrapper path has the same quoting-loss for `sh -c` (this one
is pre-existing, not introduced by r3, and was not flagged in r2).

---

## Harness (commands + exact exit codes)

Built the real binary via buck2 build of the checkout-guard target with --out (BUILD SUCCEEDED).
Drove it with JSON hook payloads on stdin, `OYA_CANONICAL_CHECKOUT` set to a realpath-resolved
canonical dir (`/private/tmp/r3-canon`; macOS `/tmp`->`/private/tmp` symlink realpath-resolved per the
r2 harness note), neutral session cwd (`/private/tmp/r3-neutral`). rc=2 => DENY, rc=0 => ALLOW.

Harness calibration (sanity): bare `git -C <canon> {switch|reset --hard|restore .|checkout}` -> rc=2
DENY; `git -C <canon> {status|fetch origin|log}` -> rc=0 ALLOW. Calibrated correctly.

### TASK 1 — r2 adversarial corpus re-driven (every one must DENY)
```
rc=2  DENY   firejail git -C <canon> switch foo
rc=2  DENY   eatmydata git -C <canon> switch foo
rc=2  DENY   proxychains git -C <canon> switch foo
rc=2  DENY   catchsegv git -C <canon> reset --hard HEAD
rc=2  DENY   flock /tmp/l git -C <canon> switch foo
rc=2  DENY   runuser -u u -- git -C <canon> switch foo
rc=2  DENY   busybox git -C <canon> switch foo
rc=2  DENY   systemd-run git -C <canon> switch foo
rc=2  DENY   cpulimit -l 50 git -C <canon> switch foo
rc=2  DENY   xargs -a file git -C <canon> checkout                  (F4-R2 separate-token flag — FIXED)
rc=2  DENY   xargs -P 4 -n 1 git -C <canon> checkout                (F4-R2 — FIXED)
rc=2  DENY   echo x | xargs -I{} git -C <canon> checkout {}
rc=2  DENY   git -C <canon> restore {.,--staged .,--worktree src/lib.rs}   (F2 — FIXED)
```
Nested/chained wrappers + previously-correct (all DENY):
```
rc=2  DENY   firejail flock /tmp/l git -C <canon> switch foo
rc=2  DENY   timeout 5 firejail git -C <canon> switch foo
rc=2  DENY   nohup firejail git -C <canon> reset --hard HEAD
rc=2  DENY   cpulimit -l 50 -- eatmydata git -C <canon> checkout foo
rc=2  DENY   {nohup|nice|stdbuf -oL|setsid|watch} git -C <canon> switch foo
rc=2  DENY   timeout 5 git -C <canon> reset --hard HEAD
rc=2  DENY   parallel git -C <canon> checkout ::: foo
rc=2  DENY   {command|exec|time|env|chronic} git -C <canon> switch foo
rc=2  DENY   env -C <canon> git switch foo
```
**Task 1 PASS — the entire r2 BLOCK corpus is closed.**

### TASK 2 — attack the new design

2(a) EXEMPT-SET — command substitution inside exempt args (caught by the separate substitution
scan at decide_with_context lib.rs:50-63, INDEPENDENT of the exempt carve-out):
```
rc=2  DENY   echo $(git -C <canon> reset --hard)
rc=2  DENY   printf %s $(git -C <canon> switch foo)
rc=2  DENY   print $(git -C <canon> reset --hard HEAD)
rc=2  DENY   true $(git -C <canon> checkout foo)
rc=2  DENY   echo `git -C <canon> reset --hard`           (backtick form)
rc=2  DENY   : $(git -C <canon> reset --hard)
rc=2  DENY   pwd $(git -C <canon> switch foo)
rc=0  ALLOW  echo git switch foo                          (textual data — correct)
rc=0  ALLOW  printf 'git reset --hard'                    (textual data — correct)
rc=0  ALLOW  print [-r --] git -C <canon> switch foo      (textual data — correct)
```
2(a) verdict: exempt set is SAFE. The dangerous `$(...)`/backtick form is caught by the substitution
path regardless of the leading exempt word; only pure textual git arguments ALLOW. `print` is
defensible: it is an output-only builtin (zsh/ksh) that never execs its args, and its `$(...)` form is
still caught. No finding here.

2(b)/2(c) — SMUGGLING / sh -c / value-flag mis-skip:
```
rc=2  DENY   firejail git -C <canon> status && git -C <canon> reset --hard   (2nd cmd caught at cmd-pos)
rc=2  DENY   firejail git status; git -C <canon> reset --hard HEAD
rc=2  DENY   firejail git -C <canon> -c alias.x='!git reset --hard' x
rc=2  DENY   firejail --profile=git git -C <canon> reset --hard             (flag value 'git' not mis-hit)
rc=2  DENY   flock -x /tmp/git git -C <canon> reset --hard
rc=2  DENY   firejail --whitelist=<canon> git -C <canon> reset --hard
rc=0  ALLOW  firejail sh -c 'git -C <canon> reset --hard'      *** BYPASS (F1-R3) ***
rc=0  ALLOW  firejail bash -c 'git -C <canon> switch foo'      *** BYPASS (F1-R3) ***
rc=0  ALLOW  flock /tmp/l sh -c 'git -C <canon> reset --hard HEAD'   *** BYPASS (F1-R3) ***
rc=0  ALLOW  cpulimit -l 50 bash -c "git -C <canon> checkout foo"    *** BYPASS (F1-R3) ***
rc=0  ALLOW  runuser -u u -- sh -c 'git -C <canon> reset --hard'     *** BYPASS (F1-R3) ***
rc=0  ALLOW  systemd-run sh -c 'git -C <canon> switch foo'           *** BYPASS (F1-R3) ***
rc=0  ALLOW  firejail GIT_DIR=<canon>/.git git reset --hard          *** BYPASS (F2-R3) ***
rc=0  ALLOW  firejail GIT_DIR=<canon>/.git GIT_WORK_TREE=<canon> git reset --hard  *** BYPASS (F2-R3) ***
rc=0  ALLOW  flock /tmp/l env GIT_DIR=<canon>/.git git reset --hard  *** BYPASS (F2-R3) ***
rc=0  ALLOW  cpulimit -l 50 GIT_DIR=<canon>/.git git checkout foo    *** BYPASS (F2-R3) ***
rc=2  DENY   firejail {env FOO=bar|FOO=bar} git -C <canon> reset --hard   (positive control: -C survives)
```
Isolation (provenance):
```
rc=2  DENY   sh -c 'git -C <canon> reset --hard'                    (bare sh -c handled at cmd-pos)
rc=0  ALLOW  nohup sh -c 'git -C <canon> reset --hard'              (PRE-EXISTING — modeled wrapper, not new)
rc=0  ALLOW  sudo sh -c 'git -C <canon> reset --hard'               (PRE-EXISTING)
rc=0  ALLOW  eval sh -c 'git -C <canon> reset --hard'               (PRE-EXISTING)
rc=2  DENY   env sh -c 'git -C <canon> reset --hard'                (env inline-cmd path preserves it)
rc=2  DENY   {env GIT_DIR=<canon>/.git|GIT_DIR=<canon>/.git} git reset --hard   (bare env GIT_DIR handled)
rc=2  DENY   nohup env GIT_DIR=<canon>/.git git reset --hard        (modeled wrapper joins FULL remainder)
```
Cross-check on r2 head (9cf241e22), built separately: nohup/sudo/eval sh -c forms already ALLOWed on
r2 — confirms the `sh -c` quoting-loss is pre-existing in the modeled-wrapper path, NOT introduced by
r3. r3's NEW scan-through inherits the same flaw for *unknown* wrappers rather than closing it.

2(d) — FALSE-POSITIVE SWEEP (zero false positives on the real workload):
```
rc=0  ALLOW  git -C <canon> {status,log,diff,show HEAD,branch -a}
rc=0  ALLOW  git -C <canon> {fetch --all --prune,pull --ff-only,merge --ff-only origin/dev,push origin dev}
rc=0  ALLOW  git -C <canon> branch -D oldbranch
rc=0  ALLOW  git worktree {add,remove,list}
rc=0  ALLOW  firejail ./build.sh ; flock /tmp/lock ./retrain.sh 685 br wt ; cpulimit -l 50 <test-runner>
rc=0  ALLOW  systemd-run buck2 build //tools/... ; make build && echo done
rc=0  ALLOW  docker run --rm myimage git status ; ssh host git -C <canon> status
rc=0  ALLOW  grep git <canon>/file ; find . -name git
rc=0  ALLOW  firejail git -C <worktree> reset --hard ; firejail git reset --hard   (non-canonical target)
```
Leader's /tmp/retrain.sh real command set (canonical = /Users/jasonlee/Developer/oyatie, cwd = worktree):
```
rc=0  ALLOW  git -C <REAL> {ls-remote origin <br>,rev-parse origin/dev,merge-base origin/dev HEAD,rev-parse --short HEAD}
rc=0  ALLOW  git fetch -q origin
rc=0  ALLOW  git clean -qfd
rc=0  ALLOW  git checkout -q -- .
rc=0  ALLOW  git checkout -q -B <br> origin/dev
rc=0  ALLOW  cat /tmp/retrain-files.txt | xargs git checkout <tip> --
rc=0  ALLOW  git add -A ; git commit -q -m retrain ; git push -qf origin <br>
rc=0  ALLOW  buck2 run //...:oya-cloud-ci-face-settle-bin -- --settle --commit
```
**Zero false positives.** Every read against canonical and every worktree-side mutation ALLOWs;
the merge-train set is unimpeded. (Conservative over-DENYs exist — `docker run ... git -C <canon>
reset --hard`, `ssh host git -C <canon> reset --hard` — but these are fail-safe, not false ALLOWs, and
do not impede the real workload.)

### TASK 3 — suites / hygiene
- buck2 test of the checkout-guard unittest target -> **28 passed; 0 failed**.
- buck2 test of the cloud-ci enforcement-liveness gate target -> **2 passed; 0 failed** (gate live).
- lsp_diagnostics on lib.rs -> **clean** (no errors/warnings).
- Signatures: 096b3a2e8 + 993a8b529 both carry `gpgsig -----BEGIN SSH SIGNATURE-----` (ED25519).
  `%G?` = `U` only because allowed-signers is not provisioned in this fresh context — environmental.
- Settle faces-last: content 993a8b529 touches ONLY lib.rs; settle 096b3a2e8 touches ONLY
  `*.generated.json` faces. Faces-only-last holds.
- Baseline key-diff (both ways): merge-base 8801af778. Full branch delta merge-base..HEAD over
  `*.generated.json` shows **0 anti-pattern keys** (forbidden_foundry/unjustified/unowned/unreachable)
  added or removed in either direction. Settle face = +5/-5, no laundering.
- Exempt-set addition is deliberate, minimal (echo/printf/print/true/false/:/pwd), and documented
  (lib.rs:434-446) as a security decision; its `command_never_executes_arguments` gate is correct and
  the dangerous `$(...)` form is independently caught. Good.
- tools/hooks/main-checkout-guard.sh is irreducible glue only (locates + execs the Rust binary;
  policy in Rust) — consistent with the irreducible-glue ledger. It FAILS OPEN on missing binary /
  unresolved repo-root (exit 0) — see residual risk.

---

## Findings (file:line, severity, confidence)

### [HIGH, confidence HIGH] F1-R3 — `<wrapper> sh -c '<git mutation>'` bypass (quoting loss on recursion)
- **lib.rs:448-461 (`unmodelled_command_git_remainder`) and the modeled-wrapper join at
  lib.rs:1097-1110 / 1259-1268.** Both recursion paths reconstruct the nested command by
  `words_until_separator(...).join(" ")`, which discards the original shell quoting. When the nested
  command is `sh -c '<script>'`, re-tokenisation splits the quoted `<script>` into separate words, so
  `shell_wrapper_script` (lib.rs:976-977) returns only the FIRST word of the script (`git`), not the
  full `git -C <canon> reset --hard`. Result: bare `git` with no mutating subcommand -> ALLOW.
- **Confirmed bypasses (rc=0 / ALLOW, expected DENY):** `firejail sh -c 'git -C <canon> reset --hard'`,
  `flock /tmp/l sh -c '...'`, `cpulimit -l 50 bash -c "..."`, `runuser -u u -- sh -c '...'`,
  `systemd-run sh -c '...'`. The pre-existing modeled-wrapper variants nohup/sudo/eval sh -c forms
  also ALLOW (verified on the r2 head too — pre-existing, not introduced by r3, and NOT flagged in r2).
- **Blast radius / exploitability:** local, no privileges. An agent that wraps work in `sh -c`/`bash -c`
  under ANY wrapper (modeled or unknown) mutates the canonical checkout silently. `firejail sh -c
  'git -C <main> reset --hard'` is an ordinary, non-exotic command. Reproduces FRIC-022 while green.
- **Minimal fix:** preserve quoting across recursion. Pass token slices (not a space-joined string) into
  `decide_with_context`, OR re-quote each word before joining so `sh -c '<body>'` survives re-tokenisation
  as one argument. Then the existing `shell_wrapper_script` path will DENY the nested mutation. This
  simultaneously closes the pre-existing modeled-wrapper sh -c leak.

### [HIGH, confidence HIGH] F2-R3 — `<unknown-wrapper> [env] GIT_DIR=<canon>/.git git <mut>` bypass (env-prefix stripped)
- **lib.rs:454.** `unmodelled_command_git_remainder` returns `words_until_separator(tokens, index)`
  starting AT the `git` token (`index`), DISCARDING every token before it — including a leading
  `GIT_DIR=`/`GIT_WORK_TREE=` assignment or an `env GIT_DIR=...` prefix that establishes the canonical
  target. The nested recursion then evaluates a bare `git reset --hard` against the neutral session
  cwd (not canonical), so the target is judged outside canonical -> ALLOW.
- **Confirmed bypasses (rc=0 / ALLOW, expected DENY):** `firejail GIT_DIR=<canon>/.git git reset --hard`,
  `firejail GIT_DIR=<canon>/.git GIT_WORK_TREE=<canon> git reset --hard`,
  `flock /tmp/l env GIT_DIR=<canon>/.git git reset --hard`,
  `cpulimit -l 50 GIT_DIR=<canon>/.git git checkout foo`. Bare (no wrapper) `GIT_DIR=<canon>/.git git
  reset --hard` correctly DENYs, and `nohup env GIT_DIR=... git ...` correctly DENYs (modeled path joins
  the FULL remainder) — proving the hole is specific to the unknown-wrapper scan-through dropping the
  pre-git env context. Positive control `firejail git -C <canon> reset --hard` DENYs (`-C` survives).
- **Blast radius / exploitability:** local, no privileges. The agent controls the environment; GIT_DIR
  targeting of the canonical `.git` is the canonical way to mutate a repo from anywhere. Same severity
  class as F1-R3.
- **Minimal fix:** in `unmodelled_command_git_remainder`, return the remainder from the FULL command
  (i.e. from the token after the wrapper word, not from the git token) so env-assignment prefixes and
  `env`-prefix context are preserved for the recursion — exactly what the modeled-wrapper path already
  does. Folds into the F1-R3 fix (propagate full context, not a git-token-anchored slice).

### Note — both findings share one root cause
The scan-through is "git-token-anchored": it finds the first `git` basename and recurses on a
space-joined slice from that token. That loses (a) shell quoting of any later `sh -c` body and
(b) any pre-git env/GIT_DIR context. The correct default-closed primitive recurses on the wrapper's
FULL remainder with quoting preserved, letting the existing env/sh-c/`-C` machinery decide. The current
design is "scan for git then strip context," which is why both leaks exist.

### Resolved since r2 (verified)
- **F1-R2 unknown-wrapper class — FIXED** for the bare `<wrapper> git -C <canon> <mut>` shape
  (firejail/flock/systemd-run/cpulimit/runuser/eatmydata/proxychains/catchsegv/busybox all DENY).
- **F4-R2 xargs separate-token flag leak — FIXED** (`xargs -a file ...`, `xargs -P 4 -n 1 ...` DENY,
  via the new scan-through catching the downstream git after the unhandled flag value).
- **F3-R2 `$(command -v git)` — covered** by the substitution scan (the `$(...)` body is recursed).
- **F2 git restore — remains FIXED.**

### Positive observations (reinforce)
- Default-closed scan-through is real and well-documented; the exempt set is minimal, audited, and its
  `$(...)` escape hatch is independently caught by the substitution path. Good security reasoning.
- Zero false positives on reads, the full merge-train set, worktree ops, and the leader's retrain.sh.
- Hygiene clean: 28+2 green, LSP clean, both commits SSH-signed, faces-last settle, no key laundering
  in either direction against the true merge-base.

---

## Residual risk (single most likely production failure if merged as-is)
An agent runs `firejail sh -c 'git -C <canonical> reset --hard'` or
`flock <lock> GIT_DIR=<canonical>/.git git checkout <branch>` (or any `<wrapper> sh -c '<mut>'` /
`<wrapper> [env] GIT_DIR=<canonical> git <mut>` shape — including the modeled wrappers
nohup/sudo/eval for the sh -c form) and it silently ALLOWs, re-contaminating the canonical checkout
and reproducing FRIC-022/FRIC-1781062867 while the 28 tests stay green and the liveness gate stays
live. Secondary: the hook shim fails OPEN if the Rust binary is unbuilt (`exit 0`) — a missing/stale
binary disables the guard entirely; ensure CI/branch-protection builds it (structural enforcement, not
hook reliance).

## Required to clear
1. Make the recursion context-preserving (close F1-R3 + F2-R3 together): recurse on the wrapper's FULL
   remainder with shell quoting preserved (token-slice or re-quoted join), not a git-token-anchored
   space-joined slice. This lets the existing `sh -c`, `env`/`GIT_DIR`, and `-C` machinery decide, and
   also closes the pre-existing modeled-wrapper sh -c leak (nohup/sudo/eval).
2. Add fixtures asserting DENY for: `<wrapper> sh -c '<canonical mut>'` (firejail/flock/nohup/sudo/eval),
   and `<wrapper> [env] GIT_DIR=<canon>/.git git <mut>` (firejail/flock/cpulimit). The current
   `denies_unmodelled_wrapper_prefixed_mutations_default_closed` test asserts only the `git -C` shape
   and structurally cannot catch these — that is why they leaked past green.
3. Re-run 28+2 + new fixtures; re-drive the Task-2 sweep; confirm zero false positives on the
   merge-train + retrain.sh set.

VERDICT: **BLOCK** — two HIGH default-closed leaks remain (`<wrapper> sh -c '<mut>'` and
`<wrapper> [env] GIT_DIR=<canon> git <mut>`); the "true default-closed scan-through" claim is not yet
true. Strong progress: the full r2 corpus is closed and precision is intact.
