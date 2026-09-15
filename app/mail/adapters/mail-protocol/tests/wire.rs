use mail_api::Store;
use mail_kernel::Account;
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite::SqliteStore;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

const TOKEN: &str = "0123456789abcdef0123456789abcdef";

#[tokio::test]
async fn smtp_rejects_unsupported_message_complexity_permanently() {
    let (service, _) = service();
    let (mut client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::smtp_session(server, service.clone()));
    let references = (0..1001)
        .map(|n| format!(" <message-{n}>\r\n"))
        .collect::<String>();
    client.write_all(format!("EHLO client\r\nMAIL FROM:<>\r\nRCPT TO:<alice@example.org>\r\nDATA\r\nReferences:\r\n{references}\r\nbody\r\n.\r\nQUIT\r\n").as_bytes()).await.unwrap();
    let mut transcript = String::new();
    client.read_to_string(&mut transcript).await.unwrap();
    task.await.unwrap().unwrap();
    assert!(transcript.contains("554 5.6.0"), "{transcript}");
    assert!(!transcript.contains("250 2.0.0 Queued"));
    assert_eq!(service.deliver_pending(1).unwrap(), 0);
}

#[tokio::test]
async fn smtp_closes_oversized_data_without_waiting_for_the_terminator() {
    let (service, db) = service();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::smtp_session(server, service.clone()));
    let (mut reader, mut writer) = tokio::io::split(client);
    let writing = tokio::spawn(async move {
        writer
            .write_all(b"EHLO client\r\nMAIL FROM:<>\r\nRCPT TO:<alice@example.org>\r\nDATA\r\n")
            .await
            .unwrap();
        let mut line = vec![b'x'; 998];
        line.extend_from_slice(b"\r\n");
        for _ in 0..=mail_kernel::MAX_MESSAGE_BYTES / 1000 {
            if writer.write_all(&line).await.is_err() {
                break;
            }
        }
        // Keep the write half alive, without EOF or a DATA terminator.
        writer
    });
    let mut transcript = String::new();
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        reader.read_to_string(&mut transcript),
    )
    .await
    .expect("oversized DATA must release its connection")
    .unwrap();
    drop(writing.await.unwrap());
    task.await.unwrap().unwrap();
    assert!(transcript.contains("552 5.3.4"), "{transcript}");
    assert!(!transcript.contains("250 2.0.0 Queued"));
    assert_eq!(service.deliver_pending(1).unwrap(), 0);
    assert!(db.account("a").unwrap().messages.is_empty());
}

#[tokio::test]
async fn smtp_line_limit_excludes_the_transparency_dot() {
    let (service, db) = service();
    let (mut client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::smtp_session(server, service.clone()));
    let raw = format!(".{}\r\n", "x".repeat(997));
    client.write_all(format!("EHLO client\r\nMAIL FROM:<>\r\nRCPT TO:<alice@example.org>\r\nDATA\r\n.{raw}.\r\nQUIT\r\n").as_bytes()).await.unwrap();
    let mut transcript = String::new();
    client.read_to_string(&mut transcript).await.unwrap();
    task.await.unwrap().unwrap();
    assert!(transcript.contains("250 2.0.0 Queued"), "{transcript}");
    assert_eq!(service.deliver_pending(1).unwrap(), 1);
    assert_eq!(db.blob("a", "e1").unwrap(), raw.as_bytes());
}

