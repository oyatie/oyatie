// GATE-1 cloud-ci-cross-artifact-agreement: RED/GREEN fixture corpus + born-blocking
// live-corpus self-test. ADR-0083 Tier-3: integration tests use unwrap/expect to assert
// invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ci_cross_artifact_agreement::{
    AdrDecisionRecord, FOUNDER_PRODUCT_INTENT_PATH, GateCoverageBaseline, RatchetReport, Verdict,
    derive_masterplan_md_projection, evaluate, evaluate_adr_index_projection_parity,
    evaluate_adr_prose_frontmatter_status, evaluate_founder_product_intent_agreement,
    evaluate_masterplan_plan_evidence_crosscheck, evaluate_masterplan_projection_rederivation,
    evaluate_masterplan_read_surface_resurrections, evaluate_masterplan_v2_authority,
    evaluate_masterplan_v2_entry_surfaces, evaluate_masterplan_v2_evidence_state,
    evaluate_masterplan_v2_plan_evidence_drift, evaluate_masterplan_v2_program_coverage,
    evaluate_masterplan_v2_projection_freshness, evaluate_masterplan_v2_ratification_digest,
    evaluate_masterplan_v2_read_contract_archives, evaluate_masterplan_v2_sequencing,
    evaluate_registry_derived_policy_sync, ratchet,
};
use serde_json::Value;

type JsonMutation = Box<dyn Fn(&mut Value)>;

/// Walk up to the repo root (the dir holding specs/root-hub-pointers.json), matching the
/// existing kernel-test convention.
fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn producer_binary(root: &Path, producer_bin: Option<&str>) -> Result<PathBuf, String> {
    let Some(bin) = producer_bin else {
        return Err(
            "FAIL-CLOSED: missing OYA_CI_PRODUCER_BIN; Cargo fallback is forbidden".to_owned(),
        );
    };
    Ok(if Path::new(bin).is_absolute() {
        PathBuf::from(bin)
    } else {
        root.join(bin)
    })
}

#[test]
fn producer_binary_env_is_required_for_hermetic_gate() {
    let err = producer_binary(Path::new("/repo"), None)
        .expect_err("missing OYA_CI_PRODUCER_BIN must fail closed");
    assert!(err.contains("OYA_CI_PRODUCER_BIN"));
}

fn fixture_dir() -> PathBuf {
    repo_root().join("specs/fixtures/cross-artifact-agreement")
}

fn load_json(path: &PathBuf) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn expected_violations(fixture: &Value) -> BTreeSet<String> {
    fixture["expected_violations"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn architecture_rule_set(architecture: &Value, context: &str) -> BTreeSet<String> {
    if let Some(required_rules) = architecture.get("required_rules") {
        return string_array_set(required_rules, &format!("{context}.required_rules"));
    }

    let mut rules = BTreeSet::new();
    for key in [
        "required_microservice_rules",
        "required_clean_architecture_rules",
        "required_api_first_rules",
        "required_hyperscaler_pattern_rules",
    ] {
        if let Some(values) = architecture.get(key) {
            rules.extend(string_array_set(values, &format!("{context}.{key}")));
        }
    }
    assert!(
        !rules.is_empty(),
        "{context} must define required_rules or split required_*_rules arrays"
    );
    rules
}

fn string_array_set(value: &Value, context: &str) -> BTreeSet<String> {
    value
        .as_array()
        .unwrap_or_else(|| panic!("{context} must be an array"))
        .iter()
        .map(|item| {
            item.as_str()
                .unwrap_or_else(|| panic!("{context} contains a non-string item"))
                .to_owned()
        })
        .collect()
}

fn missing_from<'a>(expected: &'a BTreeSet<String>, actual: &'a BTreeSet<String>) -> Vec<&'a str> {
    expected
        .difference(actual)
        .map(String::as_str)
        .collect::<Vec<_>>()
}

fn assert_same_rule_set(
    left: &BTreeSet<String>,
    right: &BTreeSet<String>,
    left_name: &str,
    right_name: &str,
) {
    assert_eq!(
        left,
        right,
        "{left_name} and {right_name} architecture rule sets drifted; missing_from_{left_name}={:?}; missing_from_{right_name}={:?}",
        missing_from(right, left),
        missing_from(left, right)
    );
}

#[test]
fn cross_artifact_fixtures_execute_red_green_cases() {
    let dir = fixture_dir();
    let mut tc_paths: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("tc-") && n.ends_with(".json"))
        })
        .collect();
    tc_paths.sort();
    assert!(
        !tc_paths.is_empty(),
        "cross-artifact-agreement fixture corpus must not be empty"
    );

    let mut seen_green = false;
    let mut seen_red = false;

    for path in &tc_paths {
        let fixture = load_json(path);
        let report = evaluate(&fixture);
        let expected = expected_violations(&fixture);
        let label = path.file_name().unwrap().to_string_lossy().to_string();

        match fixture["expected_verdict"].as_str() {
            Some("GREEN") => {
                seen_green = true;
                assert_eq!(
                    report.verdict,
                    Verdict::Green,
                    "{label} should be GREEN, got violations {:?}",
                    report.violations
                );
                assert!(
                    report.violations.is_empty(),
                    "{label} GREEN must have zero violations, got {:?}",
                    report.violations
                );
            }
            Some("RED") => {
                seen_red = true;
                assert_eq!(report.verdict, Verdict::Red, "{label} should be RED");
                assert_eq!(report.violations, expected, "{label} violations mismatch");
            }
            other => panic!("{label} has unsupported expected_verdict {other:?}"),
        }
    }

    assert!(
        seen_green && seen_red,
        "cross-artifact-agreement fixtures must include BOTH RED and GREEN cases"
    );
}

#[test]
fn masterplan_v2_live_authority_contract_is_green() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let findings = evaluate_masterplan_v2_authority(&masterplan);
    assert!(
        findings.is_empty(),
        "masterplan v2 authority contract must stay green: {findings:?}"
    );
}

/// Sub-AC 4.1 masterplan structural gate: the frozen fixture corpus must keep one
/// ISOLATED fail-closed RED fixture per structural failure class — duplicate
/// work-item ids, dependency cycles, and dangling (orphan) dependency references.
/// The generic runner above only demands "some RED fixture exists"; this test pins
/// each named failure mode to its exact violation set so none can be silently
/// dropped or diluted.
#[test]
fn masterplan_structural_failure_mode_fixtures_fail_closed() {
    let cases: [(&str, &[&str]); 3] = [
        (
            "tc-XA-bad-masterplan-duplicate-work-item-id.json",
            &["masterplan_work_item_id_collision"],
        ),
        (
            "tc-XA-bad-masterplan-dependency-cycle.json",
            &[
                "masterplan_dependency_dag_invalid",
                "masterplan_sequencing_invalid",
            ],
        ),
        (
            "tc-XA-bad-masterplan-dangling-dependency-ref.json",
            &["masterplan_dependency_dag_invalid"],
        ),
    ];

    for (fixture_name, expected_codes) in cases {
        let path = fixture_dir().join(fixture_name);
        assert!(
            path.is_file(),
            "structural failure-mode fixture must exist: {}",
            path.display()
        );
        let fixture = load_json(&path);
        let report = evaluate(&fixture);
        assert_eq!(
            report.verdict,
            Verdict::Red,
            "{fixture_name} must fail closed (RED)"
        );
        let expected: BTreeSet<String> = expected_codes
            .iter()
            .map(|code| (*code).to_owned())
            .collect();
        assert_eq!(
            report.violations, expected,
            "{fixture_name} must emit exactly the pinned structural violation set"
        );
        assert_eq!(
            expected_violations(&fixture),
            expected,
            "{fixture_name} expected_violations must stay in sync with the pinned set"
        );
    }
}
#[test]
fn masterplan_v2_external_completion_claims_are_unverified_until_evidence_attaches() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let findings = evaluate_masterplan_v2_evidence_state(&masterplan);
    assert!(
        findings.is_empty(),
        "masterplan v2 evidence-state policy must keep unverified external completion claims out of done: {findings:?}"
    );

    if let Some(imports) =
        masterplan["masterplan_v2"]["external_work_item_claim_imports"].as_array()
    {
        for claim in imports {
            let refs = claim["evidence_refs"].as_array().unwrap_or_else(|| {
                panic!("each external completion claim must carry an evidence_refs array")
            });
            if refs.is_empty() {
                assert_eq!(
                    claim["masterplan_status"].as_str(),
                    Some("claimed-done-unverified")
                );
                assert_eq!(
                    claim["evidence_state"].as_str(),
                    Some("claimed-done-unverified")
                );
            }
        }
    }
}

/// Sub-AC 3 verifiability clause over the optional provider-neutral import:
/// absence is clean, malformed presence fails closed, and imported completion
/// claims cannot carry verified status without recorded evidence.
#[test]
fn masterplan_v2_external_completion_claim_import_is_optional_and_fail_closed() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let mut absent = masterplan.clone();
    absent["masterplan_v2"]
        .as_object_mut()
        .expect("masterplan_v2 must be an object")
        .remove("external_work_item_claim_imports");
    assert!(evaluate_masterplan_v2_evidence_state(&absent).is_empty());

    let mut malformed = absent;
    malformed["masterplan_v2"]["external_work_item_claim_imports"] =
        serde_json::json!("not-an-array");
    let findings = evaluate_masterplan_v2_evidence_state(&malformed);
    assert!(findings.iter().any(|finding| {
        finding.code == "masterplan_evidence_state_invalid"
            && finding.key == "masterplan_v2.external_work_item_claim_imports"
    }));
}
#[test]
fn masterplan_v2_plan_vs_evidence_drift_contract_is_green() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let findings = evaluate_masterplan_v2_plan_evidence_drift(&masterplan);
    assert!(
        findings.is_empty(),
        "masterplan v2 plan-vs-evidence drift policy must stay green: {findings:?}"
    );

    assert_eq!(
        masterplan["masterplan_v2"]["evidence_state_policy"]["validator"].as_str(),
        Some("cloud-ci-cross-artifact-agreement/masterplan-v2-plan-vs-evidence-drift"),
        "masterplan v2 must name the plan-vs-evidence drift validator as the evidence-state policy writer"
    );
}

