---
doc_class: Standard
purpose: "Value types and port traits for settings template rendering (v5 delta)"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Settings-Template Kernel — README

**Crate:** `intelligence-settings-template-kernel`  
**Layer:** Kernel (12-layer-enum L1)  
**Wave:** 2g (M02-P06, v5 delta)  
**Dependencies:** std-only (Branch Y)

## Overview

The settings-template kernel holds pure value types and port traits for canonical workspace settings rendering. It addresses the user pain point: "how are we patching the settings with the appropriate hooks / settings etc for each agent across accounts, so they are consistent and share the same skills, hooks, tools etc" (v5 §A.0).

### Purpose

Enable multi-account, multi-provider settings consistency via:
- **`SettingsTemplate`** — canonical workspace template (one per provider)
- **`SettingsRenderer` port trait** — implemented by per-provider adapters
- **`HookEvent` enum** — kernel events mapped to per-provider event names
- **`RendererMode`** — Disabled / VerifyOnly / Reconcile per v6 BLOCKER-1
- **`DriftReport`** — detected drift state (Missing / Modified / Extra / Match)

## Public API Summary

### Value Types

```rust
pub struct SettingsTemplate {
    pub version: u32,
    pub hooks: Vec<HookRef>,
    pub skills: Vec<SkillRef>,
    pub mcp_servers: Vec<McpServerRef>,
    pub permissions: Vec<PermissionEntry>,
    pub allowed_tools: Vec<AllowedTool>,
    pub provider_overrides: BTreeMap<ProviderFamilyKey, ProviderOverrides>,
}

pub struct HookRef {
    pub event: HookEvent,
    pub command: String,
    pub args: Vec<String>,
}

pub enum HookEvent {
    SessionStart, SessionEnd, Stop, SubagentStart, SubagentStop,
    PreToolUse, PostToolUse, UserPromptSubmit, PreCompact, Notification,
}

pub struct SkillRef {
    pub plugin_id: String,  // "oh-my-claudecode"
    pub skill_name: String, // "planner"
}

pub enum McpTransport { Stdio, Http, Sse, WebSocket }

pub struct McpServerRef {
    pub name: String,
    pub transport: McpTransport,
    pub command_or_url: String,
    pub env_secret_refs: Vec<(String, SecretReference)>,  // sref://
}

pub struct RenderManifest {
    pub provider_family: ProviderFamily,
    pub files: Vec<RenderedFile>,
}

pub struct RenderedFile {
    pub target_path: PathBuf,
    pub content_blake3: [u8; 32],
    pub byte_len: u64,
}

pub enum DriftState { Match, Modified, Missing, Extra }

pub struct DriftReport {
    pub provider_family: ProviderFamily,
    pub entries: Vec<DriftEntry>,
}

pub enum RendererMode { Disabled, VerifyOnly, Reconcile }
```

### Port Trait

```rust
pub trait SettingsRenderer: Send + Sync {
    fn provider_family(&self) -> ProviderFamily;
    
    async fn render(
        &self,
        template: &SettingsTemplate,
        account: &ProviderAccount,
        root: &Path,
    ) -> Result<RenderManifest, SettingsRendererError>;
    
    async fn verify(
        &self,
        template: &SettingsTemplate,
        account: &ProviderAccount,
        root: &Path,
    ) -> Result<DriftReport, SettingsRendererError>;
}
```

## Usage Example

```rust
use intelligence_settings_template_kernel::*;

// Load template from TOML file (templates/foundry-supervisor/ deleted —
// hooks pointed at missing tools/foundry-supervisor-* binaries; pass a live path).
let template = load_template_toml(template_path)?;

// Create renderer (adapter-provided)
let renderer = ClaudeRenderer::new();

// Render settings to disk
let manifest = renderer.render(&template, &account, &home_dir).await?;

// Verify (drift detection)
let report = renderer.verify(&template, &account, &home_dir).await?;
if report.entries.iter().all(|e| e.state == DriftState::Match) {
    println!("No drift detected");
} else {
    println!("Drift: {:?}", report.entries.iter().filter(|e| e.state != DriftState::Match));
}
```

## Integration with Supervisor

The kernel types compose seamlessly with the `AccountSnapshotProvider` (v4 §B.2.3):

```rust
// In supervisor-app:
let snapshot_provider = CompositeAccountSnapshotProvider::with_verify(
    file_account_provider,
    renderers,  // Vec<Box<dyn SettingsRenderer>>
    template_root,
    home_dir,
    VerifyMode::FailOnDrift,  // or VerifyMode::AutoReconcile
);

// At tick_once() time:
let accounts = snapshot_provider.snapshot().await?;
// ← automatically calls renderer.verify() for each account
// ← excludes drifted accounts (or reconciles them)
```

## Data Flow

```
SettingsTemplate (TOML file)
  │
  └─→ SettingsRenderer::render()
       ├─→ ClaudeRenderer → ~/.claude/settings.json
       ├─→ CodexRenderer → ~/.codex/config.toml + hooks.json
       └─→ GeminiRenderer → ~/.gemini/settings.json

SettingsRenderer::verify()
  ├─→ read on-disk files
  ├─→ blake3 content hash
  └─→ compare against manifest → DriftReport
```

## Reusable Building Blocks (v6 PRE-3)

Per ADR-0069, the following types are registered as reusable building blocks:

- `SettingsTemplate` — consumers: [supervisor-app, Workflow Studio (M03+)]
- `SettingsRenderer` — consumers: [supervisor-app, settings-template-adapter]
- `HookEvent` — consumers: [3 driver impls, settings-adapter, account adapters]
- `McpServerRef` — consumers: [settings-adapter, anyone wiring MCP servers per-account]

## References

- **Plan:** `ralplan-foundry-supervisor-simple-v5-delta-settings-template-2026-05-15.md` §B.2
- **v6 Amendments:** `ralplan-foundry-supervisor-simple-v6-amendments-2026-05-15.md` §PRE-3
- **ADR-0069:** Artifact capabilities + reusable building blocks
- **Design:** `docs/DESIGN.md` (settings-template axis contract)
