use super::*;
use mail_kernel::Command;

async fn command(client: &mut BufReader<tokio::io::DuplexStream>, value: &str) -> String {
    client
        .get_mut()
        .write_all(format!("{value}\r\n").as_bytes())
        .await
        .unwrap();
    let tag = value.split_once(' ').unwrap().0;
    let mut output = String::new();
    loop {
        let mut line = String::new();
        assert!(client.read_line(&mut line).await.unwrap() > 0);
        output.push_str(&line);
        if line.starts_with(&format!("{tag} ")) {
            break;
        }
    }
    output
}

#[tokio::test]
async fn concurrent_expunge_never_retargets_a_session_sequence_number() {
    let (service, db) = service();
    for subject in ["one", "two", "three"] {
        db.deliver(
            &["alice@example.org".into()],
            format!("Subject: {subject}\r\n\r\nbody").as_bytes(),
        )
        .unwrap();
    }
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    assert!(
        command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}"))
            .await
            .contains("a OK")
    );
    assert!(
        command(&mut client, "b SELECT INBOX")
            .await
            .contains("3 EXISTS")
    );
    db.execute("a", 3, vec![Command::Destroy { id: "e1".into() }])
        .unwrap();
    let store = command(&mut client, "c STORE 2 +FLAGS (\\Deleted)").await;
    assert!(store.contains("c OK"), "{store}");
    assert!(
        !store.contains("EXPUNGE"),
        "EXPUNGE forbidden during sequence STORE"
    );
    let account = db.account("a").unwrap();
    assert!(
        account
            .messages
            .iter()
            .find(|m| m.id == "e2")
            .unwrap()
            .keywords
            .contains(&"$deleted".into())
    );
    assert!(
        account
            .messages
            .iter()
            .find(|m| m.id == "e3")
            .unwrap()
            .keywords
            .is_empty()
    );
    let fetch = command(&mut client, "d FETCH 2 (UID)").await;
    assert!(fetch.contains("* 2 FETCH (UID 2"), "{fetch}");
    let noop = command(&mut client, "e NOOP").await;
    assert!(noop.contains("* 1 EXPUNGE"), "{noop}");
    let fetch = command(&mut client, "f FETCH 2 (UID)").await;
    assert!(fetch.contains("* 2 FETCH (UID 3"), "{fetch}");
    let revision = db.account("a").unwrap().revision;
    let account = db
        .execute(
            "a",
            revision,
            vec![Command::CreateMailbox {
                name: "Archive".into(),
            }],
        )
        .unwrap();
    let archive = account
        .mailboxes
        .iter()
        .find(|m| m.name == "Archive")
        .unwrap()
        .id
        .clone();
    let account = db
        .execute(
            "a",
            account.revision,
            vec![Command::SetMailboxes {
                id: "e2".into(),
                mailboxes: vec![archive],
            }],
        )
        .unwrap();
    db.execute(
        "a",
        account.revision,
        vec![Command::SetMailboxes {
            id: "e2".into(),
            mailboxes: vec!["inbox".into()],
        }],
    )
    .unwrap();
    let fetch = command(&mut client, "g FETCH 1 (UID)").await;
    assert!(
        !fetch.contains("FETCH (UID"),
        "removed UID must remain a tombstone: {fetch}"
    );
    assert!(fetch.contains("* 3 EXISTS"));
    assert!(command(&mut client, "h NOOP").await.contains("* 1 EXPUNGE"));
    assert!(
        command(&mut client, "i FETCH 2 (UID)")
            .await
            .contains("UID 4")
    );
    command(&mut client, "j LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn console_sync_fetch_returns_internaldate_without_setting_seen() {
    let (service, db) = service();
    db.execute(
        "a",
        0,
        vec![Command::Append {
            mailboxes: vec!["inbox".into()],
            received_at: 1_000_000_000,
            raw: b"Subject: console\r\n\r\nbody\r\n".to_vec(),
            keywords: vec![],
        }],
    )
    .unwrap();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "b SELECT INBOX").await;
    let fetch = command(
        &mut client,
        "c UID FETCH 1:* (UID FLAGS INTERNALDATE BODY.PEEK[])",
    )
    .await;
    assert!(fetch.contains("c OK"), "{fetch}");
    assert!(
        fetch.contains("INTERNALDATE \"09-Sep-2001 01:46:40 +0000\""),
        "{fetch}"
    );
    assert!(fetch.contains("Subject: console"));
    assert!(db.account("a").unwrap().messages[0].keywords.is_empty());
    command(&mut client, "d LOGOUT").await;
    task.await.unwrap().unwrap();
}
