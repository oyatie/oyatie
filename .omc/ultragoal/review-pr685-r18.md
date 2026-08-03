# Security Review — PR #685 ROUND 18 (G011 canonical-checkout guard) — NESTED-SUBSTITUTION-PRESERVE FIDELITY CHECK

**Scope:** `tools/oya-checkout-guard-app/src/lib.rs` (PreToolUse enforcement hook) + 2 generated cloud-ci faces.
PR jason931225/oyatie#685, branch `agent/g011-checkout-guard`, head `df06ba354`, base `dev`.
**Risk Level: HIGH.**

- r18 delta vs r17 (`e579b358e..df06ba354`): two commits.
  - `9d6cff508` `fix(checkout-guard): printf format/args fidelity + defer ${:-} default to bindings pass (review #685 r17)` —
    **lib.rs ONLY (+93/-28, no Cargo.lock churn).** Three changes: (F1) printf arm now SPLITS the FORMAT (first quote-aware word via new
    `split_first_shell_word`, backslashes preserved) from ARGS and models ONLY a BARE `%s`/`%b` (optionally `\n`/`\t`) → args, or a
    specifier-free literal → itself; any `%s`-with-trailing-literal/extra-specifier → None → sigil. (F2) the substitution pre-pass gained a
    `substitutions_only` mode: it resolves ONLY `$(…)`/backtick/ANSI-C and leaves `${…}`/`$var`/`$@`/`$N` literal, deferring `:-`/`-`/`:=`
    defaults to the bindings pass. (NEW) `static_command_output` short-circuits when an echo/printf body's `rest` contains `$(`/backtick:
    `return Some(rest.trim())` to preserve the inner substitution for inner-first fixpoint resolution. Adds 6 DENY fixtures (the r17 cases).
  - `df06ba354` `chore: settle generated cloud-ci faces` — 2 `*.generated.json` faces ONLY.
- Reviewer: fresh-context security-reviewer (Claude Opus), Torvalds/attacker lens, /using-superpowers + /using-agent-skills + /ultraqa,
  FOREGROUND, built from `/Users/jasonlee/oyatie-worktrees/g011-checkout-guard`. FOREIGN session-cwd (`/tmp`); mutation arrives via
  `-C`/binding indirection; every candidate confirmed against REAL bash with an argv-capturing fakegit (dirty→clean) that EXECs real git so
  canonical mutation is GENUINE. Adjudicator self-tested (canonical mutations → rc=2 + argv proof; reads → rc=0 clean; non-canonical
  `-C <wt|other>` → rc=0 + canonical untouched). r17 binary rebuilt for regression-vs-variant classification.

## VERDICT: BLOCK

r18 GENUINELY CLOSED both r17 findings. F1 (printf `%s`+trailing-literal) and F2 (`${V:-x}` default-eating) — all 10 r17 reproductions now
**DENY (rc=2)** through the production binary. The full named corpus r1..r17 is **45/45 DENY**. The FP sweep is **44/44 ALLOW, 0 false
positives** — `split_first_shell_word` + `substitutions_only` + nested-preserve introduce ZERO over-denial. Mechanics are green (31 unit +
liveness Pass 1, Good ED25519 on both commits, faces-last holds, 0 anti-pattern key churn, single dep serde_json unchanged, secrets-clean).

BUT the convergence claim — "the value model is fail-closed by construction and pass-ordering-correct; the sole residual is the
runtime-unknowable `$(cat)`/`$(curl)`/read-from-stdin" — is **FALSE**. I confirmed **12 statically-resolvable real-mutation bypasses across
THREE mechanisms** using NO runtime-unknowable input. Per the strict VERDICT RULE ("BLOCK on a statically-resolvable real-mutation bypass"),
this blocks. The most serious is a **NEW r18 REGRESSION**: the nested-substitution-preserve short-circuit `return Some(rest.trim())` ALLOWs
cases r17 correctly DENIED (verified: r17 rc=2, r18 rc=0 on the same inputs).

