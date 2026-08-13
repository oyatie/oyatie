//! ADR-0535 dependency-automation gate and canonical Reindeer overlay capability.
//!
//! This gate keeps dependency updates on the owned cloud-ci path instead of reintroducing a
//! third-party bot config. The policy source is root `oya-deps.toml`: a small closed-schema DATA
//! contract consumed by the future in-house Rust bump-bot. The gate is deliberately filesystem-only
//! and VCS-free so it runs the same way under GitHub Actions today and the owned runner later.
//! The sibling overlay module is a local, fail-closed generator bridge invoked by the canonical
//! `scripts/ci/regen-third-party.sh`; it does not carry merge authority.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::Path;

use toml::Value;

mod third_party_overlay;

pub use third_party_overlay::{
    ThirdPartyOverlay, ThirdPartyOverlayError, apply_third_party_buck_overlay,
    apply_third_party_buck_overlay_file,
};

pub const GATE_ID: &str = "cloud-ci-dependency-automation";
pub const CONFIG_PATH: &str = "oya-deps.toml";
const EXPECTED_SCHEMA_VERSION: &str = "1.0.0";
const EXPECTED_ENGINE: &str = "owned-rust-bump-bot";
const EXPECTED_CHANGESET_TRANSPORT: &str = "scm-facts";
const EXPECTED_EXTERNAL_BOTS: &str = "disabled";
const EXPECTED_RUST_POLICY: &str = "latest-stable";

const EXTERNAL_BOT_CONFIGS: [&str; 12] = [
    "renovate.json",
    "renovate.json5",
    ".renovaterc",
    ".renovaterc.json",
    ".renovaterc.json5",
    ".renovaterc.yml",
    ".renovaterc.yaml",
    ".renovaterc.js",
    ".github/renovate.json",
    ".github/renovate.json5",
    ".github/dependabot.yml",
    ".github/dependabot.yaml",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: &'static str,
    pub path: String,
    pub detail: String,
}

