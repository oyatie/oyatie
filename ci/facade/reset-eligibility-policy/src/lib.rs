#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use ed25519_dalek::{Signature, VerifyingKey};
use serde_json::Value;
use sha2::{Digest, Sha256};

const REQUIRED_SOURCE_IDS: [&str; 8] = [
    "github-and-protected-ci",
    "kubernetes-inventory",
    "stateful-workloads",
    "openbao-and-registry",
    "backup-jobs",
    "dns-and-edge",
    "provider-billing-and-global-inventory",
    "external-backups-and-saas-callbacks",
];
const REQUIRED_INVENTORY_IDS: [&str; 7] = [
    "talos-current:cluster",
    "kubernetes-current:persistent-volumes",
    "kubernetes-current:postgres",
    "kubernetes-current:openbao",
    "kubernetes-current:oci-registry",
    "github:jason931225/oyatie:arc-runners",
    "cloudflare-configured:active-edge",
];

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

#[derive(Clone, Debug)]
pub struct TrustedApprovalKey {
    pub verifying_key: [u8; 32],
    pub principal: String,
    pub allowed_role: String,
}

#[derive(Clone, Debug)]
pub struct RuntimeAuthority {
    pub protected_commit_sha: String,
    pub candidate_commit_sha: String,
    pub required_approval_roles: BTreeSet<String>,
    pub trusted_approval_keys: BTreeMap<String, TrustedApprovalKey>,
    pub evidence_by_source: BTreeMap<String, Vec<u8>>,
}

