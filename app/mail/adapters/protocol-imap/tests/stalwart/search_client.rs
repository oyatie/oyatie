use mail_service::MailService;
use serde_json::{Value, json};
use std::{cell::RefCell, rc::Rc, sync::Arc, time::Duration};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream};

#[derive(Clone, Copy, Debug)]
pub enum ResponseType {
    Ok,
    Bad,
}
pub enum Type {
    Tagged,
}
pub struct TestServer {
    pub server: SearchStore,
}
pub struct SearchStore;
impl SearchStore {
    pub fn search_store(&self) -> &Self {
        self
    }
    pub fn is_mysql(&self) -> bool {
        false
    }
}
pub type Outcomes = Rc<RefCell<Vec<Value>>>;
pub struct ImapConnection {
    stream: BufReader<DuplexStream>,
    tag: usize,
    command: String,
    outcomes: Outcomes,
    task: tokio::task::JoinHandle<std::io::Result<()>>,
}
pub struct Lines {
    lines: Vec<String>,
    command: String,
    outcomes: Outcomes,
}

impl ImapConnection {
    pub async fn connect(service: Arc<MailService>, outcomes: Outcomes) -> Self {
        let (client, server) = tokio::io::duplex(65536);
        let task = tokio::spawn(mail_protocol_imap::imap_session(server, service, true));
        let mut connection = Self {
            stream: BufReader::new(client),
            tag: 0,
            command: String::new(),
            outcomes,
            task,
        };
        connection
            .send(&format!("LOGIN alice@example.org {}", super::TOKEN))
            .await;
        connection.assert_read(Type::Tagged, ResponseType::Ok).await;
        connection
    }
    pub async fn send(&mut self, command: &str) {
        self.tag += 1;
        self.command = command.to_owned();
        self.stream
            .get_mut()
            .write_all(format!("t{} {command}\r\n", self.tag).as_bytes())
            .await
            .unwrap();
    }
    pub async fn assert_read(&mut self, _: Type, expected: ResponseType) -> Lines {
        let tag = format!("t{} ", self.tag);
        let lines = tokio::time::timeout(Duration::from_secs(5), async {
            let mut lines = Vec::new();
            loop {
                let mut line = String::new();
                assert!(
                    self.stream.read_line(&mut line).await.unwrap() > 0,
                    "EOF after {}: {lines:?}",
                    self.command
                );
                let done = line.starts_with(&tag);
                lines.push(line.trim_end_matches(['\r', '\n']).to_owned());
                if done {
                    return lines;
                }
            }
        })
        .await
        .expect("upstream IMAP response deadline");
        let expected = format!(
            "{tag}{}",
            match expected {
                ResponseType::Ok => "OK",
                ResponseType::Bad => "BAD",
            }
        );
        let result = Lines {
            lines,
            command: self.command.clone(),
            outcomes: self.outcomes.clone(),
        };
        result.record(
            "status",
            &expected,
            result.lines.last().unwrap().starts_with(&expected),
        );
        result
    }
    pub async fn close(mut self) {
        self.send("LOGOUT").await;
        self.assert_read(Type::Tagged, ResponseType::Ok).await;
        self.task.await.unwrap().unwrap();
    }
}
impl Lines {
    fn record(&self, assertion: &str, expected: &str, passed: bool) {
        self.outcomes.borrow_mut().push(json!({
            "command": self.command, "assertion": assertion,
            "expected": expected, "actual": self.lines, "passed": passed,
        }));
    }
}
// The original predicates are preserved exactly. Recording instead of an
// immediate panic allows every unchanged upstream assertion to run; the test
// fails after all results have been emitted if any assertion failed.
pub trait AssertResult: Sized {
    fn assert_contains(self, expected: &str) -> Self;
    fn assert_equals(self, expected: &str) -> Self;
}
impl AssertResult for Lines {
    fn assert_contains(self, expected: &str) -> Self {
        self.record(
            "contains",
            expected,
            self.lines.iter().any(|line| line.contains(expected)),
        );
        self
    }
    fn assert_equals(self, expected: &str) -> Self {
        self.record(
            "equals",
            expected,
            self.lines.iter().any(|line| line == expected),
        );
        self
    }
}
