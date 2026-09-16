use super::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use rustls::{
    ClientConfig, ClientConnection, RootCertStore, StreamOwned,
    pki_types::{CertificateDer, ServerName, pem::PemObject},
};
use std::{
    net::{TcpListener, UdpSocket},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

struct Dns {
    address: std::net::SocketAddr,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Dns {
    fn start() -> Self {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        let address = socket.local_addr().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let ended = stop.clone();
        let thread = std::thread::spawn(move || {
            let mut bytes = [0; 4096];
            while !ended.load(Ordering::Acquire) {
                let (size, peer) = match socket.recv_from(&mut bytes) {
                    Ok(result) => result,
                    Err(e)
                        if matches!(
                            e.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                        ) =>
                    {
                        continue;
                    }
                    Err(e) => panic!("local DNS: {e}"),
                };
                let request = &bytes[..size];
                assert!(request.len() >= 17);
                let mut end = 12;
                let mut labels = Vec::new();
                while request[end] != 0 {
                    labels.push(
                        std::str::from_utf8(&request[end + 1..end + 1 + usize::from(request[end])])
                            .unwrap(),
                    );
                    end += usize::from(request[end]) + 1;
                    assert!(end + 5 <= size);
                }
                end += 1;
                let typ = u16::from_be_bytes([request[end], request[end + 1]]);
                end += 4;
                let mut response = request[..end].to_vec();
                response[2..4].copy_from_slice(&[0x85, 0x80]);
                response[6..12].fill(0);
                let host = labels.join(".").to_ascii_lowercase();
                let data = match (host.as_str(), typ) {
                    ("remote.example", 15) => Some(b"\0\x0a\x02mx\x07example\0".to_vec()),
                    ("mx.example", 1) => Some(vec![127, 0, 0, 1]),
                    ("mx.example", 28) => None,
                    query => panic!("unexpected DNS query: {query:?}"),
                };
                if let Some(data) = data {
                    response[7] = 1;
                    response.extend_from_slice(&[0xc0, 0x0c]);
                    response.extend_from_slice(&typ.to_be_bytes());
                    response.extend_from_slice(&[0, 1, 0, 0, 0, 30]);
                    response.extend_from_slice(&(data.len() as u16).to_be_bytes());
                    response.extend(data);
                }
                socket.send_to(&response, peer).unwrap();
            }
        });
        Self {
            address,
            stop,
            thread: Some(thread),
        }
    }
}
impl Drop for Dns {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let result = self.thread.take().unwrap().join();
        if !std::thread::panicking() {
            result.unwrap();
        }
    }
}
fn reply<S: Read>(stream: &mut BufReader<S>, expected: &str) {
    loop {
        let mut line = String::new();
        assert!(stream.read_line(&mut line).unwrap() > 0);
        assert!(line.starts_with(expected), "{line}");
        if line.as_bytes()[3] == b' ' {
            break;
        }
    }
}
#[test]
fn configured_direct_transport_delivers_a_queued_submission_to_dns_selected_mx() {
    let fixture = Fixture::new();
    let dns = Dns::start();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    let (send, received) = std::sync::mpsc::channel();
    let remote = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "MX never contacted");
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("{error}"),
            }
        };
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut stream = BufReader::new(stream);
        stream
            .get_mut()
            .write_all(b"220 direct fixture\r\n")
            .unwrap();
        assert_eq!(line(&mut stream), "EHLO outgoing.example.org\r\n");
        stream
            .get_mut()
            .write_all(b"250-direct\r\n250 AUTH PLAIN\r\n")
            .unwrap();
        assert_eq!(line(&mut stream), "MAIL FROM:<alice@example.org>\r\n");
        stream.get_mut().write_all(b"250 sender\r\n").unwrap();
        assert_eq!(line(&mut stream), "RCPT TO:<recipient@remote.example>\r\n");
        stream.get_mut().write_all(b"250 recipient\r\n").unwrap();
        assert_eq!(line(&mut stream), "DATA\r\n");
        stream.get_mut().write_all(b"354 content\r\n").unwrap();
        let mut raw = Vec::new();
        loop {
            let mut line = Vec::new();
            assert!(stream.read_until(b'\n', &mut line).unwrap() > 0);
            if line == b".\r\n" {
                break;
            }
            raw.extend_from_slice(if line.starts_with(b"..") {
                &line[1..]
            } else {
                &line
            });
        }
        stream.get_mut().write_all(b"250 accepted\r\n").unwrap();
        send.send(raw).unwrap();
    });
    let binary = option_env!("MAIL_APP_BINARY")
        .or(option_env!("CARGO_BIN_EXE_mail-app"))
        .unwrap();
    let token = "direct-review-token-0123456789abcdef";
    assert!(
        Command::new(binary)
            .arg("provision")
            .arg(fixture.root.path().join("mail.db"))
            .args(["tenant", "alice", "alice", "alice@example.org"])
            .env("MAIL_TOKEN", token)
            .status()
            .unwrap()
            .success()
    );
    let mut command = fixture.command();
    command
        .env("MAIL_MX_DNS_SERVERS", dns.address.to_string())
        .env("MAIL_MX_HELO", "outgoing.example.org")
        .env("MAIL_MX_PORT", port.to_string())
        .env("MAIL_MX_REQUIRE_TLS", "false")
        .env("MAIL_MX_CA", &fixture.cert);
    let (mut server, endpoints) = launch(command);
    let mut roots = RootCertStore::empty();
    for cert in CertificateDer::pem_file_iter(&fixture.cert).unwrap() {
        roots.add(cert.unwrap()).unwrap();
    }
    let config = Arc::new(
        ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth(),
    );
    let connection =
        ClientConnection::new(config, ServerName::try_from("localhost").unwrap()).unwrap();
    let mut client = BufReader::new(StreamOwned::new(
        connection,
        socket(endpoint(&endpoints, "SUBMISSIONS ")),
    ));
    reply(&mut client, "220");
    for (command, expected) in [
        ("EHLO client.example".to_owned(), "250"),
        (
            format!(
                "AUTH PLAIN {}",
                STANDARD.encode(format!("\0alice@example.org\0{token}"))
            ),
            "235",
        ),
        ("MAIL FROM:<alice@example.org>".into(), "250"),
        ("RCPT TO:<recipient@remote.example>".into(), "250"),
        ("DATA".into(), "354"),
    ] {
        client
            .get_mut()
            .write_all(format!("{command}\r\n").as_bytes())
            .unwrap();
        client.get_mut().flush().unwrap();
        reply(&mut client, expected);
    }
    let raw=b"Date: Tue, 15 Sep 2026 12:00:00 +0000\r\nMessage-ID: <direct-review@example.org>\r\nFrom: alice@example.org\r\nTo: recipient@remote.example\r\nSubject: direct facade\r\n\r\n.dot\r\n";
    client
        .get_mut()
        .write_all(
            String::from_utf8_lossy(raw)
                .replace("\r\n.", "\r\n..")
                .as_bytes(),
        )
        .unwrap();
    client.get_mut().write_all(b".\r\n").unwrap();
    client.get_mut().flush().unwrap();
    reply(&mut client, "250");
    assert_eq!(received.recv_timeout(Duration::from_secs(10)).unwrap(), raw);
    remote.join().unwrap();
    client.get_mut().write_all(b"QUIT\r\n").unwrap();
    client.get_mut().flush().unwrap();
    reply(&mut client, "221");
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
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = server.0.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            assert!(Instant::now() < deadline, "outbound worker did not drain");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
