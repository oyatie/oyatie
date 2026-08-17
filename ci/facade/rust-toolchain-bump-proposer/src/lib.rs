//! Owned Rust reconciler for the ADR-0535 dependency-automation engine: reconciles the tree to a
//! desired stable Rust toolchain pin across every drift surface the cloud-ci freshness gate
//! enforces.
//!
//! Design contract (recorded in the PR body):
//! - **Declarative reconciler, not a product CLI**: the primary surface is the [`reconcile`]
//!   library API — desired state (latest stable) in, reconciled tree + report out, fail-closed on
//!   residual drift. The binary is a thin adapter a scheduled workflow or the future typed
//!   cloud-ci runner invokes, exactly like the other `oya-cloud-ci-*` automation binaries.
//! - **Pure planner**: NO network I/O. `std` has no HTTP client, and adding `reqwest`/`std::net`
//!   would violate the dependency policy (no ad-hoc dependencies) and the gate hermeticity scanner
//!   (which scans for exactly those tokens). The latest stable version arrives as a flag or
//!   environment value (`--latest-stable <v>` / `OYA_LATEST_STABLE_RUST`); the scheduled fetch
//!   belongs to the workflow step (curl to `https://static.rust-lang.org/dist/channel-rust-stable.toml`).
//! - **Pin-field surgical editor**: rewrites target the declared pin fields the evaluators
//!   actually enforce (toolchain `channel`, `oya-deps` `pin`, `rust-version`, JSON toolchain
//!   keys, Docker ARG/image pins, workflow `toolchain:` lines, `toolchains/` text) plus the
//!   explicitly curated current-policy row in `docs/standards/dependency-policy.md`. Active docs
//!   are rewritten ONLY for `rust:` image refs — never blanket version tokens, so dated
//!   snapshots and URLs (e.g. `blog.rust-lang.org/.../Rust-1.97.1/`) are never corrupted.
//! - **Self-verifying and downgrade-proof**: applying requires `latest > current` (a stale or
//!   mis-parsed input fails closed instead of rewriting the tree backward), and an equal-pin
//!   apply still runs both residual-drift validators so a partially updated tree is never
//!   reported aligned.
//! - **Zero new external dependencies**: only owned path crates (`ci-generated-artifact-freshness`
//!   for `read_pinned_rust_toolchain` + `evaluate_rust_toolchain_drift`;
//!   `ci-dependency-automation` for `evaluate_repo`) plus the workspace `serde_json` for the
//!   machine-readable reconciliation report.

#![forbid(unsafe_code)]

use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use ci_dependency_automation::{Verdict, evaluate_repo, render_findings};
use ci_generated_artifact_freshness::{evaluate_rust_toolchain_drift, read_pinned_rust_toolchain};

/// Mirrors `rust_toolchain_drift::EXCLUDED_PREFIXES` in the freshness crate. Deliberately kept
/// in step with it: the rewrite surface must equal the evaluation surface or a bump could leave
/// a scanned file stale while claiming clean.
const EXCLUDED_PREFIXES: [&str; 12] = [
    ".git/",
    "buck-out/",
    "target/",
    "third-party/",
    ".claude/",
    ".codex/",
    ".omc/",
    ".omx/",
    "node_modules/",
    "cloud/cloud-kernel/",
    "docs/audit/",
    "docs/research/",
];

/// Mirrors `rust_toolchain_drift::ACTIVE_TEXT_PATHS` in the freshness crate.
const ACTIVE_TEXT_PATHS: [&str; 8] = [
    "docs/PRD-OYATIE-FROM-SCRATCH-CANONICAL.md",
    "docs/architecture/",
    "docs/automation/",
    "docs/decisions/ADR-0700-ci-admission-live-apex.md",
    "docs/plans/",
    "docs/standards/",
    "specs/oss-stewardship-registry.json",
    "toolchains/",
];

/// The one managed-file doc whose toolchain row is a declared current-policy pin location
/// (`oya-deps.toml` declares it `update = "sync-rust-pin"`). Its `| Rust toolchain | <v> stable |`
/// row is rewritten on a bump; every other doc is rewritten only for `rust:` image refs.
const DEPENDENCY_POLICY_DOC: &str = "docs/standards/dependency-policy.md";

/// Typed, matchable error surface for the reconciler API — a typed runner can branch on
/// recoverable vs terminal outcomes without parsing display text. Wrapped causes are preserved
/// (`source()` returns them for `Io`, `Freshness`, and `DependencyAutomation`).
#[derive(Debug)]
pub enum ProposerError {
    /// The supplied latest-stable text is empty, non-numeric, or malformed.
    VersionInvalid(String),
    /// The supplied latest stable is OLDER than the pinned channel; the tree is never rewritten
    /// backward.
    VersionOlder { current: String, supplied: String },
    /// No latest-stable value was supplied (no flag and no environment variable).
    VersionUnavailable(String),
    /// Filesystem read/write/metadata failure.
    Io {
        context: String,
        source: std::io::Error,
    },
    /// The freshness drift evaluator failed to run.
    Freshness(ci_generated_artifact_freshness::FreshnessError),
    /// The ADR-0535 dependency-automation gate failed to run.
    DependencyAutomation(ci_dependency_automation::GateError),
    /// The tree is not drift-aligned after reconciliation (equal pin with stale surfaces, or
    /// residual findings after an applied bump).
    ResidualDrift(ResidualDrift),
}

impl std::fmt::Display for ProposerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposerError::VersionInvalid(message) | ProposerError::VersionUnavailable(message) => {
                formatter.write_str(message)
            }
            ProposerError::VersionOlder { current, supplied } => write!(
                formatter,
                "supplied latest stable {supplied} is not newer than the pinned {current}; \
                 refusing to rewrite the tree backward"
            ),
            ProposerError::Io { context, source } => {
                write!(formatter, "{context}: {source}")
            }
            ProposerError::Freshness(error) => write!(formatter, "freshness: {error}"),
            ProposerError::DependencyAutomation(error) => {
                write!(formatter, "dependency-automation gate: {error}")
            }
            ProposerError::ResidualDrift(residual) => {
                formatter.write_str("tree has residual drift after reconciliation:")?;
                formatter.write_str(&render_residual(residual))
            }
        }
    }
}

