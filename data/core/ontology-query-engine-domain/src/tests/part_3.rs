//! Query-engine tests: part 3.

use super::support::*;

// ---- ST2: DepthCeilingExceeded error variant ----
/// ST2-a: max_depth > MAX_QUERY_DEPTH must be rejected with the new
/// DepthCeilingExceeded variant, NOT with InvalidMaxDepth.
#[test]
fn max_depth_above_ceiling_returns_depth_ceiling_exceeded_not_invalid_max_depth() {
    let result = KnowledgeGraphQueryRequest::new(
        "ten_alpha",
        "kgq_over_depth",
        "ent_root",
        Vec::<&str>::new(),
        MAX_QUERY_DEPTH + 1,
        0,
        1,
        Vec::<&str>::new(),
        TraversalDirection::Outbound,
    );
    // DepthCeilingExceeded variant must exist and be returned here
    assert_eq!(
        result,
        Err(KnowledgeGraphQueryError::DepthCeilingExceeded),
        "max_depth > MAX_QUERY_DEPTH must return DepthCeilingExceeded"
    );
}

/// ST2-b: max_depth == MAX_QUERY_DEPTH (exactly at ceiling) must be
/// accepted — Ok result, not an error.
#[test]
fn max_depth_at_ceiling_is_accepted() {
    assert!(
        KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_at_ceiling",
            "ent_root",
            Vec::<&str>::new(),
            MAX_QUERY_DEPTH,
            0,
            1,
            Vec::<&str>::new(),
            TraversalDirection::Outbound,
        )
        .is_ok(),
        "max_depth == MAX_QUERY_DEPTH must be accepted"
    );
}

/// ST2-c: max_depth == 0 must still return InvalidMaxDepth (not
/// DepthCeilingExceeded), preserving the existing structural-invalidity
/// distinction.
#[test]
fn max_depth_zero_returns_invalid_max_depth_not_depth_ceiling_exceeded() {
    assert_eq!(
        KnowledgeGraphQueryRequest::new(
            "ten_alpha",
            "kgq_zero_depth",
            "ent_root",
            Vec::<&str>::new(),
            0,
            0,
            1,
            Vec::<&str>::new(),
            TraversalDirection::Outbound,
        ),
        Err(KnowledgeGraphQueryError::InvalidMaxDepth),
        "max_depth == 0 must return InvalidMaxDepth"
    );
}

// ST1 acceptance: a malformed consent grant id (no `lty_` prefix) must be
// rejected with `MalformedConsentGrantId`.
#[test]
fn malformed_consent_grant_id_rejected() {
    let result = KnowledgeGraphQueryRequest::new(
        "ten_alpha",
        "kgq_bad_grant",
        "ent_root",
        Vec::<&str>::new(),
        1,
        0,
        1,
        vec!["bad_id"],
        TraversalDirection::Outbound,
    );
    assert_eq!(
        result,
        Err(KnowledgeGraphQueryError::MalformedConsentGrantId {
            id: "bad_id".to_string()
        }),
        "a consent grant id without the lty_ prefix must return MalformedConsentGrantId"
    );
}

// ST1 acceptance: a well-formed consent grant id (`lty_partner`) must be
// accepted without error.
#[test]
fn well_formed_consent_grant_id_accepted() {
    let result = KnowledgeGraphQueryRequest::new(
        "ten_alpha",
        "kgq_good_grant",
        "ent_root",
        Vec::<&str>::new(),
        1,
        0,
        1,
        vec!["lty_partner"],
        TraversalDirection::Outbound,
    );
    assert!(
        result.is_ok(),
        "a consent grant id with a valid lty_ prefix must be accepted"
    );
}

// ST1 acceptance: consent_filter() on a non-empty scope returns the
// expected BTreeSet of string slices.
#[test]
fn consent_filter_returns_set_of_consented_edge_type_ids() {
    let req = KnowledgeGraphQueryRequest::new(
        "ten_alpha",
        "kgq_filter_set",
        "ent_root",
        Vec::<&str>::new(),
        1,
        0,
        1,
        vec!["lty_partner", "lty_member"],
        TraversalDirection::Outbound,
    )
    .unwrap();
    let filter = req.consent_filter();
    assert!(
        filter.contains("lty_partner"),
        "consent_filter must contain lty_partner"
    );
    assert!(
        filter.contains("lty_member"),
        "consent_filter must contain lty_member"
    );
    assert!(
        !filter.contains("lty_owns"),
        "consent_filter must not contain lty_owns"
    );
}

// ST2 acceptance: when a non-empty consent scope is supplied, the BFS must
// prune edges whose edge_type_id is absent from the scope, so downstream
// nodes reachable only via those edges are absent from the response.
//
// Graph:
//   ent_root --lty_partner--> ent_b --lty_partner--> ent_c
//   ent_root --lty_member-->  ent_d
// Scope: ["lty_partner"]
// Expected: ent_b and ent_c present; ent_d absent.
//           lty_partner edges present; lty_member edge absent.
#[test]
fn consent_filter_prunes_non_consented_edges() {
    let g = consent_graph();
    let engine = consent_engine(&g);

    let req = KnowledgeGraphQueryRequest::new(
        "ten_alpha",
        "kgq_consent_prune",
        "ent_root",
        Vec::<&str>::new(),
        3,
        0,
        1,
        vec!["lty_partner"],
        TraversalDirection::Outbound,
    )
    .unwrap();

    let response = engine.query_graph_slice(&g, req).unwrap();

    let node_ids: Vec<&str> = response
        .nodes
        .iter()
        .map(|n| n.entity_id.as_str())
        .collect();
    assert!(
        node_ids.contains(&"ent_b"),
        "ent_b (reached via consented lty_partner) must be in response nodes"
    );
    assert!(
        node_ids.contains(&"ent_c"),
        "ent_c (reached via consented lty_partner hop) must be in response nodes"
    );
    assert!(
        !node_ids.contains(&"ent_d"),
        "ent_d (reachable only via non-consented lty_member) must be absent from response nodes"
    );

    let member_edges: Vec<_> = response
        .edges
        .iter()
        .filter(|e| e.edge_type_id == "lty_member")
        .collect();
    assert!(
        member_edges.is_empty(),
        "no lty_member edge must appear in response edges when lty_member is not in consent scope"
    );

    let partner_edges: Vec<_> = response
        .edges
        .iter()
        .filter(|e| e.edge_type_id == "lty_partner")
        .collect();
    assert_eq!(
        partner_edges.len(),
        2,
        "both lty_partner edges (root->b, b->c) must appear in response edges"
    );
}
