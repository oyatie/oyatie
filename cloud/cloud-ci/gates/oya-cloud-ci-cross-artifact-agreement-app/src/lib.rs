//! # cloud-ci-cross-artifact-agreement (GATE-1)
//!
//! The cross-artifact-agreement gate that asserts every decision agrees across the
//! artifacts that must propagate it — the ADR front-matter, the spec corpus, the
//! masterplan graph, the roadmap/sequencing graph, the reciprocal supersession edges,
//! and the GENERATED faces (PHASE-0-FIREWALL-PLAN §5.2; amends ADR-0365). It evaluates a
//! cross-artifact corpus `Value` and emits a `{verdict, violations}` report; its tests
//! assert `report.violations == fixture.expected_violations` over
//! `specs/fixtures/cross-artifact-agreement/tc-*.json`.
//!
//! ## Blocking violation codes (the contract — literal strings the gate emits)
//! - `orphan_decision`        — an Accepted decision reaches NO propagation face
//!   (absent from spec AND masterplan AND roadmap): a decision nothing points at.
//! - `unpropagated_decision`  — an Accepted decision reaches SOME but not ALL of its
//!   required propagation faces (e.g. has an ADR + spec but no masterplan/roadmap node).
//! - `status_disagreement`    — the decision's status disagrees across the faces that
//!   record it (e.g. ADR `Accepted` while the roadmap node marks it `superseded`).
//! - `generated_face_drift`   — two GENERATED faces that must agree on a shared value
//!   disagree (the frozen live exhibit: `catalog.json axes_count:6` vs
//!   `contracts.json axes_count:7`).
//! - `dual_decision_collision`— two distinct decision FILES share one decision id (the
//!   historical live exhibit: the two ADR-0377 files, resolved 2026-06-12 by renumbering
//!   the newer one to ADR-0557 per FRIC-1781390000; frozen as tc-XA-bad-dup-adr-number).
//! - `decision_id_mismatch` — a decision file whose front-matter id disagrees with its
//!   filename number, which can mask duplicate-id detection.
//! - `supersession_half_edge` — a supersession edge that is not reciprocal (A `supersedes`
//!   B while B's `superseded_by` omits A, or the reverse): a half-built edge.
//! - `phantom_decision_citation` — a governed surface (a decision body, the
//!   roadmap/sequencing artifact, or the masterplan `bound_adrs`) cites an `ADR-NNNN`
//!   that resolves to NO on-disk decision file (the phantom-0397 exhibit: seven surfaces
//!   cited "ADR-0397 Pulsar 4.x + Oxia canonical event-bus" with no file at the number —
//!   audit register H-19; healed 2026-06-12 by MINTING the record, FRIC-1781430000).
//!   Frozen-empty: the producer excludes the ledgered historical inventory as reviewed
//!   shrink-only DATA, so any edge that reaches the face is NEW debt and born-blocking.
//! - `masterplan_not_sole_live_authority` — /specs/masterplan.json#masterplan_v2
//!   does not declare itself as the sole live plan authority, or a legacy surface
//!   still claims live plan authority.
//! - `masterplan_surface_disposition_incomplete` — a legacy plan/goal surface
//!   or non-Hermes harness store from the ingest set lacks an explicit absorbed,
//!   archived-with-provenance, or generated-projection disposition.
//! - `masterplan_work_item_id_collision` — two masterplan v2 work items carry one
//!   live work-item id.
//! - `masterplan_external_live_work_item_id` — a live work item uses an id outside
//!   the canonical MPV2 namespace, or carries an external id marked as live.
//! - `masterplan_dependency_dag_invalid` — a masterplan v2 dependency edge is
//!   malformed, points at a missing work item, lacks the prerequisite→dependent
//!   direction contract, participates in a cycle, or crosses program shards
//!   without a declared program-pair edge in `cross_program_edge_policy`
//!   (including a missing/malformed policy, an unknown/self/duplicate declared
//!   pair, and a stale declared pair no live edge instantiates).
//! - `masterplan_program_coverage_incomplete` — masterplan v2 lacks a complete
//!   program-sharded coverage proof for the microservice manifest index, the
//!   required ontology/workflow/intelligence/owned-stack/reorg/AST/fabric shards,
//!   or the ADR-0537 owned-stack ladder (cloud-kernel → cloud-os → cloud-k8s →
//!   cloud-services → products).
//! - `masterplan_evidence_state_invalid` — a Hermes done-card completion
//!   claim without attached evidence is not projected as claimed-done-unverified,
//!   or a masterplan completion status lacks evidence references.
//! - `masterplan_plan_evidence_drift` — masterplan v2 status/evidence policy,
//!   evidence references, or evidence-attached Hermes completion states drift
//!   from the evidence-audited plan contract.
//! - `masterplan_plan_evidence_unrecorded` — a masterplan work-item status
//!   claim (or evidence-attached Hermes import) is not cross-checkable against
//!   RECORDED completion evidence: a verified 'done' claim carries no merged
//!   commit / merged-PR / gate-run record or tracked product-completion
//!   packet, an evidence ref dangles outside the tracked tree, points at a
//!   retired (absorbed / archived-with-provenance) surface, or is a malformed
//!   recorded-evidence ref.
//! - `masterplan_projection_freshness_invalid` — generated/read projections
//!   derived from /specs/masterplan.json lack complete freshness coverage,
//!   conflict-resolution, single-writer, or no-live-authority metadata.
//! - `masterplan_projection_stale` — a derived/generated masterplan projection
//!   on disk is not byte-identical to its mechanical re-derivation from
//!   /specs/masterplan.json (stale or hand-edited), or a derived ledger/card
//!   shard breaks its re-derivation invariants (canonical wire bytes,
//!   contiguous 1-based pass_seq under canonical filenames, plan-DAG
//!   referential agreement, completion-requires-evidence).
//! - `masterplan_read_contract_invalid` — archived-with-provenance stale
//!   read paths are referenced through a non-archive read contract or read
//!   projection, or a superseded/stale plan authority (docs/MASTERPLAN.md,
//!   docs/ROADMAP.md, the retired planning specs) is RESURRECTED on disk
//!   outside the archive: a governed absorbed / archived-with-provenance /
//!   generated-projection surface whose on-disk content no longer declares
//!   itself non-live, drops its canonical-authority pointer or
//!   provenance-archive read-timing declaration, or escapes the
//!   read-surface sweep entirely.
//! - `masterplan_entry_surface_invalid` — masterplan v2 entry-surface
//!   read contracts drift from the bounded root-hub allowlist or revive a
//!   superseded entrypoint.
//!
//! - `masterplan_sequencing_invalid` — masterplan v2 lacks a fully
//!   zero-based, DAG-derived sequencing projection over every live work item.
//! - `masterplan_execution_wave_dispatch_unratified` — execution-wave dispatch
//!   is not fail-closed on an explicit founder-ratification decision.
//!
//! The evaluator is pure: the fixture (data-under-test) drives it; there are no scanner
//! special-cases. Carve-outs/exceptions live as DATA, never as evaluator branches.
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

mod plan_evidence_crosscheck;
mod projection_rederivation;
mod read_surface_resurrection;

pub use plan_evidence_crosscheck::{
    PLAN_EVIDENCE_CROSSCHECK_VALIDATOR, UNRECORDED_EVIDENCE_CODE,
    evaluate_masterplan_plan_evidence_crosscheck,
};
pub use projection_rederivation::{
    MASTERPLAN_MD_PATH, PROJECTION_REDERIVATION_VALIDATOR, STALE_PROJECTION_CODE,
    derive_masterplan_md_projection, evaluate_masterplan_projection_rederivation,
};
pub use read_surface_resurrection::{
    READ_SURFACE_RESURRECTION_VALIDATOR, RESURRECTION_CODE,
    evaluate_masterplan_read_surface_resurrections,
};
/// The gate id, matching the buck2 target + the §5.2 contract.
pub const GATE_ID: &str = "cloud-ci-cross-artifact-agreement";
const DEPENDENCY_EDGE_SEMANTICS: &str = "from is prerequisite, to is dependent";
/// Cross-program edge contract pinned by `masterplan_v2.cross_program_edge_policy.semantics`.
const CROSS_PROGRAM_EDGE_SEMANTICS: &str =
    "a dependency edge crossing program shards must be backed by a declared program-pair edge";
const MANIFEST_INDEX_REF: &str = "/specs/microservices/manifests-index.json";
const SEQUENCING_SOURCE_OF_TRUTH: &str = "/specs/masterplan.json#masterplan_v2.dependency_edges";
const SEQUENCING_DERIVATION_MODE: &str = "zero-based-rederived-from-masterplan-v2-dependency-dag";
const DISPATCH_BLOCKED_STATE: &str = "blocked";
const DISPATCH_BLOCKED_REASON: &str = "pending_founder_ratification";
const CLAIMED_DONE_UNVERIFIED_STATE: &str = "claimed-done-unverified";
const EVIDENCE_ATTACHED_STATE: &str = "evidence-attached";
const PROJECTION_FRESHNESS_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/masterplan-v2-projection-freshness";
const PLAN_EVIDENCE_DRIFT_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/masterplan-v2-plan-vs-evidence-drift";
const ENTRY_SURFACE_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/masterplan-v2-entry-surface";
const ENTRY_SURFACE_ALLOWLIST_REF: &str =
    "/specs/root-hub-pointers.json#agent_entry_surface_allowlist";
const PROJECTION_CLASS_READ: &str = "read-projection";
const READ_CONTRACT_ARCHIVED_TIMING_CLASS: &str = "provenance-archive";
const READ_CONTRACT_ENTRY_TIMING_CLASS: &str = "entry-surface";
const REQUIRED_PROGRAM_CLASSES: [&str; 8] = [
    "ontology",
    "workflow-engine",
    "workflow-studio",
    "intelligence",
    "owned-stack",
    "reorg",
    "ast-code-graph",
    "fabric",
];
const REQUIRED_OWNED_STACK_LAYERS: [&str; 6] = [
    "cloud-kernel",
    "cloud-os",
    "cloud-k8s",
    "cloud-services",
    "durability-plane",
    "governance-iam-console",
];
/// The ADR-0537 §3 owned-stack ladder in fixed rung order: kuberos kernel →
/// cloud-os → cloud-k8s → cloud services → oyatie products.
const REQUIRED_OWNED_STACK_LADDER_RUNGS: [&str; 5] = [
    "cloud-kernel",
    "cloud-os",
    "cloud-k8s",
    "cloud-services",
    "products",
];
const MASTERPLAN_V2_REF: &str = "/specs/masterplan.json#masterplan_v2";
const DISPOSITION_CANONICAL_AUTHORITY: &str = "canonical-authority";
const DISPOSITION_ABSORBED: &str = "absorbed";
const DISPOSITION_ARCHIVED_WITH_PROVENANCE: &str = "archived-with-provenance";
const DISPOSITION_GENERATED_PROJECTION: &str = "generated-projection";
const ALLOWED_SURFACE_DISPOSITIONS: [&str; 4] = [
    DISPOSITION_CANONICAL_AUTHORITY,
    DISPOSITION_ABSORBED,
    DISPOSITION_ARCHIVED_WITH_PROVENANCE,
    DISPOSITION_GENERATED_PROJECTION,
];
const REQUIRED_SURFACE_DISPOSITIONS: [(&str, &str); 13] = [
    ("/specs/masterplan.json", DISPOSITION_CANONICAL_AUTHORITY),
    (
        "/specs/masterplan.json#v1-legacy-fragments",
        DISPOSITION_ABSORBED,
    ),
    ("/specs/master-plan-sequencing.json", DISPOSITION_ABSORBED),
    (
        "/specs/planning-closure-contract.json",
        DISPOSITION_ABSORBED,
    ),
    (
        "/specs/planning-closure-status-closure-ledger.json",
        DISPOSITION_ABSORBED,
    ),
    ("docs/MASTERPLAN.md", DISPOSITION_GENERATED_PROJECTION),
    ("docs/ROADMAP.md", DISPOSITION_ARCHIVED_WITH_PROVENANCE),
    (
        ".omc/ultragoal/goals.json",
        DISPOSITION_ARCHIVED_WITH_PROVENANCE,
    ),
    (".omc/**", DISPOSITION_ARCHIVED_WITH_PROVENANCE),
    (".omx/**", DISPOSITION_ARCHIVED_WITH_PROVENANCE),
    (".gjc/**", DISPOSITION_ARCHIVED_WITH_PROVENANCE),
    ("~/.gjc/**", DISPOSITION_ARCHIVED_WITH_PROVENANCE),
    ("~/.omx/**", DISPOSITION_ARCHIVED_WITH_PROVENANCE),
];

/// The blocking codes, in canonical order. The fixtures pin exact subsets.
pub const VIOLATION_CODES: [&str; 23] = [
    "orphan_decision",
    "unpropagated_decision",
    "status_disagreement",
    "generated_face_drift",
    "dual_decision_collision",
    "decision_id_mismatch",
    "supersession_half_edge",
    "phantom_decision_citation",
    "masterplan_not_sole_live_authority",
    "masterplan_surface_disposition_incomplete",
    "masterplan_work_item_id_collision",
    "masterplan_external_live_work_item_id",
    "masterplan_dependency_dag_invalid",
    "masterplan_program_coverage_incomplete",
    "masterplan_sequencing_invalid",
    "masterplan_execution_wave_dispatch_unratified",
    "masterplan_evidence_state_invalid",
    "masterplan_plan_evidence_drift",
    "masterplan_plan_evidence_unrecorded",
    "masterplan_projection_freshness_invalid",
    "masterplan_projection_stale",
    "masterplan_read_contract_invalid",
    "masterplan_entry_surface_invalid",
];

/// The gate report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

/// A keyed violation: the bare `code` (the existing contract) PLUS the stable `key`
/// that identifies the offending unit. The going-live ratchet baselines per
/// `(code, key)`; `evaluate()` is the bare-code projection of `evaluate_keyed()`.
/// Keys for this gate are: the decision `id` (orphan/unpropagated/dual),
/// `{decision_id}@{face}` (status_disagreement), `{source_id}->{target_id}`
/// (supersession_half_edge), `{cited_id}@{source_path}` (phantom_decision_citation),
/// `{shared-value}@{sorted face names}` (generated_face_drift),
/// `masterplan_v2.surface_dispositions` keys for legacy surface or harness-store
/// disposition coverage, `masterplan_v2.dependency_edges`-scoped keys for
/// malformed/unknown/cyclic dependency edges, `program_coverage`-scoped keys
/// for missing program or microservice manifest-index coverage,
/// `masterplan_v2.plan_vs_evidence` keys for evidence-audited status/evidence
/// drift, `masterplan_v2.projection_freshness` keys for generated/read projection
/// freshness coverage, `masterplan_v2.read_contracts` keys for stale archived
/// read paths referenced as non-archive surfaces, `masterplan_v2.entry_surface`
/// keys for root-hub allowlist drift, and `masterplan_v2.sequencing` keys for
/// zero-based order/wave and founder-ratification dispatch violations.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
}

impl Finding {
    fn new(code: &str, key: &str) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectionObligation {
    path: String,
    projection_class: String,
    read_timing_class: String,
    freshness_rule: String,
}

impl Report {
    fn from_violations(violations: BTreeSet<String>) -> Self {
        let verdict = if violations.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        };
        Self {
            verdict,
            violations,
        }
    }
}

/// Evaluate a cross-artifact-agreement fixture/corpus `Value` into a report.
///
/// The fixture shape mirrors the on-disk crosswalk face:
/// ```jsonc
/// {
///   "decisions": [
///     {
///       "id": "ADR-0515",
///       "status": "Accepted",          // the ADR front-matter status
///       "in_spec": true,               // appears in the spec corpus
///       "in_masterplan": true,         // appears as a masterplan node
///       "in_roadmap": true,            // appears as a roadmap/sequencing node
///       "supersedes": ["ADR-0511"],
///       "superseded_by": [],
///       "face_statuses": {             // status as each face records it (optional)
///         "roadmap": "Accepted"
///       }
///     }
///   ],
///   "duplicate_ids": ["ADR-0377"],     // ids carried by >1 decision file (DATA signal)
///   "id_mismatches": [                 // filename id != front-matter id (the collision mask)
///     "ADR-0552-x.md:ADR-0552!=ADR-0553"
///   ],
///   "phantom_citations": [             // cited id with no decision file (phantom-0397 shape)
///     "ADR-0397@docs/decisions/ADR-0478-oya-billing-bespoke-billing-engine.md"
///   ],
///   "next_free_id": "ADR-0554",        // allocator output (producer --next-adr)
///   "generated_face_axes": {           // shared values two generated faces must agree on
///     "catalog.json": 6,
///     "contracts.json": 7
///   }
/// }
/// ```
/// Bare-code projection of [`evaluate_keyed`]: identical detection logic, keys dropped.
/// Every `tc-*.json` fixture + the born-blocking self-test keep asserting bare codes
/// against it byte-for-byte.
pub fn evaluate(fixture: &Value) -> Report {
    let violations = evaluate_keyed(fixture)
        .into_iter()
        .map(|finding| finding.code)
        .collect();
    Report::from_violations(violations)
}