The three mechanisms and their single most-damning leaks (every byte statically determinable — NO `$(cat)`, NO `$(curl)`, NO stdin), exact
production-binary exit codes, argv captured from the EXEC'd fakegit (DIRTY ` M tracked.txt`/`?? untracked.txt` → CLEAN, the real
`reset --hard` reverting the tracked mod / `clean -fdx` destroying the untracked file):
```
  *** Mechanism A (NEW r18 REGRESSION) — nested-substitution-preserve drops echo/printf modeling ***
  git -C <c> $(printf "$(echo reset) --hard")     -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> reset --hard]   (r17: rc=2 DENY)
  git -C <c> $(printf "$(echo reset --hard)")     -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> reset --hard]   (r17: rc=2 DENY)
  git -C <c> $(printf "$(echo %s) --hard" reset)  -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> reset --hard]   (r17: rc=2 DENY)
  git -C <c> $(printf "$(echo %s) -fdx" clean)    -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> clean -fdx]     (r17: rc=2 DENY)
  git -C <c> $(echo "$(echo reset) --hard")       -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> reset --hard]   (r17: rc=2 DENY)
  git -C <c> $(printf "`echo reset` --hard")      -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> reset --hard]   (r17: rc=2 DENY)
  git -C <c> $(printf "%s --hard" "$(echo reset)") -> rc=0 ALLOW *** LEAK *** argv=[-C <c> reset --hard]   (sub in ARGS, not format)
  git -C <c> $(printf "$(echo $(echo reset)) --hard") -> rc=0 ALLOW *** LEAK *** argv=[-C <c> reset --hard] (deep)
  *** Mechanism B (PRE-EXISTING, also leaks on r17) — chained binding whose VALUE is another $var, consumed by a `:-` default ***
  V=reset; W=$V; git -C <c> ${W:-x} --hard        -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> reset --hard]
  V=clean; W=$V; git -C <c> ${W:-x} -fdx          -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> clean -fdx]
  *** Mechanism C (PRE-EXISTING, also leaks on r17) — positional-param-with-default `${1:-x}` after set-- ***
  set -- reset; git -C <c> ${1:-x} --hard         -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> reset --hard]
  set -- clean; git -C <c> ${1:-x} -fdx           -> rc=0 ALLOW *** LEAK ***  argv=[-C <c> clean -fdx]
```
All MUTATE canonical end-to-end through the production binary from a FOREIGN cwd with an argv-capturing fakegit.

---

## Harness (commands + exact exit codes)

Built the real binary: `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out /tmp/r18-guard-bin` → BUILD SUCCEEDED
(Mach-O arm64). Driven via JSON hook payload `{"tool_input":{"command":"…"}}` on stdin (the real enforcement surface),
`env -i OYA_CANONICAL_CHECKOUT=<canon> PATH=… <bin>`, binary cwd=/tmp (canonical != cwd, FOREIGN). rc=2 ⇒ DENY, rc=0 ⇒ ALLOW. Secrets scan
clean; single dep `serde_json` (workspace), unchanged.

**Adjudication discipline (real-mutation only counts):** per candidate (1) fresh DIRTY real git repo (` M tracked.txt` + `?? untracked.txt`);
(2) guard rc via the real binary with the canonical path injected as a LITERAL absolute path (placeholders `@CANON@`/`@WT@`/`@OTHER@` expanded
identically for guard payload AND bash exec); (3) the EXACT command in `bash -c` from FOREIGN cwd /tmp with an argv-capturing fakegit that
EXECs real git, so `-C <canon> reset --hard` genuinely mutates; (4) diff `git status --porcelain` before/after. A LEAK requires
`guard_rc=0 AND mutated=yes`. Adjudicator self-test sound (`reset --hard`/`clean -fdx`/`restore .` → rc=2 + argv; `status`/`log` → rc=0 clean;
`-C <wt|other> reset --hard` → rc=0 + canonical untouched). r17 binary (`e579b358e`) rebuilt for regression classification.

