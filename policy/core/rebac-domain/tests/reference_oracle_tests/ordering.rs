use crate::reference_oracle_support::cases::{TENANT, direct_model, query, tuple};
use crate::reference_oracle_support::harness::{Execution, oracle_outcome, production_outcome};
use crate::reference_oracle_support::{Model, Outcome, Refusal, Rewrite, Subject};

#[test]
fn grounded_alternative_after_a_cycle_can_allow() {
    let model = direct_model("doc", "left").define(
        "doc",
        "target",
        Rewrite::Union(vec![
            Rewrite::Computed("target".to_owned()),
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
    let query = query("alice", "doc", "one", "target");
    assert_eq!(
        production_outcome(&model, &tuples, &query, &Execution::default()),
        Outcome::Allow
    );
    assert_eq!(oracle_outcome(&model, &tuples, &query), Outcome::Allow);
}

#[test]
fn earlier_reachable_error_cannot_be_revised_away() {
    let model = Model::default()
        .define(
            "doc",
            "loop",
            Rewrite::Union(vec![
                Rewrite::Computed("target".to_owned()),
                Rewrite::Computed("missing".to_owned()),
            ]),
        )
        .define(
            "doc",
            "target",
            Rewrite::Union(vec![Rewrite::Computed("loop".to_owned()), Rewrite::This]),
        );
    let tuples = vec![tuple(
        TENANT,
        "doc",
        "one",
        "target",
        Subject::object("user", "alice"),
    )];
    let query = query("alice", "doc", "one", "target");
    let expected = Outcome::Refuse(Refusal::UnknownRelation {
        object_type: "doc".to_owned(),
        relation: "missing".to_owned(),
    });
    assert_eq!(
        production_outcome(&model, &tuples, &query, &Execution::default()),
        expected
    );
    assert_eq!(oracle_outcome(&model, &tuples, &query), expected);
}
