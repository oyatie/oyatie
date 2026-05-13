---
doc_class: AgentToolProtocol
shape: protocol
status: Accepted
authority_tier: 2
length_cap: 180
date: 2026-05-12
purpose: |
  Tool-by-tool calling convention for the sanctioned primitive set
  `{grit, icm, oya-tooling-agent-read}`, plus the Directive-12 escape hatch for raw
  `git`/`gh`. Every invocation form here is the canonical form; any deviation is a
  banned-primitives-lane violation.
canonical_authority: docs/CONSTITUTION.md
foundation: ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim)
related:
  - docs/agents/AGENT-ENTRY-POINT.md
  - docs/agents/AGENT-DECISION-TREE.md
  - docs/agents/AGENT-ICM-TOPIC-CONVENTIONS.md
  - docs/standards/claude-code-harness.md
  - docs/standards/agent-instructions-discipline.md
  - docs/standards/git-workflow.md
---

# Agent Tool Protocol

> Sanctioned primitives: **`grit`**, **`icm`**, **`oya-tooling-agent-read`**. Everything else is either Directive-12 documented exception or banned. Lane: `oya-foundry-fitness-banned-primitives`. Foundation: ADR-0053 (sanctioned primitives) and ADR-0054 (scaffold-claim).

## `grit` — claim, work, hand-off

`grit` is the only sanctioned merge primitive. Never invoke `git rebase`, `git merge`, `git push`, `gh pr merge` to land work — let `grit done` orchestrate the merge queue (per [`docs/DESIGN.md`](../DESIGN.md) §3.0.5.2).

| Subcommand | Canonical form | Required flags | Notes |
|---|---|---|---|
| `claim` | `grit claim --agent <id> --intent "<one-line>" <file::Identifier>` | `--agent`, `--intent` | One claim per symbol. Stack claims for multi-symbol IPs. FK error → scaffold-claim (ADR-0054). |
| `worktree` | `grit worktree --agent <id>` | `--agent` | Auto-creates `.grit/worktrees/<id>/`. Idempotent. |
| `symbols` | `grit symbols --refresh` | — | Re-indexes after new files land. Required by scaffold-claim pattern. |
| `status` | `grit status --agent <id>` | `--agent` | Shows your active claims + worktree state. |
| `watch` | `grit watch --symbol <file::Id>` | `--symbol` OR `--agent` | Streams release/merge events. Use to coordinate around contested symbols. |
| `assign` | `grit assign --agent <id> --to <other-id> --symbol <file::Id>` | all three | Hand off mid-flight. Emits `context-oyatie` row automatically. |
| `release` | `grit release --agent <id> --reason "<one-line>"` | `--agent`, `--reason` | Drop a claim without merging. Used for D8 deferral and D9 halt. |
| `heartbeat` | `grit heartbeat --agent <id>` | `--agent` | Hourly minimum. Stale agents (>4h no heartbeat) GC'd. |
| `gc` | `grit gc --dry-run` | — | Inspect stale claims; full GC is council-architecture only. |
| `queue` | `grit queue --status` | — | Shows merge-queue head/tail. Read-only. |
| `session` | `grit session start --agent <id>` | `--agent` | 0.3.0 bug active (RM-05); prefer session-less. |
| `done` | `grit done --agent <id>` | `--agent` | Final hand-off; enqueues merge. Refuses if acceptance evidence (audit `EVT-*`) missing. |

**Forbidden grit usage:** `--no-verify`, manual `--force`, claiming symbols outside your IP's frontmatter list.

## `icm` — persistent memory + decision log

`icm` is the audit trail for autonomous decisions. Every named event in
[`AGENT-ENTRY-POINT.md`](AGENT-ENTRY-POINT.md) §6 is `icm store` with a canonical topic from
[`AGENT-ICM-TOPIC-CONVENTIONS.md`](AGENT-ICM-TOPIC-CONVENTIONS.md).

