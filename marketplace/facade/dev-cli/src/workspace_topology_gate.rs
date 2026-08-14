//! `oya gate validate workspace-topology` — canonical monorepo topology gate.
//!
//! Codifies ADR-0512: vertical-slice nesting, one workspace, bounded-context
//! crates, dependency-rule modules. Enforces seven structural invariants:
//!
//! - R1: no flat top-level `crates/` directory that contains any `Cargo.toml`
//!       (vertical-slice canon: code lives under cloud/<svc>/crates/, oya/<svc>/crates/, or libs/).
//! - R2: no nested `[workspace]` table in any member `Cargo.toml`
//!       (single root workspace; no per-service workspaces).
//! - R3: no duplicate `[package].name` across members.
//! - R4: every `members` entry resolves to an existing dir with a `Cargo.toml`
//!       (no phantom members).
//! - R5: every crate dir on disk under `cloud/`, `oya/`, `microservices/`, and
//!       `libs/` (any dir with a `[package]` `Cargo.toml`) IS a workspace member
//!       (no orphan).
//! - R6: every member path is under one of the canonical prefixes:
//!       `cloud/<svc>/crates/<crate>`, `oya/<svc>/crates/<crate>`,
//!       `microservices/<ms>/crates/<crate>`, `microservices/<ms>` (single-level),
//!       `libs/<lib>`, or `tools/<name>`.
//! - R7: every workspace member's crate-dir basename MUST equal its `[package].name`
//!       (dir==name invariant: the directory that houses a crate is named after it).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value as JsonValue, json};

use crate::usage;
use crate::workspace_manifest::read_workspace_member_paths;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkspaceTopologySeverity {
    ReportOnly,
    Enforce,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorkspaceTopologyArgs {
    repo_root: PathBuf,
    severity: WorkspaceTopologySeverity,
    emit_report_path: Option<PathBuf>,
}

/// The rule that a finding violates.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum WorkspaceTopologyRule {
    /// R1: flat top-level `crates/` dir containing a `Cargo.toml`.
    R1FlatCratesDir,
    /// R2: nested `[workspace]` table in a member manifest.
    R2NestedWorkspace,
    /// R3: duplicate `[package].name` across members.
    R3DuplicateName,
    /// R4: member path does not resolve to a dir with `Cargo.toml`.
    R4PhantomMember,
    /// R5: crate dir on disk not registered as a workspace member.
    R5OrphanCrate,
    /// R6: member path is not under a canonical prefix.
    R6InvalidLocation,
    /// R7: crate-dir basename does not equal `[package].name`.
    R7DirNameMismatch,
}

impl WorkspaceTopologyRule {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            WorkspaceTopologyRule::R1FlatCratesDir => "R1-flat-crates-dir",
            WorkspaceTopologyRule::R2NestedWorkspace => "R2-nested-workspace",
            WorkspaceTopologyRule::R3DuplicateName => "R3-duplicate-name",
            WorkspaceTopologyRule::R4PhantomMember => "R4-phantom-member",
            WorkspaceTopologyRule::R5OrphanCrate => "R5-orphan-crate",
            WorkspaceTopologyRule::R6InvalidLocation => "R6-invalid-location",
            WorkspaceTopologyRule::R7DirNameMismatch => "R7-dir-name-mismatch",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct WorkspaceTopologyFinding {
    pub rule: WorkspaceTopologyRule,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorkspaceTopologyReport {
    pub members_scanned: usize,
    pub findings: Vec<WorkspaceTopologyFinding>,
    pub enforced: bool,
}

impl WorkspaceTopologyReport {
    /// Render the report as a `serde_json::Value` for `--emit-report`.
    pub(crate) fn to_json_value(&self) -> JsonValue {
        let findings = self
            .findings
            .iter()
            .map(|f| {
                json!({
                    "rule": f.rule.as_str(),
                    "detail": f.detail,
                })
            })
            .collect::<Vec<_>>();
        json!({
            "members_scanned": self.members_scanned,
            "finding_count": self.findings.len(),
            "enforced": self.enforced,
            "findings": findings,
        })
    }
}

pub(crate) fn parse_workspace_topology_validate_args(
    args: Vec<String>,
) -> Result<WorkspaceTopologyArgs, String> {
    let mut repo_root = PathBuf::from(".");
    let mut severity = WorkspaceTopologySeverity::Enforce;
    let mut emit_report_path = None;

    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--repo-root" => repo_root = next_path(&mut iter)?,
            "--emit-report" => emit_report_path = Some(next_path(&mut iter)?),
            "--enforce" => severity = WorkspaceTopologySeverity::Enforce,
            "--report-only" => severity = WorkspaceTopologySeverity::ReportOnly,
            _ => return Err(usage()),
        }
    }

