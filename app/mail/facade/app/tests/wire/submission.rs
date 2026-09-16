use super::*;
use base64::{Engine, engine::general_purpose::STANDARD};

pub(super) fn verify(address: &str, config: Arc<ClientConfig>, token: &str) {
    // Plain SMTP must never elicit a greeting or an AUTH challenge here.
    let mut plaintext = TcpStream::connect(address).unwrap();
    plaintext
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    plaintext.write_all(b"EHLO plaintext.example\r\n").unwrap();
    let mut bytes = [0; 128];
    if let Ok(n) = plaintext.read(&mut bytes) {
        assert!(!bytes[..n].starts_with(b"220"));
        assert!(!bytes[..n].starts_with(b"250"));
    }
    let connection =
        ClientConnection::new(config, ServerName::try_from("localhost").unwrap()).unwrap();
    let tcp = TcpStream::connect(address).unwrap();
    tcp.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    tcp.set_write_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut client = BufReader::new(StreamOwned::new(connection, tcp));
    smtp_reply(&mut client, "220");
    let auth = STANDARD.encode(format!("\0alice@example.org\0{token}"));
    for (command, expected) in [
        ("EHLO client.example".into(), "250"),
        ("MAIL FROM:<alice@example.org>".into(), "530"),
        (format!("AUTH PLAIN {auth}"), "235"),
        ("MAIL FROM:<impostor@example.org>".into(), "550"),
        ("MAIL FROM:<alice@example.org>".into(), "250"),
        ("RCPT TO:<alice@example.org>".into(), "250"),
        ("RCPT TO:<bob@remote.org>".into(), "250"),
        ("RCPT TO:<reject@remote.org>".into(), "250"),
        ("DATA".into(), "354"),
        (
            "From: alice@example.org\r\nDate: Tue, 15 Sep 2026 00:00:00 +0000\r\nMessage-ID: <submission@example.org>\r\nSubject: submitted\r\n\r\nTLS submission\r\n.".into(),
            "250",
        ),
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
