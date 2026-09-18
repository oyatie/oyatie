use base64::{Engine, engine::general_purpose::STANDARD};
use mail_api::{Action, MetadataStore, Policy};
use mail_kernel::{Account, Error};
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite_store::SqliteStore;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, DuplexStream};

const TOKEN: &str = "0123456789abcdef0123456789abcdef";
#[path = "submission/message.rs"]
mod message;

#[test]
fn submission_rejects_forged_or_ambiguous_authorship_before_queueing() {
    let (service, _) = service(Arc::new(OwnerPolicy));
    let recipients = ["alice@example.org".into()];
    for raw in [
        "Subject: no author\r\n\r\nbody",
        "From: mallory@example.org\r\n\r\nbody",
        "From: alice@example.org\r\nFrom: mallory@example.org\r\n\r\nbody",
        "From: alice@example.org, mallory@example.org\r\n\r\nbody",
        "From: alice@example.org\r\nSender: mallory@example.org\r\n\r\nbody",
        "From: alice@example.org\r\nSender: alice@example.org\r\nSender: alice@example.org\r\n\r\nbody",
    ] {
        assert!(
            service
                .submit(
                    TOKEN,
                    "alice@example.org",
                    "alice@example.org",
                    &recipients,
                    raw.as_bytes()
                )
                .is_err(),
            "{raw}"
        );
    }
    assert_eq!(service.deliver_pending(1).unwrap(), 0);
    service
        .submit(
            TOKEN,
            "alice@example.org",
            "alice@example.org",
            &recipients,
            b"From: Alice <ALICE@example.org>\r\nSender: alice@example.org\r\n\r\nbody",
        )
        .unwrap();
    assert_eq!(service.deliver_pending(1).unwrap(), 1);
}

fn service(policy: Arc<dyn Policy>) -> (Arc<MailService>, Arc<SqliteStore>) {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy,
    });
    (service, db)
}

fn plain(authz: &str, username: &str) -> String {
    STANDARD.encode(format!("{authz}\0{username}\0{TOKEN}"))
}

async fn reply(client: &mut BufReader<DuplexStream>, code: &str) -> String {
    let mut result = String::new();
    loop {
        let mut line = String::new();
        assert!(client.read_line(&mut line).await.unwrap() > 0);
        assert!(line.starts_with(code), "{line}");
        result.push_str(&line);
        if line.as_bytes()[3] == b' ' {
            return result;
        }
    }
}

async fn command(client: &mut BufReader<DuplexStream>, command: &str, code: &str) -> String {
    client
        .get_mut()
        .write_all(format!("{command}\r\n").as_bytes())
        .await
        .unwrap();
    reply(client, code).await
}

