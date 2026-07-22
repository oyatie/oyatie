#![forbid(unsafe_code)]

//! Pure validation for the pre-roadmap Stage-1 evidence and authority closure program.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

const PROGRAM_SCHEMA_ID: &str = "oyatie/stage1-closure-program/v1";
const EPOCH_SCHEMA_ID: &str = "oyatie/stage1-evidence-epoch/v2";
const PROTECTED_FACTS_SCHEMA_ID: &str = "oyatie/stage1-protected-facts/v1";
const PROGRAM_ID: &str = "correct-way-forward-before-roadmap";
const PROGRAM_REF: &str =
    "/specs/masterplan.json#masterplan_v2.planning_entry_contract.stage1_closure_program";
/// Canonical source-epoch receipt fields required by the parser and schema wire contract.
pub const SOURCE_RECEIPT_REQUIRED_FIELDS: [&str; 14] = [
    "path",
    "blob_oid",
    "sha256",
    "subject_digest",
    "principal_id",
    "issuer_authority_class",
    "authority_source_ref",
    "qualification",
    "jurisdiction_scope",
    "independence_observation",
    "validity",
    "revocation_status",
    "conflict_status",
    "signature_trust_root_binding",
];

/// Canonical required fields of `oyatie/stage1-admission-envelope/v1`.
///
/// This parser-owned list is locked against the source schema by the admission-envelope fixture
/// regression. It intentionally contains no protected-facts aliases.
pub const ADMISSION_ENVELOPE_REQUIRED_FIELDS: [&str; 24] = [
    "schema_id",
    "repository",
    "branch",
    "base_commit",
    "base_tree",
    "pr_head_commit",
    "pr_head_tree",
    "postmerge_promoted_commit",
    "postmerge_promoted_tree",
    "facts_binding",
    "program_binding",
    "evaluator_binding",
    "policy_binding",
    "schema_binding",
    "immutable_successor_binding",
    "check_suite_binding",
    "independent_review_binding",
    "branch_protection_binding",
    "postmerge_completion",
    "envelope_signature",
    "trust_root_binding",
    "roadmap_planning_authorized",
    "binding_plan_approval_allowed",
    "implementation_dispatch_allowed",
];

/// Canonical allowed fields of `oyatie/stage1-admission-envelope/v1`.
pub const ADMISSION_ENVELOPE_ALLOWED_FIELDS: [&str; 25] = [
    "schema_id",
    "repository",
    "branch",
    "base_commit",
    "base_tree",
    "pr_head_commit",
    "pr_head_tree",
    "postmerge_promoted_commit",
    "postmerge_promoted_tree",
    "facts_binding",
    "program_binding",
    "evaluator_binding",
    "policy_binding",
    "schema_binding",
    "immutable_successor_binding",
    "check_suite_binding",
    "independent_review_binding",
    "branch_protection_binding",
    "postmerge_completion",
    "envelope_signature",
    "trust_root_binding",
    "roadmap_planning_authorized",
    "binding_plan_approval_allowed",
    "implementation_dispatch_allowed",
    "extensions",
];

const STATES: [&str; 6] = [
    "HOLD_EPOCH_OPEN",
    "HOLD_EVIDENCE_COMPLETE",
    "HOLD_SUCCESSOR_FROZEN",
    "HOLD_EXIT_CANDIDATE",
    "PASS_CANDIDATE",
    "BLOCKED_QUALIFIED_HUMAN_INPUT",
];

const TERMINAL_STATES: [&str; 2] = ["PASS_CANDIDATE", "BLOCKED_QUALIFIED_HUMAN_INPUT"];

const TRANSITIONS: [(&str, &str); 8] = [
    ("HOLD_EPOCH_OPEN", "HOLD_EVIDENCE_COMPLETE"),
    ("HOLD_EVIDENCE_COMPLETE", "HOLD_SUCCESSOR_FROZEN"),
    ("HOLD_SUCCESSOR_FROZEN", "HOLD_EXIT_CANDIDATE"),
    ("HOLD_EXIT_CANDIDATE", "PASS_CANDIDATE"),
    ("HOLD_EPOCH_OPEN", "BLOCKED_QUALIFIED_HUMAN_INPUT"),
    ("HOLD_EVIDENCE_COMPLETE", "BLOCKED_QUALIFIED_HUMAN_INPUT"),
    ("HOLD_SUCCESSOR_FROZEN", "BLOCKED_QUALIFIED_HUMAN_INPUT"),
    ("HOLD_EXIT_CANDIDATE", "BLOCKED_QUALIFIED_HUMAN_INPUT"),
];

const CONTROLS: [(&str, &str, &str); 15] = [
    ("C01", "controlling_adr_chronology", "machine-verifiable"),
    ("C02", "canonical_parser_ir", "machine-verifiable"),
    ("C03", "corpus_archive_freshness", "machine-verifiable"),
    ("C04", "decision_population", "machine-verifiable"),
    ("C05", "comparator", "machine-verifiable"),
    ("C06", "legal_jcr", "qualified-human"),
    ("C07", "affected_party", "qualified-affected-party"),
    ("C08", "operations", "qualified-operations"),
    ("C09", "custody", "qualified-custody"),
    ("C10", "veto", "authorized-veto"),
    ("C11", "pilot", "machine-and-qualified-human"),
    ("C12", "immutable_successor", "machine-verifiable"),
    ("C13", "sixteen_lens_council", "independent-council"),
    ("C14", "fresh_dissent", "independent-dissent"),
    ("C15", "context_free_exit", "independent-oracle"),
];

const LENSES: [(&str, &str); 16] = [
    ("L01", "product_and_user_value"),
    ("L02", "intuitive_no_code_ux_and_accessibility"),
    ("L03", "ontology_data_and_temporal_semantics"),
    ("L04", "workflow_automation_and_compensation"),
    ("L05", "architecture_modularity_and_hyperscale"),
    ("L06", "cloud_platform_and_enterprise_infrastructure"),
    ("L07", "developer_build_release_and_supply_chain"),
    ("L08", "reliability_operations_and_observability"),
    ("L09", "security_identity_and_abuse_resistance"),
    ("L10", "privacy_residency_and_data_governance"),
    ("L11", "legal_regulatory_and_jcr"),
    ("L12", "affected_party_safety_and_ethics"),
    ("L13", "economics_finops_and_business_viability"),
    ("L14", "interoperability_supply_chain_and_ecosystem"),
    ("L15", "maintainability_evolvability_and_deprecation"),
    ("L16", "evidence_audit_governance_and_dissent"),
];

const GROUPS: [(&str, &[&str], &[&str]); 7] = [
    ("A", &["C01", "C02", "C03"], &[]),
    ("B", &["C04", "C05"], &["C01", "C02", "C03"]),
    (
        "C",
        &["C10", "C11"],
        &[
            "C01", "C02", "C03", "C04", "C05", "C06", "C07", "C08", "C09",
        ],
    ),
    ("D", &["C06", "C07", "C08", "C09"], &["C01", "C02", "C03"]),
    (
        "E",
        &[],
        &["C04", "C05", "C06", "C07", "C08", "C09", "C10", "C11"],
    ),
    (
        "F",
        &["C12", "C13", "C14"],
        &[
            "C01", "C02", "C03", "C04", "C05", "C06", "C07", "C08", "C09", "C10", "C11",
        ],
    ),
    (
        "G",
        &["C15"],
        &[
            "C01", "C02", "C03", "C04", "C05", "C06", "C07", "C08", "C09", "C10", "C11", "C12",
            "C13", "C14",
        ],
    ),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    pub findings: Vec<String>,
}

