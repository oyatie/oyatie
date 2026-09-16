use mail_api::{DeliveryOutcome, MailTransport, QueuedMessage};
use mail_smtp_relay::{Relay, RelayConfig};
use rustls::{ClientConfig, RootCertStore, ServerConfig, pki_types::PrivatePkcs8KeyDer};
use std::sync::Arc;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};
use tokio_rustls::TlsAcceptor;

async fn exchange(implicit: bool, reject: u16, trusted: bool, delay: u64) -> DeliveryOutcome {
    let rcgen::CertifiedKey { cert, signing_key } =
        rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let mut roots = RootCertStore::empty();
    if trusted {
        roots.add(cert.der().to_vec().into()).unwrap();
    }
    let client = Arc::new(
        ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth(),
    );
    let server = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![cert.der().to_vec().into()],
            PrivatePkcs8KeyDer::from(signing_key.serialize_der()).into(),
        )
        .unwrap();
    let tls = TlsAcceptor::from(Arc::new(server));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let task = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut stream = BufReader::new(stream);
        if !implicit {
            stream.get_mut().write_all(b"220 relay\r\n").await.unwrap();
            expect(&mut stream, "EHLO mail.example.org\r\n").await;
            stream
                .get_mut()
                .write_all(b"250-relay\r\n250 STARTTLS\r\n")
                .await
                .unwrap();
            expect(&mut stream, "STARTTLS\r\n").await;
            stream.get_mut().write_all(b"220 Ready\r\n").await.unwrap();
        }
        let Ok(stream) = tls.accept(stream.into_inner()).await else {
            assert!(!trusted);
            return;
        };
        let mut stream = BufReader::new(stream);
        if implicit {
            stream.get_mut().write_all(b"220 relay\r\n").await.unwrap();
        }
        expect(&mut stream, "EHLO mail.example.org\r\n").await;
        stream
            .get_mut()
            .write_all(b"250-relay\r\n250-8BITMIME\r\n250 AUTH PLAIN\r\n")
            .await
            .unwrap();
        use base64::{Engine, engine::general_purpose::STANDARD};
        expect(
            &mut stream,
            &format!("AUTH PLAIN {}\r\n", STANDARD.encode(b"\0user\0secret")),
        )
        .await;
        stream
            .get_mut()
            .write_all(b"235 Authenticated\r\n")
            .await
            .unwrap();
        expect(&mut stream, "MAIL FROM:<alice@example.org>\r\n").await;
        stream.get_mut().write_all(b"250 Sender\r\n").await.unwrap();
        expect(&mut stream, "RCPT TO:<bob@remote.org>\r\n").await;
        if reject != 0 {
            stream
                .get_mut()
                .write_all(format!("{reject} Refused\r\n").as_bytes())
                .await
                .unwrap();
            return;
        }
        stream
            .get_mut()
            .write_all(b"250 Recipient\r\n")
            .await
            .unwrap();
        expect(&mut stream, "DATA\r\n").await;
        stream
            .get_mut()
            .write_all(b"354 Content\r\n")
            .await
            .unwrap();
        expect(&mut stream, "From: alice@example.org\r\n").await;
        expect(&mut stream, "\r\n").await;
        expect(&mut stream, "..dot\r\n").await;
        expect(&mut stream, ".\r\n").await;
        tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
        stream
            .get_mut()
            .write_all(b"250 Accepted\r\n")
            .await
            .unwrap();
        // Disconnect after final acceptance: QUIT failure cannot retry accepted mail.
    });
    let relay = Relay::new(
        RelayConfig {
            host: "localhost".into(),
            port,
            helo: "mail.example.org".into(),
            implicit_tls: implicit,
            credentials: Some(("user".into(), "secret".into())),
        },
        client,
    )
    .unwrap();
    let result = relay
        .send(
            "bob@remote.org",
            &QueuedMessage {
                sender: "alice@example.org".into(),
                raw: b"From: alice@example.org\r\n\r\n.dot\r\n".to_vec(),
                received_at: 0,
                retry_at: 0,
            },
        )
        .await;
    task.await.unwrap();
    result
}

async fn expect<S: tokio::io::AsyncRead + Unpin>(stream: &mut BufReader<S>, expected: &str) {
    let mut line = String::new();
    stream.read_line(&mut line).await.unwrap();
    assert_eq!(line, expected);
}

