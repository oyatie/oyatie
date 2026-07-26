//! Service log capture and sinks.
//!
//! Mirrors the Talos logging machinery in
//! `internal/app/machined/pkg/system/runner` and `pkg/machinery/api/common`:
//! each supervised service has its log output retained in a per-service
//! [`CircularBuffer`], and lines can additionally be forwarded to one or more
//! sinks (the local console/file or a remote syslog/JSON endpoint).
//!
//! The remote forwarding boundary is modeled as the [`LogSink`] trait with an
//! in-memory implementation for tests.

use crate::circular_buffer::CircularBuffer;
use std::collections::HashMap;
use std::fmt::{self, Write as _};
use os_kernel::error::{Error, Result};

/// Default per-service retained log capacity in bytes (Talos default is 64 KiB).
pub const DEFAULT_SERVICE_LOG_BYTES: usize = 64 * 1024;

/// On-the-wire encoding of a forwarded log line.
///
/// Mirrors Talos `pkg/machinery/config` logging `format` plus the encoders in
/// `internal/app/machined/pkg/system/runner/logging`. Talos supports the
/// `json_lines` format and a `fluentd`-style envelope in addition to the raw
/// line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Forward the line verbatim (newline framed).
    Raw,
    /// Emit one JSON object per line: `{"msg":...,"talos-service":...}`.
    JsonLines,
    /// Emit a fluentd-style 2-element array: `["<tag>",{"msg":...}]`.
    Fluentd,
}

impl LogFormat {
    /// Parse from the Talos config string (`""`/`"raw"`, `"json_lines"`,
    /// `"fluentd"`).
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "" | "raw" => Ok(LogFormat::Raw),
            "json_lines" | "json" => Ok(LogFormat::JsonLines),
            "fluentd" => Ok(LogFormat::Fluentd),
            other => Err(Error::parse(format!("unknown log format '{other}'"))),
        }
    }

    /// The canonical config string.
    pub fn as_str(self) -> &'static str {
        match self {
            LogFormat::Raw => "raw",
            LogFormat::JsonLines => "json_lines",
            LogFormat::Fluentd => "fluentd",
        }
    }

    /// Encode a `(service, line)` pair into the bytes that would be written to
    /// the destination, including the trailing newline.
    pub fn encode(self, service: &str, line: &str) -> Vec<u8> {
        let line = line.trim_end_matches('\n');
        let s = match self {
            LogFormat::Raw => format!("{line}\n"),
            LogFormat::JsonLines => format!(
                "{{\"msg\":{},\"talos-service\":{}}}\n",
                json_string(line),
                json_string(service),
            ),
            LogFormat::Fluentd => format!(
                "[{},{{\"msg\":{}}}]\n",
                json_string(&format!("talos.{service}")),
                json_string(line),
            ),
        };
        s.into_bytes()
    }
}

/// Minimal JSON string escaper (no external crate). Quotes and escapes a string.
pub(crate) fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Transport scheme of a log destination URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogScheme {
    /// A local file path (`file://`).
    File,
    /// A raw TCP stream (`tcp://`).
    Tcp,
    /// A UDP datagram endpoint (`udp://`).
    Udp,
}

impl LogScheme {
    /// The scheme string.
    pub fn as_str(self) -> &'static str {
        match self {
            LogScheme::File => "file",
            LogScheme::Tcp => "tcp",
            LogScheme::Udp => "udp",
        }
    }
}

/// A parsed Talos logging destination.
///
/// Mirrors `machine.logging.destinations[]` in the machine config: a URL plus a
/// format and an optional set of `extraTags`. Talos accepts `tcp://host:port`,
/// `udp://host:port` and `file:///path` endpoints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogDestination {
    /// Transport scheme.
    pub scheme: LogScheme,
    /// Host:port for tcp/udp, or filesystem path for file.
    pub endpoint: String,
    /// Wire encoding.
    pub format: LogFormat,
    /// Static tags injected into every record (json/fluentd only), sorted.
    pub extra_tags: Vec<(String, String)>,
}

