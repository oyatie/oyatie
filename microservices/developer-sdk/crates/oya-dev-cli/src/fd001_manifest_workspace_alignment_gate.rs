//! `oya gate validate fd001-manifest-workspace-alignment` runner.
//!
//! This gate protects FD-001 product fanout from a false green where
//! microservice manifests declare implementation crates that are not present in
//! the Rust workspace yet. It is intentionally report-only-capable so agents can
//! emit reconciliation evidence before the strict fanout lock flips to blocking.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::{Value, json};

use crate::{read_package_name, read_workspace_member_paths};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Fd001ManifestWorkspaceAlignmentValidateArgs {
    pub(crate) repo_root: PathBuf,
    pub(crate) workspace_manifest_path: PathBuf,
    pub(crate) manifest_index_path: PathBuf,
    pub(crate) manifest_paths: Vec<PathBuf>,
    pub(crate) manifest_scope: ManifestScope,
    pub(crate) report_only: bool,
    pub(crate) emit_report_path: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ManifestScope {
    Fd001Material,
    AllManifests,
    ExplicitManifests,
}

impl ManifestScope {
    fn as_str(self) -> &'static str {
        match self {
            Self::Fd001Material => "fd001-material",
            Self::AllManifests => "all-manifests",
            Self::ExplicitManifests => "explicit-manifests",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Fd001ManifestWorkspaceAlignmentReport {
    pub(crate) manifest_count: usize,
    pub(crate) required_crate_count: usize,
    pub(crate) workspace_crate_count: usize,
    pub(crate) missing_crates: Vec<String>,
    pub(crate) manifests: Vec<ManifestAlignmentRow>,
    pub(crate) manifest_scope: ManifestScope,
    pub(crate) report_only: bool,
    pub(crate) emitted_report_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ManifestAlignmentRow {
    pub(crate) microservice: String,
    pub(crate) expected_microservice: Option<String>,
    pub(crate) manifest_path: PathBuf,
    pub(crate) required_crates: Vec<String>,
    pub(crate) present_crates: Vec<String>,
    pub(crate) missing_crates: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ManifestInput {
    path: PathBuf,
    expected_microservice: Option<String>,
}

pub(crate) fn parse_fd001_manifest_workspace_alignment_validate_args(
    args: Vec<String>,
) -> Result<Fd001ManifestWorkspaceAlignmentValidateArgs, String> {
    let mut parsed = Fd001ManifestWorkspaceAlignmentValidateArgs {
        repo_root: PathBuf::from("."),
        workspace_manifest_path: PathBuf::from("Cargo.toml"),
        manifest_index_path: PathBuf::from("specs/microservices/manifests-index.json"),
        manifest_paths: Vec::new(),
        manifest_scope: ManifestScope::Fd001Material,
        report_only: false,
        emit_report_path: None,
    };

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => parsed.repo_root = required_path_value(&mut args, "--repo-root")?,
            "--workspace" => {
                parsed.workspace_manifest_path = required_path_value(&mut args, "--workspace")?
            }
            "--manifest-index" => {
                parsed.manifest_index_path = required_path_value(&mut args, "--manifest-index")?;
            }
            "--manifest" => parsed
                .manifest_paths
                .push(required_path_value(&mut args, "--manifest")?),
            "--all-manifests" => parsed.manifest_scope = ManifestScope::AllManifests,
            "--report-only" => parsed.report_only = true,
            "--emit-report" => {
                parsed.emit_report_path = Some(required_path_value(&mut args, "--emit-report")?);
            }
            _ => {
                return Err(format!(
                    "unknown fd001-manifest-workspace-alignment argument: {arg}"
                ));
            }
        }
    }

    Ok(parsed)
}

pub(crate) fn validate_fd001_manifest_workspace_alignment_gate(
    mut args: Fd001ManifestWorkspaceAlignmentValidateArgs,
) -> Result<Fd001ManifestWorkspaceAlignmentReport, String> {
    if args.report_only && args.emit_report_path.is_none() {
        return Err(
            "--report-only requires --emit-report <repo-relative evidence/... path>".to_string(),
        );
    }
    if !args.manifest_paths.is_empty() && args.manifest_scope == ManifestScope::AllManifests {
        return Err(
            "--all-manifests cannot be combined with --manifest; repeat --manifest for explicit scope"
                .to_string(),
        );
    }
    if let Some(path) = &args.emit_report_path {
        validate_emit_report_path(path)?;
    }

    args.repo_root = normalize_path(Path::new("."), &args.repo_root);
    args.workspace_manifest_path = normalize_path(&args.repo_root, &args.workspace_manifest_path);
    args.manifest_index_path = normalize_path(&args.repo_root, &args.manifest_index_path);
    args.manifest_paths = args
        .manifest_paths
        .into_iter()
        .map(|path| normalize_path(&args.repo_root, &path))
        .collect();
    args.emit_report_path = args
        .emit_report_path
        .map(|path| normalize_path(&args.repo_root, &path));

    let workspace_crates = read_workspace_crate_names(&args.workspace_manifest_path)?;
    let manifest_inputs = if args.manifest_paths.is_empty() {
        read_manifest_index_inputs(
            &args.repo_root,
            &args.manifest_index_path,
            args.manifest_scope,
        )?
    } else {
        args.manifest_paths
            .iter()
            .cloned()
            .map(|path| ManifestInput {
                path,
                expected_microservice: None,
            })
            .collect()
    };
    if manifest_inputs.is_empty() {
        return Err("no microservice manifest paths provided or discovered".to_string());
    }

    let mut rows = Vec::new();
    let mut required_crates = BTreeSet::new();
    let mut missing_crates = BTreeSet::new();
    for manifest_input in manifest_inputs {
        let row = read_manifest_alignment_row(&manifest_input, &workspace_crates)?;
        required_crates.extend(row.required_crates.iter().cloned());
        missing_crates.extend(row.missing_crates.iter().cloned());
        rows.push(row);
    }

    let mut report = Fd001ManifestWorkspaceAlignmentReport {
        manifest_count: rows.len(),
        required_crate_count: required_crates.len(),
        workspace_crate_count: workspace_crates.len(),
        missing_crates: missing_crates.into_iter().collect(),
        manifests: rows,
        manifest_scope: if args.manifest_paths.is_empty() {
            args.manifest_scope
        } else {
            ManifestScope::ExplicitManifests
        },
        report_only: args.report_only,
        emitted_report_path: None,
    };

    if let Some(path) = &args.emit_report_path {
        emit_alignment_report(path, &report)?;
        report.emitted_report_path = Some(path.clone());
    }

    if !report.report_only && !report.missing_crates.is_empty() {
        return Err(report.failure_message());
    }

    Ok(report)
}

impl Fd001ManifestWorkspaceAlignmentReport {
    pub(crate) fn failure_message(&self) -> String {
        let sample = self
            .missing_crates
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        let suffix = if self.missing_crates.len() > 10 {
            format!(", ... {} more", self.missing_crates.len() - 10)
        } else {
            String::new()
        };
        format!(
            "{} manifests checked, {} required crates, workspace crates: {}, missing manifest crates: {} ({sample}{suffix})",
            self.manifest_count,
            self.required_crate_count,
            self.workspace_crate_count,
            self.missing_crates.len()
        )
    }
}

fn read_workspace_crate_names(workspace_manifest_path: &Path) -> Result<BTreeSet<String>, String> {
    let workspace_dir = workspace_manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let mut crate_names = BTreeSet::new();
    for member_path in read_workspace_member_paths(workspace_manifest_path)? {
        let package_manifest = workspace_dir.join(member_path).join("Cargo.toml");
        let package_name = read_package_name(&package_manifest)?;
        crate_names.insert(package_name);
    }
    if crate_names.is_empty() {
        Err("workspace package set is empty".to_string())
    } else {
        Ok(crate_names)
    }
}

fn read_manifest_index_inputs(
    repo_root: &Path,
    manifest_index_path: &Path,
    manifest_scope: ManifestScope,
) -> Result<Vec<ManifestInput>, String> {
    let body = fs::read_to_string(manifest_index_path).map_err(|error| {
        format!(
            "microservice manifest index unreadable: {}: {error}",
            manifest_index_path.display()
        )
    })?;
    let root: Value = serde_json::from_str(&body).map_err(|error| {
        format!(
            "microservice manifest index invalid JSON: {}: {error}",
            manifest_index_path.display()
        )
    })?;
    let microservices = root
        .get("microservices")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            format!(
                "microservice manifest index missing microservices[]: {}",
                manifest_index_path.display()
            )
        })?;
    let mut inputs = Vec::new();
    for (index, row) in microservices.iter().enumerate() {
        let fd001_material = match row.get("fd001_material") {
            Some(value) => value.as_bool().ok_or_else(|| {
                format!("microservices[{index}].fd001_material must be a boolean when present")
            })?,
            None => false,
        };
        if manifest_scope == ManifestScope::Fd001Material && !fd001_material {
            continue;
        }
        let name = row
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("microservices[{index}].name must be a string"))?;
        let manifest = row
            .get("manifest")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("microservices[{index}].manifest must be a string"))?;
        inputs.push(ManifestInput {
            path: normalize_path(repo_root, Path::new(manifest)),
            expected_microservice: Some(name.to_string()),
        });
    }
    if manifest_scope == ManifestScope::Fd001Material && inputs.is_empty() {
        return Err(format!(
            "microservice manifest index contained no fd001_material=true rows: {}",
            manifest_index_path.display()
        ));
    }
    Ok(inputs)
}

