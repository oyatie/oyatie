# Security Review — PR #685 ROUND 16 (G011 canonical-checkout guard) — CONVERGENCE CHECK

**Scope:** `tools/oya-checkout-guard-app/src/lib.rs` (the PreToolUse enforcement hook) + 2 generated cloud-ci faces.
PR jason931225/oyatie#685, branch `agent/g011-checkout-guard`, head `6d92de640`, base `dev`.
**Risk Level: HIGH.**

- r16 delta vs r15 (`9845ced76..6d92de640`): two commits.
  - `e307fbe52` `fix(checkout-guard): decode printf/echo-e/ANSI-C escapes into split whitespace (review #685 r15)`
    — **lib.rs ONLY (+47/-6, no Cargo.lock churn).** Adds `strip_quote_chars` (lib.rs:2342, removes `'`/`"` while
    KEEPING backslash escapes) and `normalize_split_ws` (lib.rs:2333, collapses `\t`/`\n`/`\r` → space). Rewires
    `static_command_output` printf (lib.rs:2207) and `echo -e`/`-ne`/`-en` (lib.rs:2186) to keep escapes, decode via
    `decode_ansi_c`, then `normalize_split_ws` so a decoded TAB/NL becomes an ARG boundary. Routes the ANSI-C `$'…'`
    binding-RHS branch (lib.rs:2025) through `emit_substitution_output(normalize_split_ws(decode_ansi_c(raw)))` so it
    re-quotes as one value then word-splits at the unquoted use. Adds 4 DENY fixtures (lib.rs:3101-3105).
  - `6d92de640` `chore: settle generated cloud-ci faces` — 2 `*.generated.json` faces ONLY.
- Reviewer: fresh-context security-reviewer (Claude Opus), Torvalds/attacker lens, /using-superpowers +
  /using-agent-skills + /ultraqa, FOREGROUND, built from `/Users/jasonlee/oyatie-worktrees/g011-checkout-guard`.
  FOREIGN session-cwd (`/tmp`); mutation arrives ONLY via `-C`/binding indirection; every candidate confirmed against
  REAL bash with an argv-capturing fakegit (dirty→clean) that EXECs real git so canonical mutation is GENUINE.
  Adjudicator self-test sound; leak-detection proven.

## VERDICT: BLOCK

r16 closes the ENTIRE r15 escape-decode class that was the r15 basis — all 14 r15 reproductions
(`printf "reset\t--hard"`, the backtick/`echo -e`/whole-argv variants, `P=$(printf …)` the r15 REGRESSION,
`printf %b`, subst-`+=`, `$'…'`-into-binding, and the `\n`/`\040`/`\x20`/`\011` siblings) now **DENY (rc=2)**
through the production binary. The 73-form accumulated corpus is **73/73 DENY**, the **r15 regression is FIXED**
(`P=$(printf "reset\t--hard"); git -C <c> $P` → rc=2), the FP sweep is **44/44 ALLOW, 0 false positives**, and
mechanics are green (31 unit + liveness Pass 1, Good ED25519, faces-last holds, 0 anti-pattern key churn). The
`strip_quote_chars` + `normalize_split_ws` surfaces introduced ZERO over-denial.

