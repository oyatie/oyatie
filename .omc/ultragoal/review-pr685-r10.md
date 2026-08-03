# Security Review — PR #685 ROUND 10 (G011 canonical-checkout guard) — CONVERGENCE CHECK

- Repo: jason931225/oyatie  Branch: agent/g011-checkout-guard  Head: 346526755  Base: dev
- r10 delta vs r9 (c6023aac7..346526755): two commits.
  - cf90f7c29 `fix(checkout-guard): ANSI-C decode + line-continuation join + positional-param sigil`
    — lib.rs ONLY (+145/-20, no Cargo.lock churn). Three model extensions:
    (1) `has_unresolved_expansion` (lib.rs:1297) now flags `$` before ANY non-space follower
    (catches `$@ $* $# $! $? $$ $- $N` positional/special params, the r9 F1 gap);
    (2) full ANSI-C decoder `decode_ansi_c` (lib.rs:1518) — `\xHH`/`\NNN` octal/`\uHHHH`/`\U`/letter
    escapes (r9 F2); (3) line-continuation join — `\`+newline dropped in `expand_with_bindings`
    (lib.rs:1364-1378, r9 F3). Subcommand stays a transformation-sigil DENYLIST
    (`is_blocked_operation || has_unresolved_expansion(subcommand)`, lib.rs:378-379) so a phantom
    `git` match's literal path arg (`grep -r git /path`) stays ALLOW. Plus 9 new DENY fixtures
    (lib.rs:2316-2324).
  - 346526755 `chore: settle generated cloud-ci faces` — 2 *.generated.json faces ONLY.
- Reviewer: fresh-context security-reviewer (Claude Opus), Torvalds/attacker lens,
  /using-superpowers + /using-agent-skills + security-and-hardening + /ultraqa, all FOREGROUND,
  built from /Users/jasonlee/oyatie-worktrees/g011-checkout-guard.

## VERDICT: BLOCK

**Risk Level: HIGH.** The r9 fix is correct and complete *for the three r9-named gaps*: all three
r9 BLOCK findings (positional `$@` in verb slot, ANSI-C hex/octal numeric escapes, backslash
line-continuation) now DENY (rc=2) through the production binary, with ZERO false positives —
including the critical `grep -r git /path` (subcommand denylist holds) and all `IFS=, read` reads.
That part of the convergence claim is sound.

BUT the r10 convergence claim — "No statically-resolvable bypass remains; the sole residual is the
runtime-unknowable `$(prog)` class" — is **FALSE**. r10 closed the r9 forms but left open a
**NEW statically-resolvable real-mutation bypass class**: the canonical `-C` target and the
mutating verb supplied through POSITIONAL/FUNCTION-ARG indirection that splits them across two
lexical scopes the guard analyzes independently. I constructed SEVEN distinct forms, each verified
DIRTY→CLEAN end-to-end through the PRODUCTION binary with an argv-capturing fakegit, and NONE uses
a runtime-unknowable `$(prog)` — every word is a literal constant in the one-liner:

1. `set -- -C <canon> reset --hard; git $@`        (whole argv in `$@` after bare `git`)
2. `set -- <canon> reset --hard; git -C $@`        (target+verb swallowed by `$@` in `-C` slot)
3. `set -- <canon> reset --hard; eval git -C $@`   (eval variant of #2)
4. `g(){ git "$@"; }; g -C <canon> reset --hard`   (function-arg indirection — strongest)
5. `g(){ git $@; }; g -C <canon> reset --hard`     (unquoted `$@` variant)
6. `g(){ command git "$@"; }; g -C <canon> reset --hard` (function + command prefix)
7. `h(){ g(){ git "$@"; }; g "$@"; }; h -C <canon> reset --hard` (nested function)

All resolve fully from the command string at parse time (literal `set --` words; literal function
body + literal call args). They are NOT the founder-accepted runtime-unknowable `$(prog)` residual.
Per the strict VERDICT RULE — "BLOCK on a statically-resolvable real-mutation bypass" — this blocks.

**NEW MECHANISM CLASS or VARIANT?** It is a VARIANT of the closed r9 F1 positional-parameter class
at its ROOT (`$@`/`$N` carrying words the guard can't bind), but it is a DISTINCT, uncovered
*surface*: r9/r10 only modeled `$@` as a VERB token (subcommand-position sigil → fail closed). r10
never modeled (a) `$@` swallowing the `-C <path>` argument so NO subcommand token remains and
`parse_git_invocation` returns None → ALLOW, nor (b) a function call binding `-C <canon>`/verb to
`$@`/`$N` inside a body the guard scans WITHOUT the call args. The guard models neither `set --`
positional binding nor function-call argument binding, so the verb and the canonical `-C` target
never appear co-located in any single scan. This is the same r1→r9 meta-pattern at one more layer.

---

## Harness (commands + exact exit codes)

Built the real binary: `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out
/tmp/r10-guard-bin` → BUILD SUCCEEDED (Mach-O arm64). Driven via JSON hook payload
`{"tool_input":{"command":"…"}}` on stdin (the real enforcement surface, main.rs run()),
`OYA_CANONICAL_CHECKOUT=<canon>`, `env -i` neutral env. rc=2 ⇒ DENY, rc=0 ⇒ ALLOW.
Calibration verified: `git -C <canon> {reset --hard|switch|restore .|checkout -- f|clean -fdx|
stash pop}` → 2; `{status|fetch|log}` → 0.

**Adjudication discipline (real-mutation only counts):** per candidate, (1) build a fresh DIRTY
real git repo (` M tracked.txt` + `?? untracked.txt` + extra branch); (2) record guard rc via the
real binary; (3) run the EXACT command in a clean `bash -c` subshell with REAL git, canon path
substituted to the live repo; (4) diff `git status --porcelain` before/after. A FINDING requires
`guard_rc=0 AND mutated=yes`. Adjudicator self-test confirmed sound: real `reset --hard`/`clean
-fdx` mutate the dirty repo, `status`/`log` do not. An argv-capture fakegit on PATH confirmed the
exact words bash hands git in each leak: `[-C] [<canon>] [reset] [--hard]`.

### TASK 1 — full accumulated corpus (r1..r9 named + ALL reproductions incl 3 r9 BLOCK findings) — PASS
52-command corpus (r1/r2 wrappers flock/runuser/cpulimit/timeout/nice/nohup/xargs/systemd-run/
firejail/eatmydata; r3 sh -c/GIT_DIR/GIT_WORK_TREE context; r4 nested escaped backtick; r5
`$(echo git)`/backtick/`eval $()`/`bash -c "$()"`/ANSI-C/$VAR/`${x:-}`/`"$(printf)"`; r6
`${x:=}/${x:+}/${x/a/e}/${x//a/e}`+nested fixpoint; r7 brace `{reset,}`/glob/`rese?`/`rese[t]`; r8
all IFS forms incl `IFS=-`/`${y}`-wrapped/`bash -c` wrapper/path-side; r9 ALL THREE BLOCK findings
— `set -- reset --hard; git -C <canon> $@` (+ clean/restore/fn-arg/bash -c/sh -c variants),
ANSI-C hex VERB `$'\x72…'`, ANSI-C hex FLAG `$'\x2d\x2dhard'`, ANSI-C octal `$'\162…'`,
line-continuation `re\<NL>set`):
- **PASS=52  FAIL=0.** ALL DENY (rc=2). Every r9 BLOCK finding (F1/F2/F3-R9) is RESOLVED through
  the production binary. The r9 function form `f(){ git -C <canon> $@; }; f reset --hard` (canon-C
  in body) also DENYs.

