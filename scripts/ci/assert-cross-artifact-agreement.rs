//! AC-0.8 cross-artifact agreement seed gate.
//!
//! This checker is local/static evidence only. It verifies that the Phase-0
//! decision propagation packet inventory maps register #1..#21 to an
//! ADR/spec/masterplan/roadmap agreement set, that generated-artifact drift and
//! unreconciled idea-refine outputs have RED fixtures, and that the gate is
//! wired through Buck2. It never runs live CI, posts statuses, mutates branch
//! protection, adds an `oya` CLI surface, or proves Phase-0 completion.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_REGISTRY: &str = "specs/cross-artifact-agreement-registry.json";
const PACKET_REGISTRY: &str = "specs/decision-propagation-packets.json";
const ROOT_BUCK: &str = "BUCK";
const ADR_0365: &str = "docs/decisions/ADR-0365-automated-adr-lifecycle-and-propagation.md";
const MASTERPLAN_GENERATED: &str = "docs/machine-readable/masterplan.generated.json";
const ROADMAP_GENERATED: &str = "docs/machine-readable/board-sync.generated.json";

const FIXTURES: &[&str] = &[
    "specs/fixtures/phase0-cross-artifact-agreement/tc-0.8-good-cross-artifact-agreement.json",
    "specs/fixtures/phase0-cross-artifact-agreement/tc-0.8-bad-missing-masterplan-roadmap.json",
    "specs/fixtures/phase0-cross-artifact-agreement/tc-0.8-bad-unreconciled-idea-refine-output.json",
    "specs/fixtures/phase0-cross-artifact-agreement/tc-0.8-bad-generated-decisions-divergence.json",
    "specs/fixtures/phase0-cross-artifact-agreement/tc-0.8-bad-missing-register-packet.json",
];

const FALSE_CLAIMS: &[&str] = &[
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
];

const TRUE_REGISTRY_FLAGS: &[&str] = &[
    "cross_artifact_agreement_measured",
    "agreement_set_required",
    "decision_propagation_packets_measured",
    "backlog_register_1_to_21_mapped",
    "idea_refine_reconciliation_required",
    "generated_file_drift_blocked",
    "architect_critic_consensus_required",
    "no_new_oya_cli_surface",
];

const REQUIRED_PACKET_FIELDS: &[&str] = &[
    "decision_id",
    "source_edit_files",
    "generated_outputs",
    "owner_lane",
    "status",
    "regeneration_command",
    "verification_gate",
    "agreement_set",
];

const AGREEMENT_ARTIFACTS: &[&str] = &["adr", "spec", "masterplan", "roadmap"];

const REQUIRED_PACKET_IDS: &[&str] = &[
    "P0.7-D01-d1-trinity",
    "P0.7-D02-effective-dating",
    "P0.7-D03-cloud-ci-buck2-enforcement",
    "P0.7-D04-pure-split-service-structure",
    "P0.7-D05-pack-root-classification",
    "P0.7-D06-multi-platform-native-client",
    "P0.7-D07-parallel-dev-coordination",
    "P0.7-D08-verification-resilience-testing",
    "P0.7-D09-frontend-ssr-islands-wasm",
    "P0.7-D10-honest-claim-compliance",
    "P0.7-D11-cross-artifact-ssot-agreement",
    "P0.7-D12-status-adr-hygiene-fixes",
    "P0.7-D13-platform-readiness-masterplan-roadmap",
    "P0.7-D14-pure-rust-tooling-discipline",
    "P0.7-D15-false-green-d1-reality-check",
    "P0.7-D16-ci-adr-sprawl-consolidation",
    "P0.7-D17-dogfood-need-sequencing",
    "P0.7-D18-merge-conflict-elimination",
    "P0.7-D19-bespoke-cloud-toolchain-services",
    "P0.7-D20-automation-ratchet",
    "P0.7-D21-claim-ceiling-no-empty-promises",
];

