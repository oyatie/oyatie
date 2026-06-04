---
purpose: Cross-agent tool-name mapping for Claude Code, Codex (OpenAI Codex CLI), Gemini (Gemini CLI), and OMC subagents.
doc_status: published
---

---
doc_class: Standard
shape: reference
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Cross-agent tool-name mapping for Claude Code, Codex (OpenAI Codex CLI),
  Gemini (Gemini CLI), and OMC subagents. Names the sanctioned tool surface
  per agent, documents tool-name differences (Codex/Gemini use different
  names than Claude Code), and codifies delegation patterns when one agent
  hands off to another. Resolves the
  `standards/multi-agent-tool-map.md` wave-2 forward-reference sentinel in
  `docs/AGENTS.md` §Per-agent appendices (Gemini).
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
planned_enforcement_ref: oya-governance-tool-map-cohesion
companion_docs:
  - docs/AGENTS.md
  - docs/standards/claude-code-harness.md
  - docs/standards/agent-instructions-discipline.md
related_adrs:
  - ADR-0053
  - ADR-0052
  - ADR-0054
---

# Multi-Agent Tool Map

## Doctrinal authority — [decision-principles.json](../../specs/decision-principles.json) + [forbidden-operations.json](../../specs/forbidden-operations.json)

Multiple agent harnesses operate on this repository (Claude Code, Codex,
Gemini, OMC subagents). Each harness exposes slightly different tool names
for the same underlying operation. This standard names the canonical surface
and the per-harness mapping so cross-harness instructions (e.g., the agent
fences in IPs) work without translation drift.

The agent operating contract is [`docs/AGENTS.md`](../AGENTS.md); the
per-harness appendices live in its §"Per-agent appendices". This standard is
the **reference table** those appendices point at.

## 1. Canonical operation set

These are the underlying operations every harness exposes (sometimes under
different names):

| Canonical operation | What it does |
|---|---|
| `read_file` | Read a file from the filesystem at an absolute path |
| `write_file` | Create or overwrite a file |
| `edit_file` | Apply a localized edit (search/replace, diff, or hunk) |
| `run_bash` | Execute a shell command |
| `glob_search` | Find files by glob pattern |
| `grep_search` | Find content by regex pattern |
| `ask_user` | Surface a question or option set to the user |
| `task_delegate` | Spawn a child agent / subagent run |
| `web_fetch` | Fetch a URL (read-only) |
| `web_search` | Perform a web search |
| `notebook_edit` | Edit a Jupyter notebook cell |
| `mcp_call` | Call a Model Context Protocol tool |

## 2. Tool-name mapping per harness

| Canonical | Claude Code | Codex CLI | Gemini CLI | OMC subagents |
|---|---|---|---|---|
| `read_file` | `Read` | `read_file` | `read_file` | `Read` (inherits Claude) |
| `write_file` | `Write` | `apply_patch` (creates) | `write_file` | `Write` |
| `edit_file` | `Edit` | `apply_patch` (edits) | `replace` / `edit` | `Edit` |
| `run_bash` | `Bash` | `shell` | `run_shell_command` | `Bash` |
| `glob_search` | `Glob` | `shell` w/ `rg --files` | `glob` | `Glob` |
| `grep_search` | `Grep` | `shell` w/ `rg` | `search_file_content` | `Grep` |
| `ask_user` | `AskUserQuestion` | (interactive prompt) | (interactive prompt) | `AskUserQuestion` |
| `task_delegate` | `Task` / `Skill` | (per-CLI; subprocess) | (via system prompts) | `Task` |
| `web_fetch` | `WebFetch` | `web_fetch` | `web_fetch` | `WebFetch` |
| `web_search` | `WebSearch` | `web_search` | `google_web_search` | `WebSearch` |
| `notebook_edit` | `NotebookEdit` | n/a | n/a | `NotebookEdit` |
| `mcp_call` | `mcp__<server>__<tool>` | `mcp__<server>__<tool>` | `mcp__<server>__<tool>` | inherits Claude |

