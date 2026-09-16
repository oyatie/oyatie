#[path = "facade_review/config.rs"]
mod config;
#[path = "facade_review/direct.rs"]
mod direct;
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

struct Fixture {
    root: tempfile::TempDir,
    cert: PathBuf,
    key: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let cert = root.path().join("cert.pem");
        let key = root.path().join("key.pem");
        let rcgen::CertifiedKey {
            cert: issued,
            signing_key,
        } = rcgen::generate_simple_self_signed(vec!["localhost".into(), "127.0.0.1".into()])
            .unwrap();
        std::fs::write(&cert, issued.pem()).unwrap();
        std::fs::write(&key, signing_key.serialize_pem()).unwrap();
        Self { root, cert, key }
    }
    fn command(&self) -> Command {
        let binary = option_env!("MAIL_APP_BINARY")
            .or(option_env!("CARGO_BIN_EXE_mail-app"))
            .unwrap();
        let mut command = Command::new(binary);
        command
            .arg("serve")
            .arg(self.root.path().join("mail.db"))
            .arg(&self.cert)
            .arg(&self.key)
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        for name in [
            "SMTP",
            "IMAP",
            "IMAP_STARTTLS",
            "SUBMISSION",
            "SUBMISSION_STARTTLS",
            "POP",
            "POP_STARTTLS",
            "HTTP",
        ] {
            command.env(format!("MAIL_{name}_LISTEN"), "127.0.0.1:0");
        }
        for name in [
            "PUBLIC_URL",
            "RELAY_HOST",
            "RELAY_PORT",
            "RELAY_HELO",
            "RELAY_CA",
            "RELAY_USERNAME",
            "RELAY_PASSWORD",
            "RELAY_STARTTLS",
            "MX_DNS_SERVERS",
            "MX_PORT",
            "MX_HELO",
            "MX_CA",
            "MX_REQUIRE_TLS",
        ] {
            command.env_remove(format!("MAIL_{name}"));
        }
        command
    }
    fn start(&self) -> (Server, Vec<String>) {
        launch(self.command())
    }
}
fn launch(mut command: Command) -> (Server, Vec<String>) {
    let mut server = Server(command.spawn().unwrap());
    let stderr = server.0.stderr.take().unwrap();
    let (send, receive) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        BufReader::new(stderr).read_line(&mut line).unwrap();
        let _ = send.send(line);
    });
    let line = receive.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(line.starts_with("mail-app: SMTP "), "{line}");
    let endpoints = line.trim().split("; ").map(str::to_owned).collect();
    (server, endpoints)
}
struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn socket(address: &str) -> TcpStream {
    let stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    stream
}
fn line(stream: &mut BufReader<TcpStream>) -> String {
    let mut line = String::new();
    stream.read_line(&mut line).unwrap();
    line
}
fn endpoint<'a>(endpoints: &'a [String], prefix: &str) -> &'a str {
    endpoints
        .iter()
        .find_map(|e| e.strip_prefix(prefix))
        .unwrap()
}
#[test]
fn pop_tls_admission_shares_mail_capacity_and_graceful_shutdown_drains_sessions() {
    let fixture = Fixture::new();
    let (mut server, endpoints) = fixture.start();
    let smtp = endpoint(&endpoints, "mail-app: SMTP ");
    let pop = endpoint(&endpoints, "POP STARTTLS ");
    let pops = endpoint(&endpoints, "POPS ");
    let mut peers = Vec::new();
    for _ in 0..64 {
        let mut peer = BufReader::new(socket(smtp));
        assert!(line(&mut peer).starts_with("220"));
        peers.push((peer, false));
    }
    for _ in 0..63 {
        let mut peer = BufReader::new(socket(pop));
        assert!(line(&mut peer).starts_with("+OK"));
        peers.push((peer, true));
    }
    let mut stalled_tls = socket(pops);
    stalled_tls
        .set_read_timeout(Some(Duration::from_millis(200)))
        .unwrap();
    let mut byte = [0];
    assert!(
        stalled_tls.read(&mut byte).is_err(),
        "POPS greeted before TLS"
    );
    let mut excess = socket(smtp);
    match excess.read(&mut byte) {
        Ok(0) => {}
        Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => {}
        result => panic!("TLS handshake did not consume shared admission: {result:?}"),
    }
    // HTTP has its own bounded connection budget, keeping administration live.
    let url = endpoint(&endpoints, "JMAP ");
    let cert = std::fs::read(&fixture.cert).unwrap();
    let client = reqwest::blocking::Client::builder()
        .add_root_certificate(reqwest::Certificate::from_pem(&cert).unwrap())
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    assert_eq!(
        client
            .get(format!("{url}/.well-known/jmap"))
            .send()
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED
    );
    drop(client);
    drop(stalled_tls);
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let mut available = BufReader::new(socket(smtp));
        if available.read_line(&mut String::new()).is_ok_and(|n| n > 0) {
            available.get_mut().write_all(b"QUIT\r\n").unwrap();
            assert!(line(&mut available).starts_with("221"));
            break;
        }
        assert!(
            Instant::now() < deadline,
            "closed TLS session leaked admission"
        );
    }
    #[cfg(unix)]
    {
        assert!(
            Command::new("kill")
                .args(["-TERM", &server.0.id().to_string()])
                .status()
                .unwrap()
                .success()
        );
        std::thread::sleep(Duration::from_millis(100));
        assert!(
            server.0.try_wait().unwrap().is_none(),
            "active POP sessions were not drained"
        );
        for (mut peer, is_pop) in peers {
            peer.get_mut().write_all(b"QUIT\r\n").unwrap();
            assert!(line(&mut peer).starts_with(if is_pop { "+OK" } else { "221" }));
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = server.0.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            assert!(Instant::now() < deadline, "shutdown did not complete");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
