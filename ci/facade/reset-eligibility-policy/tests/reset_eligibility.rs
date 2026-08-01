#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::{collections::BTreeMap, fs, path::PathBuf};

use ci_reset_eligibility_policy::{
    RuntimeAuthority, evaluate, evidence_digest, json_digest, receipt_payload,
    validate_artifact_schema,
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

fn sign(mut receipt: Value, key: &SigningKey) -> Value {
    let signature = key.sign(&receipt_payload(&receipt).expect("receipt payload"));
    receipt["signature"] = json!(hex(&signature.to_bytes()));
    receipt
}

fn positive_fixture() -> (Value, Value, Value, RuntimeAuthority) {
    let (policy, schema, mut artifact) = live();
    let approval_key = SigningKey::from_bytes(&[7; 32]);
    let grant_key = SigningKey::from_bytes(&[9; 32]);
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
    artifact["approvals"] = json!(
        ["founder", "platform-operations", "data-protection"]
            .into_iter()
            .map(|role| {
                let mut receipt = common.clone();
                receipt["role"] = json!(role);
                receipt["approver"] = json!(format!("trusted-{role}"));
                sign(receipt, &approval_key)
            })
            .collect::<Vec<_>>()
    );
    artifact["decision"] = json!({"eligible":true,"mode":"authorized-reset","default_if_unknown":"ineligible","reason_codes":[]});

    let mut grant = common;
    grant["grant_id"] = json!("protected-grant-1");
    grant["key_id"] = json!("authorization-key");
    let grant = sign(grant, &grant_key);
    let mut runtime = RuntimeAuthority::fail_closed(PROTECTED, CANDIDATE);
    runtime.evidence_by_source = evidence;
    runtime.trusted_approval_keys.insert(
        "approval-key".to_owned(),
        approval_key.verifying_key().to_bytes(),
    );
    runtime.trusted_authorization_keys.insert(
        "authorization-key".to_owned(),
        grant_key.verifying_key().to_bytes(),
    );
    runtime.authorization_grant = Some(grant);
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
    assert_eq!(
        artifact["evidence_manifest"]["status"],
        "unverified-discovery"
    );
    assert_eq!(artifact["decision"]["mode"], "preservation-migration");
}

#[test]
fn protected_positive_path_requires_rehashed_evidence_signed_receipts_and_separate_grant() {
    let (policy, schema, artifact, runtime) = positive_fixture();
    assert!(evaluate(&policy, &schema, &artifact, &runtime, NOW).is_empty());

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
fn positive_path_rejects_evidence_commit_schema_policy_and_signature_tampering() {
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
