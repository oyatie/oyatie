use super::criteria::Criterion;
use mail_kernel::Message;

pub(super) fn matches(
    c: &Criterion,
    message: &Message,
    sequence: u32,
    uid: u32,
    saved: bool,
    parsed: Option<&mail_parser::Message<'_>>,
    account: &str,
) -> bool {
    match c {
        Criterion::All => true,
        Criterion::Never => false,
        Criterion::And(items) => items
            .iter()
            .all(|c| matches(c, message, sequence, uid, saved, parsed, account)),
        Criterion::Or(a, b) => {
            matches(a, message, sequence, uid, saved, parsed, account)
                || matches(b, message, sequence, uid, saved, parsed, account)
        }
        Criterion::Nor(items) => items
            .iter()
            .all(|c| !matches(c, message, sequence, uid, saved, parsed, account)),
        Criterion::Not(c) => !matches(c, message, sequence, uid, saved, parsed, account),
        Criterion::Flag(flag) => message
            .keywords
            .iter()
            .any(|k| k.eq_ignore_ascii_case(flag)),
        Criterion::Size { larger, value } => {
            if *larger {
                message.size > *value
            } else {
                message.size < *value
            }
        }
        Criterion::Date { sent, order, value } => {
            let date = if *sent {
                parsed.and_then(|m| m.date()).copied()
            } else {
                Some(mail_parser::DateTime::from_timestamp(message.received_at))
            };
            date.is_some_and(|d| {
                let cmp = (d.year, d.month, d.day).cmp(value);
                if *order == std::cmp::Ordering::Greater {
                    cmp != std::cmp::Ordering::Less
                } else {
                    cmp == *order
                }
            })
        }
        Criterion::Header(name, value) => parsed.is_some_and(|m| header_matches(m, name, value)),
        Criterion::Body { headers, value } => {
            value.is_empty() || parsed.is_some_and(|m| body_matches(m, *headers, value))
        }
        Criterion::Set {
            uid: by_uid,
            ranges,
        } => ranges.iter().any(|(a, b)| {
            let n = if *by_uid { uid } else { sequence };
            n >= *a && n <= *b
        }),
        Criterion::Saved => saved,
        Criterion::Modseq(value) => message.modseq > *value,
        Criterion::Identity { thread, value } => {
            let (kind, id) = if *thread {
                ("T", message.thread_identity())
            } else {
                ("M", message.email_identity())
            };
            *value == super::super::object_id(kind, account, id)
        }
    }
}

fn header_matches(message: &mail_parser::Message<'_>, name: &str, value: &str) -> bool {
    message.parts[0].headers.iter().any(|h| {
        h.name.as_str().eq_ignore_ascii_case(name) && header_value_matches(message, h, value)
    })
}

fn header_value_matches(
    message: &mail_parser::Message<'_>,
    header: &mail_parser::Header<'_>,
    value: &str,
) -> bool {
    message
        .raw_message
        .get(header.offset_start as usize..header.offset_end as usize)
        .is_some_and(|raw| {
            mail_parser::parsers::MessageStream::new(raw)
                .parse_unstructured()
                .as_text()
                .is_some_and(|text| text.to_lowercase().contains(value))
        })
}

fn body_matches(message: &mail_parser::Message<'_>, headers: bool, value: &str) -> bool {
    let mut pending = vec![message];
    while let Some(message) = pending.pop() {
        if headers
            && message.parts[0]
                .headers
                .iter()
                .any(|h| header_value_matches(message, h, value))
        {
            return true;
        }
        for part in &message.parts {
            match &part.body {
                mail_parser::PartType::Text(text) | mail_parser::PartType::Html(text)
                    if text.to_lowercase().contains(value) =>
                {
                    return true;
                }
                mail_parser::PartType::Message(nested) => pending.push(nested),
                _ => {}
            }
        }
    }
    false
}
