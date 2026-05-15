---
status: pending approval
mode: deliberate
iteration: 5
delta_to: ralplan-foundry-supervisor-simple-v4-2026-05-14.md
owner_phase: M02-P06
prerequisite_adrs:
- ADR-foundry-settings-template-canonical-rendering (new)
new_crates_count: 2
new_template_payloads_count: 3
new_grit_units_estimate: 18
new_acceptance_rows: C.25 .. C.30
purpose: Auto-backfilled purpose for ralplan-foundry-supervisor-simple-v5-delta-settings-template-2026-05-15.md
---
# RALPLAN — Foundry Supervisor (Simple) — Iteration 5 (Delta Addendum)

**Scope:** v5 is an **addendum** to v4 (`ralplan-foundry-supervisor-simple-v4-2026-05-14.md`, 702 lines, APPROVED by Architect + Critic). v4 stands unchanged: 4 crates, 49 base + 14 sub-unit grit claims, 24 acceptance rows, owner phase M02-P06, Branch Y, hyper-only, atomic `dead_letter`, value-only `SessionTicket`. **v5 adds — does not modify.** v5 introduces 2 new crates, 3 in-repo template payloads, 1 CI lane, 1 ADR, 3 new Wave tasks (2g, 2h, 4d), and 6 new acceptance rows (C.25..C.30).

---

## §A.0 Why this delta

User intervention during execution onboarding (verbatim):
> "how are we patching the settings with the appropriate hooks / settings etc for each agent across accounts, so they are consistent and share the same skills, hooks, tools etc."

v4 covers session execution per-account (spawn / inject / drain / kill / idempotency). v4 does NOT cover **settings consistency across N accounts × 3 providers**. Without a canonical settings-template lane, each `ProviderAccount` row in `registry/accounts/*.toml` drifts independently: hook installation, skill availability, MCP server presence, allowed-tools, and permission rules each diverge silently. The SessionDriver call chain (v4 §B.4 step 14) assumes the per-provider stop-hook exists; if account-A has it and account-B doesn't, `drain_response` races. v5 closes the gap by materializing all per-account provider settings from a single canonical workspace template, verified at every `tick_once` snapshot.

---

## §A.1 RALPLAN-DR summary

### A.1.1 Principles (3, additive to v4 §A.1)

1. **settings-as-data** — Per-account, per-provider settings are a data artifact (in-repo template + per-account overlay), not procedural setup. Diffable, reviewable, replayable.
2. **renderer-not-kernel** — Settings rendering is an **adapter** concern (filesystem I/O, per-provider format dialects). The settings-template kernel holds value types only. Same layering rule as v4 supervisor-kernel ↔ jsonl-supervisor-adapter.
3. **verify-before-spawn** — Drift detection runs at snapshot time; an account whose rendered settings have drifted is not eligible for `RoutePolicy::select` unless `auto_reconcile=true`. Surfaces as a side-channel on the existing v4 `AccountSnapshotProvider` port; **trait signature unchanged**.

### A.1.2 Decision Drivers (top 3)

1. **multi-account-consistency** — N accounts × 3 providers = 3N distinct settings files. Manual maintenance guarantees drift within one onboarding cycle. CI-enforced parity is the only stable answer.
2. **cross-provider-parity** — Hook events, skill semantics, MCP server registration, and permission rules differ per CLI. A canonical `SettingsTemplate` value type with per-provider renderers normalizes the surface a workspace declares once.
3. **drift-free-onboarding** — Adding an account becomes a single-row append to `registry/accounts/<provider>.toml`; next `tick_once` triggers render + verify automatically. No human runbook.

> **Note (PRE-5):** Drift-detection lane (`lean-settings-drift`) is **feature-flagged off in production** until `sum(rows across registry/accounts/*.toml) ≥ 10`. Before threshold: lane runs against fixture files only in CI (passes vacuously for empty fleet). Threshold is checked at lane start via `wc -l registry/accounts/*.toml | tail -1 | awk '{print $1}'`.

### A.1.3 Viable Options

| Opt | Shape | Pros | Cons | Verdict |
|-----|-------|------|------|---------|
| **A** | Per-provider renderer adapters (`ClaudeRenderer`, `CodexRenderer`, `GeminiRenderer`) implementing one `SettingsRenderer` trait over a shared `SettingsTemplate` value type | Each renderer owns its format dialect; new provider = new adapter, zero kernel surface change; matches v4 driver-not-kernel layering | 3 small renderer impls instead of 1 | **CHOSEN** |
| **B** | Single multi-format renderer with format-flag dispatch (`render(template, &Format::Claude / Codex / Gemini)`) | One impl to maintain | Format dialect leaks into renderer (large match statement grows); new provider = touching the central renderer (lean-a10 risk on the rendering crate); per-provider edge cases (Claude hooks live in separate JSON files; Codex hooks live in `~/.codex/hooks.json`) force discriminated unions inside one function | **REJECTED** — same anti-pattern v4 rejected for `SessionDriver` (one impl per CLI, not one impl with discriminated dispatch) |

