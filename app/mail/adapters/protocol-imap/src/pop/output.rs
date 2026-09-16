use std::io;
use tokio::sync::mpsc::Sender;

const CHUNK: usize = 64 * 1024;

pub(super) struct Output {
    send: Sender<Vec<u8>>,
    buffer: Vec<u8>,
    failed: bool,
}
impl Output {
    pub fn new(send: Sender<Vec<u8>>) -> Self {
        Self {
            send,
            buffer: Vec::with_capacity(CHUNK),
            failed: false,
        }
    }
    pub fn is_closed(&self) -> bool {
        self.failed || self.send.is_closed()
    }
    pub fn bytes(&mut self, mut bytes: &[u8]) {
        while !bytes.is_empty() && !self.is_closed() {
            let count = bytes.len().min(CHUNK - self.buffer.len());
            self.buffer.extend_from_slice(&bytes[..count]);
            bytes = &bytes[count..];
            if self.buffer.len() == CHUNK {
                self.flush();
            }
        }
    }
    pub fn text(&mut self, text: &str) {
        self.bytes(text.as_bytes());
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
