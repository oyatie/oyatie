//! The join, end to end: a `Check` materialises into `EntityRecord.parents`
//! and the REAL `CedarPdp` decides on it.
//!
//! The graph half runs the actual bounded userset-rewrite walk over the
//! in-memory tuple store; the policy half is the actual cedar-policy engine
//! behind the unchanged `PolicyDecisionPoint` port. Nothing is doubled.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod join_fixtures;

use std::collections::BTreeMap;

use join_fixtures::*;
use policy_cedar_domain::rebac::RebacReadSnapshot;
use policy_decision_domain::{decide, materialize_parents};
use policy_tuple_store_inmemory::InMemoryTupleStore;
use shared_platform_contracts_kernel::pdp::Decision;

#[test]
fn a_check_materialises_into_parents_and_cedar_decides_allow() {
    // Membership is NESTED - alice is in core, core is in eng - so the
    // parent edge can only come from the rewrite walk, not from reading a
    // single tuple back.
    let mut store = InMemoryTupleStore::new();
    write(&mut store, "group:eng#member@group:core#member");
    write(&mut store, "group:core#member@user:alice");

    let namespace = model();
    let candidates = [eng_candidate()];
    let inputs = graph(&store, &namespace, RebacReadSnapshot::latest(), &candidates);

    let parents =
        materialize_parents(&inputs, &request("alice")).expect("the graph walk completes");
    assert_eq!(
        parents.parents(),
        vec![entity("Group", "eng")],
        "the nested membership must materialise as exactly the eng parent edge"
    );

    let outcome = decide(
        &pdp(),
        &inputs,
        &request("alice"),
        BTreeMap::new(),
        context_entities(),
    )
    .expect("the joined decision completes");
    assert_eq!(
        outcome.outcome.response.decision,
        Decision::Allow,
        "the graph-materialised parent edge is the only possible source of this allow"
    );
}

#[test]
fn a_candidate_the_graph_refutes_materialises_nothing_and_cedar_denies() {
    // Bob appears in the candidate LIST but not in the GRAPH. If the join
    // trusted candidates instead of walking them, this would allow.
    let mut store = InMemoryTupleStore::new();
    write(&mut store, "group:eng#member@group:core#member");
    write(&mut store, "group:core#member@user:alice");

    let namespace = model();
    let candidates = [eng_candidate()];
    let inputs = graph(&store, &namespace, RebacReadSnapshot::latest(), &candidates);

    assert!(
        materialize_parents(&inputs, &request("bob"))
            .expect("the graph walk completes")
            .parents()
            .is_empty(),
        "no membership holds for bob, so nothing may materialise"
    );
    let outcome = decide(
        &pdp(),
        &inputs,
        &request("bob"),
        BTreeMap::new(),
        context_entities(),
    )
    .expect("the joined decision completes");
    assert_eq!(outcome.outcome.response.decision, Decision::Deny);
}

#[test]
fn a_snapshot_before_the_grant_decides_deny_while_latest_decides_allow() {
    // One store, two snapshots, opposite decisions: the decision is about
    // the graph AS OF the snapshot, not about the store's current state.
    let mut store = InMemoryTupleStore::new();
    let before = write(&mut store, "group:eng#member@group:core#member");
    write(&mut store, "group:core#member@user:alice");

    let namespace = model();
    let candidates = [eng_candidate()];

    let stale = graph(
        &store,
        &namespace,
        RebacReadSnapshot::at_zookie(before),
        &candidates,
    );
    let outcome = decide(
        &pdp(),
        &stale,
        &request("alice"),
        BTreeMap::new(),
        context_entities(),
    )
    .expect("the joined decision completes");
    assert_eq!(
        outcome.outcome.response.decision,
        Decision::Deny,
        "at the pre-grant snapshot alice's membership does not exist yet"
    );

    let fresh = graph(&store, &namespace, RebacReadSnapshot::latest(), &candidates);
    let outcome = decide(
        &pdp(),
        &fresh,
        &request("alice"),
        BTreeMap::new(),
        context_entities(),
    )
    .expect("the joined decision completes");
    assert_eq!(outcome.outcome.response.decision, Decision::Allow);
}

#[test]
fn an_engine_refusal_surfaces_as_pdp_not_as_a_decision() {
    // A real PdpError from the real engine: the action slug has no mapping
    // in the loaded bundle. It must surface as DecisionError::Pdp - a
    // refusal - never be conflated with a decided Deny.
    let mut store = InMemoryTupleStore::new();
    write(&mut store, "group:eng#member@user:alice");
    let namespace = model();
    let candidates = [eng_candidate()];
    let inputs = graph(&store, &namespace, RebacReadSnapshot::latest(), &candidates);

    let mut unmapped = request("alice");
    unmapped.action = "delete".to_owned();
    let refusal = decide(
        &pdp(),
        &inputs,
        &unmapped,
        BTreeMap::new(),
        context_entities(),
    );
    assert!(
        matches!(
            &refusal,
            Err(policy_decision_domain::DecisionError::Pdp(
                policy_pdp_kernel::PdpError::UnknownAction { action }
            )) if action == "delete"
        ),
        "an unmapped action is an engine refusal, not a decision: {refusal:?}"
    );
}