### TASK 2 — CONVERGENCE TEST — ONE NEW STATICALLY-RESOLVABLE REAL-MUTATION BYPASS CLASS (BLOCK)
Enumerated every remaining word-synthesis mechanism the prompt named; adjudicated each vs REAL bash
via the production binary. Mechanisms that DENY (faithful — caught) or ALLOW-without-mutation
(faithful — no leak):
```
  arithmetic $((1)) / $((0x72))            rc=0 mut=no   (no verb string produced — faithful)
  process-subst <(echo reset)              rc=0 mut=no   (fd path, not a verb — faithful)
  process-subst cat <(git … reset --hard)  rc=2          (nested git seen — DENY)
  array ${a[@]} / ${a[0]}                   rc=2          ($-sigil + brace — DENY)
  indirect ${!ref}                          rc=2          ($-sigil — DENY)
  locale $"reset"                           rc=2          ($-sigil — DENY)
  quote-removal re""set / re''set           rc=2          (tokenizer strips quotes → reset — DENY)
  backslash-unquoted \reset / g\it          rc=2          (DENY)
  ${v//XX/}→reset                           rc=2          ($-sigil — DENY)
  command/exec git … reset --hard           rc=2          (DENY)
  brace-range re{s..s}et                     rc=2          (brace metachar — DENY)
  extglob re@(set)                          rc=0 mut=no   (bash needs shopt; no mutation — faithful)
  ANSI-C octal $'\0162…'                    rc=0 mut=no   (bash decodes \0162→0x0E+'2', NOT 'r' —
                                                          guard ALLOW is FAITHFUL, no mutation)
  $# $1 $2 $- $! $? $0 $_ $$ after set --   rc=2 mut=no   (sigil-DENIED anyway; none weaponizable)
  $'reset\c'                                rc=0 mut=no   (\c suppresses → no clean verb — faithful)
```
CONFIRMED LEAKS (guard_rc=0 ALLOW; REAL canonical mutation; via production binary):
```
  rc=0 mut=yes  set -- -C <canon> reset --hard; git $@              argv=[-C <canon> reset --hard]
  rc=0 mut=yes  set -- <canon> reset --hard;    git -C $@          argv=[-C <canon> reset --hard]
  rc=0 mut=yes  set -- <canon> reset --hard;    eval git -C $@     eval variant
  rc=0 mut=yes  g(){ git "$@"; };        g -C <canon> reset --hard  function-arg indirection
  rc=0 mut=yes  g(){ git $@; };          g -C <canon> reset --hard  unquoted $@
  rc=0 mut=yes  g(){ command git "$@"; };g -C <canon> reset --hard  fn + command prefix
  rc=0 mut=yes  h(){ g(){ git "$@";}; g "$@";}; h -C <canon> reset --hard  nested fn
  rc=0 mut=yes  g(){ git "$@"; };        g -C <canon> clean -fdx    (untracked DESTROYED)
```
Argv-capture fakegit confirms bash hands git `[-C] [<canon>] [reset] [--hard]` in each case.

