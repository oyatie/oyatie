//! Foundry settings-template kernel — pure value types (v5 delta, M02-P06).
//!
//! Per ADR-0056 (12-layer enum, port-in-kernel): kernel holds the value types
//! that adapter crates consume. No I/O. No per-provider serialization.
//! No external deps.
//!
//! Wave 2b implementation — M02-P06
//! (ralplan-foundry-supervisor-simple-v6-amendments-2026-05-15).
//!
//! Reusable building blocks (PRE-3 registry rows):
//!   - `SettingsTemplate` — canonical workspace template; one per provider family
//!   - `SettingsRenderer` — port trait; each per-provider adapter implements this
//!   - `HookEvent` — closed enum; no `Other(String)` variant
//!   - `McpServerRef` — sref://-safe reference to an MCP server entry
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// ── Re-exports ────────────────────────────────────────────────────────────────

pub use intelligence_account_domain::ProviderAccount;
pub use intelligence_account_kernel::{ProviderFamily, SecretReference};

// ── HookCommandPath newtype ───────────────────────────────────────────────────

/// Validated absolute path to a hook command binary.
///
/// Constructor rejects:
///   - empty paths
///   - NUL bytes
///   - relative paths (must start with '/')
///   - shell metacharacters: `;`, `&`, `|`, `` ` ``, `$`, `(`, `)`, `<`, `>`,
///     `"`, `'`, `\`, newline, carriage-return, tab, space
///
/// Per F-HOOKREF-COMMAND-PATH-VALIDATOR-1 (v6 HIGH).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HookCommandPath(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HookCommandPathError(pub String);

const SHELL_METACHARS: &[char] = &[
    ';', '&', '|', '`', '$', '(', ')', '<', '>', '"', '\'', '\\', '\n', '\r', '\t', ' ',
];

impl HookCommandPath {
    pub fn new(s: String) -> Result<Self, HookCommandPathError> {
        if s.is_empty() {
            return Err(HookCommandPathError("path must not be empty".to_owned()));
        }
        if s.contains('\0') {
            return Err(HookCommandPathError(
                "path must not contain NUL byte".to_owned(),
            ));
        }
        if !s.starts_with('/') {
            return Err(HookCommandPathError(
                "path must be absolute (must start with '/')".to_owned(),
            ));
        }
        if let Some(&bad) = SHELL_METACHARS.iter().find(|&&c| s.contains(c)) {
            return Err(HookCommandPathError(format!(
                "path contains shell metachar: {bad:?}"
            )));
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ── HookEvent closed enum ─────────────────────────────────────────────────────

/// Closed enum — no `Other(String)` variant per v6 amendment.
/// Adding a new event requires extending this enum (and all exhaustive matches).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    Stop,
    SubagentStop,
    SessionStart,
    SessionEnd,
    UserPromptSubmit,
    PreCompact,
    Notification,
}

// ── McpTransport ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum McpTransport {
    Stdio,
    Sse,
    Http,
    WebSocket,
}

// ── Reference types ───────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct HookRef {
    // data_class: INTERNAL_ONLY
    pub event: HookEvent, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub command: HookCommandPath, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub args: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug)]
pub struct SkillRef {
    // data_class: INTERNAL_ONLY
    pub plugin_id: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub skill_name: String, // data_class: INTERNAL_ONLY
}

/// Named struct per F-MCPSERVERREF-ENVBINDING-NAMED-STRUCT-1.
#[derive(Clone, Debug)]
pub struct EnvSecretBinding {
    // data_class: INTERNAL_ONLY
    pub env_var: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub sref: SecretReference, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug)]
pub struct McpServerRef {
    // data_class: INTERNAL_ONLY
    pub name: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub transport: McpTransport, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub command_or_url: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub env_secret_refs: Vec<EnvSecretBinding>, // data_class: SECRET
}

// ── Provider overrides ────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ProviderOverrides {
    // data_class: INTERNAL_ONLY
    pub model_override: Option<String>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub extra_allowed_tools: Vec<String>, // data_class: INTERNAL_ONLY
}

// ── SettingsTemplate ──────────────────────────────────────────────────────────

