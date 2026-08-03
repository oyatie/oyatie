# Worker brief — G011 main-checkout guard (FRIC-022/FRIC-1781062867; one worker, one PR)

Frictions (read both rows in `/Users/jasonlee/Developer/oyatie/.omc/ultragoal/friction-ledger.jsonl`): agents twice contaminated the canonical checkout `/Users/jasonlee/Developer/oyatie` (edited files there; checked out a review branch there) despite worktree-only convention. Nothing mechanical prevents it. This lane adds the guard — and establishes the repo's FIRST Rust-hook pattern (founder doctrine: new automation never ships as shell; existing .sh hooks are transitional).

Work ONLY in `/Users/jasonlee/oyatie-worktrees/g011-checkout-guard` (branch `agent/g011-checkout-guard`, base = origin/dev @ 16f2e3b54). Never touch the main checkout (yes, the irony is the point).

## Deliverables (one PR)
1. **Rust decision kernel + binary** `tools/oya-checkout-guard-app`: reads the hook event JSON on stdin (study the existing hooks for the PreToolUse Bash payload shape — command string etc.), decides ALLOW (exit 0) or DENY (exit 2 with a one-line reason naming the worktree policy + this brief's friction ids). Policy v1 (command guard only):
   - DENY when the effective target repo is the canonical checkout path AND the git operation is working-tree-mutating: `checkout <ref>`, `switch`, `reset --hard`, `clean -f*`, `rebase`, `merge` (non-ff), `stash pop/apply`. Effective target = `git -C <path>` argument if present, else session cwd.
   - ALLOW everything else there: fetch, pull --ff-only, log/show/diff/status, worktree add/remove/list, branch -D/-d, push, and ALL operations whose target is outside the canonical path (worktrees).
   - The canonical path comes from an env override `OYA_CANONICAL_CHECKOUT` (default: the repo root the hook runs in IF it is the primary worktree — detect via `git rev-parse --git-common-dir` == `.git`); never hardcode the founder's home path.
   - Decision logic = pure function in the lib with exhaustive unit tests (every verb above, -C vs cwd, worktree paths, ff vs non-ff pull, env override). No unwrap/expect/panic; forbid(unsafe_code).
2. **Hook invocation pattern (the substrate decision — document it in the PR body):** a minimal shim `tools/hooks/main-checkout-guard.sh` whose ONLY job is to exec the Rust binary. Solve binary availability honestly: prefer `exec "$(git rev-parse --show-toplevel)/tools/hooks/bin/oya-checkout-guard"` where `tools/hooks/bin/` holds a symlink/copy refreshed by a `buck2 build ... --out` step; the shim falls back to ALLOW (exit 0) with a stderr warning if the binary is missing (hooks must never brick a session — safety-net layer per enforcement-layering, gates remain canonical). Measure and report invocation latency (must be <100ms warm). If you find a strictly better repo-consistent pattern, use it and justify.
3. **Wiring:** register the shim in BOTH `.claude/settings.json` (PreToolUse Bash matcher) and `.codex/hooks.json` — the enforcement-liveness gate (#669) requires dual wiring and will fail closed otherwise (your dogfood proof: the gate stays green).
4. **Ledger row** appended to the friction ledger marking the shim as irreducible glue on the zero-shell ledger, and FRIC-022/FRIC-1781062867 hooks-layer fix delivered.

## MANDATORY pre-PR adversarial self-review (new dispatch protocol)
Before opening the PR: run a FRESH `codex exec` process with the rubric file `/Users/jasonlee/Developer/oyatie/.omc/ultragoal/RUBRIC-torvalds-review.md` plus your branch name and a pointer to this brief, capture its verdict, FIX all CRITICAL/HIGH findings, and include the final verdict + findings-fixed list in the PR body. The leader runs an independent pass after; your review does not grant merge authority.

## Rules
- buck2 build + buck2 test = green signal; lock refresh ONLY via `cargo metadata >/dev/null`; settle protocol (content commits → git add all → materialize → faces-only settle commit); SSH-signed; push -u origin agent/g011-checkout-guard; PR to dev citing FRIC-022 + FRIC-1781062867 + the rust-hooks doctrine.
- DO NOT wire the hook to deny operations the leader's merge-train legitimately performs (see ALLOW list) — if the guard would have blocked any command pattern visible in this repo's recent reflog, that is a RED design finding to fix before PR.
