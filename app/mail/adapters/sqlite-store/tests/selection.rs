use mail_api::{Events, MetadataStore, Precondition};
use mail_kernel::{Account, Command, Error};
use mail_sqlite_store::SqliteStore;

#[test]
fn selected_records_update_incrementally_and_roll_back_with_mail_and_events() {
    let path = std::env::temp_dir().join(format!(
        "mail-selection-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let db = SqliteStore::open(&path).unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    db.deliver(&["alice@example.org".into()], b"Subject: first\r\n\r\nbody")
        .unwrap();
    let sql = rusqlite::Connection::open(&path).unwrap();
    // An unrelated record's row is never rewritten by another message's commit.
    sql.execute_batch("CREATE TRIGGER untouched_record BEFORE UPDATE ON messages WHEN OLD.id='e1' BEGIN SELECT RAISE(ABORT,'unrelated record rewritten'); END;
        CREATE TRIGGER untouched_link BEFORE DELETE ON message_mailboxes WHEN OLD.message='e1' BEGIN SELECT RAISE(ABORT,'unrelated link rewritten'); END;").unwrap();
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
    assert_eq!(selected.messages[0].uid_in("inbox"), Some(2));
    assert!(db.messages("a", &[]).unwrap().messages.is_empty());
    assert_eq!(
        db.messages("a", &vec!["e1".into(); 257]),
        Err(Error::OverQuota)
    );
    assert_eq!(db.messages("missing", &["e1".into()]), Err(Error::NotFound));
    let inbox = db.mailbox_uids("a", "inbox").unwrap();
    assert_eq!(inbox.revision, 2);
    assert_eq!(inbox.uid_next, 3);
    assert_eq!(inbox.highest_modseq, 2);
    assert_eq!(inbox.uids, vec![(1, "e1".into()), (2, "e2".into())]);
    assert_eq!(db.mailbox_uids("a", "missing"), Err(Error::NotFound));
    sql.execute_batch("CREATE TRIGGER reject_record BEFORE INSERT ON messages BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
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
    assert_eq!(db.mailbox_uids("a", "inbox").unwrap().uid_next, 3);
    sql.execute_batch("DROP TRIGGER reject_record;").unwrap();
    db.execute(
        "a",
        Precondition::Require(2),
        vec![Command::Destroy { id: "e2".into() }],
    )
    .unwrap();
    let selected = db.messages("a", &["e2".into()]).unwrap();
    assert_eq!(selected.revision, 3);
    assert!(selected.messages.is_empty());
    assert_eq!(
        db.mailbox_uids("a", "inbox").unwrap().uids,
        vec![(1, "e1".into())]
    );
    drop((db, sql));
    let db = SqliteStore::open(&path).unwrap();
    assert_eq!(db.messages("a", &["e1".into()]).unwrap().revision, 3);
    drop(db);
    std::fs::remove_file(path).unwrap();
}
