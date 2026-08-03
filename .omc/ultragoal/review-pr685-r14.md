# Security Review — PR #685 ROUND 14 (G011 canonical-checkout guard) — CONVERGENCE CHECK

- Repo: jason931225/oyatie  Branch: agent/g011-checkout-guard  Head: d2e5d3b34  Base: dev
- r14 delta vs r13 (165ec2068..d2e5d3b34): two commits.
  - `100e28a6e` `fix(checkout-guard): static-subst-into-binding + two-phase normalize + def-strip (review #685 r13)`
    — lib.rs ONLY (+158/-10, no Cargo.lock churn). Lands three changes:
    (1) `emit_substitution_output` (lib.rs:1834) re-quotes a decoded command-substitution result when it
    is an assignment RHS AND contains whitespace, gated by `out_ends_with_assignment_lhs` (lib.rs:1847),
    so `P=$(echo "reset --hard")` fails closed (r13 F1). (2) Two-phase `normalize_static_expansions`
    (lib.rs:1308): a substitution-resolving PRE-PASS with empty bindings (`$var`/`$@`/`$N`/arrays stay
    literal, lib.rs:2028/2072) runs BEFORE collecting positionals/bindings/arrays, so `set -- $(echo …)`
    and `read … <<< "$(…)"` reassemble before capture. (3) `strip_function_defs` (lib.rs:1399) removes
    def spans so a function body (`git "$@"`) is not evaluated live at the def site; a body with its own
    `set --` is emitted verbatim (`body_rebinds_positionals`, lib.rs:1505) (r13 F2). 5 new DENY fixtures
    (lib.rs:3019-3024).
  - `d2e5d3b34` `chore: settle generated cloud-ci faces` — 2 *.generated.json faces ONLY.
- Reviewer: fresh-context security-reviewer (Claude Opus), Torvalds/attacker lens,
  /using-superpowers + /using-agent-skills + /ultraqa, all FOREGROUND, built from
  /Users/jasonlee/oyatie-worktrees/g011-checkout-guard. FOREIGN session-cwd (/tmp), mutation arrives ONLY
  via -C/binding indirection, every candidate confirmed against REAL bash with an argv-capturing fakegit
  (dirty->clean) that EXECs real git so canonical mutation is genuine.

## VERDICT: BLOCK

**Risk Level: HIGH.** r14 closes ALL EIGHT r13 reproductions — the static-substitution-into-binding
launder is fixed: `P=$(echo "reset --hard"); git -C <c> $P`, `P=$(echo "-C <c> reset --hard"); git $P`,
the printf/backtick/nested/double-hop/subst→set--/subst→read-ra siblings ALL now DENY (rc=2) through the
production binary. The r13 F2 worktree over-deny is mostly fixed too (`strip_function_defs` makes
`g(){ git "$@"; }; g -C <wt> reset --hard` and `set -- reset --hard; git -C <wt> "$@"` ALLOW again). The
36-form accumulated corpus is 36/36 DENY. The FP sweep is 0 false positives across 38 legit forms +
retrain. Mechanics are green (30 unit + liveness Pass 1, SSH-signed Good ED25519, faces-last holds, no
key laundering).

BUT the r14 convergence claim — "the convergent rule now holds and the sole residual is runtime-
unknowable `$(cat)`/`$(curl)`/read-from-stdin" — is **FALSE**. I constructed a **statically-resolvable
real-mutation bypass that uses NO command substitution at all and NO runtime-unknowable input**: the
shell scalar append operator `+=`. Confirmed DIRTY→guard ALLOW(rc=0)→real-bash CLEAN end-to-end through
the PRODUCTION binary with a foreign session-cwd and an argv-capturing fakegit. Per the strict VERDICT
RULE ("BLOCK on a statically-resolvable real-mutation bypass") this blocks.

