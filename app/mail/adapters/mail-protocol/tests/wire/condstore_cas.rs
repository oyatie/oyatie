use super::*;
use std::sync::atomic::{AtomicBool, Ordering};

struct ConcurrentWriter {
    db: Arc<SqliteStore>,
    armed: AtomicBool,
}
impl mail_api::Policy for ConcurrentWriter {
    fn authorize(
        &self,
        principal: &mail_api::Principal,
        action: mail_api::Action,
        account: &mail_api::AccountInfo,
    ) -> Result<(), mail_kernel::Error> {
        mail_api::Policy::authorize(&OwnerPolicy, principal, action, account)?;
        if action == mail_api::Action::Write && self.armed.swap(false, Ordering::SeqCst) {
            let current = self.db.account("a")?;
            self.db.execute(
                "a",
                current.revision,
                vec![Command::Keywords {
                    id: current.messages[0].id.clone(),
                    keywords: vec!["concurrent".into()],
                }],
            )?;
        }
        Ok(())
    }
}

#[tokio::test]
async fn conditional_store_rechecks_cas_when_another_writer_changes_flags_after_snapshot() {
    let (_, db) = service();
    messages(&db);
    let policy = Arc::new(ConcurrentWriter {
        db: db.clone(),
        armed: AtomicBool::new(false),
    });
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: policy.clone(),
    });
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    send(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    send(&mut client, "s SELECT INBOX (CONDSTORE)").await;
    let before = db.account("a").unwrap();
    policy.armed.store(true, Ordering::SeqCst);
    let response = send(
        &mut client,
        &format!(
            "c UID STORE 1 (UNCHANGEDSINCE {}) +FLAGS.SILENT (\\Deleted)",
            before.mail_modseq
        ),
    )
    .await;
    assert!(
        response.contains("c NO") && !response.contains("c OK"),
        "{response}"
    );
    let after = db.account("a").unwrap();
    assert_eq!(after.messages[0].keywords, ["concurrent"]);
    assert!(after.messages[0].modseq > before.mail_modseq);
    assert_eq!(after.revision, before.revision + 1);
    let response = send(&mut client, "n NOOP").await;
    assert!(
        response.contains("FLAGS (concurrent)") && response.contains("MODSEQ ("),
        "{response}"
    );
    finish(client, task).await;
}

#[tokio::test]
async fn condstore_idle_reports_one_modseq_notification_for_each_flag_change() {
    let (_, db) = service();
    messages(&db);
    let (mut client, task) = start(db.clone()).await;
    client.get_mut().write_all(b"i IDLE\r\n").await.unwrap();
    let mut initial = String::new();
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            let mut line = String::new();
            assert!(client.read_line(&mut line).await.unwrap() > 0);
            let done = line.starts_with("+ Idling");
            initial.push_str(&line);
            if done {
                break;
            }
        }
    })
    .await
    .unwrap();
    assert_eq!(initial.matches("FETCH (").count(), 3, "{initial}");
    assert_eq!(initial.matches("MODSEQ (").count(), 3, "{initial}");
    let state = db.account("a").unwrap();
    let state = db
        .execute(
            "a",
            state.revision,
            vec![Command::Keywords {
                id: state.messages[0].id.clone(),
                keywords: vec!["external".into()],
            }],
        )
        .unwrap();
    let mut notification = String::new();
    tokio::time::timeout(Duration::from_secs(3), client.read_line(&mut notification))
        .await
        .unwrap()
        .unwrap();
    assert!(
        notification.contains("UID 1 FLAGS (external)")
            && notification.contains(&format!("MODSEQ ({})", state.messages[0].modseq)),
        "{notification}"
    );
    client.get_mut().write_all(b"DONE\r\n").await.unwrap();
    let tail = tokio::time::timeout(Duration::from_secs(3), read(&mut client, "i"))
        .await
        .unwrap();
    assert!(!tail.contains("FETCH ("), "duplicated notification: {tail}");
    let noop = send(&mut client, "n NOOP").await;
    assert!(
        !noop.contains("FETCH ("),
        "repeated idle notification: {noop}"
    );
    finish(client, task).await;
}

#[tokio::test]
async fn conditional_uid_wildcard_uses_same_existing_maximum_for_store_and_modified() {
    let (_, db) = service();
    messages(&db);
    let before = db.account("a").unwrap();
    db.execute(
        "a",
        before.revision,
        vec![Command::Destroy {
            id: before.messages[2].id.clone(),
        }],
    )
    .unwrap();
    let (mut client, task) = start(db.clone()).await;
    let response = send(
        &mut client,
        "c UID STORE * (UNCHANGEDSINCE 18446744073709551615) +FLAGS.SILENT (\\Seen)",
    )
    .await;
    assert!(
        response.contains("c OK") && !response.contains("MODIFIED"),
        "{response}"
    );
    assert_eq!(db.account("a").unwrap().messages[1].keywords, ["$seen"]);
    let response = send(
        &mut client,
        "c UID STORE * (UNCHANGEDSINCE 0) +FLAGS.SILENT (\\Deleted)",
    )
    .await;
    assert!(response.contains("[MODIFIED 2]"), "{response}");
    finish(client, task).await;
}

#[tokio::test]
async fn changed_since_excludes_deletions_older_than_observed_message_modseq_within_batch() {
    let (_, db) = service();
    messages(&db);
    let before = db.account("a").unwrap();
    let after = db
        .execute(
            "a",
            before.revision,
            vec![
                Command::Destroy {
                    id: before.messages[0].id.clone(),
                },
                Command::Keywords {
                    id: before.messages[1].id.clone(),
                    keywords: vec!["second".into()],
                },
                Command::Keywords {
                    id: before.messages[2].id.clone(),
                    keywords: vec!["third".into()],
                },
            ],
        )
        .unwrap();
    let threshold = after.messages[0].modseq;
    let (mut client, task) = start(db.clone()).await;
    let response = send(
        &mut client,
        &format!("f UID FETCH 1:3 (FLAGS) (CHANGEDSINCE {threshold} VANISHED)"),
    )
    .await;
    assert!(
        response.contains("f OK") && !response.contains("VANISHED (EARLIER)"),
        "{response}"
    );
    assert!(!response.contains("UID 2 FLAGS"), "{response}");
    finish(client, task).await;
}

#[tokio::test]
async fn missing_history_commit_refuses_resync_even_when_message_rows_survive() {
    let path = std::env::temp_dir().join(format!(
        "mail-condstore-gap-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let db = Arc::new(SqliteStore::open(&path).unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    messages(&db);
    let before = db.account("a").unwrap();
    let after = db
        .execute(
            "a",
            before.revision,
            vec![Command::Destroy {
                id: before.messages[1].id.clone(),
            }],
        )
        .unwrap();
    let raw = rusqlite::Connection::open(&path).unwrap();
    raw.execute(
        "DELETE FROM history_commits WHERE account='a' AND revision=?1",
        [after.revision],
    )
    .unwrap();
    drop(raw);
    let (mut client, task) = start(db.clone()).await;
    let response = send(
        &mut client,
        &format!(
            "f UID FETCH 1:3 (FLAGS) (CHANGEDSINCE {} VANISHED)",
            before.mail_modseq
        ),
    )
    .await;
    assert!(
        response.contains("f NO") && !response.contains("f OK"),
        "{response}"
    );
    finish(client, task).await;
    drop(db);
    std::fs::remove_file(path).unwrap();
}
