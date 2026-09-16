use mail_api::{Events, Store};
use mail_kernel::{Account, Command, Error};
use mail_sqlite_store::SqliteStore;

const TOKEN: &str = "0123456789abcdef0123456789abcdef";
const RAW: &[u8] = b"Message-ID: <original@example.org>\r\nSubject: durable\r\n\r\nbody\r\n";
fn database(quota: usize) -> SqliteStore {
    let db = SqliteStore::open(":memory:").unwrap();
    let mut account = Account::new("a", "t", "alice", "alice@example.org").unwrap();
    account.quota_bytes = quota;
    db.provision(account, TOKEN).unwrap();
    db.execute(
        "a",
        0,
        vec![
            Command::CreateMailbox {
                name: "Archive".into(),
            },
            Command::Append {
                mailboxes: vec!["inbox".into()],
                raw: RAW.to_vec(),
                keywords: vec!["$seen".into()],
                received_at: 1234,
            },
        ],
    )
    .unwrap();
    db
}
fn transfer(id: &str, target: &str, source: Option<&str>) -> Command {
    Command::Transfer {
        id: id.into(),
        mailbox: target.into(),
        remove_from: source.map(str::to_owned),
    }
}

#[test]
fn move_works_at_full_quota_but_copy_cannot_exceed_it() {
    let db = database(RAW.len());
    let before = db.account("a").unwrap();
    let id = &before.messages[0].id;
    assert_eq!(
        db.execute("a", before.revision, vec![transfer(id, "m1", None)]),
        Err(Error::OverQuota)
    );
    assert_eq!(db.account("a").unwrap(), before);
    let after = db
        .execute(
            "a",
            before.revision,
            vec![transfer(id, "m1", Some("inbox"))],
        )
        .unwrap();
    assert_eq!(after.messages.len(), 1);
    let message = &after.messages[0];
    assert_ne!(&message.id, id);
    assert_eq!(message.uid_in("m1"), Some(1));
    assert_eq!(message.thread_id(), before.messages[0].thread_id());
    assert_eq!(message.received_at, 1234);
    assert_eq!(message.keywords, ["$seen"]);
    assert_eq!(db.blob("a", &message.id).unwrap(), RAW);
    assert_eq!(db.blob("a", id), Err(Error::NotFound));
    let changes = db
        .message_changes("a", before.revision, after.revision)
        .unwrap();
    assert_eq!(changes.len(), 2);
}

#[test]
fn transfer_batch_rolls_back_bodies_metadata_uids_and_events_together() {
    let db = database(1000000);
    let before = db.account("a").unwrap();
    let events = db.pending("review", 100).unwrap();
    assert_eq!(
        db.execute(
            "a",
            before.revision,
            vec![
                transfer(&before.messages[0].id, "m1", Some("inbox")),
                transfer("missing", "m1", None)
            ]
        ),
        Err(Error::NotFound)
    );
    assert_eq!(db.account("a").unwrap(), before);
    assert_eq!(db.pending("review", 100).unwrap(), events);
    assert_eq!(db.blob("a", &before.messages[0].id).unwrap(), RAW);
    assert_eq!(
        db.blob("a", &format!("e{}", before.revision + 1)),
        Err(Error::NotFound)
    );
}

#[test]
fn same_mailbox_copy_and_move_allocate_fresh_uids() {
    let db = database(1000000);
    let before = db.account("a").unwrap();
    let copied = db
        .execute(
            "a",
            before.revision,
            vec![transfer(&before.messages[0].id, "inbox", None)],
        )
        .unwrap();
    assert_eq!(copied.messages.len(), 2);
    assert_eq!(copied.messages[1].uid_in("inbox"), Some(2));
    let moved = db
        .execute(
            "a",
            copied.revision,
            vec![transfer(&copied.messages[0].id, "inbox", Some("inbox"))],
        )
        .unwrap();
    assert_eq!(moved.messages.len(), 2);
    assert_eq!(moved.messages[1].uid_in("inbox"), Some(3));
    for message in &moved.messages {
        assert_eq!(db.blob("a", &message.id).unwrap(), RAW);
    }
}