    Ok(WorkspaceTopologyArgs {
        emit_report_path: emit_report_path.map(|path| resolve_repo_path(&repo_root, path)),
        repo_root,
        severity,
    })
}

pub(crate) fn validate_workspace_topology_gate(
    args: WorkspaceTopologyArgs,
) -> Result<WorkspaceTopologyReport, String> {
    let mut findings = Vec::new();

    // R1: flat top-level crates/ dir must not contain any Cargo.toml.
    check_r1_flat_crates_dir(&args.repo_root, &mut findings);

    // R4 needs the RAW declared members (a literal member whose directory is absent is
    // dropped entirely by cargo-faithful glob expansion, so it never reaches the
    // resolved-member walk below). Check every literal (non-glob) declaration here;
    // glob patterns are expanded by the scan and their phantom matches reported below.
    let entries = oya_workspace_members_kernel::read_workspace_manifest_entries(&args.repo_root)
        .map_err(|error| format!("workspace manifest entries: {error}"))?;
    for declared in &entries.members {
        if declared.contains('*') {
            continue;
        }
        let manifest_path = args.repo_root.join(declared).join("Cargo.toml");
        if !manifest_path.is_file() {
            findings.push(WorkspaceTopologyFinding {
                rule: WorkspaceTopologyRule::R4PhantomMember,
                detail: format!(
                    "member `{declared}` has no Cargo.toml at {}",
                    manifest_path.display()
                ),
            });
        }
    }
    // Cargo-faithful resolution: expand globs and require every unexcluded match to carry
    // a manifest. Missing glob matches are R4 phantoms; the existing dirs feed R2/R3/R5-R7.
    let scan = oya_workspace_members_kernel::scan_member_dirs(&args.repo_root)
        .map_err(|error| format!("workspace members unresolved: {error}"))?;
    for phantom in &scan.missing_manifests {
        let manifest_path = args.repo_root.join(phantom).join("Cargo.toml");
        findings.push(WorkspaceTopologyFinding {
            rule: WorkspaceTopologyRule::R4PhantomMember,
            detail: format!(
                "member `{phantom}` has no Cargo.toml at {}",
                manifest_path.display()
            ),
        });
    }
    let member_paths = scan.member_dirs;

    // Build member set (repo-relative paths) for orphan check (R5).
    let member_set: BTreeSet<String> = member_paths.iter().cloned().collect();

    let mut names_seen: BTreeMap<String, String> = BTreeMap::new();

    for rel_path in &member_paths {
        let member_dir = args.repo_root.join(rel_path);
        let manifest_path = member_dir.join("Cargo.toml");

        // R4: member path must resolve to an existing dir with Cargo.toml.
        if !manifest_path.is_file() {
            findings.push(WorkspaceTopologyFinding {
                rule: WorkspaceTopologyRule::R4PhantomMember,
                detail: format!(
                    "member `{rel_path}` has no Cargo.toml at {}",
                    manifest_path.display()
                ),
            });
            continue;
        }

        let contents = fs::read_to_string(&manifest_path).map_err(|error| {
            format!(
                "workspace-topology: member manifest unreadable {}: {error}",
                manifest_path.display()
            )
        })?;

        // R2: member must not declare [workspace].
        if has_workspace_table(&contents) {
            findings.push(WorkspaceTopologyFinding {
                rule: WorkspaceTopologyRule::R2NestedWorkspace,
                detail: format!(
                    "member `{rel_path}` declares a [workspace] table (only one root workspace is allowed)"
                ),
            });
        }

        // R3: duplicate package name.
        if let Some(name) = read_package_name_from_contents(&contents) {
            if let Some(prior) = names_seen.get(&name) {
                findings.push(WorkspaceTopologyFinding {
                    rule: WorkspaceTopologyRule::R3DuplicateName,
                    detail: format!(
                        "duplicate package name `{name}` in `{rel_path}` (first seen at `{prior}`)"
                    ),
                });
            } else {
                names_seen.insert(name, rel_path.clone());
            }
        }

        // R6: member path must be under a canonical prefix.
        check_r6_location(rel_path, &mut findings);

        // R7: crate-dir basename must equal [package].name.
        if let Some(name) = read_package_name_from_contents(&contents) {
            check_r7_dir_name_match(rel_path, &name, &mut findings);
        }
    }

    // R5: every crate dir on disk under microservices/ and libs/ must be a member.
    check_r5_orphan_crates(&args.repo_root, &member_set, &mut findings)?;

    findings.sort();

    let report = WorkspaceTopologyReport {
        members_scanned: member_paths.len(),
        findings,
        enforced: matches!(args.severity, WorkspaceTopologySeverity::Enforce),
    };

    if let Some(path) = &args.emit_report_path {
        write_report(path, &report)?;
    }

    Ok(report)
}

