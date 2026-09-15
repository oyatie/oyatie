#![forbid(unsafe_code)]
#[path = "wire/jmap_submission.rs"]
mod jmap_submission;
#[path = "wire/mx_config.rs"]
mod mx_config;
#[path = "wire/outbound.rs"]
mod outbound;
#[path = "wire/pop.rs"]
mod pop;
#[path = "wire/starttls.rs"]
mod starttls;
#[path = "wire/submission.rs"]
mod submission;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned, pki_types::ServerName};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    process::{Child, Command, Stdio},
    sync::Arc,
    time::Duration,
};

struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn smtp_reply<S: Read>(stream: &mut BufReader<S>, expected: &str) {
    loop {
        let mut line = String::new();
        assert!(stream.read_line(&mut line).unwrap() > 0, "SMTP EOF");
        assert!(line.starts_with(expected), "SMTP response: {line}");
        if line.as_bytes().get(3) == Some(&b' ') {
            break;
        }
    }
}

fn bounded_output(mut child: Child) -> std::io::Result<std::process::Output> {
    let deadline = std::time::Instant::now() + Duration::from_secs(180);
    while child.try_wait()?.is_none() {
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "mail CLI did not exit within 180 seconds",
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    child.wait_with_output()
}

#[test]
fn standalone_smtp_to_imaps_and_https_preserves_the_message() {
    let binary = option_env!("MAIL_APP_BINARY")
        .or(option_env!("CARGO_BIN_EXE_mail-app"))
        .expect("the build must provide the mail-app executable");
    let root = tempfile::tempdir().unwrap();
    let database = root.path().join("mail.sqlite");
    let cert_path = root.path().join("certificate.pem");
    let key_path = root.path().join("key.pem");
    let rcgen::CertifiedKey { cert, signing_key } =
        rcgen::generate_simple_self_signed(vec!["localhost".into(), "127.0.0.1".into()]).unwrap();
    std::fs::write(&cert_path, cert.pem()).unwrap();
    std::fs::write(&key_path, signing_key.serialize_pem()).unwrap();
    let relay = outbound::RelayServer::new(cert.der().to_vec().into(), signing_key.serialize_der());
    let token = "local-test-0123456789abcdef0123456789abcdef";
    let provision = Command::new(binary)
        .arg("provision")
        .arg(&database)
        .args(["tenant", "alice", "alice", "alice@example.org"])
        .env("MAIL_TOKEN", token)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(bounded_output)
        .unwrap();
    assert!(provision.status.success(), "provision failed");
    mx_config::verify(
        binary,
        &database,
        &cert_path,
        &key_path,
        cert.der().to_vec(),
        token,
    );
    let missing = Command::new(binary)
        .arg("serve")
        .arg(&database)
        .arg(root.path().join("missing.pem"))
        .arg(&key_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(bounded_output)
        .unwrap();
    assert!(!missing.status.success(), "TLS material is mandatory");
    let mut server = Server(
        Command::new(binary)
            .arg("serve")
            .arg(&database)
            .arg(&cert_path)
            .arg(&key_path)
            .env("MAIL_SMTP_LISTEN", "127.0.0.1:0")
            .env("MAIL_SUBMISSION_LISTEN", "127.0.0.1:0")
            .env("MAIL_SUBMISSION_STARTTLS_LISTEN", "127.0.0.1:0")
            .env("MAIL_IMAP_LISTEN", "127.0.0.1:0")
            .env("MAIL_IMAP_STARTTLS_LISTEN", "127.0.0.1:0")
            .env("MAIL_HTTP_LISTEN", "127.0.0.1:0")
            .env("MAIL_POP_LISTEN", "127.0.0.1:0")
            .env("MAIL_POP_STARTTLS_LISTEN", "127.0.0.1:0")
            .env("MAIL_RELAY_HOST", "127.0.0.1")
            .env("MAIL_RELAY_PORT", relay.address.port().to_string())
            .env("MAIL_RELAY_HELO", "mail.example.org")
            .env("MAIL_RELAY_CA", &cert_path)
            .env("MAIL_RELAY_STARTTLS", "false")
            .env_remove("MAIL_RELAY_USERNAME")
            .env_remove("MAIL_RELAY_PASSWORD")
            .env_remove("MAIL_PUBLIC_URL")
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let stderr = server.0.stderr.take().unwrap();
    let (ready, startup) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        let result = BufReader::new(stderr).read_line(&mut line).map(|_| line);
        let _ = ready.send(result);
    });
    let startup = startup
        .recv_timeout(Duration::from_secs(180))
        .expect("mail listener readiness timed out")
        .unwrap();
    assert!(
        startup.starts_with("mail-app: SMTP"),
        "server startup failed: {startup}"
    );
    let endpoints: Vec<_> = startup.trim().split("; ").collect();
    let smtp_address = endpoints[0].strip_prefix("mail-app: SMTP ").unwrap();
    let imap_address = endpoints[1].strip_prefix("IMAPS ").unwrap();
    let public_url = endpoints[2].strip_prefix("JMAP ").unwrap();
    let submission_address = endpoints[3].strip_prefix("SUBMISSIONS ").unwrap();
    assert_ne!(
        smtp_address, "127.0.0.1:2525",
        "listen configuration must be applied"
    );

    let raw = b"From: sender@example.net\r\nTo: alice@example.org\r\nSubject: Rust wire test\r\n\r\n.dot\r\n";
    let tcp = TcpStream::connect(smtp_address).unwrap();
    tcp.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    tcp.set_write_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut smtp = BufReader::new(tcp);
    smtp_reply(&mut smtp, "220");
    for (command, reply) in [
        ("EHLO client.example.net\r\n", "250"),
        ("MAIL FROM:<sender@example.net>\r\n", "250"),
        ("RCPT TO:<nobody@external.org>\r\n", "550"),
        ("RCPT TO:<alice@example.org>\r\n", "250"),
        ("DATA\r\n", "354"),
    ] {
        smtp.get_mut().write_all(command.as_bytes()).unwrap();
        smtp_reply(&mut smtp, reply);
    }
    smtp.get_mut()
        .write_all(
            String::from_utf8_lossy(raw)
                .replace("\r\n.", "\r\n..")
                .as_bytes(),
        )
        .unwrap();
    smtp.get_mut().write_all(b".\r\n").unwrap();
    smtp_reply(&mut smtp, "250");
    smtp.get_mut().write_all(b"QUIT\r\n").unwrap();
    smtp_reply(&mut smtp, "221");

    let client = reqwest::blocking::Client::builder()
        .add_root_certificate(reqwest::Certificate::from_pem(cert.pem().as_bytes()).unwrap())
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let response: Value = client.post(format!("{public_url}/jmap")).bearer_auth(token)
            .json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/query",{"accountId":"alice"},"queue"]]}))
            .send().unwrap().error_for_status().unwrap().json().unwrap();
        if response["methodResponses"][0][1]["ids"] == json!(["e1"]) {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "queued message was not delivered: {response}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }

    let mut roots = RootCertStore::empty();
    roots.add(cert.der().to_vec().into()).unwrap();
    let config = Arc::new(
        ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth(),
    );
    pop::verify(&endpoints, config.clone(), token, raw);
    starttls::verify(
        endpoints[4].strip_prefix("IMAP STARTTLS ").unwrap(),
        config.clone(),
        token,
    );
    let connection =
        ClientConnection::new(config.clone(), ServerName::try_from("localhost").unwrap()).unwrap();
    starttls::smtp(smtp_address, config.clone(), token, false);
    starttls::smtp(
        endpoints[5].strip_prefix("SUBMISSION STARTTLS ").unwrap(),
        config.clone(),
        token,
        true,
    );
    let tcp = TcpStream::connect(imap_address).unwrap();
    tcp.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    tcp.set_write_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut imap = BufReader::new(StreamOwned::new(connection, tcp));
    let mut greeting = String::new();
    imap.read_line(&mut greeting).unwrap();
    assert!(greeting.starts_with("* OK"));
    imap.get_mut().write_all(format!("a LOGIN alice@example.org {token}\r\nb SELECT INBOX\r\nc UID FETCH 1 (BODY.PEEK[])\r\n").as_bytes()).unwrap();
    imap.get_mut().flush().unwrap();
    let mut content = vec![];
    loop {
        let mut line = String::new();
        assert!(imap.read_line(&mut line).unwrap() > 0, "IMAP EOF");
        if line.ends_with("}\r\n") {
            let size = line
                .rsplit_once('{')
                .unwrap()
                .1
                .trim_end_matches("}\r\n")
                .parse::<usize>()
                .unwrap();
            assert_eq!(size, raw.len());
            content.resize(size, 0);
            imap.read_exact(&mut content).unwrap();
        }
        if line.starts_with("c ") {
            assert!(line.starts_with("c OK"));
            break;
        }
    }
    assert_eq!(content, raw);
    // A peer that sends no ClientHello must not hold up unrelated HTTPS clients.
    let stalled_tls = TcpStream::connect(public_url.strip_prefix("https://").unwrap()).unwrap();
    std::thread::sleep(Duration::from_millis(100));
    let response: Value = client.post(format!("{public_url}/jmap")).bearer_auth(token).json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/get",{"accountId":"alice","ids":["e1"]},"read"]]})).send().unwrap().error_for_status().unwrap().json().unwrap();
    assert_eq!(
        response["methodResponses"][0][1]["list"][0]["subject"],
        "Rust wire test"
    );
    drop(stalled_tls);
    submission::verify(submission_address, config, token);
    jmap_submission::verify(&client, public_url, token);
    relay.verify(&client, public_url, token);
    drop(imap);
    drop(smtp);
    drop(client);
    #[cfg(unix)]
    {
        assert!(
            Command::new("kill")
                .args(["-TERM", &server.0.id().to_string()])
                .status()
                .unwrap()
                .success()
        );
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = server.0.try_wait().unwrap() {
                assert!(
                    status.success(),
                    "SIGTERM must drain and exit successfully: {status}"
                );
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "SIGTERM did not finish draining"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}
