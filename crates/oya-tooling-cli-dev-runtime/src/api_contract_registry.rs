use std::fs;
use std::path::{Path, PathBuf};

use oya_foundry_api_semver_kernel::{ApiContractMetadata, ApiContractRecord};

use crate::{clean_yaml_value, parse_yaml_inline_values, slash_path};

pub(crate) fn read_api_contract_records(
    contracts_dir: &Path,
) -> Result<Vec<ApiContractRecord>, String> {
    if !contracts_dir.exists() {
        return Ok(Vec::new());
    }
    let mut artifacts = Vec::new();
    collect_api_contract_artifacts(contracts_dir, &mut artifacts)?;
    artifacts
        .into_iter()
        .map(|artifact_path| {
            let relative = artifact_path
                .strip_prefix(contracts_dir)
                .map_err(|error| format!("API contract path not under contracts dir: {error}"))?;
            let normalized_artifact_path = format!("contracts/{}", slash_path(relative));
            let metadata = read_api_contract_metadata(contracts_dir, &artifact_path)?;
            Ok(ApiContractRecord {
                artifact_path: normalized_artifact_path,
                metadata,
            })
        })
        .collect()
}

fn collect_api_contract_artifacts(
    current: &Path,
    artifacts: &mut Vec<PathBuf>,
) -> Result<(), String> {
    for entry in fs::read_dir(current)
        .map_err(|error| format!("API contracts directory unreadable: {error}"))?
    {
        let entry =
            entry.map_err(|error| format!("API contracts directory entry unreadable: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_api_contract_artifacts(&path, artifacts)?;
            continue;
        }
        if path.is_file() && is_api_contract_artifact(&path) {
            artifacts.push(path);
        }
    }
    Ok(())
}

fn is_api_contract_artifact(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    if matches!(extension, "yaml" | "yml") && is_api_contract_metadata_path(path) {
        return false;
    }
    matches!(extension, "yaml" | "yml" | "proto" | "graphql")
}

pub(crate) fn is_api_contract_metadata_path(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    file_name == "meta.yaml"
        || file_name == "meta.yml"
        || file_name.ends_with(".meta.yaml")
        || file_name.ends_with(".meta.yml")
}

fn read_api_contract_metadata(
    contracts_dir: &Path,
    artifact_path: &Path,
) -> Result<Option<ApiContractMetadata>, String> {
    let Some(metadata_path) = api_contract_metadata_path(artifact_path) else {
        return Ok(None);
    };
    let relative = metadata_path
        .strip_prefix(contracts_dir)
        .map_err(|error| format!("API metadata path not under contracts dir: {error}"))?;
    let normalized_metadata_path = format!("contracts/{}", slash_path(relative));
    let contents = fs::read_to_string(&metadata_path)
        .map_err(|error| format!("API contract metadata unreadable: {error}"))?;
    Ok(Some(parse_api_contract_metadata(
        &normalized_metadata_path,
        &contents,
    )))
}

fn api_contract_metadata_path(artifact_path: &Path) -> Option<PathBuf> {
    let parent = artifact_path.parent()?;
    let stem = artifact_path.file_stem()?.to_str()?;
    [
        parent.join(format!("{stem}.meta.yaml")),
        parent.join(format!("{stem}.meta.yml")),
        parent.join("meta.yaml"),
        parent.join("meta.yml"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn parse_api_contract_metadata(path: &str, contents: &str) -> ApiContractMetadata {
    let mut tier = String::new();
    let mut owner_team = String::new();
    let mut version = String::new();
    let mut sunset = String::new();
    let mut related_adrs = Vec::new();
    let mut in_related_adrs = false;

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if in_related_adrs && trimmed.starts_with("- ") {
            related_adrs.push(clean_yaml_value(trimmed.trim_start_matches("- ")));
            continue;
        }
        in_related_adrs = false;
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let value = clean_yaml_value(value);
        match key.trim() {
            "tier" => tier = value,
            "owner_team" | "owner" => owner_team = value,
            "version" => version = value,
            "sunset" => sunset = value,
            "related_adrs" => {
                if value.is_empty() {
                    in_related_adrs = true;
                } else {
                    related_adrs.extend(parse_yaml_inline_values(&value));
                }
            }
            _ => {}
        }
    }

    ApiContractMetadata {
        metadata_path: path.into(),
        tier,
        owner_team,
        version,
        sunset,
        related_adrs,
    }
}
