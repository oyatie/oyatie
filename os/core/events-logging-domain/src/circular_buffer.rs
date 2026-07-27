//! A byte-oriented circular buffer with line framing, used to retain the most
//! recent service log output in memory.
//!
//! Mirrors `pkg/circular` (the Talos circular buffer backing
//! `internal/app/machined/pkg/system/runner` log capture). The real one is
//! chunked and supports concurrent readers; we model the essential behavior:
//! fixed capacity, oldest bytes overwritten, and the ability to read back the
//! retained tail and split it into log lines.

/// Fixed-capacity byte ring buffer.
#[derive(Debug, Clone)]
pub struct CircularBuffer {
    buf: Vec<u8>,
    capacity: usize,
    /// Total bytes ever written (used to compute the read offset / detect
    /// overflow).
    written: u64,
}

impl CircularBuffer {
    /// Create a buffer with the given capacity (clamped to at least 1).
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        CircularBuffer {
            buf: Vec::with_capacity(capacity),
            capacity,
            written: 0,
        }
    }

    /// Capacity in bytes.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Number of bytes currently retained.
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Whether the buffer holds no bytes.
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Total bytes ever written to the buffer.
    pub fn total_written(&self) -> u64 {
        self.written
    }

    /// Whether the buffer has wrapped (lost the start of the stream).
    pub fn has_overflowed(&self) -> bool {
        self.written > self.capacity as u64
    }

    /// Append bytes, evicting oldest bytes when capacity is exceeded.
    pub fn write(&mut self, data: &[u8]) {
        self.written += data.len() as u64;
        if data.len() >= self.capacity {
            // Only the tail fits.
            let start = data.len() - self.capacity;
            self.buf.clear();
            self.buf.extend_from_slice(&data[start..]);
            return;
        }
        let overflow = (self.buf.len() + data.len()).saturating_sub(self.capacity);
        if overflow > 0 {
            self.buf.drain(0..overflow);
        }
        self.buf.extend_from_slice(data);
    }

    /// The retained bytes, oldest-first.
    pub fn bytes(&self) -> &[u8] {
        &self.buf
    }

    /// Split the retained bytes into newline-delimited lines (lossy UTF-8). A
    /// trailing partial line (no terminating newline) is included.
    pub fn lines(&self) -> Vec<String> {
        if self.buf.is_empty() {
            return Vec::new();
        }
        self.buf
            .split(|&b| b == b'\n')
            .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Clear the retained bytes but keep the written counter.
    pub fn clear(&mut self) {
        self.buf.clear();
    }

    /// The absolute byte offset (in the total stream) of the first retained
    /// byte. Equals `total_written - len` once the buffer has wrapped.
    pub fn start_offset(&self) -> u64 {
        self.written - self.buf.len() as u64
    }

    /// The absolute byte offset just past the last retained byte (== total
    /// written).
    pub fn end_offset(&self) -> u64 {
        self.written
    }

    /// Read retained bytes starting at the absolute stream `offset`.
    ///
    /// Returns the slice from `offset` to the end. If `offset` is before the
    /// retained window (the consumer fell behind and bytes were evicted) the
    /// read is clamped to the start of the window and a flag indicates the gap.
    /// The returned tuple is `(bytes, lost)` where `lost` is the number of bytes
    /// that were skipped due to eviction.
    pub fn read_from(&self, offset: u64) -> (&[u8], u64) {
        let start = self.start_offset();
        if offset >= self.end_offset() {
            return (&[], 0);
        }
        if offset < start {
            let lost = start - offset;
            return (&self.buf, lost);
        }
        // `offset - start` is bounded above by `end_offset() - start_offset()`,
        // i.e. the current buffer length, so it always fits in `usize`.
        #[allow(clippy::cast_possible_truncation)]
        let idx = (offset - start) as usize;
        (&self.buf[idx..], 0)
    }

    /// Split the retained bytes into complete lines (those terminated by `\n`),
    /// returning each line together with the absolute stream offset just past
    /// its terminating newline (a resumable cursor). A trailing partial line is
    /// not returned.
    pub fn complete_lines(&self) -> Vec<(String, u64)> {
        let mut out = Vec::new();
        let start = self.start_offset();
        let mut line_start = 0usize;
        for (i, &b) in self.buf.iter().enumerate() {
            if b == b'\n' {
                let chunk = &self.buf[line_start..i];
                let next = start + (i as u64) + 1;
                out.push((String::from_utf8_lossy(chunk).into_owned(), next));
                line_start = i + 1;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_append_and_read() {
        let mut b = CircularBuffer::with_capacity(64);
        b.write(b"hello ");
        b.write(b"world");
        assert_eq!(b.bytes(), b"hello world");
        assert_eq!(b.total_written(), 11);
        assert!(!b.has_overflowed());
    }

    #[test]
    fn evicts_oldest_on_overflow() {
        let mut b = CircularBuffer::with_capacity(5);
        b.write(b"abc");
        b.write(b"defg"); // total 7, keep last 5
        assert_eq!(b.bytes(), b"cdefg");
        assert!(b.has_overflowed());
        assert_eq!(b.total_written(), 7);
    }

    #[test]
    fn single_write_larger_than_capacity_keeps_tail() {
        let mut b = CircularBuffer::with_capacity(4);
        b.write(b"0123456789");
        assert_eq!(b.bytes(), b"6789");
        assert_eq!(b.len(), 4);
    }

    #[test]
    fn lines_split_on_newline() {
        let mut b = CircularBuffer::with_capacity(128);
        b.write(b"line1\nline2\npartial");
        let lines = b.lines();
        assert_eq!(lines, ["line1", "line2", "partial"]);
    }

    #[test]
    fn offsets_track_stream_position() {
        let mut b = CircularBuffer::with_capacity(5);
        b.write(b"abc");
        assert_eq!(b.start_offset(), 0);
        assert_eq!(b.end_offset(), 3);
        b.write(b"defg"); // wrapped: keep "cdefg", lost "ab"
        assert_eq!(b.start_offset(), 2);
        assert_eq!(b.end_offset(), 7);
        assert_eq!(b.bytes(), b"cdefg");
    }

    #[test]
    fn read_from_offset_and_gap_detection() {
        let mut b = CircularBuffer::with_capacity(5);
        b.write(b"abcdefg"); // retains "cdefg", start_offset=2
        // reading from within the window
        let (bytes, lost) = b.read_from(4);
        assert_eq!(bytes, b"efg");
        assert_eq!(lost, 0);
        // reading from before the window: clamped, reports lost bytes
        let (bytes, lost) = b.read_from(0);
        assert_eq!(bytes, b"cdefg");
        assert_eq!(lost, 2);
        // reading past the end
        let (bytes, lost) = b.read_from(100);
        assert!(bytes.is_empty());
        assert_eq!(lost, 0);
    }

    #[test]
    fn complete_lines_with_resumable_offsets() {
        let mut b = CircularBuffer::with_capacity(128);
        b.write(b"l1\nl2\npart");
        let lines = b.complete_lines();
        // "part" has no newline => excluded
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], ("l1".to_string(), 3));
        assert_eq!(lines[1], ("l2".to_string(), 6));
        // resume from the second cursor: remaining retained tail after offset 6
        let (rest, lost) = b.read_from(6);
        assert_eq!(rest, b"part");
        assert_eq!(lost, 0);
    }
}