The single most damning leak (every word a literal constant — NO `$(prog)`, NO `echo`/`printf` decode,
NO stdin):
```
  P="reset --hard"; git -C <c> $P       -> rc=2 DENY  (r13/r14: = with space → value_ok rejects → $P unresolved → fail closed)
  P=reset; P+=" --hard"; git -C <c> $P  -> rc=0 ALLOW *** LEAK *** (MUTATES canonical: reset --hard)
  P=clean; P+=" -fdx";  git -C <c> $P   -> rc=0 ALLOW *** LEAK *** (MUTATES canonical: untracked DESTROYED)
  A=-C; A+=" <c>"; A+=" reset --hard"; git $A -> rc=0 ALLOW *** LEAK *** (whole argv assembled via +=)
```
Root cause (two independent gaps, both in the binding model — lib.rs):
- `collect_same_line_bindings` (lib.rs:1745) splits on whitespace/`;` and uses `split_once('=')`. For
  the token `P+=` it yields name=`P+`, which FAILS `name_ok` (the `+` is not `_`/alphanumeric,
  lib.rs:1751). The append is therefore SILENTLY DROPPED; only the FIRST assignment `P=reset` binds.
  Bash instead CONCATENATES `reset` + ` --hard` → `P="reset --hard"`, then `git -C <c> $P` word-splits
  → `[reset][--hard]`. There is no `+=` accumulation path anywhere in the binding/array collectors.
- `out_ends_with_assignment_lhs` (lib.rs:1847) — even the command-substitution variant
  `P+=$(echo " --hard")` evades the r14 re-quote: after `P+=` is emitted, `out` ends with `P+=`;
  `strip_suffix('=')` gives `…P+`; reading identifier chars backward from `+` yields an EMPTY
  `name_rev` (`+` is not an identifier char) → returns FALSE → the whitespace value is emitted bare,
  not re-quoted. So both the literal `+=` and the substitution `+=` forms leak.

**NEW CLASS or VARIANT?** This is a **FRESH MECHANISM**, not another spelling of r13's binding-
indirection. r13 F1 was the static-substitution *decoder* laundering `$(echo …)` into a clean literal
that the binding collector then first-word-bound; r14 FIXED that exact path (re-quote + two-phase).
The r14 leak is a different construct entirely: the bash scalar-append operator `+=`, which the binding
collector has NEVER modeled (it predates and is orthogonal to substitution decoding). The cleanest PoC
contains ZERO substitution (`P=reset; P+=" --hard"`). It is the SAME r1→r13 META-pattern (the effective-
argv model diverges from bash at a value-assembly construct the collector doesn't model, and the
multi-word result is flattened to a first-word binding that drops the trailing mutating flag), but it is
a genuinely new mechanism — `+=` concatenation — not a re-spelling of any previously-named binding path.

---

## Harness (commands + exact exit codes)

Built the real binary: `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out
/tmp/r14-guard-bin` → BUILD SUCCEEDED (Mach-O arm64). Driven via JSON hook payload
`{"tool_input":{"command":"…"}}` on stdin (the real enforcement surface),
`env OYA_CANONICAL_CHECKOUT=<canon>`, binary cwd=/tmp (canonical != cwd). rc=2 ⇒ DENY, rc=0 ⇒ ALLOW.

**Adjudication discipline (real-mutation only counts):** per candidate (1) fresh DIRTY real git repo
(` M tracked.txt` + `?? untracked.txt`); (2) guard rc via real binary with the canonical path; (3) the
EXACT command in `bash -c` from FOREIGN cwd /tmp with an argv-capturing fakegit that EXECs real git, so
`-C <canon> reset --hard` genuinely mutates; (4) diff `git status --porcelain` before/after. A FINDING
requires `guard_rc=0 AND mutated=yes`. Adjudicator self-test sound (`-C <canon> {reset --hard|clean
-fdx|restore .}` mutate+DENY; `status`/bare-foreign-`reset` clean+ALLOW). Argv-capture confirms bash
hands git `[-C][<canon>][reset][--hard]` (or `clean -fdx`) in each leak.

### TASK 1 — full accumulated corpus (r1..r13 named + ALL 8 r13 reproductions) — PASS
36-form real-mutation corpus (r1 basics; r2 wrappers; r3 sh -c/GIT_DIR/GIT_WORK_TREE; r4 nested;
r5 `$(echo git)`/eval/`bash -c "$()"`; r6 param-subst; r7 brace; r9 ANSI-C; r10 set--/function; r11 ALL;
r12 ALL EIGHT; r13 ALL EIGHT including `P=$(echo "reset --hard"); git -C <c> $P` and printf/backtick/
nested/double-hop/subst→set--/subst→read-ra):
- **36/36 DENY (rc=2). 0 ALLOW.** The 8 r13 reproductions are now FIXED through the production binary.
  (Two rows show mut=no — `switch other` no-ops because no such branch exists, and bare `git "$@"` has
  empty positionals — both still correctly DENIED, faithful.)