impl LogDestination {
    /// Parse a destination URL like `tcp://1.2.3.4:5044` or `file:///var/log/x`.
    /// The `format` defaults to [`LogFormat::JsonLines`] for tcp/udp and
    /// [`LogFormat::Raw`] for file when not overridden.
    pub fn parse(url: &str) -> Result<Self> {
        let url = url.trim();
        let (scheme_str, rest) = url
            .split_once("://")
            .ok_or_else(|| Error::parse(format!("log destination '{url}' missing scheme")))?;
        let scheme = match scheme_str.to_ascii_lowercase().as_str() {
            "tcp" => LogScheme::Tcp,
            "udp" => LogScheme::Udp,
            "file" => LogScheme::File,
            other => {
                return Err(Error::parse(format!(
                    "unsupported log destination scheme '{other}'"
                )));
            }
        };
        let endpoint = rest.trim();
        if endpoint.is_empty() {
            return Err(Error::parse("log destination has empty endpoint"));
        }
        if matches!(scheme, LogScheme::Tcp | LogScheme::Udp) && !endpoint.contains(':') {
            return Err(Error::parse(format!(
                "{} destination '{endpoint}' must be host:port",
                scheme.as_str()
            )));
        }
        let format = match scheme {
            LogScheme::File => LogFormat::Raw,
            _ => LogFormat::JsonLines,
        };
        Ok(LogDestination {
            scheme,
            endpoint: endpoint.to_string(),
            format,
            extra_tags: Vec::new(),
        })
    }

    /// Builder: override the wire format.
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Builder: add a static tag, keeping `extra_tags` sorted and deduplicated
    /// by key.
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let key = key.into();
        self.extra_tags.retain(|(k, _)| k != &key);
        self.extra_tags.push((key, value.into()));
        self.extra_tags.sort_by(|a, b| a.0.cmp(&b.0));
        self
    }

    /// Reconstruct the canonical URL string.
    pub fn url(&self) -> String {
        format!("{}://{}", self.scheme.as_str(), self.endpoint)
    }
}

impl fmt::Display for LogDestination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}]", self.url(), self.format.as_str())
    }
}

/// A destination that receives forwarded log lines.
///
/// In production this would write to a file, the console, or a remote syslog /
/// JSON-over-TCP collector. We keep the boundary minimal: write a single
/// already-framed log line for a given service.
pub trait LogSink {
    /// Forward one log line tagged with its originating service.
    fn write_line(&mut self, service: &str, line: &str) -> Result<()>;

    /// Flush any buffered output. Default is a no-op.
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

/// An in-memory [`LogSink`] that records everything written, for tests and for
/// the local-console path.
#[derive(Debug, Default, Clone)]
pub struct MemorySink {
    /// Captured `(service, line)` pairs in write order.
    pub lines: Vec<(String, String)>,
    /// Whether the sink is "broken" and should fail writes (to exercise error
    /// handling).
    fail: bool,
}

impl MemorySink {
    /// Create an empty sink.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a sink whose writes always fail.
    pub fn failing() -> Self {
        MemorySink {
            lines: Vec::new(),
            fail: true,
        }
    }

    /// Number of captured lines.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Whether the sink captured nothing.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// All captured lines for one service, in order.
    pub fn for_service(&self, service: &str) -> Vec<&str> {
        self.lines
            .iter()
            .filter(|(s, _)| s == service)
            .map(|(_, l)| l.as_str())
            .collect()
    }
}

impl LogSink for MemorySink {
    fn write_line(&mut self, service: &str, line: &str) -> Result<()> {
        if self.fail {
            return Err(Error::Other("sink write failed".into()));
        }
        self.lines.push((service.to_string(), line.to_string()));
        Ok(())
    }
}

/// A byte-oriented transport boundary. In production this is a TCP/UDP socket or
/// an open file; tests use [`MemoryWriter`].
pub trait ByteWriter {
    /// Write the raw, already-encoded bytes (including any framing newline).
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<()>;
    /// Flush buffered bytes. Default no-op.
    fn flush_bytes(&mut self) -> Result<()> {
        Ok(())
    }
}

/// In-memory [`ByteWriter`] accumulating everything written.
#[derive(Debug, Default, Clone)]
pub struct MemoryWriter {
    /// All bytes written, in order.
    pub buf: Vec<u8>,
    /// Number of `flush_bytes` calls.
    pub flushes: usize,
    fail_after: Option<usize>,
    writes: usize,
}

impl MemoryWriter {
    /// Create an empty writer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a writer that fails on the write after `n` successful writes.
    pub fn failing_after(n: usize) -> Self {
        MemoryWriter {
            fail_after: Some(n),
            ..Default::default()
        }
    }

