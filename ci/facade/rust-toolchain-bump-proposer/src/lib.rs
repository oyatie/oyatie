//! Owned Rust actuator for the ADR-0535 dependency-automation engine: proposes and
//! deterministically applies a stable Rust toolchain pin bump across every drift surface the
//! cloud-ci freshness gate enforces.
//!
//! Design contract (recorded in the PR body):
//! - **Pure planner**: this binary performs NO network I/O. `std` has no HTTP client, and adding
//!   `reqwest`/`std::net` would violate the dependency policy (no ad-hoc dependencies) and the
//!   gate hermeticity scanner (which scans for exactly those tokens). The latest stable version
//!   arrives as a flag or environment value (`--latest-stable <v>` / `OYA_LATEST_STABLE_RUST`);
//!   the scheduled fetch belongs to the workflow step (curl to
//!   `https://static.rust-lang.org/dist/channel-rust-stable.toml`).
//! - **Deterministic editor**: rewrites are boundary-aware text surgery (comment- and
//!   formatting-preserving; a `toml` round-trip would drop comments), applied to the same
//!   candidate surface the drift evaluator walks.
//! - **Self-verifying**: after applying, the proposer re-runs the freshness drift evaluator AND
//!   the ADR-0535 dependency-automation gate against the tree and fails closed on residual
//!   findings.
//! - **Zero new external dependencies**: only owned path crates (`ci-generated-artifact-freshness`
//!   for `read_pinned_rust_toolchain` + `evaluate_rust_toolchain_drift`;
//!   `ci-dependency-automation` for `evaluate_repo`).

#![forbid(unsafe_code)]

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposerError(pub String);

impl std::fmt::Display for ProposerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ProposerError {}

impl From<ci_generated_artifact_freshness::FreshnessError> for ProposerError {
    fn from(error: ci_generated_artifact_freshness::FreshnessError) -> Self {
        Self(format!("freshness: {error}"))
    }
}

impl From<ci_dependency_automation::GateError> for ProposerError {
    fn from(error: ci_dependency_automation::GateError) -> Self {
        Self(format!("dependency-automation gate: {error}"))
    }
}

fn io_error(context: &str, error: std::io::Error) -> ProposerError {
    ProposerError(format!("{context}: {error}"))
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
        return Err(ProposerError(
            "latest stable version is empty; pass --latest-stable <v> or OYA_LATEST_STABLE_RUST"
                .to_owned(),
        ));
    }
    if !trimmed.chars().all(|ch| ch.is_ascii_digit() || ch == '.') {
        return Err(ProposerError(format!(
            "latest stable version {trimmed:?} must contain only digits and dots"
        )));
    }
    let parts: Vec<&str> = trimmed.split('.').collect();
    if !(2..=3).contains(&parts.len()) {
        return Err(ProposerError(format!(
            "latest stable version {trimmed:?} must be a two- or three-part semver"
        )));
    }
    for part in &parts {
        if part.is_empty() || !part.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(ProposerError(format!(
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
                ProposerError(format!(
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
/// Pure text surgery on purpose: a `toml`/`serde_json` round-trip would drop comments and
/// reformat the file, producing noisy bump diffs. Deterministic for a given input.
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
                .map_err(|error| {
                    ProposerError(format!("strip repo root from {}: {error}", path.display()))
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
        let rewritten = rewrite_version_boundary(&text, old, new);
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
        let rewritten = rewrite_version_boundary(&text, &plan.old, &plan.new);
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

/// Verify the tree against BOTH enforcement surfaces after a bump: the freshness rust-toolchain
/// drift evaluator and the ADR-0535 dependency-automation gate. Fails closed: any finding is
/// surfaced, never guessed around.
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

#[cfg(test)]
mod tests {
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
    fn boundary_rewrite_updates_ci_surface_shapes() {
        let workflow = r#"toolchain: "1.97.1"
      toolchain: 1.97.1
rustup toolchain install 1.97.1
--toolchain 1.97.1-$host
~/.rustup/toolchains/1.97.1-aarch64-apple-darwin/bin
"#;
        let rewritten = rewrite_version_boundary(workflow, "1.97.1", "1.98.0");
        assert!(rewritten.contains("toolchain: \"1.98.0\""));
        assert!(rewritten.contains("toolchain: 1.98.0"));
        assert!(rewritten.contains("rustup toolchain install 1.98.0"));
        assert!(rewritten.contains("--toolchain 1.98.0-$host"));
        assert!(rewritten.contains("~/.rustup/toolchains/1.98.0-aarch64-apple-darwin/bin"));
    }

    #[test]
    fn boundary_rewrite_updates_toml_docker_and_manifest_shapes() {
        let text = r#"[toolchain]
channel = "1.97.1"
[rust]
pin = "1.97.1"
ARG RUST_VERSION=1.97.1
FROM rust:1.97.1-bookworm AS builder
"toolchain": "1.97.1"
"rust_toolchain": "1.97.1-stable"
"#;
        let rewritten = rewrite_version_boundary(text, "1.97.1", "1.98.0");
        assert!(rewritten.contains("channel = \"1.98.0\""));
        assert!(rewritten.contains("pin = \"1.98.0\""));
        assert!(rewritten.contains("ARG RUST_VERSION=1.98.0"));
        assert!(rewritten.contains("FROM rust:1.98.0-bookworm AS builder"));
        assert!(rewritten.contains("\"toolchain\": \"1.98.0\""));
        assert!(rewritten.contains("\"rust_toolchain\": \"1.98.0-stable\""));
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

    /// Full end-to-end fixture: a minimal repo shaped like the real tree (every surface the
    /// evaluators walk), bumped 1.97.1 -> 1.98.0, then verified clean against BOTH the freshness
    /// drift evaluator and the ADR-0535 gate.
    #[test]
    fn plan_apply_verify_on_fixture_tree_is_deterministic_and_clean() {
        let root = std::env::temp_dir().join(format!(
            "oya-toolchain-bump-proposer-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);

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
        write(
            &root,
            "docs/standards/image-discipline.md",
            "| Build stage | `rust:1.97.1-slim-trixie` | digest-pinned |\n",
        );
        write(&root, "deny.toml", "[licenses]\n");
        write(&root, "specs/oss-stewardship-registry.json", "{}\n");

        let plan = plan_bump(&root, "1.97.1", "1.98.0").expect("plan");
        assert!(
            plan.changed_count() >= 7,
            "expected all surfaces planned, got {}: {:?}",
            plan.changed_count(),
            plan.changed_paths()
        );
        // Deterministic: replanning the same tree yields the same plan.
        assert_eq!(plan_bump(&root, "1.97.1", "1.98.0").expect("replan"), plan);

        apply_plan(&root, &plan).expect("apply");

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
            read(&root, "docs/standards/image-discipline.md").contains("rust:1.98.0-slim-trixie")
        );

        // The bump must be idempotent: applying the same old->new again changes nothing.
        let replan = plan_bump(&root, "1.97.1", "1.98.0").expect("replan after apply");
        assert_eq!(replan.changed_count(), 0, "{:?}", replan.changed_paths());

        // The real evaluators must certify the tree clean.
        let residual = verify_clean(&root).expect("verify");
        assert!(
            residual.is_clean(),
            "fixture tree must be clean after bump: {:#?}",
            residual
        );

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