impl std::error::Error for ProposerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProposerError::Io { source, .. } => Some(source),
            ProposerError::Freshness(error) => Some(error),
            ProposerError::DependencyAutomation(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ci_generated_artifact_freshness::FreshnessError> for ProposerError {
    fn from(error: ci_generated_artifact_freshness::FreshnessError) -> Self {
        ProposerError::Freshness(error)
    }
}

impl From<ci_dependency_automation::GateError> for ProposerError {
    fn from(error: ci_dependency_automation::GateError) -> Self {
        ProposerError::DependencyAutomation(error)
    }
}

fn io_error(context: &str, error: std::io::Error) -> ProposerError {
    ProposerError::Io {
        context: context.to_owned(),
        source: error,
    }
}

/// Read the pinned toolchain channel via the freshness crate's canonical reader.
pub fn current_pin(repo_root: &Path) -> Result<String, ProposerError> {
    Ok(read_pinned_rust_toolchain(repo_root)?)
}

/// Validate and normalize a latest-stable version string from a flag/env/file.
///
/// Accepts `1.98.0`, `v1.98.0`, `1.98` (channel-form), or the raw
/// `[pkg.rust] version` value from `channel-rust-stable.toml` (`1.97.1 (8bab26f4f 2026-07-14)` —
/// the parenthetical is truncated). Returns the normalized `1.98.0` three-part form. Anything else
/// fails closed: a bump proposer must never guess at a version.
pub fn parse_stable_version(text: &str) -> Result<String, ProposerError> {
    let trimmed = text
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_start_matches('v')
        .trim();
    if trimmed.is_empty() {
        return Err(ProposerError::VersionInvalid(
            "latest stable version is empty; pass --latest-stable <v> or OYA_LATEST_STABLE_RUST"
                .to_owned(),
        ));
    }
    if !trimmed.chars().all(|ch| ch.is_ascii_digit() || ch == '.') {
        return Err(ProposerError::VersionInvalid(format!(
            "latest stable version {trimmed:?} must contain only digits and dots"
        )));
    }
    let parts: Vec<&str> = trimmed.split('.').collect();
    if !(2..=3).contains(&parts.len()) {
        return Err(ProposerError::VersionInvalid(format!(
            "latest stable version {trimmed:?} must be a two- or three-part semver"
        )));
    }
    for part in &parts {
        if part.is_empty() || !part.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(ProposerError::VersionInvalid(format!(
                "latest stable version {trimmed:?} has a non-numeric component"
            )));
        }
    }
    let normalized = if parts.len() == 2 {
        // `1.98` -> `1.98.0`
        format!("{trimmed}.0")
    } else {
        trimmed.to_owned()
    };
    Ok(normalized)
}

/// Numeric version components for ordering; fails closed on malformed input.
fn version_parts(version: &str) -> Result<Vec<u64>, ProposerError> {
    version
        .split('.')
        .map(|part| {
            part.parse::<u64>().map_err(|_| {
                ProposerError::VersionInvalid(format!(
                    "version component {part:?} in {version:?} is not numeric"
                ))
            })
        })
        .collect()
}

/// True iff `latest` is strictly newer than `current` (numeric component order).
pub fn latest_is_newer(current: &str, latest: &str) -> Result<bool, ProposerError> {
    let current_parts = version_parts(current)?;
    let latest_parts = version_parts(latest)?;
    for (older, newer) in current_parts.iter().zip(latest_parts.iter()) {
        if newer != older {
            return Ok(newer > older);
        }
    }
    Ok(latest_parts.len() > current_parts.len())
}

/// Boundary-aware version replacement: replaces every occurrence of `old` whose surrounding
/// characters are neither digits nor dots, so `1.97.1` inside `1.97.10` or `11.97.1` is never
/// corrupted, while `1.97.1-stable`, `rust:1.97.1-slim`, `"1.97.1"` and `1.97.1-` all rewrite.
///
/// Used for the `toolchains/` text surface (the freshness evaluator's `explicit_rust_versions`
/// contract) and as the per-line primitive for workflow pin lines.
pub fn rewrite_version_boundary(text: &str, old: &str, new: &str) -> String {
    if old.is_empty() || old == new {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(index) = rest.find(old) {
        let before_ok = index == 0
            || !(rest.as_bytes()[index - 1].is_ascii_digit() || rest.as_bytes()[index - 1] == b'.');
        let after = index + old.len();
        let after_ok = after >= rest.len()
            || !(rest.as_bytes()[after].is_ascii_digit() || rest.as_bytes()[after] == b'.');
        if before_ok && after_ok {
            out.push_str(&rest[..index]);
            out.push_str(new);
            rest = &rest[after..];
        } else {
            // Not a standalone occurrence: advance one char (multi-byte safe) and keep scanning.
            let char_len = rest[index..].chars().next().map_or(1, char::len_utf8);
            out.push_str(&rest[..index + char_len]);
            rest = &rest[index + char_len..];
        }
    }
    out.push_str(rest);
    out
}

/// Rewrite a TOML key's quoted value wherever the key is assigned, independent of the file's
/// whitespace and quote style: `key = "old"`, `key="old"`, `key  =  "old"`, `key = \'old\'` and any
/// indented variant all rewrite.
///
/// This previously did two literal `replace` calls against the canonical spaced form and the
/// compact form. Every other VALID TOML spelling — single quotes, extra spaces around `=` — was
/// silently left unchanged, so reconciliation would update the sibling files, fail only in
/// post-verification, and leave a PARTIALLY bumped checkout behind.
///
/// The quote character is preserved, the value is replaced only when it equals `old`, and the
/// surrounding bytes are untouched, so no formatting round-trip occurs. Only the declared pin
/// fields are targeted, never arbitrary version tokens elsewhere in the file.
fn rewrite_toml_key(text: &str, key: &str, old: &str, new: &str) -> String {
    if old.is_empty() || old == new {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    for (index, line) in text.split_inclusive('\n').enumerate() {
        let _ = index;
        match rewritten_toml_line(line, key, old, new) {
            Some(replaced) => out.push_str(&replaced),
            None => out.push_str(line),
        }
    }
    out
}

/// Rewrite one `key = "value"` assignment line, or return `None` when the line is not this key\'s
/// assignment or the value does not equal `old`.
fn rewritten_toml_line(line: &str, key: &str, old: &str, new: &str) -> Option<String> {
    let indent_len = line.len() - line.trim_start().len();
    let (indent, rest) = line.split_at(indent_len);
    let rest = rest.strip_prefix(key)?;
    // Reject a longer key that merely starts with `key` (e.g. `channel-override`).
    let after_key = rest.trim_start();
    let equals_gap = &rest[..rest.len() - after_key.len()];
    let value_part = after_key.strip_prefix('=')?;
    let value_trimmed = value_part.trim_start();
    let value_gap = &value_part[..value_part.len() - value_trimmed.len()];
    let quote = value_trimmed
        .chars()
        .next()
        .filter(|c| *c == '"' || *c == '\'')?;
    let body = &value_trimmed[quote.len_utf8()..];
    let end = body.find(quote)?;
    if &body[..end] != old {
        return None;
    }
    let tail = &body[end + quote.len_utf8()..];
    Some(format!(
        "{indent}{key}{equals_gap}={value_gap}{quote}{new}{quote}{tail}"
    ))
}

/// Rewrite one JSON object key's string value wherever the key appears, independent of the file's
/// whitespace style: `"rust": "1.97.1"`, `"rust":"1.97.1"`, `"rust" :\t"1.97.1"`, and
/// multi-line `"rust"\n  :\n  "1.97.1"` all rewrite. The value string literal is parsed like JSON
/// (escapes respected) and only replaced when it equals `old`; the surrounding formatting and the
/// rest of the file are preserved byte-for-byte, so a formatting round-trip never occurs.
fn rewrite_json_pin_value(text: &str, key: &str, old: &str, new: &str) -> String {
    if old.is_empty() || old == new {
        return text.to_owned();
    }
    let key_needle = format!("\"{key}\"");
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(index) = rest.find(&key_needle) {
        let bytes = rest.as_bytes();
        let after_key = index + key_needle.len();
        let mut pos = after_key;
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        let expect_colon = pos < bytes.len() && bytes[pos] == b':';
        if expect_colon {
            pos += 1;
            while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
                pos += 1;
            }
        }
        if !expect_colon || pos >= bytes.len() || bytes[pos] != b'"' {
            // Not a `"key": "..."` member — keep the key and keep scanning after it.
            out.push_str(&rest[..after_key]);
            rest = &rest[after_key..];
            continue;
        }
        pos += 1;
        let value_start = pos;
        let mut value_end = pos;
        let mut escaped = false;
        while value_end < bytes.len() {
            let byte = bytes[value_end];
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                break;
            }
            value_end += 1;
        }
        if value_end >= bytes.len() {
            // Unterminated string literal — keep the key and keep scanning.
            out.push_str(&rest[..after_key]);
            rest = &rest[after_key..];
            continue;
        }
        if &rest[value_start..value_end] == old {
            // Keep the key, colon, whitespace, and opening quote; swap only the value content.
            out.push_str(&rest[..value_start]);
            out.push_str(new);
            rest = &rest[value_end..];
        } else {
            // Different value under the same key: keep the whole member and keep scanning.
            out.push_str(&rest[..value_end + 1]);
            rest = &rest[value_end + 1..];
        }
    }
    out.push_str(rest);
    out
}

