#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub detail: String,
}

impl Finding {
    fn new(code: &str, key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.into(),
            detail: detail.into(),
        }
    }
}

pub fn evaluate_schema(policy: &Value, schema: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let Some(uri) = policy.get("expected_schema_uri").and_then(Value::as_str) else {
        findings.insert(Finding::new(
            "reset_policy_malformed",
            "expected_schema_uri",
            "policy requires an expected schema URI",
        ));
        return findings;
    };
    if schema.get("$id").and_then(Value::as_str) != Some(uri) {
        findings.insert(Finding::new(
            "reset_schema_binding_mismatch",
            "$id",
            "schema $id does not match policy",
        ));
    }
    let expected_version = policy
        .get("expected_schema_version")
        .and_then(Value::as_u64);
    let schema_version = schema
        .pointer("/properties/schema_version/const")
        .and_then(Value::as_u64);
    if expected_version.is_none() || schema_version != expected_version {
        findings.insert(Finding::new(
            "reset_schema_binding_mismatch",
            "schema_version",
            "schema version const does not match policy",
        ));
    }
    for field in [
        "scope",
        "sources",
        "inventory",
        "recovery",
        "hard_stops",
        "unknowns",
        "approvals",
        "decision",
    ] {
        let required = schema
            .get("required")
            .and_then(Value::as_array)
            .is_some_and(|values| values.iter().any(|value| value.as_str() == Some(field)));
        if !required {
            findings.insert(Finding::new(
                "reset_schema_fail_open",
                field,
                "required reset eligibility field is optional",
            ));
        }
    }
    findings
}

