use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use oya_application_app::{
    AutonomyTier, Capability, CapabilityCostProfile, CapabilityMcpContract, DataClass,
    privacy_data_classes_from,
};

use crate::{
    clean_scalar_value, parse_u64_field, parse_yaml_inline_values, required_scalar, usage,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FoundryCapabilitySchemaValidateArgs {
    capabilities_dir: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FoundryCapabilitySchemaRecord {
    capability: Capability,
}

pub(crate) fn parse_foundry_capability_schema_validate_args(
    args: Vec<String>,
) -> Result<FoundryCapabilitySchemaValidateArgs, String> {
    let mut parsed = FoundryCapabilitySchemaValidateArgs {
        capabilities_dir: PathBuf::from("product-control/capabilities"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(value) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--capabilities-dir" => parsed.capabilities_dir = PathBuf::from(value),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_foundry_capability_schema_gate(
    args: FoundryCapabilitySchemaValidateArgs,
) -> Result<(usize, usize, usize), String> {
    let records = read_foundry_capability_schema_records(&args.capabilities_dir)?;
    let mut seen = BTreeSet::new();
    for record in &records {
        if !seen.insert(record.capability.id.clone()) {
            return Err(format!(
                "duplicate capability id {} in capability records",
                record.capability.id
            ));
        }
    }
    Ok((records.len(), records.len(), records.len() * 2))
}

fn read_foundry_capability_schema_records(
    capabilities_dir: &Path,
) -> Result<Vec<FoundryCapabilitySchemaRecord>, String> {
    let mut records = Vec::new();
    for entry in fs::read_dir(capabilities_dir)
        .map_err(|error| format!("capabilities directory unreadable: {error}"))?
    {
        let entry = entry.map_err(|error| format!("capability entry unreadable: {error}"))?;
        let path = entry.path();
        if path.is_dir()
            || path.extension().and_then(|extension| extension.to_str()) != Some("yaml")
        {
            continue;
        }
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("capability record unreadable {}: {error}", path.display()))?;
        records.push(parse_foundry_capability_schema_record(&path, &contents)?);
    }
    records.sort_by(|left, right| left.capability.id.cmp(&right.capability.id));
    if records.is_empty() {
        Err(format!(
            "capabilities directory contains no root capability .yaml records: {}",
            capabilities_dir.display()
        ))
    } else {
        Ok(records)
    }
}

fn parse_foundry_capability_schema_record(
    path: &Path,
    contents: &str,
) -> Result<FoundryCapabilitySchemaRecord, String> {
    let capability_id = required_scalar(path, contents, "id")?;
    let namespace = required_scalar(path, contents, "namespace")?;
    let required_tier = parse_autonomy_tier(
        path,
        &required_scalar(path, contents, "autonomy_tier_required")?,
    )?;
    let data_classes = required_string_list(path, contents, "data_classes_touched")?
        .iter()
        .map(|label| parse_capability_data_class(path, label))
        .collect::<Result<Vec<_>, _>>()?;
    let touched_privacy_data_classes = privacy_data_classes_from(&data_classes).map_err(|_| {
        format!(
            "{}: data_classes_touched contains non-privacy marker",
            path.display()
        )
    })?;
    let evidence_topic = required_scalar(path, contents, "evidence_emission_topic")?;
    let cost_profile = CapabilityCostProfile::new(
        parse_cost_profile_micros(
            path,
            contents,
            "per_invocation_limit_micros",
            "per_invocation_budget_usd",
        )?,
        parse_cost_profile_micros(
            path,
            contents,
            "per_tenant_monthly_limit_micros",
            "monthly_budget_usd",
        )?,
        provider_preference(path, contents)?,
    )
    .map_err(|error| {
        format!(
            "{}: capability cost profile invalid: {error:?}",
            path.display()
        )
    })?;
    let mcp_contract = CapabilityMcpContract::new(
        required_nested_scalar(path, contents, "description", "agent_readable")?,
        required_nested_scalar(path, contents, "description", "human_readable")?,
        schema_section_to_json(path, contents, "input_schema")?,
        schema_section_to_json(path, contents, "output_schema")?,
    )
    .map_err(|error| {
        format!(
            "{}: capability MCP contract invalid: {error:?}",
            path.display()
        )
    })?;
    let capability = Capability::new_with_cost_profile_and_mcp_contract(
        capability_id,
        namespace,
        required_tier,
        touched_privacy_data_classes,
        evidence_topic,
        cost_profile,
        mcp_contract,
    )
    .map_err(|error| format!("{}: capability record invalid: {error:?}", path.display()))?;
    Ok(FoundryCapabilitySchemaRecord { capability })
}

fn parse_autonomy_tier(path: &Path, value: &str) -> Result<AutonomyTier, String> {
    match value {
        "T1" | "T1ViewOnly" => Ok(AutonomyTier::T1ViewOnly),
        "T2" | "T2Advisory" => Ok(AutonomyTier::T2Advisory),
        "T3" | "T3ExecuteWithApproval" => Ok(AutonomyTier::T3ExecuteWithApproval),
        "T4" | "T4AutoExecute" => Ok(AutonomyTier::T4AutoExecute),
        _ => Err(format!(
            "{}: unknown autonomy_tier_required {value}",
            path.display()
        )),
    }
}

fn parse_capability_data_class(path: &Path, label: &str) -> Result<DataClass, String> {
    match label.trim() {
        "PUBLIC" => Ok(DataClass::Public),
        "INTERNAL_ONLY" => Ok(DataClass::InternalOnly),
        "PII_IDENTIFYING" => Ok(DataClass::PiiIdentifying),
        "PII_SENSITIVE" => Ok(DataClass::PiiSensitive),
        "PII_QUASI_IDENTIFIER" => Ok(DataClass::PiiQuasiIdentifier),
        "PHI" => Ok(DataClass::Phi),
        "PCI" => Ok(DataClass::Pci),
        "PIPA_ARTICLE_23" | "PIPA_ARTICLE23" => Ok(DataClass::PipaArticle23),
        "SENSITIVE_PIPA_ART23" => Ok(DataClass::SensitivePipaArticle23),
        "FINANCIAL" => Ok(DataClass::Financial),
        "FINANCIAL_KR_신용정보" | "FINANCIAL_KR_CREDIT" => Ok(DataClass::FinancialKrCredit),
        "USAGE" => Ok(DataClass::Usage),
        "BEHAVIORAL_TENANT_PRODUCT" => Ok(DataClass::BehavioralTenantProduct),
        "BEHAVIORAL_ADS" => Ok(DataClass::BehavioralAds),
        "DECLARED_PREFERENCE" => Ok(DataClass::DeclaredPreference),
        "SEARCH_QUERY" => Ok(DataClass::SearchQuery),
        "AUDIT" => Ok(DataClass::Audit),
        "SECRET" => Ok(DataClass::Secret),
        "CHILDREN" => Ok(DataClass::Children),
        _ => Err(format!("{}: unknown data class {label}", path.display())),
    }
}

fn provider_preference(path: &Path, contents: &str) -> Result<Vec<String>, String> {
    let mut preference = vec![required_nested_scalar(
        path,
        contents,
        "provider",
        "preferred",
    )?];
    preference.extend(optional_nested_string_list(
        contents, "provider", "fallback",
    )?);
    Ok(preference)
}

fn parse_cost_profile_micros(
    path: &Path,
    contents: &str,
    micros_key: &str,
    usd_key: &str,
) -> Result<u64, String> {
    if let Some(value) = optional_nested_scalar(contents, "cost_profile", micros_key)? {
        return parse_u64_field(&value, micros_key)
            .map_err(|message| format!("{}: {message}", path.display()));
    }
    let usd = required_nested_scalar(path, contents, "cost_profile", usd_key)?;
    parse_usd_micros(path, usd_key, &usd)
}

fn parse_usd_micros(path: &Path, field: &str, value: &str) -> Result<u64, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.starts_with('-') {
        return Err(format!(
            "{}: {field} must be a non-negative decimal",
            path.display()
        ));
    }
    let mut parts = trimmed.split('.');
    let whole = parts
        .next()
        .ok_or_else(|| format!("{}: {field} must be a decimal", path.display()))?
        .parse::<u64>()
        .map_err(|_| format!("{}: {field} must be a decimal", path.display()))?;
    let fractional = parts.next().unwrap_or("");
    if parts.next().is_some()
        || fractional.len() > 6
        || !fractional.chars().all(|ch| ch.is_ascii_digit())
    {
        return Err(format!(
            "{}: {field} must have at most six decimal places",
            path.display()
        ));
    }
    let mut padded = fractional.to_string();
    while padded.len() < 6 {
        padded.push('0');
    }
    let fraction = padded
        .parse::<u64>()
        .map_err(|_| format!("{}: {field} must be a decimal", path.display()))?;
    whole
        .checked_mul(1_000_000)
        .and_then(|whole_micros| whole_micros.checked_add(fraction))
        .ok_or_else(|| format!("{}: {field} exceeds u64 micros", path.display()))
}

fn schema_section_to_json(path: &Path, contents: &str, section: &str) -> Result<String, String> {
    if let Some(inline) = optional_top_level_scalar(contents, section)?
        && !inline.trim().is_empty()
    {
        return Ok(inline);
    }
    let lines = required_section_lines(path, contents, section)?;
    if !lines
        .iter()
        .any(|line| yaml_key_value_matches(line, "type", "object"))
    {
        return Err(format!(
            "{}: {section} must declare type: object",
            path.display()
        ));
    }
    let required = required_schema_fields(&lines);
    let properties = schema_property_names(&lines);
    let mut json = "{\"type\":\"object\"".to_string();
    if !properties.is_empty() {
        json.push_str(",\"properties\":{");
        for (index, property) in properties.iter().enumerate() {
            if index > 0 {
                json.push(',');
            }
            json.push_str(&format!("\"{}\":{{}}", json_escape(property)));
        }
        json.push('}');
    }
    if !required.is_empty() {
        json.push_str(",\"required\":[");
        for (index, field) in required.iter().enumerate() {
            if index > 0 {
                json.push(',');
            }
            json.push_str(&format!("\"{}\"", json_escape(field)));
        }
        json.push(']');
    }
    json.push('}');
    Ok(json)
}

fn required_schema_fields(lines: &[String]) -> Vec<String> {
    let mut required = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("required:") {
            required.extend(parse_yaml_inline_values(value));
            let required_indent = indentation(line);
            for nested in lines.iter().skip(index + 1) {
                if indentation(nested) <= required_indent {
                    break;
                }
                let nested_trimmed = nested.trim();
                if let Some(item) = nested_trimmed.strip_prefix("- ") {
                    required.push(clean_scalar_value(item));
                }
            }
        }
    }
    required.sort();
    required.dedup();
    required
}

fn schema_property_names(lines: &[String]) -> Vec<String> {
    let mut properties = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if line.trim() != "properties:" {
            continue;
        }
        let properties_indent = indentation(line);
        for nested in lines.iter().skip(index + 1) {
            let nested_indent = indentation(nested);
            if nested_indent <= properties_indent {
                break;
            }
            let nested_trimmed = nested.trim();
            if nested_indent == properties_indent + 2
                && nested_trimmed.ends_with(':')
                && !nested_trimmed.starts_with('-')
            {
                properties.push(nested_trimmed.trim_end_matches(':').to_string());
            }
        }
    }
    properties.sort();
    properties.dedup();
    properties
}

