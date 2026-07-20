# Agent-harness feature currency — 2026-05

Backlog item #71 "Changelog-currency: adopt latest Claude Code + Codex features."

Scope: `.claude/`, `.codex/`, `tools/hooks/`, `docs/harness/` only. Conservative —
never break existing hooks/permissions. Buck2 / CI / toolchains untouched.

This report records (1) what Claude Code + Codex CLI offer as of 2026-05, (2) what
oyatie has now, (3) the GAP, and (4) what was APPLIED vs left as RECOMMENDATION.

---

## Sources (fetched 2026-05-30)

- Claude Code hooks reference — `https://code.claude.com/docs/en/hooks`
  (the old `docs.claude.com/en/docs/claude-code/hooks` now 301-redirects here).
- Claude Code settings reference — `https://code.claude.com/docs/en/settings`.
- Codex CLI hooks reference — `https://developers.openai.com/codex/hooks`.
- Codex CLI config reference — `https://developers.openai.com/codex/config-reference`,
  `https://developers.openai.com/codex/config-advanced`.

---

## 1. Claude Code — current capabilities

### Hook input/output contract (load-bearing for this repo)

- **All command hooks receive a single JSON object on STDIN.** The docs state
  verbatim: "no environment variables contain the hook event data." There is **no**
  `TOOL_INPUT` env var.
- Universal input fields on every event: `session_id`, `transcript_path`, `cwd`,
  `hook_event_name` (plus `permission_mode`, `agent_id`, `agent_type` situationally).
- Tool events (`PreToolUse`, `PostToolUse`, `PostToolUseFailure`, `PermissionRequest`)
  carry `tool_name` and `tool_input` (an object), and `tool_response` on Post events.
  **The path/command an Edit/Write/Bash hook needs is nested under `tool_input`** —
  e.g. `{"tool_input":{"file_path":"contracts/x.yaml"}}` or
  `{"tool_input":{"command":"cargo build"}}` — NOT at the top level.
- Decision control: exit `0` (stdout parsed as JSON if present), exit `2` (blocking;
  stderr fed back to Claude), other (non-blocking, stderr shown to user). PreToolUse
  exit-2 blocks the tool; PostToolUse exit-2 is non-blocking (stderr to Claude).
- JSON-on-stdout decision fields: top-level `continue`, `stopReason`,
  `suppressOutput`, `systemMessage`, `additionalContext`, `hookSpecificOutput`; and
  for PreToolUse `hookSpecificOutput.permissionDecision` (`allow`/`deny`/`ask`/`defer`)
  with `permissionDecisionReason` and optional `updatedInput`.

### Hook event catalog (2026-05)

The current event list is much larger than the 5 oyatie wires: `SessionStart`,
`Setup`, `SessionEnd`, `UserPromptSubmit`, `UserPromptExpansion`, `Stop`,
`StopFailure`, `PreToolUse`, `PostToolUse`, `PostToolUseFailure`, `PostToolBatch`,
`PermissionRequest`, `PermissionDenied`, `SubagentStart`, `SubagentStop`,
`TaskCreated`, `TaskCompleted`, `TeammateIdle`, `InstructionsLoaded`, `ConfigChange`,
`CwdChanged`, `FileChanged`, `PreCompact`, `PostCompact`, `MessageDisplay`,
`Notification`, `Elicitation`, `ElicitationResult`, `WorktreeCreate`, `WorktreeRemove`.

### settings.json schema (security-relevant keys)

- `permissions`: `allow`, `deny`, `ask`, `defaultMode`
  (`acceptEdits`/`plan`/`auto`/`default`/`dontAsk`/`bypassPermissions`),
  `disableBypassPermissionsMode`, `additionalDirectories`,
  `allowManagedPermissionRulesOnly` (managed). Rule syntax includes
  `WebFetch(domain:example.com)`, `MCP(server)`, `Agent(name)`.
- `sandbox`: `enabled`, `failIfUnavailable`, `autoAllowBashIfSandboxed`,
  `excludedCommands`, `allowUnsandboxedCommands`,
  `filesystem.{allowRead,denyRead,allowWrite,denyWrite,allowManagedReadPathsOnly}`,
  `network.{allowedDomains,deniedDomains,allowLocalBinding,allowUnixSockets,
  allowAllUnixSockets,allowMachLookup}`.
