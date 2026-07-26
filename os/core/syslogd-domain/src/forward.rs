//! Forwarding sinks for parsed syslog messages.
//!
//! Talos's syslogd forwards every received message into the machine log stream
//! (and, historically, into `/dev/kmsg`). The OS boundary here is the
//! [`LogSink`] trait; tests use [`MemorySink`]. A [`KmsgSink`] formats messages
//! back into a canonical `<PRI>tag: message` line for the kernel ring buffer,
//! modeled by the [`KmsgWriter`] trait.

use crate::parser::SyslogMessage;
use os_kernel::Result;

/// A boundary that accepts fully-parsed syslog messages.
pub trait LogSink {
    /// Forward one parsed message. Implementations should be cheap/non-blocking.
    fn forward(&mut self, msg: &SyslogMessage) -> Result<()>;
}

/// In-memory sink used by tests and by the controller's default routing.
#[derive(Debug, Default)]
pub struct MemorySink {
    messages: Vec<SyslogMessage>,
}

impl MemorySink {
    /// A fresh empty sink.
    pub fn new() -> MemorySink {
        MemorySink {
            messages: Vec::new(),
        }
    }

    /// All messages forwarded so far, in order.
    pub fn messages(&self) -> &[SyslogMessage] {
        &self.messages
    }

    /// Number of messages forwarded.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Whether nothing has been forwarded.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

impl LogSink for MemorySink {
    fn forward(&mut self, msg: &SyslogMessage) -> Result<()> {
        self.messages.push(msg.clone());
        Ok(())
    }
}

/// The kernel-log boundary. Real builds wire this to `/dev/kmsg`.
pub trait KmsgWriter {
    /// Write one already-formatted kmsg line.
    fn write_line(&mut self, line: &str) -> Result<()>;
}

/// Records every line that would have been written to `/dev/kmsg`.
#[derive(Debug, Default)]
pub struct MemoryKmsg {
    lines: Vec<String>,
}

impl MemoryKmsg {
    pub fn new() -> MemoryKmsg {
        MemoryKmsg { lines: Vec::new() }
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}

impl KmsgWriter for MemoryKmsg {
    fn write_line(&mut self, line: &str) -> Result<()> {
        self.lines.push(line.to_string());
        Ok(())
    }
}

/// Render a parsed message back into a canonical syslog wire line, preserving
/// the PRI header so a downstream kmsg consumer keeps the severity.
pub fn format_line(msg: &SyslogMessage) -> String {
    let mut out = String::new();
    if let Some(p) = msg.priority {
        out.push('<');
        out.push_str(&p.raw().to_string());
        out.push('>');
    }
    if let Some(tag) = &msg.tag {
        out.push_str(tag);
        if let Some(pid) = &msg.pid {
            out.push('[');
            out.push_str(pid);
            out.push(']');
        }
        out.push_str(": ");
    }
    out.push_str(&msg.message);
    out
}

/// A [`LogSink`] adapter that re-serializes messages and writes them to a
/// [`KmsgWriter`]. This is how syslogd bridges userspace logs into the kernel
/// ring buffer.
pub struct KmsgSink<W: KmsgWriter> {
    writer: W,
}

impl<W: KmsgWriter> KmsgSink<W> {
    pub fn new(writer: W) -> KmsgSink<W> {
        KmsgSink { writer }
    }

    /// Borrow the underlying writer (e.g. to inspect captured lines in tests).
    pub fn writer(&self) -> &W {
        &self.writer
    }

    /// Consume the sink, returning the writer.
    pub fn into_writer(self) -> W {
        self.writer
    }
}

impl<W: KmsgWriter> LogSink for KmsgSink<W> {
    fn forward(&mut self, msg: &SyslogMessage) -> Result<()> {
        let line = format_line(msg);
        self.writer.write_line(&line)
    }
}

/// Fans a single message out to several sinks. Errors from one sink do not
/// prevent the others from receiving the message; the first error is returned
/// after all sinks have been attempted.
#[derive(Default)]
pub struct FanOut {
    sinks: Vec<Box<dyn LogSink>>,
}

impl FanOut {
    pub fn new() -> FanOut {
        FanOut { sinks: Vec::new() }
    }

    /// Add a sink to the fan-out.
    pub fn add(&mut self, sink: Box<dyn LogSink>) {
        self.sinks.push(sink);
    }

    /// Number of registered sinks.
    pub fn len(&self) -> usize {
        self.sinks.len()
    }

    /// Whether no sinks are registered.
    pub fn is_empty(&self) -> bool {
        self.sinks.is_empty()
    }
}

impl LogSink for FanOut {
    fn forward(&mut self, msg: &SyslogMessage) -> Result<()> {
        let mut first_err = None;
        for sink in &mut self.sinks {
            if let Err(e) = sink.forward(msg)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Facility, Format, Priority, Severity, SyslogMessage};

    fn sample() -> SyslogMessage {
        SyslogMessage {
            format: Format::Rfc3164,
            priority: Some(Priority::new(Facility::Auth, Severity::Critical)),
            timestamp: None,
            hostname: None,
            tag: Some("su".into()),
            pid: Some("1234".into()),
            message: "failed login".into(),
        }
    }

    #[test]
    fn memory_sink_collects() {
        let mut sink = MemorySink::new();
        assert!(sink.is_empty());
        sink.forward(&sample()).unwrap();
        assert_eq!(sink.len(), 1);
        assert_eq!(sink.messages()[0].message, "failed login");
    }

    #[test]
    fn format_line_roundtrip_pri() {
        let line = format_line(&sample());
        assert_eq!(line, "<34>su[1234]: failed login");
    }

    #[test]
    fn format_line_no_pri_no_tag() {
        let msg = SyslogMessage {
            format: Format::Rfc3164,
            priority: None,
            timestamp: None,
            hostname: None,
            tag: None,
            pid: None,
            message: "bare".into(),
        };
        assert_eq!(format_line(&msg), "bare");
    }

    #[test]
    fn kmsg_sink_writes_formatted() {
        let mut sink = KmsgSink::new(MemoryKmsg::new());
        sink.forward(&sample()).unwrap();
        assert_eq!(
            sink.writer().lines(),
            &["<34>su[1234]: failed login".to_string()]
        );
    }

    #[test]
    fn fanout_delivers_to_all() {
        let mut fan = FanOut::new();
        fan.add(Box::new(MemorySink::new()));
        fan.add(Box::new(KmsgSink::new(MemoryKmsg::new())));
        assert_eq!(fan.len(), 2);
        fan.forward(&sample()).unwrap();
    }
}
