use serde_json::{Value, json};
use stage1_closure::{
    evaluate_epoch, evaluate_epoch_with_protected_facts, evaluate_postmerge_admission_envelope,
    evaluate_program,
};

const SUBJECT_DIGEST: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn program() -> Value {
    serde_json::from_str(include_str!("fixtures/program.json")).expect("program fixture parses")
}

fn hold_epoch() -> Value {
    serde_json::from_str(include_str!("fixtures/hold-epoch.json")).expect("epoch fixture parses")
}

fn schema(relative: &str) -> Value {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| "libs/stage1-closure".to_owned());
    let path = match relative {
        "program" => std::env::var("OYA_STAGE1_PROGRAM_SCHEMA").unwrap_or_else(|_| {
            format!("{manifest_dir}/../../specs/stage1-closure-program.schema.json")
        }),
        "epoch" => std::env::var("OYA_STAGE1_EPOCH_SCHEMA").unwrap_or_else(|_| {
            format!("{manifest_dir}/../../specs/stage1-evidence-epoch.schema.json")
        }),
        _ => panic!("unknown schema fixture {relative}"),
    };
    let source = std::fs::read_to_string(path).expect("schema is a declared readable input");
    serde_json::from_str(&source).expect("schema parses")
}

fn receipt(issuer_id: &str, issuer_class: &str) -> Value {
    json!({
        "path": format!("evidence/stage1/{issuer_id}.json"),
        "blob_oid": "1111111111111111111111111111111111111111",
        "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "subject_digest": SUBJECT_DIGEST,
        "issuer_id": issuer_id,
        "issuer_class": issuer_class,
        "authority_ref": format!("authority://{issuer_id}")
    })
}

fn pass_epoch() -> Value {
    let mut epoch = hold_epoch();
    epoch["state"] = json!("PASS_CANDIDATE");
    let controls = epoch["controls"].as_array_mut().expect("controls array");
    let classes = [
        "machine-verifiable",
        "machine-verifiable",
        "machine-verifiable",
        "machine-verifiable",
        "machine-verifiable",
        "qualified-human",
        "qualified-affected-party",
        "qualified-operations",
        "qualified-custody",
        "authorized-veto",
        "machine-and-qualified-human",
        "machine-verifiable",
        "independent-council",
        "independent-dissent",
        "independent-oracle",
    ];
    for (index, control) in controls.iter_mut().enumerate() {
        let id = control["control_id"]
            .as_str()
            .expect("control id")
            .to_owned();
        control["status"] = json!("satisfied");
        control["subject_digest"] = json!(SUBJECT_DIGEST);
        control["receipt_refs"] = if id == "C11" {
            json!([
                receipt("issuer-C11-machine", "machine-verifiable"),
                receipt("issuer-C11-human", "qualified-human")
            ])
        } else {
            json!([receipt(&format!("issuer-{id}"), classes[index])])
        };
    }
    let lenses = epoch["lenses"].as_array_mut().expect("lenses array");
    for lens in lenses {
        let id = lens["lens_id"].as_str().expect("lens id").to_owned();
        lens["status"] = json!("satisfied");
        lens["subject_digest"] = json!(SUBJECT_DIGEST);
        lens["reviewer_id"] = json!(format!("reviewer-{id}"));
        lens["independent_from_author"] = json!(true);
        lens["fresh_context"] = json!(true);
        lens["receipt_ref"] = receipt(&format!("reviewer-{id}"), "independent-council");
    }
    epoch["fresh_dissent"] = json!({
        "status": "satisfied",
        "subject_digest": SUBJECT_DIGEST,
        "reviewer_id": "fresh-dissent-reviewer",
        "fresh_context": true,
        "prior_context_used": false,
        "findings_resolved_or_carried": true,
        "receipt_ref": receipt("fresh-dissent-reviewer", "independent-dissent")
    });
    epoch["context_free_exit"] = json!({
        "status": "satisfied",
        "subject_digest": SUBJECT_DIGEST,
        "oracle_principal_id": "exit-oracle",
        "blind_reader_principal_id": "blind-reader",
        "conversation_context_used": false,
        "reproduced_verdict": "PASS_CANDIDATE",
        "oracle_receipt_ref": receipt("exit-oracle", "independent-oracle"),
        "blind_reader_receipt_ref": receipt("blind-reader", "independent-reader")
    });
    epoch["immutable_successor"] = json!({
        "frozen": true,
        "subject_digest": SUBJECT_DIGEST,
        "facts_ref": {
            "path": "ci/facade/artifact-inventory-registry/scm-facts.generated.json",
            "sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
        }
    });
    epoch
}

