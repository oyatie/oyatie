//! Per-crate hyperscaler-blessed direct-dependency allowlist gate.
//!
//! The pre-existing `dependency-seam` lane (in `oya-check-dependency-seam`)
//! reads `[workspace.dependencies]` and the per-dependency seam-isolation
//! `allowed_crates` policy, but it does NOT check each crate's own direct
//! dependencies against a hyperscaler-blessed allowlist. This gate closes
//! that gap:
//!
//! 1. It reads each workspace member's own `[dependencies]`,
//!    `[dev-dependencies]`, and `[build-dependencies]`.
//! 2. It checks every DIRECT external dependency against a data-driven
//!    blessed allowlist (`registry/dependency-blessed-allowlist.json`).
//! 3. It reports unblessed direct dependencies per-crate, with the crate path.
//!
//! Workspace-internal crates (name prefix `oya-`) and any dependency declared
//! with an explicit `path = ...` are EXEMPT — they are first-party code, not
//! external supply-chain surface.
//!
//! Default severity is report-only to honor the ADR-0092 D14
//! report-only-on-day-1 contract; callers opt in to blocking with `--enforce`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value as JsonValue, json};

use crate::usage;
use crate::workspace_manifest::{read_package_name, read_workspace_member_paths};

/// Default repo-relative path to the blessed-allowlist data file.
pub(crate) const DEFAULT_BLESSED_ALLOWLIST_PATH: &str = "registry/dependency-blessed-allowlist.json";

/// Dependency tables that contribute direct dependencies.
const DEPENDENCY_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BlessedAllowlistSeverity {
    /// Report unblessed direct deps but never fail (honors ADR-0092 D14).
    ReportOnly,
    /// Fail on the first unblessed direct dep.
    Enforce,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DependencyBlessedAllowlistArgs {
    repo_root: PathBuf,
    allowlist_path: PathBuf,
    severity: BlessedAllowlistSeverity,
    emit_report_path: Option<PathBuf>,
}

/// One unblessed direct-dependency finding, scoped to a crate.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct UnblessedDependencyFinding {
    /// Workspace member crate name (e.g. `oya-http-runtime-hyper-adapter`).
    pub crate_name: String,
    /// Repo-relative crate manifest path (e.g. `crates/<name>/Cargo.toml`).
    pub crate_path: String,
    /// The unblessed external dependency name.
    pub dependency: String,
    /// Which manifest table declared it (`dependencies` / `dev-dependencies` / `build-dependencies`).
    pub table: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DependencyBlessedAllowlistReport {
    pub crates_scanned: usize,
    pub blessed_count: usize,
    pub findings: Vec<UnblessedDependencyFinding>,
    pub enforced: bool,
}

impl DependencyBlessedAllowlistReport {
    pub fn unblessed_count(&self) -> usize {
        self.findings.len()
    }

    /// Distinct unblessed dependency names across all crates.
    pub fn distinct_unblessed(&self) -> BTreeSet<&str> {
        self.findings
            .iter()
            .map(|finding| finding.dependency.as_str())
            .collect()
    }

    /// Render the report as a `serde_json::Value` for `--emit-report`.
    ///
    /// `oya-dev-cli` does not take a direct `serde` derive dependency, so the
    /// report is constructed explicitly rather than via `#[derive(Serialize)]`.
    fn to_json_value(&self) -> JsonValue {
        let findings = self
            .findings
            .iter()
            .map(|finding| {
                json!({
                    "crate_name": finding.crate_name,
                    "crate_path": finding.crate_path,
                    "dependency": finding.dependency,
                    "table": finding.table,
                })
            })
            .collect::<Vec<_>>();
        json!({
            "crates_scanned": self.crates_scanned,
            "blessed_count": self.blessed_count,
            "unblessed_count": self.unblessed_count(),
            "distinct_unblessed_count": self.distinct_unblessed().len(),
            "enforced": self.enforced,
            "findings": findings,
        })
    }
}

pub(crate) fn parse_dependency_blessed_allowlist_args(
    args: Vec<String>,
) -> Result<DependencyBlessedAllowlistArgs, String> {
    let mut repo_root = PathBuf::from(".");
    let mut allowlist_path = PathBuf::from(DEFAULT_BLESSED_ALLOWLIST_PATH);
    let mut severity = BlessedAllowlistSeverity::ReportOnly;
    let mut emit_report_path = None;

    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--repo-root" => repo_root = next_path(&mut iter)?,
            "--allowlist" => allowlist_path = next_path(&mut iter)?,
            "--emit-report" => emit_report_path = Some(next_path(&mut iter)?),
            "--enforce" => severity = BlessedAllowlistSeverity::Enforce,
            "--report-only" => severity = BlessedAllowlistSeverity::ReportOnly,
            _ => return Err(usage()),
        }
    }

    Ok(DependencyBlessedAllowlistArgs {
        allowlist_path: resolve_repo_path(&repo_root, allowlist_path),
        emit_report_path: emit_report_path.map(|path| resolve_repo_path(&repo_root, path)),
        repo_root,
        severity,
    })
}

