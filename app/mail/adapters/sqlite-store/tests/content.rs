use mail_api::{Identity, Store};
use mail_kernel::{Account, Command, Error};
use mail_sqlite_store::SqliteStore;

#[test]
fn metadata_operations_do_not_load_or_rewrite_message_content() {
    let path = std::env::temp_dir().join(format!("mail-content-{}.sqlite", std::process::id()));
    let db = SqliteStore::open(&path).unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    let raw = vec![255; 1024 * 1024];
    db.deliver(&["alice@example.org".into()], &raw).unwrap();
    assert!(
        serde_json::to_vec(&db.account("a").unwrap()).unwrap().len() < 4096,
        "account metadata must not contain message bytes"
    );
    assert_eq!(db.blob("a", "e1").unwrap(), raw);
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch("CREATE TRIGGER immutable_content BEFORE UPDATE ON message_bodies BEGIN SELECT RAISE(ABORT,'content rewritten'); END;
        CREATE TRIGGER no_content_insert BEFORE INSERT ON message_bodies BEGIN SELECT RAISE(ABORT,'content inserted'); END;").unwrap();
    db.execute(
        "a",
        1,
        vec![Command::Keywords {
            id: "e1".into(),
            keywords: vec!["$seen".into()],
        }],
    )
    .unwrap();
    assert_eq!(db.blob("a", "e1").unwrap(), raw);
    connection.execute_batch("DROP TRIGGER no_content_insert;
        CREATE TRIGGER reject_journal BEFORE INSERT ON history_commits BEGIN SELECT RAISE(ABORT,'journal unavailable'); END;").unwrap();
    assert!(
        db.deliver(&["alice@example.org".into()], b"must roll back")
            .is_err()
    );
    assert_eq!(db.blob("a", "e3"), Err(Error::NotFound));
    assert!(
        db.execute("a", 2, vec![Command::Destroy { id: "e1".into() }])
            .is_err()
    );
    assert_eq!(db.blob("a", "e1").unwrap(), raw);
    connection
        .execute_batch("DROP TRIGGER reject_journal")
        .unwrap();
    db.execute("a", 2, vec![Command::Destroy { id: "e1".into() }])
        .unwrap();
    assert_eq!(db.blob("a", "e1"), Err(Error::NotFound));
    drop(connection);
    drop(db);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn independent_store_instances_cannot_lose_concurrent_updates() {
    let path = std::env::temp_dir().join(format!("mail-writers-{}.sqlite", std::process::id()));
    let first = SqliteStore::open(&path).unwrap();
    first
        .provision(
            Account::new("a", "t", "alice", "alice@example.org").unwrap(),
            &"a".repeat(32),
        )
        .unwrap();
    let second = SqliteStore::open(&path).unwrap();
    let barrier = std::sync::Barrier::new(2);
    let write = |store: &SqliteStore, name: &str| {
        barrier.wait();
        store.execute("a", 0, vec![Command::CreateMailbox { name: name.into() }])
    };
    let results = std::thread::scope(|scope| {
        let a = scope.spawn(|| write(&first, "First"));
        let b = scope.spawn(|| write(&second, "Second"));
        [a.join().unwrap(), b.join().unwrap()]
    });
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|r| **r == Err(Error::Conflict))
            .count(),
        1
    );
    assert_eq!(first.account("a").unwrap(), second.account("a").unwrap());
    assert_eq!(first.account("a").unwrap().revision, 1);
    drop((first, second));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn authentication_does_not_depend_on_the_mailbox_projection() {
    let path =
        std::env::temp_dir().join(format!("mail-auth-metadata-{}.sqlite", std::process::id()));
    let db = SqliteStore::open(&path).unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute("UPDATE accounts SET state=json_set(state,'$.messages','unavailable projection') WHERE id='a'", []).unwrap();
    assert_eq!(db.account("a"), Err(Error::Unavailable));
    assert_eq!(db.authenticate(&"a".repeat(32)).unwrap().subject, "alice");
    db.revoke("a").unwrap();
    assert_eq!(db.authenticate(&"a".repeat(32)), Err(Error::Forbidden));
    drop((connection, db));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn legacy_content_moves_atomically_and_is_not_rewritten_on_open() {
    let path = std::env::temp_dir().join(format!(
        "mail-content-upgrade-{}.sqlite",
        std::process::id()
    ));
    let db = SqliteStore::open(&path).unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    let mut legacy = serde_json::to_value(db.account("a").unwrap()).unwrap();
    legacy["revision"] = 9.into();
    legacy["mailboxes"][0]["uid_next"] = 43.into();
    legacy["messages"] = serde_json::json!([{"id":"e9","mailbox":"inbox","uid":42,"raw":[0,255,13,10],"keywords":[]}]);
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute(
            "UPDATE accounts SET state=?1 WHERE id='a'",
            [legacy.to_string()],
        )
        .unwrap();
    drop(db);
    let db = SqliteStore::open(&path).unwrap();
    let persisted = || {
        connection
            .query_row("SELECT state FROM accounts WHERE id='a'", [], |r| {
                r.get::<_, String>(0)
            })
            .unwrap()
    };
    assert_eq!(persisted(), legacy.to_string());
    assert_eq!(db.blob("a", "e9").unwrap(), [0, 255, 13, 10]);
    connection.execute_batch("CREATE TRIGGER stop_upgrade BEFORE INSERT ON history_commits BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    let change = || {
        vec![Command::Keywords {
            id: "e9".into(),
            keywords: vec!["$seen".into()],
        }]
    };
    assert!(db.execute("a", 9, change()).is_err());
    assert_eq!(persisted(), legacy.to_string());
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM message_bodies", [], |r| r
                .get::<_, usize>(0))
            .unwrap(),
        0
    );
    connection
        .execute_batch("DROP TRIGGER stop_upgrade")
        .unwrap();
    db.execute("a", 9, change()).unwrap();
    assert!(!persisted().contains("\"raw\""));
    assert_eq!(db.blob("a", "e9").unwrap(), [0, 255, 13, 10]);
    assert_eq!(
        db.account("a").unwrap().messages[0].uid_in("inbox"),
        Some(42)
    );
    assert_eq!(db.account("a").unwrap().mailboxes[0].uid_next, 43);
    db.execute(
        "a",
        10,
        vec![
            Command::Append {
                raw: vec![1],
                mailboxes: vec!["inbox".into()],
                keywords: vec![],
                received_at: 0,
            },
            Command::Destroy { id: "e11".into() },
        ],
    )
    .unwrap();
    assert_eq!(db.blob("a", "e11"), Err(Error::NotFound));
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM message_bodies", [], |r| r
                .get::<_, usize>(0))
            .unwrap(),
        1
    );
    drop(db);
    let db = SqliteStore::open(&path).unwrap();
    assert_eq!(db.blob("a", "e9").unwrap(), [0, 255, 13, 10]);
    drop((connection, db));
    std::fs::remove_file(path).unwrap();
}
