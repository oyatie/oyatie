//! RFC5424 structured syslog parser.
//!
//! Format:
//!
//! ```text
//! <PRI>VERSION TIMESTAMP HOSTNAME APP-NAME PROCID MSGID [SD] MSG
//! ```
//!
//! Each header field is space-separated; the nil value is `-`. Structured data
//! (`[SD]`) is parsed into `(element-id, [(param, value)])` pairs. This mirrors
//! the RFC5424 path in `internal/app/syslogd`, which accepts messages from
//! modern logging clients.

use crate::parser::{Format, SyslogMessage, split_pri};
use os_kernel::{Error, Result};
use std::collections::BTreeMap;

/// A parsed RFC5424 message with its structured-data section preserved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rfc5424Message {
    /// The normalized core message.
    pub message: SyslogMessage,
    /// Protocol version (always `1` for RFC5424).
    pub version: u8,
    /// MSGID field, if present.
    pub msgid: Option<String>,
    /// Structured data: element id -> ordered params.
    pub structured_data: Vec<StructuredElement>,
}

/// A single structured-data element, e.g. `[exampleSDID@32473 iut="3"]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredElement {
    /// The SD-ID (e.g. `exampleSDID@32473`).
    pub id: String,
    /// Ordered parameter list (RFC5424 permits repeats, so this is a Vec).
    pub params: Vec<(String, String)>,
}

impl StructuredElement {
    /// Look up the first value for `name`.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// Collect params into a map (last value wins on duplicate keys).
    pub fn as_map(&self) -> BTreeMap<String, String> {
        self.params.iter().cloned().collect()
    }
}

fn nil(field: &str) -> Option<String> {
    if field == "-" {
        None
    } else {
        Some(field.to_string())
    }
}

/// Parse a single RFC5424 line.
pub fn parse(input: &str) -> Result<Rfc5424Message> {
    let line = input.trim_end_matches(['\n', '\r', '\0']);
    if line.is_empty() {
        return Err(Error::parse("empty syslog line"));
    }

    let (priority, rest) = split_pri(line);
    if priority.is_none() {
        return Err(Error::parse("rfc5424 requires a PRI header"));
    }

    // VERSION immediately follows PRI with no space.
    let (version_str, rest) = rest
        .split_once(' ')
        .ok_or_else(|| Error::parse("rfc5424 missing header fields"))?;
    let version: u8 = version_str
        .parse()
        .map_err(|_| Error::parse("rfc5424 invalid version"))?;
    if version == 0 {
        return Err(Error::parse("rfc5424 version must be >= 1"));
    }

    // TIMESTAMP HOSTNAME APP-NAME PROCID MSGID — five space-separated fields.
    let mut fields = rest.splitn(6, ' ');
    let timestamp = fields
        .next()
        .ok_or_else(|| Error::parse("missing timestamp"))?;
    let hostname = fields
        .next()
        .ok_or_else(|| Error::parse("missing hostname"))?;
    let app_name = fields
        .next()
        .ok_or_else(|| Error::parse("missing app-name"))?;
    let procid = fields
        .next()
        .ok_or_else(|| Error::parse("missing procid"))?;
    let msgid = fields.next().ok_or_else(|| Error::parse("missing msgid"))?;
    // Remainder: "[SD] MSG" or "- MSG" or just "[SD]".
    let remainder = fields.next().unwrap_or("");

    let (structured_data, message_text) = parse_sd_and_msg(remainder)?;

    let core = SyslogMessage {
        format: Format::Rfc5424,
        priority,
        timestamp: nil(timestamp),
        hostname: nil(hostname),
        tag: nil(app_name),
        pid: nil(procid),
        message: message_text,
    };

    Ok(Rfc5424Message {
        message: core,
        version,
        msgid: nil(msgid),
        structured_data,
    })
}

/// Parse the structured-data section followed by the free-form message.
fn parse_sd_and_msg(input: &str) -> Result<(Vec<StructuredElement>, String)> {
    if let Some(stripped) = input.strip_prefix('-') {
        // NILVALUE SD; the rest (after one optional space) is the message.
        return Ok((
            Vec::new(),
            stripped.strip_prefix(' ').unwrap_or(stripped).to_string(),
        ));
    }
    if !input.starts_with('[') {
        // No SD section at all; everything is the message.
        return Ok((Vec::new(), input.to_string()));
    }

    let bytes = input.as_bytes();
    let mut i = 0;
    let mut elements = Vec::new();
    while i < bytes.len() && bytes[i] == b'[' {
        let (elem, consumed) = parse_sd_element(&input[i..])?;
        elements.push(elem);
        i += consumed;
    }
    let msg = input[i..]
        .strip_prefix(' ')
        .unwrap_or(&input[i..])
        .to_string();
    Ok((elements, msg))
}

