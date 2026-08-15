---
purpose: Cross-agent tool-name mapping for Claude Code, Codex (OpenAI Codex CLI), and Gemini (Gemini CLI). OMC columns are historical/compatibility-only.
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
  and Gemini (Gemini CLI). OMC subagent columns and `.omc/` state paths are
  historical/compatibility-only under ADR-0619. Names the sanctioned tool
  surface per live agent, documents tool-name differences, and codifies
  delegation patterns when one agent hands off to another.
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json + docs/AGENTS.md
planned_enforcement_ref: oya-governance-tool-map-cohesion
companion_docs:
  - docs/AGENTS.md
  - docs/standards/agent-instructions-discipline.md
related_adrs:
  - ADR-0053
  - ADR-0116
  - ADR-0515
  - ADR-0619
---

# Multi-Agent Tool Map

> **Harness-brand note (ADR-0619 / RR-HARNESS-0619):** OMC / OMX / GJC / Hermes
> are **not** live coordination authority. Columns and sections that mention
> OMC are retained as historical/compatibility mapping only. Live operating
> contract: [`docs/AGENTS.md`](../AGENTS.md). Merge admission: [ADR-0515](../decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md).
> Machine-local multi-model tooling is ignored and never merge or instruction authority. The
> former harness standard was deleted after this boundary moved into the operating contract.

## Doctrinal authority — [decision-principles.json](../../specs/decision-principles.json) + [forbidden-operations.json](../../specs/forbidden-operations.json)

Multiple agent runtimes may operate on this repository (Claude Code, Codex,
Gemini). Each exposes slightly different tool names for the same underlying
operation. This standard names the canonical surface and the per-runtime
mapping so cross-runtime instructions work without translation drift.

The agent operating contract is [`docs/AGENTS.md`](../AGENTS.md); the
per-agent appendices live in its §"Per-agent appendices". This standard is
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
| Codex CLI | `cargo build` | `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | AGENTS.md §Codex appendix |
| Gemini CLI | same as Codex | same as Codex | same as Codex | AGENTS.md §Gemini appendix |
| OMC subagents | inherits Claude Code | inherits Claude Code | inherits Claude Code | AGENTS.md §OMC appendix |

token-killer filtering. Codex and Gemini run the commands raw — they do
human readers.

## 5. Delegation patterns

When one agent hands off to another:

### 5.1 Claude Code → residual OMC subagent (compatibility only)

**Not a forward pattern.** Prefer plain git worktrees + protected PR. Operator-installed
multi-model tooling may assist locally but is never shared repository authority. Residual OMC
Skill/Task delegation may still appear in old sessions; do not invent new OMC-owned state under
`.omc/` as shared authority.

### 5.2 Claude Code → Codex / Gemini

Forward patterns:

1. **Isolated worktrees + PR**: each runtime works on its own branch; share
   via git, not external harness state.
2. **Optional operator-installed multi-model tooling**: local roles and dual-critic stages whose
   evidence returns through PR diffs/reviews (not merge authority).
3. **Process-based tmux / parallel CLI**: launch Codex or Gemini in a
   separate process; hand off via PR diffs or tracked evidence paths under
   `/evidence` / PR body — not `.omc/state/`.

Historical `omc ask` / `/oh-my-claudecode:*` team skills are compatibility
residue only (ADR-0619).

### 5.3 Codex / Gemini → Claude Code

Bring artifacts back via git/PR (patch, review comments, evidence links).
Do not require a shared `.omc/state/<task-id>.md` file. Cross-runtime
session inheritance is not assumed.

## 6. Memory & state interop

| Surface | Shared across runtimes? | Notes |
|---|---|---|
| `/specs`, `/registry`, `/evidence`, `/templates` | YES | Live machine-readable authority |
| Ignored agent runtime overlays | NO | Machine-local only; never shared instruction or merge authority |
| `.omc/state/`, `.omc/plans/`, `.omx/`, `.gjc/` | NO (provenance) | Gitignored residual; do not treat as shared authority (ADR-0619) |
| Claude Code skill / hook state | NO | Lives under `~/.claude/`; not portable |
| Codex installed skill state | NO | Runtime-installed outside the repository; ignored repo-local state is not shared |
| Gemini per-session config | NO | Gemini-specific |

Cross-runtime sharing rule: **shared truth is git + machine-readable specs/evidence**; ignored local
runtime overlays and residual harness directories are not admission or plan authority.

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
vice versa), merge admission still requires reviewer APPROVE plus the
single protected `oya-ci-required` context (ADR-0515). The reviewing
runtime may spawn its own per-change-class reviewer; residual OMC
subagent catalog names are historical inventory only.

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
- [`docs/AGENTS.md`](../AGENTS.md) §Per-agent appendices (Codex, Gemini; OMC legacy only).
- [ADR-0515](../decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md), [ADR-0619](../decisions/ADR-0619-zero-live-context-retirement-of-external-agent-harness-brand.md).