impl Report {
    #[must_use]
    pub fn is_green(&self) -> bool {
        self.findings.is_empty()
    }
}

#[must_use]
pub fn evaluate_program(program: &Value) -> Report {
    let mut findings = BTreeSet::new();
    require_string(program, "schema_id", PROGRAM_SCHEMA_ID, &mut findings);
    require_string(program, "program_id", PROGRAM_ID, &mut findings);
    require_bool(program, "mechanism_neutral", true, &mut findings);
    require_string(program, "initial_state", STATES[0], &mut findings);
    require_exact_string_array(program, "states", &STATES, &mut findings);
    require_exact_string_array(program, "terminal_states", &TERMINAL_STATES, &mut findings);
    validate_transitions(program.get("transitions"), &mut findings);
    validate_groups(program.get("groups"), &mut findings);
    validate_program_controls(program.get("controls"), &mut findings);
    validate_program_lenses(program.get("lenses"), &mut findings);

    let pass_effects = program.get("candidate_effects").unwrap_or(&Value::Null);
    require_bool(
        pass_effects,
        "roadmap_planning_authorized",
        false,
        &mut findings,
    );
    require_bool(
        pass_effects,
        "binding_plan_approval_allowed",
        false,
        &mut findings,
    );
    require_bool(
        pass_effects,
        "implementation_dispatch_allowed",
        false,
        &mut findings,
    );

    let mutation_policy = program.get("mutation_policy").unwrap_or(&Value::Null);
    require_bool(
        mutation_policy,
        "any_program_or_evidence_mutation_opens_new_epoch",
        true,
        &mut findings,
    );
    require_bool(
        mutation_policy,
        "prior_epoch_bytes_remain_only_in_authorized_object_history",
        true,
        &mut findings,
    );
    require_bool(
        mutation_policy,
        "readable_archive_allowed",
        false,
        &mut findings,
    );

    report(findings)
}

#[must_use]
pub fn evaluate_epoch(program: &Value, epoch: &Value) -> Report {
    evaluate_epoch_with_untrusted_shape(program, epoch, Some(&Value::Null))
}

/// Evaluates the complete source-epoch record without assuming externally authenticated facts.
///
/// A valid source fixture remains held until the separate controller and trust-root path exists.
#[must_use]
pub fn evaluate_source_epoch(program: &Value, epoch: &Value) -> Report {
    evaluate_epoch_with_untrusted_shape(program, epoch, None)
}

fn evaluate_epoch_with_untrusted_shape(
    program: &Value,
    epoch: &Value,
    protected_facts: Option<&Value>,
) -> Report {
    let mut findings: BTreeSet<String> = evaluate_program(program)
        .findings
        .into_iter()
        .map(|finding| format!("program.{finding}"))
        .collect();

    require_string(epoch, "schema_id", EPOCH_SCHEMA_ID, &mut findings);
    require_non_empty_string(epoch, "epoch_id", &mut findings);
    require_string(epoch, "program_ref", PROGRAM_REF, &mut findings);
    let state = epoch.get("state").and_then(Value::as_str);
    if state.is_none_or(|candidate| !STATES.contains(&candidate)) {
        findings.insert("state must be one canonical Stage-1 state".to_owned());
    }
    if state != Some("HOLD_EPOCH_OPEN") {
        findings.insert(
            "external authenticated Stage-1 controller and trust root are unimplemented; source evaluation cannot advance beyond HOLD_EPOCH_OPEN"
                .to_owned(),
        );
    }

    validate_subject_binding(epoch.get("subject_binding"), &mut findings);
    validate_planning(epoch.get("planning"), state, &mut findings);

    let control_states = validate_epoch_controls(epoch.get("controls"), &mut findings);
    let lens_state = validate_epoch_lenses(epoch.get("lenses"), &mut findings);
    let dissent_state = validate_fresh_dissent(epoch.get("fresh_dissent"), &mut findings);
    let successor_state =
        validate_immutable_successor(epoch.get("immutable_successor"), &mut findings);
    let exit_state = validate_context_free_exit(epoch.get("context_free_exit"), &mut findings);
    validate_common_subject(
        &control_states,
        &lens_state,
        &dissent_state,
        &successor_state,
        &exit_state,
        &mut findings,
    );
    validate_state_progress(
        state,
        &control_states,
        &lens_state,
        &dissent_state,
        &successor_state,
        &exit_state,
        epoch.get("blockers"),
        &mut findings,
    );
    if let Some(protected_facts) = protected_facts {
        validate_protected_parent_facts(epoch, protected_facts, state, &mut findings);
    }
    require_non_empty_string(epoch, "claim_ceiling", &mut findings);

    report(findings)
}

/// Structural-only validation for a future protected-facts envelope. It never authenticates a
/// producer, verifies signatures, or establishes authority.
#[must_use]
pub fn validate_protected_facts_shape(facts: &Value) -> Report {
    let mut findings = grammar_findings(facts);
    for field in [
        "immutable_successor_bundle",
        "envelope_signature",
        "trust_root_binding",
    ] {
        if !facts.get(field).is_some_and(Value::is_object) {
            findings.insert(format!("{field} must be a bound object"));
        }
    }
    findings.insert("protected-facts structure is non-authoritative; authenticated producer and trust-root verification are unimplemented".to_owned());
    report(findings)
}

/// Pure, comparison-friendly protected-facts grammar validation. A green result only means that
/// the supplied values conform to this local grammar; it never authenticates a producer, checks a
/// signature or trust root, or authorizes planning or dispatch.
#[must_use]
pub fn validate_protected_facts_grammar(facts: &Value) -> Report {
    let mut findings = grammar_findings(facts);
    findings.insert("protected-facts grammar is non-authoritative; authenticated producer and trust-root verification are unimplemented".to_owned());
    report(findings)
}

