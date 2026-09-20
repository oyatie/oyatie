//! The two fault doubles behave as documented: `Faulty` fails once and
//! persists nothing; `Broken` violates the contract exactly where the suite
//! (`tests/suite.rs`) must notice. Gate 2's row counts live in the suite.
use mail_api::{BlobStore, Consumer, MetadataStore, Precondition};
use mail_kernel::{Account, Command, Error, Retention, RetentionPolicy};
use mail_sqlite_store::SqliteStore;
use mail_sqlite_store::contract::{Broken, Counting, Faulty};

const TOKEN: &str = "0123456789abcdef0123456789abcdef";
const SMALL: usize = 256;

fn append(db: &SqliteStore, mailbox: &str, n: usize) -> Command {
    db.append(
        "a",
        vec![mailbox.into()],
        format!("Subject: {n}\r\n\r\nx").as_bytes(),
        vec![],
        n as i64,
    )
    .unwrap()
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
        store.execute(
            "a",
            Precondition::Require(0),
            vec![append(store.inner(), "inbox", 1)]
        ),
        Err(Error::Busy)
    );
    assert!(!store.armed());
    assert_eq!(store.inner().account("a").unwrap().revision, 0);
    assert_eq!(store.blob("a", "e1"), Err(Error::NotFound));
    store
        .execute(
            "a",
            Precondition::Require(0),
            vec![append(store.inner(), "inbox", 1)],
        )
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
            vec![append(&inner, "inbox", 1), append(&inner, "inbox", 2)],
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
