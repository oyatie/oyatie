---
doc_class: Runbook
purpose: "Kernel-level operational concerns: template validation and drift detection"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Settings-Template Kernel — Operations

## Template Validation

### Validate Template Structure

> **Retired path:** `templates/foundry-supervisor/` was deleted (hooks pointed at
> missing `tools/foundry-supervisor-*` binaries). Do not invoke validate/render
> against that tree until a replacement template root lands.

```bash
# Local bridge only — pass a live template path when one exists:
cargo run -p oya-dev-cli -- gate validate settings-template \
  --template <live-settings-template.toml>

# Expected: pass if:
# - All required fields present
# - No duplicate entries (e.g., duplicate hooks)
# - All hook events are valid HookEvent variants
# - All mcp_servers have transport + command/url
```

### Acceptance Criteria

- [ ] Template parses without errors
- [ ] All HookEvent variants are recognized
- [ ] No duplicate SkillRef entries per provider
- [ ] McpServerRef entries have valid URLs/paths
- [ ] ProviderOverrides omitted_hook_events are valid variants

## Drift Detection Workflow

```
SettingsRenderer::verify()
  1. open on-disk files (e.g., ~/.claude/settings.json)
  2. blake3 hash content
  3. compare against RenderManifest (saved at last render)
  4. emit DriftReport:
     - Match: hash matches
     - Modified: file exists, hash differs
     - Missing: file not found
     - Extra: file exists but not in manifest
```

### Verifying Drift Detection Works

```bash
# Render once (requires a live template path — templates/foundry-supervisor/ deleted)
cargo run -p oya-dev-cli -- settings-template render \
  --template <live-settings-template.toml> \
  --account-id test-account

# Check manifest
cat .omc/state/settings-template-manifest.json

# Manually edit rendered file
echo '"extra_field": true' >> ~/.claude/settings.json

# Verify detects drift (pass a live templates root)
cargo run -p oya-dev-cli -- gate validate settings-drift \
  --templates-root <live-templates-root> \
  --accounts-root registry/accounts

# Should report DriftState::Modified with diff
```

## Template Maintenance

### Adding a New HookEvent

1. Add variant to `HookEvent` enum in kernel:
   ```rust
   pub enum HookEvent {
       // ... existing ...
       NewEvent,  // ← new
   }
   ```

2. Update all `SettingsRenderer` implementations:
   ```rust
   // In ClaudeRenderer
   HookEvent::NewEvent => "NewEvent",
   
   // In CodexRenderer
   HookEvent::NewEvent => "NewEventNameCodexUses",
   
   // In GeminiRenderer
   HookEvent::NewEvent => /* omitted if unsupported */
   ```

3. Test round-trip:
   ```bash
   cargo test --lib hook_event_mapping
   ```

### Adding a New Provider

1. Create new renderer in adapter crate (e.g., `AwsRenderer`)
2. Implement `SettingsRenderer` trait
3. Create new template TOML under a live templates root (do not recreate `templates/foundry-supervisor/`)
4. Register in CI lane: `lean-settings-drift` subcommand

## References

- **Drift detection:** v5 § B.5 (CI lane spec)
- **Kernel source:** `crates/oya-intelligence-settings-template-kernel/src/lib.rs`
- **Adapter:** `docs/products/foundry/supervisor/settings-template-adapter/OPERATIONS.md`