fn artifact_binding(path: &str) -> Value {
    json!({
        "path": path,
        "blob_oid": "2222222222222222222222222222222222222222",
        "sha256": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
    })
}

fn protected_facts(epoch: &Value) -> Value {
    let mut receipt_bindings = Vec::new();
    let mut add = |receipt: &Value, control_id: &str| {
        let mut binding = receipt.clone();
        let object = binding.as_object_mut().expect("receipt is an object");
        object.insert("control_id".to_owned(), json!(control_id));
        object.insert("independently_observed".to_owned(), json!(true));
        object.insert(
            "trust_root_authority_ref".to_owned(),
            json!("authority://protected-parent"),
        );
        if matches!(control_id, "C06" | "C07" | "C08" | "C09" | "C10" | "C11") {
            object.insert("qualification".to_owned(), json!("qualified-for-stage1"));
            object.insert("scope".to_owned(), json!("stage1-closure"));
        }
        receipt_bindings.push(binding);
    };
    for control in epoch["controls"].as_array().expect("controls") {
        let control_id = control["control_id"].as_str().expect("control id");
        for receipt in control["receipt_refs"].as_array().expect("receipts") {
            add(receipt, control_id);
        }
    }
    for lens in epoch["lenses"].as_array().expect("lenses") {
        if lens["receipt_ref"].is_object() {
            add(&lens["receipt_ref"], "C13");
        }
    }
    if epoch["fresh_dissent"]["receipt_ref"].is_object() {
        add(&epoch["fresh_dissent"]["receipt_ref"], "C14");
    }
    if epoch["context_free_exit"]["oracle_receipt_ref"].is_object() {
        add(&epoch["context_free_exit"]["oracle_receipt_ref"], "C15");
    }
    if epoch["context_free_exit"]["blind_reader_receipt_ref"].is_object() {
        add(
            &epoch["context_free_exit"]["blind_reader_receipt_ref"],
            "C15",
        );
    }
    json!({
        "schema_id": "oyatie/stage1-protected-facts/v1",
        "protected_parent_verified": true,
        "trust_root_authority_ref": "authority://protected-parent",
        "protected_base_commit": "3333333333333333333333333333333333333333",
        "candidate_commit": "4444444444444444444444444444444444444444",
        "protected_base_tree": "5555555555555555555555555555555555555555",
        "candidate_tree": "6666666666666666666666666666666666666666",
        "subject_digest": SUBJECT_DIGEST,
        "program_binding": artifact_binding("specs/masterplan.json"),
        "schema_binding": artifact_binding("specs/stage1-evidence-epoch.schema.json"),
        "policy_binding": artifact_binding("specs/stage1-closure-program.schema.json"),
        "protected_base_repository": "github.com/jason931225/oyatie",
        "candidate_repository": "github.com/jason931225/oyatie",
        "parser_binding": artifact_binding("libs/oya-stage1-closure/src/lib.rs"),
        "producer_binding": artifact_binding("libs/oya-ci-materializer-kernel/src/lib.rs"),
        "evaluator_binding": artifact_binding("libs/oya-stage1-closure/src/lib.rs"),
        "predecessor_epoch_binding": artifact_binding("specs/stage1-evidence-epoch.schema.json"),
        "transition_receipt_binding": artifact_binding("specs/stage1-closure-program.schema.json"),
        "immutable_successor_bundle": artifact_binding("specs/masterplan.json"),
        "authority_chain_result": artifact_binding("specs/stage1-admission-envelope.schema.json"),
        "receipt_bindings": receipt_bindings
    })
}

#[test]
fn canonical_program_and_open_hold_epoch_are_green() {
    let program = program();
    assert!(evaluate_program(&program).is_green());
    assert!(evaluate_epoch(&program, &hold_epoch()).is_green());
}

#[test]
fn legal_jcr_can_run_after_foundation_and_must_precede_comparator_satisfaction() {
    let program = program();
    let legal_lane = program["groups"]
        .as_array()
        .expect("groups array")
        .iter()
        .find(|group| group["group_id"] == "D")
        .expect("D group");
    assert_eq!(
        legal_lane["requires_controls"],
        json!(["C01", "C02", "C03"]),
        "qualified legal/JCR work must not depend on already-completed comparator collection"
    );

    let mut candidate = hold_epoch();
    candidate["controls"][4]["status"] = json!("satisfied");
    candidate["controls"][4]["subject_digest"] = json!(SUBJECT_DIGEST);
    candidate["controls"][4]["receipt_refs"] = json!([receipt("issuer-C05", "machine-verifiable")]);
    let report = evaluate_epoch(&program, &candidate);
    assert!(
        report.findings.iter().any(|finding| finding.contains(
            "C05 comparator evidence cannot be satisfied before C06 legal/JCR disposition"
        )),
        "comparator satisfaction must fail closed before legal/JCR disposition: {:?}",
        report.findings
    );
}