| Subcommand | Canonical form | Use |
|---|---|---|
| `recall` | `icm recall "<query>" [-t <topic>] [-k <kw,kw>]` | Free-text search. Use BEFORE any work. |
| `recall-context` | `icm recall-context "<query>" --limit <n>` | Returns formatted block for prompt injection. |
| `store` | `icm store -t <topic> -c "<content>" -i <low\|medium\|high\|critical> -k "<kw1,kw2>"` | Record event. Importance `high` minimum for decisions/errors-resolved/halts. |
| `update` | `icm update <id> -c "<new content>"` | Correct a prior store in place; emits audit-update row. |
| `health` | `icm health` | Topic hygiene audit. Run weekly. |
| `topics` | `icm topics` | List all topics. Use to confirm canonical names. |

**Importance ladder:** `critical` = halts + autonomy-ceiling decisions; `high` = decisions, errors-resolved, direct-tool-invocations, scaffold-locks; `medium` = progress checkpoints; `low` = informational. Never store secrets, PII, or full file contents — content fields are summary form.

## `oya-tooling-agent-read` — read-only repo surface

The sanctioned read interface. Substitutes for `git log`, `git diff`, `gh pr view`, `gh pr comments` so the banned-primitives lane stays green.

| Subcommand | Substitutes | Form |
|---|---|---|
| `oya-tooling-agent-read log` | `git log` / `git show` | `oya-tooling-agent-read log --range <ref1>..<ref2> [--paths <glob>]` |
| `oya-tooling-agent-read diff` | `git diff` | `oya-tooling-agent-read diff --base <ref> --head <ref> [--paths <glob>]` |
| `oya-tooling-agent-read pr-view <n>` | `gh pr view <n>` | `oya-tooling-agent-read pr-view <pr-number>` |
| `oya-tooling-agent-read pr-comments <n>` | `gh pr view <n> --comments` | `oya-tooling-agent-read pr-comments <pr-number>` |
| `oya-tooling-agent-read audit-emit` | (no direct substitute) | `oya-tooling-agent-read audit-emit --event <EVT-id> --payload <json>` — emits the audit-chain row |

All `oya-tooling-agent-read` output is token-budgeted (rtk-style filtering by default) and emits an audit row tagged with your agent id. No write capabilities.

## Directive-12 escape hatch (raw `git` / `gh`)

Permitted iff: (a) no grit/icm primitive exists for the need, AND (b) inventing one would be over-engineering. Before the raw invocation:

```
icm store -t direct-tool-invocations \
  -c "<genuine need: e.g. 'git bisect — no grit primitive for binary-search'>" \
  -i high -k "git,<context>"
```

The lane `oya-foundry-fitness-banned-primitives` (revised per Directive 12) catches *undocumented* invocations only. ≥5 invocations of the same shape in 30 days auto-emits a `MISTAKES-LEDGER` migration-candidate row — this is a signal to extend `oya-tooling-agent-read`, not a punishment.

**Forbidden under Directive 12 even with rationale:**
- `git push --force` to `main`
- `git reset --hard` on shared refs
- `gh pr merge` (use `grit done`)
- `--no-verify` / hook bypass
- Editing `~/.claude/` from project sessions
- Touching `/Users/home/Documents/GitHub/claude-code` (read-only reference)

## Tool selection cheat-table

| Need | Use |
|---|---|
| Find prior decision on topic X | `icm recall-context "X" --limit 5` |
| See last 10 commits on path | `oya-tooling-agent-read log --range HEAD~10..HEAD --paths <p>` |
| Inspect a PR | `oya-tooling-agent-read pr-view <n>` |
| Take a symbol | `grit claim --agent <id> --intent "..." <file::Id>` |
| Hand off mid-work | `grit assign --agent <id> --to <other> --symbol <file::Id>` |
| Record a decision | `icm store -t decisions-oyatie ...` |
| Final ship | `grit done --agent <id>` after [`AGENT-COMPLETION-PROTOCOL.md`](AGENT-COMPLETION-PROTOCOL.md) |
| Emit audit chain row | `oya-tooling-agent-read audit-emit --event EVT-* --payload <json>` |

## When in doubt

If a need doesn't map to a row above:
1. Re-read the IP frontmatter — the answer is usually there.
2. `icm recall -t preferences -k "<area>"` for prior user preferences.
3. `icm recall -t decisions-oyatie -k "<area>"` for prior architectural calls.
4. If still no answer AND you cannot proceed, the case is in [`ESCALATION-MATRIX.md`](ESCALATION-MATRIX.md).