fn required_string_list(path: &Path, contents: &str, key: &str) -> Result<Vec<String>, String> {
    if let Some(inline) = optional_top_level_scalar(contents, key)?
        && !inline.trim().is_empty()
    {
        let values = parse_yaml_inline_values(&inline);
        if !values.is_empty() {
            return Ok(values);
        }
    }
    let values = section_list_values(path, contents, key)?;
    if values.is_empty() {
        Err(format!("{}: missing required field {key}", path.display()))
    } else {
        Ok(values)
    }
}

fn optional_nested_string_list(
    contents: &str,
    section: &str,
    key: &str,
) -> Result<Vec<String>, String> {
    let Some(lines) = optional_section_lines(contents, section)? else {
        return Ok(Vec::new());
    };
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let Some((actual_key, value)) = trimmed.split_once(':') else {
            continue;
        };
        if actual_key.trim() != key {
            continue;
        }
        let inline = parse_yaml_inline_values(value);
        if !inline.is_empty() {
            return Ok(inline);
        }
        let key_indent = indentation(line);
        let mut values = Vec::new();
        for nested in lines.iter().skip(index + 1) {
            if indentation(nested) <= key_indent {
                break;
            }
            if let Some(item) = nested.trim().strip_prefix("- ") {
                values.push(clean_scalar_value(item));
            }
        }
        return Ok(values);
    }
    Ok(Vec::new())
}