/// Sub-AC 4.3 plan-vs-evidence cross-check lane, born-blocking over the live
/// tree: every masterplan work-item status claim and evidence-attached external
/// import must cross-check against RECORDED completion evidence. The
/// resolution universe is the committed scm-facts face `tracked_paths` (the
/// same declared input the producer reads), so a dangling evidence pointer, a
/// ref at a retired (absorbed / archived-with-provenance) surface, or a
/// verified 'done' claim without a merged commit / merged-PR record /
/// tracked product-completion packet anywhere in the live masterplan turns
/// this test RED.
#[test]
fn masterplan_plan_evidence_crosscheck_gate_is_green_on_live_tree() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let scm_facts =
        load_json(&root.join("ci/facade/artifact-inventory-registry/scm-facts.generated.json"));
    let tracked_paths = scm_facts
        .get("tracked_paths")
        .cloned()
        .expect("committed scm-facts face must carry tracked_paths");
    let corpus = serde_json::json!({ "tracked_paths": tracked_paths });

    let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &corpus);
    assert!(
        findings.is_empty(),
        "masterplan v2 plan-vs-evidence cross-check must stay green on the live tree: {findings:?}"
    );

    let crosscheck =
        &masterplan["masterplan_v2"]["evidence_state_policy"]["plan_evidence_crosscheck"];
    assert_eq!(
        crosscheck["validator"].as_str(),
        Some("cloud-ci-cross-artifact-agreement/masterplan-v2-plan-evidence-crosscheck"),
        "masterplan v2 must declare the plan-evidence cross-check validator"
    );
    assert_eq!(
        crosscheck["violation_code"].as_str(),
        Some("masterplan_plan_evidence_unrecorded"),
        "masterplan v2 must pin the cross-check violation code"
    );
    assert_eq!(
        crosscheck["resolution_universe"].as_str(),
        Some("ci/facade/artifact-inventory-registry/scm-facts.generated.json#tracked_paths"),
        "masterplan v2 must pin the tracked-tree resolution universe this test reads"
    );
}

/// Sub-AC 4.3 fail-closed pins: the frozen fixture corpus must keep one
/// ISOLATED RED fixture per plan-vs-evidence cross-check failure class — an
/// unevidenced verified-'done' claim, evidence pointing at a retired surface,
/// and a dangling evidence pointer — each emitting exactly
/// `masterplan_plan_evidence_unrecorded`.
#[test]
fn masterplan_plan_evidence_crosscheck_fixtures_fail_closed() {
    for fixture_name in [
        "tc-XA-bad-masterplan-evidence-unrecorded-done-claim.json",
        "tc-XA-bad-masterplan-evidence-retired-surface.json",
        "tc-XA-bad-masterplan-evidence-dangling-ref.json",
    ] {
        let path = fixture_dir().join(fixture_name);
        assert!(
            path.is_file(),
            "plan-evidence cross-check failure-mode fixture must exist: {}",
            path.display()
        );
        let fixture = load_json(&path);
        let report = evaluate(&fixture);
        assert_eq!(
            report.verdict,
            Verdict::Red,
            "{fixture_name} must fail closed (RED)"
        );
        let expected: BTreeSet<String> =
            std::iter::once("masterplan_plan_evidence_unrecorded".to_owned()).collect();
        assert_eq!(
            report.violations, expected,
            "{fixture_name} must emit exactly the unrecorded-evidence violation"
        );
        assert_eq!(
            expected_violations(&fixture),
            expected,
            "{fixture_name} expected_violations must stay in sync with the pinned set"
        );
    }
}
#[test]
fn masterplan_v2_program_coverage_contract_is_green() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let manifest_index = load_json(&root.join("specs/microservices/manifests-index.json"));
    let findings = evaluate_masterplan_v2_program_coverage(&masterplan, &manifest_index);
    assert!(
        findings.is_empty(),
        "masterplan v2 program coverage must cover every manifest-index microservice: {findings:?}"
    );

    // Machine-checked coverage audit: exact set equality between the manifest
    // index enumerated at consolidation time and the program-sharded coverage
    // rows — no enumerated surface may be missing and no phantom row may exist.
    let coverage = &masterplan["masterplan_v2"]["program_coverage"];
    let covered: BTreeSet<&str> = coverage["microservices"]
        .as_array()
        .expect("program_coverage.microservices must be an array")
        .iter()
        .filter_map(|entry| entry["microservice"].as_str())
        .collect();
    let indexed: BTreeSet<&str> = manifest_index["microservices"]
        .as_array()
        .expect("manifest index microservices must be an array")
        .iter()
        .filter_map(|entry| entry["name"].as_str())
        .collect();
    assert_eq!(
        covered, indexed,
        "program coverage must be exact set coverage over /specs/microservices/manifests-index.json at consolidation time"
    );

    // The ADR-0537 owned-stack ladder must be covered rung-for-rung in order.
    let rung_layers: Vec<&str> = coverage["owned_stack_ladder"]["rungs"]
        .as_array()
        .expect("program_coverage.owned_stack_ladder.rungs must be an array")
        .iter()
        .filter_map(|rung| rung["layer"].as_str())
        .collect();
    assert_eq!(
        rung_layers,
        [
            "cloud-kernel",
            "cloud-os",
            "cloud-k8s",
            "cloud-services",
            "products"
        ],
        "owned-stack ladder coverage must enumerate every ADR-0537 rung in ladder order"
    );

    // Pillar and program shards the consolidation must explicitly carry.
    let program_ids: BTreeSet<&str> = masterplan["masterplan_v2"]["programs"]
        .as_array()
        .expect("masterplan_v2.programs must be an array")
        .iter()
        .filter_map(|program| program["id"].as_str())
        .collect();
    for required in [
        "P-FD001-PRODUCT-SURFACES",
        "P-ONTOLOGY",
        "P-WORKFLOW-ENGINE",
        "P-WORKFLOW-STUDIO",
        "P-INTELLIGENCE",
        "P-OWNED-STACK-KERNEL",
        "P-OWNED-STACK-OS",
        "P-OWNED-STACK-K8S",
        "P-OWNED-STACK-CLOUD",
        "P-OWNED-STACK-DURABILITY",
        "P-OWNED-STACK-GOVERNANCE-IAM-CONSOLE",
        "P-REORG",
        "P-AST-CODE-GRAPH",
        "P-FABRIC",
    ] {
        assert!(
            program_ids.contains(required),
            "missing required program shard {required}"
        );
    }
}

#[test]
fn masterplan_v2_projection_freshness_contract_is_green() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let generated_artifacts =
        load_json(&root.join("registry/generated-artifact-control-plane.json"));
    let findings =
        evaluate_masterplan_v2_projection_freshness(&masterplan, Some(&generated_artifacts));
    assert!(
        findings.is_empty(),
        "masterplan v2 projection freshness must cover every generated/read projection: {findings:?}"
    );

    let projections = masterplan["masterplan_v2"]["projection_freshness"]["projections"]
        .as_array()
        .expect("projection_freshness.projections must be an array");
    let covered_paths: BTreeSet<&str> = projections
        .iter()
        .filter_map(|projection| projection["path"].as_str())
        .collect();
    let expected_paths = expected_masterplan_projection_paths(&masterplan, &generated_artifacts);
    assert_eq!(
        covered_paths, expected_paths,
        "projection_freshness.projections must be exact set coverage over every generated/read projection derived from specs/masterplan.json"
    );
}
#[test]
fn masterplan_v2_read_contract_archive_gate_is_green() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let findings = evaluate_masterplan_v2_read_contract_archives(&masterplan);
    assert!(
        findings.is_empty(),
        "archived stale read paths must only be referenced as provenance archives: {findings:?}"
    );
}

#[test]
fn masterplan_v2_entry_surface_allowlist_gate_is_green() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let root_hub = load_json(&root.join("specs/root-hub-pointers.json"));
    let findings = evaluate_masterplan_v2_entry_surfaces(&masterplan, &root_hub);
    assert!(
        findings.is_empty(),
        "entry-surface read contracts must exactly match the bounded root-hub allowlist and exclude superseded entrypoints: {findings:?}"
    );
}

