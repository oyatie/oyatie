use super::*;
use std::time::Duration;

type Client = BufReader<tokio::io::DuplexStream>;

async fn until(client: &mut Client, prefix: &str) -> String {
    tokio::time::timeout(Duration::from_secs(3), async {
        let mut output = String::new();
        loop {
            let mut line = String::new();
            assert!(client.read_line(&mut line).await.unwrap() > 0, "{output}");
            let done = line.starts_with(prefix);
            output.push_str(&line);
            if done {
                return output;
            }
        }
    })
    .await
    .expect("literal response deadline")
}

async fn start() -> (
    Client,
    tokio::task::JoinHandle<std::io::Result<()>>,
    Arc<SqliteStore>,
) {
    let (service, db) = service();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    until(&mut client, "* OK").await;
    assert!(
        command(&mut client, "cap CAPABILITY\r\n", "cap ")
            .await
            .split_whitespace()
            .any(|s| s == "LITERAL+")
    );
    (client, task, db)
}

async fn command(client: &mut Client, input: &str, tag: &str) -> String {
    client.get_mut().write_all(input.as_bytes()).await.unwrap();
    until(client, tag).await
}

#[tokio::test]
async fn login_and_mailbox_literals_preserve_astring_bytes_and_pipeline() {
    let (mut client, task, db) = start().await;
    assert!(
        command(&mut client, "a LOGIN {17}\r\n", "+")
            .await
            .contains("+ Ready")
    );
    assert!(
        command(&mut client, "alice@example.org {32}\r\n", "+")
            .await
            .contains("+ Ready")
    );
    assert!(
        command(&mut client, &format!("{TOKEN}\r\n"), "a ")
            .await
            .contains("a OK")
    );
    let name = "Folder (one) \\\"quoted\\\"";
    let input = format!(
        "b CREATE {{{}+}}\r\n{name}\r\nc LIST {{0+}}\r\n {{{}+}}\r\n{name}\r\nd SELECT {{{}+}}\r\n{name}\r\ne STATUS {{{}+}}\r\n{name} (MESSAGES UIDNEXT)\r\nz LOGOUT\r\n",
        name.len(),
        name.len(),
        name.len(),
        name.len()
    );
    client.get_mut().write_all(input.as_bytes()).await.unwrap();
    let mut output = String::new();
    client.read_to_string(&mut output).await.unwrap();
    task.await.unwrap().unwrap();
    for tag in ["b", "c", "d", "e", "z"] {
        assert!(output.contains(&format!("{tag} OK")), "{output}");
    }
    assert!(
        db.account("a")
            .unwrap()
            .mailboxes
            .iter()
            .any(|m| m.name == name)
    );
    assert!(!output.contains("+ Ready"), "{output}");
}

#[tokio::test]
async fn literal_mailboxes_decode_modified_utf7_after_collecting_and_enable_utf8() {
    let (mut client, task, db) = start().await;
    assert!(
        command(
            &mut client,
            &format!("a LOGIN alice@example.org {TOKEN}\r\n"),
            "a "
        )
        .await
        .contains("a OK")
    );
    for (tag, name) in [("b", "Caf&AOk-"), ("c", "&ZeVnLIqe-")] {
        let output = command(
            &mut client,
            &format!("{tag} CREATE {{{}+}}\r\n{name}\r\n", name.len()),
            &format!("{tag} "),
        )
        .await;
        assert!(output.contains(&format!("{tag} OK")), "{output}");
    }
    assert!(
        command(&mut client, "d ENABLE UTF8=ACCEPT\r\n", "d ")
            .await
            .contains("d OK")
    );
    assert!(
        command(&mut client, "e SELECT {5+}\r\nCafé\r\n", "e ")
            .await
            .contains("e OK")
    );
    command(&mut client, "z LOGOUT\r\n", "z ").await;
    task.await.unwrap().unwrap();
    assert!(
        db.account("a")
            .unwrap()
            .mailboxes
            .iter()
            .any(|m| m.name == "日本語")
    );
}

#[tokio::test]
async fn append_literal_mailbox_and_nonsync_body_commit_before_ack() {
    let (mut client, task, db) = start().await;
    command(
        &mut client,
        &format!("a LOGIN alice@example.org {TOKEN}\r\n"),
        "a ",
    )
    .await;
    let raw = "Subject: stored\r\n\r\ninjected LOGOUT\r\n";
    let result = command(
        &mut client,
        &format!(
            "b APPEND {{5+}}\r\nINBOX (\\Seen) {{{}+}}\r\n{raw}\r\nc NOOP\r\n",
            raw.len()
        ),
        "c ",
    )
    .await;
    assert!(
        result.contains("b OK [APPENDUID 1 1]") && result.contains("c OK"),
        "{result}"
    );
    assert!(
        !result.contains("injected OK") && !result.contains("+ Ready"),
        "{result}"
    );
    assert_eq!(db.blob("a", "e1").unwrap(), raw.as_bytes());
    command(&mut client, "z LOGOUT\r\n", "z ").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn invalid_literal_encoding_consumes_frame_and_preserves_next_command() {
    let (mut client, task, db) = start().await;
    command(
        &mut client,
        &format!("a LOGIN alice@example.org {TOKEN}\r\n"),
        "a ",
    )
    .await;
    for value in [b"\xff".as_slice(), b"\0", b"\xc3"] {
        client
            .get_mut()
            .write_all(b"b CREATE {1+}\r\n")
            .await
            .unwrap();
        client.get_mut().write_all(value).await.unwrap();
        let result = command(&mut client, "\r\nc NOOP\r\n", "c ").await;
        assert!(
            result.contains("b BAD") && result.contains("c OK"),
            "{result}"
        );
    }
    assert_eq!(db.account("a").unwrap().mailboxes.len(), 1);
    command(&mut client, "z LOGOUT\r\n", "z ").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn fetch_literal_header_fields_preserve_brackets_and_never_fetch_other_headers() {
    let (mut client, task, db) = start().await;
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: chosen\r\nX-]: bracket\r\nSecret: excluded\r\n\r\nbody",
    )
    .unwrap();
    command(
        &mut client,
        &format!("a LOGIN alice@example.org {TOKEN}\r\n"),
        "a ",
    )
    .await;
    command(&mut client, "b SELECT INBOX\r\n", "b ").await;
    let result = command(
        &mut client,
        "c UID FETCH 1 (BODY.PEEK[HEADER.FIELDS ({7+}\r\nSubject {3+}\r\nX-])])\r\nnext NOOP\r\n",
        "next ",
    )
    .await;
    assert!(
        result.contains("c OK")
            && result.contains("Subject: chosen")
            && result.contains("X-]: bracket"),
        "{result}"
    );
    assert!(
        !result.contains("Secret:") && !result.contains("+ Ready"),
        "{result}"
    );
    assert!(db.account("a").unwrap().messages[0].keywords.is_empty());
    command(&mut client, "z LOGOUT\r\n", "z ").await;
    task.await.unwrap().unwrap();
}