End-to-end proof (production binary), DIRTY→guard ALLOW→CLEAN:
```
  BEFORE:  M tracked.txt | ?? untracked.txt
  GUARD rc=0 (ALLOW)   cmd: g(){ git "$@"; }; g -C <canon> reset --hard
  AFTER:  ?? untracked.txt          (the ` M` is gone — reset --hard executed)
  ---
  BEFORE:  M tracked.txt | ?? untracked.txt
  GUARD rc=0 (ALLOW)   cmd: set -- <canon> reset --hard; git -C $@
  AFTER:  ?? untracked.txt          (` M` gone — reset --hard executed)
  ---
  BEFORE:  M tracked.txt | ?? untracked.txt
  GUARD rc=0 (ALLOW)   cmd: g(){ git "$@"; }; g -C <canon> clean -fdx
  AFTER:   M tracked.txt            (untracked.txt DESTROYED — clean -fdx executed)
```

Leak-boundary map (what DENYs vs LEAKs — isolates the gap):
```
  set -- <canon>; git -C $1 reset --hard            rc=2  (literal verb co-located w/ dynamic -C)
  set -- <canon> reset; git -C $1 $2 --hard         rc=2  (path-side $1 dynamic → Unknown → DENY)
  g(){ git -C "$1" reset --hard; }; g <canon>       rc=2  (literal verb + $1 target in same body)
  g(){ git -C "$1" "$2" --hard; }; g <canon> reset  rc=2  (body has -C $1; path dynamic → DENY)
  f(){ git -C <canon> $@; }; f reset --hard         rc=2  (r9 form: canon-C + $@ verb in body)
  --- vs ---
  g(){ git "$@"; }; g -C <canon> reset --hard       rc=0  LEAK (no -C in body; verb+target at call)
  set -- <canon> reset --hard; git -C $@            rc=0  LEAK ($@ swallows -C arg AND verb)
```
The discriminator: a LEAK occurs precisely when the VERB is carried through positional/function-arg
indirection AND the guard never sees a literal mutating verb co-located with a literal canonical
`-C`. Whenever the body/scan contains `-C <canon>` plus the verb (literal or as a sigil), it DENYs.

