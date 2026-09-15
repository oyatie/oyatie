use super::*;
use mail_api::{Identity, Principal};
use mail_kernel::Error;
use std::sync::Mutex;

struct ObservedIdentity {
    inner: Arc<SqliteStore>,
    reads: Mutex<Vec<std::thread::ThreadId>>,
}
impl Identity for ObservedIdentity {
    fn authenticate(&self, token: &str) -> Result<Principal, Error> {
        self.reads.lock().unwrap().push(std::thread::current().id());
        self.inner.authenticate(token)
    }
}

pub(super) async fn ready(
    client: &mut BufReader<tokio::io::DuplexStream>,
    command: &str,
    tag: &str,
) {
    client
        .get_mut()
        .write_all(command.as_bytes())
        .await
        .unwrap();
    loop {
        let mut line = String::new();
        assert!(client.read_line(&mut line).await.unwrap() > 0);
        if line.starts_with(tag) {
            assert!(line.contains(" OK"), "{line}");
            break;
        }
    }
}

#[tokio::test]
async fn imap_storage_runs_off_the_socket_runtime_and_fetch_streams_with_backpressure() {
    let io_thread = std::thread::current().id();
    let (_, db) = service();
    let raw = format!("Subject: streamed\r\n\r\n{}\r\n", "x".repeat(1024 * 1024));
    for _ in 0..2 {
        db.deliver(&["alice@example.org".into()], raw.as_bytes())
            .unwrap();
    }
    let identity = Arc::new(ObservedIdentity {
        inner: db.clone(),
        reads: Mutex::new(vec![]),
    });
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db,
        identity: identity.clone(),
        policy: Arc::new(OwnerPolicy),
    });
    let (client, server) = tokio::io::duplex(4096);
    let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
    let mut client = BufReader::new(client);
    ready(
        &mut client,
        &format!("a LOGIN alice@example.org {TOKEN}\r\n"),
        "a ",
    )
    .await;
    ready(&mut client, "b SELECT INBOX\r\n", "b ").await;
    assert!(
        identity
            .reads
            .lock()
            .unwrap()
            .iter()
            .all(|thread| *thread != io_thread)
    );
    let before = identity.reads.lock().unwrap().len();
    client
        .get_mut()
        .write_all(b"c UID FETCH 1:* (BODY.PEEK[])\r\nd LOGOUT\r\n")
        .await
        .unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            if identity.reads.lock().unwrap().len() >= before + 2 {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert_eq!(
        identity.reads.lock().unwrap().len(),
        before + 2,
        "slow reader must prevent fetching the next body"
    );
    let mut transcript = String::new();
    client.read_to_string(&mut transcript).await.unwrap();
    assert_eq!(transcript.matches("Subject: streamed").count(), 2);
    assert!(transcript.contains("c OK") && transcript.contains("d OK"));
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn stalled_mail_readers_release_greeting_and_error_paths() {
    let (service, _) = service();
    let mut cases = Vec::new();
    for (smtp, command) in [
        (false, ""),
        (false, "broken\r\n"),
        (false, "a+ NOOP\r\n"),
        (true, ""),
        (true, "invalid\r\n"),
    ] {
        let service = service.clone();
        cases.push(tokio::spawn(async move {
            let (mut client, server) = tokio::io::duplex(if command.is_empty() { 1 } else { 64 });
            let session = tokio::spawn(async move {
                if smtp {
                    mail_protocol::smtp_session(server, service).await
                } else {
                    mail_protocol::imap_session(server, service, false).await
                }
            });
            if !command.is_empty() {
                client
                    .write_all(command.repeat(4).as_bytes())
                    .await
                    .unwrap();
            }
            let error = tokio::time::timeout(std::time::Duration::from_secs(65), session)
                .await
                .expect("stalled response must close the session")
                .unwrap()
                .unwrap_err();
            assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
            assert_eq!(error.to_string(), "mail response stalled");
            drop(client);
        }));
    }
    for case in cases {
        case.await.unwrap();
    }
}