/// R1: fail if `<repo_root>/crates/` exists AND contains any `Cargo.toml`
/// at depth 1 (i.e. `crates/<name>/Cargo.toml`).
fn check_r1_flat_crates_dir(repo_root: &Path, findings: &mut Vec<WorkspaceTopologyFinding>) {
    let crates_dir = repo_root.join("crates");
    if !crates_dir.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(&crates_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let toml = entry.path().join("Cargo.toml");
        if toml.is_file() {
            findings.push(WorkspaceTopologyFinding {
                rule: WorkspaceTopologyRule::R1FlatCratesDir,
                detail: format!(
                    "flat top-level crates/ dir contains a Cargo.toml: {}",
                    toml.display()
                ),
            });
        }
    }
}

/// R2 helper: returns true if the manifest content contains a bare `[workspace]`
/// table header (not `[workspace.dependencies]` etc.).
fn has_workspace_table(contents: &str) -> bool {
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed == "[workspace]" {
            return true;
        }
    }
    false
}

/// Extract `[package].name` from manifest contents without pulling in a TOML
/// parser — mirrors the line-oriented approach in `workspace_manifest.rs`.
fn read_package_name_from_contents(contents: &str) -> Option<String> {
    let mut in_package = false;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        if key.trim() != "name" {
            continue;
        }
        let name = value.trim().trim_matches('"').to_string();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

/// R6: validate that a workspace member path is under a canonical prefix.
///
/// Accepted forms:
/// - `libs/<lib>`                              (shared library)
/// - `tools/<name>`                            (governance / tooling app)
/// - `cloud/<svc>/crates/<crate>`             (cloud-infrastructure service crate)
/// - `oya/<svc>/crates/<crate>`               (oya application service crate)
/// - `microservices/<ms>`                      (legacy single-concern service at top of ms dir)
/// - `microservices/<ms>/crates/<crate>`       (legacy bounded-context crate nested under ms)
fn check_r6_location(rel_path: &str, findings: &mut Vec<WorkspaceTopologyFinding>) {
    let parts: Vec<&str> = rel_path.split('/').collect();
    let valid = match parts.as_slice() {
        // libs/<lib>
        ["libs", _] => true,
        // tools/<name>
        ["tools", _] => true,
        // cloud/<svc>/crates/<crate>
        ["cloud", _, "crates", _] => true,
        // oya/<svc>/crates/<crate>
        ["oya", _, "crates", _] => true,
        // microservices/<ms> (legacy)
        ["microservices", _] => true,
        // microservices/<ms>/crates/<crate> (legacy)
        ["microservices", _, "crates", _] => true,
        _ => false,
    };
    if !valid {
        findings.push(WorkspaceTopologyFinding {
            rule: WorkspaceTopologyRule::R6InvalidLocation,
            detail: format!(
                "member `{rel_path}` is not under a canonical prefix \
                 (expected libs/<lib>, tools/<name>, cloud/<svc>/crates/<crate>, \
                 oya/<svc>/crates/<crate>, microservices/<ms>, \
                 or microservices/<ms>/crates/<crate>)"
            ),
        });
    }
}

/// R7: the crate-dir basename (last path segment of `rel_path`) must equal
/// `[package].name` declared in the manifest.
fn check_r7_dir_name_match(
    rel_path: &str,
    package_name: &str,
    findings: &mut Vec<WorkspaceTopologyFinding>,
) {
    let dir_basename = rel_path.split('/').next_back().unwrap_or(rel_path);
    if dir_basename != package_name {
        findings.push(WorkspaceTopologyFinding {
            rule: WorkspaceTopologyRule::R7DirNameMismatch,
            detail: format!(
                "member `{rel_path}` dir basename `{dir_basename}` != package name `{package_name}` \
                 (dir==name invariant violated)"
            ),
        });
    }
}

/// R5: walk `cloud/`, `oya/`, `microservices/`, and `libs/` under `repo_root`
/// looking for directories that contain a `Cargo.toml` with a `[package]`
/// section but are NOT registered as workspace members.
///
/// The walk is bounded to a shallow depth to stay fast:
/// - `cloud/<svc>/crates/<crate>` (depth 4): cloud service crate.
/// - `oya/<svc>/crates/<crate>` (depth 4): oya application crate.
/// - `microservices/<ms>` (depth 2): legacy direct member.
/// - `microservices/<ms>/crates/<crate>` (depth 4): legacy nested crate.
/// - `libs/<lib>` (depth 2).
fn check_r5_orphan_crates(
    repo_root: &Path,
    member_set: &BTreeSet<String>,
    findings: &mut Vec<WorkspaceTopologyFinding>,
) -> Result<(), String> {
    // "crates-only" roots: only <top>/<svc>/crates/<crate> form is valid (no top-level members).
    for top in &["cloud", "oya"] {
        let top_dir = repo_root.join(top);
        if !top_dir.is_dir() {
            continue;
        }
        let Ok(svc_entries) = fs::read_dir(&top_dir) else {
            continue;
        };
        for svc_entry in svc_entries.flatten() {
            let svc_path = svc_entry.path();
            if !svc_path.is_dir() {
                continue;
            }
            let svc_name = match svc_path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            if is_workspace_root_dir(&svc_path) {
                continue;
            }
            let crates_dir = svc_path.join("crates");
            if !crates_dir.is_dir() {
                continue;
            }
            let Ok(crate_entries) = fs::read_dir(&crates_dir) else {
                continue;
            };
            for crate_entry in crate_entries.flatten() {
                let crate_path = crate_entry.path();
                if !crate_path.is_dir() {
                    continue;
                }
                let crate_name = match crate_path.file_name().and_then(|n| n.to_str()) {
                    Some(n) => n.to_string(),
                    None => continue,
                };
                let rel_crate = format!("{top}/{svc_name}/crates/{crate_name}");
                if is_package_dir(&crate_path) && !member_set.contains(&rel_crate) {
                    findings.push(WorkspaceTopologyFinding {
                        rule: WorkspaceTopologyRule::R5OrphanCrate,
                        detail: format!(
                            "crate dir `{rel_crate}` has a Cargo.toml with [package] but is not a workspace member"
                        ),
                    });
                }
            }
        }
    }

    // Legacy roots: microservices/ (supports both top-level and crates/ depth)
    // and libs/ (top-level only).
    for top in &["microservices", "libs"] {
        let top_dir = repo_root.join(top);
        if !top_dir.is_dir() {
            continue;
        }
        let Ok(ms_entries) = fs::read_dir(&top_dir) else {
            continue;
        };
        for ms_entry in ms_entries.flatten() {
            let ms_path = ms_entry.path();
            if !ms_path.is_dir() {
                continue;
            }
            let ms_name = match ms_path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            // Check <top>/<ms> itself.
            let rel_ms = format!("{top}/{ms_name}");
            if is_package_dir(&ms_path) && !member_set.contains(&rel_ms) {
                findings.push(WorkspaceTopologyFinding {
                    rule: WorkspaceTopologyRule::R5OrphanCrate,
                    detail: format!(
                        "crate dir `{rel_ms}` has a Cargo.toml with [package] but is not a workspace member"
                    ),
                });
            }

            // For microservices/<ms>/crates/<crate> — only microservices/ has this depth.
            if *top == "microservices" {
                let crates_dir = ms_path.join("crates");
                if !crates_dir.is_dir() {
                    continue;
                }
                let Ok(crate_entries) = fs::read_dir(&crates_dir) else {
                    continue;
                };
                for crate_entry in crate_entries.flatten() {
                    let crate_path = crate_entry.path();
                    if !crate_path.is_dir() {
                        continue;
                    }
                    let crate_name = match crate_path.file_name().and_then(|n| n.to_str()) {
                        Some(n) => n.to_string(),
                        None => continue,
                    };
                    let rel_crate = format!("microservices/{ms_name}/crates/{crate_name}");
                    if is_package_dir(&crate_path) && !member_set.contains(&rel_crate) {
                        findings.push(WorkspaceTopologyFinding {
                            rule: WorkspaceTopologyRule::R5OrphanCrate,
                            detail: format!(
                                "crate dir `{rel_crate}` has a Cargo.toml with [package] but is not a workspace member"
                            ),
                        });
                    }
                }
            }
        }
    }
    Ok(())
}

/// Returns true if `dir` contains a `Cargo.toml` that declares a `[package]`
/// section (not a workspace-only manifest).
fn is_package_dir(dir: &Path) -> bool {
    let toml = dir.join("Cargo.toml");
    if !toml.is_file() {
        return false;
    }
    let Ok(contents) = fs::read_to_string(&toml) else {
        return false;
    };
    contents.lines().any(|line| line.trim() == "[package]")
}

fn is_workspace_root_dir(dir: &Path) -> bool {
    let toml = dir.join("Cargo.toml");
    if !toml.is_file() {
        return false;
    }
    let Ok(contents) = fs::read_to_string(&toml) else {
        return false;
    };
    has_workspace_table(&contents)
}

fn write_report(path: &Path, report: &WorkspaceTopologyReport) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "workspace-topology report parent unwritable {}: {error}",
                parent.display()
            )
        })?;
    }
    let encoded = serde_json::to_string_pretty(&report.to_json_value())
        .map_err(|error| format!("workspace-topology report serialization failed: {error}"))?;
    fs::write(path, encoded).map_err(|error| {
        format!(
            "workspace-topology report write failed {}: {error}",
            path.display()
        )
    })
}

