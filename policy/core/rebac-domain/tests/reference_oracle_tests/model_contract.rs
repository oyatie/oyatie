use crate::reference_oracle_support::cases::{TENANT, direct_model, query, tuple};
use crate::reference_oracle_support::harness::{Execution, assert_match, oracle_outcome};
use crate::reference_oracle_support::{Model, Outcome, Rewrite, Subject};

#[test]
fn tuple_to_userset_reads_raw_tuples_without_a_tupleset_definition() {
    let model = direct_model("folder", "member").define(
        "doc",
        "target",
        Rewrite::TupleToUserset {
            tupleset: "parent".to_owned(),
            computed: "member".to_owned(),
        },
    );
    let tuples = vec![
        tuple(
            TENANT,
            "doc",
            "one",
            "parent",
            Subject::object("folder", "one"),
        ),
        tuple(
            TENANT,
            "folder",
            "one",
            "member",
            Subject::object("user", "alice"),
        ),
    ];
    assert_match(
        &model,
        &tuples,
        &query("alice", "doc", "one", "target"),
        &Execution::default(),
    );
    assert_eq!(
        oracle_outcome(&model, &tuples, &query("alice", "doc", "one", "target")),
        Outcome::Allow
    );
}

#[test]
fn mutation_control_preserves_last_definition_wins() {
    let query = query("alice", "doc", "one", "target");
    let direct_tuple = vec![tuple(
        TENANT,
        "doc",
        "one",
        "target",
        Subject::object("user", "alice"),
    )];
    let computed_wins = Model::default()
        .define("doc", "target", Rewrite::This)
        .define("doc", "left", Rewrite::This)
        .define("doc", "target", Rewrite::Computed("left".to_owned()));
    assert_eq!(
        oracle_outcome(&computed_wins, &direct_tuple, &query),
        Outcome::Deny
    );
    assert_match(&computed_wins, &direct_tuple, &query, &Execution::default());

    let direct_wins = Model::default()
        .define("doc", "left", Rewrite::This)
        .define("doc", "target", Rewrite::Computed("left".to_owned()))
        .define("doc", "target", Rewrite::This);
    assert_eq!(
        oracle_outcome(&direct_wins, &direct_tuple, &query),
        Outcome::Allow
    );
    assert_match(&direct_wins, &direct_tuple, &query, &Execution::default());
}