    /// Decode the accumulated bytes into UTF-8 lines.
    pub fn lines(&self) -> Vec<String> {
        String::from_utf8_lossy(&self.buf)
            .lines()
            .map(ToString::to_string)
            .collect()
    }
}

impl ByteWriter for MemoryWriter {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        if let Some(n) = self.fail_after
            && self.writes >= n {
                return Err(Error::Other("byte writer broken".into()));
            }
        self.writes += 1;
        self.buf.extend_from_slice(bytes);
        Ok(())
    }

    fn flush_bytes(&mut self) -> Result<()> {
        self.flushes += 1;
        Ok(())
    }
}

/// A [`LogSink`] that encodes each line with a [`LogFormat`] and writes it to a
/// [`ByteWriter`] transport, prepending any `extra_tags` for structured
/// formats. This is the workhorse behind the file/tcp/fluentd/json Talos sinks.
#[derive(Debug, Clone)]
pub struct FormattingSink<W: ByteWriter> {
    writer: W,
    format: LogFormat,
    extra_tags: Vec<(String, String)>,
    written: usize,
}

impl<W: ByteWriter> FormattingSink<W> {
    /// Wrap a writer with the given format.
    pub fn new(writer: W, format: LogFormat) -> Self {
        FormattingSink {
            writer,
            format,
            extra_tags: Vec::new(),
            written: 0,
        }
    }

    /// Build a sink directly from a [`LogDestination`]'s format and tags.
    pub fn for_destination(writer: W, dest: &LogDestination) -> Self {
        FormattingSink {
            writer,
            format: dest.format,
            extra_tags: dest.extra_tags.clone(),
            written: 0,
        }
    }

    /// Number of lines written.
    pub fn written(&self) -> usize {
        self.written
    }

    /// Borrow the underlying writer.
    pub fn writer(&self) -> &W {
        &self.writer
    }

    fn encode(&self, service: &str, line: &str) -> Vec<u8> {
        if self.extra_tags.is_empty() || self.format == LogFormat::Raw {
            return self.format.encode(service, line);
        }
        // Inject extra tags into the JSON/fluentd object body.
        let line = line.trim_end_matches('\n');
        let mut tags = String::new();
        for (k, v) in &self.extra_tags {
            tags.push(',');
            tags.push_str(&json_string(k));
            tags.push(':');
            tags.push_str(&json_string(v));
        }
        let s = match self.format {
            LogFormat::JsonLines => format!(
                "{{\"msg\":{},\"talos-service\":{}{}}}\n",
                json_string(line),
                json_string(service),
                tags,
            ),
            LogFormat::Fluentd => format!(
                "[{},{{\"msg\":{}{}}}]\n",
                json_string(&format!("talos.{service}")),
                json_string(line),
                tags,
            ),
            LogFormat::Raw => unreachable!(),
        };
        s.into_bytes()
    }
}

impl<W: ByteWriter> LogSink for FormattingSink<W> {
    fn write_line(&mut self, service: &str, line: &str) -> Result<()> {
        let bytes = self.encode(service, line);
        self.writer.write_bytes(&bytes)?;
        self.written += 1;
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.writer.flush_bytes()
    }
}

/// A [`LogSink`] that fans every line out to several boxed sinks. A failure in
/// any one sink is recorded but does not stop the others; the first error is
/// returned after all sinks have been attempted (mirroring Talos best-effort
/// multi-destination forwarding).
#[derive(Default)]
pub struct MultiSink {
    sinks: Vec<Box<dyn LogSink>>,
}

impl MultiSink {
    /// Create an empty fan-out.
    pub fn new() -> Self {
        MultiSink { sinks: Vec::new() }
    }

    /// Add a sink to the fan-out (builder style).
    pub fn with(mut self, sink: impl LogSink + 'static) -> Self {
        self.sinks.push(Box::new(sink));
        self
    }

    /// Number of attached sinks.
    pub fn len(&self) -> usize {
        self.sinks.len()
    }

    /// Whether no sinks are attached.
    pub fn is_empty(&self) -> bool {
        self.sinks.is_empty()
    }
}