### TASK 2 — CONVERGENCE TEST — ONE STATICALLY-RESOLVABLE REAL-MUTATION BYPASS (BLOCK); r13 F1 path CLOSED
Adjudicated every binding/word-synthesis mechanism the prompt named vs REAL bash (foreign cwd) via the
production binary.

CLOSED (caught — the r14 fixes WORK): the r13 F1 launder is dead —
```
  P=$(echo "reset --hard"); git -C <c> $P              rc=2 (r14 re-quote → $P unresolved → fail closed)
  P=$(echo "-C <c> reset --hard"); git $P              rc=2
  P=$(printf '%s' "-C <c> clean -fdx"); git $P         rc=2
  P=`echo "-C <c> reset --hard"`; git $P               rc=2 (backtick RHS)
  P="$(echo "reset --hard")"; git -C <c> $P            rc=2 (double-quoted-substitution RHS)
  export/declare/readonly/typeset P=$(echo "reset --hard"); git -C <c> $P  rc=2 (assignment-prefix forms)
  A=$(echo "-C <c> reset --hard"); B=$A; git $B         rc=2 (double hop)
  set -- $(echo "-C <c> reset --hard"); git "$@"        rc=2 (subst→set--, two-phase reassembles)
  read -ra P <<< "$(echo "-C <c> reset --hard")"; git "${P[@]}"  rc=2 (subst→read-ra)
  P=$(echo $(echo "-C <c> reset --hard")); git $P       rc=2 (nested subst)
  P=; P+=$(echo "reset --hard"); git -C <c> $P          rc=2 (single += of full multiword — value_ok rejects)
  P=$(echo res); git -C <c> ${P}et --hard               rc=2 (partial subst + literal recombine)
  T=$(echo <c>); git -C $T reset --hard                 rc=2 (subst → -C TARGET only; literal verb intact)
  f(){ g "$@"; }; g(){ git "$@"; }; f -C <c> reset --hard rc=2 (mutually-recursive functions)
  g(){ { git "$@"; }; }; g -C <c> reset --hard           rc=2 (nested braces in body)
  g(){ cat <<EOF…EOF; git "$@"; }; g -C <c> reset --hard rc=2 (here-doc in body)
  declare -a P=([0]=reset [1]=--hard); git -C <c> "${P[@]}" rc=2 (indexed array literal)
  P=(reset); P+=(--hard); git -C <c> "${P[@]}"           rc=2 (ARRAY += — modeled correctly)
  P=$(cat file); git -C <c> $P                           rc=2 (opaque $(cat) → fail closed)
  read v </dev/null; git -C <c> $v --hard                rc=2 (read → fail closed)
```
CONFIRMED LEAK (guard_rc=0 ALLOW; REAL canonical mutation, foreign cwd; production binary; ALL words
literal constants — NO `$(prog)`, NO `echo`/`printf` decode, NO stdin):
```
  rc=0 mut=yes  P=reset; P+=" --hard"; git -C <c> $P             argv=[-C <c> reset --hard]   (PURE LITERAL — no subst)
  rc=0 mut=yes  P=clean; P+=" -fdx";  git -C <c> $P              argv=[-C <c> clean -fdx]     (untracked DESTROYED)
  rc=0 mut=yes  P=reset; P+=" --hard"; git -C <c> ${P}           argv=[-C <c> reset --hard]   (brace use site)
  rc=0 mut=yes  A=-C; A+=" <c>"; A+=" reset --hard"; git $A      argv=[-C <c> reset --hard]   (whole argv via +=)
  rc=0 mut=yes  P=$(echo reset); P+=$(echo " --hard"); git -C <c> $P  argv=[-C <c> reset --hard]  (subst += variant)
  rc=0 mut=yes  P=reset; P+=$(echo " --hard"); git -C <c> $P     argv=[-C <c> reset --hard]
  rc=0 mut=yes  P=$(echo clean); P+=$(echo " -fdx"); git -C <c> $P  argv=[-C <c> clean -fdx]
  rc=0 mut=yes  P=$(echo "-C"); P+=$(echo " <c>"); P+=$(echo " reset --hard"); git $P  argv=[-C <c> reset --hard]
```
Discriminator (isolates the gap precisely — `=` vs `+=`):
```
  P="reset --hard"; git -C <c> $P       rc=2 DENY  (= w/ space → value_ok rejects → $P unresolved → fail closed)
  P=reset; P+=" --hard"; git -C <c> $P  rc=0 LEAK  (+= dropped by binding collector; $P=reset; --hard lost)
  P=restore; P+=" ."; git -C <c> $P     rc=2 DENY  (first word `restore` is itself a blocked verb → caught)
  P=log; P+=" --oneline"; git -C <c> $P rc=0 ALLOW (read op — a += fix would NOT create an FP here)
```
A LEAK occurs precisely when a scalar `+=` append puts the dangerous portion (`--hard`/`-fdx`) AFTER the
first word, because `collect_same_line_bindings` never models `+=` (token `P+=` → name `P+` → name_ok
fails → append dropped), and `out_ends_with_assignment_lhs` returns false on a `P+=` prefix (so the r14
re-quote also misses the substitution `+=` variant). Real bash concatenates the append and word-splits
at the `$P` use site.

