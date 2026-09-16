use base64::{Engine, engine::general_purpose::STANDARD};
use mail_service::MailService;
use serde_json::{Value, json};
use std::{
    io,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream};

pub const TOKEN: &str = "0123456789abcdef0123456789abcdef";
pub struct Context {
    pub service: Arc<MailService>,
    pub tasks: Mutex<Vec<tokio::task::JoinHandle<io::Result<()>>>>,
    pub outcomes: Mutex<Vec<Value>>,
}
tokio::task_local! { pub static CONTEXT: Arc<Context>; }
fn record(lines: &[String], assertion: &str, expected: &str, passed: bool) {
    CONTEXT.with(|c| {
        c.outcomes.lock().unwrap().push(
            json!({"assertion":assertion, "expected":expected, "actual":lines, "passed":passed}),
        )
    });
    assert!(passed, "{assertion} {expected}: {lines:?}");
}
pub trait AssertResult: Sized {
    fn assert_contains(self, value: &str) -> Self;
    fn assert_not_contains(self, value: &str) -> Self;
}
impl AssertResult for Vec<String> {
    fn assert_contains(self, value: &str) -> Self {
        record(
            &self,
            "contains",
            value,
            self.iter().any(|l| l.contains(value)),
        );
        self
    }
    fn assert_not_contains(self, value: &str) -> Self {
        record(
            &self,
            "not_contains",
            value,
            !self.iter().any(|l| l.contains(value)),
        );
        self
    }
}
pub enum ResponseType {
    Ok,
    Err,
    Multiline,
}
pub struct Pop3Connection {
    stream: BufReader<DuplexStream>,
    pending: Option<String>,
}
impl Pop3Connection {
    pub async fn connect() -> Self {
        let (client, server) = tokio::io::duplex(65536);
        CONTEXT.with(|c| {
            c.tasks
                .lock()
                .unwrap()
                .push(tokio::spawn(mail_protocol_imap::pop_session(
                    server,
                    c.service.clone(),
                    true,
                )))
        });
        let mut client = Self {
            stream: BufReader::new(client),
            pending: None,
        };
        client.assert_read(ResponseType::Ok).await;
        client
    }
    pub async fn send(&mut self, text: &str) {
        self.stream
            .get_mut()
            .write_all(format!("{text}\r\n").as_bytes())
            .await
            .unwrap();
        if text == "QUIT" {
            self.pending = Some(read(&mut self.stream).await);
        }
    }
    pub async fn authenticate(&mut self, user: &str, secret: &str) {
        self.send(&format!(
            "AUTH PLAIN {}",
            STANDARD.encode(format!("\0{user}\0{secret}"))
        ))
        .await;
        self.assert_read(ResponseType::Ok).await;
    }
    pub async fn assert_read(&mut self, kind: ResponseType) -> Vec<String> {
        let mut lines = vec![];
        loop {
            let line = match self.pending.take() {
                Some(line) => line,
                None => read(&mut self.stream).await,
            };
            let done =
                !matches!(kind, ResponseType::Multiline) || line == "." || line.starts_with("-ERR");
            lines.push(line);
            if done {
                break;
            }
        }
        let expected = match kind {
            ResponseType::Ok => "+OK",
            ResponseType::Err => "-ERR",
            ResponseType::Multiline => ".",
        };
        record(
            &lines,
            "status",
            expected,
            lines.last().unwrap().starts_with(expected),
        );
        lines
    }
}

pub struct SmtpConnection {
    stream: BufReader<DuplexStream>,
}
impl SmtpConnection {
    pub async fn connect() -> Self {
        let (client, server) = tokio::io::duplex(65536);
        CONTEXT.with(|c| {
            c.tasks
                .lock()
                .unwrap()
                .push(tokio::spawn(mail_protocol_imap::smtp_session(
                    server,
                    c.service.clone(),
                )))
        });
        let mut client = Self {
            stream: BufReader::new(client),
        };
        client.expect("220").await;
        client.send("EHLO upstream").await;
        client.expect("250").await;
        client
    }
    async fn send(&mut self, text: &str) {
        self.stream
            .get_mut()
            .write_all(format!("{text}\r\n").as_bytes())
            .await
            .unwrap();
    }
    async fn expect(&mut self, code: &str) {
        loop {
            let line = read(&mut self.stream).await;
            assert!(line.starts_with(code), "{line}");
            if line.as_bytes().get(3) != Some(&b'-') {
                break;
            }
        }
    }
    pub async fn ingest(&mut self, from: &str, recipients: &[&str], message: &str) {
        self.send(&format!("MAIL FROM:<{from}>")).await;
        self.expect("250").await;
        for recipient in recipients {
            self.send(&format!("RCPT TO:<{recipient}>")).await;
            self.expect("250").await;
        }
        self.send("DATA").await;
        self.expect("354").await;
        // The upstream fixture supplies SMTP transparency itself and appends
        // CRLF before its DATA terminator. Local upstream delivery also adds
        // X-Spam-Status: No (email/src/message/ingest.rs); stage that exact
        // 19-byte fixture header here so POP size predicates compare the same mail.
        self.stream
            .get_mut()
            .write_all(format!("X-Spam-Status: No\r\n{message}\r\n.\r\n").as_bytes())
            .await
            .unwrap();
        self.expect("250").await;
        let service = CONTEXT.with(|c| c.service.clone());
        assert_eq!(
            tokio::task::spawn_blocking(move || service.deliver_pending(1))
                .await
                .unwrap()
                .unwrap(),
            1
        );
        self.send("QUIT").await;
        self.expect("221").await;
    }
}
async fn read(stream: &mut BufReader<DuplexStream>) -> String {
    let mut line = String::new();
    let n = tokio::time::timeout(Duration::from_secs(5), stream.read_line(&mut line))
        .await
        .unwrap()
        .unwrap();
    assert!(n > 0, "upstream POP3 oracle connection closed");
    line.trim_end_matches(['\r', '\n']).into()
}