const KNOWN_FIXTURE_VIOLATIONS: &[&str] = &[
    "missing_masterplan_artifact",
    "missing_roadmap_artifact",
    "idea_refine_output_unreconciled",
    "generated_decisions_json_diverged",
    "missing_decision_propagation_packet_P0.7-D18-merge-conflict-elimination",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileResult {
    // data_class: INTERNAL governance evidence path, not tenant data.
    pub path: String,
    // data_class: INTERNAL deterministic gate diagnostics, not tenant data.
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureResult {
    // data_class: INTERNAL governance fixture path, not tenant data.
    pub path: String,
    // data_class: INTERNAL expected local/static verdict marker, not live CI state.
    pub expected: String,
    // data_class: INTERNAL synthetic violation labels, not production findings.
    pub observed_violations: Vec<String>,
    // data_class: INTERNAL deterministic gate diagnostics, not tenant data.
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    // data_class: INTERNAL local/static gate verdict, not branch-protection authority.
    pub verdict: String,
    // data_class: INTERNAL registry path, not tenant data.
    pub registry: String,
    // data_class: INTERNAL file-level diagnostics, not tenant data.
    pub file_results: Vec<FileResult>,
    // data_class: INTERNAL fixture diagnostics, not tenant data.
    pub fixture_results: Vec<FixtureResult>,
    // data_class: INTERNAL flattened diagnostics, not tenant data.
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub repo_root: PathBuf,
    pub registry: String,
    pub json: bool,
}

fn json_escape(input: &str) -> String {
    input
        .chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn compact_json_text(input: &str) -> String {
    input.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn bool_token(key: &str, value: bool) -> String {
    format!("\"{}\":{}", key, if value { "true" } else { "false" })
}

fn has_bool(text: &str, key: &str, value: bool) -> bool {
    compact_json_text(text).contains(&bool_token(key, value))
}

fn count_occurrences(text: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    text.match_indices(needle).count()
}

fn read_repo_file(repo_root: &Path, path: &str) -> Result<String, String> {
    fs::read_to_string(repo_root.join(path)).map_err(|error| format!("{path}: {error}"))
}

fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0_u32;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, character) in source[open..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        match character {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(open + offset);
                }
            }
            _ => {}
        }
    }
    None
}

fn object_containing_token(source: &str, token: &str) -> Option<String> {
    let token_start = source.find(token)?;
    let open = source[..token_start].rfind('{')?;
    let close = matching_brace(source, open)?;
    Some(source[open..=close].to_owned())
}

fn object_for_key(source: &str, key: &str) -> Option<String> {
    let key_token = format!("\"{}\"", key);
    let key_start = source.find(&key_token)?;
    let open = source[key_start..]
        .find('{')
        .map(|offset| key_start + offset)?;
    let close = matching_brace(source, open)?;
    Some(source[open..=close].to_owned())
}

fn has_non_empty_array(object: &str, key: &str) -> bool {
    let key_token = format!("\"{}\"", key);
    let Some(key_start) = object.find(&key_token) else {
        return false;
    };
    let Some(open_offset) = object[key_start..].find('[') else {
        return false;
    };
    let open = key_start + open_offset;
    let Some(close_offset) = object[open..].find(']') else {
        return false;
    };
    object[open + 1..open + close_offset].contains('"')
}

fn claim_boundary_failures(text: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for claim in FALSE_CLAIMS {
        if !has_bool(text, claim, false) {
            failures.push(format!("forbidden_true_or_missing_claim_{claim}"));
        }
    }
    failures
}

