# Security Review — PR #685 ROUND 8 (G011 canonical-checkout guard)

- Repo: jason931225/oyatie  Branch: agent/g011-checkout-guard  Head: 201ce78dc  Base: dev
- r8 delta vs r7 (7ac63d439..201ce78dc): exactly ONE functional line in
  tools/oya-checkout-guard-app/src/lib.rs — `has_unresolved_expansion` (lib.rs:1277)
  now matches the FULL word-altering metacharacter set ``` ` * ? [ ] { } ~ ``` (plus the
  existing `$`-expansion handling), the SAME set `target_path_is_dynamic` (lib.rs:1909)
  uses for paths. Plus 9 new brace DENY fixtures + 1 brace/glob arg-position ALLOW test.
  Settle commit 201ce78dc touches ONLY 2 *.generated.json faces. Fix commit 996b0c6d2
  touches ONLY lib.rs (no Cargo.lock churn this round).
- Reviewer: fresh-context security-reviewer (Claude Opus), Torvalds/attacker lens,
  /using-superpowers + /using-agent-skills + security-and-hardening + /ultraqa, all
  FOREGROUND, built from /Users/jasonlee/oyatie-worktrees/g011-checkout-guard.

## VERDICT: BLOCK

**Risk Level: HIGH.** The r8 metacharacter extension genuinely closes EVERY one of the 9
r7 brace reproductions (all DENY, rc=2) with ZERO precision cost — that part is correct and
complete for its stated scope (verb/command-name position only; reads keep metachar ARGS).
BUT the commit's CONVERGENCE claim is FALSE: it asserts "verb-position and path-position now
share one metacharacter closure" and "the only remaining bypass is a runtime-unknowable
$(prog) output." I constructed a **statically-resolvable real-mutation bypass that uses NO
metacharacter the closure covers and NO `$(prog)` runtime-unknowable substitution**:
shell word-splitting via a reassigned **IFS**. Both `IFS=<c>` and the variable value are
LITERAL same-line assignments the guard already parses (it even collects the binding), so
the bypass is fully determinable at parse time — it is NOT the founder-accepted
runtime-unknowable residual. Per the strict VERDICT RULE — "BLOCK on a statically-resolvable
real-mutation bypass" — this blocks.

Root cause (single, precise): `expand_with_bindings` (lib.rs:1323) substitutes `$name` ->
its bound value as a SINGLE literal word and never models IFS-driven word-splitting of an
UNQUOTED expansion. When `IFS` is reassigned to a character that appears in the value, bash
splits the expansion into multiple words at that delimiter, synthesizing a denied mutating
verb (or re-targeting `-C` onto canonical) that the post-substitution token — `resetx`,
`reset-`, `cleanz`, … — does NOT literally match and that carries NO residual sigil. Neither
`has_unresolved_expansion` (verb side) nor `target_path_is_dynamic` (path side) consults IFS,
so the r8 closures converge on an INCOMPLETE shell model. This is the r1->r8 meta-pattern at
the next layer: r8 unified the metacharacter SET across verb/path but both sides still share
the same IFS blindness, which no fixture exercises (30-green stays green).

This leak is PRE-EXISTING (the r7->r8 diff touched ONLY the has_unresolved_expansion
metachar line + tests; expand_with_bindings / collect_same_line_bindings have been
IFS-blind since at least r6). It is surfaced now because r8's commit message explicitly
claims static convergence with `$(prog)` as the SOLE residual — and IFS resplit falsifies
that claim with a clean, no-`$(...)`, no-metacharacter static form.

---

## Harness (commands + exact exit codes)

Built the real binary: `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard
--out /tmp/r8-guard-bin` (BUILD SUCCEEDED; Mach-O arm64). Driven via JSON hook payload
`{"tool_input":{"command":"…"}}` on stdin (the real enforcement surface, main.rs run()),
`OYA_CANONICAL_CHECKOUT=<canon>`, neutral cwd. rc=2 => DENY, rc=0 => ALLOW.
Calibration: `git -C <canon> {reset --hard|switch|restore .|checkout|clean -fdx|stash pop}`
-> 2; `{status|fetch|log}` -> 0 (verified).

**Adjudication discipline (real-mutation only counts):** a dedicated adjudicator
(/tmp/r8-mut.sh) per candidate: (1) builds a fresh DIRTY canonical repo (dirty tracked file
+ untracked file + extra branch); (2) records the guard rc via the real binary; (3) runs the
EXACT command in a clean `bash -c` subshell with REAL git; (4) diffs `git status --porcelain
-b` before/after. mutated=yes IFF the canonical working-tree/branch/untracked state changed.
A FINDING requires `guard_rc=0 AND mutated=yes`. An argv-capture fakegit on PATH confirmed
the exact words bash hands to git for the brace and IFS forms.

### TASK 1 — full accumulated corpus (r1..r7 named + ALL reproductions incl 9 r7 brace) — PASS
51-command corpus (transparent/unmodelled wrappers incl flock/runuser/cpulimit/timeout/
nice/nohup/xargs/parallel/env, sh -c/bash -c, GIT_DIR/GIT_WORK_TREE env context, r3
substitution `$(echo git)`/backtick/`eval $()`/`bash -c "$()"`, r4 depth, r5 ANSI-C/$VAR/
`${x:-}`/`"$(printf)"`, r6 `${x:=}/${x:+}/${x+}/${x/a/e}/${x//a/e}/${x:0:5}/${x:-$(echo)}` +
`$g` command-name, and ALL 9 r7 brace forms `{reset,}`/`{,reset}`/`{switch,}`/`{restore,}`/
`{checkout,}`/`{clean,}`/`{stash,} pop`/`{git,} -C … {reset,}`/`r{eset,}`):
- **PASS=50  FAIL=1.** The single non-DENY is bare `git -C <canon> stash` (rc=0). This is a
  PRE-EXISTING, UNCHANGED design choice — `is_blocked_operation` (lib.rs:1825, UNCHANGED
  r7->r8) blocks `stash` only with `pop`/`apply` args; bare `stash` (save-and-clean) was
  never modeled as a destructive canonical mutation in any round. Not an r8 regression; noted
  as a carried observation, does NOT block.
