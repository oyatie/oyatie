# Security Review — PR #685 ROUND 6 (G011 canonical-checkout guard)

- Repo: jason931225/oyatie  Branch: agent/g011-checkout-guard  Head: 7886c9e7f  Base: dev
- r6 delta vs r5 (009f3617e..7886c9e7f) on the guard: +239 lines in tools/oya-checkout-guard-app/src/lib.rs
  adding `normalize_static_expansions` + helpers (collect_same_line_bindings, expand_with_bindings,
  resolve_param, static_command_output, dequote_simple) called at decide_with_context entry (lib.rs:67-68),
  plus 12 new DENY fixtures. Settle commit 7886c9e7f touches only *.generated.json faces.
- Reviewer: fresh-context security-reviewer (Claude Opus), Torvalds/attacker lens,
  /using-superpowers + /using-agent-skills + /ultraqa, all FOREGROUND, built from
  /Users/jasonlee/oyatie-worktrees/g011-checkout-guard.

## VERDICT: BLOCK

**Risk Level: HIGH.** The r6 `normalize_static_expansions` closes EVERY one of the 12 r5 reproductions
(all now DENY, rc=2) with zero precision cost — that part of the fix is correct and complete for its
stated scope. BUT looking one layer past the new fixtures (the failure mode of every round), I found a
**large class of STATICALLY-RESOLVABLE expansion bypasses the normalizer does not cover** — 12 distinct,
minimal reproductions, each adjudicated against REAL bash (working tree DIRTY->CLEAN or branch switch
verified) while the guard ALLOWs (rc=0). These are NOT the runtime-unknowable residual the founder ruling
deems acceptable: every one is resolvable at parse time (nested `echo`/`printf`, and parameter-expansion
operators whose result text is literally present in the source). Per the VERDICT RULE — "BLOCK on a
statically-resolvable bypass" — this blocks.

