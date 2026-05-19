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

pub const D13_SUBCHECKS: [&str; 6] = [
    "seam-imports",
    "registry-coverage",
    "cargo-audit-shell",
    "multispectrum-evidence-attached",
    "fixture-pair-coverage",
    "change-class-declared",
];

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

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
struct FixtureManifest {
    subcheck_id: String,
    fixture_kind: String,
    case_id: String,
    expected_diagnostics: Vec<String>,
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
    subchecks.push(check_multispectrum_evidence_attached(config));
    subchecks.push(check_fixture_pair_coverage(config));
    subchecks.push(check_change_class_declared(config));

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
        .arg("audit")
        .current_dir(&config.repo_root)
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

fn check_multispectrum_evidence_attached(
    config: &DependencySeamConfig,
) -> DependencySeamSubcheckReport {
    if config.evidence_paths.is_empty() {
        return subcheck(
            "multispectrum-evidence-attached",
            vec![diagnostic(
                "multispectrum-evidence-attached",
                "MULTISPECTRUM_EVIDENCE_NOT_SPECIFIED",
                DependencySeamSeverity::ReportOnly,
                "no --evidence path supplied for current changeset".to_string(),
                Some("evidence/multispectrum".to_string()),
            )],
            Vec::new(),
        );
    }
    let mut diagnostics = Vec::new();
    for path in &config.evidence_paths {
        match read_json_file(path) {
            Ok(value) => diagnostics.extend(validate_multispectrum_value(path, &value)),
            Err(message) => diagnostics.push(diagnostic(
                "multispectrum-evidence-attached",
                "MULTISPECTRUM_EVIDENCE_MISSING",
                config.severity,
                message,
                Some(path.display().to_string()),
            )),
        }
    }
    let diagnostics = diagnostics
        .into_iter()
        .map(|mut diagnostic| {
            diagnostic.severity = severity_for(config.severity);
            diagnostic
        })
        .collect();
    subcheck("multispectrum-evidence-attached", diagnostics, Vec::new())
}

fn check_fixture_pair_coverage(config: &DependencySeamConfig) -> DependencySeamSubcheckReport {
    let mut diagnostics = Vec::new();
    for subcheck_id in D13_SUBCHECKS {
        diagnostics.extend(check_fixture_manifest(
            config,
            subcheck_id,
            "passing",
            "FIXTURE_PAIR_PASSING_DIR_MISSING",
        ));
        diagnostics.extend(check_fixture_manifest(
            config,
            subcheck_id,
            "failing",
            "FIXTURE_PAIR_FAILING_DIR_MISSING",
        ));
    }
    subcheck("fixture-pair-coverage", diagnostics, Vec::new())
}

fn check_change_class_declared(config: &DependencySeamConfig) -> DependencySeamSubcheckReport {
    if config.evidence_paths.is_empty() {
        return subcheck(
            "change-class-declared",
            vec![diagnostic(
                "change-class-declared",
                "CHANGE_CLASS_EVIDENCE_NOT_SPECIFIED",
                DependencySeamSeverity::ReportOnly,
                "no --evidence path supplied for change_class_id validation".to_string(),
                Some("evidence/multispectrum".to_string()),
            )],
            Vec::new(),
        );
    }
    let mut diagnostics = Vec::new();
    for path in &config.evidence_paths {
        match read_json_file(path) {
            Ok(value) => match change_class_id(&value) {
                Some(change_class) if canonical_change_class(&change_class) => {}
                Some(change_class) => diagnostics.push(diagnostic(
                    "change-class-declared",
                    "CHANGE_CLASS_NONCANONICAL",
                    config.severity,
                    format!("change_class_id {change_class} is not CC-1..CC-7 canonical"),
                    Some(path.display().to_string()),
                )),
                None => diagnostics.push(diagnostic(
                    "change-class-declared",
                    "CHANGE_CLASS_MISSING",
                    config.severity,
                    "evidence file lacks change_class_id".to_string(),
                    Some(path.display().to_string()),
                )),
            },
            Err(message) => diagnostics.push(diagnostic(
                "change-class-declared",
                "CHANGE_CLASS_EVIDENCE_MISSING",
                config.severity,
                message,
                Some(path.display().to_string()),
            )),
        }
    }
    subcheck("change-class-declared", diagnostics, Vec::new())
}

fn validate_multispectrum_value(path: &Path, value: &JsonValue) -> Vec<DependencySeamDiagnostic> {
    let mut diagnostics = Vec::new();
    if string_field(value, "change_id").is_none() {
        diagnostics.push(report_only_diagnostic(
            "multispectrum-evidence-attached",
            "MULTISPECTRUM_CHANGE_ID_MISSING",
            "evidence file lacks change_id".to_string(),
            Some(path.display().to_string()),
        ));
    }
    let change_class = string_field(value, "change_class_id");
    match change_class.as_deref() {
        Some(change_class) if canonical_change_class(change_class) => {}
        Some(change_class) => diagnostics.push(report_only_diagnostic(
            "multispectrum-evidence-attached",
            "CHANGE_CLASS_NONCANONICAL",
            format!("change_class_id {change_class} is not CC-1..CC-7 canonical"),
            Some(path.display().to_string()),
        )),
        None => diagnostics.push(report_only_diagnostic(
            "multispectrum-evidence-attached",
            "CHANGE_CLASS_MISSING",
            "evidence file lacks change_class_id".to_string(),
            Some(path.display().to_string()),
        )),
    }
    match string_field(value, "git_sha") {
        Some(git_sha) if valid_git_sha(&git_sha) => {}
        Some(git_sha) => diagnostics.push(report_only_diagnostic(
            "multispectrum-evidence-attached",
            "MULTISPECTRUM_GIT_SHA_INVALID",
            format!("git_sha {git_sha} must match ^[0-9a-f]{{7,40}}$"),
            Some(path.display().to_string()),
        )),
        None => diagnostics.push(report_only_diagnostic(
            "multispectrum-evidence-attached",
            "MULTISPECTRUM_GIT_SHA_MISSING",
            "evidence file lacks git_sha".to_string(),
            Some(path.display().to_string()),
        )),
    }
    if !freshness_valid(value) {
        diagnostics.push(report_only_diagnostic(
            "multispectrum-evidence-attached",
            "MULTISPECTRUM_FRESHNESS_MISSING",
            "evidence file lacks integer freshness_unix >= 1700000000".to_string(),
            Some(path.display().to_string()),
        ));
    }
    let Some(facets) = value.get("facets").and_then(JsonValue::as_object) else {
        diagnostics.push(report_only_diagnostic(
            "multispectrum-evidence-attached",
            "MULTISPECTRUM_FACETS_MISSING",
            "evidence file lacks facets object".to_string(),
            Some(path.display().to_string()),
        ));
        return diagnostics;
    };
    for required in REQUIRED_EVIDENCE_FACETS {
        match facets.get(required) {
            Some(facet) => validate_facet_value(
                path,
                required,
                facet,
                change_class.as_deref(),
                &mut diagnostics,
            ),
            None => diagnostics.push(report_only_diagnostic(
                "multispectrum-evidence-attached",
                "MULTISPECTRUM_REQUIRED_FACET_MISSING",
                format!("evidence facets omit {required}"),
                Some(path.display().to_string()),
            )),
        }
    }
    diagnostics
}

const REQUIRED_EVIDENCE_FACETS: [&str; 9] = [
    "F1_linus",
    "F2_hyperscaler",
    "F3_adversarial",
    "F4_ergonomic",
    "F5_quality",
    "F6_alternatives",
    "F7_security",
    "F8_performance",
    "F9_compliance",
];

fn validate_facet_value(
    path: &Path,
    facet_id: &str,
    facet: &JsonValue,
    change_class: Option<&str>,
    diagnostics: &mut Vec<DependencySeamDiagnostic>,
) {
    let Some(object) = facet.as_object() else {
        diagnostics.push(report_only_diagnostic(
            "multispectrum-evidence-attached",
            "MULTISPECTRUM_FACET_NOT_OBJECT",
            format!("evidence facet {facet_id} is not an object"),
            Some(path.display().to_string()),
        ));
        return;
    };
    match object.get("considered").and_then(JsonValue::as_bool) {
        Some(true) => {}
        Some(false) => {
            let reason = object
                .get("not_applicable_reason")
                .and_then(JsonValue::as_str)
                .unwrap_or_default()
                .trim();
            if reason.is_empty() {
                diagnostics.push(report_only_diagnostic(
                    "multispectrum-evidence-attached",
                    "MULTISPECTRUM_NOT_APPLICABLE_REASON_MISSING",
                    format!("evidence facet {facet_id} has considered=false without not_applicable_reason"),
                    Some(path.display().to_string()),
                ));
            }
        }
        None => diagnostics.push(report_only_diagnostic(
            "multispectrum-evidence-attached",
            "MULTISPECTRUM_FACET_CONSIDERED_MISSING",
            format!("evidence facet {facet_id} lacks boolean considered"),
            Some(path.display().to_string()),
        )),
    }
    let Some(expected_rigor) = change_class.and_then(|class| expected_rigor(class, facet_id))
    else {
        return;
    };
    let actual_rigor = object
        .get("rigor")
        .or_else(|| object.get("rigor_required"))
        .and_then(JsonValue::as_str);
    if actual_rigor != Some(expected_rigor) {
        diagnostics.push(report_only_diagnostic(
            "multispectrum-evidence-attached",
            "MULTISPECTRUM_FACET_RIGOR_MISMATCH",
            format!(
                "evidence facet {facet_id} rigor {:?} does not match {expected_rigor} for {change_class:?}",
                actual_rigor
            ),
            Some(path.display().to_string()),
        ));
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

fn report_only_diagnostic(
    subcheck_id: &str,
    code: &str,
    message: String,
    path: Option<String>,
) -> DependencySeamDiagnostic {
    diagnostic(
        subcheck_id,
        code,
        DependencySeamSeverity::ReportOnly,
        message,
        path,
    )
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
    let manifest = read_toml_file(&repo_root.join("Cargo.toml"))?;
    let members = manifest
        .get("workspace")
        .and_then(TomlValue::as_table)
        .and_then(|workspace| workspace.get("members"))
        .and_then(TomlValue::as_array)
        .ok_or_else(|| "workspace Cargo.toml lacks workspace.members".to_string())?;
    let mut out = Vec::new();
    for member in members {
        let Some(relative_path) = member.as_str() else {
            continue;
        };
        let manifest_path = repo_root.join(relative_path).join("Cargo.toml");
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

fn check_fixture_manifest(
    config: &DependencySeamConfig,
    subcheck_id: &str,
    fixture_kind: &str,
    missing_dir_code: &str,
) -> Vec<DependencySeamDiagnostic> {
    let dir = config.fixture_root.join(subcheck_id).join(fixture_kind);
    if !dir.is_dir() {
        return vec![diagnostic(
            "fixture-pair-coverage",
            missing_dir_code,
            config.severity,
            format!("{subcheck_id} missing {fixture_kind} fixture directory"),
            Some(dir.display().to_string()),
        )];
    }
    let manifest_path = dir.join("manifest.json");
    let mut diagnostics = Vec::new();
    let manifest = match read_fixture_manifest(&manifest_path) {
        Ok(manifest) => manifest,
        Err(message) => {
            diagnostics.push(diagnostic(
                "fixture-pair-coverage",
                "FIXTURE_PAIR_MANIFEST_INVALID",
                config.severity,
                message,
                Some(manifest_path.display().to_string()),
            ));
            return diagnostics;
        }
    };
    if manifest.subcheck_id != subcheck_id {
        diagnostics.push(diagnostic(
            "fixture-pair-coverage",
            "FIXTURE_PAIR_SUBCHECK_MISMATCH",
            config.severity,
            format!(
                "fixture {} declares subcheck_id {} instead of {subcheck_id}",
                manifest.case_id, manifest.subcheck_id
            ),
            Some(manifest_path.display().to_string()),
        ));
    }
    if manifest.fixture_kind != fixture_kind {
        diagnostics.push(diagnostic(
            "fixture-pair-coverage",
            "FIXTURE_PAIR_KIND_MISMATCH",
            config.severity,
            format!(
                "fixture {} declares fixture_kind {} instead of {fixture_kind}",
                manifest.case_id, manifest.fixture_kind
            ),
            Some(manifest_path.display().to_string()),
        ));
    }
    match fixture_kind {
        "passing" if !manifest.expected_diagnostics.is_empty() => diagnostics.push(diagnostic(
            "fixture-pair-coverage",
            "FIXTURE_PAIR_PASSING_EXPECTS_DIAGNOSTICS",
            config.severity,
            format!(
                "passing fixture {} declares expected diagnostics",
                manifest.case_id
            ),
            Some(manifest_path.display().to_string()),
        )),
        "failing" if manifest.expected_diagnostics.is_empty() => diagnostics.push(diagnostic(
            "fixture-pair-coverage",
            "FIXTURE_PAIR_FAILING_EXPECTS_NO_DIAGNOSTIC",
            config.severity,
            format!(
                "failing fixture {} does not declare expected diagnostics",
                manifest.case_id
            ),
            Some(manifest_path.display().to_string()),
        )),
        _ => {}
    }
    for expected in &manifest.expected_diagnostics {
        if !known_diagnostic_for_subcheck(subcheck_id, expected) {
            diagnostics.push(diagnostic(
                "fixture-pair-coverage",
                "FIXTURE_PAIR_UNKNOWN_DIAGNOSTIC",
                config.severity,
                format!(
                    "fixture {} expects unknown diagnostic {} for {}",
                    manifest.case_id, expected, subcheck_id
                ),
                Some(manifest_path.display().to_string()),
            ));
        }
    }
    diagnostics
}

fn read_fixture_manifest(path: &Path) -> Result<FixtureManifest, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("fixture manifest unreadable {}: {error}", path.display()))?;
    serde_json::from_str::<FixtureManifest>(&contents)
        .map_err(|error| format!("fixture manifest invalid {}: {error}", path.display()))
}

fn known_diagnostic_for_subcheck(subcheck_id: &str, code: &str) -> bool {
    match subcheck_id {
        "seam-imports" => matches!(
            code,
            "SEAM_SCOPE_UNPARSEABLE"
                | "SEAM_DEP_DECL_OUTSIDE_ISOLATED_CRATE"
                | "SEAM_IMPORT_OUTSIDE_ISOLATED_CRATE"
        ),
        "registry-coverage" => matches!(code, "REGISTRY_ROW_MISSING" | "REGISTRY_ROW_ORPHAN"),
        "cargo-audit-shell" => matches!(
            code,
            "CARGO_AUDIT_NONZERO" | "CARGO_AUDIT_UNAVAILABLE" | "CARGO_AUDIT_OFFLINE_SKIPPED"
        ),
        "multispectrum-evidence-attached" => {
            code.starts_with("MULTISPECTRUM_")
                || matches!(code, "CHANGE_CLASS_MISSING" | "CHANGE_CLASS_NONCANONICAL")
        }
        "fixture-pair-coverage" => code.starts_with("FIXTURE_PAIR_"),
        "change-class-declared" => matches!(
            code,
            "CHANGE_CLASS_MISSING"
                | "CHANGE_CLASS_NONCANONICAL"
                | "CHANGE_CLASS_EVIDENCE_MISSING"
                | "CHANGE_CLASS_EVIDENCE_NOT_SPECIFIED"
        ),
        _ => false,
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

fn string_field(value: &JsonValue, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(JsonValue::as_str)
        .map(str::to_string)
}

fn change_class_id(value: &JsonValue) -> Option<String> {
    string_field(value, "change_class_id")
}

fn canonical_change_class(value: &str) -> bool {
    matches!(
        value,
        "CC-1" | "CC-2" | "CC-3" | "CC-4" | "CC-5" | "CC-6" | "CC-7"
    )
}

fn valid_git_sha(value: &str) -> bool {
    (7..=40).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn freshness_valid(value: &JsonValue) -> bool {
    let Some(freshness) = value
        .get("freshness_unix")
        .and_then(JsonValue::as_i64)
        .or_else(|| {
            value
                .get("freshness_unix")
                .and_then(JsonValue::as_u64)
                .map(|v| v as i64)
        })
    else {
        return false;
    };
    freshness >= 1_700_000_000
}

fn expected_rigor(change_class: &str, facet_id: &str) -> Option<&'static str> {
    match change_class {
        "CC-1" => match facet_id {
            "F1_linus" | "F2_hyperscaler" | "F3_adversarial" | "F4_ergonomic" | "F5_quality"
            | "F6_alternatives" | "F7_security" | "F8_performance" | "F9_compliance" => {
                Some("deep")
            }
            _ => None,
        },
        "CC-2" => match facet_id {
            "F1_linus" | "F2_hyperscaler" | "F3_adversarial" | "F5_quality" | "F6_alternatives"
            | "F7_security" => Some("deep"),
            "F4_ergonomic" => Some("scan"),
            _ => None,
        },
        "CC-3" => match facet_id {
            "F1_linus" | "F4_ergonomic" | "F5_quality" | "F6_alternatives" | "F7_security" => {
                Some("deep")
            }
            "F2_hyperscaler" | "F3_adversarial" => Some("scan"),
            _ => None,
        },
        "CC-4" => match facet_id {
            "F1_linus" | "F4_ergonomic" | "F5_quality" | "F6_alternatives" => Some("deep"),
            "F2_hyperscaler" | "F3_adversarial" | "F7_security" => Some("scan"),
            _ => None,
        },
        "CC-5" => match facet_id {
            "F1_linus" | "F4_ergonomic" | "F6_alternatives" => Some("deep"),
            "F2_hyperscaler" | "F3_adversarial" | "F5_quality" | "F7_security" => Some("scan"),
            _ => None,
        },
        "CC-6" => match facet_id {
            "F7_security" => Some("deep"),
            "F1_linus" | "F2_hyperscaler" | "F3_adversarial" | "F4_ergonomic" | "F5_quality"
            | "F6_alternatives" => Some("scan"),
            _ => None,
        },
        "CC-7" => match facet_id {
            "F3_adversarial" => Some("deep"),
            "F1_linus" | "F2_hyperscaler" | "F4_ergonomic" | "F5_quality" | "F6_alternatives"
            | "F7_security" => Some("scan"),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn run_composite_returns_six_d13_sub_checks_in_canonical_order() {
        let root = fixture_repo("canonical-order");
        write_valid_repo(&root);
        let mut config = DependencySeamConfig::for_repo(&root);
        config.evidence_paths = vec![write_valid_evidence(&root, "CC-7")];
        config.fixture_root = manifest_fixture_root();

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
        config.evidence_paths = vec![write_valid_evidence(&root, "CC-7")];
        config.fixture_root = manifest_fixture_root();
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
        let mut config = DependencySeamConfig::for_repo(&root);
        config.evidence_paths = vec![write_valid_evidence(&root, "CC-7")];
        config.fixture_root = manifest_fixture_root();

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
        let mut config = DependencySeamConfig::for_repo(&root);
        config.evidence_paths = vec![write_valid_evidence(&root, "CC-7")];
        config.fixture_root = manifest_fixture_root();

        let report = validate_dependency_seam(&config).expect("report");
        assert_has_code(&report, "REGISTRY_ROW_MISSING");
        assert_has_code(&report, "REGISTRY_ROW_ORPHAN");
        cleanup(&root);
    }

    #[test]
    fn cargo_audit_shell_skips_without_failure_in_offline_mode() {
        let root = fixture_repo("offline-audit");
        write_valid_repo(&root);
        let mut config = DependencySeamConfig::for_repo(&root);
        config.fixture_root = manifest_fixture_root();
        config.evidence_paths = vec![write_valid_evidence(&root, "CC-7")];
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
    fn multispectrum_evidence_reports_missing_required_facet() {
        let root = fixture_repo("missing-facet");
        write_valid_repo(&root);
        let evidence = root.join("evidence/multispectrum/missing.json");
        fs::create_dir_all(evidence.parent().expect("parent")).expect("evidence dir");
        fs::write(
            &evidence,
            r#"{"change_id":"c","change_class_id":"CC-7","freshness_unix":1,"facets":{"F1_linus":{}}}"#,
        )
        .expect("evidence");
        let mut config = DependencySeamConfig::for_repo(&root);
        config.evidence_paths = vec![evidence];
        config.fixture_root = manifest_fixture_root();

        let report = validate_dependency_seam(&config).expect("report");
        assert_has_code(&report, "MULTISPECTRUM_REQUIRED_FACET_MISSING");
        cleanup(&root);
    }

    #[test]
    fn multispectrum_evidence_rejects_missing_git_sha_and_unreasoned_false_facet() {
        let root = fixture_repo("bad-evidence-schema");
        write_valid_repo(&root);
        let evidence = root.join("evidence/multispectrum/bad-schema.json");
        fs::create_dir_all(evidence.parent().expect("parent")).expect("evidence dir");
        fs::write(
            &evidence,
            r#"{
              "change_id":"c",
              "change_class_id":"CC-2",
              "freshness_unix":1700000000,
              "facets":{
                "F1_linus":{"considered":false,"rigor":"deep"},
                "F2_hyperscaler":{"considered":true,"rigor":"deep"},
                "F3_adversarial":{"considered":true,"rigor":"deep"},
                "F4_ergonomic":{"considered":true,"rigor":"scan"},
                "F5_quality":{"considered":true,"rigor":"deep"},
                "F6_alternatives":{"considered":true,"rigor":"deep"},
                "F7_security":{"considered":true,"rigor":"deep"},
                "F8_performance":{"considered":true},
                "F9_compliance":{"considered":true}
              }
            }"#,
        )
        .expect("evidence");
        let mut config = DependencySeamConfig::for_repo(&root);
        config.evidence_paths = vec![evidence];
        config.fixture_root = manifest_fixture_root();

        let report = validate_dependency_seam(&config).expect("report");
        assert_has_code(&report, "MULTISPECTRUM_GIT_SHA_MISSING");
        assert_has_code(&report, "MULTISPECTRUM_NOT_APPLICABLE_REASON_MISSING");
        cleanup(&root);
    }

    #[test]
    fn change_class_declared_reports_absent_and_noncanonical_ids() {
        let root = fixture_repo("bad-change-class");
        write_valid_repo(&root);
        let evidence_dir = root.join("evidence/multispectrum");
        fs::create_dir_all(&evidence_dir).expect("evidence dir");
        let missing = evidence_dir.join("missing.json");
        let noncanonical = evidence_dir.join("noncanonical.json");
        fs::write(
            &missing,
            r#"{"change_id":"c","freshness_unix":1,"facets":{"F1_linus":{}}}"#,
        )
        .expect("missing change class evidence");
        fs::write(
            &noncanonical,
            r#"{"change_id":"c","change_class_id":"CC-99","freshness_unix":1,"facets":{"F1_linus":{}}}"#,
        )
        .expect("noncanonical change class evidence");
        let mut config = DependencySeamConfig::for_repo(&root);
        config.evidence_paths = vec![missing, noncanonical];
        config.fixture_root = manifest_fixture_root();

        let report = validate_dependency_seam(&config).expect("report");
        assert_has_code(&report, "CHANGE_CLASS_MISSING");
        assert_has_code(&report, "CHANGE_CLASS_NONCANONICAL");
        cleanup(&root);
    }

    #[test]
    fn fixture_pair_coverage_passes_for_shipped_d13_fixtures() {
        let root = fixture_repo("fixture-pair");
        write_valid_repo(&root);
        let mut config = DependencySeamConfig::for_repo(&root);
        config.evidence_paths = vec![write_valid_evidence(&root, "CC-7")];
        config.fixture_root = manifest_fixture_root();

        let report = validate_dependency_seam(&config).expect("report");
        let fixture = report
            .subchecks
            .iter()
            .find(|subcheck| subcheck.id == "fixture-pair-coverage")
            .expect("fixture subcheck");
        assert_eq!(fixture.status, SubcheckStatus::Pass);
        cleanup(&root);
    }

    #[test]
    fn fixture_pair_coverage_rejects_placeholder_dirs_without_manifest() {
        let root = fixture_repo("placeholder-fixtures");
        write_valid_repo(&root);
        let fixture_root = root.join("fixtures");
        for subcheck_id in D13_SUBCHECKS {
            fs::create_dir_all(fixture_root.join(subcheck_id).join("passing"))
                .expect("passing dir");
            fs::create_dir_all(fixture_root.join(subcheck_id).join("failing"))
                .expect("failing dir");
        }
        let mut config = DependencySeamConfig::for_repo(&root);
        config.evidence_paths = vec![write_valid_evidence(&root, "CC-7")];
        config.fixture_root = fixture_root;

        let report = validate_dependency_seam(&config).expect("report");
        assert_has_code(&report, "FIXTURE_PAIR_MANIFEST_INVALID");
        cleanup(&root);
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

    fn manifest_fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
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

    fn write_valid_evidence(root: &Path, change_class: &str) -> PathBuf {
        let path = root.join("evidence/multispectrum/valid.json");
        fs::create_dir_all(path.parent().expect("parent")).expect("evidence dir");
        fs::write(&path, valid_evidence_json(change_class)).expect("valid evidence");
        path
    }

    fn valid_evidence_json(change_class: &str) -> String {
        format!(
            r#"{{
  "change_id": "dependency-seam-test",
  "change_class_id": "{change_class}",
  "git_sha": "abcdef1",
  "freshness_unix": 1700000000,
  "facets": {{
    "F1_linus": {{"considered": true, "rigor": "scan"}},
    "F2_hyperscaler": {{"considered": true, "rigor": "scan"}},
    "F3_adversarial": {{"considered": true, "rigor": "deep"}},
    "F4_ergonomic": {{"considered": true, "rigor": "scan"}},
    "F5_quality": {{"considered": true, "rigor": "scan"}},
    "F6_alternatives": {{"considered": true, "rigor": "scan"}},
    "F7_security": {{"considered": true, "rigor": "scan"}},
    "F8_performance": {{"considered": true}},
    "F9_compliance": {{"considered": true}}
  }}
}}"#
        )
    }
}
