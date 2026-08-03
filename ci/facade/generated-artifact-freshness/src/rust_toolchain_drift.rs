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

/// The hermetic Rust toolchain pin table (ADR-0392 buck2 canonical build graph). Its URLs are the
/// ONLY place the channel is spelled outside `rust-toolchain.toml`, so this gate is what keeps the
/// content-addressed compiler pinned to the version SSOT.
const RUST_PINS_PATH: &str = "toolchains/rust/pins.bzl";

const RUST_DIST_URL_PREFIX: &str = "https://static.rust-lang.org/dist/";

const RUST_DIST_ARCHIVE_SUFFIX: &str = ".tar.xz";

/// A hermetic toolchain needs ALL FOUR components: rustc alone cannot find std, and a composed
/// tree without the clippy/rustfmt binaries silently loses the lint and format drivers.
const RUST_PIN_COMPONENTS: [&str; 4] = ["clippy", "rust-std", "rustc", "rustfmt"];

/// Both arm64 and x86_64 stay first-class on both Unix platforms — 4 components x 4 triples = 16
/// cells. A short table (a prior design supplied 12) must RED: a host with no cell would otherwise
/// have to fall back to an ambient compiler.
const RUST_PIN_TRIPLES: [&str; 4] = [
    "aarch64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
];

