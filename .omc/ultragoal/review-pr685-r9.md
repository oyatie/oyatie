# Security Review — PR #685 ROUND 9 (G011 canonical-checkout guard)

- Repo: jason931225/oyatie  Branch: agent/g011-checkout-guard  Head: c6023aac7  Base: dev
- r9 delta vs r8 (201ce78dc..c6023aac7): two commits.
  - ebdce2d7c `fix(checkout-guard): fail-closed on IFS word-split resplit` — lib.rs ONLY
    (+32/-7, no Cargo.lock churn). Adds `same_line_ifs_reassigned()` (lib.rs:1334) and
    threads an `ifs_unsafe` flag through `expand_with_bindings`; when a same-line `IFS`/`IFS=…`
    word is present, ALL value-producing expansions (`${}`, `$(...)`, `$name`, backtick) are
    SUPPRESSED via `.filter(|_| !ifs_unsafe)` so the residual-sigil (verb) and dynamic-path
    (target) fail-closed rules fire on both sides. ANSI-C `$'…'` still resolves. Plus 7 new IFS
    DENY fixtures (lib.rs:2192-2199).
  - c6023aac7 `chore: settle generated cloud-ci faces` — 2 *.generated.json faces ONLY.
- Reviewer: fresh-context security-reviewer (Claude Opus), Torvalds/attacker lens,
  /using-superpowers + /using-agent-skills + security-and-hardening + /ultraqa, all FOREGROUND,
  built from /Users/jasonlee/oyatie-worktrees/g011-checkout-guard.

## VERDICT: BLOCK

**Risk Level: HIGH.** The r9 IFS fix is correct and complete *for the IFS class*: it closes
EVERY one of the 7 r8 IFS reproductions (all DENY, rc=2) with the *general* "any IFS
reassignment → fail closed" approach (not per-char), and it does so with ZERO false positives —
including the critical legit `IFS=, read …` cases, which stay ALLOW because the suppressed
`$name` only matters in arg position on a read. That part of the convergence claim is sound.

BUT the r9 commit's CONVERGENCE claim — "the only remaining bypass is a runtime-unknowable
`$(prog)` output" — is **FALSE**. The IFS suppression is a per-mechanism patch, not a model fix,
so it leaves OTHER statically-resolvable word-synthesis axes wide open. I constructed
**THREE distinct statically-resolvable real-mutation bypass classes**, each verified with an
argv-capturing fakegit and an end-to-end DIRTY→CLEAN proof through the PRODUCTION binary, and
NONE uses IFS, a covered metacharacter, or a runtime-unknowable `$(prog)`:

1. **Bare positional `$@`** after a literal `set -- reset --hard` (F1-R9). The strongest finding:
   `$@` word-splits to multiple words in BOTH bash and zsh (no `bash -c` wrapper needed — more
   robust than the r8 IFS bypass, which zsh didn't split). Guard ALLOWs (rc=0), canonical
   `reset --hard` / `clean -fdx` / `restore .` really runs.
2. **ANSI-C hex/octal/unicode escapes** `$'\x72\x65\x73\x65\x74'` → `reset` (verb) and
   `$'\x2d\x2dhard'` → `--hard` (flag) (F2-R9). The r9 premise "ANSI-C `$'…'` still resolves"
   is only HALF true: the decoder handles `\n`/`\t` and plain chars but drops the backslash on
   `\xNN`/`\NNN`/`\uNNNN`, so `\x72`→`x72`. Bash decodes all of them. Guard ALLOWs; canonical
   mutates.
3. **Backslash line-continuation** `re\<newline>set` → `reset` (F3-R9). The guard has no
   line-continuation joining anywhere; bash joins it. Guard ALLOWs; canonical mutates.

All three resolve fully from the command string at parse time (literal `set --`, literal
`$'\xNN'`, literal `\`+newline) — they are NOT the founder-accepted runtime-unknowable
`$(prog)` residual. Per the strict VERDICT RULE — "BLOCK on a statically-resolvable
real-mutation bypass" — this blocks.

Root cause is the same r1→r9 meta-pattern at the next layer: each round closes the *named*
corpus with a fixture-shaped patch and leaks the next token-synthesis axis the new fixtures
don't exercise (r5 subst, r6 param-op, r7 brace, r8 IFS, r9 → positional `$@` / ANSI-C hex /
line-continuation). The true closure is "every way bash turns a token into a different word";
r9 modeled the IFS axis but left ≥3 orthogonal axes unmodeled.

---

## Harness (commands + exact exit codes)

Built the real binary: `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out
/tmp/r9-guard-bin` → BUILD SUCCEEDED (Mach-O arm64). Driven via JSON hook payload
`{"tool_input":{"command":"…"}}` on stdin (the real enforcement surface, main.rs run()),
`OYA_CANONICAL_CHECKOUT=<canon>`, `env -i` neutral env. rc=2 ⇒ DENY, rc=0 ⇒ ALLOW.
Calibration verified: `git -C <canon> {reset --hard|switch|restore .|checkout -- f|clean -fdx|
stash pop}` → 2; `{status|fetch|log}` → 0.

**Adjudication discipline (real-mutation only counts):** per candidate, (1) build a fresh DIRTY
canonical (` M tracked.txt` + `?? untracked.txt` + extra branch); (2) record guard rc via the
real binary; (3) run the EXACT command in a clean `bash -c` subshell with REAL git; (4) diff
`git status --porcelain` before/after. A FINDING requires `guard_rc=0 AND mutated=yes`. An
argv-capture fakegit on PATH confirmed the exact words bash hands git.

### TASK 1 — full accumulated corpus (r1..r8 named + ALL reproductions incl 7 r8 IFS) — PASS
45-command corpus (transparent/unmodelled wrappers flock/runuser/cpulimit/timeout/nice/nohup/
xargs/parallel/env, sh -c/bash -c, GIT_DIR/GIT_WORK_TREE context; r3 `$(echo git)`/backtick/
`eval $()`/`bash -c "$()"`; r5 ANSI-C/$VAR/`${x:-}`/`"$(printf)"`; r6 `${x:=}/${x:+}/${x+}/
${x/a/e}/${x//a/e}/${x:0:5}/${x:-$(echo)}`+`$g` command-name; all 9 r7 brace forms; all 7 r8
IFS forms `IFS=x;y=resetx;…$y`/`IFS=-`/`IFS=z…cleanz`/`IFS=w…restorew`/`${y}`-wrapped/`bash -c`
wrapper/path-side `IFS=x;p=<canon>x;-C $p`):
- **PASS=45  FAIL=0.** ALL DENY (rc=2). Every r8 IFS reproduction now DENYs through the
  production binary — the r8 BLOCK finding (F4-R8) is RESOLVED.

### TASK 2 — CONVERGENCE TEST — THREE STATICALLY-RESOLVABLE REAL-MUTATION BYPASSES (BLOCK)
Enumerated every word-synthesis mechanism the prompt named; adjudicated each vs REAL bash via
the production binary.

CONFIRMED LEAKS (guard_rc=0 ALLOW; REAL canonical mutation; via production binary):
```
  rc=0 mut=yes  set -- reset --hard; git -C <canon> $@          DIRTY->CLEAN (reset --hard ran)
  rc=0 mut=yes  set -- clean -fdx;  git -C <canon> $@          untracked DESTROYED (clean ran)
  rc=0 mut=yes  set -- restore .;   git -C <canon> $@          DIRTY->CLEAN (restore ran)
  rc=0 mut=yes  f(){ git -C <canon> $@; }; f reset --hard      positionals via fn args
  rc=0 mut=yes  set -- reset --hard <NL> git -C <canon> $@     newline-separated outer set
  rc=0 mut=yes  bash -c 'set -- reset --hard; git -C <canon> $@'   shell-agnostic wrapper
  rc=0 mut=yes  sh   -c 'set -- reset --hard; git -C <canon> $@'   shell-agnostic wrapper
  rc=0 mut=yes  git -C <canon> $'\x72\x65\x73\x65\x74' --hard   ANSI-C hex VERB -> reset
  rc=0 mut=yes  git -C <canon> reset $'\x2d\x2dhard'            ANSI-C hex FLAG -> --hard
  rc=0 mut=yes  bash -c "git -C <canon> $'\x72…' --hard"        ANSI-C hex, wrapper
  rc=0 mut=yes  git -C <canon> re\<newline>set --hard          backslash line-continuation
```
Argv-capture fakegit confirms bash hands git `[-C] [<canon>] [reset] [--hard]` in each case.

End-to-end proof (production binary), `$@` class, DIRTY→guard ALLOW→CLEAN:
```
  BEFORE:  M tracked.txt | ?? untracked.txt
  GUARD rc=0 (ALLOW)   cmd: set -- reset --hard; git -C <canon> $@
  AFTER:  ?? untracked.txt          (the ` M` is gone — reset --hard executed)
  ---
  BEFORE untracked: ?? untracked.txt
  GUARD rc=0          cmd: set -- clean -fdx; git -C <canon> $@
  AFTER untracked:  (empty — untracked DESTROYED)
```

Special-parameter sigil-gap map (`set -- reset --hard; git -C <canon> $X`):
```
  $@ -> rc=0 (LEAK, expands positionals)   $* -> rc=2 (incidentally caught by `*` metachar)
  $# -> rc=0 (count, not weaponizable)     $0 -> rc=2 (alnum)   $1 -> rc=2 (alnum)
  $- -> rc=0 (flags, not weaponizable)     $? -> rc=2           $_ -> rc=2
  $! -> rc=0 (pid, not weaponizable)
```
`$@` is the weaponizable gap: `has_unresolved_expansion` (lib.rs:1287-1293) only treats `$`
followed by `(`/`{`/`'`/alnum/`_` as a sigil — `@` is none, and `@` is not in the word-altering
metachar set (lib.rs:1284). `expand_with_bindings` (lib.rs:1414) only consumes `$`+alpha/`_`,
so `$@` is emitted verbatim and survives as a literal non-mutating "verb" token. The guard never
models `set --`, so the split `reset --hard` is invisible.

Mechanisms that ALLOW but are FAITHFUL (no real mutation — correctly NOT leaks):
- env-prefix `IFS=x y=resetx git -C <canon> $y` → env IFS doesn't alter the *current* shell's
  word-splitting; `$y` stays one word; no mutation (and guard DENYs anyway — fail-closed).
- `mapfile … ${arr[0]}` here it didn't expand (process-substitution timing) → no mutation.
- `${@}`/`${1}` brace-wrapped → DENY (braces are metachars).

Mechanisms correctly DENIED (real verb synthesis): bare numeric `$1`/`$2` (alnum sigil), `$*`
(`*` metachar), all 8 IFS variants incl `export IFS=`, `local IFS=`, `printf -v IFS`,
`declare IFS=`, IFS via outer newline scope, `read`-set IFS; `$'reset'` simple ANSI-C; comment
`# tail` after a real mutation; tab/newline-in-value with DEFAULT IFS (`y=$'reset\t--hard'`).

RESIDUAL genuinely runtime-unknowable (`$(prog)` opaque stdout) remains accepted — BUT it is NO
LONGER the sole residual: F1/F2/F3-R9 are all parse-time-determinable and BLOCK.
Carried/observed (NOT counted as the static-leak block class): a PATH/path-relative symlink or
alias `g`→git (`g -C <canon> reset --hard`) ALLOWs and mutates, but the guard would need
filesystem symlink resolution at parse time to know `g` is git — arguably the same
runtime-unknowable family; pre-existing, unchanged this round, noted not blocking on its own.

### TASK 3 — FALSE-POSITIVE SWEEP (must ALLOW) — PASS, ZERO false positives
- 30/30 legit ALLOW: merge-train on canonical (`merge/pull --ff-only`, `commit`, `push`,
  `fetch --all`, `status`, `log`), reads with metachar/var ARGS (`log {origin/dev,HEAD}`,
  `show *.rs`, `diff HEAD~{1,2}`, `log $REF`, `diff ${BASE:-HEAD}`, `show $(git rev-parse HEAD)`,
  `log ~/notes`), for-loops, `$(date)`, `$EDITOR`, `sudo sh -c 'echo'`, worktree mutations on
  NON-canonical incl `{reset,} --hard`, `worktree list/add`, full merge-train on non-canonical.
- **CRITICAL — IFS suppression does NOT over-deny real `IFS=…` reads:** `IFS=, read -r a b c
  <<< x,y,z`, `IFS=, read -ra parts …; git -C <canon> log`, `while IFS=, read -r k v; do … done`,
  `IFS=$'\n' read -d '' -ra L < f; git status`, and `IFS=, read …; git log $sha` all ALLOW
  (rc=0). The suppression only makes `$REF`/`$sha` stay literal in ARG position, which never
  over-denies a read. 5 aggressive over-deny probes: 0 FP.
- /tmp/retrain.sh (present) — 10/10 git lines ALLOW (ls-remote/fetch/clean/checkout/add/commit/
  push/rev-parse against worktree+main checkout; none target the calibrated canonical).
- **TASK3 RESULT: ALLOW-PASS=30 + 5 over-deny probes + 10 retrain = 0 FALSE POSITIVES.**

### TASK 4 — mechanics — all green
- `//tools/oya-checkout-guard-app:oya-checkout-guard-app-unittest` → **30 passed; 0 failed**
  (7 new IFS DENY fixtures present and pass). NOTE: NO fixture exercises `$@`/ANSI-C-hex/
  line-continuation — the recurring r1→r9 reason the leak survives 30-green.
- `//cloud/cloud-ci/gates/oya-cloud-ci-enforcement-liveness-app:{…-unittest,…-gate}` →
  unittest Pass 1; gate **6/6 tests pass** ("current tree green").
- SSH sigs: ebdce2d7c + c6023aac7 both carry BEGIN SSH SIGNATURE. `%G?`=U is environmental
  (allowed-signers not provisioned in fresh context), same as r1-r8.
- Faces-last settle: c6023aac7 touches ONLY 2 *.generated.json faces (accounting-registry,
  scm-facts). Fix commit ebdce2d7c = lib.rs ONLY (no Cargo.lock churn). Holds.
- Baseline key-diff both ways (merge-base d705932d4 .. HEAD over all 4 changed generated faces):
  **0 anti-pattern keys** (forbidden_foundry|unjustified|unowned|unreachable) added OR removed.
  No laundering.

---

## Findings (file:line, severity, confidence)

### [HIGH, confidence HIGH] F1-R9 — bare positional `$@` synthesizes a denied verb the guard cannot see (static, no IFS, no metachar, no $(prog))
- Location: lib.rs:1287-1293 `has_unresolved_expansion` (the `$` sigil branch matches only
  `$`+`(`/`{`/`'`/alnum/`_` — `@` is excluded and is not in the lib.rs:1284 metachar set);
  compounded at lib.rs:1414 `expand_with_bindings` (only consumes `$`+alpha/`_`, so `$@` is
  emitted verbatim). The guard never models `set -- <words>` positional assignment.
- Confirmed (rc=0; REAL bash MUTATES; production binary): `set -- reset --hard; git -C <canon>
  $@`; `set -- clean -fdx; … $@`; `set -- restore .; … $@`; `f(){ git -C <canon> $@; }; f reset
  --hard`; newline-separated outer `set --`; `bash -c '…$@'` and `sh -c '…$@'` wrappers.
- Exploitability: local, unauthenticated, single short one-liner, no length cap. SHELL-AGNOSTIC
  WITHOUT a wrapper — `$@` word-splits in both bash and zsh (strictly worse than the r8 IFS
  bypass, which zsh did not split). Trivially emitted by an agent or an injected instruction
  (the lethal-trifecta surface CLAUDE.md warns of).
- Blast radius: silent canonical `reset --hard` / `clean -fdx` / `restore .`, reproducing
  FRIC-022/FRIC-1781062867 while all 30 unit tests + the liveness gate stay green.
- STATICALLY RESOLVABLE: `set -- reset --hard` is a literal positional assignment, fully visible
  at parse time. NOT the runtime-unknowable `$(prog)` residual — the BLOCK class.

### [HIGH, confidence HIGH] F2-R9 — ANSI-C hex/octal/unicode escapes are not decoded; bash decodes them (static, no $(prog))
- Location: lib.rs:1369-1373 — the `$'…'` decoder maps `\n`→NL, `\t`→TAB, and `other`→the literal
  char after the backslash. So `\x72`→`x72`, `\47`→`47`, `r`→`u0072`. Bash fully decodes
  `\xNN`/`\NNN`/`\uNNNN`/`\UNNNNNNNN`.
- Confirmed (rc=0; REAL bash MUTATES; production binary): `git -C <canon> $'\x72\x65\x73\x65\x74'
  --hard` (hex VERB → `reset`); `git -C <canon> reset $'\x2d\x2dhard'` (hex FLAG → `--hard`, which
  flips a non-blocked `reset` into a blocked `reset --hard`); `bash -c` wrapper variant.
  Cross-check: simple `$'reset'` DENYs (decoded), hex `$'\x72…'` ALLOWs — isolating the decoder gap.
- Exploitability: local, unauthenticated, single one-liner, shell-agnostic via `bash -c`.
  The r9 premise "ANSI-C `$'…'` still resolves" is only half-true and is exactly the gap.
- Blast radius: same as F1-R9 (silent canonical reset/clean/restore).
- STATICALLY RESOLVABLE: `$'\xNN'` is a literal constant; its value is fixed at parse time.

### [MEDIUM, confidence HIGH] F3-R9 — backslash line-continuation joins a split verb the guard never joins (static, no $(prog))
- Location: no line-continuation handling exists anywhere (grep for `\<newline>` joining in
  shell_tokens / expand_with_bindings returns nothing). lib.rs:1357-1362 pushes `\`+next char
  verbatim; the tokenizer treats `re\` and `set` as separate words.
- Confirmed (rc=0; REAL bash MUTATES; production binary): `git -C <canon> re\<newline>set --hard`
  → bash joins to `reset --hard` (fakegit shows `[reset] [--hard]`). Guard sees `re`+`set` → no
  mutating verb → ALLOW.
- Exploitability: local, unauthenticated; a literal backslash+newline in the command string.
- Blast radius: same canonical-mutation class. (Severity MEDIUM only because a raw newline in the
  middle of a token is slightly more conspicuous than `$@`/`$'\x..'`; impact identical.)
- STATICALLY RESOLVABLE: `\`+newline is a pure lexical construct, determinable at parse time.

### Note — the r1→r9 meta-pattern, now at three more word-synthesis axes
r9 correctly and generally closed the IFS axis (any `IFS` reassignment → fail closed — not
per-char, exactly as claimed, with zero false positives even on legit `IFS=, read`). But the
convergence claim treats "IFS modeled" as "static closure reached," which is false: bash has
multiple orthogonal token-synthesis axes, and r9 left at least three open — positional
parameters (`set --`/`$@`), ANSI-C numeric escapes (`$'\xNN'`), and lexical line-continuation
(`\`+NL). The durable fix is a model change (tokenize like bash: positional params, full
ANSI-C decode, line-continuation joining, then fail-closed on any unresolved/dynamic verb or
`-C` target), not another per-mechanism character/keyword patch that the next round's fixtures
won't exercise.

### Resolved since r8 (verified)
- F4-R8 (IFS word-split resplit, verb + path side): CLOSED by `same_line_ifs_reassigned` +
  expansion suppression at lib.rs:1334/1392/1403/1427/1439. All 7 r8 IFS reproductions DENY.
  Zero precision cost (30 legit + 5 IFS over-deny probes + 10 retrain all ALLOW, incl real
  `IFS=, read`). The "any IFS reassignment → fail closed" generalization is the right shape
  for that axis.

### Positive observations
- IFS fix is general (not per-char), correct, and precision-clean — the right direction for
  its axis; the legit `IFS=, read` cases are preserved.
- 30 unit + liveness gate green (6/6), SSH-signed, faces-last settle holds (2 faces, lib.rs-only
  fix, no Cargo.lock churn), no key laundering, single guard dep (serde_json), no new CVE
  surface in the delta.

---

## Residual risk (single most likely production failure if merged as-is)
An agent (or an injected instruction via the lethal-trifecta surface) emits
`bash -c 'set -- reset --hard; git -C <canonical> $@'` — a statically-resolvable form with NO
IFS, NO covered metacharacter, and NO runtime-unknowable `$(prog)` — and it silently ALLOWs,
re-contaminating the canonical checkout and reproducing FRIC-022/FRIC-1781062867 while all 30
tests and the liveness gate stay green. `$@` is strictly worse than the r8 IFS bypass: it
word-splits in BOTH bash and zsh, so it needs no `bash -c` wrapper to be shell-agnostic. The
ANSI-C-hex (`$'\x72…'`) and line-continuation (`re\<NL>set`) forms are independent static paths
to the same mutation. Because three orthogonal axes leak, the fix must model bash tokenization
(positional params + full ANSI-C decode + line-continuation join, then fail-closed), not extend
another character/keyword set.

Secondary (carried, unchanged): the hook shim fails OPEN if the Rust binary is unbuilt (main.rs
maps any error → SUCCESS); ensure CI/branch-protection builds it (structural enforcement, not
hook reliance). Carried: bare `git stash` (save) and PATH-relative alias/symlink `g`→git are not
modeled — the latter arguably needs filesystem resolution (runtime-unknowable family);
pre-existing, out of scope, noted not blocking on their own.

## Required to clear
1. Close F1-R9: model bash positional parameters — recognize `set -- <words>` (and function
   args) and fail-closed (or expand) `$@`/`$*`/`$1`… so a synthesized mutating verb/`-C` target
   is caught. At minimum, add `@`/`#`/`-`/`!`/`*` to the `$`-sigil trigger in
   `has_unresolved_expansion` so any special-parameter expansion in verb/-C/subcommand position
   fails closed.
2. Close F2-R9: decode `\xNN`/`\NNN`/`\0NN`/`\uNNNN`/`\UNNNNNNNN` in the `$'…'` handler (lib.rs
   :1369) exactly as bash does — or, if full decode is undesirable, fail-closed when an `$'…'`
   body contains an undecoded numeric escape that feeds a git verb/-C/subcommand.
3. Close F3-R9: join backslash-newline line-continuations before tokenizing (a one-line
   preprocessing step), so `re\<NL>set` is seen as `reset`.
4. Add a DENY fixture for EVERY confirmed reproduction (the `$@` verb + `$@` clean/restore + the
   `bash -c '…$@'` wrapper + the function-arg `$@` + ANSI-C-hex verb + ANSI-C-hex flag + the
   line-continuation form). Current fixtures structurally cannot catch them — the recurring r1→r9
   failure mode.
5. Re-run 30 + liveness + new fixtures; re-drive the TASK-2 sweep through the real binary;
   confirm zero false positives on the merge-train + retrain.sh + `IFS=, read` set.
6. Re-state the convergence claim honestly only after auditing ALL bash word-synthesis axes
   (positional, ANSI-C numeric, line-continuation, extglob, `{1..9}` brace-range, history if
   ever enabled) — not just IFS — and confirm the sole residual is runtime-unknowable `$(prog)`.

## Security Checklist
- [x] No hardcoded secrets (lib.rs is pure policy logic; only dep is serde_json; no new deps)
- [~] All inputs validated — IFS axis now sound; positional `$@`, ANSI-C numeric escapes, and
      line-continuation bypass static analysis → statically-resolvable real-mutation ALLOWs
- [~] Injection prevention — r8 IFS class CLOSED; F1/F2/F3-R9 OPEN (positional/ANSI-C-hex/
      line-continuation canonical-mutation)
- [x] Authorization/policy enforced for modeled forms; zero false positives across 30 legit + 5
      IFS over-deny probes + 10 retrain commands; IFS suppression does not over-deny real reads
- [x] Dependencies audited — single dep serde_json; no new guard deps; no CVE surface in delta
- [x] Tests + liveness gate green (30 unit + gate 6/6 Pass 1); SSH-signed; faces-last (2 faces,
      lib.rs-only fix, no Cargo.lock churn); no key laundering
