// GATE-1 cloud-ci-cross-artifact-agreement: RED/GREEN fixture corpus + born-blocking
// live-corpus self-test. ADR-0083 Tier-3: integration tests use unwrap/expect to assert
// invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use oya_cloud_ci_cross_artifact_agreement_app::{
    Verdict, compute_masterplan_v2_sequencing_digest, derive_masterplan_md_projection, evaluate,
    evaluate_masterplan_plan_evidence_crosscheck, evaluate_masterplan_projection_rederivation,
    evaluate_masterplan_read_surface_resurrections, evaluate_masterplan_v2_authority,
    evaluate_masterplan_v2_entry_surfaces, evaluate_masterplan_v2_evidence_state,
    evaluate_masterplan_v2_plan_evidence_drift, evaluate_masterplan_v2_program_coverage,
    evaluate_masterplan_v2_projection_freshness, evaluate_masterplan_v2_read_contract_archives,
    evaluate_masterplan_v2_sequencing,
};
use serde_json::Value;

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

/// Sub-AC 4.1 + Sub-AC 1.2 masterplan structural gate: the frozen fixture corpus
/// must keep one ISOLATED fail-closed RED fixture per structural failure class —
/// duplicate work-item ids, dependency cycles, dangling (orphan) dependency
/// references, and undeclared cross-program edge crossings.
/// The generic runner above only demands "some RED fixture exists"; this test pins
/// each named failure mode to its exact violation set so none can be silently
/// dropped or diluted.
#[test]
fn masterplan_structural_failure_mode_fixtures_fail_closed() {
    let cases: [(&str, &[&str]); 4] = [
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
        (
            "tc-XA-bad-masterplan-cross-program-edge-undeclared.json",
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
fn masterplan_v2_hermes_done_card_claims_are_unverified_until_evidence_attaches() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let findings = evaluate_masterplan_v2_evidence_state(&masterplan);
    assert!(
        findings.is_empty(),
        "masterplan v2 evidence-state policy must keep unverified Hermes done claims out of done: {findings:?}"
    );

    let imports = masterplan["masterplan_v2"]["hermes_done_card_imports"]
        .as_array()
        .expect("masterplan_v2.hermes_done_card_imports must be an array");
    let unverified_hermes_done_import = imports.iter().any(|claim| {
        claim["source_system"].as_str() == Some("hermes")
            && claim["source_status"].as_str() == Some("done")
            && claim["evidence_refs"].as_array().is_some_and(Vec::is_empty)
            && claim["masterplan_status"].as_str() == Some("claimed-done-unverified")
            && claim["evidence_state"].as_str() == Some("claimed-done-unverified")
    });
    assert!(
        unverified_hermes_done_import,
        "Hermes done-card imports without evidence must be explicitly marked claimed-done-unverified"
    );
}

/// Sub-AC 3 verifiability clause, asserted over the live per-card ledger: every
/// Hermes done-card completion claim is imported as claimed-done-unverified, no
/// claim carries a verified status without an attached evidence link, evidence
/// attachment is reflected as evidence_state=evidence-attached (and never
/// silently hidden), and the import summary's counts agree with the claims.
#[test]
fn masterplan_v2_hermes_done_card_ledger_is_per_card_and_never_verified_without_evidence() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let imports = masterplan["masterplan_v2"]["hermes_done_card_imports"]
        .as_array()
        .expect("masterplan_v2.hermes_done_card_imports must be an array");
    assert!(
        !imports.is_empty(),
        "the done-card ledger must carry per-card claims"
    );

    let mut evidence_attached = 0u64;
    let mut unverified_pending = 0u64;
    for claim in imports {
        let id = claim["source_card_id"].as_str().expect(
            "every ledger entry must name a concrete source_card_id (per-card, not count-level)",
        );
        let refs = claim["evidence_refs"]
            .as_array()
            .unwrap_or_else(|| panic!("{id}: evidence_refs must be an array"));
        let status = claim["masterplan_status"].as_str().unwrap_or_default();
        let state = claim["evidence_state"].as_str().unwrap_or_default();
        assert_eq!(
            status, "claimed-done-unverified",
            "{id}: every Hermes done-card completion claim stays unverified-pending-evidence; \
             upgrading one requires a verification pass that attaches evidence AND updates this contract"
        );
        if refs.is_empty() {
            unverified_pending += 1;
            assert_eq!(
                state, "claimed-done-unverified",
                "{id}: a claim without an attached evidence link must stay flagged"
            );
        } else {
            evidence_attached += 1;
            assert_eq!(
                state, "evidence-attached",
                "{id}: attached evidence refs must surface as evidence-attached"
            );
        }
    }

    let summary = &masterplan["masterplan_v2"]["hermes_done_card_import_summary"];
    assert_eq!(
        summary["extracted_done_count"].as_u64(),
        Some(evidence_attached + unverified_pending),
        "summary extracted_done_count must equal the number of per-card claims"
    );
    assert_eq!(
        summary["evidence_attached_count"].as_u64(),
        Some(evidence_attached),
        "summary evidence_attached_count must agree with the claims"
    );
    assert_eq!(
        summary["unverified_pending_evidence_count"].as_u64(),
        Some(unverified_pending),
        "summary unverified_pending_evidence_count must agree with the claims"
    );
    assert_eq!(
        summary["verified_count"].as_u64(),
        Some(0),
        "no done-card claim is verified in this ledger generation"
    );

    let ledger_rel = summary["ledger_artifact"]
        .as_str()
        .expect("summary must reference the forensic ledger evidence artifact");
    assert!(
        root.join(ledger_rel).is_file(),
        "forensic ledger artifact {ledger_rel} must exist"
    );
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
/// tree: every masterplan work-item status claim and evidence-attached Hermes
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
    let scm_facts = load_json(&root.join(
        "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/scm-facts.generated.json",
    ));
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
        Some(
            "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/scm-facts.generated.json#tracked_paths"
        ),
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
    let scm_facts = load_json(&root.join(
        "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/scm-facts.generated.json",
    ));
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
        if path != "/specs/masterplan.json" {
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
    let findings = evaluate_masterplan_v2_sequencing(&masterplan);

    assert!(
        findings.is_empty(),
        "masterplan v2 sequencing must stay zero-based, DAG-derived, and carry a recorded \
         founder-ratification decision before any execution-wave dispatch: {findings:?}"
    );

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
    // The sequencing identity must be embedded, versioned, and byte-stable: the
    // digest recomputed from the live DAG + waves + order must equal BOTH the
    // embedded sequencing_hash and the founder-ratified digest, so ratification
    // and gates reference one stable content identity.
    let identity = &masterplan["masterplan_v2"]["sequencing"]["sequencing_identity"];
    let recomputed = compute_masterplan_v2_sequencing_digest(&masterplan["masterplan_v2"])
        .expect("live masterplan v2 sequencing content must hash");
    assert_eq!(
        identity["sequencing_hash"].as_str(),
        Some(recomputed.as_str()),
        "sequencing_identity.sequencing_hash must equal the digest recomputed from live content"
    );
    assert!(
        identity["sequencing_version"]
            .as_u64()
            .is_some_and(|version| version >= 1),
        "sequencing_identity.sequencing_version must be a monotonic integer >= 1"
    );
    assert_eq!(
        ratification["ratified_sequencing_digest"].as_str(),
        Some(recomputed.as_str()),
        "founder ratification must bind to the live sequencing digest; a mismatch means the \
         ratified content mutated and a fresh derivation + ratification are required"
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
    let scm_facts = root
        .join("cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/scm-facts.generated.json");
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
