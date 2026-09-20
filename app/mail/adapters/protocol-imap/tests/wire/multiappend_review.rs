use super::*;
use mail_kernel::Command;

// Count-boundary cases perform over 1000 blocking authorization calls. Their
// functional response budget allows scheduling delays during concurrent gates.
const BULK_RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::test]
async fn refused_nonsync_append_drains_all_declared_bodies_without_executing_payloads() {
    for utf8 in [false, true] {
        let (mut client, task, db) = start(65536).await;
        if utf8 {
            client
                .get_mut()
                .write_all(b"u ENABLE UTF8=ACCEPT\r\n")
                .await
                .unwrap();
            until(&mut client, "u ").await;
        }
        let before = db.account("a").unwrap();
        let body = "p CREATE injected\r\nx LOGOUT\r\n";
        let prefix = if utf8 { "UTF8 (~" } else { "" };
        let end = if utf8 { ")" } else { "" };
        client.get_mut().write_all(format!(
            "b APPEND Missing {prefix}{{{}+}}\r\n{body}{end} {prefix}{{{}+}}\r\n{body}{end}\r\nnext NOOP\r\n", body.len(), body.len()
        ).as_bytes()).await.unwrap();
        let response = until(&mut client, "next ").await;
        assert!(
            response.contains("b NO [TRYCREATE]") && response.contains("next OK"),
            "{response}"
        );
        assert_eq!(response.matches("b NO").count(), 1, "{response}");
        assert!(
            !response.contains("p OK")
                && !response.contains("x OK")
                && !response.contains("+ Ready"),
            "{response}"
        );
        assert_eq!(db.account("a").unwrap(), before);
        finish(client, task).await;
    }
}

#[tokio::test]
async fn quota_refusal_drains_later_nonsync_bodies_and_rolls_back_accepted_prefix() {
    let first = "Subject: accepted prefix\r\n\r\nbody";
    let payload = "p CREATE injected\r\nx LOGOUT\r\n";
    let (mut client, task, db) = start(first.len()).await;
    let before = db.account("a").unwrap();
    client.get_mut().write_all(format!(
        "b APPEND INBOX {{{}+}}\r\n{first} {{{}+}}\r\n{payload} {{0+}}\r\n\r\nnext NOOP\r\n", first.len(), payload.len()
    ).as_bytes()).await.unwrap();
    let response = until(&mut client, "next ").await;
    assert!(
        response.contains("b NO [OVERQUOTA]") && response.contains("next OK"),
        "{response}"
    );
    assert!(
        !response.contains("p OK") && !response.contains("x OK"),
        "{response}"
    );
    assert_eq!(db.account("a").unwrap(), before);
    assert!(db.blob("a", "e1").is_err());
    finish(client, task).await;
}

#[tokio::test]
async fn rejected_nonsync_batch_stops_at_sync_marker_without_asking_for_its_body() {
    let (mut client, task, db) = start(65536).await;
    let before = db.account("a").unwrap();
    client
        .get_mut()
        .write_all(b"b APPEND Missing {1+}\r\nx {200}\r\nnext NOOP\r\n")
        .await
        .unwrap();
    let response = until(&mut client, "next ").await;
    assert!(
        response.contains("b NO [TRYCREATE]") && response.contains("next OK"),
        "{response}"
    );
    assert!(!response.contains("+ Ready"), "{response}");
    assert_eq!(db.account("a").unwrap(), before);
    finish(client, task).await;
}

#[tokio::test]
async fn malformed_literal_after_rejected_body_closes_without_executing_next_bytes() {
    let (mut client, task, db) = start(65536).await;
    let before = db.account("a").unwrap();
    client
        .get_mut()
        .write_all(b"b APPEND Missing {1+}\r\nx {oops+}\r\np CREATE injected\r\nz LOGOUT\r\n")
        .await
        .unwrap();
    let mut response = String::new();
    tokio::time::timeout(Duration::from_secs(5), client.read_to_string(&mut response))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        task.await.unwrap().unwrap_err().kind(),
        std::io::ErrorKind::InvalidData
    );
    assert!(
        !response.contains("p OK") && !response.contains("z OK"),
        "{response}"
    );
    assert_eq!(db.account("a").unwrap(), before);
}

#[tokio::test]
async fn revoked_identity_before_next_literal_is_refused_before_continuation() {
    let raw = b"Subject: discarded\r\n\r\nbody";
    let (mut client, task, db) = start(65536).await;
    let before = db.account("a").unwrap();
    client
        .get_mut()
        .write_all(format!("b APPEND INBOX {{{}}}\r\n", raw.len()).as_bytes())
        .await
        .unwrap();
    assert!(until(&mut client, "+").await.starts_with('+'));
    db.revoke("a").unwrap();
    client.get_mut().write_all(raw).await.unwrap();
    client.get_mut().write_all(b" {1}\r\n").await.unwrap();
    let response = until(&mut client, "b ").await;
    assert!(
        response.contains("b NO") && !response.contains("+ Ready"),
        "{response}"
    );
    assert_eq!(db.account("a").unwrap(), before);
    finish(client, task).await;
}

