use super::*;

fn exchange<S: Read + Write>(stream: &mut BufReader<S>, command: &str) -> String {
    stream.get_mut().write_all(command.as_bytes()).unwrap();
    let tag = command.split_once(' ').unwrap().0;
    let mut response = String::new();
    loop {
        let mut line = String::new();
        assert!(stream.read_line(&mut line).unwrap() > 0, "IMAP EOF");
        response.push_str(&line);
        if line.starts_with(&format!("{tag} ")) {
            return response;
        }
    }
}

pub(super) fn verify(address: &str, config: Arc<ClientConfig>, token: &str) {
    let tcp = TcpStream::connect(address).unwrap();
    tcp.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    tcp.set_write_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut stream = BufReader::new(tcp);
    let capability = exchange(&mut stream, "a CAPABILITY\r\n");
    assert!(capability.contains("STARTTLS"), "{capability}");
    assert!(capability.contains("LOGINDISABLED"), "{capability}");
    let login = exchange(
        &mut stream,
        &format!("b LOGIN alice@example.org {token}\r\n"),
    );
    assert!(login.contains("b NO"), "{login}");
    assert!(exchange(&mut stream, "c STARTTLS extra\r\n").contains("c BAD"));
    assert!(exchange(&mut stream, "d STARTTLS\r\n").contains("d OK"));
    assert!(stream.buffer().is_empty());
    let connection =
        ClientConnection::new(config, ServerName::try_from("localhost").unwrap()).unwrap();
    let mut tls = BufReader::new(StreamOwned::new(connection, stream.into_inner()));
    let capability = exchange(&mut tls, "e CAPABILITY\r\n");
    assert!(!capability.contains("STARTTLS"), "{capability}");
    assert!(!capability.contains("LOGINDISABLED"), "{capability}");
    assert!(
        !capability.contains("* OK"),
        "no second greeting: {capability}"
    );
    assert!(exchange(&mut tls, "f STARTTLS\r\n").contains("f BAD"));
    assert!(exchange(&mut tls, &format!("g LOGIN alice@example.org {token}\r\n")).contains("g OK"));
    assert!(exchange(&mut tls, "h SELECT INBOX\r\n").contains("h OK"));
    assert!(
        exchange(
            &mut tls,
            "i UID FETCH 1 (UID FLAGS INTERNALDATE BODY.PEEK[])\r\n"
        )
        .contains("Subject: Rust wire test")
    );
    assert!(exchange(&mut tls, "j LOGOUT\r\n").contains("j OK"));
}

pub(super) fn smtp(address: &str, config: Arc<ClientConfig>, token: &str, submission: bool) {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let tcp = TcpStream::connect(address).unwrap();
    tcp.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    tcp.set_write_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut stream = BufReader::new(tcp);
    smtp_reply(&mut stream, "220");
    stream.get_mut().write_all(b"STARTTLS\r\n").unwrap();
    smtp_reply(&mut stream, "503");
    stream.get_mut().write_all(b"EHLO console\r\n").unwrap();
    let mut capabilities = String::new();
    loop {
        let mut line = String::new();
        assert!(stream.read_line(&mut line).unwrap() > 0);
        capabilities.push_str(&line);
        if line.starts_with("250 ") {
            break;
        }
    }
    assert!(capabilities.contains("STARTTLS"), "{capabilities}");
    assert!(!capabilities.contains("AUTH"), "{capabilities}");
    stream
        .get_mut()
        .write_all(b"AUTH PLAIN invalid\r\n")
        .unwrap();
    smtp_reply(&mut stream, "538");
    if submission {
        stream
            .get_mut()
            .write_all(b"MAIL FROM:<alice@example.org>\r\n")
            .unwrap();
        smtp_reply(&mut stream, "530");
    }
    stream.get_mut().write_all(b"STARTTLS extra\r\n").unwrap();
    smtp_reply(&mut stream, "501");
    stream.get_mut().write_all(b"STARTTLS\r\n").unwrap();
    smtp_reply(&mut stream, "220");
    assert!(stream.buffer().is_empty());
    let connection =
        ClientConnection::new(config, ServerName::try_from("localhost").unwrap()).unwrap();
    let mut stream = BufReader::new(StreamOwned::new(connection, stream.into_inner()));
    stream
        .get_mut()
        .write_all(b"MAIL FROM:<alice@example.org>\r\n")
        .unwrap();
    smtp_reply(&mut stream, "503");
    stream.get_mut().write_all(b"EHLO console\r\n").unwrap();
    smtp_reply(&mut stream, "250");
    // Already in TLS: RFC 3207 §4 — the reference server answers 504 5.7.4.
    stream.get_mut().write_all(b"STARTTLS\r\n").unwrap();
    smtp_reply(&mut stream, "504 5.7.4");
    if submission {
        let credential = STANDARD.encode(format!("\0alice@example.org\0{token}"));
        stream
            .get_mut()
            .write_all(format!("AUTH PLAIN {credential}\r\n").as_bytes())
            .unwrap();
        smtp_reply(&mut stream, "235");
    }
    stream
        .get_mut()
        .write_all(b"MAIL FROM:<alice@example.org>\r\n")
        .unwrap();
    smtp_reply(&mut stream, "250");
    stream
        .get_mut()
        .write_all(b"RCPT TO:<alice@example.org>\r\n")
        .unwrap();
    smtp_reply(&mut stream, "250");
    stream.get_mut().write_all(b"RSET\r\nQUIT\r\n").unwrap();
    smtp_reply(&mut stream, "250");
    smtp_reply(&mut stream, "221");
}
