//! Shared syslog primitives: facility, severity, priority (PRI) decoding and a
//! normalized parsed-message representation.
//!
//! Both the RFC3164 (BSD) and RFC5424 parsers produce a [`SyslogMessage`]. The
//! PRI value encodes facility and severity together as `facility * 8 +
//! severity`, exactly as in `internal/app/syslogd`.

use std::fmt;

/// Maximum valid PRI value. PRI = facility(0..=23) * 8 + severity(0..=7), so the
/// largest representable value is 23*8+7 = 191.
pub const MAX_PRI: u8 = 191;

/// Syslog facility (RFC5424 table 1). The numeric value is the facility code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Facility {
    Kernel,
    User,
    Mail,
    Daemon,
    Auth,
    Syslog,
    Lpr,
    News,
    Uucp,
    Cron,
    AuthPriv,
    Ftp,
    Ntp,
    LogAudit,
    LogAlert,
    Clock,
    Local0,
    Local1,
    Local2,
    Local3,
    Local4,
    Local5,
    Local6,
    Local7,
}

impl Facility {
    /// Decode a facility code (0..=23). Returns `None` if out of range.
    pub fn from_code(code: u8) -> Option<Facility> {
        use Facility::*;
        Some(match code {
            0 => Kernel,
            1 => User,
            2 => Mail,
            3 => Daemon,
            4 => Auth,
            5 => Syslog,
            6 => Lpr,
            7 => News,
            8 => Uucp,
            9 => Cron,
            10 => AuthPriv,
            11 => Ftp,
            12 => Ntp,
            13 => LogAudit,
            14 => LogAlert,
            15 => Clock,
            16 => Local0,
            17 => Local1,
            18 => Local2,
            19 => Local3,
            20 => Local4,
            21 => Local5,
            22 => Local6,
            23 => Local7,
            _ => return None,
        })
    }

    /// The numeric facility code.
    pub fn code(self) -> u8 {
        use Facility::*;
        match self {
            Kernel => 0,
            User => 1,
            Mail => 2,
            Daemon => 3,
            Auth => 4,
            Syslog => 5,
            Lpr => 6,
            News => 7,
            Uucp => 8,
            Cron => 9,
            AuthPriv => 10,
            Ftp => 11,
            Ntp => 12,
            LogAudit => 13,
            LogAlert => 14,
            Clock => 15,
            Local0 => 16,
            Local1 => 17,
            Local2 => 18,
            Local3 => 19,
            Local4 => 20,
            Local5 => 21,
            Local6 => 22,
            Local7 => 23,
        }
    }

    /// Lowercase keyword (as used by syslog config / journald).
    pub fn keyword(self) -> &'static str {
        use Facility::*;
        match self {
            Kernel => "kern",
            User => "user",
            Mail => "mail",
            Daemon => "daemon",
            Auth => "auth",
            Syslog => "syslog",
            Lpr => "lpr",
            News => "news",
            Uucp => "uucp",
            Cron => "cron",
            AuthPriv => "authpriv",
            Ftp => "ftp",
            Ntp => "ntp",
            LogAudit => "audit",
            LogAlert => "alert",
            Clock => "clock",
            Local0 => "local0",
            Local1 => "local1",
            Local2 => "local2",
            Local3 => "local3",
            Local4 => "local4",
            Local5 => "local5",
            Local6 => "local6",
            Local7 => "local7",
        }
    }
}

/// Syslog severity (RFC5424 table 2). Lower is more severe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Emergency = 0,
    Alert = 1,
    Critical = 2,
    Error = 3,
    Warning = 4,
    Notice = 5,
    Informational = 6,
    Debug = 7,
}

impl Severity {
    /// Decode a severity code (0..=7). Returns `None` if out of range.
    pub fn from_code(code: u8) -> Option<Severity> {
        use Severity::*;
        Some(match code {
            0 => Emergency,
            1 => Alert,
            2 => Critical,
            3 => Error,
            4 => Warning,
            5 => Notice,
            6 => Informational,
            7 => Debug,
            _ => return None,
        })
    }

    /// The numeric severity code.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// Lowercase keyword.
    pub fn keyword(self) -> &'static str {
        use Severity::*;
        match self {
            Emergency => "emerg",
            Alert => "alert",
            Critical => "crit",
            Error => "err",
            Warning => "warning",
            Notice => "notice",
            Informational => "info",
            Debug => "debug",
        }
    }
}

/// A decoded PRI value combining facility and severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Priority {
    pub facility: Facility,
    pub severity: Severity,
}

impl Priority {
    /// Build from an explicit facility/severity pair.
    pub fn new(facility: Facility, severity: Severity) -> Priority {
        Priority { facility, severity }
    }

    /// Decode the raw PRI integer `facility * 8 + severity`.
    pub fn from_raw(pri: u8) -> Option<Priority> {
        if pri > MAX_PRI {
            return None;
        }
        let facility = Facility::from_code(pri / 8)?;
        let severity = Severity::from_code(pri % 8)?;
        Some(Priority { facility, severity })
    }

