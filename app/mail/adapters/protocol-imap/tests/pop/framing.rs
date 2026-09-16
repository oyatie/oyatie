use super::support::*;

#[tokio::test]
async fn retrieval_preserves_binary_bytes_dot_stuffs_and_terminates_partial_last_line() {
    let (service, db) = service();
    let raw = b"Subject: octets\r\n\r\n.dot\r\n..two\r\n\xff\0tail";
    deliver(&db, raw);
    let (mut client, task) = Client::connect(service, true).await;
    client.login().await;
    assert_eq!(
        client.ok("LIST 1").await,
        format!("+OK 1 {}\r\n", raw.len()).as_bytes()
    );
    let mut expected = format!("+OK {} octets\r\n", raw.len()).into_bytes();
    expected.extend_from_slice(b"Subject: octets\r\n\r\n..dot\r\n...two\r\n\xff\0tail\r\n.\r\n");
    assert_eq!(client.multiline("RETR 1").await, expected);
    for command in [
        "RETR 0",
        "RETR -1",
        "LIST 1 extra",
        "STAT extra",
        "TOP 1",
        "TOP 1 -1",
        "DELE 4294967296",
    ] {
        client.error(command).await;
    }
    client.close(task).await;
}

#[tokio::test]
async fn list_and_uidl_keep_original_numbers_after_deletion() {
    let (service, db) = service();
    for _ in 0..3 {
        deliver(&db, b"Subject: x\r\n\r\nx\r\n");
    }
    let (mut client, task) = Client::connect(service, true).await;
    client.login().await;
    client.ok("DELE 2").await;
    let list = String::from_utf8(client.multiline("LIST").await).unwrap();
    assert!(list.starts_with("+OK 2 messages\r\n1 "));
    assert!(!list.contains("\r\n2 "));
    assert!(list.contains("\r\n3 "));
    let uidl = String::from_utf8(client.multiline("UIDL").await).unwrap();
    assert!(uidl.contains("\r\n1 ") && uidl.contains("\r\n3 "));
    assert!(!uidl.contains("\r\n2 "));
    client.close(task).await;
}

#[tokio::test]
async fn top_matches_stalwart_total_line_count_and_zero_means_full_message() {
    let (service, db) = service();
    let raw = b"From: a@example.org\r\nTo: alice@example.org\r\nSubject: x\r\nX-Test: y\r\n\r\nbody\r\n.second\r\n";
    deliver(&db, raw);
    let (mut client, task) = Client::connect(service, true).await;
    client.login().await;
    let top = client.multiline("TOP 1 4").await;
    assert!(top.ends_with(b"X-Test: y\r\n.\r\n"));
    assert!(!String::from_utf8_lossy(&top).contains("body"));
    assert_eq!(
        client.multiline("TOP 1 0").await,
        client.multiline("RETR 1").await
    );
    client.close(task).await;
}

#[tokio::test]
async fn bare_lf_and_oversized_commands_close_without_committing_pending_deletions() {
    use mail_api::Store;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    for command in [
        b"NOOP\n".to_vec(),
        format!("{}\r\n", "x".repeat(513)).into_bytes(),
    ] {
        let (service, db) = service();
        deliver(&db, b"Subject: retained\r\n\r\nmail\r\n");
        let (mut client, task) = Client::connect(service, true).await;
        client.login().await;
        client.ok("DELE 1").await;
        client.0.get_mut().write_all(&command).await.unwrap();
        let mut rest = vec![];
        client.0.read_to_end(&mut rest).await.unwrap();
        assert_eq!(
            task.await.unwrap().unwrap_err().kind(),
            std::io::ErrorKind::InvalidData
        );
        assert_eq!(db.account("a").unwrap().messages.len(), 1);
    }
}

#[tokio::test]
async fn bare_lf_is_canonicalized_and_an_isolated_final_cr_is_preserved() {
    let (service, db) = service();
    let raw = b"Subject: x\n\n.dot\ntrailing\r";
    deliver(&db, raw);
    let (mut client, task) = Client::connect(service, true).await;
    client.login().await;
    let response = client.multiline("RETR 1").await;
    assert!(response.ends_with(b"Subject: x\r\n\r\n..dot\r\ntrailing\r\r\n.\r\n"));
    client.close(task).await;
}

#[tokio::test]
async fn stalled_retrieval_does_not_block_other_sessions_and_disconnect_releases_the_writer() {
    use std::time::Duration;
    use tokio::io::AsyncWriteExt;
    let (service, db) = service();
    let mut raw = b"Subject: large\r\n\r\n".to_vec();
    raw.extend(vec![b'x'; 2 * 1024 * 1024]);
    deliver(&db, &raw);
    let (mut slow, slow_task) = Client::connect(service.clone(), true).await;
    slow.login().await;
    slow.0.get_mut().write_all(b"RETR 1\r\n").await.unwrap();
    assert!(slow.read().await.starts_with(b"+OK"));
    let (mut fast, fast_task) = Client::connect(service, true).await;
    fast.login().await;
    assert!(fast.ok("STAT").await.starts_with(b"+OK 1 "));
    fast.close(fast_task).await;
    drop(slow);
    assert!(
        tokio::time::timeout(Duration::from_secs(2), slow_task)
            .await
            .unwrap()
            .unwrap()
            .is_err()
    );
}
