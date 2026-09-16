use super::{
    super::response::Output,
    envelope::{self, string},
};
use mail_parser::{ContentType, Message, MessagePart, MimeHeaders, PartType};

pub(super) fn validate(message: &Message<'_>) -> Result<(), &'static str> {
    let mut pending = vec![(message, 0usize, 0usize)];
    let mut count = 0;
    while let Some((message, index, depth)) = pending.pop() {
        count += 1;
        if depth > 32 || count > 1000 {
            return Err("NO [LIMIT]");
        }
        match &message.parts.get(index).ok_or("NO")?.body {
            PartType::Multipart(children) => {
                pending.extend(children.iter().map(|i| (message, *i as usize, depth + 1)))
            }
            PartType::Message(nested) => pending.push((nested, 0, depth + 1)),
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn write(message: &Message<'_>, index: usize, extended: bool, output: &mut Output) {
    if output.is_closed() {
        return;
    }
    let part = &message.parts[index];
    let content = part.content_type();
    output.extend_from_slice(b"(");
    if let PartType::Multipart(children) = &part.body {
        for child in children {
            write(message, *child as usize, extended, output);
        }
        output.extend_from_slice(b" ");
        string(
            output,
            Some(
                content
                    .and_then(|c| c.c_subtype.as_deref())
                    .unwrap_or("mixed"),
            ),
        );
        if extended {
            output.extend_from_slice(b" ");
            params(output, content);
            extensions(output, part);
        }
    } else {
        let embedded = matches!(part.body, PartType::Message(_));
        let kind =
            content
                .map(|c| c.c_type.as_ref())
                .unwrap_or(if embedded { "message" } else { "text" });
        string(output, Some(kind));
        output.extend_from_slice(b" ");
        string(
            output,
            Some(
                content
                    .and_then(|c| c.c_subtype.as_deref())
                    .unwrap_or(if embedded { "rfc822" } else { "plain" }),
            ),
        );
        output.extend_from_slice(b" ");
        if kind.eq_ignore_ascii_case("text")
            && !content.is_some_and(|c| {
                c.attributes
                    .as_ref()
                    .is_some_and(|a| a.iter().any(|p| p.name.eq_ignore_ascii_case("charset")))
            })
        {
            text_params(output, content);
        } else {
            params(output, content);
        }
        output.extend_from_slice(b" ");
        let cid = part.content_id().map(|id| format!("<{id}>"));
        string(output, cid.as_deref());
        output.extend_from_slice(b" ");
        string(output, part.content_description());
        output.extend_from_slice(b" ");
        string(
            output,
            Some(part.content_transfer_encoding().unwrap_or("7BIT")),
        );
        let bytes = &message.raw_message[part.offset_body as usize..part.offset_end as usize];
        output.extend_from_slice(format!(" {}", bytes.len()).as_bytes());
        match &part.body {
            PartType::Message(nested) => {
                output.extend_from_slice(b" ");
                envelope::write(nested, output);
                output.extend_from_slice(b" ");
                write(nested, 0, extended, output);
                output.extend_from_slice(format!(" {}", lines(bytes)).as_bytes());
            }
            _ if kind.eq_ignore_ascii_case("text") => {
                output.extend_from_slice(format!(" {}", lines(bytes)).as_bytes())
            }
            _ => {}
        }
        if extended {
            // Content-MD5 is optional in IMAP; absent digest is represented by NIL.
            output.extend_from_slice(b" NIL");
            extensions(output, part);
        }
    }
    output.extend_from_slice(b")");
}

fn lines(bytes: &[u8]) -> usize {
    bytes.iter().filter(|b| **b == b'\n').count()
}

fn params(output: &mut Output, content: Option<&ContentType<'_>>) {
    let Some(attributes) = content
        .and_then(|c| c.attributes.as_ref())
        .filter(|a| !a.is_empty())
    else {
        output.extend_from_slice(b"NIL");
        return;
    };
    output.extend_from_slice(b"(");
    for (i, attribute) in attributes.iter().enumerate() {
        if i > 0 {
            output.extend_from_slice(b" ");
        }
        string(output, Some(&attribute.name));
        output.extend_from_slice(b" ");
        string(output, Some(&attribute.value));
    }
    output.extend_from_slice(b")");
}

fn text_params(output: &mut Output, content: Option<&ContentType<'_>>) {
    output.extend_from_slice(b"(\"charset\" \"us-ascii\"");
    for attribute in content
        .and_then(|c| c.attributes.as_ref())
        .into_iter()
        .flatten()
    {
        output.extend_from_slice(b" ");
        string(output, Some(&attribute.name));
        output.extend_from_slice(b" ");
        string(output, Some(&attribute.value));
    }
    output.extend_from_slice(b")");
}

fn extensions(output: &mut Output, part: &MessagePart<'_>) {
    output.extend_from_slice(b" ");
    if let Some(disposition) = part.content_disposition() {
        output.extend_from_slice(b"(");
        string(output, Some(&disposition.c_type));
        output.extend_from_slice(b" ");
        params(output, Some(disposition));
        output.extend_from_slice(b")");
    } else {
        output.extend_from_slice(b"NIL");
    }
    output.extend_from_slice(b" ");
    if let Some(languages) = part
        .content_language()
        .as_text_list()
        .filter(|l| !l.is_empty())
    {
        output.extend_from_slice(b"(");
        for (i, language) in languages.iter().enumerate() {
            if i > 0 {
                output.extend_from_slice(b" ");
            }
            string(output, Some(language));
        }
        output.extend_from_slice(b")");
    } else if let Some(language) = part.content_language().as_text() {
        string(output, Some(language));
    } else {
        output.extend_from_slice(b"NIL");
    }
    output.extend_from_slice(b" ");
    string(output, part.content_location());
}
