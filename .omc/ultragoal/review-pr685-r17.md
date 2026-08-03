# Security Review — PR #685 ROUND 17 (G011 canonical-checkout guard) — STRUCTURAL FAIL-CLOSED-ALLOWLIST CHECK

**Scope:** `tools/oya-checkout-guard-app/src/lib.rs` (the PreToolUse enforcement hook) + 2 generated cloud-ci faces.
PR jason931225/oyatie#685, branch `agent/g011-checkout-guard`, head `e579b358e`, base `dev`.
**Risk Level: HIGH.**

- r17 delta vs r16 (`6d92de640..e579b358e`): two commits.
  - `823774b13` `fix(checkout-guard): fail-closed value-model allowlist (review #685 r16)` — **lib.rs ONLY (+109/-25, no Cargo.lock churn).**
    Rewrites the value-producing paths into fail-closed allowlists: `static_command_output(body, xpg_echo)` printf now models ONLY a
    specifier-free format or a single leading `%s`/`%b` then args, rejecting any remaining `%`; `resolve_param` requires a clean identifier
    and models only `${name}`/`${name:-d}`/`${name:=d}`/`${name-d}`; ANSI-C `$'…'` is single-quoted unconditionally (one word); `echo` decodes
    under same-line `shopt -s xpg_echo`; a new `is_value_producer(body)` makes an UNMODELED echo/printf substitution emit a
    `${__unresolved__}` sigil (fail-closed in git position) while a real-command body keeps raw text for the separate git-recursion. Sentinel
    renamed `${unresolved-dynamic-target}` → `${__unresolved__}`. Adds 5 DENY fixtures (the r16 reproductions).
  - `e579b358e` `chore: settle generated cloud-ci faces` — 2 `*.generated.json` faces ONLY.
- Reviewer: fresh-context security-reviewer (Claude Opus), Torvalds/attacker lens, /using-superpowers + /using-agent-skills + /ultraqa,
  FOREGROUND, built from `/Users/jasonlee/oyatie-worktrees/g011-checkout-guard`. FOREIGN session-cwd (`/tmp`); mutation arrives via
  `-C`/binding indirection; every candidate confirmed against REAL bash with an argv-capturing fakegit (dirty→clean) that EXECs real git so
  canonical mutation is GENUINE. Adjudicator self-test sound; leak-detection proven.

## VERDICT: BLOCK

The r17 structural refactor genuinely closed the ENTIRE r16 leak class it targeted — all 12 r16 reproductions
(`printf "%-10s--hard"`, `printf "%s%s"`, `printf -- "%s\n" reset --hard`, `printf "%c%s"`, `printf "reset%0.s"`, `${P/ZZ/ --}`,
`${P/ZZ/ -}`, `$'reset'$'\t--hard'`, `$'clean'$'\t-fdx'`, `xpg_echo` plain-echo, and the `clean -fdx` width-pad variants) now **DENY (rc=2)**
through the production binary. The sigil mechanism is structurally sound: an UNMODELED value-producer fails closed in EVERY git-feeding
position (subcommand, `-C` value, command-name, binding) and does NOT over-deny in benign read positions (`log --format=$(printf …)`,
`log -n $(printf …)`). Mechanics are green (31 unit + liveness Pass 1, Good ED25519, faces-last holds, 0 anti-pattern key churn) and the FP
sweep is **55/55 ALLOW, 0 false positives**.

BUT the convergence claim — "the value model is now fail-closed by construction; the sole residual is the runtime-unknowable
`$(cat)`/`$(curl)`/read-from-stdin" — is **FALSE**. Two of the *modeled* value paths still return a **wrong-but-non-None** value instead of
falling through to the sigil, so they never fail closed. I confirmed **10 statically-resolvable real-mutation bypasses across TWO mechanisms**
that use NO runtime-unknowable input. Per the strict VERDICT RULE ("BLOCK on a statically-resolvable real-mutation bypass"), this blocks.