pub fn registry_failures(registry: &str) -> Vec<String> {
    let mut failures = claim_boundary_failures(registry);
    for flag in TRUE_REGISTRY_FLAGS {
        if !has_bool(registry, flag, true) {
            failures.push(format!("missing_true_registry_flag_{flag}"));
        }
    }
    for token in [
        "//:cross-artifact-agreement-check",
        "scripts/ci/assert-cross-artifact-agreement.rs",
        "scripts/tests/cross_artifact_agreement_check.rs",
        "specs/decision-propagation-packets.json",
        "TC-0.8.1",
        "TC-0.8.2",
        "TC-0.8.3",
        "TC-0.14.3",
        "ADR-0365",
        "no new oya CLI surface",
        "local/static seed evidence only",
    ] {
        if !registry.contains(token) {
            failures.push(format!("registry_missing_token_{token}"));
        }
    }
    if registry.contains("oya gate") || registry.contains("oya verify") {
        failures.push("registry_must_not_route_to_oya_gate_or_verify_authority".to_owned());
    }
    failures
}

fn agreement_set_failures(packet: &str, packet_id: &str) -> Vec<String> {
    let mut failures = Vec::new();
    let Some(agreement_set) = object_for_key(packet, "agreement_set") else {
        failures.push(format!("{packet_id}:missing_agreement_set_object"));
        return failures;
    };
    for artifact in AGREEMENT_ARTIFACTS {
        let Some(artifact_object) = object_for_key(&agreement_set, artifact) else {
            failures.push(format!("{packet_id}:missing_agreement_artifact_{artifact}"));
            continue;
        };
        for field in ["path", "status"] {
            if !artifact_object.contains(&format!("\"{field}\"")) {
                failures.push(format!(
                    "{packet_id}:agreement_artifact_{artifact}_missing_{field}"
                ));
            }
        }
    }
    failures
}

pub fn packet_registry_failures(packet_registry: &str) -> Vec<String> {
    let mut failures = claim_boundary_failures(packet_registry);
    if !has_bool(
        packet_registry,
        "decision_propagation_packets_measured",
        true,
    ) {
        failures.push("missing_true_packet_flag_decision_propagation_packets_measured".to_owned());
    }
    if !has_bool(packet_registry, "backlog_register_1_to_21_mapped", true) {
        failures.push("missing_true_packet_flag_backlog_register_1_to_21_mapped".to_owned());
    }
    if !has_bool(packet_registry, "no_new_oya_cli_surface", true) {
        failures.push("missing_true_packet_flag_no_new_oya_cli_surface".to_owned());
    }
    if packet_registry.contains("oya gate") || packet_registry.contains("oya verify") {
        failures.push("packet_registry_must_not_route_to_oya_gate_or_verify_authority".to_owned());
    }
    let packet_count = count_occurrences(packet_registry, "\"decision_id\":");
    if packet_count != REQUIRED_PACKET_IDS.len() {
        failures.push(format!(
            "decision_packet_count_mismatch_expected_{}_observed_{packet_count}",
            REQUIRED_PACKET_IDS.len()
        ));
    }
    for (index, packet_id) in REQUIRED_PACKET_IDS.iter().enumerate() {
        let token = format!("\"decision_id\": \"{packet_id}\"");
        if !packet_registry.contains(&token) {
            failures.push(format!("missing_decision_propagation_packet_{packet_id}"));
            continue;
        }
        let Some(packet) = object_containing_token(packet_registry, &token) else {
            failures.push(format!("{packet_id}:packet_object_not_extractable"));
            continue;
        };
        let register_token = format!("\"register_number\": {}", index + 1);
        if !packet.contains(&register_token) {
            failures.push(format!("{packet_id}:register_number_mismatch_or_missing"));
        }
        for field in REQUIRED_PACKET_FIELDS {
            if !packet.contains(&format!("\"{field}\"")) {
                failures.push(format!("{packet_id}:missing_required_field_{field}"));
            }
        }
        for field in ["source_edit_files", "generated_outputs"] {
            if !has_non_empty_array(&packet, field) {
                failures.push(format!("{packet_id}:empty_or_missing_array_{field}"));
            }
        }
        if !packet.contains("\"verification_gate\": \"//:cross-artifact-agreement-check\"") {
            failures.push(format!(
                "{packet_id}:verification_gate_not_cross_artifact_check"
            ));
        }
        failures.extend(agreement_set_failures(&packet, packet_id));
        if packet.contains("packetized_pending_propagation") {
            let Some(judgment) = object_for_key(&packet, "temporary_human_judgment") else {
                failures.push(format!(
                    "{packet_id}:pending_packet_missing_temporary_human_judgment"
                ));
                continue;
            };
            for field in ["owner", "retirement_path", "retirement_phase"] {
                if !judgment.contains(&format!("\"{field}\"")) {
                    failures.push(format!(
                        "{packet_id}:temporary_human_judgment_missing_{field}"
                    ));
                }
            }
        }
        failures.extend(
            claim_boundary_failures(&packet)
                .into_iter()
                .map(|failure| format!("{packet_id}:{failure}")),
        );
    }
    let d18 = "P0.7-D18-merge-conflict-elimination";
    if let Some(packet) =
        object_containing_token(packet_registry, &format!("\"decision_id\": \"{d18}\""))
    {
        for token in [
            "specs/generated-artifact-registry.json",
            "merge-readiness oracle",
            "ADR-0513",
        ] {
            if !packet.contains(token) {
                failures.push(format!("{d18}:missing_required_binding_{token}"));
            }
        }
    }
    let d19 = "P0.7-D19-bespoke-cloud-toolchain-services";
    if let Some(packet) =
        object_containing_token(packet_registry, &format!("\"decision_id\": \"{d19}\""))
    {
        for token in [
            "specs/bespoke-cloud-toolchain-services.json",
            "P-TOOLCHAIN",
            "ADR-0513",
        ] {
            if !packet.contains(token) {
                failures.push(format!("{d19}:missing_required_binding_{token}"));
            }
        }
    }
    failures
}