/// Evaluate a cross-artifact-agreement corpus into the keyed finding set — the single
/// source of truth for the gate's detection logic.
pub fn evaluate_keyed(fixture: &Value) -> BTreeSet<Finding> {
    let mut findings = validate_payload_shape(fixture);

    let decisions: Vec<&Value> = fixture
        .get("decisions")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter(|decision| valid_decision_shape(decision))
                .collect()
        })
        .unwrap_or_default();

    // Index every decision's supersession edges so half-edges can be detected
    // symmetrically (A.supersedes vs B.superseded_by, and the reverse).
    let mut supersedes: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut superseded_by: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut known_ids: BTreeSet<String> = BTreeSet::new();
    for decision in &decisions {
        let Some(id) = decision.get("id").and_then(Value::as_str) else {
            continue;
        };
        known_ids.insert(id.to_owned());
        supersedes.insert(id.to_owned(), str_set(decision, "supersedes"));
        superseded_by.insert(id.to_owned(), str_set(decision, "superseded_by"));
    }

    for decision in &decisions {
        evaluate_decision(decision, &mut findings);
    }

    // supersession_half_edge: every edge must be reciprocal. Keyed by the directed pair.
    for (id, targets) in &supersedes {
        for target in targets {
            // Only assert reciprocity when the counterpart decision is in-corpus; an
            // edge to an out-of-corpus id is not evidence of a half-edge here.
            if known_ids.contains(target)
                && !superseded_by
                    .get(target)
                    .is_some_and(|set| set.contains(id))
            {
                findings.insert(Finding::new(
                    "supersession_half_edge",
                    &format!("{id}->{target}"),
                ));
            }
        }
    }
    for (id, sources) in &superseded_by {
        for source in sources {
            if known_ids.contains(source)
                && !supersedes.get(source).is_some_and(|set| set.contains(id))
            {
                findings.insert(Finding::new(
                    "supersession_half_edge",
                    &format!("{source}->{id}"),
                ));
            }
        }
    }

    // dual_decision_collision: an id carried by more than one decision file. Keyed by id.
    for id in str_array(fixture, "duplicate_ids") {
        findings.insert(Finding::new("dual_decision_collision", &id));
    }

    // decision_id_mismatch: a decision file whose front-matter id disagrees with its
    // filename number. The producer keys its dup map by the front-matter id, so a
    // mismatch silently re-keys the file and can MASK a dual_decision_collision
    // (FRIC-1781320000); it is therefore a violation in its own right. Keyed by the
    // producer's `<file>:<filename-id>!=<front-matter-id>` entry.
    for entry in str_array(fixture, "id_mismatches") {
        findings.insert(Finding::new("decision_id_mismatch", &entry));
    }

    // phantom_decision_citation: a governed surface cites a decision id with no decision
    // file on disk (the phantom-0397 shape — FRIC-1781430000). Keyed by the producer's
    // `<cited-id>@<source-path>` edge. The carve-out for the ledgered historical
    // inventory is producer-side DATA, never an evaluator branch, so anything in the
    // face IS a violation.
    for entry in str_array(fixture, "phantom_citations") {
        findings.insert(Finding::new("phantom_decision_citation", &entry));
    }

    // generated_face_drift: two generated faces disagree on a shared value. Keyed by
    // "<shared-value-name>@{sorted face names}".
    if let Some(axes) = fixture
        .get("generated_face_axes")
        .and_then(Value::as_object)
    {
        let distinct: BTreeSet<String> = axes.values().map(|value| value.to_string()).collect();
        if distinct.len() > 1 {
            let faces: Vec<&str> = axes.keys().map(String::as_str).collect();
            // BTreeMap keys are already sorted; join the disagreeing face names.
            let key = format!("axes_count@{{{}}}", faces.join(","));
            findings.insert(Finding::new("generated_face_drift", &key));
        }
    }
    if fixture.get("masterplan_v2").is_some() {
        findings.extend(evaluate_masterplan_v2_authority(fixture));
        findings.extend(evaluate_masterplan_v2_evidence_state(fixture));
        findings.extend(evaluate_masterplan_v2_plan_evidence_drift(fixture));
        findings.extend(evaluate_masterplan_v2_sequencing(fixture));
        if masterplan_projection_freshness_present(fixture) {
            findings.extend(evaluate_masterplan_v2_projection_freshness(
                fixture,
                fixture.get("generated_artifact_control_plane"),
            ));
        }
        if masterplan_read_contract_gate_present(fixture) {
            findings.extend(evaluate_masterplan_v2_read_contract_archives(fixture));
        }
        if let Some(read_surface_corpus) = fixture.get("read_surface_corpus") {
            findings.extend(evaluate_masterplan_read_surface_resurrections(
                fixture,
                read_surface_corpus,
            ));
        }
        if let Some(corpus) = fixture.get("projection_rederivation") {
            findings.extend(evaluate_masterplan_projection_rederivation(fixture, corpus));
        }
        if let Some(corpus) = fixture.get("plan_evidence_crosscheck") {
            findings.extend(evaluate_masterplan_plan_evidence_crosscheck(
                fixture, corpus,
            ));
        }
        if let Some(root_hub) = fixture.get("root_hub_pointers") {
            findings.extend(evaluate_masterplan_v2_entry_surfaces(fixture, root_hub));
        }
    }
    if let Some(manifest_index) = fixture.get("microservices_manifest_index") {
        findings.extend(evaluate_masterplan_v2_program_coverage(
            fixture,
            manifest_index,
        ));
    }

    findings
}
/// Evaluate the masterplan v2 authority contract embedded in `/specs/masterplan.json`.
///
/// This is the Sub-AC 1/2 guard: exactly one live plan authority, exactly one live
/// work-item ID namespace, no duplicate live ids, no external live ids smuggled
/// in through legacy surfaces or imported cards, and a valid dependency DAG.
pub fn evaluate_masterplan_v2_authority(masterplan: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    let Some(v2) = masterplan.get("masterplan_v2").and_then(Value::as_object) else {
        findings.insert(Finding::new(
            "masterplan_not_sole_live_authority",
            "<missing-masterplan_v2>",
        ));
        return findings;
    };

    let authority = v2.get("canonical_plan_authority");
    let authority_path = authority
        .and_then(|value| value.get("path"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let id_space = authority.and_then(|value| value.get("live_work_item_id_space"));
    let id_authority_path = id_space
        .and_then(|value| value.get("authority_path"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let id_prefix = id_space
        .and_then(|value| value.get("id_prefix"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let numeric_width = id_space
        .and_then(|value| value.get("numeric_width"))
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let external_live_ids_allowed = id_space
        .and_then(|value| value.get("external_live_ids_allowed"))
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let duplicate_ids_allowed = id_space
        .and_then(|value| value.get("duplicate_ids_allowed"))
        .and_then(Value::as_bool)
        .unwrap_or(true);

    if authority_path != "/specs/masterplan.json"
        || id_authority_path != "/specs/masterplan.json"
        || id_prefix.is_empty()
        || numeric_width == 0
        || external_live_ids_allowed
        || duplicate_ids_allowed
    {
        findings.insert(Finding::new(
            "masterplan_not_sole_live_authority",
            "<canonical-plan-authority>",
        ));
    }

    evaluate_surface_dispositions(v2.get("surface_dispositions"), &mut findings);
    let work_item_ids = evaluate_masterplan_work_items(
        v2.get("work_items"),
        id_prefix,
        numeric_width,
        &mut findings,
    );
    evaluate_masterplan_dependency_dag(
        v2.get("dependency_edges"),
        v2.get("dependency_edge_semantics"),
        v2.get("cross_program_edge_policy"),
        &work_item_ids,
        &masterplan_work_item_programs(v2.get("work_items")),
        &masterplan_program_id_set(v2.get("programs")),
        &mut findings,
    );

    findings
}
/// Evaluate masterplan v2 completion evidence-state projections.
///
/// Hermes done-card imports are forensic completion claims, not verified plan
/// completion. A Hermes source card in a `done` column may only surface as
/// `claimed-done-unverified` until evidence refs attach; all masterplan work-item
/// completion statuses likewise require evidence references.
pub fn evaluate_masterplan_v2_evidence_state(masterplan: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    let Some(v2) = masterplan.get("masterplan_v2").and_then(Value::as_object) else {
        findings.insert(Finding::new(
            "masterplan_evidence_state_invalid",
            "<missing-masterplan_v2>",
        ));
        return findings;
    };

    evaluate_work_item_completion_evidence(v2.get("work_items"), &mut findings);
    evaluate_hermes_done_card_imports(v2.get("hermes_done_card_imports"), &mut findings);

    findings
}
/// Evaluate masterplan v2 plan-vs-evidence drift.
///
/// This is the Sub-AC 3 guard for the evidence-audited MPV2 contract: status
/// claims stay tied to auditable evidence refs, legacy/local planning stores
/// cannot be laundered into completion evidence, and Hermes done-card imports
/// only surface as verified completion after evidence attaches.
pub fn evaluate_masterplan_v2_plan_evidence_drift(masterplan: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    let Some(v2) = masterplan.get("masterplan_v2").and_then(Value::as_object) else {
        findings.insert(Finding::new(
            "masterplan_plan_evidence_drift",
            "<missing-masterplan_v2>",
        ));
        return findings;
    };

    evaluate_plan_evidence_policy(v2.get("evidence_state_policy"), &mut findings);
    evaluate_work_item_plan_evidence_drift(v2.get("work_items"), &mut findings);
    evaluate_hermes_plan_evidence_drift(v2.get("hermes_done_card_imports"), &mut findings);

    findings
}

/// Evaluate freshness coverage for every generated/read projection whose live
/// truth resolves to `/specs/masterplan.json#masterplan_v2`.
///
/// The obligations are intentionally derived, not hand-listed:
/// - every non-canonical `read_contract` row is a read projection;
/// - every `surface_dispositions` row marked `generated-projection` is a
///   generated projection;
/// - every generated-artifact-control-plane row whose `source_inputs` include
///   `specs/masterplan.json` is a generated projection.
pub fn evaluate_masterplan_v2_projection_freshness(
    masterplan: &Value,
    generated_artifact_control_plane: Option<&Value>,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    let Some(v2) = masterplan.get("masterplan_v2") else {
        findings.insert(Finding::new(
            "masterplan_projection_freshness_invalid",
            "<missing-masterplan_v2>",
        ));
        return findings;
    };
    if !v2.is_object() {
        findings.insert(Finding::new(
            "masterplan_projection_freshness_invalid",
            "<malformed-masterplan_v2>",
        ));
        return findings;
    }

    let obligations = projection_obligations(v2, generated_artifact_control_plane, &mut findings);
    if obligations.is_empty() {
        findings.insert(Finding::new(
            "masterplan_projection_freshness_invalid",
            "<empty-projection-obligations>",
        ));
    }

    let Some(freshness) = v2.get("projection_freshness") else {
        findings.insert(Finding::new(
            "masterplan_projection_freshness_invalid",
            "<missing-projection_freshness>",
        ));
        return findings;
    };
    if freshness.get("source_of_truth").and_then(Value::as_str) != Some(MASTERPLAN_V2_REF) {
        findings.insert(Finding::new(
            "masterplan_projection_freshness_invalid",
            "projection_freshness.source_of_truth",
        ));
    }
    if freshness.get("validator").and_then(Value::as_str) != Some(PROJECTION_FRESHNESS_VALIDATOR) {
        findings.insert(Finding::new(
            "masterplan_projection_freshness_invalid",
            "projection_freshness.validator",
        ));
    }
    if freshness
        .get("single_writer_mutation_path")
        .and_then(Value::as_str)
        != Some("/specs/masterplan.json")
    {
        findings.insert(Finding::new(
            "masterplan_projection_freshness_invalid",
            "projection_freshness.single_writer_mutation_path",
        ));
    }

    let Some(rows) = freshness.get("projections").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "masterplan_projection_freshness_invalid",
            "projection_freshness.projections",
        ));
        return findings;
    };
    if rows.is_empty() {
        findings.insert(Finding::new(
            "masterplan_projection_freshness_invalid",
            "projection_freshness.projections",
        ));
        return findings;
    }

    let mut rows_by_path: BTreeMap<String, (&Value, usize)> = BTreeMap::new();
    for (index, row) in rows.iter().enumerate() {
        let Some(path) = row
            .get("path")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|path| !path.is_empty())
        else {
            findings.insert(Finding::new(
                "masterplan_projection_freshness_invalid",
                &format!("projection_freshness.projections[{index}].path"),
            ));
            continue;
        };
        if rows_by_path.insert(path.to_owned(), (row, index)).is_some() {
            findings.insert(Finding::new(
                "masterplan_projection_freshness_invalid",
                &format!("{path}.duplicate"),
            ));
        }
    }

    for (path, obligation) in &obligations {
        match rows_by_path.get(path) {
            Some((row, index)) => {
                validate_projection_freshness_row(row, *index, obligation, &mut findings);
            }
            None => {
                findings.insert(Finding::new(
                    "masterplan_projection_freshness_invalid",
                    path,
                ));
            }
        }
    }

    for path in rows_by_path.keys() {
        if !obligations.contains_key(path) {
            findings.insert(Finding::new(
                "masterplan_projection_freshness_invalid",
                &format!("{path}.unexpected"),
            ));
        }
    }

    findings
}

fn masterplan_projection_freshness_present(masterplan: &Value) -> bool {
    masterplan.get("generated_artifact_control_plane").is_some()
        || masterplan
            .get("masterplan_v2")
            .and_then(|v2| v2.get("projection_freshness"))
            .is_some()
        || masterplan
            .get("masterplan_v2")
            .and_then(|v2| v2.get("read_contracts"))
            .is_some()
}
/// Evaluate the masterplan v2 read-contract archive guard.
///
/// Stale read paths are the surfaces explicitly archived with provenance by the
/// consolidation. Any surviving reference to such a path must be archive-only:
/// a `read_contract`, projection freshness row, or explicit read-path reference
/// that points at an archived path must carry `read_timing_class:
/// provenance-archive`.
pub fn evaluate_masterplan_v2_read_contract_archives(masterplan: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    let Some(v2) = masterplan.get("masterplan_v2").and_then(Value::as_object) else {
        findings.insert(Finding::new(
            "masterplan_read_contract_invalid",
            "<missing-masterplan_v2>",
        ));
        return findings;
    };

    let archived_paths = archived_read_paths(v2.get("surface_dispositions"));
    if archived_paths.is_empty() {
        return findings;
    }

    evaluate_archived_read_contract_rows(v2.get("read_contracts"), &archived_paths, &mut findings);
    evaluate_archived_projection_freshness_rows(
        v2.get("projection_freshness"),
        &archived_paths,
        &mut findings,
    );
    evaluate_archived_explicit_read_path_references(
        v2.get("read_path_references"),
        &archived_paths,
        &mut findings,
    );

    findings
}

/// Evaluate the masterplan v2 bounded entry-surface contract.
///
/// The root hub owns the small allowlist of artifacts agents may treat as
/// mandatory entry surfaces. The masterplan read contracts must mark exactly
/// that same set as `entry-surface`, and no root-hub entrypoint that is marked
/// absorbed, retired, historical, provenance-only, or superseded may re-enter
/// the mandatory-read surface.
pub fn evaluate_masterplan_v2_entry_surfaces(
    masterplan: &Value,
    root_hub: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    let Some(v2) = masterplan.get("masterplan_v2").and_then(Value::as_object) else {
        findings.insert(Finding::new(
            "masterplan_entry_surface_invalid",
            "<missing-masterplan_v2>",
        ));
        return findings;
    };

    let Some(allowlist) = root_hub.get("agent_entry_surface_allowlist") else {
        findings.insert(Finding::new(
            "masterplan_entry_surface_invalid",
            "root_hub.agent_entry_surface_allowlist",
        ));
        return findings;
    };
    if !allowlist.is_object() {
        findings.insert(Finding::new(
            "masterplan_entry_surface_invalid",
            "root_hub.agent_entry_surface_allowlist",
        ));
        return findings;
    }

    if non_empty_field(allowlist, "read_timing_class") != Some(READ_CONTRACT_ENTRY_TIMING_CLASS) {
        findings.insert(Finding::new(
            "masterplan_entry_surface_invalid",
            "root_hub.agent_entry_surface_allowlist.read_timing_class",
        ));
    }
    if non_empty_field(allowlist, "validator") != Some(ENTRY_SURFACE_VALIDATOR) {
        findings.insert(Finding::new(
            "masterplan_entry_surface_invalid",
            "root_hub.agent_entry_surface_allowlist.validator",
        ));
    }
    if non_empty_field(allowlist, "source_of_truth") != Some(ENTRY_SURFACE_ALLOWLIST_REF) {
        findings.insert(Finding::new(
            "masterplan_entry_surface_invalid",
            "root_hub.agent_entry_surface_allowlist.source_of_truth",
        ));
    }

    let allowed_paths = collect_entry_surface_path_array(
        allowlist.get("paths"),
        "root_hub.agent_entry_surface_allowlist.paths",
        &mut findings,
    );
    let superseded_paths = collect_entry_surface_path_array(
        allowlist.get("superseded_entrypoints"),
        "root_hub.agent_entry_surface_allowlist.superseded_entrypoints",
        &mut findings,
    );
    let actual_paths =
        collect_masterplan_entry_surface_paths(v2.get("read_contracts"), &mut findings);

    match root_hub.get("entry_points").and_then(Value::as_object) {
        Some(entry_points) => {
            for (normalized_path, display_path) in &allowed_paths {
                let mut found = false;
                let mut stale = false;
                for entry in entry_points.values() {
                    if root_hub_entry_current_path_normalized(entry).as_deref()
                        == Some(normalized_path.as_str())
                    {
                        found = true;
                        stale |= root_hub_entrypoint_is_superseded(entry);
                    }
                }
                if !found {
                    findings.insert(Finding::new(
                        "masterplan_entry_surface_invalid",
                        &format!("{display_path}.root_hub_entry_points"),
                    ));
                }
                if stale {
                    findings.insert(Finding::new(
                        "masterplan_entry_surface_invalid",
                        &format!("{display_path}.root_hub_entry_superseded"),
                    ));
                }
            }
        }
        None => {
            findings.insert(Finding::new(
                "masterplan_entry_surface_invalid",
                "root_hub.entry_points",
            ));
        }
    }

    for (normalized_path, display_path) in &allowed_paths {
        if !actual_paths.contains_key(normalized_path) {
            findings.insert(Finding::new(
                "masterplan_entry_surface_invalid",
                &format!("{display_path}.missing_entry_surface_read_contract"),
            ));
        }
        if superseded_paths.contains_key(normalized_path) {
            findings.insert(Finding::new(
                "masterplan_entry_surface_invalid",
                &format!("{display_path}.allowlisted_superseded_entrypoint"),
            ));
        }
    }

    for (normalized_path, display_path) in &actual_paths {
        if !allowed_paths.contains_key(normalized_path) {
            findings.insert(Finding::new(
                "masterplan_entry_surface_invalid",
                &format!("{display_path}.unexpected_entry_surface_read_contract"),
            ));
        }
        if superseded_paths.contains_key(normalized_path) {
            findings.insert(Finding::new(
                "masterplan_entry_surface_invalid",
                &format!("{display_path}.superseded_entry_surface_read_contract"),
            ));
        }
    }

    findings
}

fn collect_masterplan_entry_surface_paths(
    read_contracts: Option<&Value>,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, String> {
    let mut paths = BTreeMap::new();
    let Some(read_contracts) = read_contracts.and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "masterplan_entry_surface_invalid",
            "masterplan_v2.read_contracts",
        ));
        return paths;
    };

    for (index, contract) in read_contracts.iter().enumerate() {
        if non_empty_field(contract, "read_timing_class") != Some(READ_CONTRACT_ENTRY_TIMING_CLASS)
        {
            continue;
        }
        let Some(path) = non_empty_field(contract, "path") else {
            findings.insert(Finding::new(
                "masterplan_entry_surface_invalid",
                &format!("masterplan_v2.read_contracts[{index}].path"),
            ));
            continue;
        };
        if !read_contract_audience_contains(contract, "agents") {
            findings.insert(Finding::new(
                "masterplan_entry_surface_invalid",
                &format!("{path}.read_contract.audience.agents"),
            ));
        }
        let normalized = normalize_read_path_for_match(path);
        if normalized.is_empty() {
            findings.insert(Finding::new(
                "masterplan_entry_surface_invalid",
                &format!("masterplan_v2.read_contracts[{index}].path"),
            ));
            continue;
        }
        if paths.insert(normalized, path.to_owned()).is_some() {
            findings.insert(Finding::new(
                "masterplan_entry_surface_invalid",
                &format!("{path}.duplicate_entry_surface_read_contract"),
            ));
        }
    }

    if paths.is_empty() {
        findings.insert(Finding::new(
            "masterplan_entry_surface_invalid",
            "<empty-masterplan-entry-surface-read-contracts>",
        ));
    }

    paths
}

fn collect_entry_surface_path_array(
    value: Option<&Value>,
    field_key: &str,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, String> {
    let mut paths = BTreeMap::new();
    let Some(values) = value.and_then(Value::as_array) else {
        findings.insert(Finding::new("masterplan_entry_surface_invalid", field_key));
        return paths;
    };

    if values.is_empty() {
        findings.insert(Finding::new(
            "masterplan_entry_surface_invalid",
            &format!("{field_key}.empty"),
        ));
        return paths;
    }

    for (index, value) in values.iter().enumerate() {
        let Some(path) = value
            .as_str()
            .map(str::trim)
            .filter(|path| !path.is_empty())
        else {
            findings.insert(Finding::new(
                "masterplan_entry_surface_invalid",
                &format!("{field_key}[{index}]"),
            ));
            continue;
        };
        let normalized = normalize_read_path_for_match(path);
        if normalized.is_empty() {
            findings.insert(Finding::new(
                "masterplan_entry_surface_invalid",
                &format!("{field_key}[{index}]"),
            ));
            continue;
        }
        if paths.insert(normalized, path.to_owned()).is_some() {
            findings.insert(Finding::new(
                "masterplan_entry_surface_invalid",
                &format!("{path}.duplicate_entry_surface_allowlist_path"),
            ));
        }
    }

    paths
}

fn root_hub_entry_current_path_normalized(entry: &Value) -> Option<String> {
    non_empty_field(entry, "current_path").map(normalize_read_path_for_match)
}

fn root_hub_entrypoint_is_superseded(entry: &Value) -> bool {
    entry.get("current_path").is_some_and(Value::is_null)
        || root_hub_status_field_has_stale_marker(entry, "authority_status")
        || root_hub_status_field_has_stale_marker(entry, "current_path_status")
        || root_hub_status_field_has_stale_marker(entry, "migration_phase")
        || root_hub_status_field_has_stale_marker(entry, "status")
}

fn root_hub_status_field_has_stale_marker(entry: &Value, field: &str) -> bool {
    const STALE_ENTRYPOINT_MARKERS: [&str; 5] = [
        "superseded",
        "retired",
        "provenance",
        "historical",
        "absorbed",
    ];
    non_empty_field(entry, field).is_some_and(|value| {
        let value = value.to_ascii_lowercase();
        STALE_ENTRYPOINT_MARKERS
            .iter()
            .any(|marker| value.contains(marker))
    })
}

fn read_contract_audience_contains(contract: &Value, expected: &str) -> bool {
    contract
        .get("audience")
        .and_then(Value::as_array)
        .is_some_and(|audiences| {
            audiences
                .iter()
                .any(|audience| audience.as_str() == Some(expected))
        })
}
fn masterplan_read_contract_gate_present(masterplan: &Value) -> bool {
    masterplan.get("masterplan_v2").is_some_and(|v2| {
        v2.get("read_contracts").is_some()
            || v2.get("projection_freshness").is_some()
            || v2.get("read_path_references").is_some()
    })
}

fn archived_read_paths(surfaces: Option<&Value>) -> Vec<String> {
    let Some(surfaces) = surfaces.and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut paths = BTreeSet::new();
    for surface in surfaces {
        if non_empty_field(surface, "disposition") != Some(DISPOSITION_ARCHIVED_WITH_PROVENANCE) {
            continue;
        }
        if let Some(path) = non_empty_field(surface, "path") {
            paths.insert(path.to_owned());
        }
    }

    paths.into_iter().collect()
}

fn evaluate_archived_read_contract_rows(
    read_contracts: Option<&Value>,
    archived_paths: &[String],
    findings: &mut BTreeSet<Finding>,
) {
    let Some(read_contracts) = read_contracts.and_then(Value::as_array) else {
        return;
    };

    for contract in read_contracts {
        let Some(path) = non_empty_field(contract, "path") else {
            continue;
        };
        if read_path_is_archived(path, archived_paths)
            && non_empty_field(contract, "read_timing_class")
                != Some(READ_CONTRACT_ARCHIVED_TIMING_CLASS)
        {
            findings.insert(Finding::new(
                "masterplan_read_contract_invalid",
                &format!("{path}.read_contract.read_timing_class"),
            ));
        }
    }
}

fn evaluate_archived_projection_freshness_rows(
    freshness: Option<&Value>,
    archived_paths: &[String],
    findings: &mut BTreeSet<Finding>,
) {
    let Some(rows) = freshness
        .and_then(|value| value.get("projections"))
        .and_then(Value::as_array)
    else {
        return;
    };

    for row in rows {
        let Some(path) = non_empty_field(row, "path") else {
            continue;
        };
        if read_path_is_archived(path, archived_paths)
            && non_empty_field(row, "read_timing_class")
                != Some(READ_CONTRACT_ARCHIVED_TIMING_CLASS)
        {
            findings.insert(Finding::new(
                "masterplan_read_contract_invalid",
                &format!("{path}.projection_freshness.read_timing_class"),
            ));
        }
    }
}

fn evaluate_archived_explicit_read_path_references(
    references: Option<&Value>,
    archived_paths: &[String],
    findings: &mut BTreeSet<Finding>,
) {
    let Some(references) = references.and_then(Value::as_array) else {
        return;
    };

    for (index, reference) in references.iter().enumerate() {
        let Some(path) = non_empty_field(reference, "path")
            .or_else(|| non_empty_field(reference, "target_path"))
        else {
            continue;
        };
        let read_timing_class = non_empty_field(reference, "read_timing_class")
            .or_else(|| non_empty_field(reference, "reference_timing_class"));

        if read_path_is_archived(path, archived_paths)
            && read_timing_class != Some(READ_CONTRACT_ARCHIVED_TIMING_CLASS)
        {
            findings.insert(Finding::new(
                "masterplan_read_contract_invalid",
                &format!("{path}.read_path_references[{index}].read_timing_class"),
            ));
        }
    }
}

fn read_path_is_archived(path: &str, archived_paths: &[String]) -> bool {
    archived_paths
        .iter()
        .any(|archived_path| archived_read_path_matches(path, archived_path))
}

fn archived_read_path_matches(path: &str, archived_path: &str) -> bool {
    let path = normalize_read_path_for_match(path);
    let archived_path = normalize_read_path_for_match(archived_path);
    if path == archived_path {
        return true;
    }

    archived_path.strip_suffix("/**").is_some_and(|prefix| {
        path == prefix
            || path
                .strip_prefix(prefix)
                .is_some_and(|suffix| suffix.starts_with('/'))
    })
}

fn normalize_read_path_for_match(path: &str) -> String {
    let base = path
        .trim()
        .split_once('#')
        .map_or(path.trim(), |(base, _)| base.trim());
    let mut normalized = base;
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped;
    }
    normalized
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_owned()
}