### TASK 1 — full accumulated corpus (r1..r17 named + ALL reproductions incl r17 cases) — PASS for the modeled corpus
- **Named corpus r1..r17: 45/45 DENY (rc=2).** Confirmed r17 F1/F2 FIXED: all 10 r17 leaks (`printf "%s --hard" reset`,
  `printf "%b --hard"`, `printf "%s -fdx" clean`, `printf "%s --hard HEAD"`, `P=$(printf "%s --hard" reset)`, `${V:-x}`/`${V-x}`/`${V:=x}`
  default-eating, `${V:-x}` clean side, `reset ${W:-x}` flag side) → rc=2. No regression on `$(echo $(printf "reset --hard"))`,
  `$(echo $(echo reset --hard))`, ``$(echo `printf "reset --hard"`)``, `printf "reset\t--hard"`, `printf "%s\n" reset --hard` — all DENY.
- Out-of-model-scope NON-DENY (NOT verdict basis, r1→r17-consistent): `reset --merge`/`--keep` (only `reset --hard` modeled); `git rm`
  (not in `is_blocked_operation`). The `mut=no` rows in the corpus are fixture artifacts (e.g. `reset --hard HEAD~1` has no parent after one
  commit; `stash pop` with nothing stashed) — the guard still DENIED all of them (rc=2).

### TASK 2 — CONVERGENCE TEST — 12 STATICALLY-RESOLVABLE REAL-MUTATION BYPASSES (BLOCK); r17 F1/F2 CLOSED; sigil sound
Adjudicated every value/synthesis/pass-ordering/non-value axis the prompt named vs REAL bash (foreign cwd) via the production binary.

CLOSED (caught — the r18 fixes WORK for their targeted scope):
```
  printf %s + trailing literal:  $(printf "%s --hard" reset)/"%b --hard"/"%s -fdx" clean/"%s --hard HEAD"  rc=2 (split_first_shell_word → fmt="%s --hard" not bare → None → sigil)
  printf bare %s, args modeled:  $(printf "%s" "reset --hard")  rc=2  ; $(printf "%s\n" reset --hard)  rc=2
  ${V:-x} default-eating:        V=reset; ${V:-x} --hard / ${V-x} / ${V:=x} / clean ${V:-x} -fdx / reset ${W:-x}  rc=2 (substitutions_only defers default → bindings pass resolves V)
  ${V:-${W}} nested default:     W=reset; ${V:-${W}} --hard  rc=2 ; V=reset; ${V:-${W}} --hard  rc=2
  X=${V:-x} indirection:         V=reset; X=${V:-x}; $X --hard  rc=2
  set-- then $V $2:              set -- reset --hard; V=${1}; $V $2  rc=2
  printf with escaped quote:     $(printf "%s\" --hard" reset)  rc=2 (fmt=%s" not bare → None)
  printf positional %1$s:        $(printf "%1$s --hard" reset)  rc=2
  alias / git -c / cat<<heredoc: alias g='git -C <c>'; g reset --hard / git -c core.pager=cat -C <c> reset --hard / cat<<E|bash …  rc=2
  runtime-unknowable residual:   $(cat)/$(curl)/read<stdin in binding  rc=2 (fail-closed, correct)
```

