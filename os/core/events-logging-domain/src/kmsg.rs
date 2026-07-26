//! Kernel ring-buffer (`/dev/kmsg`) reader and record parser.
//!
//! Mirrors the Talos `Dmesg` API path
//! (`internal/app/machined/pkg/controllers/runtime` + the `kmsg` package it
//! uses). The Linux kernel exposes its log ring buffer through `/dev/kmsg`,
//! where each record is a structured line of the form:
//!
//! ```text
//! <priority>,<seq>,<timestamp_us>,<flags>;<message>
//! ```
//!
//! The `priority` field packs an 8-level syslog severity together with a
//! facility (`priority = facility * 8 + severity`). We model parsing of those
//! records plus the boundary to the kernel as a trait so tests can feed
//! synthetic data without a real `/dev/kmsg`.

use os_kernel::error::{Error, Result};

/// Syslog severity levels (RFC 5424), lowest number = most severe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// System is unusable.
    Emergency,
    /// Action must be taken immediately.
    Alert,
    /// Critical conditions.
    Critical,
    /// Error conditions.
    Error,
    /// Warning conditions.
    Warning,
    /// Normal but significant condition.
    Notice,
    /// Informational messages.
    Info,
    /// Debug-level messages.
    Debug,
}

impl Severity {
    /// Build from the low 3 bits of a kmsg priority value.
    pub fn from_priority(priority: u8) -> Self {
        match priority & 0x07 {
            0 => Severity::Emergency,
            1 => Severity::Alert,
            2 => Severity::Critical,
            3 => Severity::Error,
            4 => Severity::Warning,
            5 => Severity::Notice,
            6 => Severity::Info,
            _ => Severity::Debug,
        }
    }

    /// The numeric severity (0..=7).
    pub fn code(self) -> u8 {
        match self {
            Severity::Emergency => 0,
            Severity::Alert => 1,
            Severity::Critical => 2,
            Severity::Error => 3,
            Severity::Warning => 4,
            Severity::Notice => 5,
            Severity::Info => 6,
            Severity::Debug => 7,
        }
    }

    /// Lowercase label used by the API representation.
    pub fn label(self) -> &'static str {
        match self {
            Severity::Emergency => "emerg",
            Severity::Alert => "alert",
            Severity::Critical => "crit",
            Severity::Error => "err",
            Severity::Warning => "warning",
            Severity::Notice => "notice",
            Severity::Info => "info",
            Severity::Debug => "debug",
        }
    }

    /// Whether this severity is at error level or more severe.
    pub fn is_error(self) -> bool {
        self <= Severity::Error
    }
}

/// A single parsed kernel-log record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KmsgRecord {
    /// Raw priority byte (facility * 8 + severity).
    pub priority: u8,
    /// Monotonic sequence number assigned by the kernel.
    pub sequence: u64,
    /// Timestamp in microseconds since boot.
    pub timestamp_us: u64,
    /// The log message body.
    pub message: String,
}

impl KmsgRecord {
    /// Decoded severity.
    pub fn severity(&self) -> Severity {
        Severity::from_priority(self.priority)
    }

    /// Decoded syslog facility (priority / 8).
    pub fn facility(&self) -> u8 {
        self.priority >> 3
    }

    /// Timestamp expressed in whole seconds since boot.
    pub fn timestamp_secs(&self) -> u64 {
        self.timestamp_us / 1_000_000
    }

    /// Build a record from explicit fields.
    pub fn new(priority: u8, sequence: u64, timestamp_us: u64, message: impl Into<String>) -> Self {
        KmsgRecord {
            priority,
            sequence,
            timestamp_us,
            message: message.into(),
        }
    }

