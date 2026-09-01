//! The catch-up law: a durable projection that is behind its log can be
//! brought to the log's head, and `applied_head` means what it says.
//!
//! These pin the part most easily got wrong, and the reason the API is
//! shaped as it is: catch-up resumes INCLUSIVE of the store's own head
//! entry, so a store built from a DIFFERENT log is refused rather than
//! topped up to a head that lies. See `catchup.rs` for what that does
//! and does not prove.

use foundry_projection_draft::{MemoryProjectionStore, ProjectionStore, ProjectionStoreError};
use foundry_spine::{CatchUpError, ProjectionState, catch_up, fold_from_scratch, project_through};

#[allow(dead_code)]
mod catchup_support;
#[allow(dead_code)]
mod write_through_support;

use catchup_support::{
    TENANT, assert_agrees_with_fold, corrupt, log, registry, sealed, sealed_by_another_actor,
};
use write_through_support::FailsAt;

#[test]
fn an_empty_store_catches_up_to_the_whole_log() {
    let registry = registry();
    let log = log();
    let mut store = MemoryProjectionStore::default();

    let caught = catch_up(TENANT, &registry, &mut store, &log).expect("catch up from empty");

    assert_eq!(caught.resumed_from, 0);
    assert_eq!(caught.head, 3);
    assert!(
        !caught.revalidated,
        "an empty store has no resume point to revalidate"
    );
    assert_agrees_with_fold(&store, &fold_from_scratch(TENANT, &registry, log.iter()));
}

#[test]
fn catch_up_resumes_where_the_store_stopped_rather_than_restarting() {
    let registry = registry();
    let log = log();
    let mut store = MemoryProjectionStore::default();
    let mut state = ProjectionState::new(TENANT, &registry);
    project_through(&mut state, &mut store, &log[..2]).expect("mirror the first two");

    let caught = catch_up(TENANT, &registry, &mut store, &log).expect("resume");

    assert_eq!(
        caught.resumed_from, 2,
        "resume begins at what the store durably held"
    );
    assert_eq!(caught.head, 3);
    assert!(
        caught.revalidated,
        "the resume point was re-applied and agreed"
    );
    assert_agrees_with_fold(&store, &fold_from_scratch(TENANT, &registry, log.iter()));
}

#[test]
fn a_rebuilt_store_equals_an_incrementally_written_one() {
    let registry = registry();
    let log = log();

    let mut incremental = MemoryProjectionStore::default();
    let mut state = ProjectionState::new(TENANT, &registry);
    for sealed in &log {
        project_through(&mut state, &mut incremental, std::slice::from_ref(sealed))
            .expect("mirror one at a time");
    }

    let mut rebuilt = MemoryProjectionStore::default();
    catch_up(TENANT, &registry, &mut rebuilt, &log).expect("rebuild in one call");

    // Entry-at-a-time and all-at-once are the same function of the log,
    // or "rebuild" would be a second, subtly different projector.
    assert_eq!(
        incremental.applied_head(TENANT).unwrap(),
        rebuilt.applied_head(TENANT).unwrap()
    );
    for object_ref in state.bindings.keys() {
        assert_eq!(
            incremental.get(TENANT, object_ref).unwrap(),
            rebuilt.get(TENANT, object_ref).unwrap(),
            "{object_ref} differs between the two paths"
        );
        assert_eq!(
            incremental.links_from(TENANT, object_ref).unwrap(),
            rebuilt.links_from(TENANT, object_ref).unwrap()
        );
    }
}

#[test]
fn catch_up_reproduces_the_same_poison_on_every_rebuild() {
    let registry = registry();
    // Ordinal 2 is undecodable: a poison derived from (log bytes,
    // registry), so it must land identically however often we rebuild.
    let log = vec![sealed(1, "one"), corrupt(2), sealed(3, "three")];

    let mut first = MemoryProjectionStore::default();
    let mut second = MemoryProjectionStore::default();
    catch_up(TENANT, &registry, &mut first, &log).expect("first rebuild");
    catch_up(TENANT, &registry, &mut second, &log).expect("second rebuild");

    let poisoned = first.poisoned(TENANT).unwrap();
    assert_eq!(poisoned.len(), 1, "exactly the corrupt entry poisoned");
    assert_eq!(poisoned[0].0, 2, "and it was ordinal 2");
    assert_eq!(
        poisoned,
        second.poisoned(TENANT).unwrap(),
        "a poison is a function of the log, not of when it was folded"
    );
    assert_eq!(first.applied_head(TENANT).unwrap(), 3);
}

#[test]
fn a_store_ahead_of_its_log_is_refused() {
    let registry = registry();
    let log = log();
    let mut store = MemoryProjectionStore::default();
    catch_up(TENANT, &registry, &mut store, &log).expect("reach head 3");

    // A truncated log against a store that has seen more of it: serving
    // it would answer from rows the log can no longer justify.
    let error = catch_up(TENANT, &registry, &mut store, &log[..1]).expect_err("must refuse");

    assert_eq!(
        error,
        CatchUpError::StoreAheadOfLog {
            store_head: 3,
            log_head: 1
        }
    );
    assert_eq!(
        store.applied_head(TENANT).unwrap(),
        3,
        "and changed nothing"
    );
}

