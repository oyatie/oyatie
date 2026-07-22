use std::collections::BTreeSet;

use serde_json::{Value, json};
use stage1_closure::{
    ADMISSION_ENVELOPE_ALLOWED_FIELDS, ADMISSION_ENVELOPE_REQUIRED_FIELDS,
    PROTECTED_RECEIPT_BINDING_ALLOWED_FIELDS, SOURCE_RECEIPT_REQUIRED_FIELDS,
    STAGE1_NON_AUTHORITATIVE_EXTENSION_PREFIX, evaluate_epoch, evaluate_program,
    evaluate_source_epoch, validate_admission_envelope_shape, validate_protected_facts_grammar,
    validate_protected_facts_shape,
};

const SUBJECT_DIGEST: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn program() -> Value {
    serde_json::from_str(include_str!("fixtures/program.json")).expect("program fixture parses")
}

fn hold_epoch() -> Value {
    serde_json::from_str(include_str!("fixtures/hold-epoch.json")).expect("epoch fixture parses")
}

fn admission_envelope() -> Value {
    serde_json::from_str(include_str!("fixtures/admission-envelope.json"))
        .expect("admission envelope fixture parses")
}

fn schema(relative: &str) -> Value {
    let environment = match relative {
        "program" => "OYA_STAGE1_PROGRAM_SCHEMA",
        "epoch" => "OYA_STAGE1_EPOCH_SCHEMA",
        "protected-facts" => "OYA_STAGE1_PROTECTED_FACTS_SCHEMA",
        "admission-envelope" => "OYA_STAGE1_ADMISSION_ENVELOPE_SCHEMA",
        _ => panic!("unknown schema fixture {relative}"),
    };
    let path = declared_schema_path(environment, std::env::var(environment));
    let source = std::fs::read_to_string(path).expect("schema is a declared readable input");
    serde_json::from_str(&source).expect("schema parses")
}

fn declared_schema_path(
    environment: &str,
    declared_path: Result<String, std::env::VarError>,
) -> String {
    declared_path.unwrap_or_else(|_| panic!("{environment} must be explicitly declared"))
}