impl Finding {
    fn new(code: &'static str, path: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code,
            path: path.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateError {
    Io(String),
}

impl std::fmt::Display for GateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateError::Io(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for GateError {}

#[derive(Debug, Clone)]
pub struct GateReport {
    pub verdict: Verdict,
    pub findings: BTreeSet<Finding>,
}

pub fn evaluate_repo(root: &Path) -> Result<GateReport, GateError> {
    let mut findings = BTreeSet::new();
    reject_external_bot_configs(root, &mut findings);

    let config_path = root.join(CONFIG_PATH);
    let text = match fs::read_to_string(&config_path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            findings.insert(Finding::new(
                "DEP-AUTO-MISSING-CONFIG",
                CONFIG_PATH,
                "root oya-deps.toml is required by ADR-0535",
            ));
            return Ok(report(findings));
        }
        Err(error) => {
            return Err(GateError::Io(format!(
                "read {}: {error}",
                config_path.display()
            )));
        }
    };

    let config = match text.parse::<Value>() {
        Ok(value) => value,
        Err(error) => {
            findings.insert(Finding::new(
                "DEP-AUTO-MALFORMED-CONFIG",
                CONFIG_PATH,
                format!("parse TOML: {error}"),
            ));
            return Ok(report(findings));
        }
    };

    validate_closed_schema(&config, &mut findings);
    validate_policy_values(&config, &mut findings);
    validate_managed_files(root, &config, &mut findings);
    validate_rust_pin_alignment(root, &config, &mut findings)?;

    Ok(report(findings))
}

fn report(findings: BTreeSet<Finding>) -> GateReport {
    GateReport {
        verdict: if findings.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        },
        findings,
    }
}

fn reject_external_bot_configs(root: &Path, findings: &mut BTreeSet<Finding>) {
    for rel in EXTERNAL_BOT_CONFIGS {
        if root.join(rel).exists() {
            findings.insert(Finding::new(
                "DEP-AUTO-EXTERNAL-BOT-CONFIG",
                rel,
                "ADR-0535 rejects Renovate/Dependabot adoption; use owned oya-deps.toml + Rust bump-bot",
            ));
        }
    }
}

fn validate_closed_schema(config: &Value, findings: &mut BTreeSet<Finding>) {
    let Some(root) = config.as_table() else {
        findings.insert(Finding::new(
            "DEP-AUTO-MALFORMED-CONFIG",
            CONFIG_PATH,
            "top-level TOML value must be a table",
        ));
        return;
    };

    check_keys(
        "",
        root.keys(),
        [
            "schema_version",
            "metadata",
            "automation",
            "rust",
            "supply_chain",
            "managed_file",
        ],
        findings,
    );
    check_table(
        config,
        &["metadata"],
        ["purpose", "owner", "decision", "status"],
        findings,
    );
    check_table(
        config,
        &["automation"],
        [
            "engine",
            "changeset_transport",
            "github_actions",
            "external_bots",
            "merge_authority",
        ],
        findings,
    );
    check_table(
        config,
        &["rust"],
        [
            "channel",
            "pin",
            "update_policy",
            "drift_guard",
            "exclusions",
        ],
        findings,
    );
    check_table(
        config,
        &["supply_chain"],
        [
            "license_policy",
            "advisory_policy",
            "audit_policy",
            "stewardship_registry",
            "bot_gate",
        ],
        findings,
    );

    if let Some(entries) = config.get("managed_file").and_then(Value::as_array) {
        for (idx, entry) in entries.iter().enumerate() {
            if let Some(table) = entry.as_table() {
                check_keys(
                    &format!("managed_file[{idx}]"),
                    table.keys(),
                    ["path", "role", "update", "reason"],
                    findings,
                );
            } else {
                findings.insert(Finding::new(
                    "DEP-AUTO-MALFORMED-CONFIG",
                    format!("{CONFIG_PATH}:managed_file[{idx}]"),
                    "managed_file entries must be TOML tables",
                ));
            }
        }
    }
}

fn check_table<const N: usize>(
    config: &Value,
    path: &[&str],
    allowed: [&'static str; N],
    findings: &mut BTreeSet<Finding>,
) {
    match value_at(config, path).and_then(Value::as_table) {
        Some(table) => check_keys(&path.join("."), table.keys(), allowed, findings),
        None => {
            findings.insert(Finding::new(
                "DEP-AUTO-MISSING-KEY",
                format!("{CONFIG_PATH}:{}", path.join(".")),
                "required table is missing or not a table",
            ));
        }
    };
}

fn check_keys<'a, I, const N: usize>(
    scope: &str,
    keys: I,
    allowed: [&'static str; N],
    findings: &mut BTreeSet<Finding>,
) where
    I: Iterator<Item = &'a String>,
{
    let allowed: HashSet<&str> = allowed.into_iter().collect();
    for key in keys {
        if !allowed.contains(key.as_str()) {
            let key_path = if scope.is_empty() {
                key.to_owned()
            } else {
                format!("{scope}.{key}")
            };
            findings.insert(Finding::new(
                "DEP-AUTO-UNKNOWN-KEY",
                format!("{CONFIG_PATH}:{key_path}"),
                "oya-deps.toml is a closed-schema contract; add schema support before adding keys",
            ));
        }
    }
}

fn validate_policy_values(config: &Value, findings: &mut BTreeSet<Finding>) {
    expect_string(
        config,
        &["schema_version"],
        EXPECTED_SCHEMA_VERSION,
        "DEP-AUTO-SCHEMA-VERSION",
        "schema_version must match the gate contract",
        findings,
    );
    expect_string(
        config,
        &["metadata", "decision"],
        "ADR-0535",
        "DEP-AUTO-MISSING-ADR",
        "owned dependency automation must cite ADR-0535",
        findings,
    );
    expect_string(
        config,
        &["automation", "engine"],
        EXPECTED_ENGINE,
        "DEP-AUTO-NONOWNED-ENGINE",
        "dependency automation must use the owned Rust bump-bot engine",
        findings,
    );
    expect_string(
        config,
        &["automation", "changeset_transport"],
        EXPECTED_CHANGESET_TRANSPORT,
        "DEP-AUTO-NONOWNED-TRANSPORT",
        "dependency automation must emit provider-neutral scm-facts ChangeSets",
        findings,
    );
    expect_string(
        config,
        &["automation", "external_bots"],
        EXPECTED_EXTERNAL_BOTS,
        "DEP-AUTO-EXTERNAL-BOTS-ENABLED",
        "external dependency bots stay disabled for the owned stack",
        findings,
    );
    expect_string(
        config,
        &["automation", "merge_authority"],
        "oya-ci-required",
        "DEP-AUTO-MERGE-AUTHORITY",
        "dependency updates must still merge through the single required context",
        findings,
    );
    expect_string(
        config,
        &["rust", "channel"],
        "stable",
        "DEP-AUTO-RUST-CHANNEL",
        "root workspace follows the stable Rust channel",
        findings,
    );
    expect_string(
        config,
        &["rust", "update_policy"],
        EXPECTED_RUST_POLICY,
        "DEP-AUTO-RUST-UPDATE-POLICY",
        "Rust updates should track the latest stable release",
        findings,
    );
    expect_string(
        config,
        &["supply_chain", "bot_gate"],
        GATE_ID,
        "DEP-AUTO-BOT-GATE",
        "supply-chain policy must name this enforcement gate",
        findings,
    );
}

fn expect_string(
    config: &Value,
    path: &[&str],
    expected: &str,
    code: &'static str,
    detail: &str,
    findings: &mut BTreeSet<Finding>,
) {
    match string_at(config, path) {
        Some(actual) if actual == expected => {}
        Some(actual) => {
            findings.insert(Finding::new(
                code,
                format!("{CONFIG_PATH}:{}", path.join(".")),
                format!("{detail}: expected {expected:?}, got {actual:?}"),
            ));
        }
        None => {
            findings.insert(Finding::new(
                "DEP-AUTO-MISSING-KEY",
                format!("{CONFIG_PATH}:{}", path.join(".")),
                format!("missing required string; {detail}"),
            ));
        }
    }
}

fn validate_managed_files(root: &Path, config: &Value, findings: &mut BTreeSet<Finding>) {
    let Some(entries) = config.get("managed_file").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "DEP-AUTO-MISSING-KEY",
            format!("{CONFIG_PATH}:managed_file"),
            "at least one managed_file entry is required",
        ));
        return;
    };
    if entries.is_empty() {
        findings.insert(Finding::new(
            "DEP-AUTO-MISSING-KEY",
            format!("{CONFIG_PATH}:managed_file"),
            "managed_file must not be empty",
        ));
        return;
    }

    let mut seen = HashSet::new();
    for (idx, entry) in entries.iter().enumerate() {
        let Some(path) = entry.get("path").and_then(Value::as_str) else {
            findings.insert(Finding::new(
                "DEP-AUTO-MISSING-KEY",
                format!("{CONFIG_PATH}:managed_file[{idx}].path"),
                "managed_file entries require a path",
            ));
            continue;
        };
        if path.starts_with('/') || path.contains("..") {
            findings.insert(Finding::new(
                "DEP-AUTO-BAD-MANAGED-PATH",
                format!("{CONFIG_PATH}:managed_file[{idx}].path"),
                "managed paths must be repo-relative and must not contain '..'",
            ));
            continue;
        }
        if !seen.insert(path.to_owned()) {
            findings.insert(Finding::new(
                "DEP-AUTO-DUPLICATE-MANAGED-PATH",
                path,
                "managed_file paths must be unique",
            ));
        }
        if !root.join(path).is_file() {
            findings.insert(Finding::new(
                "DEP-AUTO-MISSING-MANAGED-FILE",
                path,
                "managed_file path does not exist in the candidate tree",
            ));
        }
    }

    for required in ["rust-toolchain.toml", "Cargo.toml", "Dockerfile.distroless"] {
        if !seen.contains(required) {
            findings.insert(Finding::new(
                "DEP-AUTO-MISSING-MANAGED-FILE",
                required,
                "required Rust pin surface is not listed in managed_file",
            ));
        }
    }
}

