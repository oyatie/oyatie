use super::*;
use std::{io, time::Duration};

#[tokio::test]
async fn imap_starttls_rejects_buffered_plaintext_without_invoking_upgrade() {
    let (service, _) = service();
    let (mut client, server) = tokio::io::duplex(8192);
    client
        .write_all(format!("a STARTTLS\r\nb LOGIN alice@example.org {TOKEN}\r\n").as_bytes())
        .await
        .unwrap();
    let task = tokio::spawn(mail_protocol_imap::imap_starttls_session(
        server,
        service,
        |_| -> std::future::Ready<io::Result<tokio::io::DuplexStream>> {
            panic!("pipelined plaintext must never invoke TLS upgrade");
        },
    ));
    let mut transcript = String::new();
    client.read_to_string(&mut transcript).await.unwrap();
    task.await.unwrap().unwrap();
    assert!(
        transcript.contains("* BYE Pipelined STARTTLS refused"),
        "{transcript}"
    );
    assert!(!transcript.contains("a OK") && !transcript.contains("b OK"));
}

#[tokio::test]
async fn imap_starttls_bounds_handshake_and_propagates_failure() {
    for stalled in [false, true] {
        let (service, _) = service();
        let (client, server) = tokio::io::duplex(8192);
        let task = tokio::spawn(mail_protocol_imap::imap_starttls_session(
            server,
            service,
            move |_| async move {
                if stalled {
                    std::future::pending::<()>().await;
                }
                Err::<tokio::io::DuplexStream, _>(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "TLS rejected",
                ))
            },
        ));
        let mut client = BufReader::new(client);
        super::runtime::ready(&mut client, "a STARTTLS\r\n", "a ").await;
        let error = tokio::time::timeout(Duration::from_secs(15), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err();
        assert_eq!(
            error.kind(),
            if stalled {
                io::ErrorKind::TimedOut
            } else {
                io::ErrorKind::InvalidData
            }
        );
    }
}

async fn smtp_ready(client: &mut BufReader<tokio::io::DuplexStream>, command: &str, code: &str) {
    client
        .get_mut()
        .write_all(command.as_bytes())
        .await
        .unwrap();
    loop {
        let mut line = String::new();
        assert!(client.read_line(&mut line).await.unwrap() > 0);
        assert!(line.starts_with(code), "{line}");
        if line.as_bytes().get(3) == Some(&b' ') {
            break;
        }
    }
}

#[tokio::test]
async fn smtp_starttls_rejects_buffered_plaintext_without_invoking_upgrade() {
    for submission in [false, true] {
        let (service, db) = service();
        let (client, server) = tokio::io::duplex(8192);
        let task = tokio::spawn(mail_protocol_imap::smtp_starttls_session(
            server,
            service,
            submission,
            |_| -> std::future::Ready<io::Result<tokio::io::DuplexStream>> {
                panic!("pipelined plaintext must never invoke TLS upgrade");
            },
        ));
        let mut client = BufReader::new(client);
        smtp_ready(&mut client, "", "220").await;
        smtp_ready(&mut client, "EHLO client\r\n", "250").await;
        client
            .get_mut()
            .write_all(b"STARTTLS\r\nMAIL FROM:<alice@example.org>\r\n")
            .await
            .unwrap();
        let mut transcript = String::new();
        client.read_to_string(&mut transcript).await.unwrap();
        task.await.unwrap().unwrap();
        assert!(
            transcript.contains("554 5.5.1 Pipelined STARTTLS refused"),
            "{transcript}"
        );
        assert!(!transcript.contains("Sender accepted"));
        assert!(db.account("a").unwrap().messages.is_empty());
    }
}

#[tokio::test]
async fn smtp_starttls_bounds_handshake_and_propagates_failure() {
    for stalled in [false, true] {
        let (service, _) = service();
        let (client, server) = tokio::io::duplex(8192);
        let task = tokio::spawn(mail_protocol_imap::smtp_starttls_session(
            server,
            service,
            true,
            move |_| async move {
                if stalled {
                    std::future::pending::<()>().await;
                }
                Err::<tokio::io::DuplexStream, _>(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "TLS rejected",
                ))
            },
        ));
        let mut client = BufReader::new(client);
        smtp_ready(&mut client, "", "220").await;
        smtp_ready(&mut client, "EHLO client\r\n", "250").await;
        smtp_ready(&mut client, "STARTTLS\r\n", "220").await;
        let error = tokio::time::timeout(Duration::from_secs(15), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err();
        assert_eq!(
            error.kind(),
            if stalled {
                io::ErrorKind::TimedOut
            } else {
                io::ErrorKind::InvalidData
            }
        );
    }
}