fn receipt(principal_id: &str, issuer_authority_class: &str) -> Value {
    json!({
        "path": format!("evidence/stage1/{principal_id}.json"),
        "blob_oid": "1111111111111111111111111111111111111111",
        "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "subject_digest": SUBJECT_DIGEST,
        "principal_id": principal_id,
        "issuer_authority_class": issuer_authority_class,
        "authority_source_ref": artifact_binding(&format!("authority/{principal_id}")),
        "qualification": "stage1", "jurisdiction_scope": "stage1", "independence_observation": "observed", "validity": "valid", "revocation_status": "not-revoked", "conflict_status": "none", "signature_trust_root_binding": "root"
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
        "blind_reader_receipt_ref": receipt("blind-reader", "independent-oracle")
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
#[should_panic(expected = "OYA_STAGE1_TEST_MISSING_SCHEMA must be explicitly declared")]
fn schema_inputs_fail_closed_without_explicit_declarations() {
    let _ = declared_schema_path(
        "OYA_STAGE1_TEST_MISSING_SCHEMA",
        Err(std::env::VarError::NotPresent),
    );
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
fn source_authored_receipts_are_held_for_external_authentication() {
    let candidate = pass_epoch();
    let report = evaluate_epoch(&program(), &candidate);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("external authenticated Stage-1 controller")),
        "source-only PASS must remain held: {:?}",
        report.findings
    );
}

#[test]
fn source_epoch_fixture_matches_schema_and_parser_receipt_wire() {
    let epoch_schema = schema("epoch");
    let complete = pass_epoch();
    for field in epoch_schema["required"]
        .as_array()
        .expect("source epoch top-level required fields")
    {
        assert!(
            complete
                .get(field.as_str().expect("required field is a string"))
                .is_some(),
            "complete source epoch fixture omits required top-level field {field}"
        );
    }
    let control_required = epoch_schema["$defs"]["control_evidence"]["required"]
        .as_array()
        .expect("source control required fields");
    for control in complete["controls"].as_array().expect("complete controls") {
        for field in control_required {
            assert!(
                control
                    .get(field.as_str().expect("required field is a string"))
                    .is_some(),
                "complete source control fixture omits required field {field}"
            );
        }
    }
    let schema_required = epoch_schema["$defs"]["receipt_ref"]["required"]
        .as_array()
        .expect("source receipt required fields")
        .iter()
        .map(|field| field.as_str().expect("required field is a string"))
        .collect::<BTreeSet<_>>();
    let parser_required = SOURCE_RECEIPT_REQUIRED_FIELDS
        .into_iter()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        schema_required, parser_required,
        "source receipt wire drift"
    );

    let report = evaluate_source_epoch(&program(), &complete);
    assert_eq!(
        report.findings,
        vec![
            "external authenticated Stage-1 controller and trust root are unimplemented; source evaluation cannot advance beyond HOLD_EPOCH_OPEN".to_owned(),
        ],
        "complete source fixture must have no schema/parser wire mismatch findings"
    );

    for field in SOURCE_RECEIPT_REQUIRED_FIELDS {
        let mut missing = complete.clone();
        missing["controls"][0]["receipt_refs"][0]
            .as_object_mut()
            .expect("source receipt object")
            .remove(field);
        assert!(
            evaluate_source_epoch(&program(), &missing)
                .findings
                .iter()
                .any(|finding| finding.contains(field)),
            "missing canonical source receipt field {field} must fail"
        );
    }

    let mut unexpected = complete;
    unexpected["controls"][0]["receipt_refs"][0]["unexpected"] = json!(true);
    assert!(
        evaluate_source_epoch(&program(), &unexpected)
            .findings
            .iter()
            .any(|finding| finding.contains("unexpected")),
        "unexpected source receipt field must fail"
    );
}

#[test]
fn c15_blind_reader_uses_independent_oracle_in_source_and_protected_bindings() {
    let candidate = pass_epoch();
    let blind_reader = &candidate["context_free_exit"]["blind_reader_receipt_ref"];
    assert_eq!(
        blind_reader["issuer_authority_class"],
        json!("independent-oracle")
    );
    assert_eq!(
        evaluate_source_epoch(&program(), &candidate).findings,
        vec!["external authenticated Stage-1 controller and trust root are unimplemented; source evaluation cannot advance beyond HOLD_EPOCH_OPEN".to_owned()]
    );

    let facts = protected_facts(&candidate);
    let protected_blind_reader = facts["receipt_bindings"]
        .as_array()
        .expect("protected bindings")
        .iter()
        .find(|binding| binding["principal_id"] == "blind-reader")
        .expect("C15 blind-reader protected binding");
    assert_eq!(
        protected_blind_reader["issuer_authority_class"], blind_reader["issuer_authority_class"],
        "protected binding must exactly preserve the C15 blind-reader authority class"
    );
}

#[test]
fn nested_bindings_and_extensions_match_closed_schema_contracts() {
    let source_epoch = pass_epoch();
    let mut source_unknown_binding = source_epoch.clone();
    source_unknown_binding["controls"][0]["receipt_refs"][0]["authority_source_ref"]["unexpected"] =
        json!(true);
    assert!(
        evaluate_source_epoch(&program(), &source_unknown_binding)
            .findings
            .iter()
            .any(
                |finding| finding.contains("authority_source_ref rejects unknown field unexpected")
            ),
        "source authority binding must remain closed"
    );

    let mut protected_unknown_binding = grammar_facts();
    protected_unknown_binding["source_epoch_binding"]["unexpected"] = json!(true);
    assert!(
        validate_protected_facts_grammar(&protected_unknown_binding)
            .findings
            .iter()
            .any(
                |finding| finding.contains("source_epoch_binding rejects unknown field unexpected")
            ),
        "protected-facts binding must remain closed"
    );

    let mut protected_unknown_receipt_binding = grammar_facts();
    protected_unknown_receipt_binding["receipt_bindings"][0]["authority_source_ref"]["unexpected"] =
        json!(true);
    assert!(
        validate_protected_facts_grammar(&protected_unknown_receipt_binding)
            .findings
            .iter()
            .any(
                |finding| finding.contains("authority_source_ref rejects unknown field unexpected")
            ),
        "protected receipt binding must remain closed"
    );

    let mut invalid_extensions = grammar_facts();
    invalid_extensions["extensions"] = json!({"unqualified": true});
    assert!(
        validate_protected_facts_grammar(&invalid_extensions)
            .findings
            .iter()
            .any(|finding| finding.contains("extensions rejects unknown field unqualified")),
        "protected-facts extensions must use the non-authoritative prefix"
    );

    for schema_name in ["protected-facts", "admission-envelope"] {
        let schema = schema(schema_name);
        assert_eq!(
            schema["properties"]["extensions"]["propertyNames"]["pattern"],
            json!(format!("^{STAGE1_NON_AUTHORITATIVE_EXTENSION_PREFIX}")),
            "{schema_name} extension prefix must match parser contract"
        );
    }
}

#[test]
fn red_candidate_facts_can_never_authorize_planning_without_external_controller() {
    let candidate = pass_epoch();
    assert!(!evaluate_epoch(&program(), &candidate).is_green());
    assert!(!validate_protected_facts_shape(&protected_facts(&candidate)).is_green());
}

#[test]
fn pass_rejects_tampered_protected_receipt_and_half_of_c11() {
    let candidate = pass_epoch();
    let mut facts = protected_facts(&candidate);
    facts["receipt_bindings"]
        .as_array_mut()
        .expect("bindings")
        .retain(|binding| {
            binding["principal_id"] != "issuer-C11-human"
                && binding["principal_id"] != "reviewer-L01"
        });
    let report = validate_protected_facts_shape(&facts);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("non-authoritative"))
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
        "authority_source_ref": "authority://legal",
        "reason": "review is pending"
    }]);
    assert!(!evaluate_epoch(&program(), &candidate).is_green());
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
    facts["envelope_signature"] = json!(false);
    let report = validate_protected_facts_shape(&facts);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("envelope_signature"))
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
        validate_protected_facts_shape(&facts)
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
        validate_admission_envelope_shape(&envelope)
            .findings
            .iter()
            .any(|finding| finding.contains("non-authoritative"))
    );
}

