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
    Bye,
}
pub enum Type {
    Tagged,
    Untagged,
}
type Outcomes = Rc<RefCell<Vec<Value>>>;
type Context = (Arc<MailService>, Outcomes);
thread_local! {
    static FIXTURE: RefCell<Option<Context>> = const { RefCell::new(None) };
    static CONTEXT: RefCell<Option<(String, Outcomes)>> = const { RefCell::new(None) };
}
pub fn initialize(service: Arc<MailService>) -> Outcomes {
    let outcomes = Rc::new(RefCell::new(vec![]));
    FIXTURE.with(|fixture| *fixture.borrow_mut() = Some((service, outcomes.clone())));
    outcomes
}
fn record(lines: &[String], kind: &str, expected: &str, passed: bool) {
    CONTEXT.with(|context| {
        let context = context.borrow(); let (command, outcomes) = context.as_ref().unwrap();
        let value=json!({"command":command,"assertion":kind,"expected":expected,"actual":lines,"passed":passed});
        println!("{value}"); outcomes.borrow_mut().push(value);
    });
}
pub struct ImapConnection {
    stream: BufReader<DuplexStream>,
    prefix: String,
    tag: usize,
    command: String,
    outcomes: Outcomes,
    task: tokio::task::JoinHandle<std::io::Result<()>>,
}
impl ImapConnection {
    pub async fn connect(prefix: &[u8]) -> Self {
        let (service, outcomes) =
            FIXTURE.with(|fixture| fixture.borrow().as_ref().unwrap().clone());
        let (client, server) = tokio::io::duplex(65536);
        let task = tokio::spawn(mail_protocol::imap_session(server, service, true));
        Self {
            stream: BufReader::new(client),
            prefix: std::str::from_utf8(prefix).unwrap().trim().to_owned(),
            tag: 0,
            command: "greeting".into(),
            outcomes,
            task,
        }
    }
    pub async fn send(&mut self, command: &str) {
        self.tag += 1;
        self.command = command.to_owned();
        self.stream
            .get_mut()
            .write_all(format!("{}{} {command}\r\n", self.prefix, self.tag).as_bytes())
            .await
            .unwrap();
    }
    pub async fn send_raw(&mut self, bytes: &[u8]) {
        self.stream.get_mut().write_all(bytes).await.unwrap();
    }
    pub async fn read_prefix(&mut self, prefix: &str) -> String {
        tokio::time::timeout(Duration::from_secs(5), async {
            let mut result = String::new();
            loop {
                let mut line = String::new();
                assert!(
                    self.stream.read_line(&mut line).await.unwrap() > 0,
                    "EOF waiting for {prefix}: {result}"
                );
                let done = line.starts_with(prefix);
                result.push_str(&line);
                if done {
                    return result;
                }
            }
        })
        .await
        .expect("UIDONLY asynchronous response deadline")
    }
    pub async fn authenticate(&mut self, name: &str, secret: &str) {
        self.send(&format!("LOGIN {name} {secret}")).await;
        self.assert_read(Type::Tagged, ResponseType::Ok).await;
    }
    pub async fn assert_read(&mut self, kind: Type, expected: ResponseType) -> Vec<String> {
        let tag = format!("{}{} ", self.prefix, self.tag);
        let expected = format!(
            "{}{}",
            if matches!(kind, Type::Untagged) {
                "* "
            } else {
                &tag
            },
            match expected {
                ResponseType::Ok => "OK",
                ResponseType::No => "NO",
                ResponseType::Bad => "BAD",
                ResponseType::Bye => "BYE",
            }
        );
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
                    || (matches!(kind, Type::Untagged) && line.starts_with("* "));
                lines.push(line.trim_end_matches(['\r', '\n']).to_owned());
                if done {
                    return lines;
                }
            }
        })
        .await
        .expect("UIDONLY oracle response deadline");
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