/// Rewrite the toolchain-pin keys the freshness evaluator checks in `manifest.json` /
/// `supported-oses.json`: `toolchain.rust`, `lts_pins.rust` (key `"rust"`) and the root
/// `rust_toolchain` key (value `old-stable`). Formatting-agnostic: any JSON whitespace around the
/// colon is handled, so compact and pretty-printed manifests both update.
fn rewrite_json_rust_pins(text: &str, old: &str, new: &str) -> String {
    let mut out = rewrite_json_pin_value(
        text,
        "rust_toolchain",
        &format!("{old}-stable"),
        &format!("{new}-stable"),
    );
    out = rewrite_json_pin_value(&out, "rust_toolchain", old, new);
    rewrite_json_pin_value(&out, "rust", old, new)
}

/// Rewrite `prefix+old` image refs to `prefix+new` where the character after `old` is not a digit
/// or dot (`rust:1.97.1-bookworm`, `rust:1.97.1-slim`, and `clux/muslrust:1.97.1-stable` which
/// contains the `rust:1.97.1` substring). `rust:1.97.10` is never corrupted.
fn rewrite_image_refs(text: &str, prefix: &str, old: &str, new: &str) -> String {
    if old.is_empty() || old == new {
        return text.to_owned();
    }
    let needle = format!("{prefix}{old}");
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(index) = rest.find(&needle) {
        let after = index + needle.len();
        let after_ok = after >= rest.len()
            || !(rest.as_bytes()[after].is_ascii_digit() || rest.as_bytes()[after] == b'.');
        if after_ok {
            out.push_str(&rest[..index]);
            out.push_str(&format!("{prefix}{new}"));
            rest = &rest[after..];
        } else {
            let char_len = rest[index..].chars().next().map_or(1, char::len_utf8);
            out.push_str(&rest[..index + char_len]);
            rest = &rest[index + char_len..];
        }
    }
    out.push_str(rest);
    out
}

/// Rewrite the Docker pin surfaces the freshness evaluator checks: `ARG RUST_VERSION=old` and
/// `rust:old` image refs (FROM lines and any other refs).
fn rewrite_docker_pins(text: &str, old: &str, new: &str) -> String {
    let mut out = text.replace(
        &format!("ARG RUST_VERSION={old}"),
        &format!("ARG RUST_VERSION={new}"),
    );
    out = rewrite_image_refs(&out, "rust:", old, new);
    out
}

/// Rewrite workflow toolchain-pin lines only: `toolchain: old` (quoted or bare), `--toolchain old`,
/// `rustup toolchain install old`, and `.rustup/toolchains/old-*` cache paths. Lines without a
/// toolchain-pin marker are left byte-identical, so unrelated version tokens in workflow YAML
/// (e.g. action version strings) are never touched. Line endings are preserved exactly — `\r\n`
/// (CRLF), `\n`, and a final unterminated line all survive a rewrite, so a workflow with no
/// matching pin is byte-identical after planning/apply.
fn rewrite_workflow_pins(text: &str, old: &str, new: &str) -> String {
    const PIN_MARKERS: [&str; 4] = [
        "toolchain:",
        "--toolchain",
        "rustup toolchain",
        ".rustup/toolchains/",
    ];
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(newline) = rest.find('\n') {
        let line = &rest[..newline];
        let (content, terminator) = if let Some(stripped) = line.strip_suffix('\r') {
            (stripped, "\r\n")
        } else {
            (line, "\n")
        };
        if PIN_MARKERS.iter().any(|marker| content.contains(marker)) {
            out.push_str(&rewrite_version_boundary(content, old, new));
        } else {
            out.push_str(content);
        }
        out.push_str(terminator);
        rest = &rest[newline + 1..];
    }
    // Final line without a trailing newline (or empty tail).
    if PIN_MARKERS.iter().any(|marker| rest.contains(marker)) {
        out.push_str(&rewrite_version_boundary(rest, old, new));
    } else {
        out.push_str(rest);
    }
    out
}