#[test]
fn admission_envelope_matches_schema_fields_and_remains_non_authoritative() {
    let envelope = admission_envelope();
    let schema = schema("admission-envelope");
    let mut schema_required = schema["required"]
        .as_array()
        .expect("schema required fields")
        .iter()
        .map(|field| field.as_str().expect("required field string"))
        .collect::<Vec<_>>();
    let mut fixture_fields = envelope
        .as_object()
        .expect("fixture object")
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    schema_required.sort_unstable();
    fixture_fields.sort_unstable();
    assert_eq!(
        fixture_fields, schema_required,
        "fixture has exactly schema-required fields"
    );
    let mut parser_required = ADMISSION_ENVELOPE_REQUIRED_FIELDS.to_vec();
    parser_required.sort_unstable();
    assert_eq!(
        parser_required, schema_required,
        "parser has exactly schema-required fields"
    );
    let mut schema_fields = schema["properties"]
        .as_object()
        .expect("schema properties")
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let mut parser_fields = ADMISSION_ENVELOPE_ALLOWED_FIELDS.to_vec();
    schema_fields.sort_unstable();
    parser_fields.sort_unstable();
    assert_eq!(
        parser_fields, schema_fields,
        "parser has exactly schema-declared fields"
    );

    assert_eq!(
        validate_admission_envelope_shape(&envelope).findings,
        vec![
            "admission-envelope structure is non-authoritative; authenticated external controller is unimplemented"
        ]
    );
}

#[test]
fn admission_envelope_rejects_missing_and_unknown_canonical_fields() {
    let mut missing = admission_envelope();
    missing
        .as_object_mut()
        .expect("fixture object")
        .remove("facts_binding");
    assert!(
        validate_admission_envelope_shape(&missing)
            .findings
            .iter()
            .any(|finding| finding.contains("facts_binding"))
    );

    let mut unknown = admission_envelope();
    unknown["protected_facts_binding"] = json!({});
    assert!(
        validate_admission_envelope_shape(&unknown)
            .findings
            .iter()
            .any(|finding| finding.contains("unknown field protected_facts_binding"))
    );
}

#[test]
fn red_no_ci_facade_registration_or_generated_face_hand_edit_surface_exists() {
    assert!(!std::path::Path::new("ci/facade/stage1-closure").exists());
    assert!(!std::path::Path::new("libs/oya-stage1-closure/scm-facts.generated.json").exists());
}

#[test]
fn red_placeholder_facts_and_envelope_never_authorize() {
    let facts =
        json!({"schema_id": "oyatie/stage1-protected-facts/v1", "protected_parent_verified": true});
    let envelope = json!({"schema_id": "oyatie/stage1-admission-envelope/v1", "roadmap_planning_authorized": true});
    assert!(!validate_protected_facts_shape(&facts).is_green());
    assert!(!validate_admission_envelope_shape(&envelope).is_green());
}

