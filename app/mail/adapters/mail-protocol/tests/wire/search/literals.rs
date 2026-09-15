use super::*;
use std::time::Duration;

const SUBJECT: &str = "Café \"quote\" (round) \\ value";
const BODY: &str = "first\r\ninjected LOGOUT\r\nlast";

async fn until(client: &mut Client, prefix: &str) -> String {
    tokio::time::timeout(Duration::from_secs(5), async {
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
    .expect("SEARCH literal response deadline")
}

async fn start() -> (
    Client,
    tokio::task::JoinHandle<std::io::Result<()>>,
    Arc<SqliteStore>,
) {
    let (service, db) = super::super::service();
    db.deliver(
        &["alice@example.org".into()],
        format!("Subject: {SUBJECT}\r\n\r\n{BODY}").as_bytes(),
    )
    .unwrap();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: other\r\n\r\nother",
    )
    .unwrap();
    let (client, server) = tokio::io::duplex(16384);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    command(&mut client, &format!("a LOGIN alice@example.org {TOKEN}")).await;
    command(&mut client, "b SELECT INBOX").await;
    (client, task, db)
}

async fn literal(client: &mut Client, prefix: &str, bytes: &[u8]) {
    client
        .get_mut()
        .write_all(format!("{prefix}{{{}}}\r\n", bytes.len()).as_bytes())
        .await
        .unwrap();
    let response = until(client, "+").await;
    assert!(
        !response.contains(" BAD ") && !response.contains(" NO "),
        "{response}"
    );
    client.get_mut().write_all(bytes).await.unwrap();
}

async fn finish(client: &mut Client, task: tokio::task::JoinHandle<std::io::Result<()>>) {
    command(client, "z LOGOUT").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn search_literals_count_utf8_bytes_and_support_sync_and_nonsync_delivery() {
    for nonsync in [false, true] {
        let (mut client, task, db) = start().await;
        let before = db.account("a").unwrap();
        if nonsync {
            client
                .get_mut()
                .write_all(
                    format!(
                        "s SEARCH CHARSET UTF-8 SUBJECT {{{}+}}\r\n{SUBJECT}\r\nnext NOOP\r\n",
                        SUBJECT.len()
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();
        } else {
            literal(
                &mut client,
                "s SEARCH CHARSET UTF-8 SUBJECT ",
                SUBJECT.as_bytes(),
            )
            .await;
            client
                .get_mut()
                .write_all(b"\r\nnext NOOP\r\n")
                .await
                .unwrap();
        }
        let result = until(&mut client, "next ").await;
        assert!(
            result.contains("* SEARCH 1\r\n")
                && result.contains("s OK")
                && result.contains("next OK"),
            "{result}"
        );
        assert!(
            !result.lines().any(|line| line.starts_with('+')),
            "{result}"
        );
        assert_eq!(db.account("a").unwrap(), before);
        finish(&mut client, task).await;
    }
}

#[tokio::test]
async fn search_multiple_literals_preserve_quotes_parentheses_backslashes_and_body_newlines() {
    let (mut client, task, db) = start().await;
    let before = db.account("a").unwrap();
    literal(
        &mut client,
        "s UID SEARCH CHARSET UTF-8 HEADER ",
        b"Subject",
    )
    .await;
    literal(&mut client, " ", SUBJECT.as_bytes()).await;
    literal(&mut client, " BODY ", BODY.as_bytes()).await;
    client
        .get_mut()
        .write_all(b"\r\nnext NOOP\r\n")
        .await
        .unwrap();
    let result = until(&mut client, "next ").await;
    assert!(
        result.contains("* SEARCH 1\r\n") && result.contains("s OK") && result.contains("next OK"),
        "{result}"
    );
    assert!(
        !result.contains("injected OK") && !result.contains("* BYE"),
        "{result}"
    );
    assert_eq!(db.account("a").unwrap(), before);
    finish(&mut client, task).await;
}

#[tokio::test]
async fn search_empty_literal_and_literal_sort_share_astring_semantics() {
    let (mut client, task, _) = start().await;
    literal(&mut client, "s SEARCH SUBJECT ", b"").await;
    client.get_mut().write_all(b"\r\n").await.unwrap();
    let result = until(&mut client, "s ").await;
    assert!(
        result.contains("* SEARCH 1 2\r\n") && result.contains("s OK"),
        "{result}"
    );
    literal(
        &mut client,
        "t SORT (SUBJECT) UTF-8 SUBJECT ",
        SUBJECT.as_bytes(),
    )
    .await;
    client
        .get_mut()
        .write_all(b"\r\nnext NOOP\r\n")
        .await
        .unwrap();
    let result = until(&mut client, "next ").await;
    assert!(
        result.contains("* SORT 1\r\n") && result.contains("t OK") && result.contains("next OK"),
        "{result}"
    );
    finish(&mut client, task).await;
}

#[tokio::test]
async fn search_invalid_utf8_literal_refuses_without_consuming_the_next_command() {
    let (mut client, task, _) = start().await;
    for byte in [0xc3, 0] {
        literal(&mut client, "s SEARCH CHARSET UTF-8 SUBJECT ", &[byte]).await;
        client
            .get_mut()
            .write_all(b"\r\nnext NOOP\r\n")
            .await
            .unwrap();
        let result = until(&mut client, "next ").await;
        assert!(
            result.contains("s BAD") && result.contains("next OK"),
            "{result}"
        );
    }
    finish(&mut client, task).await;
}

#[tokio::test]
async fn search_aggregate_literal_budget_refuses_before_another_continuation() {
    let (mut client, task, _) = start().await;
    literal(&mut client, "s SEARCH OR BODY ", &vec![b'x'; 5000]).await;
    client
        .get_mut()
        .write_all(b" BODY {5000}\r\nnext NOOP\r\n")
        .await
        .unwrap();
    let result = until(&mut client, "next ").await;
    assert!(
        (result.contains("s BAD") || result.contains("s NO")) && result.contains("next OK"),
        "{result}"
    );
    assert!(
        !result.lines().any(|line| line.starts_with('+')),
        "{result}"
    );
    finish(&mut client, task).await;
}

#[tokio::test]
async fn search_invalid_and_oversized_sync_lengths_refuse_before_continuation() {
    let (mut client, task, _) = start().await;
    for marker in ["{abc}", "{-1}", "{8193}", "{184467440737095516160}"] {
        client
            .get_mut()
            .write_all(format!("s SEARCH SUBJECT {marker}\r\nnext NOOP\r\n").as_bytes())
            .await
            .unwrap();
        let result = until(&mut client, "next ").await;
        assert!(
            (result.contains("s BAD") || result.contains("s NO")) && result.contains("next OK"),
            "{marker}: {result}"
        );
        assert!(
            !result.lines().any(|line| line.starts_with('+')),
            "{result}"
        );
    }
    finish(&mut client, task).await;
}

#[tokio::test]
async fn search_oversized_nonsync_literal_closes_without_executing_payload_commands() {
    let (mut client, task, _) = start().await;
    client
        .get_mut()
        .write_all(b"s SEARCH SUBJECT {999999999+}\r\ninjected LOGOUT\r\n")
        .await
        .unwrap();
    let mut result = String::new();
    tokio::time::timeout(Duration::from_secs(5), client.read_to_string(&mut result))
        .await
        .unwrap()
        .unwrap();
    assert!(
        result.contains("s BAD") || result.contains("s NO"),
        "{result}"
    );
    assert!(!result.contains("injected OK"), "{result}");
    let _ = task.await.unwrap();
}