pub fn evaluate(policy: &Value, artifact: &Value, evaluated_at_epoch: i64) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    if policy.get("gate_id").and_then(Value::as_str) != Some("cloud-ci-reset-eligibility-policy") {
        findings.insert(Finding::new(
            "reset_policy_malformed",
            "gate_id",
            "unexpected or missing gate_id",
        ));
    }
    let expected_version = policy
        .get("expected_schema_version")
        .and_then(Value::as_u64);
    if expected_version.is_none()
        || artifact.get("schema_version").and_then(Value::as_u64) != expected_version
    {
        findings.insert(Finding::new(
            "reset_artifact_malformed",
            "schema_version",
            "artifact schema version does not match policy",
        ));
    }
    if artifact.get("$schema").and_then(Value::as_str)
        != policy.get("expected_schema_uri").and_then(Value::as_str)
    {
        findings.insert(Finding::new(
            "reset_schema_binding_mismatch",
            "$schema",
            "artifact schema URI does not match policy",
        ));
    }

    require_nonempty_string(artifact, "reset_id", &mut findings);
    for pointer in [
        "/repository/protected_commit_sha",
        "/repository/candidate_commit_sha",
    ] {
        let valid = artifact
            .pointer(pointer)
            .and_then(Value::as_str)
            .is_some_and(|value| is_hex(value, 40));
        if !valid {
            findings.insert(Finding::new(
                "reset_artifact_malformed",
                pointer,
                "repository commit must be a 40-character lowercase hexadecimal SHA",
            ));
        }
    }
    if artifact
        .pointer("/collector/method")
        .and_then(Value::as_str)
        != Some("read-only")
        || artifact
            .pointer("/collector/secret_values_excluded")
            .and_then(Value::as_bool)
            != Some(true)
    {
        findings.insert(Finding::new(
            "reset_collection_boundary_invalid",
            "collector",
            "collector must be read-only and exclude secret values",
        ));
    }
    for pointer in [
        "/scope/providers",
        "/scope/accounts",
        "/scope/regions",
        "/scope/clusters",
    ] {
        if !nonempty_unique_strings(artifact.pointer(pointer)) {
            findings.insert(Finding::new(
                "reset_scope_incomplete",
                pointer,
                "scope dimension must contain unique non-empty identifiers",
            ));
        }
    }

    let expected_manifest = policy
        .get("expected_evidence_manifest_sha256")
        .and_then(Value::as_str);
    if expected_manifest.is_none()
        || artifact
            .pointer("/evidence_manifest/sha256")
            .and_then(Value::as_str)
            != expected_manifest
    {
        findings.insert(Finding::new(
            "reset_evidence_manifest_mismatch",
            "evidence_manifest.sha256",
            "evidence manifest digest does not match the reviewed discovery manifest",
        ));
    }
    if artifact
        .pointer("/evidence_manifest/uri")
        .and_then(Value::as_str)
        .is_none_or(str::is_empty)
    {
        findings.insert(Finding::new(
            "reset_artifact_malformed",
            "evidence_manifest.uri",
            "evidence manifest URI is required",
        ));
    }

    let max_validity = policy.get("max_validity_seconds").and_then(Value::as_i64);
    let captured = artifact
        .get("captured_at")
        .and_then(Value::as_str)
        .and_then(parse_rfc3339_utc);
    let expires = artifact
        .get("expires_at")
        .and_then(Value::as_str)
        .and_then(parse_rfc3339_utc);
    match (captured, expires, max_validity) {
        (Some(captured), Some(expires), Some(max))
            if expires > captured && expires - captured <= max => {}
        _ => {
            findings.insert(Finding::new(
                "reset_evidence_window_invalid",
                "captured_at/expires_at",
                "evidence window must be positive and no longer than policy max_validity_seconds",
            ));
        }
    }

    let allowed_results = string_set(policy.get("allowed_source_results"));
    let mut sources_incomplete = false;
    let sources = artifact.get("sources").and_then(Value::as_array);
    if sources.is_none_or(Vec::is_empty) {
        findings.insert(Finding::new(
            "reset_sources_missing",
            "sources",
            "at least one source observation is required",
        ));
        sources_incomplete = true;
    } else if let Some(sources) = sources {
        let mut ids = BTreeSet::new();
        for (index, source) in sources.iter().enumerate() {
            let key = format!("sources[{index}]");
            let source_id = source
                .get("source_id")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty());
            if source_id.is_none() || !ids.insert(source_id.unwrap_or_default()) {
                findings.insert(Finding::new(
                    "reset_source_malformed",
                    &key,
                    "source_id must be non-empty and unique",
                ));
            }
            let result = source.get("result").and_then(Value::as_str);
            if result.is_none() || !allowed_results.contains(result.unwrap_or_default()) {
                findings.insert(Finding::new(
                    "reset_source_malformed",
                    &key,
                    "source result is not allowed by policy",
                ));
                sources_incomplete = true;
            } else if result != Some("observed") {
                sources_incomplete = true;
            }
            if source.get("redaction").and_then(Value::as_str) != Some("secret-values-excluded")
                || source
                    .get("evidence_uri")
                    .and_then(Value::as_str)
                    .is_none_or(str::is_empty)
            {
                findings.insert(Finding::new(
                    "reset_source_malformed",
                    &key,
                    "source requires a redacted evidence URI",
                ));
            }
            if result == Some("observed")
                && !source
                    .get("sha256")
                    .and_then(Value::as_str)
                    .is_some_and(is_sha256)
            {
                findings.insert(Finding::new(
                    "reset_source_malformed",
                    &key,
                    "observed source requires a sha256 digest",
                ));
            }
        }
    }

    let inventory = artifact.get("inventory").and_then(Value::as_array);
    if inventory.is_none_or(Vec::is_empty) {
        findings.insert(Finding::new(
            "reset_inventory_missing",
            "inventory",
            "at least one stable inventory identity is required",
        ));
    } else if let Some(inventory) = inventory {
        let mut ids = BTreeSet::new();
        for (index, item) in inventory.iter().enumerate() {
            let key = format!("inventory[{index}]");
            let stable_id = item
                .get("stable_id")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty());
            if stable_id.is_none() || !ids.insert(stable_id.unwrap_or_default()) {
                findings.insert(Finding::new(
                    "reset_inventory_malformed",
                    &key,
                    "stable_id must be non-empty and unique",
                ));
            }
            if !item
                .get("identity_hash")
                .and_then(Value::as_str)
                .is_some_and(is_sha256)
            {
                findings.insert(Finding::new(
                    "reset_inventory_malformed",
                    &key,
                    "identity_hash must be sha256",
                ));
            }
            for field in [
                "lifecycle",
                "data_class",
                "retention",
                "billing",
                "deletion_semantics",
            ] {
                if item
                    .get(field)
                    .and_then(Value::as_str)
                    .is_none_or(str::is_empty)
                {
                    findings.insert(Finding::new(
                        "reset_inventory_malformed",
                        format!("{key}.{field}"),
                        "inventory lifecycle field must be non-empty",
                    ));
                }
            }
        }
    }

    let recovery_fields = [
        "backup_verified",
        "immutable_backup_location_verified",
        "rpo_verified",
        "restore_drill_verified",
        "key_recovery_verified",
    ];
    let recovery_incomplete = recovery_fields.iter().any(|field| {
        artifact
            .pointer(&format!("/recovery/{field}"))
            .and_then(Value::as_bool)
            != Some(true)
    });
    let hard_stops_present = validate_blockers(artifact.get("hard_stops"), &mut findings);
    let unknowns_present = validate_unknowns(artifact.get("unknowns"), &mut findings);
    let approvals_incomplete = validate_approvals(policy, artifact.get("approvals"), &mut findings);

    let forbidden_secret_keys = string_set(policy.get("forbidden_secret_keys"));
    scan_forbidden_keys(artifact, "$", &forbidden_secret_keys, &mut findings);

    let stale_positive = captured.is_none_or(|capture| evaluated_at_epoch < capture)
        || expires.is_none_or(|expiry| evaluated_at_epoch >= expiry);
    let authorization_enabled = policy
        .get("reset_authorization_enabled")
        .and_then(Value::as_bool)
        == Some(true);
    let computed_eligible = authorization_enabled
        && !sources_incomplete
        && !recovery_incomplete
        && !hard_stops_present
        && !unknowns_present
        && !approvals_incomplete
        && !stale_positive
        && findings.is_empty();

    let mut reasons = BTreeSet::new();
    if sources_incomplete {
        reasons.insert("sources-incomplete");
    }
    if recovery_incomplete {
        reasons.insert("recovery-incomplete");
    }
    if hard_stops_present {
        reasons.insert("hard-stops-present");
    }
    if unknowns_present {
        reasons.insert("unknowns-present");
    }
    if approvals_incomplete {
        reasons.insert("approvals-incomplete");
    }
    if stale_positive {
        reasons.insert("evidence-expired");
    }
    if !authorization_enabled {
        reasons.insert("authorization-disabled");
    }

    let decision_eligible = artifact
        .pointer("/decision/eligible")
        .and_then(Value::as_bool);
    let expected_mode = if computed_eligible {
        "authorized-reset"
    } else {
        "preservation-migration"
    };
    if decision_eligible != Some(computed_eligible)
        || artifact.pointer("/decision/mode").and_then(Value::as_str) != Some(expected_mode)
        || artifact
            .pointer("/decision/default_if_unknown")
            .and_then(Value::as_str)
            != Some("ineligible")
    {
        findings.insert(Finding::new(
            "reset_decision_mismatch",
            "decision",
            "declared decision does not match fail-closed computed eligibility",
        ));
    }
    let declared_reasons = string_set(artifact.pointer("/decision/reason_codes"));
    if declared_reasons != reasons {
        findings.insert(Finding::new(
            "reset_reason_codes_mismatch",
            "decision.reason_codes",
            format!("declared reason codes {declared_reasons:?} do not match computed {reasons:?}"),
        ));
    }
    findings
}