#[test]
fn red_authority_receipt_binding_requires_closed_identity_and_authority_fields() {
    let schema = schema("protected-facts");
    let binding = &schema["$defs"]["authority_receipt_binding"];
    let required = binding["required"]
        .as_array()
        .expect("required authority fields");
    for field in [
        "role",
        "control_id",
        "principal_identity_binding",
        "authority_source_ref",
        "qualification",
        "jurisdiction_scope",
        "independence_observation",
        "subject_binding",
        "program_binding",
        "epoch_binding",
        "decision",
        "validity",
        "conflict_status",
        "signature_trust_root_binding",
        "path",
        "blob_oid",
        "sha256",
    ] {
        assert!(required.contains(&json!(field)), "missing required {field}");
    }
    assert_eq!(binding["additionalProperties"], false);
}

#[test]
fn protected_receipt_bindings_match_schema_allowed_fields_and_reject_legacy_fields() {
    let schema = schema("protected-facts");
    let mut schema_fields = schema["$defs"]["authority_receipt_binding"]["properties"]
        .as_object()
        .expect("protected receipt schema properties")
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let mut parser_fields = PROTECTED_RECEIPT_BINDING_ALLOWED_FIELDS.to_vec();
    schema_fields.sort_unstable();
    parser_fields.sort_unstable();
    assert_eq!(parser_fields, schema_fields, "protected receipt wire drift");

    for legacy_field in [
        "issuer_id",
        "issuer_class",
        "trust_root_authority_ref",
        "authority_ref",
        "unexpected",
    ] {
        let mut facts = grammar_facts();
        facts["receipt_bindings"][0][legacy_field] = json!(true);
        assert!(
            validate_protected_facts_grammar(&facts)
                .findings
                .iter()
                .any(|finding| finding
                    == &format!("protected receipt binding rejects unknown field {legacy_field}")),
            "legacy protected receipt field {legacy_field} must fail closed"
        );
    }
}

#[test]
fn red_control_specific_authority_roles_and_cardinality_are_declared() {
    let schema = schema("protected-facts");
    let binding_rules = schema["$defs"]["authority_receipt_binding"]["allOf"]
        .as_array()
        .expect("control role rules");
    for control in ["C06", "C07", "C08", "C09", "C10", "C13", "C14", "C15"] {
        assert!(
            binding_rules
                .iter()
                .any(|rule| rule.to_string().contains(control)),
            "missing {control} rule"
        );
    }
    let cardinality = schema["properties"]["receipt_bindings"]["allOf"]
        .as_array()
        .expect("receipt cardinality rules");
    assert!(
        cardinality
            .iter()
            .any(|rule| rule.to_string().contains("machine-pilot-evidence"))
    );
    assert!(
        cardinality
            .iter()
            .any(|rule| rule.to_string().contains("qualified-pilot-authorization"))
    );
    assert!(
        cardinality
            .iter()
            .any(|rule| rule.to_string().contains("L16"))
    );
    assert!(
        cardinality
            .iter()
            .any(|rule| rule.to_string().contains("blind-cold-reader"))
    );
    let exact = schema["allOf"]
        .as_array()
        .expect("exact role cardinality rules");
    for (control, role) in [
        ("C11", "machine-pilot-evidence"),
        ("C11", "qualified-pilot-authorization"),
        ("C15", "deterministic-oracle"),
        ("C15", "blind-cold-reader"),
        ("C15", "qualified-planning-authority"),
    ] {
        let rule = exact.iter().find(|rule| {
            let receipt_bindings = &rule["properties"]["receipt_bindings"];
            receipt_bindings["contains"]["properties"]["control_id"]["const"] == control
                && receipt_bindings["contains"]["properties"]["role"]["const"] == role
        });
        let receipt_bindings =
            &rule.expect("exact control role rule")["properties"]["receipt_bindings"];
        assert_eq!(
            receipt_bindings["minContains"], 1,
            "missing {control}/{role}"
        );
        assert_eq!(
            receipt_bindings["maxContains"], 1,
            "duplicate {control}/{role} must fail"
        );
    }
}