fn validate_emit_report_path(path: &Path) -> Result<(), String> {
    let components = path
        .components()
        .filter(|component| !matches!(component, Component::CurDir))
        .collect::<Vec<_>>();
    if components.is_empty() {
        return Err("--emit-report must be repo-relative under evidence/".to_string());
    }
    if path.is_absolute()
        || components
            .iter()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("--emit-report must be repo-relative under evidence/".to_string());
    }
    match components.first() {
        Some(Component::Normal(name)) if *name == OsStr::new("evidence") => {}
        _ => return Err("--emit-report must be repo-relative under evidence/".to_string()),
    }
    if path.file_name().is_none() {
        return Err("--emit-report must name a JSON evidence file".to_string());
    }
    Ok(())
}

fn read_manifest_alignment_row(
    manifest_input: &ManifestInput,
    workspace_crates: &BTreeSet<String>,
) -> Result<ManifestAlignmentRow, String> {
    let manifest_path = &manifest_input.path;
    let body = fs::read_to_string(manifest_path).map_err(|error| {
        format!(
            "microservice manifest unreadable: {}: {error}",
            manifest_path.display()
        )
    })?;
    let root: Value = serde_json::from_str(&body).map_err(|error| {
        format!(
            "microservice manifest invalid JSON: {}: {error}",
            manifest_path.display()
        )
    })?;
    let microservice = root
        .get("microservice")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            format!(
                "microservice manifest missing microservice string: {}",
                manifest_path.display()
            )
        })?
        .to_string();
    if let Some(expected) = &manifest_input.expected_microservice
        && expected != &microservice
    {
        return Err(format!(
            "microservice identity mismatch: {} expected {expected} from manifest index but manifest.microservice is {microservice}",
            manifest_path.display()
        ));
    }
    let bounded_contexts = root
        .get("bounded_contexts")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            format!(
                "microservice manifest missing bounded_contexts[]: {}",
                manifest_path.display()
            )
        })?;
    if bounded_contexts.is_empty() {
        return Err(format!(
            "{} bounded_contexts must not be empty",
            manifest_path.display()
        ));
    }

    let mut required = BTreeSet::new();
    for (context_index, context) in bounded_contexts.iter().enumerate() {
        let crates = context
            .get("crates")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                format!(
                    "{} bounded_contexts[{context_index}].crates must be an array",
                    manifest_path.display()
                )
            })?;
        if crates.is_empty() {
            return Err(format!(
                "{} bounded_contexts[{context_index}].crates must not be empty",
                manifest_path.display()
            ));
        }
        for (crate_index, crate_value) in crates.iter().enumerate() {
            let crate_name = crate_value.as_str().ok_or_else(|| {
                format!(
                    "{} bounded_contexts[{context_index}].crates[{crate_index}] must be a string",
                    manifest_path.display()
                )
            })?;
            required.insert(crate_name.to_string());
        }
    }
    if required.is_empty() {
        return Err(format!(
            "{} declares no required crates",
            manifest_path.display()
        ));
    }

    let present = required
        .iter()
        .filter(|crate_name| workspace_crates.contains(*crate_name))
        .cloned()
        .collect::<Vec<_>>();
    let missing = required
        .iter()
        .filter(|crate_name| !workspace_crates.contains(*crate_name))
        .cloned()
        .collect::<Vec<_>>();

    Ok(ManifestAlignmentRow {
        microservice,
        expected_microservice: manifest_input.expected_microservice.clone(),
        manifest_path: manifest_path.to_path_buf(),
        required_crates: required.into_iter().collect(),
        present_crates: present,
        missing_crates: missing,
    })
}

