# G018 closed-loop synthesis draft

Status: waiting on parallel lanes (critic, architect, verifier, planner).

Known evidence already collected:
- G015 checkpoint complete: `.omx/context/g015-wave-a-m0-m4/g015-checkpoint-complete-20260627T0040Z.json`.
- Final PR evidence: `.omx/context/g015-wave-a-m0-m4/final-pr-evidence-20260627T0035Z.json`.
- G001 aggregate remains active/in_progress; no `update_goal` called.
- `origin/dev`: `a9c24f757880e2290281b6a674ff045a0cd8e62f`.
- Open PRs to dev: 0.
- Cleanup completed for #917/#918/temp #915 worktrees; G015 leader worktree preserved due non-empty divergent obsolete diffs.
- Post-merge dev `oya-ci-required` run `28273027942` progressed from pending/queued to in_progress.

Preliminary next-wave principle:
- Do not launch implementation lanes from dirty main checkout.
- Launch only fresh isolated worktrees from `origin/dev` after dev run either passes or any failures are classified/fixed.
- Single-writer hot paths: workflow/CI top-level files, generated-artifact control-plane manifests, language-discipline registry, root-hub/master-plan specs.
