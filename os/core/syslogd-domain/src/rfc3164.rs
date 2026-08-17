//! RFC3164 (BSD) syslog parser.
//!
//! Format (loosely):
//!
//! ```text
//! <PRI>TIMESTAMP HOSTNAME TAG[PID]: MESSAGE
//! ```
//!
//! RFC3164 is notoriously under-specified, so this parser is deliberately
//! lenient — exactly like the implementation in `internal/app/syslogd`. The
//! TIMESTAMP is the classic `Mmm dd hh:mm:ss` form (15 chars). Any field that
//! cannot be recognized is folded into the message body rather than rejected.

use crate::parser::{Format, SyslogMessage, split_pri};
use os_kernel::{Error, Result};

/// The fixed width of an RFC3164 `Mmm dd hh:mm:ss` timestamp.
const TIMESTAMP_LEN: usize = 15;

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Parse a single RFC3164 line into a [`SyslogMessage`].
///
/// Returns an error only when the input is empty after trimming the trailing
/// newline; everything else is parsed best-effort.
pub fn parse(input: &str) -> Result<SyslogMessage> {
    let line = input.trim_end_matches(['\n', '\r', '\0']);
    if line.is_empty() {
        return Err(Error::parse("empty syslog line"));
    }

    let (priority, rest) = split_pri(line);

    // Try to peel a classic timestamp off the front.
    let (timestamp, rest) = take_timestamp(rest);

    // Hostname: only present when we actually had a timestamp (BSD ordering).
    let (hostname, rest) = if timestamp.is_some() {
        take_hostname(rest)
    } else {
        (None, rest)
    };

    // Tag and optional pid: "tag[pid]:" or "tag:".
    let (tag, pid, message) = take_tag(rest);

    Ok(SyslogMessage {
        format: Format::Rfc3164,
        priority,
        timestamp,
        hostname,
        tag,
        pid,
        message: message.to_string(),
    })
}

/// Recognize a leading `Mmm dd hh:mm:ss` timestamp. On success returns the
/// 15-char timestamp and the remainder with one separating space consumed.
fn take_timestamp(input: &str) -> (Option<String>, &str) {
    if input.len() < TIMESTAMP_LEN {
        return (None, input);
    }
    let candidate = &input[..TIMESTAMP_LEN];
    if !looks_like_timestamp(candidate) {
        return (None, input);
    }
    let rest = input[TIMESTAMP_LEN..].trim_start_matches(' ');
    (Some(candidate.to_string()), rest)
}

/// Heuristic timestamp validator for the BSD `Mmm dd hh:mm:ss` form.
fn looks_like_timestamp(s: &str) -> bool {
    if s.len() != TIMESTAMP_LEN {
        return false;
    }
    let b = s.as_bytes();
    // Month abbreviation.
    let month = &s[0..3];
    if !MONTHS.contains(&month) {
        return false;
    }
    // s[3] space; s[4..6] day (space-padded); s[6] space; then hh:mm:ss.
    if b[3] != b' ' || b[6] != b' ' {
        return false;
    }
    let day = &s[4..6];
    if !day.bytes().all(|c| c == b' ' || c.is_ascii_digit()) {
        return false;
    }
    let time = &s[7..15];
    let tb = time.as_bytes();
    tb[2] == b':'
        && tb[5] == b':'
        && tb[0].is_ascii_digit()
        && tb[1].is_ascii_digit()
        && tb[3].is_ascii_digit()
        && tb[4].is_ascii_digit()
        && tb[6].is_ascii_digit()
        && tb[7].is_ascii_digit()
}

/// Take the hostname (up to the next space).
fn take_hostname(input: &str) -> (Option<String>, &str) {
    match input.split_once(' ') {
        Some((host, rest)) if !host.is_empty() => (Some(host.to_string()), rest),
        _ => (None, input),
    }
}