fn require_nonempty_string(value: &Value, key: &str, findings: &mut BTreeSet<Finding>) {
    if value
        .get(key)
        .and_then(Value::as_str)
        .is_none_or(str::is_empty)
    {
        findings.insert(Finding::new(
            "reset_artifact_malformed",
            key,
            "required non-empty string is missing",
        ));
    }
}

fn nonempty_unique_strings(value: Option<&Value>) -> bool {
    let Some(values) = value.and_then(Value::as_array) else {
        return false;
    };
    if values.is_empty() {
        return false;
    }
    let strings = values
        .iter()
        .filter_map(Value::as_str)
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    strings.len() == values.len()
}

fn string_set(value: Option<&Value>) -> BTreeSet<&str> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect()
}

fn is_hex(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_sha256(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .is_some_and(|digest| is_hex(digest, 64))
}

fn validate_blockers(value: Option<&Value>, findings: &mut BTreeSet<Finding>) -> bool {
    let Some(rows) = value.and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "reset_artifact_malformed",
            "hard_stops",
            "hard_stops must be an array",
        ));
        return true;
    };
    let mut ids = BTreeSet::new();
    for (index, row) in rows.iter().enumerate() {
        let key = format!("hard_stops[{index}]");
        let id = row
            .get("id")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty());
        if id.is_none() || !ids.insert(id.unwrap_or_default()) {
            findings.insert(Finding::new(
                "reset_hard_stop_malformed",
                &key,
                "hard-stop id must be non-empty and unique",
            ));
        }
        for field in [
            "class",
            "acceptance_criteria",
            "verification_path",
            "suggested_owner",
            "dependency_notes",
        ] {
            if row
                .get(field)
                .and_then(Value::as_str)
                .is_none_or(str::is_empty)
            {
                findings.insert(Finding::new(
                    "reset_hard_stop_malformed",
                    format!("{key}.{field}"),
                    "blocker field must be non-empty",
                ));
            }
        }
    }
    !rows.is_empty()
}

