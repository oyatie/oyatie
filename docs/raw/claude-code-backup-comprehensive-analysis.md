---
doc_status: published
---

# Claude Code backup branch: comprehensive repository findings

**Date:** 2026-05-10
**Analyst:** Codex / OMX `$analyze` + `$autoresearch-goal` + `$zoom-out`
**Target:** `https://github.com/jason931225/claude-code/tree/backup`
**Local inspection clone:** `/tmp/claude-code-backup-analysis`
**Inspected backup HEAD:** `372a01d48621cadeb0c3a3a0164c4622c35cbfea`
**Artifact status:** raw research artifact; not canonical Oyatie/Foundry documentation.

> Copyright/safety note: the target README describes the repository as leaked source. This report records architectural findings, workflows, guardrails, and clean-room design lessons. It intentionally does not reproduce source bodies, proprietary implementation text, or actionable bypass instructions.

---

## 1. Scope, provenance, and confidence

### Scope inspected

The inspected branch contains a large TypeScript/TSX CLI/TUI application under `src/` plus a README. The backup branch did not include the normal package metadata needed for a complete build-and-test cycle.

Observed top-level shape:

- `README.md`
- `src/`
- `.git/`

Observed absence:

- no `package.json`
- no lockfile
- no `.github/`
- no obvious CI configuration
- no normal test runner config

Approximate source inventory under `src/`:

- ~1,902 files
- dominant languages: TypeScript / TSX
- small number of JS and MD files
- several very large files include inline source-map tails

### Confidence

- **High confidence:** repo topology, entrypoints, CLI command shape, tool contract, tool registry, tool execution lifecycle, permission modes, shell guardrail architecture, hook schema, settings schema, MCP/LSP integration, session storage/compaction architecture, team/subagent/worktree/remote-control surfaces.
- **Medium confidence:** exact behavior of feature-gated/internal-only paths, production service interactions, telemetry semantics, enterprise policy behavior.
- **Low confidence:** actual build/test health, because dependency and CI metadata are missing from the backup branch.

---

## 2. Core mental model

The repository is best understood as a mature agentic CLI runtime with these planes:

1. **Bootstrap plane**
   - `src/entrypoints/cli.tsx`
   - `src/entrypoints/init.ts`
   - `src/main.tsx`

2. **Interactive UI plane**
   - React + Ink terminal UI
   - `src/screens/REPL.tsx`
   - `src/components/`
   - `src/hooks/`
   - `src/ink/`

3. **Conversation/query plane**
   - `src/QueryEngine.ts`
   - `src/query.ts`

4. **Tool/capability plane**
   - `src/Tool.ts`
   - `src/tools.ts`
   - `src/services/tools/toolExecution.ts`
   - `src/tools/`

5. **Guardrail/permission plane**
   - `src/utils/permissions/PermissionMode.ts`
   - `src/utils/permissions/permissionSetup.ts`
   - `src/utils/permissions/permissions.ts`
   - `src/tools/BashTool/`
   - `src/tools/PowerShellTool/`

6. **Extension plane**
   - commands: `src/commands.ts`, `src/commands/`
   - skills: `src/skills/`, `src/tools/SkillTool/`
   - plugins: `src/plugins/`, `src/utils/plugins/`, `src/services/plugins/`
   - hooks: `src/types/hooks.ts`, `src/utils/hooks.ts`
   - MCP: `src/services/mcp/`, `src/tools/MCPTool/`

7. **Distributed agent/session plane**
   - subagents: `src/tools/AgentTool/`
   - teams: `src/tools/shared/spawnMultiAgent.ts`, `src/tools/TeamCreateTool/`, `src/tools/TeamDeleteTool/`
   - tasks: `src/tasks/`, `src/tools/Task*`
   - remote bridge: `src/bridge/`, `src/remote/`, `src/server/`
   - worktrees: `src/tools/EnterWorktreeTool/`, `src/tools/ExitWorktreeTool/`

8. **Persistence/memory/context plane**
   - `src/bootstrap/state.ts`
   - `src/utils/sessionStorage.ts`
   - `src/services/compact/`
   - `src/services/SessionMemory/`
   - `src/context.ts`

---

## 3. Repository topology by area

Approximate file and line counts from local inspection:

| Area | Files | Approx. lines | Purpose |
|---|---:|---:|---|
| `main.tsx` | 1 | 4,684 | Primary Commander CLI and runtime setup |
| `QueryEngine.ts` | 1 | 1,295 | Per-conversation orchestration wrapper |
| `query.ts` | 1 | 1,729 | Streaming model/tool loop |
| `Tool.ts` | 1 | 792 | Core tool interface/contract |
| `tools.ts` | 1 | 389 | Built-in/MCP tool registry |
| `tools/` | 184 | ~50k | Built-in tools and tool-specific security/UI |
| `commands/` | 207 | ~26k | Slash/local/headless command implementations |
| `components/` | 389 | ~82k | Ink/React UI components |
| `utils/` | 564 | ~180k | Permissions, settings, shell parsing, storage, git, environment, etc. |
| `services/` | 130 | ~54k | API, MCP, LSP, analytics, compaction, plugins, voice, VCR |
| `hooks/` | 104 | ~19k | React/runtime hooks |
| `bridge/` | 31 | ~12.6k | Remote-control bridge/session runner |
| `cli/` | 19 | ~12.3k | Headless/NDJSON/transport helpers |
| `entrypoints/` | 8 | ~4k | CLI/MCP/SDK bootstraps |
| `ink/` | 96 | ~19.8k | Ink runtime/fork |
| `keybindings/` | 14 | ~3.1k | Keybinding parser/defaults/validation |
| `tasks/` | 12 | ~3.3k | Background/local/remote task abstractions |
| `skills/` | 20 | ~4k | Bundled skill support |
| `memdir/` | 8 | ~1.7k | Memory directory handling |
| `vim/` | 5 | ~1.5k | Vim input state machine |
| `native-ts/` | 4 | ~4k | Native/TypeScript integration helpers |
| `screens/` | 3 | ~6k | REPL and screen-level UI |

Largest notable files:

- `src/screens/REPL.tsx`
- `src/main.tsx`
- `src/components/PromptInput/PromptInput.tsx`
- `src/commands/plugin/ManagePlugins.tsx`
- `src/components/Settings/Config.tsx`
- `src/ink/ink.tsx`
- `src/query.ts`
- `src/bootstrap/state.ts`
- `src/tools/AgentTool/AgentTool.tsx`
- `src/QueryEngine.ts`

Observation: the codebase is powerful but monolithic in key areas. Main CLI setup, REPL, prompt input, settings, and plugin management are very large seams.

---

## 4. Boot and entrypoint workflow

### `src/entrypoints/cli.tsx`

This is the outer startup shim. It sets initial environment behavior, handles fast paths before loading the heavy application, then imports `main.tsx`.

Evidence anchors:

- main fast-path setup: `src/entrypoints/cli.tsx:33`
- daemon-worker branch: `src/entrypoints/cli.tsx:100`
- bridge fast path: `src/entrypoints/cli.tsx:112`
- background sessions: `src/entrypoints/cli.tsx:185`
- template handling: `src/entrypoints/cli.tsx:212`
- worktree/tmux path: `src/entrypoints/cli.tsx:248`
- main import: `src/entrypoints/cli.tsx:288`

Key boot behaviors:

- Disables Corepack auto-pin behavior.
- Handles `--version` cheaply.
- Handles native host / MCP / daemon / bridge / background-session / template / BYOC/self-hosted / worktree-tmux special modes before importing the main app.
- Starts early input capture.
- Uses dynamic import to defer heavy startup cost.

### `src/entrypoints/init.ts`

This file performs initialization that must happen safely and in the right order.

Evidence anchors:

- init function: `src/entrypoints/init.ts:58`
- safe environment application: `src/entrypoints/init.ts:72`
- first-party logging/GrowthBook handling: `src/entrypoints/init.ts:101`
- mTLS/global HTTP setup: `src/entrypoints/init.ts:132`
- API preconnect: `src/entrypoints/init.ts:145`
- upstream proxy for remote mode: `src/entrypoints/init.ts:154`
- telemetry after trust: `src/entrypoints/init.ts:247`

Important design points:

- Some environment and trust-sensitive setup happens before full runtime initialization.
- Telemetry is initialized after workspace trust.
- Remote managed settings/policy promises are started early.
- Global HTTP/proxy/mTLS behavior is configured centrally.
- LSP cleanup and shutdown hooks are registered during init.

### `src/main.tsx`

This is the primary CLI command tree and runtime setup file.

Evidence anchors:

- top-level `program` setup: `src/main.tsx:968`
- permission mode initialization: `src/main.tsx:1392`
- worktree/team/remote flags: `src/main.tsx:3811`
- main parse: `src/main.tsx:3887`
- MCP commands: `src/main.tsx:3894`
- server command: `src/main.tsx:3962`
- SSH command: `src/main.tsx:4046`
- auth commands: `src/main.tsx:4100`
- plugin commands: `src/main.tsx:4148`
- agent listing: `src/main.tsx:4278`
- auto-mode diagnostics: `src/main.tsx:4289`
- doctor/update/install/rollback/internal commands: `src/main.tsx:4346+`

Supported execution modes include:

- interactive TUI
- `--print` non-interactive mode
- JSON and stream-JSON modes
- SDK URL streaming
- bare mode
- MCP server mode
- remote-control mode
- SSH remote mode
- worktree/tmux sessions
- assistant/KAIROS daemon mode
- team/agent identity modes
- plugin and marketplace management
- auth login/logout/status
- doctor/update/rollback/install
- internal logs/export/task/completion

---

## 5. CLI and slash command surface

The repo has two major command layers.

### 5.1 Commander subcommands in `main.tsx`

Major subcommands:

- `mcp`
- `server`
- `ssh`
- `open`
- `auth`
- `plugin` / `plugins`
- `setup-token`
- `agents`
- `auto-mode`
- `remote-control`
- `assistant`
- `doctor`
- `update` / `upgrade`
- `rollback`
- `install`
- internal `log`, `error`, `export`, `task`, `completion`

### 5.2 Slash/local commands in `src/commands.ts` and `src/commands/`

Evidence anchors:

- internal-only command registry: `src/commands.ts:224`
- built-in command registry: `src/commands.ts:258`
- command loading pipeline: `src/commands.ts:445`
- runtime command filtering: `src/commands.ts:476`
- model-invocable skill commands: `src/commands.ts:563`
- slash skill extraction: `src/commands.ts:586`
- remote-safe commands: `src/commands.ts:610`
- bridge-safe commands: `src/commands.ts:651`

Command categories:

- **Workspace/session:** `clear`, `compact`, `context`, `cost`, `diff`, `resume`, `session`, `status`, `stats`
- **Setup/config:** `config`, `doctor`, `mcp`, `memory`, `model`, `output-style`, `permissions`, `theme`
- **Dev workflows:** `review`, `security-review`, `rewind`, `branch`, `commit`, `issue`, `pr_comments`
- **UX modes:** `vim`, `keybindings`, `voice`, `statusline`
- **Extensions:** `plugin`, `reload-plugins`, `skills`
- **Remote/mobile/desktop/chrome:** `remote-env`, `mobile`, `desktop`, `chrome`
- **Internal:** bughunter, ultrareview, ultraplan, perf issue, mock limits, reset limits, telemetry/debug commands, etc.

Command loading is layered: bundled skills, built-in plugin skills, skill directories, workflows, plugin commands, plugin skills, and built-ins are composed, then filtered by availability and mode.

---

## 6. Conversation engine

### `src/QueryEngine.ts`

`QueryEngine` is the per-conversation orchestration wrapper around the lower-level streaming query loop.

Evidence anchors:

- config type: `src/QueryEngine.ts:130`
- class declaration: `src/QueryEngine.ts:184`
- `submitMessage`: `src/QueryEngine.ts:209`
- input persistence/pre-query processing: `src/QueryEngine.ts:410`
- permission context allowed-tools handling: `src/QueryEngine.ts:476`
- skills/plugins cache path: `src/QueryEngine.ts:529`
- local-command no-model path: `src/QueryEngine.ts:556`
- file-history snapshot: `src/QueryEngine.ts:641`
- query loop invocation: `src/QueryEngine.ts:675`
- transcript writes: `src/QueryEngine.ts:687`
- stream/result handling: `src/QueryEngine.ts:757`
- budget cap: `src/QueryEngine.ts:971`
- structured-output retry: `src/QueryEngine.ts:1004`
- final result/error handling: `src/QueryEngine.ts:1051`

Responsibilities:

- Accept user messages.
- Run command parsing and local command short-circuits.
- Build effective system/user context.
- Build tool and permission context.
- Load skills/plugins/agents.
- Invoke the streaming query loop.
- Persist user/assistant/compact boundaries.
- Track token budgets, structured output, retries, model fallback, and max turns.
- Handle orphaned permission requests.
- Return final assistant results.

### `src/query.ts`

This is the streaming loop that directly manages model streaming and tool execution.

Evidence anchors:

- `query()` wrapper: `src/query.ts:219`
- `queryLoop` start: `src/query.ts:241`
- token/task budget setup: `src/query.ts:280`
- skill discovery prefetch: `src/query.ts:323`
- stream request start: `src/query.ts:337`
- tool block backfill/output handling: `src/query.ts:742`
- recoverable error withholding: `src/query.ts:788`
- tool execution queueing: `src/query.ts:837`
- fallback model handling: `src/query.ts:893`
- missing tool result recovery: `src/query.ts:955`
- post-sampling hooks: `src/query.ts:999`
- abort cleanup: `src/query.ts:1011`
- prompt-too-long / compaction recovery: `src/query.ts:1062`

Core loop behavior:

1. Build request state.
2. Estimate/manage token budget.
3. Optionally discover skills.
4. Start model streaming request.
5. As assistant tool-use blocks arrive, queue tool execution.
6. Preserve observable tool inputs separately from API-bound originals.
7. Ensure every tool-use has a paired tool-result, including errors/recovery.
8. Run post-sampling hooks.
9. Handle compaction and prompt-too-long recovery.
10. Optionally fall back to another model.
11. Yield assistant/user/tool events to REPL or headless caller.

Foundry lesson: tool-use/tool-result pairing should be a protocol invariant, including failure paths.

---

## 7. Tool/capability architecture

### 7.1 Core tool contract: `src/Tool.ts`

Evidence anchors:

- `ToolPermissionContext`: `src/Tool.ts:122`
- `ToolUseContext`: `src/Tool.ts:158`
- core Tool interface: `src/Tool.ts:362`
- `buildTool` defaulting: `src/Tool.ts:783`

A tool can define:

- name and aliases
- description and prompt section
- input schema / JSON schema
- output schema
- result mapping
- validation
- permission checks
- read-only/destructive/concurrent/open-world markers
- user-interaction marker
- MCP/LSP/deferred/always-load markers
- rendering functions
- grouping and UI tags
- permission matcher preparation
- auto-classifier input
- max result size
- strictness

This is one of the strongest architectural patterns in the repo: safety metadata is attached to capability definitions rather than only living in prompts.

### 7.2 Tool registry: `src/tools.ts`

Evidence anchors:

- base tools: `src/tools.ts:193`
- deny-rule filtering: `src/tools.ts:253`
- runtime tool filtering: `src/tools.ts:271`
- tool pool assembly: `src/tools.ts:329`
- merged tool list: `src/tools.ts:369`

Base tool families include:

- Agent/subagent tools
- Bash and PowerShell
- Glob/Grep
- File read/edit/write
- Web fetch/search
- Todo
- Plan/worktree tools
- Config
- Task/team tools
- MCP resource/tools
- LSP
- REPL primitives
- Remote trigger
- Cron/sleep
- Brief/user-message tools
- Tool search
- Testing permission tool

Important registry properties:

- Built-ins and MCP tools are composed into a common pool.
- Deny rules can remove tools before the model sees them.
- Built-in tools win name collisions.
- Built-ins and MCP tools are sorted separately for prompt-cache stability.
- Bare/SIMPLE modes narrow available tools.

Foundry lesson: do not expose a prohibited capability to the model and rely on later denial. Remove it before prompt assembly.

---

## 8. Tool execution lifecycle

Primary file: `src/services/tools/toolExecution.ts`

Evidence anchors:

- `runToolUse`: `src/services/tools/toolExecution.ts:337`
- input validation: `src/services/tools/toolExecution.ts:614`
- internal field stripping/speculative classifier: `src/services/tools/toolExecution.ts:734`
- PreToolUse hooks: `src/services/tools/toolExecution.ts:795`
- permission decision: `src/services/tools/toolExecution.ts:916`
- denial handling: `src/services/tools/toolExecution.ts:995`
- tool call: `src/services/tools/toolExecution.ts:1207`
- large/sanitized output logging: `src/services/tools/toolExecution.ts:1226`
- success telemetry: `src/services/tools/toolExecution.ts:1331`
- post hooks/result assembly: `src/services/tools/toolExecution.ts:1397`

Lifecycle:

1. Find matching tool by name or alias.
2. Return structured error result for unknown tools.
3. Abort cleanly if request is canceled.
4. Parse input using tool schema.
5. Run tool-specific validation.
6. Strip internal-only fields.
7. Run PreToolUse hooks.
8. Ask permission pipeline.
9. If denied, optionally run PermissionDenied hooks and retry logic.
10. Execute tool call.
11. Persist/preview large outputs.
12. Sanitize/log output.
13. Run PostToolUse/PostToolUseFailure hooks.
14. Return tool result block.

Foundry lesson: tool execution should be a policy sandwich: validation, hooks, permission, execution, output handling, hooks, audit.

---

## 9. Permission model and guardrails

### 9.1 Permission modes

File: `src/utils/permissions/PermissionMode.ts`

Evidence anchors:

- mode schema/config: `src/utils/permissions/PermissionMode.ts:22`
- mode configs: `src/utils/permissions/PermissionMode.ts:43`
- external-mode guard: `src/utils/permissions/PermissionMode.ts:94`

Modes include:

- `default`
- `plan`
- `acceptEdits`
- `bypassPermissions`
- `dontAsk`
- feature-gated `auto`

Important point: `auto` is not equivalent to bypass. It is gated, classifier-backed, and strips dangerous persisted grants.

### 9.2 Permission setup

File: `src/utils/permissions/permissionSetup.ts`

Evidence anchors:

- dangerous Bash rule detection: `src/utils/permissions/permissionSetup.ts:84`
- dangerous PowerShell rule detection: `src/utils/permissions/permissionSetup.ts:149`
- dangerous Agent allow-rule detection: `src/utils/permissions/permissionSetup.ts:235`
- dangerous classifier permissions: `src/utils/permissions/permissionSetup.ts:287`
- dangerous permission removal: `src/utils/permissions/permissionSetup.ts:468`
- auto-mode strip/restore: `src/utils/permissions/permissionSetup.ts:505`
- permission mode transition: `src/utils/permissions/permissionSetup.ts:581`
- CLI initial mode: `src/utils/permissions/permissionSetup.ts:687`
- permission context initialization: `src/utils/permissions/permissionSetup.ts:872`

Key protections:

