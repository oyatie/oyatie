//! `oya-check-dependency-seam` — ADR-0092 D13 dependency seam lane.
//!
//! The crate keeps the dependency-policy gate reproducible in the current
//! checkout.  Day-1 severity is report-only by default per ADR-0092 D14; callers
//! can promote diagnostics to blocking with `DependencySeamSeverity::Error`.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

pub const D13_SUBCHECKS: [&str; 3] = ["seam-imports", "registry-coverage", "cargo-audit-shell"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DependencySeamSeverity {
    ReportOnly,
    Error,
}

impl DependencySeamSeverity {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "report-only" => Some(Self::ReportOnly),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencySeamConfig {
    pub repo_root: PathBuf,
    pub registry_path: PathBuf,
    pub fixture_root: PathBuf,
    pub evidence_paths: Vec<PathBuf>,
    pub offline: bool,
    pub severity: DependencySeamSeverity,
}

impl DependencySeamConfig {
    pub fn for_repo(repo_root: impl Into<PathBuf>) -> Self {
        let repo_root = repo_root.into();
        Self {
            registry_path: repo_root.join("registry/dependency-rationales.json"),
            fixture_root: repo_root.join("crates/oya-check-dependency-seam/tests/fixtures"),
            repo_root,
            evidence_paths: Vec::new(),
            offline: true,
            severity: DependencySeamSeverity::ReportOnly,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SubcheckStatus {
    Pass,
    ReportOnly,
    Skipped,
    Fail,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticSeverity {
    ReportOnly,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DependencySeamDiagnostic {
    pub subcheck_id: String,
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub path: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DependencySeamSubcheckReport {
    pub id: String,
    pub status: SubcheckStatus,
    pub diagnostics: Vec<DependencySeamDiagnostic>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DependencySeamReport {
    pub subchecks: Vec<DependencySeamSubcheckReport>,
    pub workspace_dependencies: Vec<String>,
    pub registry_entries: Vec<String>,
}

impl DependencySeamReport {
    pub fn blocking_diagnostics(&self) -> Vec<&DependencySeamDiagnostic> {
        self.subchecks
            .iter()
            .flat_map(|subcheck| subcheck.diagnostics.iter())
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
            .collect()
    }

    pub fn diagnostic_count(&self) -> usize {
        self.subchecks
            .iter()
            .map(|subcheck| subcheck.diagnostics.len())
            .sum()
    }

    pub fn status_count(&self, status: SubcheckStatus) -> usize {
        self.subchecks
            .iter()
            .filter(|subcheck| subcheck.status == status)
            .count()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DependencyRationale {
    isolated_in_crate: Option<String>,
    allowed_crates: Option<BTreeSet<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkspaceMember {
    name: String,
    relative_path: String,
    manifest_path: PathBuf,
    src_dir: PathBuf,
    dependencies: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AuditOutcome {
    status: SubcheckStatus,
    diagnostics: Vec<DependencySeamDiagnostic>,
    notes: Vec<String>,
}

pub fn validate_dependency_seam(
    config: &DependencySeamConfig,
) -> Result<DependencySeamReport, String> {
    let workspace_dependencies = read_workspace_dependencies(&config.repo_root.join("Cargo.toml"))?;
    let rationales = read_dependency_rationales(&config.registry_path)?;
    let workspace_members = read_workspace_members(&config.repo_root)?;

    let mut subchecks = Vec::with_capacity(D13_SUBCHECKS.len());
    subchecks.push(check_seam_imports(config, &rationales, &workspace_members));
    subchecks.push(check_registry_coverage(
        config.severity,
        &workspace_dependencies,
        &rationales,
    ));
    subchecks.push(check_cargo_audit_shell(config));

    Ok(DependencySeamReport {
        subchecks,
        workspace_dependencies: workspace_dependencies.into_iter().collect(),
        registry_entries: rationales.into_keys().collect(),
    })
}

fn check_seam_imports(
    config: &DependencySeamConfig,
    rationales: &BTreeMap<String, DependencyRationale>,
    workspace_members: &[WorkspaceMember],
) -> DependencySeamSubcheckReport {
    let mut diagnostics = Vec::new();
    let mut notes = Vec::new();
    for (dependency, rationale) in rationales {
        let Some(allowed_crates) = allowed_crates_for_rationale(rationale) else {
            diagnostics.push(diagnostic(
                "seam-imports",
                "SEAM_SCOPE_UNPARSEABLE",
                config.severity,
                format!(
                    "{dependency} lacks machine-readable allowed_crates and has non-exact isolated_in_crate scope"
                ),
                Some(config.registry_path.display().to_string()),
            ));
            continue;
        };
        notes.push(format!(
            "{dependency}: allowed_crates={}",
            allowed_crates.iter().cloned().collect::<Vec<_>>().join(",")
        ));
        let import_token = dependency.replace('-', "_");
        for member in workspace_members {
            if allowed_crates.contains(&member.name) {
                continue;
            }
            if member.dependencies.contains(dependency) {
                diagnostics.push(diagnostic(
                    "seam-imports",
                    "SEAM_DEP_DECL_OUTSIDE_ISOLATED_CRATE",
                    config.severity,
                    format!(
                        "{} declares {} outside allowed crates [{}]",
                        member.name,
                        dependency,
                        allowed_crates.iter().cloned().collect::<Vec<_>>().join(",")
                    ),
                    Some(member.manifest_path.display().to_string()),
                ));
            }
            let import_paths = rust_files_containing_import(&member.src_dir, &import_token);
            for path in import_paths {
                diagnostics.push(diagnostic(
                    "seam-imports",
                    "SEAM_IMPORT_OUTSIDE_ISOLATED_CRATE",
                    config.severity,
                    format!(
                        "{} imports {} outside allowed crates [{}]",
                        member.name,
                        import_token,
                        allowed_crates.iter().cloned().collect::<Vec<_>>().join(",")
                    ),
                    Some(path.display().to_string()),
                ));
            }
        }
    }
    subcheck("seam-imports", diagnostics, notes)
}

fn check_registry_coverage(
    severity: DependencySeamSeverity,
    workspace_dependencies: &BTreeSet<String>,
    rationales: &BTreeMap<String, DependencyRationale>,
) -> DependencySeamSubcheckReport {
    let mut diagnostics = Vec::new();
    for dependency in workspace_dependencies {
        if !rationales.contains_key(dependency) {
            diagnostics.push(diagnostic(
                "registry-coverage",
                "REGISTRY_ROW_MISSING",
                severity,
                format!("workspace dependency {dependency} lacks dependency-rationales row"),
                Some("registry/dependency-rationales.json".to_string()),
            ));
        }
    }
    for dependency in rationales.keys() {
        if !workspace_dependencies.contains(dependency) {
            diagnostics.push(diagnostic(
                "registry-coverage",
                "REGISTRY_ROW_ORPHAN",
                severity,
                format!("dependency-rationales row {dependency} has no workspace dependency"),
                Some("registry/dependency-rationales.json".to_string()),
            ));
        }
    }
    subcheck("registry-coverage", diagnostics, Vec::new())
}

fn check_cargo_audit_shell(config: &DependencySeamConfig) -> DependencySeamSubcheckReport {
    if config.offline {
        return DependencySeamSubcheckReport {
            id: "cargo-audit-shell".to_string(),
            status: SubcheckStatus::Skipped,
            diagnostics: Vec::new(),
            notes: vec!["CARGO_AUDIT_OFFLINE_SKIPPED: offline mode avoids network/tool bootstrap side effects".to_string()],
        };
    }
    let outcome = run_cargo_audit(config);
    DependencySeamSubcheckReport {
        id: "cargo-audit-shell".to_string(),
        status: outcome.status,
        diagnostics: outcome.diagnostics,
        notes: outcome.notes,
    }
}

fn run_cargo_audit(config: &DependencySeamConfig) -> AuditOutcome {
    let output = Command::new("cargo")
        .args(cargo_audit_args(config.offline))
        .current_dir(&config.repo_root)
        .env_remove("RUSTC_WRAPPER")
        .output();
    match output {
        Ok(output) if output.status.success() => AuditOutcome {
            status: SubcheckStatus::Pass,
            diagnostics: Vec::new(),
            notes: vec!["cargo audit exited 0".to_string()],
        },
        Ok(output) => AuditOutcome {
            status: status_for(config.severity),
            diagnostics: vec![diagnostic(
                "cargo-audit-shell",
                "CARGO_AUDIT_NONZERO",
                config.severity,
                format!("cargo audit exited nonzero: {}", output.status),
                None,
            )],
            notes: vec![String::from_utf8_lossy(&output.stderr).trim().to_string()],
        },
        Err(error) => AuditOutcome {
            status: status_for(config.severity),
            diagnostics: vec![diagnostic(
                "cargo-audit-shell",
                "CARGO_AUDIT_UNAVAILABLE",
                config.severity,
                format!("cargo audit could not be started: {error}"),
                None,
            )],
            notes: Vec::new(),
        },
    }
}

fn cargo_audit_args(offline: bool) -> Vec<&'static str> {
    if offline {
        vec!["audit", "--no-fetch", "--stale"]
    } else {
        vec!["audit", "--stale"]
    }
}

fn subcheck(
    id: &str,
    diagnostics: Vec<DependencySeamDiagnostic>,
    notes: Vec<String>,
) -> DependencySeamSubcheckReport {
    let status = if diagnostics.is_empty() {
        SubcheckStatus::Pass
    } else if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    {
        SubcheckStatus::Fail
    } else {
        SubcheckStatus::ReportOnly
    };
    DependencySeamSubcheckReport {
        id: id.to_string(),
        status,
        diagnostics,
        notes,
    }
}

fn diagnostic(
    subcheck_id: &str,
    code: &str,
    severity: DependencySeamSeverity,
    message: String,
    path: Option<String>,
) -> DependencySeamDiagnostic {
    DependencySeamDiagnostic {
        subcheck_id: subcheck_id.to_string(),
        code: code.to_string(),
        severity: severity_for(severity),
        message,
        path,
    }
}

fn severity_for(severity: DependencySeamSeverity) -> DiagnosticSeverity {
    match severity {
        DependencySeamSeverity::ReportOnly => DiagnosticSeverity::ReportOnly,
        DependencySeamSeverity::Error => DiagnosticSeverity::Error,
    }
}

fn status_for(severity: DependencySeamSeverity) -> SubcheckStatus {
    match severity {
        DependencySeamSeverity::ReportOnly => SubcheckStatus::ReportOnly,
        DependencySeamSeverity::Error => SubcheckStatus::Fail,
    }
}

fn read_workspace_dependencies(workspace_manifest: &Path) -> Result<BTreeSet<String>, String> {
    let manifest = read_toml_file(workspace_manifest)?;
    let Some(dependencies) = manifest
        .get("workspace")
        .and_then(TomlValue::as_table)
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(TomlValue::as_table)
    else {
        return Ok(BTreeSet::new());
    };
    Ok(dependencies.keys().cloned().collect())
}

fn read_dependency_rationales(
    registry_path: &Path,
) -> Result<BTreeMap<String, DependencyRationale>, String> {
    let value = read_json_file(registry_path)?;
    let entries = value
        .get("entries")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| {
            format!(
                "dependency rationales registry {} lacks entries object",
                registry_path.display()
            )
        })?;
    let mut out = BTreeMap::new();
    for (dependency, entry) in entries {
        let isolated_in_crate = entry
            .get("isolated_in_crate")
            .and_then(JsonValue::as_str)
            .map(str::to_string);
        let allowed_crates = entry
            .get("allowed_crates")
            .and_then(JsonValue::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(JsonValue::as_str)
                    .map(str::to_string)
                    .collect::<BTreeSet<_>>()
            });
        out.insert(
            dependency.clone(),
            DependencyRationale {
                isolated_in_crate,
                allowed_crates,
            },
        );
    }
    Ok(out)
}

fn allowed_crates_for_rationale(rationale: &DependencyRationale) -> Option<BTreeSet<String>> {
    if let Some(allowed_crates) = &rationale.allowed_crates {
        return Some(allowed_crates.clone());
    }
    rationale
        .isolated_in_crate
        .as_deref()
        .and_then(exact_crate_scope)
        .map(|allowed_crate| BTreeSet::from([allowed_crate.to_string()]))
}

fn read_workspace_members(repo_root: &Path) -> Result<Vec<WorkspaceMember>, String> {
    // Resolve the concrete member dirs via the canonical glob-aware resolver. The root
    // manifest lists members as GLOBS (`libs/oya-*`, ...); reading the array textually
    // here would yield `*` literals whose join()ed Cargo.toml paths never exist, so the
    // seam check would silently see ZERO members. Reuse, not re-derive.
    let member_dirs = oya_workspace_members_kernel::resolve_member_dirs(repo_root)
        .map_err(|error| error.to_string())?;
    let mut out = Vec::new();
    for relative_path in member_dirs {
        let manifest_path = repo_root.join(&relative_path).join("Cargo.toml");
        if !manifest_path.is_file() {
            continue;
        }
        let package_manifest = read_toml_file(&manifest_path)?;
        let Some(name) = package_manifest
            .get("package")
            .and_then(TomlValue::as_table)
            .and_then(|package| package.get("name"))
            .and_then(TomlValue::as_str)
        else {
            continue;
        };
        let dependencies = package_dependencies(&package_manifest);
        out.push(WorkspaceMember {
            name: name.to_string(),
            relative_path: relative_path.to_string(),
            manifest_path,
            src_dir: repo_root.join(relative_path).join("src"),
            dependencies,
        });
    }
    Ok(out)
}

fn package_dependencies(manifest: &TomlValue) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for table_name in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = manifest.get(table_name).and_then(TomlValue::as_table) {
            out.extend(table.keys().cloned());
        }
    }
    out
}

fn exact_crate_scope(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        Some(trimmed)
    } else {
        None
    }
}

fn rust_files_containing_import(src_dir: &Path, import_token: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_rust_imports(src_dir, import_token, &mut out);
    out.sort();
    out
}

fn collect_rust_imports(dir: &Path, import_token: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_imports(&path, import_token, out);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };
        if contains_rust_import(&contents, import_token) {
            out.push(path);
        }
    }
}

fn contains_rust_import(contents: &str, import_token: &str) -> bool {
    let prefix = format!("use {import_token}::");
    let path = format!("{import_token}::");
    let mut in_cfg_test_tail = false;
    for line in contents.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#[cfg(test)]") {
            // Test modules frequently carry adversarial fixture strings that
            // intentionally mention forbidden imports; production seam checks
            // must not count those as runtime imports.
            in_cfg_test_tail = true;
            continue;
        }
        if in_cfg_test_tail || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with(&prefix) || trimmed.contains(&path) {
            return true;
        }
    }
    false
}

fn read_toml_file(path: &Path) -> Result<TomlValue, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("TOML file unreadable {}: {error}", path.display()))?;
    contents
        .parse::<TomlValue>()
        .map_err(|error| format!("TOML file invalid {}: {error}", path.display()))
}

fn read_json_file(path: &Path) -> Result<JsonValue, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("JSON file unreadable {}: {error}", path.display()))?;
    serde_json::from_str::<JsonValue>(&contents)
        .map_err(|error| format!("JSON file invalid {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn run_composite_returns_three_d13_sub_checks_in_canonical_order() {
        let root = fixture_repo("canonical-order");
        write_valid_repo(&root);
        let config = DependencySeamConfig::for_repo(&root);

        let report = validate_dependency_seam(&config).expect("report");
        let ids = report
            .subchecks
            .iter()
            .map(|subcheck| subcheck.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, D13_SUBCHECKS);
        cleanup(&root);
    }

    #[test]
    fn seam_imports_reports_dependency_imported_outside_isolated_crate() {
        let root = fixture_repo("seam-import-violation");
        write_valid_repo(&root);
        write_member(
            &root,
            "crates/offender-kernel",
            "offender-kernel",
            "[dependencies]\nhyper.workspace = true\n",
            "pub fn x() { let _ = hyper::Version::HTTP_11; }\n",
        );
        append_workspace_member(&root, "crates/offender-kernel");
        let mut config = DependencySeamConfig::for_repo(&root);
        config.severity = DependencySeamSeverity::Error;

        let report = validate_dependency_seam(&config).expect("report");
        assert_has_code(&report, "SEAM_IMPORT_OUTSIDE_ISOLATED_CRATE");
        assert_has_code(&report, "SEAM_DEP_DECL_OUTSIDE_ISOLATED_CRATE");
        assert!(!report.blocking_diagnostics().is_empty());
        cleanup(&root);
    }

    #[test]
    fn seam_imports_reports_unparseable_scope_without_allowed_crates() {
        let root = fixture_repo("unparseable-scope");
        write_valid_repo(&root);
        write_registry(
            &root,
            r#"{"entries":{"hyper":{"isolated_in_crate":"adapter-layer crates"},"serde":{"allowed_crates":[]}}}"#,
        );
        let config = DependencySeamConfig::for_repo(&root);

        let report = validate_dependency_seam(&config).expect("report");
        assert_has_code(&report, "SEAM_SCOPE_UNPARSEABLE");
        cleanup(&root);
    }

    #[test]
    fn registry_coverage_reports_missing_and_orphan_rows() {
        let root = fixture_repo("registry-coverage");
        write_workspace(&root, &["crates/adapter"]);
        write_member(
            &root,
            "crates/adapter",
            "adapter",
            "[dependencies]\nhyper.workspace = true\nserde.workspace = true\n",
            "pub fn adapter() {}\n",
        );
        write_registry(
            &root,
            r#"{
              "entries": {
                "hyper": { "isolated_in_crate": "adapter" },
                "orphan": { "isolated_in_crate": "adapter" }
              }
            }"#,
        );
        let config = DependencySeamConfig::for_repo(&root);

        let report = validate_dependency_seam(&config).expect("report");
        assert_has_code(&report, "REGISTRY_ROW_MISSING");
        assert_has_code(&report, "REGISTRY_ROW_ORPHAN");
        cleanup(&root);
    }

    #[test]
    fn live_registry_authorizes_app_shell_render_envelope_boundary() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("repo root")
            .to_path_buf();
        let rationales =
            read_dependency_rationales(&repo_root.join("registry/dependency-rationales.json"))
                .expect("live dependency rationales");

        for dependency in ["serde", "serde_json"] {
            let allowed = rationales
                .get(dependency)
                .and_then(allowed_crates_for_rationale)
                .unwrap_or_default();
            assert!(
                allowed.contains("oya-application-shell-frontend"),
                "{dependency} must explicitly allow only the app-shell render-envelope boundary"
            );
        }
    }

    #[test]
    fn cargo_audit_shell_skips_without_failure_in_offline_mode() {
        let root = fixture_repo("offline-audit");
        write_valid_repo(&root);
        let mut config = DependencySeamConfig::for_repo(&root);
        config.offline = true;

        let report = validate_dependency_seam(&config).expect("report");
        let audit = report
            .subchecks
            .iter()
            .find(|subcheck| subcheck.id == "cargo-audit-shell")
            .expect("audit subcheck");
        assert_eq!(audit.status, SubcheckStatus::Skipped);
        assert!(report.blocking_diagnostics().is_empty());
        cleanup(&root);
    }

    #[test]
    fn online_cargo_audit_does_not_require_preseeded_advisory_db() {
        assert_eq!(
            cargo_audit_args(true),
            vec!["audit", "--no-fetch", "--stale"]
        );
        assert_eq!(cargo_audit_args(false), vec!["audit", "--stale"]);
    }

    fn assert_has_code(report: &DependencySeamReport, code: &str) {
        assert!(
            report
                .subchecks
                .iter()
                .flat_map(|subcheck| subcheck.diagnostics.iter())
                .any(|diagnostic| diagnostic.code == code),
            "expected diagnostic code {code}; report={report:?}"
        );
    }

    fn fixture_repo(slug: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("oya-dependency-seam-{slug}-{nanos}"));
        fs::create_dir_all(&root).expect("fixture root");
        root
    }

    fn cleanup(root: &Path) {
        let _ = fs::remove_dir_all(root);
    }

    fn write_valid_repo(root: &Path) {
        write_workspace(root, &["crates/adapter"]);
        write_member(
            root,
            "crates/adapter",
            "adapter",
            "[dependencies]\nhyper.workspace = true\n",
            "pub fn adapter() { let _ = hyper::Version::HTTP_11; }\n",
        );
        write_registry(
            root,
            r#"{"entries":{"hyper":{"isolated_in_crate":"adapter","allowed_crates":["adapter"]},"serde":{"isolated_in_crate":"adapter-layer crates","allowed_crates":[]}}}"#,
        );
    }

    fn write_workspace(root: &Path, members: &[&str]) {
        let members = members
            .iter()
            .map(|member| format!("\"{member}\""))
            .collect::<Vec<_>>()
            .join(", ");
        fs::write(
            root.join("Cargo.toml"),
            format!("[workspace]\nmembers = [{members}]\n[workspace.dependencies]\nhyper = \"1\"\nserde = \"1\"\n"),
        )
        .expect("workspace");
    }

    fn append_workspace_member(root: &Path, member: &str) {
        let path = root.join("Cargo.toml");
        let current = fs::read_to_string(&path).expect("workspace read");
        let replaced = current.replace(
            "members = [\"crates/adapter\"]",
            &format!("members = [\"crates/adapter\", \"{member}\"]"),
        );
        fs::write(path, replaced).expect("workspace update");
    }

    fn write_member(root: &Path, path: &str, name: &str, deps: &str, lib: &str) {
        let crate_dir = root.join(path);
        fs::create_dir_all(crate_dir.join("src")).expect("crate dir");
        fs::write(
            crate_dir.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{name}\"\nedition = \"2024\"\nversion = \"0.1.0\"\nlicense = \"Apache-2.0\"\n{deps}"
            ),
        )
        .expect("manifest");
        fs::write(crate_dir.join("src/lib.rs"), lib).expect("lib");
    }

    fn write_registry(root: &Path, body: &str) {
        let registry = root.join("registry");
        fs::create_dir_all(&registry).expect("registry dir");
        fs::write(registry.join("dependency-rationales.json"), body).expect("registry");
    }
}
