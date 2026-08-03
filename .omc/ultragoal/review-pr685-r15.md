# Security Review — PR #685 ROUND 15 (G011 canonical-checkout guard) — CONVERGENCE CHECK

- Repo: jason931225/oyatie  Branch: agent/g011-checkout-guard  Head: 9845ced76  Base: dev
- r15 delta vs r14 (d2e5d3b34..9845ced76): two commits.
  - `90fad259a` `fix(checkout-guard): faithful multi-word binding model + += append (review #685 r14)`
    — lib.rs ONLY (+74/-21, no Cargo.lock churn). Reworks the binding model to be FAITHFUL rather than
    fail-close: (1) `collect_same_line_bindings` (lib.rs:1747) is now quote-aware (uses `shell_tokens`),
    handles `=` and `+=` accumulation, KEEPS multi-word values whole, and drops a value ONLY if it still
    carries `$`/backtick (lib.rs:1774-1780) — replacing r14's strict `value_ok` `[alnum-_/.]` filter.
    (2) `out_ends_with_assignment_lhs` (lib.rs:1865) accepts an optional trailing `+` before `=` so the
    re-quote covers the `P+=` substitution variant. (3) `expand_with_bindings` (lib.rs:2129) routes a
    resolved `$name` through `emit_substitution_output` so a multi-word value re-quotes as an assignment
    RHS (`B=$A`) and word-splits at an unquoted use site. (4) `static_command_output` printf (lib.rs:2193)
    strips a leading `%s`/`%b` to yield the ARG. 5 new DENY fixtures + 1 new ALLOW fixture (lib.rs:3059-3082).
  - `9845ced76` `chore: settle generated cloud-ci faces` — 2 *.generated.json faces ONLY.
- Reviewer: fresh-context security-reviewer (Claude Opus), Torvalds/attacker lens,
  /using-superpowers + /using-agent-skills + /ultraqa, all FOREGROUND, built from
  /Users/jasonlee/oyatie-worktrees/g011-checkout-guard. FOREIGN session-cwd (/tmp), mutation arrives ONLY
  via -C/binding indirection, every candidate confirmed against REAL bash with an argv-capturing fakegit
  (dirty->clean) that EXECs real git so canonical mutation is genuine. Adjudicator self-test sound; leak-
  detection proven (PATH-alias `g`→git flags LEAK correctly).

## VERDICT: BLOCK

**Risk Level: HIGH.** r15 closes the entire r14 `+=` class — the scalar-append leak is dead:
`P=reset; P+=" --hard"; git -C <c> $P`, `P=clean; P+=" -fdx"; …`, `A=-C; A+=" <c>"; A+=" reset --hard";
git $A`, the brace/double-hop/subst-`+=` siblings ALL now DENY (rc=2) through the production binary, and
the faithful model removes the multi-word-read false positive (`P=log; P+=" --oneline"; git $P` ALLOWs).
The 42-form accumulated corpus is 42/42 DENY. The FP sweep is 0 false positives across 34 legit forms +
worktree/function/set--/retrain. Mechanics are green (31 unit + liveness Pass 1, SSH-signed Good ED25519,
faces-last holds, 0 anti-pattern key churn).

