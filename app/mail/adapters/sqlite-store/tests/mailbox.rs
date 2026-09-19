use mail_api::{BlobStore, Events, MetadataStore, Precondition};
use mail_kernel::{Account, Command, HistoryEntry};
use mail_sqlite_store::SqliteStore;

#[test]
fn mailbox_counters_commit_with_messages_and_roll_back_with_a_refused_history_row() {
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
            Precondition::Require(0),
            vec![Command::CreateMailbox {
                name: "Archive".into(),
            }],
        )
        .unwrap();
        let execution = db
            .execute(
                "a",
                Precondition::Require(1),
                vec![
                    db.append("a", vec!["inbox".into(), "m1".into()], &[0, 255], vec![], 0)
                        .unwrap(),
                ],
            )
            .unwrap();
        assert_eq!(
            execution.allocations,
            vec![("inbox".into(), 1), ("m1".into(), 1)]
        );
        let account = db.account("a").unwrap();
        for mailbox in &account.mailboxes {
            assert_eq!(
                (
                    mailbox.total_emails,
                    mailbox.unread_emails,
                    mailbox.size_bytes,
                    mailbox.highest_modseq
                ),
                (1, 1, 2, 2),
                "{}",
                mailbox.id
            );
        }
        assert_eq!(account.mailboxes[1].created_revision, 1);
        let page = db.history("a", 1, 10).unwrap();
        let added: Vec<_> = page
            .rows
            .iter()
            .filter(|(_, e)| matches!(e, HistoryEntry::Added { .. }))
            .collect();
        assert_eq!(added.len(), 2);
        for mailbox in ["inbox", "m1"] {
            assert_eq!(
                db.mailbox_uids("a", mailbox).unwrap().uids,
                vec![(1, "e2".into())]
            );
        }
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection.execute_batch("CREATE TRIGGER reject_history BEFORE INSERT ON history BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
        assert!(
            db.execute(
                "a",
                Precondition::Require(2),
                vec![Command::Keywords {
                    id: "e2".into(),
                    keywords: vec!["$seen".into()]
                }]
            )
            .is_err()
        );
        assert_eq!(db.account("a").unwrap(), account);
        assert_eq!(db.pending("test", 10).unwrap().len(), 2);
        assert_eq!(db.history("a", 0, 10).unwrap().rows.len(), 3);
        connection
            .execute_batch("DROP TRIGGER reject_history")
            .unwrap();
        db.execute(
            "a",
            Precondition::Require(2),
            vec![Command::Keywords {
                id: "e2".into(),
                keywords: vec!["$seen".into()],
            }],
        )
        .unwrap();
    }
    {
        let db = SqliteStore::open(&path).unwrap();
        let account = db.account("a").unwrap();
        assert!(
            account
                .mailboxes
                .iter()
                .all(|m| m.unread_emails == 0 && m.total_emails == 1 && m.highest_modseq == 3)
        );
        let page = db.history("a", 2, 10).unwrap();
        assert_eq!(
            page.rows,
            vec![(3, HistoryEntry::Flags { id: "e2".into() })]
        );
        assert_eq!(db.blob("a", "e2").unwrap(), [0, 255]);
        // Unlinking from one mailbox keeps the record; the last unlink deletes it.
        db.execute(
            "a",
            Precondition::Require(3),
            vec![Command::SetMailboxes {
                id: "e2".into(),
                mailboxes: vec!["m1".into()],
            }],
        )
        .unwrap();
        let account = db.account("a").unwrap();
        assert_eq!(account.mailboxes[0].total_emails, 0);
        assert_eq!(account.mailboxes[1].total_emails, 1);
        assert_eq!(account.messages.len(), 1);
        db.execute(
            "a",
            Precondition::Require(4),
            vec![Command::RemoveMailbox {
                id: "m1".into(),
                remove_emails: true,
            }],
        )
        .unwrap();
        let account = db.account("a").unwrap();
        assert!(account.messages.is_empty());
        assert_eq!(account.mailboxes.len(), 1);
        assert_eq!(account.used_bytes, 0);
        let page = db.history("a", 4, 10).unwrap();
        assert_eq!(
            page.rows,
            vec![
                (
                    5,
                    HistoryEntry::Removed {
                        id: "e2".into(),
                        mailbox: "m1".into(),
                        uid: 1,
                        thread: "e2".into()
                    }
                ),
                (5, HistoryEntry::Mailbox { id: "m1".into() })
            ]
        );
        assert_eq!(db.blob("a", "e2"), Err(mail_kernel::Error::NotFound));
    }
    std::fs::remove_file(path).unwrap();
}