fn section_list_values(path: &Path, contents: &str, section: &str) -> Result<Vec<String>, String> {
    let lines = required_section_lines(path, contents, section)?;
    Ok(lines
        .iter()
        .filter_map(|line| line.trim().strip_prefix("- ").map(clean_scalar_value))
        .filter(|item| !item.is_empty())
        .collect())
}

fn required_nested_scalar(
    path: &Path,
    contents: &str,
    section: &str,
    key: &str,
) -> Result<String, String> {
    optional_nested_scalar(contents, section, key)?
        .ok_or_else(|| format!("{}: missing required field {section}.{key}", path.display()))
}

fn optional_nested_scalar(
    contents: &str,
    section: &str,
    key: &str,
) -> Result<Option<String>, String> {
    let Some(lines) = optional_section_lines(contents, section)? else {
        return Ok(None);
    };
    let mut found = None;
    for line in lines {
        if indentation(&line) != 2 {
            continue;
        }
        let trimmed = line.trim();
        let Some((actual_key, value)) = trimmed.split_once(':') else {
            continue;
        };
        if actual_key.trim() == key {
            if found.is_some() {
                return Err(format!("duplicate field {section}.{key}"));
            }
            let value = clean_scalar_value(value);
            if !value.is_empty() {
                found = Some(value);
            }
        }
    }
    Ok(found)
}