/// Rewrite the declared current-policy toolchain row in `docs/standards/dependency-policy.md`
/// (`| Rust toolchain | <v> stable | ...`), which `oya-deps.toml` declares as a
/// `sync-rust-pin` managed file.
fn rewrite_dependency_policy_row(text: &str, old: &str, new: &str) -> String {
    text.replace(
        &format!("Rust toolchain | {old} stable"),
        &format!("Rust toolchain | {new} stable"),
    )
}

/// Dispatch the surgical rewrite for one candidate path. Every category targets exactly the pin
/// fields the freshness drift evaluator (and the ADR-0535 gate) enforce; nothing else in a file
/// is rewritten.
fn rewrite_for_path(rel: &str, text: &str, old: &str, new: &str) -> String {
    if rel == "rust-toolchain.toml" {
        rewrite_toml_key(text, "channel", old, new)
    } else if rel == "oya-deps.toml" {
        rewrite_toml_key(text, "pin", old, new)
    } else if rel == "Cargo.toml" || rel.ends_with("/Cargo.toml") {
        rewrite_toml_key(text, "rust-version", old, new)
    } else if rel.ends_with("manifest.json") || rel.ends_with("supported-oses.json") {
        rewrite_json_rust_pins(text, old, new)
    } else if is_dockerfile_path(rel) {
        rewrite_docker_pins(text, old, new)
    } else if rel.starts_with(".github/workflows/") {
        rewrite_workflow_pins(text, old, new)
    } else if rel.starts_with("toolchains/") {
        rewrite_version_boundary(text, old, new)
    } else if rel == DEPENDENCY_POLICY_DOC {
        rewrite_dependency_policy_row(text, old, new)
    } else if active_text_path(rel) {
        // Curated: only `rust:` image refs — never blanket version tokens in documentation.
        rewrite_image_refs(text, "rust:", old, new)
    } else {
        text.to_owned()
    }
}

/// One file in a bump plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedFile {
    pub path: String,
    pub changed: bool,
}

/// The full deterministic bump plan: every candidate file with its change flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BumpPlan {
    pub old: String,
    pub new: String,
    pub files: Vec<PlannedFile>,
}

impl BumpPlan {
    pub fn changed_count(&self) -> usize {
        self.files.iter().filter(|file| file.changed).count()
    }

    pub fn changed_paths(&self) -> Vec<&str> {
        self.files
            .iter()
            .filter(|file| file.changed)
            .map(|file| file.path.as_str())
            .collect()
    }
}

fn excluded_path(path: &str) -> bool {
    EXCLUDED_PREFIXES
        .iter()
        .any(|prefix| path.starts_with(prefix))
}

fn is_dockerfile_path(path: &str) -> bool {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("Dockerfile"))
}

fn active_text_path(path: &str) -> bool {
    ACTIVE_TEXT_PATHS
        .iter()
        .any(|prefix| path == *prefix || path.starts_with(prefix))
}

fn relevant_to_bump(path: &str) -> bool {
    path == "rust-toolchain.toml"
        || path == "oya-deps.toml"
        || path == "Cargo.toml"
        || path.ends_with("/Cargo.toml")
        || path.ends_with("manifest.json")
        || path.ends_with("supported-oses.json")
        || is_dockerfile_path(path)
        || path.starts_with(".github/workflows/")
        || path.starts_with("toolchains/")
        || active_text_path(path)
}

/// Enumerate the rewrite surface: the same walk the freshness drift evaluator performs, plus the
/// ADR-0535 gate surfaces (`oya-deps.toml`, `toolchains/BUCK`).
fn candidate_paths(repo_root: &Path) -> Result<Vec<String>, ProposerError> {
    let mut paths = Vec::new();
    let mut queue = vec![repo_root.to_path_buf()];
    while let Some(dir) = queue.pop() {
        let entries = fs::read_dir(&dir)
            .map_err(|error| io_error(&format!("read_dir {}", dir.display()), error))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| io_error(&format!("read_dir entry {}", dir.display()), error))?;
        let mut entries: Vec<PathBuf> = entries.into_iter().map(|entry| entry.path()).collect();
        entries.sort();
        for path in entries {
            let rel = path
                .strip_prefix(repo_root)
                .map_err(|error| ProposerError::Io {
                    context: format!("strip repo root from {}", path.display()),
                    source: std::io::Error::other(error),
                })?
                .to_string_lossy()
                .replace('\\', "/");
            if excluded_path(&rel) {
                continue;
            }
            let metadata = fs::symlink_metadata(&path).map_err(|error| {
                io_error(&format!("symlink_metadata {}", path.display()), error)
            })?;
            if metadata.file_type().is_symlink() {
                continue;
            }
            if metadata.is_dir() {
                queue.push(path);
                continue;
            }
            if metadata.is_file() && !rel.ends_with(".generated.json") && relevant_to_bump(&rel) {
                paths.push(rel);
            }
        }
    }
    paths.sort();
    Ok(paths)
}

/// Compute the deterministic bump plan for `old -> new` without touching disk.
pub fn plan_bump(repo_root: &Path, old: &str, new: &str) -> Result<BumpPlan, ProposerError> {
    let mut files = Vec::new();
    for rel in candidate_paths(repo_root)? {
        let text = fs::read_to_string(repo_root.join(&rel))
            .map_err(|error| io_error(&format!("read {rel}"), error))?;
        let rewritten = rewrite_for_path(&rel, &text, old, new);
        files.push(PlannedFile {
            path: rel,
            changed: rewritten != text,
        });
    }
    Ok(BumpPlan {
        old: old.to_owned(),
        new: new.to_owned(),
        files,
    })
}

/// Apply a plan's changed files. Files are rewritten from their on-disk content, so a plan
/// remains correct even if the tree moved between planning and applying.
pub fn apply_plan(repo_root: &Path, plan: &BumpPlan) -> Result<(), ProposerError> {
    for file in plan.files.iter().filter(|file| file.changed) {
        let path = repo_root.join(&file.path);
        let text = fs::read_to_string(&path)
            .map_err(|error| io_error(&format!("read {}", file.path), error))?;
        let rewritten = rewrite_for_path(&file.path, &text, &plan.old, &plan.new);
        fs::write(&path, rewritten)
            .map_err(|error| io_error(&format!("write {}", file.path), error))?;
    }
    Ok(())
}