#[test]
fn shipped_schemas_lock_hold_and_pass_ceiling() {
    let program_schema = schema("program");
    assert_eq!(
        program_schema["properties"]["candidate_effects"]["properties"]["binding_plan_approval_allowed"]
            ["const"],
        false
    );
    assert_eq!(
        program_schema["properties"]["candidate_effects"]["properties"]["implementation_dispatch_allowed"]
            ["const"],
        false
    );
    let epoch_schema = schema("epoch");
    assert_eq!(
        epoch_schema["$defs"]["planning"]["properties"]["binding_plan_approval_allowed"]["const"],
        false
    );
    assert_eq!(
        epoch_schema["$defs"]["planning"]["properties"]["implementation_dispatch_allowed"]["const"],
        false
    );
    assert_eq!(
        epoch_schema["$defs"]["blocker"]["properties"]["control_id"]["enum"],
        json!(["C06", "C07", "C08", "C09", "C10", "C11"])
    );
    assert!(
        epoch_schema["$defs"]["blocker"]["required"]
            .as_array()
            .expect("required blocker fields")
            .contains(&json!("scope"))
    );
    assert!(
        epoch_schema["$defs"]["receipt_ref"]["$comment"]
            .as_str()
            .expect("receipt routing comment")
            .contains("protected-parent evaluator")
    );
}

#[test]
fn missing_control_and_duplicate_lens_are_red() {
    let mut candidate = program();
    candidate["controls"]
        .as_array_mut()
        .expect("controls")
        .pop();
    candidate["lenses"][15]["lens_id"] = json!("L15");
    let report = evaluate_program(&candidate);
    assert!(report.findings.iter().any(|item| item.contains("controls")));
    assert!(report.findings.iter().any(|item| item.contains("lenses")));
}

#[test]
fn program_can_never_authorize_binding_approval_or_dispatch() {
    let mut candidate = program();
    candidate["candidate_effects"]["binding_plan_approval_allowed"] = json!(true);
    candidate["candidate_effects"]["implementation_dispatch_allowed"] = json!(true);
    let report = evaluate_program(&candidate);
    assert!(
        report
            .findings
            .iter()
            .any(|item| item.contains("binding_plan_approval_allowed"))
    );
    assert!(
        report
            .findings
            .iter()
            .any(|item| item.contains("implementation_dispatch_allowed"))
    );
}

#[test]
fn pass_requires_every_control_lens_dissent_successor_and_cold_exit() {
    let mut candidate = hold_epoch();
    candidate["state"] = json!("PASS_CANDIDATE");
    let report = evaluate_epoch(&program(), &candidate);
    for required in [
        "C01",
        "L01",
        "fresh_dissent",
        "immutable_successor",
        "context_free_exit",
    ] {
        assert!(
            report.findings.iter().any(|item| item.contains(required)),
            "missing finding for {required}: {:?}",
            report.findings
        );
    }
}

#[test]
fn qualified_controls_cannot_be_satisfied_without_object_bound_authority_receipts() {
    let mut candidate = hold_epoch();
    candidate["controls"][5]["status"] = json!("satisfied");
    candidate["controls"][5]["subject_digest"] = json!(SUBJECT_DIGEST);
    let report = evaluate_epoch(&program(), &candidate);
    assert!(
        report
            .findings
            .iter()
            .any(|item| item.contains("C06.receipt_refs"))
    );
}

#[test]
fn source_authored_receipts_can_never_yield_pass() {
    let candidate = pass_epoch();
    let report = evaluate_epoch(&program(), &candidate);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("independently supplied receipt_bindings")),
        "source-only PASS must fail closed: {:?}",
        report.findings
    );
}

#[test]
fn independently_bound_same_subject_pass_only_authorizes_roadmap_planning() {
    let candidate = pass_epoch();
    let report =
        evaluate_epoch_with_protected_facts(&program(), &candidate, &protected_facts(&candidate));
    assert!(
        report.is_green(),
        "unexpected findings: {:?}",
        report.findings
    );
    assert_eq!(
        candidate["planning"]["binding_plan_approval_allowed"],
        false
    );
    assert_eq!(
        candidate["planning"]["implementation_dispatch_allowed"],
        false
    );
}

#[test]
fn pass_rejects_tampered_protected_receipt_and_half_of_c11() {
    let candidate = pass_epoch();
    let mut facts = protected_facts(&candidate);
    facts["receipt_bindings"]
        .as_array_mut()
        .expect("bindings")
        .retain(|binding| {
            binding["issuer_id"] != "issuer-C11-human" && binding["issuer_id"] != "reviewer-L01"
        });
    let report = evaluate_epoch_with_protected_facts(&program(), &candidate, &facts);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("C11 requires"))
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("exact protected-parent binding"))
    );
}