pub fn buck_failures(root_buck: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for token in [
        "cross-artifact-agreement-check",
        "scripts/ci/assert-cross-artifact-agreement.rs",
        "scripts/tests/cross_artifact_agreement_check.rs",
        "specs/cross-artifact-agreement-registry.json",
        "specs/decision-propagation-packets.json",
        "rustc --edition=2021 -D warnings scripts/tests/cross_artifact_agreement_check.rs --test",
        "rustc --edition=2021 -D warnings scripts/ci/assert-cross-artifact-agreement.rs",
    ] {
        if !root_buck.contains(token) {
            failures.push(format!("root_buck_missing_{token}"));
        }
    }
    failures
}

pub fn authority_file_failures(
    adr_0365: &str,
    registry: &str,
    masterplan_generated: &str,
    roadmap_generated: &str,
) -> Vec<String> {
    let mut failures = Vec::new();
    for (label, text, tokens) in [
        (
            ADR_0365,
            adr_0365,
            &["id: ADR-0365", "affected_surfaces", "propagation-drift"][..],
        ),
        (
            DEFAULT_REGISTRY,
            registry,
            &[
                "P0.7-D11-cross-artifact-ssot-agreement",
                "P0.7-D18-merge-conflict-elimination",
                "P0.7-D19-bespoke-cloud-toolchain-services",
            ][..],
        ),
        (
            MASTERPLAN_GENERATED,
            masterplan_generated,
            &["ADR-0365-D3", "ADR-0365"][..],
        ),
        (
            ROADMAP_GENERATED,
            roadmap_generated,
            &["ADR-0365-D3", "Generated from masterplan deliverable"][..],
        ),
    ] {
        for token in tokens {
            if !text.contains(token) {
                failures.push(format!("{label}:missing_required_token_{token}"));
            }
        }
    }
    failures
}

pub fn fixture_observed_violations(fixture: &str) -> Vec<String> {
    let mut violations = Vec::new();
    let compact = compact_json_text(fixture);
    if !compact.contains("\"masterplan\":{") {
        violations.push("missing_masterplan_artifact".to_owned());
    }
    if !compact.contains("\"roadmap\":{") {
        violations.push("missing_roadmap_artifact".to_owned());
    }
    if has_bool(fixture, "idea_refine_reconciled", false) {
        violations.push("idea_refine_output_unreconciled".to_owned());
    }
    if has_bool(fixture, "generated_decisions_json_matches_source", false) {
        violations.push("generated_decisions_json_diverged".to_owned());
    }
    if fixture.contains("\"missing_packet_id\": \"P0.7-D18-merge-conflict-elimination\"") {
        violations.push(
            "missing_decision_propagation_packet_P0.7-D18-merge-conflict-elimination".to_owned(),
        );
    }
    violations
}