#[tokio::test]
async fn imap_hierarchy_uses_the_same_mailboxes_and_preserves_uids_on_move() {
    let (service, db) = service();
    db.execute(
        "a",
        0,
        vec![mail_kernel::Command::CreateMailbox {
            name: "Projects".into(),
        }],
    )
    .unwrap();
    let (mut client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    client.write_all(format!("a LOGIN alice@example.org {TOKEN}\r\nb CREATE Projects/Client\r\nc LIST \"\" \"Projects/%\"\r\nd SELECT Projects/Client\r\ne RENAME Projects/Client Client\r\nf SELECT Client\r\ng UNSUBSCRIBE Client\r\nh LSUB \"\" \"Client\"\r\ni LOGOUT\r\n").as_bytes()).await.unwrap();
    let mut result = String::new();
    client.read_to_string(&mut result).await.unwrap();
    task.await.unwrap().unwrap();
    for tag in ["b", "c", "d", "e", "f", "g", "h"] {
        assert!(result.contains(&format!("{tag} OK")), "{result}");
    }
    assert!(
        result.contains("* LIST (\\HasNoChildren) \"/\" \"Projects/Client\""),
        "{result}"
    );
    assert!(!result.contains("* LSUB"), "{result}");
    let account = db.account("a").unwrap();
    let folder = account
        .mailboxes
        .iter()
        .find(|m| m.name == "Client")
        .unwrap();
    assert!(folder.parent_id.is_none());
    assert!(!folder.is_subscribed);
    assert_eq!(folder.uid_validity, 3);
}

fn service() -> (Arc<MailService>, Arc<SqliteStore>) {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    (
        Arc::new(MailService {
            outbound: None,
            queue: db.clone(),
            store: db.clone(),
            identity: db.clone(),
            policy: Arc::new(OwnerPolicy),
        }),
        db,
    )
}

#[tokio::test]
async fn smtp_is_byte_preserving_rejects_relay_and_commits_before_ack() {
    let (service, db) = service();
    let (mut client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::smtp_session(server, service.clone()));
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

#[tokio::test]
async fn imap_reads_same_message_and_rechecks_revoked_credentials() {
    let (service, db) = service();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: shared\r\n\r\nbody\r\n",
    )
    .unwrap();
    let (client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    let mut line = String::new();
    client.read_line(&mut line).await.unwrap();
    client.get_mut().write_all(format!("a LOGIN alice@example.org {TOKEN}\r\nb SELECT INBOX\r\nc UID FETCH 1 (UID FLAGS BODY.PEEK[])\r\n").as_bytes()).await.unwrap();
    let mut transcript = String::new();
    loop {
        line.clear();
        client.read_line(&mut line).await.unwrap();
        transcript.push_str(&line);
        if line.starts_with("c OK") {
            break;
        }
    }
    assert!(transcript.contains("* 1 EXISTS"));
    assert!(transcript.contains("Subject: shared"));
    db.revoke("a").unwrap();
    client
        .get_mut()
        .write_all(b"d FETCH 1 (UID)\r\ne LOGOUT\r\n")
        .await
        .unwrap();
    let mut rest = String::new();
    client.read_to_string(&mut rest).await.unwrap();
    assert!(rest.contains("d NO"));
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn imap_refuses_credentials_on_plaintext_transport() {
    let (service, _) = service();
    let (mut client, server) = tokio::io::duplex(8192);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, false));
    client
        .write_all(format!("a LOGIN alice@example.org {TOKEN}\r\nb LOGOUT\r\n").as_bytes())
        .await
        .unwrap();
    let mut transcript = String::new();
    client.read_to_string(&mut transcript).await.unwrap();
    task.await.unwrap().unwrap();
    assert!(transcript.contains("a NO"));
}

#[tokio::test]
async fn imap_read_only_policy_can_fetch_without_write_authorization() {
    struct ReadOnly;
    impl mail_api::Policy for ReadOnly {
        fn authorize(
            &self,
            _: &mail_api::Principal,
            action: mail_api::Action,
            _: &mail_api::AccountInfo,
        ) -> Result<(), mail_kernel::Error> {
            if action == mail_api::Action::Read {
                Ok(())
            } else {
                Err(mail_kernel::Error::Forbidden)
            }
        }
    }
    let (_, db) = service();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: read only\r\n\r\nbody\r\n",
    )
    .unwrap();
    let before = db.account("a").unwrap();
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: Arc::new(ReadOnly),
    });
    let (mut client, server) = tokio::io::duplex(65536);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    client.write_all(format!(
        "a LOGIN alice@example.org {TOKEN}\r\nb SELECT INBOX\r\nc UID FETCH 1 (FLAGS BODY[])\r\nd UID STORE 1 +FLAGS (\\Seen)\r\ne LOGOUT\r\n"
    ).as_bytes()).await.unwrap();
    let mut transcript = String::new();
    client.read_to_string(&mut transcript).await.unwrap();
    task.await.unwrap().unwrap();
    assert!(transcript.contains("b OK [READ-ONLY]"), "{transcript}");
    assert!(transcript.contains("c OK"), "{transcript}");
    assert!(transcript.contains("Subject: read only"));
    assert!(transcript.contains("d NO"), "{transcript}");
    assert_eq!(db.account("a").unwrap(), before);
}
#[path = "wire/admin.rs"]
mod admin;
#[path = "wire/append.rs"]
mod append;
#[path = "wire/authenticate.rs"]
mod authenticate;
#[path = "wire/authenticate_review.rs"]
mod authenticate_review;
#[path = "wire/condstore.rs"]
mod condstore;
#[path = "wire/fetch.rs"]
mod fetch;
#[path = "wire/fetch_extensions.rs"]
mod fetch_extensions;
#[path = "wire/fetch_metadata.rs"]
mod fetch_metadata;
#[path = "wire/idle.rs"]
mod idle;
#[path = "wire/literal_limits.rs"]
mod literal_limits;
#[path = "wire/literal_review.rs"]
mod literal_review;
#[path = "wire/literals.rs"]
mod literals;
#[path = "wire/mailbox_encoding.rs"]
mod mailbox_encoding;
#[path = "wire/multiappend.rs"]
mod multiappend;
#[path = "wire/object_mailboxes.rs"]
mod object_mailboxes;
#[path = "wire/object_review.rs"]
mod object_review;
#[path = "wire/runtime.rs"]
mod runtime;
#[path = "wire/search.rs"]
mod search;
#[path = "wire/sequence.rs"]
mod sequence;
#[path = "wire/session.rs"]
mod session;
#[path = "wire/sort.rs"]
mod sort;
#[path = "wire/starttls.rs"]
mod starttls;
#[path = "wire/thread.rs"]
mod thread;
#[path = "wire/thread_review.rs"]
mod thread_review;
#[path = "wire/transfer.rs"]
mod transfer;
