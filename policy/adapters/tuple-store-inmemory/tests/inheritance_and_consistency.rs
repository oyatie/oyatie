//! The two properties that make this a Zanzibar engine rather than a lookup:
//! a relation inherited through another object, and an answer pinned to a
//! point in time.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use common::*;
use policy_cedar_domain::rebac::RebacReadSnapshot;
use policy_rebac_domain::{Expander, ExpansionBounds, ExpansionError};
use policy_tuple_store_inmemory::InMemoryTupleStore;

#[test]
fn viewer_is_inherited_through_the_parent_folder() {
    let mut store = InMemoryTupleStore::new();
    write(&mut store, "folder:budget#viewer@user:alice");
    write(&mut store, "document:q3#parent@folder:budget");

    let model = document_model();
    let expander = Expander::new(&store, &model, tenant(), RebacReadSnapshot::latest());

    // alice holds no tuple on the document at all; the grant is the folder's.
    assert!(
        expander
            .check(
                &user("user:alice"),
                &relation("viewer"),
                &object("document:q3")
            )
            .expect("the walk completes"),
        "document#viewer must follow document#parent into folder#viewer"
    );
    assert!(
        !expander
            .check(
                &user("user:bob"),
                &relation("viewer"),
                &object("document:q3")
            )
            .expect("the walk completes"),
        "a subject with no path to the document must be denied"
    );
}

#[test]
fn a_snapshot_taken_before_the_grant_cannot_see_it() {
    let mut store = InMemoryTupleStore::new();
    write(&mut store, "document:q3#parent@folder:budget");
    let before = store.head().expect("head zookie");
    let after = write(&mut store, "folder:budget#viewer@user:alice");

    let model = document_model();

    let earlier = Expander::new(&store, &model, tenant(), at(before));
    assert!(
        !earlier
            .check(
                &user("user:alice"),
                &relation("viewer"),
                &object("document:q3")
            )
            .expect("the walk completes"),
        "a read pinned before the grant must not observe it"
    );

    let later = Expander::new(&store, &model, tenant(), at(after));
    assert!(
        later
            .check(
                &user("user:alice"),
                &relation("viewer"),
                &object("document:q3")
            )
            .expect("the walk completes"),
        "a read pinned at the grant must observe it"
    );
}

#[test]
fn a_grant_on_a_later_page_is_still_found() {
    // A reader that stopped at the first page would deny this, and would look
    // correct on any fixture small enough to fit one page.
    let mut store = InMemoryTupleStore::new().with_page_size(1);
    for filler in 0..5 {
        write(
            &mut store,
            &format!("folder:budget#viewer@user:filler{filler}"),
        );
    }
    write(&mut store, "folder:budget#viewer@user:alice");
    write(&mut store, "document:q3#parent@folder:budget");

    let model = document_model();
    let expander = Expander::new(&store, &model, tenant(), RebacReadSnapshot::latest());
    assert!(
        expander
            .check(
                &user("user:alice"),
                &relation("viewer"),
                &object("document:q3")
            )
            .expect("the walk completes"),
        "the walk must page through the whole tupleset"
    );
}

#[test]
fn an_unbounded_tupleset_refuses_rather_than_truncating() {
    let mut store = InMemoryTupleStore::new().with_page_size(1);
    for filler in 0..8 {
        write(
            &mut store,
            &format!("folder:budget#viewer@user:filler{filler}"),
        );
    }
    let model = document_model();
    let bounds = ExpansionBounds {
        max_pages_per_tupleset: 2,
        ..ExpansionBounds::DEFAULT
    };
    let expander =
        Expander::new(&store, &model, tenant(), RebacReadSnapshot::latest()).with_bounds(bounds);

    assert_eq!(
        expander.check(
            &user("user:alice"),
            &relation("viewer"),
            &object("folder:budget")
        ),
        Err(ExpansionError::PageBudgetExceeded { limit: 2 }),
        "exceeding a bound must refuse, never answer from a partial read"
    );
}
