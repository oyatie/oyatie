//! MODSEQ semantics at the store boundary: one visible stamp per committed
//! batch equal to the new account revision; history rows describe exactly the
//! links, flags and threads that changed; a failed batch writes nothing.
use mail_api::{BlobStore, HistoryPage, MetadataStore, Precondition};
use mail_kernel::{Account, Command, Error, HistoryEntry};
use mail_sqlite_store::SqliteStore;

fn append(db: &SqliteStore, n: u8) -> Command {
    db.append(
        "a",
        vec!["inbox".into()],
        format!("Subject: {n}\r\n\r\nbody").as_bytes(),
        vec![],
        1,
    )
    .unwrap()
}
fn fixture() -> (SqliteStore, Account) {
    let db = SqliteStore::open(":memory:").unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        "0123456789abcdef0123456789abcdef",
    )
    .unwrap();
    let execution = db
        .execute(
            "a",
            Precondition::Require(0),
            vec![append(&db, 1), append(&db, 2), append(&db, 3)],
        )
        .unwrap();
    assert_eq!(execution.revision, 3);
    assert_eq!(execution.ids, vec!["e1", "e2", "e3"]);
    assert_eq!(
        execution.allocations,
        vec![
            ("inbox".into(), 1),
            ("inbox".into(), 2),
            ("inbox".into(), 3)
        ]
    );
    let account = db.account("a").unwrap();
    (db, account)
}
fn rows(page: &HistoryPage) -> Vec<&HistoryEntry> {
    page.rows.iter().map(|(_, e)| e).collect()
}

#[test]
fn one_batch_one_stamp_and_history_names_every_changed_link_and_flag() {
    let (db, before) = fixture();
    assert!(before.messages.iter().all(|m| m.modseq == 3));
    assert_eq!(before.mailboxes[0].highest_modseq, 3);
    assert_eq!(before.mailboxes[0].total_emails, 3);
    let execution = db
        .execute(
            "a",
            Precondition::Require(before.revision),
            vec![
                Command::Destroy {
                    id: before.messages[0].id.clone(),
                },
                Command::Keywords {
                    id: before.messages[1].id.clone(),
                    keywords: vec!["$seen".into()],
                },
                Command::Keywords {
                    id: before.messages[2].id.clone(),
                    keywords: vec!["$answered".into()],
                },
                Command::CreateMailbox {
                    name: "Work".into(),
                },
            ],
        )
        .unwrap();
    assert_eq!(execution.revision, 7);
    assert!(execution.ids.is_empty());
    let account = db.account("a").unwrap();
    assert_eq!(account.revision, 7);
    assert_eq!(account.mail_modseq, 7);
    assert!(account.messages.iter().all(|m| m.modseq == 7));
    assert_eq!(account.mailboxes[0].highest_modseq, 7);
    assert_eq!(account.mailboxes[0].total_emails, 2);
    assert_eq!(account.mailboxes[0].unread_emails, 1);
    assert_eq!(account.mailboxes[1].highest_modseq, 3);
    assert_eq!(account.mailboxes[1].created_revision, 7);
    let page = db.history("a", before.revision, 100).unwrap();
    assert_eq!(page.revision, 7);
    assert!(!page.has_more);
    assert_eq!(
        rows(&page),
        vec![
            &HistoryEntry::Flags { id: "e2".into() },
            &HistoryEntry::Flags { id: "e3".into() },
            &HistoryEntry::Removed {
                id: "e1".into(),
                mailbox: "inbox".into(),
                uid: 1,
                thread: "e1".into()
            },
            &HistoryEntry::Mailbox { id: "m7".into() },
        ]
    );
    assert_eq!(db.blob("a", "e1"), Err(Error::NotFound));
    assert_eq!(db.messages("a", &["e1".into()]).unwrap().messages.len(), 0);
}

#[test]
fn unchanged_edits_and_mailbox_only_batches_do_not_stamp_messages() {
    let (db, before) = fixture();
    let id = before.messages[0].id.clone();
    db.execute(
        "a",
        Precondition::Require(before.revision),
        vec![
            Command::Keywords {
                id: id.clone(),
                keywords: vec!["$seen".into()],
            },
            Command::Keywords {
                id,
                keywords: vec![],
            },
            Command::CreateMailbox {
                name: "Work".into(),
            },
        ],
    )
    .unwrap();
    let account = db.account("a").unwrap();
    assert_eq!(account.revision, 6);
    assert_eq!(account.mail_modseq, before.mail_modseq);
    assert_eq!(account.messages, before.messages);
    let page = db.history("a", before.revision, 100).unwrap();
    assert_eq!(
        rows(&page),
        vec![&HistoryEntry::Mailbox { id: "m6".into() }]
    );
}