/// The founder product-intent contract is an entry boundary, not a promotion path.
/// Its structural agreement checker must reject both accidental authorization and
/// cross-artifact registration drift before a live gate can mistake the artifact for
/// a satisfied Stage-1 control.
#[test]
fn founder_product_intent_validator_is_fail_closed_for_authorization_and_control_drift() {
    let root = repo_root();
    let intent = load_json(&root.join("specs/founder-product-intent.json"));
    let root_hub = load_json(&root.join("specs/root-hub-pointers.json"));
    let registry = load_json(&root.join("registry/artifact-capabilities-registry.json"));
    let graph = load_json(&root.join("registry/graph/active-artifact-contract-edges.json"));

    let mut unsafe_intent = intent.clone();
    unsafe_intent["authority_boundary"]["dispatch_allowed"] = Value::Bool(true);
    unsafe_intent["stage1_entry_requirements"]["controls"][0]["id"] =
        Value::String("renamed-control".to_owned());
    unsafe_intent["stage1_entry_requirements"]["controls"][0]["decision"] =
        Value::String("satisfied".to_owned());

    let findings =
        evaluate_founder_product_intent_agreement(&unsafe_intent, &root_hub, &registry, &graph);
    assert!(
        findings
            .iter()
            .any(|finding| finding.key == "authority_boundary.dispatch_allowed"),
        "dispatch authorization must be rejected: {findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.key == "stage1_entry_requirements.controls"),
        "Stage-1 control IDs must be exact: {findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.key == "stage1_entry_requirements.controls.nonclaim"),
        "the intent artifact must never record a satisfied Stage-1 control: {findings:?}"
    );

    let mut hidden_root_hub = root_hub.clone();
    hidden_root_hub["agent_entry_surface_allowlist"]["paths"]
        .as_array_mut()
        .expect("entry-surface paths array")
        .retain(|path| path.as_str() != Some("/specs/founder-product-intent.json"));
    let findings =
        evaluate_founder_product_intent_agreement(&intent, &hidden_root_hub, &registry, &graph);
    assert!(
        findings.iter().any(|finding| {
            finding.key == "root_hub.agent_entry_surface_allowlist.founder_product_intent"
        }),
        "the current founder intent must remain a mandatory bounded entry surface: {findings:?}"
    );

    let mut schema_misclassification = registry.clone();
    schema_misclassification["rows"]
        .as_array_mut()
        .expect("artifact capability rows")
        .iter_mut()
        .find(|row| row["artifact_id"].as_str() == Some("founder-product-intent"))
        .expect("founder product intent registry row")["artifact_profile"] =
        Value::String("schema".to_owned());
    let findings = evaluate_founder_product_intent_agreement(
        &intent,
        &root_hub,
        &schema_misclassification,
        &graph,
    );
    assert!(
        findings.iter().any(|finding| {
            finding.key == "artifact_capabilities_registry.founder-product-intent"
        }),
        "a normative founder intent must not be misclassified as a JSON Schema: {findings:?}"
    );

    let mut schema_declaration = graph.clone();
    schema_declaration["edges"]
        .as_array_mut()
        .expect("active artifact contract edges")
        .iter_mut()
        .find(|edge| edge["source"].as_str() == Some("founder-product-intent"))
        .expect("founder product intent graph edge")["target"] = Value::String("schema".to_owned());
    let findings = evaluate_founder_product_intent_agreement(
        &intent,
        &root_hub,
        &registry,
        &schema_declaration,
    );
    assert!(
        findings.iter().any(|finding| {
            finding.key == "active_artifact_contract_edges.founder-product-intent"
        }),
        "the graph must classify the normative founder intent as a spec: {findings:?}"
    );

    let mut incomplete_projection = graph.clone();
    incomplete_projection["edges"]
        .as_array_mut()
        .expect("active artifact contract edges")
        .remove(0);
    let findings = evaluate_founder_product_intent_agreement(
        &intent,
        &root_hub,
        &registry,
        &incomplete_projection,
    );
    assert!(
        findings.iter().any(|finding| {
            finding.key == "active_artifact_contract_edges.complete_registry_projection"
        }),
        "the graph must fail closed when any registry-row projection edge is missing: {findings:?}"
    );

    let mut readable_archive_resurrection = root_hub.clone();
    readable_archive_resurrection["entry_points"]["agent_durable_goal"]["retired_archive_manifest_path"] =
        Value::String(
            ".omc/archive/stale-documents/2026-05-19-planning-closure/manifest.json".to_owned(),
        );
    let findings = evaluate_founder_product_intent_agreement(
        &intent,
        &readable_archive_resurrection,
        &registry,
        &graph,
    );
    assert!(
        findings.iter().any(|finding| {
            finding.key == "root_hub.entry_points.agent_durable_goal.history_only_provenance"
        }),
        "retired goal provenance must not resurrect a readable archive path: {findings:?}"
    );
}

/// The product intent must keep temporal states and epistemic artifact types as
/// executable structure, not prose that can silently drift after review.
#[test]
fn founder_product_intent_validator_is_fail_closed_for_temporal_and_epistemic_drift() {
    let root = repo_root();
    let intent = load_json(&root.join("specs/founder-product-intent.json"));
    let root_hub = load_json(&root.join("specs/root-hub-pointers.json"));
    let registry = load_json(&root.join("registry/artifact-capabilities-registry.json"));
    let graph = load_json(&root.join("registry/graph/active-artifact-contract-edges.json"));

    let cases: [(&str, &str, JsonMutation); 6] = [
        (
            "duplicate-knowledge-class-mapping",
            "temporal_and_epistemic_contract.typed_artifacts.knowledge_class",
            Box::new(|candidate| {
                let first_class = candidate["temporal_and_epistemic_contract"]["typed_artifacts"]
                    [0]["knowledge_class"]
                    .clone();
                candidate["temporal_and_epistemic_contract"]["typed_artifacts"][1]["knowledge_class"] =
                    first_class;
            }),
        ),
        (
            "empty-typed-artifact-minimum",
            "temporal_and_epistemic_contract.typed_artifacts.minimum",
            Box::new(|candidate| {
                candidate["temporal_and_epistemic_contract"]["typed_artifacts"][0]["minimum"] =
                    Value::String(String::new());
            }),
        ),
        (
            "blank-typed-artifact-minimum",
            "temporal_and_epistemic_contract.typed_artifacts.minimum",
            Box::new(|candidate| {
                candidate["temporal_and_epistemic_contract"]["typed_artifacts"][0]["minimum"] =
                    Value::String(" \t\n".to_owned());
            }),
        ),
        (
            "time-state-drift",
            "temporal_and_epistemic_contract.time_states",
            Box::new(|candidate| {
                candidate["temporal_and_epistemic_contract"]["time_states"][0]["id"] =
                    Value::String("timeless".to_owned());
            }),
        ),
        (
            "blank-time-state-rule",
            "temporal_and_epistemic_contract.time_states.rule",
            Box::new(|candidate| {
                candidate["temporal_and_epistemic_contract"]["time_states"][0]["rule"] =
                    Value::String(" \t\n".to_owned());
            }),
        ),
        (
            "artifact-class-pair-swap",
            "temporal_and_epistemic_contract.typed_artifacts.mapping",
            Box::new(|candidate| {
                let first_class = candidate["temporal_and_epistemic_contract"]["typed_artifacts"]
                    [0]["knowledge_class"]
                    .clone();
                let second_class = candidate["temporal_and_epistemic_contract"]["typed_artifacts"]
                    [1]["knowledge_class"]
                    .clone();
                candidate["temporal_and_epistemic_contract"]["typed_artifacts"][0]["knowledge_class"] =
                    second_class;
                candidate["temporal_and_epistemic_contract"]["typed_artifacts"][1]["knowledge_class"] =
                    first_class;
            }),
        ),
    ];

    for (name, expected_key, mutate) in cases {
        let mut candidate = intent.clone();
        mutate(&mut candidate);
        let findings =
            evaluate_founder_product_intent_agreement(&candidate, &root_hub, &registry, &graph);
        assert!(
            findings.iter().any(|finding| finding.key == expected_key),
            "{name} must fail closed with {expected_key}: {findings:?}"
        );
    }
}

/// The operational-world vocabulary is future-facing product intent, not a backdoor
/// amendment of Accepted governance or a route around Stage-1 comparator controls.
/// These mutations are deliberately data-only: the validator must reject each one
/// without relying on a reviewer to reinterpret prose at admission time.
#[test]
fn founder_product_intent_validator_is_fail_closed_for_operational_world_type_and_comparator_drift()
{
    let root = repo_root();
    let intent = load_json(&root.join("specs/founder-product-intent.json"));
    let root_hub = load_json(&root.join("specs/root-hub-pointers.json"));
    let registry = load_json(&root.join("registry/artifact-capabilities-registry.json"));
    let graph = load_json(&root.join("registry/graph/active-artifact-contract-edges.json"));

    let cases: [(&str, &str, JsonMutation); 8] = [
        (
            "game-engine-model-removed",
            "game_engine_product_model",
            Box::new(|candidate| {
                candidate
                    .as_object_mut()
                    .expect("founder intent object")
                    .remove("game_engine_product_model");
            }),
        ),
        (
            "change-accounting-removed",
            "change_accounting",
            Box::new(|candidate| {
                candidate
                    .as_object_mut()
                    .expect("founder intent object")
                    .remove("change_accounting");
            }),
        ),
        (
            "future-types-rewrite-accepted-authority",
            "game_engine_product_model.governance_type_system_boundary",
            Box::new(|candidate| {
                candidate["game_engine_product_model"]["governance_type_system_boundary"] =
                    Value::String(
                        "Operational-world types replace the Accepted type_system.".to_owned(),
                    );
            }),
        ),
        (
            "future-types-completeness-erased",
            "game_engine_product_model.type_contract.future_accepted_successor_requirements",
            Box::new(|candidate| {
                candidate["game_engine_product_model"]["type_contract"]
                    .as_object_mut()
                    .expect("operational-world type contract")
                    .remove("future_accepted_successor_requirements");
            }),
        ),
        (
            "comparator-precedes-legal-jcr",
            "benchmark_and_measurement_contract.comparator_admission",
            Box::new(|candidate| {
                candidate["benchmark_and_measurement_contract"]["comparator_admission"]["claim_allowed"] =
                    Value::Bool(true);
            }),
        ),
        (
            "harvested-palantir-made-evidence",
            "founder_execution_authorization.pipeline_evolution_contract.research_basis.sources[palantir-apollo-helm-rollouts].evidence_eligible",
            Box::new(|candidate| {
                let source = candidate["founder_execution_authorization"]
                    ["pipeline_evolution_contract"]["research_basis"]["sources"]
                    .as_array_mut()
                    .expect("research sources")
                    .iter_mut()
                    .find(|source| source["source_id"].as_str() == Some("palantir-apollo-helm-rollouts"))
                    .expect("Palantir harvested source");
                source["evidence_eligible"] = Value::Bool(true);
            }),
        ),
        (
            "uncollected-engine-pointer-made-evidence",
            "benchmark_and_measurement_contract.game_engine_comparator_refs[unity-entities-concepts].evidence_eligible",
            Box::new(|candidate| {
                candidate["benchmark_and_measurement_contract"]["game_engine_comparator_refs"]
                    .as_array_mut()
                    .expect("game-engine comparator references")
                    .iter_mut()
                    .find(|source| source["source_id"].as_str() == Some("unity-entities-concepts"))
                    .expect("Unity pointer")["evidence_eligible"] = Value::Bool(true);
            }),
        ),
        (
            "hold-planning-erased",
            "authority_boundary.planning_state",
            Box::new(|candidate| {
                candidate["authority_boundary"]["planning_state"] =
                    Value::String("PASS(Planning)".to_owned());
            }),
        ),
    ];

    for (name, expected_key, mutate) in cases {
        let mut candidate = intent.clone();
        mutate(&mut candidate);
        let findings =
            evaluate_founder_product_intent_agreement(&candidate, &root_hub, &registry, &graph);
        assert!(
            findings.iter().any(|finding| finding.key == expected_key),
            "{name} must fail closed with {expected_key}: {findings:?}"
        );
    }
}

