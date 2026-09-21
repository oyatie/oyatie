use mail_kernel::{Error, MAX_MESSAGE_BYTES, MAX_SUBMISSION_BYTES};
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
    // `mail-auth` writes one `h=` entry per occurrence of a signed name it
    // finds (0.13.3 `dkim/canonicalize.rs`), so a message repeating one grows
    // its own signature without bound and a message accepted at the ceiling
    // stops fitting once signed -- a `554` and a permanent bounce for mail
    // this server promised to deliver. Refused here, which is RFC-conformant:
    // RFC 5322 section 3.6 permits at most one `To`, `Cc`, `Subject` or
    // `Message-ID` and exactly one `From` and `Date`; RFC 2045 permits one
    // `MIME-Version` (section 4), one `Content-Type` (section 5) and one
    // `Content-Transfer-Encoding` (section 6) per entity. With the repeat
    // refused, `h=` is at most `SIGNED_HEADERS` twice over for every message
    // that reaches the signer, which is what `SIGNATURE_ALLOWANCE` measures.
    let mut signed = [false; mail_kernel::SIGNED_HEADERS.len()];
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
            if let Some(at) = mail_kernel::SIGNED_HEADERS
                .iter()
                .position(|covered| name.eq_ignore_ascii_case(covered.as_bytes()))
                && std::mem::replace(&mut signed[at], true)
            {
                return Err(Error::Invalid);
            }
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
    // Both submitters normalize here -- SMTP `submit` and JMAP `submit_email`,
    // which takes any stored blob and so covers IMAP APPEND too -- so this is
    // the one place that decides what may reach the outbound queue. A message
    // that only fits before it is signed is one the outbound path bounces.
    if output.len() > MAX_SUBMISSION_BYTES {
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

#[cfg(test)]
mod tests {
    use super::normalize;
    use mail_kernel::{Error, MAX_DATA_BYTES, SIGNED_HEADERS};

    const ADDRESS: &str = "alice@example.org";

    /// Every signed header once, `repeats` copies of `extra` after them, and
    /// a body padded until the whole message is `size`.
    fn submittable(extra: &str, repeats: usize, size: usize) -> Vec<u8> {
        let mut raw = Vec::with_capacity(size);
        for header in [
            "From: <alice@example.org>",
            "To: <bob@example.net>",
            "Cc: <carol@example.net>",
            "Subject: a realistic subject line about the reporting deadline",
            "Date: Mon, 21 Sep 2026 12:34:56 +0000",
            "Message-ID: <0123456789abcdef0123456789abcdef@example.org>",
            "MIME-Version: 1.0",
            "Content-Type: text/plain; charset=utf-8",
            "Content-Transfer-Encoding: 8bit",
        ] {
            raw.extend_from_slice(header.as_bytes());
            raw.extend_from_slice(b"\r\n");
        }
        for _ in 0..repeats {
            raw.extend_from_slice(extra.as_bytes());
            raw.extend_from_slice(b"\r\n");
        }
        raw.extend_from_slice(b"\r\n");
        let line = b"padding to the ceiling, sixty-four bytes of body per line...\r\n";
        // Room for the short final line kept back, so the fill never truncates
        // one and leaves a bare `\r` that would be refused for its own reason.
        while raw.len() + line.len() + 2 <= size {
            raw.extend_from_slice(line);
        }
        raw.resize(size - 2, b'x');
        raw.extend_from_slice(b"\r\n");
        raw
    }

    /// `mail-auth` writes one `h=` entry per occurrence of a signed name it
    /// finds, in the message's own spelling, so a message that repeats one
    /// grows its own signature without bound. At the ceiling, a hundred extra
    /// `Content-Transfer-Encoding` lines put the signed bytes past what
    /// outbound wire validation allows: both transports answer `554`, the
    /// queue writes a DSN and deletes the job, and the sender's own mail
    /// bounces. Refused here, where the same message without the repeats is
    /// still accepted -- so it is the repetition that is refused, not the size.
    #[test]
    fn a_ceiling_message_that_repeats_a_signed_header_is_refused() {
        let repeated = "Content-Transfer-Encoding: 8bit";
        normalize(&submittable(repeated, 0, MAX_DATA_BYTES), ADDRESS)
            .expect("a message at the ceiling is submittable");
        assert_eq!(
            normalize(&submittable(repeated, 100, MAX_DATA_BYTES), ADDRESS).err(),
            Some(Error::Invalid)
        );
    }

    /// The refusal covers every name the facade signs, not only the one that
    /// measurably overruns. RFC 5322 section 3.6 permits at most one `To`,
    /// `Cc`, `Subject` or `Message-ID` and exactly one `From` and `Date`; RFC
    /// 2045 permits one `MIME-Version` (section 4), one `Content-Type`
    /// (section 5) and one `Content-Transfer-Encoding` (section 6) per
    /// entity. Refusing the second is conformant, not a local restriction.
    #[test]
    fn every_signed_header_is_refused_a_second_time() {
        for name in SIGNED_HEADERS {
            let raw = submittable(&format!("{name}: repeated"), 1, 4096);
            assert_eq!(
                normalize(&raw, ADDRESS).err(),
                Some(Error::Invalid),
                "{name}"
            );
        }
    }
}