fn validate_rust_pin_alignment(
    root: &Path,
    config: &Value,
    findings: &mut BTreeSet<Finding>,
) -> Result<(), GateError> {
    let Some(pin) = string_at(config, &["rust", "pin"]) else {
        findings.insert(Finding::new(
            "DEP-AUTO-MISSING-KEY",
            format!("{CONFIG_PATH}:rust.pin"),
            "Rust pin is required",
        ));
        return Ok(());
    };
    if !looks_like_semver(pin) {
        findings.insert(Finding::new(
            "DEP-AUTO-RUST-PIN-FORMAT",
            format!("{CONFIG_PATH}:rust.pin"),
            "Rust pin must be a full stable semver like 1.96.0",
        ));
    }

    let toolchain = read_required(root, "rust-toolchain.toml")?;
    let channel = toolchain
        .parse::<Value>()
        .ok()
        .and_then(|v| string_at(&v, &["toolchain", "channel"]).map(str::to_owned));
    if channel.as_deref() != Some(pin) {
        findings.insert(Finding::new(
            "DEP-AUTO-RUST-PIN-DRIFT",
            "rust-toolchain.toml",
            format!("toolchain.channel must equal oya-deps rust.pin {pin}"),
        ));
    }

    expect_file_contains(
        root,
        "Cargo.toml",
        &format!("rust-version = \"{pin}\""),
        findings,
    )?;
    expect_file_contains(
        root,
        "Dockerfile.distroless",
        &format!("ARG RUST_VERSION={pin}"),
        findings,
    )?;
    expect_file_contains(root, "build/toolchains/BUCK", pin, findings)?;
    Ok(())
}