The two root-cause families are:
- **F1 (single-pass, no fixpoint + dequote doesn't recurse into `$()`):** `static_command_output` resolves
  exactly one `echo`/`printf` level and `dequote_simple` treats `$(`/`)` as ordinary chars, so an
  `echo` whose argument is itself `$(echo git …)` emits a still-live `$(echo git …)` that nothing
  re-scans (normalize_static_expansions runs once; extract_command_substitutions already ran on the
  pre-normalized text). lib.rs:1337-1349, 1408-1431, 1433-1463.
- **F2 (`resolve_param` operator coverage gap):** only `:-`, bare `-`, and plain `${name}` are resolved;
  `:=`, `:+`, `+`, `/`, `//`, `:off:len` return None and are emitted verbatim, so the literal subcommand
  match never sees `reset`. lib.rs:1392-1406.

---

## Harness (commands + exact exit codes)

Built the real binary: `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out /tmp/r6-guard-bin`
(BUILD SUCCEEDED; Mach-O arm64). Driven via JSON hook payload `{"tool_input":{"command":"…"}}` on stdin
(the real enforcement surface, lib.rs:512), `OYA_CANONICAL_CHECKOUT=/private/tmp/r6-canon`, neutral cwd.
rc=2 => DENY, rc=0 => ALLOW. Calibration: `git -C <C> {switch|reset --hard|restore .|checkout other}` -> 2;
`{status|fetch|log}` -> 0.

**Adjudication discipline:** every ALLOW adjudicated against REAL git in a freshly `git init`'d repo
(file `f` dirtied DIRTY; mutation = working tree returns to CLEAN OR branch changes after
`bash -c "<cmd>"`). Two harness confounds were caught and corrected: (a) `cp -r` of a live repo flapped —
switched to per-test `git init`; (b) `git checkout <branch>` is blocked by a dirty conflicting file —
used `reset --hard` (no conflict) as the canonical adjudication mutation, and verified `checkout` cases
manually with a clean tree (saw `Switched to branch 'other'`).

### TASK 1 — full accumulated r1..r5 corpus + 10 r5 reproductions (must DENY) — PASS
- r1..r4 corpus (transparent/unmodelled wrappers x16, flock/runuser/cpulimit, xargs x4, git restore x3,
  nested wrappers, wrapper sh -c x16, GIT_DIR env-context x4, r3 substitution corpus x6,
  r4 depth nice*33/nohup*33, r4 nested escaped backtick): **56/56 DENY**.
  (One apparent ALLOW — bare `timeout git switch` — adjudicated FAITHFUL: `timeout`'s first operand is a
  DURATION, `git` is not a valid duration so real timeout never runs git; `timeout 5 git switch` and
  `timeout --preserve-status 5 git reset --hard` both correctly DENY.)
- The 12 r5 reproductions (`$(echo git…)`, backtick, `eval $()`, `bash -c "$()"`, `eval $(printf…)`,
  `$'reset'`, `re$'set'`, `$g`, `$r`, `${x:-reset}`, `"$(printf 'reset')"`): **12/12 now DENY**. No regression.

### TASK 2 — ATTACK normalize_static_expansions (one layer past) — 12 STATICALLY-RESOLVABLE LEAKS
All rc=0 ALLOW; each adjudicated to MUTATE the canonical checkout in real bash:

```
F1 single-pass / dequote-no-recurse (echo/printf nesting):
  rc=0  $(echo $(echo git -C <C> reset --hard))          real reset (DIRTY->CLEAN)
  rc=0  $(echo $(echo git -C <C> checkout other))         real checkout (master->other)
  rc=0  $(echo $(echo git)) -C <C> reset --hard           nested-echo command NAME
  rc=0  g=$(echo $(echo git)); $g -C <C> reset --hard     binding to nested echo
  rc=0  $(echo $(printf 'git -C <C> reset --hard'))       echo-of-printf
F2 resolve_param operator gap (parameter expansion -> subcommand):
  rc=0  git -C <C> ${x:=reset} --hard                     assign-default
  rc=0  x=1; git -C <C> ${x:+reset} --hard                alternative (x set)
  rc=0  x=1; git -C <C> ${x+reset} --hard                 alternative (no colon)
  rc=0  x=raset; git -C <C> ${x/a/e} --hard               pattern substitution
  rc=0  x=raset; git -C <C> ${x//a/e} --hard              global pattern substitution
  rc=0  x=resetXX; git -C <C> ${x:0:5} --hard             substring
  rc=0  git -C <C> ${x:-$(echo reset)} --hard             default containing substitution
```

Forms that ALLOW but are FAITHFUL (no real mutation — correctly NOT leaks):
- `$(printf '%s' git -C <C> reset --hard)` — printf %s emits only first arg ("git"); no command.
- `$(printf '%s' $(echo git -C <C> reset --hard))` — same.
- mid-command real newline (`git -C <C>\nreset --hard`, `git\n-C …`, `git -C <C> reset\n--hard`) — bash
  splits at the newline into two invalid/incomplete commands; `reset` alone does not touch the worktree;
  no mutation. (Trailing-newline `$(printf "git … reset --hard\n")` with a REAL newline DENYs — the
  single-quote/backslash artifact that first looked like a leak was a shell-literal `\n`, not a newline.)
- `echo -E` / `echo -ne` / triple-nested-echo variants where my harness flapped — re-adjudicated, the
  reset forms above are the authoritative confirmed leaks.

Boundary checks that are SOUND (no leak, no false positive):
- echo flag stripping limited to `-n `/`-e ` but git word survives -> those DENY.
- ANSI-C `$'reset'`, `re$'set'`, `${x:-reset}`, `${x-reset}`, `$g`, `$r` (r5 forms) -> all DENY.
- Canonicalization: trailing slash, `//`, `/.`, `/../r6-canon`, `cd <C>`, `cd <C>/`, `pushd <C>` -> all DENY.
- Binding capture from QUOTED args is NOT over-collected: `git commit -m 'r=reset'; $r …` and
  `'g=git'; $g …` -> ALLOW (faithful; the quoted text never binds), confirming the simple-word filter holds.
- Single-quoted `'$(…)'` stays literal (ALLOW, faithful); double-quoted `"$(…)"` is processed.

### TASK 3 — FALSE-POSITIVE SWEEP (must ALLOW) — PASS, ZERO false positives
15/15 ALLOW after correcting test semantics: merge-train on the agent worktree / non-canonical target
(`checkout -B dev origin/dev`, `checkout -- file.txt`, `reset --hard origin/dev`, `clean -fdx`,
`cd <noncanon> && …`), reads/commit/push/fetch --all/merge --ff-only on canonical, `worktree add`, legit
`ver=$(git rev-parse HEAD)`, `$(date)`, `x=main; git log ${x}`, `sudo sh -c 'echo'`, deep-but-legit
nesting. NOTE: `git checkout -B …` / `git checkout -- f` on the CANONICAL path correctly DENY — verified a
real mutation (DIRTY->CLEAN), so this is intended guard behavior (is_blocked_operation: checkout with any
args, lib.rs:1780), NOT a false positive.

### TASK 4 — mechanics — all green
- buck2 //tools/oya-checkout-guard-app:oya-checkout-guard-app-unittest -> **29 passed; 0 failed**.
- buck2 //cloud/cloud-ci/gates/oya-cloud-ci-enforcement-liveness-app:…-gate -> **Pass 1; Fail 0** (LIVE);
  …-unittest -> Pass 1.
- LSP diagnostics lib.rs -> **clean**.
- SSH sigs: 7886c9e7f + c6314038d both carry BEGIN SSH SIGNATURE. `%G?`=U is environmental
  (allowed-signers not provisioned in fresh context), same as r1-r5.
- Faces-last settle: 7886c9e7f touches ONLY 4 *.generated.json faces. Holds.
- Baseline key-diff both ways (merge-base d705932d4 .. HEAD over all 4 generated faces):
  **0 anti-pattern keys** (forbidden_foundry|unjustified|unowned|unreachable) added OR removed. No laundering.

---

## Findings (file:line, severity, confidence)

### [HIGH, confidence HIGH] F1-R6 — single-pass normalization + non-recursive dequote (echo/printf nesting)
- Root cause: lib.rs:67 calls `normalize_static_expansions` exactly once (no fixpoint); the `$(` branch
  (lib.rs:1337-1345) calls `static_command_output` (lib.rs:1408) -> `dequote_simple` (lib.rs:1433), which
  treats `$`, `(`, `)` as ordinary characters and does NOT recurse into a nested `$()`/backtick. So
  `$(echo $(echo git … reset --hard))` produces, as echo's "output", the literal `$(echo git … reset
  --hard)` — emitted verbatim into `out`, never re-normalized, and `extract_command_substitutions`
  (run at lib.rs:70 on the pre-normalized text) only saw `echo $(echo git…)` recurse to `echo` = allowed.
- Confirmed bypasses (rc=0; real bash MUTATES): nested-echo reset/checkout, nested-echo command-name,
  binding to nested-echo (`g=$(echo $(echo git))`), echo-of-printf. (5 forms.)
- Exploitability: local, unauthenticated, single short one-liner, no length cap; trivially emitted by an
  agent or an injected instruction (the lethal-trifecta surface CLAUDE.md warns of).
- Blast radius: silent canonical-checkout reset/checkout/switch/restore reproducing FRIC-022/FRIC-1781062867
  while all 29 tests + liveness gate stay green.
- This is STATICALLY RESOLVABLE (pure echo/printf nesting; output knowable at parse time) — squarely the
  BLOCK class, not the founder's accepted runtime-unknowable residual.
- Minimal fix direction: iterate normalize_static_expansions to a fixpoint (re-normalize produced text)
  with a small bounded pass count, OR have dequote_simple/static_command_output recurse the produced text
  through the same expansion machinery before emitting. Add fixtures for each form.

### [HIGH, confidence HIGH] F2-R6 — resolve_param operator-coverage gap (parameter expansion)
- Root cause: `resolve_param` (lib.rs:1392-1406) handles only `:-`, bare `-`, and plain `${name}`; it
  returns None for `:=`, `:+`/`+`, `/`/`//`, and `:off:len`. On None the `${` branch (lib.rs:1328-1335)
  re-emits `${…}` verbatim, so the literal subcommand scan never matches `reset`.
