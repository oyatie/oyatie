use mail_api::Store;
use mail_kernel::Account;
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite::SqliteStore;
use std::{io, sync::Arc, time::Duration};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream};

pub const TOKEN: &str = "0123456789abcdef0123456789abcdef";
pub type Task = tokio::task::JoinHandle<io::Result<()>>;

pub fn service() -> (Arc<MailService>, Arc<SqliteStore>) {
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

pub fn deliver(db: &SqliteStore, body: &[u8]) {
    db.deliver(&["alice@example.org".into()], body).unwrap();
}

pub struct Client(pub BufReader<DuplexStream>);
impl Client {
    pub async fn connect(service: Arc<MailService>, encrypted: bool) -> (Self, Task) {
        let (client, server) = tokio::io::duplex(65536);
        let task = tokio::spawn(mail_protocol::pop_session(server, service, encrypted));
        let mut client = Self(BufReader::new(client));
        assert!(client.read().await.starts_with(b"+OK"));
        (client, task)
    }
    pub async fn read(&mut self) -> Vec<u8> {
        let mut line = vec![];
        let n = tokio::time::timeout(Duration::from_secs(5), self.0.read_until(b'\n', &mut line))
            .await
            .expect("POP3 response deadline")
            .unwrap();
        assert!(n > 0, "POP3 connection unexpectedly closed");
        line
    }
    pub async fn command(&mut self, command: &str) -> Vec<u8> {
        self.0
            .get_mut()
            .write_all(format!("{command}\r\n").as_bytes())
            .await
            .unwrap();
        self.read().await
    }
    pub async fn ok(&mut self, command: &str) -> Vec<u8> {
        let line = self.command(command).await;
        assert!(
            line.starts_with(b"+OK"),
            "{}",
            String::from_utf8_lossy(&line)
        );
        line
    }
    pub async fn error(&mut self, command: &str) {
        let line = self.command(command).await;
        assert!(
            line.starts_with(b"-ERR"),
            "{}",
            String::from_utf8_lossy(&line)
        );
    }
    pub async fn multiline(&mut self, command: &str) -> Vec<u8> {
        let mut response = self.ok(command).await;
        loop {
            let line = self.read().await;
            response.extend_from_slice(&line);
            if line == b".\r\n" {
                return response;
            }
        }
    }
    pub async fn login(&mut self) {
        self.ok("USER alice@example.org").await;
        self.ok(&format!("PASS {TOKEN}")).await;
    }
    pub async fn close(mut self, task: Task) {
        self.ok("QUIT").await;
        task.await.unwrap().unwrap();
    }
}