impl RuntimeAuthority {
    pub fn fail_closed(
        protected_commit_sha: impl Into<String>,
        candidate_commit_sha: impl Into<String>,
    ) -> Self {
        Self {
            protected_commit_sha: protected_commit_sha.into(),
            candidate_commit_sha: candidate_commit_sha.into(),
            required_approval_roles: ["founder", "platform-operations", "data-protection"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            trusted_approval_keys: BTreeMap::new(),
            evidence_by_source: BTreeMap::new(),
        }
    }
}

pub fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

pub fn json_digest(value: &Value) -> String {
    sha256(&serde_json::to_vec(value).expect("serializing a serde_json::Value cannot fail"))
}

pub fn evidence_digest(evidence: &BTreeMap<String, Vec<u8>>) -> String {
    let mut hasher = Sha256::new();
    for (source_id, bytes) in evidence {
        hasher.update((source_id.len() as u64).to_be_bytes());
        hasher.update(source_id.as_bytes());
        hasher.update(Sha256::digest(bytes));
    }
    format!("sha256:{:x}", hasher.finalize())
}

/// Validate this artifact with every JSON Schema keyword used by the committed schema.
/// This is deliberately a schema-specific Draft 2020-12 subset, not a claim of general
/// JSON Schema support. `evaluate_schema` rejects any unsupported schema keyword.
pub fn validate_artifact_schema(schema: &Value, artifact: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    validate_node(schema, schema, artifact, "$", &mut findings);
    findings
}

pub fn evaluate_schema(policy: &Value, schema: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    if policy
        .get("reset_authorization_enabled")
        .and_then(Value::as_bool)
        != Some(false)
    {
        findings.insert(Finding::new(
            "reset_policy_candidate_authority",
            "reset_authorization_enabled",
            "repo policy must remain false; W0-D evaluates eligibility only and has no actuation authority",
        ));
    }
    if policy.get("max_validity_seconds").and_then(Value::as_i64) != Some(86_400) {
        findings.insert(Finding::new(
            "reset_policy_malformed",
            "max_validity_seconds",
            "repo policy must retain the fixed 24-hour ceiling",
        ));
    }
    let allowed = string_set(policy.get("allowed_source_results"));
    let expected_allowed = ["observed", "unknown", "denied", "error"]
        .into_iter()
        .collect::<BTreeSet<_>>();
    if allowed != expected_allowed {
        findings.insert(Finding::new(
            "reset_policy_malformed",
            "allowed_source_results",
            "candidate policy cannot broaden or narrow source result semantics",
        ));
    }
    let forbidden = string_set(policy.get("forbidden_secret_keys"));
    let expected_forbidden = [
        "password",
        "token",
        "secret_value",
        "private_key",
        "credential_value",
        "recovery_key",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    if forbidden != expected_forbidden {
        findings.insert(Finding::new(
            "reset_policy_malformed",
            "forbidden_secret_keys",
            "candidate policy cannot weaken the secret-bearing field denylist",
        ));
    }
    let Some(uri) = policy.get("expected_schema_uri").and_then(Value::as_str) else {
        findings.insert(Finding::new(
            "reset_policy_malformed",
            "expected_schema_uri",
            "policy requires an expected schema URI",
        ));
        return findings;
    };
    if schema.get("$schema").and_then(Value::as_str)
        != Some("https://json-schema.org/draft/2020-12/schema")
        || schema.get("$id").and_then(Value::as_str) != Some(uri)
    {
        findings.insert(Finding::new(
            "reset_schema_binding_mismatch",
            "$schema/$id",
            "schema must declare Draft 2020-12 and the policy-bound identifier",
        ));
    }
    if schema
        .pointer("/properties/schema_version/const")
        .and_then(Value::as_u64)
        != policy
            .get("expected_schema_version")
            .and_then(Value::as_u64)
    {
        findings.insert(Finding::new(
            "reset_schema_binding_mismatch",
            "schema_version",
            "schema version const does not match policy",
        ));
    }
    for (pointer, expected, key) in [
        (
            "/properties/sources/items/properties/source_id/enum",
            REQUIRED_SOURCE_IDS.as_slice(),
            "sources.source_id",
        ),
        (
            "/properties/inventory/items/properties/stable_id/enum",
            REQUIRED_INVENTORY_IDS.as_slice(),
            "inventory.stable_id",
        ),
    ] {
        if string_set(schema.pointer(pointer)) != expected.iter().copied().collect() {
            findings.insert(Finding::new(
                "reset_schema_binding_mismatch",
                key,
                "schema must retain the policy-owned closed identifier set",
            ));
        }
    }
    let supported: BTreeSet<&str> = [
        "$schema",
        "$id",
        "$defs",
        "$ref",
        "title",
        "description",
        "type",
        "required",
        "properties",
        "additionalProperties",
        "items",
        "minItems",
        "uniqueItems",
        "minLength",
        "pattern",
        "format",
        "enum",
        "const",
    ]
    .into_iter()
    .collect();
    reject_unsupported_keywords(schema, "$", &supported, true, &mut findings);
    validate_schema_keyword_shapes(schema, schema, "$", &mut findings);
    findings
}

pub fn evaluate(
    policy: &Value,
    schema: &Value,
    artifact: &Value,
    runtime: &RuntimeAuthority,
    evaluated_at_epoch: i64,
) -> BTreeSet<Finding> {
    let mut findings = evaluate_schema(policy, schema);
    findings.extend(validate_artifact_schema(schema, artifact));
    if artifact.get("$schema") != policy.get("expected_schema_uri") {
        findings.insert(Finding::new(
            "reset_schema_binding_mismatch",
            "$schema",
            "artifact schema URI does not match policy",
        ));
    }
    if artifact
        .pointer("/repository/binding_source")
        .and_then(Value::as_str)
        != Some("ci-runtime-authority")
    {
        findings.insert(Finding::new(
            "reset_repository_binding_invalid",
            "repository.binding_source",
            "commit identities must come from CI runtime authority",
        ));
    }
    if !is_hex(&runtime.protected_commit_sha, 40) || !is_hex(&runtime.candidate_commit_sha, 40) {
        findings.insert(Finding::new(
            "reset_runtime_authority_malformed",
            "repository",
            "runtime commit identities must be exact 40-character lowercase SHAs",
        ));
    }

    let captured = artifact
        .get("captured_at")
        .and_then(Value::as_str)
        .and_then(parse_rfc3339_utc);
    let expires = artifact
        .get("expires_at")
        .and_then(Value::as_str)
        .and_then(parse_rfc3339_utc);
    let max = policy.get("max_validity_seconds").and_then(Value::as_i64);
    if !matches!((captured, expires, max), (Some(c), Some(e), Some(m)) if e > c && e - c <= m) {
        findings.insert(Finding::new(
            "reset_evidence_window_invalid",
            "captured_at/expires_at",
            "evidence window must be positive and no longer than the fixed policy maximum",
        ));
    }

    let mut sources_incomplete = false;
    let mut source_ids = BTreeSet::new();
    for (index, source) in artifact
        .get("sources")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let key = format!("sources[{index}]");
        let id = source
            .get("source_id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if id.is_empty() || !source_ids.insert(id) {
            findings.insert(Finding::new(
                "reset_source_malformed",
                &key,
                "source_id must be non-empty and unique",
            ));
        }
        if source.get("result").and_then(Value::as_str) == Some("observed") {
            match runtime.evidence_by_source.get(id) {
                Some(bytes)
                    if source.get("sha256").and_then(Value::as_str)
                        == Some(sha256(bytes).as_str()) => {}
                _ => {
                    findings.insert(Finding::new("reset_source_evidence_unverified", &key, "observed evidence bytes are absent or do not rehash to the declared digest"));
                    sources_incomplete = true;
                }
            }
        } else {
            sources_incomplete = true;
        }
    }
    let expected_source_ids = REQUIRED_SOURCE_IDS.into_iter().collect::<BTreeSet<_>>();
    if source_ids != expected_source_ids {
        findings.insert(Finding::new(
            "reset_source_set_mismatch",
            "sources",
            "sources must contain exactly the eight policy-owned source identifiers",
        ));
        sources_incomplete = true;
    }

    let mut inventory_ids = BTreeSet::new();
    for (index, item) in artifact
        .get("inventory")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let id = item
            .get("stable_id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if id.is_empty() || !inventory_ids.insert(id) {
            findings.insert(Finding::new(
                "reset_inventory_malformed",
                format!("inventory[{index}]"),
                "stable_id must be non-empty and unique",
            ));
        }
    }
    let expected_inventory_ids = REQUIRED_INVENTORY_IDS.into_iter().collect::<BTreeSet<_>>();
    if inventory_ids != expected_inventory_ids {
        findings.insert(Finding::new(
            "reset_inventory_set_mismatch",
            "inventory",
            "inventory must contain exactly the seven policy-owned stable identifiers",
        ));
    }
    let rehashed_evidence_digest = evidence_digest(&runtime.evidence_by_source);
    let manifest_verified = artifact
        .pointer("/evidence_manifest/status")
        .and_then(Value::as_str)
        == Some("verified")
        && artifact
            .pointer("/evidence_manifest/sha256")
            .and_then(Value::as_str)
            == Some(rehashed_evidence_digest.as_str())
        && !runtime.evidence_by_source.is_empty();
    if !manifest_verified {
        sources_incomplete = true;
    }