#[test]
fn qualified_human_blocker_requires_a_blocked_c06_through_c11_control_and_scope() {
    let mut candidate = hold_epoch();
    candidate["state"] = json!("BLOCKED_QUALIFIED_HUMAN_INPUT");
    candidate["controls"][5]["status"] = json!("blocked");
    candidate["blockers"] = json!([{
        "control_id": "C06",
        "input_class": "legal-review",
        "required_qualification": "licensed-jcr-reviewer",
        "scope": "stage1-closure",
        "authority_ref": "authority://legal",
        "reason": "review is pending"
    }]);
    assert!(
        evaluate_epoch_with_protected_facts(&program(), &candidate, &protected_facts(&candidate))
            .is_green()
    );
    candidate["blockers"][0]["control_id"] = json!("C01");
    let report = evaluate_epoch(&program(), &candidate);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("allowed qualified-human"))
    );
    candidate["blockers"][0]["control_id"] = json!("C06");
    candidate["blockers"][0]["scope"] = json!("");
    let report = evaluate_epoch(&program(), &candidate);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("scope"))
    );
}

#[test]
fn pass_rejects_same_principal_council_or_context_using_exit() {
    let mut candidate = pass_epoch();
    candidate["lenses"][1]["reviewer_id"] = candidate["lenses"][0]["reviewer_id"].clone();
    candidate["context_free_exit"]["conversation_context_used"] = json!(true);
    let report = evaluate_epoch(&program(), &candidate);
    assert!(
        report
            .findings
            .iter()
            .any(|item| item.contains("reviewer_id"))
    );
    assert!(
        report
            .findings
            .iter()
            .any(|item| item.contains("conversation_context_used"))
    );
}

#[test]
fn red_candidate_self_authorization_and_intermediate_advance_without_protected_facts_are_rejected()
{
    let mut intermediate = hold_epoch();
    intermediate["state"] = json!("HOLD_EVIDENCE_COMPLETE");
    let missing = evaluate_epoch(&program(), &intermediate);
    assert!(
        missing
            .findings
            .iter()
            .any(|finding| finding.contains("protected"))
    );

    let candidate = pass_epoch();
    let mut facts = protected_facts(&candidate);
    facts["protected_parent_verified"] = json!(false);
    let report = evaluate_epoch_with_protected_facts(&program(), &candidate, &facts);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("protected_parent_verified"))
    );
}

#[test]
fn red_scm_facts_contamination_and_same_pr_authority_grammar_mutation_are_rejected() {
    let mut epoch = hold_epoch();
    epoch["subject_binding"]["facts_schema"] = json!("oya-ci/scm-facts/v2");
    assert!(
        evaluate_epoch(&program(), &epoch)
            .findings
            .iter()
            .any(|finding| finding.contains("facts_schema"))
    );

    let mut mutated_program = program();
    mutated_program["controls"][5]["authority_class"] =
        json!("x-stage1-non-authoritative-self-declared");
    assert!(
        evaluate_program(&mutated_program)
            .findings
            .iter()
            .any(|finding| finding.contains("authority_class"))
    );
}

#[test]
fn red_incomplete_successor_and_pr_head_envelope_are_rejected() {
    let candidate = pass_epoch();
    let mut facts = protected_facts(&candidate);
    facts["immutable_successor_bundle"] = json!(null);
    assert!(
        evaluate_epoch_with_protected_facts(&program(), &candidate, &facts)
            .findings
            .iter()
            .any(|finding| finding.contains("immutable_successor_bundle"))
    );

    let envelope = json!({
        "schema_id": "oyatie/stage1-admission-envelope/v1",
        "postmerge_promoted_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "pr_head_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "required_check_identity": "oya-ci-required",
        "app_identity": "cloud-ci",
        "protected_facts_binding": {}, "authority_chain_result": {}, "immutable_successor_bundle": {},
        "roadmap_planning_authorized": true,
        "binding_plan_approval_allowed": false,
        "implementation_dispatch_allowed": false
    });
    assert!(
        evaluate_postmerge_admission_envelope(&envelope)
            .findings
            .iter()
            .any(|finding| finding.contains("must not equal pr_head_sha"))
    );
}

#[test]
fn red_no_ci_facade_registration_or_generated_face_hand_edit_surface_exists() {
    assert!(!std::path::Path::new("ci/facade/stage1-closure").exists());
    assert!(!std::path::Path::new("libs/oya-stage1-closure/scm-facts.generated.json").exists());
}
