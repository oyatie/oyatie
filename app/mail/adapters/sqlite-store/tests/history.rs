//! `history/{revision}` paging, consumer-bounded compaction, and the SMTP
//! duplicate suppression `deliver_once` applies on the way into INBOX.
use mail_api::{BlobStore, Consumer, MetadataStore, Precondition};
use mail_kernel::{
    Account, Command, Error, HistoryEntry, MailboxProperties, Retention, RetentionPolicy,
};
use mail_sqlite_store::SqliteStore;

const TOKEN: &str = "0123456789abcdef0123456789abcdef";

fn append(db: &SqliteStore, n: usize) -> Command {
    db.append(
        "a",
        vec!["inbox".into()],
        format!("Subject: {n}\r\n\r\nx").as_bytes(),
        vec![],
        n as i64,
    )
    .unwrap()
}

/// Revisions 1..=3 append e1..e3 in one batch (three `Added` rows at 3),
/// 4 flags e1, 5 destroys e2, 6 creates mailbox m6: six rows in four commits.
fn fixture() -> SqliteStore {
    let db = SqliteStore::open(":memory:").unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    let batches = [
        vec![append(&db, 1), append(&db, 2), append(&db, 3)],
        vec![Command::Keywords {
            id: "e1".into(),
            keywords: vec!["$seen".into()],
        }],
        vec![Command::Destroy { id: "e2".into() }],
        vec![Command::CreateMailbox {
            name: "Later".into(),
        }],
    ];
    for (n, commands) in batches.into_iter().enumerate() {
        let execution = db
            .execute("a", Precondition::Observed(0), commands)
            .unwrap();
        assert_eq!(execution.revision, [3, 4, 5, 6][n]);
    }
    db
}

fn rows_policy(max_rows: u64) -> RetentionPolicy {
    RetentionPolicy {
        max_age_secs: i64::MAX,
        max_rows,
    }
}

#[test]
fn history_pages_whole_revisions_and_rejects_a_future_cursor() {
    let db = fixture();
    // The first revision alone holds three rows: a two-row page cannot
    // complete it.
    assert_eq!(db.history("a", 0, 2), Err(Error::OverQuota));
    let first = db.history("a", 0, 3).unwrap();
    assert_eq!((first.since, first.revision, first.floor), (0, 3, 0));
    assert!(first.has_more);
    assert_eq!(first.rows.len(), 3);
    assert!(first.rows.iter().all(|(revision, entry)| {
        *revision == 3 && matches!(entry, HistoryEntry::Added { mailbox, .. } if mailbox == "inbox")
    }));
    let rest = db.history("a", first.revision, 10).unwrap();
    assert!(!rest.has_more);
    assert_eq!(rest.revision, 6);
    assert_eq!(
        rest.rows,
        [
            (4, HistoryEntry::Flags { id: "e1".into() }),
            (
                5,
                HistoryEntry::Removed {
                    id: "e2".into(),
                    mailbox: "inbox".into(),
                    uid: 2,
                    thread: "e2".into(),
                }
            ),
            (6, HistoryEntry::Mailbox { id: "m6".into() }),
        ]
    );
    let current = db.history("a", 6, 10).unwrap();
    assert!(current.rows.is_empty() && !current.has_more && !current.below_floor());
    assert_eq!(db.history("a", 7, 10), Err(Error::Conflict));
    assert_eq!(db.history("nobody", 0, 10), Err(Error::NotFound));
}

#[test]
fn compaction_by_rows_advances_the_floor_and_deletes_the_rows_below_it() {
    let db = fixture();
    // Keep two rows: 6 and 5 stay, 4 and the three at 3 go.
    assert_eq!(
        db.compact_history("a", 0, rows_policy(2), &[]),
        Ok(Retention::Advanced { floor: 4 })
    );
    assert_eq!(db.account("a").unwrap().history_floor, 4);
    let below = db.history("a", 3, 10).unwrap();
    assert!(below.below_floor() && below.rows.is_empty());
    assert_eq!((below.floor, below.revision), (4, 6));
    let above = db.history("a", 4, 10).unwrap();
    assert_eq!(above.rows.len(), 2);
    assert_eq!(above.rows[0].0, 5);
    // Nothing more to do at the same policy; a looser one never lowers it.
    assert_eq!(
        db.compact_history("a", 0, rows_policy(2), &[]),
        Ok(Retention::Advanced { floor: 4 })
    );
    assert_eq!(
        db.compact_history("a", 0, rows_policy(1000), &[]),
        Ok(Retention::Advanced { floor: 4 })
    );
}

