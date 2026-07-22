//! Fail-closed structural agreement for the founder product-intent entry boundary.
//!
//! This checker only verifies that the four declared faces continue to describe a
//! non-dispatching, unresolved Stage-1 entry contract. It deliberately has no path
//! that can mark a control satisfied or promote `HOLD(Planning)`.

use std::collections::{BTreeSet, HashMap, HashSet};

use serde_json::{Value, json};

use crate::Finding;

pub const FOUNDER_PRODUCT_INTENT_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/founder-product-intent";
const INVALID_CODE: &str = "founder_product_intent_agreement_invalid";
/// Canonical current founder product-intent authority path.
pub const FOUNDER_PRODUCT_INTENT_PATH: &str = "/specs/founder-product-intent.json";
const AGENT_DURABLE_GOAL_HISTORY_ONLY_PROVENANCE_RULE: &str = "Historical source provenance is available only through Git history; the active tree must not contain or direct readers to a readable archive, manifest, or local compatibility copy.";

const REQUIRED_SECTIONS: [&str; 20] = [
    "authority_boundary",
    "provenance",
    "founder_execution_authorization",
    "durable_goal",
    "stage1_entry_requirements",
    "advisory_16_lens_council",
    "benchmark_and_measurement_contract",
    "participant_and_journey_census",
    "product_model",
    "game_engine_product_model",
    "change_accounting",
    "experience_contract",
    "data_and_lineage_contract",
    "temporal_and_epistemic_contract",
    "intelligence_and_automation_contract",
    "safety_rights_and_recourse",
    "representative_scenarios",
    "economics_and_sustainability_contract",
    "future_specification_requirements",
    "durability",
];

const STAGE1_CONTROL_IDS: [&str; 15] = [
    "adr_chronology",
    "decision_parser_ir",
    "corpus_archive_freshness",
    "decision_population",
    "legal_jcr",
    "comparator",
    "affected_party",
    "operations",
    "custody",
    "veto",
    "pilot",
    "immutable_successor",
    "council_16_lens",
    "fresh_dissent",
    "context_free_exit",
];

const COUNCIL_LENS_IDS: [&str; 16] = [
    "L01-product-outcomes-and-north-star",
    "L02-intuitive-ux-accessibility-and-adoption",
    "L03-ontology-data-time-and-epistemics",
    "L04-architecture-boundaries-and-monorepo",
    "L05-distributed-systems-reliability-and-resilience",
    "L06-security-identity-trust-and-custody",
    "L07-privacy-consent-and-data-governance",
    "L08-legal-regulatory-and-jcr",
    "L09-affected-party-safety-fairness-and-recourse",
    "L10-developer-experience-maintainability-and-quality",
    "L11-cloud-platform-tenancy-and-sovereignty",
    "L12-economics-finops-and-business-sustainability",
    "L13-ecosystem-integration-portability-and-supply-chain",
    "L14-migration-deprecation-compatibility-and-change",
    "L15-adversarial-abuse-failure-and-dissent",
    "L16-evidence-measurement-comparator-and-reproducibility",
];

const RECEIPT_FIELDS: [&str; 14] = [
    "control_id",
    "candidate_commit",
    "candidate_digest",
    "corpus_scope_digest",
    "evidence_ids",
    "evidence_digests",
    "qualified_owner_principal",
    "independent_reviewer_principal",
    "decision",
    "recorded_at",
    "valid_until_or_invalidation_events",
    "claim_ceiling",
    "signature_or_protected_receipt",
    "reproduction_command_or_method",
];

const RECEIPT_DECISIONS: [&str; 5] = [
    "satisfied",
    "unsatisfied",
    "unknown",
    "expired",
    "superseded",
];

const CONCURRENCY_PIPELINE_KEYS: [&str; 3] = [
    "parallel_lanes",
    "serialization_points",
    "promotion_barrier",
];

const PIPELINE_EVOLUTION_REQUIRED_KEYS: [&str; 37] = [
    "outcome",
    "authority_rule",
    "work_graph_contract",
    "closed_loop",
    "parallelism_rule",
    "serialization_rule",
    "lifecycle_graph_rule",
    "trusted_control_rule",
    "impact_rule",
    "demand_driven_execution_rule",
    "node_face_resource_evidence_contract",
    "test_and_evidence_rule",
    "automation_safety_governor",
    "exception_rule",
    "security_and_supply_chain_rule",
    "merge_and_release_separation",
    "promotion_state_machine",
    "candidate_evidence_minimum",
    "health_measures",
    "measurement_use_rule",
    "learning_rule",
    "research_rule",
    "self_modification_boundary",
    "implementation_claim_ceiling",
    "generated_artifact_rule",
    "failure_taxonomy_rule",
    "diagnosability_rule",
    "continuous_vulnerability_invalidation_rule",
    "tenant_pipeline_isolation_rule",
    "runtime_promotion_fail_closed_rule",
    "current_protected_admission_rule",
    "post_merge_closure_rule",
    "pipeline_slo_rule",
    "capacity_rule",
    "research_basis",
    "productization_and_portability_rule",
    "pipeline_migration_rule",
];

const WORK_GRAPH_REQUIRED_FIELDS: [&str; 11] = [
    "stable_work_id",
    "owner_principal_or_explicitly_unassigned",
    "scope_and_owned_paths",
    "dependencies_and_serialization_keys",
    "risk_and_authority_class",
    "acceptance_and_stop_conditions",
    "verification_and_evidence_contract",
    "claim_ceiling",
    "freshness_and_invalidation_events",
    "rollback_or_abandonment_path",
    "state",
];

const WORK_GRAPH_STATES: [&str; 11] = [
    "proposed",
    "ready",
    "active",
    "blocked",
    "verified",
    "admitted",
    "stable",
    "rejected",
    "rolled_back",
    "superseded",
    "retired",
];

