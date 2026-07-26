//! Early kernel-log (`/dev/kmsg`) and console logging.
//!
//! Before any real logging stack exists, Talos' init writes diagnostics to the
//! kernel ring buffer via `/dev/kmsg`, which the kernel timestamps and exposes
//! through `dmesg`. Each `/dev/kmsg` line is prefixed with a syslog priority
//! `<N>` where `N = facility*8 + level`. Init uses the `LOG_KERN`-ish facility
//! but practically formats `<level>message`.
//!
//! This module formats those records purely (host-testable) and writes them
//! through a [`KmsgSink`] trait so the Linux binary can target the real
//! `/dev/kmsg` while tests capture into a buffer.

use std::fmt;

/// Syslog severity levels (RFC 5424), the subset init emits.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Level {
    Emergency = 0,
    Alert = 1,
    Critical = 2,
    Error = 3,
    Warning = 4,
    Notice = 5,
    Info = 6,
    Debug = 7,
}

impl Level {
    /// Short tag used in human-facing console lines.
    pub fn tag(self) -> &'static str {
        match self {
            Level::Emergency => "EMERG",
            Level::Alert => "ALERT",
            Level::Critical => "CRIT",
            Level::Error => "ERROR",
            Level::Warning => "WARN",
            Level::Notice => "NOTICE",
            Level::Info => "INFO",
            Level::Debug => "DEBUG",
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.tag())
    }
}

/// Syslog facility. The kernel uses facility 0 (`kern`); user-space daemons
/// commonly use 3 (`daemon`). Talos init writes via the `user`(1) facility.
pub const FACILITY_USER: u8 = 1;
pub const FACILITY_DAEMON: u8 = 3;

/// Compute the syslog priority value `facility*8 + level`.
pub fn priority(facility: u8, level: Level) -> u8 {
    facility * 8 + level as u8
}

/// Format a `/dev/kmsg` line: `<PRI>message\n`. The kernel parses the leading
/// `<N>` and strips it, recording the level. A trailing newline terminates the
/// record.
pub fn format_kmsg(facility: u8, level: Level, msg: &str) -> String {
    // /dev/kmsg records must be single-line; collapse embedded newlines.
    let sanitized = msg.replace('\n', " ");
    format!("<{}>{}\n", priority(facility, level), sanitized)
}

/// Format a human-facing console line: `[ LEVEL ] message`.
pub fn format_console(level: Level, msg: &str) -> String {
    format!("[ {:<6} ] {}", level.tag(), msg)
}

/// Sink for log records. The Linux binary writes to `/dev/kmsg`; tests use
/// [`BufferSink`].
pub trait KmsgSink {
    fn write_record(&mut self, record: &str);
}

/// A logger that fans a message out to kmsg (machine-readable) and optionally
/// the console (human-readable), respecting a minimum level threshold.
pub struct Logger {
    facility: u8,
    min_level: Level,
    quiet_console: bool,
}

impl Logger {
    /// Default init logger: user facility, log Info and above, console on.
    pub fn new() -> Self {
        Logger {
            facility: FACILITY_USER,
            min_level: Level::Info,
            quiet_console: false,
        }
    }

    /// Set the minimum level emitted (records below are dropped).
    pub fn with_min_level(mut self, level: Level) -> Self {
        self.min_level = level;
        self
    }

    /// Suppress console output (e.g. when `quiet` is on the kernel cmdline);
    /// kmsg still receives everything at/above threshold.
    pub fn quiet_console(mut self, quiet: bool) -> Self {
        self.quiet_console = quiet;
        self
    }

    /// Override the syslog facility.
    pub fn with_facility(mut self, facility: u8) -> Self {
        self.facility = facility;
        self
    }

    /// True if a record at `level` would be emitted.
    pub fn enabled(&self, level: Level) -> bool {
        level <= self.min_level
    }

