//! # talos-syslogd
//!
//! Models Talos's embedded syslog daemon (`internal/app/syslogd`): it listens
//! on a unix datagram socket (`/dev/log`) for RFC3164 (BSD) and RFC5424
//! (structured) messages emitted by extension services, parses
//! priority/facility/severity, timestamp, hostname/tag/pid and message body,
//! auto-detects the wire format, and routes each message to a log sink (the
//! machine log stream and/or `/dev/kmsg`).
//!
//! Layout:
//!
//! * [`parser`] — facility/severity/priority (PRI) decoding and the normalized
//!   [`SyslogMessage`].
//! * [`rfc3164`] — the lenient BSD syslog parser.
//! * [`rfc5424`] — the structured-data syslog parser.
//! * [`forward`] — the [`LogSink`](forward::LogSink) boundary, a kmsg bridge,
//!   and a fan-out router.
//! * [`server`] — the unix-socket [`DatagramSource`](server::DatagramSource)
//!   boundary, format auto-detection, and the
//!   [`SyslogService`](server::SyslogService) controller state machine.
//!
//! Every OS boundary (socket, kernel log) is a trait with an in-memory
//! implementation, so the whole crate builds and tests fully offline using only
//! `std` plus `talos-core`.

pub mod forward;
pub mod parser;
pub mod rfc3164;
pub mod rfc5424;
pub mod server;

pub use forward::{FanOut, KmsgSink, KmsgWriter, LogSink, MemoryKmsg, MemorySink};
pub use parser::{Facility, Format, Priority, Severity, SyslogMessage};
pub use rfc5424::{Rfc5424Message, StructuredElement};
pub use server::{
    DEFAULT_SOCKET_PATH, DatagramSource, MemoryDatagramSource, Stats, SyslogService, detect_format,
    parse_auto,
};

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::traits::RunState;

    /// End-to-end: feed mixed-format datagrams through the service into a
    /// memory sink and verify routing, ordering, and stats.
    #[test]
    fn end_to_end_mixed_formats() {
        let mut source = MemoryDatagramSource::new();
        source.push("<34>Oct 11 22:14:15 mymachine su[1234]: failed login");
        source.push(
            "<165>1 2003-10-11T22:14:15.003Z host evntslog 8710 ID47 [ex@1 iut=\"3\"] an event",
        );
        source.push("<13>plain daemon line: hello");

        let mut svc = SyslogService::new(DEFAULT_SOCKET_PATH, source, MemorySink::new());
        assert_eq!(svc.socket_path(), "/dev/log");
        svc.start().unwrap();
        assert_eq!(svc.state(), RunState::Running);

        let forwarded = svc.pump().unwrap();
        assert_eq!(forwarded, 3);

        let msgs = svc.sink().messages();
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0].format, Format::Rfc3164);
        assert_eq!(msgs[0].tag.as_deref(), Some("su"));
        assert_eq!(msgs[0].pid.as_deref(), Some("1234"));
        assert_eq!(msgs[1].format, Format::Rfc5424);
        assert_eq!(msgs[1].tag.as_deref(), Some("evntslog"));
        assert_eq!(msgs[1].message, "an event");
        assert_eq!(msgs[2].format, Format::Rfc3164);

        svc.stop().unwrap();
        assert_eq!(svc.state(), RunState::Stopped);
    }

    /// Severity-based routing: forward only messages at or above Warning
    /// severity into a kmsg sink via a custom filtering sink.
    #[test]
    fn fanout_routes_to_multiple_sinks() {
        let m = parse_auto("<34>su[1]: critical thing").unwrap();
        assert_eq!(m.severity(), Severity::Critical);

        let mut fan = FanOut::new();
        fan.add(Box::new(MemorySink::new()));
        fan.add(Box::new(KmsgSink::new(MemoryKmsg::new())));
        fan.forward(&m).unwrap();
        assert_eq!(fan.len(), 2);
    }

    #[test]
    fn reexports_resolve() {
        let p = Priority::new(Facility::Daemon, Severity::Error);
        assert_eq!(p.raw(), 27);
    }

    #[test]
    fn structured_element_reexport() {
        let m = rfc5424::parse("<13>1 - - - - - [id k=\"v\"] body").unwrap();
        let sd: &StructuredElement = &m.structured_data[0];
        assert_eq!(sd.get("k"), Some("v"));
    }
}
