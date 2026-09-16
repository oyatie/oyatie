use mail_api::Store;
use mail_kernel::{Account, Command, Error};
use mail_sqlite_store::SqliteStore;
fn append(n: u8) -> Command {
    Command::Append {
        mailboxes: vec!["inbox".into()],
        raw: format!("Subject: {n}\r\n\r\nbody").into_bytes(),
        keywords: vec![],
        received_at: 1,
    }
}
fn provision(db: &SqliteStore) {
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        "0123456789abcdef0123456789abcdef",
    )
    .unwrap();
}
#[test]
fn numeric_threshold_includes_whole_atomic_batch_without_weakening_exact_state_history() {
    let db = SqliteStore::open(":memory:").unwrap();
    provision(&db);
    db.execute("a", 0, vec![append(1), append(2)]).unwrap();
    assert_eq!(db.message_changes("a", 1, 2), Err(Error::Conflict));
    for since in [0, 1] {
        let changes = db.message_changes_after("a", since, 2).unwrap();
        assert_eq!(changes.len(), 2);
        assert!(
            changes
                .iter()
                .all(|c| c.revision == 2 && c.before.is_none())
        );
        assert_eq!(
            changes
                .iter()
                .map(|c| c.after.as_ref().unwrap().modseq)
                .collect::<Vec<_>>(),
            [3, 3]
        );
    }
    assert!(db.message_changes_after("a", 2, 2).unwrap().is_empty());
    assert_eq!(db.message_changes_after("a", 0, 1), Err(Error::Conflict));
    assert_eq!(db.message_changes_after("a", 3, 2), Err(Error::Conflict));
    assert_eq!(db.message_changes_after("a", 2, 3), Err(Error::Conflict));
    assert!(db.message_changes_after("other", 0, 2).is_err());
}
#[test]
fn numeric_history_refuses_pruned_chain_and_remains_durable_after_restart() {
    let path = std::env::temp_dir().join(format!(
        "mail-threshold-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    {
        let db = SqliteStore::open(&path).unwrap();
        provision(&db);
        db.execute("a", 0, vec![append(1), append(2)]).unwrap();
        db.execute("a", 2, vec![append(3), append(4)]).unwrap();
        assert_eq!(db.message_changes_after("a", 1, 4).unwrap().len(), 4);
    }
    {
        let db = SqliteStore::open(&path).unwrap();
        assert_eq!(db.message_changes_after("a", 3, 4).unwrap().len(), 2);
        let raw = rusqlite::Connection::open(&path).unwrap();
        raw.execute(
            "DELETE FROM history_commits WHERE account='a' AND revision=2",
            [],
        )
        .unwrap();
        for since in [0, 1] {
            assert_eq!(
                db.message_changes_after("a", since, 4),
                Err(Error::Conflict)
            );
        }
        assert_eq!(db.message_changes_after("a", 3, 4).unwrap().len(), 2);
    }
    std::fs::remove_file(path).unwrap();
}
