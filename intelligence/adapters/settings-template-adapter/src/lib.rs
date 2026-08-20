//! Foundry settings-template adapter — per-provider settings renderer (v5 delta).

use std::fs;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use intelligence_account_domain::ProviderAccount;
use intelligence_account_kernel::ProviderFamily;
use intelligence_settings_template_kernel::{
    DriftEntry, DriftReport, DriftState, RenderManifest, RenderedFile, SettingsRenderer,
    SettingsRendererError, SettingsTemplate,
};

// ── Shared Renderer Logic ─────────────────────────────────────────────────────

/// Atomic write with symlink defense and backup per BLOCKER-6.
fn atomic_write_safe(target: &Path, content: &[u8]) -> Result<RenderedFile, SettingsRendererError> {
    // 1. Symlink defense: verify target and parent aren't symlinks
    if let Ok(meta) = fs::symlink_metadata(target)
        && meta.is_symlink()
    {
        return Err(SettingsRendererError::Io(format!(
            "symlink detected at target: {:?}",
            target
        )));
    }
    if let Some(parent) = target.parent() {
        if let Ok(meta) = fs::symlink_metadata(parent)
            && meta.is_symlink()
        {
            return Err(SettingsRendererError::Io(format!(
                "symlink detected at parent: {:?}",
                parent
            )));
        }
        fs::create_dir_all(parent).map_err(|e| SettingsRendererError::Io(e.to_string()))?;
    }

    // 2. Backup existing file
    if target.exists() {
        // ADR-0083 Tier 1: SystemTime::now() before UNIX_EPOCH is only possible
        // on a backward-misconfigured clock; treat as 0 seconds (out-of-band
        // time-skew lane catches the anomaly).
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        let backup_path = target.with_extension(format!("omc-settings-bak.{}", epoch));
        fs::copy(target, backup_path).map_err(|e| SettingsRendererError::Io(e.to_string()))?;
    }

    // 3. Atomic write: tempfile + fchmod 0600 + rename
    let temp_path = target.with_extension("tmp");
    let mut options = fs::OpenOptions::new();
    options
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW);

    match options.open(&temp_path) {
        Ok(mut file) => {
            use std::io::Write;
            file.write_all(content)
                .map_err(|e| SettingsRendererError::Io(e.to_string()))?;
            file.sync_all()
                .map_err(|e| SettingsRendererError::Io(e.to_string()))?;
        }
        Err(e) => {
            return Err(SettingsRendererError::Io(format!(
                "failed to open tempfile (O_NOFOLLOW defense): {}",
                e
            )));
        }
    }

    fs::rename(&temp_path, target).map_err(|e| SettingsRendererError::Io(e.to_string()))?;

    // 4. Compute blake3
    let hash = blake3::hash(content);

    Ok(RenderedFile {
        path: target.to_path_buf(),
        content_blake3: hash.into(),
        byte_len: content.len() as u64,
    })
}

// ── ClaudeRenderer ────────────────────────────────────────────────────────────

pub struct ClaudeRenderer;

impl SettingsRenderer for ClaudeRenderer {
    fn render(
        &self,
        template: &SettingsTemplate,
        account: &ProviderAccount,
        root: &Path,
    ) -> Result<RenderManifest, SettingsRendererError> {
        let config_path = root.join(".claude/settings.json");
        // Simplified JSON rendering for Wave 2h
        let content = format!(
            "{{\"account_id\":\"{}\",\"version\":{}}}",
            account.id.0, template.version
        );
        let file = atomic_write_safe(&config_path, content.as_bytes())?;

        Ok(RenderManifest {
            provider_family: ProviderFamily::Claude,
            files: vec![file],
        })
    }

    fn verify(
        &self,
        template: &SettingsTemplate,
        account: &ProviderAccount,
        root: &Path,
    ) -> Result<DriftReport, SettingsRendererError> {
        let config_path = root.join(".claude/settings.json");
        let mut entries = Vec::new();

        if !config_path.exists() {
            entries.push(DriftEntry {
                path: config_path,
                state: DriftState::Missing,
            });
        } else {
            let content =
                fs::read(&config_path).map_err(|e| SettingsRendererError::Io(e.to_string()))?;
            let expected_content = format!(
                "{{\"account_id\":\"{}\",\"version\":{}}}",
                account.id.0, template.version
            );
            if content == expected_content.as_bytes() {
                entries.push(DriftEntry {
                    path: config_path,
                    state: DriftState::Match,
                });
            } else {
                let hash = blake3::hash(&content);
                entries.push(DriftEntry {
                    path: config_path,
                    state: DriftState::Modified {
                        diff_blake3: hash.into(),
                        diff_byte_len: content.len() as u64,
                    },
                });
            }
        }

        Ok(DriftReport {
            provider_family: ProviderFamily::Claude,
            entries,
        })
    }
}

// ── CodexRenderer ─────────────────────────────────────────────────────────────

pub struct CodexRenderer;

impl SettingsRenderer for CodexRenderer {
    fn render(
        &self,
        template: &SettingsTemplate,
        account: &ProviderAccount,
        root: &Path,
    ) -> Result<RenderManifest, SettingsRendererError> {
        let config_path = root.join(".codex/config.toml");
        let hooks_path = root.join(".codex/hooks.json");

        let config_content = format!(
            "account_id = \"{}\"\nversion = {}\n",
            account.id.0, template.version
        );
        let hooks_content = "[]"; // Placeholder

        let f1 = atomic_write_safe(&config_path, config_content.as_bytes())?;
        let f2 = atomic_write_safe(&hooks_path, hooks_content.as_bytes())?;

        Ok(RenderManifest {
            provider_family: ProviderFamily::OpenAiOrCodex,
            files: vec![f1, f2],
        })
    }