- Hooks governance keys: `disableAllHooks`, `allowManagedHooksOnly`,
  `allowedHttpHookUrls`, `httpHookAllowedEnvVars`.

---

## 2. Codex CLI — current capabilities

- Hooks live in `~/.codex/hooks.json`, `~/.codex/config.toml`, `<repo>/.codex/hooks.json`,
  or `<repo>/.codex/config.toml` (inline `[hooks]`). Project-local hooks load only
  when the project `.codex/` layer is trusted.
- **Schema is identical in shape to Claude Code**: `{"hooks": {"<Event>": [{"matcher":
  "...", "hooks": [{"type":"command","command":"...","timeout":600}]}]}}`. `matcher`
  is optional (`"*"`/`""`/omit = match all). Only `type:"command"` runs today.
- **Event names are PascalCase**: `SessionStart`, `SubagentStart`, `PreToolUse`,
  `PermissionRequest`, `PostToolUse`, `PreCompact`, `PostCompact`, `UserPromptSubmit`,
  `SubagentStop`, `Stop`.
- Input: one JSON object on STDIN with `session_id`, `transcript_path`, `cwd`,
  `hook_event_name`, `model`, `permission_mode`, plus event-specific `tool_name` /
  `tool_input`. Same exit-code semantics as Claude Code (0 ok, 2 block).
- `notify`, provider, and telemetry keys are **ignored in project-local
  `.codex/config.toml`** and only honored at user-level `~/.codex/config.toml`; Codex
  prints a startup warning otherwise. (Relevant: never try to ship those project-scoped.)

---

## 3. What oyatie has now

- `.claude/settings.json` (commit `5cbfae71a`): OS-enforced `sandbox` (Seatbelt) with
  `filesystem.denyRead`/`denyWrite` + `network.allowedDomains`; `permissions.deny`
  mirroring credential reads + `Bash(docker *)`; `disableBypassPermissionsMode:
  "disable"`. Hooks: `SessionStart`, `UserPromptSubmit`, `Stop`, `PreToolUse`
  (Bash/Task), `PostToolUse` (Edit|MultiEdit|Write, Bash|WebFetch|WebSearch).
- `.codex/hooks.json`: same 5-event wiring; PascalCase keys; matches the current
  Codex schema exactly.
- `tools/hooks/`: local advisory shell hooks. The former retired local hook bootstrap generator was retired after this audit window because it was local-shell glue with no durable cloud-native product value; `docs/security.md` documents the remaining model.

This is already a **strong, current** baseline. The sandbox+permissions redesign and
the PascalCase Codex schema are both up to date.

---

## 4. GAP analysis