/// Stage-1 dependency order and every founder-intent nonclaim boundary are closed
/// structures. Required prose may be retained for people, but it can never carry
/// the permissive semantics that the machine gate must decide.
#[test]
fn founder_product_intent_validator_is_fail_closed_for_stage1_dependencies_and_contradictory_exceptions()
 {
    let root = repo_root();
    let intent = load_json(&root.join("specs/founder-product-intent.json"));
    let root_hub = load_json(&root.join("specs/root-hub-pointers.json"));
    let registry = load_json(&root.join("registry/artifact-capabilities-registry.json"));
    let graph = load_json(&root.join("registry/graph/active-artifact-contract-edges.json"));

    let cases: [(&str, &str, JsonMutation); 8] = [
        (
            "missing-comparator-dependency",
            "stage1_entry_requirements.controls[comparator].depends_on",
            Box::new(|candidate| {
                candidate["stage1_entry_requirements"]["controls"]
                    .as_array_mut()
                    .expect("Stage-1 controls")
                    .iter_mut()
                    .find(|control| control["id"].as_str() == Some("comparator"))
                    .expect("comparator control")
                    .as_object_mut()
                    .expect("comparator control object")
                    .remove("depends_on");
            }),
        ),
        (
            "reversed-legal-comparator-dependency",
            "stage1_entry_requirements.controls.dependencies.order",
            Box::new(|candidate| {
                let controls = candidate["stage1_entry_requirements"]["controls"]
                    .as_array_mut()
                    .expect("Stage-1 controls");
                let comparator = controls
                    .iter_mut()
                    .find(|control| control["id"].as_str() == Some("comparator"))
                    .expect("comparator control");
                comparator["depends_on"] = serde_json::json!(["legal_jcr"]);
                let legal = controls
                    .iter_mut()
                    .find(|control| control["id"].as_str() == Some("legal_jcr"))
                    .expect("legal/JCR control");
                legal["depends_on"] = serde_json::json!(["comparator"]);
            }),
        ),
        (
            "unknown-dependency",
            "stage1_entry_requirements.controls.dependencies.known_ids",
            Box::new(|candidate| {
                candidate["stage1_entry_requirements"]["controls"]
                    .as_array_mut()
                    .expect("Stage-1 controls")[0]["depends_on"] =
                    serde_json::json!(["invented-control"]);
            }),
        ),
        (
            "cyclic-dependencies",
            "stage1_entry_requirements.controls.dependencies.cycle",
            Box::new(|candidate| {
                let controls = candidate["stage1_entry_requirements"]["controls"]
                    .as_array_mut()
                    .expect("Stage-1 controls");
                controls[0]["depends_on"] = serde_json::json!(["decision_parser_ir"]);
                controls[1]["depends_on"] = serde_json::json!(["adr_chronology"]);
            }),
        ),
        (
            "type-boundary-exception-appended",
            "game_engine_product_model.governance_type_system_boundary",
            Box::new(|candidate| {
                candidate["game_engine_product_model"]["governance_type_system_boundary"]["exception"] =
                    Value::String("may amend Accepted authority".to_owned());
            }),
        ),
        (
            "comparator-rule-exception-appended",
            "benchmark_and_measurement_contract.comparator_admission",
            Box::new(|candidate| {
                candidate["benchmark_and_measurement_contract"]["comparator_admission"]["exception"] =
                    Value::String("C05 may bypass C06".to_owned());
            }),
        ),
        (
            "research-quarantine-exception-appended",
            "founder_execution_authorization.pipeline_evolution_contract.research_basis.sources[palantir-apollo-helm-rollouts].quarantine",
            Box::new(|candidate| {
                let source = candidate["founder_execution_authorization"]
                    ["pipeline_evolution_contract"]["research_basis"]["sources"]
                    .as_array_mut()
                    .expect("research sources")
                    .iter_mut()
                    .find(|source| source["source_id"].as_str() == Some("palantir-apollo-helm-rollouts"))
                    .expect("Palantir source");
                source["quarantine"]["exception"] = Value::Bool(true);
            }),
        ),
        (
            "change-accounting-exception-appended",
            "change_accounting[0]",
            Box::new(|candidate| {
                candidate["change_accounting"][0]["exception"] =
                    Value::String("dispatch may proceed".to_owned());
            }),
        ),
    ];

    for (name, expected_key, mutate) in cases {
        let mut candidate = intent.clone();
        mutate(&mut candidate);
        let findings =
            evaluate_founder_product_intent_agreement(&candidate, &root_hub, &registry, &graph);
        assert!(
            findings.iter().any(|finding| finding.key == expected_key),
            "{name} must fail closed with {expected_key}: {findings:?}"
        );
    }
}

/// Pipeline evolution is itself a safety-critical contract. These isolated RED
/// mutations pin the structural controls that keep parallel work from turning
/// into self-authorizing promotion or a weaker safety tier.
#[test]
fn founder_product_intent_validator_is_fail_closed_for_pipeline_contract_drift() {
    let root = repo_root();
    let intent = load_json(&root.join("specs/founder-product-intent.json"));
    let root_hub = load_json(&root.join("specs/root-hub-pointers.json"));
    let registry = load_json(&root.join("registry/artifact-capabilities-registry.json"));
    let graph = load_json(&root.join("registry/graph/active-artifact-contract-edges.json"));

    let cases: [(&str, JsonMutation); 27] = [
        (
            "missing-work-graph-contract",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]
                    .as_object_mut()
                    .expect("pipeline evolution contract object")
                    .remove("work_graph_contract");
            }),
        ),
        (
            "work-graph-state-drift",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]["work_graph_contract"]
                    ["states"][0] = Value::String("self-promoting".to_owned());
            }),
        ),
        (
            "promotion-state-drift",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]["promotion_state_machine"]
                    [0] = Value::String("auto-promoted".to_owned());
            }),
        ),
        (
            "automation-tier-drift",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]
                    ["automation_safety_governor"]
                    .as_object_mut()
                    .expect("automation safety governor object")
                    .remove("GATE");
            }),
        ),
        (
            "empty-safety-classification",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]["automation_safety_governor"]
                    ["classification_rule"] = Value::String(String::new());
            }),
        ),
        (
            "candidate-control-dual-evaluation-erased",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]["trusted_control_rule"] =
                    Value::String("candidate controls itself".to_owned());
            }),
        ),
        (
            "merge-runtime-separation-erased",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]["merge_and_release_separation"] =
                    Value::String("merge is runtime promotion".to_owned());
            }),
        ),
        (
            "proposed-artifact-self-promotion",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]["authority_rule"] =
                    Value::String(
                        "proposed implementation becomes authority through use".to_owned(),
                    );
            }),
        ),
        (
            "hold-pass-promotion-barrier-erased",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["concurrency_pipeline"]["promotion_barrier"] =
                    Value::String("any candidate may dispatch and merge immediately".to_owned());
            }),
        ),
        (
            "implementation-claim-ceiling-erased",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]
                    .as_object_mut()
                    .expect("pipeline evolution contract object")
                    .remove("implementation_claim_ceiling");
            }),
        ),
        (
            "generated-face-lifecycle-erased",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]["generated_artifact_rule"] =
                    Value::String("generated output is editable cache data".to_owned());
            }),
        ),
        (
            "demand-driven-execution-bypass-inserted",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]
                    ["demand_driven_execution_rule"] = Value::String(
                    "every requested face may evaluate the full universe and ignore unknown dependencies"
                        .to_owned(),
                );
            }),
        ),
        (
            "node-face-resource-evidence-erased",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]
                    .as_object_mut()
                    .expect("pipeline evolution contract object")
                    .remove("node_face_resource_evidence_contract");
            }),
        ),
        (
            "auto-merge-bypass-inserted",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]["automation_safety_governor"]
                    ["AUTO"] = Value::String("AUTO may merge and bypass admission".to_owned());
            }),
        ),
        (
            "tenant-isolation-erased",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]["tenant_pipeline_isolation_rule"] =
                    Value::String("tenants share pipeline credentials and evidence".to_owned());
            }),
        ),
        (
            "runtime-held-backfill-boundary-erased",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]["runtime_promotion_fail_closed_rule"] =
                    Value::String("backfilled evidence immediately promotes artifacts".to_owned());
            }),
        ),
        (
            "vulnerability-invalidation-becomes-live-claim",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]["continuous_vulnerability_invalidation_rule"] =
                    Value::String("the proposed pipeline is live authority".to_owned());
            }),
        ),
        (
            "protected-admission-exact-head-erased",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]["current_protected_admission_rule"] =
                    Value::String("old head may merge without review or CI".to_owned());
            }),
        ),
        (
            "post-merge-completion-overclaim",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]["post_merge_closure_rule"] =
                    Value::String("merge receipt is product completion".to_owned());
            }),
        ),
        (
            "resource-lineage-health-erased",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]
                    .as_object_mut()
                    .expect("pipeline evolution contract object")
                    .remove("resource_lineage_and_health_rule");
            }),
        ),
        (
            "probabilistic-evaluation-boundary-erased",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]
                    .as_object_mut()
                    .expect("pipeline evolution contract object")
                    .remove("probabilistic_evaluation_rule");
            }),
        ),
        (
            "research-evidence-self-authorizes",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]["research_basis"]
                    ["claim_boundary"] =
                    Value::String("external sources authorize the roadmap".to_owned());
            }),
        ),
        (
            "portability-cutover-equivalence-erased",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]["productization_and_portability_rule"] =
                    Value::String("one vendor runner is sufficient".to_owned());
            }),
        ),
        (
            "pipeline-migration-deletion-erased",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]["pipeline_migration_rule"] =
                    Value::String("keep every retired controller indefinitely".to_owned());
            }),
        ),
        (
            "research-source-scope-erased",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]
                    ["research_basis"]["sources"][0]
                    .as_object_mut()
                    .expect("research source object")
                    .remove("scope");
            }),
        ),
        (
            "research-source-id-collision",
            Box::new(|candidate| {
                let first_id = candidate["founder_execution_authorization"]
                    ["pipeline_evolution_contract"]["research_basis"]["sources"][0]
                    ["source_id"]
                    .clone();
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]["research_basis"]
                    ["sources"][1]["source_id"] = first_id;
            }),
        ),
        (
            "research-source-non-https",
            Box::new(|candidate| {
                candidate["founder_execution_authorization"]["pipeline_evolution_contract"]["research_basis"]
                    ["sources"][0]["url"] =
                    Value::String("file:///untrusted-local-reference".to_owned());
            }),
        ),
    ];

    for (name, mutate) in cases {
        let mut candidate = intent.clone();
        mutate(&mut candidate);
        let findings =
            evaluate_founder_product_intent_agreement(&candidate, &root_hub, &registry, &graph);
        assert!(
            !findings.is_empty(),
            "{name} must fail closed instead of weakening the pipeline contract"
        );
    }
}

