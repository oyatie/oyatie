use serde_json::Value;
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
    internal_registry_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FoundryCapabilitySchemaRecord {
    capability: Capability,
}

pub(crate) fn parse_foundry_capability_schema_validate_args(
    args: Vec<String>,
) -> Result<FoundryCapabilitySchemaValidateArgs, String> {
    let mut parsed = FoundryCapabilitySchemaValidateArgs {
        capabilities_dir: PathBuf::from("registry/capability-templates"),
        internal_registry_path: PathBuf::from("registry/capabilities/foundry-internal.json"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(value) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--capabilities-dir" => parsed.capabilities_dir = PathBuf::from(value),
            "--internal-registry" => parsed.internal_registry_path = PathBuf::from(value),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_foundry_capability_schema_gate(
    args: FoundryCapabilitySchemaValidateArgs,
) -> Result<(usize, usize, usize, usize), String> {
    let repo_root = infer_repo_root(&args.internal_registry_path, &args.capabilities_dir);
    let records = read_foundry_capability_schema_records(&args.capabilities_dir)?;
    let internal_records =
        read_foundry_internal_registry_records(&args.internal_registry_path, &repo_root)?;
    let mut seen = BTreeSet::new();
    for record in &records {
        if !seen.insert(record.capability.id.clone()) {
            return Err(format!(
                "duplicate capability id {} in capability records",
                record.capability.id
            ));
        }
    }
    for capability_id in &internal_records {
        if !seen.insert(capability_id.clone()) {
            return Err(format!(
                "duplicate capability id {capability_id} across capability templates and internal registry"
            ));
        }
    }
    Ok((
        records.len() + internal_records.len(),
        records.len(),
        records.len() * 2,
        internal_records.len(),
    ))
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

fn read_foundry_internal_registry_records(
    path: &Path,
    repo_root: &Path,
) -> Result<Vec<String>, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "internal capability registry unreadable {}: {error}",
            path.display()
        )
    })?;
    let value: Value = serde_json::from_str(&contents).map_err(|error| {
        format!(
            "internal capability registry invalid JSON {}: {error}",
            path.display()
        )
    })?;
    validate_foundry_internal_registry_value(path, &value, repo_root)
}

