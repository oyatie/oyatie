//! Gate 2 — the rows an operation maps depend on the operation, not on the
//! account: every probe at 10 000 messages costs ≤ 1.5× what it costs at
//! 100, and stays under an absolute ceiling. No whole-account key.
use super::{append, at, open};
use crate::contract::Counting;
use crate::contract::suite::{Fixture, Gate, GateFailure};
use mail_api::{
    BlobStore, MetadataStore, Precondition, Store, SubmissionAcceptance, SubmissionStore,
};
use mail_kernel::{
    Command, EnvelopeAddress, Error, HistoryEntry, RetentionPolicy, SubmissionEnvelope,
    SubmissionRecord, UndoStatus,
};
use std::collections::BTreeMap;

const SMALL: usize = 256;
const THREAD: usize = 500;
type Probe<S> = fn(&Counting<S>) -> Result<u64, Error>;

/// Account `a`: a `THREAD`-message thread rooted at `<1@t>` plus `noise`
/// singletons in INBOX, and `SMALL` messages in a second mailbox.
fn fixture<F: Fixture>(fixture: &F, noise: usize) -> Result<Counting<F::Store>, Error> {
    let db = open(fixture, &[]);
    db.execute(
        "a",
        Precondition::Observed(0),
        vec![Command::CreateMailbox {
            name: "Small".into(),
        }],
    )?;
    let mut commands = vec![append(&db, "inbox", 1, "", "T")?];
    for n in 2..=THREAD {
        commands.push(append(&db, "inbox", n, "<1@t>", "T")?);
    }
    db.execute("a", Precondition::Observed(0), commands)?;
    let mut n = THREAD;
    for (mailbox, total) in [("inbox", noise), ("m1", SMALL)] {
        let mut remaining = total;
        while remaining > 0 {
            let batch = remaining.min(500);
            let mut commands = Vec::with_capacity(batch);
            for _ in 0..batch {
                n += 1;
                commands.push(append(&db, mailbox, n, "", &format!("N{n}"))?);
            }
            db.execute("a", Precondition::Observed(0), commands)?;
            remaining -= batch;
        }
    }
    Ok(Counting::new(db))
}

fn revision<S: Store>(store: &Counting<S>) -> Result<u64, Error> {
    Ok(store.inner().account("a")?.revision)
}

fn flag_mutation<S: Store>(store: &Counting<S>) -> Result<u64, Error> {
    let revision = revision(store)?;
    store.reset();
    store.execute(
        "a",
        Precondition::Observed(revision),
        vec![Command::Keywords {
            id: "e2".into(),
            keywords: vec!["$seen".into()],
        }],
    )?;
    if store.calls("execute") != 1 {
        return Err(Error::Invalid);
    }
    Ok(store.reads())
}

fn selection<S: Store>(store: &Counting<S>) -> Result<u64, Error> {
    store.reset();
    let ids: Vec<String> = (2..2 + SMALL).map(|n| format!("e{n}")).collect();
    let selected = store.messages("a", &ids)?;
    if selected.messages.len() != SMALL {
        return Err(Error::Invalid);
    }
    Ok(store.reads())
}

fn small_mailbox<S: Store>(store: &Counting<S>) -> Result<u64, Error> {
    store.reset();
    let selected = store.mailbox_uids("a", "m1")?;
    // One call of the selection's own method and nothing else.
    if selected.uids.len() != SMALL || store.total_calls() != 1 || store.calls("messages") != 0 {
        return Err(Error::Invalid);
    }
    Ok(store.reads())
}

fn reply_into_thread<S: Store>(store: &Counting<S>) -> Result<u64, Error> {
    let revision = revision(store)?;
    let reply = append(store.inner(), "inbox", 99_999, "<1@t>", "T")?;
    store.reset();
    let execution = store.execute("a", Precondition::Observed(revision), vec![reply])?;
    let joined = store.messages("a", &execution.ids)?;
    // Ids are one sequence: the second mailbox took 1, the root is e2.
    let root = store.messages("a", &["e2".into()])?;
    let (joined, root) = (
        joined.messages.first().ok_or(Error::NotFound)?,
        root.messages.first().ok_or(Error::NotFound)?,
    );
    if joined.thread_identity() != root.thread_identity() {
        return Err(Error::Invalid);
    }
    Ok(store.reads())
}

fn changes_after_ten<S: Store>(store: &Counting<S>) -> Result<u64, Error> {
    let since = revision(store)?;
    for n in 0..10 {
        let revision = revision(store)?;
        store.execute(
            "a",
            Precondition::Observed(revision),
            vec![Command::Keywords {
                id: format!("e{}", 3 + n),
                keywords: vec!["$seen".into()],
            }],
        )?;
    }
    store.reset();
    let page = store.history("a", since, 100)?;
    if page.rows.len() != 10 || page.has_more {
        return Err(Error::Invalid);
    }
    Ok(store.reads())
}