fn projection_obligations(
    v2: &Value,
    generated_artifact_control_plane: Option<&Value>,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, ProjectionObligation> {
    let surface_dispositions = surface_dispositions_by_path(v2.get("surface_dispositions"));
    let mut obligations = BTreeMap::new();

    let Some(read_contracts) = v2.get("read_contracts").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "masterplan_projection_freshness_invalid",
            "read_contracts",
        ));
        return obligations;
    };

    for (index, contract) in read_contracts.iter().enumerate() {
        let Some(path) = contract
            .get("path")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|path| !path.is_empty())
        else {
            findings.insert(Finding::new(
                "masterplan_projection_freshness_invalid",
                &format!("read_contracts[{index}].path"),
            ));
            continue;
        };
        if path == "/specs/masterplan.json" {
            continue;
        }
        let Some(read_timing_class) = contract
            .get("read_timing_class")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            findings.insert(Finding::new(
                "masterplan_projection_freshness_invalid",
                &format!("{path}.read_timing_class"),
            ));
            continue;
        };
        let Some(freshness_rule) = contract
            .get("freshness_rule")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            findings.insert(Finding::new(
                "masterplan_projection_freshness_invalid",
                &format!("{path}.freshness_rule"),
            ));
            continue;
        };
        let projection_class = if surface_dispositions
            .get(path)
            .is_some_and(|disposition| disposition == DISPOSITION_GENERATED_PROJECTION)
        {
            DISPOSITION_GENERATED_PROJECTION
        } else {
            PROJECTION_CLASS_READ
        };
        obligations.insert(
            path.to_owned(),
            ProjectionObligation {
                path: path.to_owned(),
                projection_class: projection_class.to_owned(),
                read_timing_class: read_timing_class.to_owned(),
                freshness_rule: freshness_rule.to_owned(),
            },
        );
    }

    for (path, disposition) in &surface_dispositions {
        if disposition == DISPOSITION_GENERATED_PROJECTION && !obligations.contains_key(path) {
            findings.insert(Finding::new(
                "masterplan_projection_freshness_invalid",
                &format!("{path}.read_contract"),
            ));
            obligations.insert(
                path.to_owned(),
                ProjectionObligation {
                    path: path.to_owned(),
                    projection_class: DISPOSITION_GENERATED_PROJECTION.to_owned(),
                    read_timing_class: String::new(),
                    freshness_rule: String::new(),
                },
            );
        }
    }

    add_generated_artifact_projection_obligations(
        generated_artifact_control_plane,
        &mut obligations,
        findings,
    );

    obligations
}

fn surface_dispositions_by_path(surfaces: Option<&Value>) -> BTreeMap<String, String> {
    let mut dispositions = BTreeMap::new();
    let Some(surfaces) = surfaces.and_then(Value::as_array) else {
        return dispositions;
    };
    for surface in surfaces {
        let Some(path) = surface
            .get("path")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|path| !path.is_empty())
        else {
            continue;
        };
        let Some(disposition) = surface
            .get("disposition")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        dispositions.insert(path.to_owned(), disposition.to_owned());
    }
    dispositions
}

fn add_generated_artifact_projection_obligations(
    generated_artifact_control_plane: Option<&Value>,
    obligations: &mut BTreeMap<String, ProjectionObligation>,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(control_plane) = generated_artifact_control_plane else {
        return;
    };
    let Some(artifacts) = control_plane.get("artifacts").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "masterplan_projection_freshness_invalid",
            "generated_artifact_control_plane.artifacts",
        ));
        return;
    };
    for (index, artifact) in artifacts.iter().enumerate() {
        if !artifact_source_inputs_include_masterplan(artifact) {
            continue;
        }
        let Some(path) = artifact
            .get("path")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|path| !path.is_empty())
        else {
            findings.insert(Finding::new(
                "masterplan_projection_freshness_invalid",
                &format!("generated_artifact_control_plane.artifacts[{index}].path"),
            ));
            continue;
        };
        match obligations.get_mut(path) {
            Some(obligation) => {
                obligation.projection_class = DISPOSITION_GENERATED_PROJECTION.to_owned();
            }
            None => {
                findings.insert(Finding::new(
                    "masterplan_projection_freshness_invalid",
                    &format!("{path}.read_contract"),
                ));
                obligations.insert(
                    path.to_owned(),
                    ProjectionObligation {
                        path: path.to_owned(),
                        projection_class: DISPOSITION_GENERATED_PROJECTION.to_owned(),
                        read_timing_class: String::new(),
                        freshness_rule: String::new(),
                    },
                );
            }
        }
    }
}

fn artifact_source_inputs_include_masterplan(artifact: &Value) -> bool {
    artifact
        .get("source_inputs")
        .and_then(Value::as_array)
        .is_some_and(|inputs| {
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

fn validate_projection_freshness_row(
    row: &Value,
    index: usize,
    obligation: &ProjectionObligation,
    findings: &mut BTreeSet<Finding>,
) {
    let path = obligation.path.as_str();
    expect_string_field(
        row,
        "projection_class",
        obligation.projection_class.as_str(),
        &format!("{path}.projection_class"),
        findings,
    );
    expect_string_field(
        row,
        "source_of_truth",
        MASTERPLAN_V2_REF,
        &format!("{path}.source_of_truth"),
        findings,
    );
    expect_string_field(
        row,
        "freshness_gate",
        PROJECTION_FRESHNESS_VALIDATOR,
        &format!("{path}.freshness_gate"),
        findings,
    );
    expect_string_field(
        row,
        "conflict_resolution",
        MASTERPLAN_V2_REF,
        &format!("{path}.conflict_resolution"),
        findings,
    );
    expect_string_field(
        row,
        "single_writer_mutation_path",
        "/specs/masterplan.json",
        &format!("{path}.single_writer_mutation_path"),
        findings,
    );
    expect_string_field(
        row,
        "read_timing_class",
        obligation.read_timing_class.as_str(),
        &format!("{path}.read_timing_class"),
        findings,
    );
    expect_string_field(
        row,
        "freshness_rule",
        obligation.freshness_rule.as_str(),
        &format!("{path}.freshness_rule"),
        findings,
    );
    expect_bool_field(
        row,
        "is_live_plan_authority",
        false,
        &format!("{path}.is_live_plan_authority"),
        findings,
    );
    expect_bool_field(
        row,
        "live_work_item_ids_allowed",
        false,
        &format!("{path}.live_work_item_ids_allowed"),
        findings,
    );
    expect_bool_field(
        row,
        "status_claims_allowed_without_evidence",
        false,
        &format!("{path}.status_claims_allowed_without_evidence"),
        findings,
    );

    let drift_policy = row
        .get("drift_policy")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default();
    if !drift_policy.contains("fail-closed") {
        findings.insert(Finding::new(
            "masterplan_projection_freshness_invalid",
            &format!("{path}.drift_policy"),
        ));
    }
    if !non_empty_string_array(row.get("evidence_refs")) {
        findings.insert(Finding::new(
            "masterplan_projection_freshness_invalid",
            &format!("{path}.evidence_refs"),
        ));
    }
    if row.get("path").and_then(Value::as_str) != Some(path) {
        findings.insert(Finding::new(
            "masterplan_projection_freshness_invalid",
            &format!("projection_freshness.projections[{index}].path"),
        ));
    }
}

fn expect_string_field(
    row: &Value,
    field: &str,
    expected: &str,
    key: &str,
    findings: &mut BTreeSet<Finding>,
) {
    if expected.is_empty() || row.get(field).and_then(Value::as_str) != Some(expected) {
        findings.insert(Finding::new("masterplan_projection_freshness_invalid", key));
    }
}

fn expect_bool_field(
    row: &Value,
    field: &str,
    expected: bool,
    key: &str,
    findings: &mut BTreeSet<Finding>,
) {
    if row.get(field).and_then(Value::as_bool) != Some(expected) {
        findings.insert(Finding::new("masterplan_projection_freshness_invalid", key));
    }
}
/// Evaluate the masterplan v2 program-sharded coverage proof against the
/// microservice manifest index.
///
/// This is the Sub-AC 2/3 guard: every microservice named by
/// `/specs/microservices/manifests-index.json` must be assigned to an existing
/// masterplan program shard, the consolidation must explicitly carry the
/// ontology, workflow-engine, workflow-studio, intelligence, owned-stack, reorg,
/// AST code-graph, and fabric program classes, and the ADR-0537 owned-stack
/// ladder (cloud-kernel → cloud-os → cloud-k8s → cloud-services → products)
/// must map every rung to known covering program shards.
pub fn evaluate_masterplan_v2_program_coverage(
    masterplan: &Value,
    manifest_index: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    let Some(v2) = masterplan.get("masterplan_v2").and_then(Value::as_object) else {
        findings.insert(Finding::new(
            "masterplan_program_coverage_incomplete",
            "<missing-masterplan_v2>",
        ));
        return findings;
    };

    let program_ids = evaluate_program_shards(v2.get("programs"), &mut findings);
    evaluate_required_program_classes(
        v2.get("programs"),
        v2.get("program_coverage"),
        &mut findings,
    );
    evaluate_work_item_program_membership(v2.get("work_items"), &program_ids, &mut findings);
    evaluate_microservice_manifest_coverage(
        v2.get("program_coverage"),
        manifest_index,
        &program_ids,
        &mut findings,
    );
    evaluate_owned_stack_ladder(v2.get("program_coverage"), &program_ids, &mut findings);

    findings
}

/// Evaluate the masterplan v2 sequencing/dispatch guard.
///
/// This is the Sub-AC 4 guard: sequencing must be re-derived from the MPV2 DAG,
/// all order and execution-wave indices must be zero-based and contiguous, and
/// execution-wave dispatch must stay fail-closed until a founder-ratification
/// decision is recorded.
pub fn evaluate_masterplan_v2_sequencing(masterplan: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    let Some(v2) = masterplan.get("masterplan_v2").and_then(Value::as_object) else {
        findings.insert(Finding::new(
            "masterplan_sequencing_invalid",
            "<missing-masterplan_v2>",
        ));
        return findings;
    };
    let Some(sequencing) = v2.get("sequencing").and_then(Value::as_object) else {
        findings.insert(Finding::new(
            "masterplan_sequencing_invalid",
            "<missing-sequencing>",
        ));
        return findings;
    };

    if sequencing.get("derivation_mode").and_then(Value::as_str) != Some(SEQUENCING_DERIVATION_MODE)
    {
        findings.insert(Finding::new(
            "masterplan_sequencing_invalid",
            "masterplan_v2.sequencing.derivation_mode",
        ));
    }
    if sequencing.get("source_of_truth").and_then(Value::as_str) != Some(SEQUENCING_SOURCE_OF_TRUTH)
    {
        findings.insert(Finding::new(
            "masterplan_sequencing_invalid",
            "masterplan_v2.sequencing.source_of_truth",
        ));
    }
    if sequencing.get("index_base").and_then(Value::as_u64) != Some(0) {
        findings.insert(Finding::new(
            "masterplan_sequencing_invalid",
            "masterplan_v2.sequencing.index_base",
        ));
    }
    if sequencing
        .get("legacy_order_imported")
        .and_then(Value::as_bool)
        != Some(false)
    {
        findings.insert(Finding::new(
            "masterplan_sequencing_invalid",
            "masterplan_v2.sequencing.legacy_order_imported",
        ));
    }
    if !non_empty_string_array(sequencing.get("derivation_evidence_refs")) {
        findings.insert(Finding::new(
            "masterplan_sequencing_invalid",
            "masterplan_v2.sequencing.derivation_evidence_refs",
        ));
    }

    let work_item_ids = masterplan_sequence_work_item_ids(v2.get("work_items"));
    if work_item_ids.is_empty() {
        findings.insert(Finding::new(
            "masterplan_sequencing_invalid",
            "<empty-work-item-id-set>",
        ));
    }
    let order_positions = evaluate_zero_based_work_item_order(
        sequencing.get("work_item_order"),
        &work_item_ids,
        &mut findings,
    );
    let wave_positions = evaluate_zero_based_execution_waves(
        sequencing.get("execution_waves"),
        &work_item_ids,
        &mut findings,
    );
    evaluate_sequence_respects_dependencies(
        v2.get("dependency_edges"),
        &order_positions,
        &wave_positions,
        &mut findings,
    );

    let founder_ratified = founder_ratification_recorded(sequencing.get("founder_ratification"));
    if !founder_ratified {
        findings.insert(Finding::new(
            "masterplan_execution_wave_dispatch_unratified",
            "masterplan_v2.sequencing.founder_ratification",
        ));
    }
    evaluate_execution_wave_dispatch(
        sequencing.get("execution_wave_dispatch"),
        founder_ratified,
        &mut findings,
    );

    findings
}

fn masterplan_sequence_work_item_ids(work_items: Option<&Value>) -> BTreeSet<String> {
    work_items
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("id").and_then(Value::as_str))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn evaluate_zero_based_work_item_order(
    work_item_order: Option<&Value>,
    work_item_ids: &BTreeSet<String>,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, usize> {
    let mut positions = BTreeMap::new();
    let Some(order) = work_item_order.and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "masterplan_sequencing_invalid",
            "masterplan_v2.sequencing.work_item_order",
        ));
        return positions;
    };

    if order.len() != work_item_ids.len() {
        findings.insert(Finding::new(
            "masterplan_sequencing_invalid",
            "masterplan_v2.sequencing.work_item_order.cardinality",
        ));
    }

    let mut seen_indexes = BTreeSet::new();
    for (expected_index, entry) in order.iter().enumerate() {
        let Some(entry) = entry.as_object() else {
            findings.insert(Finding::new(
                "masterplan_sequencing_invalid",
                &format!("masterplan_v2.sequencing.work_item_order[{expected_index}]"),
            ));
            continue;
        };

        match entry.get("index").and_then(Value::as_u64) {
            Some(index) => {
                if index != expected_index as u64 {
                    findings.insert(Finding::new(
                        "masterplan_sequencing_invalid",
                        &format!(
                            "masterplan_v2.sequencing.work_item_order[{expected_index}].index={index}"
                        ),
                    ));
                }
                seen_indexes.insert(index as usize);
            }
            None => {
                findings.insert(Finding::new(
                    "masterplan_sequencing_invalid",
                    &format!("masterplan_v2.sequencing.work_item_order[{expected_index}].index"),
                ));
            }
        }

        let Some(work_item_id) = entry.get("work_item_id").and_then(Value::as_str) else {
            findings.insert(Finding::new(
                "masterplan_sequencing_invalid",
                &format!("masterplan_v2.sequencing.work_item_order[{expected_index}].work_item_id"),
            ));
            continue;
        };
        if !work_item_ids.contains(work_item_id) {
            findings.insert(Finding::new(
                "masterplan_sequencing_invalid",
                &format!("{work_item_id}@masterplan_v2.sequencing.work_item_order"),
            ));
        }
        if positions
            .insert(work_item_id.to_owned(), expected_index)
            .is_some()
        {
            findings.insert(Finding::new(
                "masterplan_sequencing_invalid",
                &format!("{work_item_id}@masterplan_v2.sequencing.work_item_order.duplicate"),
            ));
        }
    }

    for expected_index in 0..order.len() {
        if !seen_indexes.contains(&expected_index) {
            findings.insert(Finding::new(
                "masterplan_sequencing_invalid",
                &format!("missing-index-{expected_index}@masterplan_v2.sequencing.work_item_order"),
            ));
        }
    }
    for id in work_item_ids {
        if !positions.contains_key(id) {
            findings.insert(Finding::new(
                "masterplan_sequencing_invalid",
                &format!("{id}@masterplan_v2.sequencing.work_item_order.missing"),
            ));
        }
    }

    positions
}

fn evaluate_zero_based_execution_waves(
    execution_waves: Option<&Value>,
    work_item_ids: &BTreeSet<String>,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, usize> {
    let mut wave_positions = BTreeMap::new();
    let Some(waves) = execution_waves.and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "masterplan_sequencing_invalid",
            "masterplan_v2.sequencing.execution_waves",
        ));
        return wave_positions;
    };
    if waves.is_empty() && !work_item_ids.is_empty() {
        findings.insert(Finding::new(
            "masterplan_sequencing_invalid",
            "masterplan_v2.sequencing.execution_waves.empty",
        ));
    }

    let mut seen_wave_indexes = BTreeSet::new();
    for (expected_wave_index, wave) in waves.iter().enumerate() {
        let Some(wave) = wave.as_object() else {
            findings.insert(Finding::new(
                "masterplan_sequencing_invalid",
                &format!("masterplan_v2.sequencing.execution_waves[{expected_wave_index}]"),
            ));
            continue;
        };

        match wave.get("wave_index").and_then(Value::as_u64) {
            Some(wave_index) => {
                if wave_index != expected_wave_index as u64 {
                    findings.insert(Finding::new(
                        "masterplan_sequencing_invalid",
                        &format!(
                            "masterplan_v2.sequencing.execution_waves[{expected_wave_index}].wave_index={wave_index}"
                        ),
                    ));
                }
                seen_wave_indexes.insert(wave_index as usize);
            }
            None => {
                findings.insert(Finding::new(
                    "masterplan_sequencing_invalid",
                    &format!(
                        "masterplan_v2.sequencing.execution_waves[{expected_wave_index}].wave_index"
                    ),
                ));
            }
        }

        let Some(ids) = wave.get("work_item_ids").and_then(Value::as_array) else {
            findings.insert(Finding::new(
                "masterplan_sequencing_invalid",
                &format!(
                    "masterplan_v2.sequencing.execution_waves[{expected_wave_index}].work_item_ids"
                ),
            ));
            continue;
        };
        if ids.is_empty() {
            findings.insert(Finding::new(
                "masterplan_sequencing_invalid",
                &format!("masterplan_v2.sequencing.execution_waves[{expected_wave_index}].empty"),
            ));
        }
        for value in ids {
            let Some(id) = value.as_str() else {
                findings.insert(Finding::new(
                    "masterplan_sequencing_invalid",
                    &format!(
                        "masterplan_v2.sequencing.execution_waves[{expected_wave_index}].non_string_id"
                    ),
                ));
                continue;
            };
            if !work_item_ids.contains(id) {
                findings.insert(Finding::new(
                    "masterplan_sequencing_invalid",
                    &format!("{id}@masterplan_v2.sequencing.execution_waves"),
                ));
            }
            if wave_positions
                .insert(id.to_owned(), expected_wave_index)
                .is_some()
            {
                findings.insert(Finding::new(
                    "masterplan_sequencing_invalid",
                    &format!("{id}@masterplan_v2.sequencing.execution_waves.duplicate"),
                ));
            }
        }
    }

    for expected_wave_index in 0..waves.len() {
        if !seen_wave_indexes.contains(&expected_wave_index) {
            findings.insert(Finding::new(
                "masterplan_sequencing_invalid",
                &format!(
                    "missing-wave-{expected_wave_index}@masterplan_v2.sequencing.execution_waves"
                ),
            ));
        }
    }
    for id in work_item_ids {
        if !wave_positions.contains_key(id) {
            findings.insert(Finding::new(
                "masterplan_sequencing_invalid",
                &format!("{id}@masterplan_v2.sequencing.execution_waves.missing"),
            ));
        }
    }

    wave_positions
}

