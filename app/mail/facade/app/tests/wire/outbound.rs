use super::*;
use rustls::{
    ServerConfig, ServerConnection,
    pki_types::{CertificateDer, PrivatePkcs8KeyDer},
};
use std::{
    net::{SocketAddr, TcpListener},
    sync::mpsc,
};

pub(super) struct RelayServer {
    pub address: SocketAddr,
    received: mpsc::Receiver<(String, Vec<u8>)>,
    worker: std::thread::JoinHandle<()>,
}

impl RelayServer {
    pub fn new(certificate: CertificateDer<'static>, key: Vec<u8>) -> Self {
        let tls = Arc::new(
            ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(vec![certificate], PrivatePkcs8KeyDer::from(key).into())
                .unwrap(),
        );
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        let (send, received) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + Duration::from_secs(180);
            for _ in 0..3 {
                let tcp = loop {
                    match listener.accept() {
                        Ok((tcp, _)) => break tcp,
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            assert!(
                                std::time::Instant::now() < deadline,
                                "relay acceptance timed out"
                            );
                            std::thread::sleep(Duration::from_millis(10));
                        }
                        Err(e) => panic!("{e}"),
                    }
                };
                tcp.set_nonblocking(false).unwrap();
                tcp.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
                tcp.set_write_timeout(Some(Duration::from_secs(10)))
                    .unwrap();
                let connection = ServerConnection::new(tls.clone()).unwrap();
                let mut stream = BufReader::new(StreamOwned::new(connection, tcp));
                respond(&mut stream, "220 relay\r\n");
                assert_eq!(read(&mut stream), "EHLO mail.example.org\r\n");
                respond(&mut stream, "250-relay\r\n250 8BITMIME\r\n");
                assert_eq!(read(&mut stream), "MAIL FROM:<alice@example.org>\r\n");
                respond(&mut stream, "250 Sender\r\n");
                let recipient = read(&mut stream);
                if recipient == "RCPT TO:<reject@remote.org>\r\n" {
                    respond(&mut stream, "550 No mailbox\r\n");
                    send.send((recipient, Vec::new())).unwrap();
                    continue;
                }
                assert!(matches!(
                    recipient.as_str(),
                    "RCPT TO:<bob@remote.org>\r\n" | "RCPT TO:<jmap@remote.org>\r\n"
                ));
                respond(&mut stream, "250 Recipient\r\n");
                assert_eq!(read(&mut stream), "DATA\r\n");
                respond(&mut stream, "354 Content\r\n");
                let mut raw = Vec::new();
                loop {
                    let line = read(&mut stream);
                    if line == ".\r\n" {
                        break;
                    }
                    raw.extend_from_slice(line.strip_prefix('.').unwrap_or(&line).as_bytes());
                }
                respond(&mut stream, "250 Accepted\r\n");
                send.send((recipient, raw)).unwrap();
            }
        });
        Self {
            address,
            received,
            worker,
        }
    }

    pub fn verify(self, client: &reqwest::blocking::Client, url: &str, token: &str) {
        let mut delivered = 0;
        let mut submitted = 0;
        for _ in 0..3 {
            let (recipient, raw) = self.received.recv_timeout(Duration::from_secs(20)).unwrap();
            if recipient.contains("bob@") {
                assert_eq!(
                    raw,
                    b"From: alice@example.org\r\nDate: Tue, 15 Sep 2026 00:00:00 +0000\r\nMessage-ID: <submission@example.org>\r\nSubject: submitted\r\n\r\nTLS submission\r\n"
                );
                delivered += 1;
            } else if recipient.contains("jmap@") {
                let text = String::from_utf8(raw).unwrap();
                assert!(text.contains("JMAP durable delivery"));
                assert!(!text.to_ascii_lowercase().contains("bcc:"));
                assert!(!text.contains("jmap@remote.org"));
                submitted += 1;
            }
        }
        assert_eq!(delivered, 1);
        assert_eq!(submitted, 1);
        self.worker.join().unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        loop {
            let response: Value=client.post(format!("{url}/jmap")).bearer_auth(token).json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/query",{"accountId":"alice","limit":256},"notice"]]})).send().unwrap().error_for_status().unwrap().json().unwrap();
            let ids = &response["methodResponses"][0][1]["ids"];
            let emails: Value = client.post(format!("{url}/jmap")).bearer_auth(token).json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/get",{"accountId":"alice","ids":ids,"properties":["subject"]},"notice"]]})).send().unwrap().error_for_status().unwrap().json().unwrap();
            if emails["methodResponses"][0][1]["list"]
                .as_array()
                .is_some_and(|emails| {
                    emails
                        .iter()
                        .filter(|email| email["subject"] == "Delivery failure")
                        .count()
                        == 1
                })
            {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "failure notice was not delivered: {response}"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

fn read(stream: &mut BufReader<StreamOwned<ServerConnection, TcpStream>>) -> String {
    let mut line = String::new();
    assert!(stream.read_line(&mut line).unwrap() > 0, "relay EOF");
    line
}
fn respond(stream: &mut BufReader<StreamOwned<ServerConnection, TcpStream>>, value: &str) {
    stream.get_mut().write_all(value.as_bytes()).unwrap();
    stream.get_mut().flush().unwrap();
}