pub fn fixture_policy_failures(fixture: &str) -> Vec<String> {
    let mut failures = claim_boundary_failures(fixture);
    let expected_pass = fixture.contains("\"expected_verdict\": \"PASS\"");
    let expected_fail = fixture.contains("\"expected_verdict\": \"FAIL\"");
    let observed_violations = fixture_observed_violations(fixture);
    if expected_pass {
        for token in [
            "TC-0.8-GOOD-cross-artifact-agreement-complete",
            "P0.7-D11-cross-artifact-ssot-agreement",
            "docs/decisions/ADR-0365-automated-adr-lifecycle-and-propagation.md",
            "specs/cross-artifact-agreement-registry.json",
            "docs/machine-readable/masterplan.generated.json",
            "docs/machine-readable/board-sync.generated.json",
            "P0.7-D18-merge-conflict-elimination",
            "P0.7-D19-bespoke-cloud-toolchain-services",
        ] {
            if !fixture.contains(token) {
                failures.push(format!("good_fixture_missing_token_{token}"));
            }
        }
        for flag in [
            "source_and_consumer_same_changeset",
            "idea_refine_reconciled",
            "architect_critic_consensus",
            "generated_decisions_json_matches_source",
        ] {
            if !has_bool(fixture, flag, true) {
                failures.push(format!("good_fixture_missing_true_flag_{flag}"));
            }
        }
        if !observed_violations.is_empty() {
            failures.push(format!(
                "good_fixture_has_observed_violations_{}",
                observed_violations.join("+")
            ));
        }
    } else if expected_fail {
        if observed_violations.is_empty() {
            failures.push("bad_fixture_missing_observed_violation".to_owned());
        }
        for violation in &observed_violations {
            if !KNOWN_FIXTURE_VIOLATIONS.contains(&violation.as_str()) {
                failures.push(format!("bad_fixture_unknown_violation_{violation}"));
            }
            if !fixture.contains(violation) {
                failures.push(format!("bad_fixture_violation_not_declared_{violation}"));
            }
        }
    } else {
        failures.push("fixture_missing_expected_verdict".to_owned());
    }
    failures
}

fn evaluate_path(path: &str, failures: Vec<String>) -> FileResult {
    FileResult {
        path: path.to_owned(),
        failures,
    }
}