    let recovery_incomplete = [
        "backup_verified",
        "immutable_backup_location_verified",
        "rpo_verified",
        "restore_drill_verified",
        "key_recovery_verified",
    ]
    .iter()
    .any(|field| {
        artifact
            .pointer(&format!("/recovery/{field}"))
            .and_then(Value::as_bool)
            != Some(true)
    });
    let hard_stops_present = artifact
        .get("hard_stops")
        .and_then(Value::as_array)
        .is_none_or(|v| !v.is_empty());
    let unknowns_present = artifact
        .get("unknowns")
        .and_then(Value::as_array)
        .is_none_or(|v| !v.is_empty());
    let approvals_complete = validate_approval_receipts(
        artifact,
        runtime,
        &rehashed_evidence_digest,
        policy,
        schema,
        evaluated_at_epoch,
        &mut findings,
    );
    let forbidden = string_set(policy.get("forbidden_secret_keys"));
    scan_forbidden_secrets(artifact, "$", &forbidden, &mut findings);
    let stale = captured.is_none_or(|c| evaluated_at_epoch < c)
        || expires.is_none_or(|e| evaluated_at_epoch >= e);
    // W0-D is intentionally dormant: it can prove prerequisites incomplete or complete,
    // but cannot authorize destructive actuation. A future protected one-time actuation
    // boundary must define operation/scope binding, a protected nonce, and atomic consumption.
    let computed_eligible = false;

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
    if !approvals_complete {
        reasons.insert("approvals-incomplete");
    }
    reasons.insert("authorization-disabled");
    if stale {
        reasons.insert("evidence-expired");
    }
    let expected_mode = "preservation-migration";
    if artifact
        .pointer("/decision/eligible")
        .and_then(Value::as_bool)
        != Some(computed_eligible)
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
    if string_set(artifact.pointer("/decision/reason_codes")) != reasons {
        findings.insert(Finding::new(
            "reset_reason_codes_mismatch",
            "decision.reason_codes",
            "declared reason codes do not match the computed decision",
        ));
    }
    findings
}

