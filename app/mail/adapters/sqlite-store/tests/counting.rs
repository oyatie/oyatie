//! Cost gates: the rows a mutation, a selection and a mailbox read touch
//! depend on the selection, not on the account. Plus the two fault doubles.
use mail_api::{Consumer, MetadataStore, Precondition};
use mail_kernel::{Account, Command, Error, Retention, RetentionPolicy};
use mail_sqlite_store::SqliteStore;
use mail_sqlite_store::contract::{Broken, Counting, Faulty};

const TOKEN: &str = "0123456789abcdef0123456789abcdef";
const SMALL: usize = 256;

fn append(mailbox: &str, n: usize) -> Command {
    Command::Append {
        mailboxes: vec![mailbox.into()],
        raw: format!("Subject: {n}\r\n\r\nx").into_bytes(),
        keywords: vec![],
        received_at: n as i64,
    }
}

/// Account `a` with `inbox_messages` in INBOX and `SMALL` in mailbox `m1`.
fn fixture(inbox_messages: usize) -> Counting<SqliteStore> {
    let db = SqliteStore::open(":memory:").unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    db.execute(
        "a",
        Precondition::Observed(0),
        vec![Command::CreateMailbox {
            name: "Small".into(),
        }],
    )
    .unwrap();
    let mut n = 0;
    for (mailbox, total) in [("inbox", inbox_messages), ("m1", SMALL)] {
        let mut remaining = total;
        while remaining > 0 {
            let batch = remaining.min(500);
            let commands = (0..batch)
                .map(|_| {
                    n += 1;
                    append(mailbox, n)
                })
                .collect();
            db.execute("a", Precondition::Observed(0), commands)
                .unwrap();
            remaining -= batch;
        }
    }
    let account = db.account("a").unwrap();
    assert_eq!(account.messages.len(), inbox_messages + SMALL);
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

fn flag_mutation(store: &Counting<SqliteStore>) -> u64 {
    store.reset();
    let revision = store.inner().account("a").unwrap().revision;
    store.reset();
    store
        .execute(
            "a",
            Precondition::Observed(revision),
            vec![Command::Keywords {
                id: "e2".into(),
                keywords: vec!["$seen".into()],
            }],
        )
        .unwrap();
    assert_eq!(store.calls("execute"), 1);
    store.reads()
}

fn selection(store: &Counting<SqliteStore>) -> u64 {
    store.reset();
    let ids: Vec<String> = (2..2 + SMALL).map(|n| format!("e{n}")).collect();
    let selected = store.messages("a", &ids).unwrap();
    assert_eq!(selected.messages.len(), SMALL);
    store.reads()
}

fn small_mailbox(store: &Counting<SqliteStore>) -> u64 {
    store.reset();
    let selected = store.mailbox_uids("a", "m1").unwrap();
    assert_eq!(selected.uids.len(), SMALL);
    store.reads()
}

#[test]
fn rows_read_depend_on_the_selection_not_on_the_account() {
    let small = fixture(100);
    let large = fixture(10_000);
    gate(
        "flag mutation",
        flag_mutation(&small),
        flag_mutation(&large),
        8,
    );
    gate(
        "256-message selection",
        selection(&small),
        selection(&large),
        3 * SMALL as u64,
    );
    gate(
        "256-message mailbox",
        small_mailbox(&small),
        small_mailbox(&large),
        2 * SMALL as u64,
    );
    // Each probe resets first: only the last probe's call is on the counter.
    assert_eq!(large.calls("mailbox_uids"), 1);
    assert_eq!(large.calls("messages"), 0);
    assert_eq!(large.total_calls(), 1);
}

#[test]
fn faulty_store_surfaces_the_injected_error_once_and_persists_nothing() {
    let store = Faulty::new(SqliteStore::open(":memory:").unwrap());
    store
        .inner()
        .provision(
            Account::new("a", "t", "alice", "alice@example.org").unwrap(),
            TOKEN,
        )
        .unwrap();
    store.fail_at("execute", Error::Busy);
    assert_eq!(
        store.account("a").unwrap().revision,
        0,
        "other methods pass"
    );
    assert!(store.armed());
    assert_eq!(
        store.execute("a", Precondition::Require(0), vec![append("inbox", 1)]),
        Err(Error::Busy)
    );
    assert!(!store.armed());
    assert_eq!(store.inner().account("a").unwrap().revision, 0);
    assert_eq!(store.blob("a", "e1"), Err(Error::NotFound));
    store
        .execute("a", Precondition::Require(0), vec![append("inbox", 1)])
        .unwrap();
    assert_eq!(store.account("a").unwrap().revision, 1);
    store.fail_next(Error::Unavailable);
    assert_eq!(store.account("a"), Err(Error::Unavailable));
    assert_eq!(store.account("a").unwrap().revision, 1);
    store.fail_at("history", Error::Conflict);
    store.disarm();
    assert!(store.history("a", 0, 10).is_ok());
}

#[test]
fn broken_store_violates_the_contract_where_the_suite_must_notice() {
    let inner = SqliteStore::open(":memory:").unwrap();
    inner
        .provision(
            Account::new("a", "t", "alice", "alice@example.org").unwrap(),
            TOKEN,
        )
        .unwrap();
    inner
        .execute(
            "a",
            Precondition::Require(0),
            vec![append("inbox", 1), append("inbox", 2)],
        )
        .unwrap();
    let store = Broken::new(inner);
    // A stale `Require` is re-applied instead of conflicting.
    assert_eq!(
        store.inner().execute(
            "a",
            Precondition::Require(0),
            vec![Command::CreateMailbox { name: "x".into() }]
        ),
        Err(Error::Conflict)
    );
    let execution = store
        .execute(
            "a",
            Precondition::Require(0),
            vec![Command::CreateMailbox { name: "x".into() }],
        )
        .unwrap();
    assert_eq!(execution.revision, 3);
    // A one-id selection returns the whole account.
    assert_eq!(
        store.messages("a", &["e1".into()]).unwrap().messages.len(),
        2
    );
    assert_eq!(
        store
            .inner()
            .messages("a", &["e1".into()])
            .unwrap()
            .messages
            .len(),
        1
    );
    // The mailbox map comes from the full projection but matches.
    assert_eq!(
        store.mailbox_uids("a", "inbox").unwrap(),
        store.inner().mailbox_uids("a", "inbox").unwrap()
    );
    // Consumer cursors are ignored, and a page below the floor claims completeness.
    let policy = RetentionPolicy {
        max_age_secs: i64::MAX,
        max_rows: 1,
    };
    assert_eq!(
        store.compact_history("a", 0, policy, &[(Consumer::FoundryRecords, 0)]),
        Ok(Retention::Advanced { floor: 2 })
    );
    let page = store.history("a", 0, 10).unwrap();
    assert!(!page.below_floor() && page.rows.is_empty());
    assert!(store.inner().history("a", 0, 10).unwrap().below_floor());
}
