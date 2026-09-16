use mail_kernel::{Error, MAX_MESSAGE_BYTES};
use mail_parser::{HeaderName, HeaderValue, MessageParser};

/// Normalize only authenticated submissions; inbound message bytes are retained.
pub(super) fn normalize(raw: &[u8], address: &str) -> Result<Vec<u8>, Error> {
    validate_author(raw, address)?;
    let end = raw
        .windows(4)
        .position(|v| v == b"\r\n\r\n")
        .ok_or(Error::Invalid)?
        + 2;
    let message = MessageParser::default()
        .parse_headers(raw)
        .ok_or(Error::Invalid)?;
    let mut output = Vec::with_capacity(raw.len() + 128);
    for name in [HeaderName::Date, HeaderName::MessageId] {
        let mut headers = message.headers().iter().filter(|h| h.name == name);
        if let Some(header) = headers.next() {
            let valid = match (&name, &header.value) {
                (HeaderName::Date, HeaderValue::DateTime(date)) => date.is_valid(),
                (HeaderName::MessageId, HeaderValue::Text(id)) => {
                    id.split_once('@')
                        .is_some_and(|(left, right)| !left.is_empty() && !right.is_empty())
                        && !id
                            .bytes()
                            .any(|b| b <= 32 || b >= 127 || matches!(b, b'<' | b'>'))
                }
                _ => false,
            };
            if !valid || headers.next().is_some() {
                return Err(Error::Invalid);
            }
        } else if name == HeaderName::Date {
            output.extend_from_slice(
                format!(
                    "Date: {}\r\n",
                    mail_builder::headers::date::Date::now().to_rfc822()
                )
                .as_bytes(),
            );
        } else {
            output.extend_from_slice(b"Message-ID: ");
            mail_builder::headers::message_id::generate_message_id_header(
                &mut output,
                address.rsplit_once('@').ok_or(Error::Invalid)?.1,
            );
            output.extend_from_slice(b"\r\n");
        }
    }
    let mut blind = false;
    let mut first = true;
    for line in raw[..end].split_inclusive(|b| *b == b'\n') {
        let field = line.strip_suffix(b"\r\n").ok_or(Error::Invalid)?;
        if field.iter().any(|b| (*b < 32 && *b != b'\t') || *b == 127) {
            return Err(Error::Invalid);
        }
        if field.starts_with(b" ") || field.starts_with(b"\t") {
            if first {
                return Err(Error::Invalid);
            }
        } else {
            let colon = field
                .iter()
                .position(|b| *b == b':')
                .ok_or(Error::Invalid)?;
            let name = &field[..colon];
            if name.is_empty() || !name.iter().all(|b| (33..=126).contains(b)) {
                return Err(Error::Invalid);
            }
            blind = name.eq_ignore_ascii_case(b"Bcc") || name.eq_ignore_ascii_case(b"Resent-Bcc");
        }
        if !blind {
            output.extend_from_slice(line);
        }
        first = false;
    }
    output.extend_from_slice(&raw[end..]);
    if !output.ends_with(b"\r\n") {
        output.extend_from_slice(b"\r\n");
    }
    if output.split_inclusive(|b| *b == b'\n').any(|line| {
        line.len() > 1000
            || !line.ends_with(b"\r\n")
            || line[..line.len().saturating_sub(2)].contains(&b'\r')
            || line.contains(&0)
    }) {
        return Err(Error::Invalid);
    }
    if output.len() > MAX_MESSAGE_BYTES {
        return Err(Error::OverQuota);
    }
    Ok(output)
}

fn validate_author(raw: &[u8], address: &str) -> Result<(), Error> {
    use mail_parser::{Address, HeaderName, HeaderValue, MessageParser};
    if raw.len() > mail_kernel::MAX_MESSAGE_BYTES {
        return Err(Error::OverQuota);
    }
    let message = MessageParser::default()
        .parse_headers(raw)
        .ok_or(Error::Invalid)?;
    for (name, required) in [(HeaderName::From, true), (HeaderName::Sender, false)] {
        let mut headers = message.headers().iter().filter(|h| h.name == name);
        match headers.next().map(|h| &h.value) {
            Some(HeaderValue::Address(Address::List(values))) if values.len() == 1 => {
                if !values[0]
                    .address
                    .as_ref()
                    .is_some_and(|a| a.eq_ignore_ascii_case(address))
                {
                    return Err(Error::Forbidden);
                }
            }
            None if !required => continue,
            _ => return Err(Error::Invalid),
        }
        if headers.next().is_some() {
            return Err(Error::Invalid);
        }
    }
    Ok(())
}
