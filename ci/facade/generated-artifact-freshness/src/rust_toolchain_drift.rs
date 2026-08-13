//! Rust toolchain pin drift checks for the cloud-ci freshness gate.
//!
//! This is intentionally a Rust API behind the existing freshness gate binary,
//! not a Python/shell script. The repo-specific scope stays small and explicit:
//! canonical Rust pins must follow `rust-toolchain.toml` across manifests,
//! Dockerfiles, CI surfaces/toolchains, and active standards/spec text.

use std::collections::{BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

use crate::{Finding, FindingCode, FreshnessError};

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

const ACTIVE_TEXT_PATHS: [&str; 9] = [
    "docs/PRD-OYATIE-FROM-SCRATCH-CANONICAL.md",
    "docs/architecture/",
    "docs/automation/",
    "docs/decisions/ADR-0700-ci-admission-live-apex.md",
    "docs/plans/",
    "docs/standards/",
    "specs/oss-stewardship-registry.json",
    "build/toolchains/",
    "toolchains/",
];

pub fn read_pinned_rust_toolchain(repo_root: &Path) -> Result<String, FreshnessError> {
    let toolchain = parse_toml_file(&repo_root.join("rust-toolchain.toml"))?;
    toolchain
        .get("toolchain")
        .and_then(|toolchain| toolchain.get("channel"))
        .and_then(TomlValue::as_str)
        .map(str::to_owned)
        .ok_or_else(|| FreshnessError::new("rust-toolchain.toml missing toolchain.channel string"))
}

pub fn evaluate_rust_toolchain_drift(repo_root: &Path) -> Result<Vec<Finding>, FreshnessError> {
    let toolchain_path = repo_root.join("rust-toolchain.toml");
    if !toolchain_path.is_file() {
        if repo_root.join("specs/root-hub-pointers.json").is_file() {
            return Ok(vec![drift_finding(
                "rust-toolchain.toml",
                "missing canonical Rust toolchain pin",
            )]);
        }
        return Ok(Vec::new());
    }

    let want = read_pinned_rust_toolchain(repo_root)?;
    let mut findings = BTreeSet::new();

    for rel in rust_toolchain_drift_candidate_paths(repo_root)? {
        if rel == "Cargo.toml" || rel.ends_with("/Cargo.toml") {
            check_cargo_manifest(repo_root, &rel, &want, &mut findings)?;
        } else if rel.ends_with("manifest.json") || rel.ends_with("supported-oses.json") {
            check_json_manifest(repo_root, &rel, &want, &mut findings)?;
        } else if is_dockerfile_path(&rel) {
            let text = read_to_string(repo_root, &rel)?;
            check_docker_text(&rel, &text, &want, &mut findings);
        } else if rel.starts_with(".github/workflows/") || rel.starts_with("build/toolchains/") || rel.starts_with("toolchains/") {
            let text = read_to_string(repo_root, &rel)?;
            check_ci_text(&rel, &text, &want, &mut findings);
        }

        if active_text_path(&rel) {
            let text = read_to_string(repo_root, &rel)?;
            check_active_text(&rel, &text, &want, &mut findings);
        }
    }

    Ok(findings.into_iter().collect())
}

fn rust_toolchain_drift_candidate_paths(repo_root: &Path) -> Result<Vec<String>, FreshnessError> {
    let mut paths = Vec::new();
    let mut queue = VecDeque::from([repo_root.to_path_buf()]);

    while let Some(dir) = queue.pop_front() {
        for path in sorted_dir_entries(&dir)? {
            let rel = path
                .strip_prefix(repo_root)
                .map_err(|error| {
                    FreshnessError::new(format!("strip repo root from {}: {error}", path.display()))
                })?
                .to_string_lossy()
                .replace('\\', "/");
            if excluded_path(&rel) {
                continue;
            }
            let metadata = fs::symlink_metadata(&path).map_err(|error| {
                FreshnessError::new(format!(
                    "symlink_metadata {} during Rust toolchain drift scan: {error}",
                    path.display()
                ))
            })?;
            if metadata.file_type().is_symlink() {
                continue;
            }
            if metadata.is_dir() {
                queue.push_back(path);
                continue;
            }
            if metadata.is_file()
                && !rel.ends_with(".generated.json")
                && relevant_to_rust_toolchain_drift(&rel)
            {
                paths.push(rel);
            }
        }
    }

    paths.sort();
    Ok(paths)
}

fn sorted_dir_entries(dir: &Path) -> Result<Vec<PathBuf>, FreshnessError> {
    let mut entries = fs::read_dir(dir)
        .map_err(|error| FreshnessError::new(format!("read_dir {}: {error}", dir.display())))?
        .map(|entry| {
            entry.map(|entry| entry.path()).map_err(|error| {
                FreshnessError::new(format!("read_dir entry {}: {error}", dir.display()))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    Ok(entries)
}

fn excluded_path(path: &str) -> bool {
    EXCLUDED_PREFIXES
        .iter()
        .any(|prefix| path.starts_with(prefix))
}

fn active_text_path(path: &str) -> bool {
    ACTIVE_TEXT_PATHS
        .iter()
        .any(|prefix| path == *prefix || path.starts_with(prefix))
}

fn relevant_to_rust_toolchain_drift(path: &str) -> bool {
    path == "rust-toolchain.toml"
        || path == "Cargo.toml"
        || path.ends_with("/Cargo.toml")
        || path.ends_with("manifest.json")
        || path.ends_with("supported-oses.json")
        || is_dockerfile_path(path)
        || path.starts_with(".github/workflows/")
        || path.starts_with("build/toolchains/")
        || path.starts_with("toolchains/")
        || active_text_path(path)
}

fn is_dockerfile_path(path: &str) -> bool {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("Dockerfile"))
}

fn parse_toml_file(path: &Path) -> Result<TomlValue, FreshnessError> {
    let text = fs::read_to_string(path)
        .map_err(|error| FreshnessError::new(format!("read toml {}: {error}", path.display())))?;
    text.parse::<TomlValue>()
        .map_err(|error| FreshnessError::new(format!("parse toml {}: {error}", path.display())))
}

fn read_to_string(repo_root: &Path, rel: &str) -> Result<String, FreshnessError> {
    fs::read_to_string(repo_root.join(rel))
        .map_err(|error| FreshnessError::new(format!("read {rel}: {error}")))
}

fn drift_finding(path: &str, detail: impl Into<String>) -> Finding {
    Finding::new(FindingCode::RustToolchainDrift, path, detail)
}

fn check_cargo_manifest(
    repo_root: &Path,
    rel: &str,
    want: &str,
    findings: &mut BTreeSet<Finding>,
) -> Result<(), FreshnessError> {
    let manifest = parse_toml_file(&repo_root.join(rel))?;
    if rel == "Cargo.toml" {
        let workspace = manifest
            .get("workspace")
            .and_then(|workspace| workspace.get("package"))
            .and_then(|package| package.get("rust-version"))
            .and_then(TomlValue::as_str);
        if workspace != Some(want) {
            findings.insert(drift_finding(
                rel,
                format!(
                    "workspace rust-version is {}, want {want}",
                    workspace.unwrap_or("<missing>")
                ),
            ));
        }
    }

    let Some(package) = manifest.get("package") else {
        return Ok(());
    };
    if let Some(rust_version) = package
        .get("rust-version")
        .and_then(TomlValue::as_str)
        .filter(|value| *value != want)
    {
        findings.insert(drift_finding(
            rel,
            format!("package rust-version is {rust_version}, want {want}"),
        ));
    }
    Ok(())
}

fn check_json_manifest(
    repo_root: &Path,
    rel: &str,
    want: &str,
    findings: &mut BTreeSet<Finding>,
) -> Result<(), FreshnessError> {
    let data: JsonValue = serde_json::from_str(&read_to_string(repo_root, rel)?)
        .map_err(|error| FreshnessError::new(format!("parse json {rel}: {error}")))?;

    if let Some(rust) = data
        .get("toolchain")
        .and_then(|toolchain| toolchain.get("rust"))
        .and_then(JsonValue::as_str)
        .filter(|value| *value != want)
    {
        findings.insert(drift_finding(
            rel,
            format!("toolchain.rust is {rust}, want {want}"),
        ));
    }
    if let Some(rust) = data
        .get("lts_pins")
        .and_then(|pins| pins.get("rust"))
        .and_then(JsonValue::as_str)
        .filter(|value| *value != want)
    {
        findings.insert(drift_finding(
            rel,
            format!("lts_pins.rust is {rust}, want {want}"),
        ));
    }
    let want_stable = format!("{want}-stable");
    if let Some(rust) = data
        .get("rust_toolchain")
        .and_then(JsonValue::as_str)
        .filter(|value| *value != want_stable)
    {
        findings.insert(drift_finding(
            rel,
            format!("rust_toolchain is {rust}, want {want_stable}"),
        ));
    }
    Ok(())
}

fn check_docker_text(rel: &str, text: &str, want: &str, findings: &mut BTreeSet<Finding>) {
    for line in text.lines() {
        if let Some(value) = line
            .trim_start()
            .strip_prefix("ARG RUST_VERSION=")
            .and_then(|rest| rest.split_whitespace().next())
            .filter(|value| *value != want)
        {
            findings.insert(drift_finding(
                rel,
                format!("ARG RUST_VERSION={value}, want {want}"),
            ));
        }
        let Some(tag) = line.trim_start().strip_prefix("FROM rust:") else {
            continue;
        };
        let tag = tag.split_whitespace().next().unwrap_or_default();
        let version = leading_version(tag);
        if !version.is_empty() && version != want {
            findings.insert(drift_finding(rel, format!("FROM rust:{tag}, want {want}")));
        }
    }
}

fn check_ci_text(rel: &str, text: &str, want: &str, findings: &mut BTreeSet<Finding>) {
    for line in text.lines() {
        if let Some(toolchain) = ci_surface_toolchain_value(line) {
            let version = leading_version(&toolchain);
            if version.is_empty() {
                findings.insert(drift_finding(
                    rel,
                    format!(
                        "CI surface uses non-canonical Rust toolchain {toolchain}, want {want}"
                    ),
                ));
            } else if version != want {
                findings.insert(drift_finding(
                    rel,
                    format!("CI surface toolchain pins {version}, want {want}"),
                ));
            }
        }
        if let Some(version) =
            version_after(line, ".rustup/toolchains/").filter(|version| version != want)
        {
            findings.insert(drift_finding(
                rel,
                format!("rustup cache path pins {version}, want {want}"),
            ));
        }
        if let Some(version) =
            version_after(line, "rustup toolchain install ").filter(|version| version != want)
        {
            findings.insert(drift_finding(
                rel,
                format!("rustup install pins {version}, want {want}"),
            ));
        }
    }
    if rel.starts_with("build/toolchains/") || rel.starts_with("toolchains/") {
        for version in explicit_rust_versions(text) {
            if version != want {
                findings.insert(drift_finding(
                    rel,
                    format!("toolchain text pins {version}, want {want}"),
                ));
            }
        }
    }
}

fn ci_surface_toolchain_value(line: &str) -> Option<String> {
    let value = line.trim_start().strip_prefix("toolchain:")?.trim();
    let value = value.split('#').next().unwrap_or_default().trim();
    let value = value.trim_matches(['"', '\'']);
    (!value.is_empty()).then(|| value.to_owned())
}

fn check_active_text(rel: &str, text: &str, want: &str, findings: &mut BTreeSet<Finding>) {
    for stale in [
        "1.95.0",
        "rust:1-bookworm",
        "rust:1.95",
        "RUST_VERSION=1.82",
    ] {
        if text.contains(stale) {
            findings.insert(drift_finding(
                rel,
                format!("active text still contains stale Rust pin {stale}"),
            ));
        }
    }

    let mut rest = text;
    while let Some((_, after)) = rest.split_once("rust:") {
        let version = leading_version(after);
        if !version.is_empty() && version != want {
            findings.insert(drift_finding(
                rel,
                format!("Rust image rust:{version} is not patch-pinned to {want}"),
            ));
        }
        rest = after.get(1..).unwrap_or_default();
    }
}

fn version_after(line: &str, needle: &str) -> Option<String> {
    let after = line.split_once(needle)?.1.trim_start();
    let version = leading_version(after);
    (!version.is_empty()).then_some(version)
}

fn leading_version(text: &str) -> String {
    text.chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect()
}

fn explicit_rust_versions(text: &str) -> BTreeSet<String> {
    let bytes = text.as_bytes();
    let mut versions = BTreeSet::new();
    let mut index = 0;
    while index + 2 < bytes.len() {
        if bytes[index] == b'1' && bytes[index + 1] == b'.' && bytes[index + 2].is_ascii_digit() {
            let start = index;
            index += 2;
            while index < bytes.len() && (bytes[index].is_ascii_digit() || bytes[index] == b'.') {
                index += 1;
            }
            versions.insert(String::from_utf8_lossy(&bytes[start..index]).to_string());
            continue;
        }
        index += 1;
    }
    versions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ci_text_rejects_floating_stable_and_stale_rustup_pin() {
        let mut findings = BTreeSet::new();
        check_ci_text(
            ".github/workflows/x.yml",
            "toolchain: stable\n      toolchain: 1.95.0\nrustup toolchain install 1.95.0\n",
            "1.96.0",
            &mut findings,
        );

        assert_eq!(findings.len(), 3, "{findings:#?}");
        assert!(
            findings
                .iter()
                .all(|finding| finding.code == FindingCode::RustToolchainDrift)
        );
        assert!(findings.iter().any(|finding| {
            finding
                .detail
                .contains("non-canonical Rust toolchain stable")
        }));
        assert!(
            findings
                .iter()
                .any(|finding| { finding.detail.contains("CI surface toolchain pins 1.95.0") })
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.detail.contains("rustup install pins 1.95.0"))
        );
    }

    #[test]
    fn ci_text_rejects_quoted_stale_pin_and_named_channels() {
        let mut findings = BTreeSet::new();
        check_ci_text(
            ".github/workflows/x.yml",
            "toolchain: \"1.95.0\"\ntoolchain: 'nightly'\ntoolchain: beta # comment\n",
            "1.96.0",
            &mut findings,
        );

        assert_eq!(findings.len(), 3, "{findings:#?}");
        assert!(
            findings
                .iter()
                .any(|finding| { finding.detail.contains("CI surface toolchain pins 1.95.0") })
        );
        assert!(findings.iter().any(|finding| {
            finding
                .detail
                .contains("non-canonical Rust toolchain nightly")
        }));
        assert!(
            findings
                .iter()
                .any(|finding| finding.detail.contains("non-canonical Rust toolchain beta"))
        );
    }

    #[test]
    fn ci_text_accepts_matching_workflow_action_pin() {
        let mut findings = BTreeSet::new();
        check_ci_text(
            ".github/workflows/x.yml",
            "toolchain: 1.96.0\ntoolchain: \"1.96.0\"\nrustup toolchain install 1.96.0\n",
            "1.96.0",
            &mut findings,
        );

        assert!(findings.is_empty(), "{findings:#?}");
    }

    #[test]
    fn docker_text_requires_exact_patch_tag() {
        let mut findings = BTreeSet::new();
        check_docker_text(
            "Dockerfile",
            "FROM rust:1-bookworm\nARG RUST_VERSION=1.95.0\n",
            "1.96.0",
            &mut findings,
        );

        assert_eq!(findings.len(), 2, "{findings:#?}");
        assert!(
            findings
                .iter()
                .all(|finding| finding.code == FindingCode::RustToolchainDrift)
        );
    }

    #[test]
    fn active_text_rejects_minor_or_stale_rust_image() {
        let mut findings = BTreeSet::new();
        check_active_text(
            "docs/standards/container-image-convention.md",
            "Prefer rust:1.95-slim over rust:1-bookworm? no.",
            "1.96.0",
            &mut findings,
        );

        assert!(!findings.is_empty());
        assert!(
            findings
                .iter()
                .any(|finding| finding.detail.contains("not patch-pinned to 1.96.0"))
        );
    }
}
