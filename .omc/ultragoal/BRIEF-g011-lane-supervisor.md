# Worker brief — G011 lane supervisor (FRIC-1781110000; one worker, one PR)

Friction (read the row in `/Users/jasonlee/Developer/oyatie/.omc/ultragoal/friction-ledger.jsonl`, id `FRIC-1781110000`): a headless `codex exec` lane worker died silently at a context-compaction boundary, leaving a dirty worktree and no PR; nothing detected it — the dispatch ledger records dispatch and PR-open only, so a dead worker is indistinguishable from a slow one. Founder doctrine: staleness/liveness failures are process failures needing mechanical detectors; new automation never ships as shell.

Work ONLY in a worktree you create: `git -C /Users/jasonlee/Developer/oyatie worktree add /Users/jasonlee/oyatie-worktrees/g011-lane-supervisor -b agent/g011-lane-supervisor origin/dev` (fetch first; base = current origin/dev). NEVER touch the main checkout working tree.

## Deliverables (one PR)
1. **Rust tool** `tools/oya-lane-supervisor-app` (single-concern per ADR-0132; local-bridge CLI, retirement-marked like all CLI per repo policy — zero merge authority; the durable home for lane orchestration is the cloud-ci substrate, say so in the crate docs):
   - `dispatch --brief <file> --worktree <path> --branch <name> [--log <path>]`: spawn `codex exec --dangerously-bypass-approvals-and-sandbox -C <worktree> <prompt-from-brief-pointer>` detached, capture child PID, start timestamp, log path; append a `dispatched` row to `.omc/ultragoal/dispatch-ledger.jsonl` (schema-compatible with existing rows — read the file first; never rewrite existing rows, append-only).
   - `reap`: for each ledger lane whose latest row is non-terminal, check process liveness (kill -0 semantics via sysinfo or /proc-free approach portable to macOS), log-file mtime, and `gh pr list --head <branch>` presence; append `exited` (with exit status if obtainable from a wait-file written by a small wrapper), `stalled` (alive but log mtime older than --stall-minutes, default 30), or `dead` (no process, no PR) rows. Exit 1 if any lane is dead/stalled (gate-hookable).
   - `status`: human/JSON summary of every lane's latest state.
   - Decision/parse logic = pure functions in the lib with exhaustive unit tests (ledger row parsing with unknown fields preserved, terminal-state lattice, stall thresholding with injected clock — NO Date::now in lib logic, clock is a trait). No unwrap/expect/panic in production paths; `#![forbid(unsafe_code)]`; BUCK + manifest hygiene + registry catalog entry like sibling tools/oya-*-app crates (study `tools/oya-buck-test-wiring-app` or `tools/oya-checkout-guard-app` on origin branches / existing tools for conventions).
2. **Brief-template amendment:** add a "commit-early discipline" section to `.omc/ultragoal/TEAMMATE-PREAMBLE.md`: workers MUST commit compiling WIP before any long build/test phase and immediately after each self-review fix round (compaction-death containment).
3. **Ledger row** appended marking FRIC-1781110000 fix-delivered (tool layer; gate wiring is a follow-on once cloud-ci owns lane state).

## Rules
- buck2 build + buck2 test = the green signal; cargo supplementary only; lock refresh ONLY via `cargo metadata >/dev/null`.
- SETTLE PROTOCOL (mandatory): all content commits FIRST → `git add` everything → run `infra/ci/materialize-cloud-ci-generated-faces.sh .` → FACES-ONLY settle commit. Never hand-edit `*.generated.json`.
- MANDATORY pre-PR adversarial self-review: fresh `codex exec` with `/Users/jasonlee/Developer/oyatie/.omc/ultragoal/RUBRIC-torvalds-review.md` + your branch + this brief; fix all CRITICAL/HIGH; include verdict + findings-fixed in the PR body. The leader runs an independent pass after.
- SSH-signed commits; push -u origin agent/g011-lane-supervisor; PR to dev citing FRIC-1781110000 + rust-tools doctrine. Final output line: `PR_OPENED: <number>` or `BLOCKED: <reason>`.