The two mechanisms and their single most-damning leaks (every byte statically determinable — NO `$(cat)`, NO `$(curl)`, NO stdin), exact
production-binary exit codes, argv captured from the EXEC'd fakegit:
```
  git -C <c> $(printf "%s --hard" reset)   -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> reset --hard]  (Mechanism A: printf %s + trailing literal)
  V=reset; git -C <c> ${V:-x} --hard       -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> reset --hard]  (Mechanism B: ${name:-default} default-eating)
  V=reset; git -C <c> ${V-x} --hard        -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> reset --hard]
  V=reset; git -C <c> ${V:=x} --hard       -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> reset --hard]
  V=clean; git -C <c> ${V:-x} -fdx         -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> clean -fdx]
  W=--hard; git -C <c> reset ${W:-x}       -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> reset --hard]   (flag side)
  git -C <c> $(printf "%b --hard" reset)   -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> reset --hard]
  git -C <c> $(printf "%s -fdx" clean)     -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> clean -fdx]
  P=$(printf "%s --hard" reset); git -C <c> $P    -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> reset --hard]  (binding form)
  git -C <c> $(printf "%s --hard HEAD" reset)     -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> reset --hard HEAD]
```
All MUTATE canonical end-to-end through the production binary from a FOREIGN cwd with an argv-capturing fakegit (DIRTY ` M tracked.txt`/`?? untracked.txt` → CLEAN `?? untracked.txt`, the real `reset --hard` reverting the tracked modification).

---

## Harness (commands + exact exit codes)

Built the real binary: `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out /tmp/r17-guard-bin` → BUILD SUCCEEDED
(Mach-O arm64). Driven via JSON hook payload `{"tool_input":{"command":"…"}}` on stdin (the real enforcement surface),
`env -i OYA_CANONICAL_CHECKOUT=<canon> PATH=… <bin>`, binary cwd=/tmp (canonical != cwd, FOREIGN). rc=2 ⇒ DENY, rc=0 ⇒ ALLOW. main.rs maps
any guard ERROR → SUCCESS (fail-open), confirmed; secrets scan clean; single dep `serde_json` (workspace), unchanged.

**Adjudication discipline (real-mutation only counts):** per candidate (1) fresh DIRTY real git repo (` M tracked.txt` + `?? untracked.txt`);
(2) guard rc via the real binary with the canonical path injected as a LITERAL absolute path (placeholders `@CANON@`/`@WT@`/`@OTHER@` expanded
identically for guard payload AND bash exec); (3) the EXACT command in `bash -c` from FOREIGN cwd /tmp with an argv-capturing fakegit that
EXECs real git, so `-C <canon> reset --hard` genuinely mutates; (4) diff `git status --porcelain` before/after. A LEAK requires
`guard_rc=0 AND mutated=yes`. Adjudicator self-test sound: `git -C <canon> {reset --hard|clean -fdx|restore .}` → guard rc=2 + argv proves
mutation; `status`/`log` → rc=0 clean; `git -C <wt|other> reset --hard` → guard rc=0 + canonical untouched (mutated=no); fakegit EXEC of real
git verified. Leak-detection proven on the runtime-unknowable residual family (`$(cat)`/`$(curl)`/`read<file` all fail-closed rc=2).

### TASK 1 — full accumulated corpus (r1..r16 named + ALL reproductions incl 12 r16 cases) — PASS for the modeled corpus
70-form real-mutation corpus (r1 basics; r2 wrappers/env; r3 sh -c/GIT_DIR/GIT_WORK_TREE; r4/r5 subst/eval; r6 param-subst; r7 brace/r8 IFS;
r9 ANSI-C; r10 set--/function; r11 read-ra here-string; r12 argv-position binding; r13 static-subst-into-binding; r14 `+=`/multi-word;
r15 escape-decode; **r16 ALL twelve reproductions** — printf width/multi-spec/repeating, `${P/pat/repl}`, concatenated `$'…'`, xpg_echo):
- **66/70 DENY (rc=2).** The entire r15 escape-decode class AND the entire r16 value-synthesis class (all 12 r16 cases) **DENY**.
- 2 NON-DENY are out-of-model-scope, NOT verdict basis: `git -C <c> reset --merge` and `reset --keep HEAD~1` (the guard blocks `reset --hard`
  only; `--merge`/`--keep` are index/merge ops outside the modeled FRIC-022 verb set — same on r16; not a bypass of the model).