const ACTIVE_TEXT_PATHS: [&str; 8] = [
    "docs/PRD-OYATIE-FROM-SCRATCH-CANONICAL.md",
    "docs/architecture/",
    "docs/automation/",
    "docs/decisions/ADR-0392-buck2-canonical-build-graph.md",
    "docs/plans/",
    "docs/standards/",
    "specs/oss-stewardship-registry.json",
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
        } else if rel.starts_with(".github/workflows/") || rel.starts_with("toolchains/") {
            let text = read_to_string(repo_root, &rel)?;
            check_ci_text(&rel, &text, &want, &mut findings);
            if rel == RUST_PINS_PATH {
                check_rust_pins(&rel, &text, &want, &mut findings);
            }
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
    if rel.starts_with("toolchains/") {
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

/// Validate the hermetic toolchain pin table: every archive URL must carry the
/// `rust-toolchain.toml` channel, and all 16 cells (4 components x 4 triples) must be present with
/// their own digest. A short table would leave a supported host with no pinned compiler.
fn check_rust_pins(rel: &str, text: &str, want: &str, findings: &mut BTreeSet<Finding>) {
    let mut cells: BTreeSet<(&str, &str)> = BTreeSet::new();
    let mut urls = 0usize;

    for after_prefix in text.split(RUST_DIST_URL_PREFIX).skip(1) {
        urls += 1;
        let Some((path, _)) = after_prefix.split_once(RUST_DIST_ARCHIVE_SUFFIX) else {
            findings.insert(drift_finding(
                rel,
                format!("pinned dist URL does not name a {RUST_DIST_ARCHIVE_SUFFIX} archive"),
            ));
            continue;
        };
        let archive = path.rsplit('/').next().unwrap_or(path);
        let Some((component, rest)) = RUST_PIN_COMPONENTS.iter().find_map(|component| {
            archive
                .strip_prefix(&format!("{component}-"))
                .map(|rest| (*component, rest))
        }) else {
            findings.insert(drift_finding(
                rel,
                format!("pinned archive {archive} names no known Rust component"),
            ));
            continue;
        };
        let Some((version, triple)) = rest.split_once('-') else {
            findings.insert(drift_finding(
                rel,
                format!("pinned archive {archive} is not <component>-<version>-<triple>"),
            ));
            continue;
        };
        if version != want {
            findings.insert(drift_finding(
                rel,
                format!("pinned archive {archive} pins {version}, want {want}"),
            ));
        }
        let Some(triple) = RUST_PIN_TRIPLES.iter().find(|known| **known == triple) else {
            findings.insert(drift_finding(
                rel,
                format!("pinned archive {archive} targets unsupported triple {triple}"),
            ));
            continue;
        };
        cells.insert((component, triple));
    }

    for component in RUST_PIN_COMPONENTS {
        for triple in RUST_PIN_TRIPLES {
            if !cells.contains(&(component, triple)) {
                findings.insert(drift_finding(
                    rel,
                    format!("missing hermetic toolchain pin cell {component}/{triple}"),
                ));
            }
        }
    }

    // Every cell pins distinct bytes, so a duplicated or dropped digest means one cell is either
    // unverified or pinned to another cell's archive.
    let digests = distinct_sha256_literals(text);
    if digests != urls {
        findings.insert(drift_finding(
            rel,
            format!("{urls} pinned archive URLs but {digests} distinct sha256 digests"),
        ));
    }
}

fn distinct_sha256_literals(text: &str) -> usize {
    let bytes = text.as_bytes();
    let mut literals = BTreeSet::new();
    let mut start = 0;
    for index in 0..=bytes.len() {
        let hex = index < bytes.len()
            && bytes[index].is_ascii_hexdigit()
            && !bytes[index].is_ascii_uppercase();
        if hex {
            continue;
        }
        if index - start == 64 {
            literals.insert(&text[start..index]);
        }
        start = index + 1;
    }
    literals.len()
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

    /// Renders a pin table with one distinct digest per cell, the shape `toolchains/rust/pins.bzl`
    /// has.
    fn pins_fixture(version: &str, triples: &[&str]) -> String {
        let mut text = String::new();
        let mut digest = 0u32;
        for component in RUST_PIN_COMPONENTS {
            for triple in triples {
                digest += 1;
                text.push_str(&format!(
                    "(\"{RUST_DIST_URL_PREFIX}2026-07-16/{component}-{version}-{triple}{RUST_DIST_ARCHIVE_SUFFIX}\", \"{digest:064x}\"),\n"
                ));
            }
        }
        text
    }

    #[test]
    fn rust_pins_accept_all_sixteen_cells_on_the_canonical_channel() {
        let mut findings = BTreeSet::new();
        check_rust_pins(
            RUST_PINS_PATH,
            &pins_fixture("1.97.1", &RUST_PIN_TRIPLES),
            "1.97.1",
            &mut findings,
        );

        assert!(findings.is_empty(), "{findings:#?}");
    }

    #[test]
    fn rust_pins_reject_a_short_table() {
        let mut findings = BTreeSet::new();
        check_rust_pins(
            RUST_PINS_PATH,
            &pins_fixture(
                "1.97.1",
                &[
                    "aarch64-apple-darwin",
                    "aarch64-unknown-linux-gnu",
                    "x86_64-unknown-linux-gnu",
                ],
            ),
            "1.97.1",
            &mut findings,
        );

        assert_eq!(findings.len(), 4, "{findings:#?}");
        assert!(findings.iter().all(|finding| {
            finding.code == FindingCode::RustToolchainDrift
                && finding
                    .detail
                    .contains("missing hermetic toolchain pin cell")
                && finding.detail.ends_with("/x86_64-apple-darwin")
        }));
    }

    #[test]
    fn rust_pins_reject_a_channel_that_diverges_from_the_version_ssot() {
        let mut findings = BTreeSet::new();
        check_rust_pins(
            RUST_PINS_PATH,
            &pins_fixture("1.96.0", &RUST_PIN_TRIPLES),
            "1.97.1",
            &mut findings,
        );

        assert_eq!(findings.len(), 16, "{findings:#?}");
        assert!(
            findings
                .iter()
                .all(|finding| finding.detail.contains("pins 1.96.0, want 1.97.1"))
        );
    }

    #[test]
    fn rust_pins_reject_a_duplicated_digest() {
        let table = pins_fixture("1.97.1", &RUST_PIN_TRIPLES);
        let duplicated = table.replace(&format!("{:064x}", 2), &format!("{:064x}", 1));
        let mut findings = BTreeSet::new();
        check_rust_pins(RUST_PINS_PATH, &duplicated, "1.97.1", &mut findings);

        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings
                .iter()
                .any(|finding| finding
                    .detail
                    .contains("16 pinned archive URLs but 15 distinct sha256 digests"))
        );
    }

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