fn validate_approval_receipts(
    artifact: &Value,
    runtime: &RuntimeAuthority,
    evidence: &str,
    policy: &Value,
    schema: &Value,
    now: i64,
    findings: &mut BTreeSet<Finding>,
) -> bool {
    let mut approved = BTreeSet::new();
    let mut principals = BTreeSet::new();
    let mut key_ids = BTreeSet::new();
    let mut public_keys = BTreeSet::new();
    for (index, receipt) in artifact
        .get("approvals")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let role = receipt
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let key_id = receipt
            .get("key_id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let trusted = runtime.trusted_approval_keys.get(key_id);
        let identity_valid = trusted.is_some_and(|key| {
            receipt.get("approver").and_then(Value::as_str) == Some(key.principal.as_str())
                && role == key.allowed_role.as_str()
        });
        let cryptographically_valid = identity_valid
            && receipt_binding_valid(receipt, artifact, runtime, evidence, policy, schema, now)
            && verify_receipt(receipt, trusted.map(|key| &key.verifying_key));
        let valid = cryptographically_valid
            && trusted.is_some_and(|key| {
                principals.insert(key.principal.as_str())
                    && key_ids.insert(key_id)
                    && public_keys.insert(key.verifying_key)
            });
        if valid {
            approved.insert(role.to_owned());
        } else {
            findings.insert(Finding::new("reset_approval_receipt_invalid", format!("approvals[{index}]"), "approval must use a unique principal and key bound by runtime authority to exactly one allowed role, and its signature must bind reset/evidence/commits/policy/schema"));
        }
    }
    runtime.required_approval_roles.is_subset(&approved)
}

fn receipt_binding_valid(
    receipt: &Value,
    artifact: &Value,
    runtime: &RuntimeAuthority,
    evidence: &str,
    policy: &Value,
    schema: &Value,
    now: i64,
) -> bool {
    let artifact_captured = artifact
        .get("captured_at")
        .and_then(Value::as_str)
        .and_then(parse_rfc3339_utc);
    let artifact_expires = artifact
        .get("expires_at")
        .and_then(Value::as_str)
        .and_then(parse_rfc3339_utc);
    let approved = receipt
        .get("approved_at")
        .and_then(Value::as_str)
        .and_then(parse_rfc3339_utc);
    let receipt_expires = receipt
        .get("expires_at")
        .and_then(Value::as_str)
        .and_then(parse_rfc3339_utc);
    receipt.get("reset_id") == artifact.get("reset_id")
        && receipt.get("evidence_sha256").and_then(Value::as_str) == Some(evidence)
        && receipt.get("protected_commit_sha").and_then(Value::as_str)
            == Some(runtime.protected_commit_sha.as_str())
        && receipt.get("candidate_commit_sha").and_then(Value::as_str)
            == Some(runtime.candidate_commit_sha.as_str())
        && receipt.get("schema_sha256").and_then(Value::as_str)
            == Some(json_digest(schema).as_str())
        && receipt.get("policy_sha256").and_then(Value::as_str)
            == Some(json_digest(policy).as_str())
        && matches!(
            (artifact_captured, artifact_expires, approved, receipt_expires),
            (Some(captured), Some(artifact_end), Some(approved_at), Some(receipt_end))
                if captured <= approved_at
                    && approved_at <= artifact_end
                    && approved_at <= now
                    && now < receipt_end
                    && receipt_end <= artifact_end
                    && approved_at.checked_add(86_400).is_some_and(|ceiling| receipt_end <= ceiling)
        )
}