/// Live GATE-1 integration: the four founder-intent faces must remain mutually
/// coherent while preserving HOLD(Planning) and non-dispatching control status.
#[test]
fn founder_product_intent_agreement_gate_is_green_on_live_corpus() {
    let root = repo_root();
    let findings = evaluate_founder_product_intent_agreement(
        &load_json(&root.join("specs/founder-product-intent.json")),
        &load_json(&root.join("specs/root-hub-pointers.json")),
        &load_json(&root.join("registry/artifact-capabilities-registry.json")),
        &load_json(&root.join("registry/graph/active-artifact-contract-edges.json")),
    );
    assert!(
        findings.is_empty(),
        "founder product intent, root hub, registry row, and graph edge must agree without \
         promoting Stage-1: {findings:?}"
    );
}

#[test]
fn artifact_profile_defaults_and_root_hub_have_exact_identifier_parity() {
    let root = repo_root();
    let defaults = load_json(&root.join("specs/artifact-profile-defaults.json"));
    let root_hub = load_json(&root.join("specs/root-hub-pointers.json"));
    let canonical_ids: BTreeSet<&str> = defaults["profiles"]
        .as_object()
        .expect("artifact profile defaults profiles object")
        .keys()
        .map(String::as_str)
        .collect();
    let pointer = &root_hub["entry_points"]["artifact_profile_defaults"];
    let pointer_ids: BTreeSet<&str> = pointer["canonical_profile_ids"]
        .as_array()
        .expect("root-hub artifact-profile canonical_profile_ids array")
        .iter()
        .map(|value| value.as_str().expect("canonical profile id string"))
        .collect();

    assert_eq!(
        pointer["canonical_profile_count"].as_u64(),
        Some(canonical_ids.len() as u64),
        "root-hub artifact-profile count must equal the canonical defaults"
    );
    assert_eq!(
        pointer_ids, canonical_ids,
        "root-hub artifact-profile identifiers must exactly match the canonical defaults"
    );
}

/// Sub-AC 4.4 fail-closed pins: the frozen fixture corpus must keep one
/// ISOLATED RED fixture per read-contract/entry-surface failure class — a
/// superseded plan authority resurrected/re-exposed outside the archive
/// (docs/ROADMAP.md with its archive markers stripped plus a non-archive
/// read-path reference), a superseded entrypoint revived into the mandatory
/// entry surface, and an entry surface unbounded beyond the root-hub
/// allowlist (docs/MASTERPLAN.md promoted) — each pinned to its exact
/// violation set so none can be silently dropped or diluted. A GREEN
/// companion fixture keeps the full lane exercisable end to end.
#[test]
fn masterplan_read_contract_entry_surface_fixtures_fail_closed() {
    let cases: [(&str, &[&str]); 3] = [
        (
            "tc-XA-bad-masterplan-read-contract-resurrected-roadmap.json",
            &["masterplan_read_contract_invalid"],
        ),
        (
            "tc-XA-bad-masterplan-entry-surface-resurrected-superseded.json",
            &["masterplan_entry_surface_invalid"],
        ),
        (
            "tc-XA-bad-masterplan-entry-surface-unbounded.json",
            &["masterplan_entry_surface_invalid"],
        ),
    ];

    for (fixture_name, expected_codes) in cases {
        let path = fixture_dir().join(fixture_name);
        assert!(
            path.is_file(),
            "read-contract/entry-surface failure-mode fixture must exist: {}",
            path.display()
        );
        let fixture = load_json(&path);
        let report = evaluate(&fixture);
        assert_eq!(
            report.verdict,
            Verdict::Red,
            "{fixture_name} must fail closed (RED)"
        );
        let expected: BTreeSet<String> = expected_codes
            .iter()
            .map(|code| (*code).to_owned())
            .collect();
        assert_eq!(
            report.violations, expected,
            "{fixture_name} must emit exactly the pinned read-contract/entry-surface violation set"
        );
        assert_eq!(
            expected_violations(&fixture),
            expected,
            "{fixture_name} expected_violations must stay in sync with the pinned set"
        );
    }

    let green = fixture_dir().join("tc-XA-good-masterplan-read-surface-archive-clean.json");
    assert!(
        green.is_file(),
        "read-contract/entry-surface GREEN fixture must exist: {}",
        green.display()
    );
    let report = evaluate(&load_json(&green));
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "the GREEN read-surface fixture must stay green, got {:?}",
        report.violations
    );
}

/// Sub-AC 4.4 resurrection sweep, born-blocking over the live tree: every
/// governed on-disk read surface (each `surface_dispositions` repo-file row
/// dispositioned absorbed / archived-with-provenance / generated-projection —
/// docs/MASTERPLAN.md, docs/ROADMAP.md, the retired planning specs, the
/// repo-local provenance stores) must still carry its archive markers on
/// disk. Stripping the archive front-matter from docs/ROADMAP.md, deleting
/// the absorbed status from a retired spec, or re-filling any superseded
/// authority with live-looking plan content turns this test RED. Tracked-tree
/// membership comes from the committed scm-facts face, the same declared
/// input the plan-evidence cross-check lane reads.
#[test]
fn masterplan_read_surface_resurrection_gate_is_green_on_live_tree() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let corpus = live_read_surface_corpus(&root, &masterplan);

    let findings = evaluate_masterplan_read_surface_resurrections(&masterplan, &corpus);
    assert!(
        findings.is_empty(),
        "superseded/stale plan authorities must stay archived on disk (no resurrection outside the archive): {findings:?}"
    );

    // The sweep must actually have surfaces to police: an empty corpus here
    // would mean the disposition ledger lost its governed read surfaces.
    let swept = corpus["surfaces"].as_array().expect("surfaces").len();
    assert!(
        swept >= 5,
        "live resurrection sweep must cover the governed read surfaces, swept only {swept}"
    );
}

/// Assemble the live read-surface corpus from the repo tree: one row per
/// governed `surface_dispositions` repo-file path, carrying tracked-tree
/// existence plus the on-disk facts (Markdown front-matter block, parsed
/// JSON document, or an opaque-data marker for non-document provenance
/// files).
fn live_read_surface_corpus(root: &Path, masterplan: &Value) -> Value {
    let scm_facts =
        load_json(&root.join("ci/facade/artifact-inventory-registry/scm-facts.generated.json"));
    let tracked: BTreeSet<&str> = scm_facts["tracked_paths"]
        .as_array()
        .expect("committed scm-facts face must carry tracked_paths")
        .iter()
        .filter_map(Value::as_str)
        .collect();

    let mut surfaces = Vec::new();
    let dispositions = masterplan["masterplan_v2"]["surface_dispositions"]
        .as_array()
        .expect("masterplan v2 must carry surface_dispositions");
    for surface in dispositions {
        let Some(disposition) = surface.get("disposition").and_then(Value::as_str) else {
            continue;
        };
        if !matches!(
            disposition,
            "absorbed" | "archived-with-provenance" | "generated-projection"
        ) {
            continue;
        }
        let Some(path) = surface.get("path").and_then(Value::as_str) else {
            continue;
        };
        if path.contains('#') || path.contains('*') || path.starts_with('~') {
            continue;
        }
        let rel_path = path.trim_start_matches('/');
        let on_disk = root.join(rel_path);
        let exists = tracked.contains(rel_path) && on_disk.is_file();
        let mut row = serde_json::json!({ "path": path, "exists": exists });
        if exists {
            if rel_path.ends_with(".md") {
                let content = fs::read_to_string(&on_disk).expect("read governed markdown surface");
                row["front_matter"] = Value::String(markdown_front_matter(&content));
            } else if rel_path.ends_with(".json") {
                row["document"] = load_json(&on_disk);
            } else {
                row["opaque_data"] = Value::Bool(true);
            }
        }
        surfaces.push(row);
    }

    serde_json::json!({ "surfaces": surfaces })
}

/// Extract the leading `---` front-matter block from a Markdown file; when a
/// file carries no front-matter fence, the (bounded) head of the file is the
/// scanned surface, so a marker-free live document still fails the sweep.
fn markdown_front_matter(content: &str) -> String {
    if let Some(rest) = content.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---")
    {
        return rest[..end].to_owned();
    }
    content.chars().take(4096).collect()
}

