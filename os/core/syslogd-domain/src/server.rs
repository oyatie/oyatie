//! The syslog service: unix-socket ingestion, format auto-detection, routing,
//! and a service controller state machine.
//!
//! Talos's syslogd listens on a unix datagram socket (`/dev/log`), auto-detects
//! whether each datagram is RFC3164 or RFC5424, parses it, and forwards it to
//! the log sink. The socket boundary is modeled by the [`DatagramSource`]
//! trait; tests feed datagrams through [`MemoryDatagramSource`].

use crate::forward::LogSink;
use crate::parser::{Format, SyslogMessage, split_pri};
use crate::{rfc3164, rfc5424};
use os_kernel::traits::RunState;
use os_kernel::{Error, Result};

/// Default unix socket path syslogd binds, mirroring Talos.
pub const DEFAULT_SOCKET_PATH: &str = "/dev/log";

/// A source of inbound syslog datagrams (one message per datagram).
pub trait DatagramSource {
    /// Receive the next datagram, or `Ok(None)` when the source is drained.
    fn recv(&mut self) -> Result<Option<Vec<u8>>>;
}

/// In-memory datagram source backed by a queue of byte payloads.
#[derive(Debug, Default)]
pub struct MemoryDatagramSource {
    queue: std::collections::VecDeque<Vec<u8>>,
}

impl MemoryDatagramSource {
    pub fn new() -> MemoryDatagramSource {
        MemoryDatagramSource {
            queue: std::collections::VecDeque::new(),
        }
    }

    /// Enqueue a datagram payload.
    pub fn push(&mut self, payload: impl Into<Vec<u8>>) {
        self.queue.push_back(payload.into());
    }

    /// Number of pending datagrams.
    pub fn pending(&self) -> usize {
        self.queue.len()
    }
}

impl DatagramSource for MemoryDatagramSource {
    fn recv(&mut self) -> Result<Option<Vec<u8>>> {
        Ok(self.queue.pop_front())
    }
}

/// Decide which wire format a datagram uses.
///
/// RFC5424 always has a `<PRI>VERSION ` prefix where VERSION is a non-zero
/// digit run immediately after the `>`; RFC3164 never has a bare version token
/// there. This mirrors the sniffing logic in `internal/app/syslogd`.
pub fn detect_format(input: &str) -> Format {
    let line = input.trim_end_matches(['\n', '\r', '\0']);
    let (pri, rest) = split_pri(line);
    if pri.is_none() {
        return Format::Rfc3164;
    }
    // After the PRI, RFC5424 has "VERSION SP". Look for digits then a space.
    let version: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if version.is_empty() {
        return Format::Rfc3164;
    }
    // Must be followed by a space and parse to a non-zero version <= 9
    // (RFC5424 version is a single nonzero digit in practice).
    let after = &rest[version.len()..];
    match (version.parse::<u8>(), after.starts_with(' ')) {
        (Ok(v), true) if v >= 1 => Format::Rfc5424,
        _ => Format::Rfc3164,
    }
}

/// Parse a datagram, auto-detecting the format, into a normalized message.
pub fn parse_auto(input: &str) -> Result<SyslogMessage> {
    match detect_format(input) {
        Format::Rfc5424 => rfc5424::parse(input).map(|m| m.message),
        Format::Rfc3164 => rfc3164::parse(input),
    }
}

/// Per-service routing/processing statistics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    /// Total datagrams pulled from the source.
    pub received: u64,
    /// Datagrams successfully parsed and forwarded.
    pub forwarded: u64,
    /// Datagrams that failed to parse (e.g. empty/garbage).
    pub parse_errors: u64,
    /// Datagrams that parsed but a sink rejected.
    pub forward_errors: u64,
    /// Datagrams dropped because the payload was not valid UTF-8.
    pub invalid_utf8: u64,
}

/// The syslog service controller. Owns a [`DatagramSource`] and a [`LogSink`],
/// runs a small lifecycle state machine, and pumps messages source -> sink.
pub struct SyslogService<S: DatagramSource, K: LogSink> {
    socket_path: String,
    source: S,
    sink: K,
    state: RunState,
    stats: Stats,
}

impl<S: DatagramSource, K: LogSink> SyslogService<S, K> {
    /// Build a service bound (logically) to `socket_path`.
    pub fn new(socket_path: impl Into<String>, source: S, sink: K) -> Self {
        SyslogService {
            socket_path: socket_path.into(),
            source,
            sink,
            state: RunState::Initialized,
            stats: Stats::default(),
        }
    }

