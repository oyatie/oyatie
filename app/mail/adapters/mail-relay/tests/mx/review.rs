use super::legacy_review::accept;
use super::*;
use hickory_resolver::proto::{
    op::{Message, OpCode, ResponseCode},
    rr::{
        Name, RData, Record, RecordType,
        rdata::{A, MX},
    },
};
use std::{net::Ipv4Addr, sync::Mutex, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, UdpSocket},
};
struct Oracle {
    address: std::net::SocketAddr,
    queries: Arc<Mutex<Vec<String>>>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Oracle {
    fn drop(&mut self) {
        self.task.abort();
    }
}
async fn dns(mode: &'static str) -> Oracle {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let address = socket.local_addr().unwrap();
    let queries = Arc::new(Mutex::new(vec![]));
    let captured = queries.clone();
    let task = tokio::spawn(async move {
        let mut bytes = [0; 8192];
        loop {
            let (size, peer) = socket.recv_from(&mut bytes).await.unwrap();
            let request = Message::from_vec(&bytes[..size]).unwrap();
            let query = request.queries[0].clone();
            let name = query.name().to_utf8();
            captured
                .lock()
                .unwrap()
                .push(format!("{name} {}", query.query_type()));
            let mut response = Message::response(request.id, OpCode::Query);
            response.metadata.authoritative = true;
            response.metadata.recursion_available = true;
            response.add_query(query.clone());
            let mut records = vec![];
            if query.query_type() == RecordType::MX {
                let hosts = match mode {
                    "self" => vec![(10, "outgoing.example.org".into())],
                    "loop" => vec![
                        (10, "better.example".into()),
                        (20, "outgoing.example.org".into()),
                        (20, "equal.example".into()),
                        (30, "worse.example".into()),
                    ],
                    "bounds" => (0..65).map(|i| (i, format!("mx{i}.example"))).collect(),
                    "mixed" => vec![(0, ".".into()), (10, "first.example".into())],
                    "equal" => vec![(10, "first.example".into()), (10, "second.example".into())],
                    _ => vec![(10, "first.example".into()), (20, "second.example".into())],
                };
                records.extend(hosts.into_iter().map(|(priority, host)| {
                    RData::MX(MX::new(priority, Name::from_ascii(host).unwrap()))
                }));
            } else if mode == "servfail" {
                response.metadata.response_code = ResponseCode::ServFail;
            } else if mode != "loop" && query.query_type() == RecordType::A {
                records.push(RData::A(A(Ipv4Addr::LOCALHOST)));
            }
            for data in records {
                response.add_answer(Record::from_rdata(query.name().clone(), 30, data));
            }
            socket
                .send_to(&response.to_vec().unwrap(), peer)
                .await
                .unwrap();
        }
    });
    Oracle {
        address,
        queries,
        task,
    }
}
fn direct(oracle: &Oracle, port: u16) -> MxTransport {
    MxTransport::new(
        MxConfig {
            helo: "outgoing.example.org".into(),
            port,
            dns_servers: vec![oracle.address],
            require_tls: false,
        },
        Arc::new(
            ClientConfig::builder()
                .with_root_certificates(RootCertStore::empty())
                .with_no_client_auth(),
        ),
    )
    .unwrap()
}
async fn line(stream: &mut BufReader<tokio::net::TcpStream>) -> String {
    let mut line = String::new();
    tokio::time::timeout(Duration::from_secs(5), stream.read_line(&mut line))
        .await
        .unwrap()
        .unwrap();
    line
}
async fn envelope(stream: &mut BufReader<tokio::net::TcpStream>) {
    stream.get_mut().write_all(b"220 mx\r\n").await.unwrap();
    assert_eq!(line(stream).await, "EHLO outgoing.example.org\r\n");
    stream
        .get_mut()
        .write_all(b"250-mx\r\n250 AUTH PLAIN\r\n")
        .await
        .unwrap();
    assert_eq!(
        line(stream).await,
        "MAIL FROM:<sender@example.org>\r\n",
        "MX must receive no relay AUTH credentials"
    );
    stream.get_mut().write_all(b"250 sender\r\n").await.unwrap();
    assert!(line(stream).await.starts_with("RCPT TO:"));
    stream
        .get_mut()
        .write_all(b"250 recipient\r\n")
        .await
        .unwrap();
    assert_eq!(line(stream).await, "DATA\r\n");
    stream.get_mut().write_all(b"354 body\r\n").await.unwrap();
    loop {
        let next = line(stream).await;
        assert!(!next.is_empty(), "SMTP body truncated");
        if next == ".\r\n" {
            break;
        }
    }
}
async fn final_reply(code: u16) -> (DeliveryOutcome, usize) {
    let dns = dns("ambiguous").await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = tokio::spawn(async move {
        let first = accept(&listener).await;
        let mut first = BufReader::new(first);
        envelope(&mut first).await;
        if code != 0 {
            first
                .get_mut()
                .write_all(format!("{code} settled\r\n").as_bytes())
                .await
                .unwrap();
        }
        drop(first);
        if let Ok(Ok((second, _))) =
            tokio::time::timeout(Duration::from_millis(500), listener.accept()).await
        {
            let mut second = BufReader::new(second);
            envelope(&mut second).await;
            second
                .get_mut()
                .write_all(b"250 accepted\r\n")
                .await
                .unwrap();
            2
        } else {
            1
        }
    });
    let outcome = direct(&dns, port)
        .send("u@review.example", &message())
        .await;
    (outcome, server.await.unwrap())
}
#[tokio::test]
async fn final_data_acknowledgment_loss_does_not_replay_to_another_mx() {
    let (outcome, delivered) = final_reply(0).await;
    assert_eq!(
        delivered, 1,
        "ambiguous DATA response immediately replayed a complete message to another MX"
    );
    assert_eq!(outcome, DeliveryOutcome::Temporary(451));
}
#[tokio::test]
async fn explicit_data_result_controls_retry_and_settles_acceptance() {
    for (code, expected, count) in [
        (250, DeliveryOutcome::Delivered, 1),
        (550, DeliveryOutcome::Permanent(550), 1),
        (451, DeliveryOutcome::Delivered, 2),
        (354, DeliveryOutcome::Temporary(451), 1),
    ] {
        let (outcome, delivered) = final_reply(code).await;
        assert_eq!(delivered, count, "SMTP {code}");
        assert_eq!(outcome, expected, "SMTP {code}");
    }
}
#[tokio::test]
async fn mx_loop_cutoff_removes_equal_and_worse_routes() {
    let oracle = dns("loop").await;
    assert_eq!(
        direct(&oracle, 25)
            .send("u@review.example", &message())
            .await,
        DeliveryOutcome::Permanent(550)
    );
    let queries = oracle.queries.lock().unwrap();
    assert!(queries.iter().any(|q| q.starts_with("better.example.")));
    assert!(
        !queries.iter().any(
            |q| ["outgoing.example.org.", "equal.example.", "worse.example."]
                .iter()
                .any(|host| q.starts_with(host))
        ),
        "{queries:?}"
    );
}
#[tokio::test]
async fn excessive_or_invalid_mx_sets_do_not_trigger_address_lookups() {
    for (mode, expected) in [
        ("bounds", DeliveryOutcome::Temporary(451)),
        ("mixed", DeliveryOutcome::Permanent(556)),
        ("self", DeliveryOutcome::Permanent(554)),
    ] {
        let oracle = dns(mode).await;
        assert_eq!(
            direct(&oracle, 25)
                .send("u@review.example", &message())
                .await,
            expected
        );
        assert!(
            oracle
                .queries
                .lock()
                .unwrap()
                .iter()
                .all(|q| q.ends_with(" MX"))
        );
    }
}
#[tokio::test]
async fn equal_priority_peers_are_tried_and_transient_address_errors_remain_temporary() {
    let closed = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = closed.local_addr().unwrap().port();
    drop(closed);
    let oracle = dns("equal").await;
    assert_eq!(
        direct(&oracle, port)
            .send("u@review.example", &message())
            .await,
        DeliveryOutcome::Temporary(451)
    );
    let queries = oracle.queries.lock().unwrap().clone();
    for host in ["first.example.", "second.example."] {
        assert!(queries.iter().any(|q| q.starts_with(host)), "{queries:?}");
    }
    let oracle = dns("servfail").await;
    assert_eq!(
        direct(&oracle, port)
            .send("u@review.example", &message())
            .await,
        DeliveryOutcome::Temporary(451)
    );
}
#[tokio::test]
async fn refused_starttls_never_falls_back_to_plaintext_mail() {
    let dns = Dns::start().await;
    for required in [false, true] {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let task = tokio::spawn(async move {
            let stream = accept(&listener).await;
            let mut stream = BufReader::new(stream);
            stream.get_mut().write_all(b"220 mx\r\n").await.unwrap();
            assert!(line(&mut stream).await.starts_with("EHLO "));
            stream
                .get_mut()
                .write_all(b"250-mx\r\n250 STARTTLS\r\n")
                .await
                .unwrap();
            assert_eq!(line(&mut stream).await, "STARTTLS\r\n");
            stream
                .get_mut()
                .write_all(b"454 unavailable\r\n")
                .await
                .unwrap();
            assert!(
                line(&mut stream).await.is_empty(),
                "plaintext downgrade after STARTTLS refusal"
            );
        });
        assert_eq!(
            transport(&dns, port, required)
                .send("u@implicit.example", &message())
                .await,
            DeliveryOutcome::Temporary(451)
        );
        task.await.unwrap();
    }
}