BUT the r15 convergence claim — "the faithful binding model holds; the sole residual is the runtime-
unknowable `$(cat)`/`$(curl)`/read-from-stdin class" — is **FALSE**. r15 traded r14's strict `value_ok`
fail-closed binding filter for faithful multi-word retention, but the faithful model does NOT model the
escape-decoding bash performs inside `printf` format strings, `printf %b`, `echo -e`, and ANSI-C `$'…'`
quoting when those land in a binding value. Bash decodes `\t`/`\n`/`\040`/`\x20`/`\011` to REAL whitespace
and word-splits; the guard keeps the literal backslash-escape, sees ONE non-whitespace word, fails to
re-quote (its whitespace check is false on `reset\t--hard`) and fails to word-split — so the trailing
mutating flag is never seen and the command ALLOWs. I constructed **10 statically-resolvable real-mutation
bypasses** that use NO runtime-unknowable input. Per the strict VERDICT RULE ("BLOCK on a statically-
resolvable real-mutation bypass"), this blocks. One of these is a **REGRESSION vs r14**: the pure-`=`
form `P=$(printf 'reset\t--hard'); git -C <c> $P` DENIED at r14 (d2e5d3b34, rc=2 — strict `value_ok`
rejected the backslash) and now ALLOWs at r15 (rc=0 — loosened filter binds it).

The two single most-damning leaks (every byte statically determinable — NO `$(cat)`, NO `$(curl)`, NO stdin):
```
  git -C <c> $(printf "reset\t--hard")             -> rc=0 ALLOW *** LEAK ***  (no binding, no +=, no echo)
  P=reset; P+=$'\t--hard'; git -C <c> $P           -> rc=0 ALLOW *** LEAK ***  (pure shell ANSI-C, no subst)
  P=$(printf "reset\t--hard"); git -C <c> $P       -> rc=0 ALLOW *** LEAK ***  (REGRESSION — r14 DENIED)
```
All three MUTATE canonical (`reset --hard`) end-to-end through the production binary from a FOREIGN cwd
with an argv-capturing fakegit (argv `[-C][<canon>][reset][--hard]`, DIRTY→CLEAN).

Root cause (lib.rs):
- `static_command_output` printf (lib.rs:2193) returns the format string via `dequote_simple` WITHOUT
  decoding backslash escapes. Bash's `printf 'reset\t--hard'` decodes `\t`→TAB; the guard yields the
  literal 2-char `\t`. The decoded result reaches `emit_substitution_output` (lib.rs:1852) whose re-quote
  gate is `produced.chars().any(char::is_whitespace)` — FALSE on `reset\t--hard` (backslash+`t` are not
  whitespace) → NOT re-quoted → emitted bare → bound whole → at the `$P`/`$(…)` use site the guard word-
  splits only on REAL whitespace → one junk word `reset\t--hard` → `is_blocked_operation` sees verb
  `reset\t--hard` ≠ `reset` → ALLOW. Same gap for `printf %b`, `echo -e`, and `$'…'`-into-binding.
- `collect_same_line_bindings` (lib.rs:1774-1780) — r15 replaced the strict `value_ok` `[alnum-_/.]`
  filter with `!value.contains('$') && !value.contains('`')`. A value carrying a literal backslash-escape
  (`reset\t--hard`) now PASSES and binds, where r14's `value_ok` rejected it (`\` not allowed) → fail
  closed. This loosening is the REGRESSION source for the pure-`=` printf form.
- The ANSI-C `$'…'` decoder (`decode_ansi_c`, lib.rs:2022) DOES decode a DIRECT arg
  (`git -C <c> $'reset\t--hard'` → DENY, correct), but when `$'…'` is a binding RHS the decoded-tab value
  is captured/re-emitted so the binding retains it as a single word that is not re-split at the use site
  → LEAK.

**NEW CLASS or VARIANT?** This is a **FRESH MECHANISM**: a *decode-gap* between bash's escape decoding
(`printf` format, `printf %b`, `echo -e`, ANSI-C `$'…'`-into-binding) and the guard's literal handling,
exposed by r15's loosened binding filter. It is NOT a re-spelling of r14's scalar `+=` (which r15 closed)
nor r13's static-substitution launder (which r14 closed). It IS the same r1→r14 META-pattern (the
effective-argv model diverges from bash at a value construct the guard models imperfectly; the multi-word
bash result is flattened to a single guard word and the trailing mutating flag is dropped), but via a new
axis — *escape decoding* — and one form (`P=$(printf 'reset\t--hard'); …`) is a genuine REGRESSION that
r14 denied and r15 now allows.

---

## Harness (commands + exact exit codes)

Built the real binary: `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out
/tmp/r15-guard-bin` → BUILD SUCCEEDED (Mach-O arm64). Also built r14 head (d2e5d3b34) → /tmp/r14-guard-bin
for the regression comparison. Driven via JSON hook payload `{"tool_input":{"command":"…"}}` on stdin
(the real enforcement surface), `env OYA_CANONICAL_CHECKOUT=<canon>`, binary cwd=/tmp (canonical != cwd).
rc=2 ⇒ DENY, rc=0 ⇒ ALLOW.