    /// Emit a record to kmsg and (unless quiet) the console. Returns true if it
    /// was emitted, false if filtered by level.
    pub fn log(
        &self,
        level: Level,
        msg: &str,
        kmsg: &mut dyn KmsgSink,
        console: &mut dyn KmsgSink,
    ) -> bool {
        if !self.enabled(level) {
            return false;
        }
        kmsg.write_record(&format_kmsg(self.facility, level, msg));
        if !self.quiet_console {
            console.write_record(&format_console(level, msg));
        }
        true
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

/// In-memory [`KmsgSink`] capturing every record for assertions.
#[derive(Default)]
pub struct BufferSink {
    pub records: Vec<String>,
}

impl BufferSink {
    pub fn new() -> Self {
        Self::default()
    }
    /// True if any captured record contains `needle`.
    pub fn contains(&self, needle: &str) -> bool {
        self.records.iter().any(|r| r.contains(needle))
    }
}

impl KmsgSink for BufferSink {
    fn write_record(&mut self, record: &str) {
        self.records.push(record.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_computation() {
        // user.info = 1*8 + 6 = 14
        assert_eq!(priority(FACILITY_USER, Level::Info), 14);
        // daemon.err = 3*8 + 3 = 27
        assert_eq!(priority(FACILITY_DAEMON, Level::Error), 27);
        // kern.emerg = 0
        assert_eq!(priority(0, Level::Emergency), 0);
    }

    #[test]
    fn kmsg_format_has_priority_prefix_and_newline() {
        let line = format_kmsg(FACILITY_USER, Level::Warning, "disk slow");
        // user.warning = 12
        assert_eq!(line, "<12>disk slow\n");
    }

    #[test]
    fn kmsg_collapses_newlines() {
        let line = format_kmsg(FACILITY_USER, Level::Info, "a\nb\nc");
        assert_eq!(line, "<14>a b c\n");
        // Only the terminator newline remains.
        assert_eq!(line.matches('\n').count(), 1);
    }

    #[test]
    fn console_format_is_human_readable() {
        let line = format_console(Level::Error, "boom");
        assert_eq!(line, "[ ERROR  ] boom");
    }

    #[test]
    fn levels_order_by_severity() {
        assert!(Level::Emergency < Level::Error);
        assert!(Level::Error < Level::Info);
        assert!(Level::Info < Level::Debug);
    }

    #[test]
    fn logger_filters_below_min_level() {
        let log = Logger::new().with_min_level(Level::Warning);
        assert!(log.enabled(Level::Error));
        assert!(log.enabled(Level::Warning));
        assert!(!log.enabled(Level::Info));
        assert!(!log.enabled(Level::Debug));
    }

    #[test]
    fn logger_writes_to_both_sinks() {
        let log = Logger::new();
        let mut kmsg = BufferSink::new();
        let mut console = BufferSink::new();
        assert!(log.log(Level::Info, "hello", &mut kmsg, &mut console));
        assert!(kmsg.contains("<14>hello"));
        assert!(console.contains("hello"));
    }

    #[test]
    fn quiet_console_suppresses_console_only() {
        let log = Logger::new().quiet_console(true);
        let mut kmsg = BufferSink::new();
        let mut console = BufferSink::new();
        assert!(log.log(Level::Notice, "x", &mut kmsg, &mut console));
        assert_eq!(kmsg.records.len(), 1);
        assert_eq!(console.records.len(), 0);
    }

    #[test]
    fn filtered_record_writes_nothing() {
        let log = Logger::new().with_min_level(Level::Error);
        let mut kmsg = BufferSink::new();
        let mut console = BufferSink::new();
        assert!(!log.log(Level::Debug, "noise", &mut kmsg, &mut console));
        assert!(kmsg.records.is_empty());
        assert!(console.records.is_empty());
    }

    #[test]
    fn with_facility_changes_prefix() {
        let log = Logger::new().with_facility(FACILITY_DAEMON);
        let mut kmsg = BufferSink::new();
        let mut console = BufferSink::new();
        log.log(Level::Error, "x", &mut kmsg, &mut console);
        // daemon.err = 27
        assert!(kmsg.contains("<27>x"));
    }
}