const PROMOTION_STATES: [&str; 10] = [
    "proposed",
    "verified",
    "integrated_candidate",
    "admitted",
    "held",
    "eligible",
    "progressively_exposed",
    "stable",
    "rejected",
    "rolled_back",
];

const AUTOMATION_SAFETY_GOVERNOR_KEYS: [&str; 4] =
    ["AUTO", "ADVISE", "GATE", "classification_rule"];

const TIME_STATE_IDS: [&str; 3] = ["past", "present", "future"];

const KNOWLEDGE_CLASS_MAPPINGS: [(&str, &str); 14] = [
    ("SourceAssertion", "source_assertion"),
    ("ReportedClaim", "reported_claim"),
    ("DerivedState", "computed_state"),
    ("InferredRelationship", "inferred_relationship"),
    ("AnomalySignal", "anomaly_signal"),
    ("CausalHypothesis", "causal_hypothesis"),
    ("CausalClaim", "causal_claim"),
    ("Forecast", "forecast"),
    ("Scenario", "scenario"),
    ("Recommendation", "recommendation"),
    ("OptimizationProposal", "optimization_proposal"),
    ("Decision", "decision"),
    ("AuthorizedAction", "authorized_action"),
    ("Outcome", "verified_outcome"),
];

/// Check the founder-intent spec plus its root-hub, capability-registry, and graph faces.
/// Missing, malformed, or drifted data yields a stable blocking finding.
pub fn evaluate_founder_product_intent_agreement(
    intent: &Value,
    root_hub: &Value,
    registry: &Value,
    graph: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    for section in REQUIRED_SECTIONS {
        if intent.get(section).is_none_or(Value::is_null) {
            invalid(&mut findings, section);
        }
    }

    let boundary = &intent["authority_boundary"];
    require_string(
        &mut findings,
        boundary,
        "planning_state",
        "HOLD(Planning)",
        "authority_boundary.planning_state",
    );
    for field in [
        "architecture_ratified",
        "binding_plan_approval_allowed",
        "dispatch_allowed",
        "implementation_authorized",
        "roadmap_authorized",
    ] {
        if boundary.get(field).and_then(Value::as_bool) != Some(false) {
            invalid(&mut findings, &format!("authority_boundary.{field}"));
        }
    }

    validate_pipeline_contract(&mut findings, &intent["founder_execution_authorization"]);
    validate_temporal_and_epistemic_contract(
        &mut findings,
        &intent["temporal_and_epistemic_contract"],
    );
    validate_operational_world_contract(&mut findings, &intent["game_engine_product_model"]);
    validate_change_accounting(&mut findings, intent.get("change_accounting"));
    validate_comparator_contract(&mut findings, &intent["benchmark_and_measurement_contract"]);

    let stage1 = &intent["stage1_entry_requirements"];
    let controls = stage1.get("controls").and_then(Value::as_array);
    if !exact_ids(controls, &STAGE1_CONTROL_IDS) {
        invalid(&mut findings, "stage1_entry_requirements.controls");
    }
    if let Some(controls) = controls {
        for control in controls {
            let unresolved = control
                .get("qualified_owner_status")
                .and_then(Value::as_str)
                .is_some_and(|value| value.starts_with("unresolved"));
            if !unresolved {
                invalid(&mut findings, "stage1_entry_requirements.controls.nonclaim");
                break;
            }
            if control.get("decision").is_some() {
                invalid(&mut findings, "stage1_entry_requirements.controls.nonclaim");
                break;
            }
        }
        validate_stage1_control_dependencies(&mut findings, controls);
    }
    if !stage1
        .get("authority_status")
        .and_then(Value::as_str)
        .is_some_and(|value| value.contains("no control is satisfied"))
    {
        invalid(&mut findings, "stage1_entry_requirements.authority_status");
    }

    let receipt = &stage1["receipt_contract"];
    if !exact_string_values(receipt.get("required_fields"), &RECEIPT_FIELDS) {
        invalid(
            &mut findings,
            "stage1_entry_requirements.receipt_contract.required_fields",
        );
    }
    if !exact_string_values(receipt.get("allowed_decisions"), &RECEIPT_DECISIONS) {
        invalid(
            &mut findings,
            "stage1_entry_requirements.receipt_contract.allowed_decisions",
        );
    }

    let council = &intent["advisory_16_lens_council"];
    if !exact_ids(
        council.get("lenses").and_then(Value::as_array),
        &COUNCIL_LENS_IDS,
    ) {
        invalid(&mut findings, "advisory_16_lens_council.lenses");
    }
    if !council
        .get("stage1_nonclaim")
        .and_then(Value::as_str)
        .is_some_and(|value| value.contains("not the independently ratified"))
    {
        invalid(&mut findings, "advisory_16_lens_council.stage1_nonclaim");
    }

    validate_root_hub(&mut findings, root_hub);
    validate_registry_row(&mut findings, registry);
    validate_graph_edge(&mut findings, graph);
    validate_complete_graph_projection(&mut findings, registry, graph);
    findings
}

fn validate_stage1_control_dependencies(findings: &mut BTreeSet<Finding>, controls: &[Value]) {
    let positions: HashMap<&str, usize> = controls
        .iter()
        .enumerate()
        .filter_map(|(index, control)| {
            control
                .get("id")
                .and_then(Value::as_str)
                .map(|id| (id, index))
        })
        .collect();
    let expected_ids: HashSet<&str> = STAGE1_CONTROL_IDS.into_iter().collect();
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();

    for control in controls {
        let Some(id) = control.get("id").and_then(Value::as_str) else {
            continue;
        };
        let key = format!("stage1_entry_requirements.controls[{id}].depends_on");
        let Some(dependencies) = control.get("depends_on").and_then(Value::as_array) else {
            invalid(findings, &key);
            continue;
        };
        let mut seen = HashSet::new();
        let mut valid_dependencies = Vec::with_capacity(dependencies.len());
        for dependency in dependencies {
            let Some(dependency_id) = dependency.as_str() else {
                invalid(findings, &key);
                continue;
            };
            if !expected_ids.contains(dependency_id) || !positions.contains_key(dependency_id) {
                invalid(
                    findings,
                    "stage1_entry_requirements.controls.dependencies.known_ids",
                );
                continue;
            }
            if dependency_id == id || !seen.insert(dependency_id) {
                invalid(
                    findings,
                    "stage1_entry_requirements.controls.dependencies.self_or_duplicate",
                );
                continue;
            }
            if positions[dependency_id] >= positions[id] {
                invalid(
                    findings,
                    "stage1_entry_requirements.controls.dependencies.order",
                );
            }
            valid_dependencies.push(dependency_id);
        }
        graph.insert(id, valid_dependencies);
        if id == "comparator" && !exact_string_values(control.get("depends_on"), &["legal_jcr"]) {
            invalid(
                findings,
                "stage1_entry_requirements.controls[comparator].depends_on",
            );
        }
    }
    if has_stage1_dependency_cycle(&graph) {
        invalid(
            findings,
            "stage1_entry_requirements.controls.dependencies.cycle",
        );
    }
}