**Adjudication discipline (real-mutation only counts):** per candidate (1) fresh DIRTY real git repo
(` M tracked.txt` + `?? untracked.txt`); (2) guard rc via real binary with the canonical path; (3) the
EXACT command in `bash -c` from FOREIGN cwd /tmp with an argv-capturing fakegit that EXECs real git, so
`-C <canon> reset --hard` genuinely mutates; (4) diff `git status --porcelain` before/after. A FINDING
requires `guard_rc=0 AND mutated=yes`. Adjudicator self-test sound (`-C <canon> {reset --hard|clean
-fdx|restore .}` DENY+clean; `status`/`log`/bare-foreign-`reset` ALLOW+clean). Leak-detection proven: the
PATH-alias `g -C <canon> reset --hard` (runtime-unknowable family) correctly flagged `mut=yes LEAK`.

### TASK 1 — full accumulated corpus (r1..r14 named + ALL reproductions incl r14 +=) — PASS
43-form real-mutation corpus (r1 basics; r2 wrappers/env; r3 sh -c/GIT_DIR/GIT_WORK_TREE; r4 nested;
r5 `$(echo git)`/eval/`bash -c "$()"`; r6 param-subst; r7 brace; r9 ANSI-C direct; r10 set--/function;
r11 read-ra; r12 argv-position binding; r13 ALL EIGHT static-subst-into-binding; r14 ALL `+=` forms
incl `A=-C; A+=" <c>"; A+=" reset --hard"; git $A` and the subst-`+=` variants):
- **42/42 actually-blocked DENY (rc=2). 0 ALLOW.** The r14 `+=` class is now FIXED through the production
  binary. The 43rd entry `git -C <c> stash -- .` ALLOWs — this is CORRECT and intentional, NOT a leak:
  `is_blocked_operation` (lib.rs:2669) blocks `stash pop`/`stash apply` (writes to worktree) but permits
  `stash` save/push (recoverable via `stash pop`). `stash pop`/`stash apply` both DENY. My harness entry
  was mis-specified (a save is not a restore). Excluding it: 42/42 of the blocked set DENY.

### TASK 2 — CONVERGENCE TEST — TEN STATICALLY-RESOLVABLE REAL-MUTATION BYPASSES (BLOCK); r14 += CLOSED
Adjudicated every binding/word-synthesis/assignment mechanism the prompt named vs REAL bash (foreign cwd)
via the production binary.