CONFIRMED LEAKS (guard_rc=0 ALLOW; REAL canonical mutation, foreign cwd; production binary; ALL words STATICALLY determinable):
```
  *** Mechanism A — NEW r18 REGRESSION: nested-substitution-preserve short-circuit drops echo/printf value modeling ***
  rc=0 mut=yes  git -C <c> $(printf "$(echo reset) --hard")        argv=[-C <c> reset --hard]   (r17 DENIED this)
  rc=0 mut=yes  git -C <c> $(printf "$(echo reset --hard)")        argv=[-C <c> reset --hard]   (r17 DENIED)
  rc=0 mut=yes  git -C <c> $(printf "$(echo %s) --hard" reset)     argv=[-C <c> reset --hard]   (r17 DENIED)
  rc=0 mut=yes  git -C <c> $(printf "$(echo %s) -fdx" clean)       argv=[-C <c> clean -fdx]     (r17 DENIED; untracked DESTROYED)
  rc=0 mut=yes  git -C <c> $(echo "$(echo reset) --hard")          argv=[-C <c> reset --hard]   (r17 DENIED)
  rc=0 mut=yes  git -C <c> $(printf "`echo reset` --hard")         argv=[-C <c> reset --hard]   (r17 DENIED; backtick)
  rc=0 mut=yes  git -C <c> $(printf "%s --hard" "$(echo reset)")   argv=[-C <c> reset --hard]   (sub in ARGS not format)
  rc=0 mut=yes  git -C <c> $(printf "$(echo $(echo reset)) --hard") argv=[-C <c> reset --hard]  (deep)
  *** Mechanism B — chained binding (value is another $var) consumed by a `:-` default (pre-existing; r17 also rc=0) ***
  rc=0 mut=yes  V=reset; W=$V;   git -C <c> ${W:-x} --hard         argv=[-C <c> reset --hard]
  rc=0 mut=yes  V=reset; W=${V}; git -C <c> ${W:-x} --hard         argv=[-C <c> reset --hard]
  rc=0 mut=yes  V=clean; W=$V;   git -C <c> ${W:-x} -fdx           argv=[-C <c> clean -fdx]
  *** Mechanism C — positional-param-with-default ${1:-x} after set-- (pre-existing; r17 also rc=0) ***
  rc=0 mut=yes  set -- reset; git -C <c> ${1:-x} --hard            argv=[-C <c> reset --hard]
  rc=0 mut=yes  set -- clean; git -C <c> ${1:-x} -fdx              argv=[-C <c> clean -fdx]
  rc=0 mut=yes  set -- reset --hard; git -C <c> ${1:-x} ${2:-y}    argv=[-C <c> reset --hard]
```
(Also pre-existing, NON-VALUE path, both r17+r18 rc=0: `bash <(echo "git -C <c> reset --hard")` process substitution — flagged as scope.)

RESIDUAL genuinely runtime-unknowable (verified fail-closed rc=2): `$(cat file)`, `$(curl …)`, `read` from stdin/pipe. These are NOT the sole
residual — Mechanisms A/B/C above are parse-time-determinable and BLOCK.

Discriminator (isolates each gap precisely — the QUOTES around the format are the Mechanism-A discriminator):
```
  git -C <c> $(printf "reset --hard")               rc=2 DENY  (literal format, no nested sub → modeled correctly → verb caught)
  git -C <c> $(printf "$(echo reset) --hard")       rc=0 LEAK  (nested sub in rest → short-circuit returns rest VERBATIM incl. quotes →
                                                                printf modeling NEVER re-runs → produced text carries `"` → verb at git
                                                                position not matched → ALLOW; bash strips quotes → reset --hard)
  git -C <c> $(printf $(echo reset) --hard)         rc=2 DENY  (UNQUOTED nested sub → no surviving quote chars → verb caught)
  V=reset; git -C <c> ${V:-x} --hard                rc=2 DENY  (substitutions_only defers default; bindings pass resolves V → reset)
  V=reset; W=$V; git -C <c> ${W:-x} --hard          rc=0 LEAK  (W's collected value is the literal `$V` — collect_same_line_bindings does not
                                                                chase $V → ${W:-x} resolves W to `$V` (not reset) → verb mismodeled → ALLOW)
  V=reset; X=${V:-x}; git -C <c> $X --hard          rc=2 DENY  (X bound to a non-default-bearing var → resolved)
  set -- reset; git -C <c> ${1:-x} --hard           rc=0 LEAK  (${1:-x} positional-with-default not resolved against set-- $1 → ALLOW)
