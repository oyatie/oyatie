use mail_api::{Events, Store};
use mail_kernel::{Account, Command};
use mail_sqlite::SqliteStore;

fn append(db: &SqliteStore, account: &str, id: &str, refs: &str, subject: &str) -> Account {
    let state = db.account(account).unwrap();
    db.execute(
        account,
        state.revision,
        vec![Command::Append {
            mailboxes: vec!["inbox".into()],
            keywords: vec![],
            received_at: 1,
            raw: format!(
                "Message-ID: <{id}>\r\nReferences: {refs}\r\nSubject: {subject}\r\n\r\nbody"
            )
            .into_bytes(),
        }],
    )
    .unwrap()
}

#[test]
fn thread_merges_are_account_scoped_durable_and_journaled_atomically() {
    let path = std::env::temp_dir().join(format!("mail-threads-{}.sqlite", std::process::id()));
    let db = SqliteStore::open(&path).unwrap();
    for id in ["a", "b"] {
        db.provision(
            Account::new(id, id, id, &format!("{id}@example.org")).unwrap(),
            &id.repeat(32),
        )
        .unwrap();
    }
    append(&db, "a", "root@t", "", "Project Alpha");
    let before = append(&db, "a", "separate@t", "", "Re: Project Alpha");
    assert_ne!(
        before.messages[0].thread_id(),
        before.messages[1].thread_id()
    );
    let sql = rusqlite::Connection::open(&path).unwrap();
    sql.execute_batch("CREATE TRIGGER reject_thread_commit BEFORE INSERT ON history_commits BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    let raw = b"Message-ID: <bridge@t>\r\nReferences: <root@t> <separate@t>\r\nSubject: Re: Project Alpha\r\n\r\nbody";
    assert!(db.deliver(&["a@example.org".into()], raw).is_err());
    assert_eq!(db.account("a").unwrap(), before);
    assert_eq!(db.pending("foundry", 10).unwrap().len(), 2);
    sql.execute_batch("DROP TRIGGER reject_thread_commit")
        .unwrap();
    db.deliver(&["a@example.org".into()], raw).unwrap();
    let merged = db.account("a").unwrap();
    assert!(merged.messages.iter().all(|m| m.thread_id() == "e1"));
    let changes = db.message_changes("a", 2, 3).unwrap();
    assert!(
        changes
            .iter()
            .any(|c| c.id == "e2" && c.before.is_some() && c.after.is_some())
    );
    append(&db, "a", "different@t", "<root@t>", "Different subject");
    append(&db, "b", "unrelated@t", "", "Project Alpha");
    let other = append(&db, "b", "bridge@t", "<root@t>", "Re: Project Alpha");
    assert_ne!(other.messages[0].thread_id(), other.messages[1].thread_id());
    db.execute("a", 4, vec![Command::Destroy { id: "e1".into() }])
        .unwrap();
    drop(db);
    let db = SqliteStore::open(&path).unwrap();
    let state = append(&db, "a", "late@t", "<root@t>", "Re: Project Alpha");
    assert_eq!(state.messages.last().unwrap().thread_id(), "e1");
    assert_ne!(
        state
            .messages
            .iter()
            .find(|m| m.id == "e4")
            .unwrap()
            .thread_id(),
        "e1"
    );
    assert_eq!(
        sql.query_row(
            "SELECT count(*) FROM thread_members WHERE account='a' AND message='e1'",
            [],
            |r| r.get::<_, usize>(0)
        )
        .unwrap(),
        0
    );
    drop((db, sql));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn replies_arriving_before_the_parent_join_the_same_thread() {
    let db = SqliteStore::open(":memory:").unwrap();
    db.provision(
        Account::new("a", "t", "a", "a@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    append(&db, "a", "reply@t", "<parent@t>", "Re: Topic");
    let state = append(&db, "a", "parent@t", "", "Topic");
    assert_eq!(state.messages[0].thread_id(), state.messages[1].thread_id());
}

#[test]
fn removing_all_members_before_an_append_does_not_resurrect_a_destroyed_thread() {
    let db = SqliteStore::open(":memory:").unwrap();
    db.provision(
        Account::new("a", "t", "a", "a@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    append(&db, "a", "parent@t", "", "Topic");
    let state = db.execute("a",1,vec![Command::Destroy {id:"e1".into()}, Command::Append {
        mailboxes:vec!["inbox".into()], keywords:vec![], received_at:1,
        raw:b"Subject: Re: Topic\r\nReferences: <parent@t>\r\nMessage-ID: <late@t>\r\n\r\nbody".to_vec(),
    }]).unwrap();
    assert_eq!(state.messages[0].thread_id(), "e3");
}

#[test]
fn legacy_thread_backfill_is_atomic_and_preserves_surviving_thread_ids() {
    let path =
        std::env::temp_dir().join(format!("mail-thread-legacy-{}.sqlite", std::process::id()));
    let db = SqliteStore::open(&path).unwrap();
    db.provision(
        Account::new("a", "t", "a", "a@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    append(&db, "a", "root@t", "", "Topic");
    append(&db, "a", "reply@t", "<root@t>", "Re: Topic");
    db.execute("a", 2, vec![Command::Destroy { id: "e1".into() }])
        .unwrap();
    let sql = rusqlite::Connection::open(&path).unwrap();
    // Decode an actual old snapshot, without either immutable identity field.
    let mut legacy = serde_json::to_value(db.account("a").unwrap()).unwrap();
    for message in legacy["messages"].as_array_mut().unwrap() {
        let message = message.as_object_mut().unwrap();
        message.remove("email_identity");
        message.remove("thread_identity");
    }
    sql.execute(
        "UPDATE accounts SET state=?1 WHERE id='a'",
        [legacy.to_string()],
    )
    .unwrap();
    let before = db.account("a").unwrap();
    let bytes = db.blob("a", "e2").unwrap();
    assert_eq!(before.messages[0].email_identity(), "e2");
    assert_eq!(before.messages[0].thread_identity(), "e2");
    assert_eq!(
        sql.query_row("SELECT count(*) FROM thread_indexed", [], |r| r
            .get::<_, usize>(0))
            .unwrap(),
        0
    );
    drop(db);
    let db = SqliteStore::open(&path).unwrap();
    assert_eq!(db.account("a").unwrap(), before);
    sql.execute_batch("CREATE TRIGGER reject_backfill BEFORE INSERT ON thread_references BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(
        db.execute(
            "a",
            3,
            vec![Command::Keywords {
                id: "e2".into(),
                keywords: vec!["$seen".into()]
            }]
        )
        .is_err()
    );
    assert_eq!(db.account("a").unwrap(), before);
    assert_eq!(db.blob("a", "e2").unwrap(), bytes);
    sql.execute_batch("DROP TRIGGER reject_backfill;").unwrap();
    let state = db
        .execute(
            "a",
            3,
            vec![Command::Keywords {
                id: "e2".into(),
                keywords: vec!["$seen".into()],
            }],
        )
        .unwrap();
    assert_eq!(state.messages[0].thread_id(), "e1");
    assert_eq!(state.messages[0].email_identity(), "e2");
    assert_eq!(state.messages[0].thread_identity(), "e2");
    drop(db);
    let db = SqliteStore::open(&path).unwrap();
    assert_eq!(db.account("a").unwrap(), state);
    let state = append(&db, "a", "late@t", "<root@t>", "Re: Topic");
    assert!(state.messages.iter().all(|m| m.thread_identity() == "e2"));
    assert!(state.messages.iter().all(|m| m.thread_id() == "e1"));
    assert_eq!(db.blob("a", "e2").unwrap(), bytes);
    drop((db, sql));
    std::fs::remove_file(path).unwrap();
}
