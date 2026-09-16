use super::*;
use std::time::Duration;

async fn until(client: &mut Client, prefix: &str) -> String {
    tokio::time::timeout(Duration::from_secs(5), async {
        let mut result = String::new();
        loop {
            let mut line = String::new();
            assert!(client.read_line(&mut line).await.unwrap() > 0, "{result}");
            let done = line.starts_with(prefix);
            result.push_str(&line);
            if done {
                return result;
            }
        }
    })
    .await
    .expect("literal review response deadline")
}

async fn start(selected: bool) -> (Client, tokio::task::JoinHandle<std::io::Result<()>>) {
    let (service, db) = super::super::service();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: target\r\n\r\nbody",
    )
    .unwrap();
    let (client, server) = tokio::io::duplex(32768);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    if selected {
        command(&mut client, "b SELECT INBOX").await;
    }
    (client, task)
}

async fn finish(mut client: Client, task: tokio::task::JoinHandle<std::io::Result<()>>) {
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn search_quoted_markers_remain_data_and_malformed_sync_markers_never_continue() {
    let (mut client, task) = start(true).await;
    let response = command(&mut client, "s SEARCH SUBJECT \"{4+}\"").await;
    assert!(response.contains("s OK"), "{response}");
    for suffix in ["SUBJECT \"unterminated {4}", "SUBJECT x{4}", "SUBJECT ~{4}"] {
        client
            .get_mut()
            .write_all(format!("s SEARCH {suffix}\r\nnext NOOP\r\n").as_bytes())
            .await
            .unwrap();
        let response = until(&mut client, "next ").await;
        assert!(
            response.contains("s BAD") && response.contains("next OK"),
            "{response}"
        );
        assert!(
            !response.lines().any(|line| line.starts_with('+')),
            "{response}"
        );
    }
    finish(client, task).await;
}

#[tokio::test]
async fn search_glued_nonsync_and_literal8_markers_cannot_execute_payload_commands() {
    for suffix in [
        "SUBJECT x{17+}",
        "SUBJECT ~{17+}",
        "SUBJECT \"unterminated {17+}",
    ] {
        let (mut client, task) = start(true).await;
        client
            .get_mut()
            .write_all(format!("s SEARCH {suffix}\r\ninjected LOGOUT\r\n\r\n").as_bytes())
            .await
            .unwrap();
        let mut response = String::new();
        tokio::time::timeout(Duration::from_secs(5), client.read_to_string(&mut response))
            .await
            .unwrap()
            .unwrap();
        assert!(response.contains("s BAD"), "{response}");
        assert!(
            !response.contains("injected OK") && !response.contains("* BYE"),
            "{response}"
        );
        let _ = task.await.unwrap();
    }
}

#[tokio::test]
async fn search_literals_refuse_unselected_and_unauthenticated_states_before_continuation() {
    let (mut client, task) = start(false).await;
    client
        .get_mut()
        .write_all(b"s SEARCH SUBJECT {4}\r\nnext NOOP\r\n")
        .await
        .unwrap();
    let response = until(&mut client, "next ").await;
    assert!(
        response.contains("s BAD") && response.contains("next OK"),
        "{response}"
    );
    assert!(
        !response.lines().any(|line| line.starts_with('+')),
        "{response}"
    );
    finish(client, task).await;

    let (service, _) = super::super::service();
    let (client, server) = tokio::io::duplex(4096);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    client
        .get_mut()
        .write_all(b"s SEARCH SUBJECT {4}\r\nnext NOOP\r\n")
        .await
        .unwrap();
    let response = until(&mut client, "next ").await;
    assert!(
        response.contains("s BAD") && response.contains("next OK"),
        "{response}"
    );
    assert!(
        !response.lines().any(|line| line.starts_with('+')),
        "{response}"
    );
    finish(client, task).await;
}

#[tokio::test]
async fn search_literal_budget_applies_to_the_entire_command_before_next_continuation() {
    let (mut client, task) = start(true).await;
    client
        .get_mut()
        .write_all(b"s SEARCH SUBJECT {4000}\r\n")
        .await
        .unwrap();
    until(&mut client, "+").await;
    client.get_mut().write_all(&vec![b'a'; 4000]).await.unwrap();
    client
        .get_mut()
        .write_all(b" BODY {4200}\r\nnext NOOP\r\n")
        .await
        .unwrap();
    let response = until(&mut client, "next ").await;
    assert!(
        response.contains("s BAD") && response.contains("next OK"),
        "{response}"
    );
    assert!(
        !response.lines().any(|line| line.starts_with('+')),
        "{response}"
    );
    finish(client, task).await;
}

#[tokio::test]
async fn search_nul_and_split_utf8_literals_refuse_without_consuming_the_next_command() {
    for raw in [
        b"s SEARCH SUBJECT {3+}\r\na\0b\r\nnext NOOP\r\n".as_slice(),
        "s SEARCH CHARSET UTF-8 SUBJECT {1+}\r\né\r\nnext NOOP\r\n".as_bytes(),
    ] {
        let (mut client, task) = start(true).await;
        client.get_mut().write_all(raw).await.unwrap();
        let response = until(&mut client, "next ").await;
        assert!(
            response.contains("s BAD") && response.contains("next OK"),
            "{response}"
        );
        assert!(!response.contains("* SEARCH"), "{response}");
        finish(client, task).await;
    }
}

#[tokio::test]
async fn search_truncated_literal_eof_releases_the_session_without_a_success_response() {
    let (mut client, task) = start(true).await;
    client
        .get_mut()
        .write_all(b"s SEARCH SUBJECT {8}\r\n")
        .await
        .unwrap();
    until(&mut client, "+").await;
    client.get_mut().write_all(b"ab").await.unwrap();
    client.get_mut().shutdown().await.unwrap();
    let mut response = String::new();
    tokio::time::timeout(Duration::from_secs(5), client.read_to_string(&mut response))
        .await
        .unwrap()
        .unwrap();
    assert!(!response.contains("s OK"), "{response}");
    let result = task.await.unwrap();
    assert!(result.is_err());
}

#[tokio::test]
async fn search_revoked_credentials_refuse_literal_before_continuation() {
    let (service, db) = super::super::service();
    let (client, server) = tokio::io::duplex(4096);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "b SELECT INBOX").await;
    db.revoke("a").unwrap();
    client
        .get_mut()
        .write_all(b"s SEARCH SUBJECT {4}\r\nz LOGOUT\r\n")
        .await
        .unwrap();
    let response = until(&mut client, "z ").await;
    assert!(
        response.contains("s BAD") || response.contains("s NO"),
        "{response}"
    );
    assert!(
        !response.lines().any(|line| line.starts_with('+')),
        "{response}"
    );
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn search_literal_limit_reserves_the_final_command_crlf_before_continuation() {
    for size in [8175, 8176] {
        let (mut client, task) = start(true).await;
        // "SUBJECT {8176}" + marker CRLF +8176octets already fills8192,
        // leaving no room for the mandatory command-terminating CRLF.
        client
            .get_mut()
            .write_all(format!("s SEARCH SUBJECT {{{size}}}\r\nnext NOOP\r\n").as_bytes())
            .await
            .unwrap();
        let response = until(&mut client, "next ").await;
        assert!(
            response.contains("s BAD") && response.contains("next OK"),
            "{response}"
        );
        assert!(
            !response.lines().any(|line| line.starts_with('+')),
            "{response}"
        );
        finish(client, task).await;
    }
    let (mut client, task) = start(true).await;
    client
        .get_mut()
        .write_all(b"s SEARCH SUBJECT {8174}\r\n")
        .await
        .unwrap();
    until(&mut client, "+").await;
    client.get_mut().write_all(&vec![b'a'; 8174]).await.unwrap();
    client
        .get_mut()
        .write_all(b"\r\nnext NOOP\r\n")
        .await
        .unwrap();
    let response = until(&mut client, "next ").await;
    assert!(
        response.contains("s OK") && response.contains("next OK"),
        "{response}"
    );
    finish(client, task).await;
}