fn emit_alignment_report(
    path: &Path,
    report: &Fd001ManifestWorkspaceAlignmentReport,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "cannot create report directory {}: {error}",
                parent.display()
            )
        })?;
    }

    let manifests = report
        .manifests
        .iter()
        .map(|row| {
            json!({
                "microservice": row.microservice,
                "expected_microservice": row.expected_microservice,
                "manifest_path": row.manifest_path.display().to_string(),
                "required_crate_count": row.required_crates.len(),
                "present_crate_count": row.present_crates.len(),
                "missing_crate_count": row.missing_crates.len(),
                "required_crates": row.required_crates,
                "present_crates": row.present_crates,
                "missing_crates": row.missing_crates,
            })
        })
        .collect::<Vec<_>>();

    let body = json!({
        "gate": "fd001-manifest-workspace-alignment",
        "schema_version": 1,
        "mode": if report.report_only { "report-only" } else { "blocking" },
        "manifest_scope": report.manifest_scope.as_str(),
        "manifest_count": report.manifest_count,
        "required_crate_count": report.required_crate_count,
        "workspace_crate_count": report.workspace_crate_count,
        "missing_crate_count": report.missing_crates.len(),
        "missing_crates": report.missing_crates,
        "manifests": manifests,
    });

    let rendered = serde_json::to_string_pretty(&body)
        .map_err(|error| format!("cannot render alignment report: {error}"))?;
    fs::write(path, format!("{rendered}\n"))
        .map_err(|error| format!("cannot write alignment report {}: {error}", path.display()))
}

fn required_path_value(
    args: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn normalize_path(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() || base == Path::new(".") {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}