pub fn receipt_payload(receipt: &Value) -> Option<Vec<u8>> {
    let fields = [
        "role",
        "approver",
        "reset_id",
        "evidence_sha256",
        "protected_commit_sha",
        "candidate_commit_sha",
        "schema_sha256",
        "policy_sha256",
        "approved_at",
        "expires_at",
        "key_id",
    ];
    let mut payload = String::new();
    for field in fields {
        if let Some(value) = receipt.get(field).and_then(Value::as_str) {
            payload.push_str(field);
            payload.push('=');
            payload.push_str(value);
            payload.push('\n');
        }
    }
    (!payload.is_empty()).then(|| payload.into_bytes())
}

fn verify_receipt(receipt: &Value, key: Option<&[u8; 32]>) -> bool {
    let (Some(key), Some(signature_hex), Some(payload)) = (
        key,
        receipt.get("signature").and_then(Value::as_str),
        receipt_payload(receipt),
    ) else {
        return false;
    };
    let Some(signature_bytes) = decode_hex_64(signature_hex) else {
        return false;
    };
    VerifyingKey::from_bytes(key)
        .ok()
        .is_some_and(|verifying_key| {
            verifying_key
                .verify_strict(&payload, &Signature::from_bytes(&signature_bytes))
                .is_ok()
        })
}

fn validate_node(
    root: &Value,
    schema: &Value,
    value: &Value,
    path: &str,
    findings: &mut BTreeSet<Finding>,
) {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        let Some(target) = reference.strip_prefix('#').and_then(|p| root.pointer(p)) else {
            findings.insert(Finding::new("reset_schema_invalid_ref", path, reference));
            return;
        };
        validate_node(root, target, value, path, findings);
    }
    if let Some(constant) = schema.get("const")
        && value != constant
    {
        schema_error(findings, path, "const");
    }
    if let Some(values) = schema.get("enum").and_then(Value::as_array)
        && !values.contains(value)
    {
        schema_error(findings, path, "enum");
    }
    if let Some(kind) = schema.get("type").and_then(Value::as_str) {
        let correct = match kind {
            "object" => value.is_object(),
            "array" => value.is_array(),
            "string" => value.is_string(),
            "boolean" => value.is_boolean(),
            "integer" => value.is_i64() || value.is_u64(),
            _ => false,
        };
        if !correct {
            schema_error(findings, path, "type");
            return;
        }
    }
    if let Some(object) = value.as_object() {
        let properties = schema.get("properties").and_then(Value::as_object);
        for required in schema
            .get("required")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if !object.contains_key(required) {
                schema_error(findings, &format!("{path}.{required}"), "required");
            }
        }
        for (key, child) in object {
            if let Some(child_schema) = properties.and_then(|schemas| schemas.get(key)) {
                validate_node(
                    root,
                    child_schema,
                    child,
                    &format!("{path}.{key}"),
                    findings,
                );
            } else if schema.get("additionalProperties").and_then(Value::as_bool) == Some(false) {
                schema_error(findings, &format!("{path}.{key}"), "additionalProperties");
            }
        }
    }
    if let Some(array) = value.as_array() {
        if schema
            .get("minItems")
            .and_then(Value::as_u64)
            .is_some_and(|min| array.len() < min as usize)
        {
            schema_error(findings, path, "minItems");
        }
        if schema.get("uniqueItems").and_then(Value::as_bool) == Some(true)
            && array
                .iter()
                .enumerate()
                .any(|(i, v)| array[..i].contains(v))
        {
            schema_error(findings, path, "uniqueItems");
        }
        if let Some(item_schema) = schema.get("items") {
            for (index, child) in array.iter().enumerate() {
                validate_node(
                    root,
                    item_schema,
                    child,
                    &format!("{path}[{index}]"),
                    findings,
                );
            }
        }
    }
    if let Some(text) = value.as_str() {
        if schema
            .get("minLength")
            .and_then(Value::as_u64)
            .is_some_and(|min| text.chars().count() < min as usize)
        {
            schema_error(findings, path, "minLength");
        }
        if let Some(pattern) = schema.get("pattern").and_then(Value::as_str) {
            let valid = match pattern {
                "^[0-9a-f]{40}$" => is_hex(text, 40),
                "^sha256:[0-9a-f]{64}$" => is_sha256(text),
                "^[0-9a-f]{128}$" => is_hex(text, 128),
                "^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$" => {
                    parse_rfc3339_utc(text).is_some()
                }
                _ => false,
            };
            if !valid {
                schema_error(findings, path, "pattern");
            }
        }
        if schema.get("format").and_then(Value::as_str) == Some("date-time")
            && parse_rfc3339_utc(text).is_none()
        {
            schema_error(findings, path, "format");
        }
    }
}

