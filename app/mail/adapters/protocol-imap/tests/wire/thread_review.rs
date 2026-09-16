use super::search::command;
use super::*;

#[tokio::test]
async fn thread_accepts_required_charset_after_utf8_negotiation() {
    let (service, db) = service();
    db.deliver(
        &["alice@example.org".into()],
        "Subject: café\r\n\r\nbody".as_bytes(),
    )
    .unwrap();
    let (client, server) = tokio::io::duplex(4096);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "e ENABLE UTF8=ACCEPT").await;
    command(&mut client, "s SELECT INBOX").await;
    for verb in ["THREAD", "UID THREAD"] {
        for algorithm in ["REFERENCES", "ORDEREDSUBJECT"] {
            let response = command(
                &mut client,
                &format!("t {verb} {algorithm} UTF-8 SUBJECT café"),
            )
            .await;
            assert!(
                response.contains("* THREAD (1)\r\n") && response.contains("t OK"),
                "{response}"
            );
        }
    }
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn thread_accepts_literal_search_criteria_with_selected_state_and_uid_dispatch() {
    for uid in [false, true] {
        let (service, db) = service();
        db.deliver(
            &["alice@example.org".into()],
            b"Subject: target\r\n\r\nbody",
        )
        .unwrap();
        let (client, server) = tokio::io::duplex(4096);
        let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
        let mut client = BufReader::new(client);
        command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
        command(&mut client, "s SELECT INBOX").await;
        let verb = if uid { "UID THREAD" } else { "THREAD" };
        client
            .get_mut()
            .write_all(format!("t {verb} REFERENCES UTF-8 SUBJECT {{6}}\r\n").as_bytes())
            .await
            .unwrap();
        let mut response = String::new();
        assert!(client.read_line(&mut response).await.unwrap() > 0);
        assert!(response.starts_with('+'), "{response}");
        client
            .get_mut()
            .write_all(b"target\r\nnext NOOP\r\n")
            .await
            .unwrap();
        response.clear();
        loop {
            let mut line = String::new();
            assert!(client.read_line(&mut line).await.unwrap() > 0);
            let done = line.starts_with("next ");
            response.push_str(&line);
            if done {
                break;
            }
        }
        assert!(
            response.contains("* THREAD (1)\r\n") && response.contains("t OK"),
            "{response}"
        );
        command(&mut client, "z LOGOUT").await;
        task.await.unwrap().unwrap();
    }
}
