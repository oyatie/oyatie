//! An expansion failure is a refusal, and the engine is never consulted.
//!
//! The one place a double appears in the join suite: it exists to prove a
//! negative - that no path builds a slice from a partially walked graph.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod join_fixtures;

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};

use join_fixtures::*;
use policy_cedar_domain::rebac::RebacReadSnapshot;
use policy_decision_domain::{DecisionError, MembershipCandidate, PolicyDecisionPoint, decide};
use policy_pdp_kernel::{EntitySlice, PdpError, PdpOutcome};
use policy_tuple_store_inmemory::InMemoryTupleStore;
use shared_platform_contracts_kernel::pdp::{AuthorizationRequest, PolicyVersion};

/// Refuses every call; proves a code path never consulted the engine.
struct MustNotDecide(AtomicBool);

impl PolicyDecisionPoint for MustNotDecide {
    fn authorize(
        &self,
        _request: &AuthorizationRequest,
        _entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        self.0.store(true, Ordering::SeqCst);
        Err(PdpError::Evaluation {
            detail: "the engine must never be consulted on this path".to_owned(),
        })
    }

    fn loaded_policy_version(&self) -> PolicyVersion {
        PolicyVersion::new("psv-join-000001").expect("valid version token")
    }
}

#[test]
fn an_expansion_failure_is_a_refusal_and_the_engine_is_never_consulted() {
    // The candidate names a relation the model does not define. That is an
    // ExpansionError - the graph was NOT fully walked - and it must surface
    // as a refusal without the engine ever seeing a slice built from a
    // partial answer.
    let store = InMemoryTupleStore::new();
    let namespace = model();
    let candidates = [MembershipCandidate {
        object: object("group:eng"),
        relation: relation("undefined_relation"),
        parent: entity("Group", "eng"),
    }];
    let inputs = graph(&store, &namespace, RebacReadSnapshot::latest(), &candidates);

    let engine = MustNotDecide(AtomicBool::new(false));
    let refusal = decide(
        &engine,
        &inputs,
        &request("alice"),
        BTreeMap::new(),
        context_entities(),
    );
    assert!(
        matches!(refusal, Err(DecisionError::Expansion(_))),
        "an unwalked graph is a refusal, not a decision: {refusal:?}"
    );
    assert!(
        !engine.0.load(Ordering::SeqCst),
        "expansion failed, so the engine must never have been consulted"
    );
}

#[test]
fn an_invalid_slice_is_a_refusal_and_the_engine_is_never_consulted() {
    // A realistic PEP mistake: a context entity whose uid duplicates the
    // principal's own. The slice contract refuses duplicates - and that
    // refusal must surface as InvalidSlice with the engine unreached, not
    // be swallowed on the way to authorize.
    let mut store = InMemoryTupleStore::new();
    write(&mut store, "group:eng#member@user:alice");
    let namespace = model();
    let candidates = [eng_candidate()];
    let inputs = graph(&store, &namespace, RebacReadSnapshot::latest(), &candidates);

    let mut entities = context_entities();
    entities.push(policy_pdp_kernel::EntityRecord {
        uid: request("alice").principal,
        attributes: BTreeMap::new(),
        parents: Vec::new(),
    });

    let engine = MustNotDecide(AtomicBool::new(false));
    let refusal = decide(
        &engine,
        &inputs,
        &request("alice"),
        BTreeMap::new(),
        entities,
    );
    assert!(
        matches!(refusal, Err(DecisionError::InvalidSlice(_))),
        "a duplicate uid violates the slice contract and must refuse: {refusal:?}"
    );
    assert!(
        !engine.0.load(Ordering::SeqCst),
        "the slice never validated, so the engine must never have been consulted"
    );
}

#[test]
fn a_failure_on_the_second_candidate_aborts_the_whole_materialisation() {
    // The first candidate HOLDS in the graph; the second names an undefined
    // relation. A partial answer - keep the first parent, skip the second -
    // is an answer about a graph that was never fully consulted, and must
    // refuse instead.
    let mut store = InMemoryTupleStore::new();
    write(&mut store, "group:eng#member@user:alice");
    let namespace = model();
    let candidates = [
        eng_candidate(),
        MembershipCandidate {
            object: object("group:ops"),
            relation: relation("undefined_relation"),
            parent: entity("Group", "ops"),
        },
    ];
    let inputs = graph(&store, &namespace, RebacReadSnapshot::latest(), &candidates);

    let engine = MustNotDecide(AtomicBool::new(false));
    let refusal = decide(
        &engine,
        &inputs,
        &request("alice"),
        BTreeMap::new(),
        context_entities(),
    );
    assert!(
        matches!(refusal, Err(DecisionError::Expansion(_))),
        "one unwalkable candidate must refuse the whole decision: {refusal:?}"
    );
    assert!(!engine.0.load(Ordering::SeqCst));
}