#[test]
fn conditional_batches_conflict_and_observed_batches_reapply() {
    let (db, before) = fixture();
    assert_eq!(
        db.execute("a", Precondition::Require(1), vec![append(&db, 9)]),
        Err(Error::Conflict)
    );
    assert_eq!(db.account("a").unwrap(), before);
    // A stale observer's Destroy of a vanished record is a no-op; its other
    // commands still apply on the current state.
    db.execute(
        "a",
        Precondition::Require(before.revision),
        vec![Command::Destroy { id: "e1".into() }],
    )
    .unwrap();
    let execution = db
        .execute(
            "a",
            Precondition::Observed(before.revision),
            vec![
                Command::Destroy { id: "e1".into() },
                Command::Keywords {
                    id: "e2".into(),
                    keywords: vec!["$seen".into()],
                },
            ],
        )
        .unwrap();
    assert_eq!(execution.revision, 5);
    let account = db.account("a").unwrap();
    assert_eq!(account.messages.len(), 2);
    assert_eq!(account.messages[0].keywords, vec!["$seen"]);
    // The same command against a fresh observation is a real NotFound.
    assert_eq!(
        db.execute(
            "a",
            Precondition::Observed(5),
            vec![Command::Destroy { id: "e1".into() }]
        ),
        Err(Error::NotFound)
    );
}

#[test]
fn failed_batch_consumes_no_uids_and_writes_nothing() {
    let (db, before) = fixture();
    assert_eq!(
        db.execute(
            "a",
            Precondition::Require(before.revision),
            vec![
                append(&db, 4),
                append(&db, 5),
                Command::Destroy { id: "zz".into() }
            ]
        ),
        Err(Error::NotFound)
    );
    assert_eq!(db.account("a").unwrap(), before);
    assert_eq!(db.blob("a", "e4"), Err(Error::NotFound));
    let execution = db
        .execute(
            "a",
            Precondition::Require(before.revision),
            vec![append(&db, 4)],
        )
        .unwrap();
    assert_eq!(execution.allocations, vec![("inbox".into(), 4)]);
}

#[test]
fn every_ingest_path_rethreads_and_stamps_bridged_messages() {
    for mode in 0..3 {
        let db = SqliteStore::open(":memory:").unwrap();
        db.provision(
            Account::new("a", "t", "alice", "alice@example.org").unwrap(),
            "0123456789abcdef0123456789abcdef",
        )
        .unwrap();
        for id in ["one@t", "two@t"] {
            let old = db.account("a").unwrap();
            db.execute(
                "a",
                Precondition::Require(old.revision),
                vec![
                    db.append(
                        "a",
                        vec!["inbox".into()],
                        &format!("Message-ID: <{id}>\r\nSubject: thread\r\n\r\nbody").into_bytes(),
                        vec![],
                        1,
                    )
                    .unwrap(),
                ],
            )
            .unwrap();
        }
        let before = db.account("a").unwrap();
        assert_ne!(
            before.messages[0].thread_id(),
            before.messages[1].thread_id()
        );
        let raw =
            b"Message-ID: <bridge@t>\r\nReferences: <one@t> <two@t>\r\nSubject: thread\r\n\r\nbody";
        match mode {
            0 => {
                db.execute(
                    "a",
                    Precondition::Require(before.revision),
                    vec![
                        db.append("a", vec!["inbox".into()], raw, vec![], 1)
                            .unwrap(),
                    ],
                )
                .unwrap();
            }
            1 => db.deliver_once("a", "commit-key", raw, 1).unwrap(),
            _ => db.deliver(&["alice@example.org".into()], raw).unwrap(),
        }
        let after = db.account("a").unwrap();
        assert!(
            after
                .messages
                .iter()
                .all(|m| m.thread_id() == after.messages[0].thread_id())
        );
        assert_eq!(after.messages[0].modseq, before.messages[0].modseq);
        assert_eq!(after.messages[1].modseq, after.revision);
        assert_eq!(after.messages[2].modseq, after.revision);
        assert_eq!(after.mail_modseq, after.revision);
        let page = db.history("a", before.revision, 100).unwrap();
        assert!(rows(&page).contains(&&HistoryEntry::Thread { id: "e2".into() }));
        if mode == 1 {
            db.deliver_once("a", "commit-key", raw, 1).unwrap();
            assert_eq!(db.account("a").unwrap(), after);
        }
    }
}
