use mail_api::{Events, Identity, Store};
use mail_kernel::{Account, Command, Error};
use mail_sqlite_store::SqliteStore;

const TOKEN: &str = "0123456789abcdef0123456789abcdef";

#[test]
fn delivery_is_atomic_durable_and_emits_replayable_events() {
    let path = std::env::temp_dir().join(format!(
        "mail-durable-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
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
                0,
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
            0,
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
}

#[test]
fn temporary_blobs_survive_restart_obey_quota_expire_and_stay_account_scoped() {
    let path = std::env::temp_dir().join(format!(
        "mail-blobs-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
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
                "UPDATE blobs SET expires_at=unixepoch()-1 WHERE account='a'",
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

#[test]
fn legacy_single_mailbox_snapshots_migrate_without_changing_message_bytes_or_uids() {
    let path = std::env::temp_dir().join(format!(
        "mail-legacy-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    {
        let db = SqliteStore::open(&path).unwrap();
        db.provision(
            Account::new("a", "t", "alice", "alice@example.org").unwrap(),
            TOKEN,
        )
        .unwrap();
        let connection = rusqlite::Connection::open(&path).unwrap();
        let legacy = serde_json::json!({"id":"a","tenant":"t","owner":"alice","address":"alice@example.org",
            "revision":9,"quota_bytes":4096,"mailboxes":[{"id":"inbox","name":"INBOX","role":"inbox","uid_next":43,"uid_validity":17}],
            "messages":[{"id":"e9","mailbox":"inbox","uid":42,"raw":[0,255,13,10],"keywords":["$seen"]}]});
        connection
            .execute(
                "UPDATE accounts SET state=?1 WHERE id='a'",
                [legacy.to_string()],
            )
            .unwrap();
        connection.pragma_update(None, "user_version", 0).unwrap();
    }
    for _ in 0..2 {
        let db = SqliteStore::open(&path).unwrap();
        let account = db.account("a").unwrap();
        assert_eq!(account.revision, 9);
        assert_eq!(account.messages[0].uid_in("inbox"), Some(42));
        assert_eq!(
            db.blob("a", &account.messages[0].id).unwrap(),
            [0, 255, 13, 10]
        );
        assert_eq!(account.messages[0].keywords, ["$seen"]);
        assert!(db.message_changes("a", 0, 9).is_err());
        assert!(db.message_changes("a", 9, 9).unwrap().is_empty());
        assert_eq!(account.mailboxes[0].uid_next, 43);
        assert_eq!(account.mailboxes[0].uid_validity, 17);
        assert!(db.pending("test", 10).unwrap().is_empty());
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn journal_survives_restart_and_refuses_gaps_and_partial_commits() {
    let path = std::env::temp_dir().join(format!(
        "mail-history-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
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
            db.execute("a", 1, vec![Command::Destroy { id: "e1".into() }])
                .is_err()
        );
        assert_eq!(db.account("a").unwrap().revision, 1);
        assert_eq!(db.pending("test", 10).unwrap().len(), 1);
        assert_eq!(db.message_changes("a", 0, 1).unwrap().len(), 1);
        db2.execute_batch("DROP TRIGGER reject_history").unwrap();
        db.execute(
            "a",
            1,
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
        let changes = db.message_changes("a", 0, 3).unwrap();
        assert_eq!(changes.len(), 2);
        assert!(changes[0].before.is_none());
        assert_eq!(changes[0].after.as_ref().unwrap().id, "e1");
        assert!(changes[1].before.as_ref().unwrap().keywords.is_empty());
        assert_eq!(changes[1].after.as_ref().unwrap().keywords, ["$seen"]);
        assert_eq!(changes[1].revision, 3);
        assert!(
            db.message_changes("a", 2, 3).is_err(),
            "intermediate batch state was never committed"
        );
        assert!(db.message_changes("a", 0, 4).is_err());
        assert!(db.message_changes("a", 3, 2).is_err());
        let db2 = rusqlite::Connection::open(&path).unwrap();
        db2.execute("DELETE FROM history_commits WHERE revision=1", [])
            .unwrap();
        assert!(
            db.message_changes("a", 0, 3).is_err(),
            "missing history must never be reported as complete"
        );
        assert_eq!(db.message_changes("a", 1, 3).unwrap().len(), 1);
    }
    std::fs::remove_file(path).unwrap();
}