fn grammar_findings(facts: &Value) -> BTreeSet<String> {
    let mut findings = BTreeSet::new();
    require_string(facts, "schema_id", PROTECTED_FACTS_SCHEMA_ID, &mut findings);
    let required_bindings = [
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
    ];
    for field in [
        "protected_base_repository",
        "candidate_repository",
        "protected_base_branch",
        "candidate_branch",
    ] {
        require_non_empty_string(facts, field, &mut findings);
    }
    for field in [
        "protected_base_commit",
        "candidate_commit",
        "protected_base_tree",
        "candidate_tree",
    ] {
        if facts
            .get(field)
            .and_then(Value::as_str)
            .is_none_or(|value| !is_hex(value, 40) && !is_hex(value, 64))
        {
            findings.insert(format!("{field} must be a Git object identifier"));
        }
    }
    for field in required_bindings {
        validate_artifact_binding(
            facts.get(field).unwrap_or(&Value::Null),
            field,
            &mut findings,
        );
    }
    let allowed = [
        "schema_id",
        "subject_digest",
        "program_digest",
        "epoch_digest",
        "protected_base_repository",
        "candidate_repository",
        "protected_base_branch",
        "candidate_branch",
        "protected_base_commit",
        "candidate_commit",
        "protected_base_tree",
        "candidate_tree",
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
        "receipt_bindings",
        "extensions",
    ];
    if let Some(object) = facts.as_object() {
        for key in object.keys() {
            if !allowed.contains(&key.as_str()) {
                findings.insert(format!("protected facts rejects unknown field {key}"));
            }
        }
    }
    for field in ["subject_digest", "program_digest", "epoch_digest"] {
        if valid_subject_digest(facts.get(field)).is_none() {
            findings.insert(format!("{field} must be sha256-bound"));
        }
    }
    let Some(receipts) = facts.get("receipt_bindings").and_then(Value::as_array) else {
        findings.insert("receipt_bindings must be an array".to_owned());
        return findings;
    };
    let required = [
        (
            "C06",
            "qualified-legal-compliance",
            "qualified-human",
            "legal-jcr",
        ),
        (
            "C07",
            "affected-party-representation",
            "qualified-affected-party",
            "affected-party-recourse",
        ),
        (
            "C08",
            "operations-owner-capacity",
            "qualified-operations",
            "named-operations-capacity",
        ),
        (
            "C09",
            "security-evidence-custody",
            "qualified-custody",
            "security-evidence-custody",
        ),
        (
            "C10",
            "veto-owner-closure",
            "authorized-veto",
            "exact-veto-owner",
        ),
        (
            "C11",
            "machine-pilot-evidence",
            "machine-verifiable",
            "machine-pilot-evidence",
        ),
        (
            "C11",
            "qualified-pilot-authorization",
            "qualified-human",
            "qualified-pilot-authorization",
        ),
        (
            "C14",
            "fresh-dissent",
            "independent-dissent",
            "fresh-independent-dissent",
        ),
        (
            "C15",
            "deterministic-oracle",
            "independent-oracle",
            "deterministic-oracle",
        ),
        (
            "C15",
            "blind-cold-reader",
            "independent-oracle",
            "blind-cold-reader",
        ),
        (
            "C15",
            "qualified-planning-authority",
            "independent-oracle",
            "qualified-planning-authority",
        ),
    ];
    for (control, role, authority_class, qualification) in required {
        let matching = receipts
            .iter()
            .filter(|receipt| {
                receipt.get("control_id").and_then(Value::as_str) == Some(control)
                    && receipt.get("role").and_then(Value::as_str) == Some(role)
            })
            .collect::<Vec<_>>();
        if matching.len() != 1 {
            findings.insert(format!("{control}/{role} requires exactly one receipt"));
            continue;
        }
        let receipt = matching[0];
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
            validate_artifact_binding(
                receipt.get(field).unwrap_or(&Value::Null),
                field,
                &mut findings,
            );
        }
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
            validate_artifact_binding(
                receipt.get(field).unwrap_or(&Value::Null),
                field,
                &mut findings,
            );
        }
        if !matches!(
            receipt.get("decision").and_then(Value::as_str),
            Some("satisfied" | "blocked" | "abstained" | "dissent")
        ) {
            findings.insert(format!("{control}/{role} decision must be declared"));
        }
        for field in ["path", "blob_oid", "sha256"] {
            if receipt.get(field).is_none() {
                findings.insert(format!("{control}/{role} {field} is required"));
            }
        }
        if receipt
            .get("issuer_authority_class")
            .and_then(Value::as_str)
            != Some(authority_class)
        {
            findings.insert(format!(
                "{control}/{role} issuer_authority_class must equal {authority_class}"
            ));
        }
        if receipt.get("qualification_class").and_then(Value::as_str) != Some(qualification) {
            findings.insert(format!(
                "{control}/{role} qualification_class must equal {qualification}"
            ));
        }
        if receipt.get("subject_digest") != facts.get("subject_digest")
            || receipt.get("program_digest") != facts.get("program_digest")
            || receipt.get("epoch_digest") != facts.get("epoch_digest")
        {
            findings.insert(format!(
                "{control}/{role} must match subject/program/epoch digests"
            ));
        }
        if receipt
            .get("principal_id")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            findings.insert(format!("{control}/{role} principal_id must be non-empty"));
        }
    }
    let c11_count = receipts
        .iter()
        .filter(|receipt| receipt.get("control_id").and_then(Value::as_str) == Some("C11"))
        .count();
    if c11_count != 2 {
        findings.insert("C11 permits exactly its machine and qualified-pilot receipts".to_owned());
    }
    let mut lens_principals = BTreeSet::new();
    for lens in LENSES.map(|(lens, _)| lens) {
        let matching = receipts
            .iter()
            .filter(|receipt| {
                receipt.get("control_id").and_then(Value::as_str) == Some("C13")
                    && receipt.get("lens_id").and_then(Value::as_str) == Some(lens)
            })
            .collect::<Vec<_>>();
        if matching.len() != 1 {
            findings.insert(format!(
                "C13/{lens} requires exactly one independent lens receipt"
            ));
            continue;
        }
        let receipt = matching[0];
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
            validate_artifact_binding(
                receipt.get(field).unwrap_or(&Value::Null),
                field,
                &mut findings,
            );
        }
        if receipt.get("role").and_then(Value::as_str) != Some("independent-council")
            || receipt
                .get("issuer_authority_class")
                .and_then(Value::as_str)
                != Some("independent-council")
            || receipt.get("qualification_class").and_then(Value::as_str)
                != Some("independent-lens-reviewer")
        {
            findings.insert(format!(
                "C13/{lens} requires independent-council reviewer authority"
            ));
        }
        if receipt.get("subject_digest") != facts.get("subject_digest")
            || receipt.get("program_digest") != facts.get("program_digest")
            || receipt.get("epoch_digest") != facts.get("epoch_digest")
        {
            findings.insert(format!(
                "C13/{lens} must match subject/program/epoch digests"
            ));
        }
        match receipt.get("principal_id").and_then(Value::as_str) {
            Some(principal) if lens_principals.insert(principal) => {}
            _ => {
                findings.insert("C13 requires sixteen distinct principal_id values".to_owned());
            }
        }
    }
    if receipts
        .iter()
        .filter(|receipt| receipt.get("control_id").and_then(Value::as_str) == Some("C13"))
        .count()
        != 16
    {
        findings.insert("C13 permits exactly sixteen lens receipts".to_owned());
    }
    let c15_principals = receipts
        .iter()
        .filter(|receipt| receipt.get("control_id").and_then(Value::as_str) == Some("C15"))
        .filter_map(|receipt| receipt.get("principal_id").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();
    if c15_principals.len() != 3 {
        findings.insert("C15 requires three distinct principal_id values".to_owned());
    }
    if receipts
        .iter()
        .filter(|receipt| receipt.get("control_id").and_then(Value::as_str) == Some("C15"))
        .count()
        != 3
    {
        findings.insert("C15 permits exactly three exit receipts".to_owned());
    }
    findings
}

