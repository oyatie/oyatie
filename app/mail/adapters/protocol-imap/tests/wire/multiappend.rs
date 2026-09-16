use super::*;
use std::time::Duration;

type Client = BufReader<tokio::io::DuplexStream>;
type Task = tokio::task::JoinHandle<std::io::Result<()>>;

async fn until(client: &mut Client, prefix: &str) -> String {
    until_with_timeout(client, prefix, Duration::from_secs(5)).await
}

async fn until_with_timeout(client: &mut Client, prefix: &str, timeout: Duration) -> String {
    tokio::time::timeout(timeout, async {
        let mut response = String::new();
        loop {
            let mut line = String::new();
            assert!(client.read_line(&mut line).await.unwrap() > 0, "{response}");
            let done = line.starts_with(prefix);
            response.push_str(&line);
            if done {
                return response;
            }
        }
    })
    .await
    .expect("MULTIAPPEND response deadline")
}

async fn start(quota: usize) -> (Client, Task, Arc<SqliteStore>) {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    let mut account = Account::new("a", "t", "alice", "alice@example.org").unwrap();
    account.quota_bytes = quota;
    db.provision(account, TOKEN).unwrap();
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: Arc::new(OwnerPolicy),
    });
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    client
        .get_mut()
        .write_all(format!("a LOGIN alice@example.org {TOKEN}\r\n").as_bytes())
        .await
        .unwrap();
    assert!(until(&mut client, "a ").await.contains("a OK"));
    (client, task, db)
}

