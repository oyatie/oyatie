# DELTA reviewer of record — PR #685 ROUND 4 (G011 main-checkout guard)

- Repo: jason931225/oyatie  Branch: agent/g011-checkout-guard  Head: dff604060  Base: dev
- Fix under review: 69467777b "fix(checkout-guard): preserve env/quote context in default-closed
  recursion (review #685 r3)" (+ settle dff604060). Content delta vs r3 head (096b3a2e8):
  1 file, +60/-16 on tools/oya-checkout-guard-app/src/lib.rs (the rest is generated faces).
- Reviewer: fresh-context DELTA (Claude Opus), attacker/Torvalds lens, /using-superpowers +
  /using-agent-skills, /oh-my-claudecode:ultraqa, all FOREGROUND. r1/r2/r3 all BLOCK.

## VERDICT: **BLOCK**

The r4 fix is correct and complete for what it claimed: BOTH r3 HIGH leaks are closed across unmodelled
AND modeled wrappers. `<wrapper> sh -c '<mut>'` and `<wrapper> [env] GIT_DIR=<canon>/.git git <mut>`
now DENY for firejail/flock/cpulimit/runuser/systemd-run AND for the modeled sudo/nohup/nice/setsid set.
The `requote_word` round-trip is sound: embedded single quotes, `$`, separators inside an sh -c body, and
GIT_DIR env-context all survive re-tokenisation and DENY. The `eval` bare-join was correctly left
unrequoted — I empirically proved (fake-git argv capture) that real `eval` concatenate-then-reparse
genuinely destroys the quoting too, so the ALLOW is faithful, not a hole. Zero false positives on the
full merge-train + retrain.sh + wrapped-tool set. 28 tests + liveness gate green, LSP clean, both commits
SSH-signed, faces-last settle, no key laundering.

BUT the r4 commit raised MAX_NESTED_COMMAND_DEPTH 4->32 to "not exhaust the depth budget" — and left the
exhaustion behavior **fail-OPEN**. Every recursion site is `if depth < MAX { recurse }` with NO else and
NO deny on exhaustion, so the 33rd nested wrapper level is never evaluated and falls through to
Decision::Allow. A 33-deep wrapper chain (107 chars minimum: `x x ... x33 git -C <canon> reset --hard`)
silently ALLOWs a canonical mutation. This is the SAME meta-pattern as r1/r2/r3: the named corpus is
closed, the structural property leaks one layer deeper — here, one recursion level past the new ceiling.
A second, lower-severity bypass: nested escaped backticks evade the substitution scanner.

---

## Harness (commands + exact exit codes)

Built the real binary via `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out
/tmp/r4-guard-bin` (BUILD SUCCEEDED; Mach-O arm64; sanity DENY=2). Drove it with JSON hook payloads on
stdin, `OYA_CANONICAL_CHECKOUT=/private/tmp/r4-canon` (realpath-resolved; macOS /tmp->/private/tmp),
neutral session cwd `/private/tmp/r4-neutral`. rc=2 => DENY, rc=0 => ALLOW. Calibrated: bare
`git -C <C> {switch|reset --hard|restore .|checkout}` -> rc=2; `git -C <C> {status|fetch|log}` -> rc=0.

Several ALLOW results were adjudicated against REAL-SHELL semantics using a fake `git`/`firejail`/`evil`
on PATH that logs argv, to distinguish a true guard bypass (mutation executes against canonical) from a
faithful model of a shell construct that is itself a no-op/syntax-error.

### TASK 1 — full accumulated corpus + the two r3 leaks (all must DENY)
```
rc=2  DENY   firejail|eatmydata|proxychains|catchsegv|busybox|systemd-run git -C <C> switch foo
rc=2  DENY   flock /tmp/l git -C <C> switch foo ; runuser -u u -- git -C <C> switch foo
rc=2  DENY   cpulimit -l 50 git -C <C> switch foo ; catchsegv git -C <C> reset --hard HEAD
rc=2  DENY   xargs -a file git -C <C> checkout ; xargs -P 4 -n 1 git -C <C> checkout   (F4-R2)
rc=2  DENY   echo x | xargs -I{} git -C <C> checkout {}
rc=2  DENY   git -C <C> restore {. , --staged . , --worktree src/lib.rs}               (F2)
rc=2  DENY   firejail flock /tmp/l systemd-run git -C <C> switch foo   (nested unmodelled)
rc=2  DENY   timeout 5 firejail git -C <C> switch foo ; nohup firejail git -C <C> reset --hard HEAD
--- r3 F1: <wrapper> sh -c '<mut>' (NOW DENY) ---
rc=2  DENY   firejail sh -c 'git -C <C> reset --hard'
rc=2  DENY   sudo sh -c 'git -C <C> reset --hard'
rc=2  DENY   nohup sh -c 'git -C <C> reset --hard'
rc=2  DENY   nice -n5 sh -c 'git -C <C> reset --hard'
rc=2  DENY   flock /tmp/l sh -c 'git -C <C> reset --hard HEAD'
rc=2  DENY   cpulimit -l 50 bash -c 'git -C <C> checkout foo'
rc=2  DENY   runuser -u u -- sh -c 'git -C <C> reset --hard'
rc=2  DENY   systemd-run sh -c 'git -C <C> switch foo'
--- r3 F2: <wrapper> [env] GIT_DIR=<canon>/.git git <mut> (NOW DENY) ---
rc=2  DENY   firejail GIT_DIR=<C>/.git git reset --hard
rc=2  DENY   firejail GIT_DIR=<C>/.git GIT_WORK_TREE=<C> git reset --hard
rc=2  DENY   flock /tmp/l env GIT_DIR=<C>/.git git reset --hard
rc=2  DENY   cpulimit -l 50 GIT_DIR=<C>/.git git checkout foo
```
**TASK 1 PASS — entire accumulated corpus closed; both r3 HIGH leaks fixed (unmodelled + modeled).**

### TASK 2 — attack requote_word + full-remainder recursion

2(a) requote round-trip (embedded quotes / mixed quoting):
```
rc=2  DENY   firejail sh -c 'git -C <C> reset --hard ; : '\''its'\'''   (embedded '\'' -> real shell runs git; guard DENY)
rc=2  DENY   firejail sh -c 'git -C <C> reset --hard '\''HEAD'\'''      (embedded quote, real git runs; DENY)
rc=2  DENY   firejail sh -c 'sh -c "git -C <C> reset --hard"'           (nested sh -c)
rc=2  DENY   firejail sh -c 'true; git -C <C> reset --hard'             (; inside body stays in body)
rc=2  DENY   firejail sh -c 'echo x | git -C <C> reset --hard'          (| inside body)
rc=2  DENY   firejail sh -cx 'git -C <C> reset --hard' ; bash -xc '...' (attached -c flag clusters)
rc=2  DENY   firejail sh -c 'git -C <C> reset --hard $x'                ($ -> single-quoted, faithful)
rc=0  ALLOW  firejail sh -c 'git -C <C> reset --hard'\''; echo'    => real shell: body has UNBALANCED quote, sh syntax-errors, git never runs. Faithful ALLOW (verified w/ fake git).
rc=0  ALLOW  firejail sh -c 'git -C <C> commit -m "x"'             => `commit` is NOT a blocked op (lib.rs:1483-1500); correct.
```
Verdict 2(a): requote_word is SOUND. Every payload whose body actually executes a canonical mutation in
a real shell DENYs; the two ALLOWs are faithful models of shell no-ops (syntax error / non-mutating
subcommand), proven with argv-capturing fake git. No false negative in the embedded-quote class.

2(b) safe-charset breakout — the safe set `-_/=.:,@+%` + alnum contains NONE of the tokenizer's word
delimiters (whitespace, `; | & ( ) \n \r '"' \`), so a bare safe word cannot introduce a new word
boundary on re-tokenise. Confirmed:
```
rc=2  DENY   firejail GIT_DIR=<C>/.git git reset --hard          ('=','/','.' all safe -> one word, faithful)
rc=0  ALLOW  firejail GIT_DIR=<C>/.git;evil git reset --hard     => the ';' is an OUTER separator; real shell runs `evil git reset --hard` with NO GIT_DIR in env -> no canonical mutation (verified: evil ran git as plain arg). Faithful ALLOW.
```
Verdict 2(b): no safe-charset breakout. SAFE.

2(c) DEPTH (limit 32) — **BYPASS FOUND (F1-R4)**. Recursion is fail-OPEN past the ceiling:
```
rc=2  DENY   x*30 git -C <C> reset --hard
rc=2  DENY   x*31 git -C <C> reset --hard
rc=2  DENY   x*32 git -C <C> reset --hard
rc=0  ALLOW  x*33 git -C <C> reset --hard          *** BYPASS (F1-R4) — 107-char single command ***
rc=0  ALLOW  nice*33 git -C <C> reset --hard       *** modeled pass-through wrapper ***
rc=0  ALLOW  nohup*33 git -C <C> reset --hard      *** modeled ***
rc=0  ALLOW  sudo*33 git -C <C> reset --hard       *** privilege wrapper ***
rc=0  ALLOW  (nohup sudo nice setsid)*10 git -C <C> reset --hard   *** mixed realistic chain ***
```
- `nice`*5 in a real shell DOES exec `git -C /canon reset --hard` (fake-git argv=[-C /canon reset --hard]),
  so these chains genuinely execute the mutation; the guard ALLOWs them.
- `env`*40 still DENYs — env uses the non-recursive `env_prefix` state machine (no depth consumed); only
  the wrapper-recursion paths exhaust the budget.
- Combining substitution + wrappers does NOT lower the wrapper count (git is checked inline at the frame
  it lands in); threshold is stable at 33 raw wrapper levels.
- NO command-length / token cap exists anywhere (grep'd) — the 107-char bypass is fully reachable.

2(d) eval bare-join — **SAFE (verified, not a hole)**:
```
rc=0  ALLOW  eval sh -c 'git -C <C> reset --hard'      (+ bash/dash/zsh, + eval firejail/nohup sh -c)
rc=2  DENY   eval git -C <C> reset --hard              (positive control: bare git in eval still DENY)
rc=2  DENY   eval GIT_DIR=<C>/.git git reset --hard ; eval env GIT_DIR=<C>/.git git reset --hard
```
Real-shell proof (fake git): `eval sh -c 'git -C /canon reset --hard'` -> git ran with argv=[] (i.e.
`sh -c git` with reset/--hard as positional params to the inner sh) — the mutation NEVER reaches git.
Control `sh -c 'git ...'` (no eval) -> git ran with argv=[-C /canon reset --hard] and the guard DENYs it.
So leaving eval unrequoted is correct: eval's concatenate-then-reparse destroys the quoting in reality
exactly as the guard models. CONFIRMED SAFE.

2(e) command-substitution / backtick:
```
rc=2  DENY   echo $(git -C <C> reset --hard) ; echo `git -C <C> reset --hard`
rc=2  DENY   printf %s $(git -C <C> switch foo) ; : $(git -C <C> checkout foo) ; x=$(git -C <C> reset --hard)
rc=2  DENY   echo $(firejail git -C <C> reset --hard) ; echo $(sh -c "git -C <C> reset --hard")
rc=2  DENY   true $(eval git -C <C> reset --hard) ; echo $(echo $(git -C <C> reset --hard))   (nested $())
rc=2  DENY   firejail sh -c "echo $(git -C <C> reset --hard)"
rc=0  ALLOW  echo `echo \`git -C <C> reset --hard\``   *** BYPASS (F2-R4) nested escaped backticks ***
```
Real-shell proof (fake git, zsh AND bash): the nested-backtick payload runs `git -C /canon reset --hard`
(argv=[-C /canon reset --hard]) — a REAL canonical mutation the guard ALLOWs.

### TASK 3 — FALSE-POSITIVE SWEEP (zero false positives)
```
rc=0  ALLOW  git -C <C> {status,log,diff,show HEAD,branch -a,rev-parse HEAD,ls-remote origin,merge-base origin/dev HEAD}
rc=0  ALLOW  git -C <C> {fetch --all --prune, push origin dev, commit -m, add -A, merge --ff-only, pull --ff-only}
rc=0  ALLOW  git checkout -q -B review-branch origin/dev ; git fetch -q origin ; git clean -qfd ; git checkout -q -- .
rc=0  ALLOW  git add -A ; git commit -q -m retrain ; git push -qf origin review-branch   (worktree-side)
rc=0  ALLOW  gh pr merge 685 --squash ; buck2 run //...:...face-settle-bin -- --settle --commit
rc=0  ALLOW  buck2 build //tools/... ; cargo metadata --format-version 1 ; infra/ci/materialize-faces.sh
rc=0  ALLOW  cat /tmp/retrain-files.txt | xargs git checkout origin/dev --
rc=0  ALLOW  git worktree {add /tmp/wt review-branch, remove /tmp/wt, list}
rc=0  ALLOW  sudo sh -c 'echo hello' ; firejail ./build.sh ; flock /tmp/lock ./retrain.sh 685 br wt ; systemd-run buck2 build //tools/...
rc=0  ALLOW  git -C <neutral> reset --hard ; firejail git reset --hard   (non-canonical target)
```
**Zero false positives.** The fix preserves precision exactly. Legit commands nest at depth 0-4 — a
huge margin below the 32 ceiling, so fixing the depth fail-open to fail-CLOSED would have ZERO
false-positive risk (no legitimate command approaches 33 wrappers).

### TASK 4 — suites / hygiene
- buck2 test checkout-guard unittest -> **28 passed; 0 failed** (8 new r3-leak assertions folded into
  `denies_unmodelled_wrapper_prefixed_mutations_default_closed`; new fixtures lines 1781-1791 cover
  exactly the F1/F2 shapes — but NONE assert a deep nest; the deepest new fixture is 3 wrappers).
- buck2 test enforcement-liveness gate (`...-gate`, tests/enforcement_liveness.rs) -> **6 passed; 0
  failed** (`enforcement_liveness_face_reports_current_tree_green` — gate LIVE).
- lsp_diagnostics lib.rs -> **clean**.
- SSH sigs: 69467777b + dff604060 both carry `BEGIN SSH SIGNATURE` (ED25519). `%G?`=`U` is environmental
  (allowed-signers not provisioned in fresh context).
- Faces-last settle: 69467777b touches ONLY lib.rs; dff604060 touches ONLY `*.generated.json`. Holds.
- Baseline key-diff both ways: merge-base 8801af778. Across merge-base..HEAD over `*.generated.json`,
  **0 anti-pattern keys** (forbidden_foundry/unjustified/unowned/unreachable) added OR removed. Face churn
  is benign provenance (row_count 18201->18209 for new test rows, source_inputs_digest, last_touch_commit
  pointers). No laundering.

---

## Findings (file:line, severity, confidence)

### [HIGH, confidence HIGH] F1-R4 — depth-exhaustion is fail-OPEN; 33-deep wrapper nest bypasses
- **Root cause: lib.rs:8 (`MAX_NESTED_COMMAND_DEPTH = 32`) + every recursion site
  (lib.rs:50, 175, 195, 264, 310, 333, 383, 409).** Each is `if depth < MAX_NESTED_COMMAND_DEPTH {
  recurse... }` with NO `else` and NO `Decision::Deny` on exhaustion. When the wrapper chain reaches the
  33rd level, the nested `decide_with_context` call is skipped and control falls through to
  `command_position = false` and finally `Decision::Allow` (lib.rs:425). The r4 commit raised the limit
  4->32 to "not exhaust the depth budget" but a higher fail-open ceiling does not close the class — it
  just moves the wall.
- **Confirmed bypass (rc=0 / ALLOW, expected DENY):** `x*33 git -C <canon> reset --hard` (107 chars),
  `nice*33 ...`, `nohup*33 ...`, `sudo*33 ...`, `(nohup sudo nice setsid)*10 ...`. Boundary is exact:
  32 wrappers DENY, 33 ALLOW. `nice`-chains were proven to actually exec the mutation in a real shell.
- **Exploitability:** local, no privileges, single 107-char command line. There is NO command-length or
  token cap anywhere, so the chain is trivially constructible by an agent or an injected instruction.
  Fail-open means a missing/over-long nest disables the guard rather than denying.
- **Blast radius:** identical to F1/F2-R3 — silent canonical-checkout mutation reproducing
  FRIC-022/FRIC-1781062867, while 28 tests + liveness gate stay green (no deep-nest fixture exists).
- **Minimal fix (zero false-positive risk):** make depth exhaustion fail CLOSED. At every recursion site,
  when `depth >= MAX_NESTED_COMMAND_DEPTH` and a nested command exists, return `Decision::Deny` (or hoist
  a single guard: if the depth cap is hit with remaining un-evaluated nested command, deny). Legit
  commands nest <=4, so denying at 32 has no precision cost. The limit then becomes a true wall regardless
  of its value, instead of a soft fail-open ceiling.

### [MEDIUM, confidence HIGH] F2-R4 — nested escaped backticks evade the substitution scanner
- **lib.rs:1248-1260 (`extract_command_substitutions`, backtick branch).** The backtick handler locates
  the closing backtick with a plain `command[start..].find('`')` and does NOT account for escaped `\``
  inside the body. For `` `echo \`git -C <canon> reset --hard\`` ``, it extracts only `echo \` (up to the
  first inner backtick) and never recurses on the real `git ... reset --hard`. The `$()` branch
  (`extract_balanced_dollar_command`, lib.rs:1266-1289) tracks `\\` correctly, so the equivalent nested
  `$()` form DENYs — the gap is backtick-specific.
- **Confirmed bypass (rc=0 / ALLOW, expected DENY):** `` echo `echo \`git -C <canon> reset --hard\`` ``.
  Proven in BOTH zsh and bash (fake git argv=[-C /canon reset --hard]) that the inner mutation executes
  against canonical. Single-level backtick and nested `$()` both correctly DENY.
- **Exploitability:** local, no privileges, but requires the deliberate nested-backtick `\`` escaping
  idiom — an unusual, deprecated construct unlikely to appear organically (hence MEDIUM, not HIGH). Still
  attacker-constructible.
- **Blast radius:** silent canonical mutation, same class as the substitution leaks.
- **Minimal fix:** in the backtick branch, skip backslash-escaped backticks when searching for the close
  (or normalise `\`` and recurse), mirroring the escape-aware logic already in
  `extract_balanced_dollar_command`. Add a fixture asserting DENY for the nested-backtick form.

### Note — both findings are the r1->r4 meta-pattern recurring
The named corpus (F1/F2-R3) is genuinely and well closed. Both residuals are the structural property
leaking one layer past where the new fixtures assert: depth one level past the ceiling (no deep-nest
fixture), and the one substitution form whose escape handling diverges from the `$()` path (no
nested-backtick fixture). Same shape as r1/r2/r3: fix the enumerated cases, the property leaks deeper.

### Resolved since r3 (verified)
- **F1-R3 `<wrapper> sh -c '<mut>'` — FIXED** for unmodelled (firejail/flock/cpulimit/runuser/systemd-run)
  AND modeled (sudo/nohup/nice/setsid) wrappers, via `requote_word` in `unmodelled_wrapper_remainder`
  (lib.rs:458-470) and `skip_flag_args_and_join` (lib.rs:1135-1141).
- **F2-R3 `<wrapper> [env] GIT_DIR=<canon> git <mut>` — FIXED** by recursing on the FULL re-quoted
  remainder (not a git-token-anchored slice), preserving env/GIT_DIR context.
- requote_word embedded-quote/separator/`$` round-trip — verified faithful (no false negative).
- eval bare-join — verified SAFE against real-shell semantics (not a hole).
- Full r1/r2 corpus, git restore, xargs separate-token flags — all remain DENY.

### Positive observations (reinforce)
- The requote design is the right primitive and the docstrings (lib.rs:448-488) are accurate and honest
  about the prior unsoundness. The `eval` carve-out is correct and the reasoning checks out empirically.
- Zero false positives on reads, the full merge-train, retrain.sh, wrapped tools, and worktree ops.
- Hygiene clean: 28 + 6 green, LSP clean, both commits SSH-signed, faces-last settle, no key laundering.

---

## Residual risk (single most likely production failure if merged as-is)
An agent (or an injected instruction via the lethal-trifecta surface CLAUDE.md warns about) emits a
33+-deep wrapper chain — e.g. `nice nice ... (x33) git -C <canonical> reset --hard` or
`sudo sudo ... git -C <canonical> checkout <branch>` — and it silently ALLOWs, re-contaminating the
canonical checkout and reproducing FRIC-022/FRIC-1781062867 while all 28 tests and the liveness gate stay
green (no deep-nest fixture exists to catch it). Secondary: the nested-escaped-backtick form bypasses the
substitution scanner. Tertiary (carried from r3, unchanged): the hook shim fails OPEN if the Rust binary
is unbuilt; ensure CI/branch-protection builds it (structural enforcement, not hook reliance).

## Required to clear
1. Make depth exhaustion fail CLOSED (close F1-R4): at the recursion sites, when the depth cap is reached
   with a nested command still to evaluate, return `Decision::Deny`. Zero false-positive risk (legit
   nesting <=4). This makes MAX_NESTED_COMMAND_DEPTH a real wall instead of a fail-open ceiling.
2. Fix the backtick escape gap (close F2-R4): make the backtick scanner in `extract_command_substitutions`
   escape-aware (skip `\``), matching `extract_balanced_dollar_command`.
3. Add fixtures asserting DENY for: a >MAX deep wrapper nest (e.g. 33x a wrapper + canonical mut) AND a
   nested escaped-backtick canonical mutation. The current fixtures structurally cannot catch either —
   exactly why both leaked past green (the r1->r4 pattern).
4. Re-run 28 + 6 + new fixtures; re-drive the Task-2 depth + backtick sweeps; confirm zero false
   positives on the merge-train + retrain.sh set.

VERDICT: **BLOCK** — one HIGH (depth-exhaustion fail-open; 33-deep wrapper nest mutates canonical) and
one MEDIUM (nested escaped backticks evade the substitution scanner) remain. Strong, genuine progress:
both r3 HIGH leaks are fully closed across unmodelled AND modeled wrappers, requote_word is sound, the
eval carve-out is verified safe, and precision is intact with zero false positives. The fix is one
structural change away (fail-closed-on-exhaustion) from closing the depth class for good.

## Security Checklist
- [x] No hardcoded secrets (none introduced; lib.rs is pure policy logic)
- [x] All inputs validated (shell command parsed via tokenizer; requote round-trip verified sound)
- [~] Injection prevention verified — git-mutation-injection class: F1/F2-R3 CLOSED; F1-R4 depth + F2-R4
      backtick remain OPEN
- [x] Authorization/policy enforced — default-closed scan-through, env/GIT_DIR/-C/sh-c machinery correct
- [x] Dependencies audited — single dep serde_json; no new deps; no CVE surface in this delta
- [x] Tests + liveness gate green; LSP clean; SSH-signed; faces-last; no key laundering