- All 9 r7 brace reproductions: **DENY (rc=2)** through the production binary. The r7 BLOCK
  finding (F3-R7) is RESOLVED.

### TASK 2 — CONVERGENCE TEST — CONFIRMED STATICALLY-RESOLVABLE REAL-MUTATION BYPASS (IFS)
Enumerated every verb/command-name synthesis mechanism; adjudicated each against REAL bash.

CONFIRMED LEAK CLASS — IFS word-split resplit (guard_rc=0 ALLOW; REAL canonical mutation;
re-confirmed through the production binary). End-to-end proof: dirty canonical (`M tracked`)
-> guard rc=0 ALLOW -> command runs -> canonical CLEAN (`[]`):
```
  rc=0  IFS=x; y=resetx;   git -C <canon> $y --hard       real reset    (DIRTY->CLEAN)
  rc=0  IFS=-; y=reset-;   git -C <canon> $y --hard       real reset    (DIRTY->CLEAN)
  rc=0  IFS=z; c=cleanz;   git -C <canon> $c -fdx         real clean    (untracked REMOVED)
  rc=0  IFS=w; r=restorew; git -C <canon> $r .            real restore  (DIRTY->CLEAN)
  rc=0  IFS=x; y=resetx;   git -C <canon> ${y} --hard     real reset    (brace-wrapped, splits too)
  rc=0  bash -c 'IFS=x; y=resetx; git -C <canon> $y --hard'   real reset (shell-agnostic wrapper)
  rc=0  IFS=x; p=<canon>x; git -C $p reset --hard         real reset    (PATH-side IFS resplit too)
```
Argv-capture fakegit confirms bash hands git the split words (e.g. `-C <canon> reset --hard`).
The IFS bypass is parse-time deterministic: `IFS=x` and `y=resetx` are LITERAL same-line
assignments (the guard already collects `y=resetx` as a simple-word binding). Both the
verb-side (`$y` -> `resetx`, no sigil, not a literal verb) and the path-side (`$p` -> `…x`,
not within canonical) miss it identically because neither models IFS — so r8's "identical
closures" converge on an incomplete model.

Shell-sensitivity note (does NOT save it): zsh does not word-split unquoted parameters by
default, so the BARE `IFS=x; y=resetx; git … $y` form mutates under bash but not zsh. The
`bash -c '…'` wrapper form FORCES bash semantics regardless of the caller's login shell and
mutates unconditionally — confirmed guard_rc=0 mutated=yes. An agent or an injected
instruction (the lethal-trifecta surface CLAUDE.md warns of) controls the full command string
and can emit the `bash -c` wrapper, so the bypass is reachable independent of the host shell.

