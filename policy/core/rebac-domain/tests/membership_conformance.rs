#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod finite_model;

use std::collections::BTreeSet;

use finite_model::families::{
    BOOLEAN_MODELS, CYCLE_MODELS, INHERITANCE_MODELS, boolean, cycle, inheritance,
};
use finite_model::store::{render, render_unavailable};
use finite_model::{Member, Model, Object, member_ids, reference};
use policy_cedar_domain::rebac::{
    RebacObjectRef, RebacReadSnapshot, RebacRelation, RebacSubjectRef, RebacTupleStoreError,
};
use policy_rebac_domain::{Expander, ExpansionError};

#[test]
fn literal_set_algebra_and_computed_memberships_anchor_the_oracle() {
    let membership = reference(&boolean(11));
    let member = BTreeSet::from(["alice", "bob"]);
    let editor = BTreeSet::from(["bob"]);

    assert_eq!(member_ids(&membership, "document", "one", "member"), member);
    assert_eq!(member_ids(&membership, "document", "one", "editor"), editor);
    assert_eq!(
        member_ids(&membership, "document", "one", "alias"),
        BTreeSet::from(["alice", "bob"]),
        "computed usersets preserve Alice's membership"
    );
    assert_eq!(
        member_ids(&membership, "document", "one", "either")
            .into_iter()
            .collect::<Vec<_>>(),
        member.union(&editor).copied().collect::<Vec<_>>()
    );
    assert_eq!(
        member_ids(&membership, "document", "one", "both")
            .into_iter()
            .collect::<Vec<_>>(),
        member.intersection(&editor).copied().collect::<Vec<_>>()
    );
    assert_eq!(
        member_ids(&membership, "document", "one", "unblocked")
            .into_iter()
            .collect::<Vec<_>>(),
        member.difference(&editor).copied().collect::<Vec<_>>()
    );
}

#[test]
fn inherited_alice_membership_is_a_literal_oracle_anchor() {
    let membership = reference(&inheritance(17));
    assert_eq!(
        member_ids(&membership, "document", "one", "viewer"),
        BTreeSet::from(["alice"])
    );
}

#[test]
fn empty_and_terminal_bearing_positive_cycles_have_literal_outcomes() {
    let empty_cycle = reference(&cycle(48));
    assert!(member_ids(&empty_cycle, "group", "a", "member").is_empty());
    assert!(member_ids(&empty_cycle, "group", "b", "member").is_empty());

    let terminal_cycle = reference(&cycle(49));
    assert_eq!(
        member_ids(&terminal_cycle, "group", "a", "member"),
        BTreeSet::from(["alice"])
    );
    assert_eq!(
        member_ids(&terminal_cycle, "group", "b", "member"),
        BTreeSet::from(["alice"])
    );
}

#[derive(Default)]
struct CorpusStats {
    models: usize,
    queries: usize,
    positive: usize,
    negative: usize,
    reads: usize,
    continuation_reads: usize,
}

fn native_object(object: &Object) -> RebacObjectRef {
    RebacObjectRef::new(object.object_type, object.object_id).expect("model object is valid")
}

fn exercise_family(family: &str, model_count: usize, build: fn(usize) -> Model) -> CorpusStats {
    let mut stats = CorpusStats::default();
    for mask in 0..model_count {
        let model = build(mask);
        let expected = reference(&model);
        stats.models += 1;
        for page_size in [1, 32] {
            let rendered = render(&model, page_size);
            for relation in &model.relations {
                for object in model
                    .objects
                    .iter()
                    .filter(|object| object.object_type == relation.object_type)
                {
                    for user in &model.users {
                        let want = expected.contains(&Member {
                            subject: user.clone(),
                            relation: relation.name,
                            object: object.clone(),
                        });
                        if page_size == 1 {
                            stats.queries += 1;
                            if want {
                                stats.positive += 1;
                            } else {
                                stats.negative += 1;
                            }
                        }
                        let subject = RebacSubjectRef::object(native_object(user));
                        let native_relation =
                            RebacRelation::new(relation.name).expect("model relation is valid");
                        let native_object = native_object(object);
                        let got = Expander::new(
                            &rendered.store,
                            &rendered.namespace,
                            rendered.tenant.clone(),
                            RebacReadSnapshot::at(rendered.snapshot.clone()),
                        )
                        .check(&subject, &native_relation, &native_object);
                        assert_eq!(
                            got,
                            Ok(want),
                            "family={family} mask={mask:#x} page_size={page_size} query={} on {}#{}",
                            user.object_id,
                            object.object_id,
                            relation.name
                        );
                    }
                }
            }
            stats.reads += rendered.store.reads();
            stats.continuation_reads += rendered.store.continuation_reads();
        }
    }
    assert!(
        stats.positive > 0,
        "{family} corpus had no positive memberships"
    );
    assert!(
        stats.negative > 0,
        "{family} corpus had no negative memberships"
    );
    assert!(
        stats.reads > 0,
        "{family} corpus never read its tuple fixture"
    );
    stats
}

#[test]
fn all_boolean_tuple_subsets_match_for_every_declared_query_and_page_size() {
    let stats = exercise_family("boolean", BOOLEAN_MODELS, boolean);
    assert_eq!(stats.models, 256);
    assert_eq!(stats.queries, 7_168);
    assert!(stats.continuation_reads > 0);
}

#[test]
fn all_positive_cycle_tuple_subsets_match_for_every_membership_query() {
    let stats = exercise_family("positive-cycle", CYCLE_MODELS, cycle);
    assert_eq!(stats.models, 64);
    assert_eq!(stats.queries, 256);
    assert!(stats.continuation_reads > 0);
}

#[test]
fn all_inheritance_tuple_subsets_match_for_every_valid_query() {
    let stats = exercise_family("inheritance", INHERITANCE_MODELS, inheritance);
    assert_eq!(stats.models, 256);
    assert_eq!(stats.queries, 3_072);
    assert!(stats.continuation_reads > 0);
}

#[test]
fn unavailable_store_is_an_exact_typed_refusal() {
    let rendered = render_unavailable(&boolean(0));
    let result = Expander::new(
        &rendered.store,
        &rendered.namespace,
        rendered.tenant.clone(),
        RebacReadSnapshot::at(rendered.snapshot.clone()),
    )
    .check(
        &RebacSubjectRef::object(
            RebacObjectRef::new("user", "alice").expect("literal subject is valid"),
        ),
        &RebacRelation::new("member").expect("literal relation is valid"),
        &RebacObjectRef::new("document", "one").expect("literal object is valid"),
    );

    assert_eq!(
        result,
        Err(ExpansionError::Store(RebacTupleStoreError::Backend(
            "finite conformance store unavailable".to_owned()
        )))
    );
}
