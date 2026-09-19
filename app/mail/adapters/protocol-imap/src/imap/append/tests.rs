use super::*;
use mail_api::MetadataStore;
use mail_kernel::{Account, MAX_MESSAGE_BYTES};
use mail_service::OwnerPolicy;
use mail_sqlite_store::SqliteStore;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

#[test]
fn aggregate_limit_counts_body_and_zero_length_message_metadata_before_allocation() {
    let mut append = Append {
        mailbox: "inbox".into(),
        scope: "append:test".into(),
        revision: 0,
        remaining: usize::MAX,
        buffered: 0,
        bodies: vec![],
    };
    let literal = Literal::parse(&[format!("{{{MAX_MESSAGE_BYTES}}}")], false).unwrap();
    assert!(append.reserve(&literal).is_ok());
    assert_eq!(append.reserve(&literal), Err("NO [TOOBIG]"));
    let zero = Literal::parse(&["{0}".into()], false).unwrap();
    append.buffered = BUFFER_LIMIT;
    assert_eq!(append.reserve(&zero), Err("NO [TOOBIG]"));
}

#[tokio::test]
async fn whole_batch_deadline_does_not_restart_for_each_literal() {
    let token = "0123456789abcdef0123456789abcdef";
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        token,
    )
    .unwrap();
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: Arc::new(OwnerPolicy),
    });
    let session = Session {
        credential: token.into(),
        account_id: "a".into(),
        ..Session::default()
    };
    let (client, server) = tokio::io::duplex(4096);
    let mut client = BufReader::new(client);
    let started = tokio::time::Instant::now();
    let task = tokio::spawn(async move {
        let parts = ["b", "APPEND", "INBOX", "{1}"].map(String::from);
        read(&mut BufReader::new(server), service, &session, &parts)
            .await
            .err()
            .unwrap()
    });
    let mut response = String::new();
    assert!(client.read_line(&mut response).await.unwrap() > 0);
    assert!(response.starts_with('+'), "{response}");
    tokio::time::sleep(Duration::from_secs(151)).await;
    client.get_mut().write_all(b"a {1}\r\n").await.unwrap();
    response.clear();
    assert!(client.read_line(&mut response).await.unwrap() > 0);
    assert!(response.starts_with('+'), "{response}");
    let error = tokio::time::timeout_at(started + Duration::from_secs(305), task)
        .await
        .expect("batch deadline was restarted")
        .unwrap();
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert_eq!(error.to_string(), "IMAP literal deadline");
    assert!(db.account("a").unwrap().messages.is_empty());
    assert!(db.blob("a", "e1").is_err());
}

#[tokio::test]
async fn message_count_boundary_accepts_one_thousand_and_rolls_back_one_thousand_one() {
    let token = "0123456789abcdef0123456789abcdef";
    for count in [1000, 1001] {
        let db = Arc::new(SqliteStore::open(":memory:").unwrap());
        db.provision(
            Account::new("a", "t", "alice", "alice@example.org").unwrap(),
            token,
        )
        .unwrap();
        let service = Arc::new(MailService {
            outbound: None,
            queue: db.clone(),
            store: db.clone(),
            identity: db.clone(),
            policy: Arc::new(OwnerPolicy),
        });
        let (mut client, server) = tokio::io::duplex(65536);
        let task = tokio::spawn(crate::imap_session(server, service, true));
        let messages = " {0+}\r\n".repeat(count - 1);
        client.write_all(format!("a LOGIN alice@example.org {token}\r\nb APPEND INBOX {{0+}}\r\n{messages}\r\nz LOGOUT\r\n").as_bytes()).await.unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).await.unwrap();
        if count == 1000 {
            task.await.unwrap().unwrap();
            assert!(response.contains("b OK [APPENDUID 1 1:1000]"), "{response}");
            assert_eq!(db.account("a").unwrap().messages.len(), 1000);
        } else {
            assert_eq!(
                task.await.unwrap().unwrap_err().kind(),
                io::ErrorKind::InvalidData
            );
            assert!(response.contains("b NO [MESSAGELIMIT 1000]"), "{response}");
            let account = db.account("a").unwrap();
            assert!(account.messages.is_empty());
            assert_eq!(account.revision, 0);
            assert_eq!(account.mailboxes[0].uid_next, 1);
            assert!(db.blob("a", "e1").is_err());
        }
    }
}
