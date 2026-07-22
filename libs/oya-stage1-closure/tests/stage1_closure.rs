use std::collections::BTreeSet;

use serde_json::{Value, json};
use stage1_closure::{
    ADMISSION_ENVELOPE_ALLOWED_FIELDS, ADMISSION_ENVELOPE_REQUIRED_FIELDS,
    PROTECTED_RECEIPT_BINDING_ALLOWED_FIELDS, SOURCE_RECEIPT_REQUIRED_FIELDS,
    STAGE1_NON_AUTHORITATIVE_EXTENSION_PREFIX, evaluate_epoch, evaluate_program,
    evaluate_protected_facts_linkage, evaluate_source_epoch, validate_admission_envelope_shape,
    validate_protected_facts_grammar, validate_protected_facts_shape,
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
    epoch["controls"][12]["receipt_refs"] = json!([epoch["lenses"][0]["receipt_ref"].clone()]);
    epoch["controls"][13]["receipt_refs"] = json!([epoch["fresh_dissent"]["receipt_ref"].clone()]);
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
    let mut facts = grammar_facts();
    for control in epoch["controls"].as_array().expect("controls") {
        let control_id = control["control_id"].as_str().expect("control id");
        let role = match control_id {
            "C01" | "C02" | "C03" | "C04" | "C05" | "C12" => Some("machine-evidence"),
            "C06" => Some("qualified-legal-compliance"),
            "C07" => Some("affected-party-representation"),
            "C08" => Some("operations-owner-capacity"),
            "C09" => Some("security-evidence-custody"),
            "C10" => Some("veto-owner-closure"),
            "C11" => None,
            "C15" => Some("qualified-planning-authority"),
            _ => continue,
        };
        for receipt in control["receipt_refs"]
            .as_array()
            .expect("control receipts")
        {
            let role = role.or_else(|| match receipt["issuer_authority_class"].as_str() {
                Some("machine-verifiable") => Some("machine-pilot-evidence"),
                Some("qualified-human") => Some("qualified-pilot-authorization"),
                _ => None,
            });
            map_protected_receipt(
                &mut facts,
                receipt,
                control_id,
                role.expect("known role"),
                None,
            );
        }
    }
    for lens in epoch["lenses"].as_array().expect("lenses") {
        let lens_id = lens["lens_id"].as_str().expect("lens id");
        map_protected_receipt(
            &mut facts,
            &lens["receipt_ref"],
            "C13",
            "independent-council",
            Some(lens_id),
        );
    }
    map_protected_receipt(
        &mut facts,
        &epoch["fresh_dissent"]["receipt_ref"],
        "C14",
        "fresh-dissent",
        None,
    );
    for (field, role) in [
        ("oracle_receipt_ref", "deterministic-oracle"),
        ("blind_reader_receipt_ref", "blind-cold-reader"),
    ] {
        map_protected_receipt(
            &mut facts,
            &epoch["context_free_exit"][field],
            "C15",
            role,
            None,
        );
    }
    facts
}

fn map_protected_receipt(
    facts: &mut Value,
    source: &Value,
    control_id: &str,
    role: &str,
    lens_id: Option<&str>,
) {
    let binding = facts["receipt_bindings"]
        .as_array_mut()
        .expect("protected receipt bindings")
        .iter_mut()
        .find(|binding| {
            binding["control_id"] == control_id
                && binding["role"] == role
                && lens_id.is_none_or(|lens_id| binding["lens_id"] == lens_id)
        })
        .expect("schema-valid protected receipt binding");
    for field in [
        "path",
        "blob_oid",
        "sha256",
        "subject_digest",
        "principal_id",
        "issuer_authority_class",
        "authority_source_ref",
    ] {
        binding[field] = source[field].clone();
    }
}