#[tokio::test]
async fn concurrent_revision_change_before_final_body_reapplies_every_append() {
    let raw = b"Subject: discarded\r\n\r\nbody";
    let (mut client, task, db) = start(65536).await;
    client
        .get_mut()
        .write_all(format!("b APPEND INBOX {{{}+}}\r\n", raw.len()).as_bytes())
        .await
        .unwrap();
    client.get_mut().write_all(raw).await.unwrap();
    client
        .get_mut()
        .write_all(format!(" {{{}}}\r\n", raw.len()).as_bytes())
        .await
        .unwrap();
    assert!(until(&mut client, "+").await.starts_with('+'));
    let current = db.account("a").unwrap();
    let concurrent = db
        .execute(
            "a",
            mail_api::Precondition::Observed(current.revision),
            vec![Command::CreateMailbox {
                name: "Concurrent".into(),
            }],
        )
        .map(|_| db.account("a").unwrap())
        .unwrap();
    client.get_mut().write_all(raw).await.unwrap();
    client
        .get_mut()
        .write_all(b"\r\nnext NOOP\r\n")
        .await
        .unwrap();
    let response = until(&mut client, "next ").await;
    assert!(
        response.contains("b OK [APPENDUID") && response.contains("next OK"),
        "{response}"
    );
    let account = db.account("a").unwrap();
    assert_eq!(account.mailboxes.len(), concurrent.mailboxes.len());
    assert!(account.mailboxes.iter().any(|m| m.name == "Concurrent"));
    assert_eq!(account.messages.len(), 2);
    for message in &account.messages {
        assert_eq!(db.blob("a", &message.id).unwrap(), raw);
    }
    finish(client, task).await;
}

#[tokio::test]
async fn zero_byte_multiappend_batch_has_a_command_count_bound_before_commit() {
    let (mut client, task, db) = start(65536).await;
    let before = db.account("a").unwrap();
    let input = format!(
        "b APPEND INBOX {{0+}}\r\n{}\r\nz LOGOUT\r\n",
        " {0+}\r\n".repeat(1000)
    );
    client.get_mut().write_all(input.as_bytes()).await.unwrap();
    let mut response = String::new();
    tokio::time::timeout(BULK_RESPONSE_TIMEOUT, client.read_to_string(&mut response))
        .await
        .unwrap()
        .unwrap();
    assert!(
        response.contains("b NO") && !response.contains("b OK"),
        "{response}"
    );
    assert_eq!(db.account("a").unwrap(), before);
    let _ = task.await.unwrap();
}

#[tokio::test]
async fn message_count_refusal_precedes_sync_continuation_and_preserves_next_command() {
    let (mut client, task, db) = start(65536).await;
    let before = db.account("a").unwrap();
    let input = format!(
        "b APPEND INBOX {{0+}}\r\n{} {{1}}\r\nnext NOOP\r\n",
        " {0+}\r\n".repeat(999)
    );
    client.get_mut().write_all(input.as_bytes()).await.unwrap();
    let response = until_with_timeout(&mut client, "next ", BULK_RESPONSE_TIMEOUT).await;
    assert!(response.contains("b NO [MESSAGELIMIT 1000]"), "{response}");
    assert!(response.contains("next OK"), "{response}");
    assert!(!response.contains("+ Ready"), "{response}");
    assert_eq!(db.account("a").unwrap(), before);
    assert!(db.blob("a", "e1").is_err());
    finish(client, task).await;
}

/// The writer's side of the batch-lease contract: a store answering `Busy`
/// (a live lease on the hosted tier; SQLite never does) is re-attempted off
/// the worker inside the command's deadline, and the client sees one `OK`.
#[tokio::test]
async fn a_batch_above_the_lease_size_reattempts_busy_off_the_worker_until_commit() {
    use mail_sqlite_store::contract::Faulty;
    // The wrapper owns its handle; identity and queue use a second one.
    let path = std::env::temp_dir().join(format!(
        "mail-lease-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let plain = Arc::new(SqliteStore::open(&path).unwrap());
    plain
        .provision(
            Account::new("a", "t", "alice", "alice@example.org").unwrap(),
            TOKEN,
        )
        .unwrap();
    let db = Arc::new(Faulty::new(SqliteStore::open(&path).unwrap()));
    let service = Arc::new(MailService {
        outbound: None,
        queue: plain.clone(),
        store: db.clone(),
        identity: plain.clone(),
        policy: Arc::new(OwnerPolicy),
    });
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    client
        .get_mut()
        .write_all(format!("a LOGIN alice@example.org {TOKEN}\r\n").as_bytes())
        .await
        .unwrap();
    until(&mut client, "a ").await;
    db.fail_times("execute", mail_kernel::Error::Busy, 3);
    let raw = b"Subject: leased\r\n\r\nbody";
    let mut batch = b"b APPEND INBOX".to_vec();
    for _ in 0..(mail_api::BATCH_LEASE_MESSAGES + 1) {
        batch.extend_from_slice(format!(" {{{}+}}\r\n", raw.len()).as_bytes());
        batch.extend_from_slice(raw);
    }
    batch.extend_from_slice(b"\r\n");
    let started = std::time::Instant::now();
    client.get_mut().write_all(&batch).await.unwrap();
    let response = until_with_timeout(&mut client, "b ", BULK_RESPONSE_TIMEOUT).await;
    assert!(response.contains("b OK [APPENDUID"), "{response}");
    assert!(started.elapsed() < mail_service::MUTATION_DEADLINE);
    assert!(
        !db.armed(),
        "every planned Busy was answered by a re-attempt"
    );
    assert_eq!(
        plain.account("a").unwrap().messages.len(),
        mail_api::BATCH_LEASE_MESSAGES + 1
    );
    finish(client, task).await;
    drop((db, plain));
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(path.with_extension(format!("sqlite{suffix}")));
    }
}