#[test]
fn legacy_transfer_preserves_bodies_threads_and_account_isolation_across_rollback_and_restart() {
    let path = std::env::temp_dir().join(format!(
        "mail-transfer-legacy-{}.sqlite",
        std::process::id()
    ));
    let db = SqliteStore::open(&path).unwrap();
    let sql = rusqlite::Connection::open(&path).unwrap();
    for (id, raw) in [
        ("a", RAW),
        ("b", b"Subject: other account\r\n\r\nprivate\r\n".as_slice()),
    ] {
        db.provision(
            Account::new(id, id, id, &format!("{id}@example.org")).unwrap(),
            &id.repeat(32),
        )
        .unwrap();
        let mut legacy = serde_json::to_value(db.account(id).unwrap()).unwrap();
        legacy["revision"] = 9.into();
        legacy["mailboxes"][0]["uid_next"] = 43.into();
        legacy["messages"] = serde_json::json!([{
            "id":"e9", "mailbox":"inbox", "uid":42, "raw":raw,
            "keywords":["$seen"], "received_at":1234
        }]);
        sql.execute(
            "UPDATE accounts SET state=?1 WHERE id=?2",
            rusqlite::params![legacy.to_string(), id],
        )
        .unwrap();
    }
    let before = db.account("a").unwrap();
    assert_eq!(before.messages[0].email_identity(), "e9");
    assert_eq!(before.messages[0].thread_identity(), "e9");
    let other = db.account("b").unwrap();
    let other_body = db.blob("b", "e9").unwrap();
    sql.execute_batch("CREATE TRIGGER reject_transfer BEFORE INSERT ON history_commits BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(
        db.execute("a", 9, vec![transfer("e9", "inbox", Some("inbox"))])
            .is_err()
    );
    assert_eq!(db.account("a").unwrap(), before);
    assert_eq!(db.blob("a", "e9").unwrap(), RAW);
    assert_eq!(db.blob("a", "e10"), Err(Error::NotFound));
    assert_eq!(
        sql.query_row("SELECT count(*) FROM message_bodies", [], |r| r
            .get::<_, usize>(0))
            .unwrap(),
        0
    );
    sql.execute_batch("DROP TRIGGER reject_transfer").unwrap();
    let moved = db
        .execute("a", 9, vec![transfer("e9", "inbox", Some("inbox"))])
        .unwrap();
    assert_eq!(moved.messages[0].thread_id(), "e9");
    assert_eq!(moved.messages[0].email_identity(), "e9");
    assert_eq!(moved.messages[0].thread_identity(), "e9");
    assert_eq!(moved.messages[0].uid_in("inbox"), Some(43));
    assert_eq!(moved.messages[0].received_at, 1234);
    assert_eq!(moved.messages[0].keywords, ["$seen"]);
    assert_eq!(db.blob("a", "e9"), Err(Error::NotFound));
    assert_eq!(db.blob("a", "e10").unwrap(), RAW);
    assert_eq!(db.account("b").unwrap(), other);
    assert_eq!(db.blob("b", "e9").unwrap(), other_body);
    assert_eq!(db.blob("b", "e10"), Err(Error::NotFound));
    drop(db);
    let db = SqliteStore::open(&path).unwrap();
    assert_eq!(db.account("a").unwrap(), moved);
    assert_eq!(db.blob("a", "e10").unwrap(), RAW);
    assert_eq!(db.account("b").unwrap(), other);
    assert_eq!(db.blob("b", "e9").unwrap(), other_body);
    let linked = db
        .execute(
            "a",
            moved.revision,
            vec![Command::Append {
                mailboxes: vec!["inbox".into()],
                raw: b"Message-ID: <reply@example.org>\r\nReferences: <original@example.org>\r\nSubject: Re: durable\r\n\r\nreply\r\n".to_vec(),
                keywords: vec![],
                received_at: 1235,
            }],
        )
        .unwrap();
    assert_eq!(linked.messages.len(), 2);
    assert!(linked.messages.iter().all(|m| m.thread_id() == "e9"));
    assert!(linked.messages.iter().all(|m| m.thread_identity() == "e9"));
    assert_eq!(linked.messages[0].email_identity(), "e9");
    drop((sql, db));
    std::fs::remove_file(path).unwrap();
}