/// Structural-only validation for a future post-merge admission envelope. It never derives an
/// effective planning PASS; an external authenticated controller remains required.
#[must_use]
pub fn validate_admission_envelope_shape(envelope: &Value) -> Report {
    let mut findings = BTreeSet::new();
    require_string(
        envelope,
        "schema_id",
        "oyatie/stage1-admission-envelope/v1",
        &mut findings,
    );
    for field in ["repository", "branch"] {
        require_non_empty_string(envelope, field, &mut findings);
    }
    for field in [
        "base_commit",
        "base_tree",
        "pr_head_commit",
        "pr_head_tree",
        "postmerge_promoted_commit",
        "postmerge_promoted_tree",
    ] {
        if envelope
            .get(field)
            .and_then(Value::as_str)
            .is_none_or(|value| !is_hex(value, 40) && !is_hex(value, 64))
        {
            findings.insert(format!("{field} must be a Git object identifier"));
        }
    }
    for field in [
        "facts_binding",
        "program_binding",
        "evaluator_binding",
        "policy_binding",
        "schema_binding",
        "immutable_successor_binding",
        "check_suite_binding",
        "independent_review_binding",
        "branch_protection_binding",
        "postmerge_completion",
        "envelope_signature",
        "trust_root_binding",
    ] {
        validate_admission_binding(
            envelope.get(field).unwrap_or(&Value::Null),
            field,
            &mut findings,
        );
    }
    if envelope.get("postmerge_promoted_commit") == envelope.get("pr_head_commit") {
        findings.insert("postmerge_promoted_commit must not equal pr_head_commit".to_owned());
    }
    if let Some(object) = envelope.as_object() {
        for field in object.keys() {
            if !ADMISSION_ENVELOPE_ALLOWED_FIELDS.contains(&field.as_str()) {
                findings.insert(format!("admission envelope rejects unknown field {field}"));
            }
        }
    }
    if let Some(extensions) = envelope.get("extensions") {
        match extensions.as_object() {
            Some(extensions) => {
                for field in extensions.keys() {
                    if !field.starts_with("x-stage1-non-authoritative-") {
                        findings.insert(format!("extensions rejects unknown field {field}"));
                    }
                }
            }
            None => {
                findings.insert("extensions must be an object".to_owned());
            }
        }
    }
    require_bool(
        envelope,
        "roadmap_planning_authorized",
        false,
        &mut findings,
    );
    require_bool(
        envelope,
        "binding_plan_approval_allowed",
        false,
        &mut findings,
    );
    require_bool(
        envelope,
        "implementation_dispatch_allowed",
        false,
        &mut findings,
    );
    report_with_admission_hold(findings)
}

fn report_with_admission_hold(mut findings: BTreeSet<String>) -> Report {
    findings.insert("admission-envelope structure is non-authoritative; authenticated external controller is unimplemented".to_owned());
    report(findings)
}

fn validate_admission_binding(value: &Value, field: &str, findings: &mut BTreeSet<String>) {
    let Some(binding) = value.as_object() else {
        findings.insert(format!("{field} must be a binding object"));
        return;
    };
    if binding
        .get("path")
        .and_then(Value::as_str)
        .is_none_or(|candidate| candidate.trim().is_empty())
    {
        findings.insert(format!("{field}.path must be a non-empty string"));
    }
    if binding
        .get("blob_oid")
        .and_then(Value::as_str)
        .is_none_or(|candidate| !is_hex(candidate, 40) && !is_hex(candidate, 64))
    {
        findings.insert(format!("{field}.blob_oid must be a Git object identifier"));
    }
    if binding
        .get("sha256")
        .and_then(Value::as_str)
        .is_none_or(|candidate| !is_hex(candidate, 64))
    {
        findings.insert(format!("{field}.sha256 must be 64 hex bytes"));
    }
    for key in binding.keys() {
        if !matches!(key.as_str(), "path" | "blob_oid" | "sha256") {
            findings.insert(format!("{field} rejects unknown field {key}"));
        }
    }
}

#[derive(Clone, Debug, Default)]
struct EvidenceState {
    satisfied: bool,
    blocked: bool,
    subject_digest: Option<String>,
    principals: BTreeSet<String>,
}

fn report(findings: BTreeSet<String>) -> Report {
    Report {
        findings: findings.into_iter().collect(),
    }
}

fn require_string(value: &Value, field: &str, expected: &str, findings: &mut BTreeSet<String>) {
    if value.get(field).and_then(Value::as_str) != Some(expected) {
        findings.insert(format!("{field} must equal {expected}"));
    }
}

fn require_non_empty_string(value: &Value, field: &str, findings: &mut BTreeSet<String>) {
    if value
        .get(field)
        .and_then(Value::as_str)
        .is_none_or(|candidate| candidate.trim().is_empty())
    {
        findings.insert(format!("{field} must be a non-empty string"));
    }
}

fn require_bool(value: &Value, field: &str, expected: bool, findings: &mut BTreeSet<String>) {
    if value.get(field).and_then(Value::as_bool) != Some(expected) {
        findings.insert(format!("{field} must equal {expected}"));
    }
}

fn require_exact_string_array(
    value: &Value,
    field: &str,
    expected: &[&str],
    findings: &mut BTreeSet<String>,
) {
    let actual = value
        .get(field)
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).collect::<Vec<_>>());
    if actual.as_deref() != Some(expected) {
        findings.insert(format!("{field} must equal the canonical ordered set"));
    }
}

fn validate_transitions(value: Option<&Value>, findings: &mut BTreeSet<String>) {
    let Some(transitions) = value.and_then(Value::as_array) else {
        findings.insert("transitions must be the canonical transition array".to_owned());
        return;
    };
    let actual = transitions
        .iter()
        .filter_map(|transition| {
            Some((
                transition.get("from")?.as_str()?,
                transition.get("to")?.as_str()?,
            ))
        })
        .collect::<Vec<_>>();
    if actual != TRANSITIONS {
        findings.insert("transitions must equal the canonical fail-closed graph".to_owned());
    }
}

fn validate_groups(value: Option<&Value>, findings: &mut BTreeSet<String>) {
    let Some(groups) = value.and_then(Value::as_array) else {
        findings.insert("groups must contain exact A-G records".to_owned());
        return;
    };
    if groups.len() != GROUPS.len() {
        findings.insert("groups must contain exactly A-G".to_owned());
    }
    for (index, (group_id, owned, required)) in GROUPS.iter().enumerate() {
        let Some(group) = groups.get(index) else {
            findings.insert(format!("groups missing {group_id}"));
            continue;
        };
        if group.get("group_id").and_then(Value::as_str) != Some(*group_id) {
            findings.insert(format!("groups[{index}].group_id must equal {group_id}"));
        }
        require_exact_string_array(group, "owned_controls", owned, findings);
        require_exact_string_array(group, "requires_controls", required, findings);
    }
    let owned = groups
        .iter()
        .filter_map(|group| group.get("owned_controls")?.as_array())
        .flatten()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    let owned_set = owned.iter().copied().collect::<BTreeSet<_>>();
    let expected = CONTROLS
        .iter()
        .map(|(id, _, _)| *id)
        .collect::<BTreeSet<_>>();
    if owned.len() != CONTROLS.len() || owned_set != expected {
        findings.insert("groups.owned_controls must partition C01-C15 exactly once".to_owned());
    }
}