/// Residual-drift report after an applied bump: freshness drift findings + ADR-0535 gate
/// findings. Empty both ways means the tree is clean.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResidualDrift {
    pub drift_findings: Vec<String>,
    pub gate_findings: Vec<String>,
}

impl ResidualDrift {
    pub fn is_clean(&self) -> bool {
        self.drift_findings.is_empty() && self.gate_findings.is_empty()
    }
}

fn render_residual(residual: &ResidualDrift) -> String {
    let mut out = String::new();
    for finding in &residual.drift_findings {
        out.push_str(&format!("\n  drift: {finding}"));
    }
    for finding in &residual.gate_findings {
        out.push_str(&format!("\n  gate: {finding}"));
    }
    out
}

/// Verify the tree against BOTH enforcement surfaces: the freshness rust-toolchain drift
/// evaluator and the ADR-0535 dependency-automation gate. Fails closed: any finding is surfaced,
/// never guessed around.
pub fn verify_clean(repo_root: &Path) -> Result<ResidualDrift, ProposerError> {
    let drift = evaluate_rust_toolchain_drift(repo_root)?;
    let drift_findings = drift
        .iter()
        .map(|finding| format!("{} {}: {}", finding.code, finding.key, finding.detail))
        .collect();

    let gate = evaluate_repo(repo_root)?;
    let gate_findings = if gate.verdict == Verdict::Green {
        Vec::new()
    } else {
        render_findings(&gate).lines().map(str::to_owned).collect()
    };

    Ok(ResidualDrift {
        drift_findings,
        gate_findings,
    })
}

/// Outcome of a reconciliation pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconcileOutcome {
    /// The pinned channel already equals the desired latest stable AND the tree is drift-clean.
    UpToDate,
    /// The tree was rewritten to the desired pin and both validators certify it clean.
    Bumped,
}

impl Display for ReconcileOutcome {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            ReconcileOutcome::UpToDate => "up-to-date",
            ReconcileOutcome::Bumped => "bumped",
        })
    }
}

/// Machine-readable reconciliation report — the declarative surface a scheduled workflow or the
/// future typed cloud-ci runner consumes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileReport {
    pub current: String,
    pub latest: String,
    pub outcome: ReconcileOutcome,
    pub changed_files: Vec<String>,
}

/// Reconcile the tree to the desired latest stable pin — the primary entry point of this
/// capability.
///
/// - A `latest` that is OLDER than the pinned channel fails closed (never rewrites backward).
/// - A `latest` EQUAL to the pinned channel still runs both residual-drift validators; a stale
///   tree (partial prior update, or drift introduced by another change) is reported as an error,
///   never as "up to date".
/// - A newer `latest` plans, applies, then re-verifies with the freshness drift evaluator AND the
///   ADR-0535 gate; residual drift after the rewrite fails closed.
pub fn reconcile(repo_root: &Path, latest: &str) -> Result<ReconcileReport, ProposerError> {
    let latest = parse_stable_version(latest)?;
    let current = current_pin(repo_root)?;

    if !latest_is_newer(&current, &latest)? {
        if current == latest {
            let residual = verify_clean(repo_root)?;
            if !residual.is_clean() {
                return Err(ProposerError::ResidualDrift(residual));
            }
            return Ok(ReconcileReport {
                current,
                latest,
                outcome: ReconcileOutcome::UpToDate,
                changed_files: Vec::new(),
            });
        }
        return Err(ProposerError::VersionOlder {
            current,
            supplied: latest,
        });
    }

    let plan = plan_bump(repo_root, &current, &latest)?;
    apply_plan(repo_root, &plan)?;
    let residual = verify_clean(repo_root)?;
    if !residual.is_clean() {
        return Err(ProposerError::ResidualDrift(residual));
    }

    Ok(ReconcileReport {
        current,
        latest,
        outcome: ReconcileOutcome::Bumped,
        changed_files: plan
            .changed_paths()
            .into_iter()
            .map(str::to_owned)
            .collect(),
    })
}

#[cfg(test)]
mod tests {

    /// Reviewer finding: only the two canonical spellings were rewritten, so any other VALID
    /// TOML formatting left the pin behind and produced a partially bumped checkout.
    #[test]
    fn rewrite_toml_key_handles_every_valid_pin_spelling() {
        for (input, expected) in [
            ("channel = \"1.97.1\"\n", "channel = \"1.98.0\"\n"),
            ("channel=\"1.97.1\"\n", "channel=\"1.98.0\"\n"),
            ("channel  =  \"1.97.1\"\n", "channel  =  \"1.98.0\"\n"),
            ("channel = \'1.97.1\'\n", "channel = \'1.98.0\'\n"),
            ("  channel = \"1.97.1\"\n", "  channel = \"1.98.0\"\n"),
            (
                "channel = \"1.97.1\" # pinned\n",
                "channel = \"1.98.0\" # pinned\n",
            ),
        ] {
            assert_eq!(
                rewrite_toml_key(input, "channel", "1.97.1", "1.98.0"),
                expected,
                "spelling not rewritten: {input:?}"
            );
        }
    }

    /// A key that merely PREFIXES another key must not be rewritten, and a non-matching value
    /// must be left alone.
    #[test]
    fn rewrite_toml_key_does_not_touch_lookalike_keys_or_other_values() {
        assert_eq!(
            rewrite_toml_key(
                "channel-override = \"1.97.1\"\n",
                "channel",
                "1.97.1",
                "1.98.0"
            ),
            "channel-override = \"1.97.1\"\n"
        );
        assert_eq!(
            rewrite_toml_key("channel = \"1.90.0\"\n", "channel", "1.97.1", "1.98.0"),
            "channel = \"1.90.0\"\n"
        );
    }

    use super::*;

    fn write(root: &Path, rel: &str, content: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("create dirs");
        fs::write(path, content).expect("write fixture");
    }

    fn read(root: &Path, rel: &str) -> String {
        fs::read_to_string(root.join(rel)).expect("read fixture")
    }

