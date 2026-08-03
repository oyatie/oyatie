# Fable review of record — PR #685 (G011 main-checkout guard)

- Repo: jason931225/oyatie  Branch: agent/g011-checkout-guard  Head: 826e4c153  Base: dev @ 152e7cdd
- Reviewer: fresh-context Fable (Claude Opus), Torvalds-lens + hyperscaler-lens rubric, attacker lens
- Harness: /using-superpowers + /using-agent-skills + /oh-my-claudecode:ultraqa, all FOREGROUND
- Worker self-review: NONE (codex died on quota before its self-review). This verdict gates the merge.

## VERDICT: **BLOCK**

One HIGH-severity bypass class defeats the policy's central intent. Everything else (intent match,
ALLOW-list fidelity, wiring, evidence, signatures, latency, repo doctrine) is clean and genuinely
high quality. The block is narrow and fixable; recommend fix-then-re-review, not redesign.

---

## What I verified by running it (commands + exact outputs)

1. **buck2 unit tests** (from /Users/jasonlee/oyatie-worktrees/g011-checkout-guard):
   `buck2 test //tools/oya-checkout-guard-app:oya-checkout-guard-app-unittest`
   -> `test result: ok. 24 passed; 0 failed`  (Pass 1). All 24 cited tests EXIST and assert what
   they claim (byte-equal DENY_REASON incl. both friction ids; not `.contains` laundering). No
   phantom-test problem (cf. PR #644).

2. **buck2 enforcement-liveness dogfood gate**:
   `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-enforcement-liveness-app/...`
   -> `test result: ok. 6 passed; 0 failed`. Dual-wiring census 11->12 hooks, 18->20 command refs
   matches the two added wiring lines. Gate stays GREEN (the dogfood proof holds).

3. **Built the real binary** `buck2 build //tools/oya-checkout-guard-app:oya-checkout-guard --out`
   -> BUILD SUCCEEDED. Drove it with crafted Claude/Codex JSON payloads on stdin.

4. **Default canonical detection in the REAL tree** (no env override):
   - from the lane worktree: common-dir=/Users/jasonlee/Developer/oyatie/.git -> mutating op against
     canonical = exit 2 (DENY); against the worktree itself = exit 0 (ALLOW). Correct.
   - from the main checkout: common-dir=.git -> bare `git switch foo` = exit 2 (DENY). Correct.

5. **Shim fail-open**: binary absent -> stderr warning + exit 0 (ALLOW). Binary present + canonical
   target -> the policy reason on stderr + exit 2 (DENY). Hooks never brick a session. Correct.

6. **Warm latency** of the real binary (50 iters, worst-case nested command):
   `p50=1.91ms p95=2.29ms max=2.49ms` — far under the <100ms budget. (Shim adds one `git rev-parse`
   + exec; still well within budget.)

7. **ALLOW-list fidelity (the brief's anti-requirement: never block the leader's legit ops)** — all
   ALLOW: `fetch`, `pull --ff-only`, `merge --ff-only`, `worktree add/remove/list`, `push`,
   `branch -D/-d`, `status/log/diff/show`, and ALL worktree-targeted mutations. ZERO false positives
   observed. The merge-train is not impeded.

8. **No-panic / unsafe**: only `panic!` is at lib.rs:1492 inside `#[cfg(test)]`. Production code has
   no unwrap/expect/panic; both files carry `#![forbid(unsafe_code)]`. LSP diagnostics: clean on both.

9. **Signatures**: all 24 commits `%G? = G` (Good ED25519). Settle protocol followed (content `fix:`
   commits then `chore: settle ... faces`). Evidence/audit-chain.jsonl is append-only (one new line).
   gate-baseline.generated.json diff is producer-mechanical & purely additive (no debt laundered or
   dropped — verified key-diff both directions). ADR-0523 glue ledger row added (5->6) and correctly
   describes fail-open exec-only shim. Catalog yaml + changelog present.

---

## Findings

### [HIGH, confidence HIGH] F1 — Wrapper-command prefix bypass defeats the policy
- **lib.rs:325-375 (the `for` token loop) + :375 `command_position = false`.** When the FIRST word
  in command position is anything the parser does not recognize as a known wrapper (`env`, `command`,
  `exec`, `builtin`, `eval`, `cd/pushd/popd`, sh/bash/dash/zsh, `function`, a reserved word, or
  `git`), the loop falls through to `command_position = false` and stops inspecting the rest of the
  simple command. Any command-runner wrapper that takes a command as its argument therefore passes a
  blocked git op straight through. Confirmed against the built binary (canonical=/repo/oyatie):

  ```
  ALLOW  nohup git -C /repo/oyatie switch foo            (expected DENY)
  ALLOW  nice git -C /repo/oyatie switch foo             (expected DENY)
  ALLOW  timeout 5 git -C /repo/oyatie reset --hard HEAD (expected DENY)
  ALLOW  stdbuf -oL git -C /repo/oyatie switch foo       (expected DENY)
  ALLOW  setsid git -C /repo/oyatie switch foo           (expected DENY)
  ALLOW  echo x | xargs git -C /repo/oyatie checkout     (expected DENY)
  ALLOW  echo x | xargs -I{} git -C /repo/oyatie checkout {}   (expected DENY)
  ALLOW  watch git -C /repo/oyatie switch foo            (expected DENY)
  ALLOW  parallel git -C /repo/oyatie checkout ::: foo   (expected DENY)
  ALLOW  ls git -C /repo/oyatie switch foo               (expected DENY)
  ```
  Survives subshells `(nohup git ...)`, pipes `echo foo | timeout 5 git ... checkout`, `&&`
  (`true && nohup git ...`), leading env-assigns (`A=1 B=2 nohup git ...`), and `;` chaining.
- **Why it matters:** the rubric is explicit — "a check that can be evaded by the exact input class
  it polices" is a finding, and "every bypass you find that the ALLOW-list policy intends to block =
  HIGH." `timeout`/`xargs`/`nohup` are ordinary commands an agent (or a review critic — the exact
  FRIC-1781062867 actor) types routinely; this is not an exotic evasion. The 24 tests never cover a
  command-runner wrapper, so the green suite gives false assurance against precisely the policy's
  core promise. The worker's commit series closed env-prefix/subshell/cd-chain/embedded-alias
  bypasses but missed the simplest one: an unrecognized leading wrapper word.