/// Split `tag[pid]: message` (or `tag: message`, or bare message).
fn take_tag(input: &str) -> (Option<String>, Option<String>, &str) {
    // The tag ends at the first ':' (followed optionally by a space). Per BSD,
    // the tag is alphanumeric and stops at the first non-alnum char, but
    // implementations commonly key off the colon. We key off the colon and
    // then extract an optional [pid].
    let colon = match input.find(':') {
        Some(i) => i,
        None => return (None, None, input),
    };
    let head = &input[..colon];
    // A tag should not contain spaces; if it does, treat the whole thing as a
    // message (no recognizable tag).
    if head.is_empty() || head.contains(' ') {
        return (None, None, input);
    }
    let message = input[colon + 1..].trim_start_matches(' ');

    // Extract optional "[pid]" suffix.
    if let Some(open) = head.find('[')
        && head.ends_with(']')
    {
        let tag = &head[..open];
        let pid = &head[open + 1..head.len() - 1];
        if !pid.is_empty() && pid.bytes().all(|c| c.is_ascii_digit()) {
            return (Some(tag.to_string()), Some(pid.to_string()), message);
        }
    }
    (Some(head.to_string()), None, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Facility, Severity};

    #[test]
    fn full_message() {
        let m = parse("<34>Oct 11 22:14:15 mymachine su[1234]: failed login").unwrap();
        assert_eq!(m.format, Format::Rfc3164);
        let p = m.priority.unwrap();
        assert_eq!(p.facility, Facility::Auth);
        assert_eq!(p.severity, Severity::Critical);
        assert_eq!(m.timestamp.as_deref(), Some("Oct 11 22:14:15"));
        assert_eq!(m.hostname.as_deref(), Some("mymachine"));
        assert_eq!(m.tag.as_deref(), Some("su"));
        assert_eq!(m.pid.as_deref(), Some("1234"));
        assert_eq!(m.message, "failed login");
    }

    #[test]
    fn no_pid() {
        let m = parse("<13>Jan  1 00:00:00 host crond: started").unwrap();
        assert_eq!(m.tag.as_deref(), Some("crond"));
        assert!(m.pid.is_none());
        assert_eq!(m.message, "started");
        // single-digit day is space padded.
        assert_eq!(m.timestamp.as_deref(), Some("Jan  1 00:00:00"));
    }

    #[test]
    fn no_pri_no_timestamp() {
        let m = parse("plain message with no header").unwrap();
        assert!(m.priority.is_none());
        assert!(m.timestamp.is_none());
        assert!(m.hostname.is_none());
        assert!(m.tag.is_none());
        assert_eq!(m.message, "plain message with no header");
    }

    #[test]
    fn pri_only_then_tag() {
        // PRI present but no timestamp; hostname is skipped, tag still parsed.
        let m = parse("<30>dockerd: container started").unwrap();
        assert_eq!(m.priority.unwrap().facility, Facility::Daemon);
        assert!(m.timestamp.is_none());
        assert!(m.hostname.is_none());
        assert_eq!(m.tag.as_deref(), Some("dockerd"));
        assert_eq!(m.message, "container started");
    }

    #[test]
    fn trailing_newline_trimmed() {
        let m = parse("<13>tag: hello\n").unwrap();
        assert_eq!(m.message, "hello");
    }

    #[test]
    fn empty_is_error() {
        assert!(parse("\n").is_err());
        assert!(parse("").is_err());
    }

    #[test]
    fn invalid_pid_kept_in_tag() {
        // [abc] is not a numeric pid, so it stays as part of the tag head.
        let m = parse("<13>weird[abc]: text").unwrap();
        assert_eq!(m.tag.as_deref(), Some("weird[abc]"));
        assert!(m.pid.is_none());
        assert_eq!(m.message, "text");
    }

    #[test]
    fn message_with_colon_but_spaced_head() {
        // Head before colon contains a space -> not a tag, whole is message.
        let m = parse("<13>this is: not a tag").unwrap();
        assert!(m.tag.is_none());
        assert_eq!(m.message, "this is: not a tag");
    }

    #[test]
    fn bad_timestamp_folds_into_body() {
        // "Foo 11 22:14:15" is not a valid month -> no timestamp recognized.
        let m = parse("<13>Foo 11 22:14:15 host tag: msg").unwrap();
        assert!(m.timestamp.is_none());
        // Without a timestamp, hostname is not separated; tag still found.
        assert_eq!(m.tag.as_deref(), None);
    }
}