BUT the r16 convergence claim — "the faithful word-binding+escape model now holds; the sole residual is the
runtime-unknowable `$(cat)`/`$(curl)`/read-from-stdin" — is **FALSE**. The r16 fix models exactly the escape
shapes r15 named, but the guard's *static value-producing model* remains a strict subset of bash's. I constructed
**12 statically-resolvable real-mutation bypasses across FOUR new mechanisms** that use NO runtime-unknowable input.
Per the strict VERDICT RULE ("BLOCK on a statically-resolvable real-mutation bypass"), this blocks. None of these
is a regression vs r15 (r15's escape forms are all closed); all four are **fresh mechanisms** on the same r1→r15
META-pattern (the effective-argv model diverges from bash at a value construct the guard models imperfectly; the
multi-word bash result is flattened to one guard word and the trailing mutating flag is dropped).

The five single most-damning leaks (every byte statically determinable — NO `$(cat)`, NO `$(curl)`, NO stdin),
exact production-binary exit codes:
```
  git -C <c> $(printf "%-10s--hard" reset)         -> rc=0 ALLOW *** LEAK ***  (printf width/left-justify pad → spaces)
  git -C <c> $(printf -- "%s\n" reset --hard)      -> rc=0 ALLOW *** LEAK ***  (printf repeating format, multiple args)
  P=$'reset'$'\t--hard'; git -C <c> $P             -> rc=0 ALLOW *** LEAK ***  (concatenated ANSI-C $'…'$'…' into binding)
  P=resetZZhard; git -C <c> ${P/ZZ/ --}            -> rc=0 ALLOW *** LEAK ***  (${P/pat/repl} subst produces whitespace)
  shopt -s xpg_echo; git -C <c> $(echo "reset\t--hard") -> rc=0 ALLOW *** LEAK *** (xpg_echo plain-echo decode)
```
All MUTATE canonical end-to-end through the production binary from a FOREIGN cwd with an argv-capturing fakegit
(argv `[-C][<canon>][reset][--hard]` / `[…][clean][-fdx]`, DIRTY→CLEAN).

---

## Harness (commands + exact exit codes)

Built the real binary: `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out /tmp/r16-guard-bin`
→ BUILD SUCCEEDED (Mach-O arm64). Driven via JSON hook payload `{"tool_input":{"command":"…"}}` on stdin (the real
enforcement surface), `env OYA_CANONICAL_CHECKOUT=<canon>`, binary cwd=/tmp (canonical != cwd). rc=2 ⇒ DENY, rc=0
⇒ ALLOW. main.rs maps any guard ERROR → SUCCESS (fail-open), confirmed.

**Adjudication discipline (real-mutation only counts):** per candidate (1) fresh DIRTY real git repo
(` M tracked.txt` + `?? untracked.txt`); (2) guard rc via the real binary with the canonical path injected as a
LITERAL absolute path (placeholders `@CANON@`/`@WT@`/`@OTHER@` expanded identically for guard payload AND bash exec
so both see the same text); (3) the EXACT command in `bash -c` from FOREIGN cwd /tmp with an argv-capturing fakegit
that EXECs real git, so `-C <canon> reset --hard` genuinely mutates; (4) diff `git status --porcelain` before/after.
A LEAK requires `guard_rc=0 AND mutated=yes`. Adjudicator self-test sound: `git -C <canon> {reset --hard|clean
-fdx|restore .}` → rc=2 + argv proves mutation; `status`/`log` → rc=0 clean; `git -C <wt|other> reset --hard` → rc=0
clean (canonical untouched); fakegit EXEC of real git verified (WT `reset --hard` cleared ` M`, argv captured).
Leak-detection proven on the runtime-unknowable residual family.

### TASK 1 — full accumulated corpus (r1..r15 named + ALL reproductions incl r15 escape cases) — PASS
73-form real-mutation corpus (r1 basics; r2 wrappers/env; r3 sh -c/GIT_DIR/GIT_WORK_TREE; r4/r5 subst/eval;
r6 param-subst; r7 brace/r8 IFS; r9 ANSI-C direct; r10 set--/function; r11 read-ra here-string; r12 argv-position
binding; r13 ALL EIGHT static-subst-into-binding; r14 ALL `+=` + chained/double-hop; **r15 ALL escape-decode
reproductions** incl the regression `P=$(printf "reset\t--hard"); git -C <c> $P`, `printf %b`, subst-`+=`,
`$'…'`-into-binding, `echo -e`, backtick, whole-argv, and the `\n`/`\040`/`\x20`/`\011` siblings):
- **73/73 DENY (rc=2). 0 ALLOW.** The r15 escape-decode class is FIXED through the production binary.
- **r15 REGRESSION CONFIRMED FIXED:** `P=$(printf "reset\t--hard"); git -C <c> $P` → rc=2 DENY (r15 was rc=0).
- `stash pop`/`stash apply` correctly DENY (rc=2). `stash` save permitted (recoverable), as in r15.

### TASK 2 — CONVERGENCE TEST — TWELVE STATICALLY-RESOLVABLE REAL-MUTATION BYPASSES (BLOCK); r15 escape class CLOSED
Adjudicated every escape/value-synthesis axis the prompt named vs REAL bash (foreign cwd) via the production binary.

CLOSED (caught — the r16 fixes WORK; r15 escape class is dead):
```
  git -C <c> $(printf "reset\t--hard")             rc=2   (decode + normalize_split_ws → word-split, caught)
  git -C <c> $(echo -e "reset\t--hard")            rc=2
  P=$(printf "reset\t--hard"); git -C <c> $P       rc=2   *** r15 REGRESSION now FIXED ***
  P=$(printf "%b" "reset\t--hard"); git -C <c> $P  rc=2
  P=reset; P+=$(printf "\t--hard"); git -C <c> $P  rc=2
  P=$'reset\t--hard'; git -C <c> $P                rc=2   (single ANSI-C into binding — now decoded+re-quoted)
  P=reset; P+=$'\t--hard'; git -C <c> $P           rc=2
  git -C <c> $(printf "reset\n--hard"|\040|\x20|\011)  rc=2 (sibling escapes)
  git -C <c> $(printf "%s" "reset --hard")         rc=2   (the ONE modeled printf shape — single leading %s)
  git -C <c> $(printf "%*s" 0 "reset --hard")      rc=2   (caught: collapses to leading-%s-like? still denies)
  ${P^^}/${P,,}/${P,}/${P^} case-mod                rc=2   (bash 3.2 on this host throws "bad substitution"→no-mut; guard fail-closes anyway)
  ${P:off:len} slice; ${P#..}/${P%..} strip         rc=2   (resolve_param returns None for slice/strip → fail-closed AND caught)
  { git …; } / ( git … ) / true && { git …; }       rc=2   (command grouping — verb still visible, caught)
  ((x=1)); / x=$((..)); git -C <c> reset --hard     rc=2   (arithmetic prefix)
  here-doc <<EOF body with direct git                rc=2   (git verb visible in the command word)
  P=$(cat f) / $(curl) / read P < f ; git -C <c> $P rc=2   (runtime-unknowable in BINDING form → fail-closed, correct)
```

CONFIRMED LEAKS (guard_rc=0 ALLOW; REAL canonical mutation, foreign cwd; production binary; ALL words STATICALLY
determinable — NO `$(cat)`, NO `$(curl)`, NO read-from-stdin):
```
  *** Mechanism A — printf FORMAT-DIRECTIVE class (guard models ONLY a leading %s/%b or specifier-free format) ***
  rc=0 mut=yes  git -C <c> $(printf "%-10s--hard" reset)        argv=[-C <c> reset --hard]  (%-10s left-justify pad → spaces)
  rc=0 mut=yes  git -C <c> $(printf "%s%s" reset " --hard")     argv=[-C <c> reset --hard]  (two %s specifiers)
  rc=0 mut=yes  git -C <c> $(printf -- "%s\n" reset --hard)     argv=[-C <c> reset --hard]  (repeating format, multiple args)
  rc=0 mut=yes  git -C <c> $(printf "%c%s" r "eset --hard")     argv=[-C <c> reset --hard]  (%c%s)
  rc=0 mut=yes  git -C <c> $(printf "reset%0.s --hard" x)       argv=[-C <c> reset --hard]  (%0.s discard mid-format)
  rc=0 mut=yes  P=$(printf "%-6sclean -fdx" ""); git -C <c> $P  argv=[-C <c> clean -fdx]    (width-pad clean — untracked DESTROYED)
  rc=0 mut=yes  git -C <c> $(printf -- "%s\n" clean -fdx)       argv=[-C <c> clean -fdx]    (repeating format clean)
  *** Mechanism B — param pattern-substitution ${P/pat/repl} producing whitespace (resolve_param returns None) ***
  rc=0 mut=yes  P=resetZZhard; git -C <c> ${P/ZZ/ --}          argv=[-C <c> reset --hard]  (${P/ZZ/ --} → reset --hard)
  rc=0 mut=yes  P=cleanZZfdx;  git -C <c> ${P/ZZ/ -}           argv=[-C <c> clean -fdx]
  *** Mechanism C — concatenated ANSI-C $'…'$'…' into a binding (segments not joined+decoded as one value) ***
  rc=0 mut=yes  P=$'reset'$'\t--hard'; git -C <c> $P           argv=[-C <c> reset --hard]
  rc=0 mut=yes  P=$'clean'$'\t-fdx';   git -C <c> $P           argv=[-C <c> clean -fdx]
  *** Mechanism D — xpg_echo makes plain echo decode (guard decodes only echo -e/-ne/-en) ***
  rc=0 mut=yes  shopt -s xpg_echo; git -C <c> $(echo "reset\t--hard")  argv=[-C <c> reset --hard]
```

RESIDUAL genuinely runtime-unknowable (verified, behave correctly in BINDING form, founder-accepted): `$(cat file)`,
`$(curl …)`, `read` from stdin/pipe — all rc=2 fail-closed in `P=$(…); git -C <c> $P` form. NOTE a consistency wart
(NOT the verdict basis): the DIRECT-at-argv forms `git -C <c> $(cat f)` and `echo "reset --hard" | xargs git -C <c>`
ALLOW (rc=0) — these still read a file/pipe at RUNTIME so they are the runtime-unknowable residual, but the guard is
MORE permissive at the direct-argv position than in a binding (binding fail-closes, direct does not). Out-of-scope
for the verdict (runtime read), but worth hardening for symmetry. The TWELVE leaks above are parse-time-determinable
and BLOCK.

Discriminator (isolates the gap precisely — MODELED vs UNMODELED static shape):
```
  git -C <c> $(printf "%s" "reset --hard")     rc=2 DENY  (single leading %s — the ONE modeled printf shape)
  git -C <c> $(printf "%-10s--hard" reset)     rc=0 LEAK  (width directive — UNMODELED → literal %-10s retained)
  git -C <c> $(printf -- "%s\n" reset --hard)  rc=0 LEAK  (repeating format — handler strips one %s, returns "reset")
  P=$'reset\t--hard'; git -C <c> $P            rc=2 DENY  (ONE ANSI-C segment — now decoded+re-quoted+split)
  P=$'reset'$'\t--hard'; git -C <c> $P         rc=0 LEAK  (TWO concatenated segments — not joined as one value)
```
A LEAK occurs precisely when bash produces whitespace-separated argv from a value construct the guard's STATIC
output model does not reproduce: a printf format directive other than a single leading `%s`/`%b`, a `${P/pat/repl}`
pattern substitution, two adjacent `$'…'` ANSI-C segments, or plain `echo` under `xpg_echo`.

### TASK 3 — FALSE-POSITIVE SWEEP (must ALLOW) — 44/44 legit ALLOW; 0 FP
- **All legit forms ALLOW (rc=0), 0 FP.** Merge-train on canonical (status/status --short/log --oneline -5/diff/
  show HEAD/rev-parse HEAD/fetch --all --prune/branch -a/merge --ff-only/pull --ff-only); commit -m with tricky
  messages (`"msg with \t and =val"`, `"reset --hard fix"`, `"set IFS=, and reset"`, `"fix: x=y mapping"`,
  `"key=val<TAB>tab in msg"`); `bash /tmp/retrain.sh`; other-repo + worktree mutations (`git -C <other|wt> reset
  --hard`/`clean -fdx`); multi-word READ vars (`P=log; P+=" --oneline"; git $P`, `P="show HEAD"; git $P`,
  `P=log; P+=" --oneline"; P+=" -5"; git $P`); the NEW FP-risk surfaces — `echo -e "build<TAB>step<TAB>done"`,
  `printf '%s\n' hello world`, `echo -e "reset done"`, `M=$(printf "%s\n" "release v1.2"); commit -m "$M"` — all
  ALLOW; reads with `"$@"`/`"${P[@]}"`/`"$BRANCH"`/`"$COMMIT"`/`--format="%H"`; `V=$(git rev-parse)`; for/while;
  `$(date)`; `$EDITOR`; `sudo sh -c "echo hi"`; `OPT="--config=core.x=1"; git log $OPT`; `printf '%s'` on read-verbs
  (`P=$(printf "%s" "status --short"); git $P`, `git -C <c> $(printf "%s" "log --oneline")`).
- **`strip_quote_chars` + `normalize_split_ws` introduced ZERO over-denial.** The decode path correctly fires only
  for `echo -e`/`printf`/`$'…'` substitution output, never for a quoted commit message or a benign multi-word read.

### TASK 4 — mechanics — all green
- `//tools/oya-checkout-guard-app:oya-checkout-guard-app-unittest` → **31 passed; 0 failed** (Pass 1). The 4 new r15
  escape DENY fixtures present and pass. NOTE: NO fixture exercises the TASK-2 leak shapes (`printf "%-10s…"`,
  `printf "%s\n" a b`, `${P/pat/repl}`, `$'…'$'…'`, `xpg_echo`) — the recurring r1→r15 reason a leak survives a green
  suite (the suite asserts the modeled shapes, not the unmodeled ones).
- `//cloud/cloud-ci/gates/oya-cloud-ci-enforcement-liveness-app:…-unittest` → Pass 1 (Fail 0).
- SSH sigs: `e307fbe52` + `6d92de640` both carry Good ED25519 (SHA256:5grGNUtX9Zgmy1SWne6wF9DR8W1ElUQaF/Z8SYRz8E8).
  (`allowed_signers: Not a directory` is the same worktree-config artifact as r15, not a signature-validity defect.)
- Faces-last settle: `6d92de640` touches ONLY 2 `*.generated.json` faces (accounting-registry, scm-facts). Fix
  commit `e307fbe52` = lib.rs ONLY (+47/-6, no Cargo.lock churn). Holds.
- Baseline key-diff both ways (`9845ced76..6d92de640` over generated faces): **0 anti-pattern keys**
  (forbidden_foundry|unjustified|unowned|unreachable) added OR removed. Face changes are pure provenance
  (source_inputs_digest, last_touch_commit→e307fbe52, head_time_secs). No laundering.
- Dependencies: single dep `serde_json` (workspace), unchanged; no new CVE surface. No secrets in the lib.rs delta.

---

## Findings (file:line, severity, confidence)

### [HIGH, confidence HIGH] F1-R16 — printf FORMAT-DIRECTIVE gap: any directive other than a single leading `%s`/`%b` is unmodeled, so a width/repeating/multi-specifier printf assembles a verb+flag the guard never sees → ALLOW of a canonical reset/clean (statically resolvable; NO `$(cat)`/`$(curl)`/stdin).
- Location: `static_command_output` printf arm (lib.rs:2207-2224). The handler strips ONE leading `%s`/`%b` (with an
  optional `\n`/`\t`) and otherwise returns the format string verbatim through `decode_ansi_c`+`normalize_split_ws`.
  It does not interpret `%-10s` (width/justify pads with REAL spaces), `%s%s`/`%c%s` (multiple specifiers),
  `%0.s` (discard mid-format), or printf's **format-reuse with N args** (`printf "%s\n" reset --hard` → `reset␤--hard`).
- Confirmed (rc=0; REAL bash MUTATES from foreign cwd; production binary; DIRTY→CLEAN; argv `[-C <c> reset --hard]`/
  `[… clean -fdx]`): `git -C <c> $(printf "%-10s--hard" reset)`; `git -C <c> $(printf "%s%s" reset " --hard")`;
  `git -C <c> $(printf -- "%s\n" reset --hard)`; `git -C <c> $(printf "%c%s" r "eset --hard")`;
  `git -C <c> $(printf "reset%0.s --hard" x)`; `P=$(printf "%-6sclean -fdx" ""); git -C <c> $P`;
  `git -C <c> $(printf -- "%s\n" clean -fdx)`.
- Exploitability: local, unauthenticated, single short one-liner, no length cap; `printf` width/repeating formats are
  standard. Trivially emitted by an agent or an injected instruction (the lethal-trifecta surface CLAUDE.md warns of).
- Blast radius: silent canonical `reset --hard` / `clean -fdx`, reproducing FRIC-022/FRIC-1781062867 while all 31
  unit tests + the liveness gate stay green.
- Classification: FRESH MECHANISM (printf format-directive model is a strict subset of bash printf). Same r1→r15
  META-pattern, new axis. NOT a regression (r15's escape forms are closed).
- Required fix: model the produced text of a printf with the named directives — at minimum decode `%-Ns`/`%Ns`
  width as padding, expand format-reuse over the supplied ARGS, and treat any UNMODELED format specifier as
  fail-closed (leave `$(…)` unresolved → fail-closed at the git decision), rather than returning the literal format.

### [HIGH, confidence HIGH] F2-R16 — `${P/pat/repl}` (and `${P//pat/repl}`) pattern-substitution producing whitespace is unmodeled: `resolve_param` returns None so the guard keeps the literal `${…}` (fail-closed at the guard) while bash performs the substitution and word-splits → ALLOW of a real mutation.
- Location: `resolve_param` (lib.rs:2162) handles only `${name}`, `${name:-default}`, `${name-default}`; returns
  None for `${P/pat/repl}`, `${P^^}`, `${P,,}`, `${P:off:len}`, `${P#..}`, `${P%..}`. For the slice/case/strip forms
  the verb still appears at the use site so the guard catches them; but for `${P/pat/repl}` whose REPLACEMENT injects
  a space (`${P/ZZ/ --}`), bash produces `reset --hard` while the guard sees the literal `${P/ZZ/ --}` token and
  cannot match the verb.
- Confirmed (rc=0; argv `[-C <c> reset --hard]`/`[… clean -fdx]`): `P=resetZZhard; git -C <c> ${P/ZZ/ --}`;
  `P=cleanZZfdx; git -C <c> ${P/ZZ/ -}`.
- Classification: FRESH MECHANISM (pattern-substitution value-synthesis), same META-pattern.
- Required fix: model `${P/pat/repl}`/`${P//pat/repl}` against the same-line binding (the pattern+replacement are
  literal and statically determinable), OR fail-closed (leave `${…}` unresolved is NOT enough — that ALREADY happens
  and still leaks because the verb is hidden behind the brace; the guard must treat an unresolved `${P/…}` whose name
  IS bound as a value-producing expansion it cannot prove safe → suppress the git decision / fail-closed, the way
  IFS-reassignment suppresses expansion).

### [MEDIUM→HIGH, confidence HIGH] F3-R16 — concatenated ANSI-C `$'…'$'…'` into a binding is not joined+decoded as one value, so the re-quote/word-split path misses it → ALLOW.
- Location: ANSI-C branch (lib.rs:2005-2026) decodes ONE `$'…'` segment and routes it through
  `emit_substitution_output(normalize_split_ws(decode_ansi_c(raw)))`; but two ADJACENT `$'…'` segments
  (`$'reset'$'\t--hard'`) are not concatenated into a single decoded value before the binding is captured, so the
  collected `P` value does not carry the decoded TAB as splittable whitespace at the use site.
- Confirmed (rc=0; argv `[-C <c> reset --hard]`/`[… clean -fdx]`): `P=$'reset'$'\t--hard'; git -C <c> $P`;
  `P=$'clean'$'\t-fdx'; git -C <c> $P`.
- Classification: FRESH MECHANISM (string concatenation of ANSI-C segments), same META-pattern. The single-segment
  form `P=$'reset\t--hard'; git -C <c> $P` is correctly CLOSED by r16 — this is the adjacent-segment sibling.
- Required fix: concatenate adjacent quoted/ANSI-C segments of one word before decode, so the joined value carries
  the decoded whitespace through `emit_substitution_output`'s re-quote gate.

### [MEDIUM, confidence HIGH] F4-R16 — `shopt -s xpg_echo` makes plain `echo` decode escapes, but the guard decodes only `echo -e`/`-ne`/`-en` → ALLOW.
- Location: `static_command_output` echo arm (lib.rs:2186-2204): `decode` is gated on the literal `-e`/`-ne`
  prefix; under `xpg_echo` plain `echo "reset\t--hard"` decodes the TAB at runtime, which the guard misses.
- Confirmed (rc=0; argv `[-C <c> reset --hard]`): `shopt -s xpg_echo; git -C <c> $(echo "reset\t--hard")`.
- Note: `shopt`/`xpg_echo` is a same-line, statically-visible toggle → parse-time-determinable, not runtime-unknowable.
- Required fix: when a same-line `shopt -s xpg_echo` (or `set -o posix` in some builds) is present, decode plain
  `echo` the same as `echo -e`; OR fail-closed when echo output feeds an unquoted substitution under xpg_echo.

### [LOW, confidence HIGH] F5-R16 — direct-at-argv `$(cat)`/`xargs`-pipe forms ALLOW while the binding form fail-closes (consistency wart, runtime-unknowable → out of verdict scope).
- `git -C <c> $(cat f)` and `echo "reset --hard" | xargs git -C <c>` ALLOW (rc=0); `P=$(cat f); git -C <c> $P`
  fail-closes (rc=2). Both read a file/pipe at RUNTIME so both are the founder-accepted runtime-unknowable residual,
  but the guard is asymmetric. Not the verdict basis; harden for symmetry (treat a direct unquoted `$(prog)` / a
  pipe into `xargs git` at the argv of a `-C <canonical>` command the same conservative way as a binding).

### Resolved since r15 (verified)
- The ENTIRE r15 escape-decode class DENIES through the production binary: `printf "reset\t--hard"` (direct, backtick,
  whole-argv), `echo -e`, `printf %b`, subst-`+=`, single `$'…'`-into-binding, and the `\n`/`\040`/`\x20`/`\011`
  siblings. The r15 REGRESSION `P=$(printf "reset\t--hard"); git -C <c> $P` is FIXED (rc=2).
- The multi-word READ false-positive stays closed; the new decode/quote surfaces add 0 false positives (44/44 ALLOW).

### Positive observations
- The r16 fix is surgical and correct for its stated scope: it closes the entire r15 escape-decode class AND the r15
  regression with general (not per-fixture) logic — `strip_quote_chars` preserves escapes, `decode_ansi_c` decodes
  them, `normalize_split_ws` turns decoded whitespace into argv boundaries, and the binding RHS re-quotes then
  word-splits. 31 unit + liveness green, Good ED25519, faces-last holds (2 faces, lib.rs-only fix, no Cargo.lock
  churn), 0 anti-pattern key churn, single guard dep (serde_json), no new CVE surface, no secrets in delta.

### Note — the r1→…→r16 meta-pattern, now at the value-MODEL-COMPLETENESS axis
r16 correctly closed the escape-DECODING gap r15 named. But the convergence claim again equates "the named (r15
escape) forms denied" with "static closure reached," which is false: the guard's STATIC value-producing model
(`static_command_output` for printf/echo, `resolve_param` for `${…}`, the ANSI-C segment handling) is a strict
SUBSET of what bash produces. Every leak this round is bash producing whitespace-separated argv from a value
construct the guard's model does not reproduce — a printf format directive other than a single leading `%s`/`%b`,
a `${P/pat/repl}` substitution, two adjacent `$'…'` segments, or plain echo under xpg_echo. The durable fix is NOT
another enumerated decode patch; it is to make the value-producing paths **fail-closed by default**: any `$(printf …)`
/`$(echo …)`/`${…}` whose exact output the guard cannot PROVE leaves `$(…)`/`${…}` unresolved → fail-closed at the
git decision (matching the `P="reset --hard"` multi-word and IFS-reassignment handling), with an allowlist of the
EXACTLY-modeled shapes — rather than returning a best-effort literal that silently under-approximates bash.

---

## Residual risk (single most likely production failure if merged as-is)
An agent (or an injected instruction via the lethal-trifecta surface) emits
`bash -c 'git -C <canonical> $(printf "%-10s--hard" reset)'` or
`bash -c 'P=resetZZhard; git -C <canonical> ${P/ZZ/ --}'` or
`bash -c "P=\$'reset'\$'\t--hard'; git -C <canonical> \$P"` — all statically-resolvable, ZERO command output read at
runtime, ZERO read-from-stdin — and it silently ALLOWs, re-contaminating the canonical checkout and reproducing
FRIC-022/FRIC-1781062867 while all 31 tests and the liveness gate stay green. The leak survives because the guard's
static value model (printf format directives, `${P/pat/repl}`, concatenated `$'…'`, xpg_echo) is narrower than bash.

Secondary (carried, unchanged): the hook shim fails OPEN if the Rust binary is unbuilt (main.rs maps any error →
SUCCESS); ensure CI/branch-protection builds it (structural enforcement, not hook reliance). Genuinely
runtime-unknowable family (pre-existing, out of scope, fail-closed in binding form): `read` from stdin/pipe;
`$(cat)`/`$(curl)` opaque stdout (NOTE F5-R16: leaks at the DIRECT-argv position — harden for symmetry);
PATH/symlink alias `g`→git binary; interactive-only `alias`.

## Required to clear
1. Close F1-R16 (printf format directives): model `%-Ns`/`%Ns` width padding, multi-specifier formats, and printf
   format-reuse over the supplied ARGS; treat any UNMODELED printf format specifier as fail-closed (leave `$(…)`
   unresolved) rather than returning the literal format string.
2. Close F2-R16 (`${P/pat/repl}`): model the pattern substitution against the same-line binding (statically
   determinable), OR suppress the git decision when a bound name appears in an unmodeled `${name/…}` expansion.
3. Close F3-R16: concatenate adjacent `$'…'`/quoted segments of one word before decode so the joined value carries
   the decoded whitespace through the re-quote/word-split path.
4. Close F4-R16: decode plain `echo` as `echo -e` when a same-line `shopt -s xpg_echo` is present (or fail-closed).
5. Harden F5-R16 for symmetry: treat a direct unquoted `$(prog)` / `xargs git` pipe at the argv of a `-C <canonical>`
   command the same conservative way as a binding.
6. Prefer the STRUCTURAL fix: make the static value-producing paths fail-closed by default with an explicit allowlist
   of exactly-modeled shapes, instead of returning a best-effort under-approximation of bash output.
7. Add a DENY fixture for EVERY confirmed F1..F4 reproduction (the 12 leaks above + the `clean -fdx` variants). Current
   fixtures structurally cannot catch them (no width/repeating-format/pattern-subst/concatenated-ANSI-C/xpg_echo shape).
8. Re-run 31 + liveness + new fixtures; re-drive the TASK-2 sweep through the real binary with a FOREIGN session-cwd;
   confirm zero false positives on the merge-train + retrain + multi-word reads + the new decode/quote forms.
9. Re-state the convergence claim honestly only after auditing the value-MODEL-COMPLETENESS axis (printf format
   directives, `${name/…}`/`${name:off:len}` expansions, adjacent-quote concatenation, xpg_echo) — and confirm the
   sole residual is genuinely runtime-unknowable `$(prog)`/read-from-stdin.

## Security Checklist
- [x] No hardcoded secrets (lib.rs is pure policy logic; only dep is serde_json; no new deps; secrets scan clean)
- [~] All inputs validated — r15 escape-decode axis CLOSED (incl the r15 regression); but the guard's static value
      model is a strict subset of bash → 12 statically-resolvable real-mutation ALLOWs across 4 fresh mechanisms
      (F1 printf format directives, F2 `${P/pat/repl}`, F3 concatenated `$'…'`, F4 xpg_echo)
- [~] Injection prevention — r15 escape class CLOSED; F1..F4 OPEN (statically-assembled canonical mutation). F5
      direct-`$(cat)`/xargs asymmetry noted (runtime residual, out of verdict scope)
- [x] Authorization/policy enforced for modeled forms; 44/44 legit + worktree/other-repo/retrain ALLOW with 0 FP;
      the new decode/quote surfaces introduce no over-denial
- [x] Dependencies audited — single dep serde_json; no new guard deps; no CVE surface in delta
- [x] Tests + liveness gate green (31 unit + gate Pass 1); SSH-signed (Good ED25519); faces-last (2 faces, lib.rs-only
      fix, no Cargo.lock churn); 0 anti-pattern key churn

---

VERDICT: **BLOCK** — 12 statically-resolvable real-mutation bypasses across 4 fresh mechanisms (NEW, not r15
variants): F1 printf format-directives (`%-Ns`/`%s%s`/repeating `%s\n a b`/`%c%s`/`%0.s`), F2 `${P/pat/repl}`
pattern-substitution, F3 concatenated ANSI-C `$'…'$'…'`, F4 plain-`echo` under `xpg_echo`. Each ALLOWs a canonical
`reset --hard`/`clean -fdx` end-to-end through the production binary from a FOREIGN cwd with NO `$(cat)`/`$(curl)`/
read-from-stdin. The r15 escape-decode class (and its regression) is fully closed and there are zero real-command
false positives (44/44 ALLOW) and clean mechanics, but the strict VERDICT RULE blocks on any statically-resolvable
real-mutation bypass. The convergence claim ("sole residual is the runtime-unknowable class") is FALSE: the residual
is the guard's static value-model completeness, which remains a strict subset of bash.