- 2 NON-DENY are the verdict-determinative leaks surfaced below (`${V:-x}` default-eating; carried into TASK 2).
- `git -C <c> rm -rf .` ALLOWs and DOES delete tracked files, but `git rm` is NOT in `is_blocked_operation` (lib.rs:2769 models
  checkout/switch/restore/reset --hard/clean -f/rebase/merge/pull/stash pop/branch -f) — a deliberate, r1→r16-consistent scope boundary
  (recoverable via index/HEAD, like permitted `stash` save). Noted as scope, not a verdict-basis bypass.

### TASK 2 — CONVERGENCE TEST — 10 STATICALLY-RESOLVABLE REAL-MUTATION BYPASSES (BLOCK); r16 class CLOSED; sigil sound
Adjudicated every value/synthesis axis the prompt named vs REAL bash (foreign cwd) via the production binary.

CLOSED (caught — the r17 fixes WORK; the r16 class is dead):
```
  git -C <c> $(printf "%-10s--hard" reset)        rc=2   (width directive unmodeled → None → sigil → fail-closed)
  git -C <c> $(printf "%s%s" reset " --hard")     rc=2   (multi-spec → args.contains('%') → None → sigil)
  git -C <c> $(printf -- "%s\n" reset --hard)     rc=2   (repeating format — handler no longer strips one %s; %-recheck → None)
  git -C <c> $(printf "%c%s" r "eset --hard")     rc=2
  git -C <c> $(printf "reset%0.s --hard" x)       rc=2
  P=$(printf "%-6sclean -fdx" ""); git -C <c> $P  rc=2
  git -C <c> $(printf -- "%s\n" clean -fdx)       rc=2
  P=resetZZhard; git -C <c> ${P/ZZ/ --}           rc=2   (${P/x/y} unmodeled → resolve_param None → literal ${…} → fail-closed verb)
  P=cleanZZfdx;  git -C <c> ${P/ZZ/ -}            rc=2
  P=$'reset'$'\t--hard'; git -C <c> $P            rc=2   (ANSI-C single-quoted always → segments join into one word → splittable)
  P=$'clean'$'\t-fdx';   git -C <c> $P            rc=2
  shopt -s xpg_echo; git -C <c> $(echo "reset\t--hard")  rc=2   (xpg_echo plumbed into static_command_output → decode → split → caught)
  git -C <c> $(printf "%s" "reset --hard")        rc=2   (exact-%s modeled → args=reset --hard → split → caught)
  git -C <c> ${UNSET:-reset} --hard               rc=2   (default IS the verb, name unbound → reset → caught)
  V=reset; git -C <c> ${V:+reset --hard}          rc=2   (${name:+alt} unmodeled → None → literal ${…} → fail-closed)
  P=$(cat f)/$(curl)/read P<f; git -C <c> $P      rc=2   (runtime-unknowable in binding → fail-closed, correct)
  git -c core.pager=cat -C <c> reset --hard       rc=2   (git -c passes through; verb visible)
  git --exec-path=<c> reset --hard                rc=2
  alias g='git -C <c>'; g reset --hard            rc=2   (alias modeled → __shell_alias blocked)
```