pub(crate) fn validate_dependency_blessed_allowlist_gate(
    args: DependencyBlessedAllowlistArgs,
) -> Result<DependencyBlessedAllowlistReport, String> {
    let blessed = read_blessed_allowlist(&args.allowlist_path)?;
    let members = read_workspace_member_manifests(&args.repo_root)?;

    let mut findings = Vec::new();
    for member in &members {
        for (table, dependency) in &member.external_dependencies {
            if !blessed.contains(dependency) {
                findings.push(UnblessedDependencyFinding {
                    crate_name: member.name.clone(),
                    crate_path: member.relative_manifest.clone(),
                    dependency: dependency.clone(),
                    table: table.clone(),
                });
            }
        }
    }
    findings.sort();

    let report = DependencyBlessedAllowlistReport {
        crates_scanned: members.len(),
        blessed_count: blessed.len(),
        findings,
        enforced: matches!(args.severity, BlessedAllowlistSeverity::Enforce),
    };

    if let Some(path) = &args.emit_report_path {
        write_report(path, &report)?;
    }

    Ok(report)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MemberManifest {
    name: String,
    relative_manifest: String,
    /// (table, dependency-name) pairs for every external direct dep.
    external_dependencies: Vec<(String, String)>,
}

fn read_blessed_allowlist(path: &Path) -> Result<BTreeSet<String>, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "dependency-blessed-allowlist data file unreadable {}: {error}",
            path.display()
        )
    })?;
    let value: JsonValue = serde_json::from_str(&contents).map_err(|error| {
        format!(
            "dependency-blessed-allowlist data file invalid {}: {error}",
            path.display()
        )
    })?;
    let blessed = value
        .get("blessed")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| {
            format!(
                "dependency-blessed-allowlist {} lacks a `blessed` object",
                path.display()
            )
        })?;
    if blessed.is_empty() {
        return Err(format!(
            "dependency-blessed-allowlist {} `blessed` object is empty",
            path.display()
        ));
    }
    Ok(blessed.keys().cloned().collect())
}

fn read_workspace_member_manifests(repo_root: &Path) -> Result<Vec<MemberManifest>, String> {
    let workspace_manifest = repo_root.join("Cargo.toml");
    let relative_paths = read_workspace_member_paths(&workspace_manifest)?;

    let mut out = Vec::new();
    for relative_path in relative_paths {
        let manifest_path = repo_root.join(&relative_path).join("Cargo.toml");
        if !manifest_path.is_file() {
            continue;
        }
        let Ok(name) = read_package_name(&manifest_path) else {
            // A workspace member without a `[package]` name (e.g. a nested
            // virtual manifest) carries no first-party crate to attribute.
            continue;
        };
        out.push(MemberManifest {
            name,
            relative_manifest: format!("{relative_path}/Cargo.toml"),
            external_dependencies: external_dependencies(&manifest_path)?,
        });
    }
    Ok(out)
}

