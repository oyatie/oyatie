use super::*;
use mail_kernel::Account;
use mail_service::OwnerPolicy;
use mail_sqlite::SqliteStore;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{DuplexStream, ReadBuf};

struct ObservedWrite {
    stream: DuplexStream,
    first: Option<tokio::sync::oneshot::Sender<u8>>,
}

impl AsyncRead for ObservedWrite {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buffer)
    }
}

impl AsyncWrite for ObservedWrite {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        let result = Pin::new(&mut self.stream).poll_write(cx, bytes);
        if let Poll::Ready(Ok(count)) = &result
            && *count > 0
            && let Some(first) = self.first.take()
        {
            let _ = first.send(bytes[0]);
        }
        result
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

fn session() -> (Arc<MailService>, Session) {
    let token = "0123456789abcdef0123456789abcdef";
    let store = Arc::new(SqliteStore::open(":memory:").unwrap());
    store
        .provision(
            Account::new("a", "t", "alice", "alice@example.org").unwrap(),
            token,
        )
        .unwrap();
    (
        Arc::new(MailService {
            outbound: None,
            queue: store.clone(),
            identity: store.clone(),
            store,
            policy: Arc::new(OwnerPolicy),
        }),
        Session {
            credential: token.into(),
            account_id: "a".into(),
            ..Session::default()
        },
    )
}

#[tokio::test]
async fn idle_lifetime_closes_a_silent_authenticated_client() {
    let (service, mut session) = session();
    let (mut client, server) = tokio::io::duplex(1024);
    let task = tokio::spawn(async move {
        bounded(
            &mut BufReader::new(server),
            service,
            &mut session,
            "i",
            Duration::from_millis(200),
        )
        .await
    });
    let mut transcript = String::new();
    tokio::time::timeout(
        Duration::from_secs(5),
        client.read_to_string(&mut transcript),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(task.await.unwrap().unwrap());
    // The lifetime includes the initial blocking authorization refresh. If it
    // expires there, the connection is still aligned and only BYE is written.
    assert!(
        matches!(
            transcript.as_str(),
            "* BYE IDLE lifetime exceeded\r\n" | "+ Idling\r\n* BYE IDLE lifetime exceeded\r\n"
        ),
        "{transcript}"
    );
}

#[tokio::test]
async fn idle_lifetime_during_partial_output_closes_without_injecting_a_response() {
    let (service, mut session) = session();
    let (mut client, server) = tokio::io::duplex(1);
    let (first, written) = tokio::sync::oneshot::channel();
    let server = ObservedWrite {
        stream: server,
        first: Some(first),
    };
    let task = tokio::spawn(async move {
        bounded(
            &mut BufReader::new(server),
            service,
            &mut session,
            "i",
            // Leave startup headroom for the blocking pool under native linking
            // load. The successful-write witness below establishes the branch.
            Duration::from_secs(5),
        )
        .await
    });
    let first = tokio::time::timeout(Duration::from_secs(10), written)
        .await
        .unwrap()
        .unwrap();
    if first != b'+' {
        task.abort();
    }
    assert_eq!(first, b'+', "IDLE must enter partial greeting output");
    // Observe without reading: the single byte remains in the duplex buffer,
    // forcing the greeting write to stay pending until the lifetime expires.
    assert!(
        tokio::time::timeout(Duration::from_secs(10), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
    );
    let mut transcript = String::new();
    client.read_to_string(&mut transcript).await.unwrap();
    assert_eq!(transcript, "+");
}
