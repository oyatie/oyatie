use crate::reference_oracle_support::cases::{TENANT, direct_model, query, tuple};
use crate::reference_oracle_support::harness::{Execution, oracle_outcome, production_outcome};
use crate::reference_oracle_support::{Model, Outcome, Refusal, Rewrite, Subject};

#[test]
fn mutation_controls_cover_unknown_paths_and_negation_transitions() {
    reachable_and_short_circuited_unknowns();
    positive_cycles_do_not_hide_reachable_unknowns();
    stratified_negation_grounds_lower_relations();
    model_and_data_negation_cycles();
    positive_cycles_reach_the_least_fixed_point();
}

fn reachable_and_short_circuited_unknowns() {
    let query = query("alice", "doc", "one", "target");
    let reachable =
        Model::default().define("doc", "target", Rewrite::Computed("missing".to_owned()));
    let expected = Outcome::Refuse(Refusal::UnknownRelation {
        object_type: "doc".to_owned(),
        relation: "missing".to_owned(),
    });
    assert_eq!(oracle_outcome(&reachable, &[], &query), expected);
    assert_eq!(
        production_outcome(&reachable, &[], &query, &Execution::default()),
        expected
    );

    let short_circuited = direct_model("doc", "left").define(
        "doc",
        "target",
        Rewrite::Union(vec![
            Rewrite::Computed("left".to_owned()),
            Rewrite::Computed("missing".to_owned()),
        ]),
    );
    let tuples = vec![tuple(
        TENANT,
        "doc",
        "one",
        "left",
        Subject::object("user", "alice"),
    )];
    assert_eq!(
        oracle_outcome(&short_circuited, &tuples, &query),
        Outcome::Allow
    );
    assert_eq!(
        production_outcome(&short_circuited, &tuples, &query, &Execution::default()),
        Outcome::Allow,
        "an unreachable unknown branch must not erase a prior grant"
    );
}

fn positive_cycles_do_not_hide_reachable_unknowns() {
    let query = query("alice", "doc", "one", "target");
    let model = Model::default().define(
        "doc",
        "target",
        Rewrite::Union(vec![
            Rewrite::Computed("target".to_owned()),
            Rewrite::Computed("missing".to_owned()),
        ]),
    );
    let expected = Outcome::Refuse(Refusal::UnknownRelation {
        object_type: "doc".to_owned(),
        relation: "missing".to_owned(),
    });
    assert_eq!(
        production_outcome(&model, &[], &query, &Execution::default()),
        expected
    );
    assert_eq!(
        oracle_outcome(&model, &[], &query),
        expected,
        "cyclic self-support cannot short-circuit a reachable refusal"
    );
}

fn stratified_negation_grounds_lower_relations() {
    let query = query("alice", "doc", "one", "target");
    let tuples = vec![tuple(
        TENANT,
        "doc",
        "one",
        "target",
        Subject::object("user", "alice"),
    )];
    let model = Model::default()
        .define("doc", "loop", Rewrite::Computed("loop".to_owned()))
        .define(
            "doc",
            "target",
            Rewrite::Difference(
                Box::new(Rewrite::This),
                Box::new(Rewrite::Computed("loop".to_owned())),
            ),
        );
    assert_eq!(oracle_outcome(&model, &tuples, &query), Outcome::Allow);
    assert_eq!(
        production_outcome(&model, &tuples, &query, &Execution::default()),
        Outcome::Allow
    );

    let masked_unknown = Model::default()
        .define(
            "doc",
            "loop",
            Rewrite::Union(vec![
                Rewrite::Computed("loop".to_owned()),
                Rewrite::Computed("missing".to_owned()),
            ]),
        )
        .define(
            "doc",
            "target",
            Rewrite::Difference(
                Box::new(Rewrite::This),
                Box::new(Rewrite::Computed("loop".to_owned())),
            ),
        );
    let expected = Outcome::Refuse(Refusal::UnknownRelation {
        object_type: "doc".to_owned(),
        relation: "missing".to_owned(),
    });
    assert_eq!(oracle_outcome(&masked_unknown, &tuples, &query), expected);
    assert_eq!(
        production_outcome(&masked_unknown, &tuples, &query, &Execution::default()),
        expected
    );
}

fn model_and_data_negation_cycles() {
    let query = query("alice", "doc", "one", "target");
    let non_stratified = Model::default()
        .define("doc", "banned", Rewrite::Computed("target".to_owned()))
        .define(
            "doc",
            "target",
            Rewrite::Difference(
                Box::new(Rewrite::This),
                Box::new(Rewrite::Computed("banned".to_owned())),
            ),
        );
    let expected = Outcome::Refuse(Refusal::NonStratified {
        object_type: "doc".to_owned(),
        relation: "banned".to_owned(),
    });
    assert_eq!(oracle_outcome(&non_stratified, &[], &query), expected);
    assert_eq!(
        production_outcome(&non_stratified, &[], &query, &Execution::default()),
        expected
    );

    let data_cycle = direct_model("doc", "banned").define(
        "doc",
        "target",
        Rewrite::Difference(
            Box::new(Rewrite::This),
            Box::new(Rewrite::Computed("banned".to_owned())),
        ),
    );
    let tuples = vec![
        tuple(
            TENANT,
            "doc",
            "one",
            "target",
            Subject::object("user", "alice"),
        ),
        tuple(
            TENANT,
            "doc",
            "one",
            "banned",
            Subject::userset("doc", "one", "target"),
        ),
    ];
    let expected = Outcome::Refuse(Refusal::NegatedCycleInData {
        object_type: "doc".to_owned(),
        relation: "target".to_owned(),
    });
    assert_eq!(oracle_outcome(&data_cycle, &tuples, &query), expected);
    assert_eq!(
        production_outcome(&data_cycle, &tuples, &query, &Execution::default()),
        expected
    );
}

fn positive_cycles_reach_the_least_fixed_point() {
    let model = direct_model("group", "member");
    let tuples = vec![
        tuple(
            TENANT,
            "group",
            "a",
            "member",
            Subject::userset("group", "b", "member"),
        ),
        tuple(
            TENANT,
            "group",
            "b",
            "member",
            Subject::userset("group", "a", "member"),
        ),
    ];
    let query = query("alice", "group", "a", "member");
    assert_eq!(oracle_outcome(&model, &tuples, &query), Outcome::Deny);
    assert_eq!(
        production_outcome(&model, &tuples, &query, &Execution::default()),
        Outcome::Deny
    );
}