/// Parse a single `[id param="value" ...]` element, returning it and the number
/// of bytes consumed from `input` (which must start with `[`).
fn parse_sd_element(input: &str) -> Result<(StructuredElement, usize)> {
    let bytes = input.as_bytes();
    debug_assert_eq!(bytes[0], b'[');
    let mut i = 1;

    // SD-ID: up to the first space or ']'.
    let id_start = i;
    while i < bytes.len() && bytes[i] != b' ' && bytes[i] != b']' {
        i += 1;
    }
    if i >= bytes.len() {
        return Err(Error::parse("rfc5424 unterminated structured data"));
    }
    let id = input[id_start..i].to_string();
    if id.is_empty() {
        return Err(Error::parse("rfc5424 empty SD-ID"));
    }

    let mut params = Vec::new();
    loop {
        match bytes[i] {
            b']' => {
                i += 1; // consume ']'
                break;
            }
            b' ' => {
                i += 1;
                // Parse a `name="value"` pair.
                let name_start = i;
                while i < bytes.len() && bytes[i] != b'=' && bytes[i] != b']' {
                    i += 1;
                }
                if i >= bytes.len() || bytes[i] != b'=' {
                    return Err(Error::parse("rfc5424 malformed SD-PARAM"));
                }
                let name = input[name_start..i].to_string();
                i += 1; // consume '='
                if i >= bytes.len() || bytes[i] != b'"' {
                    return Err(Error::parse("rfc5424 SD-PARAM value not quoted"));
                }
                i += 1; // consume opening quote
                let mut value = String::new();
                loop {
                    if i >= bytes.len() {
                        return Err(Error::parse("rfc5424 unterminated SD-PARAM value"));
                    }
                    match bytes[i] {
                        b'\\' if i + 1 < bytes.len() => {
                            // Escaped char (\", \\, \]).
                            value.push(bytes[i + 1] as char);
                            i += 2;
                        }
                        b'"' => {
                            i += 1; // consume closing quote
                            break;
                        }
                        c => {
                            value.push(c as char);
                            i += 1;
                        }
                    }
                }
                params.push((name, value));
            }
            _ => return Err(Error::parse("rfc5424 malformed structured data")),
        }
    }

    Ok((StructuredElement { id, params }, i))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Facility, Severity};

    const RFC_EXAMPLE: &str = r#"<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - 'su root' failed for lonvick on /dev/pts/8"#;

    #[test]
    fn rfc5424_canonical_example() {
        let m = parse(RFC_EXAMPLE).unwrap();
        assert_eq!(m.version, 1);
        let p = m.message.priority.unwrap();
        assert_eq!(p.facility, Facility::Auth);
        assert_eq!(p.severity, Severity::Critical);
        assert_eq!(
            m.message.timestamp.as_deref(),
            Some("2003-10-11T22:14:15.003Z")
        );
        assert_eq!(m.message.hostname.as_deref(), Some("mymachine.example.com"));
        assert_eq!(m.message.tag.as_deref(), Some("su"));
        assert!(m.message.pid.is_none()); // PROCID was '-'
        assert_eq!(m.msgid.as_deref(), Some("ID47"));
        assert!(m.structured_data.is_empty());
        assert_eq!(
            m.message.message,
            "'su root' failed for lonvick on /dev/pts/8"
        );
    }

    #[test]
    fn structured_data_parsed() {
        let line = r#"<165>1 2003-10-11T22:14:15.003Z host evntslog 8710 ID47 [exampleSDID@32473 iut="3" eventSource="Application"] an event occurred"#;
        let m = parse(line).unwrap();
        assert_eq!(m.message.pid.as_deref(), Some("8710"));
        assert_eq!(m.structured_data.len(), 1);
        let sd = &m.structured_data[0];
        assert_eq!(sd.id, "exampleSDID@32473");
        assert_eq!(sd.get("iut"), Some("3"));
        assert_eq!(sd.get("eventSource"), Some("Application"));
        assert_eq!(m.message.message, "an event occurred");
    }

    #[test]
    fn multiple_sd_elements() {
        let line = r#"<165>1 - - - - - [a x="1"][b@1 y="2"] body"#;
        let m = parse(line).unwrap();
        assert_eq!(m.structured_data.len(), 2);
        assert_eq!(m.structured_data[0].id, "a");
        assert_eq!(m.structured_data[1].id, "b@1");
        assert_eq!(m.structured_data[1].get("y"), Some("2"));
        assert_eq!(m.message.message, "body");
    }

    #[test]
    fn all_nil_fields() {
        let m = parse("<13>1 - - - - - -").unwrap();
        assert!(m.message.timestamp.is_none());
        assert!(m.message.hostname.is_none());
        assert!(m.message.tag.is_none());
        assert!(m.message.pid.is_none());
        assert!(m.msgid.is_none());
        assert!(m.structured_data.is_empty());
        assert_eq!(m.message.message, "");
    }

    #[test]
    fn escaped_quote_in_value() {
        let line = r#"<13>1 - - - - - [id k="a\"b"] msg"#;
        let m = parse(line).unwrap();
        assert_eq!(m.structured_data[0].get("k"), Some(r#"a"b"#));
    }

    #[test]
    fn requires_pri() {
        assert!(parse("1 - - - - - msg").is_err());
    }

    #[test]
    fn rejects_version_zero() {
        assert!(parse("<13>0 - - - - - msg").is_err());
    }

    #[test]
    fn as_map_works() {
        let line = r#"<13>1 - - - - - [id a="1" b="2"] x"#;
        let m = parse(line).unwrap();
        let map = m.structured_data[0].as_map();
        assert_eq!(map.get("a").map(String::as_str), Some("1"));
        assert_eq!(map.get("b").map(String::as_str), Some("2"));
    }

    #[test]
    fn unterminated_sd_is_error() {
        assert!(parse(r#"<13>1 - - - - - [id k="v"#).is_err());
    }
}
