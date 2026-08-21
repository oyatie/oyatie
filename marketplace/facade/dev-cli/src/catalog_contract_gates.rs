use std::fs;
use std::path::{Path, PathBuf};

use check_cohesion::validate_cohesion_fitness;
use check_slo_coverage::{SloCatalogRecord, validate_slo_coverage};

use crate::{read_cross_axis_contracts, read_workspace_member_crate_ids, usage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SloCoverageValidateArgs {
    registry_dir: PathBuf,
}

pub(crate) fn parse_slo_coverage_validate_args(
    args: Vec<String>,
) -> Result<SloCoverageValidateArgs, String> {
    let mut parsed = SloCoverageValidateArgs {
        registry_dir: PathBuf::from("registry/catalog"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--registry" => parsed.registry_dir = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_slo_coverage_gate(args: SloCoverageValidateArgs) -> Result<usize, String> {
    let records = read_slo_catalog_records(&args.registry_dir)?;
    let report = validate_slo_coverage(&records)
        .map_err(|error| format!("catalog SLO row invalid: {error:?}"))?;
    Ok(report.records_checked)
}

fn read_slo_catalog_records(registry_dir: &Path) -> Result<Vec<SloCatalogRecord>, String> {
    let entries = fs::read_dir(registry_dir)
        .map_err(|error| format!("registry directory unreadable: {error}"))?;
    let mut records = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("registry entry unreadable: {error}"))?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("yaml") {
            continue;
        }
        let crate_id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| format!("catalog path has invalid file name: {}", path.display()))?;
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("catalog record unreadable {}: {error}", path.display()))?;
        records.push(SloCatalogRecord {
            crate_id: crate_id.to_string(),
            slo: parse_catalog_slo(&contents),
        });
    }
    if records.is_empty() {
        Err("registry directory contains no .yaml records".to_string())
    } else {
        Ok(records)
    }
}

fn parse_catalog_slo(contents: &str) -> Option<String> {
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        if key.trim() == "slo" {
            return Some(value.trim().to_string());
        }
    }
    None
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CohesionValidateArgs {
    workspace_manifest_path: PathBuf,
    registry_dir: PathBuf,
    contracts_path: PathBuf,
}

pub(crate) fn parse_cohesion_validate_args(
    args: Vec<String>,
) -> Result<CohesionValidateArgs, String> {
    let mut parsed = CohesionValidateArgs {
        workspace_manifest_path: PathBuf::from("Cargo.toml"),
        registry_dir: PathBuf::from("registry/catalog"),
        contracts_path: PathBuf::from("docs/machine-readable/contracts.json"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--workspace" => parsed.workspace_manifest_path = PathBuf::from(path),
            "--registry" => parsed.registry_dir = PathBuf::from(path),
            "--contracts" => parsed.contracts_path = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_cohesion_gate(args: CohesionValidateArgs) -> Result<(usize, usize), String> {
    let contracts = read_cross_axis_contracts(&args.contracts_path)?;
    let catalog_crate_ids = read_catalog_crate_ids(&args.registry_dir)?;
    let workspace_crate_ids = read_workspace_member_crate_ids(&args.workspace_manifest_path)?;
    let report = validate_cohesion_fitness(&contracts, catalog_crate_ids, workspace_crate_ids)
        .map_err(|error| format!("cross-axis contract registry invalid: {error:?}"))?;
    Ok((report.contracts_checked, report.implemented_sources_checked))
}

fn read_catalog_crate_ids(registry_dir: &Path) -> Result<Vec<String>, String> {
    let entries = fs::read_dir(registry_dir)
        .map_err(|error| format!("registry directory unreadable: {error}"))?;
    let mut crate_ids = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("registry entry unreadable: {error}"))?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("yaml") {
            continue;
        }
        let crate_id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| format!("catalog path has invalid file name: {}", path.display()))?;
        crate_ids.push(crate_id.to_string());
    }
    Ok(crate_ids)
}