- Broad shell grants are stripped in auto mode.
- Dangerous Bash/PowerShell patterns are not inherited blindly.
- Broad Agent grants are treated as dangerous.
- Settings, CLI, and session rules merge into a permission context.
- Bypass can be disabled by config/feature gates.
- Base tools can become a deny-everything-not-listed policy.
- Additional directories are explicitly tracked.
- Symlink/current-directory issues are handled.

### 9.3 Runtime permission evaluation

File: `src/utils/permissions/permissions.ts`

Evidence anchors:

- rule sources: `src/utils/permissions/permissions.ts:109`
- MCP/server-level rule matching: `src/utils/permissions/permissions.ts:233`
- Agent deny filtering: `src/utils/permissions/permissions.ts:304`
- main permission wrapper: `src/utils/permissions/permissions.ts:473`
- auto-classifier path: `src/utils/permissions/permissions.ts:560`
- no-prompt handling: `src/utils/permissions/permissions.ts:929`
- rule-based checks: `src/utils/permissions/permissions.ts:1071`
- inner permission flow: `src/utils/permissions/permissions.ts:1158`
- denial threshold handling: `src/utils/permissions/permissions.ts:984`

Simplified evaluation order:

1. explicit deny rules
2. explicit ask rules
3. tool-specific validation/checks
4. tool-level deny/ask
5. content-specific ask
6. safety checks
7. permission mode handling
8. whole-tool allow
9. passthrough to ask/deny depending on mode and prompt availability

Auto mode includes:

- classifier decision
- safe allowlist fast paths
- transcript-too-long handling
- classifier unavailable behavior
- denial tracking
- safety-check immunity for specific checks

Foundry lesson: permission should be layered and compositional, not a single allow/deny function.

---

## 10. Shell guardrails

Shell execution is the most deeply defended part of the repo.

### 10.1 Bash tool

File: `src/tools/BashTool/BashTool.tsx`

Evidence anchors:

- input schema: `src/tools/BashTool/BashTool.tsx:227`
- simulated sed edit flow: `src/tools/BashTool/BashTool.tsx:355`
- tool definition: `src/tools/BashTool/BashTool.tsx:420`
- input validation: `src/tools/BashTool/BashTool.tsx:524`
- permission delegation: `src/tools/BashTool/BashTool.tsx:539`
- result mapping: `src/tools/BashTool/BashTool.tsx:555`
- tool call: `src/tools/BashTool/BashTool.tsx:624`
- shell command runner: `src/tools/BashTool/BashTool.tsx:826`

Capabilities:

- command, timeout, description, background execution
- background tasks
- sandbox hints
- image output handling
- large-output persistence
- git-operation tracking
- index-lock detection
- cwd reset if shell exits outside project
- sandbox violation annotation
- sleep/polling guidance toward background/monitor tools

### 10.2 Bash permission parser

File: `src/tools/BashTool/bashPermissions.ts`

Evidence anchors:

- AST/tree-sitter path: `src/tools/BashTool/bashPermissions.ts:1661`
- shadow telemetry: `src/tools/BashTool/bashPermissions.ts:1701`
- too-complex/semantic failure handling: `src/tools/BashTool/bashPermissions.ts:1741`
- sandbox auto-allow: `src/tools/BashTool/bashPermissions.ts:1829`
- exact/prompt classifier handling: `src/tools/BashTool/bashPermissions.ts:1845`
- operator/pipe validation: `src/tools/BashTool/bashPermissions.ts:1973`
- legacy/heredoc handling: `src/tools/BashTool/bashPermissions.ts:2078`
- subcommand fanout: `src/tools/BashTool/bashPermissions.ts:2144`
- redirection validation: `src/tools/BashTool/bashPermissions.ts:2227`
- final allow/suggestion path: `src/tools/BashTool/bashPermissions.ts:2333`

### 10.3 Bash read-only validation

File: `src/tools/BashTool/readOnlyValidation.ts`

Evidence anchors:

- strict flag parsing: `src/tools/BashTool/readOnlyValidation.ts:1241`
- `git ls-remote` URL rejection: `src/tools/BashTool/readOnlyValidation.ts:1306`
- runtime expansion rejection: `src/tools/BashTool/readOnlyValidation.ts:1328`
- read-only constraint function: `src/tools/BashTool/readOnlyValidation.ts:1867`

Defenses include:

- UNC path rejection
- runtime variable/glob rejection in read-only contexts
- git global flag checks
- bare repo structure checks
- git-internal path protection
- compound `cd && git` handling
- fail-closed behavior for unrecognized shapes

### 10.4 Bash security validator

File: `src/tools/BashTool/bashSecurity.ts`

Defended categories include:

- heredoc substitution
- git commit parsing
- dangerous variables in pipes/redirections
- backticks and escaped operators
- obfuscated flags
- brace expansion
- control characters
- tree-sitter versus regex quote-context differences

### 10.5 PowerShell parallel

Relevant files:

- `src/tools/PowerShellTool/PowerShellTool.tsx`
- `src/tools/PowerShellTool/powershellPermissions.ts`
- `src/tools/PowerShellTool/powershellSecurity.ts`
- `src/tools/PowerShellTool/readOnlyValidation.ts`
- `src/tools/PowerShellTool/pathValidation.ts`
- `src/tools/PowerShellTool/gitSafety.ts`
- `src/tools/PowerShellTool/commonParameters.ts`
- `src/tools/PowerShellTool/clmTypes.ts`

Evidence anchors:

- main PowerShell permission checker: `src/tools/PowerShellTool/powershellPermissions.ts:627`
- parse-failed fallback defenses: `src/tools/PowerShellTool/powershellPermissions.ts:759`
- PowerShell read-only checks: `src/tools/PowerShellTool/readOnlyValidation.ts:1162`
- git safety/runtime-variable rejection: `src/tools/PowerShellTool/readOnlyValidation.ts:1519+`

Foundry lesson: shell tools require language-specific security modules. Prompt-only safety or generic regex checks are insufficient.

---

## 11. Filesystem, search, web, and LSP tools

### 11.1 File tools

`src/tools/FileReadTool/FileReadTool.ts`

- tool definition: line ~337
- read-only marker: line ~376
- permission check: line ~398
- input validation: line ~418
- call path: line ~496

`src/tools/FileEditTool/FileEditTool.ts`

- tool definition: line ~86
- permission check: line ~125
- validation: line ~137
- settings-file edit validation: line ~346
- call path: line ~387

`src/tools/FileWriteTool/FileWriteTool.ts`

- tool definition: line ~94
- permission check: line ~135
- validation: line ~153
- call path: line ~223

Patterns:

- file read has dedup/state awareness
- edit/write capture pre-edit backups
- settings edits receive schema validation
- permission checks are separate from input validation
- rendering is tool-specific

### 11.2 Search tools

`src/tools/GlobTool/GlobTool.ts`

- tool definition: line ~57
- read-only marker: line ~79
- validation: line ~94
- permission: line ~135
- call: line ~154

`src/tools/GrepTool/GrepTool.ts`

- tool definition: line ~160
- read-only marker: line ~186
- validation: line ~201
- permission: line ~233
- call: line ~310

### 11.3 Web tools

`src/tools/WebFetchTool/WebFetchTool.ts`

- input schema: line ~24
- tool definition: line ~66
- read-only marker: line ~98
- permission: line ~104
- validation: line ~191
- call: line ~208

`src/tools/WebSearchTool/WebSearchTool.ts`

- input schema: line ~25
- tool definition: line ~152
- read-only marker: line ~203
- permission: line ~209
- validation: line ~235
- call: line ~254

### 11.4 LSP tool

`src/tools/LSPTool/LSPTool.ts`

- input schema: line ~59
- tool definition: line ~127
- read-only marker: line ~149
- validation: line ~155
- permission: line ~210
- call: line ~224

Foundry lesson: read-only does not mean policy-free. Even read-only tools need path/source/data-class controls and output limits.

---

## 12. Agents, subagents, teams, and tasks

### 12.1 AgentTool

File: `src/tools/AgentTool/AgentTool.tsx`

Evidence anchors:

- base schema: `src/tools/AgentTool/AgentTool.tsx:82`
- full schema with team/isolation params: `src/tools/AgentTool/AgentTool.tsx:90`
- gated schema stripping: `src/tools/AgentTool/AgentTool.tsx:110`
- tool definition: `src/tools/AgentTool/AgentTool.tsx:196`
- team spawn branch: `src/tools/AgentTool/AgentTool.tsx:282`
- fork-subagent routing: `src/tools/AgentTool/AgentTool.tsx:318`
- remote isolation branch: `src/tools/AgentTool/AgentTool.tsx:430`
- prompt construction: `src/tools/AgentTool/AgentTool.tsx:483`
- async/assistant forcing: `src/tools/AgentTool/AgentTool.tsx:555`
- worker permission context: `src/tools/AgentTool/AgentTool.tsx:568`
- permission check: `src/tools/AgentTool/AgentTool.tsx:1281`

Capabilities:

- spawn specialized subagents
- optional model override
- optional background execution
- optional team name/name for teammate spawn
- optional worktree isolation
- internal-only remote isolation path
- fork mode where child inherits parent conversation/system prompt under a feature gate
- recursive-fork prevention
- teammate nesting prevention
- in-process teammate background-spawn prevention

### 12.2 Agent runtime

File: `src/tools/AgentTool/runAgent.ts`

Evidence anchors:

- `runAgent`: `src/tools/AgentTool/runAgent.ts:248`
- fork context messages: `src/tools/AgentTool/runAgent.ts:368`
- background prompt/check behavior: `src/tools/AgentTool/runAgent.ts:453`
- cache-safe/fork prompt handling: `src/tools/AgentTool/runAgent.ts:679`
- cleanup of fork messages/background bash: `src/tools/AgentTool/runAgent.ts:829`

### 12.3 Agent definitions

File: `src/tools/AgentTool/loadAgentsDir.ts`

Evidence anchors:

- agent schema fields: `src/tools/AgentTool/loadAgentsDir.ts:80`
- active agent set: `src/tools/AgentTool/loadAgentsDir.ts:180`
- background frontmatter parsing: `src/tools/AgentTool/loadAgentsDir.ts:575`
- isolation parsing: `src/tools/AgentTool/loadAgentsDir.ts:607`

Agent definitions support frontmatter, background behavior, worktree/remote isolation, model and prompt metadata, built-ins, and user/project/plugin sources.

### 12.4 Team spawning

File: `src/tools/shared/spawnMultiAgent.ts`

Evidence anchors:

- permission propagation: `src/tools/shared/spawnMultiAgent.ts:200`
- split-pane/tmux teammate creation: `src/tools/shared/spawnMultiAgent.ts:302`
- in-process teammate creation: `src/tools/shared/spawnMultiAgent.ts:837`
- public `spawnTeammate`: `src/tools/shared/spawnMultiAgent.ts:1088`

Team backends:

- tmux/iTerm split-pane mode
- separate tmux swarm window/session mode
- in-process teammate mode using AsyncLocalStorage
- mailbox delivery for tmux teammates
- direct prompt delivery for in-process teammates
- background task registration for teammate visibility

Safety point: plan mode can override inherited bypass permissions for teammates.

### 12.5 Task tools and task runtime

Tool folders:

- `TaskCreateTool`
- `TaskGetTool`
- `TaskListTool`
- `TaskOutputTool`
- `TaskStopTool`
- `TaskUpdateTool`

Task types:

- DreamTask
- InProcessTeammateTask
- LocalAgentTask
- LocalMainSessionTask
- LocalShellTask
- RemoteAgentTask

Foundry lesson: agent execution should produce task objects with lifecycle, output paths, cancellation, ownership, and UI/audit visibility.

---

## 13. Skills and plugins

### 13.1 Skills

Relevant paths:

- `src/skills/`
- `src/tools/SkillTool/SkillTool.ts`
- `src/commands/skills/`

Evidence anchors:

- SkillTool definition: `src/tools/SkillTool/SkillTool.ts:331`
- SkillTool input schema: `src/tools/SkillTool/SkillTool.ts:291`
- SkillTool permission check: `src/tools/SkillTool/SkillTool.ts:432`
- SkillTool call path: `src/tools/SkillTool/SkillTool.ts:580`
- skill hook registration note: `src/tools/SkillTool/SkillTool.ts:761`
- skill search feature gate: `src/tools/SkillTool/SkillTool.ts:966`

Skills can be bundled, plugin-provided, directory-loaded, model-invocable, hook-providing, and feature-gated for remote/MCP discovery.

### 13.2 Plugins

Relevant paths:

- `src/plugins/builtinPlugins.ts`
- `src/types/plugin.ts`
- `src/utils/plugins/`
- `src/services/plugins/`
- `src/commands/plugin/`

Evidence anchors:

- plugin manifest/type model: `src/types/plugin.ts`
- built-in plugin registry: `src/plugins/builtinPlugins.ts:23`
- built-in plugin command exposure: `src/plugins/builtinPlugins.ts:105`
- background plugin installation: `src/services/plugins/PluginInstallationManager.ts:51`
- plugin manifest loading: `src/utils/plugins/pluginLoader.ts:1147`
- plugin cache clearing: `src/utils/plugins/pluginLoader.ts:3225`

Plugin surfaces include:

- commands
- skills
- hooks
- MCP servers
- plugin settings
- marketplace metadata
- built-in plugin registry
- enable/disable/update/install/uninstall flows
- background marketplace materialization and refresh

Enterprise/security controls include plugin-only customization, marketplace allowlists, plugin-root/data substitution in hook execution, and managed policy gates.

Foundry lesson: plugins are bundles of capabilities, skills, hooks, MCP, and config—not just code packages.

---

## 14. Hooks

### 14.1 Hook schema

File: `src/types/hooks.ts`

Evidence anchors:

- hook event type utilities: `src/types/hooks.ts:22`
- prompt request/response schema: `src/types/hooks.ts:26`
- sync hook response schema: `src/types/hooks.ts:50`
- async hook response schema: `src/types/hooks.ts:171`
- callback context: `src/types/hooks.ts:202`
- permission behavior fields: `src/types/hooks.ts:267`

Hook events include:

- `PreToolUse`
- `PostToolUse`
- `PostToolUseFailure`
- `PermissionDenied`
- `PermissionRequest`
- `UserPromptSubmit`
- `SessionStart`
- `SessionEnd`
- `Setup`
- `Stop`
- `SubagentStart`
- `SubagentStop`
- `CwdChanged`
- `FileChanged`
- `Elicitation`
- `ElicitationResult`
- `WorktreeCreate`

Hook outputs can:

- add context
- deny/allow/ask permissions
- update tool input
- update MCP tool output
- retry after permission denial
- provide elicitation responses
- watch paths
- block with reason
- suppress output
- continue/stop

### 14.2 Hook runtime

File: `src/utils/hooks.ts`

Evidence anchors:

- base hook input: `src/utils/hooks.ts:301`
- HookResult type: `src/utils/hooks.ts:338`
- JSON output processing: `src/utils/hooks.ts:550`
- shell execution setup: `src/utils/hooks.ts:760`
- env file handling: `src/utils/hooks.ts:919`
- matcher logic: `src/utils/hooks.ts:1337`
- permission `if` matcher: `src/utils/hooks.ts:1390`
- hook config assembly: `src/utils/hooks.ts:1492`
- event matching: `src/utils/hooks.ts:1603`
- core execute loop: `src/utils/hooks.ts:1952`
- PreToolUse execution: `src/utils/hooks.ts:3410`
- PostToolUse execution: `src/utils/hooks.ts:3460`
- PermissionDenied execution: `src/utils/hooks.ts:3529`
- UserPromptSubmit execution: `src/utils/hooks.ts:3826`
- SessionStart execution: `src/utils/hooks.ts:3867`
- Setup execution: `src/utils/hooks.ts:3902`
- SubagentStart execution: `src/utils/hooks.ts:3932`
- SessionEnd execution: `src/utils/hooks.ts:4097`
- PermissionRequest execution: `src/utils/hooks.ts:4157`
- Cwd/File changed execution: `src/utils/hooks.ts:4260`
- Elicitation execution: `src/utils/hooks.ts:4470`

Hook runtime supports command hooks, HTTP hooks, function hooks, plugin hooks, skill hooks, managed-policy-only hooks, event-specific matching, hook environment variables, `CLAUDE_ENV_FILE`, timeouts, async-first-line detection, prompt requests from hook stdout, blocking errors, and trust-aware execution.

Foundry lesson: hooks should be a typed extension ABI with policy controls, not arbitrary lifecycle scripts.

---

## 15. Settings and managed policy

Primary file: `src/utils/settings/types.ts`

Evidence anchors:

- environment variable schema: `src/utils/settings/types.ts:33`
- permission schema: `src/utils/settings/types.ts:42`
- MCP allowlist schema: `src/utils/settings/types.ts:112`
- denied MCP schema: `src/utils/settings/types.ts:161`
- plugin-only locked surfaces: `src/utils/settings/types.ts:244`
- main settings schema: `src/utils/settings/types.ts:262`
- permissions config: `src/utils/settings/types.ts:372`
- model allowlist: `src/utils/settings/types.ts:379`
- MCP allow/deny: `src/utils/settings/types.ts:416`
- hook disable/managed-only/HTTP allowlist: `src/utils/settings/types.ts:458`
- managed permission rules only: `src/utils/settings/types.ts:501`
- managed MCP servers only: `src/utils/settings/types.ts:509`
- strict plugin-only customization: `src/utils/settings/types.ts:517`
- enabled plugins: `src/utils/settings/types.ts:559`
- marketplace allowlists: `src/utils/settings/types.ts:601`
- auto-mode classifier config: `src/utils/settings/types.ts:984`

Managed settings can control:

- hook execution
- HTTP hook URL allowlists
- HTTP hook env var interpolation
- permission rules
- MCP allow/deny
- model allowlists
- plugin customization surfaces
- marketplace sources
- auto-mode classifier allow/soft-deny/environment config
- remote session policy
- channel plugin allowlists

Remote managed settings live under `src/services/remoteManagedSettings/`. Policy limits live under `src/services/policyLimits/index.ts` and include fail-open/fail-closed distinctions.

Foundry lesson: separate user, project, local, managed, remote-managed, and policy-limit sources. Make precedence explicit.

---

## 16. MCP integration

Relevant paths:

- `src/services/mcp/`
- `src/tools/MCPTool/`
- `src/tools/McpAuthTool/`
- `src/tools/ListMcpResourcesTool/`
- `src/tools/ReadMcpResourceTool/`
- `src/commands/mcp/`

### 16.1 MCP config

File: `src/services/mcp/config.ts`

Evidence anchors:

- enterprise MCP path: `src/services/mcp/config.ts:62`
- server signature: `src/services/mcp/config.ts:202`
- plugin MCP dedup: `src/services/mcp/config.ts:223`
- Claude.ai MCP dedup: `src/services/mcp/config.ts:281`
- policy filtering: `src/services/mcp/config.ts:536`
- project config: `src/services/mcp/config.ts:843`
- scope loading: `src/services/mcp/config.ts:888`
- config by name: `src/services/mcp/config.ts:1033`
- approved project servers: `src/services/mcp/config.ts:1164`
- config parsing: `src/services/mcp/config.ts:1297`
- managed-only MCP policy: `src/services/mcp/config.ts:1485`
- server disabled/enabled: `src/services/mcp/config.ts:1528`

MCP scopes:

- project
- user
- local
- enterprise
- Claude.ai
- plugin-provided
- dynamic

