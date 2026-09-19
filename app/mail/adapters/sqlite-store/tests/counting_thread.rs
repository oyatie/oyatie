//! Gate 2 rows for threading, history and submissions: a reply into a
//! 500-message thread, CHANGEDSINCE after ten changes, a read below the
//! floor, and submission accept/cancel map rows bounded by the operation,
//! not by the account (10 000 messages cost ≤ 1.5× what 100 cost).
use mail_api::{MetadataStore, Precondition, SubmissionAcceptance, SubmissionStore};
use mail_kernel::{
    Account, Command, EnvelopeAddress, RetentionPolicy, SubmissionEnvelope, SubmissionRecord,
    UndoStatus,
};
use mail_sqlite_store::SqliteStore;
use mail_sqlite_store::contract::Counting;
use std::collections::BTreeMap;

const TOKEN: &str = "0123456789abcdef0123456789abcdef";
const THREAD: usize = 500;
const RAW: &[u8] =
    b"From: alice@example.org\r\nTo: remote@example.net\r\nSubject: queued\r\n\r\nbody\r\n";

fn message(n: usize, refs: &str, subject: &str) -> Command {
    Command::Append {
        mailboxes: vec!["inbox".into()],
        raw: format!("Message-ID: <{n}@t>\r\nReferences: {refs}\r\nSubject: {subject}\r\n\r\nx")
            .into_bytes(),
        keywords: vec![],
        received_at: n as i64,
    }
}

/// One `THREAD`-message thread rooted at `<1@t>`, then `noise` singletons.
fn fixture(noise: usize) -> Counting<SqliteStore> {
    let db = SqliteStore::open(":memory:").unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    db.deliver(&["alice@example.org".into()], RAW).unwrap();
    let mut commands = vec![message(1, "", "T")];
    commands.extend((2..=THREAD).map(|n| message(n, "<1@t>", "T")));
    db.execute("a", Precondition::Observed(0), commands)
        .unwrap();
    let mut n = THREAD;
    let mut remaining = noise;
    while remaining > 0 {
        let batch = remaining.min(500);
        let commands = (0..batch)
            .map(|_| {
                n += 1;
                message(n, "", &format!("N{n}"))
            })
            .collect();
        db.execute("a", Precondition::Observed(0), commands)
            .unwrap();
        remaining -= batch;
    }
    Counting::new(db)
}

fn gate(name: &str, small: u64, large: u64, ceiling: u64) {
    println!("counting gate {name}: rows at 100 = {small}, rows at 10000 = {large}");
    assert!(
        small <= ceiling,
        "{name}: {small} rows at 100 exceeds {ceiling}"
    );
    assert!(
        large as f64 <= 1.5 * small as f64,
        "{name}: {large} rows at 10000 vs {small} at 100"
    );
}

fn revision(store: &Counting<SqliteStore>) -> u64 {
    store.inner().account("a").unwrap().revision
}

fn reply_into_thread(store: &Counting<SqliteStore>) -> u64 {
    let revision = revision(store);
    store.reset();
    let execution = store
        .execute(
            "a",
            Precondition::Observed(revision),
            vec![message(99_999, "<1@t>", "T")],
        )
        .unwrap();
    let joined = store.messages("a", &execution.ids).unwrap();
    let root = store.messages("a", &["e2".into()]).unwrap();
    assert_eq!(
        joined.messages[0].thread_identity(),
        root.messages[0].thread_identity(),
        "the reply joined the root's thread"
    );
    store.reads()
}

fn changes_after_ten(store: &Counting<SqliteStore>) -> u64 {
    let since = revision(store);
    for n in 0..10 {
        let revision = revision(store);
        store
            .execute(
                "a",
                Precondition::Observed(revision),
                vec![Command::Keywords {
                    id: format!("e{}", 3 + n),
                    keywords: vec!["$seen".into()],
                }],
            )
            .unwrap();
    }
    store.reset();
    let page = store.history("a", since, 100).unwrap();
    assert_eq!(page.rows.len(), 10);
    assert!(!page.has_more);
    store.reads()
}

fn below_floor(store: &Counting<SqliteStore>) -> u64 {
    store
        .compact_history(
            "a",
            i64::MAX / 2,
            RetentionPolicy {
                max_age_secs: 0,
                max_rows: 1,
            },
            &[],
        )
        .unwrap();
    store.reset();
    let page = store.history("a", 0, 100).unwrap();
    assert!(page.below_floor());
    assert!(page.rows.is_empty());
    store.reads()
}

fn submission_accept_cancel(store: &Counting<SqliteStore>) -> u64 {
    let address = |email: &str| EnvelopeAddress {
        email: email.into(),
        parameters: BTreeMap::new(),
    };
    let head = store.submissions("a", None).unwrap().revision;
    let acceptance = SubmissionAcceptance {
        record: SubmissionRecord {
            id: String::new(),
            email_id: "e1".into(),
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
        email_revision: revision(store),
        raw: RAW.into(),
        allow_remote: true,
    };
    store.reset();
    let accepted = store.accept_submission("a", head, acceptance).unwrap();
    let id = accepted.records[0].id.clone();
    store
        .cancel_submission("a", accepted.revision, &id)
        .unwrap();
    store.reads()
}

#[test]
fn thread_history_and_submission_rows_depend_on_the_operation_not_on_the_account() {
    let small = fixture(100);
    let large = fixture(10_000);
    gate(
        "reply into a 500-message thread",
        reply_into_thread(&small),
        reply_into_thread(&large),
        16,
    );
    gate(
        "CHANGEDSINCE after 10 changes",
        changes_after_ten(&small),
        changes_after_ten(&large),
        16,
    );
    gate(
        "submission accept and cancel",
        submission_accept_cancel(&small),
        submission_accept_cancel(&large),
        8,
    );
    gate(
        "since below the floor",
        below_floor(&small),
        below_floor(&large),
        1,
    );
}
