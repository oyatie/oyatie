# Context Snapshot: cloud-intelligence remaining work autopilot

## Task seed
Finish remaining Oyatie cloud-intelligence work under `$autopilot` while preserving prior constraints: no inherited external project name outside provenance/source metadata, cloud-native only, adapters own translation/security transient integrations, and work must remain isolated from dev.

## Current evidence
- Main checkout is dev and behind origin/dev; implementation work is isolated in an existing worktree branch.
- Existing foundation PR: https://github.com/jason931225/oyatie/pull/644
- PR #644 head branch: agent/cloud-intelligence-xproxy-20260610
- PR #644 was previously reported green with oya-ci-required success and contains Slice 0/foundation parity work.
- Prior user consensus: foundations first, then one workflow end-to-end in each PR until covered.

## Required contracts
- Read `/specs/root-hub-pointers.json` first; treat `docs/AGENTS.md` as operating contract.
- Use sanctioned project primitives for repo flow: git plus governance gates (`oya-gate`/`oya-verify` where applicable, noting docs mention cloud-ci-required-status transition).
- Use isolated worktree/branch; target PR against dev; do not overwrite dev.
- Verify with fresh evidence before claiming completion.

## Autopilot phase
Current phase: deep-interview. Material ambiguity remains: whether “remaining work” means expand PR #644, create follow-up PR(s), or wait for #644 merge then continue from dev.

## Likely recommended path pending clarification
Keep PR #644 as the green foundation PR. Then implement the next unit as a separate one-workflow PR after foundation is merged, or a clearly marked stacked follow-up only if the user explicitly accepts the dependency.