fn protected_binding_mut<'a>(
    facts: &'a mut Value,
    control_id: &str,
    role: &str,
    lens_id: Option<&str>,
) -> &'a mut Value {
    facts["receipt_bindings"]
        .as_array_mut()
        .expect("protected receipt bindings")
        .iter_mut()
        .find(|binding| {
            binding["control_id"] == control_id
                && binding["role"] == role
                && lens_id.is_none_or(|lens_id| binding["lens_id"] == lens_id)
        })
        .expect("protected receipt binding")
}

#[test]
fn canonical_program_and_open_hold_epoch_are_green() {
    let program = program();
    assert!(evaluate_program(&program).is_green());
    assert!(evaluate_epoch(&program, &hold_epoch()).is_green());
}

#[test]
fn public_protected_facts_linkage_remains_non_authoritative_and_held() {
    let epoch = pass_epoch();
    let facts = protected_facts(&epoch);
    let source_receipt_count = epoch["controls"]
        .as_array()
        .expect("controls")
        .iter()
        .map(|control| {
            control["receipt_refs"]
                .as_array()
                .expect("control receipts")
                .len()
        })
        .sum::<usize>()
        + epoch["lenses"].as_array().expect("lenses").len()
        + 3;
    assert_eq!(
        source_receipt_count, 35,
        "public fixture receipt population"
    );
    assert_eq!(
        facts["receipt_bindings"]
            .as_array()
            .expect("protected receipts")
            .len(),
        33,
        "public fixture must retain exactly the required protected receipt population"
    );
    assert!(
        validate_protected_facts_grammar(&facts)
            .findings
            .iter()
            .all(|finding| finding.contains("non-authoritative")),
        "cross-link fixture must be schema-valid before linkage: {:?}",
        validate_protected_facts_grammar(&facts).findings
    );
    let report = evaluate_protected_facts_linkage(&program(), &epoch, &facts);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("non-authoritative"))
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("HOLD_EPOCH_OPEN"))
    );
    assert!(
        report.findings.iter().all(|finding| {
            finding.contains("external authenticated Stage-1 controller")
                || finding.contains("protected-facts linkage is structural")
        }),
        "full source-to-protected linkage must have only intentional HOLD denials: {:?}",
        report.findings
    );
}

