use mail_kernel::Message;
use mail_parser::{MimeHeaders, PartType};
use serde_json::{Value, json};

pub(super) fn valid(raw: &[u8]) -> bool {
    let mut seen = false;
    for line in raw.split(|b| *b == b'\n') {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        if line.is_empty() {
            break;
        }
        if matches!(line.first(), Some(b' ' | b'\t')) {
            if !seen {
                return false;
            }
            continue;
        }
        let Some(colon) = line.iter().position(|b| *b == b':') else {
            return false;
        };
        if colon == 0 || !line[..colon].iter().all(|b| (33..=126).contains(b)) {
            return false;
        }
        seen = true;
    }
    seen && mail_parser::MessageParser::default().parse(raw).is_some()
}

fn part(message: &mail_parser::Message<'_>, id: &str, index: u32, args: &Value) -> Value {
    let p = &message.parts[index as usize];
    let typ = p
        .content_type()
        .map(|t| format!("{}/{}", t.c_type, t.c_subtype.as_deref().unwrap_or("plain")))
        .unwrap_or_else(|| "text/plain".into());
    let parts = match &p.body {
        PartType::Multipart(ids) => Some(
            ids.iter()
                .map(|index| part(message, id, *index, args))
                .collect::<Vec<_>>(),
        ),
        _ => None,
    };
    let mut value = json!({"partId":index.to_string(),"blobId":format!("{id}.{index}"),"type":typ,"size":p.contents().len(),"name":p.attachment_name(),
        "charset":p.content_type().and_then(|t|t.attribute("charset")),"disposition":p.content_disposition().map(|d|d.c_type.as_ref()),
        "cid":p.content_id(),"language":p.content_language().as_text_list(),"location":p.content_location(),"subParts":parts});
    if let Some(properties) = args["bodyProperties"].as_array() {
        value
            .as_object_mut()
            .unwrap()
            .retain(|k, _| properties.iter().any(|v| v == k));
    }
    value
}

pub(super) fn email(
    message: &Message,
    raw: Option<&[u8]>,
    args: &Value,
    mut response_limit: usize,
) -> Result<Value, &'static str> {
    let keywords: serde_json::Map<_, _> = message
        .keywords
        .iter()
        .map(|k| (k.clone(), json!(true)))
        .collect();
    let mailboxes: serde_json::Map<_, _> = message
        .mailboxes
        .keys()
        .map(|id| (id.clone(), json!(true)))
        .collect();
    let mut result = json!({"id":message.id,"blobId":message.id,"threadId":message.thread_id(),"mailboxIds":mailboxes,"keywords":keywords,"size":message.size,
        "subject":"","receivedAt":mail_parser::DateTime::from_timestamp(message.received_at).to_rfc3339(),"hasAttachment":false,
        "textBody":[],"htmlBody":[],"attachments":[],"bodyStructure":null,"bodyValues":{},"preview":""});
    if let Some(parsed) = raw.and_then(|raw| mail_parser::MessageParser::default().parse(raw)) {
        topology(&parsed)?;
        super::headers::render(&parsed, args, &mut result, &mut response_limit)?;
        result["subject"] = json!(parsed.subject().unwrap_or(""));
        result["hasAttachment"] = json!(parsed.attachment_count() > 0);
        result["preview"] = json!(parsed.body_preview(256).unwrap_or_default());
        result["sentAt"] = json!(parsed.date().map(|d| d.to_timezone(0).to_rfc3339()));
        for (property, ids) in [
            ("textBody", &parsed.text_body),
            ("htmlBody", &parsed.html_body),
            ("attachments", &parsed.attachments),
        ] {
            if !requested(args, property) {
                continue;
            }
            result[property] = json!(
                ids.iter()
                    .map(|index| part(&parsed, &message.id, *index, args))
                    .collect::<Vec<_>>()
            );
        }
        if requested(args, "bodyStructure") {
            result["bodyStructure"] = part(&parsed, &message.id, 0, args);
        }
        for (index, p) in parsed.parts.iter().enumerate() {
            if requested(args, "bodyValues")
                && (args["fetchAllBodyValues"] == true
                    || args["fetchTextBodyValues"] == true
                        && parsed.text_body.contains(&(index as u32))
                    || args["fetchHTMLBodyValues"] == true
                        && parsed.html_body.contains(&(index as u32)))
                && let Some(text) = p.text_contents()
            {
                let max = args["maxBodyValueBytes"]
                    .as_u64()
                    .unwrap_or(u64::MAX)
                    .min(usize::MAX as u64) as usize;
                let mut end = max.min(text.len());
                while !text.is_char_boundary(end) {
                    end -= 1;
                }
                result["bodyValues"][index.to_string()] = json!({"value":&text[..end],"isTruncated":end<text.len(),"isEncodingProblem":p.is_encoding_problem});
            }
        }
    }
    Ok(result)
}

pub(super) fn blob(
    service: &mail_service::MailService,
    token: &str,
    account: &str,
    id: &str,
) -> Result<Vec<u8>, mail_kernel::Error> {
    use mail_kernel::Error;
    let mut path = id.split('.');
    let root = path.next().ok_or(Error::NotFound)?;
    let indices = path
        .map(|part| part.parse::<usize>().map_err(|_| Error::NotFound))
        .take(33)
        .collect::<Result<Vec<_>, _>>()?;
    // Bound work for caller-composed nested IDs. Each component names a decoded
    // part within the previous blob, using the same parser as Email/parse.
    if indices.len() > 32 {
        return Err(Error::NotFound);
    }
    let mut raw = service.download(token, account, root)?;
    for index in indices {
        let parsed = mail_parser::MessageParser::default()
            .parse(&raw)
            .ok_or(Error::NotFound)?;
        raw = parsed
            .parts
            .get(index)
            .ok_or(Error::NotFound)?
            .contents()
            .to_vec();
    }
    Ok(raw)
}

fn topology(message: &mail_parser::Message<'_>) -> Result<(), &'static str> {
    if message.parts.len() > 1000 {
        return Err("tooLarge");
    }
    let mut pending = vec![(0usize, 0usize)];
    while let Some((index, depth)) = pending.pop() {
        if depth > 32 {
            return Err("tooLarge");
        }
        if let PartType::Multipart(children) = &message.parts.get(index).ok_or("tooLarge")?.body {
            pending.extend(children.iter().map(|index| (*index as usize, depth + 1)));
        }
    }
    Ok(())
}

pub(super) fn requested(args: &Value, name: &str) -> bool {
    args["properties"]
        .as_array()
        .is_none_or(|properties| properties.iter().any(|p| p == name))
}