Root cause (lib.rs):
- The guard models neither `set -- <words>` positional binding nor function-call argument binding.
- For `git -C $@`: `parse_git_invocation` (lib.rs:771) consumes `-C` then takes `$@` as the path
  arg (→ `resolve_target_path` → `target_path_is_dynamic("$@")` true → Unknown → would-block), but
  with `$@` consuming the rest there is NO subcommand token, so the `while` loop falls through to
  `return None` (lib.rs:879) → the whole git word is skipped → ALLOW.
- For `g(){ git "$@"; }; g …`: the body `git "$@"` scans with target = session_cwd (no `-C canon`)
  → `blocked_target=false` → ALLOW; the call site `g` is not `git` and has no sigil
  (`has_unresolved_expansion("g")`=false, lib.rs:349) → not evaluated as git → ALLOW. The
  `-C <canon> reset --hard` words live only at the call site, never bound into the body scan.

RESIDUAL genuinely runtime-unknowable (`$(prog)`/`$(cat file)` opaque stdout) remains accepted —
BUT it is NOT the sole residual: the 7 forms above are parse-time-determinable and BLOCK.
Carried/observed (NOT counted, same runtime-unknowable family, pre-existing, unchanged): a
PATH/symlink/alias `g`→git binary, which needs filesystem resolution at parse time.

### TASK 3 — FALSE-POSITIVE SWEEP (must ALLOW) — PASS, ZERO false positives
- 34/34 legit ALLOW: full merge-train on canonical (`merge/pull --ff-only`, `commit`, `push`,
  `fetch --all --prune`, `status`, `log`, `diff`, `show`, `rev-parse`), reads with metachar/var/
  ANSI-C/positional ARGS (`log $REF`, `log {origin/dev,HEAD}`, `show *.rs`, `diff HEAD~{1,2}`,
  `diff ${BASE:-HEAD}`, `show $(git rev-parse HEAD)`, `log ~/notes`, `log $@`, `log $'\x48EAD'`,
  `show $1`), worktree mutations on NON-canonical (`reset --hard`, `clean -fdx`, `worktree
  list/add`), `$(date)`, `$EDITOR`, `sudo sh -c 'echo'`.
- **CRITICAL `grep -r git /path` stays ALLOW** (3 forms incl `grep -rn git /etc`, `rg git /tmp`):
  the subcommand transformation-sigil DENYLIST does NOT flag a literal path arg of a phantom git
  match. The r10 commit's central precision claim is verified true.
- **`IFS=, read` not over-denied** (4 forms incl `IFS=, read -ra parts …; git -C <canon> log`,
  `while IFS=, read …`, `IFS=, read -r a b; git -C <canon> log $a`) — all ALLOW.
- /tmp/retrain.sh (present) — 10/10 git lines ALLOW (ls-remote/fetch/clean/checkout/add/commit/
  push/rev-parse against the live repo + worktree; none target the calibrated canonical).
- **TASK3 RESULT: ALLOW-PASS=34 + 10 retrain = 0 FALSE POSITIVES.**

### TASK 4 — mechanics — all green
- `//tools/oya-checkout-guard-app:oya-checkout-guard-app-unittest` → **30 passed; 0 failed**
  (9 new r9 DENY fixtures present and pass). NOTE: NO fixture exercises the TASK-2 LEAK forms
  (target-via-`$@`-path / function-arg split) — the recurring r1→r9 reason the leak survives
  30-green.
- `//cloud/cloud-ci/gates/oya-cloud-ci-enforcement-liveness-app:{…-unittest,…-gate}` →
  unittest Pass 1; gate **6/6 tests pass** (`enforcement_liveness_face_reports_current_tree_green`).
- SSH sigs: cf90f7c29 + 346526755 both carry BEGIN SSH SIGNATURE. `%G?`=U is environmental
  (allowed-signers not provisioned in fresh context), same as r1-r9.
- Faces-last settle: 346526755 touches ONLY 2 *.generated.json faces (accounting-registry,
  scm-facts). Fix commit cf90f7c29 = lib.rs ONLY (no Cargo.lock churn). Tracked faces clean at
  HEAD. Holds.
