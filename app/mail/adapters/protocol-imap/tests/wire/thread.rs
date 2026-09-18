use super::search::command;
use super::*;

#[tokio::test]
async fn thread_groups_shared_conversations_filters_queries_and_keeps_uid_identity() {
    let (service, db) = service();
    for raw in [
        "Message-ID: <one@example.org>\r\nSubject: topic\r\n\r\none",
        "Message-ID: <two@example.org>\r\nReferences: <one@example.org>\r\nSubject: Re: topic\r\n\r\ntwo",
        "Message-ID: <three@example.org>\r\nSubject: other\r\n\r\nthree",
    ] {
        db.deliver(&["alice@example.org".into()], raw.as_bytes())
            .unwrap();
    }
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "s SELECT INBOX").await;
    for algorithm in ["REFERENCES", "ORDEREDSUBJECT"] {
        let result = command(&mut client, &format!("t THREAD {algorithm} UTF-8 ALL")).await;
        assert!(result.contains("* THREAD (1 2)(3)"), "{result}");
    }
    let result = command(&mut client, "t UID THREAD REFERENCES UTF-8 SUBJECT other").await;
    assert!(result.contains("* THREAD (3)"), "{result}");
    let before = db.account("a").unwrap();
    db.execute(
        "a",
        mail_api::Precondition::Observed(before.revision),
        vec![mail_kernel::Command::Destroy {
            id: before.messages[0].id.clone(),
        }],
    )
    .unwrap();
    let result = command(&mut client, "t THREAD REFERENCES UTF-8 ALL").await;
    assert!(
        result.contains("* THREAD (2)(3)") && !result.contains("EXPUNGE"),
        "{result}"
    );
    command(&mut client, "n NOOP").await;
    let result = command(&mut client, "t THREAD REFERENCES UTF-8 ALL").await;
    assert!(result.contains("* THREAD (1)(2)"), "{result}");
    let result = command(&mut client, "t UID THREAD REFERENCES UTF-8 ALL").await;
    assert!(result.contains("* THREAD (2)(3)"), "{result}");
    for query in [
        "BOGUS UTF-8 ALL",
        "REFERENCES UTF-8",
        "REFERENCES UTF-8 OR ALL",
        "REFERENCES UTF-8 RETURN (SAVE) ALL",
    ] {
        let result = command(&mut client, &format!("b THREAD {query}")).await;
        assert!(result.contains("b BAD"), "{result}");
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}
