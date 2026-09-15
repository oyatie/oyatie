use mail_api::{DeliveryOutcome, MailTransport, QueuedMessage};
use mail_relay::{MxConfig, MxTransport};
use rustls::{ClientConfig, RootCertStore};
use std::sync::Arc;
use support::{Dns, smtp};

fn message() -> QueuedMessage {
    QueuedMessage {
        sender: "sender@example.org".into(),
        raw: b"From: sender@example.org\r\n\r\n.dot\r\n".to_vec(),
        received_at: 0,
        retry_at: 0,
    }
}

fn transport(dns: &Dns, port: u16, required: bool) -> MxTransport {
    MxTransport::new(
        MxConfig {
            helo: "outgoing.example.org".into(),
            port,
            dns_servers: vec![dns.address],
            require_tls: required,
        },
        Arc::new(
            ClientConfig::builder()
                .with_root_certificates(RootCertStore::empty())
                .with_no_client_auth(),
        ),
    )
    .unwrap()
}

#[tokio::test]
async fn mx_preferences_fallback_addresses_and_final_acceptance() {
    let dns = Dns::start().await;
    let (port, received) = smtp(0).await;
    assert_eq!(
        transport(&dns, port, false)
            .send("user@mx.example", &message())
            .await,
        DeliveryOutcome::Delivered
    );
    assert_eq!(received.await.unwrap(), message().raw);
    let queries = dns.queries.lock().unwrap();
    let failed = queries
        .iter()
        .position(|q| q.starts_with("down.mx.example."))
        .unwrap();
    let good = queries
        .iter()
        .position(|q| q.starts_with("up.mx.example."))
        .unwrap();
    assert!(
        failed < good,
        "MX preference order was ignored: {queries:?}"
    );
}

#[tokio::test]
async fn no_mx_uses_implicit_address_but_null_mx_and_nxdomain_are_permanent() {
    let dns = Dns::start().await;
    for domain in ["null.example", "missing.example"] {
        assert_eq!(
            transport(&dns, 25, false)
                .send(&format!("u@{domain}"), &message())
                .await,
            DeliveryOutcome::Permanent(556)
        );
    }
    assert_eq!(
        transport(&dns, 25, false)
            .send("u@servfail.example", &message())
            .await,
        DeliveryOutcome::Temporary(451)
    );
    let (port, received) = smtp(0).await;
    assert_eq!(
        transport(&dns, port, false)
            .send("u@implicit.example", &message())
            .await,
        DeliveryOutcome::Delivered
    );
    received.await.unwrap();
    assert!(
        !dns.queries
            .lock()
            .unwrap()
            .iter()
            .any(|q| q == "null.example. A" || q == "missing.example. A")
    );
}

#[tokio::test]
async fn required_tls_refuses_plaintext_and_smtp_refusal_is_not_reclassified() {
    let dns = Dns::start().await;
    let (port, received) = smtp(0).await;
    assert_eq!(
        transport(&dns, port, true)
            .send("u@implicit.example", &message())
            .await,
        DeliveryOutcome::Temporary(451)
    );
    assert!(received.await.unwrap().is_empty());
    let (port, received) = smtp(550).await;
    assert_eq!(
        transport(&dns, port, false)
            .send("u@implicit.example", &message())
            .await,
        DeliveryOutcome::Permanent(550)
    );
    assert!(received.await.unwrap().is_empty());
}

#[tokio::test]
async fn malformed_envelopes_are_refused_before_dns() {
    let dns = Dns::start().await;
    let transport = transport(&dns, 25, false);
    for recipient in ["u@bad\r\nDATA", "missing-domain", "u@."] {
        assert_eq!(
            transport.send(recipient, &message()).await,
            DeliveryOutcome::Permanent(554)
        );
    }
    assert!(dns.queries.lock().unwrap().is_empty());
}

#[path = "mx/support.rs"]
mod support;

#[path = "mx/legacy_review.rs"]
mod legacy_review;
#[path = "mx/review.rs"]
mod review;