impl LogSink for MultiSink {
    fn write_line(&mut self, service: &str, line: &str) -> Result<()> {
        let mut first_err = None;
        for sink in &mut self.sinks {
            if let Err(e) = sink.write_line(service, line)
                && first_err.is_none() {
                    first_err = Some(e);
                }
        }
        match first_err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    fn flush(&mut self) -> Result<()> {
        let mut first_err = None;
        for sink in &mut self.sinks {
            if let Err(e) = sink.flush()
                && first_err.is_none() {
                    first_err = Some(e);
                }
        }
        match first_err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}

/// Registry of per-service log buffers with optional fan-out to a sink.
///
/// This is the in-memory heart of the `Logs` API: services write their stdout
/// here, the most recent output is retained per service, and `read`/`tail`
/// serve it back to clients.
pub struct LogRegistry<S: LogSink = MemorySink> {
    buffers: HashMap<String, CircularBuffer>,
    capacity: usize,
    sink: Option<S>,
}

impl LogRegistry<MemorySink> {
    /// Create a registry with the default per-service capacity and no sink.
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_SERVICE_LOG_BYTES)
    }

    /// Create a registry with the given per-service capacity and no sink.
    pub fn with_capacity(capacity: usize) -> Self {
        LogRegistry {
            buffers: HashMap::new(),
            capacity: capacity.max(1),
            sink: None,
        }
    }
}

impl Default for LogRegistry<MemorySink> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: LogSink> LogRegistry<S> {
    /// Attach a sink that receives every forwarded line. Replaces any existing
    /// sink and returns the new registry (builder style).
    pub fn with_sink<T: LogSink>(self, sink: T) -> LogRegistry<T> {
        LogRegistry {
            buffers: self.buffers,
            capacity: self.capacity,
            sink: Some(sink),
        }
    }

    /// Number of services with retained logs.
    pub fn service_count(&self) -> usize {
        self.buffers.len()
    }

    /// Whether a service has any retained logs.
    pub fn has_service(&self, service: &str) -> bool {
        self.buffers.contains_key(service)
    }

    /// Names of all known services (unordered).
    pub fn services(&self) -> Vec<String> {
        self.buffers.keys().cloned().collect()
    }

    /// Append a complete log line for a service (a newline is added if absent),
    /// retaining it in the per-service buffer and forwarding to the sink.
    pub fn append_line(&mut self, service: &str, line: &str) -> Result<()> {
        let buf = self
            .buffers
            .entry(service.to_string())
            .or_insert_with(|| CircularBuffer::with_capacity(self.capacity));
        buf.write(line.as_bytes());
        if !line.ends_with('\n') {
            buf.write(b"\n");
        }
        if let Some(sink) = self.sink.as_mut() {
            sink.write_line(service, line.trim_end_matches('\n'))?;
        }
        Ok(())
    }

    /// Append raw bytes (which may contain several lines) for a service. Only
    /// the buffer is updated; sink forwarding happens at line granularity via
    /// [`Self::append_line`].
    pub fn append_bytes(&mut self, service: &str, data: &[u8]) {
        let buf = self
            .buffers
            .entry(service.to_string())
            .or_insert_with(|| CircularBuffer::with_capacity(self.capacity));
        buf.write(data);
    }

    /// Read all retained lines for a service, oldest-first.
    pub fn read(&self, service: &str) -> Result<Vec<String>> {
        self.buffers
            .get(service)
            .map(CircularBuffer::lines)
            .ok_or_else(|| Error::not_found(format!("no logs for service '{service}'")))
    }

    /// Read up to the last `n` retained lines for a service, oldest-first.
    pub fn tail(&self, service: &str, n: usize) -> Result<Vec<String>> {
        let mut lines = self.read(service)?;
        if lines.len() > n {
            lines.drain(0..lines.len() - n);
        }
        Ok(lines)
    }

    /// Total retained bytes for a service.
    pub fn retained_bytes(&self, service: &str) -> usize {
        self.buffers.get(service).map_or(0, CircularBuffer::len)
    }

    /// Drop a service's retained logs entirely.
    pub fn clear_service(&mut self, service: &str) -> bool {
        self.buffers.remove(service).is_some()
    }