### A.1.4 Pre-mortem (2 scenarios)

| Scenario | Mitigation lane | Acceptance |
|----------|----------------|------------|
| **(a) Config-path-divergence** — Codex or Gemini CLI changes the canonical config path between releases (e.g. moves `~/.codex/config.toml` → `~/.config/codex/config.toml`); renderer writes to the old path, CLI reads from the new one | `lean-settings-drift` CI lane reads `templates/foundry-supervisor/<provider>.toml` `target_path` field and compares against a smoke-spawn of each CLI's `--show-config` (or equivalent). Mismatch fails the lane. | C.26 + open-question "verify-codex-and-gemini-config-paths-at-CI-time" |
| **(b) Hook-event-mismatch** — A `HookRef { event: HookEvent::Stop, ... }` declared in the canonical template maps to a provider event name the provider doesn't recognize (e.g. Codex `Stop` is `SubagentStop` in its terminology); supervisor's `drain_response` polls a hook that never fires | Per-provider `HookEventMap` in each renderer translates kernel `HookEvent` enum → provider's actual event name. Build-time table tested via `tests/hook_event_mapping.rs` against fixtures captured from each CLI's `--list-hook-events` output (or, when unsupported, against the capability-seed file pattern v4 already uses for `stop_hook_supported`). | C.27 + open-question "capture-codex-gemini-hook-event-vocabulary" |

---

## §B.0 Existing surfaces composed (file:line — verified)