fn validate_unknowns(value: Option<&Value>, findings: &mut BTreeSet<Finding>) -> bool {
    let Some(rows) = value.and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "reset_artifact_malformed",
            "unknowns",
            "unknowns must be an array",
        ));
        return true;
    };
    let mut ids = BTreeSet::new();
    for (index, row) in rows.iter().enumerate() {
        let key = format!("unknowns[{index}]");
        let id = row
            .get("id")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty());
        if id.is_none() || !ids.insert(id.unwrap_or_default()) {
            findings.insert(Finding::new(
                "reset_unknown_malformed",
                &key,
                "unknown id must be non-empty and unique",
            ));
        }
        for field in ["owner", "closure_probe"] {
            if row
                .get(field)
                .and_then(Value::as_str)
                .is_none_or(str::is_empty)
            {
                findings.insert(Finding::new(
                    "reset_unknown_malformed",
                    format!("{key}.{field}"),
                    "unknown closure field must be non-empty",
                ));
            }
        }
    }
    !rows.is_empty()
}

fn validate_approvals(
    policy: &Value,
    value: Option<&Value>,
    findings: &mut BTreeSet<Finding>,
) -> bool {
    let required = string_set(policy.get("required_approval_roles"));
    if required.is_empty() {
        findings.insert(Finding::new(
            "reset_policy_malformed",
            "required_approval_roles",
            "policy must require at least one approval role",
        ));
        return true;
    }
    let Some(rows) = value.and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "reset_artifact_malformed",
            "approvals",
            "approvals must be an array",
        ));
        return true;
    };
    let approved = rows
        .iter()
        .filter(|row| row.get("approved").and_then(Value::as_bool) == Some(true))
        .filter_map(|row| row.get("role").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();
    !required.is_subset(&approved)
}

fn scan_forbidden_keys(
    value: &Value,
    path: &str,
    forbidden: &BTreeSet<&str>,
    findings: &mut BTreeSet<Finding>,
) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let child_path = format!("{path}.{key}");
                if forbidden.contains(key.as_str()) {
                    findings.insert(Finding::new(
                        "reset_secret_bearing_field",
                        &child_path,
                        "secret-bearing field names are forbidden in reset evidence",
                    ));
                }
                scan_forbidden_keys(child, &child_path, forbidden, findings);
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                scan_forbidden_keys(child, &format!("{path}[{index}]"), forbidden, findings);
            }
        }
        _ => {}
    }
}

