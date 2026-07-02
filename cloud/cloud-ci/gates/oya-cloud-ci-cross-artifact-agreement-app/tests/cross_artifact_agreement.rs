// GATE-1 cloud-ci-cross-artifact-agreement: RED/GREEN fixture corpus + born-blocking
// live-corpus self-test. ADR-0083 Tier-3: integration tests use unwrap/expect to assert
// invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use oya_cloud_ci_cross_artifact_agreement_app::{
    Verdict, evaluate, evaluate_masterplan_v2_authority, evaluate_masterplan_v2_entry_surfaces,
    evaluate_masterplan_v2_evidence_state, evaluate_masterplan_v2_plan_evidence_drift,
    evaluate_masterplan_v2_program_coverage, evaluate_masterplan_v2_projection_freshness,
    evaluate_masterplan_v2_read_contract_archives, evaluate_masterplan_v2_sequencing,
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
fn masterplan_v2_sequencing_is_zero_based_and_dispatch_blocked_pending_founder() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let findings = evaluate_masterplan_v2_sequencing(&masterplan);
    let codes: BTreeSet<&str> = findings
        .iter()
        .map(|finding| finding.code.as_str())
        .collect();

    assert!(
        !codes.contains("masterplan_sequencing_invalid"),
        "masterplan v2 sequencing must stay zero-based and DAG-derived: {findings:?}"
    );
    assert!(
        codes.contains("masterplan_execution_wave_dispatch_unratified"),
        "execution-wave dispatch must remain blocked until founder ratification is recorded"
    );
    assert!(
        findings.iter().all(|finding| {
            finding.key != "masterplan_v2.sequencing.execution_wave_dispatch.not_blocked"
        }),
        "pending-founder state must be fail-closed, not merely unratified: {findings:?}"
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
