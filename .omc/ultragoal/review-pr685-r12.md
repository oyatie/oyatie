# Security Review — PR #685 ROUND 12 (G011 canonical-checkout guard) — CONVERGENCE CHECK

- Repo: jason931225/oyatie  Branch: agent/g011-checkout-guard  Head: ffca115dd  Base: dev
- r12 delta vs r11 (33958e907..ffca115dd): two commits.
  - `b5133a7f9` `fix(checkout-guard): model quoted positionals, read-arrays, shift, multi-hop functions (review #685 r11)`
    — lib.rs ONLY (+200/-2, no Cargo.lock churn). Closes the three r11 roots:
    (A) SURGICAL quoted re-split `quoted_split_expansion` (lib.rs:1710) — a double-quoted body that is
    EXACTLY `$@`/`$*`/`${@}`/`${*}` or a `${NAME[@|*|N]}` array subscript word-splits; everything else
    (incl. `bash -c "$(…)"`) keeps quotes; plus `read -a/-ra NAME <<< "literal"` array binding
    `collect_array_bindings` (lib.rs:1657) + `expand_array_subscript` (lib.rs:1733). (B) `shift [n]`
    reindex at top level (`collect_positional_params` lib.rs:1328) and `strip_leading_shift`
    (lib.rs:1416) inside inlined function bodies. (C) multi-hop function inlining to a depth-bounded
    fixpoint `inline_with_defs` (lib.rs:1362). 9 new DENY fixtures (lib.rs:2808-2818).
  - `ffca115dd` `chore: settle generated cloud-ci faces` — 2 *.generated.json faces ONLY.
- Reviewer: fresh-context security-reviewer (Claude Opus), Torvalds/attacker lens,
  /using-superpowers + /using-agent-skills + /ultraqa, all FOREGROUND, built from
  /Users/jasonlee/oyatie-worktrees/g011-checkout-guard. FOREIGN session-cwd (/tmp), mutation arrives
  ONLY via -C/binding indirection, every candidate confirmed against REAL bash with an
  argv-capturing fakegit (dirty->clean).

## VERDICT: BLOCK

**Risk Level: HIGH.** r12 closes all NINE r11 reproductions — every r11 BLOCK form (quoted top-level
`git "$@"`/`"${@}"`, `read -a/-ra … <<<` array + `"${P[@]}"`, top-level + in-body `shift`/`shift N`,
2-hop and 3-hop function chains) now DENYs (rc=2) through the PRODUCTION binary, with ZERO false
positives, and the stated `bash -c "$(…)"` regression is genuinely fixed (mutating subst DENYs, benign
subst ALLOWs). The fix direction (model bash binding/re-tokenisation) is correct and precision-clean.

BUT the r12 convergence claim — "Residual = runtime-unknowable $(prog)/read-from-stdin" — is **FALSE**.
The new binding model is STILL INCOMPLETE: I constructed **EIGHT distinct statically-resolvable
real-mutation bypasses**, each verified DIRTY→guard ALLOW→real-bash CLEAN end-to-end through the
PRODUCTION binary with a foreign session-cwd and an argv-capturing fakegit. NONE uses a
runtime-unknowable `$(prog)`/stdin — every word is a literal constant in the one-liner. Per the strict
VERDICT RULE ("BLOCK on a statically-resolvable real-mutation bypass") this blocks.

The single most damning leak is the **BASE CASE of variable binding** — a plain quoted string variable
with unquoted word-split, simpler than anything r11 raised:
```
  x=-C; y=/repo/oyatie; git $x $y reset --hard         -> rc=2 DENY  (single-token values captured)
  P="-C /repo/oyatie reset --hard"; git $P             -> rc=0 ALLOW *** LEAK *** (MUTATES canonical)
```
Root cause: `collect_same_line_bindings` (lib.rs:1623) splits on whitespace and its `value_ok` filter
only accepts `[A-Za-z0-9-_/.]`. A QUOTED MULTI-WORD value (`P="-C /repo/oyatie reset --hard"`) becomes
the whitespace-word `P="-C` whose value `"-C` contains `"` → rejected → the binding is DROPPED, `$P`
stays literal `$P`, no `-C`/canonical/verb is visible in the command text → fall-through → ALLOW. Real
bash binds `P` to the full string and `git $P` word-splits it into `[-C][<canon>][reset][--hard]`.

