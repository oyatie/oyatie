use crate::reference_oracle_support::cases::{TENANT, direct_model, query, tuple};
use crate::reference_oracle_support::harness::{
    Execution, oracle_outcome, production_outcome, production_session_outcomes,
};
use crate::reference_oracle_support::{Bounds, Outcome, Refusal, Rewrite, Subject};

#[test]
fn mutation_controls_cover_every_bound_at_and_over_the_limit() {
    tuple_and_page_boundaries();
    depth_boundary();
    candidate_and_cumulative_tuple_boundaries();
}

fn tuple_and_page_boundaries() {
    let model = direct_model("doc", "target");
    let query = query("alice", "doc", "one", "target");
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
            "target",
            Subject::object("user", "bob"),
        ),
    ];
    assert_eq!(oracle_outcome(&model, &tuples, &query), Outcome::Allow);

    let tuple_at_limit = Execution {
        bounds: Bounds {
            tuples: 2,
            ..Bounds::GENEROUS
        },
        page_size: 2,
        ..Execution::default()
    };
    assert_eq!(
        production_outcome(&model, &tuples, &query, &tuple_at_limit),
        Outcome::Allow
    );
    let tuple_over_limit = Execution {
        bounds: Bounds {
            tuples: 1,
            ..Bounds::GENEROUS
        },
        ..tuple_at_limit.clone()
    };
    assert_eq!(
        production_outcome(&model, &tuples, &query, &tuple_over_limit),
        Outcome::Refuse(Refusal::TupleBudgetExceeded(1))
    );

    let page_at_limit = Execution {
        bounds: Bounds {
            pages: 2,
            ..Bounds::GENEROUS
        },
        page_size: 1,
        ..Execution::default()
    };
    assert_eq!(
        production_outcome(&model, &tuples, &query, &page_at_limit),
        Outcome::Allow
    );
    let page_over_limit = Execution {
        bounds: Bounds {
            pages: 1,
            ..Bounds::GENEROUS
        },
        ..page_at_limit
    };
    assert_eq!(
        production_outcome(&model, &tuples, &query, &page_over_limit),
        Outcome::Refuse(Refusal::PageBudgetExceeded(1))
    );
}

fn depth_boundary() {
    let model =
        direct_model("doc", "left").define("doc", "target", Rewrite::Computed("left".to_owned()));
    let query = query("alice", "doc", "one", "target");
    let tuples = vec![tuple(
        TENANT,
        "doc",
        "one",
        "left",
        Subject::object("user", "alice"),
    )];
    assert_eq!(oracle_outcome(&model, &tuples, &query), Outcome::Allow);
    let at_limit = Execution {
        bounds: Bounds {
            depth: 1,
            ..Bounds::GENEROUS
        },
        ..Execution::default()
    };
    assert_eq!(
        production_outcome(&model, &tuples, &query, &at_limit),
        Outcome::Allow
    );
    let over_limit = Execution {
        bounds: Bounds {
            depth: 0,
            ..Bounds::GENEROUS
        },
        ..at_limit
    };
    assert_eq!(
        production_outcome(&model, &tuples, &query, &over_limit),
        Outcome::Refuse(Refusal::DepthExceeded(0))
    );
}

fn candidate_and_cumulative_tuple_boundaries() {
    let model = direct_model("doc", "target");
    let query = query("alice", "doc", "one", "target");
    let tuples = vec![tuple(
        TENANT,
        "doc",
        "one",
        "target",
        Subject::object("user", "alice"),
    )];
    assert_eq!(oracle_outcome(&model, &tuples, &query), Outcome::Allow);

    let candidate_bounds = Bounds {
        candidates: 2,
        ..Bounds::GENEROUS
    };
    assert_eq!(
        production_session_outcomes(&model, &tuples, &query, candidate_bounds, 3),
        vec![
            Outcome::Allow,
            Outcome::Allow,
            Outcome::Refuse(Refusal::CandidateBudgetExceeded(2)),
        ]
    );

    let cumulative_tuple_bounds = Bounds {
        candidates: 2,
        tuples: 1,
        ..Bounds::GENEROUS
    };
    assert_eq!(
        production_session_outcomes(&model, &tuples, &query, cumulative_tuple_bounds, 2),
        vec![
            Outcome::Allow,
            Outcome::Refuse(Refusal::TupleBudgetExceeded(1)),
        ],
        "the second candidate must not reset total tuple spend"
    );
}
