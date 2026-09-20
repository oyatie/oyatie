use mail_api::{Events, HistoryPage, Identity, MetadataStore, Precondition};
use mail_kernel::{Account, Command, Error, HistoryEntry};
use mail_sqlite_store::SqliteStore;

const TOKEN: &str = "0123456789abcdef0123456789abcdef";

fn temp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "mail-{name}-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn delivery_is_atomic_durable_and_emits_replayable_events() {
    let path = temp("durable");
    {
        let db = SqliteStore::open(&path).unwrap();
        db.provision(
            Account::new("a", "t", "alice", "alice@example.org").unwrap(),
            TOKEN,
        )
        .unwrap();
        assert_eq!(db.authenticate("wrong"), Err(Error::Forbidden));
        assert_eq!(db.authenticate(TOKEN).unwrap().account, "a");
        assert_eq!(
            db.deliver(
                &["alice@example.org".into(), "unknown@external.org".into()],
                b"no partial delivery"
            ),
            Err(Error::NotFound)
        );
        assert!(db.account("a").unwrap().messages.is_empty());
        assert!(db.pending("test", 10).unwrap().is_empty());
        db.deliver(
            &["alice@example.org".into(), "alice@example.org".into()],
            b"Subject: preserved\r\n\r\n\xff",
        )
        .unwrap();
    }
    {
        let db = SqliteStore::open(&path).unwrap();
        let account = db.account("a").unwrap();
        assert_eq!(account.messages.len(), 1);
        assert_eq!(
            db.blob("a", &account.messages[0].id).unwrap(),
            b"Subject: preserved\r\n\r\n\xff"
        );
        assert_eq!(
            db.execute(
                "a",
                Precondition::Require(0),
                vec![Command::CreateMailbox {
                    name: "stale".into()
                }]
            ),
            Err(Error::Conflict)
        );
        let events = db.pending("test", 10).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].revision, 1);
        db.acknowledge("test", events[0].sequence).unwrap();
        assert!(db.pending("test", 10).unwrap().is_empty());
        db.revoke("a").unwrap();
        assert_eq!(db.authenticate(TOKEN), Err(Error::Forbidden));
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn failing_batch_rolls_back_all_commands_and_outbox() {
    let db = SqliteStore::open(":memory:").unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    assert_eq!(
        db.execute(
            "a",
            Precondition::Require(0),
            vec![
                Command::CreateMailbox {
                    name: "work".into()
                },
                Command::DeleteMailbox { id: "inbox".into() }
            ]
        ),
        Err(Error::Forbidden)
    );
    assert_eq!(db.account("a").unwrap().revision, 0);
    assert_eq!(db.account("a").unwrap().mailboxes.len(), 1);
    assert!(db.pending("test", 10).unwrap().is_empty());
    assert!(db.history("a", 0, 10).unwrap().rows.is_empty());
}

#[test]
fn temporary_blobs_survive_restart_obey_quota_expire_and_stay_account_scoped() {
    let path = temp("blobs");
    let id;
    {
        let db = SqliteStore::open(&path).unwrap();
        let mut account = Account::new("a", "t", "alice", "alice@example.org").unwrap();
        account.quota_bytes = 4;
        db.provision(account, TOKEN).unwrap();
        db.provision(
            Account::new("b", "other", "bob", "bob@example.org").unwrap(),
            &"b".repeat(32),
        )
        .unwrap();
        id = db.put_blob("a", b"\0\xff\r\n").unwrap();
        assert_eq!(db.put_blob("a", b"\0\xff\r\n").unwrap(), id);
        assert_eq!(db.put_blob("a", b"x"), Err(Error::OverQuota));
        assert_eq!(db.blob("b", &id), Err(Error::NotFound));
        assert_eq!(db.put_blob("missing", b"x"), Err(Error::NotFound));
        assert_eq!(db.account("a").unwrap().revision, 0);
        assert!(db.pending("test", 10).unwrap().is_empty());
    }
    {
        let db = SqliteStore::open(&path).unwrap();
        assert_eq!(db.blob("a", &id).unwrap(), b"\0\xff\r\n");
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection
            .execute(
                "UPDATE blob_reservations SET expires_at=unixepoch()-1 WHERE account='a'",
                [],
            )
            .unwrap();
        assert_eq!(db.blob("a", &id), Err(Error::NotFound));
        let replacement = db.put_blob("a", b"new!").unwrap();
        assert_eq!(db.blob("a", &replacement).unwrap(), b"new!");
        assert_eq!(db.blob("a", &id), Err(Error::NotFound));
    }
    std::fs::remove_file(path).unwrap();
}

fn rows(page: &HistoryPage) -> Vec<&HistoryEntry> {
    page.rows.iter().map(|(_, e)| e).collect()
}

#[test]
fn history_survives_restart_and_a_refused_commit_row_rolls_back_the_batch() {
    let path = temp("history");
    {
        let db = SqliteStore::open(&path).unwrap();
        db.provision(
            Account::new("a", "t", "alice", "alice@example.org").unwrap(),
            TOKEN,
        )
        .unwrap();
        db.deliver(
            &["alice@example.org".into()],
            b"Subject: journal\r\n\r\nbody",
        )
        .unwrap();
        let db2 = rusqlite::Connection::open(&path).unwrap();
        db2.execute_batch("CREATE TRIGGER reject_history BEFORE INSERT ON history_commits BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
        assert!(
            db.execute(
                "a",
                Precondition::Require(1),
                vec![Command::Destroy { id: "e1".into() }]
            )
            .is_err()
        );
        assert_eq!(db.account("a").unwrap().revision, 1);
        assert_eq!(db.account("a").unwrap().messages.len(), 1);
        assert_eq!(db.pending("test", 10).unwrap().len(), 1);
        assert_eq!(db.history("a", 0, 10).unwrap().rows.len(), 1);
        db2.execute_batch("DROP TRIGGER reject_history").unwrap();
        db.execute(
            "a",
            Precondition::Require(1),
            vec![
                Command::Keywords {
                    id: "e1".into(),
                    keywords: vec!["$seen".into()],
                },
                Command::CreateMailbox {
                    name: "Archive".into(),
                },
            ],
        )
        .unwrap();
    }
    {
        let db = SqliteStore::open(&path).unwrap();
        let page = db.history("a", 0, 10).unwrap();
        assert_eq!(page.revision, 3);
        assert!(!page.has_more);
        assert_eq!(
            rows(&page),
            vec![
                &HistoryEntry::Added {
                    id: "e1".into(),
                    mailbox: "inbox".into(),
                    uid: 1
                },
                &HistoryEntry::Flags { id: "e1".into() },
                &HistoryEntry::Mailbox { id: "m3".into() },
            ]
        );
        assert_eq!(
            page.rows.iter().map(|(r, _)| *r).collect::<Vec<_>>(),
            [1, 3, 3]
        );
        // The intermediate state of the batch was never a revision of its own.
        assert!(db.history("a", 2, 10).unwrap().rows.len() == 2);
        assert_eq!(db.history("a", 4, 10), Err(Error::Conflict));
    }
    std::fs::remove_file(path).unwrap();
}