/// Collect `(table, name)` pairs for every DIRECT external dependency declared
/// in a crate manifest.
///
/// A dependency is EXEMPT (not external supply-chain surface) when either:
/// - its name starts with `oya-` (first-party workspace crate), or
/// - it is declared with an explicit `path = ...` (local path dependency).
///
/// The workspace already hand-parses Cargo manifests rather than taking a
/// direct `toml` crate dependency (see `workspace_manifest.rs`); this parser
/// follows that same line-oriented convention.
fn external_dependencies(manifest_path: &Path) -> Result<Vec<(String, String)>, String> {
    let contents = fs::read_to_string(manifest_path).map_err(|error| {
        format!(
            "package manifest unreadable {}: {error}",
            manifest_path.display()
        )
    })?;
    let mut findings: BTreeSet<(String, String)> = BTreeSet::new();
    let mut current_table: Option<&str> = None;
    for raw_line in contents.lines() {
        let line = strip_inline_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if let Some(section) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            current_table = DEPENDENCY_TABLES
                .into_iter()
                .find(|table| *table == section.trim());
            continue;
        }
        let Some(table_name) = current_table else {
            continue;
        };
        let Some(alias) = dependency_key(line) else {
            continue;
        };
        if alias.starts_with("oya-") {
            continue;
        }
        // Inline-table specs declared with an explicit `path = ...` are
        // first-party local deps and exempt.
        if is_path_dependency(line) {
            continue;
        }
        // Honor an explicit `package = "..."` rename: the crates.io crate is
        // the renamed-to package, not the local alias key.
        let resolved = renamed_package(line).unwrap_or_else(|| alias.to_string());
        if resolved.starts_with("oya-") {
            continue;
        }
        findings.insert((table_name.to_string(), resolved));
    }
    Ok(findings.into_iter().collect())
}

/// Extract the dependency key (left of the first `=` or `.`) from a manifest
/// line such as `tokio.workspace = true`, `serde = "1"`, or
/// `foo = { package = "bar" }`.
fn dependency_key(line: &str) -> Option<&str> {
    let key_region = line.split('=').next().unwrap_or(line);
    // `tokio.workspace = true` -> dependency name is the segment before `.`.
    let key = key_region.split('.').next().unwrap_or(key_region).trim();
    if key.is_empty() {
        None
    } else {
        Some(key)
    }
}

fn is_path_dependency(line: &str) -> bool {
    inline_table_field(line, "path").is_some()
}

fn renamed_package(line: &str) -> Option<String> {
    inline_table_field(line, "package")
}

/// Read a string field (e.g. `package = "rand"` or `path = "../x"`) out of an
/// inline-table dependency spec line.
fn inline_table_field(line: &str, field: &str) -> Option<String> {
    let needle = format!("{field} =");
    let start = line.find(&needle)? + needle.len();
    let rest = line[start..].trim_start();
    if let Some(after_quote) = rest.strip_prefix('"') {
        let end = after_quote.find('"')?;
        return Some(after_quote[..end].to_string());
    }
    // Bare (non-string) value such as `path = true` is not a real field value
    // we care about; treat presence of the key alone as truthy for `path`.
    if field == "path" {
        return Some(String::new());
    }
    None
}

fn strip_inline_comment(line: &str) -> &str {
    // Cargo manifests do not use `#` inside dependency string values in this
    // workspace; a `#` therefore always begins a comment.
    match line.find('#') {
        Some(index) => &line[..index],
        None => line,
    }
}

fn write_report(path: &Path, report: &DependencyBlessedAllowlistReport) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "dependency-blessed-allowlist report parent unwritable {}: {error}",
                parent.display()
            )
        })?;
    }
    let encoded = serde_json::to_string_pretty(&report.to_json_value()).map_err(|error| {
        format!("dependency-blessed-allowlist report serialization failed: {error}")
    })?;
    fs::write(path, encoded).map_err(|error| {
        format!(
            "dependency-blessed-allowlist report write failed {}: {error}",
            path.display()
        )
    })
}