CONFIRMED LEAKS (guard_rc=0 ALLOW; REAL canonical mutation, foreign cwd; production binary; ALL words STATICALLY determinable — NO `$(cat)`,
NO `$(curl)`, NO read-from-stdin):
```
  *** Mechanism A — printf "%s"/"%b" with TRAILING LITERAL format text (the allowlist accepts it but MIS-MODELS the word order) ***
  rc=0 mut=yes  git -C <c> $(printf "%s --hard" reset)         argv=[-C <c> reset --hard]   (bash: %s←reset, then literal " --hard")
  rc=0 mut=yes  git -C <c> $(printf "%b --hard" reset)         argv=[-C <c> reset --hard]
  rc=0 mut=yes  git -C <c> $(printf "%s -fdx" clean)           argv=[-C <c> clean -fdx]     (untracked DESTROYED)
  rc=0 mut=yes  git -C <c> $(printf "%s --hard HEAD" reset)    argv=[-C <c> reset --hard HEAD]
  rc=0 mut=yes  P=$(printf "%s --hard" reset); git -C <c> $P   argv=[-C <c> reset --hard]   (binding form)
  *** Mechanism B — ${name:-default}/${name-default}/${name:=default} default-eating (empty-binding first pass eats the default) ***
  rc=0 mut=yes  V=reset; git -C <c> ${V:-x} --hard            argv=[-C <c> reset --hard]
  rc=0 mut=yes  V=reset; git -C <c> ${V-x}  --hard            argv=[-C <c> reset --hard]
  rc=0 mut=yes  V=reset; git -C <c> ${V:=x} --hard            argv=[-C <c> reset --hard]
  rc=0 mut=yes  V=clean; git -C <c> ${V:-x} -fdx             argv=[-C <c> clean -fdx]
  rc=0 mut=yes  W=--hard; git -C <c> reset ${W:-x}           argv=[-C <c> reset --hard]    (flag side)
```

RESIDUAL genuinely runtime-unknowable (verified fail-closed rc=2 in binding form, founder-accepted): `$(cat file)`, `$(curl …)`, `read` from
stdin/pipe. These are NOT the sole residual — Mechanisms A and B above are parse-time-determinable and BLOCK.

Discriminator (isolates each gap precisely — MODELED-CORRECTLY vs MIS-MODELED static shape):
```
  git -C <c> $(printf "%s" "reset --hard")     rc=2 DENY  (format is EXACTLY %s → args modeled correctly → reset --hard → caught)
  git -C <c> $(printf "%s --hard" reset)       rc=0 LEAK  (format is %s + literal " --hard" → handler strips %s, treats "--hard reset"
                                                            as produced text → verb "--hard" not blocked → ALLOW; bash makes "reset --hard")
  V=reset; git -C <c> ${V}    --hard           rc=2 DENY  (${V} → resolve_param("V") → reset → caught)
  V=reset; git -C <c> ${V:-x} --hard           rc=0 LEAK  (first expand pass uses EMPTY bindings → ${V:-x} eats default → "x" BEFORE V=reset
                                                            is collected on pass 2 → verb "x" not blocked → ALLOW; bash makes reset)
```
A LEAK occurs precisely when (A) a printf format is `%s`/`%b` plus trailing LITERAL text the allowlist accepts but emits in the wrong order, or
(B) a `${name:-d}`/`${name-d}`/`${name:=d}` whose name is bound to a verb on the SAME line is resolved to its benign default on the
empty-binding first pass before the binding is collected.

### TASK 3 — FALSE-POSITIVE SWEEP (must ALLOW) — 55/55 legit ALLOW; 0 FP
- **All legit forms ALLOW (rc=0), 0 FP.** Merge-train on canonical (status/status --short/log --oneline -5/log --format="%H %an"/
  log --format='%H%x09%an'/diff/diff --stat/show HEAD/rev-parse HEAD/fetch --all --prune/branch -a/merge --ff-only/pull --ff-only/remote -v/
  tag -l); `stash` save + `stash list`; `add -A`; commit -m with tricky messages (`"x=y"`, `"reset --hard fix"`, `"set IFS=, and reset"`,
  `"msg with \t and =val"`, `"key=val<TAB>tab in msg"`); `M=$(printf "%s\n" "release v1.2"); commit -m "$M"`; `V=$(git rev-parse HEAD)`;
  `git log --format="%H" | head`; multi-word READ vars (`P=log; P+=" --oneline"; git $P`, `P="show HEAD"; git $P`,
  `P=log; P+=" --oneline"; P+=" -5"; git $P`, `P="status --short"; git $P`); the NEW FP-risk surfaces —
  `git -C <c> $(printf "%s" "log --oneline")`, `P=$(printf "%s" "status --short"); git $P`, `echo -e "build\tstep\tdone"`,
  `printf '%s\n' hello world`, `printf '%s' build-ok`, `echo -e "reset done"` — all ALLOW; worktree + other-repo mutations
  (`git -C <wt|other> reset --hard`/`clean -fdx`/`checkout -f`); `bash /tmp/retrain.sh`; `sh -c`/`sudo sh -c`; for/while; `$(date)`; `$EDITOR`;
  `"$@"`; `OPT="--config=core.x=1"; git log $OPT`; non-mutating verbs (`checkout` bare, `reset` bare, `reset --soft`, `clean -n`,
  `clean --dry-run`, `worktree list`).
