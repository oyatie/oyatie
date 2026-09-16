use super::support::*;
use base64::{Engine, engine::general_purpose::STANDARD};

#[tokio::test]
async fn user_pass_capa_and_plain_sasl_follow_authorization_state() {
    let (service, _) = service();
    let (mut client, task) = Client::connect(service.clone(), true).await;
    let capa = String::from_utf8(client.multiline("CAPA").await).unwrap();
    for capability in ["SASL PLAIN", "IMPLEMENTATION", "UIDL", "TOP", "PIPELINING"] {
        assert!(capa.contains(capability), "{capa}");
    }
    client.ok("NOOP").await;
    client.error(&format!("PASS {TOKEN}")).await;
    client.ok("USER alice@example.org").await;
    client.error("PASS wrong_secret").await;
    client.login().await;
    client.error("USER alice@example.org").await;
    client.ok("UTF8").await;
    client.close(task).await;
    let (mut client, task) = Client::connect(service, true).await;
    let response = STANDARD.encode(format!("\0alice@example.org\0{TOKEN}"));
    client.ok(&format!("AUTH PLAIN {response}")).await;
    client.close(task).await;
}

#[tokio::test]
async fn plaintext_and_cross_account_credentials_cannot_authenticate() {
    let (service, _) = service();
    let (mut client, task) = Client::connect(service.clone(), false).await;
    let capa = String::from_utf8(client.multiline("CAPA").await).unwrap();
    assert!(!capa.contains("SASL PLAIN"));
    client.error("USER alice@example.org").await;
    client.error(&format!("PASS {TOKEN}")).await;
    client.error("AUTH PLAIN").await;
    client.close(task).await;
    let (mut client, task) = Client::connect(service, true).await;
    client.ok("USER bob@example.org").await;
    client.error(&format!("PASS {TOKEN}")).await;
    let response = STANDARD.encode(format!("bob@example.org\0alice@example.org\0{TOKEN}"));
    client.error(&format!("AUTH PLAIN {response}")).await;
    client.login().await;
    client.close(task).await;
}

#[tokio::test]
async fn sasl_continuation_cancel_and_malformed_input_do_not_authenticate() {
    let (service, _) = service();
    let (mut client, task) = Client::connect(service, true).await;
    assert_eq!(client.command("AUTH PLAIN").await, b"+ \r\n");
    client.error("*").await;
    client.error("STAT").await;
    client.error("AUTH PLAIN invalid-base64").await;
    client.error("STAT").await;
    assert_eq!(client.command("AUTH PLAIN").await, b"+ \r\n");
    client
        .ok(&STANDARD.encode(format!("\0alice@example.org\0{TOKEN}")))
        .await;
    client.close(task).await;
}

#[tokio::test]
async fn stls_rejects_pipelined_plaintext_and_does_not_call_upgrade() {
    use std::io;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let (service, _) = service();
    let (mut client, server) = tokio::io::duplex(8192);
    client
        .write_all(format!("STLS\r\nUSER alice@example.org\r\nPASS {TOKEN}\r\n").as_bytes())
        .await
        .unwrap();
    let task = tokio::spawn(mail_protocol_imap::pop_starttls_session(
        server,
        service,
        |_| -> std::future::Ready<io::Result<tokio::io::DuplexStream>> {
            panic!("buffered plaintext must not enter TLS upgrade")
        },
    ));
    let mut transcript = String::new();
    client.read_to_string(&mut transcript).await.unwrap();
    task.await.unwrap().unwrap();
    assert!(transcript.contains("-ERR Pipelined STLS refused"));
    assert!(!transcript.contains("Authenticated"));
}

#[tokio::test]
async fn stls_upgrade_resets_state_without_a_second_greeting() {
    use tokio::io::BufReader;
    let (service, _) = service();
    let (client, server) = tokio::io::duplex(8192);
    let task = tokio::spawn(mail_protocol_imap::pop_starttls_session(
        server,
        service,
        |stream| std::future::ready(Ok(stream)),
    ));
    let mut client = Client(BufReader::new(client));
    assert!(client.read().await.starts_with(b"+OK"));
    let capa = String::from_utf8(client.multiline("CAPA").await).unwrap();
    assert!(capa.contains("STLS"));
    client.ok("STLS").await;
    client.error(&format!("PASS {TOKEN}")).await;
    let capa = String::from_utf8(client.multiline("CAPA").await).unwrap();
    assert!(capa.contains("SASL PLAIN") && !capa.contains("STLS"));
    client.error("STLS").await;
    client.login().await;
    client.close(task).await;
}

#[tokio::test]
async fn stls_propagates_handshake_failure() {
    use std::io;
    use tokio::io::BufReader;
    let (service, _) = service();
    let (client, server) = tokio::io::duplex(8192);
    let task = tokio::spawn(mail_protocol_imap::pop_starttls_session(
        server,
        service,
        |_| {
            std::future::ready(Err::<tokio::io::DuplexStream, _>(io::Error::new(
                io::ErrorKind::InvalidData,
                "TLS rejected",
            )))
        },
    ));
    let mut client = Client(BufReader::new(client));
    client.read().await;
    client.ok("STLS").await;
    assert_eq!(
        task.await.unwrap().unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
}

#[tokio::test]
async fn repeated_authentication_failure_closes_the_connection() {
    use tokio::io::AsyncReadExt;
    let (service, _) = service();
    let (mut client, task) = Client::connect(service, true).await;
    for _ in 0..5 {
        client.ok("USER alice@example.org").await;
        client.error("PASS wrong").await;
    }
    let mut rest = String::new();
    client.0.read_to_string(&mut rest).await.unwrap();
    task.await.unwrap().unwrap();
    assert_eq!(rest, "-ERR Too many authentication failures\r\n");
}
