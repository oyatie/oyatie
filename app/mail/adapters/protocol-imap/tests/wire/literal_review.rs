use super::*;
use std::time::Duration;

type Client = BufReader<tokio::io::DuplexStream>;
type Task = tokio::task::JoinHandle<std::io::Result<()>>;

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
    .expect("general literal response deadline")
}

async fn command(client: &mut Client, input: &str) -> String {
    client
        .get_mut()
        .write_all(format!("{input}\r\n").as_bytes())
        .await
        .unwrap();
    until(
        client,
        &format!("{} ", input.split_ascii_whitespace().next().unwrap()),
    )
    .await
}

async fn start(authenticate: bool, encrypted: bool) -> (Client, Task, Arc<SqliteStore>) {
    let (service, db) = service();
    let (client, server) = tokio::io::duplex(32768);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, encrypted));
    let mut client = BufReader::new(client);
    until(&mut client, "* OK").await;
    if authenticate {
        assert!(
            command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}"))
                .await
                .contains("a OK")
        );
    }
    (client, task, db)
}

async fn literal(client: &mut Client, prefix: &str, bytes: &[u8]) {
    client
        .get_mut()
        .write_all(format!("{prefix}{{{}}}\r\n", bytes.len()).as_bytes())
        .await
        .unwrap();
    let mut response = String::new();
    tokio::time::timeout(Duration::from_secs(5), client.read_line(&mut response))
        .await
        .unwrap()
        .unwrap();
    assert!(
        response.starts_with('+'),
        "expected continuation: {response}"
    );
    client.get_mut().write_all(bytes).await.unwrap();
}

