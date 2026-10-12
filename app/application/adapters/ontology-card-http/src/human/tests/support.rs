use super::super::{HumanOntologyCardSource, HumanOntologyReadSource};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc::{Receiver, channel},
};

pub const OBJECT: &str = r#"{"object_ref":"ent_alpha","written_revision":1,"upcast_state":"upcast_pending","properties":{"name":{"value_type":"string","data_class":"PII","value":"Ada"}}}"#;

pub fn cookie(letter: char) -> String {
    format!("__Host-oya_session=v1.{}A", letter.to_string().repeat(42))
}

pub fn source(base: &str, cookie: &str) -> HumanOntologyReadSource {
    HumanOntologyCardSource::for_loopback(base)
        .unwrap()
        .read_source(cookie)
        .unwrap()
}

pub fn stub(replies: Vec<(u16, String, String)>) -> (String, Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let (sender, seen) = channel();
    std::thread::spawn(move || {
        for (status, body, headers) in replies {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(3)))
                .unwrap();
            let mut request = Vec::new();
            let mut chunk = [0; 1024];
            while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                let n = stream.read(&mut chunk).unwrap();
                if n == 0 {
                    break;
                }
                request.extend_from_slice(&chunk[..n]);
            }
            let _ = sender.send(String::from_utf8(request).unwrap());
            let _ = write!(
                stream,
                "HTTP/1.1 {status} Response\r\ncontent-type: application/json\r\ncache-control: no-store\r\ncontent-length: {}\r\n{headers}connection: close\r\n\r\n{body}",
                body.len()
            );
        }
    });
    (base, seen)
}

pub fn raw_stub(reply: Vec<u8>, delay: std::time::Duration) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut chunk = [0; 1024];
        let _ = stream.read(&mut chunk);
        std::thread::sleep(delay);
        let _ = stream.write_all(&reply);
    });
    base
}
