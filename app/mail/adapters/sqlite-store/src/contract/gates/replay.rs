//! Gate 4 — crash and replay: a delivery receipt makes the mailbox commit
//! idempotent across a worker crash and a re-claim; an ambiguous remote
//! acknowledgement settles an outbound job once — the second settlement is
//! refused by the epoch and enqueues no second notice.
use super::{RAW, at, open};
use crate::contract::suite::{Fixture, Gate, GateFailure};
use mail_api::{DeliveryOutcome, DeliveryQueue, DeliveryTarget, MetadataStore, SubmissionQueue};
use mail_kernel::Error;

pub fn run<F: Fixture>(fixture: &F, gate: &mut Gate) -> Result<(), GateFailure> {
    let store = open(fixture, &[]);
    let targets = [DeliveryTarget {
        account: "a".into(),
        address: "alice@example.org".into(),
    }];
    at(
        gate,
        "fixture",
        store.enqueue("s@example.net", &targets, RAW),
    )?;
    // Worker 1 commits the mailbox, then crashes before settling the job.
    let first = at(gate, "receipt", store.claim(1))?
        .pop()
        .ok_or_else(|| gate.store("receipt", Error::NotFound))?;
    let message = at(gate, "receipt", store.queued_message(&first))?;
    at(
        gate,
        "receipt",
        store.deliver_once("a", &first.delivery_id(), &message.raw, message.received_at),
    )?;
    let replay = store.deliver_once("a", &first.delivery_id(), &message.raw, message.received_at);
    gate.check(
        "receipt",
        replay.is_ok(),
        format!("replay of the same key: {replay:?}"),
    )?;
    let count = at(gate, "receipt", store.account("a"))?.messages.len();
    gate.check(
        "receipt",
        count == 1,
        format!("{count} messages after a replayed delivery"),
    )?;
    // The job is settled by whoever holds the live epoch, once.
    let settled = store.finish(&first, Ok(()));
    gate.check("settle once", settled.is_ok(), format!("{settled:?}"))?;
    let again = store.finish(&first, Ok(()));
    gate.check(
        "settle once",
        again == Err(Error::Conflict),
        format!("second settlement: {again:?}"),
    )?;
    gate.check(
        "settle once",
        at(gate, "settle once", store.claim(10))?.is_empty(),
        "the job was still claimable",
    )?;
    // Ambiguous remote ack: the first settlement wins; the stale one is
    // refused and no second notice is queued for the sender.
    at(
        gate,
        "fixture",
        store.enqueue_submission("a", "alice@example.org", &["one@remote.org".into()], RAW),
    )?;
    let lease = at(gate, "ambiguous ack", store.claim_outbound(1))?
        .pop()
        .ok_or_else(|| gate.store("ambiguous ack", Error::NotFound))?;
    at(
        gate,
        "ambiguous ack",
        store.finish_outbound(&lease, DeliveryOutcome::Delivered),
    )?;
    let stale = store.finish_outbound(&lease, DeliveryOutcome::Permanent(550));
    gate.check(
        "ambiguous ack",
        stale == Err(Error::Conflict),
        format!("stale settlement: {stale:?}"),
    )?;
    let notices = at(gate, "ambiguous ack", store.claim(10))?.len();
    gate.check(
        "ambiguous ack",
        notices == 0,
        format!("{notices} notices queued after a delivered job"),
    )
}
