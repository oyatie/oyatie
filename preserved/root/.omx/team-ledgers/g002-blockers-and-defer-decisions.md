# G002 Blockers and Defer Decisions

Created: 2026-06-26  
Goal: `G002-m0-trunk-and-active-pr-intake`  
Aggregate snapshot: `.omx/ultragoal/checkpoints/get-goal-active-20260626T2310Z.json`  
Collision map: `.omx/team-ledgers/g002-intake-collision-map.md`  
Prune/preserve plan: `.omx/team-ledgers/g002-worktree-prune-preserve-plan.md`  
Guarded prune script: `.omx/team-ledgers/g002-prune-candidates.sh`  
Root-dirt classification: `.omx/team-ledgers/g002-root-dirt-classification.md`

## Current status

G002 has a current non-destructive intake package, but it is **not complete**.

## Completion blockers

1. **Destructive worktree cleanup not authorized/executed**
   - 30 worktrees are clean/merged prune candidates.
   - Dry-run validated they would be removed with `removed=0 skipped=0 mode=--dry-run`.
   - Actual removal requires running `.omx/team-ledgers/g002-prune-candidates.sh --execute` and verifying `git worktree list --porcelain` afterwards.

2. **Preserve/provenance decisions remain**
   - 18 dirty worktrees require owner/provenance checks.
   - 11 clean detached worktrees are not proven in `origin/dev`.
   - 32 clean branch worktrees are not proven in `origin/dev`.

3. **Root checkout dirt remains unresolved**
   - `.codex/hooks.json` is a Lane 1 hygiene candidate.
   - `goal.json` and `slice06-*.log` deletions are cleanup candidates with reference checks.
   - `specs/capability-registry.json` is hot/leader-only governance/spec-like data.
   - `cloud/cloud-intelligence/.omc/` is runtime/tool state drift.

4. **G015 broad Team fanout remains blocked**
   - Use fresh isolated worktrees from current `origin/dev` only.
   - Do not use the dirty root checkout as a Team launch root.

## Safe next choices

### Option A — Destructive cleanup lane
Run the guarded script with `--execute`, then verify every removal by worktree list/readback. This is destructive and requires explicit authorization.

### Option B — Defer cleanup and launch read-only verifier only
Keep all worktrees, record them as deferred blockers, and launch no broad implementation Team. A read-only verifier can classify owners/provenance further.

### Option C — Fresh-worktree narrow implementation lane
Do not clean stale worktrees yet; create a fresh isolated worktree from `origin/dev` for one narrow non-hot lane only. G015 broad fanout still remains blocked.

## Non-decision recorded

This artifact does not choose A/B/C. It records that G002 cannot honestly be marked complete until one of these choices is made and evidenced.