fn next_path(iter: &mut impl Iterator<Item = String>) -> Result<PathBuf, String> {
    iter.next().map(PathBuf::from).ok_or_else(usage)
}

fn resolve_repo_path(repo_root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        repo_root.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn scratch_root(slug: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "oya-blessed-allowlist-{slug}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("scratch root created");
        root
    }

    fn cleanup(root: &Path) {
        let _ = fs::remove_dir_all(root);
    }

    fn write_allowlist(root: &Path, blessed: &[&str]) -> PathBuf {
        let entries = blessed
            .iter()
            .map(|name| format!("\"{name}\": {{\"rationale\": \"test\"}}"))
            .collect::<Vec<_>>()
            .join(",");
        let path = root.join("allowlist.json");
        fs::write(&path, format!("{{\"blessed\":{{{entries}}}}}")).expect("allowlist written");
        path
    }

    fn write_workspace(root: &Path, members: &[&str]) {
        let rendered = members
            .iter()
            .map(|member| format!("\"{member}\""))
            .collect::<Vec<_>>()
            .join(", ");
        fs::write(
            root.join("Cargo.toml"),
            format!("[workspace]\nmembers = [{rendered}]\n"),
        )
        .expect("workspace manifest written");
    }

    fn write_member(root: &Path, path: &str, name: &str, deps_block: &str) {
        let crate_dir = root.join(path);
        fs::create_dir_all(crate_dir.join("src")).expect("crate dir created");
        fs::write(
            crate_dir.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{name}\"\nedition = \"2024\"\nversion = \"0.1.0\"\nlicense = \"Apache-2.0\"\n{deps_block}"
            ),
        )
        .expect("member manifest written");
        fs::write(crate_dir.join("src/lib.rs"), "pub fn it() {}\n").expect("member lib written");
    }

    fn run(root: &Path, allowlist: &Path, enforce: bool) -> DependencyBlessedAllowlistReport {
        let mut args = vec![
            "--repo-root".to_string(),
            root.to_str().expect("utf8 root").to_string(),
            "--allowlist".to_string(),
            allowlist.to_str().expect("utf8 allowlist").to_string(),
        ];
        if enforce {
            args.push("--enforce".to_string());
        }
        let parsed = parse_dependency_blessed_allowlist_args(args).expect("args parse");
        validate_dependency_blessed_allowlist_gate(parsed).expect("gate runs")
    }

    #[test]
    fn flags_crate_with_planted_unblessed_dependency() {
        let root = scratch_root("planted-unblessed");
        write_workspace(&root, &["crates/offender"]);
        write_member(
            &root,
            "crates/offender",
            "offender-adapter",
            "[dependencies]\ntokio.workspace = true\nsketchy-unblessed-crate = \"1\"\n",
        );
        let allowlist = write_allowlist(&root, &["tokio", "serde"]);

        let report = run(&root, &allowlist, false);

        assert_eq!(report.crates_scanned, 1);
        assert_eq!(report.unblessed_count(), 1);
        let finding = &report.findings[0];
        assert_eq!(finding.crate_name, "offender-adapter");
        assert_eq!(finding.dependency, "sketchy-unblessed-crate");
        assert_eq!(finding.crate_path, "crates/offender/Cargo.toml");
        assert_eq!(finding.table, "dependencies");
        cleanup(&root);
    }

    #[test]
    fn all_blessed_crate_passes_with_no_findings() {
        let root = scratch_root("all-blessed");
        write_workspace(&root, &["crates/clean"]);
        write_member(
            &root,
            "crates/clean",
            "clean-adapter",
            // tokio + serde are blessed; the oya-* path dep and an explicit
            // path dep are both exempt.
            "[dependencies]\ntokio.workspace = true\nserde.workspace = true\noya-kernel = { path = \"../oya-kernel\" }\nsome-vendored = { path = \"../vendor/some-vendored\" }\n",
        );
        let allowlist = write_allowlist(&root, &["tokio", "serde"]);

        let report = run(&root, &allowlist, true);

        assert_eq!(report.crates_scanned, 1);
        assert_eq!(report.unblessed_count(), 0, "report={report:?}");
        assert!(report.enforced);
        cleanup(&root);
    }

    #[test]
    fn dev_and_build_dependencies_are_checked_too() {
        let root = scratch_root("dev-build-tables");
        write_workspace(&root, &["crates/multi"]);
        write_member(
            &root,
            "crates/multi",
            "multi-crate",
            "[dependencies]\nserde.workspace = true\n[dev-dependencies]\nunblessed-dev = \"1\"\n[build-dependencies]\nunblessed-build = \"1\"\n",
        );
        let allowlist = write_allowlist(&root, &["serde"]);

        let report = run(&root, &allowlist, false);

        let by_table: BTreeSet<(&str, &str)> = report
            .findings
            .iter()
            .map(|finding| (finding.table.as_str(), finding.dependency.as_str()))
            .collect();
        assert!(by_table.contains(&("dev-dependencies", "unblessed-dev")));
        assert!(by_table.contains(&("build-dependencies", "unblessed-build")));
        assert_eq!(report.unblessed_count(), 2, "report={report:?}");
        cleanup(&root);
    }

    #[test]
    fn allowlist_is_data_driven_changing_file_changes_verdict() {
        let root = scratch_root("data-driven");
        write_workspace(&root, &["crates/svc"]);
        write_member(
            &root,
            "crates/svc",
            "svc-runtime",
            "[dependencies]\nchrono = \"0.4\"\n",
        );

        // chrono NOT blessed -> flagged.
        let strict = write_allowlist(&root, &["tokio"]);
        let strict_report = run(&root, &strict, false);
        assert_eq!(strict_report.unblessed_count(), 1);
        assert!(strict_report.distinct_unblessed().contains("chrono"));

        // Same crate, allowlist now blesses chrono -> clean. Proves the
        // verdict is driven by the data file, not hard-coded in Rust.
        let permissive = write_allowlist(&root, &["tokio", "chrono"]);
        let permissive_report = run(&root, &permissive, false);
        assert_eq!(permissive_report.unblessed_count(), 0);
        cleanup(&root);
    }

    #[test]
    fn package_rename_resolves_to_crates_io_name() {
        let root = scratch_root("package-rename");
        write_workspace(&root, &["crates/renamer"]);
        write_member(
            &root,
            "crates/renamer",
            "renamer-app",
            // Local alias `fast-rng` maps to crates.io crate `rand`.
            "[dependencies]\nfast-rng = { package = \"rand\", version = \"0.8\" }\n",
        );
        let allowlist = write_allowlist(&root, &["rand"]);

        let report = run(&root, &allowlist, false);

        assert_eq!(
            report.unblessed_count(),
            0,
            "renamed dep should resolve to its crates.io package name; report={report:?}"
        );
        cleanup(&root);
    }

    #[test]
    fn missing_blessed_object_is_an_error() {
        let root = scratch_root("bad-allowlist");
        write_workspace(&root, &["crates/svc"]);
        write_member(
            &root,
            "crates/svc",
            "svc",
            "[dependencies]\nserde.workspace = true\n",
        );
        let bad = root.join("bad.json");
        fs::write(&bad, "{\"not_blessed\": {}}").expect("bad allowlist written");
        let parsed = parse_dependency_blessed_allowlist_args(vec![
            "--repo-root".to_string(),
            root.to_str().expect("utf8").to_string(),
            "--allowlist".to_string(),
            bad.to_str().expect("utf8").to_string(),
        ])
        .expect("args parse");
        let result = validate_dependency_blessed_allowlist_gate(parsed);
        assert!(result.is_err(), "missing blessed object must error");
        cleanup(&root);
    }
}