fn validate_foundry_internal_registry_value(
    path: &Path,
    value: &Value,
    repo_root: &Path,
) -> Result<Vec<String>, String> {
    let Some(records) = value.as_array() else {
        return Err(format!(
            "{}: internal capability registry must be a JSON array",
            path.display()
        ));
    };
    if records.is_empty() {
        return Err(format!(
            "{}: internal capability registry must contain at least one capability record",
            path.display()
        ));
    }

    let mut ids = BTreeSet::new();
    let mut capability_ids = Vec::new();
    for (index, record) in records.iter().enumerate() {
        let object = record.as_object().ok_or_else(|| {
            format!(
                "{}[{index}]: internal capability registry row must be an object",
                path.display()
            )
        })?;
        let id = required_json_string(path, index, object, "id")?;
        if !id.starts_with("foundry.") {
            return Err(format!(
                "{}[{index}]: capability id {id:?} must stay in the foundry.* namespace",
                path.display()
            ));
        }
        if !ids.insert(id.clone()) {
            return Err(format!(
                "{}[{index}]: duplicate internal capability id {id}",
                path.display()
            ));
        }

        let namespace = required_json_string(path, index, object, "namespace")?;
        if !namespace.starts_with("foundry.") || !id.starts_with(&format!("{namespace}.")) {
            return Err(format!(
                "{}[{index}]: namespace {namespace:?} must be a foundry.* prefix of id {id:?}",
                path.display()
            ));
        }

        required_json_string(path, index, object, "name")?;
        required_json_string(path, index, object, "owner_team")?;
        let status = required_json_string(path, index, object, "status")?;
        if !matches!(
            status.as_str(),
            "planned" | "published" | "operational" | "deprecated"
        ) {
            return Err(format!(
                "{}[{index}]: status {status:?} must be planned, published, operational, or deprecated",
                path.display()
            ));
        }

        let required_tier = required_json_string(path, index, object, "autonomy_tier_required")?;
        if !matches!(required_tier.as_str(), "T1" | "T2" | "T3" | "T4") {
            return Err(format!(
                "{}[{index}]: autonomy_tier_required {required_tier:?} must be canonical T1/T2/T3/T4",
                path.display()
            ));
        }

        let data_classes = required_json_string_array(path, index, object, "data_classes_touched")?;
        for data_class in data_classes {
            parse_capability_data_class(path, &data_class)?;
        }

        let evidence_topic = required_json_string(path, index, object, "evidence_emission_topic")?;
        match object
            .get("evidence_emit_required")
            .and_then(Value::as_bool)
        {
            Some(true) => {}
            Some(false) => {
                return Err(format!(
                    "{}[{index}]: evidence_emit_required must remain true for internal Foundry capabilities",
                    path.display()
                ));
            }
            None => {
                return Err(format!(
                    "{}[{index}]: missing boolean evidence_emit_required",
                    path.display()
                ));
            }
        }
        if !evidence_topic.contains(&id) {
            return Err(format!(
                "{}[{index}]: evidence_emission_topic {evidence_topic:?} must include capability id {id:?}",
                path.display()
            ));
        }

        let prd_ref = required_json_string(path, index, object, "prd_ref")?;
        validate_current_capability_reference(repo_root, path, index, "prd_ref", &prd_ref)?;
        let task_ref = required_json_string(path, index, object, "task_ref")?;
        validate_current_capability_reference(repo_root, path, index, "task_ref", &task_ref)?;
        let test_ref = required_json_string(path, index, object, "test_ref")?;
        validate_current_capability_reference(repo_root, path, index, "test_ref", &test_ref)?;
        let verification_ref = required_json_string(path, index, object, "verification_ref")?;
        validate_current_capability_reference(
            repo_root,
            path,
            index,
            "verification_ref",
            &verification_ref,
        )?;

        let cost_profile = required_json_object(path, index, object, "cost_profile")?;
        require_json_scalar(
            cost_profile,
            path,
            index,
            "cost_profile",
            "per_invocation_budget_usd",
        )?;
        require_json_scalar(
            cost_profile,
            path,
            index,
            "cost_profile",
            "monthly_budget_usd",
        )?;

        let mcp_contract = required_json_object(path, index, object, "mcp_contract")?;
        for field in ["agent_readable", "human_readable"] {
            required_json_string(path, index, mcp_contract, field)?;
        }
        for field in ["input_schema_ref", "output_schema_ref"] {
            let reference = required_json_string(path, index, mcp_contract, field)?;
            validate_current_capability_reference(repo_root, path, index, field, &reference)?;
        }

        required_json_string_array(path, index, object, "failure_modes")?;
        let maturity = required_json_object(path, index, object, "maturity")?;
        required_json_string(path, index, maturity, "claim_boundary")?;
        let admission_ref = required_json_string(path, index, maturity, "admission_ref")?;
        validate_current_capability_reference(
            repo_root,
            path,
            index,
            "maturity.admission_ref",
            &admission_ref,
        )?;

        capability_ids.push(id);
    }

    Ok(capability_ids)
}

fn required_json_object<'a>(
    path: &Path,
    index: usize,
    object: &'a serde_json::Map<String, Value>,
    field: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    object
        .get(field)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{}[{index}]: missing object {field}", path.display()))
}

fn required_json_string(
    path: &Path,
    index: usize,
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<String, String> {
    object
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            format!(
                "{}[{index}]: missing non-empty string {field}",
                path.display()
            )
        })
}

fn required_json_string_array(
    path: &Path,
    index: usize,
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<Vec<String>, String> {
    let Some(values) = object.get(field).and_then(Value::as_array) else {
        return Err(format!(
            "{}[{index}]: missing non-empty string array {field}",
            path.display()
        ));
    };
    if values.is_empty() {
        return Err(format!(
            "{}[{index}]: {field} must contain at least one string",
            path.display()
        ));
    }
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .map(ToOwned::to_owned)
                .ok_or_else(|| {
                    format!(
                        "{}[{index}]: {field} entries must be non-empty strings",
                        path.display()
                    )
                })
        })
        .collect()
}

fn require_json_scalar(
    object: &serde_json::Map<String, Value>,
    path: &Path,
    index: usize,
    section: &str,
    field: &str,
) -> Result<(), String> {
    match object.get(field) {
        Some(Value::String(value)) if !value.trim().is_empty() => Ok(()),
        Some(Value::Number(_)) => Ok(()),
        _ => Err(format!(
            "{}[{index}]: {section}.{field} must be a non-empty string or number",
            path.display()
        )),
    }
}