#[test]
fn public_linkage_requires_unique_full_contextual_c13_and_c15_bindings() {
    let epoch = pass_epoch();
    let green = protected_facts(&epoch);
    for field in [
        "path",
        "blob_oid",
        "sha256",
        "subject_digest",
        "principal_id",
        "issuer_authority_class",
        "authority_source_ref",
    ] {
        let mut mutated = green.clone();
        let binding =
            protected_binding_mut(&mut mutated, "C13", "independent-council", Some("L01"));
        binding[field] = json!("tampered");
        let report = evaluate_protected_facts_linkage(&program(), &epoch, &mutated);
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.contains("C13/L01 source receipt lacks an exact")),
            "tampered mapped field {field} must break C13 linkage: {:?}",
            report.findings
        );
    }

    let mut lens_permutation = green.clone();
    protected_binding_mut(
        &mut lens_permutation,
        "C13",
        "independent-council",
        Some("L01"),
    )["lens_id"] = json!("L02");
    assert!(
        evaluate_protected_facts_linkage(&program(), &epoch, &lens_permutation)
            .findings
            .iter()
            .any(|finding| finding.contains("C13/L01 source receipt lacks an exact"))
    );

    let mut role_swap = green.clone();
    protected_binding_mut(&mut role_swap, "C15", "deterministic-oracle", None)["role"] =
        json!("blind-cold-reader");
    assert!(
        evaluate_protected_facts_linkage(&program(), &epoch, &role_swap)
            .findings
            .iter()
            .any(|finding| finding
                .contains("C15/deterministic-oracle source receipt lacks an exact"))
    );

    let mut missing_third_role = green.clone();
    missing_third_role["receipt_bindings"]
        .as_array_mut()
        .expect("receipt bindings")
        .retain(|binding| {
            binding["control_id"] != "C15" || binding["role"] != "qualified-planning-authority"
        });
    assert!(
        evaluate_protected_facts_linkage(&program(), &epoch, &missing_third_role)
            .findings
            .iter()
            .any(|finding| finding
                .contains("C15/qualified-planning-authority source receipt lacks an exact"))
    );

    let mut duplicate = green;
    let duplicate_binding =
        protected_binding_mut(&mut duplicate, "C13", "independent-council", Some("L01")).clone();
    duplicate["receipt_bindings"]
        .as_array_mut()
        .expect("receipt bindings")
        .push(duplicate_binding);
    assert!(
        evaluate_protected_facts_linkage(&program(), &epoch, &duplicate)
            .findings
            .iter()
            .any(|finding| finding.contains("C13/L01 source receipt has multiple exact"))
    );
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
fn protected_facts_schema_has_full_rust_receipt_parity() {
    let schema = schema("protected-facts");
    let binding_rules = schema["$defs"]["authority_receipt_binding"]["allOf"]
        .as_array()
        .expect("control role rules");
    assert!(binding_rules.iter().any(|rule| rule.get("oneOf").is_some()));
    let cardinality = schema["properties"]["receipt_bindings"]["allOf"]
        .as_array()
        .expect("receipt cardinality rules");
    let required_pairs = [
        ("C01", "machine-evidence"),
        ("C02", "machine-evidence"),
        ("C03", "machine-evidence"),
        ("C04", "machine-evidence"),
        ("C05", "machine-evidence"),
        ("C06", "qualified-legal-compliance"),
        ("C07", "affected-party-representation"),
        ("C08", "operations-owner-capacity"),
        ("C09", "security-evidence-custody"),
        ("C10", "veto-owner-closure"),
        ("C11", "machine-pilot-evidence"),
        ("C11", "qualified-pilot-authorization"),
        ("C12", "machine-evidence"),
        ("C14", "fresh-dissent"),
        ("C15", "deterministic-oracle"),
        ("C15", "blind-cold-reader"),
        ("C15", "qualified-planning-authority"),
    ];
    for (control, role) in required_pairs {
        let rule = cardinality.iter().find(|rule| {
            rule["contains"]["properties"]["control_id"]["const"].as_str() == Some(control)
                && rule["contains"]["properties"]["role"]["const"].as_str() == Some(role)
        });
        let receipt_bindings = rule.expect("exact control role rule");
        assert_eq!(
            receipt_bindings["minContains"], 1,
            "missing {control}/{role}"
        );
        assert_eq!(
            receipt_bindings["maxContains"], 1,
            "duplicate {control}/{role} must fail"
        );
    }
    for lens in 1..=16 {
        let lens = format!("L{lens:02}");
        let rule = cardinality.iter().find(|rule| {
            rule["contains"]["properties"]["control_id"]["const"].as_str() == Some("C13")
                && rule["contains"]["properties"]["lens_id"]["const"].as_str()
                    == Some(lens.as_str())
        });
        let receipt_bindings = rule.expect("exact C13 lens cardinality rule");
        assert_eq!(receipt_bindings["minContains"], 1, "missing C13/{lens}");
        assert_eq!(
            receipt_bindings["maxContains"], 1,
            "duplicate C13/{lens} must fail"
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
            "C01",
            "machine-evidence",
            "machine-verifiable",
            "machine-evidence",
            "foundation-01",
        ),
        grammar_receipt(
            "C02",
            "machine-evidence",
            "machine-verifiable",
            "machine-evidence",
            "foundation-02",
        ),
        grammar_receipt(
            "C03",
            "machine-evidence",
            "machine-verifiable",
            "machine-evidence",
            "foundation-03",
        ),
        grammar_receipt(
            "C04",
            "machine-evidence",
            "machine-verifiable",
            "machine-evidence",
            "foundation-04",
        ),
        grammar_receipt(
            "C05",
            "machine-evidence",
            "machine-verifiable",
            "machine-evidence",
            "foundation-05",
        ),
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
            "C12",
            "machine-evidence",
            "machine-verifiable",
            "machine-evidence",
            "foundation-12",
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
fn receipt_population_parity_rejects_each_missing_requirement_and_unsupported_pair() {
    let required_pairs = [
        ("C01", "machine-evidence"),
        ("C02", "machine-evidence"),
        ("C03", "machine-evidence"),
        ("C04", "machine-evidence"),
        ("C05", "machine-evidence"),
        ("C06", "qualified-legal-compliance"),
        ("C07", "affected-party-representation"),
        ("C08", "operations-owner-capacity"),
        ("C09", "security-evidence-custody"),
        ("C10", "veto-owner-closure"),
        ("C11", "machine-pilot-evidence"),
        ("C11", "qualified-pilot-authorization"),
        ("C12", "machine-evidence"),
        ("C14", "fresh-dissent"),
        ("C15", "deterministic-oracle"),
        ("C15", "blind-cold-reader"),
        ("C15", "qualified-planning-authority"),
    ];
    let green = grammar_facts();

    for (control, role) in required_pairs {
        let mut missing = green.clone();
        missing["receipt_bindings"]
            .as_array_mut()
            .expect("bindings")
            .retain(|receipt| !(receipt["control_id"] == control && receipt["role"] == role));
        assert!(
            !validate_protected_facts_grammar(&missing).is_green(),
            "removing {control}/{role} must fail"
        );
    }
    for lens in 1..=16 {
        let lens = format!("L{lens:02}");
        let mut missing = green.clone();
        missing["receipt_bindings"]
            .as_array_mut()
            .expect("bindings")
            .retain(|receipt| !(receipt["control_id"] == "C13" && receipt["lens_id"] == lens));
        assert!(
            !validate_protected_facts_grammar(&missing).is_green(),
            "removing C13/{lens} must fail"
        );
    }

    let supported = BTreeSet::from(required_pairs);
    let roles = [
        "machine-evidence",
        "qualified-legal-compliance",
        "affected-party-representation",
        "operations-owner-capacity",
        "security-evidence-custody",
        "veto-owner-closure",
        "machine-pilot-evidence",
        "qualified-pilot-authorization",
        "independent-council",
        "fresh-dissent",
        "deterministic-oracle",
        "blind-cold-reader",
        "qualified-planning-authority",
    ];
    for control_number in 1..=15 {
        let control = format!("C{control_number:02}");
        for role in roles {
            if supported.contains(&(control.as_str(), role))
                || (control == "C13" && role == "independent-council")
            {
                continue;
            }
            let mut unsupported = green.clone();
            unsupported["receipt_bindings"]
                .as_array_mut()
                .expect("bindings")
                .push(grammar_receipt(
                    &control,
                    role,
                    "wrong-authority",
                    "wrong-qualification",
                    "unsupported",
                ));
            assert!(
                !validate_protected_facts_grammar(&unsupported).is_green(),
                "unsupported {control}/{role} must fail"
            );
        }
    }
    let mut unsupported_lens = green;
    let receipt = unsupported_lens["receipt_bindings"]
        .as_array_mut()
        .expect("bindings")
        .iter_mut()
        .find(|receipt| receipt["control_id"] == "C13" && receipt["lens_id"] == "L01")
        .expect("C13/L01");
    receipt["lens_id"] = json!("L17");
    assert!(!validate_protected_facts_grammar(&unsupported_lens).is_green());
}

#[test]
fn schema_equivalent_closed_shapes_reject_locator_types_extra_records_and_unknown_keys() {
    type Mutation = (&'static str, fn(&mut Value));

    for (field, value) in [
        ("path", json!(17)),
        ("path", json!("")),
        ("blob_oid", json!("not-an-oid")),
        ("sha256", json!("not-a-digest")),
    ] {
        let mut facts = grammar_facts();
        facts["receipt_bindings"][0][field] = value;
        assert!(
            validate_protected_facts_grammar(&facts)
                .findings
                .iter()
                .any(|finding| finding.contains(field)),
            "protected receipt {field} must use its schema shape"
        );
    }
    let mut unsupported = grammar_facts();
    unsupported["receipt_bindings"]
        .as_array_mut()
        .expect("bindings")
        .push(grammar_receipt(
            "C01",
            "qualified-legal-compliance",
            "machine-verifiable",
            "machine-evidence",
            "unsupported",
        ));
    assert!(
        validate_protected_facts_grammar(&unsupported)
            .findings
            .iter()
            .any(|finding| finding.contains("unsupported control/role/lens"))
    );

    let program_mutations: [Mutation; 7] = [
        ("program", |value| value["unexpected"] = json!(true)),
        ("transition", |value| {
            value["transitions"][0]["unexpected"] = json!(true)
        }),
        ("group", |value| {
            value["groups"][0]["unexpected"] = json!(true)
        }),
        ("program control", |value| {
            value["controls"][0]["unexpected"] = json!(true)
        }),
        ("program lens", |value| {
            value["lenses"][0]["unexpected"] = json!(true)
        }),
        ("candidate_effects", |value| {
            value["candidate_effects"]["unexpected"] = json!(true)
        }),
        ("mutation_policy", |value| {
            value["mutation_policy"]["unexpected"] = json!(true)
        }),
    ];
    for (shape, mutate) in program_mutations {
        let mut candidate = program();
        mutate(&mut candidate);
        assert!(
            evaluate_program(&candidate)
                .findings
                .iter()
                .any(|finding| finding.contains(shape) && finding.contains("unknown field"))
        );
    }

    let epoch_mutations: [Mutation; 8] = [
        ("epoch", |value| value["unexpected"] = json!(true)),
        ("subject_binding", |value| {
            value["subject_binding"]["unexpected"] = json!(true)
        }),
        ("planning", |value| {
            value["planning"]["unexpected"] = json!(true)
        }),
        ("epoch control", |value| {
            value["controls"][0]["unexpected"] = json!(true)
        }),
        ("epoch lens", |value| {
            value["lenses"][0]["unexpected"] = json!(true)
        }),
        ("fresh_dissent", |value| {
            value["fresh_dissent"]["unexpected"] = json!(true)
        }),
        ("immutable_successor", |value| {
            value["immutable_successor"]["unexpected"] = json!(true)
        }),
        ("context_free_exit", |value| {
            value["context_free_exit"]["unexpected"] = json!(true)
        }),
    ];
    for (shape, mutate) in epoch_mutations {
        let mut candidate = hold_epoch();
        mutate(&mut candidate);
        assert!(
            evaluate_source_epoch(&program(), &candidate)
                .findings
                .iter()
                .any(|finding| finding.contains(shape) && finding.contains("unknown field"))
        );
    }

    let mut successor_ref = pass_epoch();
    successor_ref["immutable_successor"]["facts_ref"]["unexpected"] = json!(true);
    assert!(
        evaluate_source_epoch(&program(), &successor_ref)
            .findings
            .iter()
            .any(|finding| finding.contains("immutable_successor.facts_ref")
                && finding.contains("unknown field"))
    );

    let mut open_successor_ref = hold_epoch();
    open_successor_ref["immutable_successor"]["facts_ref"] = json!({"unexpected": true});
    assert!(
        evaluate_source_epoch(&program(), &open_successor_ref)
            .findings
            .iter()
            .any(|finding| finding.contains("immutable_successor.facts_ref")
                && finding.contains("unknown field"))
    );

    let mut blocker = hold_epoch();
    blocker["blockers"] = json!([{
        "control_id": "C06",
        "input_class": "legal-review",
        "required_qualification": "licensed-jcr-reviewer",
        "scope": "stage1-closure",
        "authority_source_ref": "authority://legal",
        "reason": "review is pending",
        "unexpected": true
    }]);
    assert!(
        evaluate_source_epoch(&program(), &blocker)
            .findings
            .iter()
            .any(|finding| finding.contains("blockers[0]") && finding.contains("unknown field"))
    );
}

#[test]
fn pending_epoch_requires_every_schema_member_before_hold_evaluation() {
    fn assert_pending_invalid(epoch: &Value, label: &str) {
        let report = evaluate_source_epoch(&program(), epoch);
        assert!(
            !report.is_green(),
            "schema-invalid pending epoch was accepted: {label}"
        );
        assert!(
            report
                .findings
                .iter()
                .all(|finding| !finding.contains("external authenticated Stage-1 controller")),
            "shape failure must be reported before controller HOLD: {label}: {:?}",
            report.findings
        );
    }

    assert!(evaluate_source_epoch(&program(), &hold_epoch()).is_green());

    for epoch_id in [
        json!(7),
        json!("epoch-1"),
        json!("stage1-epoch-"),
        json!("stage1-epoch/a"),
    ] {
        let mut candidate = hold_epoch();
        candidate["epoch_id"] = epoch_id;
        assert_pending_invalid(&candidate, "epoch_id pattern");
    }

    for field in ["immutable_successor", "blockers"] {
        let mut candidate = hold_epoch();
        candidate
            .as_object_mut()
            .expect("epoch object")
            .remove(field);
        assert_pending_invalid(&candidate, field);
    }
    for field in ["control_id", "status", "subject_digest", "receipt_refs"] {
        let mut candidate = hold_epoch();
        candidate["controls"][0]
            .as_object_mut()
            .expect("control")
            .remove(field);
        assert_pending_invalid(&candidate, field);
    }
    for (field, value) in [("subject_digest", json!(7)), ("receipt_refs", json!({}))] {
        let mut candidate = hold_epoch();
        candidate["controls"][0][field] = value;
        assert_pending_invalid(&candidate, field);
    }
    let mut pending_receipt = hold_epoch();
    pending_receipt["controls"][0]["receipt_refs"] =
        json!([receipt("pending-C01", "machine-verifiable")]);
    assert!(evaluate_source_epoch(&program(), &pending_receipt).is_green());
    for (field, value) in [
        ("path", json!(7)),
        ("blob_oid", json!("not-an-oid")),
        ("sha256", json!("not-a-digest")),
        ("subject_digest", json!(7)),
        ("subject_digest", json!("sha256:not-a-digest")),
        ("principal_id", json!(7)),
        ("principal_id", json!("")),
        ("issuer_authority_class", json!(7)),
        ("issuer_authority_class", json!("")),
        ("authority_source_ref", json!("not-an-object")),
        ("qualification", json!(7)),
        ("jurisdiction_scope", json!(7)),
        ("independence_observation", json!(7)),
        ("validity", json!(7)),
        ("revocation_status", json!(7)),
        ("conflict_status", json!(7)),
        ("signature_trust_root_binding", json!(7)),
    ] {
        let mut candidate = pending_receipt.clone();
        candidate["controls"][0]["receipt_refs"][0][field] = value;
        assert_pending_invalid(&candidate, field);
    }
    for field in [
        "lens_id",
        "status",
        "subject_digest",
        "reviewer_id",
        "receipt_ref",
    ] {
        let mut candidate = hold_epoch();
        candidate["lenses"][0]
            .as_object_mut()
            .expect("lens")
            .remove(field);
        assert_pending_invalid(&candidate, field);
    }
    for field in [
        "status",
        "subject_digest",
        "reviewer_id",
        "fresh_context",
        "prior_context_used",
        "findings_resolved_or_carried",
        "receipt_ref",
    ] {
        let mut candidate = hold_epoch();
        candidate["fresh_dissent"]
            .as_object_mut()
            .expect("dissent")
            .remove(field);
        assert_pending_invalid(&candidate, field);
    }
    for field in [
        "status",
        "subject_digest",
        "oracle_principal_id",
        "blind_reader_principal_id",
        "conversation_context_used",
        "reproduced_verdict",
        "oracle_receipt_ref",
        "blind_reader_receipt_ref",
    ] {
        let mut candidate = hold_epoch();
        candidate["context_free_exit"]
            .as_object_mut()
            .expect("exit")
            .remove(field);
        assert_pending_invalid(&candidate, field);
    }
    for field in ["frozen", "subject_digest", "facts_ref"] {
        let mut candidate = hold_epoch();
        candidate["immutable_successor"]
            .as_object_mut()
            .expect("successor")
            .remove(field);
        assert_pending_invalid(&candidate, field);
    }
    let mut candidate = hold_epoch();
    candidate["blockers"] = json!([{
        "control_id": "C06",
        "input_class": "legal-review",
        "required_qualification": "licensed-jcr-reviewer",
        "scope": "stage1-closure",
        "authority_source_ref": "authority://legal",
        "reason": "review is pending"
    }]);
    for field in [
        "control_id",
        "input_class",
        "required_qualification",
        "scope",
        "authority_source_ref",
        "reason",
    ] {
        let mut mutation = candidate.clone();
        mutation["blockers"][0]
            .as_object_mut()
            .expect("blocker")
            .remove(field);
        assert_pending_invalid(&mutation, field);
    }
}

#[test]
fn lowercase_hex_and_protected_facts_preflight_fail_closed_before_lifecycle() {
    fn uppercase_first_hex(value: &str) -> String {
        format!("A{}", &value[1..])
    }

    for (field, value) in [
        (
            "blob_oid",
            uppercase_first_hex("1111111111111111111111111111111111111111"),
        ),
        (
            "sha256",
            uppercase_first_hex("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        ),
        (
            "subject_digest",
            "sha256:Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
        ),
        (
            "sha256",
            "gbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned(),
        ),
    ] {
        let mut pending = hold_epoch();
        pending["controls"][0]["receipt_refs"] =
            json!([receipt("pending-C01", "machine-verifiable")]);
        pending["controls"][0]["receipt_refs"][0][field] = json!(value);
        assert!(!evaluate_source_epoch(&program(), &pending).is_green());
    }

    let mut successor = pass_epoch();
    successor["immutable_successor"]["facts_ref"]["sha256"] = json!(uppercase_first_hex(
        "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
    ));
    assert!(
        evaluate_source_epoch(&program(), &successor)
            .findings
            .iter()
            .any(|finding| finding.contains("immutable_successor.facts_ref.sha256"))
    );

    let mut epoch = pass_epoch();
    epoch["state"] = json!("HOLD_EPOCH_OPEN");
    let facts = protected_facts(&epoch);
    let valid_hold = evaluate_protected_facts_linkage(&program(), &epoch, &facts);
    assert!(
        valid_hold
            .findings
            .iter()
            .all(|finding| finding.contains("protected-facts linkage is structural")),
        "valid HOLD linkage must be intentional only: {:?}",
        valid_hold.findings
    );

    let mut malformed_binding = facts.clone();
    malformed_binding["program_binding"]["blob_oid"] = json!(uppercase_first_hex(
        "dddddddddddddddddddddddddddddddddddddddd"
    ));
    assert!(
        evaluate_protected_facts_linkage(&program(), &epoch, &malformed_binding)
            .findings
            .iter()
            .any(|finding| finding.contains("program_binding.blob_oid"))
    );

    let mut malformed_receipt = facts;
    malformed_receipt["receipt_bindings"][0]["sha256"] = json!(uppercase_first_hex(
        "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
    ));
    let report = evaluate_protected_facts_linkage(&program(), &epoch, &malformed_receipt);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.contains("protected receipt binding.sha256"))
    );
    assert!(
        report
            .findings
            .iter()
            .all(|finding| !finding.contains("external authenticated Stage-1 controller"))
    );
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