- **Minimal fix:** treat the unrecognized-leading-word case as "fail-closed scan-through" rather than
  "stop scanning." Concretely: maintain an allowlist of known transparent wrappers
  (`nohup nice timeout stdbuf setsid ionice chronic xargs watch parallel time env command exec
  builtin sudo doas`) whose trailing tokens form a nested command, and recurse `decide_with_context`
  on the remainder (as already done for `env`/`command`/sh wrappers). Safer still given the open-
  ended wrapper space: when the leading command word is unknown AND a later in-position word is a
  `git` invocation targeting the canonical checkout, DENY (scan-through default-closed) instead of
  short-circuiting at `command_position = false`. Either way, add fixtures for the rows above.

### [MEDIUM, confidence HIGH] F2 — `git restore` (and `git restore --staged`/`--worktree`) not in policy
- **lib.rs:1249-1263 `is_blocked_operation`.** Policy v1 blocks `checkout <ref>` and
  `checkout -- <file>` (args non-empty) but NOT `restore`. Confirmed: `git -C /repo/oyatie restore .`
  -> ALLOW, while the equivalent `git -C /repo/oyatie checkout -- somefile` -> DENY.
- **Why it matters:** `git restore` is the modern, increasingly-default equivalent of
  `git checkout -- <path>` — it overwrites/discards uncommitted working-tree changes. The friction
  rows are about working-tree contamination and *loss*; allowing `restore` leaves a working-tree-
  destroying verb open while its legacy twin is closed — an internal inconsistency, not just a gap.
- **Minimal fix:** add `"restore" => true` (or scope to `--worktree`/default which mutates the
  working tree; `--staged`-only touches the index) plus a fixture. The brief's list is "v1"; restore
  belongs in it on the same rationale as `checkout -- file`.

### [LOW, confidence MEDIUM] F3 — Command-substitution-resolved git binary path bypass
- **lib.rs:325 `command_basename_is(word, "git")`.** `"$(command -v git)" -C /repo/oyatie switch foo`
  -> ALLOW: the leading word is an unresolved `$(...)` placeholder so it is not recognized as git,
  and the *inner* substitution (`command -v git`) is itself non-mutating, so recursion finds nothing.
- **Why it matters:** real but contrived; an agent is far more likely to type `git`/`nohup git` than
  `$(command -v git)`. Folds naturally into the F1 scan-through fix (unknown leading word -> default
  closed when a canonical-targeted git appears downstream). Tracking, not blocking on its own.

### Positive observations (reinforce)
- Genuinely strong adversarial engineering: tokenizer with quote/escape handling, recursive descent
  into `env -S`, `sh -c`, `eval`, command substitutions, git aliases (`-c alias.x`, `--config-env`,
  `GIT_CONFIG_COUNT/KEY/VALUE`), `--git-dir`/`--work-tree` worktree disambiguation, cd/pushd/popd cwd
  tracking with subshell stack, dynamic-path fail-closed. The 24 tests are real and assert byte-equal.
- Fail-open shim + fail-closed decision kernel is the correct split per enforcement-layering doctrine.
- First Rust-hook pattern is clean: pure decision fn in lib, thin main, BUCK lib/bin/test triad,
  forbid(unsafe_code), no production panics. Hyperscaler lens: policy-in-tested-code + shell-as-glue
  is the right call and is justified in the ADR-0523 ledger row. Owned-architecture lens: the
  decision is a pure function over an event struct — no transient-dependency idiosyncrasy leaks into a
  trait; clean cutover shape.

---

## Residual risk (single most likely production failure even if merged as-is)
An agent or review critic running a perfectly ordinary `timeout 600 git -C <canonical> reset --hard`,
`nohup git -C <canonical> switch <branch>`, or `... | xargs git -C <canonical> checkout` silently
ALLOWs and re-contaminates the canonical checkout — reproducing FRIC-022/FRIC-1781062867 verbatim
while the guard reports itself live and green. The guard's own test suite would still pass, so the
regression is invisible until it blocks a dev fast-forward again. Close F1 before merge.
