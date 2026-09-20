//! SMTP on the wire: byte preservation, relay refusal, and the session
//! transfer quota covering the message body.
use super::*;

/// The session's transfer quota covers the message body, not only the
/// command lines: the reference charges every byte the peer sends.
#[tokio::test]
async fn smtp_transfer_quota_covers_the_data_body_and_closes_the_session() {
    let (service, db) = service();
    let params = mail_protocol_imap::SmtpParams {
        transfer_bytes: 120,
        ..mail_protocol_imap::SmtpParams::default()
    };
    let (mut client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(async move {
        mail_protocol_imap::smtp_session_with(server, service, &params).await
    });
    let body = "x".repeat(200);
    client
        .write_all(
            format!("EHLO c\r\nMAIL FROM:<s@example.net>\r\nRCPT TO:<alice@example.org>\r\nDATA\r\n{body}\r\n.\r\n")
                .as_bytes(),
        )
        .await
        .unwrap();
    let mut result = String::new();
    client.read_to_string(&mut result).await.unwrap();
    task.await.unwrap().unwrap();
    assert!(result.contains("354 "), "{result}");
    assert!(result.contains("452 4.7.28"), "{result}");
    assert!(!result.contains("250 2.0.0 Queued"), "{result}");
    assert!(db.account("a").unwrap().messages.is_empty());
}

#[tokio::test]
async fn smtp_is_byte_preserving_rejects_relay_and_commits_before_ack() {
    let (service, db) = service();
    let (mut client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::smtp_session(server, service.clone()));
    client.write_all(b"EHLO client.example\r\nMAIL FROM:<>\r\nRCPT TO:<x@remote.example>\r\nRCPT TO:<alice@example.org>\r\nDATA\r\nSubject: wire\r\n\r\n..dot\r\n\xff\r\n.\r\nQUIT\r\n").await.unwrap();
    let mut result = String::new();
    client.read_to_string(&mut result).await.unwrap();
    task.await.unwrap().unwrap();
    assert!(result.contains("550 "));
    assert!(result.contains("354 "));
    assert!(result.contains("250 2.0.0 Queued"));
    assert!(db.account("a").unwrap().messages.is_empty());
    assert_eq!(service.deliver_pending(1).unwrap(), 1);
    assert_eq!(
        db.blob("a", "e1").unwrap(),
        b"Subject: wire\r\n\r\n.dot\r\n\xff\r\n"
    );
}

/// RFC 3207 §4.2 discards session state at the upgrade, not the bytes the
/// peer has already spent: one quota and one lifetime per connection.
#[tokio::test]
async fn the_transfer_quota_is_not_replenished_by_a_starttls_upgrade() {
    let (service, _db) = service();
    let params = mail_protocol_imap::SmtpParams {
        transfer_bytes: 60,
        ..mail_protocol_imap::SmtpParams::default()
    };
    let (mut client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(async move {
        mail_protocol_imap::smtp_starttls_session_with(
            server,
            service,
            false,
            |s| async move { Ok(s) },
            &params,
        )
        .await
    });
    // 35 bytes before the upgrade, more after: the quota covers both
    // halves. Each command is sent on its own (pipelined STARTTLS is
    // refused) and its reply drained before the next.
    let mut buf = [0u8; 4096];
    let mut result = String::new();
    for command in [
        "EHLO client.example.net\r\n",
        "STARTTLS\r\n",
        "EHLO client.example.net\r\n",
        "NOOP\r\n",
    ] {
        client.write_all(command.as_bytes()).await.unwrap();
        if let Ok(Ok(n)) =
            tokio::time::timeout(std::time::Duration::from_millis(200), client.read(&mut buf)).await
        {
            result.push_str(&String::from_utf8_lossy(&buf[..n]));
        }
    }
    drop(client);
    task.await.unwrap().unwrap();
    assert!(result.contains("220 2.0.0 Ready to start TLS."), "{result}");
    assert!(result.contains("452 4.7.28"), "{result}");
}

/// An over-long command line is refused once, and the command after it is
/// still answered: the reply lockstep survives a peer's bad line.
#[tokio::test]
async fn an_over_long_command_line_is_refused_without_swallowing_the_next_command() {
    let (service, _db) = service();
    let (mut client, server) = tokio::io::duplex(1024 * 1024);
    let task = tokio::spawn(mail_protocol_imap::smtp_session(server, service));
    // Sized so the read that crosses the 64 KiB bound is also the one
    // carrying the newline — the case that can swallow the next command.
    let mut command = vec![b'A'; 65540];
    command.extend_from_slice(b"\r\nNOOP\r\nQUIT\r\n");
    client.write_all(&command).await.unwrap();
    let mut result = String::new();
    client.read_to_string(&mut result).await.unwrap();
    task.await.unwrap().unwrap();
    assert_eq!(result.matches("554 5.3.4").count(), 1, "{result}");
    assert!(result.contains("250 2.0.0 OK"), "NOOP unanswered: {result}");
    assert!(result.contains("221 2.0.0 Bye."), "{result}");
}

/// The advertised `SIZE` is the session's own ceiling, and it binds every
/// place that can refuse an oversized message: a client that believes the
/// banner must not be accepted at `MAIL FROM` and then refused at the dot,
/// and a client that declares nothing must still be stopped by the reader.
#[tokio::test]
async fn the_advertised_size_is_the_ceiling_the_session_actually_enforces() {
    let params = mail_protocol_imap::SmtpParams {
        max_message_size: 512,
        ..mail_protocol_imap::SmtpParams::default()
    };
    let session = |script: String| {
        let (service, db) = service();
        let params = params.clone();
        async move {
            let (mut client, server) = tokio::io::duplex(65536);
            let task = tokio::spawn(async move {
                mail_protocol_imap::smtp_session_with(server, service, &params).await
            });
            client.write_all(script.as_bytes()).await.unwrap();
            let mut result = String::new();
            client.read_to_string(&mut result).await.unwrap();
            task.await.unwrap().unwrap();
            (result, db)
        }
    };

    // The banner advertises this session's ceiling, not the build's constant.
    let (advertised, _) = session("EHLO client.example\r\nQUIT\r\n".to_owned()).await;
    assert!(advertised.contains("250-SIZE 512\r\n"), "{advertised}");
    assert!(
        !advertised.contains(&format!("SIZE {}", mail_kernel::MAX_MESSAGE_BYTES)),
        "{advertised}"
    );

    // A declared size over the ceiling is refused before the body is sent.
    let (declared, db) =
        session("EHLO client.example\r\nMAIL FROM:<s@example.net> SIZE=513\r\nQUIT\r\n".to_owned())
            .await;
    assert!(declared.contains("552 5.3.4"), "{declared}");
    assert!(db.account("a").unwrap().messages.is_empty());

    // An undeclared body over the ceiling is refused by the reader.
    let body = "x".repeat(600);
    let (undeclared, db) = session(format!(
        "EHLO client.example\r\nMAIL FROM:<s@example.net>\r\nRCPT TO:<alice@example.org>\r\nDATA\r\n{body}\r\n.\r\nQUIT\r\n"
    ))
    .await;
    assert!(undeclared.contains("552 5.3.4"), "{undeclared}");
    assert!(!undeclared.contains("250 2.0.0 Queued"), "{undeclared}");
    assert!(db.account("a").unwrap().messages.is_empty());
}