fn validate_program_controls(value: Option<&Value>, findings: &mut BTreeSet<String>) {
    let Some(controls) = value.and_then(Value::as_array) else {
        findings.insert("controls must contain exact C01-C15 records".to_owned());
        return;
    };
    if controls.len() != CONTROLS.len() {
        findings.insert("controls must contain exactly C01-C15".to_owned());
    }
    let mut ids = BTreeSet::new();
    for (index, (id, name, authority_class)) in CONTROLS.iter().enumerate() {
        let Some(control) = controls.get(index) else {
            findings.insert(format!("controls missing {id}"));
            continue;
        };
        let actual_id = control.get("control_id").and_then(Value::as_str);
        if actual_id != Some(*id) {
            findings.insert(format!("controls[{index}].control_id must equal {id}"));
        }
        if let Some(actual_id) = actual_id
            && !ids.insert(actual_id)
        {
            findings.insert(format!("controls duplicate control_id {actual_id}"));
        }
        if control.get("name").and_then(Value::as_str) != Some(*name) {
            findings.insert(format!("controls.{id}.name must equal {name}"));
        }
        if control.get("authority_class").and_then(Value::as_str) != Some(*authority_class) {
            findings.insert(format!(
                "controls.{id}.authority_class must equal {authority_class}"
            ));
        }
    }
}

fn validate_program_lenses(value: Option<&Value>, findings: &mut BTreeSet<String>) {
    let Some(lenses) = value.and_then(Value::as_array) else {
        findings.insert("lenses must contain exact L01-L16 records".to_owned());
        return;
    };
    if lenses.len() != LENSES.len() {
        findings.insert("lenses must contain exactly L01-L16".to_owned());
    }
    let mut ids = BTreeSet::new();
    for (index, (id, name)) in LENSES.iter().enumerate() {
        let Some(lens) = lenses.get(index) else {
            findings.insert(format!("lenses missing {id}"));
            continue;
        };
        let actual_id = lens.get("lens_id").and_then(Value::as_str);
        if actual_id != Some(*id) {
            findings.insert(format!("lenses[{index}].lens_id must equal {id}"));
        }
        if let Some(actual_id) = actual_id
            && !ids.insert(actual_id)
        {
            findings.insert(format!("lenses duplicate lens_id {actual_id}"));
        }
        if lens.get("name").and_then(Value::as_str) != Some(*name) {
            findings.insert(format!("lenses.{id}.name must equal {name}"));
        }
    }
}

fn validate_subject_binding(value: Option<&Value>, findings: &mut BTreeSet<String>) {
    let subject = value.unwrap_or(&Value::Null);
    require_string(subject, "facts_schema", PROTECTED_FACTS_SCHEMA_ID, findings);
    require_string(subject, "facts_field", "protected_facts_ref", findings);
    require_bool(subject, "required", true, findings);
}

fn validate_planning(value: Option<&Value>, _state: Option<&str>, findings: &mut BTreeSet<String>) {
    let planning = value.unwrap_or(&Value::Null);
    require_string(planning, "planning_state", "HOLD(Planning)", findings);
    require_bool(planning, "roadmap_planning_authorized", false, findings);
    require_bool(planning, "binding_plan_approval_allowed", false, findings);
    require_bool(planning, "implementation_dispatch_allowed", false, findings);
}

fn validate_epoch_controls(
    value: Option<&Value>,
    findings: &mut BTreeSet<String>,
) -> BTreeMap<String, EvidenceState> {
    let Some(controls) = value.and_then(Value::as_array) else {
        findings.insert("controls must contain exact C01-C15 evidence records".to_owned());
        return BTreeMap::new();
    };
    if controls.len() != CONTROLS.len() {
        findings.insert("controls must contain exactly C01-C15 evidence records".to_owned());
    }
    let mut states = BTreeMap::new();
    let mut ids = BTreeSet::new();
    for (index, (id, _, authority_class)) in CONTROLS.iter().enumerate() {
        let Some(control) = controls.get(index) else {
            findings.insert(format!("controls missing {id}"));
            continue;
        };
        let actual_id = control.get("control_id").and_then(Value::as_str);
        if actual_id != Some(*id) {
            findings.insert(format!("controls[{index}].control_id must equal {id}"));
        }
        if let Some(actual_id) = actual_id
            && !ids.insert(actual_id)
        {
            findings.insert(format!("controls duplicate control_id {actual_id}"));
        }
        let status = control.get("status").and_then(Value::as_str);
        if status.is_none_or(|candidate| !matches!(candidate, "pending" | "satisfied" | "blocked"))
        {
            findings.insert(format!("controls.{id}.status is invalid"));
        }
        let mut state = EvidenceState {
            satisfied: status == Some("satisfied"),
            blocked: status == Some("blocked"),
            subject_digest: valid_subject_digest(control.get("subject_digest")),
            principals: BTreeSet::new(),
        };
        if state.satisfied {
            let Some(subject_digest) = state.subject_digest.as_deref() else {
                findings.insert(format!("controls.{id}.subject_digest must be sha256-bound"));
                states.insert((*id).to_owned(), state);
                continue;
            };
            let Some(receipts) = control.get("receipt_refs").and_then(Value::as_array) else {
                findings.insert(format!("controls.{id}.receipt_refs must be non-empty"));
                states.insert((*id).to_owned(), state);
                continue;
            };
            if receipts.is_empty() {
                findings.insert(format!("controls.{id}.receipt_refs must be non-empty"));
            }
            for (receipt_index, receipt) in receipts.iter().enumerate() {
                if let Some(principal) = validate_receipt(
                    receipt,
                    &format!("controls.{id}.receipt_refs[{receipt_index}]"),
                    subject_digest,
                    authority_class,
                    findings,
                ) {
                    state.principals.insert(principal);
                }
            }
        }
        states.insert((*id).to_owned(), state);
    }
    states
}

fn validate_epoch_lenses(
    value: Option<&Value>,
    findings: &mut BTreeSet<String>,
) -> BTreeMap<String, EvidenceState> {
    let Some(lenses) = value.and_then(Value::as_array) else {
        findings.insert("lenses must contain exact L01-L16 evidence records".to_owned());
        return BTreeMap::new();
    };
    if lenses.len() != LENSES.len() {
        findings.insert("lenses must contain exactly L01-L16 evidence records".to_owned());
    }
    let mut states = BTreeMap::new();
    let mut reviewer_ids = BTreeSet::new();
    for (index, (id, _)) in LENSES.iter().enumerate() {
        let Some(lens) = lenses.get(index) else {
            findings.insert(format!("lenses missing {id}"));
            continue;
        };
        if lens.get("lens_id").and_then(Value::as_str) != Some(*id) {
            findings.insert(format!("lenses[{index}].lens_id must equal {id}"));
        }
        let status = lens.get("status").and_then(Value::as_str);
        if status.is_none_or(|candidate| !matches!(candidate, "pending" | "satisfied" | "blocked"))
        {
            findings.insert(format!("lenses.{id}.status is invalid"));
        }
        let mut state = EvidenceState {
            satisfied: status == Some("satisfied"),
            blocked: status == Some("blocked"),
            subject_digest: valid_subject_digest(lens.get("subject_digest")),
            principals: BTreeSet::new(),
        };
        if state.satisfied {
            let reviewer_id = lens
                .get("reviewer_id")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty());
            let Some(reviewer_id) = reviewer_id else {
                findings.insert(format!("lenses.{id}.reviewer_id must be non-empty"));
                states.insert((*id).to_owned(), state);
                continue;
            };
            if !reviewer_ids.insert(reviewer_id.to_owned()) {
                findings.insert(format!(
                    "lenses.{id}.reviewer_id must be independent and unique"
                ));
            }
            if lens.get("independent_from_author").and_then(Value::as_bool) != Some(true) {
                findings.insert(format!("lenses.{id}.independent_from_author must be true"));
            }
            if lens.get("fresh_context").and_then(Value::as_bool) != Some(true) {
                findings.insert(format!("lenses.{id}.fresh_context must be true"));
            }
            let Some(subject_digest) = state.subject_digest.as_deref() else {
                findings.insert(format!("lenses.{id}.subject_digest must be sha256-bound"));
                states.insert((*id).to_owned(), state);
                continue;
            };
            let Some(receipt) = lens.get("receipt_ref") else {
                findings.insert(format!("lenses.{id}.receipt_ref is required"));
                states.insert((*id).to_owned(), state);
                continue;
            };
            if validate_receipt(
                receipt,
                &format!("lenses.{id}.receipt_ref"),
                subject_digest,
                "independent-council",
                findings,
            )
            .as_deref()
                != Some(reviewer_id)
            {
                findings.insert(format!(
                    "lenses.{id}.receipt_ref principal_id must equal reviewer_id"
                ));
            }
            state.principals.insert(reviewer_id.to_owned());
        }
        states.insert((*id).to_owned(), state);
    }
    states
}