**NEW CLASS or VARIANT?** A mix. F1-R12(a)/(b)/(d)/(e) are **NEW CLASSES** r11 never reached (literal
ARRAY ASSIGNMENT `P=(…)`/`declare -a`, quoted/whitespace-bearing STRING-VAR word-split, `eval`-hidden
`set --`/binding, in-FUNCTION-BODY `set --` and `local x=` not scoped). F1-R12(c) (variable array
INDEX `${P[$n]}`) is a VARIANT of the r11 array-subscript surface (only literal-digit/@/* modeled). All
share the r1→r11 meta-pattern one layer deeper: r12 modeled the binding FORMS it enumerated but the
collectors are still ad-hoc allowlists, not a faithful replay of bash word-binding.

---

## Harness (commands + exact exit codes)

Built the real binary: `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out
/tmp/r12-guard-bin` → BUILD SUCCEEDED (Mach-O arm64). Driven via JSON hook payload
`{"tool_input":{"command":"…"}}` on stdin (the real enforcement surface), `env -i
OYA_CANONICAL_CHECKOUT=/repo/oyatie`, binary cwd=/tmp (canonical != cwd). rc=2 ⇒ DENY, rc=0 ⇒ ALLOW.
Calibration: `git -C <canon> {reset --hard|switch|clean -fdx}` → 2; `{status|fetch|log}` → 0; bare
`git reset --hard` from foreign cwd → 0.

**Adjudication discipline (real-mutation only counts):** per candidate (1) fresh DIRTY real git repo
(` M tracked.txt` + `?? untracked.txt`); (2) guard rc via real binary with literal `/repo/oyatie`;
(3) EXACT command in `bash -c` from FOREIGN cwd /tmp with REAL git, canon→live repo; (4) diff
`git status --porcelain`. A FINDING requires `guard_rc=0 AND mutated=yes`. Adjudicator self-test sound
(`-C <canon> reset --hard`/`clean -fdx` mutate; `status`/bare-foreign-`reset` clean). Argv-capturing
fakegit confirms bash hands git `[-C][<canon>][reset][--hard]` (or `clean -fdx`) in each leak.

### TASK 1 — full accumulated corpus (r1..r11 named + ALL reproductions incl 9 r11 cases) — PASS
43-form real-mutation corpus (r1 basics; r2 wrappers flock/timeout/nice/nohup/xargs/runuser/env; r3
sh -c/bash -c/GIT_DIR/GIT_WORK_TREE; r4 nested; r5 `$(echo git)`/`eval`/`bash -c "$()"`; r6 param-subst
`${x}`/`${x:-}`; r7 brace; r9 ANSI-C/line-continuation; r10 ALL SEVEN set--/function; r11 ALL NINE —
quoted `"$@"`/`"${@}"`, two read-array, three shift, two multi-hop):
- **All 43 real-mutation forms DENY (rc=2).** The 9 r11 reproductions are now FIXED through the
  production binary. (One self-inflicted malformed line `set -- X -C <c> reset --hard; shift; git -C $@`
  rc=0 is FAITHFUL: after shift `$@`=`-C <c> reset --hard`, so `git -C $@` = `git -C -C <c> reset
  --hard` and REAL git rejects `cannot change to '-C'` → mutated=no. The genuine r11 spelling
  `set -- X <c> reset --hard; shift; git -C $@` and the r12 fixture `…; shift; git "$@"` both DENY.)

### TASK 2 — CONVERGENCE TEST — EIGHT NEW STATICALLY-RESOLVABLE REAL-MUTATION BYPASSES (BLOCK)
Adjudicated every binding/word-synthesis mechanism the prompt named vs REAL bash (foreign cwd) via the
production binary. Mechanisms that DENY (caught) or ALLOW-without-mutation (faithful):
```
  read -ra P <<<; git ${P[0]} ${P[1]} ${P[2]} ${P[3]}           rc=2  (literal-digit index — DENY)
  read -ra P <<<; git "${P[0]}" "${P[1]}" "${P[2]}" "${P[3]}"   rc=2  (DENY)
  ${#P[@]} present then git "${P[@]}"                            rc=2  (DENY)
  mixed literal+array  git -C "${P[@]}" / git -C <c> "${P[@]}"  rc=2  (DENY)
  set -- <c> reset --hard; git -C "$1" "$2" "$3" / "${1}"…      rc=2  (DENY)
  function with if-guard / recursive base-case / { …; } body    rc=2  (DENY)
  git(){ command git "$@"; } self-redefine                       rc=2  (DENY)
  g(){ local x=1; git "$@"; } (local NOT used as arg)           rc=2  (DENY)
  function-kw / for x; do; getopts loop; eval set -- "$@" (after real set --)  rc=2  (DENY)
  top-level + in-body shift / shift N                            rc=2  (DENY)
  P=(… ); git "${P[*]}" (single joined word)                    rc=0 mut=no (real git rejects mega-arg — faithful)
  read -ra P (from stdin, no <<<)                                rc=0 mut=no (runtime-unknowable — faithful residual)
  git -C <c> $(echo reset) --hard                                rc=2  (static subst decoded — DENY)
```
CONFIRMED LEAKS (guard_rc=0 ALLOW; REAL canonical mutation, foreign cwd; production binary; all words
literal constants — NO $(prog), NO stdin):
```
NEW CLASS (a) — literal ARRAY ASSIGNMENT not modeled (collect_array_bindings only handles read <<<):
  rc=0 mut=yes  P=(-C <c> reset --hard); git "${P[@]}"                argv=[-C <c> reset --hard]
  rc=0 mut=yes  P=(-C <c> reset --hard); git ${P[@]}                  argv=[-C <c> reset --hard]
  rc=0 mut=yes  P=(-C <c> reset --hard); git ${P[*]}                  argv=[-C <c> reset --hard]
  rc=0 mut=yes  declare -a P=(-C <c> reset --hard); git "${P[@]}"     (also declare -a P=([0]=…), P+=(…), A=()/B=())
NEW CLASS (b) — quoted / whitespace-bearing STRING-VAR word-split not captured (value_ok rejects " and space):
  rc=0 mut=yes  P="-C <c> reset --hard"; git $P                       argv=[-C <c> reset --hard]
  rc=0 mut=yes  P=$(echo "-C <c> reset --hard"); git $P               (RHS static-subst, then split)
VARIANT (c) — variable array INDEX ${P[$n]} not modeled (only @/*/literal-digit):
  rc=0 mut=yes  read -ra P <<< "x -C <c> reset --hard"; n=1; git "${P[$n]}" "${P[2]}" "${P[3]}" "${P[4]}"