CLOSED (caught — the r15 fixes WORK; r14 += class is dead):
```
  P=reset; P+=" --hard"; git -C <c> $P                 rc=2 (faithful: multi-word value, $P word-splits, reset+--hard, blocked)
  P=clean; P+=" -fdx";  git -C <c> $P                  rc=2
  A=-C; A+=" <c>"; A+=" reset --hard"; git $A          rc=2 (whole argv via += — now modeled)
  P=reset; P+=" --hard"; git -C <c> ${P}               rc=2 (brace use site)
  P="reset --hard"; git -C <c> $P                      rc=2 (= multi-word — still fail closed)
  P+=$(echo "-C <c> reset --hard"); git $P             rc=2 (subst += of full multiword)
  A=reset B=--hard; git -C <c> $A $B                   rc=2 (multiple assignments, both used)
  A=-C B="reset --hard"; git $A <c> $B                 rc=2
  declare/export/readonly/typeset/local P=reset; P+=" --hard"; git -C <c> $P   rc=2 (assignment-prefix forms)
  (P=reset; P+=" --hard"); git -C <c> $P               rc=2 (subshell assignment scope — bash: $P empty, no mut; guard DENY-safe)
  ((x=1)); P=reset; P+=" --hard"; git -C <c> $P        rc=2 (arithmetic prefix)
  P[0]=-C; P[1]=<c>; P[2]=reset; P[3]=--hard; git "${P[@]}"  rc=2 (array-element assign)
  declare -a P; P[0]=reset; P[1]=--hard; git -C <c> "${P[@]}" rc=2
  Q="reset --hard"; P=$Q; git -C <c> $P                rc=2 (nested var, multi-word source)
  A="-C <c> reset --hard"; B=$A; git $B                rc=2 (double hop)
  P="a=b"; git -C <c> reset --hard                     rc=2 (quoted = in value — direct reset still blocked)
  IFS=,; P="reset,--hard"; git -C <c> $P               rc=2 (IFS reassign → ifs_unsafe → fail closed)
  IFS=:; P="reset --hard"; git -C <c> $P               rc=2
  P="reset"; P+="=x"; git -C <c> $P                    rc=0 (FAITHFUL: P=reset=x, not a verb; bash no-mut too)
  P=$(cat file); git -C <c> $P                         rc=2 (opaque $(cat) → fail closed)
  P=$(curl -s http://x); git -C <c> $P                 rc=2 (fail closed)
  read P < /dev/null; git -C <c> $P --hard             rc=2 (read-from-stdin → fail closed)
  read -ra P; git -C <c> "${P[@]}"                     rc=2 (read-from-stdin array → fail closed)
```
CONFIRMED LEAKS (guard_rc=0 ALLOW; REAL canonical mutation, foreign cwd; production binary; ALL words
STATICALLY determinable — NO `$(cat)`, NO `$(curl)`, NO read-from-stdin):
```
  rc=0 mut=yes  git -C <c> $(printf "reset\t--hard")               argv=[-C <c> reset --hard]  (no binding, no +=, no echo)
  rc=0 mut=yes  git -C <c> `printf "reset\t--hard"`                argv=[-C <c> reset --hard]  (backtick variant)
  rc=0 mut=yes  git -C <c> $(echo -e "reset\t--hard")              argv=[-C <c> reset --hard]  (echo -e decode)
  rc=0 mut=yes  git $(printf -- "-C\t<c>\treset\t--hard")          argv=[-C <c> reset --hard]  (WHOLE argv via printf)
  rc=0 mut=yes  P=$(printf "reset\t--hard"); git -C <c> $P         argv=[-C <c> reset --hard]  *** REGRESSION: r14 DENIED ***
  rc=0 mut=yes  P=$(printf "%b" "reset\t--hard"); git -C <c> $P    argv=[-C <c> reset --hard]  (printf %b decode)
  rc=0 mut=yes  P=reset; P+=$(printf "\t--hard"); git -C <c> $P    argv=[-C <c> reset --hard]  (subst += + decode)
  rc=0 mut=yes  P=clean; P+=$(printf "\t-fdx"); git -C <c> $P      argv=[-C <c> clean -fdx]    (untracked DESTROYED)
  rc=0 mut=yes  P=$'reset\t--hard'; git -C <c> $P                  argv=[-C <c> reset --hard]  (ANSI-C into binding)
  rc=0 mut=yes  P=reset; P+=$'\t--hard'; git -C <c> $P             argv=[-C <c> reset --hard]  (ANSI-C += — pure shell, no subst)
```
Sibling whitespace escapes ALL leak identically: `\n`, `\040` (octal space), `\x20` (hex space), `\011`
(octal tab), `\012` — bash decodes each, the guard keeps the literal backslash.

Discriminator (isolates the gap precisely — DIRECT vs INTO-BINDING, and r14 vs r15):
```
  git -C <c> $'reset\t--hard'             rc=2 DENY  (DIRECT ANSI-C: decode_ansi_c → real tab → word-split → caught)
  P=$'reset\t--hard'; git -C <c> $P       rc=0 LEAK  (INTO-BINDING: value kept whole, not re-split at use)
  P=$(printf 'reset\t--hard'); git -C <c> $P   r14 rc=2 DENY  →  r15 rc=0 LEAK   *** REGRESSION ***
  P=$(printf '%s' "reset\t--hard"); git -C <c> $P   rc=0 mut=no  (FAITHFUL: %s does not decode in bash either)
```
A LEAK occurs precisely when bash decodes an embedded escape to REAL whitespace (in a `printf` format,
`printf %b` arg, `echo -e`, or `$'…'` quoting) inside a value the guard binds/captures, while the guard
keeps the literal backslash-escape — so the guard sees one non-whitespace word, does not re-quote
(`is_whitespace` false), does not word-split, and the trailing `--hard`/`-fdx` is never recognized.

