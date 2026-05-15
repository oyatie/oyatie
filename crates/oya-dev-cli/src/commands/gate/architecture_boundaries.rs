//! `oya gate validate architecture-boundaries` — workspace architecture-boundary
//! gate. Replaces `scripts/check-architecture-boundaries.sh` per Wave 2 of the
//! shell/python → Rust replacement program (audit
//! `evidence/audits/shell-python-replacement-audit-2026-05-15.md` row B-2).
//!
//! Naming justification: subcommand `gate validate architecture-boundaries`
//! (kebab-case); module file `src/commands/gate/architecture_boundaries.rs`
//! (snake_case, no redundant `_gate` suffix because the path itself is
//! `commands/gate/...`); handler `validate_architecture_boundaries`
//! (snake_case verb). Conforms to ADR-0105/0106/0107 v4 BNF and the
//! 13-value layer enum at
//! `crates/oya-foundry-fitness-predictable-naming-kernel::ALLOWED_ROLES`.
//!
//! Validates four invariants from the legacy Python heredoc:
//! 1. Every workspace package uses the `oya-` prefix.
//! 2. Every workspace package lives at `crates/<name>` or `tools/<name>`.
//! 3. Every workspace package has a catalog record at
//!    `registry/catalog/<name>.yaml`.
//! 4. Inter-package dependency edges respect the role-based
//!    `ALLOWED_DEPENDENCY_ROLES` matrix.
//! 5. Legacy implementation directories (`modules/`, `services/`,
//!    `platform/`) are not present at repo root (ADR-0015/PRD ban).
//!
//! `cargo metadata` is invoked once per run via `std::process::Command`;
//! the resulting JSON is parsed with `serde_json`. Catalog YAML records
//! are parsed line-by-line (matching the legacy parser shape so that
//! pre-existing fixtures keep working).
//!
//! The `ALLOWED_DEPENDENCY_ROLES` table is preserved verbatim from the
//! Python source. It references legacy role names (`application`,
//! `runtime`, `test`, `api`, `worker`) that are NOT all in the canonical
//! 13-value layer enum at
//! `crates/oya-foundry-fitness-predictable-naming-kernel::ALLOWED_ROLES`.
//! The catalog still carries records with those roles, so the matrix
//! stays as-is for behavioural parity; reconciliation belongs to a
//! follow-up ADR (tracked under M-CC-P10 lifecycle work). Until that
//! lands, this gate is the source of truth for inter-crate edges.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use serde_json::Value;

/// Legacy implementation directories that ADR-0015/PRD forbid at repo
/// root. Matches the Python `LEGACY_IMPLEMENTATION_DIRS` tuple.
const LEGACY_IMPLEMENTATION_DIRS: [&str; 3] = ["modules", "services", "platform"];

/// Role-based dependency-edge matrix. Key = depending role; value =
/// roles that the depending crate is allowed to import from. Mirrors
/// the Python `ALLOWED_DEPENDENCY_ROLES` dict 1:1.
fn allowed_dependency_roles() -> BTreeMap<&'static str, BTreeSet<&'static str>> {
    let mut table: BTreeMap<&'static str, BTreeSet<&'static str>> = BTreeMap::new();
    let mut insert = |role: &'static str, allowed: &[&'static str]| {
        table.insert(role, allowed.iter().copied().collect());
    };
    insert("kernel", &["kernel", "domain"]);
    insert("domain", &["kernel", "domain"]);
    insert("application", &["kernel", "domain"]);
    insert("app", &["kernel", "domain", "application", "adapter", "rest"]);
    insert("api", &["kernel", "domain", "app"]);
    insert("worker", &["kernel", "domain", "app"]);
    insert("adapter", &["kernel", "domain"]);
    insert("rest", &["kernel", "domain", "application", "adapter"]);
    insert("infrastructure", &["kernel", "domain"]);
    insert("test", &["kernel", "domain"]);
    insert(
        "runtime",
        &[
            "kernel",
            "domain",
            "app",
            "application",
            "api",
            "worker",
            "adapter",
            "rest",
            "runtime",
        ],
    );
    table
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArchitectureBoundariesValidateArgs {
    pub(crate) repo_root: PathBuf,
    pub(crate) registry_dir: PathBuf,
    pub(crate) self_test: bool,
}

