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
