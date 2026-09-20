//! Gate 1 is the suite itself green with the oracles unchanged; Gates 2–6
//! are the modules here. Protocol-level rows (two-writer STORE, 20 msg/s,
//! 100 sessions, admission release under cancel, slow readers) live in
//! `protocol-imap/tests` and are cited by the suite's acceptance notes.
use super::suite::{AccountSpec, Fixture, Gate, GateFailure};
use mail_api::{BlobStore, Consumer};
use mail_kernel::Command;

pub(crate) mod cost;
pub(crate) mod feed;
pub(crate) mod feed_retention;
pub(crate) mod fence;
pub(crate) mod replay;
pub(crate) mod sweep;

pub(super) const ALICE: AccountSpec<'static> = ("a", "t", "alice@example.org");
pub(super) const RAW: &[u8] = b"Subject: gate\r\n\r\nbody";

/// A message body persisted for account `a`, ready to append.
pub(super) fn append<S: BlobStore>(
    store: &S,
    mailbox: &str,
    n: usize,
    refs: &str,
    subject: &str,
) -> Result<Command, mail_kernel::Error> {
    store.append(
        "a",
        vec![mailbox.into()],
        format!("Message-ID: <{n}@t>\r\nReferences: {refs}\r\nSubject: {subject}\r\n\r\nx")
            .as_bytes(),
        vec![],
        n as i64,
    )
}

pub(super) fn open<F: Fixture>(fixture: &F, consumers: &[Consumer]) -> F::Store {
    fixture.open(consumers, &[ALICE])
}

/// Report a gate's store error as its failure.
pub(super) fn at<T>(
    gate: &Gate,
    case: &'static str,
    result: Result<T, mail_kernel::Error>,
) -> Result<T, GateFailure> {
    result.map_err(|error| gate.store(case, error))
}