fn has_stage1_dependency_cycle(graph: &HashMap<&str, Vec<&str>>) -> bool {
    fn visit<'a>(
        node: &'a str,
        graph: &HashMap<&'a str, Vec<&'a str>>,
        states: &mut HashMap<&'a str, u8>,
    ) -> bool {
        match states.get(node) {
            Some(1) => return true,
            Some(2) => return false,
            _ => {}
        }
        states.insert(node, 1);
        if graph.get(node).is_some_and(|dependencies| {
            dependencies
                .iter()
                .any(|dependency| visit(dependency, graph, states))
        }) {
            return true;
        }
        states.insert(node, 2);
        false
    }

    let mut states = HashMap::new();
    graph
        .keys()
        .copied()
        .any(|node| visit(node, graph, &mut states))
}

fn validate_operational_world_contract(findings: &mut BTreeSet<Finding>, model: &Value) {
    require_contains_all(
        findings,
        model,
        "status",
        &["founder-intent-only", "non-binding", "ratified"],
        "game_engine_product_model.status",
    );
    require_contains_all(
        findings,
        model,
        "purpose",
        &[
            "product-design mental model",
            "not as a claim",
            "roadmap dispatch",
        ],
        "game_engine_product_model.purpose",
    );
    if !exact_string_values(
        model.get("world_contract"),
        &[
            "Model a revisioned world/state through stable entities, composable components and traits, explicit systems, and durable events.",
            "Represent business, effective, processing, and simulation time explicitly; preserve their distinctions rather than deriving one silently from another.",
            "Support deterministic preview and replay from declared inputs, versions, policies, and schedules, with uncertainty, external nondeterminism, and claim limits visible.",
            "Treat scenes and views as contextual projections of governed state, and support a visual world editor that authors canonical representations rather than a separate opaque visual truth.",
            "Support multi-user and tenant-scoped worlds, policy gates, save/load/audit, compensation and rollback where technically possible, and dependency-aware scheduling.",
        ],
    ) {
        invalid(findings, "game_engine_product_model.world_contract");
    }

    let types = &model["type_contract"];
    require_contains_all(
        findings,
        types,
        "rule",
        &[
            "first-class",
            "tenant, pack, or platform",
            "not a closed vendor enum",
        ],
        "game_engine_product_model.type_contract.rule",
    );
    if !exact_string_values(
        types.get("requirements"),
        &[
            "extensible, customizable, configurable, and no-code capable",
            "composed from stable definitions, components, traits, templates, defaults, and constraints rather than closed vendor enums",
            "versioned and content-addressed with provenance",
            "visually authored to canonical IR with round-trip traceability",
            "policy-bound and compatibility-classified",
            "previewable before application",
            "reversibly migratable, or explicitly approved with a stated irreversible boundary, affected population, recourse, and rollback or compensation limit",
        ],
    ) {
        invalid(
            findings,
            "game_engine_product_model.type_contract.requirements",
        );
    }
    if !exact_string_values(
        types.get("future_accepted_successor_requirements"),
        &[
            "Stable logical EntityId and TypeId remain distinct from immutable revision digests; external aliases and merge, split, and rekey history remain explicit.",
            "Component applicability, cardinality, precedence, conflict handling, and cycle detection are declared and fail closed when incomplete or conflicting.",
            "Commands, decisions, events, observations, corrections, projections, and effects remain explicit distinct kinds rather than one overloaded record.",
            "Preview is isolated from live effects and declares its clock, randomness, external I/O, uncertainty, and non-effect boundary.",
            "Projections are rebuildable and non-authoritative; visual writes compile to canonical IR and declare round-trip loss.",
            "Defaults retain provenance, templates create candidates rather than authority, and constraint conflicts fail closed.",
            "Compatibility is classified independently across producer, consumer, stored-data, UI, policy, and authority boundaries.",
            "Migrations freeze the affected population and require coexistence, dry-run, checkpoints, partial-failure recovery, refusal, and verification.",
            "Unknown extensions are preserved where possible and every semantic loss is explicitly disclosed.",
            "These are future Accepted-successor requirements only; they select no implementation, architecture, data model, framework, or roadmap.",
        ],
    ) {
        invalid(
            findings,
            "game_engine_product_model.type_contract.future_accepted_successor_requirements",
        );
    }
    let boundary = &model["governance_type_system_boundary"];
    if !exact_object_keys(
        Some(boundary),
        &[
            "vocabulary_status",
            "current_authority",
            "prohibited_effects",
            "successor_requirement",
            "implementation_selected",
            "roadmap_authorized",
        ],
    ) || boundary.get("vocabulary_status").and_then(Value::as_str)
        != Some("future-extension-vocabulary-only")
        || boundary.get("current_authority").and_then(Value::as_str)
            != Some("current-accepted-governance-knowledge-graph-type-system")
        || !exact_string_values(
            boundary.get("prohibited_effects"),
            &["replace", "amend", "rename", "reinterpret"],
        )
        || boundary
            .get("successor_requirement")
            .and_then(Value::as_str)
            != Some("separately-accepted-immutable-successor")
        || boundary
            .get("implementation_selected")
            .and_then(Value::as_bool)
            != Some(false)
        || boundary.get("roadmap_authorized").and_then(Value::as_bool) != Some(false)
    {
        invalid(
            findings,
            "game_engine_product_model.governance_type_system_boundary",
        );
    }
    if !exact_string_values(
        model.get("hard_limits"),
        &[
            "Legal authority is established by qualified principals and applicable law, never by a world-state transition.",
            "Privacy, consent, purpose limitation, retention, residency, and deletion constraints bind the world model and cannot be reduced to state.",
            "Accounting requires independently governed records, controls, reconciliation, and accountable sign-off; simulation or replay does not establish a financial fact.",
            "Distributed consistency limits, partial failure, and external concurrency require explicit semantics; a deterministic local replay does not prove global consistency.",
            "Irreversible external effects require separately established authority, policy, confirmation, and recourse; they are not made reversible by representation.",
            "Human accountability, affected-party rights, explanation, approval, appeal, remedy, and legal recourse remain human and institutional obligations, not game mechanics.",
        ],
    ) {
        invalid(findings, "game_engine_product_model.hard_limits");
    }
}