fn validate_fresh_dissent(value: Option<&Value>, findings: &mut BTreeSet<String>) -> EvidenceState {
    let dissent = value.unwrap_or(&Value::Null);
    let status = dissent.get("status").and_then(Value::as_str);
    if status.is_none_or(|candidate| !matches!(candidate, "pending" | "satisfied" | "blocked")) {
        findings.insert("fresh_dissent.status is invalid".to_owned());
    }
    let mut state = EvidenceState {
        satisfied: status == Some("satisfied"),
        blocked: status == Some("blocked"),
        subject_digest: valid_subject_digest(dissent.get("subject_digest")),
        principals: BTreeSet::new(),
    };
    if !state.satisfied {
        return state;
    }
    let reviewer_id = dissent
        .get("reviewer_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty());
    let Some(reviewer_id) = reviewer_id else {
        findings.insert("fresh_dissent.reviewer_id must be non-empty".to_owned());
        return state;
    };
    state.principals.insert(reviewer_id.to_owned());
    require_bool(dissent, "fresh_context", true, findings);
    require_bool(dissent, "prior_context_used", false, findings);
    require_bool(dissent, "findings_resolved_or_carried", true, findings);
    let Some(subject_digest) = state.subject_digest.as_deref() else {
        findings.insert("fresh_dissent.subject_digest must be sha256-bound".to_owned());
        return state;
    };
    let Some(receipt) = dissent.get("receipt_ref") else {
        findings.insert("fresh_dissent.receipt_ref is required".to_owned());
        return state;
    };
    if validate_receipt(
        receipt,
        "fresh_dissent.receipt_ref",
        subject_digest,
        "independent-dissent",
        findings,
    )
    .as_deref()
        != Some(reviewer_id)
    {
        findings.insert(
            "fresh_dissent.receipt_ref principal_id must equal fresh_dissent.reviewer_id"
                .to_owned(),
        );
    }
    state
}

fn validate_immutable_successor(
    value: Option<&Value>,
    findings: &mut BTreeSet<String>,
) -> EvidenceState {
    let successor = value.unwrap_or(&Value::Null);
    let frozen = successor.get("frozen").and_then(Value::as_bool) == Some(true);
    let state = EvidenceState {
        satisfied: frozen,
        blocked: false,
        subject_digest: valid_subject_digest(successor.get("subject_digest")),
        principals: BTreeSet::new(),
    };
    if frozen {
        if state.subject_digest.is_none() {
            findings.insert("immutable_successor.subject_digest must be sha256-bound".to_owned());
        }
        let facts_ref = successor.get("facts_ref").unwrap_or(&Value::Null);
        require_non_empty_string(facts_ref, "path", findings);
        if facts_ref
            .get("sha256")
            .and_then(Value::as_str)
            .is_none_or(|value| !is_hex(value, 64))
        {
            findings.insert("immutable_successor.facts_ref.sha256 must be 64 hex bytes".to_owned());
        }
    }
    state
}

fn validate_context_free_exit(
    value: Option<&Value>,
    findings: &mut BTreeSet<String>,
) -> EvidenceState {
    let exit = value.unwrap_or(&Value::Null);
    let status = exit.get("status").and_then(Value::as_str);
    if status.is_none_or(|candidate| !matches!(candidate, "pending" | "satisfied" | "blocked")) {
        findings.insert("context_free_exit.status is invalid".to_owned());
    }
    let mut state = EvidenceState {
        satisfied: status == Some("satisfied"),
        blocked: status == Some("blocked"),
        subject_digest: valid_subject_digest(exit.get("subject_digest")),
        principals: BTreeSet::new(),
    };
    if !state.satisfied {
        return state;
    }
    let oracle = non_empty(exit, "oracle_principal_id");
    let blind_reader = non_empty(exit, "blind_reader_principal_id");
    if oracle.is_none() {
        findings.insert("context_free_exit.oracle_principal_id must be non-empty".to_owned());
    }
    if blind_reader.is_none() {
        findings.insert("context_free_exit.blind_reader_principal_id must be non-empty".to_owned());
    }
    if oracle.is_some() && oracle == blind_reader {
        findings.insert(
            "context_free_exit oracle and blind reader principals must be distinct".to_owned(),
        );
    }
    require_bool(exit, "conversation_context_used", false, findings);
    require_string(exit, "reproduced_verdict", "PASS_CANDIDATE", findings);
    let Some(subject_digest) = state.subject_digest.as_deref() else {
        findings.insert("context_free_exit.subject_digest must be sha256-bound".to_owned());
        return state;
    };
    if let Some(oracle) = oracle {
        state.principals.insert(oracle.to_owned());
        let receipt = exit.get("oracle_receipt_ref").unwrap_or(&Value::Null);
        if validate_receipt(
            receipt,
            "context_free_exit.oracle_receipt_ref",
            subject_digest,
            "independent-oracle",
            findings,
        )
        .as_deref()
            != Some(oracle)
        {
            findings.insert(
                "context_free_exit.oracle_receipt_ref principal_id must equal oracle_principal_id"
                    .to_owned(),
            );
        }
    }
    if let Some(blind_reader) = blind_reader {
        state.principals.insert(blind_reader.to_owned());
        let receipt = exit.get("blind_reader_receipt_ref").unwrap_or(&Value::Null);
        if validate_receipt(
            receipt,
            "context_free_exit.blind_reader_receipt_ref",
            subject_digest,
            "independent-reader",
            findings,
        )
        .as_deref()
            != Some(blind_reader)
        {
            findings.insert(
                "context_free_exit.blind_reader_receipt_ref principal_id must equal blind_reader_principal_id"
                    .to_owned(),
            );
        }
    }
    state
}

fn validate_receipt(
    receipt: &Value,
    path: &str,
    expected_subject: &str,
    expected_class: &str,
    findings: &mut BTreeSet<String>,
) -> Option<String> {
    if let Some(object) = receipt.as_object() {
        for field in object.keys() {
            if !SOURCE_RECEIPT_REQUIRED_FIELDS.contains(&field.as_str()) {
                findings.insert(format!("{path} rejects unknown field {field}"));
            }
        }
    }
    if non_empty(receipt, "path").is_none() {
        findings.insert(format!("{path}.path must be non-empty"));
    }
    if receipt
        .get("blob_oid")
        .and_then(Value::as_str)
        .is_none_or(|value| !is_hex(value, 40) && !is_hex(value, 64))
    {
        findings.insert(format!("{path}.blob_oid must be a Git object identifier"));
    }
    if receipt
        .get("sha256")
        .and_then(Value::as_str)
        .is_none_or(|value| !is_hex(value, 64))
    {
        findings.insert(format!("{path}.sha256 must be 64 hex bytes"));
    }
    if receipt.get("subject_digest").and_then(Value::as_str) != Some(expected_subject) {
        findings.insert(format!(
            "{path}.subject_digest must match the frozen subject"
        ));
    }
    let principal_id = non_empty(receipt, "principal_id").map(str::to_owned);
    if principal_id.is_none() {
        findings.insert(format!("{path}.principal_id must be non-empty"));
    }
    let receipt_class = receipt
        .get("issuer_authority_class")
        .and_then(Value::as_str);
    let valid_class = if expected_class == "machine-and-qualified-human" {
        matches!(
            receipt_class,
            Some("machine-verifiable" | "qualified-human")
        )
    } else {
        receipt_class == Some(expected_class)
    };
    if !valid_class {
        findings.insert(format!(
            "{path}.issuer_authority_class must equal {expected_class}"
        ));
    }
    if !receipt
        .get("authority_source_ref")
        .is_some_and(Value::is_object)
    {
        findings.insert(format!("{path}.authority_source_ref must be bound"));
    } else {
        validate_artifact_binding(
            receipt.get("authority_source_ref").unwrap_or(&Value::Null),
            &format!("{path}.authority_source_ref"),
            findings,
        );
    }
    for field in [
        "qualification",
        "jurisdiction_scope",
        "independence_observation",
        "validity",
        "revocation_status",
        "conflict_status",
        "signature_trust_root_binding",
    ] {
        if non_empty(receipt, field).is_none() {
            findings.insert(format!("{path}.{field} must be non-empty"));
        }
    }
    principal_id
}

fn validate_protected_parent_facts(
    epoch: &Value,
    facts: &Value,
    state: Option<&str>,
    findings: &mut BTreeSet<String>,
) {
    if state == Some("HOLD_EPOCH_OPEN") {
        return;
    }
    findings.extend(grammar_findings(facts));

    require_string(facts, "schema_id", PROTECTED_FACTS_SCHEMA_ID, findings);
    validate_artifact_binding(
        facts.get("trust_root_binding").unwrap_or(&Value::Null),
        "trust_root_binding",
        findings,
    );
    require_non_empty_string(facts, "protected_base_repository", findings);
    require_non_empty_string(facts, "candidate_repository", findings);
    for field in [
        "protected_base_commit",
        "candidate_commit",
        "protected_base_tree",
        "candidate_tree",
    ] {
        if facts
            .get(field)
            .and_then(Value::as_str)
            .is_none_or(|value| !is_hex(value, 40) && !is_hex(value, 64))
        {
            findings.insert(format!(
                "protected facts {field} must be a Git object identifier"
            ));
        }
    }
    for field in [
        "program_binding",
        "schema_binding",
        "policy_binding",
        "parser_binding",
        "producer_binding",
        "evaluator_binding",
        "predecessor_epoch_binding",
        "transition_receipt_binding",
        "immutable_successor_bundle",
        "authority_chain_result",
    ] {
        validate_artifact_binding(facts.get(field).unwrap_or(&Value::Null), field, findings);
    }

    let subject = facts.get("subject_digest").and_then(Value::as_str);
    if valid_subject_digest(facts.get("subject_digest")).is_none() {
        findings.insert("protected facts subject_digest must be sha256-bound".to_owned());
    }
    let receipts = facts.get("receipt_bindings").and_then(Value::as_array);
    let Some(receipts) = receipts else {
        findings.insert(
            "state advancement requires independently supplied receipt_bindings".to_owned(),
        );
        return;
    };

    for (control_index, control) in epoch
        .get("controls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let control_id = CONTROLS
            .get(control_index)
            .map_or("unknown", |(id, _, _)| *id);
        let source_receipts = control
            .get("receipt_refs")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        for receipt in source_receipts {
            let Some(binding) = matching_protected_receipt(receipts, receipt) else {
                findings.insert(format!(
                    "controls.{control_id} receipt lacks an exact protected-parent binding"
                ));
                continue;
            };
            validate_receipt_binding(binding, control_id, subject, findings);
        }
        if control_id == "C11" && control.get("status").and_then(Value::as_str) == Some("satisfied")
        {
            let machine = source_receipts.iter().any(|receipt| {
                receipt
                    .get("issuer_authority_class")
                    .and_then(Value::as_str)
                    == Some("machine-verifiable")
                    && matching_protected_receipt(receipts, receipt).is_some()
            });
            let human = source_receipts.iter().any(|receipt| {
                receipt
                    .get("issuer_authority_class")
                    .and_then(Value::as_str)
                    == Some("qualified-human")
                    && matching_protected_receipt(receipts, receipt).is_some()
            });
            if !machine || !human {
                findings.insert(
                    "C11 requires independently bound machine and qualified-human evidence"
                        .to_owned(),
                );
            }
        }
    }

    for receipt in epoch_receipts(epoch) {
        if matching_protected_receipt(receipts, receipt).is_none() {
            findings.insert(
                "state advancement receipt lacks an exact protected-parent binding".to_owned(),
            );
        }
    }
}

fn validate_artifact_binding(value: &Value, field: &str, findings: &mut BTreeSet<String>) {
    require_non_empty_string(value, "path", findings);
    if value
        .get("blob_oid")
        .and_then(Value::as_str)
        .is_none_or(|candidate| !is_hex(candidate, 40) && !is_hex(candidate, 64))
    {
        findings.insert(format!(
            "protected facts {field}.blob_oid must be a Git object identifier"
        ));
    }
    if value
        .get("sha256")
        .and_then(Value::as_str)
        .is_none_or(|candidate| !is_hex(candidate, 64))
    {
        findings.insert(format!(
            "protected facts {field}.sha256 must be 64 hex bytes"
        ));
    }
}

fn matching_protected_receipt<'a>(bindings: &'a [Value], receipt: &Value) -> Option<&'a Value> {
    bindings.iter().find(|binding| {
        [
            "path",
            "blob_oid",
            "sha256",
            "subject_digest",
            "principal_id",
            "issuer_authority_class",
            "authority_source_ref",
        ]
        .iter()
        .all(|field| binding.get(*field) == receipt.get(*field))
    })
}