fn evaluate_sequence_respects_dependencies(
    dependency_edges: Option<&Value>,
    order_positions: &BTreeMap<String, usize>,
    wave_positions: &BTreeMap<String, usize>,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(edges) = dependency_edges.and_then(Value::as_array) else {
        return;
    };

    for (index, edge) in edges.iter().enumerate() {
        let from = edge.get("from").and_then(Value::as_str);
        let to = edge.get("to").and_then(Value::as_str);
        let (Some(from), Some(to)) = (from, to) else {
            continue;
        };
        if let (Some(from_pos), Some(to_pos)) = (order_positions.get(from), order_positions.get(to))
            && from_pos >= to_pos
        {
            findings.insert(Finding::new(
                "masterplan_sequencing_invalid",
                &format!("{from}->{to}@masterplan_v2.sequencing.work_item_order[{index}]"),
            ));
        }
        if let (Some(from_wave), Some(to_wave)) = (wave_positions.get(from), wave_positions.get(to))
            && from_wave >= to_wave
        {
            findings.insert(Finding::new(
                "masterplan_sequencing_invalid",
                &format!("{from}->{to}@masterplan_v2.sequencing.execution_waves[{index}]"),
            ));
        }
    }
}

fn founder_ratification_recorded(founder_ratification: Option<&Value>) -> bool {
    let Some(ratification) = founder_ratification else {
        return false;
    };
    ratification
        .get("decision_recorded")
        .and_then(Value::as_bool)
        == Some(true)
        && any_non_empty_field(
            ratification,
            &["decision_ref", "record_ref", "evidence_ref"],
        )
        .is_some()
        && any_non_empty_field(ratification, &["recorded_at", "decided_at", "approved_at"])
            .is_some()
        && non_empty_field(ratification, "approved_by").is_some_and(|approved_by| {
            approved_by.eq_ignore_ascii_case("founder")
                || approved_by.to_ascii_lowercase().contains("founder")
        })
        && any_non_empty_field(ratification, &["decision_status", "status"]).is_some_and(|status| {
            status.eq_ignore_ascii_case("ratified")
                || status.eq_ignore_ascii_case("approved")
                || status.eq_ignore_ascii_case("accepted")
        })
}

fn evaluate_execution_wave_dispatch(
    execution_wave_dispatch: Option<&Value>,
    founder_ratified: bool,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(dispatch) = execution_wave_dispatch else {
        findings.insert(Finding::new(
            "masterplan_execution_wave_dispatch_unratified",
            "masterplan_v2.sequencing.execution_wave_dispatch",
        ));
        return;
    };

    if dispatch
        .get("requires_founder_ratification")
        .and_then(Value::as_bool)
        != Some(true)
    {
        findings.insert(Finding::new(
            "masterplan_execution_wave_dispatch_unratified",
            "masterplan_v2.sequencing.execution_wave_dispatch.requires_founder_ratification",
        ));
    }
    if dispatch
        .get("allowed_without_founder_ratification")
        .and_then(Value::as_bool)
        != Some(false)
    {
        findings.insert(Finding::new(
            "masterplan_execution_wave_dispatch_unratified",
            "masterplan_v2.sequencing.execution_wave_dispatch.allowed_without_founder_ratification",
        ));
    }
    if !founder_ratified {
        let blocked = dispatch.get("state").and_then(Value::as_str) == Some(DISPATCH_BLOCKED_STATE)
            && dispatch.get("blocked_reason").and_then(Value::as_str)
                == Some(DISPATCH_BLOCKED_REASON);
        if !blocked {
            findings.insert(Finding::new(
                "masterplan_execution_wave_dispatch_unratified",
                "masterplan_v2.sequencing.execution_wave_dispatch.not_blocked",
            ));
        }
    }
}

fn any_non_empty_field<'a>(value: &'a Value, fields: &[&str]) -> Option<&'a str> {
    fields
        .iter()
        .find_map(|field| non_empty_field(value, field))
}

fn evaluate_work_item_completion_evidence(
    work_items: Option<&Value>,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(work_items) = work_items.and_then(Value::as_array) else {
        return;
    };

    for (index, item) in work_items.iter().enumerate() {
        let Some(status) = non_empty_field(item, "status") else {
            continue;
        };
        if !is_verified_completion_status(status) || completion_evidence_attached(item) {
            continue;
        }

        let key = non_empty_field(item, "id")
            .map(str::to_owned)
            .unwrap_or_else(|| format!("work_items[{index}]"));
        findings.insert(Finding::new(
            "masterplan_evidence_state_invalid",
            &format!("{key}@work_items[{index}].evidence_refs"),
        ));
    }
}

fn evaluate_hermes_done_card_imports(claims: Option<&Value>, findings: &mut BTreeSet<Finding>) {
    let Some(claims) = claims else {
        return;
    };
    let Some(claims) = claims.as_array() else {
        findings.insert(Finding::new(
            "masterplan_evidence_state_invalid",
            "masterplan_v2.hermes_done_card_imports",
        ));
        return;
    };

    for (index, claim) in claims.iter().enumerate() {
        if !is_hermes_done_claim(claim) || completion_evidence_attached(claim) {
            continue;
        }

        let key = hermes_done_claim_key(claim, index);
        if !any_non_empty_field(claim, &["evidence_state"])
            .is_some_and(is_claimed_done_unverified_state)
        {
            findings.insert(Finding::new(
                "masterplan_evidence_state_invalid",
                &format!("{key}@hermes_done_card_imports[{index}].evidence_state"),
            ));
        }

        if !any_non_empty_field(claim, &["masterplan_status", "status"])
            .is_some_and(is_claimed_done_unverified_state)
        {
            findings.insert(Finding::new(
                "masterplan_evidence_state_invalid",
                &format!("{key}@hermes_done_card_imports[{index}].masterplan_status"),
            ));
        }
    }
}
fn evaluate_plan_evidence_policy(policy: Option<&Value>, findings: &mut BTreeSet<Finding>) {
    let Some(policy) = policy else {
        findings.insert(Finding::new(
            "masterplan_plan_evidence_drift",
            "masterplan_v2.evidence_state_policy",
        ));
        return;
    };
    if !policy.is_object() {
        findings.insert(Finding::new(
            "masterplan_plan_evidence_drift",
            "masterplan_v2.evidence_state_policy",
        ));
        return;
    }

    if policy
        .get("status_claims_require_evidence_refs")
        .and_then(Value::as_bool)
        != Some(true)
    {
        findings.insert(Finding::new(
            "masterplan_plan_evidence_drift",
            "masterplan_v2.evidence_state_policy.status_claims_require_evidence_refs",
        ));
    }
    if !policy
        .get("hermes_done_claim_without_evidence_state")
        .and_then(Value::as_str)
        .is_some_and(is_claimed_done_unverified_state)
    {
        findings.insert(Finding::new(
            "masterplan_plan_evidence_drift",
            "masterplan_v2.evidence_state_policy.hermes_done_claim_without_evidence_state",
        ));
    }
    if !policy
        .get("evidence_attached_completion_state")
        .and_then(Value::as_str)
        .is_some_and(is_evidence_attached_state)
    {
        findings.insert(Finding::new(
            "masterplan_plan_evidence_drift",
            "masterplan_v2.evidence_state_policy.evidence_attached_completion_state",
        ));
    }
    if policy.get("validator").and_then(Value::as_str) != Some(PLAN_EVIDENCE_DRIFT_VALIDATOR) {
        findings.insert(Finding::new(
            "masterplan_plan_evidence_drift",
            "masterplan_v2.evidence_state_policy.validator",
        ));
    }
}

fn evaluate_work_item_plan_evidence_drift(
    work_items: Option<&Value>,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(work_items) = work_items else {
        findings.insert(Finding::new(
            "masterplan_plan_evidence_drift",
            "masterplan_v2.work_items",
        ));
        return;
    };
    let Some(work_items) = work_items.as_array() else {
        findings.insert(Finding::new(
            "masterplan_plan_evidence_drift",
            "masterplan_v2.work_items",
        ));
        return;
    };

    for (index, item) in work_items.iter().enumerate() {
        let key = non_empty_field(item, "id")
            .map(str::to_owned)
            .unwrap_or_else(|| format!("work_items[{index}]"));
        let scoped_key = format!("{key}@work_items[{index}]");
        evaluate_plan_evidence_refs(item, &scoped_key, findings);

        if completion_evidence_attached(item)
            && any_non_empty_field(item, &["evidence_state"])
                .is_some_and(is_claimed_done_unverified_state)
        {
            findings.insert(Finding::new(
                "masterplan_plan_evidence_drift",
                &format!("{scoped_key}.evidence_state"),
            ));
        }
    }
}

fn evaluate_hermes_plan_evidence_drift(claims: Option<&Value>, findings: &mut BTreeSet<Finding>) {
    let Some(claims) = claims else {
        return;
    };
    let Some(claims) = claims.as_array() else {
        findings.insert(Finding::new(
            "masterplan_plan_evidence_drift",
            "masterplan_v2.hermes_done_card_imports",
        ));
        return;
    };

    for (index, claim) in claims.iter().enumerate() {
        let key = hermes_done_claim_key(claim, index);
        let scoped_key = format!("{key}@hermes_done_card_imports[{index}]");
        evaluate_plan_evidence_refs(claim, &scoped_key, findings);

        if !is_hermes_done_claim(claim) {
            continue;
        }

        let evidence_attached = completion_evidence_attached(claim);
        let evidence_state = any_non_empty_field(claim, &["evidence_state"]);
        if evidence_attached && !evidence_state.is_some_and(is_evidence_attached_state) {
            findings.insert(Finding::new(
                "masterplan_plan_evidence_drift",
                &format!("{scoped_key}.evidence_state"),
            ));
        }
        if !evidence_attached && evidence_state.is_some_and(is_evidence_attached_state) {
            findings.insert(Finding::new(
                "masterplan_plan_evidence_drift",
                &format!("{scoped_key}.evidence_refs"),
            ));
        }
    }
}

fn evaluate_plan_evidence_refs(value: &Value, scoped_key: &str, findings: &mut BTreeSet<Finding>) {
    if let Some(refs) = value.get("evidence_refs") {
        let Some(refs) = refs.as_array() else {
            findings.insert(Finding::new(
                "masterplan_plan_evidence_drift",
                &format!("{scoped_key}.evidence_refs"),
            ));
            return;
        };
        for (index, evidence_ref) in refs.iter().enumerate() {
            let Some(evidence_ref) = evidence_ref
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            else {
                findings.insert(Finding::new(
                    "masterplan_plan_evidence_drift",
                    &format!("{scoped_key}.evidence_refs[{index}]"),
                ));
                continue;
            };
            if plan_evidence_ref_is_stale_or_local(evidence_ref) {
                findings.insert(Finding::new(
                    "masterplan_plan_evidence_drift",
                    &format!("{scoped_key}.evidence_refs[{index}]"),
                ));
            }
        }
    }

    for field in [
        "evidence_ref",
        "evidence_path",
        "merged_pr_ref",
        "gate_run_ref",
        "review_ref",
    ] {
        let Some(raw) = value.get(field) else {
            continue;
        };
        let Some(evidence_ref) = raw.as_str().map(str::trim).filter(|s| !s.is_empty()) else {
            findings.insert(Finding::new(
                "masterplan_plan_evidence_drift",
                &format!("{scoped_key}.{field}"),
            ));
            continue;
        };
        if plan_evidence_ref_is_stale_or_local(evidence_ref) {
            findings.insert(Finding::new(
                "masterplan_plan_evidence_drift",
                &format!("{scoped_key}.{field}"),
            ));
        }
    }
}

fn plan_evidence_ref_is_stale_or_local(evidence_ref: &str) -> bool {
    let mut normalized = evidence_ref.trim();
    while let Some(stripped) = normalized.strip_prefix('/') {
        normalized = stripped;
    }
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped;
    }
    let path = normalized
        .split_once('#')
        .map_or(normalized, |(path, _)| path);
    let lower = path.to_ascii_lowercase();

    [
        ".omc/",
        ".omx/",
        ".gjc/",
        "~/.hermes/",
        "~/.gjc/",
        "~/.omx/",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
        || matches!(
            lower.as_str(),
            "specs/master-plan-sequencing.json"
                | "specs/planning-closure-contract.json"
                | "specs/planning-closure-status-closure-ledger.json"
                | "docs/masterplan.md"
                | "docs/roadmap.md"
        )
}

fn is_hermes_done_claim(claim: &Value) -> bool {
    let source_is_hermes = any_non_empty_field(
        claim,
        &[
            "source_system",
            "source",
            "source_path",
            "source_ref",
            "source_board",
        ],
    )
    .is_some_and(|source| normalized_status_token(source).contains("hermes"));

    let source_claims_done = any_non_empty_field(
        claim,
        &[
            "source_status",
            "source_column",
            "source_state",
            "completion_claim",
        ],
    )
    .is_some_and(is_done_claim_status);

    source_is_hermes && source_claims_done
}

fn completion_evidence_attached(value: &Value) -> bool {
    non_empty_string_array(value.get("evidence_refs"))
        || any_non_empty_field(
            value,
            &[
                "evidence_ref",
                "evidence_path",
                "merged_pr_ref",
                "gate_run_ref",
                "review_ref",
            ],
        )
        .is_some()
}

fn hermes_done_claim_key(claim: &Value, index: usize) -> String {
    any_non_empty_field(
        claim,
        &[
            "source_card_id",
            "card_id",
            "task_id",
            "source_task_id",
            "source_ref",
        ],
    )
    .map(str::to_owned)
    .unwrap_or_else(|| format!("hermes_done_card_imports[{index}]"))
}

fn is_verified_completion_status(status: &str) -> bool {
    matches!(
        normalized_status_token(status).as_str(),
        "done" | "complete" | "completed" | "verifieddone" | "evidenceattacheddone"
    )
}

fn is_done_claim_status(status: &str) -> bool {
    let normalized = normalized_status_token(status);
    matches!(normalized.as_str(), "done" | "complete" | "completed")
        || normalized.contains("donecard")
}

fn is_claimed_done_unverified_state(status: &str) -> bool {
    normalized_status_token(status) == normalized_status_token(CLAIMED_DONE_UNVERIFIED_STATE)
}
fn is_evidence_attached_state(status: &str) -> bool {
    normalized_status_token(status) == normalized_status_token(EVIDENCE_ATTACHED_STATE)
}

fn normalized_status_token(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_lowercase())
        .collect()
}

fn evaluate_program_shards(
    programs: Option<&Value>,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, String> {
    let mut program_ids = BTreeMap::new();
    let Some(programs) = programs.and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "masterplan_program_coverage_incomplete",
            "<missing-programs>",
        ));
        return program_ids;
    };
    if programs.is_empty() {
        findings.insert(Finding::new(
            "masterplan_program_coverage_incomplete",
            "<empty-programs>",
        ));
        return program_ids;
    }

    for (index, program) in programs.iter().enumerate() {
        let key = non_empty_field(program, "id")
            .map(str::to_owned)
            .unwrap_or_else(|| format!("<programs[{index}]>"));
        if !program.is_object() {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("{key}@malformed"),
            ));
            continue;
        }

        for field in ["id", "program_class", "scope", "owner"] {
            if non_empty_field(program, field).is_none() {
                findings.insert(Finding::new(
                    "masterplan_program_coverage_incomplete",
                    &format!("{key}.{field}"),
                ));
            }
        }
        for field in [
            "hyperscaler_bar_exit_criteria",
            "child_work_item_classes",
            "evidence_requirements",
        ] {
            if !non_empty_string_array(program.get(field)) {
                findings.insert(Finding::new(
                    "masterplan_program_coverage_incomplete",
                    &format!("{key}.{field}"),
                ));
            }
        }

        let Some(id) = non_empty_field(program, "id") else {
            continue;
        };
        let Some(program_class) = non_empty_field(program, "program_class") else {
            continue;
        };
        if program_ids
            .insert(id.to_owned(), program_class.to_owned())
            .is_some()
        {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("program:{id}@duplicate"),
            ));
        }
        if program_class == "owned-stack" && non_empty_field(program, "owned_stack_layer").is_none()
        {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("program:{id}.owned_stack_layer"),
            ));
        }
    }

    program_ids
}

/// Dangling-reference guard for the work-item → program-shard edge: every
/// masterplan v2 work item must name a DECLARED program shard, so no work
/// item can float outside the program-sharded coverage proof. Missing or
/// malformed `work_items` is already flagged by the authority evaluator;
/// this guard only polices the reference edge itself.
fn evaluate_work_item_program_membership(
    work_items: Option<&Value>,
    program_ids: &BTreeMap<String, String>,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(work_items) = work_items.and_then(Value::as_array) else {
        return;
    };
    for item in work_items {
        let id = item
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("<missing-id>");
        match item.get("program").and_then(Value::as_str) {
            Some(program) if program_ids.contains_key(program) => {}
            Some(program) => {
                findings.insert(Finding::new(
                    "masterplan_program_coverage_incomplete",
                    &format!("work_item:{id}@unknown-program:{program}"),
                ));
            }
            None => {
                findings.insert(Finding::new(
                    "masterplan_program_coverage_incomplete",
                    &format!("work_item:{id}@missing-program"),
                ));
            }
        }
    }
}

fn evaluate_required_program_classes(
    programs: Option<&Value>,
    program_coverage: Option<&Value>,
    findings: &mut BTreeSet<Finding>,
) {
    let actual_classes: BTreeSet<String> = programs
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|program| non_empty_field(program, "program_class"))
        .map(str::to_owned)
        .collect();

    let actual_owned_stack_layers: BTreeSet<String> = programs
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|program| non_empty_field(program, "program_class") == Some("owned-stack"))
        .filter_map(|program| non_empty_field(program, "owned_stack_layer"))
        .map(str::to_owned)
        .collect();

    let required_classes = const_set(&REQUIRED_PROGRAM_CLASSES);
    let declared_required_classes = required_string_set(
        program_coverage,
        "required_program_classes",
        findings,
        "required_program_classes",
    );
    if declared_required_classes != required_classes {
        findings.insert(Finding::new(
            "masterplan_program_coverage_incomplete",
            "required_program_classes",
        ));
    }
    for program_class in required_classes {
        if !actual_classes.contains(&program_class) {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("program_class:{program_class}"),
            ));
        }
    }

    let required_owned_stack_layers = const_set(&REQUIRED_OWNED_STACK_LAYERS);
    let declared_required_owned_stack_layers = required_string_set(
        program_coverage,
        "required_owned_stack_layers",
        findings,
        "required_owned_stack_layers",
    );
    if declared_required_owned_stack_layers != required_owned_stack_layers {
        findings.insert(Finding::new(
            "masterplan_program_coverage_incomplete",
            "required_owned_stack_layers",
        ));
    }
    for layer in required_owned_stack_layers {
        if !actual_owned_stack_layers.contains(&layer) {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("owned_stack_layer:{layer}"),
            ));
        }
    }
}

fn evaluate_microservice_manifest_coverage(
    program_coverage: Option<&Value>,
    manifest_index: &Value,
    program_ids: &BTreeMap<String, String>,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(program_coverage) = program_coverage else {
        findings.insert(Finding::new(
            "masterplan_program_coverage_incomplete",
            "<missing-program-coverage>",
        ));
        return;
    };

    if program_coverage
        .get("manifest_index_ref")
        .and_then(Value::as_str)
        != Some(MANIFEST_INDEX_REF)
    {
        findings.insert(Finding::new(
            "masterplan_program_coverage_incomplete",
            "<manifest-index-ref>",
        ));
    }

    let indexed = indexed_manifest_microservices(manifest_index, findings);
    let Some(entries) = program_coverage
        .get("microservices")
        .and_then(Value::as_array)
    else {
        findings.insert(Finding::new(
            "masterplan_program_coverage_incomplete",
            "<missing-microservice-coverage>",
        ));
        return;
    };

    let mut coverage_by_name: BTreeMap<String, (String, String)> = BTreeMap::new();
    for (index, entry) in entries.iter().enumerate() {
        let key = non_empty_field(entry, "microservice")
            .map(str::to_owned)
            .unwrap_or_else(|| format!("<program_coverage.microservices[{index}]>"));
        if !entry.is_object() {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("{key}@malformed"),
            ));
            continue;
        }

        let microservice = non_empty_field(entry, "microservice");
        let program_id = non_empty_field(entry, "program_id");
        let source_manifest = non_empty_field(entry, "source_manifest");
        let coverage_status = non_empty_field(entry, "coverage_status");

        if microservice.is_none() {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("{key}.microservice"),
            ));
        }
        if let Some(program_id) = program_id {
            if !program_ids.contains_key(program_id) {
                findings.insert(Finding::new(
                    "masterplan_program_coverage_incomplete",
                    &format!("{key}@unknown-program:{program_id}"),
                ));
            }
        } else {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("{key}.program_id"),
            ));
        }
        if source_manifest.is_none() {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("{key}.source_manifest"),
            ));
        }
        if !matches!(coverage_status, Some("covered" | "retired-absorbed")) {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("{key}.coverage_status"),
            ));
        }

        let Some(microservice) = microservice else {
            continue;
        };
        if !indexed.contains_key(microservice) {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("microservice:{microservice}@not-in-index"),
            ));
        }
        let Some(source_manifest) = source_manifest else {
            continue;
        };
        let Some(coverage_status) = coverage_status else {
            continue;
        };
        if coverage_by_name
            .insert(
                microservice.to_owned(),
                (source_manifest.to_owned(), coverage_status.to_owned()),
            )
            .is_some()
        {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("microservice:{microservice}@duplicate"),
            ));
        }
    }

    for (microservice, (manifest, retired)) in indexed {
        let Some((covered_manifest, coverage_status)) = coverage_by_name.get(&microservice) else {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("microservice:{microservice}"),
            ));
            continue;
        };
        if covered_manifest != &manifest {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("microservice:{microservice}@source_manifest"),
            ));
        }
        let expected_status = if retired {
            "retired-absorbed"
        } else {
            "covered"
        };
        if coverage_status != expected_status {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("microservice:{microservice}@coverage_status"),
            ));
        }
    }
}