- **The fail-closed sigil + allowlist tightening introduced ZERO over-denial.** The `${__unresolved__}` sigil fires only for an UNMODELED
  echo/printf value-producer in a git-feeding position; benign read positions (`log --format=$(printf …)`, `log -n $(printf …)`) ALLOW.

### TASK 4 — mechanics — all green
- `//tools/oya-checkout-guard-app:oya-checkout-guard-app-unittest` → **31 passed; 0 failed** (Pass 1). The 5 new r16-reproduction DENY fixtures
  present and pass. NOTE: NO fixture exercises the TASK-2 leak shapes (`printf "%s <literal>"`, `${V:-x}` default-eating) — the recurring
  r1→r16 reason a leak survives a green suite (the suite asserts the modeled shapes, not the mis-modeled ones).
- `//cloud/cloud-ci/gates/oya-cloud-ci-enforcement-liveness-app:…-unittest` → Pass 1 (Fail 0).
- SSH sigs: `823774b13` + `e579b358e` both carry Good ED25519 (SHA256:5grGNUtX9Zgmy1SWne6wF9DR8W1ElUQaF/Z8SYRz8E8).
- Faces-last settle: `e579b358e` touches ONLY 2 `*.generated.json` faces (accounting-registry, scm-facts). Fix commit `823774b13` = lib.rs
  ONLY (+109/-25, no Cargo.lock churn). Holds.
- Baseline key-diff both ways (`6d92de640..e579b358e` over generated faces): **0 anti-pattern keys**
  (forbidden_foundry|unjustified|unowned|unreachable) added OR removed. Face changes are pure provenance (source_inputs_digest,
  last_touch_commit→823774b13, head_time_secs). No laundering.
- Dependencies: single dep `serde_json` (workspace), unchanged; no new CVE surface. No secrets in the lib.rs delta.

---

## Findings (file:line, severity, confidence)

### [HIGH, confidence HIGH] F1-R17 — printf `%s`/`%b` with TRAILING LITERAL format text is accepted by the r17 allowlist but MIS-MODELED: the handler strips the leading `%s` and emits the remaining format-literal + args in the WRONG order, so bash's real `verb flag` argv is never seen → ALLOW of a canonical reset/clean (statically resolvable; NO `$(cat)`/`$(curl)`/stdin).
- Location: `static_command_output` printf arm (lib.rs:2263-2301). After stripping a leading `%s`/`%b`, the code takes the rest of the
  format string (`" --hard"`) PLUS the printf args (`reset`), checks only `args.contains('%')`, and returns that joined text
  (`Some(args.to_owned())` → here the produced text becomes `--hard reset`) through `normalize_split_ws(decode_ansi_c(...))`. bash instead
  substitutes the ARG into the `%s` and APPENDS the literal: `printf "%s --hard" reset` → `reset --hard`. The guard's word order is inverted,
  so the leading word it sees is `--hard` (not a blocked verb) and it ALLOWs.