### 16.2 MCP connection manager/tool wrapper

Evidence anchors:

- manager component: `src/services/mcp/MCPConnectionManager.tsx:38`
- `useManageMCPConnections`: `src/services/mcp/MCPConnectionManager.tsx:48`
- MCP tool input schema: `src/tools/MCPTool/MCPTool.ts:14`
- MCP tool definition: `src/tools/MCPTool/MCPTool.ts:27`
- `isMcp` marker: `src/tools/MCPTool/MCPTool.ts:28`
- permission hook: `src/tools/MCPTool/MCPTool.ts:56`

MCP design:

- External MCP tools are wrapped into the same internal Tool interface.
- MCP resources get dedicated list/read tools.
- MCP auth gets a dedicated tool.
- Project MCP servers require approval/rejection tracking.
- Enterprise policy can allow/deny MCP.
- Plugin MCP servers are deduped and policy-filtered.

Foundry lesson: normalize every external protocol into one capability lifecycle.

---

## 17. LSP and diagnostics

Relevant paths:

- `src/services/lsp/`
- `src/tools/LSPTool/`

Evidence anchors:

- LSP manager factory: `src/services/lsp/LSPServerManager.ts:59`
- passive diagnostics formatting: `src/services/lsp/passiveFeedback.ts:43`
- diagnostic handler registration: `src/services/lsp/passiveFeedback.ts:125`
- failure isolation: `src/services/lsp/passiveFeedback.ts:131`
- async diagnostic registration: `src/services/lsp/passiveFeedback.ts:207`

Design:

- LSP servers are centrally managed.
- Diagnostics are converted into Claude attachment format.
- Failures are isolated per server.
- Repeated failures are tracked and warned.
- LSP tool provides structured code-intel calls.

Foundry lesson: language-server feedback should be passive evidence as well as an explicit tool.

---

## 18. API, auth, telemetry, analytics

Relevant paths:

- `src/services/api/`
- `src/services/oauth/`
- `src/services/policyLimits/`
- `src/services/remoteManagedSettings/`
- `src/services/analytics/`

Evidence anchors:

- OAuth flow class: `src/services/oauth/index.ts:21`
- OAuth PKCE/code flow: `src/services/oauth/index.ts:49`
- policy eligibility: `src/services/policyLimits/index.ts:167`
- policy fetch/retry: `src/services/policyLimits/index.ts:265`
- policy allowed check: `src/services/policyLimits/index.ts:510`
- policy background polling: `src/services/policyLimits/index.ts:633`
- analytics sink attachment: `src/services/analytics/index.ts:95`
- event logging: `src/services/analytics/index.ts:133`
- telemetry-after-trust: `src/entrypoints/init.ts:247`

Patterns:

- analytics events queue until sink attachment
- telemetry is delayed until trust
- proto/internal fields are stripped for non-first-party sinks
- policy limits use cache and background polling
- OAuth flow uses PKCE-style authorization code handling
- remote managed settings are fetched early and synchronized into runtime state

Foundry lesson: trust-before-telemetry and explicit telemetry sink separation should be preserved.

---

## 19. Memory, context, compaction, and session storage

### 19.1 Context injection

File: `src/context.ts`

Evidence anchors:

- git status collection: `src/context.ts:36`
- system context: `src/context.ts:116`
- user context / CLAUDE.md memory: `src/context.ts:155`
- memory disable/bare logic: `src/context.ts:162`
- current date injection: `src/context.ts:186`

Context includes:

- git branch/default branch/status/recent commits
- git user name
- CLAUDE/AGENTS-style memory files
- current date
- optional cache-breaker injection

### 19.2 Session storage

File: `src/utils/sessionStorage.ts`

Evidence anchors:

- transcript path: `src/utils/sessionStorage.ts:202`
- transcript path for current session: `src/utils/sessionStorage.ts:207`
- transcript-message type guard: `src/utils/sessionStorage.ts:139`
- chain participant logic: `src/utils/sessionStorage.ts:154`
- ephemeral progress filtering: `src/utils/sessionStorage.ts:186`
- transcript loading: `src/utils/sessionStorage.ts:3472`
- subagent transcript loading: `src/utils/sessionStorage.ts:4297`

Design points:

- JSONL transcripts
- parent UUID chain
- progress messages excluded from durable chain
- legacy progress bridging during load
- ephemeral progress not persisted as conversation state
- active session project dir honored
- large/tombstone cases considered

### 19.3 Compaction

Relevant files:

- `src/services/compact/autoCompact.ts`
- `src/services/compact/compact.ts`
- `src/services/compact/microCompact.ts`
- `src/services/compact/postCompactCleanup.ts`
- `src/services/compact/sessionMemoryCompact.ts`

Evidence anchors:

- auto-compact threshold: `src/services/compact/autoCompact.ts:104`
- auto-compact execution: `src/services/compact/autoCompact.ts:241`
- main compact function: `src/services/compact/compact.ts:387`
- PreCompact hooks: `src/services/compact/compact.ts:407`
- summary request: `src/services/compact/compact.ts:441`
- boundary marker handling: `src/services/compact/compact.ts:596`
- SessionStart after compaction: `src/services/compact/compact.ts:588`
- PostCompact hooks: `src/services/compact/compact.ts:720`

Compaction design:

- preserves boundaries
- creates summary messages
- keeps selected tail messages
- handles attachments
- integrates hooks
- handles prompt-too-long retries
- tracks pre-compact discovered tools
- supports prefix/suffix preserving modes

### 19.4 Session memory

File: `src/services/SessionMemory/sessionMemory.ts`

Evidence anchors:

- design comment: `src/services/SessionMemory/sessionMemory.ts:3`
- threshold logic: `src/services/SessionMemory/sessionMemory.ts:146`
- memory file setup: `src/services/SessionMemory/sessionMemory.ts:183`
- extraction hook: `src/services/SessionMemory/sessionMemory.ts:267`
- manual extraction: `src/services/SessionMemory/sessionMemory.ts:387`

Pattern:

- background forked subagent extracts durable session notes
- extraction waits for token and tool-call thresholds
- excessive extraction is avoided
- file read/edit tools manipulate the session memory file

Foundry lesson: keep compact summaries, session memory, project memory, tenant memory, and audit evidence as separate artifacts.

---

## 20. UI, input, keybindings, Vim, voice

### 20.1 UI stack

Major paths:

- `src/components/`
- `src/screens/REPL.tsx`
- `src/components/PromptInput/PromptInput.tsx`
- `src/ink/`
- `src/hooks/`

UI components cover:

- app shell
- REPL
- prompt input
- message rendering
- virtual message list
- markdown/code highlighting
- permission dialogs
- file diffs
- settings/config screens
- MCP approval dialogs
- plugin management
- model picker
- theme picker
- task list
- team/teammate views
- statusline
- trust dialog
- onboarding
- remote/teleport dialogs
- voice controls
- Vim input

### 20.2 Keybindings

Relevant paths:

- `src/keybindings/defaultBindings.ts`
- `src/keybindings/parser.ts`
- `src/keybindings/validate.ts`
- `src/keybindings/template.ts`

Evidence anchors:

- keybinding parser: `src/keybindings/parser.ts:189`
- validation types: `src/keybindings/validate.ts:14`
- validation function: `src/keybindings/validate.ts:310`
- voice binding warning: `src/keybindings/validate.ts:220`

The keybinding system supports JSON config, context-aware bindings, validation warnings/errors, voice-specific restrictions, and template generation.

### 20.3 Vim

Relevant paths:

- `src/vim/types.ts`
- `src/vim/transitions.ts`
- `src/vim/motions.ts`
- `src/vim/operators.ts`

Evidence anchors:

- Vim state machine types: `src/vim/types.ts:4`
- Vim mode state: `src/vim/types.ts:44`
- Vim memory/repeat state: `src/vim/types.ts:79`
- transitions: `src/vim/transitions.ts:443`

### 20.4 Voice

Relevant paths:

- `src/voice/voiceModeEnabled.ts`
- `src/services/voice.ts`
- `src/services/voiceStreamSTT.ts`
- `src/services/voiceKeyterms.ts`
- `src/context/voice.tsx`

Evidence anchors:

- voice kill switch: `src/voice/voiceModeEnabled.ts:9`
- OAuth requirement: `src/voice/voiceModeEnabled.ts:33`
- runtime check: `src/voice/voiceModeEnabled.ts:47`

Voice requires OAuth and feature/kill-switch availability.

Foundry lesson: permission UX should be domain-specific and evidence-rich, especially for file edits and mode transitions.

---

## 21. Bridge, remote control, server, and SSH

Relevant paths:

- `src/bridge/bridgeMain.ts`
- `src/bridge/sessionRunner.ts`
- `src/bridge/bridgeMessaging.ts`
- `src/bridge/bridgePermissionCallbacks.ts`
- `src/bridge/trustedDevice.ts`
- `src/bridge/workSecret.ts`
- `src/bridge/remoteBridgeCore.ts`
- `src/remote/`
- `src/server/`

Evidence anchors:

- session runner permission callback type: `src/bridge/sessionRunner.ts:29`
- child CLI args/session spawn: `src/bridge/sessionRunner.ts:292`
- streaming/control transport: `src/bridge/sessionRunner.ts:317`
- session completion/failure: `src/bridge/sessionRunner.ts:463`
- trusted device token source: `src/bridge/trustedDevice.ts:16`
- trusted device enrollment: `src/bridge/trustedDevice.ts:90`
- server command: `src/main.tsx:3962`
- SSH command: `src/main.tsx:4046`
- open command: `src/main.tsx:4059`

Bridge behavior:

- starts/attaches local CLI sessions to remote-control backend
- supports single-session and multi-session modes
- supports same-dir and worktree spawn modes
- supports resume by session ID or continuation pointer
- uses trusted device tokens
- supports permission callbacks from remote session to local process
- archives/deregisters sessions on shutdown except resumable modes
- can pre-create empty sessions for remote UI entry
- uses worktree isolation for concurrent remote sessions

Foundry lesson: remote agent sessions need explicit trust, auth, safe command allowlists, permission callbacks, and audit trails.

---

## 22. Testing and harnesses

The backup branch does not include a normal test/build harness. This is a critical limitation.

Observed:

- no `package.json`
- no lockfile
- no `.github/`
- no obvious test runner config
- only a few files with test-like names or testing helpers
- `src/services/vcr.ts` contains fixture/VCR support
- `src/services/mockRateLimits.ts` and related mock-limit commands exist

### VCR harness

File: `src/services/vcr.ts`

Evidence anchors:

- VCR enablement: `src/services/vcr.ts:23`
- generic fixture helper: `src/services/vcr.ts:36`
- fixture-missing behavior: `src/services/vcr.ts:71`
- API fixture VCR: `src/services/vcr.ts:88`
- streaming VCR: `src/services/vcr.ts:349`
- token-count VCR: `src/services/vcr.ts:382`

Design:

- hashes dehydrated inputs into fixture names
- supports recording with `VCR_RECORD`
- CI fails when fixtures are missing and recording is disabled
- normalizes dynamic fields for fixture stability
- supports streaming fixtures

Foundry lesson: use VCR fixtures for AI/API integrations, but keep the actual test suite, configs, and CI lanes present and enforceable.

---

## 23. Feature flags and gates

Feature gating appears throughout via `feature('...')` and GrowthBook-style values.

Observed feature/gate names include:

- `KAIROS`
- `BASH_CLASSIFIER`
- `TRANSCRIPT_CLASSIFIER`
- `TEAMMEM`
- `VOICE_MODE`
- `EXPERIMENTAL_SKILL_SEARCH`
- `BRIDGE_MODE`
- `COORDINATOR_MODE`
- `UDS_INBOX`
- `ULTRAPLAN`
- `TOKEN_BUDGET`
- `MCP_SKILLS`
- `CHICAGO_MCP`
- `CACHED_MICROCOMPACT`
- `DIRECT_CONNECT`
- `SSH_REMOTE`
- `AGENT_TRIGGERS`
- `MONITOR_TOOL`
- `WEB_BROWSER_TOOL`
- `WORKFLOW_SCRIPTS`
- `HISTORY_SNIP`
- `CONTEXT_COLLAPSE`
- `TERMINAL_PANEL`
- `DAEMON`
- `BG_SESSIONS`

Design pattern:

- product behavior is heavily feature-flagged
- some flags are internal-only
- some branches are intended for dead-code elimination
- runtime settings and feature flags interact with CLI options
- auto mode and remote/team surfaces are gated

Risk: feature-flag sprawl can create untested state combinations. Foundry should require owners, expiry dates, rollout state, test coverage, and removal plans for feature flags.

---

## 24. Security and privacy posture

Strong patterns:

1. **Trust before telemetry**
   - telemetry initialized after trust in `src/entrypoints/init.ts`

2. **Workspace trust**
   - remote-control paths check trusted workspace state

3. **Layered permissions**
   - settings, CLI, session, tool, mode, classifier, and hooks all participate

4. **No-prompt behavior**
   - headless/noninteractive paths avoid indefinite prompt waits and can auto-deny

5. **Auto mode is not bypass**
   - dangerous rules stripped
   - classifier used
   - denial thresholds tracked
   - some safety checks remain immune

6. **Shell-specific hardening**
   - Bash and PowerShell have separate parsers/validators
   - read-only allowlists reject runtime expansions and dangerous git flags
   - redirections, heredocs, UNC paths, bare git repos, and parser differentials are considered

7. **Managed policy**
   - admin settings can disable hooks, force managed-only hooks, enforce MCP allowlists, restrict plugin customization, restrict marketplace sources, and constrain auto-mode classifier config

8. **Plugin trust boundaries**
   - plugins are marketplace/installation managed, enable-disable capable, and policy-filtered

9. **MCP approval**
   - project MCP servers require trust/approval state

10. **Remote auth**
   - trusted device tokens and bridge session access tokens are used

Risks:

- monolithic files increase review difficulty
- feature flag sprawl creates combinatorial risk
- backup branch lacks CI/test harness
- inline source maps are a source-disclosure risk
- shell security is complex and depends on tests not included in the branch

---

## 25. Operational workflows present

### User workflows

- start interactive session
- run headless prompt
- resume/fork sessions
- switch models/effort
- manage permissions
- add directories
- configure MCP
- configure plugins
- load skills
- run review/security review
- compact context
- view cost/stats/status
- use Vim/keybindings/voice
- connect IDE/chrome/desktop/mobile
- export/log sessions
- update/rollback/install CLI
- auth login/logout/status

### Agent workflows

- spawn subagent
- spawn background agent
- spawn forked context agent
- spawn teammate
- spawn team
- use task tools
- enter/exit plan mode
- enter/exit worktree
- use todo tool
- schedule cron/sleep
- use MCP tools/resources
- use LSP
- use web/file/search/shell

### Admin/enterprise workflows

- managed settings sync
- remote managed policy
- policy limits
- MCP allow/deny
- hook restrictions
- marketplace restrictions
- plugin-only customization
- model allowlists
- remote-control policy

### Developer/internal workflows

- VCR fixture recording
- mock rate limits
- debug tool call
- perf issue
- bug hunter
- ultrareview/ultraplan
- heapdump
- cache break
- trace/log/error/export

---

## 26. 2026 external best-practice cross-check: token economy, harnesses, production architecture, and guardrails

This section adds a source-backed cross-check against current public guidance and research as of 2026-05-10. It should be read as clean-room design guidance, not as evidence about the inspected backup branch beyond the file anchors already documented above.

### 26.1 Earlier local experiment: no latent subagent channel

A local Codex subagent experiment in this session tested whether a parent agent can communicate unstated context to a child agent. The child agent could see only explicit prompt/context, forked context if enabled, structured input items, shared files/MCP artifacts, runtime messages, and final outputs. It could not recover an omitted private sentinel.

Finding: token saving has to come from explicit compression, shared artifacts, and careful context routing. There is no reliable hidden or latent context channel between agents.

Practical patterns:

- Put durable shared codebooks in files, not repeated prose: for example `CODEBOOK.md`, `agent-codebook.json`, or a project wiki page.
- Use short task envelopes after the codebook is established: `Task=T7; Module=M3; Constraints=C1,C4,C8; Return=R2`.
- Prefer file paths, symbol names, issue IDs, test names, and stable capability IDs over paragraph-length reminders.
- Keep the codebook small and stable. If the table churns every turn, it destroys both human readability and prompt-cache locality.
- Treat compressed codes as an ABI: version them, lint them, and include a fallback expansion for high-risk work.

### 26.2 Token-saving and context-economy findings

Current provider and practitioner guidance converges on one core rule: agent harnesses should be designed around cache-stable prefixes and explicit context compaction.

