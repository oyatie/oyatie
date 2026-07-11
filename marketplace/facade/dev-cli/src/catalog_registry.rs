use std::fs;
use std::path::Path;

use intelligence_catalog_domain::{CatalogRecord, CatalogRecordInput};

pub(crate) fn read_catalog_records(registry_dir: &Path) -> Result<Vec<CatalogRecord>, String> {
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
        let record = parse_catalog_record(crate_id, &contents)
            .and_then(|input| input.build().map_err(|error| format!("{error:?}")))
            .map_err(|error| format!("{}: {error}", path.display()))?;
        records.push(record);
    }
    if records.is_empty() {
        Err("registry directory contains no .yaml records".to_string())
    } else {
        Ok(records)
    }
}

fn parse_catalog_record(crate_id: &str, contents: &str) -> Result<CatalogRecordInput, String> {
    let mut context = None;
    let mut role = None;
    let mut capability = None;
    let mut plane = None;
    let mut data_classes_owned = None;
    let mut operational_classes_owned = None;
    let mut api_stability = None;
    let mut security_review = None;
    let mut supply_chain = None;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "context" => context = Some(value.to_string()),
            "role" => role = Some(value.to_string()),
            "capability" => capability = Some(value.to_string()),
            "plane" => plane = Some(value.to_string()),
            "data_classes_owned" => data_classes_owned = Some(parse_catalog_list(value)?),
            "operational_classes_owned" => {
                operational_classes_owned = Some(parse_catalog_list(value)?)
            }
            "api_stability" => api_stability = Some(value.to_string()),
            "security_review" => security_review = Some(value.to_string()),
            "supply_chain" => supply_chain = Some(value.to_string()),
            _ => {}
        }
    }
    Ok(CatalogRecordInput {
        crate_id: crate_id.to_string(),
        context: context.ok_or_else(|| "missing context".to_string())?,
        role: role.ok_or_else(|| "missing role".to_string())?,
        capability: capability.ok_or_else(|| "missing capability".to_string())?,
        plane: plane.ok_or_else(|| "missing plane".to_string())?,
        data_classes_owned: data_classes_owned
            .ok_or_else(|| "missing data_classes_owned".to_string())?,
        operational_classes_owned: operational_classes_owned.unwrap_or_default(),
        api_stability: api_stability.ok_or_else(|| "missing api_stability".to_string())?,
        security_review: security_review.ok_or_else(|| "missing security_review".to_string())?,
        supply_chain: supply_chain.ok_or_else(|| "missing supply_chain".to_string())?,
    })
}

fn parse_catalog_list(value: &str) -> Result<Vec<String>, String> {
    let trimmed = value.trim();
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| "catalog list must use [A, B] syntax".to_string())?;
    Ok(inner
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect())
}
