use super::*;
use mail_kernel::Command;
use std::time::Duration;
type Client = BufReader<tokio::io::DuplexStream>;
type Task = tokio::task::JoinHandle<std::io::Result<()>>;

async fn send(client: &mut Client, request: &str) -> String {
    tokio::time::timeout(Duration::from_secs(3), command(client, request))
        .await
        .expect("CONDSTORE review deadline")
}
async fn start(db: Arc<SqliteStore>) -> (Client, Task) {
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db,
        policy: Arc::new(OwnerPolicy),
    });
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    assert!(
        send(&mut client, &format!("a LOGIN alice@example.org {TOKEN}"))
            .await
            .contains("a OK")
    );
    assert!(send(&mut client, "e ENABLE QRESYNC").await.contains("e OK"));
    assert!(send(&mut client, "s SELECT INBOX").await.contains("s OK"));
    (client, task)
}
fn messages(db: &SqliteStore) {
    for _ in 0..3 {
        db.deliver(
            &["alice@example.org".into()],
            b"Subject: retained\r\n\r\nbody",
        )
        .unwrap();
    }
}
async fn finish(mut client: Client, task: Task) {
    send(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn conditional_saved_sequence_store_reports_only_failed_sequence_numbers() {
    let (_, db) = service();
    messages(&db);
    let (mut client, task) = start(db.clone()).await;
    let state = db.account("a").unwrap();
    db.execute(
        "a",
        mail_api::Precondition::Observed(state.revision),
        vec![Command::Destroy {
            id: state.messages[0].id.clone(),
        }],
    )
    .unwrap();
    send(&mut client, "n NOOP").await;
    send(&mut client, "q SEARCH RETURN (SAVE) UID 3").await;
    let response = send(
        &mut client,
        "c STORE $ (UNCHANGEDSINCE 18446744073709551615) +FLAGS.SILENT (\\Seen)",
    )
    .await;
    assert!(
        response.contains("c OK") && !response.contains("MODIFIED"),
        "{response}"
    );
    assert_eq!(db.account("a").unwrap().messages[1].keywords, ["$seen"]);
    let response = send(
        &mut client,
        "c STORE $ (UNCHANGEDSINCE 0) +FLAGS.SILENT (\\Deleted)",
    )
    .await;
    assert!(response.contains("[MODIFIED 2]"), "{response}");
    finish(client, task).await;
}

#[tokio::test]
async fn vanished_matches_oracle_mailbox_tombstones_even_outside_fetch_uid_set() {
    let (_, db) = service();
    messages(&db);
    let (mut client, task) = start(db.clone()).await;
    let state = db.account("a").unwrap();
    db.execute(
        "a",
        mail_api::Precondition::Observed(state.revision),
        vec![Command::Destroy {
            id: state.messages[1].id.clone(),
        }],
    )
    .unwrap();
    let response = send(
        &mut client,
        &format!(
            "f UID FETCH 1 (FLAGS) (CHANGEDSINCE {} VANISHED)",
            state.mail_modseq
        ),
    )
    .await;
    assert!(
        response.contains("f OK") && response.contains("VANISHED (EARLIER) 2"),
        "{response}"
    );
    finish(client, task).await;
}

#[tokio::test]
async fn arbitrary_modseq_inside_committed_batch_can_read_vanished_history() {
    let (_, db) = service();
    messages(&db);
    let state = db.account("a").unwrap();
    // A two-command batch advances the revision twice and stamps both
    // messages with the final revision, so the first one is a MODSEQ value
    // inside the committed batch that no record carries.
    let threshold = state.revision + 1;
    let state = db
        .execute(
            "a",
            mail_api::Precondition::Observed(state.revision),
            vec![
                Command::Keywords {
                    id: state.messages[0].id.clone(),
                    keywords: vec!["$seen".into()],
                },
                Command::Keywords {
                    id: state.messages[1].id.clone(),
                    keywords: vec!["$seen".into()],
                },
            ],
        )
        .map(|_| db.account("a").unwrap())
        .unwrap();
    db.execute(
        "a",
        mail_api::Precondition::Observed(state.revision),
        vec![Command::Destroy {
            id: state.messages[2].id.clone(),
        }],
    )
    .unwrap();
    let (mut client, task) = start(db.clone()).await;
    let response = send(
        &mut client,
        &format!("f UID FETCH 1:3 (FLAGS) (CHANGEDSINCE {threshold} VANISHED)"),
    )
    .await;
    assert!(
        response.contains("f OK") && response.contains("VANISHED (EARLIER) 3"),
        "{response}"
    );
    assert!(
        response.contains("UID 2") && response.contains("UID 1 FLAGS"),
        "{response}"
    );
    finish(client, task).await;
}

#[tokio::test]
async fn corrupt_removed_history_row_refuses_resync_instead_of_returning_incomplete_success() {
    let name = format!(
        "mail-condstore-review-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let path = std::env::temp_dir().join(name);
    let db = Arc::new(SqliteStore::open(&path).unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    messages(&db);
    let state = db.account("a").unwrap();
    db.execute(
        "a",
        mail_api::Precondition::Observed(state.revision),
        vec![Command::Destroy {
            id: state.messages[1].id.clone(),
        }],
    )
    .unwrap();
    let raw = rusqlite::Connection::open(&path).unwrap();
    raw.execute(
        "UPDATE history SET uid=NULL WHERE account='a' AND kind='removed'",
        [],
    )
    .unwrap();
    drop(raw);
    let (mut client, task) = start(db.clone()).await;
    let response = send(
        &mut client,
        &format!(
            "f UID FETCH 1:3 (FLAGS) (CHANGEDSINCE {} VANISHED)",
            state.mail_modseq
        ),
    )
    .await;
    assert!(
        response.contains("f NO") && !response.contains("f OK"),
        "{response}"
    );
    let validity = state.mailboxes[0].uid_validity;
    let response = send(
        &mut client,
        &format!(
            "s SELECT INBOX (QRESYNC ({validity} {}))",
            state.mail_modseq
        ),
    )
    .await;
    assert!(
        response.contains("s NO") && !response.contains("s OK"),
        "{response}"
    );
    finish(client, task).await;
    drop(db);
    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn malformed_conditional_commands_and_readonly_or_revoked_access_do_not_mutate() {
    let (_, db) = service();
    messages(&db);
    let (mut client, task) = start(db.clone()).await;
    let before = db.account("a").unwrap();
    assert!(
        send(&mut client, "q SEARCH MODSEQ \"1\"")
            .await
            .contains("q OK")
    );
    for request in [
        "STORE 1 (UNCHANGEDSINCE 0 extra) +FLAGS (\\Deleted)",
        "STORE 1 (UNCHANGEDSINCE 18446744073709551616) +FLAGS (\\Deleted)",
        "STORE 1 (UNCHANGEDSINCE 100) +FLAGS (\\Deleted) extra",
        "UID FETCH 1 (FLAGS) (CHANGEDSINCE 0 VANISHED extra)",
        "FETCH 1 (FLAGS) (CHANGEDSINCE 0 VANISHED)",
        "SELECT INBOX (QRESYNC (1 1 1::2))",
        "SEARCH MODSEQ \"bogus\"",
    ] {
        let response = send(&mut client, &format!("b {request}")).await;
        assert!(response.contains("b BAD"), "{request}: {response}");
        assert_eq!(db.account("a").unwrap(), before);
        assert!(send(&mut client, "v FETCH 1 (UID)").await.contains("v OK"));
    }
    let response = send(
        &mut client,
        "c STORE 0:4294967295 (UNCHANGEDSINCE 0) +FLAGS (\\Deleted)",
    )
    .await;
    assert!(
        response.contains("c OK") && response.len() < 1024,
        "{response}"
    );
    assert_eq!(db.account("a").unwrap().messages, before.messages);
    send(&mut client, "x EXAMINE INBOX").await;
    let response = send(
        &mut client,
        "c STORE 1 (UNCHANGEDSINCE 18446744073709551615) +FLAGS (\\Deleted)",
    )
    .await;
    assert!(response.contains("c NO"), "{response}");
    db.revoke("a").unwrap();
    let response = send(
        &mut client,
        "f UID FETCH 1 (FLAGS) (CHANGEDSINCE 0 VANISHED)",
    )
    .await;
    assert!(
        response.contains("f NO") && !response.contains("FETCH ("),
        "{response}"
    );
    assert_eq!(db.account("a").unwrap().messages, before.messages);
    finish(client, task).await;
}

#[path = "condstore_cas.rs"]
mod cas;