#[tokio::test]
async fn relay_uses_verified_tls_and_preserves_data_and_smtp_outcomes() {
    assert_eq!(exchange(true, 0, true, 0).await, DeliveryOutcome::Delivered);
    assert_eq!(
        exchange(false, 0, true, 0).await,
        DeliveryOutcome::Delivered
    );
    assert_eq!(
        exchange(true, 450, true, 0).await,
        DeliveryOutcome::Temporary(450)
    );
    assert_eq!(
        exchange(true, 550, true, 0).await,
        DeliveryOutcome::Permanent(550)
    );
    assert_eq!(
        exchange(true, 0, false, 0).await,
        DeliveryOutcome::Temporary(451)
    );
}

#[tokio::test]
async fn relay_refuses_command_injection_and_non_crlf_messages() {
    let config = Arc::new(
        ClientConfig::builder()
            .with_root_certificates(RootCertStore::empty())
            .with_no_client_auth(),
    );
    let settings = || RelayConfig {
        host: "localhost".into(),
        port: 1,
        helo: "mail.example.org".into(),
        implicit_tls: true,
        credentials: None,
    };
    let mut bad = settings();
    bad.helo = "host\r\nMAIL FROM:<>".into();
    assert!(Relay::new(bad, config.clone()).is_err());
    let relay = Relay::new(settings(), config).unwrap();
    for (recipient, raw) in [
        ("bob@remote.org\r\nDATA", b"body\r\n".as_slice()),
        ("bob@remote.org", b"body\n.\r\nMAIL FROM:<>\r\n"),
    ] {
        assert_eq!(
            relay
                .send(
                    recipient,
                    &QueuedMessage {
                        sender: "alice@example.org".into(),
                        raw: raw.to_vec(),
                        received_at: 0,
                        retry_at: 0,
                    }
                )
                .await,
            DeliveryOutcome::Permanent(554)
        );
    }
}

#[tokio::test]
async fn relay_refuses_tls_downgrade_and_malformed_replies_before_credentials() {
    use tokio::io::AsyncReadExt;
    for (greeting, hello) in [
        (
            b"220 relay\r\n".to_vec(),
            b"250-relay\r\n250 AUTH PLAIN\r\n".to_vec(),
        ),
        (
            b"220 relay\r\n".to_vec(),
            b"250-relay\r\n550 STARTTLS\r\n".to_vec(),
        ),
        (
            b"220 relay\r\n".to_vec(),
            format!("250-{}\r\n250 STARTTLS\r\n", "x".repeat(512)).into_bytes(),
        ),
        (b"220 bare newline\n".to_vec(), Vec::new()),
    ] {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let task = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut stream = BufReader::new(stream);
            stream.get_mut().write_all(&greeting).await.unwrap();
            if !hello.is_empty() {
                expect(&mut stream, "EHLO mail.example.org\r\n").await;
                stream.get_mut().write_all(&hello).await.unwrap();
            }
            let mut pending = Vec::new();
            tokio::time::timeout(
                std::time::Duration::from_secs(5),
                stream.read_to_end(&mut pending),
            )
            .await
            .unwrap()
            .unwrap();
            assert!(
                pending.is_empty(),
                "credentials or commands escaped onto refused connection"
            );
        });
        let relay = Relay::new(
            RelayConfig {
                host: "localhost".into(),
                port,
                helo: "mail.example.org".into(),
                implicit_tls: false,
                credentials: Some(("user".into(), "secret".into())),
            },
            Arc::new(
                ClientConfig::builder()
                    .with_root_certificates(RootCertStore::empty())
                    .with_no_client_auth(),
            ),
        )
        .unwrap();
        assert_eq!(
            relay
                .send(
                    "bob@remote.org",
                    &QueuedMessage {
                        sender: "alice@example.org".into(),
                        raw: b"From: alice@example.org\r\n\r\nbody\r\n".to_vec(),
                        received_at: 0,
                        retry_at: 0,
                    }
                )
                .await,
            DeliveryOutcome::Temporary(451)
        );
        task.await.unwrap();
    }
}

#[tokio::test]
async fn relay_waits_for_final_acceptance_beyond_the_previous_ninety_second_cutoff() {
    assert_eq!(
        exchange(true, 0, true, 91).await,
        DeliveryOutcome::Delivered
    );
}
