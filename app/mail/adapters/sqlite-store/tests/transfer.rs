use mail_api::{Events, MetadataStore, Precondition};
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
        Precondition::Require(0),
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
fn run(db: &SqliteStore, revision: u64, commands: Vec<Command>) -> Result<Account, Error> {
    db.execute("a", Precondition::Require(revision), commands)?;
    db.account("a")
}

#[test]
fn move_works_at_full_quota_but_copy_cannot_exceed_it() {
    let db = database(RAW.len());
    let before = db.account("a").unwrap();
    let id = &before.messages[0].id;
    assert_eq!(
        run(&db, before.revision, vec![transfer(id, "m1", None)]),
        Err(Error::OverQuota)
    );
    assert_eq!(db.account("a").unwrap(), before);
    let after = run(
        &db,
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
    assert_eq!(after.used_bytes, RAW.len());
    let page = db.history("a", before.revision, 10).unwrap();
    assert_eq!(page.rows.len(), 2);
    assert!(page.rows.iter().all(|(r, _)| *r == after.revision));
}

#[test]
fn transfer_batch_rolls_back_bodies_metadata_uids_and_events_together() {
    let db = database(1000000);
    let before = db.account("a").unwrap();
    let events = db.pending("review", 100).unwrap();
    assert_eq!(
        run(
            &db,
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
    assert_eq!(db.mailbox_uids("a", "m1").unwrap().uid_next, 1);
}

#[test]
fn same_mailbox_copy_and_move_allocate_fresh_uids() {
    let db = database(1000000);
    let before = db.account("a").unwrap();
    let copied = run(
        &db,
        before.revision,
        vec![transfer(&before.messages[0].id, "inbox", None)],
    )
    .unwrap();
    assert_eq!(copied.messages.len(), 2);
    assert_eq!(copied.messages[1].uid_in("inbox"), Some(2));
    let moved = run(
        &db,
        copied.revision,
        vec![transfer(&copied.messages[0].id, "inbox", Some("inbox"))],
    )
    .unwrap();
    assert_eq!(moved.messages.len(), 2);
    assert_eq!(moved.messages[1].uid_in("inbox"), Some(3));
    for message in &moved.messages {
        assert_eq!(db.blob("a", &message.id).unwrap(), RAW);
    }
    assert_eq!(
        db.mailbox_uids("a", "inbox").unwrap().uids,
        vec![(2, "e3".into()), (3, "e4".into())]
    );
}

#[test]
fn transfer_preserves_bodies_threads_and_account_isolation_across_rollback_and_restart() {
    let path = std::env::temp_dir().join(format!(
        "mail-transfer-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
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
        db.execute(
            id,
            Precondition::Require(0),
            vec![Command::Append {
                mailboxes: vec!["inbox".into()],
                raw: raw.to_vec(),
                keywords: vec!["$seen".into()],
                received_at: 1234,
            }],
        )
        .unwrap();
    }
    let before = db.account("a").unwrap();
    assert_eq!(before.messages[0].email_identity(), "e1");
    assert_eq!(before.messages[0].thread_identity(), "e1");
    let other = db.account("b").unwrap();
    let other_body = db.blob("b", "e1").unwrap();
    sql.execute_batch("CREATE TRIGGER reject_transfer BEFORE INSERT ON history_commits BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(run(&db, 1, vec![transfer("e1", "inbox", Some("inbox"))]).is_err());
    assert_eq!(db.account("a").unwrap(), before);
    assert_eq!(db.blob("a", "e1").unwrap(), RAW);
    assert_eq!(db.blob("a", "e2"), Err(Error::NotFound));
    assert_eq!(
        sql.query_row("SELECT count(*) FROM message_bodies", [], |r| r
            .get::<_, usize>(0))
            .unwrap(),
        2
    );
    sql.execute_batch("DROP TRIGGER reject_transfer").unwrap();
    let moved = run(&db, 1, vec![transfer("e1", "inbox", Some("inbox"))]).unwrap();
    assert_eq!(moved.messages[0].id, "e2");
    assert_eq!(moved.messages[0].thread_id(), "e1");
    assert_eq!(moved.messages[0].email_identity(), "e1");
    assert_eq!(moved.messages[0].thread_identity(), "e1");
    assert_eq!(moved.messages[0].uid_in("inbox"), Some(2));
    assert_eq!(moved.messages[0].received_at, 1234);
    assert_eq!(moved.messages[0].keywords, ["$seen"]);
    assert_eq!(db.blob("a", "e1"), Err(Error::NotFound));
    assert_eq!(db.blob("a", "e2").unwrap(), RAW);
    assert_eq!(db.account("b").unwrap(), other);
    assert_eq!(db.blob("b", "e1").unwrap(), other_body);
    assert_eq!(db.blob("b", "e2"), Err(Error::NotFound));
    drop(db);
    let db = SqliteStore::open(&path).unwrap();
    assert_eq!(db.account("a").unwrap(), moved);
    assert_eq!(db.blob("a", "e2").unwrap(), RAW);
    assert_eq!(db.account("b").unwrap(), other);
    assert_eq!(db.blob("b", "e1").unwrap(), other_body);
    let linked = run(
        &db,
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
    assert!(linked.messages.iter().all(|m| m.thread_id() == "e1"));
    assert!(linked.messages.iter().all(|m| m.thread_identity() == "e1"));
    assert_eq!(linked.messages[0].email_identity(), "e1");
    drop((sql, db));
    std::fs::remove_file(path).unwrap();
}