Root cause (lib.rs):
- `collect_same_line_bindings` (lib.rs:1745/1748) — `split_once('=')` on `P+=` → name `P+` → `name_ok`
  (lib.rs:1751) fails on `+`; no `+=` accumulation path exists. First assignment binds, append dropped.
- `out_ends_with_assignment_lhs` (lib.rs:1847) — `…P+=` → strip `=` → `…P+` → backward identifier read
  stops at `+` → empty `name_rev` → false; so `emit_substitution_output` (lib.rs:1834) emits the
  whitespace value bare instead of re-quoting it for the `+=` substitution variant.
Net: the r14 re-quote + two-phase + def-strip close the substitution-launder and the function over-deny,
but the binding model still does not understand `+=`, so a value assembled by append leaks the trailing
mutating flag — exactly the multi-word-first-word-binding meta-failure, via a new operator.

RESIDUAL genuinely runtime-unknowable (verified, behave correctly, founder-accepted): `$(cat file)`
(rc=2 — guard treats opaque subst as unresolved → fail closed), `$(curl …)` (rc=2 fail closed), `read`
from stdin/pipe (rc=2 fail closed). These are NOT the sole residual: the `+=` class above is parse-time-
determinable and BLOCKS. Carried/unchanged (runtime-unknowable family, pre-existing, out of scope):
PATH/symlink `g`→git binary; interactive-only `alias`; opaque `$(prog)` stdout.

### TASK 3 — FALSE-POSITIVE SWEEP (must ALLOW) — 38/38 legit ALLOW; F2 r13 over-deny mostly FIXED
- **All legit forms ALLOW (rc=0), 0 FP.** Merge-train on canonical (status/log --oneline -5/diff/show
  HEAD/rev-parse HEAD/fetch --all --prune/commit -m "msg"/commit -m "reset --hard fix"/merge --ff-only/
  pull --ff-only); reads with $-args (`git -C <c> log "$BRANCH"`, `set -- --oneline -5; git -C <c> log
  "$@"`, `read -ra P <<< "--oneline -5"; … "${P[@]}"`, `P=(--oneline -5); … "${P[@]}"`,
  `git -C <c> show "$COMMIT"`, `--format="%H"`); `V=$(git -C <c> rev-parse HEAD)`; benign subst
  (`git -C <c> $(echo status)`, `$(echo log) --oneline`, `git -C <c> log $(git -C <c> rev-parse HEAD)`);
  `IFS=, read -ra parts; git -C <c> log ${parts[0]}`; `set -- a b c; echo "$@"`; `read -a P; echo
  "${P[@]}"`; for/while; `reset(){ echo "$@"; }; reset --hard` (non-git collide); `$(date)`; `$EDITOR`;
  `sudo sh -c "echo"`; `grep -r git /path`; `P=log; P+=" --oneline"; git -C <c> $P` (the += shape on a
  READ op — confirms a += fix would not over-deny read ops).
- **retrain-style ops on /srv/other (non-canonical): ALLOW** (reset --hard/clean -fdx/checkout — none
  target the canonical; argv shows the other repo, canonical untouched).
