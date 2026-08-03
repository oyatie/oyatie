#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::{collections::BTreeMap, fs, path::PathBuf};

use ci_reset_eligibility_policy::{
    RuntimeAuthority, TrustedApprovalKey, artifact_subject_digest, evaluate, evaluate_schema,
    evidence_digest, json_digest, receipt_payload, validate_artifact_schema,
};
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{Value, json};

const NOW: i64 = 1_785_610_769;
const PROTECTED: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const CANDIDATE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SOURCE_IDS: [&str; 8] = [
    "github-and-protected-ci",
    "kubernetes-inventory",
    "stateful-workloads",
    "openbao-and-registry",
    "backup-jobs",
    "dns-and-edge",
    "provider-billing-and-global-inventory",
    "external-backups-and-saas-callbacks",
];
const INVENTORY_IDS: [&str; 7] = [
    "talos-current:cluster",
    "kubernetes-current:persistent-volumes",
    "kubernetes-current:postgres",
    "kubernetes-current:openbao",
    "kubernetes-current:oci-registry",
    "github:jason931225/oyatie:arc-runners",
    "cloudflare-configured:active-edge",
];
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
    let evidence = SOURCE_IDS
        .into_iter()
        .map(|source_id| {
            (
                source_id.to_owned(),
                format!("reviewed redacted evidence for {source_id}\n").into_bytes(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let digest = evidence_digest(&evidence);
    artifact["reset_id"] = json!("fixture-reset");
    artifact["repository"] = json!({"binding_source":"ci-runtime-authority"});
    artifact["captured_at"] = json!("2026-08-01T17:59:29Z");
    artifact["expires_at"] = json!("2026-08-02T17:59:29Z");
    artifact["evidence_manifest"] =
        json!({"uri":"protected-runtime:fixture","status":"verified","sha256":digest});
    artifact["sources"] = json!(
        SOURCE_IDS
            .into_iter()
            .map(|source_id| {
                let bytes = evidence.get(source_id).expect("fixture evidence");
                json!({
                    "source_id": source_id,
                    "result": "observed",
                    "redaction": "secret-values-excluded",
                    "evidence_uri": format!("protected-runtime:{source_id}"),
                    "sha256": ci_reset_eligibility_policy::sha256(bytes)
                })
            })
            .collect::<Vec<_>>()
    );
    artifact["recovery"] = json!({"backup_verified":true,"immutable_backup_location_verified":true,"rpo_verified":true,"restore_drill_verified":true,"key_recovery_verified":true});
    artifact["hard_stops"] = json!([]);
    artifact["unknowns"] = json!([]);

    let common = json!({
        "reset_id":"fixture-reset",
        "artifact_sha256":"sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "evidence_sha256":evidence_digest(&evidence),
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
        approvals.push(receipt);
    }
    artifact["approvals"] = json!(approvals);
    let artifact_sha256 = artifact_subject_digest(&artifact).expect("artifact subject digest");
    for index in 0..artifact["approvals"].as_array().expect("approvals").len() {
        artifact["approvals"][index]["artifact_sha256"] = json!(artifact_sha256);
        artifact["approvals"].as_array_mut().unwrap()[index] = sign(
            artifact["approvals"][index].clone(),
            &approval_signing_key(index as u8),
        );
    }
    (policy, schema, artifact, runtime)
}

fn resign_approvals(artifact: &mut Value) {
    for index in 0..artifact["approvals"].as_array().expect("approvals").len() {
        artifact["approvals"].as_array_mut().unwrap()[index] = sign(
            artifact["approvals"][index].clone(),
            &approval_signing_key(index as u8),
        );
    }
}

fn has_code(
    findings: &std::collections::BTreeSet<ci_reset_eligibility_policy::Finding>,
    code: &str,
) -> bool {
    findings.iter().any(|finding| finding.code == code)
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
fn historical_w0d_discovery_is_non_authoritative_and_fail_closed() {
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
        .get_mut(SOURCE_IDS[0])
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
fn approval_receipts_reject_decision_bearing_artifact_tampering() {
    let (policy, schema, artifact, runtime) = positive_fixture();
    let mutations: Vec<ArtifactMutation> = vec![
        (
            "scope",
            Box::new(|value| {
                value["scope"]["accounts"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!("github:unexpected/account"));
            }),
        ),
        (
            "evidence manifest",
            Box::new(|value| {
                value["evidence_manifest"]["uri"] =
                    json!("https://inventory.example/reviewed/alternate-manifest");
            }),
        ),
        (
            "source mapping",
            Box::new(|value| {
                value["sources"][0]["evidence_uri"] =
                    json!("protected-runtime:alternate-source-handle");
            }),
        ),
        (
            "inventory metadata",
            Box::new(|value| value["inventory"][0]["billing"] = json!("candidate-mutated")),
        ),
        (
            "inventory fingerprint",
            Box::new(|value| {
                value["inventory"][0]["unverified_identity_fingerprint"] = json!(
                    "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                );
            }),
        ),
        (
            "recovery",
            Box::new(|value| value["recovery"]["backup_verified"] = json!(false)),
        ),
        (
            "hard stops",
            Box::new(|value| {
                value["hard_stops"] = json!([{
                    "id":"candidate-stop",
                    "class":"candidate",
                    "acceptance_criteria":"review",
                    "verification_path":"protected review",
                    "suggested_owner":"platform-operations",
                    "dependency_notes":"candidate mutation"
                }]);
            }),
        ),
        (
            "unknowns",
            Box::new(|value| {
                value["unknowns"] = json!([{
                    "id":"candidate-unknown",
                    "owner":"platform-operations",
                    "closure_probe":"review candidate mutation"
                }]);
            }),
        ),
        (
            "captured at",
            Box::new(|value| value["captured_at"] = json!("2026-08-01T18:00:00Z")),
        ),
        (
            "expires at",
            Box::new(|value| value["expires_at"] = json!("2026-08-02T17:30:00Z")),
        ),
        (
            "decision",
            Box::new(|value| value["decision"]["eligible"] = json!(true)),
        ),
        (
            "reason codes",
            Box::new(|value| {
                value["decision"]["reason_codes"] =
                    json!(["authorization-disabled", "candidate-reason"]);
            }),
        ),
    ];

    for (name, mutate) in mutations {
        let mut changed = artifact.clone();
        mutate(&mut changed);
        let findings = evaluate(&policy, &schema, &changed, &runtime, NOW);
        assert!(
            has_code(&findings, "reset_approval_receipt_invalid"),
            "{name} mutation did not invalidate approval receipts: {findings:#?}"
        );
    }
}

#[test]
fn artifact_subject_digest_is_canonical_and_cryptographically_bound() {
    let (policy, schema, artifact, runtime) = positive_fixture();

    let expected_subject = artifact_subject_digest(&artifact).expect("artifact subject");
    let mut approval_only_change = artifact.clone();
    approval_only_change["approvals"][0]["signature"] = json!("candidate-circular-surface");
    assert_eq!(
        artifact_subject_digest(&approval_only_change).as_deref(),
        Some(expected_subject.as_str()),
        "the embedded approvals array is the sole excluded circular surface"
    );

    let mut incomplete_receipt = artifact["approvals"][0].clone();
    incomplete_receipt
        .as_object_mut()
        .unwrap()
        .remove("artifact_sha256");
    assert!(
        receipt_payload(&incomplete_receipt).is_none(),
        "receipt payload construction must fail closed when the artifact subject digest is absent"
    );

    let reordered: Value =
        serde_json::from_str(r#"{"z":{"b":2,"a":1},"approvals":[],"a":[{"d":4,"c":3}]}"#)
            .expect("reordered fixture");
    let canonical: Value =
        serde_json::from_str(r#"{"a":[{"c":3,"d":4}],"approvals":[],"z":{"a":1,"b":2}}"#)
            .expect("canonical fixture");
    assert_eq!(
        artifact_subject_digest(&reordered),
        artifact_subject_digest(&canonical),
        "subject digest must not depend on JSON object insertion order"
    );

    let mut restamped_without_signature = artifact.clone();
    restamped_without_signature["scope"]["accounts"]
        .as_array_mut()
        .unwrap()
        .push(json!("github:reviewed-secondary"));
    let changed_digest = artifact_subject_digest(&restamped_without_signature)
        .expect("changed artifact subject digest");
    for receipt in restamped_without_signature["approvals"]
        .as_array_mut()
        .unwrap()
    {
        receipt["artifact_sha256"] = json!(changed_digest);
    }
    assert!(has_code(
        &evaluate(
            &policy,
            &schema,
            &restamped_without_signature,
            &runtime,
            NOW
        ),
        "reset_approval_receipt_invalid"
    ));

    resign_approvals(&mut restamped_without_signature);
    assert!(
        evaluate(
            &policy,
            &schema,
            &restamped_without_signature,
            &runtime,
            NOW
        )
        .is_empty(),
        "a semantically valid subject change must require both digest restamping and fresh signatures"
    );
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

#[test]
fn stale_evidence_is_reported_even_when_the_manifest_is_unverified() {
    let (policy, schema, mut artifact) = live();
    artifact["captured_at"] = json!("2026-08-01T17:00:00Z");
    artifact["expires_at"] = json!("2026-08-01T18:00:00Z");
    artifact["decision"]["reason_codes"]
        .as_array_mut()
        .expect("reason codes")
        .push(json!("evidence-expired"));

    assert!(
        evaluate(
            &policy,
            &schema,
            &artifact,
            &RuntimeAuthority::fail_closed(PROTECTED, CANDIDATE),
            NOW,
        )
        .is_empty(),
        "expiry must be computed independently of manifest verification"
    );
}

#[test]
fn approval_receipts_are_bounded_by_artifact_window_and_twenty_four_hours() {
    let (policy, schema, artifact, runtime) = positive_fixture();

    let mut before_capture = artifact.clone();
    for receipt in before_capture["approvals"].as_array_mut().unwrap() {
        receipt["approved_at"] = json!("2026-08-01T17:00:00Z");
    }
    resign_approvals(&mut before_capture);
    assert!(has_code(
        &evaluate(&policy, &schema, &before_capture, &runtime, NOW),
        "reset_approval_receipt_invalid"
    ));

    let mut expiry_after_artifact = artifact.clone();
    for receipt in expiry_after_artifact["approvals"].as_array_mut().unwrap() {
        receipt["expires_at"] = json!("2026-08-02T18:30:00Z");
    }
    resign_approvals(&mut expiry_after_artifact);
    assert!(has_code(
        &evaluate(&policy, &schema, &expiry_after_artifact, &runtime, NOW),
        "reset_approval_receipt_invalid"
    ));

    let mut after_artifact = artifact.clone();
    for receipt in after_artifact["approvals"].as_array_mut().unwrap() {
        receipt["approved_at"] = json!("2026-08-02T18:10:00Z");
        receipt["expires_at"] = json!("2026-08-03T18:00:00Z");
    }
    resign_approvals(&mut after_artifact);
    assert!(has_code(
        &evaluate(&policy, &schema, &after_artifact, &runtime, 1_785_697_200,),
        "reset_approval_receipt_invalid"
    ));
}

#[test]
fn source_and_inventory_identifiers_are_exact_closed_sets() {
    let (policy, schema, artifact, runtime) = positive_fixture();
    assert_eq!(
        artifact["inventory"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|item| item["stable_id"].as_str())
            .collect::<Vec<_>>(),
        INVENTORY_IDS
    );

    let mut missing_source = artifact.clone();
    missing_source["sources"].as_array_mut().unwrap().pop();
    assert!(has_code(
        &evaluate(&policy, &schema, &missing_source, &runtime, NOW),
        "reset_source_set_mismatch"
    ));

    let mut unknown_source = artifact.clone();
    unknown_source["sources"][0]["source_id"] = json!("candidate-invented-source");
    assert!(has_code(
        &evaluate(&policy, &schema, &unknown_source, &runtime, NOW),
        "reset_source_set_mismatch"
    ));

    let mut missing_inventory = artifact.clone();
    missing_inventory["inventory"].as_array_mut().unwrap().pop();
    assert!(has_code(
        &evaluate(&policy, &schema, &missing_inventory, &runtime, NOW),
        "reset_inventory_set_mismatch"
    ));

    let mut unknown_inventory = artifact.clone();
    unknown_inventory["inventory"][0]["stable_id"] = json!("candidate:invented");
    assert!(has_code(
        &evaluate(&policy, &schema, &unknown_inventory, &runtime, NOW),
        "reset_inventory_set_mismatch"
    ));
}

#[test]
fn high_confidence_secret_values_are_rejected_without_flagging_hashes_or_ids() {
    let (policy, schema, artifact, runtime) = positive_fixture();
    assert!(
        evaluate(&policy, &schema, &artifact, &runtime, NOW).is_empty(),
        "normal SHA-256 values, commit IDs, signatures, and stable IDs must remain green"
    );

    for secret in [
        "-----BEGIN PRIVATE KEY-----MIIEvQIBADANBgkqhkiG9w0BAQEFAASC",
        "ghp_1234567890abcdefghijklmnopqrstuv",
        "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.c2lnbmF0dXJlMTIzNDU2Nzg5MA",
        "q9P/2Zd7Wm4+Lx8!Vt3#Kn6@Hs1%Rc5^Bj0&Fy7*Ua2=Ee9?",
    ] {
        let mut leaked = artifact.clone();
        leaked["inventory"][0]["lifecycle"] = json!(secret);
        assert!(has_code(
            &evaluate(&policy, &schema, &leaked, &runtime, NOW),
            "reset_secret_value_detected"
        ));
    }
}

#[test]
fn uri_bearing_secrets_are_rejected_while_public_urls_remain_green() {
    let (policy, schema, artifact, runtime) = positive_fixture();
    const LOWERCASE_TOKEN: &str = "q9p2zd7wm4lx8vt3kn6hs1rc5bj0fy7ua2ee9qw4pi6zd8lm";
    let padded_lowercase_token = format!("protected-runtime:{}{LOWERCASE_TOKEN}", "a".repeat(80));

    for secret in [
        "opaque:q9P/2Zd7Wm4+Lx8!Vt3#Kn6@Hs1%Rc5^Bj0&Fy7*Ua2=Ee9?",
        "opaque://q9P/2Zd7Wm4+Lx8!Vt3#Kn6@Hs1%Rc5^Bj0&Fy7*Ua2=Ee9?",
        "opaque://q9P2Zd7Wm4+Lx8!Vt3#Kn6@Hs1%Rc5Bj0&Fy7*Ua2=Ee9?",
    ] {
        let mut colon_secret = artifact.clone();
        colon_secret["inventory"][0]["lifecycle"] = json!(secret);
        assert!(has_code(
            &evaluate(&policy, &schema, &colon_secret, &runtime, NOW),
            "reset_secret_value_detected"
        ));
    }

    for secret in [
        "redacted-local:q9P/2Zd7Wm4+Lx8!Vt3#Kn6@Hs1%Rc5^Bj0&Fy7*Ua2=Ee9?",
        "protected-runtime:q9P/2Zd7Wm4+Lx8!Vt3#Kn6@Hs1%Rc5^Bj0&Fy7*Ua2=Ee9?",
        "redacted-local:q9P2Zd7Wm4-Lx8Vt3Kn6Hs1Rc5Bj0Fy7Ua2Ee9Qw4Pi6Zd8Lm",
        "protected-runtime:q9P2Zd7Wm4-Lx8Vt3Kn6Hs1Rc5Bj0Fy7Ua2Ee9Qw4Pi6Zd8Lm",
        "protected-runtime:q9p2zd7wm4lx8vt3kn6hs1rc5bj0fy7ua2ee9qw4pi6zd8lm",
        "redacted-local:q9p2zd7w-m4lx8vt3-kn6hs1rc-5bj0fy7u-a2ee9qw4-pi6zd8lm",
        "protected-runtime:evidence-q9p2zd7w-m4lx8vt3-kn6hs1rc-5bj0fy7u-a2ee9qw4-pi6zd8lm",
        "protected-runtime:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    ] {
        let mut opaque_secret = artifact.clone();
        opaque_secret["evidence_manifest"]["uri"] = json!(secret);
        assert!(has_code(
            &evaluate(&policy, &schema, &opaque_secret, &runtime, NOW),
            "reset_secret_value_detected"
        ));
    }

    let mut padded_secret = artifact.clone();
    padded_secret["evidence_manifest"]["uri"] = json!(padded_lowercase_token);
    assert!(has_code(
        &evaluate(&policy, &schema, &padded_secret, &runtime, NOW),
        "reset_secret_value_detected"
    ));

    for secret in [
        "note ghp_1234567890abcdefghijklmnopqrstuv",
        "-----BEGIN RSA PRIVATE KEY-----\nMIIE...\n-----END RSA PRIVATE KEY-----",
        "-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAAA\n-----END OPENSSH PRIVATE KEY-----",
        "-----BEGIN EC PRIVATE KEY-----\nMHcCAQEE\n-----END EC PRIVATE KEY-----",
    ] {
        let mut embedded_secret = artifact.clone();
        embedded_secret["inventory"][0]["lifecycle"] = json!(secret);
        assert!(has_code(
            &evaluate(&policy, &schema, &embedded_secret, &runtime, NOW),
            "reset_secret_value_detected"
        ));
    }

    for secret in [
        "HTTPS://user:pass@inventory.example",
        "hTtP://u:p@inventory.example",
        "https://user:q9P/2Zd7Wm4+Lx8!Vt3#Kn6@Hs1%Rc5^Bj0&Fy7*Ua2=Ee9?@inventory.example",
        "https://user:q9P2Zd7Wm4+Lx8!Vt3#Kn6@Hs1%Rc5Bj0&Fy7*Ua2=Ee9?@inventory.example",
        "https://inventory.example/export?X-Amz-Signature=q9P2Zd7Wm4Lx8Vt3Kn6Hs1Rc5Bj0Fy7Ua2Ee9Qw4Pi6Zd8Lm",
        "https://inventory.example/export?q9P2Zd7Wm4Lx8Vt3Kn6Hs1Rc5Bj0Fy7Ua2Ee9Qw4Pi6Zd8Lm",
        "https://inventory.example/export/q9P2Zd7Wm4Lx8Vt3Kn6Hs1Rc5Bj0Fy7Ua2Ee9Qw4Pi6Zd8Lm",
        "https://inventory.example/export#q9P2Zd7Wm4Lx8Vt3Kn6Hs1Rc5Bj0Fy7Ua2Ee9Qw4Pi6Zd8Lm",
    ] {
        let mut uri_secret = artifact.clone();
        uri_secret["evidence_manifest"]["uri"] = json!(secret);
        assert!(has_code(
            &evaluate(&policy, &schema, &uri_secret, &runtime, NOW),
            "reset_secret_value_detected"
        ));
    }

    let mut uri = artifact.clone();
    uri["evidence_manifest"]["uri"] =
        json!("https://inventory.oyatie.example/resources/cluster-primary");
    let uri_findings = evaluate(&policy, &schema, &uri, &runtime, NOW);
    assert!(
        !has_code(&uri_findings, "reset_secret_value_detected"),
        "a structurally valid URI must remain green"
    );

    let mut public_metadata_uri = artifact.clone();
    public_metadata_uri["evidence_manifest"]["uri"] = json!(
        "https://docs.example.com/guide/setup?language=en-US&version=2026.08&section=deployment"
    );
    let public_metadata_findings = evaluate(&policy, &schema, &public_metadata_uri, &runtime, NOW);
    assert!(
        !has_code(&public_metadata_findings, "reset_secret_value_detected"),
        "ordinary public URI metadata must not be entropy-scanned as one opaque value"
    );

    for public_uri in [
        "HTTPS://inventory.oyatie.example/resources/cluster-primary",
        "hTtP://docs.example/guide?language=en-US",
    ] {
        let mut case_insensitive_public_uri = artifact.clone();
        case_insensitive_public_uri["evidence_manifest"]["uri"] = json!(public_uri);
        assert!(
            !has_code(
                &evaluate(
                    &policy,
                    &schema,
                    &case_insensitive_public_uri,
                    &runtime,
                    NOW,
                ),
                "reset_secret_value_detected"
            ),
            "case-insensitive public HTTP(S) URI must remain green: {public_uri}"
        );
    }

    let mut descriptive_handle = artifact.clone();
    descriptive_handle["evidence_manifest"]["uri"] =
        json!("protected-runtime:abcdefghijklmnopqrstuvwxyz0123456789documentation");
    let descriptive_handle_findings =
        evaluate(&policy, &schema, &descriptive_handle, &runtime, NOW);
    assert!(
        !has_code(&descriptive_handle_findings, "reset_secret_value_detected"),
        "a bounded descriptive opaque handle must remain green"
    );

    let mut prefixed_descriptive_handle = artifact.clone();
    prefixed_descriptive_handle["evidence_manifest"]["uri"] =
        json!("protected-runtime:evidence-abcdefghijklmnopqrstuvwxyz0123456789documentation");
    let prefixed_descriptive_handle_findings = evaluate(
        &policy,
        &schema,
        &prefixed_descriptive_handle,
        &runtime,
        NOW,
    );
    assert!(
        !has_code(
            &prefixed_descriptive_handle_findings,
            "reset_secret_value_detected"
        ),
        "a descriptive handle after a plain-text prefix must remain green"
    );

    let mut bare_descriptive_query = artifact.clone();
    bare_descriptive_query["evidence_manifest"]["uri"] =
        json!("https://inventory.example/export?documentation");
    let bare_descriptive_query_findings =
        evaluate(&policy, &schema, &bare_descriptive_query, &runtime, NOW);
    assert!(
        !has_code(
            &bare_descriptive_query_findings,
            "reset_secret_value_detected"
        ),
        "a descriptive bare query item must remain green"
    );

    assert!(
        evaluate(&policy, &schema, &artifact, &runtime, NOW).is_empty(),
        "approved stable IDs, SHA-256 digests, commit SHAs, and signatures must remain green"
    );
}
