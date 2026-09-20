use mail_api::{Events, MetadataStore, Precondition};
use mail_kernel::{Account, Command, HistoryEntry};
use mail_sqlite_store::SqliteStore;

fn append(db: &SqliteStore, account: &str, id: &str, refs: &str, subject: &str) -> Account {
    let state = db.account(account).unwrap();
    db.execute(
        account,
        Precondition::Require(state.revision),
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
    .unwrap();
    db.account(account).unwrap()
}

#[test]
fn thread_merges_are_account_scoped_durable_and_journaled_atomically() {
    let path = std::env::temp_dir().join(format!(
        "mail-threads-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
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
    assert_eq!(merged.messages[1].modseq, 3, "bridged member is stamped");
    let page = db.history("a", 2, 10).unwrap();
    assert!(
        page.rows
            .iter()
            .any(|(r, e)| *r == 3 && *e == HistoryEntry::Thread { id: "e2".into() })
    );
    append(&db, "a", "different@t", "<root@t>", "Different subject");
    append(&db, "b", "unrelated@t", "", "Project Alpha");
    let other = append(&db, "b", "bridge@t", "<root@t>", "Re: Project Alpha");
    assert_ne!(other.messages[0].thread_id(), other.messages[1].thread_id());
    db.execute(
        "a",
        Precondition::Require(4),
        vec![Command::Destroy { id: "e1".into() }],
    )
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
    db.execute("a", Precondition::Require(1), vec![Command::Destroy { id: "e1".into() }, Command::Append {
        mailboxes: vec!["inbox".into()], keywords: vec![], received_at: 1,
        raw: b"Subject: Re: Topic\r\nReferences: <parent@t>\r\nMessage-ID: <late@t>\r\n\r\nbody".to_vec(),
    }]).unwrap();
    let state = db.account("a").unwrap();
    assert_eq!(state.messages[0].thread_id(), "e3");
}

#[test]
fn one_batch_of_replies_shares_the_root_thread_identity() {
    let db = SqliteStore::open(":memory:").unwrap();
    db.provision(
        Account::new("a", "a", "a", "a@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    let message = |id: &str, refs: &str| Command::Append {
        mailboxes: vec!["inbox".into()],
        keywords: vec![],
        received_at: 1,
        raw: format!("Message-ID: <{id}>\r\nReferences: {refs}\r\nSubject: T\r\n\r\nbody")
            .into_bytes(),
    };
    db.execute(
        "a",
        Precondition::Require(0),
        vec![
            message("root@t", ""),
            message("r1@t", "<root@t>"),
            message("r2@t", "<root@t> <r1@t>"),
        ],
    )
    .unwrap();
    let account = db.account("a").unwrap();
    assert_eq!(account.messages.len(), 3);
    assert!(
        account
            .messages
            .iter()
            .all(|m| m.thread_identity() == account.messages[0].thread_identity()),
        "{:?}",
        account
            .messages
            .iter()
            .map(|m| (m.id.clone(), m.thread_identity().to_owned()))
            .collect::<Vec<_>>()
    );
}