- **F2 r13 worktree over-deny — MOSTLY FIXED.** r14's `strip_function_defs` restores ALLOW for the
  function form `g(){ git "$@"; }; g -C <wt> reset --hard` (rc=0, mutates WT not canon) and the
  set-- form `set -- reset --hard; git -C <wt> "$@"` (rc=0). Residual: `P=(-C <wt> reset --hard);
  git "${P[@]}"` still DENIES (rc=2) — the array-bound `-C <wt>` cannot be proven non-canonical, so it
  fail-closes. This is a usability over-deny (fail-closed, SAFE), secondary, not a security hole and not
  the basis of the verdict.

### TASK 4 — mechanics — all green
- `//tools/oya-checkout-guard-app:oya-checkout-guard-app-unittest` → **30 passed; 0 failed** (5 new r13
  DENY fixtures present and pass). NOTE: NO fixture exercises the TASK-2 LEAK form (`P=reset;
  P+=" --hard"; git -C <c> $P` or any scalar `+=` shape) — the recurring r1→r13 reason the leak survives
  a 30-green suite.
- `//cloud/cloud-ci/gates/oya-cloud-ci-enforcement-liveness-app:…-unittest` → Pass 1 (Fail 0).
- SSH sigs: 100e28a6e + d2e5d3b34 both carry Good ED25519 signature
  (SHA256:5grGNUtX9Zgmy1SWne6wF9DR8W1ElUQaF/Z8SYRz8E8). (The local `allowed_signers: Not a directory`
  warning is a worktree-config artifact, not a signature-validity defect — the signature verifies Good.)
- Faces-last settle: d2e5d3b34 touches ONLY 2 *.generated.json faces (accounting-registry, scm-facts).
  Fix commit 100e28a6e = lib.rs ONLY (+158/-10, no Cargo.lock churn). Holds.
- Baseline key-diff both ways (165ec2068..d2e5d3b34 over generated faces): **0 anti-pattern keys**
  (forbidden_foundry|unjustified|unowned|unreachable) added OR removed. Face changes are pure provenance
  metadata (source_inputs_digest, last_touch_commit→100e28a6e, head_time_secs). No laundering.

---

## Findings (file:line, severity, confidence)

### [HIGH, confidence HIGH] F1-R14 — scalar `+=` append is not modeled by the binding collector → the appended mutating flag is dropped, ALLOWing a canonical reset/clean (statically resolvable; NO substitution, NO stdin)
- Location: lib.rs:1745 (`collect_same_line_bindings` `split_once('=')` → `P+=` yields name `P+`,
  fails `name_ok` at lib.rs:1751; no `+=` accumulation anywhere); lib.rs:1847
  (`out_ends_with_assignment_lhs` returns false on a `P+=` prefix, so the r14 re-quote at
  `emit_substitution_output` lib.rs:1834 also misses the `+=` substitution variant).
- Confirmed (rc=0; REAL bash MUTATES from foreign cwd; production binary; DIRTY→CLEAN; argv-capture):
  `P=reset; P+=" --hard"; git -C <c> $P` (PURE LITERAL); `P=clean; P+=" -fdx"; git -C <c> $P` (untracked
  destroyed); `P=reset; P+=" --hard"; git -C <c> ${P}`; `A=-C; A+=" <c>"; A+=" reset --hard"; git $A`
  (whole argv via +=); plus the command-substitution `+=` variants `P=$(echo reset); P+=$(echo
  " --hard"); …` and `P=$(echo "-C"); P+=$(echo " <c>"); P+=$(echo " reset --hard"); git $P`.
- Exploitability: local, unauthenticated, single short one-liner, no length cap, shell-agnostic; `+=` is
  a standard bash assignment operator. `P=reset; P+=" --hard"` is utterly idiomatic shell. Trivially
  emitted by an agent or an injected instruction (the lethal-trifecta surface CLAUDE.md warns of).
- Blast radius: silent canonical `reset --hard` / `clean -fdx`, reproducing FRIC-022/FRIC-1781062867
  while all 30 unit tests + the liveness gate stay green.
- STATICALLY RESOLVABLE: the cleanest PoC has NO command substitution at all — every byte is a literal
  the parser can read directly. NOT the runtime-unknowable `$(prog)`/stdin residual — the BLOCK class.