pub(crate) fn next_path(iter: &mut impl Iterator<Item = String>) -> Result<PathBuf, String> {
    iter.next().map(PathBuf::from).ok_or_else(usage)
}

pub(crate) fn resolve_repo_path(repo_root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        repo_root.join(path)
    }
}

/// Read workspace member manifests for use by other gates (mirrors
/// `http_stack_gate::read_workspace_member_manifests` but returns only
/// (relative_path, name) pairs — kept minimal to avoid duplication).
pub(crate) fn read_workspace_member_manifests(
    repo_root: &Path,
) -> Result<Vec<(String, String)>, String> {
    let workspace_manifest = repo_root.join("Cargo.toml");
    let paths = read_workspace_member_paths(&workspace_manifest)?;
    let mut out = Vec::new();
    for rel in paths {
        let manifest = repo_root.join(&rel).join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let Ok(contents) = fs::read_to_string(&manifest) else {
            continue;
        };
        if let Some(name) = read_package_name_from_contents(&contents) {
            out.push((rel, name));
        }
    }
    Ok(out)
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
            "oya-ws-topology-{slug}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("scratch root created");
        root
    }

    fn cleanup(root: &Path) {
        let _ = fs::remove_dir_all(root);
    }

    fn write_workspace(root: &Path, members: &[&str]) {
        let rendered = members
            .iter()
            .map(|m| format!("\"{m}\""))
            .collect::<Vec<_>>()
            .join(", ");
        fs::write(
            root.join("Cargo.toml"),
            format!("[workspace]\nmembers = [{rendered}]\n"),
        )
        .expect("workspace manifest written");
    }

    fn write_member(root: &Path, path: &str, name: &str) {
        let dir = root.join(path);
        fs::create_dir_all(dir.join("src")).expect("member dir created");
        fs::write(
            dir.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{name}\"\nedition = \"2024\"\nversion = \"0.1.0\"\nlicense = \"Apache-2.0\"\n"
            ),
        )
        .expect("member manifest written");
    }

    fn write_nested_workspace_member(root: &Path, path: &str, name: &str) {
        let dir = root.join(path);
        fs::create_dir_all(dir.join("src")).expect("member dir created");
        fs::write(
            dir.join("Cargo.toml"),
            format!(
                "[workspace]\nmembers = []\n\n[package]\nname = \"{name}\"\nedition = \"2024\"\nversion = \"0.1.0\"\nlicense = \"Apache-2.0\"\n"
            ),
        )
        .expect("nested workspace member manifest written");
    }

    fn run(root: &Path, enforce: bool) -> WorkspaceTopologyReport {
        let args = WorkspaceTopologyArgs {
            repo_root: root.to_path_buf(),
            severity: if enforce {
                WorkspaceTopologySeverity::Enforce
            } else {
                WorkspaceTopologySeverity::ReportOnly
            },
            emit_report_path: None,
        };
        validate_workspace_topology_gate(args).expect("gate runs")
    }

    // --- Happy path ---

    #[test]
    fn happy_path_canonical_tree_passes() {
        let root = scratch_root("happy");
        write_workspace(
            &root,
            &[
                "microservices/accounting/crates/oya-accounting-journal-domain",
                "libs/oya-check-brand-residue",
                "tools/oya-governance-adr-shape-app",
                "microservices/oya-crm-app",
            ],
        );
        write_member(
            &root,
            "microservices/accounting/crates/oya-accounting-journal-domain",
            "oya-accounting-journal-domain",
        );
        write_member(
            &root,
            "libs/oya-check-brand-residue",
            "oya-check-brand-residue",
        );
        write_member(
            &root,
            "tools/oya-governance-adr-shape-app",
            "oya-governance-adr-shape-app",
        );
        write_member(&root, "microservices/oya-crm-app", "oya-crm-app");
        let report = run(&root, true);
        assert!(
            report.findings.is_empty(),
            "expected no findings, got: {:?}",
            report.findings
        );
        cleanup(&root);
    }

    // --- R1: flat crates/ dir ---

    #[test]
    fn r1_flat_crates_dir_with_cargo_toml_fails() {
        let root = scratch_root("r1");
        write_workspace(&root, &["microservices/svc"]);
        write_member(&root, "microservices/svc", "oya-svc-app");
        // Place a Cargo.toml inside flat crates/.
        let flat = root.join("crates").join("oya-bad-crate");
        fs::create_dir_all(&flat).expect("flat crate dir");
        fs::write(
            flat.join("Cargo.toml"),
            "[package]\nname = \"oya-bad-crate\"\nedition = \"2024\"\nversion = \"0.1.0\"\n",
        )
        .expect("flat crate toml");
        let report = run(&root, true);
        let r1_findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule == WorkspaceTopologyRule::R1FlatCratesDir)
            .collect();
        assert!(!r1_findings.is_empty(), "expected R1 finding");
        cleanup(&root);
    }

    // --- R2: nested workspace ---

    #[test]
    fn r2_nested_workspace_table_fails() {
        let root = scratch_root("r2");
        let path = "microservices/svc/crates/oya-svc-domain";
        write_workspace(&root, &[path]);
        write_nested_workspace_member(&root, path, "oya-svc-domain");
        let report = run(&root, true);
        let r2: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule == WorkspaceTopologyRule::R2NestedWorkspace)
            .collect();
        assert!(
            !r2.is_empty(),
            "expected R2 finding, got: {:?}",
            report.findings
        );
        cleanup(&root);
    }

    // --- R3: duplicate names ---

    #[test]
    fn r3_duplicate_package_name_fails() {
        let root = scratch_root("r3");
        write_workspace(
            &root,
            &[
                "microservices/svc-a/crates/oya-svc-domain",
                "microservices/svc-b/crates/oya-svc-domain",
            ],
        );
        write_member(
            &root,
            "microservices/svc-a/crates/oya-svc-domain",
            "oya-svc-domain",
        );
        write_member(
            &root,
            "microservices/svc-b/crates/oya-svc-domain",
            "oya-svc-domain",
        );
        let report = run(&root, true);
        let r3: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule == WorkspaceTopologyRule::R3DuplicateName)
            .collect();
        assert!(
            !r3.is_empty(),
            "expected R3 finding, got: {:?}",
            report.findings
        );
        cleanup(&root);
    }

    // --- R4: phantom member ---

    #[test]
    fn r4_phantom_member_fails() {
        let root = scratch_root("r4");
        write_workspace(&root, &["microservices/ghost/crates/oya-ghost-domain"]);
        // intentionally do NOT write the member dir
        let report = run(&root, true);
        let r4: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule == WorkspaceTopologyRule::R4PhantomMember)
            .collect();
        assert!(
            !r4.is_empty(),
            "expected R4 finding, got: {:?}",
            report.findings
        );
        cleanup(&root);
    }

    // --- R5: orphan crate ---

    #[test]
    fn r5_orphan_crate_under_libs_fails() {
        let root = scratch_root("r5");
        write_workspace(&root, &["microservices/svc"]);
        write_member(&root, "microservices/svc", "oya-svc-app");
        // Create a libs crate that is NOT a member.
        let orphan = root.join("libs").join("oya-check-orphan");
        fs::create_dir_all(orphan.join("src")).expect("orphan dir");
        fs::write(
            orphan.join("Cargo.toml"),
            "[package]\nname = \"oya-check-orphan\"\nedition = \"2024\"\nversion = \"0.1.0\"\n",
        )
        .expect("orphan toml");
        let report = run(&root, true);
        let r5: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule == WorkspaceTopologyRule::R5OrphanCrate)
            .collect();
        assert!(
            !r5.is_empty(),
            "expected R5 finding, got: {:?}",
            report.findings
        );
        cleanup(&root);
    }

    // --- R6: invalid location ---

    #[test]
    fn r6_member_outside_canonical_prefix_fails() {
        let root = scratch_root("r6");
        let bad_path = "vendor/third-party/oya-bad";
        write_workspace(&root, &[bad_path]);
        write_member(&root, bad_path, "oya-bad");
        let report = run(&root, true);
        let r6: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule == WorkspaceTopologyRule::R6InvalidLocation)
            .collect();
        assert!(
            !r6.is_empty(),
            "expected R6 finding, got: {:?}",
            report.findings
        );
        cleanup(&root);
    }

    // --- R7: dir==name ---

    #[test]
    fn r7_dir_name_match_happy_path_passes() {
        let root = scratch_root("r7-happy");
        // dir basename == package name on all three canonical prefix forms.
        write_workspace(
            &root,
            &[
                "microservices/accounting/crates/oya-accounting-domain",
                "libs/oya-shared-types",
                "tools/oya-governance-adr-app",
            ],
        );
        write_member(
            &root,
            "microservices/accounting/crates/oya-accounting-domain",
            "oya-accounting-domain",
        );
        write_member(&root, "libs/oya-shared-types", "oya-shared-types");
        write_member(
            &root,
            "tools/oya-governance-adr-app",
            "oya-governance-adr-app",
        );
        let report = run(&root, true);
        let r7: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule == WorkspaceTopologyRule::R7DirNameMismatch)
            .collect();
        assert!(r7.is_empty(), "expected no R7 findings, got: {:?}", r7);
        cleanup(&root);
    }

    #[test]
    fn r7_dir_name_mismatch_fails() {
        let root = scratch_root("r7-violation");
        // Dir is named "oya-accounting-domain" but package name is "oya-accounting-core" — mismatch.
        let path = "microservices/accounting/crates/oya-accounting-domain";
        write_workspace(&root, &[path]);
        write_member(&root, path, "oya-accounting-core");
        let report = run(&root, true);
        let r7: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule == WorkspaceTopologyRule::R7DirNameMismatch)
            .collect();
        assert!(
            !r7.is_empty(),
            "expected R7 finding, got: {:?}",
            report.findings
        );
        assert!(
            r7[0].detail.contains("oya-accounting-domain"),
            "detail should mention dir basename, got: {}",
            r7[0].detail
        );
        assert!(
            r7[0].detail.contains("oya-accounting-core"),
            "detail should mention package name, got: {}",
            r7[0].detail
        );
        cleanup(&root);
    }

    // --- cloud/ and oya/ roots ---

    #[test]
    fn cloud_svc_crates_path_passes_r6() {
        let root = scratch_root("cloud-r6");
        let path = "cloud/cloud-billing/crates/oya-cloud-billing-kernel";
        write_workspace(&root, &[path]);
        write_member(&root, path, "oya-cloud-billing-kernel");
        let report = run(&root, true);
        let r6: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule == WorkspaceTopologyRule::R6InvalidLocation)
            .collect();
        assert!(
            r6.is_empty(),
            "cloud/<svc>/crates/<crate> should pass R6, got: {r6:?}"
        );
        cleanup(&root);
    }

    #[test]
    fn oya_svc_crates_path_passes_r6() {
        let root = scratch_root("oya-r6");
        let path = "oya/accounting/crates/oya-accounting-journal-domain";
        write_workspace(&root, &[path]);
        write_member(&root, path, "oya-accounting-journal-domain");
        let report = run(&root, true);
        let r6: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule == WorkspaceTopologyRule::R6InvalidLocation)
            .collect();
        assert!(
            r6.is_empty(),
            "oya/<svc>/crates/<crate> should pass R6, got: {r6:?}"
        );
        cleanup(&root);
    }

    #[test]
    fn cloud_orphan_crate_detected() {
        let root = scratch_root("cloud-r5");
        write_workspace(
            &root,
            &["oya/accounting/crates/oya-accounting-journal-domain"],
        );
        write_member(
            &root,
            "oya/accounting/crates/oya-accounting-journal-domain",
            "oya-accounting-journal-domain",
        );
        // Create an orphan cloud crate not in workspace.
        let orphan = root
            .join("cloud")
            .join("cloud-billing")
            .join("crates")
            .join("oya-cloud-billing-kernel");
        fs::create_dir_all(orphan.join("src")).expect("orphan dir");
        fs::write(
            orphan.join("Cargo.toml"),
            "[package]\nname = \"oya-cloud-billing-kernel\"\nedition = \"2024\"\nversion = \"0.1.0\"\n",
        )
        .expect("orphan toml");
        let report = run(&root, true);
        let r5: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule == WorkspaceTopologyRule::R5OrphanCrate)
            .collect();
        assert!(
            !r5.is_empty(),
            "expected R5 orphan finding for cloud crate, got: {:?}",
            report.findings
        );
        cleanup(&root);
    }

    #[test]
    fn excluded_nested_workspace_is_not_reported_as_orphan() {
        let root = scratch_root("nested-workspace-exclude");
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"cloud/*/crates/oya-*\"]\nexclude = [\"cloud/cloud-kernel\"]\n",
        )
        .expect("workspace manifest written");
        write_member(
            &root,
            "cloud/cloud-data/crates/oya-cloud-data-kernel",
            "oya-cloud-data-kernel",
        );
        fs::create_dir_all(root.join("cloud/cloud-kernel")).expect("nested workspace dir");
        fs::write(
            root.join("cloud/cloud-kernel/Cargo.toml"),
            "[workspace]\nmembers = [\"crates/oya-cloud-kernel-frame-kernel\"]\n",
        )
        .expect("nested workspace manifest");
        write_member(
            &root,
            "cloud/cloud-kernel/crates/oya-cloud-kernel-frame-kernel",
            "oya-cloud-kernel-frame-kernel",
        );

        let report = run(&root, true);
        assert!(
            report.findings.is_empty(),
            "excluded nested workspace crates must not be orphaned: {:?}",
            report.findings
        );
        cleanup(&root);
    }
}
