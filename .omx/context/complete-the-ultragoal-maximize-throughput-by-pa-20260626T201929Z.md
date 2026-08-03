# Autopilot Context: Complete Ultragoal Through Parallel Closed Loop

Updated: 2026-06-26T20:25:29Z
Activation prompt: `$autopilot Complete the $ultragoal Maximize throughput by parallelization of work. Plan the lanes before fanning out. Entire development life cycle in closed loop. ... $team $team-ledger-orchestrator $ultrawork`
Original task status: activation-prompt with substantial prior same-thread evidence.

## Desired outcome
Complete the active `.omx/ultragoal` plan through the Autopilot loop: deep-interview handoff -> ralplan consensus -> Ultragoal execution (Team where parallel lanes are disjoint and useful) -> code-review -> UltraQA -> PR/CI/merge cleanup. Maximize safe throughput, not speculative work.

## Current evidence
- `.omx/ultragoal/goals.json`: 18 goals; `G001-complete-oyatie-through-small-merge` and `G015-wave-a-m0-m4-team-foundation` in progress; 16 pending; active goal `G015-wave-a-m0-m4-team-foundation`.
- `.omx/ultragoal/brief.md`: small merge-safe waves, isolated worktrees from fresh `origin/dev`, one writer per hot file, never hand-edit `*.generated.json`, Rust + Buck2 authoritative, no Cargo verification authority, retire shell/Python/CLI authority, cloud-ci universal/productized/hermetic/K8s/API-driven, GitHub Actions transitional adapter only.
- `gh pr list --state open` returned no open PRs during preflight; prior worker PRs #908/#909/#910 and prior #903/#904 were verified merged with `oya-ci-required` success.
- `omx team status python-mjs-to-rust-re-3d81f621` previously reported missing; no live team should be resurrected.
- Current checkout is dirty and behind origin/dev; source edits must be done only from fresh isolated worktrees, not this dirty leader checkout.
- Fresh `get_goal` snapshot shows active Codex goal is the user Autopilot prompt, while `.omx/ultragoal/goals.json` expects its stable aggregate objective; Ultragoal completion checkpoints will not reconcile until that mismatch is addressed in a compatible goal context. Do not mutate Codex goal state from hooks/shell.

## Authority chain and constraints
- Read `/specs/root-hub-pointers.json` first; `docs/AGENTS.md` operating contract next.
- Plain `git` + protected PR to `dev`; merge readiness is `oya-ci-required`/cloud-ci required context + review approval.
- `.omc/**` and `.omx/**` are local/provenance for repo authority; current `.omx/ultragoal` is the live workflow state for this session, not canonical repo policy.
- No hand edits to generated JSON; use the materializer/gate path.
- Ponytail rule: delete/fence dead Python/MJS/shell/CLI surfaces; port to Rust only if still live, valid, and worth preserving.
- No new dependencies without explicit evidence and need.
- Keep one writer per hot/shared file; shared workflow/config/generated-policy files are serial lanes.
- Workers own slices end-to-end only after the ralplan consensus gate; leader owns Ultragoal ledger, conflict map, PR queue, and final gates.

## Non-goals
- Do not reopen or reassign stale workers from the missing `python-mjs-to-rust-re-3d81f621` team.
- Do not claim Ultragoal completion while goals remain incomplete or Codex goal reconciliation is false.
- Do not add broad speculative infrastructure, rewrite historical/vendor scripts, or port dead code.
- Do not use local `oya`/legacy CLI output as merge authority.
- Do not edit source in the dirty `dev` checkout.

## Decision boundaries
- Safe autonomous actions: create/update ignored `.omx` planning/state artifacts, fetch remote metadata, inspect docs/registry, create fresh worktrees, prepare team ledger, launch Team after ralplan approval, run local verification, open PRs, follow CI, merge when branch protection allows and approvals/statuses are green.
- Escalate only for destructive cleanup, credential-gated external production actions, conflicting branch-protection authority, or a true product-scope decision not resolvable from root specs/backlog.

## Likely first-wave lane shape for ralplan
1. M0 intake/merge queue/trunk freshness and stale worktree cleanup evidence.
2. M1 local runtime/state hygiene and root scratch cleanup, serializing shared root files.
3. M2 cloud-ci universal API/product boundary, avoiding workflow hot files unless sole owner.
4. M3 generated-artifact merge-surface removal/materializer policy, no hand-generated edits.
5. M4 Rust purity and Python/MJS authority retirement, delete/fence before port.
6. Cross-cutting verification/review/UltraQA lane.

## Stop condition
Autopilot can complete only after ralplan Architect then Critic approve, Ultragoal/Team execution evidence is checkpointed, all active PRs are reviewed/fixed/green/merged or explicitly terminal, code-review returns APPROVE+CLEAR from independent lanes, UltraQA passes or is explicitly skipped with evidence, and fresh `get_goal` + `omx ultragoal status/checkpoint` reconcile.