fn schema_error(findings: &mut BTreeSet<Finding>, path: &str, keyword: &str) {
    findings.insert(Finding::new(
        "reset_schema_validation",
        path,
        format!("violates {keyword}"),
    ));
}

fn reject_unsupported_keywords(
    value: &Value,
    path: &str,
    supported: &BTreeSet<&str>,
    schema_node: bool,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(object) = value.as_object() else {
        return;
    };
    for (key, child) in object {
        if schema_node && !supported.contains(key.as_str()) {
            findings.insert(Finding::new(
                "reset_schema_unsupported_keyword",
                format!("{path}.{key}"),
                "schema uses an unsupported keyword",
            ));
            continue;
        }
        match key.as_str() {
            "properties" | "$defs" => {
                if let Some(children) = child.as_object() {
                    for (name, child_schema) in children {
                        reject_unsupported_keywords(
                            child_schema,
                            &format!("{path}.{key}.{name}"),
                            supported,
                            true,
                            findings,
                        );
                    }
                }
            }
            "items" => reject_unsupported_keywords(
                child,
                &format!("{path}.{key}"),
                supported,
                true,
                findings,
            ),
            _ => {}
        }
    }
}

fn validate_schema_keyword_shapes(
    root: &Value,
    schema: &Value,
    path: &str,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(object) = schema.as_object() else {
        findings.insert(Finding::new(
            "reset_schema_keyword_shape",
            path,
            "every schema node must be an object",
        ));
        return;
    };
    for key in ["$schema", "$id", "title", "description"] {
        if object.contains_key(key) && object.get(key).and_then(Value::as_str).is_none() {
            schema_keyword_error(findings, path, key, "must be a string");
        }
    }
    if let Some(kind) = object.get("type")
        && !kind.as_str().is_some_and(|kind| {
            matches!(kind, "object" | "array" | "string" | "boolean" | "integer")
        })
    {
        schema_keyword_error(findings, path, "type", "must be one supported type string");
    }
    if let Some(required) = object.get("required") {
        let valid = required.as_array().is_some_and(|items| {
            let names = items
                .iter()
                .filter_map(Value::as_str)
                .collect::<BTreeSet<_>>();
            names.len() == items.len()
        });
        if !valid {
            schema_keyword_error(
                findings,
                path,
                "required",
                "must be an array of unique strings",
            );
        }
    }
    if let Some(reference) = object.get("$ref") {
        let valid = reference
            .as_str()
            .and_then(|value| value.strip_prefix('#'))
            .and_then(|pointer| root.pointer(pointer))
            .is_some();
        if !valid {
            schema_keyword_error(
                findings,
                path,
                "$ref",
                "must be a resolvable local JSON pointer string",
            );
        }
    }
    if let Some(value) = object.get("additionalProperties")
        && value.as_bool().is_none()
    {
        schema_keyword_error(findings, path, "additionalProperties", "must be boolean");
    }
    if let Some(value) = object.get("format")
        && value.as_str() != Some("date-time")
    {
        schema_keyword_error(findings, path, "format", "only date-time is supported");
    }
    if let Some(value) = object.get("enum") {
        let valid = value.as_array().is_some_and(|items| {
            !items.is_empty()
                && !items
                    .iter()
                    .enumerate()
                    .any(|(index, item)| items[..index].contains(item))
        });
        if !valid {
            schema_keyword_error(
                findings,
                path,
                "enum",
                "must be a non-empty array of unique JSON values",
            );
        }
    }
    if let Some(value) = object.get("items")
        && !value.is_object()
    {
        schema_keyword_error(findings, path, "items", "must be a schema object");
    }
    for key in ["minItems", "minLength"] {
        if object.contains_key(key) && object.get(key).and_then(Value::as_u64).is_none() {
            schema_keyword_error(findings, path, key, "must be a non-negative integer");
        }
    }
    if object.contains_key("uniqueItems")
        && object.get("uniqueItems").and_then(Value::as_bool).is_none()
    {
        schema_keyword_error(findings, path, "uniqueItems", "must be boolean");
    }
    if let Some(pattern) = object.get("pattern") {
        let supported = matches!(
            pattern.as_str(),
            Some(
                "^[0-9a-f]{40}$"
                    | "^sha256:[0-9a-f]{64}$"
                    | "^[0-9a-f]{128}$"
                    | "^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$"
            )
        );
        if !supported {
            schema_keyword_error(
                findings,
                path,
                "pattern",
                "must be one explicitly implemented pattern",
            );
        }
    }
    for container in ["properties", "$defs"] {
        if let Some(children) = object.get(container) {
            let Some(children) = children.as_object() else {
                schema_keyword_error(findings, path, container, "must be an object of schemas");
                continue;
            };
            for (name, child) in children {
                validate_schema_keyword_shapes(
                    root,
                    child,
                    &format!("{path}.{container}.{name}"),
                    findings,
                );
            }
        }
    }
    if let Some(items) = object.get("items")
        && items.is_object()
    {
        validate_schema_keyword_shapes(root, items, &format!("{path}.items"), findings);
    }
}

