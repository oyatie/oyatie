use super::super::response::Output;
use base64::{Engine, engine::general_purpose::STANDARD};
use mail_parser::{Addr, Address, HeaderName, Message};

pub(super) fn string(output: &mut Output, value: Option<&str>) {
    let Some(value) = value else {
        output.extend_from_slice(b"NIL");
        return;
    };
    output.extend_from_slice(b"\"");
    if value
        .bytes()
        .any(|b| b.is_ascii_control() || (!output.utf8 && !b.is_ascii()))
    {
        // Legacy IMAP uses RFC 2047 for Unicode header values. Every word stays
        // below 75 characters and splits only at a UTF-8 character boundary.
        let mut remaining = value;
        let mut first = true;
        while !remaining.is_empty() && !output.is_closed() {
            let mut end = remaining.len().min(45);
            while !remaining.is_char_boundary(end) {
                end -= 1;
            }
            if !first {
                output.extend_from_slice(b" ");
            }
            output.extend_from_slice(b"=?utf-8?B?");
            output.extend_from_slice(STANDARD.encode(&remaining.as_bytes()[..end]).as_bytes());
            output.extend_from_slice(b"?=");
            remaining = &remaining[end..];
            first = false;
        }
    } else {
        for byte in value.as_bytes() {
            if matches!(byte, b'\\' | b'\"') {
                output.extend_from_slice(b"\\");
            }
            output.extend_from_slice(std::slice::from_ref(byte));
        }
    }
    output.extend_from_slice(b"\"");
}

pub(super) fn write(message: &Message<'_>, output: &mut Output) {
    output.extend_from_slice(b"(");
    let date = message.header_raw(HeaderName::Date).map(unfold);
    string(output, date.as_deref());
    output.extend_from_slice(b" ");
    string(output, message.subject());
    let from = message.from();
    for address in [
        from,
        message.sender().filter(|a| present(a)).or(from),
        message.reply_to().filter(|a| present(a)).or(from),
        message.to(),
        message.cc(),
        message.bcc(),
    ] {
        output.extend_from_slice(b" ");
        addresses(output, address);
    }
    for header in [HeaderName::InReplyTo, HeaderName::MessageId] {
        output.extend_from_slice(b" ");
        let value = message.header_raw(header).map(unfold);
        string(output, value.as_deref());
    }
    output.extend_from_slice(b")");
}

fn addresses(output: &mut Output, value: Option<&Address<'_>>) {
    let Some(value) = value.filter(|a| present(a)) else {
        output.extend_from_slice(b"NIL");
        return;
    };
    output.extend_from_slice(b"(");
    match value {
        Address::List(list) => {
            for address in list {
                addr(output, address);
            }
        }
        Address::Group(groups) => {
            for group in groups {
                if group.name.is_some() {
                    output.extend_from_slice(b"(NIL NIL ");
                    string(output, group.name.as_deref());
                    output.extend_from_slice(b" NIL)");
                }
                for address in &group.addresses {
                    addr(output, address);
                }
                if group.name.is_some() {
                    output.extend_from_slice(b"(NIL NIL NIL NIL)");
                }
            }
        }
    }
    output.extend_from_slice(b")");
}

fn present(address: &Address<'_>) -> bool {
    match address {
        Address::List(list) => !list.is_empty(),
        Address::Group(groups) => !groups.is_empty(),
    }
}

fn unfold(value: &str) -> String {
    value
        .replace("\r\n", "")
        .replace('\n', "")
        .replace('\t', " ")
        .trim()
        .to_owned()
}

fn addr(output: &mut Output, address: &Addr<'_>) {
    output.extend_from_slice(b"(");
    string(output, address.name.as_deref());
    output.extend_from_slice(b" NIL ");
    let (mailbox, host) = address
        .address
        .as_deref()
        .map(|a| {
            a.rsplit_once('@')
                .map_or((a, None), |(local, domain)| (local, Some(domain)))
        })
        .map_or((None, None), |(local, host)| (Some(local), host));
    string(output, mailbox);
    output.extend_from_slice(b" ");
    string(output, host);
    output.extend_from_slice(b")");
}