fn infer_repo_root(internal_registry_path: &Path, capabilities_dir: &Path) -> PathBuf {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for path in [internal_registry_path, capabilities_dir] {
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            current_dir.join(path)
        };
        if let Some(repo_root) = repo_root_ancestor(&candidate) {
            return repo_root;
        }
    }
    current_dir
}

fn repo_root_ancestor(path: &Path) -> Option<PathBuf> {
    let start = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    start
        .ancestors()
        .find(|ancestor| ancestor.join("specs/root-hub-pointers.json").is_file())
        .map(Path::to_path_buf)
}

fn validate_current_capability_reference(
    repo_root: &Path,
    path: &Path,
    index: usize,
    field: &str,
    reference: &str,
) -> Result<(), String> {
    let lower = reference.to_ascii_lowercase();
    if lower.contains(".omc/")
        || lower.contains(".omx/")
        || lower.contains(".omc\\")
        || lower.contains(".omx\\")
        || lower.contains("implementation-plan")
        || lower.contains("implementation plan")
    {
        return Err(format!(
            "{}[{index}]: {field} must cite product PRD/task/spec/cloud-ci evidence, not retired .omc/.omx implementation-plan inputs: {reference:?}",
            path.display()
        ));
    }
    if lower.starts_with("oya-ci-required/cloud-ci evidence:")
        || lower == "oya-ci-required/cloud-ci gate packet"
    {
        return Err(format!(
            "{}[{index}]: {field} must cite a resolvable artifact path or evidence-required:* id, not a branded pseudo-evidence string: {reference:?}",
            path.display()
        ));
    }
    if is_valid_evidence_id(reference) {
        return Ok(());
    }

    let artifact = parse_artifact_reference(reference).ok_or_else(|| {
        format!(
            "{}[{index}]: {field} must be a resolvable repo artifact reference or evidence-required:* id, got {reference:?}",
            path.display()
        )
    })?;
    validate_artifact_reference(repo_root, path, index, field, reference, &artifact)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ArtifactReference<'a> {
    path: &'a str,
    fragment: Option<&'a str>,
    symbol: Option<&'a str>,
}

fn parse_artifact_reference(reference: &str) -> Option<ArtifactReference<'_>> {
    let reference = reference.trim();
    if reference.is_empty() || reference.contains(char::is_whitespace) {
        return None;
    }
    let reference = reference
        .strip_prefix("$ref:")
        .unwrap_or(reference)
        .strip_prefix('/')
        .unwrap_or(reference);
    let (path_and_symbol, fragment) = reference
        .split_once('#')
        .map_or((reference, None), |(path, fragment)| (path, Some(fragment)));
    let (artifact_path, symbol) = split_rust_symbol_reference(path_and_symbol);
    if artifact_path.is_empty()
        || artifact_path.starts_with('.')
        || artifact_path.contains('\\')
        || artifact_path.split('/').any(|part| part == "..")
    {
        return None;
    }
    Some(ArtifactReference {
        path: artifact_path,
        fragment,
        symbol,
    })
}

fn split_rust_symbol_reference(reference: &str) -> (&str, Option<&str>) {
    if let Some(index) = reference.find(".rs::") {
        (&reference[..index + 3], Some(&reference[index + 5..]))
    } else {
        (reference, None)
    }
}

fn validate_artifact_reference(
    repo_root: &Path,
    registry_path: &Path,
    index: usize,
    field: &str,
    original_reference: &str,
    artifact: &ArtifactReference<'_>,
) -> Result<(), String> {
    let artifact_path = repo_root.join(artifact.path);
    if !artifact_path.is_file() {
        return Err(format!(
            "{}[{index}]: {field} cites stale or invented artifact {original_reference:?}; {} does not exist",
            registry_path.display(),
            artifact_path.display()
        ));
    }

    if let Some(fragment) = artifact.fragment
        && artifact_path
            .extension()
            .and_then(|extension| extension.to_str())
            == Some("json")
    {
        validate_json_fragment(&artifact_path, fragment).map_err(|error| {
            format!(
                "{}[{index}]: {field} cites stale or invented JSON fragment {original_reference:?}: {error}",
                registry_path.display()
            )
        })?;
    }

    if let Some(symbol) = artifact.symbol {
        validate_rust_symbol_fragment(&artifact_path, symbol).map_err(|error| {
            format!(
                "{}[{index}]: {field} cites stale or invented Rust symbol {original_reference:?}: {error}",
                registry_path.display()
            )
        })?;
    }

    Ok(())
}