/// Sub-AC 4.2 mechanical re-derivation lane, born-blocking over the live tree:
/// every derived/generated masterplan projection that exists on disk must be
/// mechanically re-derivable from /specs/masterplan.json and byte-identical to
/// its re-derivation. The corpus is assembled from the ACTUAL tree (the human
/// projection, the fabric-loop flow-metrics ledger, the loop-card shard views,
/// and every on-disk generated planning face), so a stale or hand-edited
/// projection anywhere in the tree turns this test RED.
#[test]
fn masterplan_projection_rederivation_gate_is_green_on_live_tree() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let corpus = live_projection_rederivation_corpus(&root);

    // The lanes must be exercised, not vacuous: the live tree carries the
    // generated human projection, at least one recorded flow-metrics pass, and
    // at least one loop-card shard view.
    assert!(
        corpus["masterplan_md"]
            .as_str()
            .is_some_and(|md| !md.is_empty()),
        "docs/MASTERPLAN.md must exist as the generated human projection"
    );
    assert!(
        corpus["flow_metrics_passes"]
            .as_array()
            .is_some_and(|passes| !passes.is_empty()),
        "the flow-metrics ledger must carry at least one recorded pass"
    );
    assert!(
        corpus["loop_cards"]
            .as_array()
            .is_some_and(|cards| !cards.is_empty()),
        "the coordination plane must carry at least one loop-card shard view"
    );

    let findings = evaluate_masterplan_projection_rederivation(&masterplan, &corpus);
    assert!(
        findings.is_empty(),
        "every derived/generated masterplan projection must re-derive byte-identically from /specs/masterplan.json: {findings:?}"
    );

    // The derivation itself must reproduce the committed projection exactly.
    let derived = derive_masterplan_md_projection(&masterplan)
        .expect("docs/MASTERPLAN.md must be derivable from masterplan v2");
    let on_disk = fs::read_to_string(root.join("docs/MASTERPLAN.md")).expect("read MASTERPLAN.md");
    assert_eq!(
        derived, on_disk,
        "docs/MASTERPLAN.md must be byte-identical to its mechanical re-derivation"
    );
}

/// Assemble the live projection-rederivation corpus from the repo tree.
fn live_projection_rederivation_corpus(root: &Path) -> Value {
    let masterplan_md = fs::read_to_string(root.join("docs/MASTERPLAN.md")).unwrap_or_default();
    let flow_metrics_passes =
        read_projection_files(&root.join("plan/fabric-loop/flow-metrics/passes"), ".json");
    let loop_cards = read_projection_files(&root.join("plan/fabric-loop/cards"), ".json");
    let generated_projections_on_disk: Vec<Value> =
        list_file_names(&root.join("docs/machine-readable"), ".generated.json")
            .into_iter()
            .map(|name| Value::String(format!("docs/machine-readable/{name}")))
            .collect();
    let control_plane = load_json(&root.join("registry/generated-artifact-control-plane.json"));

    serde_json::json!({
        "masterplan_md": masterplan_md,
        "flow_metrics_passes": flow_metrics_passes,
        "loop_cards": loop_cards,
        "generated_projections_on_disk": generated_projections_on_disk,
        "generated_artifact_control_plane": control_plane,
    })
}

fn list_file_names(dir: &Path, suffix: &str) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| entry.file_name().to_str().map(str::to_owned))
        .filter(|name| name.ends_with(suffix))
        .collect();
    names.sort();
    names
}

fn read_projection_files(dir: &Path, suffix: &str) -> Vec<Value> {
    list_file_names(dir, suffix)
        .into_iter()
        .map(|name| {
            let content = fs::read_to_string(dir.join(&name))
                .unwrap_or_else(|e| panic!("read {}/{name}: {e}", dir.display()));
            serde_json::json!({"file_name": name, "content": content})
        })
        .collect()
}

/// Sub-AC 4.2 fail-closed pins: the frozen fixture corpus must keep one
/// ISOLATED RED fixture for a hand-edited generated projection and one for a
/// stale derived ledger, each emitting exactly `masterplan_projection_stale`.
#[test]
fn masterplan_projection_rederivation_fixtures_fail_closed() {
    for fixture_name in [
        "tc-XA-bad-masterplan-projection-hand-edited.json",
        "tc-XA-bad-masterplan-projection-stale-ledger.json",
    ] {
        let path = fixture_dir().join(fixture_name);
        assert!(
            path.is_file(),
            "projection-rederivation failure-mode fixture must exist: {}",
            path.display()
        );
        let fixture = load_json(&path);
        let report = evaluate(&fixture);
        assert_eq!(
            report.verdict,
            Verdict::Red,
            "{fixture_name} must fail closed (RED)"
        );
        let expected: BTreeSet<String> =
            std::iter::once("masterplan_projection_stale".to_owned()).collect();
        assert_eq!(
            report.violations, expected,
            "{fixture_name} must emit exactly the stale-projection violation"
        );
        assert_eq!(
            expected_violations(&fixture),
            expected,
            "{fixture_name} expected_violations must stay in sync with the pinned set"
        );
    }
}

fn expected_masterplan_projection_paths<'a>(
    masterplan: &'a Value,
    generated_artifacts: &'a Value,
) -> BTreeSet<&'a str> {
    let v2 = &masterplan["masterplan_v2"];
    let mut expected = BTreeSet::new();

    for contract in v2["read_contracts"]
        .as_array()
        .expect("masterplan_v2.read_contracts must be an array")
    {
        let path = contract["path"]
            .as_str()
            .expect("read contract path must be a string");
        if path != "/specs/masterplan.json" && path != FOUNDER_PRODUCT_INTENT_PATH {
            expected.insert(path);
        }
    }

    for surface in v2["surface_dispositions"]
        .as_array()
        .expect("masterplan_v2.surface_dispositions must be an array")
    {
        if surface["disposition"].as_str() == Some("generated-projection") {
            expected.insert(
                surface["path"]
                    .as_str()
                    .expect("generated projection surface path must be a string"),
            );
        }
    }

    for artifact in generated_artifacts["artifacts"]
        .as_array()
        .expect("generated_artifact_control_plane.artifacts must be an array")
    {
        if artifact_source_inputs_include_masterplan(artifact) {
            expected.insert(
                artifact["path"]
                    .as_str()
                    .expect("generated artifact path must be a string"),
            );
        }
    }

    expected
}

fn artifact_source_inputs_include_masterplan(artifact: &Value) -> bool {
    artifact["source_inputs"].as_array().is_some_and(|inputs| {
        inputs.iter().any(|input| {
            input
                .as_str()
                .is_some_and(source_input_refers_to_masterplan)
        })
    })
}

fn source_input_refers_to_masterplan(path: &str) -> bool {
    let path = path.trim();
    let without_fragment = path.split_once('#').map_or(path, |(path, _)| path);
    let mut normalized = without_fragment.trim_start_matches('/');
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped;
    }
    normalized == "specs/masterplan.json"
}

#[test]
fn masterplan_v2_sequencing_is_zero_based_and_founder_ratification_recorded() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let mut findings = evaluate_masterplan_v2_sequencing(&masterplan);

    // The ratification decision must be durable evidence, not a bare boolean: the
    // decision_ref must resolve to a committed evidence record.
    let ratification = &masterplan["masterplan_v2"]["sequencing"]["founder_ratification"];
    assert_eq!(
        ratification["decision_recorded"].as_bool(),
        Some(true),
        "founder-ratification decision must be recorded before execution-wave dispatch"
    );
    let decision_ref = ratification["decision_ref"]
        .as_str()
        .expect("founder_ratification.decision_ref must be a string");
    assert!(
        root.join(decision_ref).is_file(),
        "founder_ratification.decision_ref must resolve to a durable evidence record: {decision_ref}"
    );
    let evidence = load_json(&root.join(decision_ref));
    findings.extend(evaluate_masterplan_v2_ratification_digest(
        &masterplan,
        &evidence,
    ));
    assert!(
        findings.is_empty(),
        "masterplan v2 sequencing must stay zero-based, DAG-derived, and match its durable \
         founder-ratification digest before any execution-wave dispatch: {findings:?}"
    );

    // Fail-closed dispatch contract survives ratification: dispatch without a founder
    // decision stays structurally forbidden even after this decision is recorded.
    let dispatch = &masterplan["masterplan_v2"]["sequencing"]["execution_wave_dispatch"];
    assert_eq!(
        dispatch["requires_founder_ratification"].as_bool(),
        Some(true),
        "execution-wave dispatch must keep requiring founder ratification"
    );
    assert_eq!(
        dispatch["allowed_without_founder_ratification"].as_bool(),
        Some(false),
        "execution-wave dispatch must never be allowed without founder ratification"
    );
}

/// Productized false-green guard: planning closure is an architecture authority, so the
/// cloud-ci cross-artifact gate must fail whenever the contract, masterplan, or sequencing
/// sidecar carry different first-deliverable architecture rule sets. This is a pure
/// data-over-data check over JSON artifacts; it does not shell out to the legacy dev-cli gate.
#[test]
fn planning_closure_architecture_rules_agree_across_authority_artifacts() {
    let root = repo_root();
    let contract = load_json(&root.join("specs/planning-closure-contract.json"));
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let sequencing = load_json(&root.join("specs/master-plan-sequencing.json"));

    let contract_rules = architecture_rule_set(
        &contract["first_deliverable"]["architecture_exit_bar"],
        "specs/planning-closure-contract.json:first_deliverable.architecture_exit_bar",
    );
    let masterplan_rules = architecture_rule_set(
        &masterplan["planning_closure"]["first_deliverable"]["architecture_exit_bar"],
        "specs/masterplan.json:planning_closure.first_deliverable.architecture_exit_bar",
    );
    let sequencing_rules = architecture_rule_set(
        &sequencing["first_deliverable_ordering"]["architecture_exit_bar"],
        "specs/master-plan-sequencing.json:first_deliverable_ordering.architecture_exit_bar",
    );

    assert_same_rule_set(&contract_rules, &masterplan_rules, "contract", "masterplan");
    assert_same_rule_set(&contract_rules, &sequencing_rules, "contract", "sequencing");
}

