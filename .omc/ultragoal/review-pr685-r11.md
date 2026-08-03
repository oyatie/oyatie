# Security Review — PR #685 ROUND 11 (G011 canonical-checkout guard) — CONVERGENCE CHECK

- Repo: jason931225/oyatie  Branch: agent/g011-checkout-guard  Head: 33958e907  Base: dev
- r11 delta vs r10 (346526755..33958e907): two commits.
  - `2bec1798b` `fix(checkout-guard): model set-- positionals + shell function arg binding (review #685 r10)`
    — lib.rs ONLY (+311/-13, no Cargo.lock churn). New binding model in `normalize_static_expansions`
    (lib.rs:1278): each fixpoint pass now (1) `inline_function_calls` (lib.rs:1343) — inlines same-line
    `name(){body}` / `function name{body}` calls, `substitute_positionals` (lib.rs:1465) replaces
    `$@`/`$*`/`$N`/`${N}` in the body with call args and DROPS `"` quotes so `"$@"` re-tokenises as
    separate words; (2) `collect_positional_params` (lib.rs:1307) captures `set -- <words>` and
    `expand_with_bindings` (lib.rs:1601) expands `$@`/`$*`/`$N` at top level; plus a
    `parse_git_invocation` backstop (lib.rs:785) — a `-C`/`--git-dir`/`--work-tree` target still carrying
    an unresolved expansion synthesises `UNRESOLVED_TARGET_SENTINEL` (lib.rs:12) so it fails closed
    instead of `None`→ALLOW. 10 new DENY fixtures (lib.rs:2611-2620).
  - `33958e907` `chore: settle generated cloud-ci faces` — 2 *.generated.json faces ONLY.
- Reviewer: fresh-context security-reviewer (Claude Opus), Torvalds/attacker lens,
  /using-superpowers + /using-agent-skills + security-and-hardening + /ultraqa, all FOREGROUND,
  built from /Users/jasonlee/oyatie-worktrees/g011-checkout-guard.

## VERDICT: BLOCK

**Risk Level: HIGH.** r11 closes all SEVEN r10 reproductions — every r10 BLOCK form (unquoted
`set -- … ; git $@` / `git -C $@`, `g(){ git $@; }; g -C <canon> …`, `command git` body, nested
self-contained `h(){ g(){…};…}`, the `eval` and `clean -fdx` analogues) now DENYs (rc=2) through the
PRODUCTION binary, with ZERO false positives. That part of the convergence claim is sound and the fix
direction (model bash binding) is correct.