- Classification: FRESH MECHANISM (bash scalar `+=` concatenation / binding-collector `+=` blind spot),
  distinct from r13's static-substitution launder (which r14 closed). Same r1→r13 META-pattern
  (effective-argv model diverges from bash at an unmodeled value-assembly construct; multi-word result
  flattened to first-word binding, trailing mutating flag dropped), new operator.

### [LOW→MEDIUM, confidence HIGH] F2-R14 — residual fail-closed over-deny on array-bound non-canonical worktree mutation
- Location: lib.rs:418 (`subcommand_carries_value_expansion`/retargetable forcing blocked_target) +
  array/unbound literal-retention (lib.rs:2028/2072 two-phase keeps unbound array literal).
- Confirmed (rc=2; targets a NON-canonical worktree, does NOT touch canonical): `P=(-C <wt> reset
  --hard); git "${P[@]}"`. r14 FIXED the two function/`set --` r13 F2 forms (`g(){ git "$@"; };
  g -C <wt> reset --hard` and `set -- reset --hard; git -C <wt> "$@"` now ALLOW); this array form
  remains fail-closed.
- Severity: usability/fail-closed regression, NOT a security hole (the unavoidable cost of not being
  able to prove an array-hidden `-C <wt>` is non-canonical). Moot for the verdict — F1-R14 independently
  blocks.

### Note — the r1→…→r14 meta-pattern, now at the value-assembly axis
r14 correctly closed the eight r13-named static-substitution forms, restored the function/set-- worktree
allows, kept `bash -c "$(…)"` grouping + reads-with-$-args + the merge-train precision-clean, and the
two-phase + re-quote + def-strip are sound for everything they model. But the convergence claim again
treats "the named forms denied" as "static closure reached," which is false: the binding model still
does not understand the bash `+=` append operator. `collect_same_line_bindings` first-word-binds the
base assignment and drops the append; `out_ends_with_assignment_lhs` does not recognize a `P+=` LHS so
the re-quote misses the substitution `+=` variant too. The durable fix is to make the binding collector
model `+=`: detect a `NAME+=VALUE` token, CONCATENATE VALUE onto the existing binding for NAME (creating
it if absent), and run the resulting (possibly multi-word) value through the SAME value_ok / re-quote
fail-closed path the `=` assignment already uses (so a multi-word `+=` result leaves `$NAME` unresolved
→ fail closed). Equivalently: teach `out_ends_with_assignment_lhs` to accept a trailing `+` before `=`
AND teach `collect_same_line_bindings` `+=` accumulation — both, since both gates are open.

### Resolved since r13 (verified)
- All EIGHT r13 reproductions DENY through the production binary (static-substitution-into-binding
  launder fixed via re-quote + two-phase): `P=$(echo "reset --hard"); git -C <c> $P` and the printf/
  backtick/double-quoted/nested/double-hop/subst→set--/subst→read-ra siblings.
- r13 F2 worktree over-deny FIXED for the function and `set --` forms (`strip_function_defs` +
  `body_rebinds_positionals`); only the array-bound form remains fail-closed (F2-R14, secondary).
- Precision-clean on the high-FP-risk surface: reads with `"$@"`/`"${P[@]}"`/`"$BRANCH"`, `IFS=, read`,
  `grep -r git /path`, `bash -c "$(…)"` grouping, benign subst, `V=$(git rev-parse)` (0 FP).

### Positive observations
- r13-named fixes are general (not per-fixture), correct, and precision-clean. 30 unit + liveness gate
  green, SSH-signed (Good ED25519), faces-last settle holds (2 faces, lib.rs-only fix, no Cargo.lock
  churn), no key laundering, single guard dep (serde_json), no new CVE surface.

---

## Residual risk (single most likely production failure if merged as-is)
An agent (or an injected instruction via the lethal-trifecta surface) emits
`bash -c 'P=reset; P+=" --hard"; git -C <canonical> $P'` or `bash -c 'A=-C; A+=" <canonical>";
A+=" reset --hard"; git $A'` — both statically-resolvable, ZERO command substitution, ZERO read-from-
stdin (pure literal `+=` concatenation) — and it silently ALLOWs, re-contaminating the canonical
checkout and reproducing FRIC-022/FRIC-1781062867 while all 30 tests and the liveness gate stay green.
The leak survives because the binding collector does not model the bash `+=` append operator, so it
first-word-binds the base assignment and drops the appended `--hard`/`-fdx`.