/// Born-blocking self-test: GATE-1 must go RED on TODAY's real corpus. Per the firewall
/// doctrine, "a firewall that doesn't block today is the facade we're killing." This runs
/// the producer's decision-crosswalk face over the live tree and asserts the gate flags the
/// real defects:
/// - `generated_face_drift` — catalog.json axes_count:6 vs contracts.json axes_count:7
/// - `supersession_half_edge` — ADR-0511 supersedes ADR-0359 while ADR-0359 omits it
///
/// Plus two lanes that must stay CLEAN on the live corpus:
/// - `dual_decision_collision` — the historical two-ADR-0377-files exhibit was resolved
///   2026-06-12 by renumbering the newer file to ADR-0557 (FRIC-1781390000); the live
///   corpus must stay duplicate-free (the RED shape stays pinned by the frozen
///   tc-XA-bad-dup-adr-number fixture).
/// - the frozen-empty `decision_id_mismatch` lane: zero filename/front-matter id
///   disagreements today, asserted with the renumber remediation in the FAIL output.
///
/// Counts are MEASURED, not hardcoded.
#[test]
fn gate1_is_born_blocking_on_the_live_corpus() {
    let root = repo_root();
    let crosswalk = run_producer_face(&root, "decision-crosswalk");

    let report = evaluate(&crosswalk);

    assert_eq!(
        report.verdict,
        Verdict::Red,
        "GATE-1 MUST go RED on today's corpus (the firewall must block today)"
    );
    assert!(
        report.violations.contains("generated_face_drift"),
        "catalog.json axes_count:6 vs contracts.json axes_count:7 -> generated_face_drift must fire"
    );
    assert!(
        !report.violations.contains("dual_decision_collision"),
        "duplicate decision ids must stay resolved (the ADR-0377 pair was renumbered to \
         ADR-0557 per FRIC-1781390000; allocate via the accounting-registry producer's \
         --next-adr): {:?}",
        crosswalk["duplicate_ids"]
    );
    assert!(
        report.violations.contains("supersession_half_edge"),
        "a non-reciprocal supersession edge -> supersession_half_edge must fire"
    );

    // decision_id_mismatch is frozen-empty (born-blocking): the live corpus carries no
    // filename/front-matter id disagreement today, and any future occurrence is the
    // mask vector for a duplicate-numbered ADR pair (FRIC-1781320000). The remediation
    // is named here so the FAIL output alone is actionable.
    let id_mismatches = crosswalk["id_mismatches"]
        .as_array()
        .expect("id_mismatches");
    let next_free_id = crosswalk["next_free_id"].as_str().expect("next_free_id");
    assert!(
        id_mismatches.is_empty(),
        "decision_id_mismatch must stay frozen-empty: {id_mismatches:?} — renumber the newer \
         decision (filename AND front-matter id) to the next free number {next_free_id} \
         (allocate via the accounting-registry producer's --next-adr)"
    );
    assert!(
        next_free_id
            .strip_prefix("ADR-")
            .is_some_and(|digits| digits.len() == 4 && digits.chars().all(|c| c.is_ascii_digit())),
        "next_free_id must be an ADR-NNNN allocator output, got {next_free_id:?}"
    );

    // phantom_decision_citation is frozen-empty (born-blocking, FRIC-1781430000): the
    // phantom-0397 exhibit (seven governed surfaces citing "ADR-0397 Pulsar 4.x + Oxia
    // canonical event-bus" with no file at the number — audit register H-19) was healed
    // 2026-06-12 by MINTING docs/decisions/ADR-0397-pulsar-oxia-canonical-event-bus.md,
    // and the pre-existing phantom inventory is grandfathered shrink-only DATA in the
    // producer (each id ledgered with its citation sites). Any edge here is NEW debt.
    let phantom_citations = crosswalk["phantom_citations"]
        .as_array()
        .expect("phantom_citations");
    assert!(
        phantom_citations.is_empty(),
        "phantom_decision_citation must stay frozen-empty: {phantom_citations:?} — mint the \
         record at the cited number (status Proposed, reconstruction banner; allocate NEW \
         numbers via the accounting-registry producer's --next-adr) or retarget the citation"
    );
    assert!(
        !report.violations.contains("phantom_decision_citation"),
        "the live corpus must carry no phantom decision citation"
    );
    // The healed exhibit resolves: the minted ADR-0397 carries a crosswalk row, so every
    // pre-existing "ADR-0397" citation now reaches a real decision with zero retargeting.
    assert!(
        crosswalk["decisions"]
            .as_array()
            .expect("decisions")
            .iter()
            .any(|d| d["id"] == "ADR-0397"),
        "the minted ADR-0397 must appear as a decision-crosswalk row"
    );

    // The grandfathered inventory is mechanically guarded (review MEDIUM, 2026-06-12):
    // (1) ANTI-PADDING — every grandfathered id must STILL resolve to no decision file;
    //     a healed id (one that now has a crosswalk row) must leave the inventory in the
    //     healing PR, so the carve-out can never shadow a real decision.
    // (2) ANTI-GROWTH — a decrease-only ceiling (the #676 baseline+independent-ceiling
    //     pattern): laundering a NEW phantom citation by adding its id to the inventory
    //     in the same PR forces a loud edit of this pinned ceiling, which may only ever
    //     go DOWN as ids are healed (mint-or-retarget per their ledger rows).
    const GRANDFATHERED_PHANTOM_CEILING: usize = 63; // decrease-only; never raise
    let grandfathered: Vec<&str> = crosswalk["grandfathered_phantom_ids"]
        .as_array()
        .expect("grandfathered_phantom_ids")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        grandfathered.len() <= GRANDFATHERED_PHANTOM_CEILING,
        "the grandfathered phantom inventory may only SHRINK (got {}, ceiling {}): a new \
         phantom citation is never grandfathered — mint the record at the cited number or \
         retarget the citation (FRIC-1781430000)",
        grandfathered.len(),
        GRANDFATHERED_PHANTOM_CEILING
    );
    let decision_ids: BTreeSet<&str> = crosswalk["decisions"]
        .as_array()
        .expect("decisions")
        .iter()
        .filter_map(|d| d["id"].as_str())
        .collect();
    let padded: Vec<&str> = grandfathered
        .iter()
        .copied()
        .filter(|id| decision_ids.contains(id))
        .collect();
    assert!(
        padded.is_empty(),
        "grandfathered ids that now resolve to a real decision must leave the inventory \
         (remove from GRANDFATHERED_PHANTOM_DECISION_IDS + lower the ceiling): {padded:?}"
    );
    // (3) ANTI-INERT — every grandfathered id must still be CITED somewhere in the
    //     governed surfaces; an entry whose citations were all retargeted away protects
    //     nothing and is a standing silent-reintroduction ticket (the FRIC-1781280001
    //     inert-door class) — retire it (remove + lower the ceiling). Together with
    //     (1), (2) and the frozen-empty phantom lane this pins the inventory to be
    //     EXACTLY the live cited-but-missing set.
    let governed_corpus = read_governed_citation_corpus(&root);
    let inert: Vec<&str> = grandfathered
        .iter()
        .copied()
        .filter(|id| !governed_corpus.contains(*id))
        .collect();
    assert!(
        inert.is_empty(),
        "grandfathered ids no longer cited by any governed surface protect nothing — \
         retire them (remove from GRANDFATHERED_PHANTOM_DECISION_IDS + lower the \
         ceiling): {inert:?}"
    );

    // Count the real exhibits for the evidence digest.
    let decisions = crosswalk["decisions"].as_array().expect("decisions");
    let dup_ids = crosswalk["duplicate_ids"]
        .as_array()
        .expect("duplicate_ids");
    let axes = crosswalk["generated_face_axes"]
        .as_object()
        .expect("generated_face_axes");
    let unpropagated = decisions
        .iter()
        .filter(|d| {
            d["status"]
                .as_str()
                .is_some_and(|s| s.eq_ignore_ascii_case("accepted"))
                && (d["in_spec"].as_bool().unwrap_or(false)
                    || d["in_masterplan"].as_bool().unwrap_or(false)
                    || d["in_roadmap"].as_bool().unwrap_or(false))
                && !(d["in_spec"].as_bool().unwrap_or(false)
                    && d["in_masterplan"].as_bool().unwrap_or(false)
                    && d["in_roadmap"].as_bool().unwrap_or(false))
        })
        .count();

    eprintln!(
        "BORN-BLOCKING live-corpus counts: decisions={} duplicate_ids={:?} id_mismatches={:?} phantom_citations={} next_free_id={next_free_id} axes={:?} unpropagated_decision={} violations={:?}",
        decisions.len(),
        dup_ids,
        id_mismatches,
        phantom_citations.len(),
        axes,
        unpropagated,
        report.violations
    );
}

