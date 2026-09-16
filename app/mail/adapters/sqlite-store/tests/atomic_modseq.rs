use mail_api::Store;
use mail_kernel::{Account, Command};
use mail_sqlite_store::SqliteStore;
fn append(n: u8) -> Command {
    Command::Append {
        mailboxes: vec!["inbox".into()],
        raw: format!("Subject: {n}\r\n\r\nbody").into_bytes(),
        keywords: vec![],
        received_at: 1,
    }
}
fn fixture() -> (SqliteStore, Account) {
    let db = SqliteStore::open(":memory:").unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        "0123456789abcdef0123456789abcdef",
    )
    .unwrap();
    let account = db
        .execute("a", 0, vec![append(1), append(2), append(3)])
        .unwrap();
    (db, account)
}
#[test]
fn atomic_message_changes_share_final_commit_watermark_after_mailbox_only_tail() {
    let (db, before) = fixture();
    let account = db
        .execute(
            "a",
            before.revision,
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
    assert_eq!(account.mail_modseq, account.revision + 1);
    assert!(
        account
            .messages
            .iter()
            .all(|m| m.modseq == account.mail_modseq)
    );
    assert_eq!(db.account("a").unwrap(), account);
    let history = db
        .message_changes("a", before.revision, account.revision)
        .unwrap();
    assert_eq!(history.len(), 3);
    assert_eq!(history.iter().filter(|c| c.after.is_none()).count(), 1);
    assert!(
        history
            .iter()
            .filter_map(|c| c.after.as_ref())
            .all(|m| m.modseq == account.mail_modseq)
    );
}
#[test]
fn cancelled_message_edits_and_mailbox_only_changes_do_not_advance_watermark() {
    let (db, before) = fixture();
    let id = before.messages[0].id.clone();
    let account = db
        .execute(
            "a",
            before.revision,
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
    assert_eq!(account.mail_modseq, before.mail_modseq);
    assert_eq!(account.messages, before.messages);
    assert!(
        db.message_changes("a", before.revision, account.revision)
            .unwrap()
            .is_empty()
    );
}
#[test]
fn multi_message_append_uses_one_visible_stamp() {
    let (_, account) = fixture();
    assert!(
        account
            .messages
            .iter()
            .all(|m| m.modseq == account.revision + 1)
    );
}

#[test]
fn every_save_path_stamps_existing_messages_changed_by_thread_linking() {
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
                old.revision,
                vec![Command::Append {
                    mailboxes: vec!["inbox".into()],
                    keywords: vec![],
                    received_at: 1,
                    raw: format!("Message-ID: <{id}>\r\nSubject: thread\r\n\r\nbody").into_bytes(),
                }],
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
                    before.revision,
                    vec![Command::Append {
                        mailboxes: vec!["inbox".into()],
                        keywords: vec![],
                        received_at: 1,
                        raw: raw.to_vec(),
                    }],
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
        assert_eq!(after.messages[1].modseq, after.mail_modseq);
        assert_eq!(after.messages[2].modseq, after.mail_modseq);
        assert_eq!(after.mail_modseq, after.revision + 1);
        if mode == 1 {
            db.deliver_once("a", "commit-key", raw, 1).unwrap();
            assert_eq!(db.account("a").unwrap(), after);
        }
    }
}