fn validate_json_fragment(path: &Path, fragment: &str) -> Result<(), String> {
    if fragment.trim().is_empty() {
        return Ok(());
    }
    let content = fs::read_to_string(path)
        .map_err(|error| format!("unable to read {}: {error}", path.display()))?;
    let value: Value = serde_json::from_str(&content)
        .map_err(|error| format!("invalid JSON {}: {error}", path.display()))?;
    let exists = if fragment.starts_with('/') {
        value.pointer(fragment).is_some()
    } else {
        fragment
            .split('/')
            .next()
            .and_then(|key| value.as_object().map(|object| object.contains_key(key)))
            .unwrap_or(false)
    };
    if exists {
        Ok(())
    } else {
        Err(format!("{} missing fragment #{fragment}", path.display()))
    }
}

fn validate_rust_symbol_fragment(path: &Path, symbol: &str) -> Result<(), String> {
    let terminal_symbol = symbol.rsplit("::").next().unwrap_or(symbol);
    let content = fs::read_to_string(path)
        .map_err(|error| format!("unable to read {}: {error}", path.display()))?;
    if !terminal_symbol.is_empty() && content.contains(terminal_symbol) {
        Ok(())
    } else {
        Err(format!("{} missing symbol {symbol}", path.display()))
    }
}