async fn finish(mut client: Client, task: Task) {
    client.get_mut().write_all(b"z LOGOUT\r\n").await.unwrap();
    assert!(until(&mut client, "z ").await.contains("z OK"));
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn multiappend_mixed_literals_commit_once_with_distinct_metadata_and_exact_octets() {
    let first = b"Subject: first\r\n\r\n\xff\r\ninjected LOGOUT\r\n";
    let second = b"Subject: second\r\n\r\nsecond";
    let third = "Subject: café\r\n\r\nthird".as_bytes();
    for wrapper in ["UTF8 (", "UTF8 (~", "~"] {
        let (mut client, task, db) = start(first.len() + second.len() + third.len()).await;
        client
            .get_mut()
            .write_all(b"utf8 ENABLE UTF8=ACCEPT\r\n")
            .await
            .unwrap();
        assert!(until(&mut client, "utf8 ").await.contains("utf8 OK"));
        client
            .get_mut()
            .write_all(b"s SELECT INBOX\r\n")
            .await
            .unwrap();
        until(&mut client, "s ").await;
        client
            .get_mut()
            .write_all(
                format!(
                    "b APPEND INBOX (\\Seen first) \"09-Sep-2001 01:46:40 +0000\" {{{}}}\r\n",
                    first.len()
                )
                .as_bytes(),
            )
            .await
            .unwrap();
        assert!(until(&mut client, "+").await.starts_with('+'));
        client.get_mut().write_all(first).await.unwrap();
        client
            .get_mut()
            .write_all(
                format!(
                    " (\\Draft second) \"01-Jan-2020 00:00:00 +0000\" {{{}+}}\r\n",
                    second.len()
                )
                .as_bytes(),
            )
            .await
            .unwrap();
        client.get_mut().write_all(second).await.unwrap();
        client
            .get_mut()
            .write_all(format!(" (third) {wrapper}{{{}}}\r\n", third.len()).as_bytes())
            .await
            .unwrap();
        assert!(until(&mut client, "+").await.starts_with('+'));
        assert!(
            db.account("a").unwrap().messages.is_empty(),
            "batch committed before final literal"
        );
        client.get_mut().write_all(third).await.unwrap();
        if wrapper.starts_with("UTF8") {
            client.get_mut().write_all(b")").await.unwrap();
        }
        client
            .get_mut()
            .write_all(b"\r\nnext NOOP\r\n")
            .await
            .unwrap();
        let response = until(&mut client, "next ").await;
        assert!(response.contains("b OK [APPENDUID 1 1:3]"), "{response}");
        assert!(response.contains("* 3 EXISTS"), "{response}");
        assert!(!response.contains("injected OK"), "{response}");
        let account = db.account("a").unwrap();
        assert_eq!(account.messages.len(), 3);
        assert_eq!(account.messages[0].received_at, 1_000_000_000);
        assert_eq!(account.messages[1].received_at, 1_577_836_800);
        for (i, (raw, flags)) in [
            (first.as_slice(), vec!["$seen", "first"]),
            (second.as_slice(), vec!["$draft", "second"]),
            (third, vec!["third"]),
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(db.blob("a", &account.messages[i].id).unwrap(), raw);
            assert_eq!(account.messages[i].keywords, flags);
            assert_eq!(account.messages[i].mailboxes["inbox"], i as u32 + 1);
        }
        finish(client, task).await;
    }
}

#[tokio::test]
async fn multiappend_late_quota_or_metadata_refusal_leaves_no_uid_or_message() {
    let raw = b"Subject: first\r\n\r\nbody";
    for suffix in [
        "{1}",
        "(\\Recent) {1}",
        "\"31-Feb-2026 01:02:03 +0000\" {1}",
    ] {
        let (mut client, task, db) = start(raw.len()).await;
        let before = db.account("a").unwrap();
        client
            .get_mut()
            .write_all(format!("b APPEND INBOX {{{}+}}\r\n", raw.len()).as_bytes())
            .await
            .unwrap();
        client.get_mut().write_all(raw).await.unwrap();
        client
            .get_mut()
            .write_all(format!(" {suffix}\r\nnext NOOP\r\n").as_bytes())
            .await
            .unwrap();
        let response = until(&mut client, "next ").await;
        assert!(
            response.contains(if suffix == "{1}" {
                "b NO [OVERQUOTA]"
            } else {
                "b BAD"
            }),
            "{response}"
        );
        assert!(!response.contains("+ Ready"), "{response}");
        assert_eq!(db.account("a").unwrap(), before);
        finish(client, task).await;
    }
}

#[tokio::test]
async fn multiappend_late_semantic_failure_rolls_back_first_message_and_blob() {
    let first = b"Subject: first\r\n\r\nbody";
    let invalid = format!(
        "References: {}\r\n\r\nbody",
        (0..1001).map(|n| format!("<id-{n}> ")).collect::<String>()
    );
    let (mut client, task, db) = start(65536).await;
    let before = db.account("a").unwrap();
    client
        .get_mut()
        .write_all(format!("b APPEND INBOX {{{}+}}\r\n", first.len()).as_bytes())
        .await
        .unwrap();
    client.get_mut().write_all(first).await.unwrap();
    client
        .get_mut()
        .write_all(format!(" {{{}+}}\r\n{invalid}\r\nnext NOOP\r\n", invalid.len()).as_bytes())
        .await
        .unwrap();
    let response = until(&mut client, "next ").await;
    assert!(response.contains("b NO"), "{response}");
    assert_eq!(db.account("a").unwrap(), before);
    assert!(db.blob("a", "e1").is_err());
    assert!(db.blob("a", "e2").is_err());
    finish(client, task).await;
}

#[tokio::test]
async fn multiappend_malformed_nonsync_suffix_cannot_inject_commands_or_commit_prefix() {
    for suffix in ["{8+}junk", "{oops+}", "(\\Recent) {8+}"] {
        let raw = b"Subject: retained\r\n\r\nbody";
        let (mut client, task, db) = start(65536).await;
        client
            .get_mut()
            .write_all(format!("b APPEND INBOX {{{}+}}\r\n", raw.len()).as_bytes())
            .await
            .unwrap();
        client.get_mut().write_all(raw).await.unwrap();
        client
            .get_mut()
            .write_all(format!(" {suffix}\r\np CREATE injected\r\n\r\nz LOGOUT\r\n").as_bytes())
            .await
            .unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).await.unwrap();
        assert_eq!(
            task.await.unwrap().unwrap_err().kind(),
            std::io::ErrorKind::InvalidData
        );
        assert!(!response.contains("p OK"), "{response}");
        let account = db.account("a").unwrap();
        assert!(account.messages.is_empty());
        assert_eq!(account.mailboxes.len(), 1);
    }
}

#[tokio::test]
async fn multiappend_last_literal_disconnect_or_revocation_discards_whole_batch() {
    let raw = b"Subject: pending\r\n\r\nbody";
    for revoke in [false, true] {
        let (mut client, task, db) = start(65536).await;
        client
            .get_mut()
            .write_all(format!("b APPEND INBOX {{{}+}}\r\n", raw.len()).as_bytes())
            .await
            .unwrap();
        client.get_mut().write_all(raw).await.unwrap();
        client
            .get_mut()
            .write_all(format!(" {{{}}}\r\n", raw.len()).as_bytes())
            .await
            .unwrap();
        assert!(until(&mut client, "+").await.starts_with('+'));
        assert!(db.account("a").unwrap().messages.is_empty());
        if revoke {
            db.revoke("a").unwrap();
            client.get_mut().write_all(raw).await.unwrap();
            client.get_mut().write_all(b"\r\n").await.unwrap();
            assert!(until(&mut client, "b ").await.contains("b NO"));
            finish(client, task).await;
        } else {
            client.get_mut().write_all(b"short").await.unwrap();
            client.get_mut().shutdown().await.unwrap();
            assert_eq!(
                task.await.unwrap().unwrap_err().kind(),
                std::io::ErrorKind::UnexpectedEof
            );
        }
        assert!(db.account("a").unwrap().messages.is_empty());
    }
}

#[path = "multiappend_review.rs"]
mod review;
