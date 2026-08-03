# Security Review — PR #685 ROUND 7 (G011 canonical-checkout guard)

- Repo: jason931225/oyatie  Branch: agent/g011-checkout-guard  Head: 7ac63d439  Base: dev
- r7 delta vs r6 (7886c9e7f..7ac63d439): +60 net lines in tools/oya-checkout-guard-app/src/lib.rs.
  (1) normalize_static_expansions now iterates to a bounded fixpoint (<=8 passes) — closes r6 F1.
  (2) NEW has_unresolved_expansion(token) (lib.rs:1270-1285); the git block now also fires when the
  command-NAME carries a residual sigil (lib.rs:349) and blocked_operation is OR'd with
  has_unresolved_expansion(&invocation.subcommand) (lib.rs:373-374) — claimed GENERAL fail-closed for F2.
  Settle commit 7ac63d439 touches only 2 *.generated.json faces.
- Reviewer: fresh-context security-reviewer (Claude Opus), Torvalds/attacker lens,
  /using-superpowers + /using-agent-skills + /ultraqa, all FOREGROUND, built from
  /Users/jasonlee/oyatie-worktrees/g011-checkout-guard.

## VERDICT: BLOCK

**Risk Level: HIGH.** The r7 fixpoint + residual-sigil fail-closed genuinely closes EVERY one of the 12 r6
reproductions (all DENY, rc=2) with zero precision cost — that part is correct and complete for its stated
scope. BUT the claimed convergence ("any expansion the normalizer cannot statically resolve into a
provably-safe token DENYs") is FALSE. `has_unresolved_expansion` recognizes ONLY `$` and backtick sigils;
it is blind to **brace expansion** `{…}` (and glob/tilde). A single-element empty-alternative brace in the
git SUBCOMMAND or COMMAND-NAME position — `{reset,}`, `{,reset}`, `{switch,}`, `{restore,}`, `{checkout,}`,
`{clean,}`, `{,stash} pop` — is a STATICALLY-RESOLVABLE shell expansion that bash rewrites to the denied
mutating verb, yet the literal tokenizer sees a non-mutating word and `has_unresolved_expansion` returns
false. I confirmed 8 distinct minimal reproductions against REAL git (working tree DIRTY->CLEAN or branch
switch verified) AND through the REAL hook binary with the exact JSON payload (rc=0 ALLOW). Per the strict
VERDICT RULE — "BLOCK on a statically-resolvable real-mutation bypass" — this blocks. This is NOT the
founder-accepted runtime-unknowable residual: brace expansion is fully determinable at parse time (the
guard's OWN `target_path_is_dynamic`, lib.rs:1901-1906, already classifies `{` `}` `*` `?` `~` as dynamic
for PATHS — the new subcommand/command-name fail-closed check simply omits them).

Root cause (single, precise): the r7 fail-closed predicate models the WRONG closure. It enumerates `$`/`` ` ``
expansion sigils instead of "any byte that bash treats as a word-altering expansion metacharacter." Brace
(and glob/tilde) alter the resolved subcommand without a `$`. lib.rs:1270-1285 vs the path-side closure at
lib.rs:1901-1906 — the guard already has the correct metacharacter set in one place and the narrow set in
the other.

---

## Harness (commands + exact exit codes)

Built the real binary: `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out /tmp/r7-guard-bin`
(BUILD SUCCEEDED; Mach-O arm64). Driven via JSON hook payload `{"tool_input":{"command":"…"}}` on stdin
(the real enforcement surface, lib.rs:518-536), `OYA_CANONICAL_CHECKOUT=/private/tmp/r7-canon`, neutral cwd.
rc=2 => DENY, rc=0 => ALLOW. Calibration: `git -C C {switch|reset --hard|restore .|checkout other}` -> 2;
`{status|fetch|log}` -> 0 (verified).

**Adjudication discipline (real-mutation only counts):** every candidate ALLOW was adjudicated TWO ways:
(a) a fakegit argv-capture on PATH under `bash -c` to SEE the exact subcommand/target bash hands to git;
(b) a real `git init`'d repo (file `f` dirtied; mutation = working tree returns to CLEAN OR branch changes OR
untracked removed OR stash applied) under `bash -c`. Only forms that ACTUALLY mutate are reported as leaks.

### TASK 1 — full accumulated corpus (r1..r6 named + reproductions, 65 commands) — PASS, all DENY
- r1..r4 (transparent/unmodelled wrappers x17 incl. flock/runuser/cpulimit/timeout-variants, xargs/parallel x4,
  git restore x3, nested wrappers, sh -c/bash -c, GIT_DIR/GIT_WORK_TREE/env context x4): all DENY.
- r3 substitution corpus (`$(echo git)`, backtick, `eval $()`, `bash -c "$()"`, `eval $(printf)`): all DENY.
- r4 depth (nice*33, nohup*33) + nested escaped backtick: all DENY.
- r5 named (12 substitution/ANSI-C/$VAR/`${x:-}`/`"$(printf)"` forms): all DENY.
- r6 12 reproductions (nested-echo fixpoint forms F1 + `${x:=}/${x:+}/${x+}/${x/a/e}/${x//a/e}/${x:0:5}/${x:-$(echo)}`
  F2 + `$g` command-name): **all 12 now DENY**. No regression.
- **TASK1 RESULT: PASS=65  FAIL=0.**

### TASK 2 — ATTACK the GENERAL fail-closed rule — CONFIRMED STATICALLY-RESOLVABLE REAL-MUTATION BYPASS
Attack thesis: synthesize a denied mutating subcommand WITHOUT leaving a `$`/backtick residual that
has_unresolved_expansion catches. Brace expansion does exactly this.

CONFIRMED LEAKS (guard rc=0 ALLOW; REAL git mutation verified; also confirmed through the real binary):
```
  rc=0  git -C C {reset,} --hard        real reset    (DIRTY->CLEAN)  argv->[reset][--hard]
  rc=0  git -C C {,reset} --hard        real reset    (DIRTY->CLEAN)
  rc=0  git -C C {switch,} other        real switch   (master->other)
  rc=0  git -C C {,switch} other        real switch   (master->other)
  rc=0  git -C C {restore,} .           real restore  (DIRTY->CLEAN)
  rc=0  git -C C {checkout,} other      real checkout (master->other)
  rc=0  git -C C {,checkout} other      real checkout (master->other)
  rc=0  git -C C {clean,} -fdx          real clean    (untracked REMOVED)
  rc=0  git -C C {,clean} -fdx          real clean    (untracked REMOVED)
  rc=0  git -C C {stash,} pop           real stash pop (CLEAN->DIRTY)
  rc=0  {git,} -C C {reset,} --hard     brace command-name + brace subcommand (DIRTY->CLEAN)
```
Airtight: the first four were re-driven straight through `/tmp/r7-guard-bin` with the exact
`{"tool_input":{"command":…}}` JSON payload (the production enforcement surface) — all rc=0, while the
literal equivalents rc=2.

Forms that ALLOW but are FAITHFUL (no real mutation — correctly NOT leaks):
- Partial-prefix braces: `r{eset,}`, `rese{t,}`, `cl{ean,}` -> bash yields TWO words (`reset r`, etc.) ->
  git gets an extra invalid arg -> exit 128, no mutation. (Only the WHOLE-word empty-alternative brace leaks.)
- Two-nonempty braces: `{reset,reset}`, `re{set,set}`, `res{et,et}` -> two `reset` words -> git error, no mut.
- `{reset}` (no comma) -> literal `{reset}` (bash needs a comma/range to expand) -> git error, no mut.
- Quoted/escaped brace `'{reset,}'`, `\{reset,\}` -> literal -> git error, no mut. (No over-broad fix needed
  for the legit-quoted case; bash does not expand a quoted brace.)
- `gi{t,}` command-name -> `git gi` (two words) -> `git gi -C …` rejected, no mut. (Empty-alt command-name
  brace only leaks as the WHOLE word, e.g. `{git,}`.)
- `git -C C{,}`, `git -C {C,}` in -C PATH position -> `target_path_is_dynamic` flags `{`/`}` -> Unknown ->
  fail-closed DENY (rc=2). The PATH side is sound; only the subcommand/command-name side is open.
- history `!!` (non-interactive bash -c: disabled), glob `rese*`/`rese?` (no filesystem match: stays literal)
  -> git error, no mutation.

Boundary checks SOUND (no leak, no false positive): all r5/r6 `$`/`${}`/backtick/ANSI-C forms DENY; `$g`
command-name with a LITERAL non-mutating subcommand (`$g -C C log`) correctly ALLOWs (reads keep args).

### TASK 3 — FALSE-POSITIVE SWEEP (must ALLOW) — PASS, ZERO false positives
- 34/34 legit commands ALLOW: reads with residual-$ args (`git -C C log $BRANCH`, `diff ${BASE:-HEAD}`,
  `show $(git rev-parse HEAD)`, `log ${x}`), `$EDITOR notes.txt`, `g=git; $g -C C log/status`,
  non-canonical + worktree mutations (`reset --hard`/`switch`/`checkout -- f` on noncanon and WT),
  merge-train (`checkout -B dev origin/dev`, `clean -fdx`, `reset --hard origin/dev` on noncanon),
  `ver=$(git rev-parse HEAD)`, `$(date)`, `sudo sh -c 'echo'`, worktree add/list, commit/push/fetch --all/
  merge --ff-only/pull --ff-only on canonical, deep-but-legit nesting.
- /tmp/retrain.sh git lines (11): all ALLOW (ls-remote/merge-base/rev-parse on main checkout; fetch/clean/
  checkout/add/commit/push on the AGENT worktree — none target canonical). 0 false positives.
- **TASK3 RESULT: ALLOW-PASS=34 + retrain 11; FALSE-POSITIVES=0.** The residual-sigil fail-closed rule
  fires only on command-name/subcommand position, so it introduces no new false positives.

### TASK 4 — mechanics — all green
- buck2 //tools/oya-checkout-guard-app:oya-checkout-guard-app-unittest -> **29 passed; 0 failed**
  (the 12 new r6-reproduction fixtures are present and pass — but none cover the brace class, which is
  precisely why this leak survives 29-green: the r1->r7 meta-pattern).
- buck2 //cloud/cloud-ci/gates/oya-cloud-ci-enforcement-liveness-app:{…-gate, …-unittest}
  -> **Pass 2; Fail 0** (gate: 6 passed LIVE; unittest pass).
- SSH sigs: 7ac63d439 + bd40fd0d8 both carry BEGIN SSH SIGNATURE. `%G?`=U is environmental
  (allowed-signers not provisioned in fresh context), same as r1-r6.
- Faces-last settle: 7ac63d439 touches ONLY 2 *.generated.json faces. Fix commit bd40fd0d8 = lib.rs +
  Cargo.lock (Cargo.lock delta is an unrelated oya-lane-supervisor-app entry, no new guard dep; guard dep
  remains serde_json only). Holds.
- Baseline key-diff both ways (merge-base d705932d4 .. HEAD over all 4 changed generated faces):
  **0 anti-pattern keys** (forbidden_foundry|unjustified|unowned|unreachable) added OR removed. No laundering.

---

## Findings (file:line, severity, confidence)

### [HIGH, confidence HIGH] F3-R7 — has_unresolved_expansion is blind to brace expansion (and glob/tilde)
- Location: lib.rs:1270-1285 (has_unresolved_expansion), gating lib.rs:349 (command-name) and
  lib.rs:373-374 (subcommand). Asymmetric with lib.rs:1901-1906 (target_path_is_dynamic) which DOES
  treat `{ } * ? [ ] ~ $ backtick` as dynamic for PATHS.
- Root cause: the r7 fail-closed predicate enumerates only `$`/backtick expansion sigils. A single-element
  empty-alternative brace `{reset,}` / `{,reset}` (likewise switch/restore/checkout/clean/stash, and the
  command-name form `{git,}`) is a statically-resolvable shell expansion bash rewrites to the denied verb;
  the tokenizer (lib.rs:1980-1987 treats `{`/`}` as ordinary word chars) keeps it as a literal word that is
  neither the literal verb nor a `$`/backtick residual, so both the command-name check and the subcommand
  check pass it as "provably safe" and the mutation executes.
- Confirmed bypasses (rc=0; REAL bash MUTATES; also via the real hook binary): `{reset,}`/`{,reset}` --hard,
  `{switch,}`/`{,switch}`, `{restore,}`, `{checkout,}`/`{,checkout}`, `{clean,}`/`{,clean}` -fdx,
  `{stash,} pop`, `{git,} -C C {reset,} --hard`. (10+ forms across 6 mutating verbs.)
- Exploitability: local, unauthenticated, single short one-liner, no length cap; trivially emitted by an
  agent or an injected instruction (the lethal-trifecta surface CLAUDE.md warns of). The brace form is even
  MORE natural than the r5/r6 substitution gymnastics — `{reset,}` reads like an innocent typo.
- Blast radius: silent canonical-checkout reset/switch/restore/checkout/clean/stash, reproducing
  FRIC-022/FRIC-1781062867 while all 29 unit tests + the enforcement-liveness gate stay green.
- STATICALLY RESOLVABLE: brace expansion is deterministic at parse time (the guard already encodes `{`/`}`
  as dynamic in target_path_is_dynamic). Squarely the BLOCK class, not the founder-accepted
  runtime-unknowable `$(prog)` residual.
- Minimal fix direction: extend has_unresolved_expansion (or add a sibling check used at the same two call
  sites) to fail CLOSED when the command-name OR subcommand token contains ANY shell word-altering
  metacharacter not already resolved — minimally `{`, `}`, `*`, `?`, `[`, `]`, `~` in addition to
  `$`/backtick. Reuse the exact metacharacter set already in target_path_is_dynamic (lib.rs:1905) so the
  PATH-side and verb-side closures match. Add a DENY fixture for each of the 10+ confirmed reproductions
  (current fixtures structurally cannot catch them — the recurring r1->r7 failure mode). A precision check:
  quoted/escaped braces must stay ALLOW where the resolved verb is non-mutating; but since the check fires
  only on the still-braced token in verb position and TASK-3 shows reads keep args, the simplest fail-closed
  (DENY any verb-position token bearing an unresolved metacharacter) costs negligible precision.

### Note — this is the r1->r7 meta-pattern recurring (now at the closure-definition level)
Every prior round closed the named corpus and leaked one expansion form past the new fixtures. r7 is subtler:
it ADDED a general fail-closed rule, but defined "unresolved expansion" as the `$`/backtick set rather than
the full shell-metacharacter set. The fix is one layer more general than r6 yet still under-approximates the
true closure. The convergence claim in the commit message ("any expansion the normalizer cannot statically
resolve DENYs") is contradicted by brace expansion.

### Resolved since r6 (verified)
- r6 F1 (nested echo/printf): closed by the bounded fixpoint (lib.rs:1255-1263). All nested-echo forms DENY.
- r6 F2 (`${x:=}/${x:+}/${x+}/${x/a/e}/${x//a/e}/${x:0:5}/${x:-$(echo)}`, `$g` command-name): closed by the
  residual-sigil fail-closed rule (lib.rs:349, 373-374). All DENY. Zero precision cost.

### Positive observations
- The fixpoint + residual-sigil approach is the right SHAPE for a general fail-closed rule and closes all 12
  r6 leaks with zero false positives across 34 legit commands + the retrain.sh set. It just under-specifies
  the metacharacter closure.
- 29 unit + liveness gate (6) green, SSH-signed, faces-last settle holds (2 faces), no key laundering, single
  guard dep (serde_json), no new CVE surface in the delta.

---

## Residual risk (single most likely production failure if merged as-is)
An agent (or an injected instruction via the lethal-trifecta surface) emits a one-liner where the git
mutation verb is produced by a brace expansion the residual check doesn't model — e.g.
`git -C <canonical> {reset,} --hard`, `git -C <canonical> {switch,} main`, or
`git -C <canonical> {clean,} -fdx` — and it silently ALLOWs, re-contaminating the canonical checkout and
reproducing FRIC-022/FRIC-1781062867 while all 29 tests and the liveness gate stay green. `{reset,}` is a
plausible-looking typo/idiom, lowering the bar versus the r5/r6 substitution forms.
Secondary (carried, unchanged): the hook shim fails OPEN if the Rust binary is unbuilt (main.rs:18-21 maps
any error to SUCCESS) — ensure CI/branch-protection builds it (structural enforcement, not hook reliance).

## Required to clear
1. Close F3-R7: make the verb-position (command-name + subcommand) fail-closed check use the FULL shell
   word-altering metacharacter set — at minimum add `{ } * ? [ ] ~` to the `$`/backtick set in
   has_unresolved_expansion, ideally sharing one constant with target_path_is_dynamic (lib.rs:1905) so the
   path-side and verb-side closures are provably identical.
2. Add a DENY fixture for every confirmed reproduction (`{reset,}`/`{,reset}` --hard, `{switch,}`,
   `{restore,}`, `{checkout,}`, `{clean,} -fdx`, `{stash,} pop`, `{git,} -C C {reset,} --hard`) — current
   fixtures cannot catch them, exactly why they leaked past 29-green.
3. Re-run 29 + liveness + new fixtures; re-drive the TASK-2 brace sweep through the real binary; confirm zero
   false positives on the merge-train + retrain.sh set (esp. quoted-brace and partial-prefix-brace
   non-bypasses, and reads keeping residual-$ args).

## Security Checklist
- [x] No hardcoded secrets (lib.rs is pure policy logic; only dep is serde_json; no new guard deps)
- [~] All inputs validated — literal + r5 + r6 forms sound; brace expansion (and glob/tilde) in
      verb/command-name position bypasses the residual check -> 10+ statically-resolvable real-mutation ALLOWs
- [~] Injection prevention — r5/r6 corpus CLOSED; F3-R7 OPEN (brace-expansion canonical-mutation reproductions)
- [x] Authorization/policy enforced for modeled forms; zero false positives across 34 legit + 11 retrain
      commands; path-side canonicalization + brace/glob path-dynamic handling sound
- [x] Dependencies audited — single dep serde_json; no new guard deps; no CVE surface in this delta
- [x] Tests + liveness gate green (29 + gate 6 Pass 2); SSH-signed; faces-last (2 faces); no key laundering