- OpenAI prompt caching is prefix-oriented; repeated/static prompt content should go first, dynamic user-specific content should go last, and `cached_tokens` / cache-hit metrics should be logged and monitored ([OpenAI prompt caching](https://developers.openai.com/api/docs/guides/prompt-caching)).
- Anthropic's Claude Code team reports that production coding agents should preserve stable prompt order, avoid changing tools or models mid-session, defer tool loading with lightweight stubs, and make compaction calls share the parent prefix ([Anthropic: Prompt caching is everything](https://claude.com/blog/lessons-from-building-claude-code-prompt-caching-is-everything)).
- Independent 2026 research on long-horizon agentic tasks found large cost and latency improvements from prompt caching, but also found that naive full-context caching can be worse than strategic cache-block control that keeps dynamic content late and excludes volatile tool results ([Don't Break the Cache](https://arxiv.org/abs/2601.06007)).
- Google ADK now exposes context compaction primitives that summarize older workflow event history once thresholds are reached, reinforcing that compaction is a runtime concern rather than an afterthought ([Google ADK context compaction](https://adk.dev/context/compaction/)).
- OpenAI Agents SDK sessions and state guidance separate client-managed history, SDK-backed sessions, server-side conversations, and response chaining; mixing memory strategies can duplicate context unless reconciliation is intentional ([OpenAI Agents SDK sessions](https://openai.github.io/openai-agents-js/guides/sessions/), [running agents](https://openai.github.io/openai-agents-python/running_agents/)).

Foundry implications:

- Make cache hit rate a production SLO, not a billing afterthought.
- Freeze static prompt/tool/capability order deterministically.
- Model mode transitions as messages or tools, not by mutating the system prompt/tool set mid-session.
- Use cache-safe forks for summarization, compaction, skills, and side computations.
- Separate context layers: immutable operating contract, stable project memory, session transcript, compact summary, task-specific scratch, and audit evidence.
- Add token budgets per phase and per subagent. Refuse unbounded context growth; start fresh windows from durable artifacts when saturation degrades performance.

### 26.3 Harness and evaluation findings

Production-grade agents need a harness that owns control flow, state, observability, replay, and evals. The best current guidance treats the model as one component inside software, not as the whole system.

- OpenAI's agent guide frames agents as model + tools + instructions, recommends evals before cost optimization, starts with single-agent systems, and moves to multi-agent systems only when tool overload or complex branching justifies it ([OpenAI practical guide](https://openai.com/business/guides-and-resources/a-practical-guide-to-building-ai-agents/)).
- Anthropic's effective-agents guidance emphasizes simple composable workflows: routing, parallelization, orchestrator-workers, and evaluator-optimizer; parallelization is useful for independent subtasks or multiple perspectives, not as default ceremony ([Anthropic: Building Effective Agents](https://www.anthropic.com/engineering/building-effective-agents)).
- HumanLayer's 12-Factor Agents argues that strong agents are mostly software: own prompts, own the context window, unify execution and business state, support pause/resume, own control flow, keep agents small, and make the agent a stateless reducer where possible ([12-Factor Agents](https://www.humanlayer.dev/blog/12-factor-agents)).
- OpenAI eval guidance recommends explicit objectives, datasets, metrics, held-out examples, continuous evaluation, and judge tasks shaped as comparison/classification/scoring rather than unconstrained generation ([OpenAI eval best practices](https://developers.openai.com/api/docs/guides/evaluation-best-practices)).
- Trace grading adds workflow-level visibility: graded traces can benchmark changes, find regressions, and explain why an agent failed rather than only whether the final answer was acceptable ([OpenAI trace grading](https://developers.openai.com/api/docs/guides/trace-grading)).
- OpenTelemetry now has development-stage GenAI semantic conventions for model, agent, framework, event, exception, metric, OpenAI/Anthropic/Bedrock/Azure, and MCP spans; this gives a vendor-neutral shape for traces even while the standard is still maturing ([OpenTelemetry GenAI semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/)).
- AWS's 2026 Strands Evals guidance mirrors this: agent evals need cases, experiments, evaluators, expected outputs, expected tool trajectories, and multi-turn simulation because final-answer checks miss tool-selection and decision-path failures ([AWS Strands Evals](https://aws.amazon.com/blogs/machine-learning/evaluating-ai-agents-for-production-a-practical-guide-to-strands-evals/)).
- Recent research frames harness design itself as an artifact worth versioning and evaluating: VeRO focuses on versioned agent snapshots, budget-controlled evaluation, structured traces, and rewards/observations; Natural-Language Agent Harnesses externalize harness behavior through explicit contracts and durable artifacts ([VeRO](https://arxiv.org/abs/2602.22480), [Natural-Language Agent Harnesses](https://arxiv.org/abs/2603.25723)).

Foundry implications:

- Treat the harness as product code with tests, versions, changelogs, release gates, and rollback.
- Every run should emit structured spans for model calls, tool calls, handoffs, guardrails, compaction, memory retrieval, policy decisions, human approvals, and failures.
- Persist portable incident artifacts for failing runs: prompt/context hash, model/provider/version, tool schemas, retrieved snippets, policy inputs, trace IDs, outputs, and redacted sensitive payload references.
- Evaluation suites should cover final answer quality, tool trajectory, cost, latency, retry behavior, recovery from partial tool failure, policy compliance, and adversarial/untrusted context handling.
- VCR-style fixtures remain valuable for model/API integration tests, but production acceptance needs trace-level evals and realistic multi-turn simulations.

### 26.4 Production architecture findings

The architectural center of gravity is moving toward explicit capability contracts, durable state, least-privilege tools, and incremental autonomy.

- The backup branch already demonstrates many of these patterns: typed tool contracts, permission modes, hook ABI, compaction, session transcripts, MCP normalization, LSP diagnostics, subagents, teams, and worktree isolation.
- OpenAI recommends well-defined reusable tools with standardized definitions, strong instructions, single-agent-first orchestration, and multi-agent split only for complex logic or tool overload ([OpenAI practical guide](https://openai.com/business/guides-and-resources/a-practical-guide-to-building-ai-agents/)).
- LangGraph's persistence docs emphasize checkpointed graph state for human-in-the-loop workflows, memory, time-travel debugging, and fault-tolerant execution ([LangGraph persistence](https://docs.langchain.com/oss/python/langgraph/persistence)).
- MCP provides a common protocol for connecting AI applications to tools, data sources, and workflows, but this increases the need for registry, provenance, permission, and supply-chain controls ([Model Context Protocol](https://modelcontextprotocol.io/docs/getting-started/intro)).
- The 12-Factor AgentOps doctrine makes context an engineering artifact: sessions should read from a corpus on the way in and write validated lessons back on the way out; knowledge should be typed, versioned, validated, and freshness-aware ([12-Factor AgentOps](https://www.12factoragentops.com/)).

Recommended architecture shape:

1. **Capability contract layer**: schema, data classes, autonomy tier, read/write/destructive flags, timeout, max output, audit topic, eval set, owner, version, policy tags.
2. **Policy and mode layer**: tenant policy, identity, least privilege, region/data-class restrictions, incident kill switches, human approval gates, break-glass separation.
3. **Execution harness layer**: deterministic run loop, retry budgets, tool result pairing, failure taxonomy, pause/resume, typed handoffs, concurrency limits.
4. **Context compiler layer**: cache-stable static prefix, retrieved context, session state, compact summaries, codebooks, tail messages, redaction, provenance.
5. **Observability/eval layer**: OTel-compatible traces, VCR fixtures, trace grading, golden/adversarial evals, drift monitoring, release regression gates.
6. **Memory/knowledge layer**: project memory, session memory, tenant memory, compact summaries, audit evidence, and learned patterns remain separate artifacts with explicit promotion rules.

### 26.5 Guardrail and security findings

The 2026 security consensus is much stricter than early agent demos: agentic AI should be adopted carefully, incrementally, and with cyber controls as strong as the privileges granted to the agent.

- The May 2026 Five Eyes joint guidance says agentic AI systems add inherited LLM risk, wider attack surface, complexity, privilege risk, design/configuration risk, behavior risk, structural risk, and accountability risk; it recommends low-risk/non-sensitive starting points, no broad unrestricted access, layered security, strict access controls, monitoring, governance, human oversight, resilience, reversibility, and risk containment ([NCSC/CISA/NSA joint guidance](https://www.ncsc.govt.nz/protect-your-organisation/careful-adoption-of-agentic-ai-services/), [PDF](https://media.defense.gov/2026/Apr/30/2003922823/-1/-1/0/CAREFUL%20ADOPTION%20OF%20AGENTIC%20AI%20SERVICES_FINAL.PDF)).
- OpenAI's agent guide describes guardrails as layered defenses, not single filters: relevance classifiers, safety classifiers, PII filters, moderation, tool safeguards, rules-based protections, output validation, and human intervention for high-risk actions or repeated failures ([OpenAI practical guide](https://openai.com/business/guides-and-resources/a-practical-guide-to-building-ai-agents/)).
- OpenAI Agents SDK guardrail docs distinguish workflow boundaries: input guardrails run for the first agent, output guardrails for final output, and tool guardrails around every custom function-tool invocation; delegated workflows need tool guardrails, not only agent-level checks ([OpenAI Agents SDK guardrails](https://openai.github.io/openai-agents-python/guardrails/)).
- OWASP's GenAI Security Project is now broader than the LLM Top 10 and explicitly covers agentic systems; the Agentic Top 10 highlights autonomy-specific risks such as goal hijack, tool misuse, excessive agency, insecure inter-agent communication, memory/context manipulation, cascading failures, human-agent trust exploitation, and rogue agents ([OWASP GenAI Security Project](https://owasp.org/www-project-top-10-for-large-language-model-applications/), [OWASP Top 10 for Agentic Applications 2026](https://genai.owasp.org/download/52117/?tmstv=1765059207)).
- NIST AI RMF and the NIST GenAI profile remain baseline governance references for mapping, measuring, managing, and governing AI risks ([NIST AI RMF](https://www.nist.gov/itl/ai-risk-management-framework)).
- CSA's AI Controls Matrix provides a vendor-agnostic control framework for cloud AI systems and explicitly includes orchestration-layer and application-provider responsibilities ([CSA AI Controls Matrix](https://cloudsecurityalliance.org/artifacts/ai-controls-matrix)).

Foundry guardrail posture:

- Deny before model exposure: unavailable or disallowed tools should not appear in the model's effective capability set.
- Use tool-level risk ratings and enforce additional checks before read-write, destructive, financial, external-message, credential, production, or data-export actions.
- Treat tool outputs, web pages, retrieved documents, MCP descriptions, plugin manifests, and memory as untrusted inputs unless proven otherwise.
- Put instruction hierarchy and provenance into context packets so the model can distinguish developer/user instructions from untrusted retrieved data.
- Use separate agents for materially different trust zones, but enforce handoff boundaries with explicit schemas, provenance, and policy checks.
- Require signed/verified plugins and MCP servers, trusted registries, SBOMs, version pinning, license checks, and update review.
- Make human approval meaningful: show diff/risk/provenance/evidence, avoid habituating users with noisy prompts, and record approvals as audit events.
- Design for reversibility: dry runs, staged rollout, transaction logs, rollback tools, and post-action verification.

### 26.6 Consolidated best-practice checklist

P0 for a production agentic development platform:

- Stable prompt/tool ordering and cache-hit telemetry.
- Explicit context compiler with codebooks, source provenance, and token budgets.
- Capability definitions as typed, versioned, policy-bearing contracts.
- Single run lifecycle: validate → policy → guardrail → permission → execute → sanitize → audit → evaluate.
- Tool-level guardrails and least-privilege identities.
- Trace everything: model calls, tool calls, handoffs, guardrails, memory, compaction, policies, approvals, and errors.
- Continuous evals with final-output, trajectory, cost/latency, security, and recovery dimensions.
- Durable state with checkpointing, pause/resume, replay, and portable incident artifacts.
- Human-in-the-loop for irreversible/high-risk/low-confidence branches.
- Supply-chain controls for plugins, MCP servers, tool schemas, prompts, evals, and models.

P1 once P0 is stable:

- Tool search / deferred tool schema loading to reduce context without mutating tool identity.
- Multi-agent orchestration with manager and handoff patterns, but only where single-agent + tools fails evals.
- Worker isolation through worktrees/sandboxes and bounded write scopes.
- External evaluator/reviewer lanes for architecture, security, and production-readiness gates.
- Knowledge flywheel: every session emits both work product and validated lessons, with explicit promotion into project memory.

### 26.7 arXiv idea scan: additional leads to mine

Caveat: the following are mostly 2025-2026 preprints. Treat them as idea sources and experiment prompts, not settled standards. The strongest production posture is to convert promising ideas into small local prototypes, evals, and guardrail checks before adopting them.

Context and token economy:

- **Contextual Memory Virtualisation** proposes modeling session history as version-controlled DAG state with snapshot/branch/trim primitives and structurally lossless trimming. This maps well to long-running coding agents that need to fork workers without copying the whole transcript ([arXiv:2602.22402](https://arxiv.org/abs/2602.22402)).
- **Context Engineering: From Prompts to Corporate Multi-Agent Architecture** frames context quality around relevance, sufficiency, isolation, economy, and provenance. This is a useful rubric for a Foundry context compiler and for reviewing whether memory/codebook/context packets are helping or bloating the run ([arXiv:2603.09619](https://arxiv.org/abs/2603.09619)).
- **Spec Kit Agents** reports gains from phase-level, read-only context-grounding hooks before specification, planning, tasking, and implementation. This supports adding mandatory repository-probing hooks before agents write code or architecture docs ([arXiv:2604.05278](https://arxiv.org/abs/2604.05278)).
- **Loosely-Structured Software** is a conceptual architecture paper for runtime-rewired multi-agent systems. Its useful terms are view/context engineering, structure engineering, and evolution engineering: all map to controlling entropy in autonomous multi-agent platforms ([arXiv:2603.15690](https://arxiv.org/abs/2603.15690)).

Harness, verification, and observability:

- **ContextCov** turns passive instruction files such as `AGENTS.md` into executable constraints via AST checks, shell shims, and architecture validators. This is directly relevant to enforcing agent operating contracts instead of relying on prompt compliance ([arXiv:2603.00822](https://arxiv.org/abs/2603.00822)).
- **Agentproof** statically extracts workflow graphs from LangGraph, CrewAI, AutoGen, and Google ADK, then checks topology and temporal policies. Foundry should consider an equivalent preflight verifier for capability graphs, handoff graphs, and human-gate policies ([arXiv:2603.20356](https://arxiv.org/abs/2603.20356)).
- **VeRO** emphasizes versioned agent snapshots, budget-controlled evaluation, structured execution traces, rewards, and observations. This reinforces that agent optimization must version the harness and budget, not only the prompt/model ([arXiv:2602.22480](https://arxiv.org/abs/2602.22480)).
- **Natural-Language Agent Harnesses** externalizes harness behavior as executable natural-language contracts plus durable artifacts and adapters. This suggests a path for repo-native harness specs that are readable by humans but mechanically testable ([arXiv:2603.25723](https://arxiv.org/abs/2603.25723)).
- **AgentTrace: A Structured Logging Framework** splits observability into operational, cognitive, and contextual surfaces. **AgentTrace: Causal Graph Tracing** reconstructs causal graphs from multi-agent execution logs for root-cause analysis. Together they argue for causal, typed traces rather than flat logs ([arXiv:2602.10133](https://arxiv.org/abs/2602.10133), [arXiv:2603.14688](https://arxiv.org/abs/2603.14688)).
- **AgentSight** uses system-level boundary tracing to correlate high-level LLM intent with low-level system effects. Even if Foundry does not adopt eBPF, the idea is important: traces should connect intent, tool call, process/network/file effects, and policy decisions ([arXiv:2508.02736](https://arxiv.org/abs/2508.02736)).
- **AlphaEval** argues production agent evals differ from clean benchmarks because real tasks contain implicit constraints, fragmented documents, evolving expert standards, UI behavior, and long-horizon deliverables. This supports domain-owned eval construction rather than generic benchmark worship ([arXiv:2604.12162](https://arxiv.org/abs/2604.12162)).

MCP, tool poisoning, and guardrail research:

- **Are AI-assisted Development Tools Immune to Prompt Injection?** evaluates prompt-injection/tool-poisoning risks across MCP clients and compares security features such as static validation, parameter visibility, warnings, sandboxing, and audit logging. Foundry should make these feature checks part of MCP server onboarding ([arXiv:2603.21642](https://arxiv.org/abs/2603.21642)).
- **MCP-ITP** studies implicit tool poisoning where malicious metadata causes the agent to invoke a legitimate high-privilege tool. This is a strong warning that uninvoked tools and descriptions can still be attack surfaces ([arXiv:2601.07395](https://arxiv.org/abs/2601.07395)).
- **ShieldNet** proposes network-level guardrails for supply-chain injections in agentic systems and introduces a benchmark of malicious MCP tools. The key idea is to observe real network behavior, not just semantic tool traces ([arXiv:2604.04426](https://arxiv.org/abs/2604.04426)).
- **CASCADE** proposes local, layered MCP prompt-injection detection using fast filters, semantic analysis, and output filtering. Its reported recall limitations are as important as the architecture: local filters help but do not replace least privilege and approval gates ([arXiv:2604.17125](https://arxiv.org/abs/2604.17125)).
- **SMCP** proposes protocol-level improvements to MCP: identity management, mutual authentication, security context propagation, fine-grained policy, and audit logging. This aligns with treating MCP as an enterprise control-plane integration, not just a tool list ([arXiv:2602.01129](https://arxiv.org/abs/2602.01129)).
- **MCPShield** proposes a plug-in security cognition layer that probes tool behavior before invocation and updates trust after observing runtime events. Useful idea: trust scores should be stateful and evidence-based, but the final enforcement should still be deterministic policy ([arXiv:2602.14281](https://arxiv.org/abs/2602.14281)).
- **Bridging Protocol and Production** identifies production MCP gaps around identity propagation, adaptive tool budgeting, and structured error semantics. These map directly to Foundry capability runtime requirements: identity-scoped routing, per-tool timeout budgets, and machine-readable failure types ([arXiv:2603.13417](https://arxiv.org/abs/2603.13417)).

Immediate experiments worth running in Foundry:

1. Prototype `AGENTS.md`/contract-to-check extraction like ContextCov for a small doc subset.
2. Add a static graph verifier for capability/handoff graphs with human-gate policies.
3. Add MCP onboarding checks: signed source, schema diff, metadata prompt-injection scan, parameter visibility, dry-run sandbox, audit logging, and network allowlist.
4. Add context packet scoring with the five criteria: relevance, sufficiency, isolation, economy, provenance.
5. Build a trace-to-incident artifact generator that bundles causal graph, tool I/O references, policy decisions, token/cost/cache metrics, and replay instructions.
6. Test CMV-style transcript trimming against existing compaction: compare token reduction, cache-hit preservation, replay fidelity, and eval impact.

# Foundry applicability

## P0 patterns to adopt

### 1. First-class capability/tool contract

Foundry should define a `CapabilityDefinition` with:

- `id`
- `version`
- `owner`
- `input_schema`
- `output_schema`
- `data_classes`
- `autonomy_tier`
- `read_only`
- `destructive`
- `open_world`
- `requires_human`
- `concurrency`
- `timeout`
- `max_output`
- `permission_matcher`
- `audit_topic`
- `eval_set`
- `policy_tags`
- `rendering_hints`

Why: capability safety, UX, policy, and observability become part of the definition, not ad-hoc prompt text.

### 2. Capability runtime pipeline

Adopt this lifecycle:

`schema parse → validation → pre-hooks → permission → execution → cap/sanitize → post-hooks → audit → result`

Guarantee paired results even for failures.

### 3. Deny before model exposure

Filter unavailable capabilities before model prompt assembly using:

- tenant policy
- license tier
- autonomy tier
- region pack
- data-class restrictions
- runtime incident kill-switches

### 4. Stable capability ordering

Use deterministic ordering for prompt-cache stability across:

- built-in capabilities
- MCP/external tools
- tenant overlays
- regional pack tools
- plugin capabilities

### 5. Layered permission model

Implement multiple independent layers:

- static tenant policy
- runtime mode
- data-class policy
- capability-specific validation
- cross-axis contract policy
- human approval
- classifier/risk scorer when appropriate
- hooks/approval workflows
- incident kill switches

### 6. Auto mode safer than bypass

Use modes like:

- `observe`
- `plan`
- `suggest`
- `approve-edits`
- `auto`
- `break-glass`

Auto should strip dangerous grants and require classifier/risk scoring for open-world/destructive actions.

### 7. Shell hardening library

If Foundry runs shell/browser/network actions, implement specialized validators:

- AST parse first
- fail closed on ambiguity
- read-only validator
- git-specific validator
- redirection validator
- runtime variable expansion defense
- UNC/provider path defense
- command fanout limits
- sandbox integration
- parsed permission suggestions

### 8. Typed hook ABI

Define lifecycle hooks for:

- before capability use
- after capability success
- after capability failure
- permission requested
- permission denied
- agent started/stopped
- session started/ended
- context compacted
- file/resource changed
- human elicitation requested/responded

Each should have schema, timeout, managed-policy controls, audit event, blocking/nonblocking distinction, source attribution, and environment isolation.

### 9. External protocol normalization

Normalize MCP, plugins, internal SDK tools, HTTP APIs, human approvals, browser tools, and data-plane tools into one capability lifecycle.

### 10. Transcript integrity

Adopt:

- durable JSONL/event-log transcript
- parent chain
- UI progress separated from durable conversation state
- explicit compact boundaries
- tool-use/tool-result pairing guarantees
- load-time repair for legacy transcript anomalies
- session/project path disambiguation

### 11. Compaction with audit boundaries

Treat compaction as first-class:

- PreCompact hook
- summary request
- compact boundary event
- preserved tail
- attachment handling
- PostCompact hook
- audit-chain event
- replay metadata

### 12. Session memory separate from compaction

Separate:

- short-term context window
- compact summary
- session memory
- project memory
- tenant memory
- audit evidence

## P1 patterns to adapt carefully

### 13. Plugin marketplace with strict policy

Foundry can use plugins containing:

- capabilities
- skills
- hooks
- MCP servers
- UI metadata
- eval sets
- policy declarations

But add stronger supply-chain controls:

- signed manifests
- SBOM
- license validation
- autonomy tier
- data-class declarations
- region/tenant compatibility
- eval pass requirement

### 14. Agent/team orchestration

Adopt concepts, not code:

- agent definition registry
- explicit spawn metadata
- parent/child provenance
- no nested teammate ambiguity
- worktree/sandbox isolation
- background task lifecycle
- completion notifications
- kill/stop controls

### 15. Worktree isolation

Useful for code agents:

- isolated worktree per agent/session
- parent session remains clean
- worktree metadata tracked
- WorktreeCreate hooks
- safe cleanup

### 16. Remote-control sessions

Useful conceptual model:

- local runner
- remote UI
- permission callbacks
- trusted device token
- session resume pointer
- multi-session capacity
- worktree isolation
- activity feed

Foundry should implement its own secure protocol and audit model.

### 17. VCR fixtures for AI/API tests

Use fixtures for:

- model responses
- token counting
- streaming API events
- MCP responses
- external SaaS APIs

### 18. Passive diagnostics

Turn background diagnostics into evidence:

- language diagnostics
- schema diagnostics
- policy diagnostics
- contract drift
- security scanners
- license scanners

## P2 patterns to consider later

### 19. Rich terminal/operator UI

Useful pieces:

- tool-specific permission dialogs
- file diff approval
- permission debug info
- task progress panels
- virtual message list
- statusline
- theme/keybinding customization

### 20. Voice and companion UX

Less foundational unless Foundry has a human operator console requiring voice/chat companions.

### 21. Command registry layering

Foundry can use a simpler version:

- core commands
- tenant commands
- plugin commands
- capability commands
- admin commands

---

# What Foundry should avoid copying

1. **Do not copy leaked/proprietary code.** Use clean-room patterns only.
2. **Avoid monolithic REPL/main/settings files.** Split orchestration, UI, policy, execution, and persistence earlier.
3. **Do not ship source maps or sensitive source artifacts accidentally.**
4. **Do not use feature flags without lifecycle governance.** Every flag needs owner, expiry, rollout state, and tests.
5. **Do not build shell execution without extensive tests.**
6. **Do not trust plugins by default.** Require signatures, evals, policy annotations, and tenant approval.
7. **Do not initialize telemetry before trust.**
8. **Do not make bypass mode easy.** Bypass should be policy-gated, audited, and unavailable in normal tenant production contexts.

---

# Proposed Foundry implementation sequence

## Phase 1: Capability contract

Create a canonical, schema-first `CapabilityDefinition` with policy, autonomy, data-class, audit, and UI metadata.

## Phase 2: Runtime pipeline

Implement:

`validate → pre-policy → pre-hooks → permission → execute → cap/sanitize → post-hooks → audit → result`

## Phase 3: Policy/mode system

Implement safe modes and break-glass separation. Auto mode strips dangerous grants and requires risk scoring/classification.

## Phase 4: Hook ABI

Define hook events and schemas with managed-only controls.

## Phase 5: External protocol normalization

Normalize MCP/plugins/internal SDK tools into the same capability interface.

## Phase 6: Transcript and compaction

Implement event-sourced transcript, compact boundaries, evidence references, and replay metadata.

## Phase 7: Open-world tool hardening

Only after policy/runtime foundations exist, add shell/browser/network tools with specialized validators.

---

# Unknowns and limits

- The branch does not include enough metadata to build or run cleanly.
- The actual test suite is missing.
- Production feature flag behavior cannot be fully verified.
- Server-side APIs and policy behavior are inferred from client code.
- Some internal-only branches may be dead-code-eliminated in external builds.
- Line numbers refer to the local clone at commit `372a01d48621cadeb0c3a3a0164c4622c35cbfea` and may drift if the branch changes.

---

# OMX workflow note

The research report itself was delivered and the Codex goal was marked complete. A later OMX `autoresearch-goal complete` reconciliation attempt was blocked because the live Codex goal objective contained the original user request, while the autogenerated autoresearch-goal mission expected the generated handoff objective. The final OMX artifact was therefore marked `blocked` for reconciliation, not for research quality.

---

# Appendix A: Optimizing Claude-Code Patterns for Fully Agentic Autonomous Work

## A.1 Direct answer

Yes. The analyzed Claude-code branch is powerful, but its default product shape is optimized around a **human operator in a terminal**. It has many pieces that can support autonomous work -- permissions, hooks, subagents, skills, transcripts, compaction, MCP, plugins, remote bridge, VCR, LSP, and shell validators -- but those pieces are still arranged primarily around this loop:

`human intent → interactive REPL → model response → tool proposal → permission prompt / hook → human steering → transcript`

A fully agentic autonomous system should invert that loop:

`durable objective → policy envelope → machine task graph → autonomous execution → verifier gate → evidence bundle → human exception only when needed`

The difference is not only UI. It is a different runtime contract. Claude-code treats the conversation as the primary object and tools as assistant affordances. A fully autonomous Foundry-style system should treat the **goal**, **plan**, **task lease**, **policy envelope**, **verification run**, and **evidence bundle** as first-class persisted objects, with conversation as only one optional interface.

Confidence: high. This conclusion is grounded in the branch topology and specific files already analyzed above: `src/main.tsx`, `src/QueryEngine.ts`, `src/query.ts`, `src/Tool.ts`, `src/tools.ts`, `src/services/tools/toolExecution.ts`, `src/utils/permissions/*`, `src/utils/hooks.ts`, `src/commands.ts`, `src/tools/AgentTool/*`, `src/utils/agents/*`, `src/services/sessionStorage.ts`, `src/utils/compact.ts`, `src/utils/autoCompact.ts`, `src/services/mcp/config.ts`, and the shell guardrail packages under `src/tools/BashTool/*` and `src/tools/PowerShellTool/*`.

---

## A.2 What in Claude-code is human-optimized

### A.2.1 The REPL is the control plane

The CLI and REPL path (`src/entrypoints/cli.tsx`, `src/entrypoints/init.ts`, `src/main.tsx`, `src/screens/REPL.tsx`) center the operator experience. The user enters messages, picks modes, responds to prompts, views rich terminal rendering, and drives the session manually. That is excellent for pair-programming, exploratory debugging, local edits, and supervised coding.

For autonomous operation, a terminal REPL is the wrong primary control plane. Autonomous work needs a queueable, resumable, externally inspectable control plane where every work item has machine-readable state, dependencies, assigned agents, budgets, policy, and verification status.

### A.2.2 Slash commands are optimized for operator steering

`src/commands.ts` implements command loading, built-ins, skill command extraction, filtering, and remote/bridge safety checks. This is a strong extension surface for humans: `/compact`, `/agents`, `/permissions`, `/model`, `/doctor`, `/status`, `/vim`, and similar commands are discoverable and quick.

Autonomous systems should not rely on slash commands as the durable workflow layer. A slash command is a convenient manual trigger, not a robust plan object. Agentic systems need declarative workflow specs that can be scheduled, replayed, audited, resumed, and verified.

Recommended transformation:

- Keep slash commands as a human facade.
- Back every significant command with a machine-readable operation schema.
- Emit `OperationRequested`, `OperationAccepted`, `OperationCompleted`, and `OperationFailed` events.
- Allow autonomous agents to call the same operation schemas without terminal text parsing.

### A.2.3 Permission prompts assume a human is present

The permission stack is sophisticated: `PermissionMode.ts`, `permissionSetup.ts`, `permissions.ts`, Bash/PowerShell validators, hook matchers, settings allow/deny rules, MCP allowlists, and dangerous-mode controls. But the default interaction model still often assumes that a human can approve, deny, or modify the next step.

For fully autonomous work, permission prompts should be exceptional. The normal path should be **pre-authorized policy envelopes**:

- allowed repositories / working directories
- allowed file globs
- denied file globs
- allowed commands and command classes
- network allowlists
- data-class boundaries
- maximum spend / tokens / wall-clock
- maximum diff size
- maximum number of retries
- maximum autonomy tier
- required verifier gates
- rollback requirements
- escalation thresholds

The agent should operate freely inside the envelope and escalate only when it needs to cross a boundary.

### A.2.4 Rich TUI output is optimized for eyes, not automation

The branch has extensive Ink/React terminal rendering, keybindings, Vim mode, voice mode, statusline, spinners, theme support, and componentized terminal UI. This is useful for human trust and ergonomics.

Autonomous systems need different defaults:

- structured JSON events
- append-only execution logs
- task state snapshots
- machine-readable diagnostics
- stable artifact directories
- event IDs for audit correlation
- dashboards built on event streams, not terminal screen state

A human cockpit can still exist, but it should subscribe to the same event log used by automation.

### A.2.5 Conversation history is used as operational memory

The branch writes transcripts, tracks parent/child chain IDs, stores sessions, handles compaction, and injects context from git status and project files. This is strong conversational memory.

Long-running autonomous agents need operational memory as a separate substrate:

- goal state
- plan state
- task graph state
- leases and ownership
- known blockers
- assumptions
- decisions
- rejected alternatives
- verification evidence
- retry history
- rollback points
- environment fingerprints
- dependency versions
- risk register links

Conversation transcript should be evidence, not the only source of truth.

---

## A.3 What Claude-code already has that is valuable for autonomous work

Claude-code is not merely a human toy. It contains many primitives that should be preserved, generalized, or split out for autonomous systems.

### A.3.1 Tool contract and execution lifecycle

Useful existing idea:

`Tool definition → input schema → validation → permission → call → result mapping → logging / telemetry / hooks`

Relevant files:

- `src/Tool.ts`
- `src/tools.ts`
- `src/services/tools/toolExecution.ts`
- `src/tools/*`

For autonomous Foundry-style work, this should become a formal capability runtime:

`CapabilityDefinition → PolicyEvaluation → PreHook → LeaseCheck → Execute → Sanitize → PostHook → EvidenceEmit → VerifierGate`

Claude-code's tool lifecycle is a strong starting point, but it should be made more explicit, more durable, and less tied to assistant-message flow.

### A.3.2 Permission modes and risk tiers

Useful existing idea:

- default / plan / accept edits / bypass-like modes
- dangerous Bash/PowerShell stripping
- auto-mode classifier permission
- MCP permission matching
- hook-driven permission behavior

For autonomous systems, mode selection should be replaced or supplemented with autonomy tiers and scoped policies:

- **T0 Observe**: read-only, no external side effects
- **T1 Local edit**: edit bounded files, run tests, no network mutation
- **T2 Repo workflow**: branch, commit, open PR, manage local services
- **T3 Controlled production-adjacent**: staging deploys, tenant-safe migrations, guarded external calls
- **T4 Break-glass / production mutation**: requires explicit pre-approval, dual control, and audit emission

This maps cleanly to Oyatie's autonomy-ceiling concern: an agent should not gain higher autonomy because a prompt says so; it should gain it only through policy and runtime enforcement.

### A.3.3 Hooks as an extension and guardrail ABI

Useful existing idea:

- PreToolUse
- PostToolUse
- PermissionDenied
- UserPromptSubmit
- SessionStart / SessionEnd
- SubagentStart
- PermissionRequest
- environment/file/cwd hooks
- JSON input/output contracts

Relevant files:

- `src/types/hooks.ts`
- `src/utils/hooks.ts`

For autonomous work, hooks should be elevated into a governed event ABI. Important changes:

1. Every hook input and output should be versioned.
2. Hook execution should have declared side-effect class.
3. Hook failures should be classified: blocking, warning, retryable, ignored.
4. Hook results should be persisted as evidence.
5. Hooks that can change permission outcomes should require managed policy approval.
6. Hooks should not be the only source of critical state transitions; they should emit into the runtime event stream.

### A.3.4 Shell guardrails

Useful existing idea:

- Bash AST inspection
- PowerShell validation
- read-only command classification
- redirection and heredoc checks
- command injection defenses
- dangerous operator detection
- parser differential handling

Relevant files:

- `src/tools/BashTool/*`
- `src/tools/PowerShellTool/*`

For autonomous work, this class of guardrail is essential. It should be expanded into a shell policy engine with:

- command allowlists by autonomy tier
- command provenance and generated-risk score
- shell transcript capture
- environment capture
- filesystem diff snapshots before/after risky commands
- automatic rollback hooks for owned changes
- precomputed safe command templates for common workflows
- denied-command explainability

### A.3.5 Subagents, teams, tasks, and isolation

Useful existing idea:

- `AgentTool.tsx`
- `runAgent.ts`
- `loadAgentsDir.ts`
- `spawnMultiAgent.ts`
- background / async agent support
- forked message contexts
- team spawning paths
- isolation controls

This is one of the most reusable parts conceptually. The branch has a strong notion that not every task should be done by the main assistant. For fully autonomous work, this should become a real scheduler and worker system:

- task leases
- explicit write ownership
- dependency graph
- conflict detection
- result contracts
- verifier assignment
- worker health checks
- durable heartbeat
- cancellation semantics
- retry / replacement workers
- artifact ownership

Subagents should not merely be extra conversations. They should be actors in a task runtime.

### A.3.6 MCP and plugin normalization

Useful existing idea:

- MCP server config by scope
- enterprise / policy filtering
- server allow/deny handling
- plugin-provided tools
- plugin installation manager
- marketplace allowlists

Relevant files:

- `src/services/mcp/config.ts`
- `src/services/mcp/MCPConnectionManager.tsx`
- `src/tools/MCPTool.ts`
- `src/services/plugins/*`
- `src/utils/plugins/*`
- `src/types/plugin.ts`

For autonomous work, every external tool source should be normalized into a single capability model. Internal tools, MCP tools, plugin tools, remote tools, browser tools, and tenant tools should all become capabilities with:

- owner
- version
- input schema
- output schema
- data classes
- side effects
- autonomy tier
- policy requirements
- eval suite
- audit topics
- rollback expectations
- allowed tenants / regions
- rate limits
- cost model

### A.3.7 Transcript, compaction, and replay primitives

Useful existing idea:

- transcript writes
- parent/child UUID chains
- session loading
- auto-compact
- compact prompt boundaries
- VCR cassette support

Relevant files:

- `src/services/sessionStorage.ts`
- `src/utils/compact.ts`
- `src/utils/autoCompact.ts`
- `src/services/vcr.ts`

For autonomous work, this should be reframed as event sourcing plus replay:

- raw LLM turns are one event type
- tool calls are events
- policy decisions are events
- verification runs are events
- artifacts are content-addressed references
- compaction emits a summarized state object with provenance links
- replay can reconstruct why a task reached a conclusion

This is more reliable than depending on a long natural-language transcript.

---

## A.4 The main architectural shift: conversation product to agent operating system

### A.4.1 Current center of gravity

The analyzed branch's center of gravity is:

1. terminal user starts CLI
2. settings and trust are loaded
3. commands and tools are registered
4. human sends prompt
5. query engine streams LLM output
6. tool uses are validated and approved
7. output returns to terminal
8. transcript is stored
9. compaction may happen
10. user continues steering

### A.4.2 Autonomous center of gravity

A fully agentic runtime's center of gravity should be:

1. durable goal is created
2. acceptance criteria are normalized
3. policy envelope is attached
4. planner creates task graph
5. scheduler leases tasks to agents
6. agents execute within bounded authority
7. runtime records all side effects
8. verifier checks acceptance criteria
9. evidence bundle is emitted
10. unresolved exceptions become explicit escalations
11. result can be replayed or audited

### A.4.3 Consequence

The model should not be the state machine. The model should be a participant in a state machine. That distinction matters. In human use, the model can opportunistically decide what to do next because the human is present to steer. In autonomous work, the runtime must own state transitions, authority boundaries, and completion rules.

---

## A.5 Autonomous runtime objects that should be first-class

### A.5.1 Goal

A goal is not a prompt. It is a durable contract.

Suggested fields:

```yaml
goal_id: goal_...
title: string
objective: string
created_by: user|system|scheduler
repo_scope: [paths]
acceptance_criteria:
  - id: ac_001
    statement: string
    verifier: test|static_check|human_review|artifact_check|policy_check
constraints:
  max_wall_clock_minutes: number
  max_tokens: number
  max_cost_usd: number
  allowed_paths: []
  denied_paths: []
  allowed_network_hosts: []
  autonomy_ceiling: T0|T1|T2|T3|T4
required_evidence:
  - tests
  - diff_summary
  - risk_assessment
  - audit_event
status: queued|planning|executing|verifying|blocked|complete|failed|cancelled
```

Claude-code has prompts and transcripts; the autonomous system needs this explicit goal layer above them.

### A.5.2 Plan

A plan is not a prose checklist only. It is a dependency graph.

Suggested fields:

```yaml
plan_id: plan_...
goal_id: goal_...
version: 1
tasks:
  - task_id: task_001
    title: Map current implementation
    type: exploration
    dependencies: []
    owner_role: explore
    write_scope: []
    success_signal: file_refs_collected
  - task_id: task_002
    title: Implement bounded fix
    type: code_change
    dependencies: [task_001]
    owner_role: executor
    write_scope: [src/foo/**]
    success_signal: tests_pass
  - task_id: task_003
    title: Verify fix
    type: verification
    dependencies: [task_002]
    owner_role: verifier
    write_scope: []
    success_signal: acceptance_criteria_pass
```

Claude-code subagents can execute parts of this, but the analyzed branch does not present task graph as the central durable object.

### A.5.3 Policy envelope

A policy envelope is the autonomous replacement for ad-hoc permission prompts.

Suggested fields:

```yaml
policy_envelope_id: pe_...
goal_id: goal_...
autonomy_ceiling: T2
allowed_tools:
  - FileRead
  - Grep
  - Glob
  - FileEdit
  - BashTool:test-only
blocked_tools:
  - BrowserProductionAdmin
  - CloudDeployProduction
allowed_commands:
  - npm test
  - cargo test
  - cargo clippy
  - rg
  - git diff
blocked_command_patterns:
  - rm -rf /
  - git push --force
  - gh pr merge
allowed_paths:
  - src/**
  - tests/**
denied_paths:
  - .env
  - secrets/**
requires_human_approval:
  - production_mutation
  - credential_rotation
  - external_payment_action
```

Claude-code's permission code is a strong base; the autonomous improvement is making the envelope explicit and pre-bound to the goal.

### A.5.4 Task lease

A worker should never vaguely own “the task.” It should lease a precise slice.

Suggested fields:

```yaml
lease_id: lease_...
task_id: task_002
agent_id: agent_...
role: executor
write_scope:
  - src/foo/**
read_scope:
  - src/**
  - tests/**
expires_at: timestamp
heartbeat_interval_seconds: 30
conflict_policy: fail_on_overlap|allow_readonly_overlap|merge_after_review
```

This prevents autonomous agents from trampling each other.

### A.5.5 Evidence bundle

A completed autonomous task should produce evidence, not just a final message.

Suggested fields:

```yaml
evidence_bundle_id: evb_...
goal_id: goal_...
commit: sha|null
diff_summary: string
tests:
  - command: cargo test -p x
    status: pass|fail|skipped
    started_at: timestamp
    completed_at: timestamp
    log_ref: artifact://...
static_checks: []
policy_checks: []
artifacts:
  - path: docs/raw/...
    sha256: ...
known_gaps:
  - reason: no package metadata in target repo
verifier_verdict: pass|fail|inconclusive
```

Claude-code's transcript is useful but insufficient as completion evidence. Foundry should require evidence bundles.

---

## A.6 Human-optimized vs autonomous-optimized comparison

| Dimension | Human-optimized Claude-code shape | Fully autonomous optimized shape |
|---|---|---|
| Primary object | Conversation/session | Goal + plan + policy + evidence |
| Control plane | Terminal REPL | Scheduler/API/event log |
| Workflow trigger | Prompt or slash command | Durable workflow spec / queued goal |
| Permission model | Ask human at decision points | Pre-authorized policy envelope; escalate only on boundary crossing |
| State | Transcript + settings + compaction | Event-sourced runtime state + replayable artifacts |
| Completion | Assistant says done / user judges | Verifier checks acceptance criteria and emits evidence |
| Subagents | Extra assistant conversations | Leased workers in task graph with ownership and heartbeat |
| Hooks | Local automation and guardrail scripts | Versioned policy/event ABI with persisted outcomes |
| UI | Rich terminal | Machine event stream + optional cockpit |
| Memory | Conversation context and summaries | Operational memory: decisions, risks, blockers, artifacts, retries |
| Tool abstraction | Tool calls in assistant loop | Governed capabilities with side effects, data class, evals, audit |
| Failure handling | User sees error and steers | Runtime classifies failure, retries, narrows scope, or escalates |
| Security | Permission prompts and validators | Least-privilege envelopes, capability tiers, audit, sandbox, rollback |
| Testing | VCR and local tests | Replay harness + verifier gates + policy fitness lanes |
| Long-running work | Session-centric | Crash-safe durable orchestration |

---

## A.7 Specific optimizations for fully agentic autonomous work

### A.7.1 Make goal mode mandatory for long-running work

Claude-code can run interactive sessions and subagents, but autonomous work should not start from a bare prompt. The runtime should require a normalized goal object when:

- wall-clock may exceed a few minutes
- changes may touch files
- more than one agent may participate
- external systems may be called
- verification is required
- auditability matters

This avoids ambiguous “do the thing” prompts becoming unbounded operations.

### A.7.2 Separate planner, executor, verifier, and critic

Human-driven Claude-code can let one assistant plan, edit, and self-judge. Fully autonomous systems need stronger separation.

Recommended default roles:

- **Planner**: creates task graph and risk model.
- **Executor**: performs bounded write-scope implementation.
- **Verifier**: checks acceptance criteria and evidence.
- **Critic / Security reviewer**: invoked for high-risk changes.
- **Doc updater**: updates canonical docs when required.

The main runtime should enforce that the executor cannot mark its own high-risk work as complete without verifier evidence.

### A.7.3 Replace prompt-level autonomy with capability-level autonomy

A model should not gain authority because the user writes “be autonomous.” Authority should come from capability policy.

Each capability should declare:

- autonomy tier
- data classes it can read
- data classes it can write
- side-effect class
- tenant/region restrictions
- required approvals
- rollback behavior
- evidence emitted
- eval suite

This is directly relevant to Foundry: capability records should be the source of truth, not ad-hoc tool descriptions.

### A.7.4 Add a durable task scheduler

Subagent spawning is powerful, but autonomous systems need scheduling semantics:

- queue
- lease
- heartbeat
- cancel
- retry
- dependency unblock
- timeout
- replacement worker
- conflict detection
- result ingestion

Without a scheduler, multi-agent work becomes several unsynchronized conversations.

### A.7.5 Add a conflict-aware workspace model

Fully autonomous agents need predictable file ownership.

Recommended rules:

- Every write task declares write globs.
- Overlapping write globs require a coordinator.
- Read-only exploration can run in parallel with write tasks.
- Agents must diff before writing and before finalizing.
- Runtime tracks which agent created each change.
- Final integration is a separate task.

Claude-code has worktree/team concepts and agent isolation hints; Foundry should make them hard runtime rules.

### A.7.6 Require proof plans before edits

For autonomous code changes, the agent should identify how success will be proven before it edits.

Examples:

- target unit test
- integration test
- snapshot test
- typecheck
- lint
- static policy gate
- visual diff
- benchmark
- security scan

If no proof path exists, the first task should be to create one or explicitly record the verification gap.

### A.7.7 Make verification a blocking gate, not a final paragraph

In Claude-code-style human use, the assistant can report “tests not run” and the human decides. Autonomous systems should treat verification status as a state transition.

Recommended state machine:

`executing → verification_pending → verifying → verified | verification_failed | inconclusive`

Only `verified` should allow `complete`. `inconclusive` should become a blocked or escalated state, not a success.

### A.7.8 Add autonomous recovery loops

When a tool fails, the runtime should classify the failure and choose a recovery path.

Failure classes:

- transient network failure
- dependency missing
- test failure caused by current diff
- pre-existing test failure
- permission boundary hit
- ambiguous requirement
- merge conflict
- external service unavailable
- model output invalid
- verifier rejected evidence

Recovery actions:

- retry with backoff
- narrow command
- inspect logs
- run smaller test
- bisect own changes
- revert own diff
- spawn debugger
- ask planner to revise plan
- escalate with structured blocker

Claude-code has some recovery in `query.ts` around tool result backfill, recoverable errors, fallback model, and missing tool_result handling. Foundry should expand this into a general runtime recovery policy.

### A.7.9 Persist assumptions and decisions separately from chat

Autonomous agents often fail by forgetting why they chose a path. Every major decision should be recorded as structured operational memory:

```yaml
decision_id: dec_...
goal_id: goal_...
statement: Use existing hook ABI rather than create a second plugin mechanism.
rationale: Avoid duplicate extension paths.
rejected:
  - option: Build independent plugin runtime
    reason: Would fragment policy and audit behavior.
confidence: medium
created_by: planner
```

This aligns with lore-commit style: future agents should not have to rediscover rejected alternatives.

### A.7.10 Make human interruption structured

Humans should still be able to intervene, but not by corrupting runtime state. Interruption should be an event:

- pause goal
- cancel goal
- add constraint
- change acceptance criterion
- approve boundary crossing
- deny boundary crossing
- request status
- inject evidence

The runtime should then re-plan or resume from a known state.

### A.7.11 Emit audit-chain events for meaningful side effects

For Foundry, every material autonomous action should emit audit events:

- goal created
- policy envelope attached
- task leased
- capability invoked
- permission denied
- boundary escalation requested
- diff produced
- verification passed/failed
- evidence bundle sealed
- goal completed/failed/cancelled

Claude-code has telemetry and logging hooks; Foundry should formalize them as audit-chain emissions.

### A.7.12 Treat cost and latency as budgets, not surprises

Human sessions tolerate occasional expensive exploration. Autonomous loops can spiral. Budgets should be first-class:

- token budget
- tool-call budget
- wall-clock budget
- subprocess budget
- network-call budget
- retry budget
- child-agent budget

When budget is nearly exhausted, the runtime should summarize state and decide: continue under a new approval, reduce scope, or fail with evidence.

### A.7.13 Add objective-level replay and simulation

The VCR concept in Claude-code is valuable. Fully autonomous systems should replay entire goals:

- same goal object
- same policy envelope
- same capability definitions
- mocked external services
- deterministic tool outputs where possible
- expected verifier outcome

This enables regression tests for agent workflows, not just individual tools.

### A.7.14 Replace “dangerous bypass” with scoped break-glass

Claude-code-style bypass modes are useful for trusted local humans but dangerous for production autonomous systems. Foundry should provide break-glass with:

- explicit reason
- limited duration
- limited goal scope
- dual approval if high impact
- mandatory audit event
- automatic rollback plan
- post-action review

There should be no general “ignore all permission checks” autonomous mode in tenant production.

### A.7.15 Make plugins/capabilities publish through certification

A plugin should not become available to autonomous agents merely because it is installed. It should pass certification:

- schema validation
- license review
- data-class review
- autonomy-tier review
- adversarial eval
- golden eval
- supply-chain signing
- tenant allowlist
- audit topic registration

Claude-code has plugin installation and managed settings primitives; Foundry should add governance and certification gates.

---

## A.8 Reference autonomous workflow examples

### A.8.1 Autonomous bugfix workflow

1. Goal created from issue.
2. Planner extracts acceptance criteria.
3. Explore agent maps relevant files and tests.
4. Test engineer identifies failing or missing test.
5. Executor gets write lease for narrow files.
6. Executor implements fix.
7. Runtime runs targeted tests.
8. Verifier reviews diff and test evidence.
9. If pass, evidence bundle is sealed.
10. If fail, debugger gets logs and either fixes or sends task back to planner.
11. Human sees final evidence, not every intermediate prompt.

Key difference from human Claude-code: the user does not need to approve every obvious read/edit/test action inside the policy envelope.

### A.8.2 Autonomous repository research workflow

1. Goal defines repo URL, branch, scope, and output artifact.
2. Policy allows clone/read only; no external mutation.
3. Explorer maps topology.
4. Multiple analysis workers inspect independent subsystems.
5. Lead synthesizes findings.
6. Verifier checks references and artifact completeness.
7. Report is written to configured path.
8. Evidence bundle records clone commit, file counts, commands, and known gaps.

This is the workflow actually approximated by this research task. A fully autonomous runtime would have made the clone, report, goal state, and verifier outcome first-class instead of reconstructing them from chat and local files.

### A.8.3 Autonomous capability publishing workflow

1. Goal: publish capability X.
2. Planner reads capability authoring standard.
3. Executor creates capability record and eval set.
4. Policy checks autonomy tier, data classes, audit topic, license posture.
5. Test runner executes golden/adversarial eval cohorts.
6. Security reviewer checks dangerous side effects.
7. Verifier requires signed artifacts and evidence bundle.
8. Capability becomes available only after registry update and policy approval.

This maps closely to Oyatie's canonical capability requirements.

### A.8.4 Autonomous migration workflow

1. Goal: migrate schema or tenant.
2. Planner creates up/down/dry-run/rollback tasks.
3. Policy envelope limits tenants/cells/regions.
4. Executor runs dry-run first.
5. Verifier validates migration output.
6. Runtime pauses for required high-tier approval if production mutation is involved.
7. Execution emits audit events per tenant/cell.
8. Rollback test evidence is attached.

Claude-code can help write such a migration interactively. A fully autonomous runtime must govern it as a stateful, auditable operation.

---

## A.9 What to reuse, modify, or replace from Claude-code

### Reuse conceptually

- Tool definition with schemas and validation.
- Centralized tool execution lifecycle.
- Pre/post hook events.
- Permission modes and dangerous-command classification.
- Bash and PowerShell validators.
- MCP normalization.
- Plugin discovery and managed settings.
- Subagent spawning and isolation concepts.
- Transcript/session persistence.
- Compaction and context summarization.
- VCR/replay harness idea.
- LSP-assisted code intelligence.

### Modify heavily

- Replace REPL-first orchestration with goal-first orchestration.
- Replace prompt approvals with policy envelopes.
- Replace slash-command workflows with declarative workflow specs.
- Replace conversation-only memory with operational state.
- Replace subagent conversations with scheduled task leases.
- Replace final-message verification with verifier state gates.
- Replace local telemetry with audit-chain event contracts.
- Replace bypass mode with scoped break-glass.

### Avoid copying

- Monolithic orchestration files.
- UI as the runtime source of truth.
- Human prompt loops as the primary permission mechanism.
- Unbounded background agents.
- Plugin trust by installation alone.
- Source-map/source-artifact leakage.
- Feature flags without owner/expiry/test governance.
- General-purpose production bypass for autonomous agents.

---

## A.10 Proposed Foundry autonomous-agent architecture

### A.10.1 Planes

1. **Goal Plane**
   - Owns goals, acceptance criteria, constraints, and completion state.

2. **Policy Plane**
   - Owns autonomy ceiling, capability authorization, data boundaries, tenant/region restrictions, and break-glass.

3. **Planning Plane**
   - Converts goals into task graphs and proof plans.

4. **Execution Plane**
   - Leases tasks to agents and runs capabilities.

5. **Verification Plane**
   - Runs tests, static checks, evals, reviewer agents, and evidence validation.

6. **Evidence Plane**
   - Stores logs, diffs, artifacts, audit events, and sealed evidence bundles.

7. **Human Exception Plane**
   - Handles approvals, denials, clarifications, pauses, and escalations.

### A.10.2 Core services

- Goal Service
- Task Graph Service
- Lease Manager
- Capability Registry
- Policy Evaluator
- Tool/Capability Runner
- Hook/Event Dispatcher
- Agent Scheduler
- Workspace Manager
- Verification Runner
- Evidence Store
- Audit Emitter
- Memory/Decision Store
- Escalation Service

### A.10.3 Critical invariant

No agent marks a goal complete directly. Completion requires:

1. all required tasks terminal,
2. acceptance criteria evaluated,
3. verifier verdict present,
4. policy gates pass,
5. evidence bundle sealed,
6. audit event emitted.

---

## A.11 Minimal migration path from Claude-code-style architecture

### Step 1: Put a goal wrapper around sessions

Keep the conversation engine, but every autonomous session must start with a goal object and acceptance criteria.

### Step 2: Add policy envelope execution

Allow the agent to execute pre-approved read/edit/test actions without interactive prompts, but only within explicit path/tool/network bounds.

### Step 3: Add verifier-required completion

Block “complete” unless a verifier process or verifier agent accepts evidence.

### Step 4: Turn subagents into leased workers

Keep subagent prompts, but wrap them in task lease metadata and write-scope rules.

### Step 5: Event-source tool execution

Persist every tool call, policy decision, hook result, file change, test run, and verifier verdict.

### Step 6: Build replay harness

Use VCR-like fixtures to replay full goals and verify autonomous workflows.

### Step 7: Certify capabilities

Move all tools/plugins/MCP servers through capability records with autonomy tiers and evals.

---

## A.12 Practical design recommendations for Oyatie / Foundry

1. **Make capability records the canonical extension point.** Do not let ad-hoc tools bypass policy metadata.
2. **Require `data_class` metadata at the capability boundary.** This is easier than retrofitting data boundaries later.
3. **Use Cedar or equivalent policy before tool execution, not after.** Logging a violation is not enough.
4. **Use append-only audit events for runtime decisions.** Agent memory is not audit.
5. **Default to T0/T1 autonomy.** Raise autonomy only by explicit policy.
6. **Separate human UX from runtime state.** Terminal, web UI, and API should all read the same event/state model.
7. **Make verifier agents skeptical by default.** They should check evidence, not trust executor summaries.
8. **Treat shell access as a high-risk capability.** It needs the strongest validators and the tightest policy envelope.
9. **Prefer small leased tasks over giant autonomous prompts.** This improves recovery and accountability.
10. **Make every blocker structured.** “I got stuck” should become a typed blocker with cause, evidence, and proposed next action.
11. **Store rejected alternatives.** Prevent future agents from redoing known-bad paths.
12. **Design for crash recovery from day one.** The runtime should be able to resume after process death without reading terminal scrollback.
13. **Never use docs/raw as authority.** Promote only reviewed conclusions into `docs/consolidated/` with traceability.

---

## A.13 Bottom-line conclusion

Claude-code already contains many of the right ingredients for autonomy: robust tool schemas, permission logic, shell hardening, hooks, subagents, MCP, plugins, transcripts, compaction, and replay ideas. The main gap is not the absence of primitives. The gap is the **center of gravity**.

For human use, the center is the REPL conversation.

For fully autonomous work, the center must be the durable goal and its governed execution lifecycle.

The most important design move for Foundry is therefore not to copy Claude-code's terminal product. It is to extract the underlying runtime lessons and rebuild them around:

`Goal → Policy Envelope → Task Graph → Leased Agents → Capability Runtime → Verification Gate → Evidence Bundle → Audit Chain`

That architecture preserves the best parts of Claude-code while avoiding the main risk: mistaking a powerful human-operated assistant for a production-grade autonomous agent operating system.

---
---

# Part B — Three-repo cross-cutting study (Claude session, 2026-05-10)

> **Status:** appended on 2026-05-10 by a separate Claude session.
> **Scope:** the prior Part A is a deep-dive on `claude-code/backup` only. Part B widens to a three-repo comparison — `claude-code/backup` + OMC + OMX — focused on the architectural patterns the user is implicitly evaluating when designing Oyatie's hook system, doc traceability requirements, and agent operating contract.
> **Sources:** three parallel research agents at Opus tier (45 + 65 + 37 file reads via `gh api`); see "Sources scanned" footer at the end of Part B.

## B.1 Table of Contents

1. [Executive synthesis](#b2-executive-synthesis)
2. [`claude-code/backup` recap (high-level only — Part A is the deep-dive)](#b3-claude-codebackup-recap)
3. [OMC — `oh-my-claudecode`](#b4-omc--oh-my-claudecode)
4. [OMX — `oh-my-codex`](#b5-omx--oh-my-codex)
5. [Cross-cutting patterns](#b6-cross-cutting-patterns)
6. [Recommendations for Oyatie](#b7-recommendations-for-oyatie)
7. [Open follow-ups](#b8-open-follow-ups)
8. [Sources scanned](#b9-sources-scanned)

## B.2 Executive synthesis

### One-line per repo

- **`jason931225/claude-code/backup`** — snapshot of leaked upstream Anthropic Claude Code source (commit `f5a40b86`, 2026-03-31). `main...backup` shows ahead-by-1 (README only). **Not a customization** — an archive. The user is studying upstream, not patching it. (Part A above covers this in depth.)
- **OMC** (`yeachan-heo/oh-my-claudecode`, v4.13.7) — multi-agent orchestration plugin for Claude Code. Ships as plugin + npm CLI + MCP server. 12 first-class skills, 19 agents, hooks for all 10 lifecycle events. Loops driven by `Stop` returning `decision:"block"` + reason → Claude continues the turn. State at `.omc/state/<mode>-state.json`.
- **OMX** (`Yeachan-Heo/oh-my-codex`, v0.16.3) — Codex equivalent. TS + 5 Rust crates. Single multi-event hook adapter (`dist/scripts/codex-native-hook.js`) for *all* Codex native events because Codex's hook surface is narrower (PreToolUse Bash-only, no SubagentStop). Same Stop-block-continuation primitive. Workflow keywords via `KEYWORD_TRIGGER_DEFINITIONS` (60 entries).

### The dominant cross-cutting pattern

**Hooks are prompt injectors first, gates second.** All three codebases treat `{additionalContext: "..."}` as the primary hook contract. Blocking is reserved for: `Stop` continuation (drives self-driving loops), `PreToolUse` Bash command-safety, model-config sanity (OMC's Bedrock/Vertex deadlock prevention), and a handful of permission gates. The three-archetype framing — **gate / inject / lifecycle** — is empirically how production agentic-orchestration code works.

The persistence-loop primitive is identical across OMC and OMX: per-mode JSON state file + Stop hook returning `decision:"block"` with a continuation message → runtime fires another turn. Cross-runtime portable. No daemon, no queue, no separate orchestrator. Two state-file reads and a JSON return.

### What upstream does that OMC/OMX leave on the table

The leaked-source snapshot is the most generative finding. Three upstream features are stronger than what OMC/OMX use:

1. **`paths:` skill frontmatter** (`src/skills/loadSkillsDir.ts`) — gitignore-style globs that auto-load a skill *only when context files match*. The activation function over the doc tree.
2. **Skill-bundled hooks** (`hooks:` frontmatter on a SKILL.md) — a skill carries its own hook bundle that activates while the skill is loaded.
3. **Async hooks** (`{async:true, asyncTimeout}`) registered in `AsyncHookRegistry`, rewaking the model on completion. Solves the long-blocking-hook problem cleanly.

Plus three more upstream features worth knowing:

- **`InstructionsLoaded` event** — fires when CLAUDE.md / memory is (re)loaded. Doc-anchor refresh hook.
- **Coordinator mode** (`CLAUDE_CODE_COORDINATOR_MODE=1`) — coordinator system prompt + AgentTool + SendMessage/TaskStop, results arrive as `<task-notification>` XML.
- **4-tier memory taxonomy** (`user/feedback/project/reference`) with a JSON-schema-constrained side-query selector that picks ≤5 memory files.

## B.3 `claude-code/backup` recap

> Part A above is the comprehensive deep-dive (~3,000 lines). Headlines only here, framed for the cross-cutting comparison.

The `backup` branch is a snapshot of the leaked upstream Anthropic Claude Code source. `main...backup` differs by 1 file (README cosmetics). The repo is essentially the upstream archive. Hook events available upstream are richer than what either OMC or OMX exposes:

`PreToolUse`, `PostToolUse`, `PostToolUseFailure`, `PermissionDenied`, `PermissionRequest`, `PreCompact`, `PostCompact`, `SessionStart`, `SessionEnd`, `SetupHook`, `Stop`, `StopFailure`, `SubagentStart`, `SubagentStop`, `TeammateIdle`, `TaskCreated`, `TaskCompleted`, `ConfigChange`, `CwdChanged`, `FileChanged`, `InstructionsLoaded`, `UserPromptSubmit`, `Notification`, `Elicitation`, `ElicitationResult`, `WorktreeCreate`.

Hook return shape supports `decision`, `permissionDecision`, `additionalContext`, `updatedInput`, `updatedMCPToolOutput`, `watchPaths`, `initialUserMessage`. Async hooks are first-class: `{async: true, asyncTimeout}` registers in `AsyncHookRegistry`, rewakes the model on completion via task-notification injection.

Skill frontmatter (`src/skills/loadSkillsDir.ts`) supports: `name`, `description`, `whenToUse`, `allowed-tools`, `argument-hint`, `arguments`, `model`, `disable-model-invocation`, `user-invocable`, **`hooks`**, **`paths`**, `version`, `effort`, **`executionContext`**, `agent`. The bolded fields are the under-exploited features OMC/OMX don't use.

Memory architecture (`src/memdir/`): typed memory in 4 classes (user/feedback/project/reference). `findRelevantMemories.ts` runs a side-query against Sonnet with a JSON-schema-constrained selector (max 5 picks). `MEMORY.md` index is ≤200 lines / ≤25KB.

Coordinator mode (`src/coordinator/coordinatorMode.ts`): swaps in a coordinator system prompt when `CLAUDE_CODE_COORDINATOR_MODE=1`, with workers spawned via `AgentTool`, communicating via `SendMessageTool`, stopped via `TaskStopTool`. Results arrive as `<task-notification>` user-role XML.

## B.4 OMC — `oh-my-claudecode`

### B.4.1 Project elevator

`oh-my-claudecode` (v4.13.7, MIT, by Yeachan Heo) is a multi-agent orchestration plugin for Claude Code. Ships through three surfaces simultaneously: a Claude Code plugin (`.claude-plugin/plugin.json` + `.claude-plugin/marketplace.json`), an npm CLI published as `oh-my-claude-sisyphus` (binaries `omc`, `omc-cli`, `oh-my-claudecode`), and an MCP server (`bridge/mcp-server.cjs`, declared via `.mcp.json`).

The TypeScript source lives in `src/` and compiles to `dist/`; the plugin runtime is wired entirely through `.mjs` / `.cjs` hook scripts under `scripts/` so it works without a build step. A second runtime (`bridge/team-mcp.cjs`, `bridge/team.js`) launches Codex / Gemini / Claude CLI workers in tmux panes for the `omc team` family.

### B.4.2 Top-level architecture

```
User prompt -> Hooks (10 events) -> Skill resolution -> Agent dispatch -> State persistence
                  |                       |                   |                |
              hooks/hooks.json    skills/<name>/SKILL.md   agents/<name>.md   .omc/state/
              scripts/*.mjs       skill-injector.mjs       Task(...)          *-state.json
```

- Plugin manifest: `.claude-plugin/plugin.json` declares 12 top-level skills, points at `./.mcp.json`, lists `./commands/` for slash-command discovery.
- Hook entry: `hooks/hooks.json` registers the runner `scripts/run.cjs` for every event; `run.cjs` `spawnSync`s the actual `.mjs` hook with `process.execPath` (avoids the Windows `sh -> find-node.sh` PE32+ binary problem; issues #909/899/892/869).
- Skills: 37 `skills/<name>/SKILL.md` files (YAML frontmatter + body). Only 12 are first-class plugin skills; the rest are auto-discovered.
- Agents: 19 `agents/<name>.md` files with `model:` and `disallowedTools:` frontmatter. Invoked via `Task(subagent_type="oh-my-claudecode:<name>")`.
- MCP / state tools surfaced through `bridge/mcp-server.cjs`: `state_read`, `state_write`, `notepad_*`, `project_memory_*`, `wiki_*`, `lsp_*`, `ast_grep_*`.

### B.4.3 Key abstractions

- **Skill** — YAML+markdown at `skills/<name>/SKILL.md` whose body is *injected as a system prompt* when invoked. Frontmatter: `name`, `description`, `argument-hint`, `level`, `user-invocable`, `aliases`. Reusable behavior, not code.
- **Agent** — YAML+markdown at `agents/<name>.md` defining a sub-prompt + `model` tier + `disallowedTools`. Invoked via `Task()`; never code, always prompt.
- **Hook** — Node script registered in `hooks/hooks.json` against one of 10 lifecycle events. Returns JSON envelope on stdout (`{decision, reason}` or `{continue, hookSpecificOutput}`).
- **Mode** — named, persisted execution loop (ralph / autopilot / ultrawork / ultrapilot / ultraqa / pipeline / team / omcTeams / swarm). Each owns a `<mode>-state.json` and is enforced by `persistent-mode.mjs`.
- **State** — JSON blobs under `<projectRoot>/.omc/state/` (and per session `state/sessions/<sessionId>/`) plus `~/.omc/state/` fallback. Resolved through `scripts/lib/state-root.mjs`.
- **Notepad** — compaction-resistant memory at `.omc/notepad.md`, with priority/working/manual tiers exposed via MCP tools.
- **Project memory** — durable knowledge at `.omc/project-memory.json`, registered each `SessionStart` and read each tool turn.
- **Magic keyword** — sanitized substring (`autopilot`, `ralph`, `ulw`, `ralplan`, `ccg`, `ultrathink`, `tdd`, `deepsearch`, etc.) detected by `keyword-detector.mjs`, which injects a skill-invocation directive.

### B.4.4 Lifecycle of `/ralph`

1. **Input**: user types `ralph fix src/foo.ts`.
2. **`UserPromptSubmit` fires** → two hooks run in series:
   - `keyword-detector.mjs` sanitizes the prompt (strips XML, code fences, file paths, URLs, quoted spans, magic-keyword echo blocks), then `hasExplicitInvocationContext()` decides whether the keyword is *intent* vs. *reference*. If intent, writes a `<system-reminder>` invocation directive into `hookSpecificOutput.additionalContext`.
   - `skill-injector.mjs` walks `~/.claude/skills/omc-learned/`, `~/.omc/skills/`, and `<cwd>/.omc/skills/`, parses YAML frontmatter for `triggers:`, dedups against `.omc/state/skill-sessions-fallback.json` (per-session map with 1-hour TTL), and injects matching learned-skill descriptors capped at 3 KB.
3. **Pre-execution gate**: the `ralplan` skill body and `pre-tool-enforcer.mjs` PreToolUse hook inspect the prompt for "concrete anchors" (file paths, issue numbers, identifiers in camelCase / snake_case / PascalCase). If absent and the prompt is ≤15 words, the gate redirects to `/ralplan` consensus first. Bypass with `force:` or `!`.
4. **Skill body executes**: the `Steps` block in `skills/ralph/SKILL.md` runs as an in-context plan. Step 1 generates `prd.json` (session-scoped at `.omc/state/sessions/<id>/prd.json`). Step 3 delegates to `Task(subagent_type="oh-my-claudecode:executor", model="...")`. Steps 7/7.5/7.6 run reviewer (architect / critic / Codex via `omc ask codex --agent-prompt critic`), then mandatory `Skill("ai-slop-cleaner")` pass, then regression re-run.
5. **Stop fires after every assistant turn** → `persistent-mode.mjs` reads `ralph-state.json`, increments `iteration`, returns `{decision:"block", reason:"[RALPH LOOP - ITERATION i/max] Work is NOT done..."}` until `iteration >= max_iterations` (default 100, +10 once, hard-capped by `getHardMaxIterations()` when `OMC_SECURITY=strict`), or `/oh-my-claudecode:cancel` is called. Cancel writes `cancel-signal-state.json` with 30-second TTL that `persistent-mode.mjs` honours.
6. **Output**: at cancel time, summary printed; cancel removes `ralph-state.json` and skill-active-state slots.

### B.4.5 Hook system

Declared in `hooks/hooks.json`. Every entry is `node "$CLAUDE_PLUGIN_ROOT"/scripts/run.cjs <hook>.mjs` with a `timeout` in seconds. Events used:

- `UserPromptSubmit` — `keyword-detector.mjs`, `skill-injector.mjs`
- `SessionStart` — `session-start.mjs`, `project-memory-session.mjs`, `wiki-session-start.mjs`
- `PreToolUse` — `pre-tool-enforcer.mjs`
- `PostToolUse` — `post-tool-verifier.mjs`, `project-memory-posttool.mjs`, `post-tool-rules-injector.mjs`
- `PostToolUseFailure`, `PermissionRequest:Bash`, `SubagentStart`, `SubagentStop` (also `verify-deliverables.mjs`)
- `PreCompact` — three hooks pre-stage notepad / project-memory / wiki for compaction
- `Stop` — `context-guard-stop.mjs`, `persistent-mode.mjs`, `code-simplifier.mjs`
- `SessionEnd` — `session-end.mjs`, `wiki-session-end.mjs`

**Three archetypes in active use:**

- **Gate (block-or-pass):** `persistent-mode.mjs` returns `{decision:"block", reason}` when an active mode is detected; `pre-tool-enforcer.mjs` blocks `Task(model="sonnet")` calls under `CLAUDE_CODE_USE_BEDROCK=1` or `CLAUDE_CODE_USE_VERTEX=1` without a resolvable provider-specific model id.
- **Prompt-injection:** `keyword-detector.mjs` and `skill-injector.mjs` return `{continue:true, hookSpecificOutput:{hookEventName, additionalContext}}` — Claude Code splices `additionalContext` into the next assistant turn as `<system-reminder>`. The persistent-mode `reason` field is *also* a prompt injection (`[RALPH LOOP - ITERATION 3/100] ...`).
- **Lifecycle / checkpoint:** `session-start.mjs` writes a session-started marker (`{omcRoot}/state/sessions/<id>/<marker>`) with `boot_id`, `pid`, `started_at`; `pre-compact.mjs` snapshots notepad before compaction; `subagent-tracker.mjs` records every Task spawn/stop.

Universal kill switches: `DISABLE_OMC=1` and `OMC_SKIP_HOOKS=keyword-detector,notepad` — every hook reads the env at top of `main()`.

### B.4.6 Skill / agent model

**Skill files** are markdown with bespoke XML-ish tags (`<Purpose>`, `<Use_When>`, `<Steps>`, `<Final_Checklist>`, `<Tool_Usage>`, `<Examples>`, `<Escalation_And_Stop_Conditions>`). Not parsed structurally — *rhetorical scaffolding* the LLM follows. `skill-injector.mjs:parseSkillFrontmatterFallback` parses only `name`, `description`, `triggers:`. Discovery walks three roots: `<cwd>/.omc/skills/`, `~/.omc/skills/`, `<CLAUDE_CONFIG_DIR>/skills/omc-learned/` — project wins ties (via `realpathSync` dedup).

**Agent files** carry `name`, `description`, `model`, `level`, `disallowedTools` in frontmatter. Body is `<Agent_Prompt>` block with `<Role>`, `<Constraints>`, `<Investigation_Protocol>`, `<Tool_Usage>`, `<Output_Format>`. `architect.md` and `critic.md` declare `disallowedTools: Write, Edit` — hard READ-ONLY enforcement that Claude Code respects.

**Routing:** no central routing process. Each skill body *names* the agents it dispatches to via `Task(subagent_type="oh-my-claudecode:<name>", model="...")`. Tier guidance lives at `docs/shared/agent-tiers.md`. The `omc-reference` skill is the on-demand catalog.

### B.4.7 State + persistence

Layout (resolved through `scripts/lib/state-root.mjs`):

- `<cwd>/.omc/state/<mode>-state.json` (project-scoped, primary)
- `<cwd>/.omc/state/sessions/<sessionId>/<file>.json` (session-scoped; Claude Code 1.x supplies session_id via stdin JSON)
- `~/.omc/state/<mode>-state.json` (global fallback)
- `<cwd>/.omc/notepad.md`, `<cwd>/.omc/project-memory.json`, `<cwd>/.omc/plans/<name>.md`, `<cwd>/.omc/research/`, `<cwd>/.omc/logs/`

State shape (visible from `persistent-mode.mjs` reads): `{active, session_id, project_path, prompt, iteration, max_iterations, reinforcement_count, started_at, updated_at, last_checked_at, phase, current_objective, awaiting_confirmation, awaiting_confirmation_set_at}`. Stale states (>2 hr by `STALE_STATE_THRESHOLD_MS`) are ignored.

**Tombstoning:** `skill-active-state.json.active_skills[mode].completed_at` blocks SessionStart from re-loading a just-cancelled mode for 24 hours (`WORKFLOW_SLOT_TOMBSTONE_TTL_MS`).

### B.4.8 Loop / consensus mechanics

- **ralplan consensus** (`skills/ralplan/SKILL.md`) is *prompt-driven*, not coded. Steps 1–5: Planner draft → optional user review (`--interactive`) → Architect review (must include steelman antithesis + tradeoff tension) → Critic verdict (`APPROVE`/`ITERATE`/`REJECT`) → loop max 5 iterations. Two key invariants: "Steps 3 and 4 MUST run sequentially. Do NOT issue both agent Task calls in the same parallel batch" and "On Critic approval, mark the plan `pending approval` unless explicit execution approval has already been captured." Enforced by *agent constraints* and *skill body language*, not by a state machine.
- **ralph persistence** (`skills/ralph/SKILL.md` + `scripts/persistent-mode.mjs`) *is* coded. Stop hook reads `ralph-state.json`; if `iteration < max_iterations`, increments and returns `decision:"block"` with continuation reason. Stop conditions (priority order): context-limit / context >= 95% / user abort / auth error / scheduled wakeup / pending background task — all bail with `continue:true`. Mode-continue ladder: `ralph → autopilot → ultrapilot → swarm → pipeline → team → omcTeams → ultraqa → ultrawork`.

### B.4.9 Novel design decisions (OMC)

- **Hooks-as-prompt-injection is first-class.** `keyword-detector.mjs` and `skill-injector.mjs` don't *block* — they shape the *next* turn. `persistent-mode.mjs` does the inverse: pretends the assistant said something incomplete by injecting `[RALPH LOOP ...]`. Block + reason as covert prompt is how OMC fakes "modes" with no language-runtime state.
- **Sanitize prompts in two places.** `keyword-detector.mjs:sanitizeForKeywordDetection` strips code fences / XML / URLs / paths / quotes / magic-keyword echo blocks before keyword matching, *and* `sanitizePromptForState()` does it again before persisting — preventing pasted hook output from re-triggering itself. The system actively defends against its own echoes.
- **Cross-platform Node bootstrap.** `scripts/run.cjs` exists *solely* to dodge the Windows `/usr/bin/sh` PE32+ binary problem by re-execing through `process.execPath`. Has a fallback that walks `dirname(CLAUDE_PLUGIN_ROOT)` for sibling semver dirs in case `CLAUDE_PLUGIN_ROOT` points at a stale plugin version (#1007).
- **Read-only guardrails in agent frontmatter.** `architect.md`, `critic.md`, `code-reviewer` declare `disallowedTools: Write, Edit`. Reviewer roles literally cannot mutate state — enforced by Claude Code's tool gating, not by prompt convention.
- **Tombstone ledger.** `skill-active-state.json.active_skills[mode].completed_at` prevents `session-start.mjs` from auto-resurrecting a mode the user just cancelled. Check (`isWorkflowSlotTombstonedForMode`) duplicated in `persistent-mode.mjs:isAuthoritativeModeActive` — defense in depth at session boundaries.
- **PreToolUse model gate.** `pre-tool-enforcer.mjs` rejects `Task(model="sonnet")` under `CLAUDE_CODE_USE_BEDROCK=1` or `CLAUDE_CODE_USE_VERTEX=1` unless one of `OMC_SUBAGENT_MODEL`, `CLAUDE_CODE_BEDROCK_*_MODEL`, `ANTHROPIC_DEFAULT_*_MODEL` resolves. Catches a class of misconfigurations no doc lookup would surface.

## B.5 OMX — `oh-my-codex`

> All paths refer to OMX `HEAD = 09d6fd05` (v0.16.3, 2026-05-09).

### B.5.1 Project elevator

OMX is a workflow + runtime layer for OpenAI Codex CLI. Does not replace Codex; wraps and instruments it. Shipped as a single npm package (`oh-my-codex`, MIT, Node 20+, with five Rust crates packaged inside) that installs a global `omx` binary plus a Codex plugin bundle.

### B.5.2 Top-level architecture

Two languages cohabit. **TypeScript** (`src/`, ~120 modules) holds CLI, hooks, modes, state, MCP servers, orchestration. **Rust** (`crates/`) provides five binaries:

- `omx-runtime`
- `omx-runtime-core` (authority/dispatch/mailbox/replay)
- `omx-mux` (tmux abstraction)
- `omx-explore-harness` (read-only repo lookup)
- `omx-sparkshell` (bounded shell verification)

Entry points: `bin: { omx: "dist/cli/omx.js" }`, `src/cli/index.ts` (4458 lines, dispatches subcommands), `src/scripts/codex-native-hook.ts` (3205 lines, single multiplexer for **all** Codex native hook events).

### B.5.3 Key abstractions

1. **Skill** — discoverable workflow at `plugins/oh-my-codex/skills/<name>/SKILL.md`. YAML frontmatter (`name`, `description`, optional `argument-hint`). Activated by `$<keyword>` tokens.
2. **Tracked workflow mode** — stateful skill with persisted lifecycle in `.omx/state/<mode>-state.json` (e.g. `ralplan`, `ralph`, `team`, `autopilot`, `autoresearch`, `ultrawork`, `ultraqa`, `deep-interview`).
3. **Keyword registry** — single TypeScript constant table mapping prompt tokens → skills with priorities (`src/hooks/keyword-registry.ts:8`, 60 entries).
4. **Workflow-transition policy** — `src/state/workflow-transition.ts` + `workflow-transition-reconcile.ts`. Decides allow / overlap / auto-complete / deny for skill switches.
5. **Native hook adapter** — `dist/scripts/codex-native-hook.js`, the OMX-managed wrapper Codex calls.
6. **OMX plugin event** — internal vocabulary (`session-start`, `keyword-detector`, `pre-tool-use`, `post-tool-use`, `stop`, `session-end`, `turn-complete`, `session-idle`) emitted to user plugins via `HookEventEnvelope` (`src/hooks/extensibility/types.ts:30-42`).
7. **Authority lease + mailbox** — Rust `crates/omx-runtime-core`: leader election, mailbox messages, replay state for tmux team coordination.
8. **Goal-mode artifact** — Codex thread-level objective (`get_goal`/`update_goal`); skills like ralph use it as the canonical "is the work done" oracle.

### B.5.4 Lifecycle of `$ralph`

1. User types prompt in Codex CLI.
2. Codex fires `UserPromptSubmit`. Native hook config in `.codex/hooks.json` invokes `dist/scripts/codex-native-hook.js`.
3. `codex-native-hook.ts` parses payload, calls `detectKeywords()` → matches `$ralph` priority 9.
4. `recordSkillActivation()` → `evaluateWorkflowTransition()` → `reconcileWorkflowTransition()`. Writes `.omx/state/ralph-state.json` (root or session-scoped).
5. Adapter emits JSON `{ additionalContext: "<routing message>" }`. Codex injects that context for the model.
6. Model loads `plugins/oh-my-codex/skills/ralph/SKILL.md` and runs `<Steps>` (context snapshot → delegate → verify → architect verification → mandatory deslop → regression re-verify → goal-mode `update_goal`).
7. After model speaks, Codex fires `Stop`. Native adapter calls `shouldContinueRun()`; if ralph is non-terminal, returns `{ decision: "block", reason: "..." }` per native Stop continuation contract — Codex automatically fires another model turn (this is OMX's persistence loop).
8. On terminal lifecycle (`finished` | `blocked` | `failed` | `userinterlude` | `askuserQuestion`), state is sealed, `skill-active-state.json` reconciled, Stop returns no continuation.

### B.5.5 Hook system in Codex's runtime

Codex hook surface vs. Claude Code (per `docs/codex-native-hooks.md`):

| OMC name | Codex native | OMX status |
|---|---|---|
| SessionStart | SessionStart | native |
| UserPromptSubmit | UserPromptSubmit | native |
| PreToolUse | PreToolUse | **Bash-only** (native-partial) |
| PostToolUse | PostToolUse | **Bash-only** (native-partial) |
| Stop | Stop | native — uses `decision:"block"` continuation |
| PreCompact / PostCompact | yes | native (no-stdout) |
| SubagentStop | (none) | not supported |
| ask-user-question | (none) | runtime fallback |
| session-end / session-idle | (none) | runtime/notify fallback |

Three differences vs. Claude Code:

- **One adapter, all events.** `.codex/hooks.json` registers a single OMX wrapper command; the wrapper internally branches on `CodexHookEventName`. Claude Code typically runs separate scripts per event.
- **Hook output is prompt injection by default.** Adapter emits JSON `{ additionalContext: "...", systemMessage?: "..." }`. The "block" lever is exclusive to `Stop` (continuation) and to `PreToolUse` Bash matchers (deny).
- **Trust state is persisted** in `.codex/config.toml` `hooks.state."<file>:<event>:<group>:<handler>".trusted_hash` so Codex doesn't re-prompt.

User plugins (`.omx/hooks/*.mjs`) layer on top via `dispatchHookEventRuntime()` and never see raw Codex names — they receive the OMX-canonical vocabulary.

### B.5.6 Skill / agent / capability model

Two parallel skill trees ship:

- `skills/<name>/SKILL.md` — legacy native-agents form, copied to `~/.codex/skills/` by `omx setup` (43 skills).
- `plugins/oh-my-codex/skills/<name>/SKILL.md` — Codex plugin marketplace bundle (29 skills, mirrored subset).

Roles (Codex equivalent of "subagents") live in `prompts/*.md` (30 files). Read-only role prompts loaded by name.

Discovery is two-tier:

- **Workflow keywords** → `KEYWORD_TRIGGER_DEFINITIONS`. Deterministic, prompt-injection-driven.
- **Roles** → routed through `triagePrompt()` (`src/hooks/triage-heuristic.ts`) when no keyword matches. Advisory only.

### B.5.7 State + persistence

Workspace-scoped, never global. Layout (per `docs/STATE_MODEL.md`):

- `.omx/state/<mode>-state.json` (root scope) — authoritative
- `.omx/state/sessions/<session_id>/<mode>-state.json` — session-scoped, wins over root
- `.omx/state/skill-active-state.json` — compatibility/visibility layer (NOT a decision authority)
- `.omx/context/{slug}-{timestamp}.md` — pre-context intake snapshots
- `.omx/plans/`, `.omx/specs/` — planning artifacts (PRD + test-spec) that gate execution
- `.omx/logs/hooks-YYYY-MM-DD.jsonl` — plugin dispatch log
- `.omx/runtime/codex-home/<session>/` — session-scoped Codex home mirror
- `omx_wiki/` — checked-in knowledge base (markdown-first, search-first; **not** under `.omx/`)

Read precedence: explicit session > current session > root fallback. Stale roots get terminalized on reconciliation.

### B.5.8 Loop / consensus mechanics

OMX has direct analogues to OMC's ralph/ralplan/autopilot, with one runtime difference: Codex's native `Stop` `decision:"block"` returning `reason` is the **mechanism** for persistence — there is no separate ralph daemon.

- **`$ralph`** — persistence loop. Stop hook reads `shouldContinueRun()`; while non-terminal, returns block-continuation. Mandatory deslop pass + regression re-verify before completion. Goal-mode integration.
- **`$ralplan`** — alias for `$plan --consensus`. Triggers Planner → Architect → Critic → re-review (max 5 iterations). RALPLAN-DR structured deliberation.
- **`$autopilot`** — strict three-phase loop `$ralplan → $ralph → $code-review`. Non-clean review returns to `$ralplan`.
- **`$team`** — tmux-backed parallel workers. Implementation in `src/team/` plus Rust `omx-runtime-core` providing `AuthorityLease`, `DispatchLog`, `MailboxLog`, `ReplayState`. Workers can be mixed (`OMX_TEAM_WORKER_CLI_MAP=codex,claude`).

Transition rules: `deep-interview→ralplan→{team|ralph|autopilot}` are auto-complete; `team+ralph` and `ultrawork+anything` are overlap; execution→planning rollback is denied except autopilot's review loopback.

### B.5.9 OMC ↔ OMX delta

| Concern | OMC (Claude Code) | OMX (Codex) |
|---|---|---|
| Hook event vocabulary | Native (10+ events) | Subset only; SubagentStop / session-end / session-idle / ask-user-question are runtime fallbacks |
| Hook script registration | Per-event in `settings.json` | One adapter for all events in `.codex/hooks.json` |
| Prompt injection | `additionalContext` and stdout-as-context patterns | Same field name; uniform JSON shape; trust hashes recorded in `config.toml` |
| Tool-use gating | Per-tool matchers | Bash-only matchers; non-Bash interception is runtime fallback |
| Persistence loop | Stop hook with `block` returning reason; OR external orchestrator | Codex Stop's `decision:"block"` + `reason` is **the** mechanism |
| Parallel team mode | Native Claude teams primitive | tmux+Rust runtime via `omx-mux` and `omx-runtime-core` |
| User question | Native AskUserQuestion | `omx question` popup over leader pane (no native equivalent) |
| Skill discovery | `~/.claude/skills/` and plugin form | Dual: legacy `~/.codex/skills/` + plugin marketplace |
| Runtime languages | TS + (sometimes) Rust crates | TS + 5 Rust crates packaged in npm |

**Why divergence is forced:**

- Codex's Stop continuation contract is purpose-built for self-driving loops; OMC needs an outer orchestrator + Stop hook to achieve the same.
- Codex restricts native PreToolUse/PostToolUse to Bash; security-sensitive interception of MCP/file tools must live in OMX runtime.
- Codex doesn't ship a native team primitive; OMX rebuilt it on tmux+Rust.

### B.5.10 Novel design decisions (OMX)

1. **Single-binary, multi-event hook adapter.** All seven Codex native hook events route through `dist/scripts/codex-native-hook.js`. Trades a fat module for one trust-hash entry per event group and a single deployment surface.
2. **Hooks-as-prompt-injectors as the default.** Most events emit `{ additionalContext: "..." }` rather than blocking. Blocking is reserved for: Stop loop continuation, PreToolUse Bash matchers (Lore commit guard, `rm -rf dist`, document-refresh), and the `omx question` deep-interview gate.
3. **Lore Commit Protocol + Document-Refresh advisory MVP** as **per-PR Bash hooks**. PreToolUse reads `git diff --cached --name-status` and warns when a mapped source change lacks a docs/spec refresh; the suppression line `Document-refresh: not-needed | <reason>` is built into the warning UX.
4. **Two skill trees, deliberate split.** Legacy `/skills/` vs plugin `plugins/oh-my-codex/skills/`. README:68 explicitly notes plugin install archives stale legacy files so they don't shadow plugin behavior.
5. **Rust-backed coordination kernel.** `omx-runtime-core` ships authority leases, dispatch log with state machine, mailbox log, replay state — primitives for distributed-system-style team execution that don't exist in Codex itself.
6. **`omx_wiki/` is checked-in, not state.** Markdown-first, search-first, not vector-first. Native SessionStart can surface bounded wiki context.
7. **Combined workflow state contract.** OMX explicitly allows `team+ralph` and `team+ultrawork` overlaps and rejects everything else.

## B.6 Cross-cutting patterns

### B.6.1 Hooks-as-prompt-injection is the dominant pattern

All three codebases treat `{additionalContext: "..."}` as the primary hook contract. Blocking is reserved for a narrow set: Stop continuation (drives self-driving loops), PreToolUse Bash command-safety, permission gating, model-config sanity. The 3-archetype framing — gate / inject / lifecycle — is empirically how production agentic-orchestration code is shaped.

### B.6.2 State + persistence pattern is identical (OMC ↔ OMX)

OMC and OMX share the exact same state pattern:

- Per-mode JSON file at `.<runtime>/state/<mode>-state.json`
- Session-scoped subdirs: `.<runtime>/state/sessions/<session_id>/<mode>-state.json`
- Compatibility layer: `skill-active-state.json` (visibility, NOT decision authority)
- Read precedence: explicit session > current session > root fallback
- Stale roots terminalized on reconciliation, not resurrected

The persistence loop is two state-file reads + a JSON return from the Stop hook. No daemon, no queue, no separate orchestrator. Cross-runtime portable.

### B.6.3 Skills as prompt-injected behavior

In all three codebases, skills are NOT code — they are markdown documents whose body gets injected as a system prompt when invoked. The XML-ish tags (`<Purpose>`, `<Use_When>`, `<Steps>`, `<Final_Checklist>`, `<Tool_Usage>`) are *rhetorical scaffolding the LLM follows*, not parsed structure.

This makes "skill" a doc-system concept, not a code-system concept. Implication for Oyatie: skills can live in `docs/consolidated/standards/` as Reference-class docs with frontmatter `triggers:` + `paths:`.

### B.6.4 The leaked-source archive shows the user's preference for studying upstream

`jason931225/claude-code/backup` is not a customization layer — it's an archive of upstream. The user's design instincts are formed by reading upstream Anthropic source, not by patching it. That means Oyatie's design lineage runs:

upstream Claude Code source → user's design preferences → Oyatie agentic-doc-system

…and OMC/OMX are *separate* design lineages the user borrows from but does not own. Oyatie should not treat OMC/OMX as authoritative — only as evidence of what works in production for *other* people's projects.

### B.6.5 Three features upstream has that OMC/OMX leave on the table

These are the highest-leverage borrows:

- **`paths:` skill frontmatter** — gitignore-style globs that auto-load a skill only when context files match. Path-scoped activation function over the doc tree.
- **Skill-bundled hooks** (`hooks:` frontmatter) — a skill carries its own hook bundle that activates while the skill is loaded.
- **Async hooks** (`{async:true, asyncTimeout}`) — register in `AsyncHookRegistry`, rewake the model on completion. Solves long-blocking-hook problem.

Each of these is a ~10-line frontmatter pattern that's been built into the upstream runtime. Borrowing them is cheap.

### B.6.6 The cohesion-thesis at the docs level

OMX's `docs/STATE_MODEL.md` is the single canonical state-model contract. OMC has the same content but spread across several files. Oyatie's `docs/consolidated/` thesis ("one canonical tree, one file per question") matches OMX's shape. Symphony's single-SPEC.md confirms the pattern at a smaller scope.

The recurring lesson: **single canonical doc per concern, machine-traceable, end with a real checklist / contract / state diagram**. OMX does this for state. Symphony does this for service spec. Oyatie should do this for every Tier-1 concern.

## B.7 Recommendations for Oyatie

### B.7.1 High-leverage borrows (do first)

1. **Adopt `paths:` skill-frontmatter triggering.** The `oya-governance-*` lanes are already path-scoped; promote them to skills with `paths:` frontmatter so the right doc auto-injects when matching files appear in context. Replaces the "always-loaded skills" pattern with implicit per-edit selection. The single most powerful borrow.
2. **Formalize a Hook Output Contract** in `DOC-CATALOG.md` — every blocking hook emits exactly one doc anchor; anchor MUST resolve to an H2 ending in checklist-shape. Mirror OMC's stop-block-with-reason pattern but require structured anchor instead of free prose.
3. **Stop-hook + state-file as the cross-runtime persistence primitive.** Build it once for Oyatie's `oya-governance-*` lanes; works on Claude (OMC pattern) and Codex (OMX pattern) without modification.
4. **Wrap slow lanes as async hooks.** `oya-governance-{cohesion, manifest-mirror, prevention-replay}` are slow. `{async:true, asyncTimeout: <s>}` returns immediately; rewakes the agent on completion. Avoids the in-line block.
5. **Hook on `InstructionsLoaded`** for doc-anchor refresh. Whenever AGENTS.md / CONSTITUTION.md is reloaded, fire `oya-governance-authority-cohesion` to verify the chain still matches.

### B.7.2 Medium-leverage borrows

6. **Mirror upstream's typed-memory taxonomy.** MISTAKES-LEDGER → `feedback`; per-incident postmortems → `project`; vocabulary → `reference`. Side-query selector then surfaces relevant rows automatically without burning context every turn.
7. **Coordinator mode for multi-agent dispatch.** Don't reinvent worker spawn briefs. Lift upstream's `<task-notification>` XML format and `SendMessage` / `TaskStop` primitives.
8. **Runtime-neutral state directory.** `.omc/` and `.omx/` lock portability at the file-tree level. Pick `.oyatie/` (or commit to mirroring) before the surface ossifies.
9. **Tombstones for "lane already passed this SHA".** Cancel-signal + 24 h tombstone = OMC's solution to "don't re-arm a just-cancelled mode." For Oyatie's CI-lane policy, write `.oyatie/state/lanes/<lane>.completed_at` after a green run; PreToolUse blocks until that file is older than the last commit on the branch.
10. **Single canonical doc per concern.** Adopt OMX's `docs/STATE_MODEL.md` shape for every Tier-1 concern. Lock it down with a per-doc length cap + class-shape lane.

### B.7.3 What to avoid borrowing

- **Magic-keyword routing.** OMC's "12 modes from one prompt" UX is great for OMC's audience; for Oyatie with named ADRs and named lanes, explicit routing is better for traceability.
- **Implicit state schemas.** OMC's `*-state.json` is defined only by reads in `persistent-mode.mjs`. Drift in field names silently breaks enforcement. Define a JSON Schema next to each lane.
- **Aspirational checklists.** OMC's `<Final_Checklist>` blocks are prompt-only — no validator parses them. Oyatie's terminal checklists must be machine-validated (every box maps to a lane / artifact / command).
- **Dual skill trees.** OMX explicitly notes the `/skills/` vs `plugins/oh-my-codex/skills/` split is a "real failure mode." Oyatie should ship one tree, period.
- **Two-place sanitization as the only echo defense.** OMC's `keyword-detector.mjs` sanitizes prompts twice (input AND state-write) to prevent self-echo. That's a workaround for the absence of typed messages. If Oyatie's hooks output structured JSON anchors instead of prose, echo defense is moot.

## B.8 Open follow-ups

Each of these is a self-contained investigation worth running separately:

1. **`feature()` flag system** in upstream (`bun:bundle` dead-code elimination tied to `process.env.USER_TYPE === 'ant'`). Could be the model for Oyatie's capability tiers.
2. **`executionContext: 'fork'`** in upstream skill frontmatter. Unclear whether it forks a process, a context, or a worker — investigate before adopting.
3. **Skill-bundled hooks pattern** — when does a skill *bundle* a hook vs. when does the hook live separately? Pattern under-explored in OMC/OMX.
4. **`TEAMMEM` sync semantics** in upstream `services/teamMemorySync/` — multi-user memory mirror with secret guards.
5. **OMX's Lore Commit Protocol mechanics** — `git diff --cached --name-status` PreToolUse gate that warns when source changes lack docs refresh.
6. **OMC's `post-tool-rules-injector.mjs`** — reads the file the model just touched, injects matching rule blob from `.claude/rules`, `.cursor/rules`. Could be the canonical pattern for "every file edit triggers its standards."
7. **OMC's tombstone ledger duplication** (`isWorkflowSlotTombstonedForMode` checked in two places). Defense in depth against stale state at session boundaries.
8. **The upstream `feature()` table** — full enumeration of `PROACTIVE`, `KAIROS`, `AGENT_TRIGGERS`, `MONITOR_TOOL`, `COORDINATOR_MODE`, `TEAMMEM`, `WORKFLOW_SCRIPTS`, `TRANSCRIPT_CLASSIFIER`. Each is a capability that's either on or off.

## B.9 Sources scanned

- 2026-05-10 — three parallel research agents at Opus tier:
  - Agent A: `yeachan-heo/oh-my-claudecode` (45 file reads via `gh api`)
  - Agent B: `Yeachan-Heo/oh-my-codex` (65 file reads via `gh api`)
  - Agent C: `jason931225/claude-code/backup` (37 file reads via `gh api`, including `gh api repos/jason931225/claude-code/compare/main...backup`)
- All file paths cited inline; no in-repo Oyatie content read or referenced (per user constraint that Oyatie's design must come from agentic-workflow best practice + external benchmarks, not from existing in-repo content).
- Part A (above) is a separate prior research artifact authored from a Codex/OMX session covering the `claude-code/backup` branch in deeper detail; Part B (this section) widens to OMC and OMX and adds the cross-cutting comparison.
