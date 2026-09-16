use std::io;
use tokio::sync::mpsc::Sender;

const CHUNK: usize = 64 * 1024;

/// A synchronous command emits bounded chunks to the asynchronous socket task.
/// A slow or disconnected reader stops further body reads at the producer.
pub(super) struct Output {
    pub utf8: bool,
    pub condstore: bool,
    pub qresync: bool,
    pub objectid: bool,
    pub uidonly: bool,
    send: Sender<Vec<u8>>,
    buffer: Vec<u8>,
    failed: bool,
}
impl Output {
    pub fn new(send: Sender<Vec<u8>>) -> Self {
        Self {
            utf8: false,
            condstore: false,
            qresync: false,
            objectid: false,
            uidonly: false,
            send,
            buffer: Vec::with_capacity(CHUNK),
            failed: false,
        }
    }
    pub fn fetch_start(&mut self, sequence: usize, uid: u32) {
        let (id, kind) = if self.uidonly {
            (uid as usize, "UIDFETCH")
        } else {
            (sequence, "FETCH")
        };
        self.extend_from_slice(format!("* {id} {kind} (UID {uid}").as_bytes());
    }
    pub fn is_closed(&self) -> bool {
        self.failed || self.send.is_closed()
    }
    pub fn extend_from_slice(&mut self, mut bytes: &[u8]) {
        while !bytes.is_empty() && !self.is_closed() {
            let count = bytes.len().min(CHUNK - self.buffer.len());
            self.buffer.extend_from_slice(&bytes[..count]);
            bytes = &bytes[count..];
            if self.buffer.len() == CHUNK {
                self.flush();
            }
        }
    }
    fn flush(&mut self) {
        if !self.buffer.is_empty() && !self.is_closed() {
            self.failed = self
                .send
                .blocking_send(std::mem::take(&mut self.buffer))
                .is_err();
        }
    }
    pub fn finish(&mut self) -> io::Result<()> {
        self.flush();
        if self.is_closed() {
            Err(io::ErrorKind::BrokenPipe.into())
        } else {
            Ok(())
        }
    }
}