fn validate_change_accounting(findings: &mut BTreeSet<Finding>, value: Option<&Value>) {
    let Some(changes) = value.and_then(Value::as_array) else {
        invalid(findings, "change_accounting");
        return;
    };
    if changes.is_empty() {
        invalid(findings, "change_accounting");
        return;
    }
    for (index, change) in changes.iter().enumerate() {
        let key = format!("change_accounting[{index}]");
        if !exact_object_keys(
            Some(change),
            &[
                "change_id",
                "subject",
                "disposition",
                "claim_ceiling",
                "planning_state",
                "dispatch_allowed",
                "roadmap_authorized",
                "implementation_authorized",
                "authority_effect",
                "evidence_eligible",
            ],
        ) || change.get("change_id").and_then(Value::as_str)
            != Some("founder-intent-operational-world-types-2026-07-21")
            || change.get("subject").and_then(Value::as_str)
                != Some("founder-product-intent/operational-world-types")
            || change.get("disposition").and_then(Value::as_str)
                != Some("proposed-future-nonbinding")
            || change.get("claim_ceiling").and_then(Value::as_str)
                != Some("founder-intent-only-no-implementation-roadmap-or-authority")
            || change.get("planning_state").and_then(Value::as_str) != Some("HOLD(Planning)")
            || change.get("dispatch_allowed").and_then(Value::as_bool) != Some(false)
            || change.get("roadmap_authorized").and_then(Value::as_bool) != Some(false)
            || change
                .get("implementation_authorized")
                .and_then(Value::as_bool)
                != Some(false)
            || change.get("authority_effect").and_then(Value::as_str) != Some("none")
            || change.get("evidence_eligible").and_then(Value::as_bool) != Some(false)
        {
            invalid(findings, &key);
        }
    }
}