    /// Re-encode to the raw PRI integer.
    pub fn raw(self) -> u8 {
        self.facility.code() * 8 + self.severity.code()
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.facility.keyword(), self.severity.keyword())
    }
}

/// Which wire format a message was parsed from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// RFC3164 (BSD) syslog.
    Rfc3164,
    /// RFC5424 structured syslog.
    Rfc5424,
}

/// A normalized parsed syslog message produced by either parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyslogMessage {
    /// The wire format the message was decoded from.
    pub format: Format,
    /// Decoded facility/severity. `None` if the wire had no PRI header.
    pub priority: Option<Priority>,
    /// Timestamp string exactly as found on the wire (not reparsed to epoch).
    pub timestamp: Option<String>,
    /// Hostname field, if present.
    pub hostname: Option<String>,
    /// Tag / application name (the program that emitted the log).
    pub tag: Option<String>,
    /// Process id, when the tag carried a `[pid]` suffix or RFC5424 PROCID.
    pub pid: Option<String>,
    /// Free-form message text.
    pub message: String,
}

impl SyslogMessage {
    /// Convenience accessor for the severity, defaulting to
    /// [`Severity::Notice`] when no PRI header was present (matching the
    /// common syslog default).
    pub fn severity(&self) -> Severity {
        self.priority
            .map(|p| p.severity)
            .unwrap_or(Severity::Notice)
    }

    /// Convenience accessor for the facility, defaulting to
    /// [`Facility::User`] when no PRI header was present.
    pub fn facility(&self) -> Facility {
        self.priority.map(|p| p.facility).unwrap_or(Facility::User)
    }
}

/// Split a leading `<NN>` PRI header off the front of `input`.
///
/// Returns `(priority, rest)` where `rest` is the remainder after the closing
/// `>`. If `input` does not begin with a well-formed `<digits>` header, returns
/// `(None, input)` unchanged.
pub(crate) fn split_pri(input: &str) -> (Option<Priority>, &str) {
    let bytes = input.as_bytes();
    if bytes.first() != Some(&b'<') {
        return (None, input);
    }
    // Find the closing '>'. PRI is at most 3 digits.
    let close = match input[1..].find('>') {
        Some(idx) => idx + 1,
        None => return (None, input),
    };
    let digits = &input[1..close];
    if digits.is_empty() || digits.len() > 3 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return (None, input);
    }
    match digits.parse::<u16>() {
        Ok(n) if n <= MAX_PRI as u16 => (Priority::from_raw(n as u8), &input[close + 1..]),
        _ => (None, input),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pri_roundtrip_all_valid() {
        for raw in 0..=MAX_PRI {
            let p = Priority::from_raw(raw).expect("valid pri");
            assert_eq!(p.raw(), raw);
        }
    }

    #[test]
    fn pri_out_of_range_rejected() {
        assert!(Priority::from_raw(192).is_none());
        assert!(Priority::from_raw(255).is_none());
    }

    #[test]
    fn pri_decode_known_values() {
        // <13> = facility 1 (user), severity 5 (notice)
        let p = Priority::from_raw(13).unwrap();
        assert_eq!(p.facility, Facility::User);
        assert_eq!(p.severity, Severity::Notice);
        // <0> = kernel emergency
        let p0 = Priority::from_raw(0).unwrap();
        assert_eq!(p0.facility, Facility::Kernel);
        assert_eq!(p0.severity, Severity::Emergency);
        // <191> = local7 debug
        let p191 = Priority::from_raw(191).unwrap();
        assert_eq!(p191.facility, Facility::Local7);
        assert_eq!(p191.severity, Severity::Debug);
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Emergency < Severity::Debug);
        assert!(Severity::Error < Severity::Warning);
    }

    #[test]
    fn priority_display() {
        let p = Priority::new(Facility::Daemon, Severity::Error);
        assert_eq!(p.to_string(), "daemon.err");
    }

    #[test]
    fn split_pri_basic() {
        let (p, rest) = split_pri("<13>rest of line");
        assert_eq!(p.unwrap().raw(), 13);
        assert_eq!(rest, "rest of line");
    }

    #[test]
    fn split_pri_no_header() {
        let (p, rest) = split_pri("no pri here");
        assert!(p.is_none());
        assert_eq!(rest, "no pri here");
    }

    #[test]
    fn split_pri_malformed() {
        // too many digits
        assert!(split_pri("<1234>x").0.is_none());
        // non-digit
        assert!(split_pri("<ab>x").0.is_none());
        // unterminated
        assert!(split_pri("<13 no close").0.is_none());
        // out of range
        assert!(split_pri("<255>x").0.is_none());
    }

    #[test]
    fn message_defaults() {
        let m = SyslogMessage {
            format: Format::Rfc3164,
            priority: None,
            timestamp: None,
            hostname: None,
            tag: None,
            pid: None,
            message: String::new(),
        };
        assert_eq!(m.severity(), Severity::Notice);
        assert_eq!(m.facility(), Facility::User);
    }
}