fn grammar_receipt(
    control: &str,
    role: &str,
    authority: &str,
    qualification: &str,
    principal: &str,
) -> Value {
    let mut receipt = json!({
        "control_id": control,
        "role": role,
        "issuer_authority_class": authority,
        "qualification_class": qualification,
        "principal_id": principal,
        "subject_digest": SUBJECT_DIGEST,
        "program_digest": SUBJECT_DIGEST,
        "epoch_digest": SUBJECT_DIGEST
    });
    for field in [
        "principal_identity_binding",
        "authority_source_ref",
        "qualification",
        "jurisdiction_scope",
        "independence_observation",
        "subject_binding",
        "program_binding",
        "epoch_binding",
        "validity",
        "expiry",
        "revocation_status",
        "conflict_status",
        "signature_trust_root_binding",
    ] {
        receipt[field] = artifact_binding(&format!("evidence/{principal}/{field}"));
    }
    receipt["decision"] = json!("satisfied");
    receipt["path"] = json!(format!("evidence/{principal}/receipt.json"));
    receipt["blob_oid"] = json!("2222222222222222222222222222222222222222");
    receipt["sha256"] = json!("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd");
    receipt
}

fn grammar_facts() -> Value {
    let mut receipts = vec![
        grammar_receipt(
            "C06",
            "qualified-legal-compliance",
            "qualified-human",
            "legal-jcr",
            "legal",
        ),
        grammar_receipt(
            "C07",
            "affected-party-representation",
            "qualified-affected-party",
            "affected-party-recourse",
            "affected",
        ),
        grammar_receipt(
            "C08",
            "operations-owner-capacity",
            "qualified-operations",
            "named-operations-capacity",
            "operations",
        ),
        grammar_receipt(
            "C09",
            "security-evidence-custody",
            "qualified-custody",
            "security-evidence-custody",
            "custody",
        ),
        grammar_receipt(
            "C10",
            "veto-owner-closure",
            "authorized-veto",
            "exact-veto-owner",
            "veto",
        ),
        grammar_receipt(
            "C11",
            "machine-pilot-evidence",
            "machine-verifiable",
            "machine-pilot-evidence",
            "pilot-machine",
        ),
        grammar_receipt(
            "C11",
            "qualified-pilot-authorization",
            "qualified-human",
            "qualified-pilot-authorization",
            "pilot-human",
        ),
        grammar_receipt(
            "C14",
            "fresh-dissent",
            "independent-dissent",
            "fresh-independent-dissent",
            "dissent",
        ),
        grammar_receipt(
            "C15",
            "deterministic-oracle",
            "independent-oracle",
            "deterministic-oracle",
            "oracle",
        ),
        grammar_receipt(
            "C15",
            "blind-cold-reader",
            "independent-oracle",
            "blind-cold-reader",
            "reader",
        ),
        grammar_receipt(
            "C15",
            "qualified-planning-authority",
            "independent-oracle",
            "qualified-planning-authority",
            "planner",
        ),
    ];
    for lens in 1..=16 {
        receipts.push(grammar_receipt(
            "C13",
            "independent-council",
            "independent-council",
            "independent-lens-reviewer",
            &format!("lens-{lens:02}"),
        ));
        receipts.last_mut().expect("lens receipt")["lens_id"] = json!(format!("L{lens:02}"));
    }
    let mut facts = json!({
        "schema_id": "oyatie/stage1-protected-facts/v1",
        "subject_digest": SUBJECT_DIGEST,
        "program_digest": SUBJECT_DIGEST,
        "epoch_digest": SUBJECT_DIGEST,
        "receipt_bindings": receipts
    });
    for field in [
        "source_epoch_binding",
        "candidate_epoch_binding",
        "program_binding",
        "parser_binding",
        "producer_binding",
        "evaluator_binding",
        "policy_binding",
        "schema_binding",
        "predecessor_epoch_binding",
        "transition_receipt_binding",
        "immutable_successor_bundle",
        "authority_chain_result",
        "source_app_identity",
        "run_identity",
        "envelope_signature",
        "trust_root_binding",
    ] {
        facts[field] = artifact_binding(&format!("specs/{field}.json"));
    }
    for field in [
        "protected_base_repository",
        "candidate_repository",
        "protected_base_branch",
        "candidate_branch",
    ] {
        facts[field] = json!(format!("oyatie/{field}"));
    }
    for field in [
        "protected_base_commit",
        "candidate_commit",
        "protected_base_tree",
        "candidate_tree",
    ] {
        facts[field] = json!("2222222222222222222222222222222222222222");
    }
    facts
}

