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

mod bots;
mod paths;
mod pin;
mod policy;
mod report;
mod schema;
mod third_party_overlay;

pub use report::{Finding, GateError, GateReport, Verdict, render_findings};
pub use third_party_overlay::{
    ThirdPartyOverlay, ThirdPartyOverlayError, apply_third_party_buck_overlay,
    apply_third_party_buck_overlay_file,
};

pub const GATE_ID: &str = "cloud-ci-dependency-automation";
pub const CONFIG_PATH: &str = "oya-deps.toml";

pub fn evaluate_repo(root: &Path) -> Result<GateReport, GateError> {
    let mut findings = BTreeSet::new();
    bots::reject_external_bot_configs(root, &mut findings);

    let config_path = root.join(CONFIG_PATH);
    let text = match fs::read_to_string(&config_path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            findings.insert(Finding::new(
                "DEP-AUTO-MISSING-CONFIG",
                CONFIG_PATH,
                "root oya-deps.toml is required by ADR-0535",
            ));
            return Ok(report::report(findings));
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
            return Ok(report::report(findings));
        }
    };

    schema::validate_closed_schema(&config, &mut findings);
    policy::validate_policy_values(&config, &mut findings);
    paths::validate_managed_files(root, &config, &mut findings);
    paths::validate_declared_paths(root, &config, &mut findings);
    pin::validate_rust_pin_alignment(root, &config, &mut findings)?;

    Ok(report::report(findings))
}