    #[test]
    fn boundary_rewrite_never_corrupts_longer_versions() {
        let text = "1.97.1 1.97.10 11.97.1 v1.97.1";
        assert_eq!(
            rewrite_version_boundary(text, "1.97.1", "1.98.0"),
            "1.98.0 1.97.10 11.97.1 v1.98.0"
        );
    }

    #[test]
    fn toml_key_rewrite_targets_declared_pin_fields_only() {
        let toolchain = "[toolchain]\nchannel = \"1.97.1\"\nprofile = \"minimal\"\n";
        assert_eq!(
            rewrite_toml_key(toolchain, "channel", "1.97.1", "1.98.0"),
            "[toolchain]\nchannel = \"1.98.0\"\nprofile = \"minimal\"\n"
        );
        // A non-pin value of the same shape must not be touched by a different key.
        let deps = "[rust]\npin = \"1.97.1\"\n[other]\nchannel = \"not-a-version\"\n";
        assert_eq!(
            rewrite_toml_key(deps, "pin", "1.97.1", "1.98.0"),
            "[rust]\npin = \"1.98.0\"\n[other]\nchannel = \"not-a-version\"\n"
        );
        // Compact form.
        assert_eq!(
            rewrite_toml_key(
                "rust-version=\"1.97.1\"",
                "rust-version",
                "1.97.1",
                "1.98.0"
            ),
            "rust-version=\"1.98.0\""
        );
    }

    #[test]
    fn json_manifest_rewrite_targets_toolchain_keys_only() {
        let manifest = "{\n  \"toolchain\": { \"rust\": \"1.97.1\" },\n  \"rust_toolchain\": \"1.97.1-stable\",\n  \"sdk_version\": \"1.97.1\"\n}\n";
        let rewritten = rewrite_json_rust_pins(manifest, "1.97.1", "1.98.0");
        assert!(rewritten.contains("\"rust\": \"1.98.0\""));
        assert!(rewritten.contains("\"rust_toolchain\": \"1.98.0-stable\""));
        assert!(
            rewritten.contains("\"sdk_version\": \"1.97.1\""),
            "unrelated JSON fields must stay"
        );
    }

    #[test]
    fn docker_rewrite_targets_arg_and_image_refs_only() {
        let docker =
            "ARG RUST_VERSION=1.97.1\nFROM rust:1.97.1-bookworm AS builder\nRUN echo 1.97.1\n";
        let rewritten = rewrite_docker_pins(docker, "1.97.1", "1.98.0");
        assert!(rewritten.contains("ARG RUST_VERSION=1.98.0"));
        assert!(rewritten.contains("FROM rust:1.98.0-bookworm AS builder"));
        assert!(
            rewritten.contains("RUN echo 1.97.1"),
            "unrelated shell tokens must stay"
        );
    }

    #[test]
    fn workflow_rewrite_touches_only_pin_lines() {
        let workflow = r#"toolchain: "1.97.1"
      toolchain: 1.97.1
rustup toolchain install 1.97.1
--toolchain 1.97.1-$host
~/.rustup/toolchains/1.97.1-aarch64-apple-darwin/bin
uses: some/action@v1.97.1
"#;
        let rewritten = rewrite_workflow_pins(workflow, "1.97.1", "1.98.0");
        assert!(rewritten.contains("toolchain: \"1.98.0\""));
        assert!(rewritten.contains("toolchain: 1.98.0"));
        assert!(rewritten.contains("rustup toolchain install 1.98.0"));
        assert!(rewritten.contains("--toolchain 1.98.0-$host"));
        assert!(rewritten.contains("~/.rustup/toolchains/1.98.0-aarch64-apple-darwin/bin"));
        assert!(
            rewritten.contains("uses: some/action@v1.97.1"),
            "action version strings are not toolchain pins and must not be rewritten"
        );
    }

    #[test]
    fn workflow_rewrite_preserves_crlf_and_unterminated_lines() {
        // CRLF checkout: a workflow with no matching pin must come back byte-identical.
        let crlf = "name: x\r\non: push\r\njobs: {}\r\n";
        assert_eq!(
            rewrite_workflow_pins(crlf, "1.97.1", "1.98.0"),
            crlf,
            "a CRLF workflow without a matching pin must be byte-identical"
        );
        // A pin rewrite keeps CRLF terminators on every line.
        let crlf_pin = "steps:\r\n  - run: echo\r\n    toolchain: \"1.97.1\"\r\n";
        let rewritten = rewrite_workflow_pins(crlf_pin, "1.97.1", "1.98.0");
        assert_eq!(
            rewritten,
            "steps:\r\n  - run: echo\r\n    toolchain: \"1.98.0\"\r\n"
        );
        // A final line without a terminator survives.
        let no_trailing = "toolchain: \"1.97.1\"";
        assert_eq!(
            rewrite_workflow_pins(no_trailing, "1.97.1", "1.98.0"),
            "toolchain: \"1.98.0\""
        );
        assert_eq!(
            rewrite_workflow_pins("plain text", "1.97.1", "1.98.0"),
            "plain text"
        );
    }

    #[test]
    fn json_manifest_rewrite_is_formatting_agnostic() {
        // Compact formatting without spaces around the colon.
        let compact = "{\"toolchain\":{\"rust\":\"1.97.1\"},\"rust_toolchain\":\"1.97.1-stable\",\"sdk_version\":\"1.97.1\"}";
        let rewritten = rewrite_json_rust_pins(compact, "1.97.1", "1.98.0");
        assert!(
            rewritten.contains("\"rust\":\"1.98.0\""),
            "compact member must rewrite: {rewritten}"
        );
        assert!(rewritten.contains("\"rust_toolchain\":\"1.98.0-stable\""));
        assert!(
            rewritten.contains("\"sdk_version\":\"1.97.1\""),
            "unrelated fields stay"
        );

        // Whitespace variants around the colon.
        let spaced = "{\n  \"toolchain\" :\n    { \"rust\"\t: \"1.97.1\" },\n  \"rust_toolchain\": \"1.97.1-stable\"\n}\n";
        let rewritten2 = rewrite_json_rust_pins(spaced, "1.97.1", "1.98.0");
        assert!(rewritten2.contains("\"rust\"\t: \"1.98.0\""));
        assert!(rewritten2.contains("\"rust_toolchain\": \"1.98.0-stable\""));
    }