/// Canonical workspace settings template (one per provider family per account).
///
/// `provider_overrides` key is the ProviderFamily string representation
/// ("AWS", "OCI", "Claude", "OpenAIOrCodex", "Gemini") — BTreeMap gives
/// deterministic serialization order. ProviderFamily lacks Ord/Hash so a
/// String key is used per project compiler constraints.
pub struct SettingsTemplate {
    // data_class: INTERNAL_ONLY
    pub version: u32, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub hooks: Vec<HookRef>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub skills: Vec<SkillRef>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub mcp_servers: Vec<McpServerRef>, // data_class: INTERNAL_ONLY
    /// Per F-PERMISSIONENTRY-ALLOWEDTOOL-DELETE-NEWTYPE-1 (v6 MED).
    // data_class: INTERNAL_ONLY
    pub permissions: Vec<String>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub allowed_tools: Vec<String>, // data_class: INTERNAL_ONLY
    /// Key: ProviderFamily string representation (e.g. "Claude", "OpenAIOrCodex").
    // data_class: INTERNAL_ONLY
    pub provider_overrides: BTreeMap<String, ProviderOverrides>, // data_class: INTERNAL_ONLY
}

// ── Drift types ───────────────────────────────────────────────────────────────

/// Per-file drift state.
/// F-DRIFTSTATE-PAYLOAD-INLINE-1 + F-DRIFTENTRY-DIFF-HASH-NOT-RAW-1:
/// no raw diff string — blake3 hash + byte_len only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DriftState {
    Match,
    Modified {
        /// Blake3 hash of the diff content.
        diff_blake3: [u8; 32],
        diff_byte_len: u64,
    },
    Missing,
    Extra,
}

/// Single file entry in a drift report.
/// No `Option<String>` diff field per v6 amendment (F-DRIFTENTRY-DIFF-HASH-NOT-RAW-1).
#[derive(Clone, Debug)]
pub struct DriftEntry {
    // data_class: INTERNAL_ONLY
    pub path: PathBuf, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub state: DriftState, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug)]
pub struct DriftReport {
    // data_class: INTERNAL_ONLY
    pub provider_family: ProviderFamily, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub entries: Vec<DriftEntry>, // data_class: INTERNAL_ONLY
}

// ── Render manifest ───────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct RenderedFile {
    // data_class: INTERNAL_ONLY
    pub path: PathBuf, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub content_blake3: [u8; 32], // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub byte_len: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug)]
pub struct RenderManifest {
    // data_class: INTERNAL_ONLY
    pub provider_family: ProviderFamily, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub files: Vec<RenderedFile>, // data_class: INTERNAL_ONLY
}

// ── SettingsRendererError ─────────────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SettingsRendererError {
    Io(String),
    UnsupportedFormat(String),
    HookEventNotMapped(HookEvent),
    SecretRefUnresolved(String),
    InvalidTemplate(String),
}

// ── SettingsRenderer port trait ───────────────────────────────────────────────

pub trait SettingsRenderer: Send + Sync {
    /// Render the template for `account` into `root`, returning a manifest of
    /// written files (paths + blake3 hashes). Writes are atomic per BLOCKER-6
    /// (tempfile + fchmod 0600 + rename; O_NOFOLLOW symlink defense).
    fn render(
        &self,
        template: &SettingsTemplate,
        account: &ProviderAccount,
        root: &Path,
    ) -> Result<RenderManifest, SettingsRendererError>;

    /// Verify the rendered state on disk matches what `template` would produce.
    /// Returns a `DriftReport`; does not write any files.
    /// Per BLOCKER-1: results cached per (account_id, template_blake3) with TTL
    /// `SupervisorConfig::settings_verify_debounce_secs`.
    fn verify(
        &self,
        template: &SettingsTemplate,
        account: &ProviderAccount,
        root: &Path,
    ) -> Result<DriftReport, SettingsRendererError>;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // HookCommandPath validation

    #[test]
    fn hook_command_path_rejects_relative() {
        assert!(HookCommandPath::new("relative/path/cmd".to_owned()).is_err());
    }

    #[test]
    fn hook_command_path_rejects_empty() {
        assert!(HookCommandPath::new("".to_owned()).is_err());
    }

    #[test]
    fn hook_command_path_rejects_nul() {
        assert!(HookCommandPath::new("/bin/foo\0bar".to_owned()).is_err());
    }