fn evaluate_owned_stack_ladder(
    program_coverage: Option<&Value>,
    program_ids: &BTreeMap<String, String>,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(ladder) = program_coverage.and_then(|value| value.get("owned_stack_ladder")) else {
        findings.insert(Finding::new(
            "masterplan_program_coverage_incomplete",
            "<missing-owned-stack-ladder>",
        ));
        return;
    };

    if !non_empty_string_array(ladder.get("doctrine_refs")) {
        findings.insert(Finding::new(
            "masterplan_program_coverage_incomplete",
            "owned_stack_ladder.doctrine_refs",
        ));
    }

    let Some(rungs) = ladder.get("rungs").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "masterplan_program_coverage_incomplete",
            "<missing-owned-stack-ladder-rungs>",
        ));
        return;
    };
    if rungs.len() != REQUIRED_OWNED_STACK_LADDER_RUNGS.len() {
        findings.insert(Finding::new(
            "masterplan_program_coverage_incomplete",
            "owned_stack_ladder.rungs@count",
        ));
    }

    let mut seen_layers: BTreeSet<String> = BTreeSet::new();
    for (index, rung) in rungs.iter().enumerate() {
        let Some(layer) = non_empty_field(rung, "layer") else {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("owned_stack_ladder.rungs[{index}].layer"),
            ));
            continue;
        };
        let key = format!("owned_stack_ladder:{layer}");
        if !seen_layers.insert(layer.to_owned()) {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("{key}@duplicate"),
            ));
        }
        if REQUIRED_OWNED_STACK_LADDER_RUNGS.get(index).copied() != Some(layer) {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("{key}@ladder-order"),
            ));
        }
        if rung.get("rung").and_then(Value::as_u64) != Some(index as u64) {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("{key}@rung-index"),
            ));
        }
        if !non_empty_string_array(rung.get("source_anchors")) {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("{key}.source_anchors"),
            ));
        }
        let Some(covering) = rung.get("program_ids").and_then(Value::as_array) else {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("{key}.program_ids"),
            ));
            continue;
        };
        if covering.is_empty() {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("{key}.program_ids"),
            ));
        }
        for program_id in covering {
            let Some(program_id) = program_id
                .as_str()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            else {
                findings.insert(Finding::new(
                    "masterplan_program_coverage_incomplete",
                    &format!("{key}@malformed-program-id"),
                ));
                continue;
            };
            if !program_ids.contains_key(program_id) {
                findings.insert(Finding::new(
                    "masterplan_program_coverage_incomplete",
                    &format!("{key}@unknown-program:{program_id}"),
                ));
            }
        }
    }

    for layer in REQUIRED_OWNED_STACK_LADDER_RUNGS {
        if !seen_layers.contains(layer) {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("owned_stack_ladder:{layer}"),
            ));
        }
    }
}
fn indexed_manifest_microservices(
    manifest_index: &Value,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, (String, bool)> {
    let mut indexed = BTreeMap::new();
    let Some(microservices) = manifest_index
        .get("microservices")
        .and_then(Value::as_array)
    else {
        findings.insert(Finding::new(
            "masterplan_program_coverage_incomplete",
            "<manifest-index.microservices>",
        ));
        return indexed;
    };

    for (index, entry) in microservices.iter().enumerate() {
        let Some(name) = non_empty_field(entry, "name") else {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("<manifest-index.microservices[{index}].name>"),
            ));
            continue;
        };
        let Some(manifest) = non_empty_field(entry, "manifest") else {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("microservice:{name}@manifest"),
            ));
            continue;
        };
        let retired = entry.get("status").and_then(Value::as_str) == Some("retired")
            || entry.get("do_not_treat_as_active").and_then(Value::as_bool) == Some(true);
        if indexed
            .insert(name.to_owned(), (manifest.to_owned(), retired))
            .is_some()
        {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("microservice:{name}@manifest-index-duplicate"),
            ));
        }
    }

    indexed
}

fn required_string_set(
    container: Option<&Value>,
    field: &str,
    findings: &mut BTreeSet<Finding>,
    key: &str,
) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    let Some(array) = container
        .and_then(|value| value.get(field))
        .and_then(Value::as_array)
    else {
        findings.insert(Finding::new(
            "masterplan_program_coverage_incomplete",
            &format!("<missing-{key}>"),
        ));
        return values;
    };
    if array.is_empty() {
        findings.insert(Finding::new(
            "masterplan_program_coverage_incomplete",
            &format!("<empty-{key}>"),
        ));
        return values;
    }
    for (index, value) in array.iter().enumerate() {
        if let Some(value) = value
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            values.insert(value.to_owned());
        } else {
            findings.insert(Finding::new(
                "masterplan_program_coverage_incomplete",
                &format!("<malformed-{key}-{index}>"),
            ));
        }
    }

    values
}

fn non_empty_field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn non_empty_string_array(value: Option<&Value>) -> bool {
    value.and_then(Value::as_array).is_some_and(|items| {
        !items.is_empty()
            && items
                .iter()
                .all(|item| item.as_str().is_some_and(|value| !value.trim().is_empty()))
    })
}

fn const_set(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn evaluate_surface_dispositions(surfaces: Option<&Value>, findings: &mut BTreeSet<Finding>) {
    let Some(surfaces) = surfaces.and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "masterplan_not_sole_live_authority",
            "<missing-surface-dispositions>",
        ));
        findings.insert(Finding::new(
            "masterplan_surface_disposition_incomplete",
            "<missing-surface-dispositions>",
        ));
        return;
    };

    let allowed_dispositions = const_set(&ALLOWED_SURFACE_DISPOSITIONS);
    let mut by_path: BTreeMap<String, &Value> = BTreeMap::new();
    let mut live_authorities = BTreeSet::new();
    for (index, surface) in surfaces.iter().enumerate() {
        let path = non_empty_field(surface, "path").unwrap_or("<missing-path>");
        if by_path.insert(path.to_owned(), surface).is_some() {
            findings.insert(Finding::new(
                "masterplan_surface_disposition_incomplete",
                &format!("{path}.duplicate"),
            ));
        }

        let disposition = non_empty_field(surface, "disposition");
        if !disposition.is_some_and(|value| allowed_dispositions.contains(value)) {
            findings.insert(Finding::new(
                "masterplan_surface_disposition_incomplete",
                &format!("{path}.disposition"),
            ));
        }
        if non_empty_field(surface, "provenance").is_none() {
            findings.insert(Finding::new(
                "masterplan_surface_disposition_incomplete",
                &format!("{path}.provenance"),
            ));
        }
        if non_empty_field(surface, "work_item_id_policy").is_none() {
            findings.insert(Finding::new(
                "masterplan_surface_disposition_incomplete",
                &format!("{path}.work_item_id_policy"),
            ));
        }

        if surface
            .get("is_live_plan_authority")
            .and_then(Value::as_bool)
            == Some(true)
        {
            live_authorities.insert(path.to_owned());
        }

        if path == "<missing-path>" {
            findings.insert(Finding::new(
                "masterplan_surface_disposition_incomplete",
                &format!("surface_dispositions[{index}].path"),
            ));
        }
    }

    if live_authorities != BTreeSet::from(["/specs/masterplan.json".to_owned()]) {
        let key = if live_authorities.is_empty() {
            "<no-live-plan-authority>".to_owned()
        } else {
            live_authorities.into_iter().collect::<Vec<_>>().join(",")
        };
        findings.insert(Finding::new("masterplan_not_sole_live_authority", &key));
    }

    for (path, expected_disposition) in REQUIRED_SURFACE_DISPOSITIONS {
        let Some(surface) = by_path.get(path) else {
            findings.insert(Finding::new(
                "masterplan_surface_disposition_incomplete",
                path,
            ));
            continue;
        };

        if non_empty_field(surface, "disposition") != Some(expected_disposition) {
            findings.insert(Finding::new(
                "masterplan_surface_disposition_incomplete",
                &format!("{path}.disposition"),
            ));
        }

        let expected_live = expected_disposition == DISPOSITION_CANONICAL_AUTHORITY;
        if surface
            .get("is_live_plan_authority")
            .and_then(Value::as_bool)
            != Some(expected_live)
        {
            findings.insert(Finding::new(
                "masterplan_surface_disposition_incomplete",
                &format!("{path}.is_live_plan_authority"),
            ));
        }

        if !expected_live && non_empty_field(surface, "absorbed_into") != Some(MASTERPLAN_V2_REF) {
            findings.insert(Finding::new(
                "masterplan_surface_disposition_incomplete",
                &format!("{path}.absorbed_into"),
            ));
        }
    }
}

fn evaluate_masterplan_work_items(
    work_items: Option<&Value>,
    id_prefix: &str,
    numeric_width: usize,
    findings: &mut BTreeSet<Finding>,
) -> BTreeSet<String> {
    let mut known_ids = BTreeSet::new();
    let Some(work_items) = work_items.and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "masterplan_external_live_work_item_id",
            "<missing-work-items>",
        ));
        return known_ids;
    };
    if work_items.is_empty() {
        findings.insert(Finding::new(
            "masterplan_external_live_work_item_id",
            "<empty-work-items>",
        ));
        return known_ids;
    }

    let mut seen = BTreeSet::new();
    for (index, item) in work_items.iter().enumerate() {
        let key = item
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("<missing-id>");
        if item.get("id").and_then(Value::as_str).is_some() {
            known_ids.insert(key.to_owned());
        }
        if !seen.insert(key.to_owned()) {
            findings.insert(Finding::new("masterplan_work_item_id_collision", key));
        }
        if !valid_masterplan_work_item_id(key, id_prefix, numeric_width) {
            findings.insert(Finding::new("masterplan_external_live_work_item_id", key));
        }
        if item.get("external_live_id").is_some()
            || item
                .get("external_live_ids")
                .and_then(Value::as_array)
                .is_some_and(|ids| !ids.is_empty())
            || item.get("legacy_id_is_live").and_then(Value::as_bool) == Some(true)
        {
            findings.insert(Finding::new(
                "masterplan_external_live_work_item_id",
                &format!("{key}@work_items[{index}]"),
            ));
        }
    }
    known_ids
}

fn evaluate_masterplan_dependency_dag(
    dependency_edges: Option<&Value>,
    dependency_edge_semantics: Option<&Value>,
    cross_program_edge_policy: Option<&Value>,
    work_item_ids: &BTreeSet<String>,
    work_item_programs: &BTreeMap<String, String>,
    program_ids: &BTreeSet<String>,
    findings: &mut BTreeSet<Finding>,
) {
    if dependency_edge_semantics.and_then(Value::as_str) != Some(DEPENDENCY_EDGE_SEMANTICS) {
        findings.insert(Finding::new(
            "masterplan_dependency_dag_invalid",
            "<dependency-edge-semantics>",
        ));
    }

    let Some(dependency_edges) = dependency_edges else {
        findings.insert(Finding::new(
            "masterplan_dependency_dag_invalid",
            "<missing-dependency-edges>",
        ));
        return;
    };
    let Some(dependency_edges) = dependency_edges.as_array() else {
        findings.insert(Finding::new(
            "masterplan_dependency_dag_invalid",
            "<malformed-dependency-edges>",
        ));
        return;
    };

    let mut adjacency: BTreeMap<String, BTreeSet<String>> = work_item_ids
        .iter()
        .map(|id| (id.clone(), BTreeSet::new()))
        .collect();
    // (from-item, to-item, from-program, to-program) for every well-formed edge
    // whose endpoints live in DIFFERENT program shards: the program-boundary
    // crossings the cross-program lane must see declared.
    let mut cross_program_crossings: Vec<(String, String, String, String)> = Vec::new();

    for (index, edge) in dependency_edges.iter().enumerate() {
        let Some(edge) = edge.as_object() else {
            findings.insert(Finding::new(
                "masterplan_dependency_dag_invalid",
                &format!("<malformed-dependency-edge-{index}>"),
            ));
            continue;
        };

        let from = edge.get("from").and_then(Value::as_str);
        let to = edge.get("to").and_then(Value::as_str);
        let mut well_formed_edge = true;

        if from.is_none() {
            findings.insert(Finding::new(
                "masterplan_dependency_dag_invalid",
                &format!("<malformed-dependency-edge-{index}.from>"),
            ));
            well_formed_edge = false;
        }
        if to.is_none() {
            findings.insert(Finding::new(
                "masterplan_dependency_dag_invalid",
                &format!("<malformed-dependency-edge-{index}.to>"),
            ));
            well_formed_edge = false;
        }
        if edge
            .get("relationship")
            .is_some_and(|value| !value.is_string())
        {
            findings.insert(Finding::new(
                "masterplan_dependency_dag_invalid",
                &format!("<malformed-dependency-edge-{index}.relationship>"),
            ));
            well_formed_edge = false;
        }

        let Some(from) = from else {
            continue;
        };
        let Some(to) = to else {
            continue;
        };

        if !work_item_ids.contains(from) {
            findings.insert(Finding::new(
                "masterplan_dependency_dag_invalid",
                &format!("{from}@dependency_edges[{index}].from"),
            ));
            well_formed_edge = false;
        }
        if !work_item_ids.contains(to) {
            findings.insert(Finding::new(
                "masterplan_dependency_dag_invalid",
                &format!("{to}@dependency_edges[{index}].to"),
            ));
            well_formed_edge = false;
        }
        if from == to {
            findings.insert(Finding::new(
                "masterplan_dependency_dag_invalid",
                &format!("{from}->{to}@self"),
            ));
            well_formed_edge = false;
        }

        if well_formed_edge {
            adjacency
                .entry(from.to_owned())
                .or_default()
                .insert(to.to_owned());
            if let (Some(from_program), Some(to_program)) =
                (work_item_programs.get(from), work_item_programs.get(to))
                && from_program != to_program
            {
                cross_program_crossings.push((
                    from.to_owned(),
                    to.to_owned(),
                    from_program.clone(),
                    to_program.clone(),
                ));
            }
        }
    }

    evaluate_masterplan_cross_program_edges(
        cross_program_edge_policy,
        &cross_program_crossings,
        program_ids,
        findings,
    );

    evaluate_masterplan_dependency_cycles(&adjacency, findings);
}

/// Cross-program edge lane of the dependency-DAG gate.
///
/// A work-item dependency edge whose endpoints live in different program
/// shards is a program-boundary crossing and must be backed by an explicitly
/// declared program-pair row in
/// `masterplan_v2.cross_program_edge_policy.declared_program_edges`
/// (`{from_program, to_program, rationale}` in prerequisite→dependent
/// direction, matching `dependency_edge_semantics`). Fails closed on:
///
/// - an undeclared crossing (`{from}->{to}@cross-program-undeclared(..)`);
/// - a missing policy while crossings exist;
/// - a wrong/missing `semantics` pin or a non-array `declared_program_edges`;
/// - a malformed declared row, a same-program (self) pair, a duplicate pair,
///   or a pair naming a program id absent from `masterplan_v2.programs`;
/// - a stale declared pair that NO live edge instantiates (anti-staleness:
///   the allowlist cannot outlive the graph it authorizes).
fn evaluate_masterplan_cross_program_edges(
    policy: Option<&Value>,
    cross_program_crossings: &[(String, String, String, String)],
    program_ids: &BTreeSet<String>,
    findings: &mut BTreeSet<Finding>,
) {
    // pair -> instantiated-by-a-live-edge
    let mut declared_pairs: BTreeMap<(String, String), bool> = BTreeMap::new();

    if let Some(policy) = policy {
        if policy.get("semantics").and_then(Value::as_str) != Some(CROSS_PROGRAM_EDGE_SEMANTICS) {
            findings.insert(Finding::new(
                "masterplan_dependency_dag_invalid",
                "<cross-program-edge-semantics>",
            ));
        }
        match policy
            .get("declared_program_edges")
            .and_then(Value::as_array)
        {
            None => {
                findings.insert(Finding::new(
                    "masterplan_dependency_dag_invalid",
                    "<malformed-cross-program-declared-edges>",
                ));
            }
            Some(declared) => {
                for (index, row) in declared.iter().enumerate() {
                    let from_program = row.get("from_program").and_then(Value::as_str);
                    let to_program = row.get("to_program").and_then(Value::as_str);
                    let (Some(from_program), Some(to_program)) = (from_program, to_program) else {
                        findings.insert(Finding::new(
                            "masterplan_dependency_dag_invalid",
                            &format!("<malformed-cross-program-declared-edge-{index}>"),
                        ));
                        continue;
                    };
                    if from_program == to_program {
                        findings.insert(Finding::new(
                            "masterplan_dependency_dag_invalid",
                            &format!(
                                "{from_program}->{to_program}@cross_program_edge_policy.declared_program_edges[{index}].self"
                            ),
                        ));
                        continue;
                    }
                    if !program_ids.contains(from_program) {
                        findings.insert(Finding::new(
                            "masterplan_dependency_dag_invalid",
                            &format!(
                                "{from_program}@cross_program_edge_policy.declared_program_edges[{index}].from_program"
                            ),
                        ));
                    }
                    if !program_ids.contains(to_program) {
                        findings.insert(Finding::new(
                            "masterplan_dependency_dag_invalid",
                            &format!(
                                "{to_program}@cross_program_edge_policy.declared_program_edges[{index}].to_program"
                            ),
                        ));
                    }
                    if declared_pairs
                        .insert((from_program.to_owned(), to_program.to_owned()), false)
                        .is_some()
                    {
                        findings.insert(Finding::new(
                            "masterplan_dependency_dag_invalid",
                            &format!(
                                "{from_program}->{to_program}@cross_program_edge_policy.declared_program_edges[{index}].duplicate"
                            ),
                        ));
                    }
                }
            }
        }
    } else if !cross_program_crossings.is_empty() {
        findings.insert(Finding::new(
            "masterplan_dependency_dag_invalid",
            "<missing-cross-program-edge-policy>",
        ));
    }

    for (from, to, from_program, to_program) in cross_program_crossings {
        match declared_pairs.get_mut(&(from_program.clone(), to_program.clone())) {
            Some(instantiated) => *instantiated = true,
            None => {
                findings.insert(Finding::new(
                    "masterplan_dependency_dag_invalid",
                    &format!("{from}->{to}@cross-program-undeclared({from_program}->{to_program})"),
                ));
            }
        }
    }
    for ((from_program, to_program), instantiated) in &declared_pairs {
        if !instantiated {
            findings.insert(Finding::new(
                "masterplan_dependency_dag_invalid",
                &format!(
                    "{from_program}->{to_program}@cross_program_edge_policy.declared_program_edges.unused"
                ),
            ));
        }
    }
}

/// id -> program shard for every work item declaring both. Lenient by design:
/// the work-item→program dangling-reference guard lives in the program-coverage
/// lane; this map only feeds program-boundary crossing detection.
fn masterplan_work_item_programs(work_items: Option<&Value>) -> BTreeMap<String, String> {
    let mut programs = BTreeMap::new();
    let Some(work_items) = work_items.and_then(Value::as_array) else {
        return programs;
    };
    for item in work_items {
        if let (Some(id), Some(program)) = (
            item.get("id").and_then(Value::as_str),
            item.get("program").and_then(Value::as_str),
        ) {
            programs.insert(id.to_owned(), program.to_owned());
        }
    }
    programs
}