- Confirmed (rc=0; REAL bash MUTATES from foreign cwd; production binary; DIRTY→CLEAN; argv captured):
  `git -C <c> $(printf "%s --hard" reset)` → `[-C <c> reset --hard]`; `git -C <c> $(printf "%b --hard" reset)`;
  `git -C <c> $(printf "%s -fdx" clean)` → `[-C <c> clean -fdx]`; `git -C <c> $(printf "%s --hard HEAD" reset)`;
  `P=$(printf "%s --hard" reset); git -C <c> $P` (binding form).
- Exploitability: local, unauthenticated, single short one-liner, no length cap; `printf "%s <text>" arg` is standard printf. Trivially
  emitted by an agent or an injected instruction (the lethal-trifecta surface CLAUDE.md warns of).
- Blast radius: silent canonical `reset --hard` / `clean -fdx`, reproducing FRIC-022/FRIC-1781062867 while all 31 unit tests + the liveness
  gate stay green.
- Classification: gap INSIDE the r17 printf allowlist (the allowlist's "single leading %s/%b then args" admits a trailing format literal but
  models it incorrectly). NOT a regression vs r16 — r16 modeled this via a different code path; here it is newly admitted-but-mis-modeled.
  Same r1→r16 META-pattern: the static value model diverges from bash at a value construct the allowlist accepts but reproduces wrongly.
- Required fix: when stripping a leading `%s`/`%b`, REQUIRE the remainder of the FORMAT (before the args) to be empty/whitespace; if the
  format carries trailing literal text after `%s`, either model the true bash word order (arg substituted INTO the format position, literal
  appended) or treat it as unmodeled → emit the `${__unresolved__}` sigil → fail closed. Do not return the args+format-tail as produced text.

### [HIGH, confidence HIGH] F2-R17 — `${name:-default}`/`${name-default}`/`${name:=default}` DEFAULT-EATING: the first (empty-binding) normalization pass resolves the `:-`/`-`/`:=` default to its literal BEFORE the same-line `name=value` binding is collected, so a name bound to a mutating verb is replaced by its benign default at the guard but expands to the verb at runtime → ALLOW.
- Location: `normalize_static_expansions` two-pass loop (lib.rs:1318-1322) + `resolve_param` (lib.rs:2190-2222). Pass 1 calls
  `expand_with_bindings(&inlined, &[] /*EMPTY bindings*/, …)`. For `${V:-x}`, `resolve_param("V:-x", &[])` finds no binding for `V` and
  returns `Some("x")` (the default). The command becomes `V=reset; git -C <c> x --hard` BEFORE pass 2 collects `V=reset`. The verb is now the
  literal default `x`, not `reset`, so no blocked op matches → ALLOW. At runtime bash sees `V` SET and expands `${V:-x}` → `reset` → mutates.
- Confirmed (rc=0; argv `[-C <c> reset --hard]`/`[… clean -fdx]`): `V=reset; git -C <c> ${V:-x} --hard`; `V=reset; git -C <c> ${V-x} --hard`;
  `V=reset; git -C <c> ${V:=x} --hard`; `V=clean; git -C <c> ${V:-x} -fdx`; `W=--hard; git -C <c> reset ${W:-x}` (flag side).
- Note: this same empty-binding-first-pass default resolution is present in r16 too (built and compared `6d92de640`: identical rc=0). It is a
  LATENT pre-existing bug that the r16 review MISSED — r16 only tested `${V:-reset}` where the DEFAULT was the verb and `V` was unbound (works,
  because the default IS reset); it never tested the inverse (name BOUND to a verb, benign default). r17's `resolve_param` rewrite preserved
  the defect. Either way it is a statically-resolvable real-mutation bypass and BLOCKS under the strict rule.
- Classification: pre-existing latent bug, newly surfaced; same META-pattern (value model under-approximates bash because the two-phase
  empty-binding pass and the second binding-aware pass disagree on `${name:-default}`).
- Required fix: do NOT resolve a `${name:-default}`/`${name-default}`/`${name:=default}` to its DEFAULT on the empty-binding pass — defer the
  default until bindings are known (or, on the empty-binding pass, leave the form literal so pass 2 can resolve `name` to its real value).
  Equivalently: when `name` is (or could be) a same-line binding, resolve against the collected bindings, not against empty bindings.