NEW CLASS (d) — eval-HIDDEN set --/binding (eval body not parsed for set --):
  rc=0 mut=yes  eval "set -- -C <c> reset --hard"; git "$@"           argv=[-C <c> reset --hard]
  rc=0 mut=yes  C="-C <c> reset --hard"; eval git $C                  (eval re-evaluates the string)
NEW CLASS (e) — in-FUNCTION-BODY set -- / local-as-arg not scoped:
  rc=0 mut=yes  g(){ set -- -C <c> reset --hard; git "$@"; }; g       (body set -- rebinds; not tracked)
  rc=0 mut=yes  r(){ local a=-C; git "$a" "$@"; }; r <c> reset --hard (local a not bound in body)
  (clean -fdx analogue of class (a) also confirmed: untracked.txt DESTROYED)
```
Argv-capture fakegit confirms bash hands git `[-C][<canon>][reset][--hard]` (or `clean -fdx`) in each.

End-to-end proof (production binary, foreign cwd, DIRTY→guard ALLOW→argv→CLEAN):
```
  guard_rc=0  P="-C <c> reset --hard"; git $P                       argv: -C <c> reset --hard   → ` M` gone
  guard_rc=0  P=(-C <c> reset --hard); git "${P[@]}"               argv: -C <c> reset --hard   → ` M` gone
  guard_rc=0  eval "set -- -C <c> reset --hard"; git "$@"          argv: -C <c> reset --hard   → ` M` gone
  guard_rc=0  g(){ set -- -C <c> reset --hard; git "$@"; }; g      argv: -C <c> reset --hard   → ` M` gone
  guard_rc=0  r(){ local a=-C; git "$a" "$@"; }; r <c> reset --hard argv: -C <c> reset --hard  → ` M` gone
  guard_rc=0  P=(-C <c> clean -fdx); git "${P[@]}"                 argv: -C <c> clean -fdx     → untracked.txt DESTROYED