| # | Gap | Severity | Disposition |
|---|-----|----------|-------------|
| G1 | **3 PostToolUse hooks silently no-op under real Claude Code.** `spec-version-pin-suggester.sh`, `adr-orphan-detect.sh`, `vacuous-green-gate-detect.sh` extract the file path with `jq '.path // .file_path'` against the **top-level** stdin object, but Claude Code nests it at `.tool_input.file_path`. So the advisory never fires in production. The CI governance harness only passes because it drives them via a `TOOL_INPUT` env var with a flat `{"path":...}` — a path real Claude Code never uses. | High (dead guidance) | **APPLIED** |
| G2 | `pre-dispatch-guide.sh` (PreToolUse:Task) reads `.prompt` top-level; real Task input nests at `.tool_input.prompt`, so dispatch guidance never fires. | Medium | **APPLIED** |
| G3 | Misleading `$TOOL_INPUT` env-var reads across 5 hooks imply Claude Code sets that env var. It does not (docs: "no env var carries event data"). Harmless (stdin fallback covers it) but a latent footgun. | Low | **APPLIED-partial** (kept as documented fallback for the CI harness; comments now state stdin is the real source) |
| G4 | Pre-existing broken test: `scripts/tests/governance-hooks-retired-vcs-surfaces.test.sh:43` calls `tools/hooks/retired-vcs-surface-inventory.sh`, which commit `451987f24` deleted. Test exits 127. Unrelated to harness currency. | Medium (red test) | **RECOMMENDATION** (out of scope; fixing risks parent's CI work) |
| G5 | `install.sh` summary still handled a source-specific home-hook detection branch after that harness was retired (ADR-0335/0247). Dead branch. | Low | **RECOMMENDATION** |
| G6 | `_note` drift: `install.sh` CODEX_CONTENT `_note` ("PascalCase event keys…") differs from the committed `.codex/hooks.json` `_note` ("Removed 4 dead hooks…"). Cosmetic; both describe the same valid schema. | Low | **RECOMMENDATION** |
| G7 | New Claude Code events unused: `SubagentStop`/`SessionEnd`/`PreCompact`. Could host future guidance, but adding them now is speculative and risks noise. | Info | **RECOMMENDATION** (defer) |
| G8 | Optional sandbox hardening keys available: `autoAllowBashIfSandboxed` (fewer prompts once sandboxed), `permissions.defaultMode`, `WebFetch(domain:...)` allow rules, and (per `docs/security.md`'s own note) moving `denyRead`/`allowedDomains`/`disableBypassPermissionsMode` + `failIfUnavailable:true` into **managed** scope for a hard guarantee. | Info | **RECOMMENDATION** (policy/managed-scope decision; not a worktree change) |

---

## 5. APPLIED changes (this branch)

All four are minimal, commented-with-WHY, and **backward compatible** — they ADD the
nested `.tool_input.*` keys to the jq extraction while keeping the existing flat keys,
so the CI governance harness (`tools/governance/adr-0221-governance-gates.sh`, which
sets `TOOL_INPUT='{"path":...}'`) keeps passing unchanged.

1. `tools/hooks/spec-version-pin-suggester.sh` — jq filter now
   `.tool_input.file_path // .tool_input.path // .path // .file_path`.
2. `tools/hooks/adr-orphan-detect.sh` — same nested-first filter.
3. `tools/hooks/vacuous-green-gate-detect.sh` — same nested-first filter.
4. `tools/hooks/pre-dispatch-guide.sh` — filter now
   `.tool_input.prompt // .tool_input.description // .prompt // .description // .input`.

### Verification performed

- **Real Claude Code stdin shape** (`env -u TOOL_INPUT`, payload nested under
  `tool_input`): all four hooks now fire correctly (previously silent).
- **Legacy env-var shape** (`TOOL_INPUT='{"path":...}'`): all still fire — no regression.
- **Existing CI gates** green: `adr-0221-governance-gates.sh` `version-pin`,
  `orphan-citation`, `vacuous-green` all pass.
- `no-cargo-enforcer.sh` (already correct, reads `.tool_input.command`) still blocks
  `cargo build` with exit 2; `stale-tool-suggester.sh` (already includes
  `.tool_input.command`) still detects retired `oya git`/`oya vcs`.
- All config JSON re-validated with `jq empty`.

Not changed (deliberately): `no-cargo-enforcer.sh` and `injection-content-scanner.sh`
already parse `.tool_input` correctly via python and need no edit;
`stale-tool-suggester.sh` already includes `.tool_input.command`.

---

## 6. RECOMMENDATIONS (not applied — need owner decision)

- **R1 (G4):** Fix or retire `scripts/tests/governance-hooks-retired-vcs-surfaces.test.sh`
  line 43 — the referenced `retired-vcs-surface-inventory.sh` was deleted in
  `451987f24`. Either restore the inventory hook or drop that assertion. This is a
  currently-red test independent of harness currency; left for the CI owner.
- **R2 (G5):** Drop the dead source-specific detection branch and home-hook summary
  line from the retired hook bootstrap generator (external agent harness retired per ADR-0335/0247).
- **R3 (G6):** Re-sync the `install.sh` CODEX_CONTENT `_note` and the committed
  `.codex/hooks.json` `_note` so the generator and its output match verbatim.
- **R4 (G3 full):** Optionally delete the `$TOOL_INPUT` env-var branch entirely once
  the CI governance harness is migrated to pipe JSON on stdin (matching production).
  Deferred because it would require editing the CI harness (parent-owned surface).
- **R5 (G8):** Consider `sandbox.autoAllowBashIfSandboxed: true` to cut permission
  prompts once the OS sandbox is active, and per `docs/security.md`'s own scope note,
  promote `denyRead` + `allowedDomains` + `disableBypassPermissionsMode` +
  `failIfUnavailable: true` into **managed** settings for a non-loosenable guarantee.
  These are policy decisions, not worktree edits.
- **R6 (G7):** If future guidance is wanted, `SubagentStop` and `SessionEnd` are the
  natural homes (e.g. end-of-subagent evidence reminder). Defer until there's a concrete
  need — adding empty hooks now only adds noise.
