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

## Dual-critic independence (mandatory honesty)

Dual-critic is **not** “two prompts in one model family.”

| Label | Allowed when | Counts for `require_cross_model_critics` |
|-------|----------------|------------------------------------------|
| `cross_model` | ≥2 critic instances via `.grok/bin/mm-role CRITIC` (or equivalent) with **distinct** `provider` values from `multi-model-roles.json` / actual CLI used | Yes |
| `same_family` | Session subagents / single provider with split lenses only | **No** — may inform humans; **must not** clear merge-check when cross-model is required |
| `unknown` | Providers not declared on packet | **No** — fail closed |

Every dual-critic packet MUST include:

```json
{
  "independence": "cross_model|same_family|unknown",
  "critics": [
    {"id": "A", "provider": "anthropic|openai-codex|xai|…", "model": "…", "verdict": "APPROVE|REQUEST_CHANGES"},
    {"id": "B", "provider": "…", "model": "…", "verdict": "…"}
  ],
  "verdict": "APPROVE|REQUEST_CHANGES",
  "real": true
}
```

`real=true` only if each critic inspected evidence independently (paths, commands, or diffs cited).  
Laundering same-family APPROVE as multi-model is a **process defect** (F-SAME-FAMILY-CRITIC-LAUNDER).

## Kit presence

- Authority trunk (`origin/dev`) SHOULD contain the `.grok/` kit (bins + harness + roles). Journals under `mm-runs/` and `memory/` stay gitignored.
- Implementation **worktrees** MUST either inherit kit from base or run `mm-bootstrap` before claiming multi-model delivery.
- Missing kit ⇒ do not claim dual-critic independence; fix process (land kit / bootstrap), not only product code.

## CI push thrash

While `oya-ci-required` is **in_progress** on the PR tip: prefer armed re-poll over additional pushes.  
Doc-only or evidence-only commits SHOULD be held or batched until the tip run settles, unless the push fixes the **active red**. Cap successive CI-cancelling pushes (see `drive.v1.json` `push_budget`).

## Mechanical TDD / anti-false-green (implement path)

Implement stages are **ordered and fail closed** (`programs/IMPLEMENT-LIFECYCLE.md`):

1. **RED** — failing tests recorded (`proof_failed=true`, no product code)  
2. **IMPLEMENT** — minimal fix only  
3. **GREEN** — same suite passes  
4. **INTEGRATION** — boundary coverage  
5. **FALSE_GREEN_SCAN** — no skips, no deleted RED, no weakened asserts, no hand-edited generated faces  
6. **REVIEW** (dual critic) → **SIMPLIFY** → **HARDEN** → **VERIFY** → admit  

Forbidden: implement-without-red; skip/ignore tests to go green; “green” without exit codes.

## Process-edit loop (Bun)

If workers thrash (stubs, test skips, destructive git, scope creep, same-family critic laundering, kit bypass, implement-without-red): fail the stage, append `process_edits.md`, and fix the harness/prompts before retrying.
