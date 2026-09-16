use super::*;

#[tokio::test]
async fn imap_copy_and_move_preserve_content_with_distinct_destination_uids() {
    let (service, db) = service();
    let raw = b"Message-ID: <copy@example.org>\r\nSubject: copied\r\n\r\nbody\r\n";
    let account = db
        .execute(
            "a",
            0,
            vec![
                mail_kernel::Command::CreateMailbox {
                    name: "Archive".into(),
                },
                mail_kernel::Command::Append {
                    mailboxes: vec!["inbox".into()],
                    raw: raw.to_vec(),
                    keywords: vec!["$seen".into()],
                    received_at: 1_000_000_000,
                },
            ],
        )
        .unwrap();
    let original = &account.messages[0];
    let (mut client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    client.write_all(format!("a LOGIN alice@example.org {TOKEN}\r\nb SELECT INBOX\r\nc COPY 1 Archive\r\nd UID COPY 1 Archive\r\ne UID MOVE 1 Archive\r\nf SELECT Archive\r\ng UID FETCH 1:* (FLAGS INTERNALDATE BODY.PEEK[])\r\nh LOGOUT\r\n").as_bytes()).await.unwrap();
    let mut transcript = String::new();
    client.read_to_string(&mut transcript).await.unwrap();
    task.await.unwrap().unwrap();
    for tag in ["c", "d", "e", "f", "g"] {
        assert!(transcript.contains(&format!("{tag} OK")), "{transcript}");
    }
    assert!(transcript.contains("c OK [COPYUID 2 1 1]"), "{transcript}");
    assert!(transcript.contains("d OK [COPYUID 2 1 2]"), "{transcript}");
    assert!(transcript.contains("[COPYUID 2 1 3]"), "{transcript}");
    assert!(transcript.find("[COPYUID 2 1 3]").unwrap() < transcript.find("* 1 EXPUNGE").unwrap());
    assert!(transcript.contains("* 1 EXPUNGE"), "{transcript}");
    assert_eq!(transcript.matches("Subject: copied").count(), 3);
    let account = db.account("a").unwrap();
    assert_eq!(account.messages.len(), 3);
    assert!(
        account
            .messages
            .iter()
            .all(|m| m.id != original.id && m.uid_in("inbox").is_none())
    );
    for message in &account.messages {
        assert_eq!(message.thread_id(), original.thread_id());
        assert_eq!(message.received_at, original.received_at);
        assert_eq!(message.keywords, original.keywords);
        assert_eq!(db.blob("a", &message.id).unwrap(), raw);
    }
}

#[tokio::test]
async fn imap_sequence_copy_preserves_tombstones_until_selection_is_safe() {
    let (service, db) = service();
    db.execute(
        "a",
        0,
        vec![mail_kernel::Command::CreateMailbox {
            name: "Archive".into(),
        }],
    )
    .unwrap();
    for subject in ["one", "two", "three"] {
        db.deliver(
            &["alice@example.org".into()],
            format!("Subject: {subject}\r\n\r\nbody\r\n").as_bytes(),
        )
        .unwrap();
    }
    let (client, server) = tokio::io::duplex(4096);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    client
        .get_mut()
        .write_all(format!("a LOGIN alice@example.org {TOKEN}\r\nb SELECT INBOX\r\n").as_bytes())
        .await
        .unwrap();
    loop {
        let mut line = String::new();
        client.read_line(&mut line).await.unwrap();
        if line.starts_with("b OK") {
            break;
        }
    }
    let account = db.account("a").unwrap();
    db.execute(
        "a",
        account.revision,
        vec![mail_kernel::Command::Destroy {
            id: account.messages[0].id.clone(),
        }],
    )
    .unwrap();
    client
        .get_mut()
        .write_all(b"c COPY 2 Archive\r\nd LOGOUT\r\n")
        .await
        .unwrap();
    let mut transcript = String::new();
    client.read_to_string(&mut transcript).await.unwrap();
    task.await.unwrap().unwrap();
    assert!(transcript.contains("c OK"), "{transcript}");
    let account = db.account("a").unwrap();
    let copy = account
        .messages
        .iter()
        .find(|m| m.uid_in("m1").is_some())
        .unwrap();
    assert!(
        String::from_utf8(db.blob("a", &copy.id).unwrap())
            .unwrap()
            .contains("Subject: two")
    );
}

#[tokio::test]
async fn uid_expunge_removes_only_selected_deleted_messages() {
    let (service, db) = service();
    for n in 1..=3 {
        let account = db.account("a").unwrap();
        db.execute(
            "a",
            account.revision,
            vec![mail_kernel::Command::Append {
                mailboxes: vec!["inbox".into()],
                raw: format!("Subject: {n}\r\n\r\nbody\r\n").into_bytes(),
                keywords: if n < 3 {
                    vec!["$deleted".into()]
                } else {
                    vec![]
                },
                received_at: 1234,
            }],
        )
        .unwrap();
    }
    let (mut client, server) = tokio::io::duplex(4096);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    client.write_all(format!("a LOGIN alice@example.org {TOKEN}\r\nb SELECT INBOX\r\nc UID EXPUNGE 2\r\nd UID EXPUNGE 3\r\ne UID SEARCH ALL\r\nf CAPABILITY\r\ng LOGOUT\r\n").as_bytes()).await.unwrap();
    let mut result = String::new();
    client.read_to_string(&mut result).await.unwrap();
    task.await.unwrap().unwrap();
    assert!(
        result.contains("c OK") && result.contains("d OK"),
        "{result}"
    );
    assert_eq!(result.matches("EXPUNGE\r\n").count(), 1, "{result}");
    assert!(result.contains("* 2 EXPUNGE"), "{result}");
    assert!(result.contains("* SEARCH 1 3"), "{result}");
    assert!(result.contains("IMAP4rev1 UIDPLUS MOVE"), "{result}");
    assert_eq!(db.account("a").unwrap().messages.len(), 2);
}