fn expect_file_contains(
    root: &Path,
    rel: &str,
    needle: &str,
    findings: &mut BTreeSet<Finding>,
) -> Result<(), GateError> {
    let text = read_required(root, rel)?;
    if !text.contains(needle) {
        findings.insert(Finding::new(
            "DEP-AUTO-RUST-PIN-DRIFT",
            rel,
            format!("expected to contain {needle:?}"),
        ));
    }
    Ok(())
}

fn read_required(root: &Path, rel: &str) -> Result<String, GateError> {
    let path = root.join(rel);
    fs::read_to_string(&path).map_err(|e| GateError::Io(format!("read {}: {e}", path.display())))
}

fn string_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    value_at(value, path).and_then(Value::as_str)
}

fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}

fn looks_like_semver(value: &str) -> bool {
    let parts: Vec<&str> = value.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

pub fn render_findings(report: &GateReport) -> String {
    if report.findings.is_empty() {
        return format!("{GATE_ID}: GREEN — oya-deps.toml owned updater contract is valid");
    }
    let mut out = format!(
        "{GATE_ID}: RED — {} dependency automation finding(s)",
        report.findings.len()
    );
    for finding in &report.findings {
        out.push_str(&format!(
            "\n{} {}: {}",
            finding.code, finding.path, finding.detail
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_format_requires_three_numeric_parts() {
        assert!(looks_like_semver("1.96.0"));
        assert!(!looks_like_semver("1.96"));
        assert!(!looks_like_semver("nightly-2026-02-28"));
        assert!(!looks_like_semver("1.96.x"));
    }

    #[test]
    fn render_green_is_compact() {
        let report = report(BTreeSet::new());
        assert_eq!(report.verdict, Verdict::Green);
        assert!(render_findings(&report).contains("GREEN"));
    }

    #[test]
    fn closed_schema_rejects_unknown_top_level_key() {
        let config = r#"
schema_version = "1.0.0"
unexpected = true
[metadata]
purpose = "x"
owner = "x"
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
pin = "1.96.0"
update_policy = "latest-stable"
drift_guard = "x"
exclusions = []
[supply_chain]
license_policy = "deny.toml"
advisory_policy = "cargo-deny"
audit_policy = "cargo-vet"
stewardship_registry = "specs/oss-stewardship-registry.json"
bot_gate = "cloud-ci-dependency-automation"
"#
        .parse::<Value>()
        .unwrap();
        let mut findings = BTreeSet::new();
        validate_closed_schema(&config, &mut findings);
        assert!(
            findings
                .iter()
                .any(|finding| finding.code == "DEP-AUTO-UNKNOWN-KEY")
        );
    }
}