    #[test]
    fn active_text_rewrite_is_rust_image_refs_only_not_blanket_docs() {
        // The reviewer-reported corruption case: a dated LTS snapshot row and a URL ending in
        // Rust-1.97.1/ must survive a bump untouched, while rust: image refs in the same file
        // follow the pin.
        let doc = "| Rust toolchain | 1.97.1 | 2026-05-28 | https://blog.rust-lang.org/2026/05/28/Rust-1.97.1/ |\n\
                   Build stage: `rust:1.97.1-slim-trixie`, `clux/muslrust:1.97.1-stable`\n";
        let rewritten = rewrite_image_refs(doc, "rust:", "1.97.1", "1.98.0");
        assert!(
            rewritten.contains("Rust-1.97.1/"),
            "URLs must not be rewritten"
        );
        assert!(
            rewritten.contains("| Rust toolchain | 1.97.1 | 2026-05-28 |"),
            "dated snapshot rows must not be rewritten"
        );
        assert!(rewritten.contains("rust:1.98.0-slim-trixie"));
        assert!(rewritten.contains("clux/muslrust:1.98.0-stable"));
    }

    #[test]
    fn dependency_policy_row_is_the_only_curated_doc_pin() {
        let doc = "| Rust toolchain | 1.97.1 stable | Debian / distroless base | trixie / static-debian13 |\n\
                   compatible with the current Rust 1.97.1 workspace pin (per §1.1).\n";
        let rewritten = rewrite_dependency_policy_row(doc, "1.97.1", "1.98.0");
        assert!(rewritten.contains("| Rust toolchain | 1.98.0 stable |"));
        assert!(
            rewritten.contains("current Rust 1.97.1 workspace pin"),
            "prose beyond the declared row must stay"
        );
    }

    #[test]
    fn rewrite_is_noop_for_equal_or_absent_versions() {
        assert_eq!(
            rewrite_version_boundary("a 1.97.1 b", "1.97.1", "1.97.1"),
            "a 1.97.1 b"
        );
        assert_eq!(
            rewrite_version_boundary("no version here", "1.97.1", "1.98.0"),
            "no version here"
        );
        assert_eq!(rewrite_version_boundary("", "1.97.1", "1.98.0"), "");
    }

    #[test]
    fn version_comparison_orders_numerically() {
        assert!(latest_is_newer("1.97.1", "1.98.0").expect("compare"));
        assert!(!latest_is_newer("1.98.0", "1.98.0").expect("compare"));
        assert!(!latest_is_newer("1.98.1", "1.98.0").expect("compare"));
        assert!(!latest_is_newer("1.99.0", "1.98.0").expect("compare"));
        assert!(latest_is_newer("1.98.0", "1.98.1").expect("compare"));
    }

