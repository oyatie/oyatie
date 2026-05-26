---
doc_class: Standard
shape: ~
length_cap: 250
authority_tier: 2
status: pending approval
purpose: |
  Claude Code harness contract for oyatie. Defines the sanctioned-primitive
  triad (`grit`, `icm`, `oya-tooling-agent-read`), the Directive-12 pragmatic
  git/gh exception with documented rationale, the grit claim→work→done
  lifecycle, icm topic conventions, Stop-hook expectations, and PreToolUse /
  PostToolUse / Stop / SessionStart hooks ordering. Resolves the
  `standards/claude-code-harness.md` wave-2 forward-reference sentinel in
  `docs/AGENTS.md` §Per-agent appendices (Claude Code).
lift_target: oyatie/docs/standards/claude-code-harness.md
canonical_authority: docs/CONSTITUTION.md
enforced_by: oya-governance-banned-primitives
companion_docs:
  - docs/AGENTS.md
  - docs/standards/multi-agent-tool-map.md
  - docs/standards/git-workflow.md
  - docs/standards/agent-instructions-discipline.md
---

# Claude Code Harness

## Constitutional authority — [CONSTITUTION.md](../CONSTITUTION.md)

This standard governs the Claude Code harness (the long-lived agent runtime
documented at <https://docs.anthropic.com/en/docs/claude-code/>). Per
[`docs/AGENTS.md`](../AGENTS.md) §Per-agent appendices (Claude Code), this
standard is the place to look for sanctioned-primitive enforcement, hook
configuration, and OMC magic-keyword routing.

## 1. Sanctioned-primitive contract

Per [`.omc/plans/MASTERPLAN.md`](../plans/MASTERPLAN.md) Directive 12 and
the agentic-pipeline cutover (`ralplan-oyatie-sst-consolidation.md`), the
default sanctioned tool surface inside an `<!-- agent-instructions -->`
fence is the **triad**:

| Tool | Role | Source |
|---|---|---|
| [`grit`](https://github.com/rtk-ai/grit) | Git-for-agents: claim, work, done lifecycle; merge queue | rtk-ai/grit |
| [`icm`](https://github.com/rtk-ai/icm) | Persistent memory across sessions; MCP-native | rtk-ai/icm |
| `oya-tooling-agent-read` | In-tree read primitives (Foundry-owned); composes grit + icm + Foundry indexes | this repo |

Versions: per
[`.omc/scratch/lts-versions-verified-2026-05-12.md`](../specs/lts-versions-verified-2026-05-12.md)
— `grit v0.3.0`, `icm v0.10.39` (both Apache-2.0).

## 2. Directive-12 pragmatic-git rule

Direct `git` / `gh` invocation inside agent fences is **permitted** when:

1. No grit / icm / `oya-tooling-agent-read` primitive exists for the
   intended operation.
2. Inventing a wrapper would be over-engineering (a one-shot operation;
   < 5 invocations per 30 days across the repo).
3. The agent logs a rationale via:
   ```sh
   icm store -t direct-tool-invocations \
     -c "<one-line rationale>" \
     -i high -k "git,<context>"
   ```
   **BEFORE** the invocation.

The revised lane `oya-governance-banned-primitives` catches
**undocumented** `git` / `gh` calls in agent-instruction sections, not all
calls. Repeat invocations (≥ 5 same-shape in 30 days) auto-emit a
migration-candidate row in `docs/MISTAKES-LEDGER.md`.

Full rationale, examples, and migration-candidate workflow are in
[`git-workflow.md`](git-workflow.md).

## 3. grit claim→work→done lifecycle

The canonical inner loop for a Claude Code agent inside a long-running
session:

```
grit claim <symbol>      # acquire a work-stealing lease on a file::Identifier
  ↓
icm recall "<query>"     # bring relevant memory into context
  ↓
<perform edits>          # via Read / Edit / Write tools
  ↓
cargo nextest run …      # evidence per testing.md §2
  ↓
icm store …              # capture decisions per CLAUDE.md §store triggers
  ↓
grit done <symbol>       # release lease + emit audit-chain EVT-GRIT-DONE
```

Rules:

1. Every IP (Implementation Plan) under `.omc/plans/milestones/**` names
   the symbol an agent claims as a real `file::Identifier`
   (per MASTERPLAN §6 dual-audience contract).
2. `grit claim` failures (already-claimed, stale lease, merge conflict)
   require icm-store of the failure mode before retry.
3. `grit done` MUST run AFTER `cargo nextest` + `cargo clippy -- -D warnings`
   + `cargo deny check` are green locally (per AGENTS.md D9–D11).
4. Aborted work emits `grit abandon <symbol>` with reason; never leave a
   dangling lease.

## 4. icm topic conventions

Per project CLAUDE.md (ICM mandatory):

| Topic prefix | Use |
|---|---|
| `errors-resolved` | Per fix; `-i high -k "<error-keyword>,<root-cause>"` |
| `decisions-<project>` | Architecture / design decision |
| `preferences` | User preference discovered |
| `context-<project>` | Significant task completion summary |
| `direct-tool-invocations` | Directive-12 git/gh rationale (per §2) |
| `mistakes-ledger-candidates` | Pre-postmortem failure observations |
| `agent-handoff-<phase>` | IP-complete payload for next agent (per MASTERPLAN §6) |

Importance levels: `critical` (preferences, security) / `high` (errors,
decisions) / `medium` (context) / `low` (informational).

Recall pattern at session start:
```sh
icm recall-context "<task-keywords>" --limit 5
```

## 5. Active hooks

Per [`docs/AGENTS.md`](../AGENTS.md) §Per-agent appendices, the following
hooks MUST be configured for every project session:

| Hook event | Script | Purpose | Order |
|---|---|---|---|
| `SessionStart` | `scripts/hooks/memory-bootstrap.mjs` | inject icm recall context; load skills | 1 |
| `SessionStart` | `scripts/hooks/load-omc-skills.mjs` | register local OMC skills | 2 |
| `PreToolUse:Bash` | `scripts/hooks/banned-primitives.mjs` | check sanctioned-primitive contract + Directive-12 rationale | 1 |
| `PreToolUse:Bash` | `scripts/hooks/rtk-rewrite.mjs` | rewrite `<cmd>` → `rtk <cmd>` per CLAUDE.md | 2 |
| `PreToolUse:Bash` | `scripts/hooks/guard-pr-merge-review.mjs` | refuse `gh pr merge` without `## Code Review` | 3 |
| `PostToolUse:Bash` | `scripts/hooks/telemetry.mjs` | emit `EVT-TOOL-INVOKED` audit-chain record | 1 |
| `Stop` | `scripts/hooks/loop-cancellation.mjs` | re-walk Done-Definition; refuse silent exit on long-running loops | 1 |
| `Stop` | `scripts/hooks/icm-progress-flush.mjs` | write a progress summary to icm if >20 tool calls since last store | 2 |

Ordering is **stable** — earlier hooks gate later hooks. A hook failure is
a signal: fix the underlying issue, do not skip (per CONSTITUTION §Do Item
2, §Avoid Item 2).

## 6. OMC magic-keyword routing

The OMC plugin recognizes keyword triggers in user prompts and routes to
the matching skill. Detail per
[`/oh-my-claudecode:`](../STANDARDS-AND-TEMPLATES.md) catalog:

| Keyword | Skill | Use |
|---|---|---|
| `autopilot` | `/oh-my-claudecode:autopilot` | Full autonomous execution loop |
| `ralph` | `/oh-my-claudecode:ralph` | Self-referential loop until verifier passes |
| `ulw` / `ultrawork` | `/oh-my-claudecode:ultrawork` | Parallel high-throughput task completion |
| `team` | `/oh-my-claudecode:team` | N coordinated agents on a shared task list |
| `ralplan` | `/oh-my-claudecode:ralplan` | Consensus planning entrypoint |
| `cancelomc` | `/oh-my-claudecode:cancel` | Cancel any active OMC mode |

Every long-running loop (autopilot, ralph, ultrawork, team) MUST re-walk
the Done-Definition checklist before exiting; the `Stop:loop-cancellation`
hook enforces this.

## 7. Skill loading

Project-level always-loaded skills (per AGENTS.md §Per-agent appendices):

- `coding-standards`, `tdd-workflow`,
  `superpowers:test-driven-development`,
  `superpowers:verification-before-completion`,
  `superpowers:systematic-debugging`, `search-first`.

Language/domain skills load from file context: `rust-*`, `frontend-*`,
`postgres-patterns`, `healthcare-phi-compliance`.

Custom OMC subagents (per AGENTS.md §OMC): `executor`, `architect`,
`verifier`, `code-reviewer`, `silent-failure-hunter`, `tdd-guide`,
`doc-updater`, `planner`, `critic`, `debugger`, `tracer`, `explore`,
`designer`, `writer`, `qa-tester`.

## 8. Boundaries

- Claude Code MUST NOT edit `~/.claude/` from a project session — user-
  machine state. The lane `oya-governance-user-machine-guard` checks.
- Claude Code MUST NOT touch the read-only reference path
  `/Users/home/Documents/GitHub/claude-code`.
- Local `AGENTS.md` files under sub-directories MAY narrow context but
  MUST NOT lower the canonical bar.

## 9. Self-test

Before relying on hook / harness changes, run:

```sh
npm --prefix /Users/home/.codex test
```

Hook failures during the self-test are blocking; debug and fix before
making the harness change live for the session.

## 10. Cancellation

Per [`docs/AGENTS.md`](../AGENTS.md) §Long-running loop rule:
`/oh-my-claudecode:cancel` is the **only** sanctioned cancellation path.
The cancellation re-walks the Done-Definition checklist and either declares
the loop complete or records the structural block.

## 11. Anti-patterns

1. **`git` / `gh` calls in agent fences without an `icm store -t
   direct-tool-invocations` log.** The revised lane refuses.
2. **Skipping the `Stop:loop-cancellation` hook** by exiting via process
   kill. Use `/oh-my-claudecode:cancel`.
3. **icm-store omitted** after a significant task (per CLAUDE.md §store
   triggers). The `Stop:icm-progress-flush` hook catches the silent case.
4. **Editing `~/.claude/CLAUDE.md` from a project session.**
5. **Custom Claude Code skill that bypasses the merge-gate hook.**

## 12. Sources scanned

- [Anthropic — Claude Code docs](https://docs.anthropic.com/en/docs/claude-code/).
- [Anthropic — Claude Code memory](https://docs.anthropic.com/en/docs/claude-code/memory).
- [rtk-ai/grit](https://github.com/rtk-ai/grit), [rtk-ai/icm](https://github.com/rtk-ai/icm).
- [`.omc/plans/MASTERPLAN.md`](../plans/MASTERPLAN.md) §2 Directive 12.
- [`docs/AGENTS.md`](../AGENTS.md) §Per-agent appendices (Claude Code).
- [`docs/CONSTITUTION.md`](../CONSTITUTION.md) §Prohibitions Item 2 (no
  hook bypass).