```

### TASK 3 — FALSE-POSITIVE SWEEP (must ALLOW) — 44/44 legit ALLOW; 0 FP
- **All legit forms ALLOW (rc=0), 0 false positives.** Merge-train reads (status/status --short/log --oneline -5/log --format="%H %an"/
  log --format="%H"/diff --stat/show HEAD/rev-parse HEAD/fetch --all --prune/branch -a/remote -v/tag -l/stash list); `add -A`; commit -m with
  tricky messages (`"x=y"`, `"reset --hard fix"`, `"set IFS=, and reset"`); `${V:-status}` reads (`${V:-status}`,
  `V=log; ${V:-status} --oneline`, `${UNSET:-status} --short`); printf reads (`$(printf "%s" "log --oneline")`, `printf "%s\n" "release v1.2"`,
  `$(printf '%s\n' status)`, `P=$(printf "%s" "status --short"); git $P`); multi-word READ vars (`P=log; P+=" --oneline"; git $P`,
  `P="show HEAD"; git $P`, `P="status --short"; git $P`); `V=$(git rev-parse HEAD)`; echo/printf normal; worktree + other-repo mutations
  (`git -C <wt|other> reset --hard`/`clean -fdx`/`checkout -f`); `bash /tmp/retrain.sh`; for/while; `$(date)`; `$EDITOR`; `sudo sh -c`;
  non-mutating verbs (checkout bare, reset --soft, clean -n, worktree list).
- **The NEW FP-risk surfaces add ZERO over-denial.** `split_first_shell_word` (`$(printf "%s" "log --oneline")` ALLOWs), `substitutions_only`
  (`${V:-status}` reads ALLOW), and nested-preserve (`$(printf "$(echo %s)" "log --oneline")` ALLOWs) do not over-deny any legit read.

### TASK 4 — mechanics — all green
- `//tools/oya-checkout-guard-app:oya-checkout-guard-app-unittest` → **31 passed; 0 failed** (Pass 1). NOTE: NO fixture exercises the
  Mechanism-A/B/C leak shapes (`$(printf "$(echo …) --hard")`, `W=$V; ${W:-x}`, `set --; ${1:-x}`) — the recurring r1→r17 reason a leak
  survives a green suite (the suite asserts the modeled shapes, not the mis-modeled ones). The 6 new fixtures cover only the r17 cases.
- `//cloud/cloud-ci/gates/oya-cloud-ci-enforcement-liveness-app:…-unittest` → Pass 1 (Fail 0).
- SSH sigs: `9d6cff508` + `df06ba354` both carry Good ED25519 (SHA256:5grGNUtX9Zgmy1SWne6wF9DR8W1ElUQaF/Z8SYRz8E8).
- Faces-last settle: `df06ba354` touches ONLY 2 `*.generated.json` faces (accounting-registry, scm-facts). Fix commit `9d6cff508` = lib.rs
  ONLY (+93/-28, no Cargo.lock churn). Holds.
- Baseline key-diff both ways (`e579b358e..df06ba354` over generated faces): **0 anti-pattern keys**
  (forbidden_foundry|unjustified|unowned|unreachable) added OR removed. Face changes are pure provenance (source_inputs_digest
  f8bbdb5e→267ffa13, last_touch_commit, lib.rs blob hash, head_time_secs). No laundering.
- Dependencies: single dep `serde_json` (workspace), unchanged; no new CVE surface. No secrets in the lib.rs delta (the word "token" appears
  only in a `shell_tokens` code comment).

---

## Findings (file:line, severity, confidence)

### [HIGH, confidence HIGH] F1-R18 (NEW REGRESSION) — the nested-substitution-preserve short-circuit in `static_command_output` returns the echo/printf `rest` VERBATIM (quotes intact) whenever `rest` contains `$(`/backtick, dropping ALL value modeling; after the inner substitution resolves, the produced text carries literal quote chars / unmodeled format so the canonical verb is never matched → ALLOW of a canonical reset/clean that r17 DENIED.
- Location: `static_command_output` early return (lib.rs:2264-2268):
  `if (rest.contains("$(") || rest.contains('`')) && matches!(command_basename(cmd), Some("echo"|"printf")) { return Some(rest.trim().to_owned()); }`.
  For `printf "$(echo reset) --hard"`, `rest = "$(echo reset) --hard"` is returned VERBATIM as the produced output — the printf FORMAT/ARGS
  model (the F1 split, the `%s` allowlist) never runs. After the fixpoint resolves the inner `$(echo reset)`, the produced text is
  `"reset --hard"` WITH its surrounding double-quote characters; emitted at the git-argument position, the leading token is not matched as a
  blocked verb (the quotes survive into the verb scan), so the guard ALLOWs. bash strips the quotes and runs `reset --hard`. The short-circuit
  fires for a substitution ANYWHERE in `rest` (format OR args), so `$(printf "%s --hard" "$(echo reset)")` leaks too.
