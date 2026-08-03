# DELTA reviewer of record — PR #685 ROUND 5 (G011 main-checkout guard)

- Repo: jason931225/oyatie  Branch: agent/g011-checkout-guard  Head: 009f3617e  Base: dev
- Fix under review: e0b09b2ac "fix(checkout-guard): fail-closed on depth exhaustion + escape-aware
  backtick scan (review #685 r4)" (+ settle 009f3617e). Delta vs r4 head (dff604060): 1 file,
  +71/-4 on tools/oya-checkout-guard-app/src/lib.rs (rest is 2 generated faces).
- Reviewer: fresh-context DELTA (Claude Opus), attacker/Torvalds lens, /using-superpowers +
  /using-agent-skills + /ultraqa, all FOREGROUND. r1/r2/r3/r4 all BLOCK.

## VERDICT: **BLOCK**

The r5 fix is correct and complete **for exactly what it claimed**: BOTH r4 residuals are genuinely
closed. (a) Depth exhaustion now fails CLOSED at `decide_with_context` entry (lib.rs:55-59): every
recursion-bearing path re-enters that function, so `nice*33`, `nohup sudo *17`, mixed chains, and
arbitrarily deep nests (tested to *500) all DENY, with the deny firing as a true wall regardless of
inner content and ZERO false positives on legitimate nesting (legit ≤4; the ALLOW→DENY transition is at
~28 wrappers, far above anything real). (b) The backtick scanner is now escape-aware
(`find_unescaped_backtick` lib.rs:1281 + `unescape_backtick_body` lib.rs:1300): the r4 nested escaped
backtick `echo \`echo \`git…\`\`` now DENYs, as do triple-nesting, `$()`-inside-backtick,
backtick-inside-`$()`, escaped-`$`, and double-escaped variants. 29 unit + enforcement-liveness gate
green, LSP clean, faces-last settle holds, both commits SSH-signed, 0 anti-pattern key laundering.

BUT — looking ONE LAYER PAST the r5 fixtures (the failure mode every round) — I found a **large,
systemic class of confirmed canonical-mutation bypasses that the accumulated review has not yet caught.**
The guard's tokenizer/parser does not model shell EXPANSIONS that produce a command word or a git
subcommand. Whenever the `git` word, the subcommand (`reset`/`switch`/`checkout`/`restore`), or the
whole command line is produced by an expansion the tokenizer can't see through, the guard fails OPEN and
ALLOWs a real canonical mutation. 10 distinct, minimal, realistic reproductions confirmed below — each
proven against REAL git (working-tree + HEAD actually change) in bash AND zsh AND sh. These are
**pre-existing** (the r4 head `dff604060` binary ALLOWs them identically — NOT a regression introduced by
r5), but they are live, trivially constructible, and reproduce FRIC-022/FRIC-1781062867. Per the review
contract — BLOCK on ANY concrete bypass a realistic adversary (agent or critic running a shell command)
can execute against the canonical checkout — this blocks.

---

## Harness (commands + exact exit codes)

Built the real binary: `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out
/tmp/r5-guard-bin` (BUILD SUCCEEDED; Mach-O arm64). Drove it with JSON hook payloads
`{"tool_input":{"command":"…"}}` on stdin (the real enforcement surface, lib.rs:500-520),
`OYA_CANONICAL_CHECKOUT=/private/tmp/r5-canon`, neutral session cwd. rc=2 ⇒ DENY, rc=0 ⇒ ALLOW.
Calibrated: `git -C <C> {switch|reset --hard|restore .|checkout}` → rc=2; `{status|fetch|log}` → rc=0.

**Adjudication discipline (the r4 lesson):** every ALLOW was adjudicated against REAL git semantics in a
throwaway repo (file `f` dirtied with a `DIRTY` marker; mutation = working-tree OR `rev-parse HEAD`
actually changes after `bash/zsh/sh -c "<cmd>"`). A naive fakegit that exit-0's on anything PRODUCED A
FALSE ALARM on the `\<space>` class (`git -C <C> reset\ --hard`) — real git rejects `reset --hard` as a
single unknown subcommand (`'reset --hard' is not a git command`, NO mutation), so the guard's ALLOW
there is FAITHFUL, not a hole. The 10 findings below all pass the REAL-git mutation test (HEAD/worktree
verified changed), e.g. `eval $(echo git -C <C> checkout other)` was confirmed to switch HEAD
5538e52→fd8341a (branch master→other) — a real checkout.

### TASK 1 — full accumulated r1+r2+r3+r4 corpus (all must DENY) — **PASS 46/46**
```
rc=2 DENY  16 transparent/unmodelled wrappers (firejail|eatmydata|proxychains|catchsegv|busybox|
           systemd-run|timeout|nohup|nice|setsid|sudo|stdbuf|chronic|ionice|taskset|watch) git switch
rc=2 DENY  flock / runuser / cpulimit git <mut>
rc=2 DENY  xargs {-a,-P -n,-I{}} git checkout                              (F4-R2)
rc=2 DENY  git restore {., --staged ., --worktree <f>}                     (F2)
rc=2 DENY  nested unmodelled triple; timeout firejail; nohup firejail git <mut>
rc=2 DENY  r3 F1: <wrapper> {sh,bash} -c '<mut>'  x8 (firejail|sudo|nohup|nice|flock|cpulimit|runuser|systemd-run)
rc=2 DENY  r3 F2: <wrapper> [env] GIT_DIR=<canon>/.git [GIT_WORK_TREE=…] git <mut>  x4
rc=2 DENY  r3 substitution corpus: $()/backtick/assign/$()+firejail/$()+sh-c/nested $()  x6
```
Entire accumulated corpus closed — no regression.

### TASK 2 — verify r4 fixes + boundary false-positive — **PASS**
```
rc=2 DENY  nice*{33,40,64,100,500} git -C <C> reset --hard   (F1-R4 depth fail-closed: now a true wall)
rc=2 DENY  nohup*33 ; sudo*33 ; (nohup sudo)*17 ; (nohup sudo nice setsid)*10 git <mut>
rc=2 DENY  echo `echo \`git -C <C> reset --hard\``           (F2-R4 nested escaped backtick now DENY)
rc=0 ALLOW nice*{1,2,3,4} git -C <C> status                 (legit shallow read — NO false positive)
ALLOW→DENY transition for read-only deep nests is at ~28 wrappers — far above legit (≤4); the depth
fail-closed never fires on any realistic command. No boundary false positive (31 vs 32 both safe-side).
```

### TASK 3 — ATTACK THE NEW CODE / one layer past (real-git adjudicated)
3(a) `unescape_backtick_body` / `find_unescaped_backtick` — **SOUND, 0 false negatives.**
```
rc=2 DENY  nested escaped backtick; triple-nested; $()-in-backtick; backtick-in-$(); escaped-$ in body;
           escaped-$( in body; assign-backtick; printf-nested-backtick; dquoted-backtick
rc=0 ALLOW double-escaped \\` ; escaped-backtick-in-dquotes ; backslash-newline ; \<space> in subcommand
           — ALL adjudicated FAITHFUL: real git does NOT mutate (literal backslash / inert / unknown
           subcommand). The new escape-aware backtick code is correct and does not leak.
```
3(b) depth fail-closed — **SOUND.** All 9 recursion sites (lib.rs:62,186,206,276,321,344,394,420) plus
     `eval` (lib.rs:276) re-enter `decide_with_context`, which fails closed at entry (lib.rs:55). No
     recursion-bearing helper bypasses the entry: `extract_command_substitutions`,
     `extract_balanced_dollar_command`, `find_unescaped_backtick`, `unescape_backtick_body` are
     non-recursive and do not call `decide_with_context`. No false positive at the 31/32 boundary.
3(c)/3(d) — **8 CONFIRMED BYPASS FORMS in two structural classes (see Findings).**

### TASK 4 — FALSE-POSITIVE SWEEP — **PASS 34/34 ALLOW, zero false positives**
```
rc=0 ALLOW git -C <C> {status,log,diff,show HEAD,branch -a,rev-parse,ls-remote,merge-base,fetch --all
           --prune,push origin dev,commit -m,add -A,merge --ff-only,pull --ff-only}
rc=0 ALLOW full merge-train + retrain.sh seq (checkout -B/fetch/clean/checkout --/add/commit/push)
rc=0 ALLOW gh pr merge ; buck2 build ; buck2 test ; cargo metadata ; xargs checkout origin/dev --
rc=0 ALLOW git worktree {add,remove,list} ; sudo sh -c 'echo' ; firejail ./build.sh ; flock retrain.sh
rc=0 ALLOW reset --hard on NON-canonical target ; firejail reset non-canon
rc=0 ALLOW deep-but-legit: nice nohup status ; firejail sh -c "git status" ; nice*4 fetch
```
Precision intact. The new fail-closed has zero precision cost on legitimate commands.

### TASK 5 — suites / hygiene — **all green**
- buck2 //tools/oya-checkout-guard-app:oya-checkout-guard-app-unittest → **29 passed; 0 failed**
  (includes new `depth_exhaustion_fails_closed_not_open`; nested-escaped-backtick fixture in
  `denies_unmodelled_wrapper_prefixed_mutations_default_closed`).
- buck2 enforcement-liveness gate (…-app-gate) → **Pass 1; 0 fail** (gate LIVE).
- LSP diagnostics lib.rs → **clean**.
- SSH sigs: e0b09b2ac + 009f3617e both carry BEGIN SSH SIGNATURE (ED25519). `%G?`=U is environmental
  (allowed-signers not provisioned in fresh context), same as r4.
- Faces-last settle: e0b09b2ac touches ONLY lib.rs; 009f3617e touches ONLY 2 *.generated.json. Holds.
- Baseline key-diff both ways: merge-base 8801af778. Across r4→r5 AND full merge-base→r5 over
  *.generated.json, **0 anti-pattern keys** (forbidden_foundry/unjustified/unowned/unreachable) added or
  removed. Face churn is pure provenance (last_touch_commit lib.rs pointer, head_time_secs). No
  laundering. r4 fixtures (nice*33, nested-backtick, bare reset) all still DENY (no regression).

---

## Findings (file:line, severity, confidence)

### [HIGH, confidence HIGH] F1-R5 — substitution-as-command / `eval`+substitution bypass
- **Root cause: lib.rs:61 + lib.rs:273-288 + lib.rs:451-456.** `extract_command_substitutions` treats a
  command substitution's body as DATA — it recurses into `$(…)`/`` `…` `` and evaluates the *inner*
  command, but it never models that the substitution's OUTPUT becomes the OUTER command line. When the
  body is `echo git -C <canon> reset --hard`, the recursion sees `echo` (in
  `command_never_executes_arguments`, lib.rs:451) with `git` as a mere echo ARGUMENT → ALLOW; the outer
  frame then treats `$(…)` as a single produced word, never as a `git` command. `eval` (lib.rs:273)
  bare-joins its tail and recurses, but `eval $(echo git …)` / `eval \`echo git …\`` hit the same gap:
  the `$()` "prints" git as data, so neither the substitution recursion nor the eval recursion ever sees
  a bare `git reset --hard` command.
- **Confirmed bypasses (rc=0 / ALLOW; real git HEAD/worktree MUTATES in bash+zsh+sh):**
  ```
  $(echo git -C <canon> reset --hard)                     # bare substitution-as-command (simplest)
  `echo git -C <canon> reset --hard`                      # bare backtick substitution-as-command
  eval $(echo git -C <canon> reset --hard)                # proven: switched HEAD master→other on checkout
  eval `echo git -C <canon> reset --hard`
  bash -c "$(echo git -C <canon> reset --hard)"           # sh -c $()
  eval $(printf "git -C <canon> reset --hard")
  ```
- **Exploitability:** local, no privileges, single short command line. No length/token cap. Trivially
  constructible by an agent or an injected instruction (the lethal-trifecta surface CLAUDE.md warns of).
- **Blast radius:** silent canonical-checkout mutation (reset/checkout/switch/restore) reproducing
  FRIC-022/FRIC-1781062867, while all 29 tests + liveness gate stay green (no substitution-as-command
  fixture exists).
- **Why missed:** same r1→r4 meta-pattern. The substitution machinery models "is there a git command
  INSIDE the substitution?" but not "does the substitution PRODUCE a git command?". The `eval` carve-out
  was verified safe by r4 ONLY for `eval sh -c '…'` (concat-then-reparse destroys quoting); `eval $(echo
  git …)` is a different shape the r4 proof did not cover.
- **Minimal fix direction:** when a command word is itself a command substitution `$(…)`/`` `…` `` (i.e.
  the substitution sits in command position, or is the argument of `eval`), evaluate the substitution's
  *textual output* as a command — model `echo`/`printf` literal-arg output as the produced command line —
  OR fail CLOSED when a command-position word is an unresolved substitution/`eval`-of-substitution that
  could yield a git invocation. Add fixtures: bare `$(echo git … <mut>)`, `eval $(echo git … <mut>)`,
  `bash -c "$(echo git … <mut>)"`.

### [HIGH, confidence HIGH] F2-R5 — expansion-produced command word / subcommand bypass
- **Root cause: lib.rs:335 (`command_basename_is(word,"git")`) + lib.rs:356
  (`is_blocked_operation(&invocation.subcommand,…)`) + shell_tokens lib.rs:1683-1689.** The guard matches
  the literal token `git` and a literal subcommand token (`reset`/`switch`/…). `shell_tokens` does not
  perform parameter/ANSI-C expansion, so any expansion that yields the command name or subcommand evades
  the literal match. ANSI-C `$'reset'` tokenizes to `$reset` (the `$` is pushed as a literal char,
  lib.rs:1689 `_ => current.push(ch)`; the single quotes strip to `reset`) → subcommand `$reset` ≠
  `reset` → ALLOW; real bash expands `$'reset'`→`reset` → mutation.
- **Confirmed bypasses (rc=0 / ALLOW; real git MUTATES in bash+zsh+sh):**
  ```
  git -C <canon> $'reset' --hard                          # ANSI-C quoting on subcommand
  git -C <canon> re$'set' --hard                          # split ANSI-C (re + $'set')
  g=git; $g -C <canon> reset --hard                       # git-as-variable (command name)
  r=reset; git -C <canon> $r --hard                       # subcommand-as-variable
  git -C <canon> ${x:-reset} --hard                       # default-value expansion
  git -C <canon> "$(printf 'reset')" --hard               # substitution-as-subcommand
  ```
- **Exploitability:** local, no privileges, single short command line. `$'reset'` and `${x:-reset}` are
  one-liners with no setup; the variable forms need a trivial `g=git;`/`r=reset;` prefix in the same line.
- **Blast radius:** identical to F1-R5 — silent canonical mutation, 29 tests + gate stay green.
- **Minimal fix direction:** the tokenizer must resolve (or conservatively fail-closed on) ANSI-C `$'…'`,
  parameter expansion `$VAR`/`${VAR…}`, and same-line assignments before matching the `git` word and
  subcommand — OR fail CLOSED when a command-position word OR the git subcommand token contains an
  unresolved `$`/`${`/`$'`. Legit commands never need an expansion to name `git` or its subcommand, so
  fail-closed-on-unresolved-expansion-in-command-position has negligible precision cost (verify against
  the merge-train). Add fixtures for each form above.

### Note — both findings are the r1→r5 meta-pattern recurring
The named r4 corpus (depth + nested-backtick) is genuinely and well closed, with zero false positives.
Both residuals are the structural property leaking one layer past where the new fixtures assert: the
guard models commands written *literally*, but a shell EXPANSION (substitution-as-command, `eval`+`$()`,
ANSI-C, `$VAR`, `${x:-…}`) can synthesize the very `git <mut>` the guard is built to deny, and no fixture
exercises an expansion-produced command. Same shape as r1→r4: close the enumerated cases, the property
leaks deeper. The r5 commit did NOT introduce these (r4 head dff604060 ALLOWs them identically), but the
review contract blocks on any executable canonical-mutation bypass regardless of provenance.

### Resolved since r4 (verified)
- **F1-R4 depth fail-OPEN → FIXED.** Entry-guard fail-closed (lib.rs:55-59) covers all 9 recursion sites
  + `eval`; deep nests DENY to *500; zero false positive on legit ≤4 nesting.
- **F2-R4 nested-escaped-backtick → FIXED.** Escape-aware scan (lib.rs:1281,1300); nested/triple/$()-mix
  variants DENY; double-escaped/`\<space>` ALLOWs are faithful (real git no-op, verified).
- Full r1/r2/r3 corpus, git restore, xargs, wrapper sh -c, GIT_DIR env-context — all remain DENY.

### Positive observations (reinforce)
- The r5 fix is precise, minimal, and exactly addresses both r4 findings. The fail-closed-at-entry design
  is the right structural primitive (a real wall, not a soft ceiling) and the escape-aware backtick code
  is correct. Docstrings (lib.rs:50-54,1279-1299) are accurate.
- Zero false positives across 34 legit commands (full merge-train, retrain.sh, wrapped tools, worktree
  ops, deep-but-legit nesting). 29 + liveness green, LSP clean, SSH-signed, faces-last, no key laundering.

---

## Residual risk (single most likely production failure if merged as-is)
An agent (or an injected instruction via the lethal-trifecta surface) emits a one-line command in which
the git mutation is produced by a shell expansion the tokenizer can't see through — e.g.
`$(echo git -C <canonical> reset --hard)`, `eval $(echo git -C <canonical> checkout <branch>)`,
`git -C <canonical> $'reset' --hard`, or `g=git; $g -C <canonical> reset --hard` — and it silently
ALLOWs, re-contaminating the canonical checkout and reproducing FRIC-022/FRIC-1781062867 while all 29
tests and the liveness gate stay green (no expansion-produced-command fixture exists to catch it).
Secondary (carried, unchanged): the hook shim fails OPEN if the Rust binary is unbuilt — ensure
CI/branch-protection builds it (structural enforcement, not hook reliance).

## Required to clear
1. Close F1-R5: model substitution-as-command — when a `$(…)`/`` `…` ``/`eval $(…)` sits in command
   position (or is eval's argument) and its literal output is a git invocation, DENY; minimally, fail
   CLOSED when a command-position word is an unresolved substitution that could yield git.
2. Close F2-R5: resolve or fail-closed on ANSI-C `$'…'`, `$VAR`/`${VAR…}`, same-line assignments, and
   substitution in the command-name/subcommand position before the literal `git`/subcommand match.
3. Add fixtures asserting DENY for EVERY confirmed reproduction above (10 forms). The current fixtures
   structurally cannot catch any of them — exactly why they leaked past 29-green (the r1→r5 pattern).
4. Re-run 29 + liveness + new fixtures; re-drive the Task-3 substitution/expansion sweep; confirm zero
   false positives on the merge-train + retrain.sh set (esp. legit `$(…)` reads and `eval` of non-git).

VERDICT: **BLOCK** — two HIGH bypass classes (substitution-as-command / `eval`+`$()`; and
expansion-produced command word/subcommand via ANSI-C, `$VAR`, `${x:-…}`, substitution-as-subcommand),
10 confirmed minimal reproductions, each proven to mutate the canonical checkout in real bash/zsh/sh
while the guard ALLOWs. The r5 fix itself is correct and complete for its stated scope — both r4
residuals are genuinely closed with zero false positives, 29 tests + liveness gate green, hygiene clean.
The remaining bypasses are pre-existing (not an r5 regression) but live and trivially exploitable; the
class is "the guard tokenizes literal commands but a shell expansion can synthesize the denied mutation."

## Security Checklist
- [x] No hardcoded secrets (lib.rs is pure policy logic; no new deps)
- [~] All inputs validated — shell command parsed via tokenizer; LITERAL forms sound, but EXPANSION-
      produced command words/subcommands (substitution-as-command, eval+$(), ANSI-C, $VAR, ${x:-…})
      bypass the literal match → F1/F2-R5 OPEN
- [~] Injection prevention — r4 depth + nested-backtick CLOSED; substitution-as-command + expansion-
      produced-token classes OPEN (10 confirmed canonical-mutation reproductions)
- [x] Authorization/policy enforced for literal commands — default-closed scan-through, env/GIT_DIR/-C/
      sh-c/depth machinery correct; zero false positives on 34 legit commands
- [x] Dependencies audited — single dep serde_json; no new deps; no CVE surface in this delta
- [x] Tests + liveness gate green (29 + 1); LSP clean; SSH-signed; faces-last; no key laundering
