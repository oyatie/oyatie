use mail_service::MailService;
use serde_json::{Value, json};
use std::{cell::RefCell, rc::Rc, sync::Arc, time::Duration};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream};
#[path = "core_result.rs"]
mod result;
pub use result::AssertResult;
#[derive(Clone, Copy, Debug)]
pub enum ResponseType {
    Ok,
    No,
    Bad,
}
#[derive(Clone, Copy, Debug)]
pub enum Type {
    Tagged,
    Untagged,
    Continuation,
    Status,
}
pub type Outcomes = Rc<RefCell<Vec<Value>>>;
pub struct ImapConnection {
    stream: BufReader<DuplexStream>,
    tag: &'static str,
    command: String,
    last_raw: Vec<u8>,
    outcomes: Outcomes,
    task: tokio::task::JoinHandle<std::io::Result<()>>,
}
thread_local! {
    static CONTEXT: RefCell<Option<(String, Outcomes)>> = const { RefCell::new(None) };
}
pub fn record(lines: &[String], kind: &str, expected: &str, passed: bool) {
    CONTEXT.with(|context| {
        let context = context.borrow();
        let (command, outcomes) = context.as_ref().unwrap();
        let value = json!({"command":command,"assertion":kind,"expected":expected,"actual":lines,"passed":passed});
        println!("{value}");
        outcomes.borrow_mut().push(value);
    });
}
impl ImapConnection {
    /// Opens a duplex session with upstream's constant per-connection tag;
    /// `login` authenticates as `(address, token)`.
    pub async fn connect(
        service: Arc<MailService>,
        outcomes: Outcomes,
        login: Option<(&str, &str)>,
        tag: &'static str,
    ) -> Self {
        let (client, server) = tokio::io::duplex(65536);
        let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
        let mut connection = Self {
            stream: BufReader::new(client),
            tag,
            command: "greeting".into(),
            last_raw: Vec::new(),
            outcomes,
            task,
        };
        connection
            .assert_read(Type::Untagged, ResponseType::Ok)
            .await;
        if let Some((address, token)) = login {
            connection.send(&format!("LOGIN {address} {token}")).await;
            connection.assert_read(Type::Tagged, ResponseType::Ok).await;
        }
        connection
    }
    pub async fn send(&mut self, command: &str) {
        self.command = command.to_owned();
        self.send_raw(&format!("{}{command}\r\n", self.tag)).await;
    }
    pub async fn send_raw(&mut self, value: &str) {
        self.stream
            .get_mut()
            .write_all(value.as_bytes())
            .await
            .unwrap();
    }
    pub async fn send_untagged(&mut self, command: &str) {
        self.send_raw(&format!("{command}\r\n")).await;
    }
    pub fn assert_last_contains_bytes(&self, pattern: &[u8]) -> &Self {
        let found = self.last_raw.windows(pattern.len()).any(|w| w == pattern);
        CONTEXT.with(|context| {
            *context.borrow_mut() = Some((self.command.clone(), self.outcomes.clone()))
        });
        record(&[], "contains_bytes", &format!("{pattern:02x?}"), found);
        self
    }
    pub async fn assert_read(&mut self, kind: Type, expected: ResponseType) -> Vec<String> {
        let prefix = match kind {
            Type::Tagged => self.tag,
            Type::Untagged | Type::Status => "* ",
            Type::Continuation => "+",
        };
        self.last_raw.clear();
        let lines = tokio::time::timeout(Duration::from_secs(5), async {
            let mut lines = vec![];
            loop {
                let mut raw = Vec::new();
                assert!(
                    self.stream.read_until(b'\n', &mut raw).await.unwrap() > 0,
                    "EOF: {} {lines:?}",
                    self.command
                );
                self.last_raw.extend_from_slice(&raw);
                let line = String::from_utf8_lossy(&raw)
                    .trim_end_matches(['\r', '\n'])
                    .to_owned();
                let done = line.starts_with(prefix);
                lines.push(line);
                if done {
                    return lines;
                }
            }
        })
        .await
        .unwrap_or_else(|_| panic!("response deadline: {} ({kind:?})", self.command));
        let expected = if matches!(kind, Type::Continuation | Type::Status) {
            prefix.to_owned()
        } else {
            format!(
                "{prefix}{}",
                match expected {
                    ResponseType::Ok => "OK",
                    ResponseType::No => "NO",
                    ResponseType::Bad => "BAD",
                }
            )
        };
        CONTEXT.with(|context| {
            *context.borrow_mut() = Some((self.command.clone(), self.outcomes.clone()))
        });
        record(
            &lines,
            "status",
            &expected,
            lines.last().unwrap().starts_with(&expected),
        );
        lines
    }
    pub async fn close(mut self) {
        self.send("LOGOUT").await;
        self.assert_read(Type::Tagged, ResponseType::Ok).await;
        self.task.await.unwrap().unwrap();
    }
}