Mechanisms that ALLOW but are FAITHFUL (no real mutation — correctly NOT leaks):
- `$((arith))` in verb (`$((0))reset`, `re$((0))set`): `$` handling -> token resolves to a
  non-verb; no mutation.
- history `!!:reset` (non-interactive bash -c: history disabled) -> literal -> git error.
- `\x67it` -> `xgit` (backslash-x is literal x), not git -> no mutation.
- single-line `alias g=git; shopt -s expand_aliases; g … reset` -> alias does NOT expand on
  the same parse pass in `bash -c` (fakegit saw nothing) -> no mutation.
- glob verb `rese[t]`/`rese*` (no filesystem match) -> stays literal -> git error; `~reset`
  now DENYs (tilde covered) with no mutation.
- partial-prefix brace `r{eset,}` -> `reset r` (two words) -> git extra-arg error -> no mut.

Mechanisms correctly DENIED (verb actually synthesized to a mutating canonical op):
- `${!r}` indirect, `${x}`/`$v` simple-binding, `$"reset"` locale, here-string `read v<<<…;
  $v`, `eval $(printf %b …)`, `command git …`, function-def shadowing (`reset(){…}; reset`),
  `g\it` (backslash-in-unquoted resolves to git): ALL DENY (rc=2). Reads with a LITERAL
  non-mutating subcommand and a residual-$ ARG correctly ALLOW.

RESIDUAL (genuinely runtime-unknowable, founder-accepted): `$(prog)` whose stdout is not
statically determinable — e.g. `git -C <canon> $(some-cmd) --hard`. The guard treats all
`$(…)` as opaque and ALLOWs; this is the accepted irreducible class for a parse-based
safety-net. NOTE: the IFS finding is NOT in this class — it requires no `$(…)` and resolves
fully at parse time.

### TASK 3 — FALSE-POSITIVE SWEEP (must ALLOW) — PASS, ZERO false positives
- 30/30 legit commands ALLOW: reads with metachar ARGS (`log {a,b}`, `show *.rs`,
  `diff HEAD~{1,2}`, `log {origin/dev,HEAD}`), reads with residual-$ args (`log $REF`,
  `diff ${BASE:-HEAD}`, `show $(git rev-parse HEAD)`), merge-train on canonical
  (`merge --ff-only`, `pull --ff-only`, `commit`, `push`, `fetch --all`), worktree mutations
  incl brace on NON-canonical (`reset --hard`, `{reset,} --hard`, `switch`, `checkout -- f`,
  `clean -fdx`), `ver=$(git rev-parse)`, `$(date)`, `$EDITOR notes.txt`, `sudo sh -c 'echo'`,
  `worktree list`/`worktree add`, full merge-train on non-canonical (`checkout -B dev
  origin/dev`, `reset --hard origin/dev`, `clean -fdx`).
- /tmp/retrain.sh (present this round) — all 8 git lines ALLOW (ls-remote/fetch/clean/
  checkout/add/commit/push/rev-parse against the agent worktree + main checkout, none target
  the calibrated canonical). 0 false positives.
- **TASK3 RESULT: ALLOW-PASS=30 + retrain 8; FALSE-POSITIVES=0.** The broadened metacharacter
  set does NOT over-deny any real merge-train/dev command: the check fires ONLY in
  verb/command-name position, so brace/glob/tilde in PATH/REF/PATHSPEC args of a read stay
  ALLOW. The r8 precision claim holds.

### TASK 4 — mechanics — all green
- buck2 //tools/oya-checkout-guard-app:oya-checkout-guard-app-unittest -> **30 passed; 0
  failed** (9 new brace fixtures + new brace/glob arg-position ALLOW test present and pass).
  NOTE: no fixture exercises the IFS resplit class — the recurring r1->r8 reason the leak
  survives 30-green.
- buck2 //cloud/cloud-ci/gates/oya-cloud-ci-enforcement-liveness-app:{…-unittest, …-gate}
  -> **Pass 1 each; Fail 0** (gate: 4 commands LIVE, green).
- SSH sigs: 996b0c6d2 + 201ce78dc both carry BEGIN SSH SIGNATURE. `%G?`=U is environmental
  (allowed-signers not provisioned in fresh context), same as r1-r7.
- Faces-last settle: 201ce78dc touches ONLY 2 *.generated.json faces (accounting-registry,
  scm-facts). Fix commit 996b0c6d2 = lib.rs ONLY (no Cargo.lock churn — cleaner than r7).
  Holds.
