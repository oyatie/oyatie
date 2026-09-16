use super::*;
use std::time::Duration;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};

pub(super) async fn accept(listener: &TcpListener) -> tokio::net::TcpStream {
    tokio::time::timeout(Duration::from_secs(5), listener.accept())
        .await
        .unwrap()
        .unwrap()
        .0
}
async fn line(stream: &mut BufReader<tokio::net::TcpStream>) -> String {
    let mut line = String::new();
    tokio::time::timeout(Duration::from_secs(5), stream.read_line(&mut line))
        .await
        .unwrap()
        .unwrap();
    line
}
#[tokio::test]
async fn optional_tls_direct_delivery_falls_back_to_helo_without_auth_or_extensions() {
    let dns = Dns::start().await;
    for (code, required, eight_bit) in [
        (500, false, false),
        (502, false, false),
        (500, true, false),
        (502, true, false),
        (502, false, true),
    ] {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let (stream, _) = tokio::time::timeout(Duration::from_secs(5), listener.accept())
                .await
                .unwrap()
                .unwrap();
            let mut stream = BufReader::new(stream);
            stream
                .get_mut()
                .write_all(b"220 legacy SMTP\r\n")
                .await
                .unwrap();
            assert_eq!(line(&mut stream).await, "EHLO outgoing.example.org\r\n");
            stream
                .get_mut()
                .write_all(format!("{code}-EHLO unavailable\r\n{code} AUTH PLAIN\r\n").as_bytes())
                .await
                .unwrap();
            let hello = line(&mut stream).await;
            if required {
                assert!(hello.is_empty(), "required TLS fell back to plaintext");
                return false;
            }
            assert_eq!(hello, "HELO outgoing.example.org\r\n");
            stream
                .get_mut()
                .write_all(b"250-legacy\r\n250-8BITMIME\r\n250 AUTH PLAIN\r\n")
                .await
                .unwrap();
            if eight_bit {
                assert!(
                    line(&mut stream).await.is_empty(),
                    "HELO reply activated ESMTP capabilities"
                );
                return false;
            }
            assert_eq!(
                line(&mut stream).await,
                "MAIL FROM:<sender@example.org>\r\n"
            );
            stream.get_mut().write_all(b"250 sender\r\n").await.unwrap();
            assert_eq!(line(&mut stream).await, "RCPT TO:<u@implicit.example>\r\n");
            stream
                .get_mut()
                .write_all(b"250 recipient\r\n")
                .await
                .unwrap();
            assert_eq!(line(&mut stream).await, "DATA\r\n");
            stream.get_mut().write_all(b"354 body\r\n").await.unwrap();
            loop {
                let line = line(&mut stream).await;
                assert!(!line.is_empty());
                if line == ".\r\n" {
                    break;
                }
            }
            stream
                .get_mut()
                .write_all(b"250 accepted\r\n")
                .await
                .unwrap();
            true
        });
        let mut mail = message();
        if eight_bit {
            mail.raw = "Subject: plain\r\n\r\ncafé\r\n".as_bytes().to_vec();
        }
        let outcome = transport(&dns, port, required)
            .send("u@implicit.example", &mail)
            .await;
        assert_eq!(server.await.unwrap(), !required && !eight_bit);
        assert_eq!(
            outcome,
            if required {
                DeliveryOutcome::Temporary(451)
            } else if eight_bit {
                DeliveryOutcome::Permanent(554)
            } else {
                DeliveryOutcome::Delivered
            }
        );
    }
}
