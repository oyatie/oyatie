//! Re-porting a MOVED upstream, which is the property a continuously maintained port rests on.
//!
//! Everything else in this crate proves the engine is deterministic: the same source twice gives
//! the same bytes, and bytes that change with no axis to account for them are RED. Neither says
//! anything about the case that actually happens in service — upstream releases a new version, the
//! engine re-runs, and the output is different because the SOURCE is different.
//!
//! That case has to come back GREEN and it has to come back EXPLAINED. Green because nothing is
//! wrong; explained because the receipt has to name what changed. An engine that reported
//! `Unexplained` every time a dependency moved would be telling its operator that the engine is
//! broken, over and over, until nobody read the signal again — and the signal is the only thing
//! standing between a maintained port and a fork nobody dares regenerate.
//!
//! The pair is a real second EXTRACTION of one package at two versions, at the same unit id, with
//! the two changes a dependency bump actually makes: a body changed and a declaration appeared. It
//! is not a hand-edited receipt, because a hand-edited receipt would prove something about the
//! edit.

use std::collections::BTreeSet;

use port_engine_api::ReceiptAxis;
use port_engine_app::driver;
use port_engine_kernel::{Delta, Verdict, verify};

/// A moved upstream is GREEN and EXPLAINED, and the axis that explains it is the source.
#[test]
fn a_moved_upstream_re_ports_green_and_explained() {
    let before = driver::port_go_drift_before().expect("the earlier version must port");
    let after = driver::port_go_drift_after().expect("the later version must port");

    let verification = verify(
        &before.receipt,
        &before.emitted,
        &after.receipt,
        &after.emitted,
    );

    assert_eq!(
        verification.verdict,
        Verdict::Green,
        "a dependency bump is not a defect: {verification:?}"
    );
    match verification.delta {
        Delta::Explained { regions, axes } => {
            assert!(
                !regions.is_empty(),
                "the changed regions must be named, not merely counted"
            );
            // EXACTLY the source axis. The engine, the rules, the toolchain and the formatter are
            // the same run of the same code across the pair, so any other axis moving would mean
            // the receipt is describing something other than what changed.
            assert_eq!(
                axes,
                BTreeSet::from([ReceiptAxis::Snapshot]),
                "only the source moved, so only the source axis may explain it"
            );
        }
        other => panic!("a moved upstream must be Explained, got {other:?}"),
    }
}

/// The re-port carries the upstream's change into the emitted crate, rather than merely differing.
///
/// `Explained` is about the RECEIPT. This is about the output: an engine could satisfy the delta
/// check while emitting something unrelated to what upstream actually did, and the two claims are
/// worth separating because only one of them is what a maintainer cares about.
#[test]
fn a_moved_upstream_carries_its_change_into_the_emit() {
    let before = driver::assemble_modules(
        &driver::port_go_drift_before().expect("the earlier version must port"),
    );
    let after = driver::assemble_modules(
        &driver::port_go_drift_after().expect("the later version must port"),
    );

    assert!(before.contains("value * 2"), "the earlier body:\n{before}");
    assert!(after.contains("value * 3"), "the changed body:\n{after}");
    assert!(
        !before.contains("fn offset") && after.contains("pub fn offset(value: i64) -> i64"),
        "a declaration that appeared upstream must appear in the port:\n{after}"
    );
}

/// Re-porting the SAME version twice is still `Unchanged`, so the drift signal means something.
///
/// A check that reported `Explained` for every re-run would pass the test above while being
/// useless: the point is that the engine distinguishes a moved upstream from a still one.
#[test]
fn re_porting_a_still_upstream_stays_unchanged() {
    let first = driver::port_go_drift_after().expect("the later version must port");
    let second = driver::port_go_drift_after().expect("the later version must port again");

    let verification = verify(
        &first.receipt,
        &first.emitted,
        &second.receipt,
        &second.emitted,
    );
    assert_eq!(verification.verdict, Verdict::Green);
    assert_eq!(verification.delta, Delta::Unchanged);
}