### [LOW, confidence HIGH] F3-R17 — `git rm` (and `git rm --cached`) is outside `is_blocked_operation` and ALLOWs a destructive tracked-file deletion on the canonical checkout (scope observation, not a model bypass).
- Location: `is_blocked_operation` (lib.rs:2769) models checkout/switch/restore/reset --hard/clean -f/rebase/merge/pull/stash pop/branch -f.
  `git -C <c> rm -rf .` deletes tracked files in the canonical worktree (confirmed mutated=yes), but `rm` is a deliberate, r1→r16-consistent
  scope boundary (recoverable via index/HEAD, like the permitted `stash` save). Not the verdict basis; flagged for the threat model owner to
  confirm `git rm` is intentionally out of scope.

### Resolved since r16 (verified)
- The ENTIRE r16 value-synthesis class DENIES through the production binary: printf width/multi-spec/repeating (`%-10s`, `%s%s`, `%s\n a b`,
  `%c%s`, `%0.s`), `${P/pat/repl}`, concatenated `$'…'$'…'`, and plain `echo` under `xpg_echo`. The sigil mechanism fails closed in every
  git-feeding position and does not over-deny benign read positions.
- The multi-word READ false-positive stays closed; the new sigil/allowlist surfaces add 0 false positives (55/55 ALLOW).

### Positive observations
- The r17 refactor is the right STRUCTURAL direction and is well-executed for its targeted scope: printf and `resolve_param` are now
  allowlists, ANSI-C `$'…'` is treated as the quoting construct it is (segments group correctly), `xpg_echo` is plumbed through, and the
  `${__unresolved__}` sigil gives a clean fail-closed path for unmodeled value-producers that does not over-deny reads. 31 unit + liveness
  green, Good ED25519, faces-last holds (2 faces, lib.rs-only fix, no Cargo.lock churn), 0 anti-pattern key churn, single dep serde_json, no
  new CVE surface, no secrets in delta.

### Note — the r1→…→r17 meta-pattern, now at the allowlist-FIDELITY axis
r17 correctly made the unmodeled value paths fail closed (the sigil), which closed the entire r16 class. But the convergence claim again
equates "the named (r16) forms denied" with "static closure reached," which is false. The residual is no longer a MISSING decode — it is two
MODELED paths that return a wrong-but-non-None value: (A) printf `%s`+trailing-literal emits the produced text in the wrong word order, and
(B) `${name:-default}` is resolved to its default on the empty-binding first pass before the same-line binding is collected. Both bypass the
sigil precisely because they DON'T return None. The durable fix is to tighten each modeled path so it returns a value ONLY when it can
reproduce bash's exact output (require an empty format-tail after `%s`; defer `:-`/`-`/`:=` defaults until bindings are known) and otherwise
fall through to the `${__unresolved__}` sigil — i.e. extend the fail-closed-by-construction discipline to the INSIDE of the allowlist, not
just its boundary.

---

## Residual risk (single most likely production failure if merged as-is)
An agent (or an injected instruction via the lethal-trifecta surface) emits `bash -c 'git -C <canonical> $(printf "%s --hard" reset)'` or
`bash -c 'V=reset; git -C <canonical> ${V:-x} --hard'` — statically-resolvable, ZERO command output read at runtime, ZERO read-from-stdin —
and it silently ALLOWs, re-contaminating the canonical checkout and reproducing FRIC-022/FRIC-1781062867 while all 31 tests and the liveness
gate stay green. The leak survives because two MODELED value paths return a wrong-but-non-None value instead of failing closed via the sigil.

Secondary (carried, unchanged): the hook shim fails OPEN if the Rust binary is unbuilt (main.rs maps any error → SUCCESS); ensure
CI/branch-protection builds it (structural enforcement, not hook reliance). Genuinely runtime-unknowable family (pre-existing, out of scope,
fail-closed in binding form): `read` from stdin/pipe; `$(cat)`/`$(curl)` opaque stdout. Scope: `git rm` (F3-R17) destructive-but-unmodeled.