| Surface | File | Lines | Notes |
|---------|------|-------|-------|
| `ProviderFamily` enum (4 variants used by templates: Claude, OpenAiOrCodex, Gemini; Aws+Oci unused here) | `crates/oya-foundry-account-kernel/src/lib.rs` | 23-29 | **Re-used as value; zero new public API on existing kernel.** Reachable via the existing `try_from(&str)` round-trip at L40-50. |
| `SecretReference` (`sref://...` scheme) + `SecretReferenceError` | `crates/oya-foundry-account-kernel/src/lib.rs` | 57-79 | Templates reference secrets only by `sref://`; renderer never serializes raw material to disk. |
| `SecretStorePort` impl (in-memory ref) on `OpenBaoAdapter` | `crates/oya-foundry-account-adapter-openbao/src/lib.rs` | 26 (impl) + 88-164 (10 unit tests) | Spawn-time env-var injection resolves `sref://...` through this port at supervisor-app run-time; renderer never calls it. |
| `AccountSnapshotProvider` port (v4 §B.2.3) | `crates/oya-foundry-supervisor-kernel/src/lib.rs::AccountSnapshotProvider` (new in v4 grit unit #24) | n/a — v4 declares it | v5 extends the **semantics** of `snapshot()` via composition (verify-via-side-channel); **trait signature unchanged**. |
| `templates/` directory | `templates/` (verified `ls templates/`) | — | Existing root with `checklists/INDEX.md` and 12 `*-template.{md,yaml,json}` files. v5 adds new subdir `templates/foundry-supervisor/` (declared as scaffold artifact). |
| `registry/accounts/` directory | path declared in v4 §B.10 row "registry/accounts/ directory (v4 patch #7)" | — | Scaffold artifact, not yet created. v5 templates point at this directory for the `target_secret_refs` field. |
| **Codex CLI config layout** (verified ground truth, `ls /Users/jasonlee/.codex/`) | `~/.codex/config.toml` (3.2K real TOML) + `~/.codex/hooks.json` (9.2K, hooks ARE supported) + `~/.codex/AGENTS.md` | — | **Verified real, not assumed.** Hooks are out-of-band in `hooks.json`, NOT inline in `config.toml`. CodexRenderer materializes BOTH files. |
| **Gemini CLI config layout** (verified ground truth, `ls /Users/jasonlee/.gemini/`) | `~/.gemini/settings.json` (1.0K real JSON) + `~/.gemini/GEMINI.md` | — | **Verified real, not assumed.** No separate hooks file observed; hooks (if supported) likely inline in `settings.json` or unsupported. See §B.6 RISK row. |
| **Claude CLI config layout** (existing project precedent, this very repo) | `.claude/settings.json` (project-local, observed via plugin docs); `~/.claude/settings.json` (user-global) | — | Hooks are inline in `settings.json` under a `hooks` key (PreToolUse, PostToolUse, Stop, SubagentStart, SubagentStop, SessionStart, SessionEnd, UserPromptSubmit, PreCompact, Notification). Skills are inline under a `skills` key with `plugin:skill` notation. |

---

## §B.1 New crates (2 — additive to v4 §B.1)

| Crate | v4-BNF + 12-layer-enum justification |
|-------|--------------------------------------|
| `oya-foundry-settings-template-kernel` | `oya-<foundry>-<settings-template>-<kernel>` — foundry is the registered µservice (Cargo.toml:290); settings-template is the new feature compound (mirrors v4 `jsonl-supervisor`); kernel = layer #1 (pure value types). Allowed deps: std + tokio (already baseline). No I/O. No per-provider serialization. |
| `oya-foundry-settings-template-adapter-fs` | `oya-<foundry>-<settings-template>-<adapter-fs>` — settings-template same compound; adapter-fs = layer #5; the ONLY crate that writes to `~/.claude/`, `~/.codex/`, `~/.gemini/` filesystem locations. Allowed deps: std + tokio + workspace `bytes` (already baseline at Cargo.toml:481-492). Hand-rolled TOML + JSON parsers (no `serde`, no `toml` crate — Branch Y; reuses the precedent set by v4 `oya-foundry-jsonl-supervisor-adapter` hand-rolled framing and `capability-registry-app:L96` hand-rolled JSON). |

---

## §B.2 Public contracts (kernel surface)

All types value-only. No `&`/`Arc`/`Box<dyn>` in struct fields. Same invariant fence as v4 `SessionTicket`.

#### B.2.1 `oya-foundry-settings-template-kernel` types

```rust
use oya_foundry_account_kernel::{ProviderFamily, SecretReference};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Versioned template. Top-level value the renderer + verifier consume.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettingsTemplate {
    pub version: u32,                              // monotonically incremented on schema change
    pub hooks: Vec<HookRef>,
    pub skills: Vec<SkillRef>,                     // Claude-only; ignored by Codex/Gemini renderers
    pub mcp_servers: Vec<McpServerRef>,
    pub permissions: Vec<PermissionEntry>,         // e.g. "Bash(grit *)"
    pub allowed_tools: Vec<AllowedTool>,           // e.g. "Read", "Bash"
    pub provider_overrides: BTreeMap<ProviderFamilyKey, ProviderOverrides>,
}

/// BTreeMap key wrapper (ProviderFamily is Copy but not Ord by derive; explicit key type).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProviderFamilyKey(pub ProviderFamily);

/// Hook events: superset; each renderer maps to its provider's event vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HookEvent {
    SessionStart, SessionEnd,
    Stop, SubagentStart, SubagentStop,
    PreToolUse, PostToolUse,
    UserPromptSubmit, PreCompact, Notification,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HookRef {
    pub event: HookEvent,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillRef {
    pub plugin_id: String,         // e.g. "oh-my-claudecode"
    pub skill_name: String,        // e.g. "planner"
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum McpTransport { Stdio, Http, Sse, WebSocket }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpServerRef {
    pub name: String,
    pub transport: McpTransport,
    pub command_or_url: String,                    // path for Stdio; URL for Http/Sse/WebSocket
    pub env_secret_refs: Vec<(String, SecretReference)>,  // env-var name + sref; resolved at spawn
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PermissionEntry { pub rule: String }    // verbatim e.g. "Bash(grit *)"

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowedTool { pub name: String }        // verbatim e.g. "Read"

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ProviderOverrides {
    pub disable_skills: bool,                      // true for Codex + Gemini by default
    pub extra_hooks: Vec<HookRef>,                 // provider-specific additions
    pub omitted_hook_events: Vec<HookEvent>,       // provider does not support these
}

/// Render manifest: what was written, with content hash for drift detection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderManifest {
    pub provider_family: ProviderFamily,
    pub files: Vec<RenderedFile>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderedFile {
    pub target_path: PathBuf,
    pub content_blake3: [u8; 32],
    pub byte_len: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DriftState { Match, Modified, Missing, Extra }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriftReport {
    pub provider_family: ProviderFamily,
    pub entries: Vec<DriftEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriftEntry {
    pub target_path: PathBuf,
    pub state: DriftState,
    pub diff: Option<String>,                      // populated only for Modified
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SettingsRendererError {
    Io(String),
    UnsupportedFormat(String),
    HookEventNotMapped(HookEvent),
    SecretRefUnresolved(String),
    InvalidTemplate(String),
}
```

#### B.2.2 `oya-foundry-settings-template-kernel` port (sync trait, Branch Y)

```rust
use std::path::Path;
use oya_foundry_account_domain::ProviderAccount;

/// Per-provider renderer port. Each provider gets one impl in the adapter crate.
pub trait SettingsRenderer: Send + Sync {
    /// Identity. Used by CI lane to enumerate which providers a workspace touches.
    fn provider_family(&self) -> ProviderFamily;

    /// Render the canonical template to disk under `root` (typically `~`).
    /// Atomic: tempfile + rename(2). Idempotent: re-rendering with the same
    /// (template, account) pair is a no-op (manifest hashes match existing files).
    fn render(
        &self,
        template: &SettingsTemplate,
        account: &ProviderAccount,
        root: &Path,
    ) -> Result<RenderManifest, SettingsRendererError>;

    /// Compare the on-disk state under `root` against what `render` would
    /// produce, without writing anything. Returns per-file drift entries.
    fn verify(
        &self,
        template: &SettingsTemplate,
        account: &ProviderAccount,
        root: &Path,
    ) -> Result<DriftReport, SettingsRendererError>;
}
```

**Invariant fence:** `fn _f<T: Send + Sync + 'static>() {} _f::<SettingsTemplate>(); _f::<RenderManifest>(); _f::<DriftReport>();` (no `static_assertions` dep — workspace lacks it; same pattern as v4 `SessionTicket`).

---

## §B.3 Template payload shape (3 in-repo files)

Hand-rolled TOML (parser hand-rolled in the adapter; same precedent as v4 `registry/capabilities/foundry-supervisor.toml`). Example for Claude:

```toml
# templates/foundry-supervisor/claude.toml
version = 1
provider_family = "Claude"                # CamelCase round-trips to ProviderFamily::Claude (account-kernel:48)
target_root = "~"                         # renderer expands; CodexRenderer/GeminiRenderer override

[[hooks]]
event = "Stop"
command = "$WORKSPACE_ROOT/tools/foundry-supervisor-stop-hook"
args = []

[[hooks]]
event = "SessionStart"
command = "$WORKSPACE_ROOT/tools/foundry-supervisor-session-start"
args = []

[[skills]]
plugin_id = "oh-my-claudecode"
skill_name = "planner"

[[mcp_servers]]
name = "oya-mcp-gateway"
transport = "Stdio"
command_or_url = "$WORKSPACE_ROOT/target/release/oya-mcp-gateway"

[[mcp_servers.env_secret_refs]]
env_var = "OYA_MCP_TOKEN"
sref = "sref://openbao/foundry/mcp-gateway/token"

[[permissions]]
rule = "Bash(grit *)"

[[permissions]]
rule = "Bash(rtk *)"

[[allowed_tools]]
name = "Read"

[[allowed_tools]]
name = "Write"

[[allowed_tools]]
name = "Bash"
```

`codex.toml` and `gemini.toml` have **identical schema** but their `provider_family` differs and `provider_overrides` are populated:

`codex.toml`: `provider_family = "OpenAIOrCodex"`; `[provider_overrides] disable_skills = true; omitted_hook_events = []` (Codex DOES support hooks — verified `~/.codex/hooks.json` 9.2K).

`gemini.toml`: `provider_family = "Gemini"`; `[provider_overrides] disable_skills = true; omitted_hook_events = ["SubagentStart","SubagentStop","PreCompact","PreToolUse","PostToolUse"]` (RISK §B.6 R2: GeminiRenderer hooks no-op until capability confirmed; capability-seed row records `hooks_supported = false`).

**Provider-specific render targets** (renderer-side, not in the TOML):

| Provider | Render target paths (under `root`) |
|----------|-------------------------------------|
| Claude | `~/.claude/settings.json` (merged with existing top-level keys; the renderer writes ONLY the keys it owns: `hooks`, `skills`, `mcp_servers` → `mcpServers`, `permissions`, `allowedTools`). |
| Codex | `~/.codex/config.toml` (merged) + `~/.codex/hooks.json` (replaced wholly under a documented `[foundry-supervisor]` namespace; verified file exists at 9.2K). |
| Gemini | `~/.gemini/settings.json` (merged; hooks omitted per `provider_overrides.omitted_hook_events` until capability confirmed). |

---

## §B.4 Integration with v4 (zero existing-kernel surface change)

`SupervisorApp::new` signature **does NOT change.** The new crate slots in via composition only.

```
v5 production wiring (in supervisor-app `fn new`):

  let renderers: Vec<Box<dyn SettingsRenderer>> = vec![
      Box::new(ClaudeRenderer::new()),
      Box::new(CodexRenderer::new()),
      Box::new(GeminiRenderer::new()),
  ];
  let template_root = workspace_root.join("templates/foundry-supervisor");
  let snapshot_provider = CompositeAccountSnapshotProvider::with_verify(
      file_account_provider,         // reads registry/accounts/*.toml (v4 §B.2.5 wiring)
      renderers,
      template_root,
      home_dir,
      VerifyMode::FailOnDrift,       // alt: VerifyMode::AutoReconcile
  );
  // SupervisorApp::new(drivers, inbox, snapshot_provider, outbox, config)   ← v4 signature unchanged
```

`AccountSnapshotProvider::snapshot()` semantics extension (no signature change):

> An implementation MAY invoke `SettingsRenderer::verify` for each `ProviderAccount` it would return. If `VerifyMode::FailOnDrift` and any account's `DriftReport.entries` contains a non-`Match` state, the account is **excluded** from the returned `Vec<ProviderAccount>`. If `VerifyMode::AutoReconcile`, the implementation calls `SettingsRenderer::render` for the drifting account, then includes it in the snapshot. Drift outcomes are written to `.omc/state/settings-drift-report.json` for the CI lane to consume. The trait signature `fn snapshot(&self) -> Vec<ProviderAccount>` is unchanged.

`tick_once` call chain (v4 §B.4) is **byte-identical**. Step 2 (`accounts = self.accounts.snapshot()`) silently picks up the verify+render behavior through the composite snapshot provider — no rewiring of steps 3..17.

---

## §B.5 CI lane `lean-settings-drift`

**Verification command** (added to v4 §B.11 as row 22):

```
cargo run -p oya-dev-cli -- gate validate settings-drift \
    --templates-root templates/foundry-supervisor \
    --accounts-root registry/accounts \
    --report-out .omc/state/settings-drift-report.json
```

**Subcommand wiring:** `oya-dev-cli` (`crates/oya-dev-cli/src/main.rs`, verified real binary) gains a `settings-drift` subcommand. The subcommand:

1. Loads each `templates/foundry-supervisor/<provider>.toml`.
2. Enumerates `registry/accounts/<provider>.toml` rows.
3. For each (template × account) pair, invokes the matching `SettingsRenderer::verify` against a CI-isolated `root` (NOT the runner's `$HOME` — uses `$CARGO_TARGET_TMPDIR/settings-drift/<account_id>`; a pre-step calls `render` once to seed the isolated root).
4. Writes consolidated `DriftReport` set to the report path.
5. Exit 0 ⟺ every entry is `DriftState::Match`. Exit 1 with non-empty report otherwise.

**Pass/fail criteria** (acceptance row C.26): exit code 0 against the 3 example account files committed under `registry/accounts/<provider>.example.toml`.

**Why this is not a new HTTP stack:** purely file-based (loads TOML, calls `verify`, writes JSON report). Zero new external deps; hand-rolled JSON emission for the report follows the v4 supervisor-conformance `build.rs` precedent.

---

## §B.6 Provider-specific RISKs

| # | RISK | Verification command (real ground truth verified) | Mitigation |
|---|------|---------------------------------------------------|------------|
| **R1** | **Codex hook-event vocabulary** is captured in `~/.codex/hooks.json` (9.2K, verified present), but the documented event-name set for that file is not yet checked into the workspace as a fixture. | `ls -la /Users/jasonlee/.codex/hooks.json` → exists (9.2K, verified); capture as fixture via `cp ~/.codex/hooks.json crates/oya-foundry-settings-template-adapter-fs/tests/fixtures/codex-hooks.observed.json` at Wave-4d scaffold time; `tests/hook_event_mapping.rs::codex_event_names_round_trip` asserts `HookEvent` → Codex event name → `HookEvent` for every variant the fixture declares. | Not a no-op — Codex DOES support hooks. Renderer materializes both `config.toml` and `hooks.json`. Capability-seed row records observed event vocabulary. |
| **R2** | **Gemini hook surface** is undocumented in observable ground truth: `~/.gemini/settings.json` is 1.0K with no `hooks` key visible to a casual inspection. CLI may support hooks via a different config file, may not support hooks at all, or may evolve the schema. | `ls -la /Users/jasonlee/.gemini/` → only `settings.json`, `GEMINI.md`, `state.json`, `oauth_creds.json`, no separate hooks file; `gemini --help` and `gemini config --help` outputs to be captured at Wave-4d scaffold time into `crates/oya-foundry-settings-template-adapter-fs/tests/fixtures/gemini-cli-help.observed.txt`. | **Renderer reduces hooks to no-op for Gemini until capability confirmed.** Capability-seed file `registry/capabilities/foundry-supervisor.toml` (v4 §B.5) gains a per-driver `hooks_supported = false` row for `gemini-driver` — same pattern v4 uses for `request_id_supported = false`. No invented capability. |
| **R3** | **Path divergence between releases** — CLI may move config root (`~/.codex` → `~/.config/codex`). | At CI-lane init, the subcommand reads each renderer's `canonical_paths_for_provider(home: &Path) -> Vec<PathBuf>` and asserts at least one matches an observed CLI environment hint (e.g. `CODEX_HOME`, `GEMINI_HOME`) when those env vars exist; otherwise falls back to the documented default and emits a CI warning. | Logged as open question; no auto-mitigation in v5 — flagged for capability check in subsequent phase. |

**No invented capability rule:** if at scaffold time a fixture cannot be captured for a provider, the corresponding renderer ships with the relevant `HookEvent` variants in `omitted_hook_events` and the capability-seed file records the limitation. v5 ships honest capability surface, not aspirational.

---

## §B.7 grit claim units (estimate: 18 net-new — anchored to M02-P06)

Anchor: `M02-P06-foundry-supervisor` (existing v4 phase). Sub-claim intent: `M02-P06-settings-template` (nested under the v4 phase claim).

| Order | Unit (file::Identifier) | Notes |
|-------|------------------------|-------|
| v5.1 | `docs/decisions/ADR-NNNN-foundry-settings-template-canonical-rendering.md::header` | ADR (§B.8) |
| v5.2 | `Cargo.toml::members` | Add 2 new crates to workspace |
| v5.3 | `crates/oya-foundry-settings-template-kernel/src/lib.rs::SettingsTemplate` | Value type |
| v5.4 | `crates/oya-foundry-settings-template-kernel/src/lib.rs::HookRef` + `HookEvent` | Enum + struct |
| v5.5 | `crates/oya-foundry-settings-template-kernel/src/lib.rs::SkillRef` | |
| v5.6 | `crates/oya-foundry-settings-template-kernel/src/lib.rs::McpServerRef` + `McpTransport` | |
| v5.7 | `crates/oya-foundry-settings-template-kernel/src/lib.rs::PermissionEntry` + `AllowedTool` | |
| v5.8 | `crates/oya-foundry-settings-template-kernel/src/lib.rs::ProviderOverrides` + `ProviderFamilyKey` | |
| v5.9 | `crates/oya-foundry-settings-template-kernel/src/lib.rs::RenderManifest` + `RenderedFile` + `DriftReport` + `DriftEntry` + `DriftState` + `SettingsRendererError` | |
| v5.10 | `crates/oya-foundry-settings-template-kernel/src/lib.rs::SettingsRenderer` | Port trait |
| v5.11 | `crates/oya-foundry-settings-template-adapter-fs/src/lib.rs::ClaudeRenderer` | Per-provider impl |
| v5.12 | `crates/oya-foundry-settings-template-adapter-fs/src/lib.rs::CodexRenderer` | Per-provider impl (writes BOTH config.toml + hooks.json) |
| v5.13 | `crates/oya-foundry-settings-template-adapter-fs/src/lib.rs::GeminiRenderer` | Per-provider impl (hooks no-op per R2) |
| v5.14 | `crates/oya-foundry-settings-template-adapter-fs/src/lib.rs::parse_template_toml` + `emit_drift_report_json` | Hand-rolled TOML parser + JSON emitter (no serde) |
| v5.15 | `templates/foundry-supervisor/claude.toml::header` | Template payload |
| v5.16 | `templates/foundry-supervisor/codex.toml::header` | Template payload |
| v5.17 | `templates/foundry-supervisor/gemini.toml::header` | Template payload |
| v5.18 | `crates/oya-dev-cli/src/commands/settings_drift.rs::run` | CI-lane subcommand wiring |

**Doc-coverage rows (lean-a5, additive):** 5 docs × 2 crates = 10 new doc files under `docs/foundry/settings-template/{kernel,adapter-fs}/{README,architecture,operations,security,sample-payloads}.md`. Counted as ~10 sub-grit-units folded into the per-crate scaffold step (matches v4 §B.10 doc-coverage row treatment).

---

## §B.8 Doc + ADR footprint (additive)

| Artifact | Path |
|----------|------|
| ADR | `docs/decisions/ADR-NNNN-foundry-settings-template-canonical-rendering.md` |
| Doc-coverage (kernel) | `docs/foundry/settings-template/kernel/{README,architecture,operations,security,sample-payloads}.md` |
| Doc-coverage (adapter-fs) | `docs/foundry/settings-template/adapter-fs/{README,architecture,operations,security,sample-payloads}.md` |
| Templates | `templates/foundry-supervisor/{claude,codex,gemini}.toml` |
| CI report (runtime artifact, not committed) | `.omc/state/settings-drift-report.json` |
| Test fixtures (captured at scaffold) | `crates/oya-foundry-settings-template-adapter-fs/tests/fixtures/{codex-hooks.observed.json, gemini-cli-help.observed.txt}` |

**ADR `ADR-foundry-settings-template-canonical-rendering`:**
- **Decision:** Canonical `SettingsTemplate` value type in `oya-foundry-settings-template-kernel` + per-provider `SettingsRenderer` impls in `oya-foundry-settings-template-adapter-fs`; drift verified at `AccountSnapshotProvider::snapshot()` time.
- **Drivers:** multi-account-consistency, cross-provider-parity, drift-free-onboarding (mirrors §A.1.2).
- **Alternatives considered:**
  - (a) Script-based per-provider settings (rejected: drift impossible to verify after the fact; no canonical source-of-truth).
  - (b) Provider-native settings inheritance (rejected: Claude/Codex/Gemini config files do not natively support cross-account inheritance — verified by inspection of `~/.claude/settings.json`, `~/.codex/config.toml`, `~/.gemini/settings.json`).
  - (c) MCP-based dynamic settings injection (rejected: requires server-side settings push; not all CLIs expose live settings updates over MCP; would also leak the boot-time dependency on a running MCP server before per-account settings exist).
- **Why chosen:** matches v4 "supervisor-driver-not-kernel" — settings rendering is a kernel concern (value types) + adapter concern (filesystem I/O); composes into existing `AccountSnapshotProvider` without any existing-kernel surface change.
- **Consequences:** +2 crates (kernel + adapter-fs); +3 template payloads; +1 CI lane (`lean-settings-drift`); per-provider RISK rows in capability-seed file if Codex/Gemini surfaces differ from current spec; renderer becomes the canonical authority for what an account's settings look like (workspace gives up the ability to hand-edit `~/.claude/settings.json` without provoking drift).
- **Follow-ups:** subsequent phase captures verified Codex hook-event vocabulary fixture; subsequent phase confirms Gemini hook support state; capability-seed file gains `hooks_supported` per-driver row.

---

## §B.9 Wave additions (3 new team tasks)

Slotted into the existing 14-task team graph. Wave 1 + 3 (a-c) + 5 unchanged. Additions:

| Wave | Task id | Description | Depends on |
|------|---------|-------------|-----------|
| **2g** | `settings-template-kernel` | Implement `oya-foundry-settings-template-kernel` (grit units v5.3..v5.10): value types + `SettingsRenderer` trait. Doc-coverage 5 docs. Unit tests cover invariant fence + `SettingsTemplate` equality. | Wave 1 |
| **2h** | `settings-template-adapter-fs` | Implement `oya-foundry-settings-template-adapter-fs` (grit units v5.11..v5.14): 3 renderer impls + hand-rolled TOML parser + JSON emitter. Atomic tempfile+rename; idempotent re-render; secret-ref resolution at-spawn (NOT at-render). Doc-coverage 5 docs. Unit tests cover round-trip TOML, drift detection (Modified/Missing/Extra), no-raw-secrets grep. | Wave 2g |
| **4d** | `settings-template-payloads-and-lane` | Materialize templates (`templates/foundry-supervisor/{claude,codex,gemini}.toml`, grit units v5.15..v5.17); ADR (v5.1); `oya-dev-cli` `settings-drift` subcommand (v5.18); CI lane registration in `lean-settings-drift`. Capture fixtures from local `~/.codex/hooks.json` + `gemini --help` per §B.6 R1, R2. Append `hooks_supported` rows to `registry/capabilities/foundry-supervisor.toml`. | Wave 2h + Wave 4b (registry/accounts/) |

No changes to Waves 1, 2a-f, 3a-c, 4a-c, 5. Wave 4d composes with 4b (registry/accounts/ scaffold) — 4d depends on 4b but is otherwise independent.

---

## §C.25 .. C.30 — New acceptance rows (additive to v4 §C)

| # | Acceptance criterion | Verification |
|---|---------------------|--------------|
| **C.25** | Both new crates build under workspace | `rtk cargo build -p oya-foundry-settings-template-kernel && rtk cargo build -p oya-foundry-settings-template-adapter-fs` exits 0 |
| **C.26** | `lean-settings-drift` CI lane green against example account files | `cargo run -p oya-dev-cli -- gate validate settings-drift --templates-root templates/foundry-supervisor --accounts-root registry/accounts --report-out .omc/state/settings-drift-report.json` exits 0 when run against `registry/accounts/{claude,codex,gemini}.example.toml` |
| **C.27** | Drift detection works — hand-edit a rendered file, verify reports `DriftState::Modified` with non-empty `diff` | Test: render once; mutate `~/.claude/settings.json` (under CI-isolated root) `permissions[0].rule`; re-run `verify`; assert `entries.iter().any(\|e\| e.state == DriftState::Modified && e.diff.is_some())` |
| **C.28** | Re-render is idempotent — second `render` invocation against an already-rendered root is a no-op (manifest hashes match all existing files; no writes occur) | Test: render once, capture `mtime` for every file in manifest; render second time; assert all `mtime`s unchanged |
| **C.29** | No raw secrets in any rendered file — scan rendered output for SecretReference resolution artifacts | Test: render against an account whose `secret_ref` resolves to a known-high-entropy fixture token (e.g. 64-char hex string only present in OpenBao); `grep -RFq '<the-known-token>' $RENDER_ROOT` MUST exit 1 (no match). Also assert every rendered `mcpServers.*.env` entry references an env-var name, never a literal value |
| **C.30** | Codex + Gemini RISK status materialized — capability-seed file records observed limitations | `registry/capabilities/foundry-supervisor.toml` contains `hooks_supported = true` for `claude-driver`, `hooks_supported = true` for `codex-driver` (or `= false` with capability-seed reason if R1 fixture cannot be captured), and `hooks_supported = false` for `gemini-driver` until R2 is resolved. `grep -c 'hooks_supported' registry/capabilities/foundry-supervisor.toml >= 3`. Open question `verify-codex-and-gemini-config-paths-at-CI-time` written to `.omc/plans/open-questions.md` |

---

## §E — Iteration-5 change log (delta-only)

| Section | What this addendum adds |
|---------|-------------------------|
| §A.0 | User-intent gap statement (verbatim quote) |
| §A.1 | 3 principles + 3 drivers + 2 options (A chosen, B rejected) + 2 pre-mortems |
| §B.0 | Existing surfaces composed (ProviderFamily, SecretReference, SecretStorePort, AccountSnapshotProvider, dirs, verified Codex/Gemini/Claude layouts) |
| §B.1 | 2 crates (kernel + adapter-fs) with naming justification |
| §B.2 | 12 value types + 1 trait + invariant fence |
| §B.3 | TOML schema for 3 payloads + per-provider render target table |
| §B.4 | Composition wiring; v4 trait signatures unchanged |
| §B.5 | CI lane `lean-settings-drift` + dev-cli subcommand |
| §B.6 | R1 Codex hooks, R2 Gemini surface, R3 path divergence |
| §B.7 | 18 grit units anchored to M02-P06-settings-template sub-claim |
| §B.8 | ADR + 10 doc-coverage files + 2 fixtures |
| §B.9 | 3 Wave tasks (2g, 2h, 4d) — additive to v4's 14 |
| §C.25..C.30 | 6 acceptance rows |

**v4 sections untouched:** all of v4 §A..§F. v5 composes additively.

---

## §F — v5 Plan summary (executor cheat sheet)

- **Mode:** deliberate; iteration 5 of 5; **delta-only addendum** to v4.
- **Dep branch:** Y (zero net-new external deps — Cargo.toml:481-492 baseline).
- **New crates:** 2 (settings-template-kernel + settings-template-adapter-fs).
- **New template payloads:** 3 (`templates/foundry-supervisor/{claude,codex,gemini}.toml`).
- **New ADR:** 1 (`ADR-foundry-settings-template-canonical-rendering`).
- **New CI lane:** 1 (`lean-settings-drift`, wired through `oya-dev-cli`).
- **New public APIs on existing kernels:** 0 (composition only; `AccountSnapshotProvider` semantics extends without signature change).
- **Grit claim units:** 18 net-new + ~10 doc-coverage rows.
- **Wave additions:** 3 (Wave 2g, 2h, 4d) on top of v4's 14 tasks.
- **Acceptance rows:** C.25..C.30 (6 net-new).
- **Critical RISKs:** R1 Codex hook-event vocabulary (fixture capturable from `~/.codex/hooks.json`, verified 9.2K real); R2 Gemini hook-surface unverified (degrades to no-op + capability-seed row); R3 config-path divergence (CI smoke; open question for follow-up phase).
- **No invented capability:** Codex hooks confirmed real (verified `~/.codex/hooks.json`); Gemini hooks honestly degraded until verified.
- **Open questions:** appended to `/Users/jasonlee/oyatie/.omc/plans/open-questions.md` under section `ralplan-foundry-supervisor-simple-v5-delta — 2026-05-15`.

**Does this delta capture your intent?**
- "proceed" — hand off to Architect + Critic for iteration-5 consensus review (deliberate mode)
- "adjust [X]" — return to interview
- "restart" — discard this delta (v4 remains approved and intact)
