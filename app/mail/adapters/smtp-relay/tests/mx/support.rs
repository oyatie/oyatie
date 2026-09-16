use hickory_resolver::proto::{
    op::{Message, OpCode, ResponseCode},
    rr::{
        Name, RData, Record, RecordType,
        rdata::{A, AAAA, MX},
    },
};
use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, UdpSocket},
    task::JoinHandle,
};

pub struct Dns {
    pub address: SocketAddr,
    pub queries: Arc<Mutex<Vec<String>>>,
    task: JoinHandle<()>,
}

impl Dns {
    pub async fn start() -> Self {
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let address = socket.local_addr().unwrap();
        let queries = Arc::new(Mutex::new(Vec::new()));
        let captured = queries.clone();
        let task = tokio::spawn(async move {
            let mut bytes = [0; 4096];
            loop {
                let (size, peer) = socket.recv_from(&mut bytes).await.unwrap();
                let request = Message::from_vec(&bytes[..size]).unwrap();
                let query = request.queries[0].clone();
                let name = query.name().to_lowercase().to_utf8();
                captured
                    .lock()
                    .unwrap()
                    .push(format!("{name} {}", query.query_type()));
                let mut response = Message::response(request.id, OpCode::Query);
                response.metadata.recursion_desired = true;
                response.metadata.recursion_available = true;
                response.metadata.authoritative = true;
                response.add_query(query.clone());
                let records = match (name.as_str(), query.query_type()) {
                    ("missing.example.", _) => {
                        response.metadata.response_code = ResponseCode::NXDomain;
                        vec![]
                    }
                    ("servfail.example.", _) => {
                        response.metadata.response_code = ResponseCode::ServFail;
                        vec![]
                    }
                    ("null.example.", RecordType::MX) => vec![RData::MX(MX::new(0, Name::root()))],
                    ("mx.example.", RecordType::MX) => vec![
                        RData::MX(MX::new(20, Name::from_ascii("up.mx.example.").unwrap())),
                        RData::MX(MX::new(10, Name::from_ascii("down.mx.example.").unwrap())),
                    ],
                    ("up.mx.example." | "implicit.example.", RecordType::A) => {
                        vec![RData::A(A(Ipv4Addr::LOCALHOST))]
                    }
                    ("down.mx.example.", RecordType::AAAA) => {
                        vec![RData::AAAA(AAAA(Ipv6Addr::LOCALHOST))]
                    }
                    _ => vec![],
                };
                for data in records {
                    response.add_answer(Record::from_rdata(query.name().clone(), 30, data));
                }
                socket
                    .send_to(&response.to_vec().unwrap(), peer)
                    .await
                    .unwrap();
            }
        });
        Self {
            address,
            queries,
            task,
        }
    }
}

impl Drop for Dns {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub async fn smtp(reject: u16) -> (u16, JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let task = tokio::spawn(async move {
        let (stream, _) =
            tokio::time::timeout(std::time::Duration::from_secs(5), listener.accept())
                .await
                .unwrap()
                .unwrap();
        let mut stream = BufReader::new(stream);
        stream
            .get_mut()
            .write_all(b"220 receiver\r\n")
            .await
            .unwrap();
        let mut line = String::new();
        stream.read_line(&mut line).await.unwrap();
        assert_eq!(line, "EHLO outgoing.example.org\r\n");
        stream
            .get_mut()
            .write_all(b"250-receiver\r\n250 8BITMIME\r\n")
            .await
            .unwrap();
        line.clear();
        if stream.read_line(&mut line).await.unwrap() == 0 {
            return vec![];
        }
        assert_eq!(line, "MAIL FROM:<sender@example.org>\r\n");
        stream.get_mut().write_all(b"250 sender\r\n").await.unwrap();
        line.clear();
        stream.read_line(&mut line).await.unwrap();
        assert!(line.starts_with("RCPT TO:<"));
        if reject != 0 {
            stream
                .get_mut()
                .write_all(format!("{reject} refused\r\n").as_bytes())
                .await
                .unwrap();
            return vec![];
        }
        stream
            .get_mut()
            .write_all(b"250 recipient\r\n")
            .await
            .unwrap();
        line.clear();
        stream.read_line(&mut line).await.unwrap();
        assert_eq!(line, "DATA\r\n");
        stream.get_mut().write_all(b"354 data\r\n").await.unwrap();
        let mut raw = Vec::new();
        loop {
            let mut line = Vec::new();
            assert!(stream.read_until(b'\n', &mut line).await.unwrap() > 0);
            if line == b".\r\n" {
                break;
            }
            raw.extend_from_slice(if line.starts_with(b".") {
                &line[1..]
            } else {
                &line
            });
        }
        stream
            .get_mut()
            .write_all(b"250 accepted\r\n")
            .await
            .unwrap();
        raw
    });
    (port, task)
}
