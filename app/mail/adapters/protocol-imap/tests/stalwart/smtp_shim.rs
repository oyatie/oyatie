//! The upstream SMTP inbound suites drive `Session<DummyIo>`: a white-box
//! session whose `data` they read and set. This shim gives them the same
//! surface over our server on a duplex stream: `data` mirrors what the wire
//! reveals (EHLO domain, envelope, resets on STARTTLS), never more.
use super::registry_shim::Registry;
use mail_protocol_imap::SmtpParams;
use mail_service::MailService;
use std::{net::IpAddr, sync::Arc, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SessionAddress {
    pub address: String,
}
impl SessionAddress {
    pub fn new(address: String) -> Self {
        Self { address }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AccountCache {
    pub name: String,
}
#[derive(Clone, Debug)]
pub struct AccountInfo {
    pub account_id: u32,
    pub addresses: Vec<String>,
    pub account: Arc<AccountCache>,
}
#[derive(Clone, Debug, Default)]
pub struct SpfOutput;

pub struct Stream {
    pub tls: bool,
    pub rx_buf: Vec<u8>,
    pub tx_buf: Vec<u8>,
}

/// Upstream's per-session record. Fields the wire reveals are kept in step
/// with the server's replies; the counters are the suite's own to set.
pub struct SessionData {
    pub mail_from: Option<SessionAddress>,
    pub rcpt_to: Vec<SessionAddress>,
    pub helo_domain: String,
    pub spf_ehlo: Option<SpfOutput>,
    pub authenticated_as: Option<AccountInfo>,
    pub bytes_left: usize,
    pub rcpt_errors: u32,
    pub auth_errors: u32,
    pub remote_ip_str: String,
    pub remote_ip: IpAddr,
    pub valid_until: std::time::Instant,
}

pub struct Session {
    pub stream: Stream,
    pub data: SessionData,
    service: Arc<MailService>,
    registry: Arc<Registry>,
    link: Option<(DuplexStream, tokio::task::JoinHandle<()>)>,
}

impl Session {
    pub fn new(service: Arc<MailService>, registry: Arc<Registry>) -> Self {
        Self {
            stream: Stream {
                tls: false,
                rx_buf: vec![],
                tx_buf: vec![],
            },
            data: SessionData {
                mail_from: None,
                rcpt_to: vec![],
                helo_domain: String::new(),
                spf_ehlo: None,
                authenticated_as: None,
                bytes_left: 0,
                rcpt_errors: 0,
                auth_errors: 0,
                remote_ip_str: "127.0.0.1".into(),
                remote_ip: "127.0.0.1".parse().unwrap(),
                valid_until: std::time::Instant::now(),
            },
            service,
            registry,
            link: None,
        }
    }

    /// Upstream applies the registry's session parameters when the suite
    /// calls this; until then a session runs with the defaults.
    pub async fn eval_session_params(&mut self) {
        let params = self.registry.params(&self.data.remote_ip_str);
        self.start(params).await;
    }

    /// Start (or restart) the server session with `params` for this
    /// session's remote address and TLS state; the greeting is consumed, as
    /// upstream's `Session::test` emits none.
    async fn start(&mut self, params: SmtpParams) {
        let params = SmtpParams {
            peer: self
                .data
                .remote_ip_str
                .parse()
                .unwrap_or(self.data.remote_ip),
            ..params
        };
        let (client, server) = tokio::io::duplex(65536);
        let service = self.service.clone();
        let tls = self.stream.tls;
        let task = tokio::spawn(async move {
            let _ = if tls {
                mail_protocol_imap::smtp_tls_session_with(server, service, &params).await
            } else {
                mail_protocol_imap::smtp_starttls_session_with(
                    server,
                    service,
                    false,
                    |s| async move { Ok(s) },
                    &params,
                )
                .await
            };
        });
        self.link = Some((client, task));
        let _greeting = self.collect(Duration::from_secs(2)).await;
        self.stream.tx_buf.clear();
    }

    async fn link(&mut self) -> &mut DuplexStream {
        if self.link.is_none() {
            self.start(SmtpParams::default()).await;
        }
        &mut self.link.as_mut().unwrap().0
    }

    /// Read replies until the last line is a final one (`NNN `) or nothing
    /// arrives for `quiet`; returns whether the peer closed. The session
    /// must have been started.
    async fn collect(&mut self, quiet: Duration) -> bool {
        let mut closed = false;
        let mut buf = [0u8; 8192];
        loop {
            let Some((link, _)) = self.link.as_mut() else {
                return true;
            };
            match tokio::time::timeout(quiet, link.read(&mut buf)).await {
                Ok(Ok(0)) => {
                    closed = true;
                    break;
                }
                Ok(Ok(n)) => self.stream.tx_buf.extend_from_slice(&buf[..n]),
                Ok(Err(_)) => {
                    closed = true;
                    break;
                }
                Err(_) => break,
            }
            if self.final_line() {
                break;
            }
        }
        closed
    }

    fn final_line(&self) -> bool {
        let text = String::from_utf8_lossy(&self.stream.tx_buf);
        text.ends_with("\r\n")
            && text
                .trim_end()
                .rsplit("\r\n")
                .next()
                .is_some_and(|l| l.len() >= 4 && l.as_bytes()[3] == b' ')
    }

    /// Write `bytes` to the server; `Ok(false)` after a STARTTLS acceptance,
    /// `Err(())` when the server closed the connection.
    pub async fn ingest(&mut self, bytes: &[u8]) -> Result<bool, ()> {
        self.stream.tx_buf.clear();
        let complete = bytes.ends_with(b"\n");
        if self.link().await.write_all(bytes).await.is_err() {
            return Err(());
        }
        let quiet = if complete {
            Duration::from_secs(2)
        } else {
            Duration::from_millis(150)
        };
        let mut closed = self.collect(quiet).await;
        if !closed {
            // A final reply may be the last thing the server sends.
            closed = self.collect(Duration::from_millis(50)).await;
        }
        let text = String::from_utf8_lossy(&self.stream.tx_buf).to_string();
        self.mirror(bytes, &text);
        if closed {
            return Err(());
        }
        Ok(!(text.starts_with("220 2.0.0") && bytes.to_ascii_uppercase().starts_with(b"STARTTLS")))
    }

    /// Keep `data` in step with what the exchange established.
    fn mirror(&mut self, sent: &[u8], reply: &str) {
        let line = String::from_utf8_lossy(sent).trim().to_string();
        let upper = line.to_ascii_uppercase();
        if !reply.starts_with('2') {
            return;
        }
        if upper.starts_with("STARTTLS") {
            self.stream.tls = true;
            self.data.mail_from = None;
            self.data.rcpt_to.clear();
            self.data.helo_domain.clear();
            self.data.spf_ehlo = None;
            self.data.authenticated_as = None;
        } else if let Some(host) = upper
            .strip_prefix("EHLO ")
            .or_else(|| upper.strip_prefix("HELO "))
        {
            self.data.helo_domain = line[line.len() - host.len()..].to_string();
            self.data.mail_from = None;
            self.data.rcpt_to.clear();
        } else if upper.starts_with("MAIL FROM:") {
            self.data.mail_from = Some(SessionAddress::new(between(&line)));
            self.data.rcpt_to.clear();
        } else if upper.starts_with("RCPT TO:") {
            self.data.rcpt_to.push(SessionAddress::new(between(&line)));
        } else if upper == "RSET" {
            self.data.mail_from = None;
            self.data.rcpt_to.clear();
        }
    }

    pub fn write_rx(&mut self, data: &str) {
        self.stream.rx_buf.extend_from_slice(data.as_bytes());
    }

    /// Send what `write_rx` queued and read until the server closes or goes
    /// quiet, as upstream's connection loop would.
    pub async fn handle_conn(&mut self) -> bool {
        let pending = std::mem::take(&mut self.stream.rx_buf);
        self.stream.tx_buf.clear();
        let _ = self.link().await.write_all(&pending).await;
        // Read until the server closes (a limit) or falls silent.
        loop {
            let before = self.stream.tx_buf.len();
            if self.collect(Duration::from_secs(2)).await || self.stream.tx_buf.len() == before {
                break;
            }
        }
        true
    }

    pub fn response(&mut self) -> Vec<String> {
        let lines: Vec<String> = String::from_utf8_lossy(&self.stream.tx_buf)
            .split("\r\n")
            .filter(|l| !l.is_empty())
            .map(str::to_owned)
            .collect();
        self.stream.tx_buf.clear();
        lines
    }
}

fn between(line: &str) -> String {
    line.split_once('<')
        .and_then(|(_, rest)| rest.split_once('>'))
        .map(|(address, _)| address.to_owned())
        .unwrap_or_default()
}
