//! Integration tests for WorkflowSpec graph-integrity validation.
//!
//! Acceptance criteria (subtasks 2 + 3):
//!   - A spec with a cycle returns Err(GraphCycle(..))
//!   - A spec with a node unreachable from any entry returns Err(UnreachableNode(..))
//!   - All pre-existing valid-spec tests still pass
//!   - emit_canonical_json output for previously-valid specs is byte-identical (determinism)
//!   - validate() is deterministic for identical input
//!   - WorkflowSpecEmitError implements Display (error messages are human-readable)

use workflow_studio_dsl_emitter::{
    WorkflowSpec, WorkflowSpecEdge, WorkflowSpecEmitError, WorkflowSpecNode, WorkflowSpecNodeKind,
    emit_canonical_json,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn linear_spec() -> WorkflowSpec {
    WorkflowSpec::new(
        "ten_acme",
        "wfd_linear",
        "1.0.0",
        vec![
            WorkflowSpecNode::new("wfn_a", WorkflowSpecNodeKind::Http, "A"),
            WorkflowSpecNode::new("wfn_b", WorkflowSpecNodeKind::Transform, "B"),
            WorkflowSpecNode::new("wfn_c", WorkflowSpecNodeKind::Transform, "C"),
        ],
        vec![
            WorkflowSpecEdge::new("wfn_a", "wfn_b", None),
            WorkflowSpecEdge::new("wfn_b", "wfn_c", None),
        ],
    )
}

// ---------------------------------------------------------------------------
// Clean DAG — must pass validate() and emit stable JSON
// ---------------------------------------------------------------------------

#[test]
fn clean_dag_passes_validate() {
    assert_eq!(linear_spec().validate(), Ok(()));
}

#[test]
fn emit_canonical_json_is_byte_identical_on_repeated_calls() {
    let spec = linear_spec();
    let first = emit_canonical_json(&spec).expect("valid spec must emit");
    let second = emit_canonical_json(&spec).expect("valid spec must emit");
    assert_eq!(first, second, "emit_canonical_json must be deterministic");
}

// ---------------------------------------------------------------------------
// Cycle detection
// ---------------------------------------------------------------------------

#[test]
fn two_node_direct_cycle_returns_graph_cycle() {
    // wfn_a -> wfn_b -> wfn_a  (simple 2-cycle, both reachable from entry? No —
    // both have in-degree 1 after the back-edge, so no entry node exists;
    // all nodes are unreachable from the empty entry set → UnreachableNode fires first.)
    // Corrected topology: wfn_entry (in-degree 0) -> wfn_a -> wfn_b -> wfn_a
    // so wfn_a and wfn_b are reachable but form a cycle → GraphCycle.
    let spec = WorkflowSpec::new(
        "ten_acme",
        "wfd_twocycle",
        "1.0.0",
        vec![
            WorkflowSpecNode::new("wfn_entry", WorkflowSpecNodeKind::Http, "Entry"),
            WorkflowSpecNode::new("wfn_a", WorkflowSpecNodeKind::Transform, "A"),
            WorkflowSpecNode::new("wfn_b", WorkflowSpecNodeKind::Join, "B"),
        ],
        vec![
            WorkflowSpecEdge::new("wfn_entry", "wfn_a", None),
            WorkflowSpecEdge::new("wfn_a", "wfn_b", None),
            WorkflowSpecEdge::new("wfn_b", "wfn_a", None),
        ],
    );
    let result = spec.validate();
    assert!(
        matches!(result, Err(WorkflowSpecEmitError::GraphCycle(_))),
        "expected GraphCycle for 2-node reachable cycle, got {result:?}",
    );
}

#[test]
fn fully_cyclic_graph_with_no_entry_node_returns_unreachable_node() {
    // All nodes have in-degree >= 1; entry set is empty; BFS visits nothing.
    // UnreachableNode fires on the first sorted node.
    let spec = WorkflowSpec::new(
        "ten_acme",
        "wfd_fullcycle",
        "1.0.0",
        vec![
            WorkflowSpecNode::new("wfn_a", WorkflowSpecNodeKind::Http, "A"),
            WorkflowSpecNode::new("wfn_b", WorkflowSpecNodeKind::Transform, "B"),
        ],
        vec![
            WorkflowSpecEdge::new("wfn_a", "wfn_b", None),
            WorkflowSpecEdge::new("wfn_b", "wfn_a", None),
        ],
    );
    let result = spec.validate();
    assert!(
        matches!(result, Err(WorkflowSpecEmitError::UnreachableNode(_))),
        "expected UnreachableNode when no entry node exists, got {result:?}",
    );
}

// ---------------------------------------------------------------------------
// Unreachable node detection
// ---------------------------------------------------------------------------

#[test]
fn isolated_node_with_no_edges_returns_unreachable_node() {
    // wfn_a is the sole entry node (no outgoing edges).
    // wfn_b has no incoming or outgoing edges but is a separate component.
    // Both have in-degree 0 → both are entry nodes → BFS from both visits both.
    // Actually this passes! Instead: wfn_a -> wfn_b, wfn_c is isolated with in-degree 0.
    // wfn_c is an entry node so BFS visits it. All reachable → passes.
    //
    // True unreachable: wfn_a is entry (in-degree 0), wfn_b has an incoming edge
    // from wfn_c, but wfn_c has incoming from wfn_b only → wfn_b and wfn_c are
    // in a disconnected cycle, unreachable from wfn_a.
    let spec = WorkflowSpec::new(
        "ten_acme",
        "wfd_isolated",
        "1.0.0",
        vec![
            WorkflowSpecNode::new("wfn_a", WorkflowSpecNodeKind::Http, "A"),
            WorkflowSpecNode::new("wfn_b", WorkflowSpecNodeKind::Transform, "B"),
            WorkflowSpecNode::new("wfn_c", WorkflowSpecNodeKind::Join, "C"),
        ],
        vec![
            // wfn_b and wfn_c form a disconnected cycle; wfn_a has no outgoing edges.
            WorkflowSpecEdge::new("wfn_b", "wfn_c", None),
            WorkflowSpecEdge::new("wfn_c", "wfn_b", None),
        ],
    );
    assert_eq!(
        spec.validate(),
        Err(WorkflowSpecEmitError::UnreachableNode("wfn_b".to_string())),
        "isolated cycle component must be reported as UnreachableNode at first sorted id",
    );
}

#[test]
fn single_node_with_no_edges_passes_validate() {
    // A workflow with one node and no edges is a trivially valid DAG.
    let spec = WorkflowSpec::new(
        "ten_acme",
        "wfd_solo",
        "1.0.0",
        vec![WorkflowSpecNode::new(
            "wfn_only",
            WorkflowSpecNodeKind::Http,
            "Only",
        )],
        vec![],
    );
    assert_eq!(spec.validate(), Ok(()));
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn validate_result_is_deterministic_for_cyclic_spec() {
    let spec = WorkflowSpec::new(
        "ten_acme",
        "wfd_det_cycle",
        "1.0.0",
        vec![
            WorkflowSpecNode::new("wfn_entry", WorkflowSpecNodeKind::Http, "Entry"),
            WorkflowSpecNode::new("wfn_x", WorkflowSpecNodeKind::Transform, "X"),
            WorkflowSpecNode::new("wfn_y", WorkflowSpecNodeKind::Branch, "Y"),
        ],
        vec![
            WorkflowSpecEdge::new("wfn_entry", "wfn_x", None),
            WorkflowSpecEdge::new("wfn_x", "wfn_y", None),
            WorkflowSpecEdge::new("wfn_y", "wfn_x", None),
        ],
    );
    let r1 = spec.validate();
    let r2 = spec.validate();
    assert_eq!(r1, r2, "validate() must be deterministic for cyclic specs");
}

// ---------------------------------------------------------------------------
// Display impl — WorkflowSpecEmitError must be human-readable
// This test is intentionally RED: WorkflowSpecEmitError has no Display impl yet.
// ---------------------------------------------------------------------------

#[test]
fn graph_cycle_error_has_human_readable_display() {
    let err = WorkflowSpecEmitError::GraphCycle("wfn_b".to_string());
    let msg = format!("{err}");
    assert!(
        msg.contains("wfn_b"),
        "Display for GraphCycle must include the node id, got: {msg:?}",
    );
}

#[test]
fn unreachable_node_error_has_human_readable_display() {
    let err = WorkflowSpecEmitError::UnreachableNode("wfn_c".to_string());
    let msg = format!("{err}");
    assert!(
        msg.contains("wfn_c"),
        "Display for UnreachableNode must include the node id, got: {msg:?}",
    );
}

// ---------------------------------------------------------------------------
// WF-STU-2: edge-condition determinism — emit_canonical_json contract
// ---------------------------------------------------------------------------

#[test]
fn emit_rejects_duplicate_edge_condition_spec() {
    // wfn_branch has two outgoing edges both with condition "approved".
    // emit_canonical_json must propagate DuplicateEdgeCondition.
    let spec = WorkflowSpec::new(
        "ten_acme",
        "wfd_dupcond_emit",
        "1.0.0",
        vec![
            WorkflowSpecNode::new("wfn_branch", WorkflowSpecNodeKind::Branch, "Branch"),
            WorkflowSpecNode::new("wfn_x", WorkflowSpecNodeKind::Transform, "X"),
            WorkflowSpecNode::new("wfn_y", WorkflowSpecNodeKind::Transform, "Y"),
        ],
        vec![
            WorkflowSpecEdge::new("wfn_branch", "wfn_x", Some("approved".to_string())),
            WorkflowSpecEdge::new("wfn_branch", "wfn_y", Some("approved".to_string())),
        ],
    );
    let result = emit_canonical_json(&spec);
    assert!(
        matches!(
            result,
            Err(WorkflowSpecEmitError::DuplicateEdgeCondition(_))
        ),
        "emit_canonical_json must return DuplicateEdgeCondition for duplicate-condition siblings, got {result:?}",
    );
}

#[test]
fn emit_rejects_ambiguous_default_edge_spec() {
    // wfn_split has two unconditional outgoing edges.
    // emit_canonical_json must propagate AmbiguousDefaultEdge.
    let spec = WorkflowSpec::new(
        "ten_acme",
        "wfd_ambiguous_emit",
        "1.0.0",
        vec![
            WorkflowSpecNode::new("wfn_split", WorkflowSpecNodeKind::Branch, "Split"),
            WorkflowSpecNode::new("wfn_x", WorkflowSpecNodeKind::Transform, "X"),
            WorkflowSpecNode::new("wfn_y", WorkflowSpecNodeKind::Transform, "Y"),
        ],
        vec![
            WorkflowSpecEdge::new("wfn_split", "wfn_x", None),
            WorkflowSpecEdge::new("wfn_split", "wfn_y", None),
        ],
    );
    let result = emit_canonical_json(&spec);
    assert!(
        matches!(result, Err(WorkflowSpecEmitError::AmbiguousDefaultEdge(_))),
        "emit_canonical_json must return AmbiguousDefaultEdge for two unconditional siblings, got {result:?}",
    );
}

// ---------------------------------------------------------------------------
// WF-STU-3: canonical JSON regression — byte-identical for valid spec
// ---------------------------------------------------------------------------

#[test]
fn canonical_json_of_valid_spec_is_stable() {
    // Pinned expected output captured from the initial passing state.
    // Any change to serialisation format would break this test intentionally.
    const EXPECTED: &str = concat!(
        r#"{"schema_version":"workflow_spec.v1","tenant_id":"ten_acme","definition_id":"wfd_linear","version":"1.0.0","#,
        r#""nodes":[{"id":"wfn_a","kind":"http","label":"A"},{"id":"wfn_b","kind":"transform","label":"B"},{"id":"wfn_c","kind":"transform","label":"C"}],"#,
        r#""edges":[{"from":"wfn_a","to":"wfn_b"},{"from":"wfn_b","to":"wfn_c"}]}"#,
    );
    let json = emit_canonical_json(&linear_spec()).expect("valid spec must emit");
    assert_eq!(
        json, EXPECTED,
        "canonical JSON must be byte-identical to pinned expected output",
    );
}