## Required to clear
1. Close F1-R17 (printf `%s`+trailing-literal): require an empty/whitespace format-tail after the leading `%s`/`%b` before treating the args
   as the produced text; otherwise emit the `${__unresolved__}` sigil → fail closed. Model the true bash word order if you keep modeling it.
2. Close F2-R17 (`${name:-default}` default-eating): do NOT resolve `:-`/`-`/`:=` defaults on the empty-binding first pass; defer until
   same-line bindings are collected, or leave the form literal on pass 1 so pass 2 resolves `name` to its real bound value.
3. Confirm F3-R17 scope: decide whether `git rm`/`git rm --cached` belongs in `is_blocked_operation`; document the decision either way.
4. Add a DENY fixture for EVERY confirmed F1/F2 reproduction (the 10 leaks above + the `clean -fdx` variants). Current fixtures structurally
   cannot catch them (no `%s`+trailing-literal or `${V:-x}`-with-bound-name shape).
5. Re-run 31 + liveness + new fixtures; re-drive the TASK-2 sweep through the real binary with a FOREIGN session-cwd; confirm zero false
   positives on the merge-train + retrain + multi-word reads + the printf/`${}` read forms.
6. Re-state the convergence claim honestly only after auditing the allowlist-FIDELITY axis (printf format-tail after `%s`; `${name:-default}`
   pass ordering) — and confirm the sole residual is genuinely runtime-unknowable `$(cat)`/`$(curl)`/read-from-stdin.

## Security Checklist
- [x] No hardcoded secrets (lib.rs is pure policy logic; only dep is serde_json; no new deps; secrets scan clean)
- [~] All inputs validated — r16 value-synthesis class CLOSED (sigil fail-closed at the boundary); but two MODELED paths mis-model bash
      → 10 statically-resolvable real-mutation ALLOWs across 2 mechanisms (F1 printf `%s`+trailing-literal, F2 `${name:-default}` default-eating)
- [~] Injection prevention — r16 class CLOSED; F1/F2 OPEN (statically-assembled canonical mutation). F3 `git rm` scope noted
- [x] Authorization/policy enforced for correctly-modeled forms; 55/55 legit + worktree/other-repo/retrain ALLOW with 0 FP; sigil introduces
      no over-denial
- [x] Dependencies audited — single dep serde_json; no new guard deps; no CVE surface in delta
- [x] Tests + liveness gate green (31 unit + gate Pass 1); SSH-signed (Good ED25519); faces-last (2 faces, lib.rs-only fix, no Cargo.lock
      churn); 0 anti-pattern key churn

---

VERDICT: **BLOCK** — 10 statically-resolvable real-mutation bypasses across 2 mechanisms: F1 printf `%s`/`%b` with trailing literal format
text (`printf "%s --hard" reset` → `reset --hard`, mis-modeled word order, ALLOW), F2 `${name:-default}`/`${name-default}`/`${name:=default}`
default-eating (empty-binding first pass resolves the default before the same-line binding is collected, ALLOW). Each ALLOWs a canonical
`reset --hard`/`clean -fdx` end-to-end through the production binary from a FOREIGN cwd with NO `$(cat)`/`$(curl)`/read-from-stdin. The r16
value-synthesis class (printf width/multi-spec/repeating, `${P/pat/repl}`, concatenated `$'…'`, xpg_echo) is fully closed, the sigil mechanism
is structurally sound (fails closed in every git-feeding position, no read over-denial), there are zero real-command false positives
(55/55 ALLOW), and mechanics are clean — but the strict VERDICT RULE blocks on any statically-resolvable real-mutation bypass. The convergence
claim ("the value model is fail-closed by construction; the sole residual is the runtime-unknowable class") is FALSE: two MODELED value paths
return a wrong-but-non-None value and bypass the sigil. NEW mechanism vs r16 variant: F1 is a fidelity gap inside the new printf allowlist
(distinct from r16's width/multi-spec/repeating, all now closed); F2 is a latent pass-ordering bug the r16 review missed (also present in r16).
