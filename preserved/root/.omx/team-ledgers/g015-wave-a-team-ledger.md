# G015 Pre-Fanout Team Ledger — Wave A M0-M4 Foundation

Created: 2026-06-26  
Wrapper goal: `G015-wave-a-m0-m4-team-foundation`  
Precondition goal: `G002-m0-trunk-and-active-pr-intake`  
Collision map: `.omx/team-ledgers/g002-intake-collision-map.md`  
Plan source: `.omx/plans/ralplan-complete-ultragoal-through-team-ledger-full-replan.md`

## Launch status

**Do not launch yet.**

G015 is a Team wrapper, not the next execution unit. It can launch only after G002 closes the current collision risks:

- root checkout behind `origin/dev` by 202 commits;
- pre-existing dirty/untracked files;
- 92 registered worktrees;
- no accepted stale-worktree prune/preserve ledger yet.

## Required launch inputs

Before Team launch, the leader must record:

- fresh `git fetch --prune origin dev` evidence;
- fresh `git status --short --branch` evidence from a clean launch root or fresh worktree;
- updated `gh pr list --base dev --state open` evidence;
- stale worktree classification/prune-preserve ledger;
- worker ownership matrix with no hot-file overlaps;
- fresh `get_goal` snapshot showing the aggregate Codex goal is active;
- generated-file policy reminder: no `*.generated.json` hand edits.

## Proposed worker lanes after G002

| Worker | Role | Owned paths/surfaces | Avoid paths/surfaces | Acceptance | Verification |
| --- | --- | --- | --- | --- | --- |
| W1 | executor/debugger | Repo hygiene/runtime-state boundaries: root scratch/logs, runtime-state drift, hook config evidence only | `.omx/ultragoal/**`, generated files, CI workflows, root authority specs | Dirt classified or reduced with reference checks | `git status`, `rg/git grep` references, targeted syntax checks |
| W2 | executor/test-engineer | Universal cloud-ci product boundary under non-hot `cloud/**` / `crates/**` seams | `.github/workflows/**`, `oya-ci.toml`, generated artifacts | One small product-boundary improvement or no-code evidence ledger | Targeted Buck2/Rust tests; no Cargo-only authority |
| W3 | executor/verifier | Generated-artifact conflict controls: generators/materializers/drift-check code | `*.generated.json`, generated face outputs | Conflict surface shrinks or evidence ledger proves no safe change | Materialization/drift check; prove no generated output hand edit |
| W4 | executor/dependency-expert | Rust purity / Python-MJS authority inventory and retirement candidates | historical/vendor-only scripts unless revalidated; hot policy files | One live authority retired/fenced or precise keep/delete inventory | Inventory before/after, Buck2 where touched, ponytail delete-first evidence |
| W5 | verifier/code-reviewer/architect | Verification matrix, review evidence, G012 production-bar ledger, UltraQA prep | Implementation except test/evidence artifacts | Review-ready evidence exists before merge attempts | Independent review, UltraQA scenarios, fresh `get_goal` |

## Shared shutdown gate

Team may shut down only when:

1. every assigned task is closed;
2. every PR has `oya-ci-required` green or is explicitly blocked;
3. review threads are resolved;
4. no worker owns a shared/hot file;
5. generated-file policy is clean;
6. leader has a fresh `get_goal` snapshot;
7. leader records the durable Ultragoal checkpoint or blocker.

## Current no-launch verdict

G015 remains blocked for broad fanout until G002 completes stale worktree classification/prune-preserve decisions. If throughput is needed before destructive cleanup approval, use one read-only verifier lane only, scoped to worktree classification with no deletion and no source writes.

## G002 completion update (2026-06-26T23:24Z)

G002 intake/fanout gating checkpoint was accepted by OMX.

Evidence:

- Checkpoint output: `.omx/context/g002-intake/g002-checkpoint-complete-20260626T2324Z.json`
- Post-prune summary: `.omx/team-ledgers/g002-post-prune-summary.md`
- Final classification: `.omx/context/g002-intake/final-worktree-classification-20260626T2322Z.json`
- CI evidence: `.omx/context/g002-intake/origin-dev-checks-20260626T2317Z.txt`

Updated launch rule:

- G015 may launch only from fresh isolated worktrees based on current `origin/dev`.
- Do not use the dirty root checkout.
- Do not use preserved dirty or detached worktrees.
- Worker branches must be newly named and disjoint from preserved local branches.
- Hot/serial surfaces remain leader-owned.