- Baseline key-diff both ways (merge-base d705932d4 .. HEAD over all 4 changed generated faces):
  **0 anti-pattern keys** (forbidden_foundry|unjustified|unowned|unreachable) added OR removed.
  No laundering.

---

## Findings (file:line, severity, confidence)

### [HIGH, confidence HIGH] F1-R10 — verb carried via positional/function-arg indirection splits target+verb across scopes the guard analyzes independently (static, no $(prog))
- Location: lib.rs:771-880 `parse_git_invocation` (consumes `-C $@` as a dynamic path then returns
  None when `$@` leaves no subcommand token — lib.rs:879); lib.rs:349 (`g` call site has no sigil →
  not evaluated as git); the guard models neither `set -- <words>` positional binding nor
  function-call argument binding anywhere (`grep "set --"` → only comments/fixtures).
- Confirmed (rc=0; REAL bash MUTATES; production binary, DIRTY→CLEAN): seven forms —
  `set -- -C <canon> reset --hard; git $@`; `set -- <canon> reset --hard; git -C $@`;
  `…; eval git -C $@`; `g(){ git "$@"; }; g -C <canon> reset --hard` (+ unquoted, + `command git`,
  + nested `h`); and the `clean -fdx` analogue (untracked destroyed).
- Exploitability: local, unauthenticated, single short one-liner, no length cap, shell-agnostic
  (`set --`/`$@`/functions work in bash and sh). Trivially emitted by an agent or an injected
  instruction (the lethal-trifecta surface CLAUDE.md warns of).
- Blast radius: silent canonical `reset --hard` / `clean -fdx` / `restore .`, reproducing
  FRIC-022/FRIC-1781062867 while all 30 unit tests + the liveness gate stay green.
- STATICALLY RESOLVABLE: `set -- -C <canon> reset --hard` and the function body+call args are
  literal constants, fully visible at parse time. NOT the runtime-unknowable `$(prog)` residual —
  the BLOCK class.
- Classification: VARIANT of closed r9 F1 (positional `$@`) at the root, but a DISTINCT uncovered
  surface — r9/r10 modeled `$@` only as a VERB token; they never modeled `$@` swallowing the `-C`
  argument, nor function-call arg binding.

### Note — the r1→r9→r10 meta-pattern, now at positional/function-arg BINDING
r10 correctly closed the three r9-named gaps (positional `$@` in verb slot, ANSI-C numeric escapes,
line-continuation), each generally and precision-clean (zero FP, `grep -r git /path` preserved).
But the convergence claim again treats "the named forms modeled" as "static closure reached," which
is false: r10 made `$@` fail-closed as a VERB but never modeled the BINDINGS that feed `$@`/`$N`
(`set --`, function call args), so the complementary split — verb in the indirection, `-C <canon>`
at the call/elsewhere — leaks. The durable fix is a model change: track `set -- <words>` and
function-call argument binding, expand `$@`/`$*`/`$N` against them, and fail-closed when a git
invocation's effective argv (after binding) targets canonical with a mutating verb — OR, minimally,
fail-closed whenever a `git -C <dynamic>` consumes the entire remainder leaving no subcommand
(the `parse_git_invocation`→None path at lib.rs:879 currently ALLOWs).

### Resolved since r9 (verified)
- F1-R9 (positional `$@` in VERB slot): CLOSED by extended `$`-sigil in `has_unresolved_expansion`
  (lib.rs:1297). `set -- reset --hard; git -C <canon> $@` and the fn-arg/bash -c/sh -c variants all
  DENY. (The COMPLEMENTARY split — `$@` in the `-C`/whole-argv slot, or fn-arg target binding — is
  F1-R10, still open.)
- F2-R9 (ANSI-C hex/octal/unicode escapes): CLOSED by `decode_ansi_c` (lib.rs:1518). `$'\x72…'`,
  `$'\145…'`, mixed `$'\x72\145…'` all DENY. (`$'\0162'` ALLOWs but bash also does NOT produce a
  verb there — faithful, no mutation.)
