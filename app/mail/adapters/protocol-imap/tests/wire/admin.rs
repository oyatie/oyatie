use super::*;
use mail_kernel::{Command, Error};

fn prepared() -> (Arc<MailService>, Arc<SqliteStore>) {
    let (service, db) = service();
    db.execute(
        "a",
        mail_api::Precondition::Observed(0),
        vec![
            Command::CreateMailbox {
                name: "Archive".into(),
            },
            Command::Append {
                mailboxes: vec!["inbox".into(), "m1".into()],
                raw: b"Subject: shared\r\n\r\nbody\r\n".to_vec(),
                keywords: vec![],
                received_at: 1234,
            },
            Command::Append {
                mailboxes: vec!["inbox".into()],
                raw: b"Subject: inbox\r\n\r\nbody\r\n".to_vec(),
                keywords: vec![],
                received_at: 1234,
            },
        ],
    )
    .unwrap();
    (service, db)
}
async fn transcript(service: Arc<MailService>, commands: &str) -> String {
    let (mut client, server) = tokio::io::duplex(8192);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    client
        .write_all(
            format!("a LOGIN alice@example.org {TOKEN}\r\n{commands}z LOGOUT\r\n").as_bytes(),
        )
        .await
        .unwrap();
    let mut transcript = String::new();
    client.read_to_string(&mut transcript).await.unwrap();
    task.await.unwrap().unwrap();
    transcript
}

#[tokio::test]
async fn delete_removes_populated_mailbox_but_preserves_other_message_copies() {
    let (service, db) = prepared();
    let before = db.account("a").unwrap();
    let source = before.messages[0].id.clone();
    let raw = db.blob("a", &source).unwrap();
    let state = db
        .execute(
            "a",
            mail_api::Precondition::Observed(before.revision),
            vec![Command::Append {
                mailboxes: vec!["m1".into()],
                raw: b"Subject: archive only\r\n\r\nbody\r\n".to_vec(),
                keywords: vec![],
                received_at: 1234,
            }],
        )
        .map(|_| db.account("a").unwrap())
        .unwrap();
    let removed = state.messages.last().unwrap().id.clone();
    let result = transcript(
        service,
        "b DELETE Archive\r\nc SELECT INBOX\r\nd UID SEARCH ALL\r\n",
    )
    .await;
    assert!(result.contains("b OK"), "{result}");
    assert!(result.contains("* SEARCH 1 2"), "{result}");
    let state = db.account("a").unwrap();
    assert_eq!(state.mailboxes.len(), 1);
    assert_eq!(state.messages.len(), 2);
    assert_eq!(db.blob("a", &source).unwrap(), raw);
    assert_eq!(db.blob("a", &removed), Err(Error::NotFound));
    assert_eq!(
        state.messages[0].uid_in("inbox"),
        before.messages[0].uid_in("inbox")
    );
}

#[tokio::test]
async fn expunge_and_close_respect_membership_and_read_only_selection() {
    let (service, db) = prepared();
    let shared = db.account("a").unwrap().messages[0].id.clone();
    let result=transcript(service,"b SELECT INBOX\r\nc STORE 1 +FLAGS (\\Deleted)\r\nd EXPUNGE\r\ne CHECK\r\nf STORE 1 +FLAGS (\\Deleted)\r\ng CLOSE\r\nh FETCH 1 (UID)\r\ni EXAMINE Archive\r\nj CLOSE\r\n").await;
    for tag in ["b", "c", "d", "e", "f", "g", "i", "j"] {
        assert!(result.contains(&format!("{tag} OK")), "{result}");
    }
    assert!(result.contains("h BAD"), "{result}");
    assert_eq!(
        result.matches("EXPUNGE\r\n").count(),
        1,
        "CLOSE must not emit expunge: {result}"
    );
    let state = db.account("a").unwrap();
    assert_eq!(state.messages.len(), 1);
    assert_eq!(state.messages[0].id, shared);
    assert_eq!(state.messages[0].uid_in("m1"), Some(1));
    assert_eq!(state.messages[0].uid_in("inbox"), None);
    assert!(db.blob("a", &shared).is_ok());
}
