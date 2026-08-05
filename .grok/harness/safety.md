# Multi-model harness safety policy

This harness is **not** merge authority. Live merge readiness remains reviewer APPROVE + `oya-ci-required` (ADR-0515).

## Git baseline (mandatory)

- Implementation lanes start from `origin/dev` at create time (`behind == 0`).
- PREFLIGHT fails closed when the working tree is behind `origin/dev` or contains unrelated dirty paths.
- EXECUTOR must not run unless PREFLIGHT is green (or an explicit, journaled `--allow-diverged` exception that still blocks execute by default).

## Forbidden operations (workers and orchestrator automation)

- `git reset --hard`
- `git stash` / `git stash pop`
- Force push (`git push --force` / `-f`)
- `git clean -fdx` without human confirmation
- Hand-editing any `*.generated.json`
- Skipping, deleting, or weakening tests/gates to go green
- Sandbox `danger-full-access` without explicit human flag
- Claiming merge readiness without the single required context `oya-ci-required`

## Preferred isolation

- One EXECUTOR write root per admitted slice
- `git worktree` from `origin/dev` for implementation lanes
- Critics/architect/security stay read-only

## Fail closed

Missing dual-critic confirmation, missing verify exits, schema failure, or dirty/diverged base ⇒ **stop**, do not proceed.

## Process-edit loop (Bun)

If workers thrash (stubs, test skips, destructive git, scope creep): fail the stage, append `process_edits.md`, and fix the harness/prompts before retrying.