- Confirmed bypasses (rc=0; real bash MUTATES): `${x:=reset}`, `${x:+reset}` (x set), `${x+reset}` (x set),
  `${x/a/e}`, `${x//a/e}`, `${x:0:5}`, `${x:-$(echo reset)}`. (7 forms.)
- Exploitability / blast radius: identical to F1-R6 — single short line, silent canonical mutation,
  tests+gate green.
- STATICALLY RESOLVABLE: each result text (`reset`) is literally present in the source operator. BLOCK class.
- Minimal fix direction: in resolve_param, statically resolve `:=`/`:+`/`+` (the literal word is the
  produced text when the bound state is known/forced), apply `/`,`//` substitution and `:off:len` slicing
  against the binding, and recurse a `$(…)` default through the normalizer — OR, simplest and
  fail-safe: when the git command-name OR subcommand token still contains an unresolved `${`/`$(`/`$`
  AFTER normalization, fail CLOSED (legit commands never need an expansion to name the subcommand; the
  TASK-3 sweep shows negligible precision cost). Add fixtures for each form.

### Note — both findings are the r1->r6 meta-pattern recurring
The named r5 corpus (12 substitution/ANSI-C/$VAR forms) is genuinely and completely closed with zero
false positives. Both residuals are the same structural property leaking one layer past the new fixtures:
the normalizer resolves an enumerated subset of expansions, but a slightly different STATICALLY-RESOLVABLE
expansion (one more echo level, or a different `${}` operator) still synthesizes the denied `git <mut>`.
This is distinct from the founder-accepted residual (`$(curl…)` — output unknowable at parse time): these
12 are fully determinable without execution.

