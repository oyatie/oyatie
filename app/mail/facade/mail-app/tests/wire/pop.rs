use super::*;

fn reply<S: Read>(stream: &mut BufReader<S>) -> String {
    let mut line = String::new();
    assert!(stream.read_line(&mut line).unwrap() > 0);
    line
}

pub(super) fn verify(endpoints: &[&str], config: Arc<ClientConfig>, token: &str, raw: &[u8]) {
    for (index, label, starttls) in [(6, "POPS ", false), (7, "POP STARTTLS ", true)] {
        let address = endpoints
            .get(index)
            .expect("POP listener missing")
            .strip_prefix(label)
            .unwrap();
        let tcp = TcpStream::connect(address).unwrap();
        tcp.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
        tcp.set_write_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut tcp = BufReader::new(tcp);
        if starttls {
            assert!(reply(&mut tcp).starts_with("+OK"));
            tcp.get_mut()
                .write_all(b"USER alice@example.org\r\n")
                .unwrap();
            assert!(reply(&mut tcp).starts_with("-ERR"));
            tcp.get_mut().write_all(b"STLS\r\n").unwrap();
            assert!(reply(&mut tcp).starts_with("+OK"));
            assert!(tcp.buffer().is_empty());
        }
        let connection =
            ClientConnection::new(config.clone(), ServerName::try_from("localhost").unwrap())
                .unwrap();
        let mut stream = BufReader::new(StreamOwned::new(connection, tcp.into_inner()));
        if !starttls {
            assert!(reply(&mut stream).starts_with("+OK"));
        }
        for command in [
            "USER alice@example.org\r\n".to_string(),
            format!("PASS {token}\r\n"),
            "RETR 1\r\n".into(),
        ] {
            stream.get_mut().write_all(command.as_bytes()).unwrap();
            stream.get_mut().flush().unwrap();
            assert!(reply(&mut stream).starts_with("+OK"));
        }
        let mut retrieved = Vec::new();
        loop {
            let mut line = Vec::new();
            assert!(stream.read_until(b'\n', &mut line).unwrap() > 0);
            if line == b".\r\n" {
                break;
            }
            retrieved.extend_from_slice(if line.starts_with(b"..") {
                &line[1..]
            } else {
                &line
            });
        }
        assert_eq!(retrieved, raw);
        stream.get_mut().write_all(b"QUIT\r\n").unwrap();
        stream.get_mut().flush().unwrap();
        assert!(reply(&mut stream).starts_with("+OK"));
    }
}
