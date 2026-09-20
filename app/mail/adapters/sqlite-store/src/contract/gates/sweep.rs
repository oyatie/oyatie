//! Gate 6, store half — a cancelled upload is swept once its reservation
//! lapses, and a body a commit still links survives the same sweep. The
//! admission halves (a cancelled writer under conflict releases admission
//! within 10 s; a slow reader cannot hold admission) are
//! `protocol-imap/tests/jmap_runtime_review.rs` and `wire/idle.rs`.
use super::{RAW, at, open};
use crate::contract::suite::{Fixture, Gate, GateFailure};
use mail_api::{BlobStore, MetadataStore, Precondition, UPLOAD_RESERVATION_SECS};
use mail_kernel::Error;

pub fn run<F: Fixture>(fixture: &F, gate: &mut Gate) -> Result<(), GateFailure> {
    let store = open(fixture, &[]);
    let upload = at(gate, "cancelled upload", store.put_blob("a", RAW))?;
    let linked = at(
        gate,
        "fixture",
        store.append(
            "a",
            vec!["inbox".into()],
            b"Subject: kept\r\n\r\nx",
            vec![],
            1,
        ),
    )?;
    at(
        gate,
        "fixture",
        store.execute("a", Precondition::Observed(0), vec![linked]),
    )?;
    // Before the reservation lapses nothing is swept.
    let early = at(gate, "cancelled upload", store.orphan_sweep(0, 100))?;
    gate.check(
        "cancelled upload",
        early == 0,
        format!("{early} bodies swept inside the reservation"),
    )?;
    gate.check(
        "cancelled upload",
        store.blob("a", &upload).is_ok(),
        "the upload vanished inside its reservation",
    )?;
    // After it lapses the upload is gone and the linked body is not.
    let late = at(
        gate,
        "cancelled upload",
        store.orphan_sweep(
            mail_api::Clock.now_secs() + UPLOAD_RESERVATION_SECS + 1,
            100,
        ),
    )?;
    gate.check(
        "cancelled upload",
        late == 1,
        format!("{late} bodies swept after the reservation"),
    )?;
    let gone = store.blob("a", &upload);
    gate.check(
        "cancelled upload",
        gone == Err(Error::NotFound),
        format!("swept upload still readable: {gone:?}"),
    )?;
    let kept = at(gate, "linked body survives", store.blob("a", "e1"))?;
    gate.check(
        "linked body survives",
        kept == b"Subject: kept\r\n\r\nx",
        "the linked body was swept",
    )
}
