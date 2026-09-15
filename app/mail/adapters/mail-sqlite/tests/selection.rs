use mail_api::{Events, Store};
use mail_kernel::{Account, Command, Error};
use mail_sqlite::SqliteStore;

#[test]
fn selected_metadata_updates_incrementally_and_rolls_back_with_mail_and_events() {
    let path = std::env::temp_dir().join(format!("mail-selection-{}.sqlite", std::process::id()));
    let db = SqliteStore::open(&path).unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    db.deliver(&["alice@example.org".into()], b"Subject: first\r\n\r\nbody")
        .unwrap();
    let sql = rusqlite::Connection::open(&path).unwrap();
    sql.execute_batch("CREATE TRIGGER untouched_metadata BEFORE UPDATE ON message_metadata WHEN OLD.id='e1' BEGIN SELECT RAISE(ABORT,'unrelated metadata rewritten'); END;").unwrap();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: second\r\n\r\nbody",
    )
    .unwrap();
    let selected = db
        .messages("a", &["e2".into(), "missing".into(), "e2".into()])
        .unwrap();
    assert_eq!(selected.revision, 2);
    assert_eq!(selected.messages.len(), 1);
    assert_eq!(selected.messages[0].id, "e2");
    assert!(db.messages("a", &[]).unwrap().messages.is_empty());
    assert_eq!(
        db.messages("a", &vec!["e1".into(); 257]),
        Err(Error::OverQuota)
    );
    assert_eq!(db.messages("missing", &["e1".into()]), Err(Error::NotFound));
    sql.execute_batch("CREATE TRIGGER reject_metadata BEFORE INSERT ON message_metadata BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(
        db.deliver(
            &["alice@example.org".into()],
            b"Subject: refused\r\n\r\nbody"
        )
        .is_err()
    );
    assert_eq!(db.account("a").unwrap().revision, 2);
    assert_eq!(db.messages("a", &["e1".into()]).unwrap().revision, 2);
    assert_eq!(db.blob("a", "e3"), Err(Error::NotFound));
    assert_eq!(db.pending("test", 10).unwrap().len(), 2);
    sql.execute_batch("DROP TRIGGER reject_metadata;").unwrap();
    db.execute("a", 2, vec![Command::Destroy { id: "e2".into() }])
        .unwrap();
    let selected = db.messages("a", &["e2".into()]).unwrap();
    assert_eq!(selected.revision, 3);
    assert!(selected.messages.is_empty());
    drop((db, sql));
    let db = SqliteStore::open(&path).unwrap();
    assert_eq!(db.messages("a", &["e1".into()]).unwrap().revision, 3);
    drop(db);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn legacy_writers_invalidate_the_projection_and_reads_preserve_the_snapshot() {
    let path = std::env::temp_dir().join(format!(
        "mail-selection-legacy-{}.sqlite",
        std::process::id()
    ));
    let db = SqliteStore::open(&path).unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    db.deliver(&["alice@example.org".into()], b"Subject: first\r\n\r\nbody")
        .unwrap();
    let mut old = db.account("a").unwrap();
    old.apply(Command::Keywords {
        id: "e1".into(),
        keywords: vec!["$flagged".into()],
    })
    .unwrap();
    let state = serde_json::to_string(&old).unwrap();
    let sql = rusqlite::Connection::open(&path).unwrap();
    // An older writer knows only the authoritative account snapshot.
    sql.execute("UPDATE accounts SET state=?1 WHERE id='a'", [&state])
        .unwrap();
    assert_eq!(
        sql.query_row("SELECT count(*) FROM message_index_state", [], |r| r
            .get::<_, usize>(0))
            .unwrap(),
        0
    );
    drop(db);
    let db = SqliteStore::open(&path).unwrap();
    let selected = db.messages("a", &["e1".into()]).unwrap();
    assert_eq!(selected.revision, 2);
    assert_eq!(selected.messages[0].keywords, ["$flagged"]);
    assert_eq!(
        sql.query_row("SELECT state FROM accounts WHERE id='a'", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        state
    );
    sql.execute_batch("CREATE TRIGGER reject_reindex BEFORE INSERT ON message_metadata BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(
        db.deliver(&["alice@example.org".into()], b"Subject: next\r\n\r\nbody")
            .is_err()
    );
    assert_eq!(
        sql.query_row("SELECT state FROM accounts WHERE id='a'", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        state
    );
    sql.execute_batch("DROP TRIGGER reject_reindex;").unwrap();
    db.deliver(&["alice@example.org".into()], b"Subject: next\r\n\r\nbody")
        .unwrap();
    assert_eq!(
        db.messages("a", &["e1".into()]).unwrap().messages[0].keywords,
        ["$flagged"]
    );
    assert_eq!(
        sql.query_row(
            "SELECT revision FROM message_index_state WHERE account='a'",
            [],
            |r| r.get::<_, u64>(0)
        )
        .unwrap(),
        3
    );
    drop((db, sql));
    std::fs::remove_file(path).unwrap();
}