```

Leak-boundary map (isolates the gap):
```
  x=-C; y=<c>; git $x $y reset --hard           rc=2 DENY  (single-token alnum values captured + expanded)
  P="-C <c> reset --hard"; git $P               rc=0 LEAK  (quoted multi-word value → binding dropped by value_ok)
  read -ra P <<< "…"; git "${P[@]}"             rc=2 DENY  (read-array modeled)
  P=(-C <c> reset --hard); git "${P[@]}"        rc=0 LEAK  (literal array assignment NOT modeled)
  read -ra P <<< "…"; git "${P[2]}"             rc=2 DENY  (literal-digit index modeled)
  read -ra P <<< "…"; n=1; git "${P[$n]}"       rc=0 LEAK  (variable index NOT modeled)
  set -- -C <c> reset --hard; git "$@"          rc=2 DENY  (top-level set -- captured)
  g(){ set -- -C <c> reset --hard; git "$@";};g rc=0 LEAK  (in-body set -- NOT scoped)
  eval set -- "$@" (after real set --)          rc=2 DENY  (preceding real set -- seen)
  eval "set -- -C <c> reset --hard"; git "$@"   rc=0 LEAK  (set -- hidden inside eval string)
```
Discriminator: a LEAK occurs precisely when the binding is established by a mechanism r12's ad-hoc
collectors do NOT enumerate — literal array assignment, a quoted/whitespace-bearing string value, a
variable array index, an `eval`-wrapped `set --`/command, or an in-function-body `set --`/`local`.

Root cause (lib.rs):
- (a) `collect_array_bindings` (lib.rs:1657) only matches `read -a/-ra NAME <<<`; bash native array
  assignment `NAME=(...)` / `declare -a NAME=(...)` / `NAME+=(...)` is never collected, so `${NAME[@]}`
  stays unresolved.
- (b) `collect_same_line_bindings` (lib.rs:1623) whitespace-splits then `value_ok` rejects any value
  containing `"`/space; a quoted multi-word RHS is dropped entirely, leaving `$P` literal.
- (c) `expand_array_subscript` (lib.rs:1733) only resolves `@`/`*`/literal-digit subscripts; a `$n`
  index is None → unresolved.
- (d) the `eval` arm decodes `eval <words>` but not an `eval "<string>"` whose string is itself a
  `set --`/binding statement; the hidden `set --` is never fed to `collect_positional_params`.
- (e) `collect_positional_params`/`collect_same_line_bindings` run on the WHOLE inlined string but do
  not RE-SCOPE a `set --`/`local` that appears INSIDE a function body before that body's own `$@`/`$N`
  bind, so an in-body `set --`/`local a=` is not applied to the body's git argv.

RESIDUAL genuinely runtime-unknowable (`read` from stdin/pipe, `$(prog)` opaque stdout) remains
accepted and behaves faithfully — BUT it is NOT the sole residual: the 8 classes above are
parse-time-determinable and BLOCK. Carried/unchanged (runtime-unknowable family, pre-existing): PATH/
symlink `g`→git binary; interactive-only `alias`; opaque `$(prog)`.

### TASK 3 — FALSE-POSITIVE SWEEP (must ALLOW) — PASS, ZERO false positives
- **40/40 legit ALLOW.** Full merge-train on canonical (`status/log/diff/show/rev-parse/fetch --all
  --prune/commit -m "…"/push/merge --ff-only/pull --ff-only`), commit messages containing the words
  reset/clean/checkout/`reset --hard`; reads with quoted positional/array/var args (the FP-risk
  surface): `set -- --oneline -5; git -C <c> log "$@"`, `g(){ git -C <c> log "$@"; }; g --oneline`,
  `git -C <c> log "$BRANCH"`, `read -ra P <<< '--oneline -5'; git -C <c> log "${P[@]}"`,
  `P=(--oneline -5); git -C <c> log "${P[@]}"`, `git -C <c> {diff,status} "$@"`; `set -- a b c;
  echo "$@"/echo $@`, `read -a P <<< 'a b c'; echo "${P[@]}"`, for/while loops, `reset(){ echo "$@";
  }; reset --hard` (non-git collide), `git -C <c> log $(git -C <c> rev-parse HEAD)`, `$(date)`,
  `$EDITOR`, `sudo sh -c 'echo'`, `IFS=, read -ra parts …; git -C <c> log ${parts[0]}`.
