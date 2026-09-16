use serde_json::Value;
use std::io::{self, Write};

pub(super) const MAX_RESPONSE: usize = 32 * 1024 * 1024;

/// Bound serialized bytes before retaining another result or cloning a reference.
pub(super) fn charge(value: &Value, remaining: &mut usize) -> Result<(), &'static str> {
    let mut pending = vec![(value, 0)];
    let mut nodes = 0usize;
    while let Some((value, depth)) = pending.pop() {
        nodes += 1;
        if depth > 64 || nodes > 262144 {
            return Err("limit");
        }
        match value {
            Value::Array(values) => pending.extend(values.iter().map(|v| (v, depth + 1))),
            Value::Object(values) => pending.extend(values.values().map(|v| (v, depth + 1))),
            _ => {}
        }
    }
    struct Counter(usize);
    impl Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0 = self
                .0
                .checked_sub(bytes.len())
                .ok_or_else(|| io::Error::from(io::ErrorKind::FileTooLarge))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter(*remaining);
    serde_json::to_writer(&mut counter, value).map_err(|_| "limit")?;
    *remaining = counter.0;
    Ok(())
}

pub(super) struct Buffer {
    pub bytes: Vec<u8>,
    limit: usize,
}
impl Buffer {
    pub fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
        }
    }
}
impl Write for Buffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(io::ErrorKind::FileTooLarge.into());
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
