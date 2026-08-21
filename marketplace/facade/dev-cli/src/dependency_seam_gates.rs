use std::fs;
use std::path::{Path, PathBuf};

use check_dependency_seam::{
    DependencySeamConfig, DependencySeamReport, DependencySeamSeverity, validate_dependency_seam,
};

use crate::usage;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DependencySeamValidateArgs {
    config: DependencySeamConfig,
    emit_report_path: Option<PathBuf>,
}

pub(crate) fn parse_dependency_seam_validate_args(
    args: Vec<String>,
) -> Result<DependencySeamValidateArgs, String> {
    let mut repo_root = PathBuf::from(".");
    let mut registry_path = PathBuf::from("registry/dependency-rationales.json");
    let mut fixture_root = PathBuf::from("crates/oya-check-dependency-seam/tests/fixtures");
    let mut evidence_paths: Vec<PathBuf> = Vec::new();
    let mut offline = true;
    let mut severity = DependencySeamSeverity::ReportOnly;
    let mut emit_report_path = None;

    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--repo-root" => repo_root = next_path(&mut iter)?,
            "--registry" => registry_path = next_path(&mut iter)?,
            "--fixture-root" => fixture_root = next_path(&mut iter)?,
            "--evidence" => evidence_paths.push(next_path(&mut iter)?),
            "--emit-report" => emit_report_path = Some(next_path(&mut iter)?),
            "--offline" => offline = true,
            "--online-audit" => offline = false,
            "--severity" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                severity = DependencySeamSeverity::parse(&value).ok_or_else(|| {
                    "dependency-seam: --severity must be report-only or error".to_string()
                })?;
            }
            _ => return Err(usage()),
        }
    }

    let repo_root = repo_root;
    let config = DependencySeamConfig {
        repo_root: repo_root.clone(),
        registry_path: resolve_repo_path(&repo_root, registry_path),
        fixture_root: resolve_repo_path(&repo_root, fixture_root),
        evidence_paths: evidence_paths
            .into_iter()
            .map(|path| resolve_repo_path(&repo_root, path))
            .collect(),
        offline,
        severity,
    };
    Ok(DependencySeamValidateArgs {
        config,
        emit_report_path: emit_report_path.map(|path| resolve_repo_path(&repo_root, path)),
    })
}

pub(crate) fn validate_dependency_seam_gate(
    args: DependencySeamValidateArgs,
) -> Result<DependencySeamReport, String> {
    let report = validate_dependency_seam(&args.config)?;
    if let Some(path) = args.emit_report_path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "dependency-seam report parent unreadable {}: {error}",
                    parent.display()
                )
            })?;
        }
        let encoded = serde_json::to_string_pretty(&report)
            .map_err(|error| format!("dependency-seam report serialization failed: {error}"))?;
        fs::write(&path, encoded).map_err(|error| {
            format!(
                "dependency-seam report write failed {}: {error}",
                path.display()
            )
        })?;
    }
    Ok(report)
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