- Baseline key-diff both ways (merge-base d705932d4 .. HEAD over all 4 changed generated
  faces): **0 anti-pattern keys** (forbidden_foundry|unjustified|unowned|unreachable) added
  OR removed. No laundering.

---

## Findings (file:line, severity, confidence)

### [HIGH, confidence HIGH] F4-R8 — IFS resplit synthesizes a denied verb/target that the guard cannot see (static, no metachar, no $(prog))
- Location: lib.rs:1323 `expand_with_bindings` (substitutes `$name` -> bound value as ONE
  literal word, no IFS word-splitting); compounded at lib.rs:1270 `has_unresolved_expansion`
  (verb side, post-substitution token carries no sigil) and lib.rs:1909 `target_path_is_dynamic`
  (path side, post-substitution path has no metachar). lib.rs:1296 `collect_same_line_bindings`
  even collects `IFS=x` and `y=resetx` as bindings but nothing consumes IFS for splitting.
- Root cause: r8 unified the verb-side and path-side METACHARACTER set, but BOTH closures
  still model `$name` expansion as quoted single-word substitution. Bash applies word-splitting
  on the UNQUOTED expansion using the CURRENT IFS; when IFS is reassigned (literally, same
  line) to a character inside the value, the expansion splits into multiple words, producing a
  mutating verb (`resetx`->`reset`) or re-targeting `-C` onto canonical (`<canon>x`->`<canon>`).
  The post-split words match neither the literal verb table nor any residual-sigil check.
- Confirmed bypasses (rc=0; REAL bash MUTATES; via the production binary): `IFS=x; y=resetx;
  git -C <canon> $y --hard`; `IFS=-; y=reset-; … $y --hard`; `IFS=z; c=cleanz; … $c -fdx`;
  `IFS=w; r=restorew; … $r .`; `${y}` brace-wrapped variant; `bash -c '…'` shell-agnostic
  wrapper; and the PATH-side form `IFS=x; p=<canon>x; git -C $p reset --hard`. 6 verb-side +
  1 path-side, across reset/clean/restore.
- Exploitability: local, unauthenticated, single short one-liner, no length cap. The
  `bash -c 'IFS=x; …'` wrapper is shell-agnostic (forces bash regardless of the caller's
  zsh/bash). Trivially emitted by an agent or an injected instruction.
- Blast radius: silent canonical-checkout reset --hard / clean -fdx / restore, reproducing
  FRIC-022/FRIC-1781062867 while all 30 unit tests + the enforcement-liveness gate stay green.
- STATICALLY RESOLVABLE: `IFS=<c>` and the variable value are literal same-line assignments
  fully visible at parse time (the guard already parses both). NOT the founder-accepted
  runtime-unknowable `$(prog)` residual — the BLOCK class per the strict VERDICT RULE.
- Minimal fix direction (choose one, defense-in-depth prefers the broadest):
  (a) Detect a same-line `IFS=` reassignment to anything other than the default
      (` \t\n`); if present, treat every subsequent UNQUOTED `$name`/`${name}` expansion
      that contains the IFS delimiter as fail-closed (apply the split, or DENY the whole
      command-name/subcommand if any resulting word is a mutating verb / re-targets canonical).
  (b) Simpler and stricter: if a non-default `IFS=` assignment appears anywhere on the line
      and any unquoted variable expansion feeds the git command-name, -C target, or
      subcommand, DENY (fail-closed) — the legit use of a custom IFS feeding a git VERB/-C is
      essentially nil, and TASK-3 shows no real command relies on it.
  (c) Model word-splitting in `expand_with_bindings`: when substituting an UNQUOTED `$name`,
      split the value on the current same-line IFS and emit multiple words, then run the
      existing verb/target classification over the split result.
  Add a DENY fixture for each confirmed reproduction (reset/clean/restore verb-side + the
  path-side `-C $p` form + the `bash -c` wrapper) — current fixtures structurally cannot catch
  them, the recurring r1->r8 failure mode.

### Note — the r1->r8 meta-pattern, now at the word-splitting layer
Each prior round closed the named corpus and leaked the next expansion form past the new
fixtures: r5 substitution, r6 param-op, r7 brace, r8 ... IFS resplit. r8 correctly unified the
metacharacter SET across verb and path (a genuine improvement) but the underlying model still
treats `$name` as a single quoted word. The convergence claim is therefore premature: the true
closure is "every way bash can transform a token into a different word," of which
metacharacter expansion is one axis and IFS word-splitting is another orthogonal axis the
guard does not model. The path-side and verb-side now SHARE the same blind spot, which is why
a single mechanism (IFS) defeats both at once.

