---
doc_class: AgentToolProtocol
shape: protocol
status: Accepted
authority_tier: 2
length_cap: 180
date: 2026-05-12
purpose: |
  Tool-by-tool calling convention for the sanctioned primitive set
  `git`/`gh`. Every invocation form here is the canonical form; any deviation is a
  banned-primitives-lane violation.
canonical_authority: docs/CONSTITUTION.md
foundation: ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim)
related:
  - docs/agents/AGENT-ENTRY-POINT.md
  - docs/agents/AGENT-DECISION-TREE.md
  - docs/standards/claude-code-harness.md  # retirement tombstone only (ADR-0619)
  - docs/standards/agent-instructions-discipline.md
  - docs/standards/git-workflow.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
doc_status: published
---

# Agent Tool Protocol




| Subcommand | Canonical form | Required flags | Notes |
|---|---|---|---|




| Subcommand | Canonical form | Use |
|---|---|---|

**Importance ladder:** `critical` = halts + autonomy-ceiling decisions; `high` = decisions, errors-resolved, direct-tool-invocations, scaffold-locks; `medium` = progress checkpoints; `low` = informational. Never store secrets, PII, or full file contents — content fields are summary form.

## `tooling-agent-read` — read-only repo surface

The sanctioned read interface. Substitutes for `git log`, `git diff`, `gh pr view`, `gh pr comments` so the banned-primitives lane stays green.

| Subcommand | Substitutes | Form |
|---|---|---|
| `tooling-agent-read log` | `git log` / `git show` | `tooling-agent-read log --range <ref1>..<ref2> [--paths <glob>]` |
| `tooling-agent-read diff` | `git diff` | `tooling-agent-read diff --base <ref> --head <ref> [--paths <glob>]` |
| `tooling-agent-read pr-view <n>` | `gh pr view <n>` | `tooling-agent-read pr-view <pr-number>` |
| `tooling-agent-read pr-comments <n>` | `gh pr view <n> --comments` | `tooling-agent-read pr-comments <pr-number>` |
| `tooling-agent-read audit-emit` | (no direct substitute) | `tooling-agent-read audit-emit --event <EVT-id> --payload <json>` — emits the audit-chain row |


## Directive-12 escape hatch (raw `git` / `gh`)


```
  -i high -k "git,<context>"
```

The lane `governance-banned-primitives` (revised per Directive 12) catches *undocumented* invocations only. ≥5 invocations of the same shape in 30 days auto-emits a `MISTAKES-LEDGER` migration-candidate row — this is a signal to extend `tooling-agent-read`, not a punishment.

**Forbidden under Directive 12 even with rationale:**
- `git push --force` to `main`
- `git reset --hard` on shared refs
- `--no-verify` / hook bypass
- Editing `~/.claude/` from project sessions
- Touching `/Users/home/Documents/GitHub/claude-code` (read-only reference)

## Tool selection cheat-table

| Need | Use |
|---|---|
| See last 10 commits on path | `tooling-agent-read log --range HEAD~10..HEAD --paths <p>` |
| Inspect a PR | `tooling-agent-read pr-view <n>` |
| Emit audit chain row | `tooling-agent-read audit-emit --event EVT-* --payload <json>` |

## When in doubt

If a need doesn't map to a row above:
1. Re-read the IP frontmatter — the answer is usually there.
4. If still no answer AND you cannot proceed, the case is in [`ESCALATION-MATRIX.md`](ESCALATION-MATRIX.md).