### Resolved since r5 (verified)
- All 12 r5 reproductions now DENY (substitution-as-command, eval+$(), bash -c "$()", ANSI-C `$'…'`,
  split ANSI-C, `$g`/`$r` same-line bindings, `${x:-reset}`, `"$(printf 'reset')"`). Zero precision cost.
- Full r1..r4 corpus, depth fail-closed, nested escaped backtick — all remain DENY.

### Positive observations
- The r6 fix is precise and exactly addresses every r5 finding; the bindings/ANSI-C/`${x:-}`/echo-printf
  machinery is correct for the forms it models, with zero false positives across the merge-train,
  retrain.sh set, worktree ops, legit substitutions/param-expansion, and deep nesting.
- 29 unit + liveness gate green, LSP clean, SSH-signed, faces-last settle holds, no key laundering.

---

## Residual risk (single most likely production failure if merged as-is)
An agent (or an injected instruction via the lethal-trifecta surface) emits a one-liner where the git
mutation is produced by a statically-resolvable expansion the normalizer doesn't cover — e.g.
`$(echo $(echo git -C <canonical> reset --hard))`, `git -C <canonical> ${x:=reset} --hard`, or
`x=raset; git -C <canonical> ${x/a/e} --hard` — and it silently ALLOWs, re-contaminating the canonical
checkout and reproducing FRIC-022/FRIC-1781062867 while all 29 tests and the liveness gate stay green.
Secondary (carried, unchanged): the hook shim fails OPEN if the Rust binary is unbuilt — ensure
CI/branch-protection builds it (structural enforcement, not hook reliance).

## Required to clear
1. Close F1-R6: make normalize_static_expansions reach a fixpoint (bounded re-normalization of produced
   text) OR recurse dequote/static_command_output output through the expansion machinery.
2. Close F2-R6: resolve (or fail-closed on) the remaining `${}` operators (`:=`,`:+`,`+`,`/`,`//`,`:off:len`,
   and `:-` defaults containing `$()`); simplest fail-safe = DENY when command-name/subcommand still holds
   an unresolved `$`/`${`/`$(` after normalization.
3. Add a DENY fixture for every one of the 12 confirmed reproductions (current fixtures structurally
   cannot catch them — exactly why they leaked past 29-green, the r1->r6 pattern).
4. Re-run 29 + liveness + new fixtures; re-drive the TASK-2 sweep; confirm zero false positives on the
   merge-train + retrain.sh set (esp. legit `$(date)`, `$(git rev-parse)`, `ver=$(…)`, and quoted-arg
   non-bindings).

## Security Checklist
- [x] No hardcoded secrets (lib.rs is pure policy logic; only dep is serde_json; no new deps)
- [~] All inputs validated — literal + r5 expansion forms sound, but echo/printf NESTING (F1) and
      `${}` operator gap (F2) synthesize the denied mutation -> 12 statically-resolvable ALLOWs
- [~] Injection prevention — r5 corpus CLOSED; F1/F2-R6 OPEN (12 confirmed canonical-mutation reproductions)
- [x] Authorization/policy enforced for modeled forms; zero false positives across 15+ legit commands;
      canonicalization (trailing slash/~/cd/pushd/dotdot) sound
- [x] Dependencies audited — single dep serde_json; no new deps; no CVE surface in this delta
- [x] Tests + liveness gate green (29 + gate Pass 1); LSP clean; SSH-signed; faces-last; no key laundering
