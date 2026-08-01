#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::{collections::BTreeMap, fs, path::PathBuf};

use ci_reset_eligibility_policy::{
    RuntimeAuthority, TrustedApprovalKey, evaluate, evaluate_schema, evidence_digest, json_digest,
    receipt_payload, validate_artifact_schema,
};
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{Value, json};

const NOW: i64 = 1_785_610_769;
const PROTECTED: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const CANDIDATE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
type ArtifactMutation = (&'static str, Box<dyn Fn(&mut Value)>);

fn env_path(name: &str) -> PathBuf {
    PathBuf::from(std::env::var(name).unwrap_or_else(|_| panic!("missing Buck location {name}")))
}

fn read_json_env(name: &str) -> Value {
    let path = env_path(name);
    serde_json::from_str(
        &fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display())),
    )
    .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn live() -> (Value, Value, Value) {
    (
        read_json_env("OYA_RESET_POLICY"),
        read_json_env("OYA_RESET_SCHEMA"),
        read_json_env("OYA_RESET_ARTIFACT"),
    )
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn approval_signing_key(index: u8) -> SigningKey {
    SigningKey::from_bytes(&[7 + index; 32])
}

fn sign(mut receipt: Value, key: &SigningKey) -> Value {
    let signature = key.sign(&receipt_payload(&receipt).expect("receipt payload"));
    receipt["signature"] = json!(hex(&signature.to_bytes()));
    receipt
}

fn positive_fixture() -> (Value, Value, Value, RuntimeAuthority) {
    let (policy, schema, mut artifact) = live();
    let evidence = BTreeMap::from([(
        "fixture-source".to_owned(),
        b"reviewed redacted evidence\n".to_vec(),
    )]);
    let digest = evidence_digest(&evidence);
    artifact["reset_id"] = json!("fixture-reset");
    artifact["repository"] = json!({"binding_source":"ci-runtime-authority"});
    artifact["captured_at"] = json!("2026-08-01T17:59:29Z");
    artifact["expires_at"] = json!("2026-08-02T17:59:29Z");
    artifact["evidence_manifest"] =
        json!({"uri":"protected-runtime:fixture","status":"verified","sha256":digest});
    artifact["sources"] = json!([{
        "source_id":"fixture-source", "result":"observed", "redaction":"secret-values-excluded",
        "evidence_uri":"protected-runtime:fixture-source", "sha256":ci_reset_eligibility_policy::sha256(b"reviewed redacted evidence\n")
    }]);
    artifact["recovery"] = json!({"backup_verified":true,"immutable_backup_location_verified":true,"rpo_verified":true,"restore_drill_verified":true,"key_recovery_verified":true});
    artifact["hard_stops"] = json!([]);
    artifact["unknowns"] = json!([]);

    let common = json!({
        "reset_id":"fixture-reset", "evidence_sha256":evidence_digest(&evidence),
        "protected_commit_sha":PROTECTED, "candidate_commit_sha":CANDIDATE,
        "schema_sha256":json_digest(&schema), "policy_sha256":json_digest(&policy),
        "approved_at":"2026-08-01T18:00:00Z", "expires_at":"2026-08-02T17:00:00Z",
        "key_id":"approval-key", "signature":"00"
    });
    artifact["decision"] = json!({"eligible":false,"mode":"preservation-migration","default_if_unknown":"ineligible","reason_codes":["authorization-disabled"]});

    let mut runtime = RuntimeAuthority::fail_closed(PROTECTED, CANDIDATE);
    runtime.evidence_by_source = evidence;
    let mut approvals = Vec::new();
    for (index, role) in ["founder", "platform-operations", "data-protection"]
        .into_iter()
        .enumerate()
    {
        let key = approval_signing_key(index as u8);
        let key_id = format!("approval-key-{role}");
        runtime.trusted_approval_keys.insert(
            key_id.clone(),
            TrustedApprovalKey {
                verifying_key: key.verifying_key().to_bytes(),
                principal: format!("trusted-{role}"),
                allowed_role: role.to_owned(),
            },
        );
        let mut receipt = common.clone();
        receipt["role"] = json!(role);
        receipt["approver"] = json!(format!("trusted-{role}"));
        receipt["key_id"] = json!(key_id);
        approvals.push(sign(receipt, &key));
    }
    artifact["approvals"] = json!(approvals);
    (policy, schema, artifact, runtime)
}

#[test]
fn buck_graph_declares_all_policy_schema_artifact_and_root_inputs() {
    for name in [
        "OYA_RESET_POLICY",
        "OYA_RESET_SCHEMA",
        "OYA_RESET_ARTIFACT",
        "OYA_ROOT_MARKER",
    ] {
        let path = env_path(name);
        assert!(
            path.is_file(),
            "declared Buck input missing: {name}={}",
            path.display()
        );
    }
    let root: Value = serde_json::from_str(
        &fs::read_to_string(env_path("OYA_ROOT_MARKER")).expect("root marker bytes"),
    )
    .expect("root marker JSON");
    assert!(
        root.pointer("/entry_points/reset_eligibility_schema")
            .is_some()
    );
}

#[test]
fn live_w0d_discovery_is_non_authoritative_and_fail_closed() {
    let (policy, schema, artifact) = live();
    let runtime = RuntimeAuthority::fail_closed(PROTECTED, CANDIDATE);
    let findings = evaluate(&policy, &schema, &artifact, &runtime, NOW);
    assert!(
        findings.is_empty(),
        "{}",
        findings
            .iter()
            .map(|f| format!("{} {}: {}", f.code, f.key, f.detail))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(policy["reset_authorization_enabled"], false);
    assert!(
        policy["authority_boundary"]
            .as_str()
            .unwrap()
            .contains("no reset actuation path")
    );
    assert_eq!(
        schema["properties"]["decision"]["properties"]["mode"]["const"],
        "preservation-migration"
    );
    assert_eq!(
        artifact["evidence_manifest"]["status"],
        "unverified-discovery"
    );
    assert_eq!(artifact["decision"]["mode"], "preservation-migration");
}

#[test]
fn complete_protected_evidence_and_approvals_remain_dormant_without_actuation_authority() {
    let (policy, schema, artifact, runtime) = positive_fixture();
    assert!(evaluate(&policy, &schema, &artifact, &runtime, NOW).is_empty());
    assert_eq!(artifact["decision"]["eligible"], false);
    assert_eq!(
        artifact["decision"]["reason_codes"],
        json!(["authorization-disabled"])
    );

    let mut candidate_policy = policy.clone();
    candidate_policy["reset_authorization_enabled"] = json!(true);
    let mut candidate_artifact = artifact.clone();
    candidate_artifact["approvals"] = json!([{"role":"founder","approver":"candidate","approved_at":"2026-08-01T18:00:00Z","expires_at":"2026-08-02T17:00:00Z","reset_id":"fixture-reset","evidence_sha256":evidence_digest(&runtime.evidence_by_source),"protected_commit_sha":PROTECTED,"candidate_commit_sha":CANDIDATE,"schema_sha256":json_digest(&schema),"policy_sha256":json_digest(&candidate_policy),"key_id":"candidate-key","signature":"00"}]);
    let candidate_runtime = RuntimeAuthority::fail_closed(PROTECTED, CANDIDATE);
    assert!(
        !evaluate(
            &candidate_policy,
            &schema,
            &candidate_artifact,
            &candidate_runtime,
            NOW
        )
        .is_empty(),
        "candidate policy and approvals must never self-authorize"
    );
}

#[test]
fn dormant_eligibility_path_rejects_evidence_commit_schema_policy_and_signature_tampering() {
    let (policy, schema, artifact, runtime) = positive_fixture();
    let mut tampered_runtime = runtime.clone();
    tampered_runtime
        .evidence_by_source
        .get_mut("fixture-source")
        .unwrap()
        .push(b'!');
    assert!(!evaluate(&policy, &schema, &artifact, &tampered_runtime, NOW).is_empty());
    let mut wrong_commit = runtime.clone();
    wrong_commit.candidate_commit_sha = "cccccccccccccccccccccccccccccccccccccccc".to_owned();
    assert!(!evaluate(&policy, &schema, &artifact, &wrong_commit, NOW).is_empty());
    let mut changed_schema = schema.clone();
    changed_schema["title"] = json!("candidate-mutated");
    assert!(!evaluate(&policy, &changed_schema, &artifact, &runtime, NOW).is_empty());
    let mut changed_policy = policy.clone();
    changed_policy["max_validity_seconds"] = json!(172800);
    assert!(!evaluate(&changed_policy, &schema, &artifact, &runtime, NOW).is_empty());
    let mut forged = artifact.clone();
    forged["approvals"][0]["approver"] = json!("candidate");
    assert!(!evaluate(&policy, &schema, &forged, &runtime, NOW).is_empty());
}

#[test]
fn approvals_reject_role_swaps_principal_reuse_key_reuse_and_untrusted_keys() {
    let (policy, schema, artifact, runtime) = positive_fixture();

    let mut role_swap = artifact.clone();
    role_swap["approvals"][0]["role"] = json!("platform-operations");
    role_swap["approvals"].as_array_mut().unwrap()[0] =
        sign(role_swap["approvals"][0].clone(), &approval_signing_key(0));
    assert!(!evaluate(&policy, &schema, &role_swap, &runtime, NOW).is_empty());

    let mut principal_reuse = runtime.clone();
    principal_reuse
        .trusted_approval_keys
        .get_mut("approval-key-platform-operations")
        .unwrap()
        .principal = "trusted-founder".to_owned();
    let mut principal_artifact = artifact.clone();
    principal_artifact["approvals"][1]["approver"] = json!("trusted-founder");
    principal_artifact["approvals"].as_array_mut().unwrap()[1] = sign(
        principal_artifact["approvals"][1].clone(),
        &approval_signing_key(1),
    );
    assert!(!evaluate(&policy, &schema, &principal_artifact, &principal_reuse, NOW).is_empty());

    let mut key_reuse = runtime.clone();
    let founder_key = key_reuse.trusted_approval_keys["approval-key-founder"].verifying_key;
    key_reuse
        .trusted_approval_keys
        .get_mut("approval-key-platform-operations")
        .unwrap()
        .verifying_key = founder_key;
    let mut key_reuse_artifact = artifact.clone();
    key_reuse_artifact["approvals"].as_array_mut().unwrap()[1] = sign(
        key_reuse_artifact["approvals"][1].clone(),
        &approval_signing_key(0),
    );
    assert!(!evaluate(&policy, &schema, &key_reuse_artifact, &key_reuse, NOW).is_empty());

    let mut untrusted = runtime.clone();
    untrusted
        .trusted_approval_keys
        .remove("approval-key-founder");
    assert!(!evaluate(&policy, &schema, &artifact, &untrusted, NOW).is_empty());
}

#[test]
fn malformed_supported_schema_keyword_shapes_are_red() {
    let (policy, schema, _, _) = positive_fixture();
    let mutations: Vec<ArtifactMutation> = vec![
        ("required", Box::new(|v| v["required"] = json!("reset_id"))),
        ("type", Box::new(|v| v["type"] = json!(["object", "null"]))),
        (
            "ref",
            Box::new(|v| v["properties"]["scope"] = json!({"$ref": 7})),
        ),
        (
            "additionalProperties",
            Box::new(|v| v["additionalProperties"] = json!("false")),
        ),
        (
            "format",
            Box::new(|v| v["$defs"]["utcTimestamp"]["format"] = json!("email")),
        ),
        (
            "enum",
            Box::new(|v| {
                v["properties"]["evidence_manifest"]["properties"]["status"]["enum"] =
                    json!("verified")
            }),
        ),
        (
            "items",
            Box::new(|v| v["properties"]["sources"]["items"] = json!([])),
        ),
    ];
    for (name, mutate) in mutations {
        let mut changed = schema.clone();
        mutate(&mut changed);
        assert!(
            !evaluate_schema(&policy, &changed).is_empty(),
            "malformed {name} keyword shape was accepted"
        );
    }
}

#[test]
fn schema_semantics_reject_nested_required_type_format_ref_and_additional_properties_mutations() {
    let (_, schema, artifact, _) = positive_fixture();
    let mutations: Vec<ArtifactMutation> = vec![
        (
            "nested required",
            Box::new(|v| {
                v["recovery"]
                    .as_object_mut()
                    .unwrap()
                    .remove("backup_verified");
            }),
        ),
        (
            "type",
            Box::new(|v| v["recovery"]["backup_verified"] = json!("true")),
        ),
        (
            "format",
            Box::new(|v| v["captured_at"] = json!("not-a-date")),
        ),
        (
            "ref",
            Box::new(|v| v["evidence_manifest"]["sha256"] = json!("sha256:nope")),
        ),
        (
            "additionalProperties",
            Box::new(|v| v["collector"]["candidate_escape"] = json!(true)),
        ),
    ];
    for (name, mutate) in mutations {
        let mut changed = artifact.clone();
        mutate(&mut changed);
        assert!(
            !validate_artifact_schema(&schema, &changed).is_empty(),
            "{name} mutation was accepted"
        );
    }
}

#[test]
fn schema_semantics_apply_ref_siblings_and_closed_objects_without_properties() {
    let referenced_with_sibling = json!({
        "$defs": {"text": {"type": "string"}},
        "$ref": "#/$defs/text",
        "minLength": 5
    });
    assert!(
        !validate_artifact_schema(&referenced_with_sibling, &json!("four")).is_empty(),
        "a valid $ref sibling constraint must not be skipped"
    );
    assert!(
        validate_artifact_schema(&referenced_with_sibling, &json!("valid")).is_empty(),
        "$ref and its sibling constraint should compose for a valid value"
    );

    let closed_object = json!({"type": "object", "additionalProperties": false});
    assert!(
        !validate_artifact_schema(&closed_object, &json!({"candidate_escape": true})).is_empty(),
        "additionalProperties=false must close an object even without properties"
    );
    assert!(
        validate_artifact_schema(&closed_object, &json!({})).is_empty(),
        "an empty closed object should remain valid"
    );
}
