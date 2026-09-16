use mail_service::MailService;
use serde_json::{Value, json};
use std::{cell::RefCell, rc::Rc, sync::Arc, time::Duration};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream};
#[path = "condstore_result.rs"]
mod result;
pub use result::AssertResult;
#[derive(Clone, Copy, Debug)]
pub enum ResponseType {
    Ok,
    No,
    Bad,
}
pub enum Type {
    Tagged,
    Continuation,
}
pub type Outcomes = Rc<RefCell<Vec<Value>>>;
pub struct ImapConnection {
    stream: BufReader<DuplexStream>,
    tag: usize,
    command: String,
    outcomes: Outcomes,
    task: tokio::task::JoinHandle<std::io::Result<()>>,
}
thread_local! {
    static CONTEXT: RefCell<Option<(String, Outcomes)>> = const { RefCell::new(None) };
}
fn record(lines: &[String], kind: &str, expected: &str, passed: bool) {
    CONTEXT.with(|context| {
        let context = context.borrow(); let (command, outcomes) = context.as_ref().unwrap();
        let value=json!({"command":command,"assertion":kind,"expected":expected,"actual":lines,"passed":passed});
        println!("{value}"); outcomes.borrow_mut().push(value);
    });
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
    pub async fn send_raw(&mut self, value: &str) {
        self.stream
            .get_mut()
            .write_all(value.as_bytes())
            .await
            .unwrap();
    }
    pub async fn send_untagged(&mut self, command: &str) {
        self.stream
            .get_mut()
            .write_all(format!("{command}\r\n").as_bytes())
            .await
            .unwrap();
    }
    pub async fn assert_read(&mut self, kind: Type, expected: ResponseType) -> Vec<String> {
        let tag = format!("t{} ", self.tag);
        let lines = tokio::time::timeout(Duration::from_secs(5), async {
            let mut lines = vec![];
            loop {
                let mut line = String::new();
                assert!(
                    self.stream.read_line(&mut line).await.unwrap() > 0,
                    "EOF: {} {lines:?}",
                    self.command
                );
                let done = line.starts_with(&tag)
                    || (matches!(kind, Type::Continuation) && line.starts_with('+'));
                lines.push(line.trim_end_matches(['\r', '\n']).to_owned());
                if done {
                    return lines;
                }
            }
        })
        .await
        .expect("upstream CONDSTORE response deadline");
        let expected = if matches!(kind, Type::Continuation) {
            "+".to_owned()
        } else {
            format!(
                "{tag}{}",
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
