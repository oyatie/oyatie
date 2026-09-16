use super::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use std::path::Path;

pub(super) fn verify(binary: &str, db: &Path, cert: &Path, key: &Path, der: Vec<u8>, token: &str) {
    for (settings, expected) in [
        (
            vec![("MAIL_MX_REQUIRE_TLS", "true")],
            Some("MAIL_MX_DNS_SERVERS"),
        ),
        (
            vec![
                ("MAIL_MX_DNS_SERVERS", "127.0.0.1:9"),
                ("MAIL_RELAY_HOST", "localhost"),
            ],
            Some("mutually exclusive"),
        ),
        (
            vec![("MAIL_MX_DNS_SERVERS", "invalid")],
            Some("invalid socket address"),
        ),
        (
            vec![
                ("MAIL_MX_DNS_SERVERS", "127.0.0.1:9"),
                ("MAIL_MX_HELO", "mail.example.org"),
            ],
            None,
        ),
    ] {
        let mut command = Command::new(binary);
        command
            .arg("serve")
            .arg(db)
            .arg(cert)
            .arg(key)
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        for name in [
            "SMTP",
            "SUBMISSION",
            "SUBMISSION_STARTTLS",
            "IMAP",
            "IMAP_STARTTLS",
            "POP",
            "POP_STARTTLS",
            "HTTP",
        ] {
            command.env(format!("MAIL_{name}_LISTEN"), "127.0.0.1:0");
        }
        for name in [
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
            "PUBLIC_URL",
        ] {
            command.env_remove(format!("MAIL_{name}"));
        }
        command.env("MAIL_MX_CA", cert).envs(settings);
        let mut server = Server(command.spawn().unwrap());
        let stderr = server.0.stderr.take().unwrap();
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut line = String::new();
            let result = BufReader::new(stderr).read_line(&mut line).map(|_| line);
            let _ = send.send(result);
        });
        let line = receive
            .recv_timeout(Duration::from_secs(10))
            .unwrap()
            .unwrap();
        if let Some(expected) = expected {
            assert!(line.contains(expected), "expected {expected}: {line}");
            assert!(!line.starts_with("mail-app: SMTP"));
            continue;
        }
        assert!(line.starts_with("mail-app: SMTP"), "{line}");
        let address = line
            .split("; ")
            .nth(3)
            .unwrap()
            .strip_prefix("SUBMISSIONS ")
            .unwrap();
        let mut roots = RootCertStore::empty();
        roots.add(der.clone().into()).unwrap();
        let config = Arc::new(
            ClientConfig::builder()
                .with_root_certificates(roots)
                .with_no_client_auth(),
        );
        let connection =
            ClientConnection::new(config, ServerName::try_from("localhost").unwrap()).unwrap();
        let tcp = TcpStream::connect(address).unwrap();
        tcp.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        tcp.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut client = BufReader::new(StreamOwned::new(connection, tcp));
        smtp_reply(&mut client, "220");
        let auth = STANDARD.encode(format!("\0alice@example.org\0{token}"));
        for (command, expected) in [
            ("EHLO test.example".to_string(), "250"),
            (format!("AUTH PLAIN {auth}"), "235"),
            ("MAIL FROM:<alice@example.org>".into(), "250"),
            ("RCPT TO:<recipient@remote.example>".into(), "250"),
            ("RSET".into(), "250"),
            ("QUIT".into(), "221"),
        ] {
            client
                .get_mut()
                .write_all(format!("{command}\r\n").as_bytes())
                .unwrap();
            client.get_mut().flush().unwrap();
            smtp_reply(&mut client, expected);
        }
    }
}
