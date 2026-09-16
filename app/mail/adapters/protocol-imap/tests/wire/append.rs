use super::*;

async fn login(client: &mut BufReader<tokio::io::DuplexStream>) {
    client
        .get_mut()
        .write_all(format!("a LOGIN alice@example.org {TOKEN}\r\n").as_bytes())
        .await
        .unwrap();
    until(client, "a ").await;
}
async fn until(client: &mut BufReader<tokio::io::DuplexStream>, prefix: &str) -> String {
    let mut all = String::new();
    loop {
        let mut line = String::new();
        assert!(client.read_line(&mut line).await.unwrap() > 0);
        all.push_str(&line);
        if line.starts_with(prefix) {
            return all;
        }
    }
}

#[tokio::test]
async fn append_commits_literal_flags_and_date_before_acknowledgement() {
    let (service, db) = service();
    let (client, server) = tokio::io::duplex(4096);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    login(&mut client).await;
    client
        .get_mut()
        .write_all(b"b SELECT INBOX\r\n")
        .await
        .unwrap();
    until(&mut client, "b ").await;
    let raw = b"Subject: uploaded\r\n\r\n..body\r\n";
    client
        .get_mut()
        .write_all(
            format!(
                "c APPEND INBOX (\\Seen custom) \"09-Sep-2001 01:46:40 +0000\" {{{}}}\r\n",
                raw.len()
            )
            .as_bytes(),
        )
        .await
        .unwrap();
    let mut continuation = String::new();
    client.read_line(&mut continuation).await.unwrap();
    assert!(continuation.starts_with('+'), "{continuation}");
    assert!(db.account("a").unwrap().messages.is_empty());
    client.get_mut().write_all(raw).await.unwrap();
    client
        .get_mut()
        .write_all(b"\r\nd UID FETCH 1 (UID FLAGS INTERNALDATE BODY.PEEK[])\r\ne LOGOUT\r\n")
        .await
        .unwrap();
    let mut transcript = String::new();
    client.read_to_string(&mut transcript).await.unwrap();
    task.await.unwrap().unwrap();
    assert!(transcript.contains("c OK [APPENDUID 1 1]"), "{transcript}");
    assert!(transcript.contains("* 1 EXISTS"), "{transcript}");
    assert!(transcript.contains("d OK"), "{transcript}");
    let account = db.account("a").unwrap();
    let message = &account.messages[0];
    assert_eq!(message.received_at, 1_000_000_000);
    assert_eq!(message.keywords, ["$seen", "custom"]);
    assert_eq!(db.blob("a", &message.id).unwrap(), raw);
}

#[tokio::test]
async fn append_refuses_bad_metadata_before_continuation_and_revocation_before_commit() {
    let (service, db) = service();
    let (client, server) = tokio::io::duplex(4096);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    login(&mut client).await;
    for (command, expected) in [
        ("b APPEND Missing {5}\r\n", "NO [TRYCREATE]"),
        ("b APPEND INBOX {26214401}\r\n", "NO [TOOBIG]"),
        ("b APPEND INBOX (\\Recent) {5}\r\n", "BAD"),
        (
            "b APPEND INBOX \"31-Feb-2026 01:02:03 +0000\" {5}\r\n",
            "BAD",
        ),
        (
            "b APPEND INBOX \"09-Sep-2001 01:46:40 +2460\" {5}\r\n",
            "BAD",
        ),
        (
            "b APPEND INBOX \"09-Sep-2001 01:46:40 +9999\" {5}\r\n",
            "BAD",
        ),
        (
            "b APPEND INBOX \"09-Sep-0001 01:46:40 +0000\" {5}\r\n",
            "BAD",
        ),
    ] {
        client
            .get_mut()
            .write_all(command.as_bytes())
            .await
            .unwrap();
        let mut response = String::new();
        client.read_line(&mut response).await.unwrap();
        assert!(
            response.starts_with(&format!("b {expected}")),
            "{command}: {response}"
        );
    }
    let raw = b"Subject: refused\r\n\r\nbody\r\n";
    client
        .get_mut()
        .write_all(format!("c APPEND INBOX {{{}}}\r\n", raw.len()).as_bytes())
        .await
        .unwrap();
    let mut continuation = String::new();
    client.read_line(&mut continuation).await.unwrap();
    assert!(continuation.starts_with('+'), "{continuation}");
    db.revoke("a").unwrap();
    client.get_mut().write_all(raw).await.unwrap();
    client
        .get_mut()
        .write_all(b"\r\nd LOGOUT\r\n")
        .await
        .unwrap();
    let mut transcript = String::new();
    client.read_to_string(&mut transcript).await.unwrap();
    task.await.unwrap().unwrap();
    assert!(transcript.contains("c NO"), "{transcript}");
    assert!(db.account("a").unwrap().messages.is_empty());
}