    #[test]
    fn parse_stable_version_normalizes_and_fails_closed() {
        assert_eq!(parse_stable_version("1.98.0").expect("parse"), "1.98.0");
        assert_eq!(parse_stable_version("v1.98.0").expect("parse"), "1.98.0");
        assert_eq!(parse_stable_version(" 1.98 ").expect("parse"), "1.98.0");
        assert_eq!(
            parse_stable_version("1.97.1 (8bab26f4f 2026-07-14)").expect("parse"),
            "1.97.1",
            "the raw channel-rust-stable.toml [pkg.rust] version value must parse"
        );
        assert!(parse_stable_version("").is_err());
        assert!(parse_stable_version("stable").is_err());
        assert!(parse_stable_version("1.x.0").is_err());
        assert!(parse_stable_version("1.98.0.1").is_err());
    }

    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    fn fixture_root() -> PathBuf {
        let nonce = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "oya-toolchain-bump-proposer-test-{}-{nonce}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }

    /// Full end-to-end fixture: a minimal repo shaped like the real tree (every surface the
    /// evaluators walk), bumped 1.97.1 -> 1.98.0 via the reconciler, then certified clean against
    /// BOTH the freshness drift evaluator and the ADR-0535 gate.
    #[test]
    fn reconcile_bumps_fixture_tree_deterministically_and_clean() {
        let root = fixture_root();
        write(&root, "specs/root-hub-pointers.json", "{}\n");
        write(
            &root,
            "rust-toolchain.toml",
            "[toolchain]\nchannel = \"1.97.1\"\ncomponents = [\"rustfmt\", \"clippy\"]\nprofile = \"minimal\"\n",
        );
        write(&root, "oya-deps.toml", &oya_deps_fixture("1.97.1"));
        write(
            &root,
            "Cargo.toml",
            "[workspace.package]\nrust-version = \"1.97.1\"\n",
        );
        write(
            &root,
            "Dockerfile.distroless",
            "ARG RUST_VERSION=1.97.1\nFROM rust:${RUST_VERSION}-alpine AS builder\n",
        );
        write(
            &root,
            "toolchains/BUCK",
            "# Rust 1.97.1 toolchain\n# ~/.rustup/toolchains/1.97.1-aarch64-apple-darwin/bin\n",
        );
        write(
            &root,
            ".github/workflows/oya-ci-required.yml",
            "toolchain: \"1.97.1\"\nrustup toolchain install 1.97.1\n",
        );
        write(
            &root,
            "tenancy/manifest.json",
            "{\n  \"toolchain\": { \"rust\": \"1.97.1\" },\n  \"rust_toolchain\": \"1.97.1-stable\"\n}\n",
        );
        // A dated snapshot doc with a URL must survive the bump (regression for the review finding).
        write(
            &root,
            "docs/standards/lts-versions-verified.md",
            "| Rust toolchain | 1.97.1 | 2026-05-28 | https://blog.rust-lang.org/2026/05/28/Rust-1.97.1/ |\n\
             Build stage: `rust:1.97.1-slim-trixie`\n",
        );
        write(
            &root,
            "docs/standards/dependency-policy.md",
            "| Rust toolchain | 1.97.1 stable | Debian / distroless base | trixie |\n",
        );
        write(&root, "deny.toml", "[licenses]\n");
        write(&root, "specs/oss-stewardship-registry.json", "{}\n");

        let report = reconcile(&root, "1.98.0").expect("reconcile");
        assert_eq!(report.outcome, ReconcileOutcome::Bumped);
        assert_eq!(report.current, "1.97.1");
        assert_eq!(report.latest, "1.98.0");
        assert!(
            report.changed_files.len() >= 7,
            "expected all pin surfaces reconciled, got {:?}",
            report.changed_files
        );

        assert_eq!(
            read(&root, "rust-toolchain.toml"),
            "[toolchain]\nchannel = \"1.98.0\"\ncomponents = [\"rustfmt\", \"clippy\"]\nprofile = \"minimal\"\n"
        );
        assert!(read(&root, "oya-deps.toml").contains("pin = \"1.98.0\""));
        assert!(read(&root, "Cargo.toml").contains("rust-version = \"1.98.0\""));
        assert!(read(&root, "Dockerfile.distroless").contains("ARG RUST_VERSION=1.98.0"));
        assert!(read(&root, "toolchains/BUCK").contains("# Rust 1.98.0 toolchain"));
        assert!(read(&root, "toolchains/BUCK").contains("1.98.0-aarch64-apple-darwin"));
        assert!(
            read(&root, ".github/workflows/oya-ci-required.yml").contains("toolchain: \"1.98.0\"")
        );
        assert!(read(&root, "tenancy/manifest.json").contains("\"rust\": \"1.98.0\""));
        assert!(
            read(&root, "tenancy/manifest.json").contains("\"rust_toolchain\": \"1.98.0-stable\"")
        );
        assert!(
            read(&root, "docs/standards/lts-versions-verified.md")
                .contains("rust:1.98.0-slim-trixie")
        );
        // The dated row and URL stay intact.
        assert!(read(&root, "docs/standards/lts-versions-verified.md").contains("Rust-1.97.1/"));
        assert!(
            read(&root, "docs/standards/lts-versions-verified.md")
                .contains("| Rust toolchain | 1.97.1 | 2026-05-28 |")
        );
        // The declared dependency-policy row follows the pin.
        assert!(
            read(&root, "docs/standards/dependency-policy.md")
                .contains("| Rust toolchain | 1.98.0 stable |")
        );

        // The real evaluators must certify the tree clean.
        let residual = verify_clean(&root).expect("verify");
        assert!(
            residual.is_clean(),
            "fixture tree must be clean after reconcile: {:#?}",
            residual
        );

        // Idempotence: a second reconcile reports up-to-date.
        let second = reconcile(&root, "1.98.0").expect("reconcile again");
        assert_eq!(second.outcome, ReconcileOutcome::UpToDate);
        assert!(second.changed_files.is_empty());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn reconcile_rejects_older_version_fail_closed() {
        let root = fixture_root();
        write(&root, "specs/root-hub-pointers.json", "{}\n");
        write(
            &root,
            "rust-toolchain.toml",
            "[toolchain]\nchannel = \"1.97.1\"\n",
        );
        write(&root, "oya-deps.toml", &oya_deps_fixture("1.97.1"));
        write(
            &root,
            "Cargo.toml",
            "[workspace.package]\nrust-version = \"1.97.1\"\n",
        );
        write(&root, "Dockerfile.distroless", "ARG RUST_VERSION=1.97.1\n");
        write(&root, "toolchains/BUCK", "# Rust 1.97.1 toolchain\n");
        write(
            &root,
            "docs/standards/dependency-policy.md",
            "| Rust toolchain | 1.97.1 stable |\n",
        );
        write(&root, "deny.toml", "[licenses]\n");
        write(&root, "specs/oss-stewardship-registry.json", "{}\n");

        let error = reconcile(&root, "1.96.0").expect_err("older latest must fail closed");
        match error {
            ProposerError::VersionOlder { current, supplied } => {
                assert_eq!(current, "1.97.1");
                assert_eq!(supplied, "1.96.0");
            }
            other => panic!("expected VersionOlder, got {other}"),
        }
        // The tree must be untouched.
        assert!(read(&root, "rust-toolchain.toml").contains("1.97.1"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn reconcile_equal_pin_with_stale_tree_fails_closed() {
        let root = fixture_root();
        write(&root, "specs/root-hub-pointers.json", "{}\n");
        // Channel already at 1.98.0, but Cargo.toml rust-version was not updated (partial update).
        write(
            &root,
            "rust-toolchain.toml",
            "[toolchain]\nchannel = \"1.98.0\"\n",
        );
        write(&root, "oya-deps.toml", &oya_deps_fixture("1.98.0"));
        write(
            &root,
            "Cargo.toml",
            "[workspace.package]\nrust-version = \"1.97.1\"\n",
        );
        write(&root, "Dockerfile.distroless", "ARG RUST_VERSION=1.98.0\n");
        write(&root, "toolchains/BUCK", "# Rust 1.98.0 toolchain\n");
        write(
            &root,
            "docs/standards/dependency-policy.md",
            "| Rust toolchain | 1.98.0 stable |\n",
        );
        write(&root, "deny.toml", "[licenses]\n");
        write(&root, "specs/oss-stewardship-registry.json", "{}\n");

        let error = reconcile(&root, "1.98.0").expect_err("equal pin with drift must fail closed");
        match error {
            ProposerError::ResidualDrift(residual) => {
                assert!(
                    !residual.is_clean(),
                    "residual must carry the drift findings"
                );
            }
            other => panic!("expected ResidualDrift, got {other}"),
        }

        let _ = fs::remove_dir_all(&root);
    }

    fn oya_deps_fixture(pin: &str) -> String {
        format!(
            r#"schema_version = "1.0.0"

[metadata]
purpose = "fixture"
owner = "cloud-ci-platform"
decision = "ADR-0535"
status = "accepted"

[automation]
engine = "owned-rust-bump-bot"
changeset_transport = "scm-facts"
github_actions = "adapter-only"
external_bots = "disabled"
merge_authority = "oya-ci-required"

[rust]
channel = "stable"
pin = "{pin}"
update_policy = "latest-stable"
drift_guard = "ci/facade/generated-artifact-freshness/src/rust_toolchain_drift.rs"
exclusions = ["cloud/cloud-kernel/"]

[supply_chain]
license_policy = "deny.toml"
advisory_policy = "cargo-deny"
audit_policy = "cargo-vet"
stewardship_registry = "specs/oss-stewardship-registry.json"
bot_gate = "cloud-ci-dependency-automation"

[[managed_file]]
path = "rust-toolchain.toml"
role = "rust-toolchain-pin"
update = "sync-rust-pin"
reason = "fixture"

[[managed_file]]
path = "Cargo.toml"
role = "workspace-msrv"
update = "sync-rust-pin"
reason = "fixture"

[[managed_file]]
path = "Dockerfile.distroless"
role = "container-builder-toolchain"
update = "sync-rust-pin"
reason = "fixture"

[[managed_file]]
path = "toolchains/BUCK"
role = "buck2-toolchain-comment"
update = "sync-rust-pin"
reason = "fixture"
"#
        )
    }
}