- **Non-canonical worktree mutations ALLOW** (4): `git -C <wt> reset --hard`, `set -- reset --hard;
  git -C <wt> "$@"`, `g(){ git "$@"; }; g -C <wt> reset --hard`, `P=(-C <wt> reset --hard);
  git "${P[@]}"`.
- **Positional carrying a verb STRING used as benign arg ALLOWs**: `set -- 'reset --hard'; git -C <c>
  commit -m "$1"`.
- **`grep -r git /path` / `rg git` stays ALLOW**; **regression `bash -c "$(echo 'git -C <c> status')"`
  ALLOWs** while the mutating `bash -c "$(echo 'git -C <c> reset --hard')"` DENYs (rc=2) — surgical
  quote-strip preserves `bash -c "$(…)"` grouping.
- **/tmp/retrain.sh present** — 10/10 git lines ALLOW (ls-remote/fetch/clean/checkout/add/commit/push/
  merge-base/rev-parse; none target the calibrated canonical `/repo/oyatie`).
- **TASK3 RESULT: ALLOW-PASS=40 legit + 10 retrain = 0 FALSE POSITIVES.** The high-FP-risk surgical
  quote-strip + array/shift machinery does NOT mis-deny any legitimate command.

### TASK 4 — mechanics — all green
- `//tools/oya-checkout-guard-app:oya-checkout-guard-app-unittest` → **30 passed; 0 failed** (9 new
  r11 DENY fixtures present and pass). NOTE: NO fixture exercises the TASK-2 LEAK forms (literal array
  assignment, quoted string-var split, variable array index, eval-hidden set--, in-body set--/local) —
  the recurring r1→r11 reason the leak survives 30-green (verified absent via grep).
- `//cloud/cloud-ci/gates/oya-cloud-ci-enforcement-liveness-app:{…-unittest,…-gate}` → unittest Pass 1;
  gate Pass 1 (Fail 0).
- SSH sigs: b5133a7f9 + ffca115dd both carry Good ED25519 signature
  (SHA256:5grGNUtX9Zgmy1SWne6wF9DR8W1ElUQaF/Z8SYRz8E8).
- Faces-last settle: ffca115dd touches ONLY 2 *.generated.json faces (accounting-registry, scm-facts).
  Fix commit b5133a7f9 = lib.rs ONLY (+200/-2, no Cargo.lock churn). Holds.
- Baseline key-diff both ways (merge-base d705932d4 .. HEAD over all 4 changed generated faces):
  **0 anti-pattern keys** (forbidden_foundry|unjustified|unowned|unreachable) added OR removed. No
  laundering.

---

## Findings (file:line, severity, confidence)

### [HIGH, confidence HIGH] F1-R12 — binding collectors are ad-hoc allowlists, not faithful word-binding: literal array assignment / quoted string-var split / variable array index / eval-hidden set-- / in-function-body set--/local all bypass static analysis (static, no $(prog))
- Location: lib.rs:1657 (`collect_array_bindings` matches ONLY `read -a/-ra NAME <<<`; native
  `NAME=(…)`/`declare -a`/`NAME+=(…)` unmodeled); lib.rs:1623 (`collect_same_line_bindings`
  whitespace-split + `value_ok` rejects `"`/space → quoted multi-word value dropped); lib.rs:1733
  (`expand_array_subscript` resolves only `@`/`*`/literal-digit — not `${P[$n]}`); the `eval` arm
  (does not parse an `eval "set -- …"`/`eval git $C` string for hidden binding); lib.rs:1328/1623
  (`collect_positional_params`/`collect_same_line_bindings` do not re-scope a `set --`/`local` that
  appears INSIDE a function body).
- Confirmed (rc=0; REAL bash MUTATES from foreign cwd; production binary, DIRTY→CLEAN; argv-capture):
  eight classes — (a) `P=(-C <c> reset --hard); git "${P[@]}"`/`${P[@]}`/`${P[*]}`/`declare -a`/`P+=`;
  (b) `P="-C <c> reset --hard"; git $P` and `P=$(echo "…"); git $P`; (c) `read -ra P <<< "x -C <c>
  reset --hard"; n=1; git "${P[$n]}" …`; (d) `eval "set -- -C <c> reset --hard"; git "$@"` and
  `C="-C <c> reset --hard"; eval git $C`; (e) `g(){ set -- -C <c> reset --hard; git "$@"; }; g` and
  `r(){ local a=-C; git "$a" "$@"; }; r <c> reset --hard`; plus the `clean -fdx` analogue (untracked
  destroyed).