#[tokio::test]
async fn append_discards_truncated_literal_without_mutation() {
    let (service, db) = service();
    let (client, server) = tokio::io::duplex(4096);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    login(&mut client).await;
    client
        .get_mut()
        .write_all(b"b APPEND INBOX {100}\r\n")
        .await
        .unwrap();
    let mut continuation = String::new();
    client.read_line(&mut continuation).await.unwrap();
    assert!(continuation.starts_with('+'), "{continuation}");
    client.get_mut().write_all(b"short").await.unwrap();
    client.get_mut().shutdown().await.unwrap();
    assert_eq!(
        task.await.unwrap().unwrap_err().kind(),
        std::io::ErrorKind::UnexpectedEof
    );
    assert!(db.account("a").unwrap().messages.is_empty());
}

#[tokio::test]
async fn append_rechecks_revision_and_rejects_invalid_literal_terminators() {
    for conflict in [true, false] {
        let (service, db) = service();
        let (client, server) = tokio::io::duplex(4096);
        let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
        let mut client = BufReader::new(client);
        login(&mut client).await;
        let raw = b"Subject: atomic\r\n\r\nbody\r\n";
        client
            .get_mut()
            .write_all(format!("b APPEND INBOX {{{}}}\r\n", raw.len()).as_bytes())
            .await
            .unwrap();
        let mut continuation = String::new();
        client.read_line(&mut continuation).await.unwrap();
        assert!(continuation.starts_with('+'));
        if conflict {
            db.execute(
                "a",
                0,
                vec![mail_kernel::Command::CreateMailbox {
                    name: "Concurrent".into(),
                }],
            )
            .unwrap();
        }
        client.get_mut().write_all(raw).await.unwrap();
        client
            .get_mut()
            .write_all(if conflict {
                b"\r\nc LOGOUT\r\n"
            } else {
                b"xxc LOGOUT\r\n"
            })
            .await
            .unwrap();
        if conflict {
            let mut result = String::new();
            client.read_to_string(&mut result).await.unwrap();
            assert!(
                result.contains("b NO") && result.contains("c OK"),
                "{result}"
            );
            task.await.unwrap().unwrap();
            assert_eq!(db.account("a").unwrap().mailboxes.len(), 2);
        } else {
            assert_eq!(
                task.await.unwrap().unwrap_err().kind(),
                std::io::ErrorKind::InvalidData
            );
        }
        assert!(db.account("a").unwrap().messages.is_empty());
    }
}

#[tokio::test]
async fn append_requires_write_permission_before_requesting_literal() {
    struct ReadOnly;
    impl mail_api::Policy for ReadOnly {
        fn authorize(
            &self,
            _: &mail_api::Principal,
            action: mail_api::Action,
            _: &mail_api::AccountInfo,
        ) -> Result<(), mail_kernel::Error> {
            if action == mail_api::Action::Read {
                Ok(())
            } else {
                Err(mail_kernel::Error::Forbidden)
            }
        }
    }
    let (_, db) = service();
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: Arc::new(ReadOnly),
    });
    let (client, server) = tokio::io::duplex(4096);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    login(&mut client).await;
    client
        .get_mut()
        .write_all(b"b APPEND INBOX {20}\r\nc LOGOUT\r\n")
        .await
        .unwrap();
    let mut result = String::new();
    client.read_to_string(&mut result).await.unwrap();
    task.await.unwrap().unwrap();
    assert!(result.starts_with("b NO"), "{result}");
    assert!(!result.contains("+ Ready"));
    assert!(db.account("a").unwrap().messages.is_empty());
}
