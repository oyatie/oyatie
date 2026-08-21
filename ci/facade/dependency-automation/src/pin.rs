use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use toml::Value;

use crate::CONFIG_PATH;
use crate::report::{Finding, GateError};
use crate::schema::string_at;

pub(crate) fn validate_rust_pin_alignment(
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

fn looks_like_semver(value: &str) -> bool {
    let parts: Vec<&str> = value.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
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
}
