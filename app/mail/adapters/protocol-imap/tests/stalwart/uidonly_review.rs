use super::{ImapConnection, ResponseType, Type, client, fixture};
use mail_kernel::Command;
async fn request(client: &mut ImapConnection, text: &str, kind: ResponseType) -> String {
    client.send(text).await;
    let lines = client.assert_read(Type::Tagged, kind).await;
    let expected = format!("{kind:?}").to_ascii_uppercase();
    assert_eq!(
        lines.last().unwrap().split_ascii_whitespace().nth(1),
        Some(expected.as_str()),
        "{lines:?}"
    );
    lines.join("\n")
}
async fn connect() -> ImapConnection {
    let mut client = ImapConnection::connect(b"review").await;
    client.assert_read(Type::Untagged, ResponseType::Ok).await;
    let response = request(
        &mut client,
        &format!("LOGIN alice@example.org {}", super::TOKEN),
        ResponseType::Ok,
    )
    .await;
    assert!(response.contains(" OK "), "{response}");
    client
}
#[tokio::test]
async fn malformed_qresync_is_fully_validated_before_uidonly_operation_refusal() {
    let service = fixture();
    client::initialize(service.clone());
    let mut client = connect().await;
    request(&mut client, "EXAMINE INBOX", ResponseType::Ok).await;
    request(&mut client, "ENABLE UIDONLY QRESYNC", ResponseType::Ok).await;
    let before = service.store.account("a").unwrap();
    let mut wrong = vec![];
    for command in [
        "SELECT INBOX (QRESYNC (1 1 (bogus 1)))",
        "SELECT INBOX (QRESYNC (1 1 (1 bogus)))",
        "SELECT INBOX (QRESYNC (1 1 (1 1)) UNKNOWN)",
        "SELECT INBOX (QRESYNC (1 1 (1 1)) CONDSTORE UNKNOWN)",
        "SELECT INBOX (QRESYNC (1 1 (1: 1)))",
    ] {
        let response = request(&mut client, command, ResponseType::Bad).await;
        if !response.contains(" BAD ") || response.contains("[UIDREQUIRED]") {
            wrong.push((command, response));
        }
        let response = request(
            &mut client,
            "UID STORE 1 +FLAGS (\\Deleted)",
            ResponseType::No,
        )
        .await;
        assert!(
            response.contains(" NO "),
            "failed SELECT must retain readonly selection: {response}"
        );
        assert_eq!(service.store.account("a").unwrap(), before);
    }
    client.close().await;
    assert!(
        wrong.is_empty(),
        "parser errors must precede UIDONLY semantic check: {wrong:?}"
    );
}
#[tokio::test]
async fn uidonly_rejects_disallowed_command_classes_even_without_arguments() {
    let service = fixture();
    client::initialize(service.clone());
    let mut client = connect().await;
    request(&mut client, "SELECT INBOX", ResponseType::Ok).await;
    request(&mut client, "ENABLE UIDONLY", ResponseType::Ok).await;
    let before = service.store.account("a").unwrap();
    let mut wrong = vec![];
    for command in ["FETCH", "STORE", "SEARCH", "COPY", "MOVE", "SORT", "THREAD"] {
        let response = request(&mut client, command, ResponseType::Bad).await;
        if !response.contains("BAD [UIDREQUIRED]") {
            wrong.push((command, response));
        }
    }
    assert_eq!(service.store.account("a").unwrap(), before);
    client.close().await;
    assert!(
        wrong.is_empty(),
        "upstream rejects command class before argument parsing: {wrong:?}"
    );
}
#[tokio::test]
async fn nonexistent_mailbox_refusal_precedes_valid_qresync_uidonly_operation_check() {
    client::initialize(fixture());
    let mut client = connect().await;
    request(&mut client, "EXAMINE INBOX", ResponseType::Ok).await;
    request(&mut client, "ENABLE UIDONLY QRESYNC", ResponseType::Ok).await;
    let response = request(
        &mut client,
        "SELECT Missing (QRESYNC (1 1 (1 1)))",
        ResponseType::No,
    )
    .await;
    assert!(
        response.contains(" NO ") && !response.contains("UIDREQUIRED"),
        "{response}"
    );
    let response = request(
        &mut client,
        "UID STORE 1 +FLAGS (\\Deleted)",
        ResponseType::No,
    )
    .await;
    assert!(response.contains(" NO "), "{response}");
    client.close().await;
}
#[tokio::test]
async fn unauthenticate_resets_all_negotiated_extensions_and_refuses_stale_selection() {
    let service = fixture();
    client::initialize(service.clone());
    let mut client = connect().await;
    request(
        &mut client,
        "ENABLE UIDONLY QRESYNC OBJECTID+ UTF8=ACCEPT",
        ResponseType::Ok,
    )
    .await;
    request(&mut client, "SELECT INBOX", ResponseType::Ok).await;
    request(
        &mut client,
        "UID SEARCH RETURN (SAVE) ALL",
        ResponseType::Ok,
    )
    .await;
    request(&mut client, "UNAUTHENTICATE", ResponseType::Ok).await;
    let caps = request(&mut client, "CAPABILITY", ResponseType::Ok).await;
    assert!(
        !caps.contains("UIDONLY") && !caps.contains("UNAUTHENTICATE"),
        "{caps}"
    );
    let response = request(&mut client, "UID FETCH $ (FLAGS)", ResponseType::No).await;
    assert!(
        !response.contains("FETCH (") && response.contains(" NO "),
        "{response}"
    );
    request(
        &mut client,
        &format!("LOGIN alice@example.org {}", super::TOKEN),
        ResponseType::Ok,
    )
    .await;
    let response = request(&mut client, "SELECT INBOX", ResponseType::Ok).await;
    assert!(
        !response.contains("OBJECTID") && !response.contains("HIGHESTMODSEQ"),
        "{response}"
    );
    let response = request(&mut client, "FETCH 1 (FLAGS)", ResponseType::Ok).await;
    assert!(
        response.contains("* 1 FETCH (")
            && !response.contains("UIDFETCH")
            && !response.contains("MODSEQ"),
        "{response}"
    );
    assert!(
        !request(&mut client, "UID FETCH $ FLAGS", ResponseType::Ok)
            .await
            .contains("FETCH (")
    );
    let account = service.store.account("a").unwrap();
    service
        .store
        .execute(
            "a",
            account.revision,
            vec![Command::Destroy {
                id: account.messages[0].id.clone(),
            }],
        )
        .unwrap();
    let response = request(&mut client, "NOOP", ResponseType::Ok).await;
    assert!(
        response.contains("* 1 EXPUNGE") && !response.contains("VANISHED"),
        "{response}"
    );
    let response = request(&mut client, "UID FETCH 2 (FLAGS)", ResponseType::Ok).await;
    assert!(
        response.contains("* 1 FETCH (UID 2") && !response.contains("UIDFETCH"),
        "{response}"
    );
    client.close().await;
}
#[tokio::test]
async fn uidonly_uidsearch_parser_errors_precede_sequence_criterion_refusal() {
    client::initialize(fixture());
    let mut client = connect().await;
    request(&mut client, "SELECT INBOX", ResponseType::Ok).await;
    request(&mut client, "ENABLE UIDONLY", ResponseType::Ok).await;
    for command in [
        "UID SEARCH 1 UNKNOWN",
        "UID SEARCH OR 1",
        "UID SEARCH (1) HEADER",
        "UID SORT (ARRIVAL) UTF-8 1 UNKNOWN",
        "UID THREAD REFERENCES UTF-8 1 UNKNOWN",
    ] {
        let response = request(&mut client, command, ResponseType::Bad).await;
        assert!(
            response.contains(" BAD ") && !response.contains("UIDREQUIRED"),
            "{command}: {response}"
        );
    }
    let response = request(
        &mut client,
        "UID SEARCH HEADER Subject \"1\"",
        ResponseType::Ok,
    )
    .await;
    assert!(response.contains(" OK "), "{response}");
    client.close().await;
}
#[tokio::test]
async fn uidonly_idle_rechecks_revocation_before_reporting_new_message_metadata() {
    use mail_api::Store;
    use mail_kernel::Account;
    use mail_service::{MailService, OwnerPolicy};
    use mail_sqlite_store::SqliteStore;
    use std::{sync::Arc, time::Duration};
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        super::TOKEN,
    )
    .unwrap();
    for _ in 0..2 {
        db.deliver(
            &["alice@example.org".into()],
            b"Subject: private\r\n\r\nprivate",
        )
        .unwrap();
    }
    let account = db.account("a").unwrap();
    db.execute(
        "a",
        account.revision,
        vec![Command::Destroy {
            id: account.messages[0].id.clone(),
        }],
    )
    .unwrap();
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: Arc::new(OwnerPolicy),
    });
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    client
        .get_mut()
        .write_all(
            format!(
                "a LOGIN alice@example.org {}\r\nb SELECT INBOX\r\nc ENABLE UIDONLY\r\nd IDLE\r\n",
                super::TOKEN
            )
            .as_bytes(),
        )
        .await
        .unwrap();
    let initial = tokio::time::timeout(Duration::from_secs(5), async {
        let mut response = String::new();
        loop {
            let mut line = String::new();
            assert!(client.read_line(&mut line).await.unwrap() > 0);
            let done = line.starts_with('+');
            response.push_str(&line);
            if done {
                return response;
            }
        }
    })
    .await
    .unwrap();
    assert!(
        initial.contains("* 2 UIDFETCH (UID 2 FLAGS ())"),
        "{initial}"
    );
    db.revoke("a").unwrap();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: withheld\r\n\r\nnew private",
    )
    .unwrap();
    let mut response = String::new();
    tokio::time::timeout(Duration::from_secs(5), client.read_to_string(&mut response))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        response,
        "* BYE IDLE authorization or storage unavailable\r\n"
    );
    task.await.unwrap().unwrap();
}