pub fn evaluate(repo_root: &Path, registry_path: &str) -> Result<Evaluation, String> {
    let registry = read_repo_file(repo_root, registry_path)?;
    let packets = read_repo_file(repo_root, PACKET_REGISTRY)?;
    let root_buck = read_repo_file(repo_root, ROOT_BUCK)?;
    let adr_0365 = read_repo_file(repo_root, ADR_0365)?;
    let masterplan_generated = read_repo_file(repo_root, MASTERPLAN_GENERATED)?;
    let roadmap_generated = read_repo_file(repo_root, ROADMAP_GENERATED)?;

    let file_results = vec![
        evaluate_path(registry_path, registry_failures(&registry)),
        evaluate_path(PACKET_REGISTRY, packet_registry_failures(&packets)),
        evaluate_path(ROOT_BUCK, buck_failures(&root_buck)),
        evaluate_path(
            ADR_0365,
            authority_file_failures(
                &adr_0365,
                &registry,
                &masterplan_generated,
                &roadmap_generated,
            )
            .into_iter()
            .filter(|failure| failure.starts_with(ADR_0365))
            .collect(),
        ),
        evaluate_path(
            MASTERPLAN_GENERATED,
            authority_file_failures(
                &adr_0365,
                &registry,
                &masterplan_generated,
                &roadmap_generated,
            )
            .into_iter()
            .filter(|failure| failure.starts_with(MASTERPLAN_GENERATED))
            .collect(),
        ),
        evaluate_path(
            ROADMAP_GENERATED,
            authority_file_failures(
                &adr_0365,
                &registry,
                &masterplan_generated,
                &roadmap_generated,
            )
            .into_iter()
            .filter(|failure| failure.starts_with(ROADMAP_GENERATED))
            .collect(),
        ),
    ];

    let fixture_results = FIXTURES
        .iter()
        .map(|path| {
            let fixture = read_repo_file(repo_root, path)?;
            let expected = if fixture.contains("\"expected_verdict\": \"PASS\"") {
                "PASS"
            } else if fixture.contains("\"expected_verdict\": \"FAIL\"") {
                "FAIL"
            } else {
                "UNKNOWN"
            };
            Ok(FixtureResult {
                path: (*path).to_owned(),
                expected: expected.to_owned(),
                observed_violations: fixture_observed_violations(&fixture),
                failures: fixture_policy_failures(&fixture),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let failures = file_results
        .iter()
        .flat_map(|result| {
            result
                .failures
                .iter()
                .map(|failure| format!("{}:{failure}", result.path))
        })
        .chain(fixture_results.iter().flat_map(|result| {
            result
                .failures
                .iter()
                .map(|failure| format!("{}:{failure}", result.path))
        }))
        .collect::<Vec<_>>();

    let verdict = if failures.is_empty() { "PASS" } else { "FAIL" }.to_owned();
    Ok(Evaluation {
        verdict,
        registry: registry_path.to_owned(),
        file_results,
        fixture_results,
        failures,
    })
}

fn array_json(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn to_json(evaluation: &Evaluation) -> String {
    let files = evaluation
        .file_results
        .iter()
        .map(|result| {
            format!(
                "{{\"path\":\"{}\",\"failures\":{}}}",
                json_escape(&result.path),
                array_json(&result.failures)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let fixtures = evaluation
        .fixture_results
        .iter()
        .map(|result| {
            format!(
                "{{\"path\":\"{}\",\"expected\":\"{}\",\"observed_violations\":{},\"failures\":{}}}",
                json_escape(&result.path),
                json_escape(&result.expected),
                array_json(&result.observed_violations),
                array_json(&result.failures)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"verdict\":\"{}\",\"registry\":\"{}\",\"cross_artifact_agreement_measured\":true,\"packet_count\":{},\"fixture_count\":{},\"file_results\":[{}],\"fixture_results\":[{}],\"p0_0_green\":false,\"phase0_complete\":false,\"failures\":{}}}",
        json_escape(&evaluation.verdict),
        json_escape(&evaluation.registry),
        REQUIRED_PACKET_IDS.len(),
        evaluation.fixture_results.len(),
        files,
        fixtures,
        array_json(&evaluation.failures)
    )
}

pub fn parse_args() -> Config {
    let mut repo_root = env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut registry = DEFAULT_REGISTRY.to_owned();
    let mut json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = args
                    .next()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
            }
            "--registry" => {
                registry = args.next().unwrap_or_else(|| DEFAULT_REGISTRY.to_owned());
            }
            "--json" => json = true,
            _ => {}
        }
    }
    Config {
        repo_root,
        registry,
        json,
    }
}

fn main() {
    let config = parse_args();
    let evaluation = match evaluate(&config.repo_root, &config.registry) {
        Ok(evaluation) => evaluation,
        Err(error) => Evaluation {
            verdict: "FAIL".to_owned(),
            registry: config.registry,
            file_results: Vec::new(),
            fixture_results: Vec::new(),
            failures: vec![error],
        },
    };
    if config.json {
        println!("{}", to_json(&evaluation));
    } else {
        println!("{}", evaluation.verdict);
        for failure in &evaluation.failures {
            eprintln!("{failure}");
        }
    }
    if evaluation.verdict != "PASS" {
        std::process::exit(1);
    }
}