async fn finish(mut client: Client, task: Task) {
    command(&mut client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn login_accepts_two_byte_counted_literals_and_keeps_pipelined_commands() {
    for nonsync in [false, true] {
        let (mut client, task, _) = start(false, true).await;
        if nonsync {
            client
                .get_mut()
                .write_all(
                    format!(
                        "l LOGIN {{17+}}\r\nalice@example.org {{{}+}}\r\n{TOKEN}\r\nnext NOOP\r\n",
                        TOKEN.len()
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();
        } else {
            literal(&mut client, "l LOGIN ", b"alice@example.org").await;
            literal(&mut client, " ", TOKEN.as_bytes()).await;
            client
                .get_mut()
                .write_all(b"\r\nnext NOOP\r\n")
                .await
                .unwrap();
        }
        let response = until(&mut client, "next ").await;
        assert!(
            response.contains("l OK") && response.contains("next OK"),
            "{response}"
        );
        assert!(
            !response.contains(" BAD ") && !response.contains(" NO "),
            "{response}"
        );
        finish(client, task).await;
    }
}

#[tokio::test]
async fn mailbox_literals_preserve_names_and_share_create_rename_list_status_semantics() {
    let (mut client, task, db) = start(true, true).await;
    let name = "Client {folder}";
    literal(&mut client, "c CREATE ", name.as_bytes()).await;
    client.get_mut().write_all(b"\r\n").await.unwrap();
    assert!(until(&mut client, "c ").await.contains("c OK"));
    literal(&mut client, "r RENAME ", name.as_bytes()).await;
    literal(&mut client, " ", b"&ZcWITA-").await; // modified UTF7: 旅行
    client.get_mut().write_all(b"\r\n").await.unwrap();
    assert!(until(&mut client, "r ").await.contains("r OK"));
    assert!(
        db.account("a")
            .unwrap()
            .mailboxes
            .iter()
            .any(|m| m.name == "旅行")
    );
    client
        .get_mut()
        .write_all(b"l LIST {0+}\r\n {1+}\r\n*\r\ns STATUS {8+}\r\n&ZcWITA- (MESSAGES UIDNEXT)\r\n")
        .await
        .unwrap();
    let response = until(&mut client, "s ").await;
    assert!(
        response.contains("l OK") && response.contains("s OK") && response.contains("&ZcWITA-"),
        "{response}"
    );
    literal(&mut client, "d DELETE ", b"&ZcWITA-").await;
    client.get_mut().write_all(b"\r\n").await.unwrap();
    assert!(until(&mut client, "d ").await.contains("d OK"));
    finish(client, task).await;
}

#[tokio::test]
async fn malformed_nonsync_literals_never_mutate_state_or_execute_payload_commands() {
    for header in [
        b"m CREATE x{17+}".as_slice(),
        b"m CREATE \"unterminated {17+}",
        b"m NOOP {17+}",
        b"m LOGIN alice@example.org {999999999+}",
        b"m CREATE \xff {17+}",
        b"bad? CREATE {17+}",
        b"m CREATE {17+} ",
        b"m CREATE {17+}junk",
        b"m APPEND &bad {17+}",
        b"m APPEND INBOX x{17+}",
        b"m APPEND INBOX {999999999+}",
    ] {
        let (mut client, task, db) = start(true, true).await;
        let before = db.account("a").unwrap();
        let mut input = header.to_vec();
        input.extend_from_slice(b"\r\ninjected LOGOUT\r\n\r\nend LOGOUT\r\n");
        client.get_mut().write_all(&input).await.unwrap();
        let mut response = String::new();
        tokio::time::timeout(Duration::from_secs(5), client.read_to_string(&mut response))
            .await
            .unwrap()
            .unwrap();
        assert!(
            !response.contains("injected OK") && !response.contains("m OK"),
            "{header:?}: {response}"
        );
        assert_eq!(
            db.account("a").unwrap(),
            before,
            "malformed literal must not create its marker as a mailbox"
        );
        let _ = task.await.unwrap();
    }
}

#[tokio::test]
async fn literal_controls_cannot_inject_commands_or_replace_saved_mailbox_state() {
    for (prefix, suffix) in [
        ("c CREATE ", ""),
        ("c FETCH 1 (BODY.PEEK[HEADER.FIELDS (", ")])"),
    ] {
        for payload in [b"x\0y".as_slice(), b"x\r\ninjected LOGOUT\r\ny", &[0xff]] {
            let (mut client, task, db) = start(true, true).await;
            db.deliver(
                &["alice@example.org".into()],
                b"Secret: excluded\r\n\r\nbody",
            )
            .unwrap();
            assert!(
                command(&mut client, "s SELECT INBOX")
                    .await
                    .contains("s OK")
            );
            let before = db.account("a").unwrap();
            let mut input = format!("{prefix}{{{}+}}\r\n", payload.len()).into_bytes();
            input.extend_from_slice(payload);
            input.extend_from_slice(format!("{suffix}\r\nnext NOOP\r\n").as_bytes());
            client.get_mut().write_all(&input).await.unwrap();
            let response = until(&mut client, "next ").await;
            assert!(
                response.contains("c BAD") || response.contains("c NO"),
                "{response}"
            );
            assert!(
                response.contains("next OK") && !response.contains("injected OK"),
                "{response}"
            );
            assert!(!response.contains("Secret:"), "{response}");
            assert_eq!(db.account("a").unwrap(), before);
            finish(client, task).await;
        }
    }
}

#[tokio::test]
async fn plaintext_login_and_revoked_mailbox_access_refuse_before_literal_continuation() {
    let (mut client, task, _) = start(false, false).await;
    client
        .get_mut()
        .write_all(b"l LOGIN {17}\r\nnext NOOP\r\n")
        .await
        .unwrap();
    let response = until(&mut client, "next ").await;
    assert!(
        response.contains("l BAD") || response.contains("l NO"),
        "{response}"
    );
    assert!(
        !response.lines().any(|line| line.starts_with('+')),
        "{response}"
    );
    finish(client, task).await;
    let (mut client, task, db) = start(true, true).await;
    db.revoke("a").unwrap();
    client
        .get_mut()
        .write_all(b"c CREATE {4}\r\nz LOGOUT\r\n")
        .await
        .unwrap();
    let response = until(&mut client, "z ").await;
    assert!(
        response.contains("c BAD") || response.contains("c NO"),
        "{response}"
    );
    assert!(
        !response.lines().any(|line| line.starts_with('+')),
        "{response}"
    );
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn append_mailbox_literal_hands_message_body_to_the_existing_append_reader() {
    let (mut client, task, db) = start(true, true).await;
    literal(&mut client, "p APPEND ", b"INBOX").await;
    let raw = b"Subject: literal body\r\n\r\ninjected LOGOUT\r\n";
    literal(&mut client, " (\\Seen) ", raw).await;
    client
        .get_mut()
        .write_all(b"\r\nnext NOOP\r\n")
        .await
        .unwrap();
    let response = until(&mut client, "next ").await;
    assert!(
        response.contains("p OK [APPENDUID") && response.contains("next OK"),
        "{response}"
    );
    assert!(!response.contains("injected OK"), "{response}");
    let account = db.account("a").unwrap();
    assert_eq!(account.messages.len(), 1);
    assert_eq!(db.blob("a", &account.messages[0].id).unwrap(), raw);
    assert_eq!(account.messages[0].keywords, ["$seen"]);
    finish(client, task).await;
}