- Confirmed regression vs r17 (rebuilt `e579b358e`): r17 rc=2 DENY → r18 rc=0 ALLOW on every Mechanism-A case
  (`$(printf "$(echo reset) --hard")`, `$(printf "$(echo %s) --hard" reset)`, `$(echo "$(echo reset) --hard")`,
  ``$(printf "`echo reset` --hard")``, `$(printf "$(echo reset --hard)")`, deep `$(printf "$(echo $(echo reset)) --hard")`).
  All MUTATE canonical (DIRTY→CLEAN; argv `[-C <c> reset --hard]`/`[-C <c> clean -fdx]`).
- Exploitability: local, unauthenticated, single short one-liner, no runtime input — `printf "$(echo reset) --hard"` is fully static.
  Trivially emitted by an agent or an injected instruction (the lethal-trifecta surface CLAUDE.md warns of).
- Blast radius: silent canonical `reset --hard` / `clean -fdx`, reproducing FRIC-022/FRIC-1781062867 while all 31 unit tests + the liveness
  gate stay green.
- Classification: NEW r18 mechanism — a regression introduced by the inner-first nested-substitution-preserve change. The control
  `$(printf "reset --hard")` (no nested sub) and the UNQUOTED `$(printf $(echo reset) --hard)` both DENY; only the quoted nested-sub form leaks.
- Required fix: do NOT return the value-producer `rest` verbatim. Either (a) recurse: resolve the inner substitution FIRST, then re-feed the
  resolved body through `static_command_output` so the printf/echo FORMAT/ARGS model actually runs on the resolved text (and strips quotes
  before the verb scan); or (b) when a value-producer body still contains an unresolved `$(`/backtick after a bounded number of fixpoint
  iterations, emit `${__unresolved__}` → fail closed. Returning `rest` verbatim bypasses both the model and the sigil.

### [HIGH, confidence HIGH] F2-R18 (pre-existing, also leaks on r17) — a chained binding whose VALUE is another `$var` (`W=$V`), consumed by a `${W:-default}`, mis-models: `collect_same_line_bindings` records W's value as the LITERAL `$V` (it does not chase the indirection), so `${W:-x}` resolves W to `$V` (not its transitive value `reset`) and the verb is mismodeled → ALLOW.
- Location: `collect_same_line_bindings` + `resolve_param`/`binding_value` (the binding-aware pass, lib.rs:1318-1322 + ~2190). On the bindings
  pass, `${W:-x}` finds W bound to `$V` (literal), uses that as the value (non-empty → default not taken), but `$V` is not transitively
  resolved at that site → the modeled verb is `$V`/empty, never `reset`. bash performs the full chain `W=$V=reset`.
- Confirmed (rc=0; argv `[-C <c> reset --hard]`/`[… clean -fdx]`): `V=reset; W=$V; ${W:-x} --hard`; `V=reset; W=${V}; ${W:-x} --hard`;
  `V=clean; W=$V; ${W:-x} -fdx`. r17 binary: identical rc=0 (pre-existing). NOTE `X=${V:-x}; $X` (no default at the USE site) DENIES — the leak
  needs the default-bearing form at the use site combined with an indirect binding value.
- Required fix: resolve binding VALUES transitively (chase `W=$V` to its bound `$V`) before they feed a `${name:-default}` use, or treat an
  indirect (`$`-containing) binding value as unresolved → fail closed at the use site.

### [HIGH, confidence HIGH] F3-R18 (pre-existing, also leaks on r17) — a positional-param-with-default `${1:-x}` after `set -- reset` is not resolved against the positional params, so the modeled verb is the benign default → ALLOW.
- Location: `expand_with_bindings` positional-with-brace path + `collect_positional_params`. `${1:-x}` (brace form with a `:-` default) is not
  resolved to `$1=reset` on the bindings pass (the `${…}`/`resolve_param` path handles named params, not positional-with-default), so the
  default `x` is modeled. bash expands `${1:-x}` → `reset`.
