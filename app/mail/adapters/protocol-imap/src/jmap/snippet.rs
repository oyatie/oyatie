use super::{
    filter::{Filter, MAX_SCAN_BYTES},
    method::error,
};
use mail_kernel::Error;
use mail_service::MailService;
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub(super) fn get(
    service: &MailService,
    token: &str,
    args: &Value,
    mut response_limit: usize,
) -> Result<Value, &'static str> {
    let account = args["accountId"].as_str().ok_or("invalidArguments")?;
    let filter = Filter::parse(&args["filter"])?;
    let ids = args["emailIds"].as_array().ok_or("invalidArguments")?;
    if ids.len() > 256 {
        return Err("tooManyObjects");
    }
    let ids = ids
        .iter()
        .map(|id| id.as_str().map(str::to_owned).ok_or("invalidArguments"))
        .collect::<Result<Vec<_>, _>>()?;
    let selected = service.messages(token, account, &ids).map_err(|e| {
        if e == Error::Unavailable {
            "serverFail"
        } else {
            "accountNotFound"
        }
    })?;
    let mut subject_terms = vec![];
    let mut body_terms = vec![];
    filter.terms("subject", &mut subject_terms);
    filter.terms("body", &mut body_terms);
    let mut remaining = MAX_SCAN_BYTES;
    let mut list = Vec::new();
    let mut seen = BTreeSet::new();
    for message in &selected.messages {
        if !seen.insert(&message.id) {
            continue;
        }
        let (subject, preview) = if subject_terms.is_empty() && body_terms.is_empty() {
            (None, None)
        } else {
            filter.charge(message.size, &mut remaining)?;
            let raw = service.download(token, account, &message.id).map_err(|e| {
                if e == Error::NotFound {
                    "serverFail"
                } else {
                    error(e)
                }
            })?;
            let (subject, body, _) = super::filter::text(&raw)?;
            (
                highlight(&subject, &subject_terms),
                highlight(&body, &body_terms),
            )
        };
        let snippet = json!({"emailId":message.id,"subject":subject,"preview":preview});
        super::limits::charge(&snippet, &mut response_limit)?;
        list.push(snippet);
    }
    let missing: BTreeSet<_> = ids.iter().filter(|id| !seen.contains(id)).collect();
    Ok(
        json!({"accountId":account,"list":list,"notFound":if missing.is_empty() { Value::Null } else { json!(missing) }}),
    )
}

fn highlight(text: &str, terms: &[&str]) -> Option<String> {
    // Keep folded-byte positions mapped back to original UTF-8 boundaries so
    // case expansions (for example İ) cannot split characters or markup.
    let folded = text.to_lowercase();
    let mut spans = Vec::new();
    for term in terms {
        for (start, matched) in folded.match_indices(term) {
            spans.push((start, start + matched.len()));
            if spans.len() == 4096 {
                break;
            }
        }
        if spans.len() == 4096 {
            break;
        }
    }
    let mut boundaries: Vec<_> = spans.iter().flat_map(|(a, b)| [*a, *b]).collect();
    boundaries.sort_unstable();
    boundaries.dedup();
    let mut mapping = std::collections::BTreeMap::new();
    let mut next = 0;
    let mut offset = 0;
    for (start, ch) in text.char_indices() {
        let end = offset + ch.to_lowercase().map(char::len_utf8).sum::<usize>();
        while next < boundaries.len() && boundaries[next] < end {
            mapping.insert(
                boundaries[next],
                (
                    start,
                    if boundaries[next] == offset {
                        start
                    } else {
                        start + ch.len_utf8()
                    },
                ),
            );
            next += 1;
        }
        offset = end;
    }
    mapping.insert(folded.len(), (text.len(), text.len()));
    for span in &mut spans {
        *span = (mapping[&span.0].0, mapping[&span.1].1);
    }
    spans.sort_unstable();
    let first = spans.first()?.0;
    let start = text[..first]
        .char_indices()
        .rev()
        .nth(80)
        .map_or(0, |(index, _)| index);
    let end = text[start..]
        .char_indices()
        .nth(256)
        .map_or(text.len(), |(index, _)| start + index);
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (a, b) in spans {
        if a >= end {
            break;
        }
        if b <= start {
            continue;
        }
        let a = a.max(start);
        let b = b.min(end);
        if let Some(last) = merged.last_mut()
            && a <= last.1
        {
            last.1 = last.1.max(b);
        } else {
            merged.push((a, b));
        }
    }
    let mut result = String::new();
    if start > 0 {
        result.push('…');
    }
    let mut position = start;
    for (a, b) in merged {
        escape(&text[position..a], &mut result);
        result.push_str("<mark>");
        escape(&text[a..b], &mut result);
        result.push_str("</mark>");
        position = b;
    }
    escape(&text[position..end], &mut result);
    if end < text.len() {
        result.push('…');
    }
    Some(result)
}
fn escape(text: &str, result: &mut String) {
    for ch in text.chars() {
        match ch {
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            _ => result.push(ch),
        }
    }
}