fn schema_keyword_error(findings: &mut BTreeSet<Finding>, path: &str, keyword: &str, detail: &str) {
    findings.insert(Finding::new(
        "reset_schema_keyword_shape",
        format!("{path}.{keyword}"),
        detail,
    ));
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
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn is_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|v| is_hex(v, 64))
}
fn decode_hex_64(value: &str) -> Option<[u8; 64]> {
    if value.len() != 128 {
        return None;
    }
    let mut out = [0; 64];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}

fn scan_forbidden_secrets(
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
                        "secret-bearing field names are forbidden",
                    ));
                }
                scan_forbidden_secrets(child, &child_path, forbidden, findings);
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                scan_forbidden_secrets(child, &format!("{path}[{index}]"), forbidden, findings);
            }
        }
        Value::String(text) if looks_like_secret_value(path, text) => {
            findings.insert(Finding::new(
                "reset_secret_value_detected",
                path,
                "high-confidence secret-like value is forbidden even under an innocuous field name",
            ));
        }
        _ => {}
    }
}

fn looks_like_secret_value(path: &str, value: &str) -> bool {
    if value.contains("-----BEGIN PRIVATE KEY-----")
        || value.starts_with("ghp_")
        || value.starts_with("github_pat_")
        || value.starts_with("sk_live_")
        || value.starts_with("xoxb-")
    {
        return true;
    }
    if looks_like_jwt(value) {
        return true;
    }
    if value.len() < 40
        || value.bytes().any(|byte| byte.is_ascii_whitespace())
        || is_approved_structured_value(path, value)
    {
        return false;
    }
    let mut classes = [false; 4];
    let mut counts = [0_u32; 128];
    for byte in value.bytes() {
        if !byte.is_ascii() {
            return false;
        }
        classes[0] |= byte.is_ascii_lowercase();
        classes[1] |= byte.is_ascii_uppercase();
        classes[2] |= byte.is_ascii_digit();
        classes[3] |= byte.is_ascii_punctuation();
        counts[usize::from(byte)] += 1;
    }
    if classes.into_iter().filter(|present| *present).count() < 3
        || counts.into_iter().filter(|count| *count > 0).count() < 16
    {
        return false;
    }
    let length = value.len() as f64;
    let entropy = counts
        .into_iter()
        .filter(|count| *count > 0)
        .map(|count| {
            let probability = f64::from(count) / length;
            -probability * probability.log2()
        })
        .sum::<f64>();
    entropy >= 4.3
}

fn is_approved_structured_value(path: &str, value: &str) -> bool {
    is_uri_field(path) && (is_hierarchical_uri(value) || is_approved_opaque_uri(value))
        || path.ends_with(".stable_id") && REQUIRED_INVENTORY_IDS.contains(&value)
        || is_sha256_field(path) && is_sha256(value)
        || is_commit_field(path) && is_hex(value, 40)
        || path.ends_with(".signature") && is_hex(value, 128)
}