Sources: [Claude Code tool reference](https://docs.anthropic.com/en/docs/claude-code/),
[Codex CLI AGENTS.md convention](https://agents.md),
[Gemini CLI docs](https://github.com/google-gemini/gemini-cli),
cross-tool [AGENTS.md spec](https://agents.md).

## 3. Sanctioned tool surface per agent

Inside an `<!-- agent-instructions -->` fence (per
[`agent-instructions-discipline.md`](agent-instructions-discipline.md)),
the agent MUST use only the triad + the canonical tool surface above:

| Agent | Default-sanctioned | Allowed with rationale | Forbidden |
|---|---|---|---|
| Gemini CLI | `read_file`, `write_file`, `replace`, `run_shell_command`, `glob`, `search_file_content` | Direct `git`/`gh` per Directive 12 | sandbox-escape commands |
| OMC subagent | inherits Claude Code's surface | inherits Claude Code's Directive-12 extensions | inherits Claude Code's bans |

The lane `oya-governance-tool-map-cohesion` validates that the
per-harness sanctioned set in each agent appendix matches this table.

## 4. Per-harness invocation surface (commands)

| Harness | Build | Test | Lint | Source |
|---|---|---|---|---|
| Codex CLI | `buck2 build //...` or trusted target subset | `buck2 test //... --show-output` or trusted target subset | Buck2-invoked lint/static-analysis target | AGENTS.md §Codex appendix |
| Gemini CLI | same as Codex | same as Codex | same as Codex | AGENTS.md §Gemini appendix |
| OMC subagents | inherits Claude Code | inherits Claude Code | inherits Claude Code | AGENTS.md §OMC appendix |

token-killer filtering. Codex and Gemini run the commands raw — they do
human readers.

## 5. Delegation patterns

When one agent hands off to another:

### 5.1 Claude Code → OMC subagent

Inside a Claude Code session, delegate via the `Skill` or `Task` tool:

```
Skill(skill="oh-my-claudecode:planner", args="…")
Task(subagent="executor", task="…")
```

The subagent inherits the parent session's environment (cwd, hooks,
sanctioned tools) but writes outputs to `.omc/state/`. Cancellation
propagates downward.

### 5.2 Claude Code → Codex / Gemini

Two patterns:

1. **Parallel review via `omc ask`**: dispatch the question to Codex or
   Gemini via the `omc ask` skill (sanctioned per
   [`/oh-my-claudecode:ask`](../STANDARDS-AND-TEMPLATES.md));
   capture artifacts to `.omc/state/`.
2. **Process-based tmux team**: launch a Codex or Gemini CLI in a tmux
   pane via `/oh-my-claudecode:omc-teams`; the orchestrator reads the
   pane state at handoff.

`.omc/state/`.

### 5.3 Codex / Gemini → Claude Code

The reverse direction requires the user to bring the artifact (e.g., a
generated patch) back into the Claude Code session manually OR via a
shared state file `.omc/state/<task-id>.md`. Cross-harness session
inheritance is not assumed.

## 6. Memory & state interop

| Surface | Shared across harnesses? | Notes |
|---|---|---|
| `.omc/state/`, `.omc/notepad.md`, `.omc/project-memory.json` | YES | Repo-checked-in (or session-scoped per file); all harnesses read |
| `.omc/plans/` | YES | Working drafts of plans / IPs; all harnesses read+write |
| Claude Code skill / hook state | NO | Lives under `~/.claude/`; not portable |
| Codex `.codex/skills/`, `.codex/worktree_init.sh` | YES (repo-local) | Codex-specific tooling |
| Gemini per-session config | NO | Gemini-specific |

Cross-harness sharing rule: **state lives under `.omc/`**, harness-specific
config lives under per-harness directories (`~/.claude/`, `.codex/`, …).

## 7. MCP server interop

All three harnesses (Claude Code, Codex, Gemini) support MCP. Servers
registered under `.mcp.json` (or per-harness equivalent) are shared
config when the file is in the repo root. Per
[`/oh-my-claudecode:mcp-setup`](../STANDARDS-AND-TEMPLATES.md), the
identically across harnesses where possible.

## 8. Local AGENTS.md narrowing

Per [`docs/AGENTS.md`](../AGENTS.md) §Boundaries, sub-directory
`AGENTS.md` files MAY narrow tool / context bounds but MUST NOT lower the
canonical bar. A sub-directory cannot grant a tool that the root
contract forbids.

## 9. Reviewer-agent verdicts in cross-harness PRs

When a PR is authored by Codex or Gemini and reviewed by Claude Code (or
vice versa), the merge-gate hook still requires the reviewer-agent
verdict in `## Code Review` (per AGENTS.md D8). The reviewing harness is
free to spawn its own per-change-class reviewer agent (the OMC subagent
catalog is the canonical pool, but Gemini may use its own equivalents
named in `agent-instructions-discipline.md` §6).

## 10. Anti-patterns

1. **Hard-coded harness-specific tool names in IP / runbook prose.** Use
   the canonical name and let this table do the mapping.
3. **Cross-harness state assumed via Claude Code skill memory.** Always
4. **MCP server registered for one harness only when it serves all
   three.** Move to `.mcp.json` (or per-harness equivalent).

## 11. Sources scanned

- Cross-tool [AGENTS.md convention](https://agents.md).
- [Anthropic — Claude Code tools](https://docs.anthropic.com/en/docs/claude-code/).
- [OpenAI Codex CLI docs](https://github.com/openai/codex).
- [Gemini CLI](https://github.com/google-gemini/gemini-cli).
- [`docs/AGENTS.md`](../AGENTS.md) §Per-agent appendices (Codex, Gemini, OMC).
- [`docs/standards/claude-code-harness.md`](claude-code-harness.md).
