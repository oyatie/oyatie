# Security Review — PR #685 ROUND 13 (G011 canonical-checkout guard) — CONVERGENCE CHECK

- Repo: jason931225/oyatie  Branch: agent/g011-checkout-guard  Head: 165ec2068  Base: dev
- r13 delta vs r12 (ffca115dd..165ec2068): two commits.
  - `170a27b37` `fix(checkout-guard): convergent fail-closed on argv-position binding (review #685 r12)`
    — lib.rs ONLY (+62/-5, no Cargo.lock churn). Lands the CONVERGENT rule:
    (1) `subcommand_carries_value_expansion` (lib.rs:1642) — when the git subcommand token carries a
    `$`/backtick expansion (text arbitrary → could hide `-C <canonical>`), force `blocked_target=true`
    (lib.rs:418) regardless of cwd; brace/glob/tilde EXCLUDED (verb-in-place, can't retarget) so
    `git -C <wt> {reset,} --hard` stays ALLOW. (2) unbound `$@`/`$*`/`${@}`/array expansions (no visible
    `set --`/`read` in scope) stay LITERAL (lib.rs:1754, 1889, 1919) → `has_unresolved_expansion` →
    `blocked_operation=true` → DENY. 7 new DENY fixtures (lib.rs:2869-2876).
  - `165ec2068` `chore: settle generated cloud-ci faces` — 2 *.generated.json faces ONLY.
- Reviewer: fresh-context security-reviewer (Claude Opus), Torvalds/attacker lens,
  /using-superpowers + /using-agent-skills + /ultraqa, all FOREGROUND, built from
  /Users/jasonlee/oyatie-worktrees/g011-checkout-guard. FOREIGN session-cwd (/tmp), mutation arrives
  ONLY via -C/binding indirection, every candidate confirmed against REAL bash with an argv-capturing
  fakegit (dirty->clean) that delegates to real git so canonical mutation is genuine.

## VERDICT: BLOCK

**Risk Level: HIGH.** r13 closes ALL EIGHT r12 reproductions — every r12 BLOCK form now DENYs (rc=2)
through the PRODUCTION binary: literal array assignment `P=(…); git "${P[@]}"`, quoted multi-word
string-var split `P="…"; git $P`, eval-hidden `set --`, in-function-body `set --`/`local`, variable
array index `${P[$n]}`, bare unbound `git "$@"`. Zero false positives across the 42-case legit suite +
10 retrain lines. The convergent rule direction (fail-closed on an argv-position `$`-expansion that
can't be proven free of a canonical `-C` + mutating verb) is correct and precision-clean for every form
where the dangerous expansion is STILL VISIBLE as a `$`/backtick in the subcommand token at decision
time.

BUT the r13 convergence claim — "any binding spelling that lands an unresolved $-expansion in the git
subcommand position now denies by construction, so the sole residual is runtime-unknowable
$(prog)/stdin" — is **FALSE**. I constructed **EIGHT distinct statically-resolvable real-mutation
bypasses**, each verified DIRTY→guard ALLOW→real-bash CLEAN end-to-end through the PRODUCTION binary
with a foreign session-cwd and an argv-capturing fakegit. NONE uses a runtime-unknowable
`$(prog)`/stdin — every word is a literal constant, and the substitution is one the guard ITSELF
statically decodes (`static_command_output`, lib.rs:2013, handles `echo`/`printf`). Per the strict
VERDICT RULE ("BLOCK on a statically-resolvable real-mutation bypass") this blocks.

The single most damning leak is the EXACT r12 class-(b) form (quoted/whitespace-bearing string-var
word-split — which the r13 commit message explicitly claims to "subsume") reached through a STATIC
command-substitution RHS that r13 left open:
```
  P="reset --hard"; git -C <canon> $P            -> rc=2 DENY  (r13 fix: $P unresolved → fail closed)
  P=$(echo "reset --hard"); git -C <canon> $P    -> rc=0 ALLOW *** LEAK *** (MUTATES canonical)
```
Root cause: `normalize_static_expansions` (lib.rs:1293) runs a fixpoint. Pass 1 — `expand_with_bindings`
hits `$(echo "reset --hard")` via the `'$'(` arm (lib.rs:1938), calls `static_command_output`
(lib.rs:1940) → produces the LITERAL `reset --hard`, splices it in AS BARE TEXT, yielding
`P=reset --hard; git -C <canon> $P`. Pass 2 — `collect_same_line_bindings` (lib.rs:1656) whitespace-
splits and binds `P=reset` (the word `reset` PASSES `value_ok`; the trailing `--hard` becomes a detached
token). `$P` then resolves to the clean literal `reset`. The guard sees `git -C <canon> reset` (literal
subcommand, no `--hard` arg) → `is_blocked_operation("reset", [])` is FALSE and neither
`has_unresolved_expansion` nor `subcommand_carries_value_expansion` fires (the subcommand is now a clean
literal) → ALLOW. Real bash assigns the FULL `reset --hard` to `P` as one value and `git -C <canon> $P`
word-splits → `[reset][--hard]` → canonical reset. The static decode that r13 relies on to be SAFE is
the very thing that LAUNDERS the `$`-expansion into a clean literal BEFORE the convergent rule can see
it — and then loses the trailing arg.

**NEW CLASS or VARIANT?** VARIANT of r12 class-(b) (quoted/whitespace-bearing string-var word-split),
reached via the `static_command_output` substitution-into-binding path. Same r1→r12 meta-pattern one
layer deeper: r13 added the convergent argv-position-expansion sentinel but it only fires while the
expansion is LITERALLY a `$`/backtick at decision time; the static-substitution decoder resolves
`$(echo …)`/`$(printf …)`/`` `echo …` `` to a clean literal first, so the sentinel never engages, and
the multi-word output is then flattened to a first-word binding (dropping `--hard`/`-fdx`).

---

## Harness (commands + exact exit codes)

Built the real binary: `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out
/tmp/r13-guard-bin` → BUILD SUCCEEDED (Mach-O arm64). Driven via JSON hook payload
`{"tool_input":{"command":"…"}}` on stdin (the real enforcement surface), `env OYA_CANONICAL_CHECKOUT=<canon>`,
binary cwd=/tmp (canonical != cwd). rc=2 ⇒ DENY, rc=0 ⇒ ALLOW.
Calibration (8/8): `git -C <canon> {reset --hard|switch x|clean -fdx|restore .}` → 2;
`{status|log|fetch --all --prune}` → 0; bare `git reset --hard` from foreign cwd → 0.

**Adjudication discipline (real-mutation only counts):** per candidate (1) fresh DIRTY real git repo
(` M tracked.txt` + `?? untracked.txt`); (2) guard rc via real binary with the canonical path; (3)
EXACT command in `bash -c` from FOREIGN cwd /tmp with an argv-capturing fakegit that EXECs real git, so
`-C <canon> reset --hard` genuinely mutates; (4) diff `git status --porcelain`. A FINDING requires
`guard_rc=0 AND mutated=yes`. Adjudicator self-test sound (`-C <canon> reset --hard`/`clean -fdx`
mutate+DENY; `status`/bare-foreign-`reset` clean+ALLOW). Argv-capture confirms bash hands git
`[-C][<canon>][reset][--hard]` (or `clean -fdx`) in each leak.

### TASK 1 — full accumulated corpus (r1..r12 named + ALL reproductions incl 8 r12 cases) — PASS
60-form real-mutation corpus (r1 basics; r2 wrappers; r3 sh -c/GIT_DIR/GIT_WORK_TREE; r4 nested;
r5 `$(echo git)`/eval/`bash -c "$()"`; r6 param-subst; r7 brace; r9 ANSI-C/line-continuation; r10
set--/function; r11 ALL NINE; r12 ALL EIGHT):
- **57/60 DENY (rc=2).** The 8 r12 reproductions are now FIXED through the production binary. 3 ALLOWs
  adjudicated: `git -C <canon> branch -D x` (mut=NO — `branch -D` is a ref delete, not a working-tree
  mutation; out of model scope, faithful and consistent with r1..r12); `runuser -c '…'`
  (mut=NO — runuser absent on this host, inner git never ran; environment artifact, not a guard
  decision about a real mutation); `P=$(echo "-C <canon> reset --hard"); git $P` (mut=**YES** → see
  TASK 2 — this is a real bypass, not a faithful allow).

### TASK 2 — CONVERGENCE TEST — EIGHT STATICALLY-RESOLVABLE REAL-MUTATION BYPASSES (BLOCK)
Adjudicated every binding/word-synthesis mechanism the prompt named vs REAL bash (foreign cwd) via the
production binary. Mechanisms that DENY (caught — the convergent rule + existing machinery WORK):
```
  x=res; y=et; git -C <c> ${x}${y} --hard          rc=2 (concat two bindings → reset)
  x=re; git -C <c> ${x}set --hard                  rc=2 (binding+literal concat)
  git -C <c> r"eset" / 'res''et' / re""set --hard  rc=2 (adjacent quote splice)
  git -C <c> -c alias.x="!git reset --hard" x       rc=2 (shell-alias)
  git -c alias.x="!git -C <c> reset --hard" x       rc=2 (alias-body retarget)
  git --exec-path=/tmp -C <c> reset --hard          rc=2 (option injection)
  git -C <c> rev-parse HEAD; git -C <c> reset --hard rc=2 (`;` hidden mutation — both seen)
  git -C <c> log && git -C <c> reset --hard         rc=2 (`&&` chain)
  P=($(echo "-C <c> reset --hard")); git "${P[@]}"  rc=2 (subst INSIDE array assign — `$(` seen)
  T=$(echo "<c>"); git -C $T reset --hard           rc=2 (subst → -C TARGET only; literal verb intact)
  Q=$(echo "merge x"/"pull"/"rebase x"); git -C <c> $Q  rc=2 (single-word mutating verb survives binding)
  Q=$(echo "restore ."); git -C <c> $Q              rc=2 (restore — verb is first word)
  read v < /dev/stdin; git -C <c> $v --hard         rc=2 (runtime-unknowable → fail closed)
  P=`cat /tmp/nope`; git -C <c> $P                  rc=2 (backtick cat unknowable → fail closed)
```
CONFIRMED LEAKS (guard_rc=0 ALLOW; REAL canonical mutation, foreign cwd; production binary; all words
literal constants the guard ITSELF statically decodes — NO opaque $(prog), NO stdin):
```
  rc=0 mut=yes  P=$(echo "reset --hard"); git -C <c> $P                argv=[-C <c> reset --hard]
  rc=0 mut=yes  P=$(echo "-C <c> reset --hard"); git $P                argv=[-C <c> reset --hard]
  rc=0 mut=yes  P=$(echo -C <c> reset --hard); git $P                  argv=[-C <c> reset --hard]
  rc=0 mut=yes  P=$(printf -- "-C <c> reset --hard"); git $P           argv=[-C <c> reset --hard]
  rc=0 mut=yes  P=`echo "-C <c> reset --hard"`; git $P                 argv=[-C <c> reset --hard]
  rc=0 mut=yes  P=$(echo "clean -fdx"); git -C <c> $P                  argv=[-C <c> clean -fdx]  (untracked DESTROYED)
  rc=0 mut=yes  P=$(echo $(echo "reset --hard")); git -C <c> $P        argv=[-C <c> reset --hard]  (nested subst)
  rc=0 mut=yes  A=$(echo "reset --hard"); B=$A; git -C <c> $B          argv=[-C <c> reset --hard]  (double hop)
  rc=0 mut=yes  set -- $(echo "-C <c> reset --hard"); git "$@"         argv=[-C <c> reset --hard]  (subst→set--)
  rc=0 mut=yes  read -ra P <<< "$(echo "-C <c> reset --hard")"; git "${P[@]}"  argv=[-C <c> reset --hard]
```
Discriminator (isolates the gap):
```
  P="reset --hard"; git -C <c> $P                rc=2 DENY  (quoted RHS dropped by value_ok → $P unresolved → r13 sentinel)
  P=$(echo "reset --hard"); git -C <c> $P        rc=0 LEAK  (static subst decoded to literal, then first-word binding loses --hard)
  P=$(echo "switch"); git -C <c> $P x            rc=2 DENY  (single mutating verb = first word, survives binding)
  P=$(echo "reset"); git -C <c> $P --hard        rc=2 DENY  ($P=reset literal, --hard literal arg → blocked op)
```
A LEAK occurs precisely when a STATIC command substitution (`$(echo …)`/`$(printf …)`/`` `echo …` ``)
produces a MULTI-WORD value whose dangerous arg is NOT the first word — the guard splices the decoded
text in, then `collect_same_line_bindings` binds only the first word (`reset`) and drops the rest
(`--hard`), while real bash binds the whole value and word-splits it at the `git $P` use site.

Root cause (lib.rs):
- `normalize_static_expansions` (lib.rs:1293) decodes `$(echo …)`/`$(printf …)`/backtick via
  `static_command_output` (lib.rs:1940/1976/2013) and splices the output in as BARE TEXT.
- `collect_same_line_bindings` (lib.rs:1656) then whitespace-splits the spliced `P=reset --hard` and
  binds `P=reset` (value_ok passes the clean first word), losing `--hard`.
- The convergent `subcommand_carries_value_expansion`/`has_unresolved_expansion` checks (lib.rs:1642/
  1615) only fire while the subcommand token LITERALLY still carries a `$`/backtick — but the static
  decode has already turned `$P` into the clean literal `reset`, so the sentinel never engages.
Net: r13's convergent rule and r13's own static-substitution decoder are in tension — the decoder
defeats the sentinel for exactly the multi-word case.

RESIDUAL genuinely runtime-unknowable: `$(cat file)`/`$(curl)` (opaque stdout → empty verb position →
no mutating verb visible → faithful ALLOW), `read` from stdin/pipe (fail-closed DENY). These are the
founder-accepted residual and behave correctly — BUT they are NOT the sole residual: the 8 classes above
are parse-time-determinable (the guard decodes `echo`/`printf` itself) and BLOCK. Carried/unchanged
(runtime-unknowable family, pre-existing, out of scope): PATH/symlink `g`→git binary; interactive-only
`alias`; opaque `$(prog)`.

### TASK 3 — FALSE-POSITIVE SWEEP (must ALLOW) — 42/44 legit + 10 retrain; 2 worktree OVER-DENIES
- **42/44 legit ALLOW.** Full merge-train on canonical (status/log/diff/show/rev-parse/fetch --all
  --prune/commit -m "msg"/push/merge --ff-only/pull --ff-only), commit messages containing reset/clean/
  `reset --hard`/`clean -fdx`; reads with $-args (the r13 FP-risk surface): `git -C <c> log "$BRANCH"`,
  `set -- --oneline -5; git -C <c> log "$@"`, `g(){ git -C <c> log "$@"; }; g --oneline`,
  `read -ra P <<< '--oneline -5'; git -C <c> log "${P[@]}"`, `P=(--oneline -5); git -C <c> log
  "${P[@]}"`, `git -C <c> diff "$@"`, `git -C <c> log $(git -C <c> rev-parse HEAD)`,
  `git -C <c> show "$COMMIT"`, `--format="%H"`, metachar args (`"*.rs"`, `"HEAD@{1}"`,
  `"HEAD~1..HEAD"`); `set -- a b c; echo "$@"`, `read -a P; echo "${P[@]}"`, for/while loops,
  `reset(){ echo "$@"; }; reset --hard` (non-git collide), `$(date)`, `$EDITOR`, `sudo sh -c`,
  `IFS=, read -ra parts; git -C <c> log ${parts[0]}`, `grep -r git /tmp`/`rg git /tmp`,
  benign subst `git -C <c> $(echo status)`/`$(echo log) --oneline`.
- **/srv/other-repo retrain-style: 10/10 ALLOW** (ls-remote/fetch/clean -fdx/checkout/add/commit/push/
  merge-base/rev-parse/reset --hard origin/main — none target the canonical).
- **2 OVER-DENIES on legitimate NON-CANONICAL worktree mutations (rc=2):**
  `g(){ git "$@"; }; g -C <wt> reset --hard` and `P=(-C <wt> reset --hard); git "${P[@]}"`.
  Adjudicated: both target the WORKTREE (real bash → `-C <worktree> reset --hard`, mutates the worktree,
  does NOT touch canonical). r12 EXPLICITLY certified BOTH as must-ALLOW (r12 TASK 3 non-canonical
  worktree list). r13 regresses them to DENY because the `-C <wt>` is hidden inside an unresolved
  function-`"$@"`/array binding that the guard cannot prove is non-canonical → fail-closed. The DIRECT
  equivalents still ALLOW (`git -C <wt> reset --hard`, `set -- reset --hard; git -C <wt> "$@"` → rc=0).
  This is a USABILITY regression / fail-closed over-deny, NOT a security hole — and is the unavoidable
  price of r13's convergent rule (the guard can't distinguish `-C <canon>` from `-C <wt>` once inside an
  unresolved binding). Reported as secondary; the verdict is independently forced by TASK 2.

### TASK 4 — mechanics — all green
- `//tools/oya-checkout-guard-app:oya-checkout-guard-app-unittest` → **30 passed; 0 failed** (7 new
  r12 convergent DENY fixtures present and pass). NOTE: NO fixture exercises the TASK-2 LEAK forms
  (`P=$(echo "reset --hard"); git -C <c> $P` and the printf/backtick/nested/double-hop/subst→set--/
  subst→read-ra siblings) — the recurring r1→r12 reason the leak survives 30-green.
- `//cloud/cloud-ci/gates/oya-cloud-ci-enforcement-liveness-app:{…-unittest,…-gate}` → unittest Pass 1;
  gate Pass 1 (Fail 0).
- SSH sigs: 170a27b37 + 165ec2068 both carry Good ED25519 signature
  (SHA256:5grGNUtX9Zgmy1SWne6wF9DR8W1ElUQaF/Z8SYRz8E8).
- Faces-last settle: 165ec2068 touches ONLY 2 *.generated.json faces (accounting-registry, scm-facts).
  Fix commit 170a27b37 = lib.rs ONLY (+62/-5, no Cargo.lock churn). Holds.
- Baseline key-diff both ways (ffca115dd..165ec2068 over generated faces): **0 anti-pattern keys**
  (forbidden_foundry|unjustified|unowned|unreachable) added OR removed. No laundering.

---

## Findings (file:line, severity, confidence)

### [HIGH, confidence HIGH] F1-R13 — static command-substitution decode launders a $-expansion into a clean literal first-word binding, dropping the trailing mutating arg → convergent rule never engages (static, no opaque $(prog))
- Location: lib.rs:1293 (`normalize_static_expansions` fixpoint splices `static_command_output`
  results as bare text); lib.rs:1940/1976 (`expand_with_bindings` `$(`/backtick arms call
  `static_command_output`); lib.rs:2013 (`static_command_output` statically decodes `echo`/`printf`);
  lib.rs:1656 (`collect_same_line_bindings` then binds only the first whitespace word of the spliced
  multi-word value, dropping `--hard`/`-fdx`); lib.rs:1642/1615 (the convergent
  `subcommand_carries_value_expansion`/`has_unresolved_expansion` sentinels only fire while the token
  LITERALLY carries `$`/backtick — already gone after decode).
- Confirmed (rc=0; REAL bash MUTATES from foreign cwd; production binary; DIRTY→CLEAN; argv-capture):
  `P=$(echo "reset --hard"); git -C <c> $P`; `P=$(echo "-C <c> reset --hard"); git $P` (and unquoted /
  printf / backtick / nested `$(echo $(echo …))` siblings); `P=$(echo "clean -fdx"); git -C <c> $P`
  (untracked destroyed); `A=$(echo "reset --hard"); B=$A; git -C <c> $B` (double hop);
  `set -- $(echo "-C <c> reset --hard"); git "$@"`; `read -ra P <<< "$(echo "-C <c> reset --hard")";
  git "${P[@]}"`.
- Exploitability: local, unauthenticated, single short one-liner, no length cap, shell-agnostic
  (`$(echo …)`/backtick/printf substitution + string-var split works in bash AND sh). Trivially emitted
  by an agent or an injected instruction (the lethal-trifecta surface CLAUDE.md warns of). `P=$(echo
  "reset --hard")` is an utterly idiomatic shell construction.
- Blast radius: silent canonical `reset --hard` / `clean -fdx` / `restore .`, reproducing
  FRIC-022/FRIC-1781062867 while all 30 unit tests + the liveness gate stay green.
- STATICALLY RESOLVABLE: `echo`/`printf`/backtick-echo output is decoded by the guard ITSELF
  (`static_command_output`); every word is a literal constant visible at parse time. NOT the
  runtime-unknowable `$(prog)`/stdin residual — the BLOCK class.
- Classification: VARIANT of r12 class-(b) (quoted/whitespace-bearing string-var word-split — the form
  the r13 commit claims to subsume), reached via the static-substitution-into-binding path. Same
  r1→r12 meta-pattern: r13's convergent sentinel only sees the expansion while it is literally
  `$`/backtick; the static decode resolves it to a clean literal first, defeating the sentinel, then
  flattens the multi-word output to a first-word binding.

### [LOW→MEDIUM, confidence HIGH] F2-R13 — fail-closed over-deny on legitimate non-canonical worktree mutations bound via function-`"$@"`/array
- Location: lib.rs:418 (`subcommand_retargetable` forces `blocked_target=true`) combined with the
  unbound-positional/array literal-retention (lib.rs:1754/1889/1919).
- Confirmed (rc=2; targets a NON-canonical worktree, does NOT touch canonical): `g(){ git "$@"; };
  g -C <wt> reset --hard`; `P=(-C <wt> reset --hard); git "${P[@]}"`. r12 explicitly certified both as
  must-ALLOW; r13 regresses them to DENY. DIRECT equivalents (`git -C <wt> reset --hard`,
  `set -- reset --hard; git -C <wt> "$@"`) still ALLOW.
- Severity: usability/fail-closed regression, NOT a security hole. It is the inherent cost of the
  convergent rule (cannot distinguish `-C <canon>` from `-C <wt>` inside an unresolved binding). Whether
  it counts as a blocking real-command FP is debatable; it is moot because F1-R13 independently blocks.

### Note — the r1→…→r13 meta-pattern, now at the static-decode/convergent-rule tension boundary
r13 correctly closed the eight r12-named forms, kept the `bash -c "$(…)"` grouping intact, is
precision-clean on the merge-train + reads-with-$-args + retrain set, and the convergent direction is
right. But the convergence claim again treats "the named forms denied" as "static closure reached,"
which is false: the convergent sentinel and the guard's OWN static command-substitution decoder are in
tension. The decoder (necessary to catch `$(echo git … reset)` as a verb) resolves `$(echo "reset
--hard")` to a clean literal BEFORE the sentinel runs, and the first-word string-binding model then
drops the trailing `--hard`. The durable fix is to make the EFFECTIVE-argv model faithful to bash:
when a static command substitution produces a value assigned to a variable (`P=$(echo …)`) or fed to
`set --`/`read -ra`, preserve the FULL value as a single binding and word-split it at the `$P`/`"$@"`/
`"${P[@]}"` USE site (so `--hard`/`-fdx`/an injected `-C <canon>` survives) — OR, minimally, treat a
binding whose RHS is a (decoded) MULTI-WORD command-substitution as unresolved at the use site and
fail closed, exactly as the quoted-string RHS already does (lib.rs:1656 value_ok). Equivalently: do not
let `static_command_output` splice multi-word text into a position that `collect_same_line_bindings`
will then bind by first-word-only.

### Resolved since r12 (verified)
- All EIGHT r12 reproductions DENY through the production binary: literal array assignment, quoted
  multi-word string-var split, variable array index `${P[$n]}`, eval-hidden `set --`, in-function-body
  `set --`/`local`, bare unbound `git "$@"`/`"${@}"`.
- The convergent rule is precision-clean on the high-FP-risk surface: reads with `"$@"`/`"${P[@]}"`/
  `"$BRANCH"` args, `IFS=, read`, `grep -r git /path`, `bash -c "$(…)"` grouping, benign subst all
  preserved (0 FP across 42 legit + 10 retrain).

### Positive observations
- r12-named fixes are general (not per-fixture), correct, and precision-clean. 30 unit + liveness gate
  green (gate Pass 1, unittest Pass 1), SSH-signed (Good ED25519), faces-last settle holds (2 faces,
  lib.rs-only fix, no Cargo.lock churn), no key laundering, single guard dep (serde_json), no new CVE
  surface.

---

## Residual risk (single most likely production failure if merged as-is)
An agent (or an injected instruction via the lethal-trifecta surface) emits
`bash -c 'P=$(echo "reset --hard"); git -C <canonical> $P'` or
`bash -c 'P=$(echo "-C <canonical> reset --hard"); git $P'` — both statically-resolvable, NO opaque
`$(prog)`, NO `read`-from-stdin (the guard decodes `echo` itself) — and it silently ALLOWs,
re-contaminating the canonical checkout and reproducing FRIC-022/FRIC-1781062867 while all 30 tests and
the liveness gate stay green. The leak survives because r13's convergent sentinel only fires on a
LITERAL `$`/backtick in the subcommand, but the guard's own static-substitution decoder resolves that
to a clean literal first and then drops the trailing mutating arg.

Secondary (carried, unchanged): the hook shim fails OPEN if the Rust binary is unbuilt (main.rs maps
any error → SUCCESS); ensure CI/branch-protection builds it (structural enforcement, not hook
reliance). Genuinely runtime-unknowable family (pre-existing, out of scope, faithful): PATH/symlink
alias `g`→git binary; interactive-only `alias`; `read` from stdin/pipe; `$(prog)` opaque stdout.

## Required to clear
1. Close F1-R13: make the EFFECTIVE-argv model faithful to bash for STATIC command substitutions
   assigned to bindings/`set --`/`read -ra`. Preserve the full decoded value as ONE binding value and
   word-split it at the USE site (`$P`/`"$@"`/`"${P[@]}"`), so `--hard`/`-fdx`/an injected `-C <canon>`
   survives — instead of splicing the multi-word text in and binding only the first word. Minimal
   alternative: a binding whose RHS is a decoded MULTI-WORD `$(echo|printf …)`/backtick MUST be treated
   as unresolved at the use site (fail closed), matching the existing quoted-string RHS handling.
2. Add a DENY fixture for EVERY confirmed F1-R13 reproduction: `P=$(echo "reset --hard"); git -C <c>
   $P`; `P=$(echo "-C <c> reset --hard"); git $P` (+ unquoted/printf/backtick/nested);
   `P=$(echo "clean -fdx"); git -C <c> $P`; `A=$(echo "reset --hard"); B=$A; git -C <c> $B`;
   `set -- $(echo "-C <c> reset --hard"); git "$@"`; `read -ra P <<< "$(echo "-C <c> reset --hard")";
   git "${P[@]}"`. Current fixtures structurally cannot catch them.
3. Re-evaluate F2-R13: decide whether function-`"$@"`/array-bound NON-canonical worktree mutations
   should ALLOW (matching r12's certified behavior) or stay fail-closed; if the latter, document the
   intentional usability regression so it is not re-flagged.
4. Re-run 30 + liveness + new fixtures; re-drive the TASK-2 sweep through the real binary with a FOREIGN
   session-cwd; confirm zero false positives on the merge-train + retrain.sh + `IFS=, read` +
   `grep -r git /path` + quoted-arg-read set.
5. Re-state the convergence claim honestly only after auditing the STATIC-SUBSTITUTION-INTO-BINDING
   axis (echo/printf/backtick output assigned to a var/`set --`/`read -ra`, then word-split at use) in
   addition to the literal-binding axes already covered — and confirm the sole residual is genuinely
   runtime-unknowable `$(prog)`/read-from-stdin.

## Security Checklist
- [x] No hardcoded secrets (lib.rs is pure policy logic; only dep is serde_json; no new deps)
- [~] All inputs validated — r12 axes (literal array, quoted string-var, var-index, eval-hidden,
      in-body set--/local, bare `$@`) now CLOSED; but a STATIC command-substitution
      (`$(echo …)`/`$(printf …)`/backtick) assigned to a binding/`set --`/`read -ra` is decoded to a
      clean literal and flattened to a first-word binding (dropping `--hard`/`-fdx`) →
      statically-resolvable real-mutation ALLOW (F1-R13)
- [~] Injection prevention — r12 classes CLOSED; F1-R13 OPEN (static-subst-into-binding canonical
      mutation). Worktree over-deny (F2-R13) is a fail-closed regression, not an injection hole.
- [x] Authorization/policy enforced for modeled forms; 42/44 legit + 10 retrain ALLOW with 0 FP;
      `grep -r git /path`, `IFS=, read`, quoted-arg reads, `bash -c "$(…)"` grouping all preserved
      (the 2 worktree over-denies are F2-R13, tracked separately)
- [x] Dependencies audited — single dep serde_json; no new guard deps; no CVE surface in delta
- [x] Tests + liveness gate green (30 unit + gate Pass 1 / unittest Pass 1); SSH-signed (Good ED25519);
      faces-last (2 faces, lib.rs-only fix, no Cargo.lock churn); no key laundering