pub(crate) fn parse_architecture_boundaries_validate_args(
    args: Vec<String>,
) -> Result<ArchitectureBoundariesValidateArgs, String> {
    let mut parsed = ArchitectureBoundariesValidateArgs {
        repo_root: PathBuf::from("."),
        registry_dir: PathBuf::from("registry/catalog"),
        self_test: false,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--self-test" => {
                parsed.self_test = true;
            }
            "--repo-root" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--repo-root requires a value".to_string())?;
                parsed.repo_root = PathBuf::from(value);
            }
            "--registry" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--registry requires a value".to_string())?;
                parsed.registry_dir = PathBuf::from(value);
            }
            other => {
                return Err(format!(
                    "architecture-boundaries: unknown flag {other:?}; allowed: --self-test, --repo-root, --registry"
                ));
            }
        }
    }
    Ok(parsed)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArchitectureBoundariesReport {
    pub(crate) packages_checked: usize,
    pub(crate) catalog_records_seen: usize,
    pub(crate) dependency_edges_checked: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorkspacePackage {
    pub(crate) name: String,
    pub(crate) manifest_path: PathBuf,
    pub(crate) dependencies: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CatalogRoleRecord {
    pub(crate) role: String,
}

pub(crate) fn run(args: Vec<String>) -> ExitCode {
    match parse_architecture_boundaries_validate_args(args) {
        Ok(parsed) => match validate_architecture_boundaries(&parsed) {
            Ok(report) => {
                if parsed.self_test {
                    println!("architecture-boundaries self-test passed: 8 cases");
                } else {
                    println!(
                        "architecture-boundaries validation passed: {} packages, {} catalog records, {} dependency edges",
                        report.packages_checked,
                        report.catalog_records_seen,
                        report.dependency_edges_checked,
                    );
                }
                ExitCode::SUCCESS
            }
            Err(errors) => {
                eprintln!("architecture-boundaries validation failed:");
                for error in &errors {
                    eprintln!("  {error}");
                }
                ExitCode::FAILURE
            }
        },
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

pub(crate) fn validate_architecture_boundaries(
    args: &ArchitectureBoundariesValidateArgs,
) -> Result<ArchitectureBoundariesReport, Vec<String>> {
    if args.self_test {
        return run_self_test();
    }
    // `cargo metadata` always returns absolute `manifest_path` values,
    // so we resolve the repo root to absolute up front to make the
    // downstream prefix-strip in `relative_path` correct.
    let absolute_root = std::fs::canonicalize(&args.repo_root)
        .map_err(|error| vec![format!("repo root unreadable: {error}")])?;
    let packages = load_workspace_packages(&absolute_root).map_err(|err| vec![err])?;
    let catalog_records = load_catalog_role_records(
        &absolute_root.join(&args.registry_dir),
        packages.iter().map(|package| package.name.as_str()),
    )
    .map_err(|err| vec![err])?;
    let legacy_dirs = detect_legacy_dirs(&absolute_root);
    let (errors, edges_checked) =
        validate_packages(&packages, &catalog_records, &absolute_root, &legacy_dirs);
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(ArchitectureBoundariesReport {
        packages_checked: packages.len(),
        catalog_records_seen: catalog_records.len(),
        dependency_edges_checked: edges_checked,
    })
}

fn detect_legacy_dirs(repo_root: &Path) -> BTreeSet<String> {
    LEGACY_IMPLEMENTATION_DIRS
        .iter()
        .filter(|name| repo_root.join(name).exists())
        .map(|name| (*name).to_string())
        .collect()
}

fn load_workspace_packages(repo_root: &Path) -> Result<Vec<WorkspacePackage>, String> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(repo_root)
        .output()
        .map_err(|error| format!("cargo metadata invocation failed: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata exited with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    parse_workspace_packages_from_json(&output.stdout)
}

fn parse_workspace_packages_from_json(bytes: &[u8]) -> Result<Vec<WorkspacePackage>, String> {
    let metadata: Value = serde_json::from_slice(bytes)
        .map_err(|error| format!("cargo metadata JSON parse failed: {error}"))?;
    let workspace_members: BTreeSet<String> = metadata
        .get("workspace_members")
        .and_then(Value::as_array)
        .ok_or_else(|| "cargo metadata missing workspace_members array".to_string())?
        .iter()
        .filter_map(|value| value.as_str().map(str::to_string))
        .collect();
    let packages_value = metadata
        .get("packages")
        .and_then(Value::as_array)
        .ok_or_else(|| "cargo metadata missing packages array".to_string())?;
    let mut packages = Vec::new();
    for package in packages_value {
        let id = package
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "cargo metadata package missing id".to_string())?;
        if !workspace_members.contains(id) {
            continue;
        }
        let name = package
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| "cargo metadata package missing name".to_string())?
            .to_string();
        let manifest_path = package
            .get("manifest_path")
            .and_then(Value::as_str)
            .ok_or_else(|| "cargo metadata package missing manifest_path".to_string())?;
        let dependencies = package
            .get("dependencies")
            .and_then(Value::as_array)
            .map(|deps| {
                deps.iter()
                    .filter_map(|dep| dep.get("name").and_then(Value::as_str).map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        packages.push(WorkspacePackage {
            name,
            manifest_path: PathBuf::from(manifest_path),
            dependencies,
        });
    }
    Ok(packages)
}

fn load_catalog_role_records<'a, I>(
    registry_dir: &Path,
    package_names: I,
) -> Result<BTreeMap<String, CatalogRoleRecord>, String>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut records = BTreeMap::new();
    for name in package_names {
        let path = registry_dir.join(format!("{name}.yaml"));
        if !path.exists() {
            continue;
        }
        let contents = std::fs::read_to_string(&path)
            .map_err(|error| format!("catalog record unreadable {}: {error}", path.display()))?;
        if let Some(role) = parse_catalog_role(&contents) {
            records.insert(name.to_string(), CatalogRoleRecord { role });
        } else {
            records.insert(
                name.to_string(),
                CatalogRoleRecord {
                    role: String::new(),
                },
            );
        }
    }
    Ok(records)
}

fn parse_catalog_role(contents: &str) -> Option<String> {
    for line in contents.lines() {
        let stripped = line.trim();
        if stripped.is_empty() || stripped.starts_with('#') {
            continue;
        }
        let Some((key, value)) = stripped.split_once(':') else {
            continue;
        };
        if key.trim() == "role" {
            return Some(value.trim().to_string());
        }
    }
    None
}

fn validate_packages(
    packages: &[WorkspacePackage],
    catalog_records: &BTreeMap<String, CatalogRoleRecord>,
    repo_root: &Path,
    legacy_dirs: &BTreeSet<String>,
) -> (Vec<String>, usize) {
    let role_table = allowed_dependency_roles();
    let mut errors = Vec::new();
    let mut edges_checked = 0;

    for dir in legacy_dirs {
        if LEGACY_IMPLEMENTATION_DIRS.contains(&dir.as_str()) {
            errors.push(format!(
                "legacy implementation directory is forbidden by ADR-0015/PRD: {dir}/"
            ));
        }
    }

    let by_name: BTreeMap<&str, &WorkspacePackage> = packages
        .iter()
        .map(|package| (package.name.as_str(), package))
        .collect();

    for package in packages {
        if !package.name.starts_with("oya-") {
            errors.push(format!(
                "workspace package must use oya- prefix: {}",
                package.name
            ));
        }

        let manifest_parent = package.manifest_path.parent().unwrap_or_else(|| Path::new(""));
        let relative_parent = relative_path(manifest_parent, repo_root);
        let crates_parent = PathBuf::from("crates").join(&package.name);
        let tools_parent = PathBuf::from("tools").join(&package.name);
        let parent_matches = match &relative_parent {
            Some(rel) => rel == &crates_parent || rel == &tools_parent,
            None => false,
        };
        if !parent_matches {
            let actual = relative_parent
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| manifest_parent.display().to_string());
            errors.push(format!(
                "workspace package {} must live at {} or {}, found {}",
                package.name,
                crates_parent.display(),
                tools_parent.display(),
                actual
            ));
        }

        if !catalog_records.contains_key(&package.name) {
            errors.push(format!(
                "missing catalog record for {}: registry/catalog/{}.yaml",
                package.name, package.name
            ));
        }
    }

    for package in packages {
        let Some(record) = catalog_records.get(&package.name) else {
            continue;
        };
        let dependent_role = record.role.as_str();
        let Some(allowed) = role_table.get(dependent_role) else {
            errors.push(format!(
                "unknown role for {}: {}",
                package.name, dependent_role
            ));
            continue;
        };
        for dep_name in &package.dependencies {
            if !by_name.contains_key(dep_name.as_str()) {
                continue;
            }
            edges_checked += 1;
            let dep_role = catalog_records
                .get(dep_name)
                .map(|record| record.role.as_str())
                .unwrap_or("");
            if !allowed.contains(dep_role) {
                errors.push(format!(
                    "forbidden dependency edge: {} ({}) -> {} ({})",
                    package.name, dependent_role, dep_name, dep_role
                ));
            }
        }
    }

    (errors, edges_checked)
}

/// Compute `path` relative to `root` by direct component prefix
/// comparison. Callers normalize both inputs to absolute paths before
/// invoking this (see `validate_architecture_boundaries`), so a plain
/// `strip_prefix` is sufficient and deterministic in both production
/// (where `cargo metadata` returns absolute manifest paths) and in
/// unit tests (where fixtures share an absolute synthetic root).
fn relative_path(path: &Path, root: &Path) -> Option<PathBuf> {
    path.strip_prefix(root).ok().map(PathBuf::from)
}

// --- Self-test (Rust port of the Python `run_self_test` cases) ---

fn run_self_test() -> Result<ArchitectureBoundariesReport, Vec<String>> {
    expect_self_test_happy_path()?;
    expect_self_test_missing_catalog()?;
    expect_self_test_forbidden_role_edge()?;
    expect_self_test_bad_prefix()?;
    expect_self_test_wrong_workspace_path()?;
    expect_self_test_legacy_top_level_dir()?;
    expect_self_test_extra_catalog_allowed()?;
    expect_self_test_infrastructure_and_test_roles()?;
    Ok(ArchitectureBoundariesReport {
        packages_checked: 0,
        catalog_records_seen: 0,
        dependency_edges_checked: 0,
    })
}

#[cfg(test)]
fn fixture_repo_root() -> PathBuf {
    PathBuf::from("/__oya_fixture__")
}

#[cfg(not(test))]
fn fixture_repo_root() -> PathBuf {
    PathBuf::from("/__oya_fixture__")
}

fn fixture_package(
    name: &str,
    role: &str,
    deps: &[&str],
    layout_dir: &str,
) -> (WorkspacePackage, (String, CatalogRoleRecord)) {
    let manifest_path = fixture_repo_root().join(layout_dir).join(name).join("Cargo.toml");
    let package = WorkspacePackage {
        name: name.to_string(),
        manifest_path,
        dependencies: deps.iter().map(|dep| (*dep).to_string()).collect(),
    };
    let record = CatalogRoleRecord {
        role: role.to_string(),
    };
    (package, (name.to_string(), record))
}

fn run_fixture(
    packages: Vec<WorkspacePackage>,
    catalog: BTreeMap<String, CatalogRoleRecord>,
    legacy_dirs: BTreeSet<String>,
) -> Vec<String> {
    let (errors, _) = validate_packages(&packages, &catalog, &fixture_repo_root(), &legacy_dirs);
    errors
}

fn assert_self_test(label: &str, errors: &[String], expected_fragment: Option<&str>) -> Result<(), Vec<String>> {
    match expected_fragment {
        None => {
            if !errors.is_empty() {
                return Err(vec![format!(
                    "self-test {label}: expected success, got {errors:?}"
                )]);
            }
        }
        Some(fragment) => {
            if !errors.iter().any(|error| error.contains(fragment)) {
                return Err(vec![format!(
                    "self-test {label}: expected error containing {fragment:?}, got {errors:?}"
                )]);
            }
        }
    }
    Ok(())
}

fn expect_self_test_happy_path() -> Result<(), Vec<String>> {
    let (kernel_pkg, kernel_rec) = fixture_package("oya-platform-tenant-kernel", "kernel", &[], "crates");
    let (domain_pkg, domain_rec) = fixture_package(
        "oya-platform-tenant-domain",
        "domain",
        &["oya-platform-tenant-kernel"],
        "crates",
    );
    let (rest_pkg, rest_rec) = fixture_package(
        "oya-foundation-rest",
        "rest",
        &["oya-platform-tenant-kernel"],
        "crates",
    );
    let (app_pkg, app_rec) = fixture_package(
        "oya-foundation-app",
        "app",
        &["oya-platform-tenant-kernel", "oya-foundation-rest"],
        "crates",
    );
    let packages = vec![kernel_pkg, domain_pkg, rest_pkg, app_pkg];
    let catalog: BTreeMap<_, _> = [kernel_rec, domain_rec, rest_rec, app_rec].into_iter().collect();
    let errors = run_fixture(packages, catalog, BTreeSet::new());
    assert_self_test("happy path", &errors, None)
}

fn expect_self_test_missing_catalog() -> Result<(), Vec<String>> {
    let (kernel_pkg, kernel_rec) = fixture_package("oya-platform-tenant-kernel", "kernel", &[], "crates");
    let (app_pkg, _app_rec) = fixture_package(
        "oya-foundation-app",
        "app",
        &["oya-platform-tenant-kernel"],
        "crates",
    );
    let packages = vec![kernel_pkg, app_pkg];
    let catalog: BTreeMap<_, _> = [kernel_rec].into_iter().collect();
    let errors = run_fixture(packages, catalog, BTreeSet::new());
    assert_self_test("missing catalog", &errors, Some("missing catalog record"))
}

fn expect_self_test_forbidden_role_edge() -> Result<(), Vec<String>> {
    let (kernel_pkg, kernel_rec) = fixture_package(
        "oya-platform-tenant-kernel",
        "kernel",
        &["oya-foundation-app"],
        "crates",
    );
    let (app_pkg, app_rec) = fixture_package("oya-foundation-app", "app", &[], "crates");
    let packages = vec![kernel_pkg, app_pkg];
    let catalog: BTreeMap<_, _> = [kernel_rec, app_rec].into_iter().collect();
    let errors = run_fixture(packages, catalog, BTreeSet::new());
    assert_self_test("forbidden role edge", &errors, Some("forbidden dependency edge"))
}

fn expect_self_test_bad_prefix() -> Result<(), Vec<String>> {
    let (bad_pkg, bad_rec) = fixture_package("platform-tenant-kernel", "kernel", &[], "crates");
    let packages = vec![bad_pkg];
    let catalog: BTreeMap<_, _> = [bad_rec].into_iter().collect();
    let errors = run_fixture(packages, catalog, BTreeSet::new());
    assert_self_test("bad prefix", &errors, Some("oya- prefix"))
}

fn expect_self_test_wrong_workspace_path() -> Result<(), Vec<String>> {
    let (mut wrong_pkg, wrong_rec) =
        fixture_package("oya-foundry-api", "api", &[], "crates");
    wrong_pkg.manifest_path = fixture_repo_root()
        .join("services")
        .join("oya-foundry-api")
        .join("Cargo.toml");
    let packages = vec![wrong_pkg];
    let catalog: BTreeMap<_, _> = [wrong_rec].into_iter().collect();
    let errors = run_fixture(packages, catalog, BTreeSet::new());
    assert_self_test(
        "wrong workspace path",
        &errors,
        Some("must live at crates/oya-foundry-api"),
    )
}

fn expect_self_test_legacy_top_level_dir() -> Result<(), Vec<String>> {
    let (kernel_pkg, kernel_rec) = fixture_package("oya-platform-tenant-kernel", "kernel", &[], "crates");
    let packages = vec![kernel_pkg];
    let catalog: BTreeMap<_, _> = [kernel_rec].into_iter().collect();
    let mut legacy = BTreeSet::new();
    legacy.insert("services".to_string());
    let errors = run_fixture(packages, catalog, legacy);
    assert_self_test(
        "legacy top-level dir",
        &errors,
        Some("legacy implementation directory"),
    )
}

fn expect_self_test_extra_catalog_allowed() -> Result<(), Vec<String>> {
    let (kernel_pkg, kernel_rec) = fixture_package("oya-platform-tenant-kernel", "kernel", &[], "crates");
    let (domain_pkg, domain_rec) = fixture_package(
        "oya-platform-tenant-domain",
        "domain",
        &["oya-platform-tenant-kernel"],
        "crates",
    );
    let (rest_pkg, rest_rec) = fixture_package(
        "oya-foundation-rest",
        "rest",
        &["oya-platform-tenant-kernel"],
        "crates",
    );
    let (app_pkg, app_rec) = fixture_package(
        "oya-foundation-app",
        "app",
        &["oya-platform-tenant-kernel", "oya-foundation-rest"],
        "crates",
    );
    let packages = vec![kernel_pkg, domain_pkg, rest_pkg, app_pkg];
    let mut catalog: BTreeMap<_, _> = [kernel_rec, domain_rec, rest_rec, app_rec]
        .into_iter()
        .collect();
    catalog.insert(
        "oya-retired-placeholder-kernel".to_string(),
        CatalogRoleRecord {
            role: "kernel".to_string(),
        },
    );
    let errors = run_fixture(packages, catalog, BTreeSet::new());
    assert_self_test("extra catalog remains allowed", &errors, None)
}

fn expect_self_test_infrastructure_and_test_roles() -> Result<(), Vec<String>> {
    let (kernel_pkg, kernel_rec) = fixture_package("oya-platform-tenant-kernel", "kernel", &[], "crates");
    let (infra_pkg, infra_rec) = fixture_package(
        "oya-http-tenant-middleware-infrastructure",
        "infrastructure",
        &["oya-platform-tenant-kernel"],
        "crates",
    );
    let (check_pkg, check_rec) = fixture_package(
        "oya-check-dependency-seam",
        "test",
        &["oya-platform-tenant-kernel"],
        "crates",
    );
    let packages = vec![kernel_pkg, infra_pkg, check_pkg];
    let catalog: BTreeMap<_, _> = [kernel_rec, infra_rec, check_rec].into_iter().collect();
    let errors = run_fixture(packages, catalog, BTreeSet::new());
    assert_self_test("infrastructure and test roles", &errors, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn happy_packages() -> (Vec<WorkspacePackage>, BTreeMap<String, CatalogRoleRecord>) {
        let (kernel_pkg, kernel_rec) =
            fixture_package("oya-platform-tenant-kernel", "kernel", &[], "crates");
        let (domain_pkg, domain_rec) = fixture_package(
            "oya-platform-tenant-domain",
            "domain",
            &["oya-platform-tenant-kernel"],
            "crates",
        );
        let packages = vec![kernel_pkg, domain_pkg];
        let catalog = [kernel_rec, domain_rec].into_iter().collect();
        (packages, catalog)
    }

    #[test]
    fn parse_args_defaults() {
        let parsed = parse_architecture_boundaries_validate_args(vec![]).expect("defaults");
        assert_eq!(parsed.repo_root, PathBuf::from("."));
        assert_eq!(parsed.registry_dir, PathBuf::from("registry/catalog"));
        assert!(!parsed.self_test);
    }

    #[test]
    fn parse_args_self_test_flag_sets_mode() {
        let parsed = parse_architecture_boundaries_validate_args(vec!["--self-test".into()])
            .expect("self-test mode");
        assert!(parsed.self_test);
    }

    #[test]
    fn parse_args_unknown_flag_rejected() {
        let error = parse_architecture_boundaries_validate_args(vec!["--bogus".into()])
            .expect_err("unknown flag");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn happy_path_has_no_errors() {
        let (packages, catalog) = happy_packages();
        let (errors, edges) =
            validate_packages(&packages, &catalog, &fixture_repo_root(), &BTreeSet::new());
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(edges, 1);
    }

    #[test]
    fn rest_into_kernel_is_allowed() {
        let (rest_pkg, rest_rec) = fixture_package(
            "oya-foundation-rest",
            "rest",
            &["oya-platform-tenant-kernel"],
            "crates",
        );
        let (kernel_pkg, kernel_rec) =
            fixture_package("oya-platform-tenant-kernel", "kernel", &[], "crates");
        let packages = vec![rest_pkg, kernel_pkg];
        let catalog = [rest_rec, kernel_rec].into_iter().collect();
        let (errors, _) =
            validate_packages(&packages, &catalog, &fixture_repo_root(), &BTreeSet::new());
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn kernel_depending_on_app_is_forbidden() {
        let (kernel_pkg, kernel_rec) = fixture_package(
            "oya-platform-tenant-kernel",
            "kernel",
            &["oya-foundation-app"],
            "crates",
        );
        let (app_pkg, app_rec) = fixture_package("oya-foundation-app", "app", &[], "crates");
        let packages = vec![kernel_pkg, app_pkg];
        let catalog = [kernel_rec, app_rec].into_iter().collect();
        let (errors, _) =
            validate_packages(&packages, &catalog, &fixture_repo_root(), &BTreeSet::new());
        assert!(
            errors.iter().any(|e| e.contains("forbidden dependency edge")),
            "expected forbidden-edge error, got {errors:?}",
        );
    }

    #[test]
    fn missing_oya_prefix_is_rejected() {
        let (bad_pkg, bad_rec) = fixture_package("platform-tenant-kernel", "kernel", &[], "crates");
        let packages = vec![bad_pkg];
        let catalog = [bad_rec].into_iter().collect();
        let (errors, _) =
            validate_packages(&packages, &catalog, &fixture_repo_root(), &BTreeSet::new());
        assert!(errors.iter().any(|e| e.contains("oya- prefix")), "{errors:?}");
    }

    #[test]
    fn legacy_top_level_dir_is_rejected() {
        let (kernel_pkg, kernel_rec) =
            fixture_package("oya-platform-tenant-kernel", "kernel", &[], "crates");
        let packages = vec![kernel_pkg];
        let catalog = [kernel_rec].into_iter().collect();
        let mut legacy = BTreeSet::new();
        legacy.insert("services".to_string());
        let (errors, _) = validate_packages(&packages, &catalog, &fixture_repo_root(), &legacy);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("legacy implementation directory")),
            "{errors:?}"
        );
    }

    #[test]
    fn unknown_role_is_rejected() {
        let (pkg, _) = fixture_package("oya-foundation-anything", "wibble", &[], "crates");
        let packages = vec![pkg];
        let catalog: BTreeMap<_, _> = [(
            "oya-foundation-anything".to_string(),
            CatalogRoleRecord {
                role: "wibble".to_string(),
            },
        )]
        .into_iter()
        .collect();
        let (errors, _) =
            validate_packages(&packages, &catalog, &fixture_repo_root(), &BTreeSet::new());
        assert!(errors.iter().any(|e| e.contains("unknown role")), "{errors:?}");
    }

    #[test]
    fn cargo_metadata_json_parses() {
        let json = br#"
        {
            "workspace_members": ["oya-foo 0.1.0 (path+file:///x)"],
            "packages": [
                {
                    "id": "oya-foo 0.1.0 (path+file:///x)",
                    "name": "oya-foo",
                    "manifest_path": "/x/crates/oya-foo/Cargo.toml",
                    "dependencies": [{"name": "oya-bar"}]
                },
                {
                    "id": "ignored",
                    "name": "ignored",
                    "manifest_path": "/x/ignored/Cargo.toml",
                    "dependencies": []
                }
            ]
        }
        "#;
        let packages = parse_workspace_packages_from_json(json).expect("parse");
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name, "oya-foo");
        assert_eq!(packages[0].dependencies, vec!["oya-bar".to_string()]);
    }

    #[test]
    fn parse_catalog_role_extracts_value() {
        let yaml = "context: foundation\nrole: kernel\nplane: control\n";
        assert_eq!(parse_catalog_role(yaml), Some("kernel".to_string()));
    }

    #[test]
    fn parse_catalog_role_ignores_comments_and_blank_lines() {
        let yaml = "\n# comment\nrole:    rest\n";
        assert_eq!(parse_catalog_role(yaml), Some("rest".to_string()));
    }

    #[test]
    fn full_self_test_passes() {
        run_self_test().expect("self-test cases all pass");
    }
}