fn is_valid_evidence_id(reference: &str) -> bool {
    let Some(suffix) = reference.trim().strip_prefix("evidence-required:") else {
        return false;
    };
    (3..=80).contains(&suffix.len())
        && suffix.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
        && suffix
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
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
        "FINANCIAL_REGULATED_CREDIT" => Ok(DataClass::FinancialRegulatedCredit),
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
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn internal_registry_rejects_shallow_rows() {
        let registry = json!([
            {
                "id": "foundry.account.list",
                "name": "List provider accounts",
                "autonomy_tier": "T1Read",
                "evidence_emit_required": true
            }
        ]);

        let repo_root = temp_capability_repo("shallow");
        let error = validate_foundry_internal_registry_value(
            &repo_root.join("registry/capabilities/foundry-internal.json"),
            &registry,
            &repo_root,
        )
        .expect_err("shallow row is rejected");

        assert!(
            error.contains("missing non-empty string namespace"),
            "error={error}"
        );
    }

    #[test]
    fn internal_registry_accepts_rich_rows() {
        let registry = json!([rich_internal_record(
            "foundry.account.list",
            "foundry.account"
        )]);

        let repo_root = temp_capability_repo("rich");
        let ids = validate_foundry_internal_registry_value(
            &repo_root.join("registry/capabilities/foundry-internal.json"),
            &registry,
            &repo_root,
        )
        .expect("rich row accepted");

        assert_eq!(ids, vec!["foundry.account.list"]);
    }

    #[test]
    fn internal_registry_rejects_retired_plan_references() {
        let mut record = rich_internal_record("foundry.account.list", "foundry.account");
        record["test_ref"] = json!(".omx/plans/retired-implementation-plan.md");
        let registry = json!([record]);

        let repo_root = temp_capability_repo("retired-plan");
        let error = validate_foundry_internal_registry_value(
            &repo_root.join("registry/capabilities/foundry-internal.json"),
            &registry,
            &repo_root,
        )
        .expect_err("retired plan reference is rejected");

        assert!(
            error.contains("retired .omc/.omx implementation-plan inputs"),
            "error={error}"
        );
    }

    #[test]
    fn internal_registry_rejects_branded_pseudo_evidence_refs() {
        let mut record = rich_internal_record("foundry.account.list", "foundry.account");
        record["verification_ref"] =
            json!("oya-ci-required/cloud-ci evidence: foundry-capability-schema");
        let registry = json!([record]);
        let repo_root = temp_capability_repo("pseudo-evidence");

        let error = validate_foundry_internal_registry_value(
            &repo_root.join("registry/capabilities/foundry-internal.json"),
            &registry,
            &repo_root,
        )
        .expect_err("branded pseudo-evidence reference is rejected");

        assert!(
            error.contains("branded pseudo-evidence string"),
            "error={error}"
        );
    }

    #[test]
    fn internal_registry_rejects_stale_schema_refs() {
        let mut record = rich_internal_record("foundry.account.list", "foundry.account");
        record["mcp_contract"]["input_schema_ref"] =
            json!("specs/microservices/intelligence.json#missing-foundry-schema");
        let registry = json!([record]);
        let repo_root = temp_capability_repo("stale-schema");

        let error = validate_foundry_internal_registry_value(
            &repo_root.join("registry/capabilities/foundry-internal.json"),
            &registry,
            &repo_root,
        )
        .expect_err("stale schema reference is rejected");

        assert!(
            error.contains("stale or invented JSON fragment"),
            "error={error}"
        );
    }

    fn rich_internal_record(id: &str, namespace: &str) -> Value {
        json!({
            "id": id,
            "namespace": namespace,
            "name": "List provider accounts",
            "status": "published",
            "owner_team": "axis-intelligence",
            "autonomy_tier": "T1Read",
            "autonomy_tier_required": "T1",
            "data_classes_touched": ["INTERNAL_ONLY"],
            "evidence_emit_required": true,
            "evidence_emission_topic": format!("foundry.capability.invoke:{id}"),
            "prd_ref": "specs/microservices/intelligence.json#acceptance_criteria",
            "task_ref": "tasks/intel-capability-registry-affected-target-index-plan.md",
            "test_ref": "marketplace/facade/dev-cli/src/foundry_capability_schema_gates.rs::tests::internal_registry_accepts_rich_rows",
            "verification_ref": "marketplace/facade/dev-cli/src/foundry_capability_schema_gates.rs::tests::internal_registry_accepts_rich_rows",
            "cost_profile": {
                "per_invocation_budget_usd": "0.01",
                "monthly_budget_usd": "10"
            },
            "mcp_contract": {
                "agent_readable": "Read Foundry account metadata for an authorized internal operator.",
                "human_readable": "Read Foundry account metadata.",
                "input_schema_ref": "specs/microservices/intelligence.json#contracts",
                "output_schema_ref": "specs/microservices/intelligence.json#contracts"
            },
            "failure_modes": [
                "tenant_scope_missing",
                "evidence_emission_failed"
            ],
            "maturity": {
                "claim_boundary": "Registry metadata only; no runtime maturity claim is made without cloud-ci/oya-ci evidence.",
                "admission_ref": "docs/decisions/ADR-0700-ci-admission-live-apex.md"
            }
        })
    }
    fn temp_capability_repo(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "oya-foundry-capability-schema-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("registry/capabilities")).expect("registry dir");
        std::fs::create_dir_all(root.join("specs/microservices")).expect("specs dir");
        std::fs::create_dir_all(root.join("tasks")).expect("tasks dir");
        std::fs::create_dir_all(root.join("marketplace/facade/dev-cli/src")).expect("source dir");
        std::fs::create_dir_all(root.join("docs/decisions")).expect("decisions dir");
        std::fs::write(
            root.join("specs/root-hub-pointers.json"),
            r#"{"entry_points":{}}"#,
        )
        .expect("root hub fixture");
        std::fs::write(
            root.join("specs/microservices/intelligence.json"),
            r#"{"_meta":{"spec_id":"PRD-INTELLIGENCE"},"acceptance_criteria":[],"contracts":{}}"#,
        )
        .expect("intelligence PRD fixture");
        std::fs::write(
            root.join("tasks/intel-capability-registry-affected-target-index-plan.md"),
            "Current task artifact.\n",
        )
        .expect("task fixture");
        std::fs::write(
            root.join("marketplace/facade/dev-cli/src/foundry_capability_schema_gates.rs"),
            "fn internal_registry_accepts_rich_rows() {}\n",
        )
        .expect("test symbol fixture");
        std::fs::write(
            root.join("docs/decisions/ADR-0700-ci-admission-live-apex.md"),
            "# ADR-0515\n",
        )
        .expect("admission fixture");
        root
    }
}
