---
doc_class: Standard
purpose: "Kernel placement, value-only invariant, and adapter composition"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Settings-Template Kernel — Architecture

## 12-Layer Placement

```
L1: Kernel          ← intelligence-settings-template-kernel (THIS CRATE)
    ├─ SettingsTemplate (value type)
    ├─ HookRef, HookEvent (value types)
    ├─ SkillRef, McpServerRef (value types)
    ├─ RenderManifest, DriftReport (value types)
    └─ SettingsRenderer (port trait)

L4: Adapter         ← intelligence-settings-template-adapter
    ├─ ClaudeRenderer (impl SettingsRenderer)
    ├─ CodexRenderer (impl SettingsRenderer)
    └─ GeminiRenderer (impl SettingsRenderer)

L5: Application     ← intelligence-supervisor-app
    └─ CompositeAccountSnapshotProvider (composes renderers)
```

## Value-Only Invariant

All types in this kernel are pure values (no `&`, `Arc`, `Box<dyn>`):

```rust
// ✓ OK: value types
pub struct SettingsTemplate {
    pub hooks: Vec<HookRef>,           // owned vector
    pub skills: Vec<SkillRef>,         // owned vector
    pub mcp_servers: Vec<McpServerRef>,// owned vector
}

// ✗ NOT OK: references
pub struct SettingsTemplate {
    pub hook_iter: impl Iterator<HookRef>,  // ✗ trait object
    pub config: &'static str,                // ✗ reference
}
```

**Benefit:** Templates can be cloned, serialized, and moved across async boundaries without allocation overhead.

## Adapter Composition Pattern

Each provider gets one `SettingsRenderer` impl:

```rust
pub struct ClaudeRenderer {
    // no state needed — stateless renderer
}

impl SettingsRenderer for ClaudeRenderer {
    fn provider_family(&self) -> ProviderFamily {
        ProviderFamily::Claude
    }
    
    async fn render(&self, template, account, root) -> Result<RenderManifest> {
        // knows how to write to ~/.claude/settings.json
    }
    
    async fn verify(&self, template, account, root) -> Result<DriftReport> {
        // knows how to compare ~/.claude/settings.json against template
    }
}
```

### Per-Provider Render Targets

| Provider | Render target(s) | Kernel knowledge |
|----------|------------------|------------------|
| Claude | `~/.claude/settings.json` | Zero — adapter knows the path |
| Codex | `~/.codex/config.toml` + `~/.codex/hooks.json` | Zero — adapter knows the paths |
| Gemini | `~/.gemini/settings.json` | Zero — adapter knows the path |

The kernel defines **what to render** (the values in `SettingsTemplate`). The adapter defines **where to render** and **how to format** it.

## Data Structure Invariants

### `HookEvent` Enum

Superset of all provider events. Each renderer maps kernel events to provider events:

```
Kernel (HookEvent enum)
  ├─ Stop
  ├─ SessionStart
  ├─ SubagentStart
  └─ ...

Claude renderer
  └─ HookEvent → Claude hook name (1:1)
     ├─ Stop → "Stop"
     ├─ SessionStart → "SessionStart"
     └─ ...

Codex renderer
  └─ HookEvent → Codex event name (1:1)
     ├─ Stop → "Stop"
     ├─ SessionStart → "SessionStart"
     └─ ...

Gemini renderer
  └─ HookEvent → Gemini event name (many omitted due to limited support)
     ├─ Stop → (omitted; not supported)
     ├─ SessionStart → (omitted; not supported)
     └─ ...
```


### `DriftState` Enum

Every rendered file falls into exactly one category:

```
DriftState::Match       ← file on-disk matches manifest
DriftState::Modified    ← file exists but content changed
DriftState::Missing     ← file should exist but doesn't
DriftState::Extra       ← file exists but not in manifest (unexpected)
```

## References

- **ADR-0056:** 12-layer enum + port-in-kernel
- **v5 Plan § B.2:** Kernel public API + contracts
- **Adapter:** `docs/products/foundry/supervisor/settings-template-adapter/README.md`
