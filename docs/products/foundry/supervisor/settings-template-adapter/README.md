---
doc_class: Standard
purpose: "Per-provider settings renderer implementations and atomic write patterns"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Settings-Template Adapter — README

**Crate:** `intelligence-settings-template-adapter`  
**Layer:** Adapter (12-layer-enum L4)  
**Wave:** 2h (M02-P06, v5 delta)  
**Renamed:** `oya-intelligence-settings-template-adapter-fs` → `oya-intelligence-settings-template-adapter` (v6 PRE-1)

## Overview

The settings-template adapter implements the `SettingsRenderer` port trait for each provider (Claude, Codex, Gemini). It knows:
- **Where to write** files (`~/.claude/`, `~/.codex/`, `~/.gemini/`)
- **How to format** settings for each provider (JSON, TOML, hybrid)
- **How to reconcile** drift (atomic tempfile + rename)
- **How to defend** against symlinks (O_NOFOLLOW, O_CLOEXEC)

## Public API

### `SettingsRenderer` Implementations

```rust
pub struct ClaudeRenderer;
impl SettingsRenderer for ClaudeRenderer { ... }

pub struct CodexRenderer;
impl SettingsRenderer for CodexRenderer { ... }

pub struct GeminiRenderer;
impl SettingsRenderer for GeminiRenderer { ... }
```

Each implements:
```rust
fn provider_family(&self) -> ProviderFamily
async fn render(
    &self,
    template: &SettingsTemplate,
    account: &ProviderAccount,
    root: &Path,
) -> Result<RenderManifest, SettingsRendererError>

async fn verify(
    &self,
    template: &SettingsTemplate,
    account: &ProviderAccount,
    root: &Path,
) -> Result<DriftReport, SettingsRendererError>
```

### Render Targets (Verified Ground Truth)

| Provider | Files | Notes |
|----------|-------|-------|
| **Claude** | `~/.claude/settings.json` | Merged (only writes kernel-owned keys) |
| **Codex** | `~/.codex/config.toml` (merged) + `~/.codex/hooks.json` (replaced) | Hooks are out-of-band per v5 §B.0 |
| **Gemini** | `~/.gemini/settings.json` | Merged; hooks omitted (limited support) |

## Atomic Write Pattern (v6 BLOCKER-6)

Every renderer uses this sequence:

```rust
async fn render(&self, template, account, root) -> Result<RenderManifest> {
    // 1. Open parent dir with O_NOFOLLOW; verify it's a real directory
    let parent_fd = open(root, O_RDONLY|O_NOFOLLOW)?;
    let stat = fstat(parent_fd)?;
    assert!(is_dir(&stat) && owner_is_current_uid(&stat) && mode <= 0o755);
    close(parent_fd);
    
    // 2. Open target file with O_NOFOLLOW; reject if symlink
    let target_fd = open(target_path, O_RDONLY|O_NOFOLLOW)?;
    let existing_stat = fstat(target_fd)?;
    assert!(!is_symlink(&existing_stat));  // reject symlinks
    
    // 3. Read existing content via fd (NOT fresh path open)
    let existing = read_via_fd(target_fd);
    close(target_fd);
    
    // 4. Merge with new content
    let merged = merge_json(&existing, &new_content)?;
    
    // 5. Backup before write
    let backup_path = format!("{}/.omc-settings-bak.{}", target_path, epoch_secs);
    copy_file(target_path, backup_path)?;
    
    // 6. Write to tempfile
    let tmp_path = format!("{}.tmp", target_path);
    let tmp_fd = open(tmp_path, O_CREAT|O_WRONLY|O_EXCL)?;
    write(tmp_fd, &merged)?;
    fchmod(tmp_fd, 0o600)?;  // ← BEFORE rename
    close(tmp_fd);
    
    // 7. Atomic rename
    rename(tmp_path, target_path)?;
    
    Ok(RenderManifest { ... })
}
```

## Drift Detection

```rust
async fn verify(&self, template, account, root) -> Result<DriftReport> {
    let mut entries = Vec::new();
    
    for file in &manifest.files {
        let target = root.join(&file.target_path);
        
        // Check if file exists
        if !target.exists() {
            entries.push(DriftEntry {
                state: DriftState::Missing,
                diff: None,
            });
            continue;
        }
        
        // Read and hash
        let content = std::fs::read(&target)?;
        let hash = blake3(&content);
        
        // Compare
        if hash == file.content_blake3 {
            entries.push(DriftEntry {
                state: DriftState::Match,
                diff: None,
            });
        } else {
            let diff = diff_text(&file.original, &content);
            entries.push(DriftEntry {
                state: DriftState::Modified,
                diff: Some(diff),
            });
        }
    }
    
    // Check for extra files
    let on_disk = std::fs::read_dir(root)?;
    for entry in on_disk {
        let path = entry.path();
        if !manifest.files.iter().any(|f| f.target_path == path) {
            entries.push(DriftEntry {
                state: DriftState::Extra,
                diff: None,
            });
        }
    }
    
    Ok(DriftReport { entries })
}
```

## Usage Example

```rust
use intelligence_settings_template_adapter::*;
use oya_intelligence_settings_template_kernel::*;

let claude = ClaudeRenderer::new();
let codex = CodexRenderer::new();
let gemini = GeminiRenderer::new();

let renderers: Vec<Box<dyn SettingsRenderer>> = vec![
    Box::new(claude),
    Box::new(codex),
    Box::new(gemini),
];

// In supervisor-app:
let snapshot_provider = CompositeAccountSnapshotProvider::with_verify(
    account_provider,
    renderers,
    template_root,
    home_dir,
    VerifyMode::Reconcile,  // auto-render if drift detected
);
```

## References

- **Kernel API:** `crates/oya-intelligence-settings-template-kernel/src/lib.rs`
- **v5 Plan § B.2..B.5:** Adapter spec + per-provider layouts + CI lane
- **v6 Amendments § BLOCKER-6:** Atomic-tempfile sequence + symlink defense
- **Verified ground truth:** v5 § B.0 (actual CLI paths)
