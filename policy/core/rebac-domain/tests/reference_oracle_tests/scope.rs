use crate::reference_oracle_support::cases::{TENANT, direct_model, query, tuple};
use crate::reference_oracle_support::harness::{
    Execution, assert_match, oracle_outcome, production_outcome,
};
use crate::reference_oracle_support::{Outcome, Refusal, Subject};

#[test]
fn pagination_tenant_scope_and_cancellation_never_return_partial_allow() {
    let model = direct_model("doc", "target");
    let query = query("alice", "doc", "one", "target");
    let paged = vec![
        tuple(
            TENANT,
            "doc",
            "one",
            "target",
            Subject::object("user", "bob"),
        ),
        tuple(
            TENANT,
            "doc",
            "one",
            "target",
            Subject::object("user", "alice"),
        ),
    ];
    let one_tuple_pages = Execution {
        page_size: 1,
        ..Execution::default()
    };
    assert_match(&model, &paged, &query, &one_tuple_pages);
    assert_eq!(oracle_outcome(&model, &paged, &query), Outcome::Allow);

    let cancelled = Execution {
        cancel_on_read: Some(2),
        ..one_tuple_pages.clone()
    };
    assert_eq!(
        production_outcome(&model, &paged, &query, &cancelled),
        Outcome::Refuse(Refusal::Cancelled),
        "a later-page cancellation cannot preserve the partial scan's value"
    );

    let mismatched_snapshot = Execution {
        snapshot_tenant: Some("tenant_b".to_owned()),
        ..Execution::default()
    };
    assert_eq!(
        production_outcome(&model, &paged, &query, &mismatched_snapshot),
        Outcome::Refuse(Refusal::TenantScope),
        "a store-issued snapshot for another tenant must refuse before traversal"
    );

    let foreign = vec![tuple(
        "tenant_b",
        "doc",
        "one",
        "target",
        Subject::object("user", "alice"),
    )];
    assert_match(&model, &foreign, &query, &Execution::default());
    assert_eq!(oracle_outcome(&model, &foreign, &query), Outcome::Deny);
}