    /// Borrow the attached sink, if any.
    pub fn sink(&self) -> Option<&S> {
        self.sink.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_and_read_per_service() {
        let mut reg = LogRegistry::new();
        reg.append_line("etcd", "starting etcd").unwrap();
        reg.append_line("etcd", "etcd is healthy").unwrap();
        reg.append_line("kubelet", "kubelet up").unwrap();
        assert_eq!(reg.service_count(), 2);
        assert_eq!(
            reg.read("etcd").unwrap(),
            ["starting etcd", "etcd is healthy"]
        );
        assert_eq!(reg.read("kubelet").unwrap(), ["kubelet up"]);
    }

    #[test]
    fn unknown_service_is_not_found() {
        let reg = LogRegistry::new();
        let err = reg.read("nope").unwrap_err();
        assert_eq!(err.kind(), "not_found");
    }

    #[test]
    fn tail_limits_lines() {
        let mut reg = LogRegistry::new();
        for i in 0..5 {
            reg.append_line("svc", &format!("line {i}")).unwrap();
        }
        let last2 = reg.tail("svc", 2).unwrap();
        assert_eq!(last2, ["line 3", "line 4"]);
    }

    #[test]
    fn sink_receives_forwarded_lines() {
        let mut reg = LogRegistry::new().with_sink(MemorySink::new());
        reg.append_line("etcd", "hello\n").unwrap();
        reg.append_line("etcd", "world").unwrap();
        let sink = reg.sink().unwrap();
        assert_eq!(sink.for_service("etcd"), ["hello", "world"]);
        assert_eq!(sink.len(), 2);
    }

    #[test]
    fn failing_sink_propagates_error() {
        let mut reg = LogRegistry::new().with_sink(MemorySink::failing());
        let err = reg.append_line("svc", "x").unwrap_err();
        assert_eq!(err.kind(), "other");
        // buffer still retained the line even though the sink failed.
        assert_eq!(reg.read("svc").unwrap(), ["x"]);
    }

    #[test]
    fn capacity_eviction_via_buffer() {
        let mut reg = LogRegistry::with_capacity(16);
        for i in 0..20 {
            reg.append_line("svc", &format!("{i:02}")).unwrap();
        }
        assert!(reg.retained_bytes("svc") <= 16);
        // oldest lines were evicted, newest retained.
        let lines = reg.read("svc").unwrap();
        assert!(lines.last().unwrap() == "19");
    }

    #[test]
    fn log_format_parse_roundtrip() {
        assert_eq!(LogFormat::parse("").unwrap(), LogFormat::Raw);
        assert_eq!(LogFormat::parse("raw").unwrap(), LogFormat::Raw);
        assert_eq!(
            LogFormat::parse("json_lines").unwrap(),
            LogFormat::JsonLines
        );
        assert_eq!(LogFormat::parse("FLUENTD").unwrap(), LogFormat::Fluentd);
        assert!(LogFormat::parse("xml").is_err());
        for f in [LogFormat::Raw, LogFormat::JsonLines, LogFormat::Fluentd] {
            assert_eq!(LogFormat::parse(f.as_str()).unwrap(), f);
        }
    }

    #[test]
    fn log_format_encodes_each_shape() {
        assert_eq!(LogFormat::Raw.encode("etcd", "hello\n"), b"hello\n");
        let j = LogFormat::JsonLines.encode("etcd", "hi");
        assert_eq!(
            String::from_utf8(j).unwrap(),
            "{\"msg\":\"hi\",\"talos-service\":\"etcd\"}\n"
        );
        let f = LogFormat::Fluentd.encode("etcd", "hi");
        assert_eq!(
            String::from_utf8(f).unwrap(),
            "[\"talos.etcd\",{\"msg\":\"hi\"}]\n"
        );
    }

    #[test]
    fn json_escaping() {
        let j = LogFormat::JsonLines.encode("svc", "a\"b\\c\n\tend");
        let s = String::from_utf8(j).unwrap();
        assert!(s.contains("a\\\"b\\\\c\\n\\tend"));
    }

    #[test]
    fn destination_parse_tcp_and_file() {
        let tcp = LogDestination::parse("tcp://10.0.0.1:5044").unwrap();
        assert_eq!(tcp.scheme, LogScheme::Tcp);
        assert_eq!(tcp.endpoint, "10.0.0.1:5044");
        assert_eq!(tcp.format, LogFormat::JsonLines);
        assert_eq!(tcp.url(), "tcp://10.0.0.1:5044");

        let file = LogDestination::parse("file:///var/log/talos.log").unwrap();
        assert_eq!(file.scheme, LogScheme::File);
        assert_eq!(file.format, LogFormat::Raw);
        assert_eq!(file.endpoint, "/var/log/talos.log");

        let udp = LogDestination::parse("udp://[::1]:514").unwrap();
        assert_eq!(udp.scheme, LogScheme::Udp);
    }

    #[test]
    fn destination_parse_rejects_bad() {
        assert!(LogDestination::parse("10.0.0.1:5044").is_err()); // no scheme
        assert!(LogDestination::parse("tcp://nohostport").is_err());
        assert!(LogDestination::parse("ftp://x:1").is_err());
        assert!(LogDestination::parse("tcp://").is_err());
    }

    #[test]
    fn destination_with_format_and_tags() {
        let d = LogDestination::parse("tcp://h:1")
            .unwrap()
            .with_format(LogFormat::Fluentd)
            .with_tag("cluster", "prod")
            .with_tag("zone", "a")
            .with_tag("cluster", "prod2"); // dedup by key
        assert_eq!(d.format, LogFormat::Fluentd);
        assert_eq!(d.extra_tags.len(), 2);
        // sorted by key
        assert_eq!(d.extra_tags[0].0, "cluster");
        assert_eq!(d.extra_tags[0].1, "prod2");
        assert_eq!(d.extra_tags[1].0, "zone");
        assert_eq!(d.to_string(), "tcp://h:1 [fluentd]");
    }

    #[test]
    fn formatting_sink_writes_encoded_bytes() {
        let dest = LogDestination::parse("tcp://h:1").unwrap();
        let mut sink = FormattingSink::for_destination(MemoryWriter::new(), &dest);
        sink.write_line("etcd", "boom").unwrap();
        sink.write_line("etcd", "again").unwrap();
        sink.flush().unwrap();
        assert_eq!(sink.written(), 2);
        let lines = sink.writer().lines();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"msg\":\"boom\""));
        assert_eq!(sink.writer().flushes, 1);
    }

    #[test]
    fn formatting_sink_injects_extra_tags() {
        let dest = LogDestination::parse("tcp://h:1")
            .unwrap()
            .with_tag("dc", "us-east");
        let mut sink = FormattingSink::for_destination(MemoryWriter::new(), &dest);
        sink.write_line("kubelet", "x").unwrap();
        let line = &sink.writer().lines()[0];
        assert!(line.contains("\"dc\":\"us-east\""));
        assert!(line.contains("\"talos-service\":\"kubelet\""));
    }

    #[test]
    fn formatting_sink_propagates_writer_failure() {
        let mut sink = FormattingSink::new(MemoryWriter::failing_after(1), LogFormat::Raw);
        assert!(sink.write_line("svc", "ok").is_ok());
        assert!(sink.write_line("svc", "boom").is_err());
        assert_eq!(sink.written(), 1);
    }

    #[test]
    fn multi_sink_fans_out_and_aggregates_errors() {
        let mut multi = MultiSink::new()
            .with(MemorySink::new())
            .with(MemorySink::failing())
            .with(MemorySink::new());
        assert_eq!(multi.len(), 3);
        // failing middle sink yields an error, but others still received the line.
        let err = multi.write_line("svc", "hi").unwrap_err();
        assert_eq!(err.kind(), "other");
        assert!(multi.flush().is_ok());
    }

    #[test]
    fn multi_sink_all_ok() {
        let mut multi = MultiSink::new()
            .with(MemorySink::new())
            .with(MemorySink::new());
        assert!(multi.write_line("svc", "hi").is_ok());
        assert!(!multi.is_empty());
    }

    #[test]
    fn registry_with_formatting_sink_end_to_end() {
        let dest = LogDestination::parse("tcp://collector:5044").unwrap();
        let sink = FormattingSink::for_destination(MemoryWriter::new(), &dest);
        let mut reg = LogRegistry::new().with_sink(sink);
        reg.append_line("etcd", "started\n").unwrap();
        reg.append_line("etcd", "ready").unwrap();
        // buffer retained both lines
        assert_eq!(reg.read("etcd").unwrap(), ["started", "ready"]);
        // sink encoded both as json
        let sink = reg.sink().unwrap();
        assert_eq!(sink.written(), 2);
        assert!(sink.writer().lines()[0].contains("\"msg\":\"started\""));
    }
}