#[test]
fn a_store_outage_halts_catch_up_and_a_later_call_resumes_from_there() {
    let registry = registry();
    let log = log();
    let mut store = FailsAt {
        inner: MemoryProjectionStore::default(),
        fail_on_ordinal: 2,
        fail_head: false,
    };

    let error = catch_up(TENANT, &registry, &mut store, &log).expect_err("the store is out");
    assert!(
        matches!(error, CatchUpError::Mirror(_)),
        "an outage is infrastructure, never a poison: {error:?}"
    );
    assert_eq!(
        store.applied_head(TENANT).unwrap(),
        1,
        "entries before the outage stayed durable"
    );

    // The outage clears. Nothing was wedged.
    store.fail_on_ordinal = 0;
    let caught = catch_up(TENANT, &registry, &mut store, &log).expect("resume after the outage");

    assert_eq!(caught.resumed_from, 1);
    assert_eq!(caught.head, 3);
    assert_agrees_with_fold(&store, &fold_from_scratch(TENANT, &registry, log.iter()));
}

#[test]
fn a_store_holding_a_different_log_is_refused_rather_than_topped_up() {
    let registry = registry();
    let mine = log();
    let mut store = MemoryProjectionStore::default();
    catch_up(TENANT, &registry, &mut store, &mine).expect("reach head 3");

    // Same ordinals, same objects, same keys — one different principal
    // on the entry the store stopped at, plus a fourth entry that would
    // make the store LOOK caught up if the resume point went unchecked.
    let theirs = vec![
        sealed(1, "one"),
        sealed(2, "two"),
        sealed_by_another_actor(3, "three"),
        sealed(4, "four"),
    ];

    let error = catch_up(TENANT, &registry, &mut store, &theirs).expect_err("must refuse");

    assert_eq!(error, CatchUpError::DivergentResumePoint { ordinal: 3 });
    assert_eq!(
        store.applied_head(TENANT).unwrap(),
        3,
        "the store neither advanced nor absorbed the other log"
    );
    assert_eq!(
        store.get(TENANT, "ent_3").unwrap().unwrap().last_actor,
        "prn_alice",
        "and it still holds what its own log wrote"
    );
}

#[test]
fn catch_up_re_mirrors_nothing_the_store_already_holds() {
    let registry = registry();
    let log: Vec<_> = (1..=40)
        .map(|ordinal| sealed(ordinal, &format!("reading {ordinal}")))
        .collect();
    let mut built = MemoryProjectionStore::default();
    catch_up(TENANT, &registry, &mut built, &log[..20]).expect("reach head 20");

    // Refusing ordinal 5 — well inside the prefix the store holds.
    // Re-mirroring from ordinal 1 would trip it, so the write cost is
    // bounded by the STORE's head rather than the log's.
    let mut store = FailsAt {
        inner: built,
        fail_on_ordinal: 5,
        fail_head: false,
    };

    let caught =
        catch_up(TENANT, &registry, &mut store, &log).expect("resume without re-mirroring");

    assert_eq!(caught.resumed_from, 20);
    assert_eq!(caught.head, 40);
}

#[test]
fn a_cold_start_is_never_reported_as_divergence() {
    let registry = registry();
    // A long log against a store that has never been written: the shape
    // of a fresh deployment, and the one case that MUST NOT refuse.
    let log: Vec<_> = (1..=40)
        .map(|ordinal| sealed(ordinal, &format!("reading {ordinal}")))
        .collect();
    let mut store = MemoryProjectionStore::default();

    let caught = catch_up(TENANT, &registry, &mut store, &log).expect("a cold start must succeed");

    // A caller refusing boot on divergence keys off this distinction,
    // so it is pinned rather than left to arithmetic a future edit could
    // change: ordinals are dense from 1 and `resumed_from` is 0, so the
    // divergence arm is unreachable here by construction.
    assert_eq!(caught.resumed_from, 0);
    assert!(!caught.revalidated);
    assert_eq!(caught.head, 40, "and it caught up the whole log");
}

#[test]
fn an_unreadable_store_refuses_catch_up_instead_of_rebuilding_from_zero() {
    let registry = registry();
    let log = log();
    let mut store = FailsAt {
        inner: MemoryProjectionStore::default(),
        fail_on_ordinal: 0,
        fail_head: true,
    };

    let error = catch_up(TENANT, &registry, &mut store, &log).expect_err("must refuse");

    // Treating an unreadable head as 0 would rebuild the whole log over
    // a store whose contents are unknown — the destructive reading of
    // an infrastructure failure.
    assert!(
        matches!(
            error,
            CatchUpError::Read(ProjectionStoreError::Storage { .. })
        ),
        "{error:?}"
    );
}
