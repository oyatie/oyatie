use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use check_release_pack::{
    ComplianceRegulatorRef, ReleaseEvidencePackManifest, ReleaseEvidencePackPolicy,
    ReleaseEvidencePackRecord, validate_release_evidence_packs,
};

use crate::{
    clean_scalar_value, extract_json_array_for_key, extract_json_object_for_key,
    find_matching_json_delimiter, next_arg, parse_bool_field, parse_json_string_value,
    parse_u32_cell_field, parse_u64_field, quoted_json_len, required_field, scalar_value, usage,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReleaseEvidencePackValidateArgs {
    manifest_path: PathBuf,
    compliance_matrix_path: PathBuf,
    require_records: bool,
}

pub(crate) fn parse_release_evidence_pack_validate_args(
    args: Vec<String>,
) -> Result<ReleaseEvidencePackValidateArgs, String> {
    let mut parsed = ReleaseEvidencePackValidateArgs {
        manifest_path: PathBuf::from("registry/release/evidence-packs.tsv"),
        compliance_matrix_path: PathBuf::from("docs/machine-readable/compliance.json"),
        require_records: false,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--manifest" => parsed.manifest_path = PathBuf::from(next_arg(&mut iter)?),
            "--compliance" => parsed.compliance_matrix_path = PathBuf::from(next_arg(&mut iter)?),
            "--require-records" => parsed.require_records = true,
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_release_evidence_pack_gate(
    args: ReleaseEvidencePackValidateArgs,
) -> Result<(usize, usize, usize), String> {
    let (manifest, records) = read_release_evidence_pack_manifest(&args.manifest_path)?;
    let known_regulators = read_compliance_regulator_refs(&args.compliance_matrix_path)?;
    let policy = if args.require_records {
        ReleaseEvidencePackPolicy::release_blocking_sla()
    } else {
        ReleaseEvidencePackPolicy::compliance_matrix_sla()
    };
    let report = validate_release_evidence_packs(manifest, records, known_regulators, policy)
        .map_err(|error| format!("release evidence pack invalid: {error:?}"))?;
    Ok((
        report.known_regulators_checked,
        report.records_checked,
        report.published_records_checked,
    ))
}

fn read_release_evidence_pack_manifest(
    manifest_path: &Path,
) -> Result<(ReleaseEvidencePackManifest, Vec<ReleaseEvidencePackRecord>), String> {
    let contents = fs::read_to_string(manifest_path).map_err(|error| {
        format!(
            "release evidence-pack manifest unreadable {}: {error}",
            manifest_path.display()
        )
    })?;
    let mut release_version = None;
    let mut empty_scope_rationale = None;
    let mut seen_header = false;
    let mut records = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(comment) = trimmed.strip_prefix('#') {
            if let Some(value) = scalar_value(comment.trim(), "release_version") {
                release_version = Some(value);
            } else if let Some(value) = scalar_value(comment.trim(), "empty_scope_rationale") {
                empty_scope_rationale = Some(value);
            }
            continue;
        }
        if trimmed.starts_with("regulator\t") {
            seen_header = true;
            continue;
        }
        if !seen_header {
            return Err(format!(
                "{}: expected regulator TSV header before records",
                manifest_path.display()
            ));
        }
        records.push(parse_release_evidence_pack_row(manifest_path, trimmed)?);
    }
    if !seen_header {
        return Err(format!(
            "{}: missing regulator TSV header",
            manifest_path.display()
        ));
    }
    Ok((
        ReleaseEvidencePackManifest {
            release_version: release_version.unwrap_or_default(),
            empty_scope_rationale: empty_scope_rationale.unwrap_or_default(),
        },
        records,
    ))
}

fn parse_release_evidence_pack_row(
    manifest_path: &Path,
    row: &str,
) -> Result<ReleaseEvidencePackRecord, String> {
    let cells = row
        .split('\t')
        .map(|cell| cell.trim().trim_matches('`').to_string())
        .collect::<Vec<_>>();
    if cells.len() != 18 {
        return Err(format!(
            "{}: release evidence-pack row must have 18 TSV columns: {row}",
            manifest_path.display()
        ));
    }
    Ok(ReleaseEvidencePackRecord {
        regulator: cells[0].clone(),
        region: cells[1].clone(),
        pack_id: cells[2].clone(),
        release_version: cells[3].clone(),
        audit_cycle: cells[4].clone(),
        coverage_window_start: cells[5].clone(),
        coverage_window_end: cells[6].clone(),
        owner_team: cells[7].clone(),
        evidence_pack_ref: cells[8].clone(),
        cosign_attestation_ref: cells[9].clone(),
        audit_event_id: cells[10].clone(),
        requested_at_epoch_minutes: parse_u64_field(&cells[11], "requested_at_epoch_minutes")?,
        regenerated_at_epoch_minutes: parse_u64_field(&cells[12], "regenerated_at_epoch_minutes")?,
        controls_mapped: parse_u32_cell_field(manifest_path, "controls_mapped", &cells[13])?,
        evidence_links: parse_u32_cell_field(manifest_path, "evidence_links", &cells[14])?,
        trust_portal_mirror_regenerated: parse_bool_field(
            manifest_path,
            "trust_portal_mirror_regenerated",
            &cells[15],
        )?,
        regulator_notification_sent: parse_bool_field(
            manifest_path,
            "regulator_notification_sent",
            &cells[16],
        )?,
        status: cells[17].clone(),
    })
}

fn read_compliance_regulator_refs(path: &Path) -> Result<Vec<ComplianceRegulatorRef>, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "machine-readable compliance matrix unreadable {}: {error}",
            path.display()
        )
    })?;
    let regulators_object = extract_json_object_for_key(&contents, "regulators_per_region")
        .ok_or_else(|| "compliance matrix missing regulators_per_region".to_string())?;
    let mut refs = BTreeMap::new();
    for regulator in collect_json_array_string_values(regulators_object) {
        refs.insert(regulator.clone(), ComplianceRegulatorRef { regulator });
    }
    let standards_array = extract_json_array_for_key(&contents, "cross_regional_standards")
        .ok_or_else(|| "compliance matrix missing cross_regional_standards".to_string())?;
    for regulator in json_string_values_in_array(standards_array) {
        refs.insert(regulator.clone(), ComplianceRegulatorRef { regulator });
    }
    Ok(refs.into_values().collect())
}

fn collect_json_array_string_values(contents: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut rest = contents;
    while let Some(array_start_index) = rest.find('[') {
        let array = &rest[array_start_index..];
        let Some(array_end_index) = find_matching_json_delimiter(array, '[', ']') else {
            break;
        };
        values.extend(json_string_values_in_array(&array[1..array_end_index]));
        rest = &array[array_end_index + 1..];
    }
    values
}

fn json_string_values_in_array(array: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut rest = array;
    while let Some(quote_index) = rest.find('"') {
        let value_start = &rest[quote_index..];
        let Some(value) = parse_json_string_value(value_start) else {
            break;
        };
        if !value.trim().is_empty() {
            values.push(value);
        }
        let Some(consumed) = quoted_json_len(value_start) else {
            break;
        };
        rest = &value_start[consumed..];
    }
    values
}