fn validate_comparator_contract(findings: &mut BTreeSet<Finding>, contract: &Value) {
    let admission = &contract["comparator_admission"];
    if !exact_object_keys(
        Some(admission),
        &[
            "control_id",
            "legal_prerequisite_control_id",
            "fresh_scope_specific_qualified_legal_jcr_required",
            "evidence_eligible",
            "claim_allowed",
            "implementation_selected",
        ],
    ) || admission.get("control_id").and_then(Value::as_str) != Some("comparator")
        || admission
            .get("legal_prerequisite_control_id")
            .and_then(Value::as_str)
            != Some("legal_jcr")
        || admission
            .get("fresh_scope_specific_qualified_legal_jcr_required")
            .and_then(Value::as_bool)
            != Some(true)
        || admission.get("evidence_eligible").and_then(Value::as_bool) != Some(false)
        || admission.get("claim_allowed").and_then(Value::as_bool) != Some(false)
        || admission
            .get("implementation_selected")
            .and_then(Value::as_bool)
            != Some(false)
    {
        invalid(
            findings,
            "benchmark_and_measurement_contract.comparator_admission",
        );
    }

    let Some(pointers) = contract
        .get("game_engine_comparator_refs")
        .and_then(Value::as_array)
    else {
        invalid(
            findings,
            "benchmark_and_measurement_contract.game_engine_comparator_refs",
        );
        return;
    };
    if pointers.is_empty() {
        invalid(
            findings,
            "benchmark_and_measurement_contract.game_engine_comparator_refs",
        );
    }
    for pointer in pointers {
        let source_id = pointer
            .get("source_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let key =
            format!("benchmark_and_measurement_contract.game_engine_comparator_refs[{source_id}]");
        if !exact_object_keys(
            Some(pointer),
            &[
                "source_id",
                "url",
                "declared_scope",
                "classification",
                "collection_status",
                "retrieved_at",
                "source_version",
                "content_digest",
                "legal_jcr_disposition",
                "evidence_eligible",
                "automated_expansion_allowed",
                "use",
            ],
        ) || pointer.get("classification").and_then(Value::as_str)
            != Some("inactive-source-pointer")
            || pointer.get("collection_status").and_then(Value::as_str)
                != Some("uncollected-for-admitted-comparator-use")
            || pointer.get("retrieved_at") != Some(&Value::Null)
            || pointer.get("source_version").and_then(Value::as_str) != Some("Unknown")
            || pointer.get("content_digest").and_then(Value::as_str) != Some("Unknown")
            || pointer.get("legal_jcr_disposition").and_then(Value::as_str)
                != Some(
                    "Unknown-uncollected-pending-fresh-scope-specific-qualified-legal-JCR-disposition",
                )
            || pointer.get("evidence_eligible").and_then(Value::as_bool) != Some(false)
            || pointer
                .get("automated_expansion_allowed")
                .and_then(Value::as_bool)
                != Some(false)
        {
            invalid(findings, &format!("{key}.evidence_eligible"));
        }
    }
}

fn validate_temporal_and_epistemic_contract(findings: &mut BTreeSet<Finding>, contract: &Value) {
    let time_states = contract.get("time_states").and_then(Value::as_array);
    if !exact_ids(time_states, &TIME_STATE_IDS) {
        invalid(findings, "temporal_and_epistemic_contract.time_states");
    }
    if time_states.is_none_or(|states| {
        states.iter().any(|state| {
            state
                .get("rule")
                .and_then(Value::as_str)
                .is_none_or(|value| value.trim().is_empty())
        })
    }) {
        invalid(findings, "temporal_and_epistemic_contract.time_states.rule");
    }

    let expected_classes: Vec<&str> = KNOWLEDGE_CLASS_MAPPINGS
        .iter()
        .map(|(_, knowledge_class)| *knowledge_class)
        .collect();
    if !exact_string_values(contract.get("knowledge_classes"), &expected_classes) {
        invalid(
            findings,
            "temporal_and_epistemic_contract.knowledge_classes",
        );
    }

    let Some(artifacts) = contract.get("typed_artifacts").and_then(Value::as_array) else {
        invalid(findings, "temporal_and_epistemic_contract.typed_artifacts");
        return;
    };
    if artifacts.len() != KNOWLEDGE_CLASS_MAPPINGS.len() {
        invalid(findings, "temporal_and_epistemic_contract.typed_artifacts");
    }

    let mut artifact_ids = HashSet::new();
    let mut mapped_classes = HashSet::new();
    let mut observed_mappings = HashSet::new();
    for artifact in artifacts {
        let Some(artifact_id) = artifact.get("id").and_then(Value::as_str) else {
            invalid(
                findings,
                "temporal_and_epistemic_contract.typed_artifacts.id",
            );
            continue;
        };
        if artifact_id.is_empty() || !artifact_ids.insert(artifact_id) {
            invalid(
                findings,
                "temporal_and_epistemic_contract.typed_artifacts.id",
            );
        }

        let Some(knowledge_class) = artifact.get("knowledge_class").and_then(Value::as_str) else {
            invalid(
                findings,
                "temporal_and_epistemic_contract.typed_artifacts.knowledge_class",
            );
            continue;
        };
        if knowledge_class.is_empty() || !mapped_classes.insert(knowledge_class) {
            invalid(
                findings,
                "temporal_and_epistemic_contract.typed_artifacts.knowledge_class",
            );
        }
        observed_mappings.insert((artifact_id, knowledge_class));

        if artifact
            .get("minimum")
            .and_then(Value::as_str)
            .is_none_or(|value| value.trim().is_empty())
        {
            invalid(
                findings,
                "temporal_and_epistemic_contract.typed_artifacts.minimum",
            );
        }
    }

    let expected_mappings: HashSet<(&str, &str)> =
        KNOWLEDGE_CLASS_MAPPINGS.iter().copied().collect();
    if observed_mappings != expected_mappings {
        invalid(
            findings,
            "temporal_and_epistemic_contract.typed_artifacts.mapping",
        );
    }
    if mapped_classes != expected_classes.iter().copied().collect() {
        invalid(
            findings,
            "temporal_and_epistemic_contract.typed_artifacts.knowledge_class",
        );
    }

    require_contains_all(
        findings,
        contract,
        "mapping_rule",
        &[
            "Every knowledge class has exactly one explicit typed-artifact mapping",
            "must not silently promote",
        ],
        "temporal_and_epistemic_contract.mapping_rule",
    );
    require_contains_all(
        findings,
        contract,
        "rule",
        &[
            "must never collapse different knowledge classes",
            "valid, event, observed, recorded, and effective time",
        ],
        "temporal_and_epistemic_contract.rule",
    );
}

fn validate_pipeline_contract(findings: &mut BTreeSet<Finding>, authorization: &Value) {
    let concurrency = &authorization["concurrency_pipeline"];
    if !exact_object_keys(Some(concurrency), &CONCURRENCY_PIPELINE_KEYS) {
        invalid(
            findings,
            "founder_execution_authorization.concurrency_pipeline.keys",
        );
    }
    require_contains_all(
        findings,
        concurrency,
        "promotion_barrier",
        &[
            "Before qualified PASS(Planning)",
            "cannot amend the controlling roadmap",
            "implementation dispatch",
            "After PASS(Planning)",
        ],
        "founder_execution_authorization.concurrency_pipeline.promotion_barrier",
    );
    require_contains_all(
        findings,
        authorization,
        "hold_interaction",
        &["HOLD(Planning)", "PASS(Planning)"],
        "founder_execution_authorization.hold_interaction",
    );

    let evolution = &authorization["pipeline_evolution_contract"];
    if !has_object_keys(Some(evolution), &PIPELINE_EVOLUTION_REQUIRED_KEYS) {
        invalid(
            findings,
            "founder_execution_authorization.pipeline_evolution_contract.required_keys",
        );
    }
    require_contains_all(
        findings,
        evolution,
        "authority_rule",
        &[
            "Proposed",
            "implemented artifacts",
            "do not become authority",
            "agent inference",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.authority_rule",
    );

    let work_graph = &evolution["work_graph_contract"];
    if !exact_string_values(
        work_graph.get("required_fields"),
        &WORK_GRAPH_REQUIRED_FIELDS,
    ) {
        invalid(
            findings,
            "founder_execution_authorization.pipeline_evolution_contract.work_graph_contract.required_fields",
        );
    }
    if !exact_string_values(work_graph.get("states"), &WORK_GRAPH_STATES) {
        invalid(
            findings,
            "founder_execution_authorization.pipeline_evolution_contract.work_graph_contract.states",
        );
    }

    if !exact_string_values(evolution.get("promotion_state_machine"), &PROMOTION_STATES) {
        invalid(
            findings,
            "founder_execution_authorization.pipeline_evolution_contract.promotion_state_machine",
        );
    }

    let governor = &evolution["automation_safety_governor"];
    if !exact_object_keys(Some(governor), &AUTOMATION_SAFETY_GOVERNOR_KEYS) {
        invalid(
            findings,
            "founder_execution_authorization.pipeline_evolution_contract.automation_safety_governor.keys",
        );
    }
    for tier in ["AUTO", "ADVISE", "GATE"] {
        if governor
            .get(tier)
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            invalid(
                findings,
                &format!(
                    "founder_execution_authorization.pipeline_evolution_contract.automation_safety_governor.{tier}"
                ),
            );
        }
    }
    if governor
        .get("classification_rule")
        .and_then(Value::as_str)
        .is_none_or(str::is_empty)
    {
        invalid(
            findings,
            "founder_execution_authorization.pipeline_evolution_contract.automation_safety_governor.classification_rule",
        );
    }

    require_contains_all(
        findings,
        evolution,
        "trusted_control_rule",
        &[
            "candidate revision is untrusted data",
            "protected control state",
            "cannot weaken",
            "prior protected control",
            "proposed successor",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.trusted_control_rule",
    );
    require_contains_all(
        findings,
        evolution,
        "demand_driven_execution_rule",
        &[
            "requested face or action evaluates only its declared transitive dependency slice",
            "no runner may eagerly compute unrelated faces",
            "Unknown, undeclared, incomplete, stale, conflicting, or graph-changing dependencies broaden evaluation to the applicable full universe or block",
            "never silently bypass",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.demand_driven_execution_rule",
    );
    require_contains_all(
        findings,
        evolution,
        "node_face_resource_evidence_contract",
        &[
            "every executed node, requested face, and action",
            "cold and warm wall time",
            "user and system CPU",
            "max RSS",
            "input bytes and bytes scanned",
            "cache state, key identity, hit/miss and invalidation reason",
            "I/O",
            "queue wait and contention",
            "deterministic output hash",
            "no numeric threshold is implied",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.node_face_resource_evidence_contract",
    );
    require_contains_all(
        findings,
        evolution,
        "merge_and_release_separation",
        &[
            "does not prove runtime readiness",
            "Runtime promotion separately requires",
            "immutable artifact",
            "automatic stop or rollback",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.merge_and_release_separation",
    );
    require_contains_all(
        findings,
        evolution,
        "implementation_claim_ceiling",
        &[
            "does not claim every mechanism is implemented",
            "current lifecycle authority",
            "exact runtime receipts",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.implementation_claim_ceiling",
    );
    require_contains_all(
        findings,
        evolution,
        "generated_artifact_rule",
        &[
            "never hand-edited authority",
            "tracked or not-tracked-in-git",
            "materialize through the registered controller",
            "regenerate-twice determinism",
            "stale, unregistered, manually altered, or re-tracked decommitted",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.generated_artifact_rule",
    );
    require_contains_all(
        findings,
        evolution,
        "failure_taxonomy_rule",
        &[
            "code regression",
            "policy violation",
            "security or privacy finding",
            "infrastructure failure",
            "suspected flake",
            "cancellation",
            "timeout",
            "first failing evidence",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.failure_taxonomy_rule",
    );
    require_contains_all(
        findings,
        evolution,
        "diagnosability_rule",
        &[
            "typed, redacted, retained evidence packet",
            "exact subject, base, merge-base",
            "reproduction path",
            "never become a second merge authority",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.diagnosability_rule",
    );
    require_contains_all(
        findings,
        evolution,
        "continuous_vulnerability_invalidation_rule",
        &[
            "fresh vulnerability",
            "re-evaluates already admitted and stable artifacts",
            "does not claim the current Proposed vulnerability-intelligence pipeline is live authority",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.continuous_vulnerability_invalidation_rule",
    );
    require_contains_all(
        findings,
        evolution,
        "tenant_pipeline_isolation_rule",
        &[
            "tenant-scoped",
            "default-deny across tenant boundaries",
            "Negative cross-tenant fixtures",
            "does not claim those Phase-0 gaps are closed",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.tenant_pipeline_isolation_rule",
    );
    require_contains_all(
        findings,
        evolution,
        "runtime_promotion_fail_closed_rule",
        &[
            "begins runtime evaluation in held, not eligible",
            "deterministic, non-LLM decision",
            "backfilled evidence remains held",
            "cannot mint promotion authority",
            "new attributable receipt",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.runtime_promotion_fail_closed_rule",
    );
    require_contains_all(
        findings,
        evolution,
        "current_protected_admission_rule",
        &[
            "current protected target",
            "signed",
            "exact head",
            "unresolved review threads",
            "oya-ci-required",
            "invalidates prior admission evidence",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.current_protected_admission_rule",
    );
    require_contains_all(
        findings,
        evolution,
        "post_merge_closure_rule",
        &[
            "not product completion",
            "exact claim ceiling",
            "claims blocked",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.post_merge_closure_rule",
    );
    require_contains_all(
        findings,
        evolution,
        "resource_lineage_and_health_rule",
        &[
            "one typed, versioned lineage graph",
            "source, input, intermediate, output, user-facing, and sink roles",
            "schema, content, freshness, build, schedule, sync, latency, error, and outcome health",
            "dynamically scoped monitoring",
            "orphaned, unowned, or unmonitored",
            "does not become authority",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.resource_lineage_and_health_rule",
    );
    require_contains_all(
        findings,
        evolution,
        "probabilistic_evaluation_rule",
        &[
            "versioned evaluation suites",
            "frozen population",
            "repeated runs and variance",
            "model, prompt, tool, data, policy, and evaluator versions",
            "subgroup and affected-party outcomes",
            "passing evaluation never establishes authority",
            "draft-only",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.probabilistic_evaluation_rule",
    );
    require_contains_all(
        findings,
        evolution,
        "pipeline_slo_rule",
        &[
            "service-level objectives",
            "false-green and false-red escape rates",
            "never spend them to bypass",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.pipeline_slo_rule",
    );
    require_contains_all(
        findings,
        evolution,
        "capacity_rule",
        &[
            "maximum safe useful concurrency",
            "ready work graph",
            "reduce concurrency automatically",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.capacity_rule",
    );
    validate_research_basis(findings, evolution.get("research_basis"));
    require_contains_all(
        findings,
        evolution,
        "productization_and_portability_rule",
        &[
            "version and digest",
            "fresh checkout and immutable inputs",
            "equivalence evidence",
            "avoid making any vendor, UI, CLI, or hosted control plane the only way",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.productization_and_portability_rule",
    );
    require_contains_all(
        findings,
        evolution,
        "pipeline_migration_rule",
        &[
            "old and new authorities",
            "cutover and rollback conditions",
            "remove the retired mechanism",
            "readable authority surfaces",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.pipeline_migration_rule",
    );
    require_contains_all(
        findings,
        governor,
        "AUTO",
        &["may never merge it", "bypass a gate", "widen a baseline"],
        "founder_execution_authorization.pipeline_evolution_contract.automation_safety_governor.AUTO.prohibited_authority",
    );
}

fn validate_research_basis(findings: &mut BTreeSet<Finding>, value: Option<&Value>) {
    let Some(research_basis) = value else {
        invalid(
            findings,
            "founder_execution_authorization.pipeline_evolution_contract.research_basis",
        );
        return;
    };
    if research_basis
        .get("retrieved_at")
        .and_then(Value::as_str)
        .is_none_or(str::is_empty)
    {
        invalid(
            findings,
            "founder_execution_authorization.pipeline_evolution_contract.research_basis.retrieved_at",
        );
    }
    require_contains_all(
        findings,
        research_basis,
        "claim_boundary",
        &[
            "external evidence",
            "do not override Oyatie authority",
            "prove implementation",
            "authorize a roadmap",
        ],
        "founder_execution_authorization.pipeline_evolution_contract.research_basis.claim_boundary",
    );
    let Some(sources) = research_basis.get("sources").and_then(Value::as_array) else {
        invalid(
            findings,
            "founder_execution_authorization.pipeline_evolution_contract.research_basis.sources",
        );
        return;
    };
    if sources.is_empty() {
        invalid(
            findings,
            "founder_execution_authorization.pipeline_evolution_contract.research_basis.sources",
        );
        return;
    }

    let mut source_ids = HashSet::new();
    let mut urls = HashSet::new();
    for (index, source) in sources.iter().enumerate() {
        let index_key = format!(
            "founder_execution_authorization.pipeline_evolution_contract.research_basis.sources[{index}]"
        );
        let Some(source_id) = source.get("source_id").and_then(Value::as_str) else {
            invalid(findings, &format!("{index_key}.source_id"));
            continue;
        };
        if source_id.is_empty()
            || !source_id.chars().all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
            })
            || !source_ids.insert(source_id)
        {
            invalid(findings, &format!("{index_key}.source_id"));
        }
        let key = format!(
            "founder_execution_authorization.pipeline_evolution_contract.research_basis.sources[{source_id}]"
        );

        let Some(url) = source.get("url").and_then(Value::as_str) else {
            invalid(findings, &format!("{key}.url"));
            continue;
        };
        if !url.starts_with("https://") || !urls.insert(url) {
            invalid(findings, &format!("{key}.url"));
        }

        for field in ["scope", "adoption"] {
            if source
                .get(field)
                .and_then(Value::as_str)
                .is_none_or(str::is_empty)
            {
                invalid(findings, &format!("{key}.{field}"));
            }
        }
        if !exact_object_keys(
            Some(source),
            &[
                "source_id",
                "url",
                "scope",
                "adoption",
                "classification",
                "collection_status",
                "legal_jcr_disposition",
                "evidence_eligible",
                "automated_expansion_allowed",
                "quarantine",
            ],
        ) || source.get("classification").and_then(Value::as_str) != Some("harvested-provenance")
            || source.get("collection_status").and_then(Value::as_str)
                != Some("retrieved-before-qualified-legal-jcr-disposition")
            || source.get("legal_jcr_disposition").and_then(Value::as_str)
                != Some("unresolved-pending-fresh-scope-specific-qualified-legal-jcr-disposition")
            || source.get("evidence_eligible").and_then(Value::as_bool) != Some(false)
            || source
                .get("automated_expansion_allowed")
                .and_then(Value::as_bool)
                != Some(false)
        {
            invalid(findings, &format!("{key}.evidence_eligible"));
        }
        let quarantine = &source["quarantine"];
        if !exact_object_keys(
            Some(quarantine),
            &[
                "status",
                "preserved",
                "evidence_eligible",
                "automated_expansion_allowed",
                "claim_allowed",
                "legal_jcr_disposition",
            ],
        ) || quarantine.get("status").and_then(Value::as_str)
            != Some("harvested-provenance-not-admitted-evidence")
            || quarantine.get("preserved").and_then(Value::as_bool) != Some(true)
            || quarantine.get("evidence_eligible").and_then(Value::as_bool) != Some(false)
            || quarantine
                .get("automated_expansion_allowed")
                .and_then(Value::as_bool)
                != Some(false)
            || quarantine.get("claim_allowed").and_then(Value::as_bool) != Some(false)
            || quarantine
                .get("legal_jcr_disposition")
                .and_then(Value::as_str)
                != Some("unresolved")
        {
            invalid(findings, &format!("{key}.quarantine"));
        }
    }
}

fn validate_root_hub(findings: &mut BTreeSet<Finding>, root_hub: &Value) {
    let entry = &root_hub["entry_points"]["founder_product_intent"];
    require_string(
        findings,
        entry,
        "current_path",
        FOUNDER_PRODUCT_INTENT_PATH,
        "root_hub.entry_points.founder_product_intent.current_path",
    );
    if !root_hub["agent_quick_start_protocol"]["step_1_read_authority"]
        .as_str()
        .is_some_and(|value| value.contains(".founder_product_intent"))
    {
        invalid(
            findings,
            "root_hub.agent_quick_start_protocol.founder_product_intent",
        );
    }
    if !root_hub["agent_entry_surface_allowlist"]["paths"]
        .as_array()
        .is_some_and(|paths| {
            paths
                .iter()
                .any(|path| path.as_str() == Some(FOUNDER_PRODUCT_INTENT_PATH))
        })
    {
        invalid(
            findings,
            "root_hub.agent_entry_surface_allowlist.founder_product_intent",
        );
    }

    let durable_goal = &root_hub["entry_points"]["agent_durable_goal"];
    let history_only_provenance = durable_goal
        .get("history_only_provenance_rule")
        .and_then(Value::as_str)
        == Some(AGENT_DURABLE_GOAL_HISTORY_ONLY_PROVENANCE_RULE);
    if !history_only_provenance
        || durable_goal.get("retired_archive_manifest_path").is_some()
        || durable_goal.get("archive_manifest_status").is_some()
    {
        invalid(
            findings,
            "root_hub.entry_points.agent_durable_goal.history_only_provenance",
        );
    }
}

fn validate_registry_row(findings: &mut BTreeSet<Finding>, registry: &Value) {
    let valid = registry["rows"].as_array().is_some_and(|rows| {
        rows.iter().any(|row| {
            row["artifact_id"].as_str() == Some("founder-product-intent")
                && row["artifact_path"].as_str() == Some(FOUNDER_PRODUCT_INTENT_PATH)
                && row["artifact_format"].as_str() == Some("json")
                && row["artifact_profile"].as_str() == Some("spec")
        })
    });
    if !valid {
        invalid(
            findings,
            "artifact_capabilities_registry.founder-product-intent",
        );
    }
}

fn validate_graph_edge(findings: &mut BTreeSet<Finding>, graph: &Value) {
    let valid = graph["edges"].as_array().is_some_and(|edges| {
        edges.iter().any(|edge| {
            edge["source"].as_str() == Some("founder-product-intent")
                && edge["target"].as_str() == Some("spec")
                && edge["edge_type"].as_str() == Some("declares")
        })
    });
    if !valid {
        invalid(
            findings,
            "active_artifact_contract_edges.founder-product-intent",
        );
    }
}

fn validate_complete_graph_projection(
    findings: &mut BTreeSet<Finding>,
    registry: &Value,
    graph: &Value,
) {
    let Some(rows) = registry.get("rows").and_then(Value::as_array) else {
        invalid(
            findings,
            "active_artifact_contract_edges.complete_registry_projection",
        );
        return;
    };

    let mut edges = Vec::with_capacity(rows.len());
    for row in rows {
        let Some(source) = row.get("artifact_id").and_then(Value::as_str) else {
            invalid(
                findings,
                "active_artifact_contract_edges.complete_registry_projection",
            );
            return;
        };
        let Some(target) = row.get("artifact_profile").and_then(Value::as_str) else {
            invalid(
                findings,
                "active_artifact_contract_edges.complete_registry_projection",
            );
            return;
        };
        edges.push(json!({
            "source": source,
            "target": target,
            "edge_type": "declares"
        }));
    }

    let expected = json!({
        "$schema_ref": "specs/knowledge-graph-schema.json",
        "_artifact_id": "active-artifact-contract-edges",
        "_meta": {
            "emitter": "oya-dev-cli gate validate active-artifact-contract",
            "layer": "semantic",
            "purpose": "Generated graph edges that connect active machine-readable artifacts to their declared schemas, registries, templates, and ledgers."
        },
        "edges": edges
    });
    if graph != &expected {
        invalid(
            findings,
            "active_artifact_contract_edges.complete_registry_projection",
        );
    }
}

fn exact_ids(values: Option<&Vec<Value>>, expected: &[&str]) -> bool {
    values.is_some_and(|values| {
        values.len() == expected.len()
            && values
                .iter()
                .all(|value| value.get("id").and_then(Value::as_str).is_some())
            && values
                .iter()
                .filter_map(|value| value["id"].as_str())
                .collect::<HashSet<_>>()
                == expected.iter().copied().collect()
    })
}

fn exact_string_values(value: Option<&Value>, expected: &[&str]) -> bool {
    value.and_then(Value::as_array).is_some_and(|values| {
        values.len() == expected.len()
            && values.iter().all(Value::is_string)
            && values
                .iter()
                .filter_map(Value::as_str)
                .collect::<HashSet<_>>()
                == expected.iter().copied().collect()
    })
}

fn has_object_keys(value: Option<&Value>, expected: &[&str]) -> bool {
    value
        .and_then(Value::as_object)
        .is_some_and(|object| expected.iter().all(|key| object.contains_key(*key)))
}

fn exact_object_keys(value: Option<&Value>, expected: &[&str]) -> bool {
    value.and_then(Value::as_object).is_some_and(|object| {
        object.len() == expected.len()
            && object.keys().map(String::as_str).collect::<HashSet<_>>()
                == expected.iter().copied().collect()
    })
}

fn require_contains_all(
    findings: &mut BTreeSet<Finding>,
    object: &Value,
    field: &str,
    required_fragments: &[&str],
    key: &str,
) {
    if !object
        .get(field)
        .and_then(Value::as_str)
        .is_some_and(|value| {
            required_fragments
                .iter()
                .all(|fragment| value.contains(fragment))
        })
    {
        invalid(findings, key);
    }
}

fn require_string(
    findings: &mut BTreeSet<Finding>,
    object: &Value,
    field: &str,
    expected: &str,
    key: &str,
) {
    if object.get(field).and_then(Value::as_str) != Some(expected) {
        invalid(findings, key);
    }
}

fn invalid(findings: &mut BTreeSet<Finding>, key: &str) {
    findings.insert(Finding::new(INVALID_CODE, key));
}