- Exploitability: local, unauthenticated, single short one-liner, no length cap, shell-agnostic
  (`NAME=(…)`/quoted string var/`eval`/in-body `set --` work in bash; the string-var split works in
  sh too). Trivially emitted by an agent or an injected instruction (the lethal-trifecta surface
  CLAUDE.md warns of). The string-var form `P="…"; git $P` is the BASE CASE of variable binding —
  simpler and more idiomatic than any r11 form.
- Blast radius: silent canonical `reset --hard` / `clean -fdx` / `restore .`, reproducing
  FRIC-022/FRIC-1781062867 while all 30 unit tests + the liveness gate stay green.
- STATICALLY RESOLVABLE: all words (`NAME=(…)`, the quoted value, the `eval` string, the in-body
  `set --`, `n=1`) are literal constants visible at parse time. NOT the runtime-unknowable
  `$(prog)`/stdin residual — the BLOCK class.
- Classification: F1-R12(a)/(b)/(d)/(e) are NEW CLASSES r11 never reached (array assignment, quoted
  string-var split, eval-hidden binding, in-function-body scope); F1-R12(c) is a VARIANT of the r11
  array-subscript surface (variable vs literal index). Same r1→r11 meta-pattern: r12 modeled the
  enumerated binding forms but the collectors remain ad-hoc allowlists, not a faithful replay of bash
  word-binding.

### Note — the r1→…→r12 meta-pattern, now at the binding-COLLECTOR completeness boundary
r12 correctly closed the nine r11-named forms, fixed the `bash -c "$(…)"` regression, and is
precision-clean (0 FP across 40 legit + 10 retrain; `grep -r git /path`, `IFS=, read`, non-canonical
worktrees, quoted-arg reads all preserved). But the convergence claim again treats "the named forms
modeled" as "static closure reached," which is false: r12 added per-form collectors (`read`-array,
`shift`, multi-hop inline, surgical quoted split) but each is an allowlist that omits sibling spellings
of the SAME binding concept — native array assignment vs `read`-array, single-token vs quoted-multiword
string var, literal vs variable subscript, visible vs `eval`-hidden `set --`, top-level vs
in-function-body scope. The durable fix is to stop enumerating binding spellings and instead build the
git word's EFFECTIVE argv by replaying bash word-binding (all variable/array/positional assignments
incl. `NAME=(…)`, quoted values, `eval` strings, and function-body scopes), expanding into SEPARATE
re-tokenised words — OR, minimally, FAIL CLOSED whenever a `git` word is adjacent to ANY unresolved
`$VAR`/`${...}`/`"$@"`/`"${...[@]}"` expansion that could carry a `-C`/`--git-dir`/`--work-tree`
target plus a mutating verb (the parse_git_invocation sentinel already does this for the `-C $@` path;
extend it to bare `git $VAR`/`git "${ARR[@]}"` argv-position expansions of unknown content).

### Resolved since r11 (verified)
- r11 ROOT A (quoted top-level positional/array re-split): `git "$@"`/`"${@}"` and the two
  `read`-array forms all DENY. Surgical `quoted_split_expansion` works and does NOT break
  `bash -c "$(…)"` (regression fixed, confirmed both directions).
- r11 ROOT B (`shift`/`shift N`): top-level and in-function-body shift reindex all DENY.
- r11 ROOT C (multi-hop functions): 2-hop and 3-hop chains DENY via `inline_with_defs` fixpoint.

### Positive observations
- r11-named fixes are general (not per-fixture), correct, and precision-clean — the right direction;
  the `bash -c "$(…)"` regression is genuinely fixed; non-canonical worktree mutations and quoted-arg
  reads preserved (0 FP across 40 legit + 10 retrain).
- 30 unit + liveness gate green (gate Pass 1, unittest Pass 1), SSH-signed (Good ED25519), faces-last
  settle holds (2 faces, lib.rs-only fix, no Cargo.lock churn), no key laundering, single guard dep
  (serde_json), no new CVE surface.

---