BUT the r11 convergence claim — "Residual = runtime-unknowable $(prog)" — is **FALSE**. The new
binding model is INCOMPLETE: I constructed **NINE distinct statically-resolvable real-mutation
bypasses**, each verified DIRTY→guard ALLOW→real-bash CLEAN end-to-end through the PRODUCTION binary
with a foreign session-cwd (so mutation arrives ONLY via the `-C`/binding indirection, never via cwd)
and an argv-capturing fakegit. NONE uses a runtime-unknowable `$(prog)` — every word is a literal
constant in the one-liner. Per the strict VERDICT RULE ("BLOCK on a statically-resolvable
real-mutation bypass") this blocks.

The single most damning leak is a **DIRECT QUOTED-SIBLING REGRESSION of the r11 fixture itself**:
```
  set -- -C <canon> reset --hard; git $@      -> rc=2 DENY   (the r11 fixture, lib.rs:2613)
  set -- -C <canon> reset --hard; git "$@"    -> rc=0 ALLOW  *** LEAK *** (quoted sibling, mutates)
```
r11 added the UNQUOTED fixture and left the QUOTED (more idiomatic) form open. `substitute_positionals`
drops `"` quotes ONLY inside FUNCTION BODIES (lib.rs:1472); the TOP-LEVEL `expand_with_bindings` path
(lib.rs:1684/1693) expands a top-level `"$@"`/`"${@}"` to the JOINED string as ONE token (the `"` was
already consumed by `shell_tokens` before expansion, lib.rs:2393), so `git` receives a single mega-arg
`-C <canon> reset --hard` that never re-tokenises into `[-C][path][verb]` — no `-C` match, no
subcommand → fall-through → ALLOW.

**NEW CLASS or VARIANT?** VARIANT of the closed r10 F1 positional/function-arg-binding class at the
root, but it surfaces in three DISTINCT uncovered mechanics r11's model does not cover:
(A) quoted top-level positional/array NOT re-split into words; (B) positional-binding op `shift` not
modeled; (C) only ONE function-inline hop per fixpoint pass, so multi-hop / mutually-recursive chains
leak. It is the same r1→r10 meta-pattern one layer deeper: r11 modeled the BINDING but not the full
re-tokenisation/rebinding semantics that feed it.

---

## Harness (commands + exact exit codes)

Built the real binary: `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out
/tmp/r11-guard-bin` → BUILD SUCCEEDED (Mach-O arm64). Driven via JSON hook payload
`{"tool_input":{"command":"…"}}` on stdin (the real enforcement surface, main.rs run()),
`OYA_CANONICAL_CHECKOUT=/repo/oyatie`, `env -i` neutral env, binary cwd=/tmp (canonical != cwd).
rc=2 ⇒ DENY, rc=0 ⇒ ALLOW. Calibration verified: `git -C <canon> {reset --hard|switch|restore .|
checkout -- f|clean -fdx|stash pop}` → 2; `{status|fetch|log}` → 0; bare `git reset` from foreign cwd
→ 0 (does not touch canonical).

**Adjudication discipline (real-mutation only counts):** per candidate, (1) build a fresh DIRTY real
git repo (` M tracked.txt` + `?? untracked.txt`); (2) record guard rc via the real binary; (3) run the
EXACT command in a clean `bash -c` from a FOREIGN cwd with REAL git, canon path substituted to the
live repo; (4) diff `git status --porcelain` before/after. A FINDING requires `guard_rc=0 AND
mutated=yes`. Adjudicator self-test confirmed sound: `reset --hard`/`clean -fdx` via `-C` mutate the
dirty repo, `status`/`log` and bare `git reset` from foreign cwd do not. Foreign-cwd discipline
eliminated harness artifacts (the earlier `alias git` "leak" was a cwd artifact — non-interactive
`bash -c` does NOT expand aliases; from a foreign cwd it is `mutated=no`, faithful ALLOW).

### TASK 1 — full accumulated corpus (r1..r10 named + ALL reproductions incl 7 r10 set--/function cases) — PASS
60-command corpus (r1 basics; r2 wrappers flock/runuser/cpulimit/timeout/nice/nohup/xargs/systemd-run/
firejail/eatmydata; r3 sh -c/GIT_DIR/GIT_WORK_TREE; r4 nested escaped backtick + depth; r5
`$(echo git)`/backtick/`eval $()`/`bash -c "$()"`/ANSI-C/$VAR/`${x:-}`/`"$(printf)"`; r6 nested
fixpoint + `${x:=}`/`${x:+}`/`${x/a/e}`; r7 brace/glob; r8 IFS; r9 positional-as-verb + ANSI-C
hex/octal + line-continuation; r10 ALL SEVEN — `set -- -C <canon> reset --hard; git $@`,
`set -- <canon> reset --hard; git -C $@`, `…; eval git -C $@`, `g(){ git "$@"; }; g -C <canon> …`,
unquoted `$@`, `command git` body, nested `h`, `clean -fdx` analogue):
- **PASS=60  FAIL=0** real reproductions ALL DENY (rc=2). (A lone bare `eval git -C $@` with NO
  preceding `set --` rc=0 is FAITHFUL — empty `$@`, real bash prints git usage rc=129, NO mutation;
  the actual r10 case #3 `set -- <canon> reset --hard; eval git -C $@` DENYs.) All SEVEN r10 BLOCK
  forms RESOLVED through the production binary.

### TASK 2 — CONVERGENCE TEST — NINE NEW STATICALLY-RESOLVABLE REAL-MUTATION BYPASSES (BLOCK)
Adjudicated every binding/word-synthesis mechanism the prompt named vs REAL bash (foreign cwd) via the
production binary. Mechanisms that DENY (caught) or ALLOW-without-mutation (faithful):
```
  nested-DEF self-contained h(){ g(){ git "$@";}; g "$@";}; h …  rc=2  (inlined in one body — DENY)
  nested braces in body g(){ { git "$@"; }; }; …                 rc=2  (DENY)
  git redefining itself git(){ command git "$@"; }; git …        rc=2  (DENY)
  heredoc / ;-chained / true; git in body                        rc=2  (DENY)
  local x=1 / declare x=1 in body                                rc=2  (DENY)
  func via var name f=g; … ; $f -C <canon> …                     rc=2  (DENY)
  function-kw form / function g(){…}                             rc=2  (DENY)
  body $1 $2 / ${1} ${2} / $* (unquoted)                         rc=2  (DENY)
  set -- in SUBSHELL ( set -- … ); git -C $@                     rc=2 mut=no (positionals don't escape — faithful)
  body "$*" (single joined word)                                 rc=2 mut=no (DENY anyway)
  set -- a..i <canon> reset --hard; git -C ${10} ${11} ${12}     rc=2  (DENY)
  set -- <canon> reset --hard; git -C $1 $2 $3                   rc=2  (DENY)
  getopts loop then set --; git -C $@                            rc=2  (DENY)
  alias git='git -C <canon>'; git reset --hard                  rc=0 mut=no (non-interactive bash -c does NOT expand aliases — faithful)
```
CONFIRMED LEAKS (guard_rc=0 ALLOW; REAL canonical mutation, foreign cwd; production binary):
```
ROOT A — quoted top-level positional/array NOT re-split into words (4):
  rc=0 mut=yes  set -- -C <canon> reset --hard; git "$@"             argv=[-C <canon> reset --hard]  *** quoted sibling of r11 fixture ***
  rc=0 mut=yes  set -- -C <canon> reset --hard; git "${@}"           argv=[-C <canon> reset --hard]
  rc=0 mut=yes  read -ra P <<< "-C <canon> reset --hard"; git "${P[@]}"  argv=[-C <canon> reset --hard]  (read-array binding not modeled)
  rc=0 mut=yes  read -a  P <<< '-C <canon> reset --hard'; git "${P[@]}"  argv=[-C <canon> reset --hard]
ROOT B — positional-binding op `shift` not modeled (3):
  rc=0 mut=yes  set -- X <canon> reset --hard; shift; git -C $@       argv=[-C <canon> reset --hard]
  rc=0 mut=yes  set -- A B <canon> reset --hard; shift 2; git -C $@   argv=[-C <canon> reset --hard]
  rc=0 mut=yes  g(){ shift; git "$@"; }; g X -C <canon> reset --hard  argv=[-C <canon> reset --hard]
ROOT C — only ONE function-inline hop per fixpoint pass; multi-hop / mutual recursion leaks (2):
  rc=0 mut=yes  a(){ b "$@"; }; b(){ git "$@"; }; a -C <canon> reset --hard            argv=[-C <canon> reset --hard]
  rc=0 mut=yes  x(){ y "$@";}; y(){ z "$@";}; z(){ git "$@";}; x -C <canon> reset --hard argv=[-C <canon> reset --hard]
  (clean -fdx analogue of ROOT A also confirmed: untracked.txt DESTROYED)
```
Argv-capture fakegit confirms bash hands git `[-C] [<canon>] [reset] [--hard]` in each case.

End-to-end proof (production binary, foreign cwd), DIRTY→guard ALLOW→CLEAN:
```
  BEFORE:  M tracked.txt | ?? untracked.txt
  GUARD rc=0 (ALLOW)   cmd: set -- -C <canon> reset --hard; git "$@"
  AFTER:  ?? untracked.txt          (the ` M` is gone — reset --hard executed on canonical)
  ---
  GUARD rc=0 (ALLOW)   cmd: set -- X <canon> reset --hard; shift; git -C $@   →  ` M` gone
  GUARD rc=0 (ALLOW)   cmd: read -ra P <<< "-C <canon> reset --hard"; git "${P[@]}"  →  ` M` gone
  GUARD rc=0 (ALLOW)   cmd: h(){ g "$@"; }; g(){ git "$@"; }; h -C <canon> reset --hard  →  ` M` gone
  GUARD rc=0 (ALLOW)   cmd: set -- -C <canon> clean -fdx; git "$@"   →  untracked.txt DESTROYED
```

Leak-boundary map (isolates the gap):
```
  set -- -C <canon> reset --hard; git $@      rc=2 DENY  (unquoted — words re-split, sigil seen)
  set -- -C <canon> reset --hard; git "$@"    rc=0 LEAK  (quoted — ONE joined token, never re-split)
  g(){ git "$@"; }; g -C <canon> reset --hard rc=2 DENY  (FUNCTION body — substitute_positionals drops ")
  h(){ g "$@"; }; g(){ git "$@"; }; h …       rc=0 LEAK  (2 hops — only 1 inlined per fixpoint pass)
  set -- <canon> reset --hard; git -C $1 $2   rc=2 DENY  (no shift — positionals indexed straight)
  set -- X <canon> reset; shift; git -C $@    rc=0 LEAK  (shift reindex not modeled)
```
Discriminator: a LEAK occurs precisely when the verb+`-C`-target reassemble into git's argv via a
binding step r11's model does NOT replay — top-level quote-stripping/array re-split, `shift`
reindexing, or a 2nd function hop.

Root cause (lib.rs):
- ROOT A: `substitute_positionals` (lib.rs:1472) drops `"` only inside FUNCTION BODIES. The top-level
  `expand_with_bindings` `$@`/`${@}` arms (lib.rs:1670/1684/1693) emit the JOINED positional string;
  `shell_tokens` (lib.rs:2393) already consumed the surrounding `"`, so the result is ONE token. git's
  first arg is the mega-string `-C <canon> reset --hard`, never matched as `-C`/subcommand → ALLOW.
  `read … <<<; "${P[@]}"` array binding is not modeled at all (no `read`/array tracking).
- ROOT B: `collect_positional_params` (lib.rs:1307) captures only `set -- <words>`; `shift`/`shift N`
  reindexing is not applied, so `$@` after `shift` still expands to the ORIGINAL words (which here
  still contain the canonical+verb but offset/garbled enough that the literal `-C $@` form swallows the
  remainder and parse_git_invocation's sentinel does not fire for the reconstructed argv).
- ROOT C: `inline_function_calls` (lib.rs:1343) inlines one call layer; `collect_function_defs` runs on
  the raw string each pass, but a call to a SECOND top-level function inside an inlined body is not
  re-inlined within the bounded fixpoint before the git decision, so multi-hop/mutual recursion leaks.

RESIDUAL genuinely runtime-unknowable (`$(prog)`/`$(cat file)` opaque stdout) remains accepted — BUT
it is NOT the sole residual: the 9 forms above are parse-time-determinable and BLOCK. Carried/observed
(NOT counted, runtime-unknowable family, pre-existing, unchanged): PATH/symlink/alias `g`→git binary
(needs filesystem resolution); interactive-only `alias` (non-interactive `bash -c` does not expand).

### TASK 3 — FALSE-POSITIVE SWEEP (must ALLOW) — PASS, ZERO false positives
- 37/37 legit ALLOW: full merge-train on canonical (`status/log/diff/show/rev-parse/fetch --all
  --prune/commit -m "…"/push/merge --ff-only/pull --ff-only`), `set -- a b c; echo $@`, function defs
  that DON'T call git, reads with positional/`$N` args (`git -C <c> log $@`, `g(){ git -C <c> log
  "$@"; }; g --oneline`), for/while loops, `$(date)`, `$EDITOR`, `git log $(git rev-parse HEAD)`,
  `sudo sh -c 'echo'`, `IFS=, read` forms.
- 12/12 HARD-FP probes ALLOW (the quote-drop + positional-expansion highest-FP surfaces): function
  bodies that run git READS via `"$@"` (`git -C <c> status "$@"`, `git -C <c> diff "$@"`), commit
  messages containing the words reset/clean/checkout, `git commit -m "$@"`,
  `set -- 'reset --hard'; git commit -m "$1"` (positional carrying a mutating-verb STRING used as a
  benign arg → correctly ALLOWs), NON-canonical worktree mutations via positional/function binding
  (`set -- reset --hard; git -C /…/wt $@`, `g(){ git "$@"; }; g -C /…/wt reset --hard` → ALLOW), and
  `reset(){ echo "$@"; }; reset --hard` (a non-git function named to collide).
- **`grep -r git /path` stays ALLOW** (3 forms: `grep -r git /etc`, `grep -rn git /tmp`, `rg git /var/log`).
- **`IFS=, read` not over-denied** (`IFS=, read -ra parts …; git -C <c> log ${parts[0]}`, `while IFS=,
  read …`).
- /tmp/retrain.sh (present) — 10/10 git lines ALLOW (ls-remote/fetch/clean/checkout/add/commit/push/
  merge-base/rev-parse against the live repo+worktree; none target the calibrated canonical).
- **TASK3 RESULT: ALLOW-PASS=37 + 12 hard + 10 retrain = 0 FALSE POSITIVES.** The high-FP-risk
  quote-dropping + positional-expansion machinery does NOT mis-deny any legitimate command.

### TASK 4 — mechanics — all green
- `//tools/oya-checkout-guard-app:oya-checkout-guard-app-unittest` → **30 passed; 0 failed**
  (10 new r10 DENY fixtures present and pass). NOTE: NO fixture exercises the TASK-2 LEAK forms
  (quoted top-level `"$@"`/`"${@}"`, `read`-array, `shift`, multi-hop function) — the recurring
  r1→r10 reason the leak survives 30-green.
- `//cloud/cloud-ci/gates/oya-cloud-ci-enforcement-liveness-app:{…-unittest,…-gate}` →
  unittest Pass 1; gate **6/6 tests pass** (`enforcement_liveness_face_reports_current_tree_green`).
- SSH sigs: 2bec1798b + 33958e907 both carry BEGIN SSH SIGNATURE (Good ED25519
  SHA256:5grGNUtX9Zgmy1SWne6wF9DR8W1ElUQaF/Z8SYRz8E8). `%G?`=U is environmental (allowed_signers not
  provisioned in fresh context), same as r1-r10.
- Faces-last settle: 33958e907 touches ONLY 2 *.generated.json faces (accounting-registry, scm-facts).
  Fix commit 2bec1798b = lib.rs ONLY (no Cargo.lock churn). Holds.
- Baseline key-diff both ways (merge-base d705932d4 .. HEAD over all 4 changed generated faces):
  **0 anti-pattern keys** (forbidden_foundry|unjustified|unowned|unreachable) added OR removed. The
  only churn is provenance digest, `last_touch_commit` rotation, and a commit-timestamp row. No
  laundering.

---

## Findings (file:line, severity, confidence)

### [HIGH, confidence HIGH] F1-R11 — quoted top-level positional/array binding NOT re-split, plus `shift` and multi-hop function gaps (static, no $(prog))
- Location: lib.rs:1472 (`substitute_positionals` drops `"` ONLY in function bodies); lib.rs:1670/1684/
  1693 (top-level `expand_with_bindings` `$@`/`${@}` emit a JOINED single token — never re-split after
  the `"` was consumed by shell_tokens at lib.rs:2393); lib.rs:1307 (`collect_positional_params`
  ignores `shift`); lib.rs:1343 (`inline_function_calls` inlines only one hop per fixpoint pass); no
  `read`/array-binding modeling anywhere.
- Confirmed (rc=0; REAL bash MUTATES from foreign cwd; production binary, DIRTY→CLEAN): nine forms —
  ROOT A `git "$@"` / `git "${@}"` / `read -ra P …; git "${P[@]}"` / `read -a P …; git "${P[@]}"`;
  ROOT B `…; shift; git -C $@` / `…; shift 2; git -C $@` / `g(){ shift; git "$@"; }; g X -C <canon> …`;
  ROOT C `a(){ b "$@"; }; b(){ git "$@"; }; a -C <canon> …` / 3-deep chain; plus the `clean -fdx`
  analogue (untracked destroyed).
- Exploitability: local, unauthenticated, single short one-liner, no length cap, shell-agnostic
  (`set --`/`$@`/`"$@"`/`shift`/`read`/functions work in bash and sh). Trivially emitted by an agent
  or an injected instruction (the lethal-trifecta surface CLAUDE.md warns of). The quoted `"$@"` form
  is the MORE idiomatic spelling than the unquoted fixtured form.
- Blast radius: silent canonical `reset --hard` / `clean -fdx` / `restore .`, reproducing
  FRIC-022/FRIC-1781062867 while all 30 unit tests + the liveness gate stay green.
- STATICALLY RESOLVABLE: all words (`set --`/`read <<<`/function bodies+call args) are literal
  constants, fully visible at parse time. NOT the runtime-unknowable `$(prog)` residual — the BLOCK class.
- Classification: VARIANT of the closed r10 F1 positional/function-arg-binding class at the root, but a
  DISTINCT uncovered surface — r11 modeled the BINDING but not the full re-tokenisation/rebinding
  semantics: quoted top-level positional/array re-split, `shift` reindex, and the 2nd function hop.

### Note — the r1→r10→r11 meta-pattern, now at re-tokenisation/rebinding of the modeled binding
r11 correctly closed the seven r10-named forms and the fix direction (model bash binding) is right and
precision-clean (zero FP, `grep -r git /path` + `IFS=, read` preserved). But the convergence claim
again treats "the named forms modeled" as "static closure reached," which is false: r11 modeled
`set --` and one-hop function inlining but NOT (a) quote-stripping/array re-split for a TOP-LEVEL
quoted positional/`"${P[@]}"` (only function-body quotes are dropped), (b) `shift`/`shift N` reindex,
nor (c) multi-hop / mutually-recursive function chains (one inline hop per fixpoint pass). The durable
fix is to make the binding model REPLAY shell semantics end-to-end: expand positionals/arrays into
SEPARATE re-tokenised words regardless of quoting or scope, apply `shift`, and iterate function
inlining to a fixpoint (with a recursion bound that FAILS CLOSED on exhaustion, never ALLOW) — OR,
minimally, fail-closed whenever a `git` word's effective argv (after ANY binding step) cannot be proven
free of a canonical-targeted mutating verb.

### Resolved since r10 (verified)
- r10 F1 (positional/function-arg split, unquoted `$@`): the SEVEN r10 reproductions all DENY.
  `set -- -C <canon> reset --hard; git $@`, `set -- <canon> reset --hard; git -C $@`,
  `g(){ git "$@"; }; g -C <canon> …` (function body — quotes dropped), `command git` body, nested
  self-contained `h`, `eval` and `clean -fdx` analogues. (The COMPLEMENTARY quoted/`shift`/multi-hop
  forms are F1-R11, still open.)
- parse_git_invocation backstop (lib.rs:785): a `-C`/`--git-dir`/`--work-tree` target that is still a
  dynamic expansion synthesises `UNRESOLVED_TARGET_SENTINEL` → DENY instead of None→ALLOW. Verified for
  the unquoted `git -C $@` path.

### Positive observations
- r10-named fixes are general (not per-fixture), correct, and precision-clean — the right direction;
  `grep -r git /path` and `IFS=, read` preserved (0 FP across 37 legit + 12 hard + 10 retrain).
- 30 unit + liveness gate green (6/6), SSH-signed (Good ED25519), faces-last settle holds (2 faces,
  lib.rs-only fix, no Cargo.lock churn), no key laundering, single guard dep (serde_json), no new CVE
  surface.

---

## Residual risk (single most likely production failure if merged as-is)
An agent (or an injected instruction via the lethal-trifecta surface) emits
`bash -c 'set -- -C <canonical> reset --hard; git "$@"'` (the QUOTED sibling of the r11 fixture) or
`read -ra P <<< "-C <canonical> reset --hard"; git "${P[@]}"` — statically-resolvable, NO `$(prog)`,
NO covered metacharacter — and it silently ALLOWs, re-contaminating the canonical checkout and
reproducing FRIC-022/FRIC-1781062867 while all 30 tests and the liveness gate stay green. Because the
verb+`-C`-target reassemble into git's argv via a binding step r11 does not replay (top-level
quote-strip/array re-split, `shift`, or a 2nd function hop), neither scan sees a literal mutating verb
co-located with a literal `-C <canon>`.

Secondary (carried, unchanged): the hook shim fails OPEN if the Rust binary is unbuilt (main.rs maps
any error → SUCCESS); ensure CI/branch-protection builds it (structural enforcement, not hook
reliance). Carried, runtime-unknowable family: PATH/symlink alias `g`→git binary; interactive-only
`alias`; `$(prog)` opaque stdout — pre-existing, out of scope, not blocking on their own.

## Required to clear
1. Close F1-R11: make positional/array expansion produce SEPARATE re-tokenised words regardless of
   quoting or scope (top-level `"$@"`/`"${@}"`/`"${P[@]}"` must behave like the function-body path that
   already drops quotes); model `read … <<< <literal>` array binding; apply `shift`/`shift N` reindex;
   iterate `inline_function_calls` to a fixpoint so multi-hop / mutually-recursive chains resolve, with
   recursion exhaustion FAILING CLOSED. At minimum: fail-closed whenever a git word's effective argv
   (after ANY binding step) cannot be proven free of a canonical-targeted mutating verb.
2. Add a DENY fixture for EVERY confirmed F1-R11 reproduction (`git "$@"`, `git "${@}"`, the two
   `read`-array forms, the three `shift` forms, the two multi-hop forms, the `clean -fdx` analogue).
   Current fixtures structurally cannot catch them — the recurring r1→r10 failure mode.
3. Re-run 30 + liveness + new fixtures; re-drive the TASK-2 sweep through the real binary with a
   FOREIGN session-cwd; confirm zero false positives on the merge-train + retrain.sh + `IFS=, read` +
   `grep -r git /path` set.
4. Re-state the convergence claim honestly only after auditing ALL re-tokenisation/rebinding axes
   (quoting, arrays, `shift`, multi-hop inlining) in addition to the binding-capture axes — and confirm
   the sole residual is runtime-unknowable `$(prog)`/opaque-stdout.

## Security Checklist
- [x] No hardcoded secrets (lib.rs is pure policy logic; only dep is serde_json; no new deps)
- [~] All inputs validated — r10 axes (unquoted positional/function-arg binding) now sound; quoted
      top-level positional/array re-split, `shift` reindex, and multi-hop function inlining bypass
      static analysis → statically-resolvable real-mutation ALLOW (F1-R11)
- [~] Injection prevention — r10 classes CLOSED; F1-R11 OPEN (quoted-positional / shift / multi-hop
      canonical-mutation)
- [x] Authorization/policy enforced for modeled forms; zero false positives across 37 legit + 12 hard
      + 10 retrain commands; `grep -r git /path` and `IFS=, read` preserved
- [x] Dependencies audited — single dep serde_json; no new guard deps; no CVE surface in delta
- [x] Tests + liveness gate green (30 unit + gate 6/6 Pass 1/2); SSH-signed; faces-last (2 faces,
      lib.rs-only fix, no Cargo.lock churn); no key laundering