RESIDUAL genuinely runtime-unknowable (verified, behave correctly, founder-accepted): `$(cat file)`
(rc=2 — opaque subst → fail closed), `$(curl …)` (rc=2), `read` from stdin/pipe (rc=2). These are NOT the
sole residual: the decode-gap class above is parse-time-determinable and BLOCKS. Carried/unchanged
(runtime-unknowable family, pre-existing, out of scope): PATH/symlink `g`→git binary; interactive-only
`alias`; opaque `$(prog)` stdout.

### TASK 3 — FALSE-POSITIVE SWEEP (must ALLOW) — 34/34 legit ALLOW; multi-word read FP CLOSED
- **All legit forms ALLOW (rc=0), 0 FP.** Merge-train on canonical (status/log --oneline -5/diff/show
  HEAD/rev-parse HEAD/fetch --all --prune/commit -m "msg"/commit -m "reset --hard fix"/commit -m "set
  IFS=, and reset"/commit -m "fix: x=y mapping"/merge --ff-only/pull --ff-only); the NEW faithful-model
  FP-risk surface — multi-word READ vars — all ALLOW: `P=log; P+=" --oneline"; git $P`,
  `P="show HEAD"; git $P`, `P="log --oneline"; git $P`, `P=log; P+=" --oneline"; P+=" -5"; git $P`,
  `F=status; git $F`; reads with $-args (`set -- --oneline -5; git -C <c> log "$@"`,
  `read -ra P <<< "--oneline -5"; … "${P[@]}"`, `P=(--oneline -5); … "${P[@]}"`, `log "$BRANCH"`,
  `show "$COMMIT"`, `--format="%H"`); `V=$(git -C <c> rev-parse HEAD)`; benign subst (`$(echo status)`,
  `$(echo log) --oneline`); `IFS=, read -ra parts <<< "a,b"; … log ${parts[0]}`; `set -- a b c; echo
  "$@"`; for/while; `$(date)`; `$EDITOR`; `sudo sh -c "echo hi"`; `OPT="--config=core.x=1"; … log $OPT`.
- **Worktree (NON-canonical) + function/set-- ALLOW**: `git -C <wt> reset --hard`, `git -C <wt> clean
  -fdx`, `g(){ git "$@"; }; g -C <wt> reset --hard`, `set -- reset --hard; git -C <wt> "$@"`, and retrain-
  style on /srv/other (reset --hard / clean -fdx). All ALLOW (target the other repo, canonical untouched).
- **Carried over-deny (F2-R14, fail-closed, SAFE)**: `P=(-C <wt> reset --hard); git "${P[@]}"` still
  DENIES (rc=2) — array-bound `-C <wt>` cannot be proven non-canonical → fail closes. Usability over-deny,
  not a security hole, not the verdict basis. The faithful binding model did NOT introduce any new FP.

### TASK 4 — mechanics — all green
- `//tools/oya-checkout-guard-app:oya-checkout-guard-app-unittest` → **31 passed; 0 failed** (5 new r14
  `+=` DENY fixtures + 1 new multi-word-read ALLOW fixture present and pass). NOTE: NO fixture exercises
  the TASK-2 decode-gap leak (`$(printf "reset\t--hard")`, `$'\t--hard'`-into-binding, `printf %b`,
  `echo -e`) — the recurring r1→r14 reason the leak survives a green suite.
- `//cloud/cloud-ci/gates/oya-cloud-ci-enforcement-liveness-app:…-unittest` → Pass 1 (Fail 0).
- SSH sigs: 90fad259a + 9845ced76 both carry Good ED25519 signature
  (SHA256:5grGNUtX9Zgmy1SWne6wF9DR8W1ElUQaF/Z8SYRz8E8). (The local `sshsig_find_principal: Not a
  directory` warning is a worktree-config artifact, not a signature-validity defect — both verify Good.)
- Faces-last settle: 9845ced76 touches ONLY 2 *.generated.json faces (accounting-registry, scm-facts).
  Fix commit 90fad259a = lib.rs ONLY (+74/-21, no Cargo.lock churn). Holds.
- Baseline key-diff both ways (d2e5d3b34..9845ced76 over generated faces): **0 anti-pattern keys**
  (forbidden_foundry|unjustified|unowned|unreachable) added OR removed. Face changes are pure provenance
  (source_inputs_digest, last_touch_commit→90fad259a, head_time_secs). No laundering.
- Dependencies: single dep `serde_json` (workspace), unchanged; no new CVE surface.

---

## Findings (file:line, severity, confidence)

### [HIGH, confidence HIGH] F1-R15 — printf/echo-e/`$'…'` escape-decode gap into a bound value → the appended/decoded mutating flag is dropped, ALLOWing a canonical reset/clean (statically resolvable; NO `$(cat)`/`$(curl)`/stdin). Includes a REGRESSION vs r14.
- Location: `static_command_output` printf (lib.rs:2193, returns `dequote_simple(format)` WITHOUT
  decoding `\t`/`\n`/`\0NN`/`\xHH`; `%b` arg likewise undecoded); `emit_substitution_output` (lib.rs:1852,
  re-quote gate `produced.chars().any(char::is_whitespace)` is FALSE on a literal `reset\t--hard`);
  `collect_same_line_bindings` filter (lib.rs:1774-1780, the loosened `!contains('$') && !contains('`')`
  binds a literal-backslash value that r14's `value_ok` rejected → REGRESSION source); ANSI-C `$'…'`
  decoded for a DIRECT arg (lib.rs:2022) but the decoded value is retained whole when it is a binding RHS.
- Confirmed (rc=0; REAL bash MUTATES from foreign cwd; production binary; DIRTY→CLEAN; argv-capture =
  `[-C][<canon>][reset][--hard]` / `[…][clean][-fdx]`):
  `git -C <c> $(printf "reset\t--hard")`; `git -C <c> \`printf "reset\t--hard"\``;
  `git -C <c> $(echo -e "reset\t--hard")`; `git $(printf -- "-C\t<c>\treset\t--hard")` (whole argv);
  `P=$(printf "reset\t--hard"); git -C <c> $P` (REGRESSION — r14 DENIED, r15 ALLOWS);
  `P=$(printf "%b" "reset\t--hard"); git -C <c> $P`; `P=reset; P+=$(printf "\t--hard"); git -C <c> $P`;
  `P=clean; P+=$(printf "\t-fdx"); git -C <c> $P`; `P=$'reset\t--hard'; git -C <c> $P`;
  `P=reset; P+=$'\t--hard'; git -C <c> $P` (pure shell, ZERO subst). Sibling escapes `\n \040 \x20 \011`
  leak identically.
- Exploitability: local, unauthenticated, single short one-liner, no length cap, shell-agnostic; `printf`,
  `echo -e`, `$'…'` are standard bash. Trivially emitted by an agent or an injected instruction (the
  lethal-trifecta surface CLAUDE.md warns of).
- Blast radius: silent canonical `reset --hard` / `clean -fdx`, reproducing FRIC-022/FRIC-1781062867
  while all 31 unit tests + the liveness gate stay green.
- STATICALLY RESOLVABLE: the cleanest PoC (`git -C <c> $(printf "reset\t--hard")`,
  `P=reset; P+=$'\t--hard'; git -C <c> $P`) has NO `$(cat)`/`$(curl)`/stdin — every byte is determinable
  at parse time. NOT the runtime-unknowable residual — the BLOCK class.
- Classification: FRESH MECHANISM (escape-decode gap between bash and the guard, exposed by r15's loosened
  binding filter), distinct from r14's scalar `+=` (which r15 closed) and r13's substitution launder
  (which r14 closed). Same r1→r14 META-pattern (effective-argv model diverges at a value construct the
  guard models imperfectly; multi-word bash result flattened to one guard word, trailing flag dropped),
  new axis (escape decoding); one form is a genuine REGRESSION vs r14.

### [LOW→MEDIUM, confidence HIGH] F2-R15 — carried fail-closed over-deny on array-bound non-canonical worktree mutation (unchanged from r14)
- Location: array/unbound literal-retention path (lib.rs ~2056) + retargetable forcing blocked_target.
- Confirmed (rc=2; targets a NON-canonical worktree, does NOT touch canonical): `P=(-C <wt> reset
  --hard); git "${P[@]}"`. r15 unchanged (the function/`set --` worktree allows remain restored).
- Severity: usability/fail-closed regression, NOT a security hole. Moot for the verdict — F1-R15
  independently blocks.

### Note — the r1→…→r15 meta-pattern, now at the escape-decode axis
r15 correctly closed the entire r14 scalar-`+=` class and removed the multi-word-read FP with a genuinely
better faithful model. But the convergence claim again treats "the named (r14 +=) forms denied" as "static
closure reached," which is false: the faithful model does NOT decode the escapes bash decodes inside
`printf` formats, `printf %b`, `echo -e`, and `$'…'`-into-binding, and r15's loosened binding filter
(`value_ok` → only-drop-`$`/backtick) now BINDS literal-backslash values that r14 fail-closed. The durable
fix has two parts: (a) make `static_command_output` for printf decode backslash escapes (`\t \n \r \0NN
\xHH`) the way bash does for the FORMAT string and for a `%b` ARG, so the produced text contains REAL
whitespace and `emit_substitution_output`'s `is_whitespace` re-quote gate fires; (b) when a binding RHS is
an ANSI-C `$'…'` value, decode it the same way and route through the re-quote path — OR, equivalently,
restore a fail-closed binding filter that rejects any value containing a backslash-escape sequence the
guard cannot prove whitespace-free, so a decode-ambiguous value leaves `$name` unresolved → fail closed
(matching the `P="reset --hard"` multi-word handling). Either path must run BEFORE the use-site word-split
so the trailing flag is seen.

### Resolved since r14 (verified)
- The entire r14 scalar-`+=` class DENIES through the production binary: `P=reset; P+=" --hard";
  git -C <c> $P`, `P=clean; P+=" -fdx"; …`, `A=-C; A+=" <c>"; A+=" reset --hard"; git $A`, brace/double-
  hop/subst-`+=` siblings.
- The multi-word READ false-positive is closed: `P=log; P+=" --oneline"; git $P` and `P="log --oneline";
  git $P` ALLOW (faithful multi-word binding, not a blanket deny).
- Precision-clean on the high-FP-risk surface preserved: reads with `"$@"`/`"${P[@]}"`/`"$BRANCH"`,
  `IFS=, read`, benign subst, `V=$(git rev-parse)`, worktree function/set-- mutations (0 FP).

### Positive observations
- The faithful binding model is a real improvement: it closes the r14 += leak AND removes the multi-word-
  read FP simultaneously, with general (not per-fixture) logic. 31 unit + liveness gate green, SSH-signed
  (Good ED25519), faces-last settle holds (2 faces, lib.rs-only fix, no Cargo.lock churn), 0 anti-pattern
  key churn, single guard dep (serde_json), no new CVE surface.

---

## Residual risk (single most likely production failure if merged as-is)
An agent (or an injected instruction via the lethal-trifecta surface) emits
`bash -c 'git -C <canonical> $(printf "reset\t--hard")'` or
`bash -c "P=reset; P+=\$'\t--hard'; git -C <canonical> \$P"` — both statically-resolvable, ZERO command
output read at runtime, ZERO read-from-stdin (pure escape decoding) — and it silently ALLOWs, re-
contaminating the canonical checkout and reproducing FRIC-022/FRIC-1781062867 while all 31 tests and the
liveness gate stay green. The leak survives because the guard does not model bash's escape decoding inside
printf/echo-e/`$'…'` values, and r15's loosened binding filter now binds the literal-backslash result
(a REGRESSION vs r14 for the pure-`=` printf form).

Secondary (carried, unchanged): the hook shim fails OPEN if the Rust binary is unbuilt (main.rs maps any
error → SUCCESS); ensure CI/branch-protection builds it (structural enforcement, not hook reliance).
Genuinely runtime-unknowable family (pre-existing, out of scope, faithful — all fail closed): `read` from
stdin/pipe; `$(cat)`/`$(curl)` opaque stdout; PATH/symlink alias `g`→git binary; interactive-only `alias`.

## Required to clear
1. Close F1-R15: (a) make `static_command_output` printf (lib.rs:2193) decode backslash escapes (`\t \n
   \r \f \v \0NN \xHH`) for the FORMAT string and for a `%b` ARG exactly as bash does, so the produced
   text carries REAL whitespace and `emit_substitution_output`'s re-quote gate (lib.rs:1853) fires; treat
   `echo -e` the same. (b) Decode an ANSI-C `$'…'` binding RHS through the same path, OR restore a fail-
   closed binding filter in `collect_same_line_bindings` (lib.rs:1774) that rejects any value containing a
   backslash escape the guard cannot prove whitespace-free (so `$name` stays unresolved → fail closed,
   matching `P="reset --hard"`). Fix both the substitution-decode and the binding-filter — they are
   independently open.
2. Add a DENY fixture for EVERY confirmed F1-R15 reproduction: `git -C <c> $(printf "reset\t--hard")`;
   the backtick/`echo -e`/whole-argv variants; `P=$(printf "reset\t--hard"); git -C <c> $P`;
   `P=$(printf "%b" "reset\t--hard"); git -C <c> $P`; `P=reset; P+=$(printf "\t--hard"); git -C <c> $P`;
   `P=$'reset\t--hard'; git -C <c> $P`; `P=reset; P+=$'\t--hard'; git -C <c> $P`; plus the `\n`/`\040`/
   `\x20`/`\011` sibling escapes. Current fixtures structurally cannot catch them (no escape-decode shape).
3. Keep the multi-word-read ALLOW fixtures (`P=log; P+=" --oneline"; git $P`) so the decode fix does not
   re-introduce the read FP.
4. Re-run 31 + liveness + new fixtures; re-drive the TASK-2 sweep through the real binary with a FOREIGN
   session-cwd; confirm zero false positives on the merge-train + retrain + multi-word reads + worktree
   function/set-- forms.
5. Re-state the convergence claim honestly only after auditing the ESCAPE-DECODE axis (printf format,
   printf %b, echo -e, ANSI-C `$'…'`-into-binding) in addition to the value-assembly (`+=`) and
   substitution-into-binding axes already covered — and confirm the sole residual is genuinely runtime-
   unknowable `$(prog)`/read-from-stdin.

## Security Checklist
- [x] No hardcoded secrets (lib.rs is pure policy logic; only dep is serde_json; no new deps)
- [~] All inputs validated — r14 axes (scalar `+=` value-assembly; multi-word-read FP) now CLOSED via the
      faithful binding model; but the guard does NOT model bash escape decoding inside printf/`%b`/
      `echo -e`/`$'…'`-into-binding, and r15's loosened binding filter binds literal-backslash values r14
      fail-closed → statically-resolvable real-mutation ALLOW + a REGRESSION (F1-R15)
- [~] Injection prevention — r14 += class CLOSED; F1-R15 OPEN (escape-decode-assembled canonical
      mutation). Array-bound worktree over-deny (F2-R15) is a fail-closed regression, not an injection hole.
- [x] Authorization/policy enforced for modeled forms; 34/34 legit + worktree/fn/retrain ALLOW with 0 FP;
      multi-word reads, `IFS=, read`, quoted-arg reads, benign subst, worktree function/set-- mutations all
      preserved (the 1 array worktree over-deny is F2-R15, tracked separately)
- [x] Dependencies audited — single dep serde_json; no new guard deps; no CVE surface in delta
- [x] Tests + liveness gate green (31 unit + gate Pass 1); SSH-signed (Good ED25519); faces-last (2 faces,
      lib.rs-only fix, no Cargo.lock churn); 0 anti-pattern key churn