    /// The decoded syslog facility name (subset Linux uses for kernel logging).
    pub fn facility_name(&self) -> &'static str {
        match self.facility() {
            0 => "kern",
            1 => "user",
            2 => "mail",
            3 => "daemon",
            4 => "auth",
            5 => "syslog",
            6 => "lpr",
            7 => "news",
            16..=23 => "local",
            _ => "other",
        }
    }

    /// Serialize back into the canonical `/dev/kmsg` line form
    /// (`<priority>,<seq>,<ts_us>,-;<message>`), without a trailing newline.
    /// This is the inverse of [`KmsgRecord::parse`] for the fields we model.
    pub fn to_kmsg_line(&self) -> String {
        format!(
            "{},{},{},-;{}",
            self.priority, self.sequence, self.timestamp_us, self.message
        )
    }

    /// Render in a human `dmesg`-like form: `[    <secs>.<frac>] <message>`.
    pub fn to_dmesg_line(&self) -> String {
        let secs = self.timestamp_us / 1_000_000;
        let frac = self.timestamp_us % 1_000_000;
        format!("[{:>5}.{:06}] {}", secs, frac, self.message)
    }

    /// Parse a single `/dev/kmsg` record line.
    ///
    /// Format: `<priority>,<seq>,<ts_us>,<flags>[,...];<message>`. Continuation
    /// lines (leading whitespace) are not records and are rejected.
    pub fn parse(line: &str) -> Result<Self> {
        let line = line.trim_end_matches('\n');
        if line.is_empty() {
            return Err(Error::parse("empty kmsg line"));
        }
        if line.starts_with(' ') || line.starts_with('\t') {
            return Err(Error::parse("kmsg continuation line, not a record"));
        }
        let (header, message) = line
            .split_once(';')
            .ok_or_else(|| Error::parse("kmsg record missing ';' separator"))?;
        let mut fields = header.split(',');
        let priority: u8 = fields
            .next()
            .ok_or_else(|| Error::parse("kmsg record missing priority"))?
            .trim()
            .parse()
            .map_err(|_| Error::parse("kmsg priority not a number"))?;
        let sequence: u64 = fields
            .next()
            .ok_or_else(|| Error::parse("kmsg record missing sequence"))?
            .trim()
            .parse()
            .map_err(|_| Error::parse("kmsg sequence not a number"))?;
        let timestamp_us: u64 = fields
            .next()
            .ok_or_else(|| Error::parse("kmsg record missing timestamp"))?
            .trim()
            .parse()
            .map_err(|_| Error::parse("kmsg timestamp not a number"))?;
        Ok(KmsgRecord {
            priority,
            sequence,
            timestamp_us,
            message: message.to_string(),
        })
    }
}

/// Boundary to the kernel log source. In production this wraps `/dev/kmsg`; in
/// tests an in-memory implementation supplies canned lines.
pub trait KmsgSource {
    /// Pull the next raw record line, or `None` when the source is drained
    /// (in follow mode a real source would block instead).
    fn next_line(&mut self) -> Option<String>;
}

/// An in-memory [`KmsgSource`] backed by a queue of raw lines.
#[derive(Debug, Default, Clone)]
pub struct MemoryKmsg {
    lines: std::collections::VecDeque<String>,
}

impl MemoryKmsg {
    /// Create an empty source.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a source pre-loaded with raw record lines.
    pub fn from_lines<I, S>(lines: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        MemoryKmsg {
            lines: lines.into_iter().map(Into::into).collect(),
        }
    }

    /// Append a raw record line.
    pub fn push_line(&mut self, line: impl Into<String>) {
        self.lines.push_back(line.into());
    }
}

impl KmsgSource for MemoryKmsg {
    fn next_line(&mut self) -> Option<String> {
        self.lines.pop_front()
    }
}

/// Reader that turns a [`KmsgSource`] into parsed [`KmsgRecord`]s, skipping
/// malformed/continuation lines.
pub struct KmsgReader<S: KmsgSource> {
    source: S,
}

impl<S: KmsgSource> KmsgReader<S> {
    /// Wrap a source.
    pub fn new(source: S) -> Self {
        KmsgReader { source }
    }

    /// Read the next valid record, skipping lines that fail to parse.
    pub fn next_record(&mut self) -> Option<KmsgRecord> {
        while let Some(line) = self.source.next_line() {
            if let Ok(rec) = KmsgRecord::parse(&line) {
                return Some(rec);
            }
        }
        None
    }

    /// Drain all remaining valid records.
    pub fn drain(&mut self) -> Vec<KmsgRecord> {
        let mut out = Vec::new();
        while let Some(rec) = self.next_record() {
            out.push(rec);
        }
        out
    }

    /// Drain all remaining records whose severity is at most `max` (i.e. at
    /// least as severe as `max`). For example `drain_at_least(Severity::Warning)`
    /// returns emergency..warning and drops notice/info/debug.
    pub fn drain_at_least(&mut self, max: Severity) -> Vec<KmsgRecord> {
        self.drain()
            .into_iter()
            .filter(|r| r.severity() <= max)
            .collect()
    }

    /// Drain all records with a kernel sequence strictly greater than `after`
    /// (a resumable cursor for the `Dmesg` follow path).
    pub fn drain_since(&mut self, after: u64) -> Vec<KmsgRecord> {
        self.drain()
            .into_iter()
            .filter(|r| r.sequence > after)
            .collect()
    }
}