Secondary (carried, unchanged): the hook shim fails OPEN if the Rust binary is unbuilt (main.rs maps any
error → SUCCESS); ensure CI/branch-protection builds it (structural enforcement, not hook reliance).
Genuinely runtime-unknowable family (pre-existing, out of scope, faithful — all fail closed): `read`
from stdin/pipe; `$(cat)`/`$(curl)` opaque stdout; PATH/symlink alias `g`→git binary; interactive-only
`alias`.

## Required to clear
1. Close F1-R14: make the binding model understand the bash `+=` append operator. In
   `collect_same_line_bindings` (lib.rs:1745), detect a `NAME+=VALUE` token, CONCATENATE VALUE onto the
   existing binding for NAME (create if absent), then run the combined value through the SAME value_ok
   path so a multi-word result leaves `$NAME` unresolved → fail closed (matching the `P="reset --hard"`
   handling). AND teach `out_ends_with_assignment_lhs` (lib.rs:1847) to accept an optional trailing `+`
   before the `=` so `emit_substitution_output` re-quotes the substitution `+=` variant too. Both gates
   are independently open; fix both.
2. Add a DENY fixture for EVERY confirmed F1-R14 reproduction: `P=reset; P+=" --hard"; git -C <c> $P`;
   `P=clean; P+=" -fdx"; git -C <c> $P`; `P=reset; P+=" --hard"; git -C <c> ${P}`; `A=-C; A+=" <c>";
   A+=" reset --hard"; git $A`; and the substitution variants `P=$(echo reset); P+=$(echo " --hard");
   git -C <c> $P` and `P=$(echo "-C"); P+=$(echo " <c>"); P+=$(echo " reset --hard"); git $P`. Current
   fixtures structurally cannot catch them (no `+=` shape anywhere in the suite).
3. Re-evaluate F2-R14: decide whether the array-bound non-canonical worktree mutation `P=(-C <wt> reset
   --hard); git "${P[@]}"` should ALLOW (matching r12/r13's certified worktree behavior) or stay fail-
   closed; if the latter, document the intentional usability over-deny so it is not re-flagged.
4. Re-run 30 + liveness + new fixtures; re-drive the TASK-2 sweep through the real binary with a FOREIGN
   session-cwd; confirm zero false positives on the merge-train + retrain + `IFS=, read` + `grep -r git
   /path` + quoted-arg-read + the `P=log; P+=" --oneline"` read-op shape.
5. Re-state the convergence claim honestly only after auditing the VALUE-ASSEMBLY axis (`+=` scalar
   append, and any other bash assignment operator the collector does not model) in addition to the
   substitution-into-binding axis already covered — and confirm the sole residual is genuinely
   runtime-unknowable `$(prog)`/read-from-stdin.

## Security Checklist
- [x] No hardcoded secrets (lib.rs is pure policy logic; only dep is serde_json; no new deps)
- [~] All inputs validated — r13 axes (static-substitution-into-binding launder via re-quote + two-phase;
      function-def-body over-eval via strip) now CLOSED; but the bash scalar `+=` append operator is not
      modeled by `collect_same_line_bindings` (first-word-binds the base, drops the append) and
      `out_ends_with_assignment_lhs` does not recognize a `P+=` LHS → statically-resolvable real-mutation
      ALLOW (F1-R14)
- [~] Injection prevention — r13 classes CLOSED; F1-R14 OPEN (scalar `+=`-assembled canonical mutation).
      Array-bound worktree over-deny (F2-R14) is a fail-closed regression, not an injection hole.
- [x] Authorization/policy enforced for modeled forms; 38/38 legit + retrain ALLOW with 0 FP;
      `grep -r git /path`, `IFS=, read`, quoted-arg reads, `bash -c "$(…)"` grouping, function/set-- 
      worktree mutations all preserved (the 1 array worktree over-deny is F2-R14, tracked separately)
- [x] Dependencies audited — single dep serde_json; no new guard deps; no CVE surface in delta
- [x] Tests + liveness gate green (30 unit + gate Pass 1 / unittest Pass 1); SSH-signed (Good ED25519);
      faces-last (2 faces, lib.rs-only fix, no Cargo.lock churn); no key laundering