- F3-R9 (line-continuation): CLOSED by `\`+newline drop in `expand_with_bindings` (lib.rs:1364).
  `re\<NL>set` DENYs.

### Positive observations
- r9-named fixes are general (not per-fixture), correct, and precision-clean — the right direction
  for their axes; `grep -r git /path` and `IFS=, read` are preserved (0 FP across 34+10 commands).
- 30 unit + liveness gate green (6/6), SSH-signed, faces-last settle holds (2 faces, lib.rs-only
  fix, no Cargo.lock churn), no key laundering, single guard dep (serde_json), no new CVE surface.

---

## Residual risk (single most likely production failure if merged as-is)
An agent (or an injected instruction via the lethal-trifecta surface) emits
`bash -c 'g(){ git "$@"; }; g -C <canonical> reset --hard'` or
`set -- <canonical> reset --hard; git -C $@` — statically-resolvable, NO IFS, NO covered
metacharacter, NO runtime-unknowable `$(prog)` — and it silently ALLOWs, re-contaminating the
canonical checkout and reproducing FRIC-022/FRIC-1781062867 while all 30 tests and the liveness gate
stay green. Because the verb and the canonical `-C` target are split across scopes (positional
binding or function args) the guard analyzes independently, neither scan ever sees them co-located.
The fix must model bash positional/function-arg BINDING (not just flag `$@` as a verb), or at
minimum fail-closed when a `git -C <dynamic>` invocation leaves no resolvable subcommand.

Secondary (carried, unchanged): the hook shim fails OPEN if the Rust binary is unbuilt (main.rs
maps any error → SUCCESS); ensure CI/branch-protection builds it (structural enforcement, not hook
reliance). Carried: bare `git stash` (save) and PATH-relative alias/symlink `g`→git binary are not
modeled — the latter arguably needs filesystem resolution (runtime-unknowable family); pre-existing,
out of scope, noted not blocking on their own.

## Required to clear
1. Close F1-R10: model `set -- <words>` positional binding AND function-call argument binding;
   expand `$@`/`$*`/`$N` against them so a synthesized `git -C <canon> <mutating-verb>` is caught
   regardless of which scope supplies the target vs the verb. At minimum: fail-closed in
   `parse_git_invocation` when `-C` (or `--git-dir`/`--work-tree`) consumes a DYNAMIC path token
   (`$@`/`$*`/`$N`/`$`-anything) and no literal subcommand remains — i.e. do not `return None →
   ALLOW` on the dynamic-`-C`-swallows-rest path (lib.rs:879); treat a git word whose target slot
   is a positional/special expansion as fail-closed.
2. Add a DENY fixture for EVERY confirmed F1-R10 reproduction (`set -- -C <canon> reset --hard; git
   $@`; `set -- <canon> reset --hard; git -C $@`; `…eval git -C $@`; `g(){ git "$@"; }; g -C <canon>
   reset --hard`; unquoted `$@`; `command git` body; nested `h`; the `clean -fdx` analogue). Current
   fixtures structurally cannot catch them — the recurring r1→r9 failure mode.
3. Re-run 30 + liveness + new fixtures; re-drive the TASK-2 sweep through the real binary; confirm
   zero false positives on the merge-train + retrain.sh + `IFS=, read` + `grep -r git /path` set.
4. Re-state the convergence claim honestly only after auditing ALL binding axes (positional `set
   --`, function-call args, `local`/`declare` scoped vars) in addition to the word-synthesis axes —
   and confirm the sole residual is runtime-unknowable `$(prog)`/opaque-stdout.

## Security Checklist
- [x] No hardcoded secrets (lib.rs is pure policy logic; only dep is serde_json; no new deps)
- [~] All inputs validated — r9 axes (positional-as-verb, ANSI-C numeric, line-continuation) now
      sound; positional/function-arg BINDING that splits target+verb across scopes bypasses static
      analysis → statically-resolvable real-mutation ALLOW (F1-R10)
- [~] Injection prevention — r9 classes CLOSED; F1-R10 OPEN (positional/fn-arg-split
      canonical-mutation)
- [x] Authorization/policy enforced for modeled forms; zero false positives across 34 legit + 10
      retrain commands; `grep -r git /path` and `IFS=, read` preserved
- [x] Dependencies audited — single dep serde_json; no new guard deps; no CVE surface in delta
- [x] Tests + liveness gate green (30 unit + gate 6/6 Pass 1/2); SSH-signed; faces-last (2 faces,
      lib.rs-only fix, no Cargo.lock churn); no key laundering