/// Format a slice of records back into a `/dev/kmsg`-style multi-line blob.
pub fn write_kmsg(records: &[KmsgRecord]) -> String {
    let mut out = String::new();
    for r in records {
        out.push_str(&r.to_kmsg_line());
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_record() {
        let rec = KmsgRecord::parse("6,12,1234567,-;hello kernel").unwrap();
        assert_eq!(rec.priority, 6);
        assert_eq!(rec.sequence, 12);
        assert_eq!(rec.timestamp_us, 1_234_567);
        assert_eq!(rec.timestamp_secs(), 1);
        assert_eq!(rec.message, "hello kernel");
        assert_eq!(rec.severity(), Severity::Info);
    }

    #[test]
    fn priority_decodes_facility_and_severity() {
        // facility 3 (daemon) * 8 + severity 3 (err) = 27
        let rec = KmsgRecord::parse("27,1,0,-;daemon error").unwrap();
        assert_eq!(rec.severity(), Severity::Error);
        assert_eq!(rec.facility(), 3);
        assert!(rec.severity().is_error());
    }

    #[test]
    fn rejects_malformed_lines() {
        assert!(KmsgRecord::parse("").is_err());
        assert!(KmsgRecord::parse("no-semicolon-here").is_err());
        assert!(KmsgRecord::parse(" continuation;body").is_err());
        assert!(KmsgRecord::parse("notanumber,1,0,-;x").is_err());
    }

    #[test]
    fn reader_skips_bad_lines_and_drains() {
        let src = MemoryKmsg::from_lines([
            "6,1,100,-;first",
            "garbage line",
            "3,2,200,-;second err",
            " continuation",
            "7,3,300,-;third",
        ]);
        let mut reader = KmsgReader::new(src);
        let recs = reader.drain();
        assert_eq!(recs.len(), 3);
        assert_eq!(recs[0].message, "first");
        assert_eq!(recs[1].severity(), Severity::Error);
        assert_eq!(recs[2].sequence, 3);
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Emergency < Severity::Debug);
        assert_eq!(Severity::from_priority(0), Severity::Emergency);
        assert_eq!(Severity::Warning.code(), 4);
        assert_eq!(Severity::Critical.label(), "crit");
    }

    #[test]
    fn record_roundtrips_through_kmsg_line() {
        let rec = KmsgRecord::new(27, 5, 1_500_000, "daemon error");
        let line = rec.to_kmsg_line();
        assert_eq!(line, "27,5,1500000,-;daemon error");
        assert_eq!(KmsgRecord::parse(&line).unwrap(), rec);
        assert_eq!(rec.facility_name(), "daemon");
    }

    #[test]
    fn dmesg_line_formatting() {
        let rec = KmsgRecord::new(6, 1, 1_234_567, "hello");
        assert_eq!(rec.to_dmesg_line(), "[    1.234567] hello");
    }

    #[test]
    fn facility_names() {
        assert_eq!(KmsgRecord::new(0, 0, 0, "x").facility_name(), "kern");
        assert_eq!(KmsgRecord::new(8, 0, 0, "x").facility_name(), "user");
        assert_eq!(KmsgRecord::new(16 * 8, 0, 0, "x").facility_name(), "local");
    }

    #[test]
    fn drain_at_least_filters_by_severity() {
        let src = MemoryKmsg::from_lines([
            "0,1,10,-;emerg",
            "4,2,20,-;warn",
            "6,3,30,-;info",
            "3,4,40,-;err",
            "7,5,50,-;debug",
        ]);
        let mut reader = KmsgReader::new(src);
        let recs = reader.drain_at_least(Severity::Warning);
        // keep emerg(0), warn(4), err(3); drop info(6), debug(7)
        let msgs: Vec<_> = recs.iter().map(|r| r.message.as_str()).collect();
        assert_eq!(msgs, ["emerg", "warn", "err"]);
    }

    #[test]
    fn drain_since_sequence() {
        let src = MemoryKmsg::from_lines(["6,10,10,-;a", "6,11,20,-;b", "6,12,30,-;c"]);
        let mut reader = KmsgReader::new(src);
        let recs = reader.drain_since(10);
        let seqs: Vec<_> = recs.iter().map(|r| r.sequence).collect();
        assert_eq!(seqs, [11, 12]);
    }

    #[test]
    fn write_kmsg_blob_roundtrips() {
        let recs = vec![
            KmsgRecord::new(6, 1, 100, "first"),
            KmsgRecord::new(3, 2, 200, "second"),
        ];
        let blob = write_kmsg(&recs);
        let src = MemoryKmsg::from_lines(blob.lines());
        let mut reader = KmsgReader::new(src);
        assert_eq!(reader.drain(), recs);
    }
}