- Confirmed (rc=0; argv `[-C <c> reset --hard]`/`[… clean -fdx]`): `set -- reset; ${1:-x} --hard`; `set -- clean; ${1:-x} -fdx`;
  `set -- reset --hard; ${1:-x} ${2:-y}`. r17: identical rc=0 (pre-existing).
- Required fix: resolve `${N:-default}`/`${N-default}` positional forms against the collected positionals before falling back to the default.

### [LOW, confidence HIGH] F4-R18 — process substitution `bash <(echo "git -C <c> reset --hard")` ALLOWs and mutates (pre-existing, non-value path, both r17+r18 rc=0). `git rm` remains outside `is_blocked_operation` (r17 F3 carried). Scope observations, not the verdict basis, but flagged for the threat-model owner.

### Resolved since r17 (verified)
- F1-R17 (printf `%s`+trailing-literal) CLOSED: `split_first_shell_word` keeps a quoted multi-word format as one unit and the bare-`%s`
  allowlist rejects any trailing literal → None → sigil → fail-closed. All 5 F1 reproductions DENY.
- F2-R17 (`${name:-default}` default-eating) CLOSED: `substitutions_only` leaves `${…}` literal on the pre-pass so the default is NOT eaten
  before the same-line binding is collected; the bindings pass resolves the bound name. All 5 F2 reproductions (incl. flag side) DENY.
- No regression on the r17-intended nested forms `$(echo $(printf …))` / `$(echo $(echo …))` / `printf \t` — all DENY.

### Note — the r1→…→r18 meta-pattern, now BIDIRECTIONAL (a fix RE-OPENED a closed case)
r18 correctly closed r17's two findings, but the inner-first nested-substitution-preserve change (added to make `$(echo $(printf …))` resolve
inner-first) RE-OPENED a class r17 had caught: returning the value-producer `rest` verbatim bypasses BOTH the FORMAT/ARGS model AND the
`${__unresolved__}` sigil. This is the first round where a fix introduced a NEW real-mutation regression (r17 rc=2 → r18 rc=0), not merely
left a latent gap. The convergence claim again equates "the named (r17) forms denied" with "static closure reached," which is false. The
durable fix is the same discipline the sigil already embodies: a value-producer path must return a value ONLY when it can reproduce bash's
exact output on a FULLY-resolved body, and otherwise fall through to `${__unresolved__}` — never short-circuit with raw `rest`.

---

## Residual risk (single most likely production failure if merged as-is)
An agent (or an injected instruction via the lethal-trifecta surface) emits `bash -c 'git -C <canonical> $(printf "$(echo reset) --hard")'` or
`bash -c 'V=reset; W=$V; git -C <canonical> ${W:-x} --hard'` or `bash -c 'set -- reset; git -C <canonical> ${1:-x} --hard'` — all
statically-resolvable, ZERO command output read at runtime, ZERO read-from-stdin — and it silently ALLOWs, re-contaminating the canonical
checkout and reproducing FRIC-022/FRIC-1781062867 while all 31 tests and the liveness gate stay green. The most dangerous is F1-R18: it is a
NEW regression of a case r17 explicitly DENIED.

Secondary (carried, unchanged): the hook shim fails OPEN if the Rust binary is unbuilt (main.rs maps any error → SUCCESS); ensure
CI/branch-protection builds it (structural enforcement, not hook reliance). Genuinely runtime-unknowable family (fail-closed in binding form):
`read` from stdin/pipe; `$(cat)`/`$(curl)` opaque stdout. Scope: process substitution `<(…)` (F4); `git rm` destructive-but-unmodeled.

## Required to clear
1. Close F1-R18 (nested-substitution-preserve regression): do NOT `return Some(rest.trim())` verbatim for a value-producer; resolve the inner
   substitution first then RE-RUN `static_command_output` on the resolved body (so the printf/echo model + quote-strip apply), or emit
   `${__unresolved__}` if a `$(`/backtick remains after bounded fixpoint iterations. Add DENY fixtures for every Mechanism-A reproduction
   (`$(printf "$(echo reset) --hard")`, `$(echo "$(echo reset) --hard")`, backtick + `%s` + ARGS-side + deep variants).
