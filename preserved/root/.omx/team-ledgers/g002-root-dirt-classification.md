# G002 Root Dirt Classification

Created: 2026-06-26  
Goal: `G002-m0-trunk-and-active-pr-intake`  
Raw evidence: `.omx/context/g002-intake/root-dirt-raw-20260626T2306Z.txt`  
Secondary evidence: `.omx/context/g002-intake/root-dirt-secondary-20260626T2308Z.txt`  
Codex goal snapshot: `.omx/ultragoal/checkpoints/get-goal-active-20260626T2306Z.json`

## Summary

The root checkout remains dirty and behind `origin/dev` by 202 commits. This artifact classifies dirt only; it does not modify source files.

## Classification table

| Surface | Evidence | Classification | Safe next action |
| --- | --- | --- | --- |
| `.codex/hooks.json` | Diff removes three top-level metadata keys and leaves only `hooks`; `python3 -m json.tool` passes. | Likely valid Codex strict-schema repair, but it is repo-local config drift. | Preserve as a dedicated Lane 1 hygiene candidate; do not let broad Team workers touch it. Validate with Codex hook parser before committing. |
| `goal.json` | Tracked in HEAD, deleted locally, size in HEAD 17238 bytes. | Root scratch/goal artifact candidate for deletion, but tracked. | Preserve deletion as cleanup candidate; verify references before commit. |
| `slice06-*.log` | Eight tracked root log files deleted locally; sizes range 1120..81397 bytes. | Root scratch/log artifacts; strong cleanup candidates. | Preserve deletions as cleanup candidates; verify references before commit. |
| `specs/capability-registry.json` | Untracked valid JSON, 27135 bytes; self-identifies as governance data for ADR-0562. | Hot governance/spec-like surface; not scratch. | Preserve, leader-only. Determine whether it belongs in a fresh worktree/PR before any Team fanout. |
| `cloud/cloud-intelligence/.omc/` | Untracked runtime/tool directory under source tree; secondary tree evidence captured. | Runtime/tool state drift. | Fence/ignore or remove only after owner check; no broad worker writes. |

## Reference check

Secondary evidence includes a bounded `rg` check for root `goal.json` / `slice06-*` references outside those files. Treat results as advisory because the root checkout is 202 commits behind and broad repo scans may include historical references.

## Admission impact

- G015 broad Team fanout remains blocked from root checkout.
- Fresh worker lanes must use fresh isolated worktrees from `origin/dev`.
- Lane 1 may own root hygiene only after this dirt is rebound explicitly.
- `specs/capability-registry.json` is hot/leader-only until provenance is resolved.

## Completion impact

G002 should not be checkpointed complete yet because:

1. 30 clean/merged worktree prune candidates remain unexecuted;
2. 61 worktrees require preserve/provenance decisions;
3. root checkout dirt is classified but not resolved;
4. the active Codex aggregate goal remains `active`, not `complete`.
