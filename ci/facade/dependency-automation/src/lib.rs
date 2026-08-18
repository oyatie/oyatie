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

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use toml::Value;

mod paths;
mod rust_pin;
mod schema;
mod third_party_overlay;

pub use third_party_overlay::{
    ThirdPartyOverlay, ThirdPartyOverlayError, apply_third_party_buck_overlay,
    apply_third_party_buck_overlay_file,
};

pub const GATE_ID: &str = "cloud-ci-dependency-automation";
pub const CONFIG_PATH: &str = "oya-deps.toml";
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
    pub(crate) fn new(
        code: &'static str,
        path: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
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

    schema::validate_closed_schema(&config, &mut findings);
    schema::validate_policy_values(&config, &mut findings);
    paths::validate_managed_files(root, &config, &mut findings);
    paths::validate_declared_paths(root, &config, &mut findings);
    rust_pin::validate_rust_pin_alignment(root, &config, &mut findings)?;

    Ok(report(findings))
}

pub(crate) fn report(findings: BTreeSet<Finding>) -> GateReport {
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

pub(crate) fn string_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    value_at(value, path).and_then(Value::as_str)
}

pub(crate) fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
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
        assert!(rust_pin::looks_like_semver("1.96.0"));
        assert!(!rust_pin::looks_like_semver("1.96"));
        assert!(!rust_pin::looks_like_semver("nightly-2026-02-28"));
        assert!(!rust_pin::looks_like_semver("1.96.x"));
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
        schema::validate_closed_schema(&config, &mut findings);
        assert!(
            findings
                .iter()
                .any(|finding| finding.code == "DEP-AUTO-UNKNOWN-KEY")
        );
    }
}