#[tokio::test]
async fn submission_authenticates_sender_and_rechecks_revocation_before_acceptance() {
    let (service, db) = service(Arc::new(OwnerPolicy));
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol_imap::submission_session(
        server,
        service.clone(),
        true,
    ));
    let mut client = BufReader::new(client);
    reply(&mut client, "220").await;
    assert!(
        command(&mut client, "EHLO client", "250")
            .await
            .contains("AUTH PLAIN")
    );
    command(&mut client, "MAIL FROM:<alice@example.org>", "530").await;
    command(
        &mut client,
        &format!("AUTH PLAIN {}", plain("", "mallory@example.org")),
        "535",
    )
    .await;
    command(
        &mut client,
        &format!(
            "AUTH PLAIN {}",
            plain("bob@example.org", "alice@example.org")
        ),
        "535",
    )
    .await;
    command(&mut client, "AUTH PLAIN", "334").await;
    command(&mut client, &plain("", "alice@example.org"), "235").await;
    command(&mut client, "MAIL FROM:<mallory@example.org>", "550").await;
    command(&mut client, "MAIL FROM:<>", "550").await;
    command(&mut client, "MAIL FROM:<alice@example.org>", "250").await;
    command(&mut client, "AUTH PLAIN", "503").await;
    command(&mut client, "RCPT TO:<alice@example.org>", "250").await;
    command(&mut client, "DATA", "354").await;
    command(
        &mut client,
        "From: alice@example.org\r\n\r\naccepted\r\n.",
        "250",
    )
    .await;
    assert!(db.account("a").unwrap().messages.is_empty());
    assert_eq!(service.deliver_pending(1).unwrap(), 1);
    command(&mut client, "RSET", "250").await;
    assert!(
        !command(&mut client, "EHLO client", "250")
            .await
            .contains("AUTH")
    );
    command(&mut client, "MAIL FROM:<alice@example.org>", "250").await;
    command(&mut client, "RCPT TO:<alice@example.org>", "250").await;
    command(&mut client, "DATA", "354").await;
    db.revoke("a").unwrap();
    command(
        &mut client,
        "From: alice@example.org\r\n\r\nrevoked\r\n.",
        "535",
    )
    .await;
    assert_eq!(service.deliver_pending(1).unwrap(), 0);
    command(&mut client, "QUIT", "221").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn submission_refuses_plaintext_and_does_not_advertise_auth_on_inbound() {
    let (service, _) = service(Arc::new(OwnerPolicy));
    let (mut client, server) = tokio::io::duplex(8192);
    let task = tokio::spawn(mail_protocol_imap::submission_session(
        server,
        service.clone(),
        false,
    ));
    let mut result = String::new();
    client.read_to_string(&mut result).await.unwrap();
    assert!(result.starts_with("554"));
    task.await.unwrap().unwrap();
    let (client, server) = tokio::io::duplex(8192);
    let task = tokio::spawn(mail_protocol_imap::smtp_session(server, service));
    let mut client = BufReader::new(client);
    reply(&mut client, "220").await;
    assert!(
        !command(&mut client, "EHLO client", "250")
            .await
            .contains("AUTH")
    );
    command(&mut client, "AUTH PLAIN", "538").await;
    command(&mut client, "QUIT", "221").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn submission_refuses_malformed_sasl_and_bounds_authentication_attempts() {
    let (service, _) = service(Arc::new(OwnerPolicy));
    let (client, server) = tokio::io::duplex(8192);
    let task = tokio::spawn(mail_protocol_imap::submission_session(
        server, service, true,
    ));
    let mut client = BufReader::new(client);
    reply(&mut client, "220").await;
    command(&mut client, "AUTH PLAIN", "503").await;
    command(&mut client, "HELO client", "250").await;
    command(&mut client, "AUTH PLAIN", "503").await;
    command(&mut client, "EHLO client", "250").await;
    command(&mut client, "AUTH LOGIN", "504").await;
    command(&mut client, "AUTH PLAIN", "334").await;
    command(&mut client, "*", "501").await;
    command(&mut client, "AUTH PLAIN !!!!", "501").await;
    command(
        &mut client,
        &format!("AUTH PLAIN {}", plain("", "other@example.org")),
        "535",
    )
    .await;
    reply(&mut client, "421").await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn submission_requires_policy_permission_separate_from_mailbox_write() {
    struct Deny;
    impl Policy for Deny {
        fn authorize(
            &self,
            _: &mail_api::Principal,
            action: Action,
            _: &mail_api::AccountInfo,
        ) -> Result<(), Error> {
            if action == Action::Submit {
                Err(Error::Forbidden)
            } else {
                Ok(())
            }
        }
    }
    let (service, _) = service(Arc::new(Deny));
    let (client, server) = tokio::io::duplex(8192);
    let task = tokio::spawn(mail_protocol_imap::submission_session(
        server, service, true,
    ));
    let mut client = BufReader::new(client);
    reply(&mut client, "220").await;
    command(&mut client, "EHLO client", "250").await;
    command(
        &mut client,
        &format!("AUTH PLAIN {}", plain("", "alice@example.org")),
        "535",
    )
    .await;
    command(&mut client, "MAIL FROM:<alice@example.org>", "530").await;
    command(&mut client, "QUIT", "221").await;
    task.await.unwrap().unwrap();
}