## Residual risk (single most likely production failure if merged as-is)
An agent (or an injected instruction via the lethal-trifecta surface) emits
`bash -c 'P="-C <canonical> reset --hard"; git $P'` (the base-case string-var split) or
`bash -c 'P=(-C <canonical> reset --hard); git "${P[@]}"'` (literal array assignment) — both
statically-resolvable, NO `$(prog)`, NO `read`-from-stdin — and it silently ALLOWs, re-contaminating
the canonical checkout and reproducing FRIC-022/FRIC-1781062867 while all 30 tests and the liveness
gate stay green. Because the binding is established by a mechanism r12's ad-hoc collectors do not
enumerate, neither scan sees a literal `-C <canon>` co-located with a literal mutating verb.

Secondary (carried, unchanged): the hook shim fails OPEN if the Rust binary is unbuilt (main.rs maps
any error → SUCCESS); ensure CI/branch-protection builds it (structural enforcement, not hook
reliance). Carried, runtime-unknowable family: PATH/symlink alias `g`→git binary; interactive-only
`alias`; `read` from stdin/pipe; `$(prog)` opaque stdout — pre-existing, out of scope, not blocking on
their own.

## Required to clear
1. Close F1-R12: build the git word's EFFECTIVE argv by replaying bash word-binding rather than
   enumerating spellings — model native array assignment (`NAME=(…)`, `declare -a`, `NAME+=(…)`)
   alongside `read`-array; capture QUOTED/whitespace-bearing string-var values (drop the `value_ok`
   charset restriction; track the quoted span) so `git $P` splits them; resolve variable array indices
   `${P[$n]}` (the index is itself a tracked binding); parse `eval "<string>"` for hidden
   `set --`/binding; RE-SCOPE `set --`/`local` that appears inside a function body to that body. At
   minimum: FAIL CLOSED whenever a `git` word's argv-position expansion (`$VAR`/`${ARR[@]}`/`"$@"`)
   cannot be proven free of a canonical-targeted `-C`/`--git-dir`/`--work-tree` + mutating verb.
2. Add a DENY fixture for EVERY confirmed F1-R12 reproduction (`P=(…); git "${P[@]}"`/`${P[@]}`/
   `${P[*]}`/`declare -a`/`P+=`; `P="…"; git $P`; `P=$(echo "…"); git $P`; `${P[$n]}` variable index;
   `eval "set -- …"; git "$@"`; `eval git $C`; in-body `g(){ set -- …; git "$@"; }; g`;
   `r(){ local a=-C; git "$a" "$@"; }`; the `clean -fdx` analogue). Current fixtures structurally
   cannot catch them — the recurring r1→r11 failure mode.
3. Re-run 30 + liveness + new fixtures; re-drive the TASK-2 sweep through the real binary with a
   FOREIGN session-cwd; confirm zero false positives on the merge-train + retrain.sh + `IFS=, read` +
   `grep -r git /path` + quoted-arg-read + non-canonical-worktree set.
4. Re-state the convergence claim honestly only after auditing the binding-COLLECTOR completeness axis
   (every variable/array/positional binding SOURCE: native array assignment, quoted string values,
   variable subscripts, eval-hidden statements, function-body scopes) in addition to the
   expansion/re-tokenisation axes — and confirm the sole residual is runtime-unknowable
   `$(prog)`/read-from-stdin.

## Security Checklist
- [x] No hardcoded secrets (lib.rs is pure policy logic; only dep is serde_json; no new deps)
- [~] All inputs validated — r11 axes (quoted re-split, read-array, shift, multi-hop) now sound; but
      literal array assignment, quoted/whitespace string-var word-split, variable array index,
      eval-hidden set--, and in-function-body set--/local bypass static analysis →
      statically-resolvable real-mutation ALLOW (F1-R12)
- [~] Injection prevention — r11 classes CLOSED; F1-R12 OPEN (array-assignment / string-var-split /
      var-index / eval-hidden / in-body-scope canonical-mutation)
- [x] Authorization/policy enforced for modeled forms; zero false positives across 40 legit + 10
      retrain commands; `grep -r git /path`, `IFS=, read`, non-canonical worktrees, quoted-arg reads,
      and `bash -c "$(…)"` grouping all preserved
- [x] Dependencies audited — single dep serde_json; no new guard deps; no CVE surface in delta
- [x] Tests + liveness gate green (30 unit + gate Pass 1 / unittest Pass 1); SSH-signed (Good ED25519);
      faces-last (2 faces, lib.rs-only fix, no Cargo.lock churn); no key laundering
