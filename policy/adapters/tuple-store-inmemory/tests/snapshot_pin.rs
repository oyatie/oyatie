//! The pin, tested against a store that CAN drift.
//!
//! `InMemoryTupleStore` needs `&mut self` to write and `Expander` holds `&S`,
//! so borrowck alone serialises them and no test built on it can observe the
//! pin at all. Deleting the body of `Walk::pin` left the entire suite green.
//! The port does not require that exclusion — `read_tuples` takes `&self`, so
//! any interior-mutability or shared-backend adapter is legal, and against one
//! of those an unpinned walk sees a write that landed after it began.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::cell::RefCell;

use common::*;
use policy_cedar_domain::rebac::{
    RebacReadSnapshot, RebacTuple, RebacTuplePage, RebacTupleQuery, RebacTupleStore,
    RebacTupleStoreError, Zookie,
};
use policy_rebac_domain::Expander;
use policy_tuple_store_inmemory::InMemoryTupleStore;

/// A legal adapter that writes during a read. Models a shared backend, where
/// another actor's commit lands mid-decision.
struct DriftingStore {
    inner: RefCell<InMemoryTupleStore>,
    /// Landed AFTER the first read has been served, so the pin is already
    /// taken. Landing it before the first read would be a write the decision
    /// legitimately sees, and would prove nothing.
    pending: RefCell<Option<String>>,
    reads: RefCell<usize>,
}

impl DriftingStore {
    fn new(inner: InMemoryTupleStore, lands_mid_walk: &str) -> Self {
        Self {
            inner: RefCell::new(inner),
            pending: RefCell::new(Some(lands_mid_walk.to_owned())),
            reads: RefCell::new(0),
        }
    }
}

impl RebacTupleStore for DriftingStore {
    fn write_tuple(&mut self, tuple: RebacTuple) -> Result<Zookie, RebacTupleStoreError> {
        self.inner.borrow_mut().write_tuple(tuple)
    }

    fn read_tuples(
        &self,
        query: &RebacTupleQuery,
        snapshot: RebacReadSnapshot,
    ) -> Result<RebacTuplePage, RebacTupleStoreError> {
        let served = *self.reads.borrow();
        *self.reads.borrow_mut() = served + 1;
        if served > 0
            && let Some(late) = self.pending.borrow_mut().take()
        {
            let parsed = RebacTuple::parse(tenant(), &late).expect("tuple parses");
            self.inner
                .borrow_mut()
                .write_tuple(parsed)
                .expect("the late write lands");
        }
        self.inner.borrow().read_tuples(query, snapshot)
    }
}

#[test]
fn a_write_landing_mid_walk_is_not_observed() {
    let mut seed = InMemoryTupleStore::new();
    write(&mut seed, "document:q3#parent@folder:budget");
    // alice's grant does NOT exist when the check begins; it lands during it.
    let store = DriftingStore::new(seed, "folder:budget#viewer@user:alice");

    let model = document_model();
    let expander = Expander::new(&store, &model, tenant(), RebacReadSnapshot::latest());

    assert!(
        !expander
            .check(
                &user("user:alice"),
                &relation("viewer"),
                &object("document:q3")
            )
            .expect("the walk completes"),
        "a decision must not observe a grant written after it began; without \
         the pin every read re-resolves `latest` and this returns true"
    );
}

#[test]
fn the_pin_holds_across_pages_of_one_tupleset() {
    // Page 1 takes the pin; the late write must not appear on page 2 either.
    let mut seed = InMemoryTupleStore::new().with_page_size(1);
    for filler in 0..3 {
        write(
            &mut seed,
            &format!("folder:budget#viewer@user:filler{filler}"),
        );
    }
    write(&mut seed, "document:q3#parent@folder:budget");
    let store = DriftingStore::new(seed, "folder:budget#viewer@user:alice");

    let model = document_model();
    let expander = Expander::new(&store, &model, tenant(), RebacReadSnapshot::latest());
    assert!(
        !expander
            .check(
                &user("user:alice"),
                &relation("viewer"),
                &object("document:q3")
            )
            .expect("the walk completes"),
        "later pages must read at the snapshot page one was served"
    );
}