fn is_hierarchical_uri(value: &str) -> bool {
    let Some((scheme, remainder)) = value.split_once("://") else {
        return false;
    };
    let authority = remainder
        .split_once(['/', '?', '#'])
        .map_or(remainder, |(authority, _)| authority);
    matches!(scheme, "http" | "https")
        && !remainder.is_empty()
        && !authority.is_empty()
        && !remainder.contains('@')
        && remainder.bytes().all(is_uri_tail_byte)
}

fn is_approved_opaque_uri(value: &str) -> bool {
    let Some((scheme, suffix)) = value.split_once(':') else {
        return false;
    };
    matches!(
        scheme,
        "protected-runtime" | "redacted-local" | "redacted-local-manifest"
    ) && !suffix.is_empty()
        && suffix.bytes().all(is_uri_tail_byte)
}

fn is_uri_field(path: &str) -> bool {
    path.ends_with(".uri") || path.ends_with(".evidence_uri")
}

fn is_uri_tail_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'-' | b'.'
                | b'_'
                | b'~'
                | b':'
                | b'/'
                | b'?'
                | b'#'
                | b'['
                | b']'
                | b'@'
                | b'!'
                | b'$'
                | b'&'
                | b'\''
                | b'('
                | b')'
                | b'*'
                | b'+'
                | b','
                | b';'
                | b'='
                | b'%'
        )
}

fn is_sha256_field(path: &str) -> bool {
    path.ends_with(".sha256")
        || path.ends_with(".evidence_sha256")
        || path.ends_with(".schema_sha256")
        || path.ends_with(".policy_sha256")
}

fn is_commit_field(path: &str) -> bool {
    path.ends_with(".protected_commit_sha") || path.ends_with(".candidate_commit_sha")
}

fn looks_like_jwt(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts[0].starts_with("eyJ")
        && parts[1].starts_with("eyJ")
        && parts
            .iter()
            .all(|part| part.len() >= 16 && part.bytes().all(is_base64url_byte))
}

fn is_base64url_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')
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
    if !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let days_in_month = [
        31,
        if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            29
        } else {
            28
        },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if day < 1 || day > days_in_month[(month - 1) as usize] {
        return None;
    }
    let adjusted_year = year - i64::from(month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let shifted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some((era * 146_097 + day_of_era - 719_468) * 86_400 + hour * 3_600 + minute * 60 + second)
}

#[cfg(test)]
mod tests {
    use super::{is_approved_structured_value, looks_like_secret_value};

    #[test]
    fn structured_secret_scanner_exclusions_are_exact() {
        for (path, value) in [
            (
                "$.evidence_manifest.uri",
                "https://inventory.oyatie.example/resources/cluster-primary",
            ),
            (
                "$.inventory[0].stable_id",
                "github:jason931225/oyatie:arc-runners",
            ),
            (
                "$.evidence_manifest.sha256",
                "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            ),
            (
                "$.approvals[0].candidate_commit_sha",
                "0123456789abcdef0123456789abcdef01234567",
            ),
            (
                "$.approvals[0].signature",
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            ),
        ] {
            assert!(
                is_approved_structured_value(path, value),
                "approved structured value was not excluded: {path}"
            );
        }

        let colon_secret = "opaque:q9P/2Zd7Wm4+Lx8!Vt3#Kn6@Hs1%Rc5^Bj0&Fy7*Ua2=Ee9?";
        assert!(!is_approved_structured_value(
            "$.inventory[0].lifecycle",
            colon_secret
        ));
        assert!(looks_like_secret_value(
            "$.inventory[0].lifecycle",
            colon_secret
        ));

        for value in [
            "opaque://q9P2Zd7Wm4+Lx8!Vt3#Kn6@Hs1%Rc5Bj0&Fy7*Ua2=Ee9?",
            "https://user:q9P2Zd7Wm4+Lx8!Vt3#Kn6@Hs1%Rc5Bj0&Fy7*Ua2=Ee9?@host",
        ] {
            assert!(!is_approved_structured_value(
                "$.evidence_manifest.uri",
                value
            ));
            assert!(looks_like_secret_value("$.evidence_manifest.uri", value));
        }
    }
}