    #[test]
    fn hook_command_path_rejects_shell_metachars() {
        let cases = [
            "/bin/cmd;rm",
            "/bin/cmd|pipe",
            "/bin/cmd$VAR",
            "/bin/cmd`exec`",
            "/bin/cmd&&next",
            "/bin/cmd >out",
        ];
        for bad in cases {
            assert!(
                HookCommandPath::new(bad.to_owned()).is_err(),
                "expected rejection for: {bad}"
            );
        }
    }

    #[test]
    fn hook_command_path_accepts_absolute_clean() {
        let p = HookCommandPath::new("/usr/local/bin/hook".to_owned()).unwrap();
        assert_eq!(p.as_str(), "/usr/local/bin/hook");
    }

    #[test]
    fn hook_command_path_accepts_path_with_dots() {
        let p = HookCommandPath::new("/usr/local/bin/oya.hook-v2".to_owned()).unwrap();
        assert_eq!(p.as_str(), "/usr/local/bin/oya.hook-v2");
    }

    // HookEvent closed enum

    #[test]
    fn hook_event_closed_enum_exhaustive_match() {
        // Exhaustive match — compiler will error if a new variant is added
        // without updating this test.
        let events = [
            HookEvent::PreToolUse,
            HookEvent::PostToolUse,
            HookEvent::Stop,
            HookEvent::SubagentStop,
            HookEvent::SessionStart,
            HookEvent::SessionEnd,
            HookEvent::UserPromptSubmit,
            HookEvent::PreCompact,
            HookEvent::Notification,
        ];
        for e in events {
            let _label: &str = match e {
                HookEvent::PreToolUse => "pre_tool_use",
                HookEvent::PostToolUse => "post_tool_use",
                HookEvent::Stop => "stop",
                HookEvent::SubagentStop => "subagent_stop",
                HookEvent::SessionStart => "session_start",
                HookEvent::SessionEnd => "session_end",
                HookEvent::UserPromptSubmit => "user_prompt_submit",
                HookEvent::PreCompact => "pre_compact",
                HookEvent::Notification => "notification",
            };
        }
    }

    // DriftState

    #[test]
    fn drift_state_match_not_missing() {
        assert!(matches!(DriftState::Match, DriftState::Match));
        assert!(!matches!(DriftState::Match, DriftState::Missing));
    }

    #[test]
    fn drift_state_modified_carries_blake3_and_len() {
        let hash = [42u8; 32];
        let d = DriftState::Modified {
            diff_blake3: hash,
            diff_byte_len: 128,
        };
        match d {
            DriftState::Modified {
                diff_blake3,
                diff_byte_len,
            } => {
                assert_eq!(diff_blake3[0], 42);
                assert_eq!(diff_byte_len, 128);
            }
            _ => panic!("expected Modified"),
        }
    }

    #[test]
    fn drift_state_extra_and_missing_are_distinct() {
        assert_ne!(DriftState::Extra, DriftState::Missing);
    }

    // SettingsRendererError

    #[test]
    fn renderer_error_hook_not_mapped_carries_event() {
        let e = SettingsRendererError::HookEventNotMapped(HookEvent::Stop);
        assert!(matches!(
            e,
            SettingsRendererError::HookEventNotMapped(HookEvent::Stop)
        ));
    }

    #[test]
    fn renderer_error_secret_ref_unresolved_carries_message() {
        let e = SettingsRendererError::SecretRefUnresolved("sref://missing-key".to_owned());
        match e {
            SettingsRendererError::SecretRefUnresolved(msg) => {
                assert!(msg.contains("sref://"));
            }
            _ => panic!("wrong variant"),
        }
    }

    // RenderedFile + DriftEntry field access

    #[test]
    fn rendered_file_fields_accessible() {
        let f = RenderedFile {
            path: PathBuf::from("/tmp/test.toml"),
            content_blake3: [0u8; 32],
            byte_len: 512,
        };
        assert_eq!(f.byte_len, 512);
        assert_eq!(f.content_blake3.len(), 32);
    }

    #[test]
    fn drift_entry_path_accessible() {
        let e = DriftEntry {
            path: PathBuf::from("/home/user/.claude/settings.json"),
            state: DriftState::Missing,
        };
        assert!(e.path.to_str().unwrap().contains("settings.json"));
        assert!(matches!(e.state, DriftState::Missing));
    }
}