/// A second link (JMAP copy into a second mailbox), unlink-one-of-two, and
/// the last unlink: three rows, each classified from the history alone.
fn link_classification<S: Store>(store: &Counting<S>) -> Result<u64, Error> {
    let since = revision(store)?;
    store.reset();
    let copy = Command::SetMailboxes {
        id: "e3".into(),
        mailboxes: vec!["inbox".into(), "m1".into()],
    };
    store.execute("a", Precondition::Observed(since), vec![copy])?;
    let one_of_two = Command::SetMailboxes {
        id: "e3".into(),
        mailboxes: vec!["m1".into()],
    };
    store.execute("a", Precondition::Observed(since + 1), vec![one_of_two])?;
    let last = Command::Destroy { id: "e3".into() };
    store.execute("a", Precondition::Observed(since + 2), vec![last])?;
    let reads = store.reads();
    let kinds: Vec<&str> = store
        .history("a", since, 100)?
        .rows
        .iter()
        .filter_map(|(_, entry)| match entry {
            HistoryEntry::Added { mailbox, .. } if mailbox == "m1" => Some("copied"),
            HistoryEntry::Removed { mailbox, .. } if mailbox == "inbox" => Some("unlinked"),
            HistoryEntry::Removed { mailbox, .. } if mailbox == "m1" => Some("destroyed"),
            HistoryEntry::Added { .. } | HistoryEntry::Removed { .. } => Some("other"),
            _ => None,
        })
        .collect();
    if kinds != ["copied", "unlinked", "destroyed"] {
        return Err(Error::Invalid);
    }
    Ok(reads)
}

fn submission_accept_cancel<S: Store>(store: &Counting<S>) -> Result<u64, Error> {
    let address = |email: &str| EnvelopeAddress {
        email: email.into(),
        parameters: BTreeMap::new(),
    };
    let head = store.submissions("a", None)?.revision;
    let acceptance = SubmissionAcceptance {
        record: SubmissionRecord {
            id: String::new(),
            email_id: "e2".into(),
            identity_id: "a".into(),
            thread_id: String::new(),
            envelope: SubmissionEnvelope {
                mail_from: address("alice@example.org"),
                rcpt_to: vec![address("remote@example.net")],
            },
            send_at: i64::MAX / 4,
            undo_status: UndoStatus::Pending,
            delivery_status: BTreeMap::new(),
        },
        email_revision: revision(store)?,
        raw: b"From: alice@example.org\r\nSubject: q\r\n\r\nbody\r\n".into(),
        allow_remote: true,
    };
    store.reset();
    let accepted = store
        .accept_submission("a", head, acceptance)
        .map_err(|_| Error::Invalid)?;
    let id = accepted.records.first().ok_or(Error::NotFound)?.id.clone();
    store
        .cancel_submission("a", accepted.revision, &id)
        .map_err(|_| Error::Invalid)?;
    Ok(store.reads())
}

/// Runs last: it compacts the account.
fn below_floor<S: Store>(store: &Counting<S>) -> Result<u64, Error> {
    let policy = RetentionPolicy {
        max_age_secs: 0,
        max_rows: 1,
    };
    store.compact_history("a", i64::MAX / 2, policy, &[])?;
    store.reset();
    let page = store.history("a", 0, 100)?;
    if !page.below_floor() || !page.rows.is_empty() {
        return Err(Error::Invalid);
    }
    Ok(store.reads())
}

pub fn run<F: Fixture>(fixture: &F, gate: &mut Gate) -> Result<(), GateFailure> {
    let small = at(gate, "fixture", self::fixture(fixture, 100))?;
    let large = at(gate, "fixture", self::fixture(fixture, 10_000))?;
    let probes: [(&'static str, Probe<F::Store>, u64); 8] = [
        ("flag mutation", flag_mutation, 8),
        ("256-message selection", selection, 3 * SMALL as u64),
        ("256-message mailbox", small_mailbox, 2 * SMALL as u64),
        ("reply into a 500-message thread", reply_into_thread, 16),
        ("CHANGEDSINCE after 10 changes", changes_after_ten, 16),
        (
            "COPY, unlink one of two, last unlink",
            link_classification,
            24,
        ),
        ("submission accept and cancel", submission_accept_cancel, 8),
        ("since below the floor", below_floor, 1),
    ];
    for (case, probe, ceiling) in probes {
        let at_100 = at(gate, case, probe(&small))?;
        let at_10k = at(gate, case, probe(&large))?;
        gate.measure(case, at_10k);
        gate.check(
            case,
            at_100 <= ceiling,
            format!("{at_100} rows at 100 messages exceeds the ceiling {ceiling}"),
        )?;
        gate.check(
            case,
            at_10k as f64 <= 1.5 * at_100 as f64,
            format!("{at_10k} rows at 10 000 messages vs {at_100} at 100"),
        )?;
    }
    Ok(())
}