fn validate_receipt_binding(
    binding: &Value,
    control_id: &str,
    expected_subject: Option<&str>,
    findings: &mut BTreeSet<String>,
) {
    if binding.get("control_id").and_then(Value::as_str) != Some(control_id) {
        findings.insert(format!("protected receipt binding must name {control_id}"));
    }
    if binding.get("subject_digest").and_then(Value::as_str) != expected_subject {
        findings.insert("protected receipt binding must match protected subject_digest".to_owned());
    }
    if !binding
        .get("independence_observation")
        .is_some_and(Value::is_object)
    {
        findings.insert("protected receipt binding must bind independence_observation".to_owned());
    }
    validate_artifact_binding(
        binding
            .get("signature_trust_root_binding")
            .unwrap_or(&Value::Null),
        "signature_trust_root_binding",
        findings,
    );
    if matches!(control_id, "C06" | "C07" | "C08" | "C09" | "C10" | "C11") {
        require_non_empty_string(binding, "qualification", findings);
        validate_artifact_binding(
            binding.get("jurisdiction_scope").unwrap_or(&Value::Null),
            "jurisdiction_scope",
            findings,
        );
    }
}

fn epoch_receipts(epoch: &Value) -> Vec<&Value> {
    let mut receipts = Vec::new();
    if let Some(controls) = epoch.get("controls").and_then(Value::as_array) {
        receipts.extend(
            controls
                .iter()
                .filter_map(|control| control.get("receipt_refs")?.as_array())
                .flatten(),
        );
    }
    if let Some(lenses) = epoch.get("lenses").and_then(Value::as_array) {
        receipts.extend(
            lenses
                .iter()
                .filter_map(|lens| lens.get("receipt_ref"))
                .filter(|receipt| !receipt.is_null()),
        );
    }
    for field in ["fresh_dissent", "context_free_exit"] {
        if let Some(value) = epoch.get(field) {
            for receipt_field in [
                "receipt_ref",
                "oracle_receipt_ref",
                "blind_reader_receipt_ref",
            ] {
                if let Some(receipt) = value
                    .get(receipt_field)
                    .filter(|receipt| !receipt.is_null())
                {
                    receipts.push(receipt);
                }
            }
        }
    }
    receipts
}

