# Process edits — agent mistake classes (2026-08-05)

**Definition applied:** *Fix the process* = edit harness / workflows / tools so the next agent cannot repeat the mistake. Not a one-off rebase, chat reminder, or symptom-only patch.

Canonical doc: `programs/hyperscaler-delivery-lanes/BUN-PARALLEL-DISCIPLINE.md`

## Mistake classes → harness changes

| Class | Observed failure | Process edit (file) |
|-------|------------------|---------------------|
| Wrong-tree preflight | Receipts minted on clean tip / dirty primary → merge-check `head_mismatch` | `.grok/bin/preflight-ci-infra` fail-closed when `--pr` and cwd HEAD ≠ `headRefOid`; fail receipt + `process_gate` |
| Passive WAIT | Agents treated `waiting_ci` as idle and stopped driving | `harness/drive.v1.json` class text; `mm-drive` stop-hook anti-passive message; `waiting_ci_requires_armed_repoll` |
| Missed tip-sync | After merge, fleet PRs stayed on stale base (scm-facts/parent red) | `workflows/open-pr-fleet.rhai` mandatory **Tip-sync** phase; merge_policy `post_merge` includes tip-sync |
| Fan-out while trial red | Capacity/path-filter expanded under pressure | `workflows/parallel-delivery-bun.rhai` expand gate; hold job when trial not ready |
| Fuzzy "fix process" | Interpreted as more operational babysitting | `BUN-PARALLEL-DISCIPLINE.md` definition table; `multi-model-roles.json` + `run-grade.v1.json` triggers; CRITIC/EXECUTOR prompts |
| Mid-run push thrash | Push during in_progress killed FULL cone | PREFLIGHT_RULE / PROCESS_RULES in fleet + bun workflows (batch → one push → wait complete) |

## Files touched this edit

- `.grok/bin/preflight-ci-infra`
- `.grok/bin/mm-drive` (stop-hook + briefs language)
- `.grok/workflows/open-pr-fleet.rhai`
- `.grok/workflows/parallel-delivery-bun.rhai`
- `.grok/programs/hyperscaler-delivery-lanes/BUN-PARALLEL-DISCIPLINE.md`
- `.grok/multi-model-roles.json`
- `.grok/harness/drive.v1.json`
- `.grok/harness/rubrics/run-grade.v1.json`
- `.grok/harness/DRIVE.md`
- `.grok/README.md`

## Verification

```text
# From origin/dev tip worktree against PR 1569 head → must exit 1
.grok/bin/preflight-ci-infra --pr 1569
# ok:false, process_gate: cwd_must_equal_pr_head
```

## Not done here

- Committing `.grok/` into the monorepo (still largely untracked except ignored mm-runs/memory) — optional follow-up so all worktrees inherit without copy.
- Wiring a hard `wait_ci_blocks_stop: true` default (would thrash); kept message + armed-repoll policy instead.