### Resolved since r7 (verified)
- F3-R7 (brace/glob/tilde in verb/command-name position): CLOSED by the metacharacter-set
  extension at lib.rs:1277. All 9 r7 brace reproductions DENY. Zero precision cost (30 legit
  commands incl reads-with-metachar-args + worktree-brace-mutations all ALLOW).

### Positive observations
- The metacharacter-set unification is the right direction and is correctly scoped to
  verb/command-name position — reads keep metachar ARGS, worktree mutations stay ALLOW, zero
  false positives across 30 legit + 8 retrain commands.
- 30 unit + liveness gate green, SSH-signed, faces-last settle holds (2 faces, lib.rs-only
  fix, no Cargo.lock churn), no key laundering, single guard dep (serde_json), no new CVE
  surface in the delta.

---

## Residual risk (single most likely production failure if merged as-is)
An agent (or an injected instruction via the lethal-trifecta surface) emits
`bash -c 'IFS=x; y=resetx; git -C <canonical> $y --hard'` (or the clean/restore/`-C $p`
variants) — a statically-resolvable form with NO metacharacter the closure covers and NO
runtime-unknowable `$(prog)` — and it silently ALLOWs, re-contaminating the canonical checkout
and reproducing FRIC-022/FRIC-1781062867 while all 30 tests and the liveness gate stay green.
Because the same IFS blindness sits on BOTH the verb side and the path side, the fix must model
word-splitting (or fail-closed on non-default same-line IFS feeding a git verb/-C), not just
extend a character set again.
Secondary (carried, unchanged): the hook shim fails OPEN if the Rust binary is unbuilt
(main.rs maps any error -> SUCCESS); ensure CI/branch-protection builds it (structural
enforcement, not hook reliance). Also carried: bare `git stash` (save) is not modeled as a
canonical mutation — low impact (recoverable), pre-existing, out of this PR's scope.

## Required to clear
1. Close F4-R8: model IFS word-splitting OR fail-closed when a non-default same-line `IFS=`
   reassignment feeds an unquoted variable expansion into the git command-name, -C target, or
   subcommand. Apply to BOTH the verb side (has_unresolved_expansion / subcommand classify)
   and the path side (target_path_is_dynamic / -C resolution) since both share the blind spot.
2. Add a DENY fixture for every confirmed reproduction: `IFS=x; y=resetx; git -C <canon> $y
   --hard`, `IFS=-; y=reset-; … $y --hard`, `IFS=z; c=cleanz; … $c -fdx`, `IFS=w; r=restorew;
   … $r .`, the `${y}` brace-wrapped form, the `bash -c '…'` wrapper, and the path-side
   `IFS=x; p=<canon>x; git -C $p reset --hard` — current fixtures cannot catch them.
3. Re-run 30 + liveness + new fixtures; re-drive the TASK-2 IFS sweep through the real binary;
   confirm zero false positives on the merge-train + retrain.sh set (esp. that no legit command
   sets a custom IFS feeding a git verb/-C).
4. Re-state the convergence claim honestly: with IFS modeled, the residual is the
   runtime-unknowable `$(prog)` class — and confirm no OTHER word-transforming axis (e.g.
   `extglob`, brace-range `{1..9}` if ever in verb position) remains unmodeled.

## Security Checklist
- [x] No hardcoded secrets (lib.rs is pure policy logic; only dep is serde_json; no new deps)
- [~] All inputs validated — literal + r5 + r6 + r7-brace forms sound; IFS word-split resplit
      in verb AND path position bypasses static analysis -> statically-resolvable real-mutation ALLOWs
- [~] Injection prevention — r7 brace class CLOSED; F4-R8 OPEN (IFS resplit canonical-mutation)
- [x] Authorization/policy enforced for modeled forms; zero false positives across 30 legit +
      8 retrain commands; path-side canonicalization + brace/glob/tilde verb+path handling sound
- [x] Dependencies audited — single dep serde_json; no new guard deps; no CVE surface in delta
- [x] Tests + liveness gate green (30 unit + gate Pass 1); SSH-signed; faces-last (2 faces,
      lib.rs-only fix); no key laundering
