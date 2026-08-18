//! RED-phase tests for structural validation slice (WF-STU-1 / WF-STU-2 / WF-STU-3).
//!
//! These tests cover acceptance-criteria gaps not yet addressed by
//! graph_integrity.rs:
//!
//!  WF-STU-1 additions
//!   - Multiple unreachable nodes are reported deterministically (first sorted id)
//!   - A node reachable via multiple paths (diamond DAG) still passes
//!   - The unreachable check fires before the cycle check when a partially-reachable
//!     cycle is also present in the same spec
//!
//!  WF-STU-2 additions
//!   - When multiple source nodes each have duplicate conditions the first sorted
//!     source node id is reported
//!   - DuplicateEdgeCondition is reported before AmbiguousDefaultEdge when both
//!     violations are present on the same source node
//!   - Three outgoing conditional edges where exactly one pair duplicates yields
//!     DuplicateEdgeCondition (not a false positive on the non-duplicate edge)
//!   - Two different conditions on the same source node is valid (no false positive)
//!
//!  WF-STU-3 additions
//!   - emit_canonical_json propagates UnreachableNode (WF-STU-1 variant via emit)
//!   - canonicalized() directly rejects a spec failing WF-STU-1 with UnreachableNode
//!   - canonicalized() directly rejects a spec failing WF-STU-2 with
//!     DuplicateEdgeCondition
//!   - A spec valid under all new checks round-trips through canonicalized() +
//!     re-parse without loss
//!
//! These tests are intentionally written against the EXISTING implementation to verify
//! all new acceptance criteria are met. If any test here fails, the implementation
//! has a gap that must be fixed in the GREEN stage.

