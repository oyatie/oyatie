use mail_api::{Identity, MetadataStore, Precondition};
use mail_kernel::{Account, Command, Error};
use mail_sqlite_store::SqliteStore;

fn temp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "mail-{name}-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn metadata_operations_do_not_load_or_rewrite_message_content() {
    let path = temp("content");
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
        Precondition::Require(1),
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
        db.execute(
            "a",
            Precondition::Require(2),
            vec![Command::Destroy { id: "e1".into() }]
        )
        .is_err()
    );
    assert_eq!(db.blob("a", "e1").unwrap(), raw);
    connection
        .execute_batch("DROP TRIGGER reject_journal")
        .unwrap();
    db.execute(
        "a",
        Precondition::Require(2),
        vec![Command::Destroy { id: "e1".into() }],
    )
    .unwrap();
    assert_eq!(db.blob("a", "e1"), Err(Error::NotFound));
    drop(connection);
    drop(db);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn independent_store_instances_cannot_lose_concurrent_updates() {
    let path = temp("writers");
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
        store.execute(
            "a",
            Precondition::Require(0),
            vec![Command::CreateMailbox { name: name.into() }],
        )
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
fn authentication_reads_only_the_account_row() {
    let path = temp("auth-metadata");
    let db = SqliteStore::open(&path).unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    db.deliver(&["alice@example.org".into()], b"Subject: x\r\n\r\nbody")
        .unwrap();
    let connection = rusqlite::Connection::open(&path).unwrap();
    // Corrupt every message record: authentication must not notice.
    connection
        .execute(
            "UPDATE messages SET modseq='not a number' WHERE account='a'",
            [],
        )
        .unwrap();
    assert_eq!(db.account("a"), Err(Error::Unavailable));
    assert_eq!(db.authenticate(&"a".repeat(32)).unwrap().subject, "alice");
    assert_eq!(db.account_info("a").unwrap().owner, "alice");
    db.revoke("a").unwrap();
    assert_eq!(db.authenticate(&"a".repeat(32)), Err(Error::Forbidden));
    drop((connection, db));
    std::fs::remove_file(path).unwrap();
}
