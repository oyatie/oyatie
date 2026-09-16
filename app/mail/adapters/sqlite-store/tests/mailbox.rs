use mail_api::{Events, Store};
use mail_kernel::{Account, Command, Error};
use mail_sqlite_store::SqliteStore;

#[test]
fn mailbox_counters_commit_with_messages_and_history_gaps_are_refused() {
    let path = std::env::temp_dir().join(format!(
        "mail-mailboxes-{}-{}.sqlite",
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
            &"a".repeat(32),
        )
        .unwrap();
        db.execute(
            "a",
            0,
            vec![Command::CreateMailbox {
                name: "Archive".into(),
            }],
        )
        .unwrap();
        db.execute(
            "a",
            1,
            vec![Command::Append {
                mailboxes: vec!["inbox".into(), "m1".into()],
                received_at: 0,
                raw: vec![0, 255],
                keywords: vec![],
            }],
        )
        .unwrap();
        let changes = db.mailbox_changes("a", 1, 2).unwrap();
        assert_eq!(changes.len(), 2);
        for change in changes {
            assert_eq!(change.before.unwrap().total_emails, 0);
            let after = change.after.unwrap();
            assert_eq!((after.total_emails, after.unread_emails), (1, 1));
        }
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection.execute_batch("CREATE TRIGGER reject_mailbox_history BEFORE INSERT ON mailbox_changes BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
        assert!(
            db.execute(
                "a",
                2,
                vec![Command::Keywords {
                    id: "e2".into(),
                    keywords: vec!["$seen".into()]
                }]
            )
            .is_err()
        );
        assert_eq!(db.account("a").unwrap().revision, 2);
        assert_eq!(db.pending("test", 10).unwrap().len(), 2);
        assert_eq!(db.message_changes("a", 0, 2).unwrap().len(), 1);
        connection
            .execute_batch("DROP TRIGGER reject_mailbox_history")
            .unwrap();
        db.execute(
            "a",
            2,
            vec![Command::Keywords {
                id: "e2".into(),
                keywords: vec!["$seen".into()],
            }],
        )
        .unwrap();
    }
    {
        let db = SqliteStore::open(&path).unwrap();
        let changes = db.mailbox_changes("a", 2, 3).unwrap();
        assert_eq!(changes.len(), 2);
        assert!(
            changes
                .iter()
                .all(|c| c.after.as_ref().unwrap().unread_emails == 0)
        );
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection
            .execute("DELETE FROM mailbox_commits WHERE revision=1", [])
            .unwrap();
        assert_eq!(db.mailbox_changes("a", 0, 3), Err(Error::Conflict));
        assert_eq!(db.mailbox_changes("a", 2, 3).unwrap().len(), 2);
        assert_eq!(db.blob("a", "e2").unwrap(), [0, 255]);
    }
    std::fs::remove_file(path).unwrap();
}
