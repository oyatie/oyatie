use super::items::{Content, Section};
use mail_parser::{Message, MessagePart, MimeHeaders, PartType};
use std::borrow::Cow;

fn locate<'a>(
    mut message: &'a Message<'a>,
    path: &[usize],
) -> Option<(&'a Message<'a>, &'a MessagePart<'a>)> {
    let mut part = message.parts.first()?;
    for (depth, number) in path.iter().enumerate() {
        let nested = matches!(part.body, PartType::Message(_)) && depth > 0;
        if depth > 0
            && let PartType::Message(nested) = &part.body
        {
            message = nested;
            part = message.parts.first()?;
        }
        part = match &part.body {
            PartType::Multipart(children) => {
                message.parts.get(*children.get(number - 1)? as usize)?
            }
            _ if (depth == 0 || nested) && *number == 1 => part,
            _ => return None,
        };
    }
    Some((message, part))
}

pub(super) fn read<'a>(
    request: &Content,
    parsed: Option<&'a Message<'a>>,
    raw: &'a [u8],
) -> Result<Option<Cow<'a, [u8]>>, &'static str> {
    if matches!(request.section, Section::Whole) && !request.binary {
        return Ok(Some(partial(request, Cow::Borrowed(raw))));
    }
    let parsed = parsed.ok_or("NO")?;
    let Some((mut message, mut part)) = locate(parsed, &request.path) else {
        return Ok(None);
    };
    let value = if request.binary {
        if part.is_encoding_problem {
            return Err("NO [UNKNOWN-CTE]");
        }
        let start = if matches!(part.body, PartType::Multipart(_)) {
            part.offset_header
        } else {
            part.offset_body
        };
        let bytes = message
            .raw_message
            .get(start as usize..part.offset_end as usize)
            .ok_or("NO")?;
        match part
            .content_transfer_encoding()
            .unwrap_or("7bit")
            .to_ascii_lowercase()
            .as_str()
        {
            "7bit" | "8bit" | "binary" => Cow::Borrowed(bytes),
            "base64" => Cow::Owned(
                mail_parser::decoders::base64::base64_decode(bytes).ok_or("NO [UNKNOWN-CTE]")?,
            ),
            "quoted-printable" => Cow::Owned(
                mail_parser::decoders::quoted_printable::quoted_printable_decode(bytes)
                    .ok_or("NO [UNKNOWN-CTE]")?,
            ),
            _ => return Err("NO [UNKNOWN-CTE]"),
        }
    } else {
        if !matches!(request.section, Section::Content | Section::Mime)
            && let PartType::Message(nested) = &part.body
        {
            message = nested;
            part = message.parts.first().ok_or("NO")?;
        }
        let source = &message.raw_message;
        match &request.section {
            Section::Whole => Cow::Borrowed(raw),
            Section::Content | Section::Text => Cow::Borrowed(
                source
                    .get(part.offset_body as usize..part.offset_end as usize)
                    .ok_or("NO")?,
            ),
            Section::Header => Cow::Borrowed(
                source
                    .get(part.offset_header as usize..part.offset_body as usize)
                    .ok_or("NO")?,
            ),
            Section::Mime | Section::Fields { .. } => {
                let headers = source
                    .get(part.offset_header as usize..part.offset_body as usize)
                    .ok_or("NO")?;
                Cow::Owned(filter_headers(headers, &request.section))
            }
        }
    };
    Ok(Some(partial(request, value)))
}

fn partial<'a>(request: &Content, mut value: Cow<'a, [u8]>) -> Cow<'a, [u8]> {
    if let Some((offset, count)) = request.partial {
        let start = offset.min(value.len());
        let end = start.saturating_add(count).min(value.len());
        value = match value {
            Cow::Borrowed(bytes) => Cow::Borrowed(&bytes[start..end]),
            Cow::Owned(mut bytes) => {
                bytes.truncate(end);
                bytes.drain(..start);
                Cow::Owned(bytes)
            }
        };
    }
    value
}

fn filter_headers(raw: &[u8], section: &Section) -> Vec<u8> {
    let mut result = Vec::new();
    let mut keep = false;
    for line in raw.split_inclusive(|b| *b == b'\n') {
        if line == b"\r\n" || line == b"\n" {
            break;
        }
        if !line.starts_with(b" ") && !line.starts_with(b"\t") {
            keep = line.iter().position(|b| *b == b':').is_some_and(|colon| {
                let name = &line[..colon];
                match section {
                    Section::Mime => name
                        .get(..8)
                        .is_some_and(|v| v.eq_ignore_ascii_case(b"content-")),
                    Section::Fields { names, exclude } => {
                        names
                            .iter()
                            .any(|n| name.eq_ignore_ascii_case(n.as_bytes()))
                            != *exclude
                    }
                    _ => false,
                }
            });
        }
        if keep {
            result.extend_from_slice(line);
        }
    }
    result.extend_from_slice(b"\r\n");
    result
}