/// The declared program-shard id set. Lenient by design: program-shard
/// structural completeness lives in the program-coverage lane; this set only
/// anchors declared cross-program pairs to real shards.
fn masterplan_program_id_set(programs: Option<&Value>) -> BTreeSet<String> {
    programs
        .and_then(Value::as_array)
        .map(|programs| {
            programs
                .iter()
                .filter_map(|program| program.get("id").and_then(Value::as_str))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn evaluate_masterplan_dependency_cycles(
    adjacency: &BTreeMap<String, BTreeSet<String>>,
    findings: &mut BTreeSet<Finding>,
) {
    let mut indegree: BTreeMap<String, usize> =
        adjacency.keys().map(|id| (id.clone(), 0_usize)).collect();
    for targets in adjacency.values() {
        for target in targets {
            if let Some(degree) = indegree.get_mut(target) {
                *degree += 1;
            }
        }
    }

    let mut ready: Vec<String> = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(id, _)| id.clone())
        .collect();
    let mut visited = 0_usize;
    while let Some(node) = ready.pop() {
        visited += 1;
        if let Some(targets) = adjacency.get(&node) {
            for target in targets {
                if let Some(degree) = indegree.get_mut(target) {
                    *degree = degree.saturating_sub(1);
                    if *degree == 0 {
                        ready.push(target.clone());
                    }
                }
            }
        }
    }

    if visited != indegree.len() {
        let cycle_nodes = indegree
            .iter()
            .filter(|(_, degree)| **degree > 0)
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>()
            .join(",");
        findings.insert(Finding::new(
            "masterplan_dependency_dag_invalid",
            &format!("cycle@{{{cycle_nodes}}}"),
        ));
    }
}

fn valid_masterplan_work_item_id(id: &str, prefix: &str, numeric_width: usize) -> bool {
    let Some(suffix) = id.strip_prefix(prefix) else {
        return false;
    };
    suffix.len() == numeric_width && suffix.chars().all(|value| value.is_ascii_digit())
}

/// Per-decision propagation + status-agreement checks. Keyed by the decision `id`
/// (plus `@face` for status_disagreement).
fn evaluate_decision(decision: &Value, findings: &mut BTreeSet<Finding>) {
    let id = decision.get("id").and_then(Value::as_str).unwrap_or("");
    let status = decision
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();

    // Propagation is only required for live (Accepted) decisions. A Superseded/Proposed
    // decision is not expected to carry masterplan/roadmap nodes.
    let is_accepted = status.eq_ignore_ascii_case("accepted");

    let in_spec = bool_field(decision, "in_spec");
    let in_masterplan = bool_field(decision, "in_masterplan");
    let in_roadmap = bool_field(decision, "in_roadmap");

    if is_accepted {
        let reaches_any = in_spec || in_masterplan || in_roadmap;
        if !reaches_any {
            // Reaches no propagation face at all: a decision nothing points at.
            findings.insert(Finding::new("orphan_decision", id));
        } else if !(in_spec && in_masterplan && in_roadmap) {
            // Reaches some faces but not all required ones.
            findings.insert(Finding::new("unpropagated_decision", id));
        }
    }

    // status_disagreement: any face records a status that differs from the ADR's status.
    if let Some(face_statuses) = decision.get("face_statuses").and_then(Value::as_object) {
        for (face, face_status) in face_statuses {
            if let Some(other) = face_status.as_str()
                && !other.trim().eq_ignore_ascii_case(status)
            {
                findings.insert(Finding::new("status_disagreement", &format!("{id}@{face}")));
            }
        }
    }
}

fn validate_payload_shape(fixture: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    let Some(decisions) = fixture.get("decisions") else {
        findings.insert(Finding::new("orphan_decision", "<missing-decisions>"));
        return findings;
    };
    let Some(decisions) = decisions.as_array() else {
        findings.insert(Finding::new("orphan_decision", "<non-array-decisions>"));
        return findings;
    };
    if decisions.is_empty() {
        findings.insert(Finding::new("orphan_decision", "<empty-decisions>"));
    }

    for (index, decision) in decisions.iter().enumerate() {
        let Some(object) = decision.as_object() else {
            findings.insert(Finding::new(
                "orphan_decision",
                &format!("<malformed-decision-{index}>"),
            ));
            continue;
        };
        if !object.get("id").is_some_and(Value::is_string) {
            findings.insert(Finding::new(
                "orphan_decision",
                &format!("<malformed-decision-{index}.id>"),
            ));
        }
        if !object.get("status").is_some_and(Value::is_string) {
            findings.insert(Finding::new(
                "orphan_decision",
                &format!("<malformed-decision-{index}.status>"),
            ));
        }
        for field in ["supersedes", "superseded_by"] {
            if decision.get(field).is_some_and(|value| !value.is_array()) {
                findings.insert(Finding::new(
                    "supersession_half_edge",
                    &format!("<malformed-decision-{index}.{field}>"),
                ));
            }
            if let Some(values) = decision.get(field).and_then(Value::as_array) {
                for (value_index, value) in values.iter().enumerate() {
                    if !value.is_string() {
                        findings.insert(Finding::new(
                            "supersession_half_edge",
                            &format!("<malformed-decision-{index}.{field}-{value_index}>"),
                        ));
                    }
                }
            }
        }
        for field in ["in_spec", "in_masterplan", "in_roadmap"] {
            if decision.get(field).is_some_and(|value| !value.is_boolean()) {
                findings.insert(Finding::new(
                    "unpropagated_decision",
                    &format!("<malformed-decision-{index}.{field}>"),
                ));
            }
        }
        if decision
            .get("face_statuses")
            .is_some_and(|value| !value.is_object())
        {
            findings.insert(Finding::new(
                "status_disagreement",
                &format!("<malformed-decision-{index}.face_statuses>"),
            ));
        }
        if let Some(face_statuses) = decision.get("face_statuses").and_then(Value::as_object) {
            for (face, value) in face_statuses {
                if !value.is_string() {
                    findings.insert(Finding::new(
                        "status_disagreement",
                        &format!("<malformed-decision-{index}.face_statuses.{face}>"),
                    ));
                }
            }
        }
    }

    validate_string_array_field(
        fixture,
        "duplicate_ids",
        "dual_decision_collision",
        "duplicate-ids",
        &mut findings,
    );
    validate_string_array_field(
        fixture,
        "id_mismatches",
        "decision_id_mismatch",
        "id-mismatches",
        &mut findings,
    );
    validate_string_array_field(
        fixture,
        "phantom_citations",
        "phantom_decision_citation",
        "phantom-citations",
        &mut findings,
    );

    if let Some(axes) = fixture.get("generated_face_axes") {
        if !axes.is_object() || axes.as_object().is_some_and(serde_json::Map::is_empty) {
            findings.insert(Finding::new(
                "generated_face_drift",
                "<malformed-generated-face-axes>",
            ));
        }
        if let Some(axes) = axes.as_object() {
            for (face, value) in axes {
                if !value.is_number() {
                    findings.insert(Finding::new(
                        "generated_face_drift",
                        &format!("<malformed-generated-face-axes.{face}>"),
                    ));
                }
            }
        }
    }

    findings
}

fn validate_string_array_field(
    fixture: &Value,
    field: &str,
    code: &str,
    sentinel: &str,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(value) = fixture.get(field) else {
        return;
    };
    let Some(values) = value.as_array() else {
        findings.insert(Finding::new(code, &format!("<malformed-{sentinel}>")));
        return;
    };
    for (index, value) in values.iter().enumerate() {
        if !value.is_string() {
            findings.insert(Finding::new(
                code,
                &format!("<malformed-{sentinel}-{index}>"),
            ));
        }
    }
}

fn valid_decision_shape(decision: &Value) -> bool {
    decision.get("id").is_some_and(Value::is_string)
        && decision.get("status").is_some_and(Value::is_string)
}

fn bool_field(value: &Value, field: &str) -> bool {
    value.get(field).and_then(Value::as_bool).unwrap_or(false)
}

fn str_set(value: &Value, field: &str) -> BTreeSet<String> {
    str_array(value, field).into_iter().collect()
}

fn str_array(value: &Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn all_four_agree_is_green() {
        let fixture = json!({
            "decisions": [{
                "id": "ADR-0515",
                "status": "Accepted",
                "in_spec": true,
                "in_masterplan": true,
                "in_roadmap": true,
                "supersedes": ["ADR-0511"],
                "superseded_by": [],
                "face_statuses": {"roadmap": "Accepted"}
            }, {
                "id": "ADR-0511",
                "status": "Superseded",
                "in_spec": true,
                "in_masterplan": false,
                "in_roadmap": false,
                "supersedes": [],
                "superseded_by": ["ADR-0515"]
            }]
        });
        let report = evaluate(&fixture);
        assert_eq!(report.verdict, Verdict::Green, "{:?}", report.violations);
    }

    #[test]
    fn axes_drift_fires_generated_face_drift() {
        let fixture = json!({
            "decisions": [valid_decision()],
            "generated_face_axes": {"catalog.json": 6, "contracts.json": 7}
        });
        assert!(
            evaluate(&fixture)
                .violations
                .contains("generated_face_drift")
        );
    }

    #[test]
    fn duplicate_id_fires_dual_decision_collision() {
        let fixture = json!({"decisions": [valid_decision()], "duplicate_ids": ["ADR-0377"]});
        assert!(
            evaluate(&fixture)
                .violations
                .contains("dual_decision_collision")
        );
    }

    /// RED fixture (FRIC-1781430000): a governed surface citing a decision id with no
    /// decision file on disk — the phantom-0397 shape — must go RED.
    #[test]
    fn phantom_citation_fires_phantom_decision_citation() {
        let fixture = json!({
            "decisions": [{
                "id": "ADR-0478",
                "status": "Accepted",
                "in_spec": true, "in_masterplan": true, "in_roadmap": true
            }],
            "phantom_citations": [
                "ADR-0397@docs/decisions/ADR-0478-oya-billing-bespoke-billing-engine.md"
            ]
        });
        let report = evaluate(&fixture);
        assert_eq!(report.verdict, Verdict::Red);
        assert!(report.violations.contains("phantom_decision_citation"));
    }

    /// An empty phantom_citations array (the healed live shape) contributes nothing.
    #[test]
    fn empty_phantom_citations_is_quiet() {
        let fixture = json!({"decisions": [valid_decision()], "phantom_citations": []});
        assert_eq!(evaluate(&fixture).verdict, Verdict::Green);
    }

    /// RED fixture (FRIC-1781320000): a filename/front-matter id disagreement — the
    /// re-keying vector that can mask a duplicate-numbered ADR pair — must go RED.
    #[test]
    fn id_mismatch_fires_decision_id_mismatch() {
        let fixture = json!({
            "decisions": [valid_decision()],
            "id_mismatches": ["ADR-0552-x.md:ADR-0552!=ADR-0553"]
        });
        let report = evaluate(&fixture);
        assert_eq!(report.verdict, Verdict::Red);
        assert!(report.violations.contains("decision_id_mismatch"));
    }

    #[test]
    fn half_supersession_fires_half_edge() {
        // ADR-0511 supersedes ADR-0359, but ADR-0359.superseded_by omits ADR-0511.
        let fixture = json!({
            "decisions": [{
                "id": "ADR-0511",
                "status": "Superseded",
                "in_spec": true, "in_masterplan": false, "in_roadmap": false,
                "supersedes": ["ADR-0359"], "superseded_by": ["ADR-0515"]
            }, {
                "id": "ADR-0359",
                "status": "Superseded",
                "in_spec": true, "in_masterplan": false, "in_roadmap": false,
                "supersedes": [], "superseded_by": ["ADR-0515"]
            }]
        });
        assert!(
            evaluate(&fixture)
                .violations
                .contains("supersession_half_edge")
        );
    }

    #[test]
    fn evaluate_keyed_carries_stable_keys() {
        // half-edge keyed by directed pair, dual keyed by id, drift keyed by faces.
        let fixture = json!({
            "decisions": [{
                "id": "ADR-0511",
                "status": "Superseded",
                "in_spec": true, "in_masterplan": false, "in_roadmap": false,
                "supersedes": ["ADR-0359"], "superseded_by": []
            }, {
                "id": "ADR-0359",
                "status": "Superseded",
                "in_spec": true, "in_masterplan": false, "in_roadmap": false,
                "supersedes": [], "superseded_by": []
            }],
            "duplicate_ids": ["ADR-0377"],
            "id_mismatches": ["ADR-0552-x.md:ADR-0552!=ADR-0553"],
            "phantom_citations": ["ADR-0397@specs/master-plan-sequencing.json"],
            "generated_face_axes": {"catalog.json": 6, "contracts.json": 7}
        });
        let findings = evaluate_keyed(&fixture);
        assert!(findings.contains(&Finding::new(
            "supersession_half_edge",
            "ADR-0511->ADR-0359"
        )));
        assert!(findings.contains(&Finding::new("dual_decision_collision", "ADR-0377")));
        assert!(findings.contains(&Finding::new(
            "decision_id_mismatch",
            "ADR-0552-x.md:ADR-0552!=ADR-0553"
        )));
        assert!(findings.contains(&Finding::new(
            "phantom_decision_citation",
            "ADR-0397@specs/master-plan-sequencing.json"
        )));
        assert!(findings.contains(&Finding::new(
            "generated_face_drift",
            "axes_count@{catalog.json,contracts.json}"
        )));
        // evaluate() is the bare-code projection.
        let projected: BTreeSet<String> = findings.iter().map(|f| f.code.clone()).collect();
        assert_eq!(evaluate(&fixture).violations, projected);
    }

    #[test]
    fn status_disagreement_keyed_by_decision_and_face() {
        let findings = evaluate_keyed(&json!({"decisions":[{
            "id":"ADR-0500","status":"Accepted",
            "in_spec":true,"in_masterplan":true,"in_roadmap":true,
            "face_statuses":{"roadmap":"Superseded"}
        }]}));
        assert!(findings.contains(&Finding::new("status_disagreement", "ADR-0500@roadmap")));
    }

    #[test]
    fn malformed_payload_shapes_fail_closed_with_existing_codes() {
        let malformed_cases = [
            ("orphan_decision", "<missing-decisions>", json!({})),
            (
                "orphan_decision",
                "<non-array-decisions>",
                json!({"decisions": {}}),
            ),
            (
                "orphan_decision",
                "<empty-decisions>",
                json!({"decisions": []}),
            ),
            (
                "orphan_decision",
                "<malformed-decision-0>",
                json!({"decisions": [null]}),
            ),
            (
                "orphan_decision",
                "<malformed-decision-0.id>",
                json!({"decisions": [{"status":"Accepted"}]}),
            ),
            (
                "orphan_decision",
                "<malformed-decision-0.status>",
                json!({"decisions": [{"id":"ADR-0500"}]}),
            ),
            (
                "dual_decision_collision",
                "<malformed-duplicate-ids>",
                json!({"decisions": [valid_decision()], "duplicate_ids": {}}),
            ),
            (
                "dual_decision_collision",
                "<malformed-duplicate-ids-0>",
                json!({"decisions": [valid_decision()], "duplicate_ids": [7]}),
            ),
            (
                "decision_id_mismatch",
                "<malformed-id-mismatches>",
                json!({"decisions": [valid_decision()], "id_mismatches": {}}),
            ),
            (
                "decision_id_mismatch",
                "<malformed-id-mismatches-0>",
                json!({"decisions": [valid_decision()], "id_mismatches": [7]}),
            ),
            (
                "phantom_decision_citation",
                "<malformed-phantom-citations>",
                json!({"decisions": [valid_decision()], "phantom_citations": {}}),
            ),
            (
                "phantom_decision_citation",
                "<malformed-phantom-citations-0>",
                json!({"decisions": [valid_decision()], "phantom_citations": [7]}),
            ),
            (
                "supersession_half_edge",
                "<malformed-decision-0.supersedes>",
                json!({"decisions": [decision_with("supersedes", json!({}))]}),
            ),
            (
                "supersession_half_edge",
                "<malformed-decision-0.supersedes-0>",
                json!({"decisions": [decision_with("supersedes", json!([7]))]}),
            ),
            (
                "supersession_half_edge",
                "<malformed-decision-0.superseded_by>",
                json!({"decisions": [decision_with("superseded_by", json!({}))]}),
            ),
            (
                "supersession_half_edge",
                "<malformed-decision-0.superseded_by-0>",
                json!({"decisions": [decision_with("superseded_by", json!([7]))]}),
            ),
            (
                "unpropagated_decision",
                "<malformed-decision-0.in_spec>",
                json!({"decisions": [decision_with("in_spec", json!("yes"))]}),
            ),
            (
                "unpropagated_decision",
                "<malformed-decision-0.in_masterplan>",
                json!({"decisions": [decision_with("in_masterplan", json!("yes"))]}),
            ),
            (
                "unpropagated_decision",
                "<malformed-decision-0.in_roadmap>",
                json!({"decisions": [decision_with("in_roadmap", json!("yes"))]}),
            ),
            (
                "status_disagreement",
                "<malformed-decision-0.face_statuses>",
                json!({"decisions": [decision_with("face_statuses", json!([]))]}),
            ),
            (
                "status_disagreement",
                "<malformed-decision-0.face_statuses.roadmap>",
                json!({"decisions": [decision_with("face_statuses", json!({"roadmap": 7}))]}),
            ),
            (
                "generated_face_drift",
                "<malformed-generated-face-axes>",
                json!({"decisions": [valid_decision()], "generated_face_axes": []}),
            ),
            (
                "generated_face_drift",
                "<malformed-generated-face-axes>",
                json!({"decisions": [valid_decision()], "generated_face_axes": {}}),
            ),
            (
                "generated_face_drift",
                "<malformed-generated-face-axes.catalog.json>",
                json!({"decisions": [valid_decision()], "generated_face_axes": {"catalog.json": "six"}}),
            ),
        ];

        for (code, key, fixture) in malformed_cases {
            let findings = evaluate_keyed(&fixture);
            assert!(
                findings.contains(&Finding::new(code, key)),
                "{code}/{key} should fail closed, got {findings:?}"
            );
            let report = evaluate(&fixture);
            assert_eq!(report.verdict, Verdict::Red, "{code}/{key}");
            assert!(report.violations.contains(code), "{code}/{key}");
        }
    }

    #[test]
    fn malformed_propagation_bool_on_non_accepted_decision_fails_closed() {
        let mut decision = valid_decision();
        decision["status"] = json!("Superseded");
        decision["in_spec"] = json!("yes");
        let fixture = json!({"decisions": [decision]});
        assert_eq!(
            evaluate_keyed(&fixture),
            BTreeSet::from([Finding::new(
                "unpropagated_decision",
                "<malformed-decision-0.in_spec>"
            )])
        );
        assert_eq!(evaluate(&fixture).verdict, Verdict::Red);
    }
    #[test]
    fn masterplan_v2_authority_accepts_single_live_namespace() {
        let findings = evaluate_masterplan_v2_authority(&minimal_masterplan_v2());
        assert!(
            findings.is_empty(),
            "minimal valid masterplan v2 authority contract should be green: {findings:?}"
        );
    }

    #[test]
    fn masterplan_v2_evidence_state_marks_hermes_done_claims_unverified_without_evidence() {
        let mut fixture = minimal_masterplan_v2();
        fixture["masterplan_v2"]["hermes_done_card_imports"] = json!([
            {
                "source_system": "hermes",
                "source_card_id": "t_done_without_evidence",
                "source_status": "done",
                "completion_claim": "hermes-done-card",
                "masterplan_status": "claimed-done-unverified",
                "evidence_state": "claimed-done-unverified",
                "evidence_refs": []
            },
            {
                "source_system": "hermes",
                "source_card_id": "t_done_with_evidence",
                "source_status": "done",
                "completion_claim": "hermes-done-card",
                "masterplan_status": "done",
                "evidence_state": "evidence-attached",
                "evidence_refs": ["fixture://merged-pr/evidence"]
            }
        ]);
        fixture["masterplan_v2"]["work_items"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "id": "MPV2-0002",
                "status": "done",
                "evidence_refs": ["fixture://work-item/evidence"]
            }));

        let findings = evaluate_masterplan_v2_evidence_state(&fixture);
        assert!(
            findings.is_empty(),
            "Hermes done claims without evidence must be unverified, and evidence-backed completion may surface: {findings:?}"
        );
    }

    #[test]
    fn masterplan_v2_evidence_state_rejects_unverified_hermes_done_as_done() {
        let mut fixture = minimal_masterplan_v2();
        fixture["masterplan_v2"]["hermes_done_card_imports"] = json!([
            {
                "source_system": "hermes",
                "source_card_id": "t_unverified_done",
                "source_status": "done",
                "completion_claim": "hermes-done-card",
                "masterplan_status": "done",
                "evidence_state": "done",
                "evidence_refs": []
            }
        ]);
        fixture["masterplan_v2"]["work_items"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "id": "MPV2-0002",
                "status": "done",
                "evidence_refs": []
            }));

        let findings = evaluate_masterplan_v2_evidence_state(&fixture);
        assert!(findings.contains(&Finding::new(
            "masterplan_evidence_state_invalid",
            "t_unverified_done@hermes_done_card_imports[0].evidence_state"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_evidence_state_invalid",
            "t_unverified_done@hermes_done_card_imports[0].masterplan_status"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_evidence_state_invalid",
            "MPV2-0002@work_items[2].evidence_refs"
        )));
    }
    #[test]
    fn masterplan_v2_plan_evidence_drift_accepts_audited_status_refs() {
        let mut fixture = minimal_masterplan_v2();
        fixture["masterplan_v2"]["work_items"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "id": "MPV2-0002",
                "status": "done",
                "evidence_refs": ["cloud/cloud-ci/gates/oya-cloud-ci-cross-artifact-agreement-app/tests/cross_artifact_agreement.rs#masterplan_v2_plan_vs_evidence_drift_contract_is_green"]
            }));
        fixture["masterplan_v2"]["hermes_done_card_imports"] = json!([
            {
                "source_system": "hermes",
                "source_card_id": "t_done_with_evidence",
                "source_status": "done",
                "completion_claim": "hermes-done-card",
                "masterplan_status": "done",
                "evidence_state": "evidence-attached",
                "evidence_refs": ["evidence/quality-gate/G013-quality-gate-checkpoint.json"]
            }
        ]);

        let findings = evaluate_masterplan_v2_plan_evidence_drift(&fixture);
        assert!(
            findings.is_empty(),
            "audited evidence refs and evidence-attached Hermes done claims should be green: {findings:?}"
        );
    }

    #[test]
    fn masterplan_v2_plan_evidence_drift_rejects_stale_refs_and_state_drift() {
        let mut fixture = minimal_masterplan_v2();
        fixture["masterplan_v2"]["evidence_state_policy"]["validator"] =
            json!("cloud-ci-cross-artifact-agreement/masterplan-v2-evidence-state");
        fixture["masterplan_v2"]["work_items"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "id": "MPV2-0002",
                "status": "done",
                "evidence_refs": [".omc/ultragoal/goals.json#done-card"]
            }));
        fixture["masterplan_v2"]["hermes_done_card_imports"] = json!([
            {
                "source_system": "hermes",
                "source_card_id": "t_evidence_attached",
                "source_status": "done",
                "completion_claim": "hermes-done-card",
                "masterplan_status": "done",
                "evidence_state": "claimed-done-unverified",
                "evidence_refs": ["fixture://merged-pr/evidence"]
            }
        ]);

        let findings = evaluate_masterplan_v2_plan_evidence_drift(&fixture);
        assert!(findings.contains(&Finding::new(
            "masterplan_plan_evidence_drift",
            "masterplan_v2.evidence_state_policy.validator"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_plan_evidence_drift",
            "MPV2-0002@work_items[2].evidence_refs[0]"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_plan_evidence_drift",
            "t_evidence_attached@hermes_done_card_imports[0].evidence_state"
        )));
    }

    #[test]
    fn masterplan_v2_authority_rejects_legacy_live_surface_duplicate_and_external_ids() {
        let mut fixture = minimal_masterplan_v2();
        fixture["masterplan_v2"]["surface_dispositions"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "path": "/specs/master-plan-sequencing.json",
                "is_live_plan_authority": true
            }));
        fixture["masterplan_v2"]["work_items"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id": "MPV2-0001"}));
        fixture["masterplan_v2"]["work_items"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "id": "FD-001",
                "external_live_id": "FD-001"
            }));

        let findings = evaluate_masterplan_v2_authority(&fixture);
        assert!(findings.contains(&Finding::new(
            "masterplan_not_sole_live_authority",
            "/specs/master-plan-sequencing.json,/specs/masterplan.json"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_work_item_id_collision",
            "MPV2-0001"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_external_live_work_item_id",
            "FD-001"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_external_live_work_item_id",
            "FD-001@work_items[3]"
        )));
    }
    #[test]
    fn masterplan_v2_authority_requires_non_hermes_surface_disposition_coverage() {
        let mut fixture = minimal_masterplan_v2();
        fixture["masterplan_v2"]["surface_dispositions"]
            .as_array_mut()
            .unwrap()
            .retain(|surface| surface.get("path").and_then(Value::as_str) != Some(".omc/**"));

        let findings = evaluate_masterplan_v2_authority(&fixture);
        assert!(findings.contains(&Finding::new(
            "masterplan_surface_disposition_incomplete",
            ".omc/**"
        )));
    }

    #[test]
    fn masterplan_v2_authority_rejects_malformed_surface_disposition_metadata() {
        let mut fixture = minimal_masterplan_v2();
        let surfaces = fixture["masterplan_v2"]["surface_dispositions"]
            .as_array_mut()
            .unwrap();
        for surface in surfaces {
            match surface.get("path").and_then(Value::as_str) {
                Some("docs/ROADMAP.md") => {
                    surface["disposition"] = json!("live-roadmap");
                }
                Some(".gjc/**") => {
                    surface.as_object_mut().unwrap().remove("absorbed_into");
                }
                _ => {}
            }
        }

        let findings = evaluate_masterplan_v2_authority(&fixture);
        assert!(findings.contains(&Finding::new(
            "masterplan_surface_disposition_incomplete",
            "docs/ROADMAP.md.disposition"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_surface_disposition_incomplete",
            ".gjc/**.absorbed_into"
        )));
    }

    #[test]
    fn masterplan_v2_dependency_dag_rejects_unknown_refs_direction_contract_and_cycles() {
        let mut malformed = minimal_masterplan_v2();
        malformed["masterplan_v2"]["dependency_edge_semantics"] =
            json!("to is prerequisite, from is dependent");
        malformed["masterplan_v2"]["dependency_edges"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "from": "MPV2-9999",
                "to": "MPV2-0001",
                "relationship": "unblocks"
            }));
        malformed["masterplan_v2"]["dependency_edges"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "from": "MPV2-0000",
                "to": "MPV2-9998",
                "relationship": "unblocks"
            }));

        let findings = evaluate_masterplan_v2_authority(&malformed);
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "<dependency-edge-semantics>"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "MPV2-9999@dependency_edges[1].from"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "MPV2-9998@dependency_edges[2].to"
        )));

        let mut cyclic = minimal_masterplan_v2();
        cyclic["masterplan_v2"]["dependency_edges"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "from": "MPV2-0001",
                "to": "MPV2-0000",
                "relationship": "unblocks"
            }));
        cyclic["masterplan_v2"]["dependency_edges"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "from": "MPV2-0001",
                "to": "MPV2-0001",
                "relationship": "unblocks"
            }));

        let findings = evaluate_masterplan_v2_authority(&cyclic);
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "cycle@{MPV2-0000,MPV2-0001}"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "MPV2-0001->MPV2-0001@self"
        )));
    }
    /// A masterplan work-item corpus with a program-boundary crossing:
    /// MPV2-0000 (P-FABRIC) -> MPV2-0001 (P-REORG) via the minimal fixture's
    /// single dependency edge.
    fn cross_program_masterplan_v2() -> Value {
        let mut masterplan = minimal_masterplan_v2();
        masterplan["masterplan_v2"]["programs"] = json!([{"id": "P-FABRIC"}, {"id": "P-REORG"}]);
        masterplan["masterplan_v2"]["work_items"] = json!([
            {"id": "MPV2-0000", "program": "P-FABRIC"},
            {"id": "MPV2-0001", "program": "P-REORG"}
        ]);
        masterplan
    }
    #[test]
    fn masterplan_v2_dependency_dag_rejects_undeclared_cross_program_edges() {
        // A crossing with NO policy at all: missing-policy + undeclared both fire.
        let missing_policy = cross_program_masterplan_v2();
        let findings = evaluate_masterplan_v2_authority(&missing_policy);
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "<missing-cross-program-edge-policy>"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "MPV2-0000->MPV2-0001@cross-program-undeclared(P-FABRIC->P-REORG)"
        )));

        // A crossing with a well-formed policy that does NOT declare the pair.
        let mut undeclared = cross_program_masterplan_v2();
        undeclared["masterplan_v2"]["cross_program_edge_policy"] = json!({
            "semantics": CROSS_PROGRAM_EDGE_SEMANTICS,
            "declared_program_edges": []
        });
        let findings = evaluate_masterplan_v2_authority(&undeclared);
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "MPV2-0000->MPV2-0001@cross-program-undeclared(P-FABRIC->P-REORG)"
        )));
        assert!(
            !findings
                .iter()
                .any(|finding| finding.key == "<missing-cross-program-edge-policy>"),
            "a present policy must not trip the missing-policy guard: {findings:?}"
        );

        // Same-program edges never require a declaration.
        let mut same_program = cross_program_masterplan_v2();
        same_program["masterplan_v2"]["work_items"] = json!([
            {"id": "MPV2-0000", "program": "P-FABRIC"},
            {"id": "MPV2-0001", "program": "P-FABRIC"}
        ]);
        let findings = evaluate_masterplan_v2_authority(&same_program);
        assert!(
            findings.is_empty(),
            "same-program edges must not require cross-program declarations: {findings:?}"
        );
    }
    #[test]
    fn masterplan_v2_dependency_dag_rejects_malformed_cross_program_policy() {
        let mut malformed = cross_program_masterplan_v2();
        malformed["masterplan_v2"]["cross_program_edge_policy"] = json!({
            "semantics": "wrong-contract",
            "declared_program_edges": [
                "not-an-object",
                {"from_program": "P-FABRIC"},
                {"from_program": "P-FABRIC", "to_program": "P-FABRIC"},
                {"from_program": "P-UNKNOWN", "to_program": "P-REORG"},
                {"from_program": "P-FABRIC", "to_program": "P-REORG"},
                {"from_program": "P-FABRIC", "to_program": "P-REORG"}
            ]
        });
        let findings = evaluate_masterplan_v2_authority(&malformed);
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "<cross-program-edge-semantics>"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "<malformed-cross-program-declared-edge-0>"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "<malformed-cross-program-declared-edge-1>"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "P-FABRIC->P-FABRIC@cross_program_edge_policy.declared_program_edges[2].self"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "P-UNKNOWN@cross_program_edge_policy.declared_program_edges[3].from_program"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "P-FABRIC->P-REORG@cross_program_edge_policy.declared_program_edges[5].duplicate"
        )));
        // The declared-but-never-instantiated P-UNKNOWN pair is stale.
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "P-UNKNOWN->P-REORG@cross_program_edge_policy.declared_program_edges.unused"
        )));
        // The live crossing IS declared, so no undeclared finding fires.
        assert!(
            !findings
                .iter()
                .any(|finding| finding.key.contains("cross-program-undeclared")),
            "a declared crossing must not fire the undeclared lane: {findings:?}"
        );

        // A policy whose declared_program_edges is not an array fails closed.
        let mut non_array = cross_program_masterplan_v2();
        non_array["masterplan_v2"]["cross_program_edge_policy"] = json!({
            "semantics": CROSS_PROGRAM_EDGE_SEMANTICS,
            "declared_program_edges": "not-an-array"
        });
        let findings = evaluate_masterplan_v2_authority(&non_array);
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "<malformed-cross-program-declared-edges>"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "MPV2-0000->MPV2-0001@cross-program-undeclared(P-FABRIC->P-REORG)"
        )));
    }
    #[test]
    fn masterplan_v2_dependency_dag_accepts_declared_cross_program_edges() {
        let mut declared = cross_program_masterplan_v2();
        declared["masterplan_v2"]["cross_program_edge_policy"] = json!({
            "semantics": CROSS_PROGRAM_EDGE_SEMANTICS,
            "declared_program_edges": [{
                "from_program": "P-FABRIC",
                "to_program": "P-REORG",
                "rationale": "test crossing"
            }]
        });
        let findings = evaluate_masterplan_v2_authority(&declared);
        assert!(
            findings.is_empty(),
            "a declared, instantiated cross-program edge must be green: {findings:?}"
        );

        // Anti-staleness: an extra declared pair no live edge instantiates fails.
        let mut stale = cross_program_masterplan_v2();
        stale["masterplan_v2"]["cross_program_edge_policy"] = json!({
            "semantics": CROSS_PROGRAM_EDGE_SEMANTICS,
            "declared_program_edges": [
                {
                    "from_program": "P-FABRIC",
                    "to_program": "P-REORG",
                    "rationale": "test crossing"
                },
                {
                    "from_program": "P-REORG",
                    "to_program": "P-FABRIC",
                    "rationale": "declared but never instantiated"
                }
            ]
        });
        let findings = evaluate_masterplan_v2_authority(&stale);
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "P-REORG->P-FABRIC@cross_program_edge_policy.declared_program_edges.unused"
        )));
    }
    /// Fail-closed contract for the masterplan structural gate: a corpus that
    /// is MISSING the structures under test (masterplan_v2 itself, the work-item
    /// set, or the dependency-edge set) must go RED, never silently green. An
    /// absent graph is indistinguishable from a deleted one; both are blocking.
    #[test]
    fn masterplan_v2_structural_gate_fails_closed_on_missing_or_malformed_inputs() {
        // No masterplan_v2 object at all.
        let findings = evaluate_masterplan_v2_authority(&json!({}));
        assert!(findings.contains(&Finding::new(
            "masterplan_not_sole_live_authority",
            "<missing-masterplan_v2>"
        )));

        // work_items missing entirely.
        let mut missing_items = minimal_masterplan_v2();
        missing_items["masterplan_v2"]
            .as_object_mut()
            .unwrap()
            .remove("work_items");
        let findings = evaluate_masterplan_v2_authority(&missing_items);
        assert!(findings.contains(&Finding::new(
            "masterplan_external_live_work_item_id",
            "<missing-work-items>"
        )));

        // work_items present but empty: an empty live ID space is not a plan.
        let mut empty_items = minimal_masterplan_v2();
        empty_items["masterplan_v2"]["work_items"] = json!([]);
        let findings = evaluate_masterplan_v2_authority(&empty_items);
        assert!(findings.contains(&Finding::new(
            "masterplan_external_live_work_item_id",
            "<empty-work-items>"
        )));

        // dependency_edges missing entirely.
        let mut missing_edges = minimal_masterplan_v2();
        missing_edges["masterplan_v2"]
            .as_object_mut()
            .unwrap()
            .remove("dependency_edges");
        let findings = evaluate_masterplan_v2_authority(&missing_edges);
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "<missing-dependency-edges>"
        )));

        // dependency_edges present but not an array.
        let mut malformed_edges = minimal_masterplan_v2();
        malformed_edges["masterplan_v2"]["dependency_edges"] = json!("not-an-array");
        let findings = evaluate_masterplan_v2_authority(&malformed_edges);
        assert!(findings.contains(&Finding::new(
            "masterplan_dependency_dag_invalid",
            "<malformed-dependency-edges>"
        )));
    }
    #[test]
    fn masterplan_v2_program_coverage_accepts_manifest_index_shards() {
        let findings = evaluate_masterplan_v2_program_coverage(
            &minimal_program_coverage_masterplan(),
            &minimal_manifest_index(),
        );
        assert!(
            findings.is_empty(),
            "minimal valid program coverage should be green: {findings:?}"
        );
    }

    #[test]
    fn masterplan_v2_program_coverage_rejects_missing_microservice_and_required_shard() {
        let mut fixture = minimal_program_coverage_masterplan();
        fixture["masterplan_v2"]["programs"]
            .as_array_mut()
            .unwrap()
            .retain(|program| {
                program.get("program_class").and_then(Value::as_str) != Some("ast-code-graph")
            });
        fixture["masterplan_v2"]["program_coverage"]["microservices"]
            .as_array_mut()
            .unwrap()
            .retain(|entry| {
                entry.get("microservice").and_then(Value::as_str) != Some("workflow-studio")
            });

        let findings = evaluate_masterplan_v2_program_coverage(&fixture, &minimal_manifest_index());
        assert!(findings.contains(&Finding::new(
            "masterplan_program_coverage_incomplete",
            "program_class:ast-code-graph"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_program_coverage_incomplete",
            "microservice:workflow-studio"
        )));
    }
    /// RED: the work-item → program-shard reference edge is a dangling-reference
    /// class — an undeclared program id or a missing program assignment must both
    /// fire, so no work item can float outside the program-sharded coverage proof.
    #[test]
    fn masterplan_v2_program_coverage_rejects_dangling_work_item_program_refs() {
        let mut fixture = minimal_program_coverage_masterplan();
        fixture["masterplan_v2"]["work_items"] = json!([{
            "id": "MPV2-0000",
            "program": "agentic-delivery-fabric"
        }, {
            "id": "MPV2-0001"
        }]);

        let findings = evaluate_masterplan_v2_program_coverage(&fixture, &minimal_manifest_index());
        assert!(findings.contains(&Finding::new(
            "masterplan_program_coverage_incomplete",
            "work_item:MPV2-0000@unknown-program:agentic-delivery-fabric"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_program_coverage_incomplete",
            "work_item:MPV2-0001@missing-program"
        )));
    }
    #[test]
    fn masterplan_v2_program_coverage_rejects_broken_owned_stack_ladder() {
        // Missing ladder entirely.
        let mut missing = minimal_program_coverage_masterplan();
        missing["masterplan_v2"]["program_coverage"]
            .as_object_mut()
            .unwrap()
            .remove("owned_stack_ladder");
        let findings = evaluate_masterplan_v2_program_coverage(&missing, &minimal_manifest_index());
        assert!(findings.contains(&Finding::new(
            "masterplan_program_coverage_incomplete",
            "<missing-owned-stack-ladder>"
        )));

        // Dropped rung plus an unknown covering program.
        let mut broken = minimal_program_coverage_masterplan();
        let rungs = broken["masterplan_v2"]["program_coverage"]["owned_stack_ladder"]["rungs"]
            .as_array_mut()
            .unwrap();
        rungs.retain(|rung| rung.get("layer").and_then(Value::as_str) != Some("cloud-os"));
        rungs[0]["program_ids"] = json!(["P-UNKNOWN"]);
        let findings = evaluate_masterplan_v2_program_coverage(&broken, &minimal_manifest_index());
        assert!(findings.contains(&Finding::new(
            "masterplan_program_coverage_incomplete",
            "owned_stack_ladder:cloud-os"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_program_coverage_incomplete",
            "owned_stack_ladder:cloud-kernel@unknown-program:P-UNKNOWN"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_program_coverage_incomplete",
            "owned_stack_ladder.rungs@count"
        )));
    }

    #[test]
    fn masterplan_v2_projection_freshness_accepts_generated_and_read_projection_coverage() {
        let findings = evaluate_masterplan_v2_projection_freshness(
            &minimal_projection_freshness_masterplan(),
            Some(&minimal_generated_artifact_control_plane()),
        );
        assert!(
            findings.is_empty(),
            "minimal generated/read projection freshness contract should be green: {findings:?}"
        );
    }

    #[test]
    fn masterplan_v2_projection_freshness_rejects_missing_and_stale_projection_rows() {
        let mut fixture = minimal_projection_freshness_masterplan();
        let projections = fixture["masterplan_v2"]["projection_freshness"]["projections"]
            .as_array_mut()
            .unwrap();
        projections.retain(|row| {
            row.get("path").and_then(Value::as_str)
                != Some("docs/machine-readable/board-sync.generated.json")
        });
        for projection in projections {
            match projection.get("path").and_then(Value::as_str) {
                Some("docs/MASTERPLAN.md") => {
                    projection["source_of_truth"] = json!("/specs/master-plan-sequencing.json");
                }
                Some("docs/ROADMAP.md") => {
                    projection["live_work_item_ids_allowed"] = json!(true);
                }
                Some("docs/machine-readable/masterplan.generated.json") => {
                    projection["evidence_refs"] = json!([]);
                }
                _ => {}
            }
        }

        let findings = evaluate_masterplan_v2_projection_freshness(
            &fixture,
            Some(&minimal_generated_artifact_control_plane()),
        );
        assert!(findings.contains(&Finding::new(
            "masterplan_projection_freshness_invalid",
            "docs/machine-readable/board-sync.generated.json"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_projection_freshness_invalid",
            "docs/MASTERPLAN.md.source_of_truth"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_projection_freshness_invalid",
            "docs/ROADMAP.md.live_work_item_ids_allowed"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_projection_freshness_invalid",
            "docs/machine-readable/masterplan.generated.json.evidence_refs"
        )));
    }
    #[test]
    fn masterplan_v2_projection_freshness_derives_fragmented_masterplan_source_inputs() {
        let registry = json!({
            "artifacts": [{
                "path": "docs/machine-readable/masterplan-fragment.generated.json",
                "source_inputs": ["/specs/masterplan.json#masterplan_v2"]
            }]
        });

        let findings = evaluate_masterplan_v2_projection_freshness(
            &minimal_projection_freshness_masterplan(),
            Some(&registry),
        );

        assert!(findings.contains(&Finding::new(
            "masterplan_projection_freshness_invalid",
            "docs/machine-readable/masterplan-fragment.generated.json.read_contract"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_projection_freshness_invalid",
            "docs/machine-readable/masterplan-fragment.generated.json"
        )));
    }
    #[test]
    fn masterplan_v2_read_contract_archives_accept_archive_only_refs() {
        let findings = evaluate_masterplan_v2_read_contract_archives(
            &minimal_projection_freshness_masterplan(),
        );
        assert!(
            findings.is_empty(),
            "archived stale read paths referenced only as provenance archives should be green: {findings:?}"
        );
    }

    #[test]
    fn masterplan_v2_read_contract_archives_reject_non_archived_refs_to_stale_paths() {
        let mut fixture = minimal_projection_freshness_masterplan();

        for contract in fixture["masterplan_v2"]["read_contracts"]
            .as_array_mut()
            .unwrap()
        {
            if contract.get("path").and_then(Value::as_str) == Some("docs/ROADMAP.md") {
                contract["read_timing_class"] = json!("on-demand");
            }
        }
        for projection in fixture["masterplan_v2"]["projection_freshness"]["projections"]
            .as_array_mut()
            .unwrap()
        {
            if projection.get("path").and_then(Value::as_str) == Some("docs/ROADMAP.md") {
                projection["read_timing_class"] = json!("on-demand");
            }
        }
        fixture["masterplan_v2"]["read_path_references"] = json!([
            {
                "path": ".omc/ultragoal/goals.json",
                "read_timing_class": "on-demand",
                "source": "fixture stale runtime pointer"
            }
        ]);

        let findings = evaluate_masterplan_v2_read_contract_archives(&fixture);
        assert!(findings.contains(&Finding::new(
            "masterplan_read_contract_invalid",
            "docs/ROADMAP.md.read_contract.read_timing_class"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_read_contract_invalid",
            "docs/ROADMAP.md.projection_freshness.read_timing_class"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_read_contract_invalid",
            ".omc/ultragoal/goals.json.read_path_references[0].read_timing_class"
        )));
    }

    #[test]
    fn masterplan_v2_entry_surface_accepts_exact_root_hub_allowlist() {
        let findings = evaluate_masterplan_v2_entry_surfaces(
            &minimal_projection_freshness_masterplan(),
            &minimal_root_hub_entry_surface_allowlist(),
        );
        assert!(
            findings.is_empty(),
            "minimal bounded entry-surface contract should be green: {findings:?}"
        );
    }

    #[test]
    fn masterplan_v2_entry_surface_rejects_superseded_and_unallowlisted_entrypoints() {
        let mut masterplan = minimal_projection_freshness_masterplan();
        for contract in masterplan["masterplan_v2"]["read_contracts"]
            .as_array_mut()
            .unwrap()
        {
            match contract.get("path").and_then(Value::as_str) {
                Some("/specs/master-plan-sequencing.json") => {
                    contract["read_timing_class"] = json!("entry-surface");
                }
                Some("docs/MASTERPLAN.md") => {
                    contract["read_timing_class"] = json!("entry-surface");
                }
                _ => {}
            }
        }

        let mut root_hub = minimal_root_hub_entry_surface_allowlist();
        root_hub["agent_entry_surface_allowlist"]["paths"]
            .as_array_mut()
            .unwrap()
            .push(json!("/specs/master-plan-sequencing.json"));

        let findings = evaluate_masterplan_v2_entry_surfaces(&masterplan, &root_hub);
        assert!(findings.contains(&Finding::new(
            "masterplan_entry_surface_invalid",
            "/specs/master-plan-sequencing.json.root_hub_entry_superseded"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_entry_surface_invalid",
            "/specs/master-plan-sequencing.json.allowlisted_superseded_entrypoint"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_entry_surface_invalid",
            "/specs/master-plan-sequencing.json.superseded_entry_surface_read_contract"
        )));
        assert!(findings.contains(&Finding::new(
            "masterplan_entry_surface_invalid",
            "docs/MASTERPLAN.md.unexpected_entry_surface_read_contract"
        )));
    }

    #[test]
    fn public_code_list_includes_every_emitted_code_used_by_tests() {
        let public: BTreeSet<&str> = VIOLATION_CODES.into_iter().collect();
        for code in [
            "orphan_decision",
            "unpropagated_decision",
            "status_disagreement",
            "generated_face_drift",
            "dual_decision_collision",
            "decision_id_mismatch",
            "supersession_half_edge",
            "phantom_decision_citation",
            "masterplan_not_sole_live_authority",
            "masterplan_surface_disposition_incomplete",
            "masterplan_work_item_id_collision",
            "masterplan_external_live_work_item_id",
            "masterplan_dependency_dag_invalid",
            "masterplan_program_coverage_incomplete",
            "masterplan_evidence_state_invalid",
            "masterplan_plan_evidence_drift",
            "masterplan_plan_evidence_unrecorded",
            "masterplan_projection_freshness_invalid",
            "masterplan_projection_stale",
            "masterplan_read_contract_invalid",
            "masterplan_entry_surface_invalid",
        ] {
            assert!(public.contains(code), "missing public code {code}");
        }
    }

    fn minimal_surface_disposition(path: &str, disposition: &str) -> Value {
        let live_authority = disposition == DISPOSITION_CANONICAL_AUTHORITY;
        let work_item_id_policy = if live_authority {
            "defines-the-single-live-id-space"
        } else {
            "must-not-mint-live-work-item-ids"
        };
        let mut surface = json!({
            "path": path,
            "disposition": disposition,
            "is_live_plan_authority": live_authority,
            "work_item_id_policy": work_item_id_policy,
            "provenance": "minimal test fixture"
        });
        if !live_authority {
            surface["absorbed_into"] = json!(MASTERPLAN_V2_REF);
        }
        surface
    }

    fn minimal_surface_dispositions() -> Value {
        Value::Array(
            REQUIRED_SURFACE_DISPOSITIONS
                .iter()
                .map(|&(path, disposition)| minimal_surface_disposition(path, disposition))
                .collect(),
        )
    }
    fn minimal_masterplan_v2() -> Value {
        json!({
            "masterplan_v2": {
                "canonical_plan_authority": {
                    "path": "/specs/masterplan.json",
                    "live_work_item_id_space": {
                        "authority_path": "/specs/masterplan.json",
                        "id_prefix": "MPV2-",
                        "numeric_width": 4,
                        "external_live_ids_allowed": false,
                        "duplicate_ids_allowed": false
                    }
                },
                "evidence_state_policy": {
                    "status_claims_require_evidence_refs": true,
                    "hermes_done_claim_without_evidence_state": CLAIMED_DONE_UNVERIFIED_STATE,
                    "evidence_attached_completion_state": EVIDENCE_ATTACHED_STATE,
                    "validator": PLAN_EVIDENCE_DRIFT_VALIDATOR,
                    "policy_ref": "test"
                },
                "surface_dispositions": minimal_surface_dispositions(),
                "work_items": [{
                    "id": "MPV2-0000"
                }, {
                    "id": "MPV2-0001"
                }],
                "dependency_edge_semantics": "from is prerequisite, to is dependent",
                "dependency_edges": [{
                    "from": "MPV2-0000",
                    "to": "MPV2-0001",
                    "relationship": "unblocks"
                }]
            }
        })
    }
    fn minimal_sequenced_masterplan(
        founder_ratification: Value,
        execution_wave_dispatch: Value,
    ) -> Value {
        let mut masterplan = minimal_masterplan_v2();
        masterplan["masterplan_v2"]["sequencing"] = json!({
            "derivation_mode": SEQUENCING_DERIVATION_MODE,
            "source_of_truth": SEQUENCING_SOURCE_OF_TRUTH,
            "index_base": 0,
            "legacy_order_imported": false,
            "derivation_evidence_refs": [
                "cloud/cloud-ci/gates/oya-cloud-ci-cross-artifact-agreement-app/src/lib.rs#evaluate_masterplan_v2_sequencing"
            ],
            "work_item_order": [
                {"index": 0, "work_item_id": "MPV2-0000"},
                {"index": 1, "work_item_id": "MPV2-0001"}
            ],
            "execution_waves": [
                {"wave_index": 0, "work_item_ids": ["MPV2-0000"]},
                {"wave_index": 1, "work_item_ids": ["MPV2-0001"]}
            ],
            "founder_ratification": founder_ratification,
            "execution_wave_dispatch": execution_wave_dispatch,
        });
        masterplan
    }
    #[test]
    fn masterplan_v2_sequencing_pending_founder_stays_fail_closed() {
        // Pending + properly blocked: the unratified finding fires, but the
        // fail-closed not_blocked guard stays quiet.
        let pending_blocked = minimal_sequenced_masterplan(
            json!({
                "decision_recorded": false,
                "decision_status": "pending_founder_ratification"
            }),
            json!({
                "requires_founder_ratification": true,
                "allowed_without_founder_ratification": false,
                "state": DISPATCH_BLOCKED_STATE,
                "blocked_reason": DISPATCH_BLOCKED_REASON
            }),
        );
        let findings = evaluate_masterplan_v2_sequencing(&pending_blocked);
        assert!(findings.contains(&Finding::new(
            "masterplan_execution_wave_dispatch_unratified",
            "masterplan_v2.sequencing.founder_ratification"
        )));
        assert!(
            !findings.iter().any(|finding| {
                finding.key == "masterplan_v2.sequencing.execution_wave_dispatch.not_blocked"
            }),
            "properly blocked pending state must not trip the not_blocked guard: {findings:?}"
        );

        // Pending + NOT blocked: the fail-closed guard must fire.
        let pending_unblocked = minimal_sequenced_masterplan(
            json!({"decision_recorded": false}),
            json!({
                "requires_founder_ratification": true,
                "allowed_without_founder_ratification": false,
                "state": "ratified-awaiting-dispatch"
            }),
        );
        let findings = evaluate_masterplan_v2_sequencing(&pending_unblocked);
        assert!(findings.contains(&Finding::new(
            "masterplan_execution_wave_dispatch_unratified",
            "masterplan_v2.sequencing.execution_wave_dispatch.not_blocked"
        )));
    }
    #[test]
    fn masterplan_v2_sequencing_rejects_incomplete_ratification_records() {
        // A bare decision_recorded=true without approver/ref/timestamp/status is
        // not a founder ratification.
        let incomplete = minimal_sequenced_masterplan(
            json!({"decision_recorded": true}),
            json!({
                "requires_founder_ratification": true,
                "allowed_without_founder_ratification": false,
                "state": DISPATCH_BLOCKED_STATE,
                "blocked_reason": DISPATCH_BLOCKED_REASON
            }),
        );
        let findings = evaluate_masterplan_v2_sequencing(&incomplete);
        assert!(findings.contains(&Finding::new(
            "masterplan_execution_wave_dispatch_unratified",
            "masterplan_v2.sequencing.founder_ratification"
        )));
    }
    #[test]
    fn masterplan_v2_sequencing_accepts_recorded_founder_ratification() {
        let ratified = minimal_sequenced_masterplan(
            json!({
                "decision_recorded": true,
                "decision_status": "ratified",
                "approved_by": "founder",
                "recorded_at": "2026-07-02T00:00:00Z",
                "decision_ref": "evidence/goals/masterplan-v2-sequencing-founder-ratification-20260702.json"
            }),
            json!({
                "requires_founder_ratification": true,
                "allowed_without_founder_ratification": false,
                "state": "ratified-awaiting-dispatch"
            }),
        );
        let findings = evaluate_masterplan_v2_sequencing(&ratified);
        assert!(
            findings.is_empty(),
            "a recorded founder ratification with fail-closed dispatch flags must be green: {findings:?}"
        );
    }
    fn minimal_program_coverage_masterplan() -> Value {
        let mut masterplan = minimal_masterplan_v2();
        masterplan["masterplan_v2"]["programs"] = json!([
            minimal_program("P-ONTOLOGY", "ontology"),
            minimal_program("P-WORKFLOW-ENGINE", "workflow-engine"),
            minimal_program("P-WORKFLOW-STUDIO", "workflow-studio"),
            minimal_program("P-INTELLIGENCE", "intelligence"),
            minimal_owned_stack_program("P-OWNED-STACK-KERNEL", "cloud-kernel"),
            minimal_owned_stack_program("P-OWNED-STACK-OS", "cloud-os"),
            minimal_owned_stack_program("P-OWNED-STACK-K8S", "cloud-k8s"),
            minimal_owned_stack_program("P-OWNED-STACK-CLOUD", "cloud-services"),
            minimal_owned_stack_program("P-OWNED-STACK-DURABILITY", "durability-plane"),
            minimal_owned_stack_program("P-OWNED-STACK-GOVIAM", "governance-iam-console"),
            minimal_program("P-REORG", "reorg"),
            minimal_program("P-AST-CODE-GRAPH", "ast-code-graph"),
            minimal_program("P-FABRIC", "fabric")
        ]);
        masterplan["masterplan_v2"]["work_items"] = json!([{
            "id": "MPV2-0000",
            "program": "P-FABRIC"
        }, {
            "id": "MPV2-0001",
            "program": "P-REORG"
        }]);
        masterplan["masterplan_v2"]["program_coverage"] = json!({
            "manifest_index_ref": "/specs/microservices/manifests-index.json",
            "required_program_classes": [
                "ontology",
                "workflow-engine",
                "workflow-studio",
                "intelligence",
                "owned-stack",
                "reorg",
                "ast-code-graph",
                "fabric"
            ],
            "required_owned_stack_layers": [
                "cloud-kernel",
                "cloud-os",
                "cloud-k8s",
                "cloud-services",
                "durability-plane",
                "governance-iam-console"
            ],
            "owned_stack_ladder": {
                "doctrine_refs": ["ADR-0520", "ADR-0536", "ADR-0537"],
                "rungs": [
                    {
                        "rung": 0,
                        "layer": "cloud-kernel",
                        "program_ids": ["P-OWNED-STACK-KERNEL"],
                        "source_anchors": ["cloud/cloud-kernel"]
                    },
                    {
                        "rung": 1,
                        "layer": "cloud-os",
                        "program_ids": ["P-OWNED-STACK-OS"],
                        "source_anchors": ["cloud/cloud-os"]
                    },
                    {
                        "rung": 2,
                        "layer": "cloud-k8s",
                        "program_ids": ["P-OWNED-STACK-K8S"],
                        "source_anchors": ["k8s"]
                    },
                    {
                        "rung": 3,
                        "layer": "cloud-services",
                        "program_ids": [
                            "P-OWNED-STACK-CLOUD",
                            "P-OWNED-STACK-DURABILITY",
                            "P-OWNED-STACK-GOVIAM"
                        ],
                        "source_anchors": ["cloud"]
                    },
                    {
                        "rung": 4,
                        "layer": "products",
                        "program_ids": [
                            "P-ONTOLOGY",
                            "P-WORKFLOW-ENGINE",
                            "P-WORKFLOW-STUDIO",
                            "P-INTELLIGENCE"
                        ],
                        "source_anchors": ["oya"]
                    }
                ]
            },
            "microservices": [
                {
                    "microservice": "ontology",
                    "program_id": "P-ONTOLOGY",
                    "source_manifest": "microservices/ontology/manifest.json",
                    "coverage_status": "covered"
                },
                {
                    "microservice": "workflow-engine",
                    "program_id": "P-WORKFLOW-ENGINE",
                    "source_manifest": "microservices/workflow-engine/manifest.json",
                    "coverage_status": "covered"
                },
                {
                    "microservice": "workflow-studio",
                    "program_id": "P-WORKFLOW-STUDIO",
                    "source_manifest": "microservices/workflow-studio/manifest.json",
                    "coverage_status": "covered"
                },
                {
                    "microservice": "intelligence",
                    "program_id": "P-INTELLIGENCE",
                    "source_manifest": "microservices/intelligence/manifest.json",
                    "coverage_status": "covered"
                },
                {
                    "microservice": "retired-legacy-svc",
                    "program_id": "P-INTELLIGENCE",
                    "source_manifest": "microservices/intelligence/manifest.json",
                    "coverage_status": "retired-absorbed"
                },
                {
                    "microservice": "cloud-iac",
                    "program_id": "P-OWNED-STACK-CLOUD",
                    "source_manifest": "microservices/cloud-iac/manifest.json",
                    "coverage_status": "covered"
                }
            ]
        });
        masterplan
    }

    fn minimal_projection_freshness_masterplan() -> Value {
        let mut masterplan = minimal_masterplan_v2();
        masterplan["masterplan_v2"]["read_contracts"] = json!([
            read_contract(
                "/specs/masterplan.json",
                "entry-surface",
                "single-writer canonical authority"
            ),
            read_contract(
                "/specs/master-plan-sequencing.json",
                "provenance-archive",
                "read only as absorbed provenance; live sequencing is derived from masterplan_v2.dependency_edges"
            ),
            read_contract(
                "docs/MASTERPLAN.md",
                "on-demand",
                "compatibility projection only; conflicts resolve to /specs/masterplan.json"
            ),
            read_contract(
                "docs/ROADMAP.md",
                "provenance-archive",
                "archived historical roadmap; conflicts resolve to /specs/masterplan.json"
            ),
            read_contract(
                "docs/machine-readable/board-sync.generated.json",
                "on-demand",
                "generated planning projection; regenerate from /specs/masterplan.json through the projection freshness gate"
            ),
            read_contract(
                "docs/machine-readable/masterplan.generated.json",
                "on-demand",
                "generated planning projection; regenerate from /specs/masterplan.json through the projection freshness gate"
            )
        ]);
        masterplan["masterplan_v2"]["projection_freshness"] = json!({
            "source_of_truth": MASTERPLAN_V2_REF,
            "validator": PROJECTION_FRESHNESS_VALIDATOR,
            "single_writer_mutation_path": "/specs/masterplan.json",
            "projections": [
                projection_row("/specs/master-plan-sequencing.json", PROJECTION_CLASS_READ, "provenance-archive", "read only as absorbed provenance; live sequencing is derived from masterplan_v2.dependency_edges"),
                projection_row("docs/MASTERPLAN.md", DISPOSITION_GENERATED_PROJECTION, "on-demand", "compatibility projection only; conflicts resolve to /specs/masterplan.json"),
                projection_row("docs/ROADMAP.md", PROJECTION_CLASS_READ, "provenance-archive", "archived historical roadmap; conflicts resolve to /specs/masterplan.json"),
                projection_row("docs/machine-readable/board-sync.generated.json", DISPOSITION_GENERATED_PROJECTION, "on-demand", "generated planning projection; regenerate from /specs/masterplan.json through the projection freshness gate"),
                projection_row("docs/machine-readable/masterplan.generated.json", DISPOSITION_GENERATED_PROJECTION, "on-demand", "generated planning projection; regenerate from /specs/masterplan.json through the projection freshness gate")
            ]
        });
        masterplan
    }

    fn read_contract(path: &str, read_timing_class: &str, freshness_rule: &str) -> Value {
        json!({
            "path": path,
            "audience": ["agents", "humans", "cloud-ci-gates"],
            "read_timing_class": read_timing_class,
            "freshness_rule": freshness_rule
        })
    }

    fn projection_row(
        path: &str,
        projection_class: &str,
        read_timing_class: &str,
        freshness_rule: &str,
    ) -> Value {
        json!({
            "path": path,
            "projection_class": projection_class,
            "source_of_truth": MASTERPLAN_V2_REF,
            "freshness_gate": PROJECTION_FRESHNESS_VALIDATOR,
            "conflict_resolution": MASTERPLAN_V2_REF,
            "single_writer_mutation_path": "/specs/masterplan.json",
            "read_timing_class": read_timing_class,
            "freshness_rule": freshness_rule,
            "is_live_plan_authority": false,
            "live_work_item_ids_allowed": false,
            "status_claims_allowed_without_evidence": false,
            "drift_policy": "fail-closed-on-conflict-or-manual-live-plan-content",
            "evidence_refs": [
                "cloud/cloud-ci/gates/oya-cloud-ci-cross-artifact-agreement-app/src/lib.rs#evaluate_masterplan_v2_projection_freshness"
            ]
        })
    }

    fn minimal_generated_artifact_control_plane() -> Value {
        json!({
            "artifacts": [
                {
                    "path": "docs/machine-readable/board-sync.generated.json",
                    "source_inputs": ["docs/decisions/**", "specs/masterplan.json", "registry/**"]
                },
                {
                    "path": "docs/machine-readable/masterplan.generated.json",
                    "source_inputs": ["specs/masterplan.json", "specs/master-plan-sequencing.json", "docs/decisions/**"]
                }
            ]
        })
    }

    fn minimal_root_hub_entry_surface_allowlist() -> Value {
        json!({
            "agent_entry_surface_allowlist": {
                "read_timing_class": "entry-surface",
                "source_of_truth": ENTRY_SURFACE_ALLOWLIST_REF,
                "validator": ENTRY_SURFACE_VALIDATOR,
                "paths": [
                    "/specs/masterplan.json"
                ],
                "superseded_entrypoints": [
                    "/specs/master-plan-sequencing.json",
                    "/specs/planning-closure-contract.json",
                    "/specs/planning-closure-status-closure-ledger.json",
                    "docs/ROADMAP.md",
                    ".omc/ultragoal/goals.json"
                ]
            },
            "entry_points": {
                "masterplan": {
                    "current_path": "/specs/masterplan.json",
                    "kind": "spec",
                    "migration_phase": "masterplan-v2-single-live-plan-authority"
                },
                "master_plan_sequencing": {
                    "current_path": "/specs/master-plan-sequencing.json",
                    "kind": "spec",
                    "migration_phase": "absorbed-by-masterplan-v2-sub-ac-1",
                    "authority_status": "provenance-archive-not-live-plan-authority"
                }
            }
        })
    }

    fn minimal_program(id: &str, program_class: &str) -> Value {
        json!({
            "id": id,
            "program_class": program_class,
            "scope": "minimal test scope",
            "owner": "platform-governance",
            "hyperscaler_bar_exit_criteria": ["exit criterion"],
            "child_work_item_classes": ["definition"],
            "evidence_requirements": ["gate evidence"]
        })
    }

    fn minimal_owned_stack_program(id: &str, owned_stack_layer: &str) -> Value {
        let mut program = minimal_program(id, "owned-stack");
        program["owned_stack_layer"] = json!(owned_stack_layer);
        program
    }

    fn minimal_manifest_index() -> Value {
        json!({
            "microservices": [
                {
                    "name": "ontology",
                    "manifest": "microservices/ontology/manifest.json"
                },
                {
                    "name": "workflow-engine",
                    "manifest": "microservices/workflow-engine/manifest.json"
                },
                {
                    "name": "workflow-studio",
                    "manifest": "microservices/workflow-studio/manifest.json"
                },
                {
                    "name": "intelligence",
                    "manifest": "microservices/intelligence/manifest.json"
                },
                {
                    "name": "retired-legacy-svc",
                    "status": "retired",
                    "do_not_treat_as_active": true,
                    "manifest": "microservices/intelligence/manifest.json"
                },
                {
                    "name": "cloud-iac",
                    "manifest": "microservices/cloud-iac/manifest.json"
                }
            ]
        })
    }
    fn valid_decision() -> Value {
        json!({
            "id":"ADR-0500",
            "status":"Accepted",
            "in_spec":true,
            "in_masterplan":true,
            "in_roadmap":true
        })
    }

    fn decision_with(field: &str, value: Value) -> Value {
        let mut decision = valid_decision();
        decision[field] = value;
        decision
    }

    #[test]
    fn each_propagation_code_fires_in_isolation() {
        // orphan_decision: accepted but reaches nothing.
        assert!(evaluate(&json!({"decisions":[{"id":"ADR-1","status":"Accepted","in_spec":false,"in_masterplan":false,"in_roadmap":false}]}))
            .violations.contains("orphan_decision"));
        // unpropagated_decision: accepted, in spec, missing masterplan/roadmap.
        assert!(evaluate(&json!({"decisions":[{"id":"ADR-1","status":"Accepted","in_spec":true,"in_masterplan":false,"in_roadmap":false}]}))
            .violations.contains("unpropagated_decision"));
        // status_disagreement: ADR Accepted, roadmap face says Superseded.
        assert!(evaluate(&json!({"decisions":[{"id":"ADR-1","status":"Accepted","in_spec":true,"in_masterplan":true,"in_roadmap":true,"face_statuses":{"roadmap":"Superseded"}}]}))
            .violations.contains("status_disagreement"));
    }
}