fn parse_rfc3339_utc(value: &str) -> Option<i64> {
    if value.len() != 20
        || &value[4..5] != "-"
        || &value[7..8] != "-"
        || &value[10..11] != "T"
        || &value[13..14] != ":"
        || &value[16..17] != ":"
        || &value[19..20] != "Z"
    {
        return None;
    }
    let year = value[0..4].parse::<i64>().ok()?;
    let month = value[5..7].parse::<i64>().ok()?;
    let day = value[8..10].parse::<i64>().ok()?;
    let hour = value[11..13].parse::<i64>().ok()?;
    let minute = value[14..16].parse::<i64>().ok()?;
    let second = value[17..19].parse::<i64>().ok()?;
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }
    let adjusted_year = year - i64::from(month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let shifted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days_since_epoch = era * 146_097 + day_of_era - 719_468;
    Some(days_since_epoch * 86_400 + hour * 3_600 + minute * 60 + second)
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::evaluate;

    const CAPTURED: i64 = 1_785_607_169;

    fn policy() -> Value {
        serde_json::from_str(include_str!("../reset-eligibility-policy.json")).expect("policy")
    }

    fn eligible_fixture() -> Value {
        json!({
            "$schema": "https://docs.oyatie.com/schemas/reset-eligibility.schema.json",
            "schema_version": 1,
            "reset_id": "fixture-reset",
            "captured_at": "2026-08-01T17:59:29Z",
            "expires_at": "2026-08-02T17:59:29Z",
            "repository": {"protected_commit_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "candidate_commit_sha": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"},
            "collector": {"id": "fixture", "method": "read-only", "secret_values_excluded": true},
            "scope": {"providers": ["fixture"], "accounts": ["fixture-account"], "regions": ["fixture-region"], "clusters": ["fixture-cluster"]},
            "evidence_manifest": {"uri": "redacted-local-manifest:fixture", "sha256": "sha256:208783ed5d85345be22f336da0dd6c5425a5176a425512fdadf42715c2064f5c"},
            "sources": [{"source_id": "fixture", "result": "observed", "redaction": "secret-values-excluded", "evidence_uri": "redacted-local:fixture", "sha256": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}],
            "inventory": [{"stable_id": "fixture:one", "identity_hash": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "lifecycle": "replaceable", "data_class": "none", "retention": "none", "billing": "known", "deletion_semantics": "verified"}],
            "recovery": {"backup_verified": true, "immutable_backup_location_verified": true, "rpo_verified": true, "restore_drill_verified": true, "key_recovery_verified": true},
            "hard_stops": [],
            "unknowns": [],
            "approvals": [
                {"role": "founder", "approver": "fixture-founder", "approved": true, "approved_at": "2026-08-01T18:00:00Z"},
                {"role": "platform-operations", "approver": "fixture-ops", "approved": true, "approved_at": "2026-08-01T18:00:00Z"},
                {"role": "data-protection", "approver": "fixture-data", "approved": true, "approved_at": "2026-08-01T18:00:00Z"}
            ],
            "decision": {"eligible": false, "mode": "preservation-migration", "default_if_unknown": "ineligible", "reason_codes": ["authorization-disabled"]}
        })
    }

    #[test]
    fn complete_fresh_evidence_cannot_self_authorize_a_reset() {
        let mut artifact = eligible_fixture();
        assert!(evaluate(&policy(), &artifact, CAPTURED + 3600).is_empty());
        artifact["decision"] = json!({"eligible":true,"mode":"authorized-reset","default_if_unknown":"ineligible","reason_codes":[]});
        assert!(!evaluate(&policy(), &artifact, CAPTURED + 3600).is_empty());
    }

    #[test]
    fn unknown_hard_stop_and_missing_recovery_force_preservation() {
        let mut artifact = eligible_fixture();
        artifact["unknowns"] = json!([{"id":"provider-scope", "owner":"platform-operations", "closure_probe":"enumerate provider accounts"}]);
        artifact["hard_stops"] = json!([{"id":"live-stateful-data", "class":"data-loss", "acceptance_criteria":"verified immutable backup", "verification_path":"restore drill", "suggested_owner":"data-protection", "dependency_notes":"none"}]);
        artifact["recovery"]["backup_verified"] = json!(false);
        artifact["decision"] = json!({"eligible":false,"mode":"preservation-migration","default_if_unknown":"ineligible","reason_codes":["authorization-disabled","hard-stops-present","unknowns-present","recovery-incomplete"]});
        assert!(evaluate(&policy(), &artifact, CAPTURED + 3600).is_empty());
        artifact["decision"] = json!({"eligible":true,"mode":"authorized-reset","default_if_unknown":"ineligible","reason_codes":[]});
        assert!(!evaluate(&policy(), &artifact, CAPTURED + 3600).is_empty());
    }

    #[test]
    fn stale_positive_or_overlong_window_is_red() {
        let artifact = eligible_fixture();
        assert!(!evaluate(&policy(), &artifact, CAPTURED + 86_401).is_empty());
        let mut overlong = eligible_fixture();
        overlong["expires_at"] = json!("2026-08-02T18:00:00Z");
        assert!(!evaluate(&policy(), &overlong, CAPTURED + 3600).is_empty());
    }

    #[test]
    fn incomplete_approvals_and_secret_bearing_fields_are_red() {
        let mut artifact = eligible_fixture();
        artifact["approvals"] = json!([]);
        artifact["token"] = json!("must-never-appear");
        assert!(!evaluate(&policy(), &artifact, CAPTURED + 3600).is_empty());
    }
}