    /// The socket path this service listens on.
    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }

    /// Current lifecycle state.
    pub fn state(&self) -> RunState {
        self.state
    }

    /// Snapshot of routing statistics.
    pub fn stats(&self) -> Stats {
        self.stats
    }

    /// Borrow the sink (e.g. to inspect forwarded messages in tests).
    pub fn sink(&self) -> &K {
        &self.sink
    }

    fn transition(&mut self, next: RunState) -> Result<()> {
        if self.state == next {
            return Ok(());
        }
        if !self.state.can_transition_to(next) {
            return Err(Error::invalid_state(format!(
                "syslogd cannot transition {:?} -> {:?}",
                self.state, next
            )));
        }
        self.state = next;
        Ok(())
    }

    /// Move the service to `Running`, going through `Preparing` (e.g. binding
    /// the socket). Idempotent if already running.
    pub fn start(&mut self) -> Result<()> {
        if self.state == RunState::Running {
            return Ok(());
        }
        self.transition(RunState::Preparing)?;
        self.transition(RunState::Running)?;
        Ok(())
    }

    /// Cleanly stop the service. Idempotent if already stopped.
    pub fn stop(&mut self) -> Result<()> {
        if self.state == RunState::Stopped {
            return Ok(());
        }
        self.transition(RunState::Stopped)
    }

    /// Process a single datagram already pulled from the source. Updates stats
    /// and forwards on success.
    pub fn process(&mut self, payload: &[u8]) -> Result<()> {
        self.stats.received += 1;
        let text = match std::str::from_utf8(payload) {
            Ok(t) => t,
            Err(_) => {
                self.stats.invalid_utf8 += 1;
                return Err(Error::parse("syslog datagram is not valid UTF-8"));
            }
        };
        match parse_auto(text) {
            Ok(msg) => match self.sink.forward(&msg) {
                Ok(()) => {
                    self.stats.forwarded += 1;
                    Ok(())
                }
                Err(e) => {
                    self.stats.forward_errors += 1;
                    Err(e)
                }
            },
            Err(e) => {
                self.stats.parse_errors += 1;
                Err(e)
            }
        }
    }

    /// Drain the source, processing every available datagram. Errors on
    /// individual datagrams are counted in [`Stats`] but do not abort the pump;
    /// returns the number of datagrams successfully forwarded.
    pub fn pump(&mut self) -> Result<u64> {
        if self.state != RunState::Running {
            return Err(Error::invalid_state("syslogd must be Running to pump"));
        }
        let mut forwarded = 0u64;
        while let Some(payload) = self.source.recv()? {
            if self.process(&payload).is_ok() {
                forwarded += 1;
            }
        }
        Ok(forwarded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forward::MemorySink;

    fn service() -> SyslogService<MemoryDatagramSource, MemorySink> {
        SyslogService::new(
            DEFAULT_SOCKET_PATH,
            MemoryDatagramSource::new(),
            MemorySink::new(),
        )
    }

    #[test]
    fn detect_3164_vs_5424() {
        assert_eq!(
            detect_format("<34>1 2003-10-11T22:14:15Z h a - - - msg"),
            Format::Rfc5424
        );
        assert_eq!(
            detect_format("<34>Oct 11 22:14:15 host su: msg"),
            Format::Rfc3164
        );
        assert_eq!(detect_format("no pri at all"), Format::Rfc3164);
        // <13>0 ... is invalid version -> treated as 3164
        assert_eq!(detect_format("<13>0 - - - - - msg"), Format::Rfc3164);
    }

    #[test]
    fn parse_auto_dispatches() {
        let m = parse_auto("<34>1 2003-10-11T22:14:15Z h su - - - hi").unwrap();
        assert_eq!(m.format, Format::Rfc5424);
        let m2 = parse_auto("<34>Oct 11 22:14:15 host su: hi").unwrap();
        assert_eq!(m2.format, Format::Rfc3164);
    }

    #[test]
    fn lifecycle_start_stop() {
        let mut svc = service();
        assert_eq!(svc.state(), RunState::Initialized);
        svc.start().unwrap();
        assert_eq!(svc.state(), RunState::Running);
        // idempotent start
        svc.start().unwrap();
        assert_eq!(svc.state(), RunState::Running);
        svc.stop().unwrap();
        assert_eq!(svc.state(), RunState::Stopped);
        // idempotent stop
        svc.stop().unwrap();
        assert_eq!(svc.state(), RunState::Stopped);
    }

    #[test]
    fn pump_requires_running() {
        let mut svc = service();
        assert!(svc.pump().is_err());
    }

    #[test]
    fn pump_processes_and_counts() {
        let mut source = MemoryDatagramSource::new();
        source.push("<34>Oct 11 22:14:15 host su[1]: a".as_bytes().to_vec());
        source.push("<165>1 - - app 22 - - structured".as_bytes().to_vec());
        source.push(vec![0xff, 0xfe]); // invalid utf8
        source.push("".as_bytes().to_vec()); // empty -> parse error
        let mut svc = SyslogService::new(DEFAULT_SOCKET_PATH, source, MemorySink::new());
        svc.start().unwrap();
        let forwarded = svc.pump().unwrap();
        assert_eq!(forwarded, 2);
        let s = svc.stats();
        assert_eq!(s.received, 4);
        assert_eq!(s.forwarded, 2);
        assert_eq!(s.invalid_utf8, 1);
        assert_eq!(s.parse_errors, 1);
        assert_eq!(svc.sink().len(), 2);
    }

    #[test]
    fn process_forwards_to_sink() {
        let mut svc = service();
        svc.start().unwrap();
        svc.process("<13>tag: hello".as_bytes()).unwrap();
        assert_eq!(svc.sink().messages()[0].message, "hello");
        assert_eq!(svc.stats().forwarded, 1);
    }

    #[test]
    fn invalid_state_transition_rejected() {
        let mut svc = service();
        // Initialized -> Stopped is not a legal direct transition.
        assert!(svc.stop().is_err());
    }

    #[test]
    fn memory_source_drains() {
        let mut src = MemoryDatagramSource::new();
        src.push(vec![1, 2, 3]);
        assert_eq!(src.pending(), 1);
        assert!(src.recv().unwrap().is_some());
        assert!(src.recv().unwrap().is_none());
    }
}