#[test]
fn protected_facts_grammar_rejects_authority_cardinality_identity_and_digest_failures() {
    let green = grammar_facts();
    assert!(
        validate_protected_facts_grammar(&green)
            .findings
            .iter()
            .all(|finding| finding.contains("non-authoritative"))
    );
    let mut missing_envelope_binding = green.clone();
    missing_envelope_binding
        .as_object_mut()
        .expect("facts")
        .remove("evaluator_binding");
    assert!(
        validate_protected_facts_grammar(&missing_envelope_binding)
            .findings
            .iter()
            .any(|finding| finding.contains("evaluator_binding"))
    );
    let mut extra_field = green.clone();
    extra_field["unrecognized"] = json!(true);
    assert!(
        validate_protected_facts_grammar(&extra_field)
            .findings
            .iter()
            .any(|finding| finding.contains("unknown field"))
    );
    for field in [
        "principal_identity_binding",
        "authority_source_ref",
        "qualification",
        "jurisdiction_scope",
        "independence_observation",
        "subject_binding",
        "program_binding",
        "epoch_binding",
        "validity",
        "expiry",
        "revocation_status",
        "conflict_status",
        "signature_trust_root_binding",
    ] {
        let mut missing = green.clone();
        let lens = missing["receipt_bindings"]
            .as_array_mut()
            .expect("receipts")
            .iter_mut()
            .find(|receipt| receipt["control_id"] == "C13" && receipt["lens_id"] == "L01")
            .expect("L01 receipt");
        lens.as_object_mut().expect("receipt").remove(field);
        assert!(
            validate_protected_facts_grammar(&missing)
                .findings
                .iter()
                .any(|finding| finding.contains(field)),
            "missing C13 {field} must fail"
        );
    }
    for index in 1..=4 {
        let mut wrong_class = green.clone();
        wrong_class["receipt_bindings"][index]["issuer_authority_class"] = json!("wrong-class");
        assert!(!validate_protected_facts_grammar(&wrong_class).is_green());
    }
    for control in ["C06", "C07", "C08", "C09", "C10"] {
        let mut missing = green.clone();
        missing["receipt_bindings"]
            .as_array_mut()
            .expect("receipts")
            .retain(|receipt| receipt["control_id"] != control);
        assert!(!validate_protected_facts_grammar(&missing).is_green());
    }
    let mut missing_c14 = green.clone();
    missing_c14["receipt_bindings"]
        .as_array_mut()
        .expect("receipts")
        .retain(|receipt| receipt["control_id"] != "C14");
    assert!(!validate_protected_facts_grammar(&missing_c14).is_green());
    let mut extra_c11 = green.clone();
    extra_c11["receipt_bindings"]
        .as_array_mut()
        .expect("receipts")
        .push(grammar_receipt(
            "C11",
            "machine-pilot-evidence",
            "machine-verifiable",
            "machine-pilot-evidence",
            "extra",
        ));
    assert!(!validate_protected_facts_grammar(&extra_c11).is_green());
    let mut same_lens_principal = green.clone();
    for receipt in same_lens_principal["receipt_bindings"]
        .as_array_mut()
        .expect("receipts")
    {
        if receipt["control_id"] == "C13" {
            receipt["principal_id"] = json!("same");
        }
    }
    assert!(!validate_protected_facts_grammar(&same_lens_principal).is_green());
    let mut duplicate_exit_principal = green.clone();
    duplicate_exit_principal["receipt_bindings"][9]["principal_id"] = json!("oracle");
    assert!(!validate_protected_facts_grammar(&duplicate_exit_principal).is_green());
    let mut unqualified_planner = green.clone();
    unqualified_planner["receipt_bindings"][10]["qualification_class"] = json!("unqualified");
    assert!(!validate_protected_facts_grammar(&unqualified_planner).is_green());
    let mut mismatched_digest = green;
    mismatched_digest["receipt_bindings"][0]["program_digest"] =
        json!("sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc");
    assert!(!validate_protected_facts_grammar(&mismatched_digest).is_green());
}

#[test]
fn receipt_validation_has_no_legacy_authority_field_reads() {
    let source = include_str!("../src/lib.rs");
    for legacy in [
        "issuer_".to_owned() + "id",
        "issuer_".to_owned() + "class",
        "trust_root_".to_owned() + "authority_ref",
        "authority_".to_owned() + "ref",
    ] {
        assert!(
            !source.contains(&legacy),
            "legacy field read remains: {legacy}"
        );
    }
}