fn required_section_lines(
    path: &Path,
    contents: &str,
    section: &str,
) -> Result<Vec<String>, String> {
    optional_section_lines(contents, section)?
        .filter(|lines| !lines.is_empty())
        .ok_or_else(|| format!("{}: missing required section {section}", path.display()))
}

fn optional_section_lines(contents: &str, section: &str) -> Result<Option<Vec<String>>, String> {
    let mut lines = Vec::new();
    let mut in_section = false;
    let mut found = false;
    for line in contents.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if !line.starts_with(char::is_whitespace) {
            if in_section {
                break;
            }
            if let Some((actual_key, value)) = line.split_once(':')
                && actual_key.trim() == section
            {
                if found {
                    return Err(format!("duplicate section {section}"));
                }
                found = true;
                in_section = true;
                if !clean_scalar_value(value).is_empty() {
                    lines.push(format!("  __inline__: {}", clean_scalar_value(value)));
                }
            }
            continue;
        }
        if in_section {
            lines.push(line.to_string());
        }
    }
    Ok(found.then_some(lines))
}

fn optional_top_level_scalar(contents: &str, key: &str) -> Result<Option<String>, String> {
    let mut found = None;
    for line in contents.lines() {
        if line.starts_with(char::is_whitespace) {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        let Some((actual_key, value)) = trimmed.split_once(':') else {
            continue;
        };
        if actual_key.trim() == key {
            if found.is_some() {
                return Err(format!("duplicate field {key}"));
            }
            found = Some(clean_scalar_value(value));
        }
    }
    Ok(found)
}

fn yaml_key_value_matches(line: &str, key: &str, expected_value: &str) -> bool {
    let trimmed = line.trim();
    let Some((actual_key, value)) = trimmed.split_once(':') else {
        return false;
    };
    actual_key.trim() == key && clean_scalar_value(value) == expected_value
}

fn indentation(line: &str) -> usize {
    line.chars()
        .take_while(|character| *character == ' ')
        .count()
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