fn validate_common_subject(
    controls: &BTreeMap<String, EvidenceState>,
    lenses: &BTreeMap<String, EvidenceState>,
    dissent: &EvidenceState,
    successor: &EvidenceState,
    exit: &EvidenceState,
    findings: &mut BTreeSet<String>,
) {
    let subjects = controls
        .values()
        .chain(lenses.values())
        .chain([dissent, successor, exit])
        .filter(|state| state.satisfied)
        .filter_map(|state| state.subject_digest.as_deref())
        .collect::<BTreeSet<_>>();
    if subjects.len() > 1 {
        findings.insert("all satisfied evidence must bind one frozen subject_digest".to_owned());
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_state_progress(
    state: Option<&str>,
    controls: &BTreeMap<String, EvidenceState>,
    lenses: &BTreeMap<String, EvidenceState>,
    dissent: &EvidenceState,
    successor: &EvidenceState,
    exit: &EvidenceState,
    blockers: Option<&Value>,
    findings: &mut BTreeSet<String>,
) {
    if controls.get("C05").is_some_and(|control| control.satisfied)
        && controls.get("C06").is_none_or(|control| !control.satisfied)
    {
        findings.insert(
            "C05 comparator evidence cannot be satisfied before C06 legal/JCR disposition"
                .to_owned(),
        );
    }

    let required_control_count = match state {
        Some("HOLD_EVIDENCE_COMPLETE") => 11,
        Some("HOLD_SUCCESSOR_FROZEN") => 12,
        Some("HOLD_EXIT_CANDIDATE") => 14,
        Some("PASS_CANDIDATE") => 15,
        _ => 0,
    };
    for (id, _, _) in CONTROLS.iter().take(required_control_count) {
        if controls.get(*id).is_none_or(|control| !control.satisfied) {
            findings.insert(format!(
                "state {} requires satisfied {id}",
                state.unwrap_or("invalid")
            ));
        }
    }
    let exit_candidate = matches!(state, Some("HOLD_EXIT_CANDIDATE") | Some("PASS_CANDIDATE"));
    if (matches!(state, Some("HOLD_SUCCESSOR_FROZEN")) || exit_candidate) && !successor.satisfied {
        findings.insert("state requires immutable_successor.frozen=true".to_owned());
    }
    if exit_candidate {
        for (id, _) in LENSES {
            if lenses.get(id).is_none_or(|lens| !lens.satisfied) {
                findings.insert(format!(
                    "state {} requires satisfied {id}",
                    state.unwrap_or("invalid")
                ));
            }
        }
        if !dissent.satisfied {
            findings.insert("state requires satisfied fresh_dissent".to_owned());
        }
    }
    if state == Some("PASS_CANDIDATE") && !exit.satisfied {
        findings.insert("state PASS_CANDIDATE requires satisfied context_free_exit".to_owned());
    }

    let mut all_reviewers = lenses
        .values()
        .flat_map(|lens| lens.principals.iter().cloned())
        .collect::<BTreeSet<_>>();
    for principal in &dissent.principals {
        if !all_reviewers.insert(principal.clone()) {
            findings.insert(
                "fresh_dissent reviewer_id must be independent from lens reviewers".to_owned(),
            );
        }
    }
    for principal in &exit.principals {
        if !all_reviewers.insert(principal.clone()) {
            findings.insert(
                "context_free_exit principals must be independent from prior reviewers".to_owned(),
            );
        }
    }

    let blocker_records = blockers.and_then(Value::as_array);
    let blocker_count = blocker_records.map_or(0, Vec::len);
    if state == Some("BLOCKED_QUALIFIED_HUMAN_INPUT") && blocker_count == 0 {
        findings.insert(
            "BLOCKED_QUALIFIED_HUMAN_INPUT requires at least one irreducible blocker".to_owned(),
        );
    }
    if state == Some("PASS_CANDIDATE") && blocker_count != 0 {
        findings.insert("PASS_CANDIDATE requires blockers to be empty".to_owned());
    }
    if state == Some("BLOCKED_QUALIFIED_HUMAN_INPUT") {
        for (index, blocker) in blocker_records
            .map_or(&[][..], Vec::as_slice)
            .iter()
            .enumerate()
        {
            let path = format!("blockers[{index}]");
            let control_id = blocker.get("control_id").and_then(Value::as_str);
            if !matches!(
                control_id,
                Some("C06" | "C07" | "C08" | "C09" | "C10" | "C11")
            ) {
                findings.insert(format!(
                    "{path}.control_id must be an allowed qualified-human control C06-C11"
                ));
                continue;
            }
            if controls
                .get(control_id.unwrap_or_default())
                .is_none_or(|control| !control.blocked)
            {
                findings.insert(format!(
                    "{path}.control_id must name a blocked, unsatisfied control"
                ));
            }
            for field in [
                "input_class",
                "required_qualification",
                "scope",
                "authority_source_ref",
                "reason",
            ] {
                if non_empty(blocker, field).is_none() {
                    findings.insert(format!("{path}.{field} must be a non-empty string"));
                }
            }
        }
    }
}

fn valid_subject_digest(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .filter(|digest| {
            digest
                .strip_prefix("sha256:")
                .is_some_and(|hex| is_hex(hex, 64))
        })
        .map(str::to_owned)
}

fn non_empty<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|candidate| !candidate.trim().is_empty())
}

fn is_hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