2. Close F2-R18 (chained-binding `:-` default): resolve binding values transitively before a `${name:-default}` use, or treat a `$`-containing
   binding value as unresolved → fail closed. Add `W=$V; ${W:-x}` DENY fixtures.
3. Close F3-R18 (positional `${1:-x}` default): resolve `${N:-default}` against collected positionals before the default. Add
   `set -- reset; ${1:-x}` DENY fixtures.
4. Decide scope on F4 (process substitution `<(…)`, `git rm`); document the decision either way.
5. Re-run 31 + liveness + new fixtures; re-drive the TASK-2 sweep through the real binary from a FOREIGN session-cwd; confirm 0 false
   positives on the merge-train + retrain + multi-word reads + the printf/`${}`/nested-sub read forms.
6. Re-state the convergence claim honestly only after auditing the value-producer FIDELITY axis on FULLY-RESOLVED bodies — and confirm the sole
   residual is genuinely runtime-unknowable `$(cat)`/`$(curl)`/read-from-stdin.

## Security Checklist
- [x] No hardcoded secrets (lib.rs is pure policy logic; only dep is serde_json; no new deps; secrets scan clean)
- [~] All inputs validated — r17 F1/F2 CLOSED; but 12 statically-resolvable real-mutation ALLOWs across 3 mechanisms (F1 nested-preserve
      NEW REGRESSION, F2 chained-binding `:-`, F3 positional `${1:-x}`)
- [~] Injection prevention — r17 class CLOSED; F1/F2/F3 OPEN (statically-assembled canonical mutation). F4 process-sub / `git rm` scope noted
- [x] Authorization/policy enforced for correctly-modeled forms; 44/44 legit + worktree/other-repo/retrain ALLOW with 0 FP; new surfaces
      introduce no over-denial
- [x] Dependencies audited — single dep serde_json; no new guard deps; no CVE surface in delta
- [x] Tests + liveness gate green (31 unit + gate Pass 1); SSH-signed (Good ED25519); faces-last (2 faces, lib.rs-only fix, no Cargo.lock
      churn); 0 anti-pattern key churn

---

VERDICT: **BLOCK** — 12 statically-resolvable real-mutation bypasses across 3 mechanisms. **NEW r18 REGRESSION (F1)**: the
nested-substitution-preserve short-circuit `return Some(rest.trim())` returns a value-producer's `rest` verbatim (quotes intact), dropping the
printf/echo FORMAT/ARGS model AND the `${__unresolved__}` sigil, so `git -C <c> $(printf "$(echo reset) --hard")` ALLOWs a canonical reset that
r17 DENIED (r17 rc=2 → r18 rc=0, verified by rebuilding e579b358e). **PRE-EXISTING (F2/F3, also rc=0 on r17)**: chained binding `W=$V`
consumed by `${W:-x}` (binding value not chased), and positional `${1:-x}` after `set --` (not resolved against positionals). Each ALLOWs a
canonical `reset --hard`/`clean -fdx` end-to-end through the production binary from a FOREIGN cwd with NO `$(cat)`/`$(curl)`/read-from-stdin.
r18 genuinely CLOSED both r17 findings (F1 printf `%s`+trailing-literal, F2 `${V:-x}` default-eating — all 10 reproductions DENY), the named
corpus r1..r17 is 45/45 DENY, there are zero real-command false positives (44/44 ALLOW), and mechanics are clean — but the strict VERDICT RULE
blocks on any statically-resolvable real-mutation bypass. The convergence claim ("the value model is fail-closed by construction and
pass-ordering-correct; the sole residual is the runtime-unknowable class") is FALSE: a value-producer path returns raw `rest` and two
default-bearing param paths return a wrong-but-non-None value, all bypassing the sigil. NEW mechanism vs r17 variant: F1 is a NEW REGRESSION
(the inner-first nested-preserve change re-opened a class r17 caught); F2/F3 are pre-existing latent bugs the r17 review did not surface.