use workflow_studio_dsl_emitter::{
    WorkflowSpec, WorkflowSpecEdge, WorkflowSpecEmitError, WorkflowSpecNode, WorkflowSpecNodeKind,
    emit_canonical_json,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn node(id: &str, label: &str) -> WorkflowSpecNode {
    WorkflowSpecNode::new(id, WorkflowSpecNodeKind::Transform, label)
}

fn edge(from: &str, to: &str) -> WorkflowSpecEdge {
    WorkflowSpecEdge::new(from, to, None)
}

fn cond_edge(from: &str, to: &str, condition: &str) -> WorkflowSpecEdge {
    WorkflowSpecEdge::new(from, to, Some(condition.to_string()))
}

fn spec(def_id: &str, nodes: Vec<WorkflowSpecNode>, edges: Vec<WorkflowSpecEdge>) -> WorkflowSpec {
    WorkflowSpec::new("ten_acme", def_id, "1.0.0", nodes, edges)
}

// ---------------------------------------------------------------------------
// WF-STU-1: unreachable-node ordering guarantee
// ---------------------------------------------------------------------------

/// When multiple nodes are unreachable, validate() reports the one with the
/// lexicographically smallest id (BTreeSet iteration order is sorted ascending).
#[test]
fn multiple_unreachable_nodes_reports_first_sorted_id() {
    // wfn_entry -> wfn_z  (only wfn_z reachable from the entry)
    // wfn_b and wfn_c are isolated with no incoming edges from entry, forming no
    // connection at all. They are separate components with in-degree 0, so BFS
    // starts from wfn_b, wfn_c, AND wfn_entry (all have in-degree 0).
    //
    // For a true unreachable case: all three must be reachable EXCEPT the targets.
    // Topology: wfn_entry is sole entry, wfn_b and wfn_c form a 2-cycle.
    // BFS from {wfn_entry} visits only wfn_entry (no outgoing edges).
    // First unreachable by sorted id: wfn_b < wfn_c.
    let s = spec(
        "wfd_multi_unreachable",
        vec![
            node("wfn_b", "B"),
            node("wfn_c", "C"),
            node("wfn_entry", "Entry"),
        ],
        vec![edge("wfn_b", "wfn_c"), edge("wfn_c", "wfn_b")],
    );
    assert_eq!(
        s.validate(),
        Err(WorkflowSpecEmitError::UnreachableNode("wfn_b".to_string())),
        "first unreachable node must be reported by lexicographic sort order",
    );
}

/// A diamond DAG (fan-out then fan-in) is a valid spec: the join node is
/// reachable via two distinct paths but must not be double-counted or rejected.
#[test]
fn diamond_dag_with_fan_out_and_fan_in_passes_validate() {
    // wfn_root -> wfn_left  -> wfn_join
    // wfn_root -> wfn_right -> wfn_join
    let s = spec(
        "wfd_diamond",
        vec![
            node("wfn_join", "Join"),
            node("wfn_left", "Left"),
            node("wfn_right", "Right"),
            node("wfn_root", "Root"),
        ],
        vec![
            cond_edge("wfn_root", "wfn_left", "left"),
            cond_edge("wfn_root", "wfn_right", "right"),
            edge("wfn_left", "wfn_join"),
            edge("wfn_right", "wfn_join"),
        ],
    );
    assert_eq!(s.validate(), Ok(()), "diamond DAG must pass validate()",);
}

/// UnreachableNode is reported before GraphCycle when the same spec contains
/// both a reachable cycle and an unreachable component.
#[test]
fn unreachable_node_reported_before_graph_cycle_when_both_present() {
    // wfn_entry -> wfn_a -> wfn_b -> wfn_a  (reachable cycle)
    // wfn_orphan is isolated (in-degree 0, no path from entry? No — in-degree 0
    // means it IS an entry node, so BFS visits it. Need a true unreachable.
    //
    // Use: wfn_entry is the only entry; wfn_a and wfn_b form a reachable cycle;
    // wfn_x and wfn_y form an isolated cycle (both in-degree >= 1, not reachable).
    // BFS visits wfn_entry, wfn_a, wfn_b. Unreachable: wfn_x, wfn_y.
    // First sorted unreachable: wfn_x < wfn_y.
    let s = spec(
        "wfd_both_violations",
        vec![
            node("wfn_a", "A"),
            node("wfn_b", "B"),
            node("wfn_entry", "Entry"),
            node("wfn_x", "X"),
            node("wfn_y", "Y"),
        ],
        vec![
            edge("wfn_entry", "wfn_a"),
            edge("wfn_a", "wfn_b"),
            edge("wfn_b", "wfn_a"), // reachable cycle
            edge("wfn_x", "wfn_y"),
            edge("wfn_y", "wfn_x"), // isolated cycle
        ],
    );
    assert_eq!(
        s.validate(),
        Err(WorkflowSpecEmitError::UnreachableNode("wfn_x".to_string())),
        "UnreachableNode must fire before GraphCycle",
    );
}

// ---------------------------------------------------------------------------
// WF-STU-2: edge-condition determinism — ordering and edge cases
// ---------------------------------------------------------------------------

/// When multiple source nodes each have duplicate conditions, the error is
/// reported for the lexicographically first source node id.
#[test]
fn duplicate_edge_condition_reports_first_sorted_source_node() {
    // Both wfn_m and wfn_n have duplicate "ok" conditions on their outgoing edges.
    // wfn_m < wfn_n lexicographically → DuplicateEdgeCondition("wfn_m").
    let s = spec(
        "wfd_multi_dup_cond",
        vec![
            node("wfn_entry", "Entry"),
            node("wfn_m", "M"),
            node("wfn_n", "N"),
            node("wfn_p", "P"),
            node("wfn_q", "Q"),
            node("wfn_r", "R"),
            node("wfn_s", "S"),
        ],
        vec![
            edge("wfn_entry", "wfn_m"),
            edge("wfn_entry", "wfn_n"),
            cond_edge("wfn_m", "wfn_p", "ok"),
            cond_edge("wfn_m", "wfn_q", "ok"), // duplicate on wfn_m
            cond_edge("wfn_n", "wfn_r", "ok"),
            cond_edge("wfn_n", "wfn_s", "ok"), // duplicate on wfn_n
        ],
    );
    assert_eq!(
        s.validate(),
        Err(WorkflowSpecEmitError::DuplicateEdgeCondition(
            "wfn_m".to_string()
        )),
        "first sorted source node with duplicate condition must be reported",
    );
}

/// When both DuplicateEdgeCondition AND AmbiguousDefaultEdge violations are
/// present on the same source node, DuplicateEdgeCondition is reported first
/// (duplicate-condition check precedes default-edge check).
#[test]
fn duplicate_condition_reported_before_ambiguous_default_on_same_node() {
    // wfn_branch has:
    //   - two "ok" conditional outgoing edges (DuplicateEdgeCondition)
    //   - two unconditional outgoing edges (AmbiguousDefaultEdge)
    // Spec requires duplicate-condition checked first.
    let s = spec(
        "wfd_both_edge_violations",
        vec![
            node("wfn_branch", "Branch"),
            node("wfn_p", "P"),
            node("wfn_q", "Q"),
            node("wfn_r", "R"),
            node("wfn_s", "S"),
        ],
        vec![
            cond_edge("wfn_branch", "wfn_p", "ok"),
            cond_edge("wfn_branch", "wfn_q", "ok"), // duplicate condition
            edge("wfn_branch", "wfn_r"),
            edge("wfn_branch", "wfn_s"), // second unconditional
        ],
    );
    assert_eq!(
        s.validate(),
        Err(WorkflowSpecEmitError::DuplicateEdgeCondition(
            "wfn_branch".to_string()
        )),
        "DuplicateEdgeCondition must be reported before AmbiguousDefaultEdge",
    );
}

/// Three outgoing conditional edges where exactly one pair shares a condition
/// yields DuplicateEdgeCondition and does not falsely reject the non-duplicate edge.
#[test]
fn three_conditional_edges_with_one_duplicate_pair_yields_duplicate_condition() {
    // wfn_branch -> wfn_p (cond "a")
    // wfn_branch -> wfn_q (cond "b")
    // wfn_branch -> wfn_r (cond "a")  ← duplicate of first edge
    let s = spec(
        "wfd_three_cond_one_dup",
        vec![
            node("wfn_branch", "Branch"),
            node("wfn_p", "P"),
            node("wfn_q", "Q"),
            node("wfn_r", "R"),
        ],
        vec![
            cond_edge("wfn_branch", "wfn_p", "a"),
            cond_edge("wfn_branch", "wfn_q", "b"),
            cond_edge("wfn_branch", "wfn_r", "a"), // duplicates first
        ],
    );
    assert_eq!(
        s.validate(),
        Err(WorkflowSpecEmitError::DuplicateEdgeCondition(
            "wfn_branch".to_string()
        )),
        "one duplicate among three conditional edges must be caught",
    );
}

/// Two outgoing edges from the same node with DISTINCT conditions is valid
/// (no false positive on different-condition siblings).
#[test]
fn two_distinct_conditions_from_same_node_passes_validate() {
    let s = spec(
        "wfd_two_distinct_cond",
        vec![
            node("wfn_branch", "Branch"),
            node("wfn_x", "X"),
            node("wfn_y", "Y"),
        ],
        vec![
            cond_edge("wfn_branch", "wfn_x", "approved"),
            cond_edge("wfn_branch", "wfn_y", "rejected"),
        ],
    );
    assert_eq!(
        s.validate(),
        Ok(()),
        "two distinct conditions from the same node must not trigger DuplicateEdgeCondition",
    );
}

// ---------------------------------------------------------------------------
// WF-STU-3: emit_canonical_json + canonicalized() propagate new errors
// ---------------------------------------------------------------------------

/// emit_canonical_json propagates UnreachableNode when the spec has an
/// unreachable component (WF-STU-1 error path through emit).
#[test]
fn emit_canonical_json_propagates_unreachable_node_error() {
    // wfn_entry has no outgoing edges; wfn_b and wfn_c form an isolated cycle.
    let s = spec(
        "wfd_emit_unreachable",
        vec![
            node("wfn_b", "B"),
            node("wfn_c", "C"),
            node("wfn_entry", "Entry"),
        ],
        vec![edge("wfn_b", "wfn_c"), edge("wfn_c", "wfn_b")],
    );
    let result = emit_canonical_json(&s);
    assert!(
        matches!(result, Err(WorkflowSpecEmitError::UnreachableNode(_))),
        "emit_canonical_json must return UnreachableNode for unreachable component, got {result:?}",
    );
}

/// canonicalized() directly rejects a spec that fails WF-STU-1 (UnreachableNode).
#[test]
fn canonicalized_rejects_spec_with_unreachable_node() {
    let s = spec(
        "wfd_canon_unreachable",
        vec![
            node("wfn_b", "B"),
            node("wfn_c", "C"),
            node("wfn_entry", "Entry"),
        ],
        vec![edge("wfn_b", "wfn_c"), edge("wfn_c", "wfn_b")],
    );
    assert!(
        matches!(
            s.canonicalized(),
            Err(WorkflowSpecEmitError::UnreachableNode(_))
        ),
        "canonicalized() must propagate UnreachableNode",
    );
}

/// canonicalized() directly rejects a spec that fails WF-STU-2 (DuplicateEdgeCondition).
#[test]
fn canonicalized_rejects_spec_with_duplicate_edge_condition() {
    let s = spec(
        "wfd_canon_dup_cond",
        vec![
            node("wfn_branch", "Branch"),
            node("wfn_x", "X"),
            node("wfn_y", "Y"),
        ],
        vec![
            cond_edge("wfn_branch", "wfn_x", "ok"),
            cond_edge("wfn_branch", "wfn_y", "ok"),
        ],
    );
    assert!(
        matches!(
            s.canonicalized(),
            Err(WorkflowSpecEmitError::DuplicateEdgeCondition(_))
        ),
        "canonicalized() must propagate DuplicateEdgeCondition",
    );
}

/// A spec valid under all new checks round-trips through canonicalized() and
/// re-parses without loss (serde round-trip stability).
#[test]
fn valid_spec_round_trips_through_canonicalized_and_serde() {
    // Multi-branch DAG that exercises all node kinds and both conditional
    // and unconditional edges.
    let s = WorkflowSpec::new(
        "ten_acme",
        "wfd_roundtrip",
        "2.3.1",
        vec![
            WorkflowSpecNode::new("wfn_cap", WorkflowSpecNodeKind::CapabilityCall, "Cap"),
            WorkflowSpecNode::new("wfn_end", WorkflowSpecNodeKind::Join, "End"),
            WorkflowSpecNode::new("wfn_review", WorkflowSpecNodeKind::HumanReview, "Review"),
            WorkflowSpecNode::new("wfn_start", WorkflowSpecNodeKind::Http, "Start"),
        ],
        vec![
            WorkflowSpecEdge::new("wfn_start", "wfn_cap", Some("trigger".to_string())),
            WorkflowSpecEdge::new("wfn_start", "wfn_review", None),
            WorkflowSpecEdge::new("wfn_cap", "wfn_end", None),
            WorkflowSpecEdge::new("wfn_review", "wfn_end", None),
        ],
    );
    let canonical = s.canonicalized().expect("valid spec must canonicalize");
    let json = serde_json::to_string(&canonical).expect("valid canonical must serialize");
    let reparsed: WorkflowSpec = serde_json::from_str(&json).expect("canonical JSON must re-parse");
    assert_eq!(
        canonical, reparsed,
        "canonicalized spec must round-trip through serde without loss",
    );
}
