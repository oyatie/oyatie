---
doc_class: Standard
purpose: "Per-provider renderer architecture and format dialect differences"
owner_team: axis-foundry
status: draft
---

# Settings-Template Adapter — Architecture

## Renderer Architecture

Each provider gets a dedicated renderer struct:

```
SettingsRenderer trait (kernel)
  │
  ├─→ ClaudeRenderer (this adapter)
  │   └─ writes ~/.claude/settings.json
  │
  ├─→ CodexRenderer (this adapter)
  │   ├─ writes ~/.codex/config.toml
  │   └─ writes ~/.codex/hooks.json
  │
  └─→ GeminiRenderer (this adapter)
      └─ writes ~/.gemini/settings.json
```

## Per-Provider Format Dialects

### Claude

**Format:** JSON  
**File:** `~/.claude/settings.json`  
**Merge strategy:** Partial merge (kernel writes only owned keys)

```json
{
  "hooks": [...],           // ← kernel writes these
  "skills": [...],          // ← kernel writes these
  "mcpServers": [...],      // ← kernel writes these
  "permissions": [...],     // ← kernel writes these
  "allowedTools": [...],    // ← kernel writes these
  "customKey": "value"      // ← left untouched
}
```

### Codex

**Format:** TOML + JSON (hybrid)  
**Files:** `~/.codex/config.toml`, `~/.codex/hooks.json`  
**Merge strategy:** Codex config is merged; hooks file is replaced wholly

**config.toml:**
```toml
[foundry-supervisor]
skills = [...]
allowed_tools = [...]
permissions = [...]

[other]
key = "value"  # ← left untouched
```

**hooks.json:**
```json
{
  "foundry-supervisor": {
    "hooks": [...]
  }
}
```

### Gemini

**Format:** JSON  
**File:** `~/.gemini/settings.json`  
**Merge strategy:** Partial merge (hooks omitted per v5 R2)

```json
{
  "allowed_tools": [...],
  "permissions": [...],
  "mcpServers": [...]
  // hooks: OMITTED (not supported; per-provider override)
}
```

## Hook Event Mapping

Each renderer translates kernel `HookEvent` enum to provider event names:

```rust
impl CodexRenderer {
    fn map_hook_event(&self, event: HookEvent) -> String {
        match event {
            HookEvent::Stop => "Stop",
            HookEvent::SessionStart => "SessionStart",
            HookEvent::SessionEnd => "SessionEnd",
            HookEvent::SubagentStart => "SubagentStart",
            HookEvent::SubagentStop => "SubagentStop",
            // ... others ...
        }
    }
}
```

**Test:** `tests/hook_event_mapping.rs` asserts round-trip (event → name → event).

## Atomicity Across Multiple Files

Codex writes TWO files: config.toml and hooks.json. The render operation is atomic PER FILE but NOT atomic across files.

**Scenario:** If crash between first rename and second rename, one file will be new and one will be old.

**Mitigation:** Next verify() will detect drift on one or both files and trigger reconcile.

## References

- **Kernel:** `docs/products/foundry/supervisor/settings-template-kernel/README.md`
- **v5 Plan § B.3:** Template payload shapes
- **v5 Plan § B.0:** Verified ground-truth provider layouts