/// Concatenate the governed citation surfaces (every decision body + the
/// roadmap/sequencing artifact + the masterplan) into one corpus string for the
/// anti-inert containment check. A plain substring probe over-approximates the
/// producer's token scan in the conservative direction: an id mentioned in ANY form
/// counts as still-cited, so an entry is only called inert when no governed surface
/// mentions it at all.
fn read_governed_citation_corpus(root: &Path) -> String {
    let mut corpus = String::new();
    let decisions_dir = root.join("docs/decisions");
    let mut paths: Vec<PathBuf> = fs::read_dir(&decisions_dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", decisions_dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .collect();
    paths.push(root.join("specs/master-plan-sequencing.json"));
    paths.push(root.join("specs/masterplan.json"));
    for path in paths {
        corpus.push_str(&fs::read_to_string(&path).unwrap_or_default());
        corpus.push('\n');
    }
    corpus
}

/// Run the producer to emit a single face to stdout, HERMETICALLY. The producer binary must be
/// provided by `OYA_CI_PRODUCER_BIN`; missing env fails closed so tests cannot silently fall back to
/// Cargo. The producer reads the materialized scm-facts face (a declared input); it never calls git.
fn run_producer_face(root: &Path, face: &str) -> Value {
    let scm_facts = root.join("ci/facade/artifact-inventory-registry/scm-facts.generated.json");
    let producer_bin = std::env::var("OYA_CI_PRODUCER_BIN").ok();
    let bin = producer_binary(root, producer_bin.as_deref()).unwrap_or_else(|e| panic!("{e}"));
    let output = Command::new(bin)
        .arg("--repo-root")
        .arg(root)
        .arg("--scm-facts")
        .arg(&scm_facts)
        .arg("--stdout")
        .arg("--face")
        .arg(face)
        .current_dir(root)
        .output()
        .expect("run producer binary");
    assert!(
        output.status.success(),
        "producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("producer face stdout is valid JSON")
}

// ===========================================================================
// Gate-coverage-gap advisory checks (born-advisory vs a frozen baseline).
//
// These three lanes close the #1327 review class that no born-blocking §5.2 code
// keys on: the defects lived in prose / derived-policy / generated-projection
// surfaces. Each check is BORN-ADVISORY — it does not join `evaluate`'s blocking
// verdict; it enforces NO-REGRESSION against the committed frozen baseline
// `gate-coverage-baseline.json`. Each live test asserts the ratchet is CLEAN: the
// live advisory finding set equals the frozen baseline exactly (zero NEW
// regressions AND zero stale burned-down rows). The baseline is born empty.
// ===========================================================================

fn gate_coverage_baseline(root: &Path) -> GateCoverageBaseline {
    let doc =
        load_json(&root.join("ci/facade/cross-artifact-agreement/gate-coverage-baseline.json"));
    GateCoverageBaseline::from_value(&doc)
}

fn assert_ratchet_clean(report: &RatchetReport, lane: &str) {
    assert!(
        report.regressions.is_empty(),
        "{lane}: NEW advisory regression(s) not in gate-coverage-baseline.json — either fix the \
         divergence or record it in the frozen baseline with a justification: {:?}",
        report.regressions
    );
    assert!(
        report.burned_down.is_empty(),
        "{lane}: a gate-coverage-baseline.json row no longer reproduces on the live corpus — \
         remove it and re-freeze (a stale phantom baseline row must never rot the ratchet): {:?}",
        report.burned_down
    );
}

/// Enumerate `docs/decisions/*.md` file names (ADR-NNNN…md), newest amendment
/// dedup applied — matching the ADR-index producer's `read_adr_decision_records`
/// dedup so the id set is apples-to-apples with the projection records.
fn decision_md_file_names(root: &Path) -> Vec<String> {
    let dir = root.join("docs/decisions");
    let mut names: Vec<String> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().to_str().map(str::to_owned))
        .filter(|name| name.starts_with("ADR-") && name.ends_with(".md"))
        .collect();
    names.sort();
    let base_ids: BTreeSet<String> = names
        .iter()
        .filter(|name| !name.contains("-amendment-"))
        .filter_map(|name| name.get(0..8).map(str::to_owned))
        .collect();
    names.retain(|name| {
        if !name.contains("-amendment-") {
            return true;
        }
        name.get(0..8).is_none_or(|id| !base_ids.contains(id))
    });
    names
}

// --- Check 1/3: prose ⇄ front-matter status agreement -----------------------

/// Extract the front-matter `status:` value and the body of an ADR markdown file.
fn adr_frontmatter_status_and_body(contents: &str) -> (Option<String>, &str) {
    let Some(rest) = contents.strip_prefix("---\n") else {
        return (None, contents);
    };
    let Some(end) = rest.find("\n---") else {
        return (None, contents);
    };
    let frontmatter = &rest[..end];
    let body = &rest[end + "\n---".len()..];
    let status = frontmatter.lines().find_map(|line| {
        let line = line.trim();
        line.strip_prefix("status:")
            .map(|value| value.trim().trim_matches('"').trim_matches('\'').to_owned())
    });
    (status, body)
}

fn live_prose_status_corpus(root: &Path) -> Value {
    let dir = root.join("docs/decisions");
    let mut adrs = Vec::new();
    for name in decision_md_file_names(root) {
        let id = name.get(0..8).unwrap_or_default().to_owned();
        let contents =
            fs::read_to_string(dir.join(&name)).unwrap_or_else(|e| panic!("read {name}: {e}"));
        let (status, body) = adr_frontmatter_status_and_body(&contents);
        let Some(status) = status else { continue };
        adrs.push(serde_json::json!({
            "id": id,
            "frontmatter_status": status,
            "body": body,
        }));
    }
    serde_json::json!({ "adrs": adrs })
}

/// Sub-check 1/3 born-advisory over the live tree: no ADR body prose contradicts
/// its own front-matter status (#1327 defect class (a): "stays Proposed" in an
/// Accepted ADR). Enforces no-regression vs the frozen baseline.
#[test]
fn adr_prose_frontmatter_status_agreement_is_advisory_clean_on_live_tree() {
    let root = repo_root();
    let policy = load_json(
        &root.join("ci/facade/cross-artifact-agreement/prose-status-agreement-policy.json"),
    );
    let corpus = live_prose_status_corpus(&root);

    let scanned = corpus["adrs"].as_array().expect("adrs").len();
    assert!(
        scanned > 100,
        "the prose⇄front-matter sweep must cover the real ADR corpus, scanned only {scanned}"
    );

    let findings = evaluate_adr_prose_frontmatter_status(&corpus, &policy);
    let report = ratchet(&findings, &gate_coverage_baseline(&root));
    assert_ratchet_clean(&report, "adr_prose_status_contradiction");
}

// --- Check 2/3: capability-registry ⇄ derived gate-policy sync ---------------

fn live_registry_policy_corpus(root: &Path) -> Value {
    serde_json::json!({
        "registry": load_json(&root.join("specs/capability-registry.json")),
        "policies": {
            "module_membership": {
                "path": "ci/facade/module-membership/capability-membership-policy.json",
                "document": load_json(
                    &root.join("ci/facade/module-membership/capability-membership-policy.json"),
                ),
            },
            "root_hygiene": {
                "path": "ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json",
                "document": load_json(
                    &root.join("ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json"),
                ),
            },
            "tier_dependency": {
                "path": "ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json",
                "document": load_json(&root.join(
                    "ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json",
                )),
            },
        },
    })
}

/// Sub-check 2/3 born-advisory over the live tree: every capability root in
/// specs/capability-registry.json is present in the three derived gate policies
/// (#1327 defect class (c): a registered capability root missing from a derived
/// policy). Enforces no-regression vs the frozen baseline.
#[test]
fn registry_derived_policy_sync_is_advisory_clean_on_live_tree() {
    let root = repo_root();
    let corpus = live_registry_policy_corpus(&root);

    let capabilities = corpus["registry"]["capabilities"]
        .as_array()
        .expect("capabilities")
        .len();
    assert!(
        capabilities >= 20,
        "the registry sync check must cover the real closed capability set, saw only {capabilities}"
    );

    let findings = evaluate_registry_derived_policy_sync(&corpus);
    let report = ratchet(&findings, &gate_coverage_baseline(&root));
    assert_ratchet_clean(&report, "registry_derived_policy_desync");
}

// --- Check 3/3: generated ADR-index projection parity -----------------------

fn adr_records_from_decisions_json(decisions: &Value) -> Vec<AdrDecisionRecord> {
    let mut records = Vec::new();
    for entry in decisions["decisions"].as_array().expect("decisions array") {
        let str_field = |field: &str| -> String {
            entry[field]
                .as_str()
                .unwrap_or_else(|| panic!("decisions.json entry missing string field {field}"))
                .to_owned()
        };
        let str_list = |field: &str| -> Vec<String> {
            entry[field]
                .as_array()
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default()
        };
        records.push(AdrDecisionRecord {
            number: u16::try_from(entry["number"].as_u64().expect("number")).expect("number u16"),
            id: str_field("adr"),
            title: str_field("title"),
            status: str_field("status"),
            owner: str_field("owner"),
            date: str_field("date"),
            path: str_field("path"),
            supersedes: str_list("supersedes"),
            superseded_by: str_list("superseded_by"),
            related: str_list("related"),
        });
    }
    records
}

/// Sub-check 3/3 born-advisory over the live tree: docs/ADR-INDEX.md and
/// docs/machine-readable/decisions.json are byte-parity with their producer's
/// re-render (via the oya-check-adr-index kernel, no shell-out) AND cover exactly
/// the docs/decisions/*.md corpus (#1327 defect class (d): projections not
/// regenerated through their producer; implements the adr-index-pipeline.md
/// promise). Enforces no-regression vs the frozen baseline.
#[test]
fn adr_index_projection_parity_is_advisory_clean_on_live_tree() {
    let root = repo_root();
    let decisions = load_json(&root.join("docs/machine-readable/decisions.json"));
    let records = adr_records_from_decisions_json(&decisions);
    let on_disk_markdown =
        fs::read_to_string(root.join("docs/ADR-INDEX.md")).expect("read docs/ADR-INDEX.md");
    let on_disk_json = fs::read_to_string(root.join("docs/machine-readable/decisions.json"))
        .expect("read docs/machine-readable/decisions.json");
    let source_adr_ids: BTreeSet<String> = decision_md_file_names(&root)
        .iter()
        .filter_map(|name| name.get(0..8).map(str::to_owned))
        .collect();

    assert!(
        records.len() > 400 && source_adr_ids.len() > 400,
        "the ADR-index parity check must cover the real corpus: {} records, {} source ids",
        records.len(),
        source_adr_ids.len()
    );

    let findings = evaluate_adr_index_projection_parity(
        &records,
        &on_disk_markdown,
        &on_disk_json,
        &source_adr_ids,
    );
    let report = ratchet(&findings, &gate_coverage_baseline(&root));
    assert_ratchet_clean(&report, "adr_index_projection_stale");
}

/// The frozen baseline must stay well-formed and, at birth, EMPTY — the three
/// checks are born-advisory-green on the live corpus after #1327. Growth is only
/// ever a reviewed, justified pre-existing divergence.
#[test]
fn gate_coverage_baseline_is_born_empty_and_wellformed() {
    let root = repo_root();
    let doc =
        load_json(&root.join("ci/facade/cross-artifact-agreement/gate-coverage-baseline.json"));
    assert_eq!(
        doc["gate_id"].as_str(),
        Some("cloud-ci-cross-artifact-agreement"),
        "the baseline must name the gate it ratchets"
    );
    let baseline = GateCoverageBaseline::from_value(&doc);
    assert!(
        baseline.keys().is_empty(),
        "the gate-coverage baseline is born empty (born-advisory-green): {:?}",
        baseline.keys()
    );
}