#[test]
fn compaction_by_age_uses_the_commit_time() {
    let db = fixture();
    let policy = RetentionPolicy {
        max_age_secs: 0,
        max_rows: 1000,
    };
    // Every commit happened before `now`: everything is old.
    assert_eq!(
        db.compact_history("a", 4_000_000_000, policy, &[]),
        Ok(Retention::Advanced { floor: 6 })
    );
    assert!(db.history("a", 0, 10).unwrap().below_floor());
    assert!(db.history("a", 6, 10).unwrap().rows.is_empty());
    // Age alone, with a generous window, retains everything.
    let db = fixture();
    let generous = RetentionPolicy {
        max_age_secs: 3600,
        max_rows: 1000,
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    assert_eq!(
        db.compact_history("a", now, generous, &[]),
        Ok(Retention::Advanced { floor: 0 })
    );
    assert_eq!(db.history("a", 0, 10).unwrap().rows.len(), 6);
}

#[test]
fn a_consumer_cursor_below_the_wanted_floor_holds_the_floor_at_the_cursor() {
    let db = fixture();
    assert_eq!(
        db.compact_history("a", 0, rows_policy(2), &[(Consumer::FoundryRecords, 3)]),
        Ok(Retention::Blocked {
            floor: 3,
            wanted: 4,
            consumer: Consumer::FoundryRecords.name().to_owned(),
        })
    );
    assert_eq!(db.account("a").unwrap().history_floor, 3);
    assert!(db.history("a", 2, 10).unwrap().below_floor());
    let page = db.history("a", 3, 10).unwrap();
    assert_eq!(page.rows.len(), 3, "rows above the held floor stay");
    // The consumer catches up: the policy floor is reached.
    assert_eq!(
        db.compact_history("a", 0, rows_policy(2), &[(Consumer::FoundryRecords, 6)]),
        Ok(Retention::Advanced { floor: 4 })
    );
    assert_eq!(db.history("a", 4, 10).unwrap().rows.len(), 2);
}

const RAW: &[u8] = b"Message-ID: <one@example.org>\r\nSubject: one\r\n\r\nbody";

fn mailbox_of(db: &SqliteStore, message: &str) -> Vec<String> {
    let account = db.account("a").unwrap();
    let message = account.messages.iter().find(|m| m.id == message).unwrap();
    message.mailboxes.keys().cloned().collect()
}

fn set_mailbox(db: &SqliteStore, message: &str, mailbox: &str) {
    db.execute(
        "a",
        Precondition::Observed(0),
        vec![Command::SetMailboxes {
            id: message.into(),
            mailboxes: vec![mailbox.into()],
        }],
    )
    .unwrap();
}

#[test]
fn deliver_once_suppresses_a_duplicate_only_in_inbox_or_junk() {
    let path = std::env::temp_dir().join(format!("mail-dedup-{}.sqlite", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let db = SqliteStore::open(&path).unwrap();
    let sql = rusqlite::Connection::open(&path).unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    let mut junk = MailboxProperties::named("Junk".into());
    junk.role = Some("junk".into());
    db.execute(
        "a",
        Precondition::Observed(0),
        vec![
            Command::SetMailbox {
                id: None,
                properties: junk,
            },
            Command::CreateMailbox {
                name: "Archive".into(),
            },
        ],
    )
    .unwrap();
    let account = db.account("a").unwrap();
    let junk = &account
        .mailboxes
        .iter()
        .find(|m| m.role.as_deref() == Some("junk"))
        .unwrap()
        .id;
    let archive = &account
        .mailboxes
        .iter()
        .find(|m| m.name == "Archive")
        .unwrap()
        .id;
    let count = || db.account("a").unwrap().messages.len();

    db.deliver_once("a", "k1", RAW, 1).unwrap();
    assert_eq!(count(), 1);
    assert_eq!(mailbox_of(&db, "e3"), ["inbox"]);
    // Same Message-ID linked to INBOX: suppressed, receipt still recorded,
    // and no body or reservation is written for the suppressed copy.
    let bodies = || -> (u64, u64) {
        let n = |t: &str| -> u64 {
            sql.query_row(&format!("SELECT count(*) FROM {t}"), [], |r| r.get(0))
                .unwrap()
        };
        (n("blob_content"), n("blob_reserved"))
    };
    let before = bodies();
    db.deliver_once(
        "a",
        "k2",
        b"Message-ID: <one@example.org>\r\nSubject: dup\r\n\r\nother",
        1,
    )
    .unwrap();
    assert_eq!(count(), 1);
    assert_eq!(bodies(), before, "a suppressed duplicate leaves no body");
    assert_eq!(
        db.deliver_once("a", "k2", b"other", 1),
        Err(Error::Conflict)
    );
    // Moved to the Junk-role mailbox: still suppressed.
    set_mailbox(&db, "e3", junk);
    db.deliver_once("a", "k3", RAW, 1).unwrap();
    assert_eq!(count(), 1);
    // Filed elsewhere: a redelivery lands in INBOX again.
    set_mailbox(&db, "e3", archive);
    db.deliver_once("a", "k4", RAW, 1).unwrap();
    assert_eq!(count(), 2);
    assert_eq!(mailbox_of(&db, "e6"), ["inbox"]);
    // A different References set is a different message.
    let referenced = b"Message-ID: <one@example.org>\r\nReferences: <zero@example.org>\r\n\r\nbody";
    db.deliver_once("a", "k5", referenced, 1).unwrap();
    assert_eq!(count(), 3);
    // Without a Message-ID nothing is ever suppressed.
    let anonymous = b"Subject: anonymous\r\n\r\nbody";
    db.deliver_once("a", "k6", anonymous, 1).unwrap();
    db.deliver_once("a", "k7", anonymous, 1).unwrap();
    assert_eq!(count(), 5);
    // Every receipt is on record.
    for key in ["k1", "k3", "k4", "k5", "k6", "k7"] {
        assert_eq!(
            db.deliver_once("a", key, b"changed", 1),
            Err(Error::Conflict)
        );
    }
}