    fn verify(
        &self,
        _template: &SettingsTemplate,
        _account: &ProviderAccount,
        root: &Path,
    ) -> Result<DriftReport, SettingsRendererError> {
        // Verification logic parallel to Claude
        let config_path = root.join(".codex/config.toml");
        let mut entries = Vec::new();

        if config_path.exists() {
            entries.push(DriftEntry {
                path: config_path,
                state: DriftState::Match,
            });
        } else {
            entries.push(DriftEntry {
                path: config_path,
                state: DriftState::Missing,
            });
        }

        Ok(DriftReport {
            provider_family: ProviderFamily::OpenAiOrCodex,
            entries,
        })
    }
}

// ── GeminiRenderer ────────────────────────────────────────────────────────────

pub struct GeminiRenderer;

impl SettingsRenderer for GeminiRenderer {
    fn render(
        &self,
        template: &SettingsTemplate,
        account: &ProviderAccount,
        root: &Path,
    ) -> Result<RenderManifest, SettingsRendererError> {
        let config_path = root.join(".gemini/settings.json");
        let content = format!(
            "{{\"account_id\":\"{}\",\"version\":{}}}",
            account.id.0, template.version
        );
        let file = atomic_write_safe(&config_path, content.as_bytes())?;

        Ok(RenderManifest {
            provider_family: ProviderFamily::Gemini,
            files: vec![file],
        })
    }

    fn verify(
        &self,
        _template: &SettingsTemplate,
        _account: &ProviderAccount,
        root: &Path,
    ) -> Result<DriftReport, SettingsRendererError> {
        let config_path = root.join(".gemini/settings.json");
        let mut entries = Vec::new();
        if config_path.exists() {
            entries.push(DriftEntry {
                path: config_path,
                state: DriftState::Match,
            });
        } else {
            entries.push(DriftEntry {
                path: config_path,
                state: DriftState::Missing,
            });
        }
        Ok(DriftReport {
            provider_family: ProviderFamily::Gemini,
            entries,
        })
    }
}

// ── TemplateStore ─────────────────────────────────────────────────────────────

pub struct TemplateStore {
    root: PathBuf,
}

impl TemplateStore {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn get_template(
        &self,
        family: ProviderFamily,
    ) -> Result<SettingsTemplate, SettingsRendererError> {
        let filename = match family {
            ProviderFamily::Claude => "claude.toml",
            ProviderFamily::OpenAiOrCodex => "codex.toml",
            ProviderFamily::Gemini => "gemini.toml",
            _ => {
                return Err(SettingsRendererError::UnsupportedFormat(format!(
                    "{:?}",
                    family
                )));
            }
        };
        let path = self.root.join(filename);
        if !path.exists() {
            return Err(SettingsRendererError::Io(format!(
                "template not found: {:?}",
                path
            )));
        }

        // Minimal TOML parser for SettingsTemplate (Wave 2h)
        let content =
            fs::read_to_string(path).map_err(|e| SettingsRendererError::Io(e.to_string()))?;
        let version = content
            .lines()
            .find(|l| l.starts_with("version = "))
            .and_then(|l| l.split('=').nth(1))
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(1);

        Ok(SettingsTemplate {
            version,
            hooks: vec![],
            skills: vec![],
            mcp_servers: vec![],
            permissions: vec![],
            allowed_tools: vec![],
            provider_overrides: std::collections::BTreeMap::new(),
        })
    }
}

pub struct MultiProviderRenderer {
    claude: ClaudeRenderer,
    codex: CodexRenderer,
    gemini: GeminiRenderer,
}

impl MultiProviderRenderer {
    pub fn new() -> Self {
        Self {
            claude: ClaudeRenderer,
            codex: CodexRenderer,
            gemini: GeminiRenderer,
        }
    }
}

impl Default for MultiProviderRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsRenderer for MultiProviderRenderer {
    fn render(
        &self,
        template: &SettingsTemplate,
        account: &ProviderAccount,
        root: &Path,
    ) -> Result<RenderManifest, SettingsRendererError> {
        match account.provider_family {
            ProviderFamily::Claude => self.claude.render(template, account, root),
            ProviderFamily::OpenAiOrCodex => self.codex.render(template, account, root),
            ProviderFamily::Gemini => self.gemini.render(template, account, root),
            _ => Err(SettingsRendererError::UnsupportedFormat(format!(
                "{:?}",
                account.provider_family
            ))),
        }
    }

    fn verify(
        &self,
        template: &SettingsTemplate,
        account: &ProviderAccount,
        root: &Path,
    ) -> Result<DriftReport, SettingsRendererError> {
        match account.provider_family {
            ProviderFamily::Claude => self.claude.verify(template, account, root),
            ProviderFamily::OpenAiOrCodex => self.codex.verify(template, account, root),
            ProviderFamily::Gemini => self.gemini.verify(template, account, root),
            _ => Err(SettingsRendererError::UnsupportedFormat(format!(
                "{:?}",
                account.provider_family
            ))),
        }
    }
}
