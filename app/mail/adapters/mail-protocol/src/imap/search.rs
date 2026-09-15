mod criteria;
mod evaluate;
mod sort;
mod thread;
use super::{
    response::Output,
    state::Selection,
    syntax::{Token, tokens},
};
use criteria::Parser;
use mail_kernel::{Account, Message};
use mail_service::MailService;
use std::collections::BTreeSet;

pub(super) fn execute(
    service: &MailService,
    token: &str,
    account: &Account,
    selected: &mut Selection,
    parts: &[String],
    messages: &[(usize, &Message)],
    output: &mut Output,
) -> Result<Option<String>, &'static str> {
    let uid = parts[1].eq_ignore_ascii_case("UID");
    let operation = &parts[if uid { 2 } else { 1 }];
    let sorting = operation.eq_ignore_ascii_case("SORT");
    let threading = operation.eq_ignore_ascii_case("THREAD");
    let input = parts.get(if uid { 3 } else { 2 }).ok_or("BAD")?;
    let tokens = tokens(input).ok_or("BAD")?;
    let mut parser = Parser {
        tokens: &tokens,
        largest: selected.ids.len() as u32,
        largest_uid: messages
            .last()
            .and_then(|(_, m)| m.uid_in(&selected.mailbox))
            .unwrap_or_default(),
    };
    if threading
        && !matches!(
            parser.text()?.to_ascii_uppercase().as_str(),
            "REFERENCES" | "ORDEREDSUBJECT"
        )
    {
        return Err("BAD");
    }
    let mut returns = BTreeSet::new();
    if matches!(parser.tokens.first(), Some(Token::Word(w)) if w.eq_ignore_ascii_case("RETURN")) {
        if threading {
            return Err("BAD");
        }
        parser.tokens = &parser.tokens[1..];
        if !matches!(parser.tokens.first(), Some(Token::Open)) {
            return Err("BAD");
        }
        parser.tokens = &parser.tokens[1..];
        while !matches!(parser.tokens.first(), Some(Token::Close)) {
            let word = parser.atom()?.to_ascii_uppercase();
            if !matches!(word.as_str(), "SAVE" | "ALL" | "COUNT" | "MIN" | "MAX") {
                return Err("BAD");
            }
            returns.insert(word);
        }
        parser.tokens = &parser.tokens[1..];
        if returns.is_empty() {
            returns.insert("ALL".into());
        }
    }
    let keys = if sorting {
        sort::parse(&mut parser)?
    } else {
        vec![]
    };
    if sorting
        || threading
        || matches!(parser.tokens.first(), Some(Token::Word(w)) if w.eq_ignore_ascii_case("CHARSET"))
    {
        if output.utf8 && !sorting && !threading {
            return Err("BAD");
        }
        if !sorting && !threading {
            parser.tokens = &parser.tokens[1..];
        }
        let charset = parser.text()?;
        if !charset.eq_ignore_ascii_case("UTF-8") && !charset.eq_ignore_ascii_case("US-ASCII") {
            return Err("NO [BADCHARSET (US-ASCII UTF-8)]");
        }
        if charset.eq_ignore_ascii_case("US-ASCII") && !input.is_ascii() {
            return Err("BAD");
        }
    }
    let criteria = parser.and(0)?;
    if !parser.tokens.is_empty() {
        return Err("BAD");
    }
    if output.uidonly && criteria.uses_sequences() {
        return Err("BAD [UIDREQUIRED]");
    }
    let has_modseq = criteria.has_modseq();
    let highest_modseq = account.mail_modseq;
    if has_modseq {
        selected.condstore = true;
        output.condstore = true;
    }
    let mut ids = Vec::new();
    let mut saved = Vec::new();
    let mut ordered = Vec::new();
    let mut budget = 32 * 1024 * 1024;
    for (sequence, message) in messages {
        if output.is_closed() {
            return Err("NO");
        }
        let raw = if criteria.needs_raw() || keys.iter().any(sort::Key::needs_raw) {
            Some(
                service
                    .download(token, &account.id, &message.id)
                    .map_err(|_| "NO")?,
            )
        } else {
            None
        };
        let parsed = raw.as_ref().and_then(|raw| {
            let parser = mail_parser::MessageParser::default();
            if criteria.needs_raw() {
                parser.parse(raw)
            } else {
                parser.parse_headers(raw)
            }
        });
        let message_uid = message.uid_in(&selected.mailbox).ok_or("NO")?;
        let is_saved = selected.saved.contains(&(message.id.clone(), message_uid));
        if evaluate::matches(
            &criteria,
            message,
            *sequence as u32 + 1,
            message_uid,
            is_saved,
            parsed.as_ref(),
            &account.id,
        ) {
            if sorting {
                ordered.push(sort::values(&keys, message, parsed.as_ref(), &mut budget)?);
            }
            ids.push(if uid {
                message_uid
            } else {
                *sequence as u32 + 1
            });
            saved.push((message.id.clone(), message_uid));
        }
    }
    if sorting {
        let mut order: Vec<_> = (0..ids.len()).collect();
        order.sort_by(|a, b| sort::compare(&keys, &ordered[*a], &ordered[*b]).then(a.cmp(b)));
        ids = order.iter().map(|i| ids[*i]).collect();
        saved = order.iter().map(|i| saved[*i].clone()).collect();
    }
    if threading {
        thread::write(messages, selected, &ids, uid, output);
        return Ok(None);
    }
    let count = ids.len();
    let min = ids.iter().copied().min();
    let max = ids.iter().copied().max();
    if returns.contains("MIN") || returns.contains("MAX") {
        // Match the oracle's result mapping: COUNT counts every match, whereas
        // ALL and SAVE receive requested numeric extrema, even for SORT.
        let extrema: Vec<_> = [("MIN", min), ("MAX", max)]
            .into_iter()
            .filter_map(|(name, value)| returns.contains(name).then_some(value).flatten())
            .collect();
        saved = extrema
            .iter()
            .filter_map(|id| ids.iter().position(|n| n == id).map(|i| saved[i].clone()))
            .collect();
        ids = extrema;
    }
    if returns.contains("SAVE") {
        selected.saved = saved.into_iter().collect();
    }
    if returns.is_empty() {
        output.extend_from_slice(if sorting { b"* SORT" } else { b"* SEARCH" });
        for id in &ids {
            output.extend_from_slice(format!(" {id}").as_bytes());
        }
        if has_modseq && highest_modseq > 0 {
            output.extend_from_slice(format!(" (MODSEQ {highest_modseq})").as_bytes());
        }
        output.extend_from_slice(b"\r\n");
    } else if returns.iter().any(|r| r != "SAVE") {
        output.extend_from_slice(
            format!(
                "* ESEARCH (TAG \"{}\"){}",
                parts[0],
                if uid { " UID" } else { "" }
            )
            .as_bytes(),
        );
        for item in ["COUNT", "MIN", "MAX", "ALL"]
            .into_iter()
            .filter(|item| returns.contains(*item))
        {
            let value = match item {
                "COUNT" => Some(count.to_string()),
                "MIN" => min.map(|id| id.to_string()),
                "MAX" => max.map(|id| id.to_string()),
                "ALL" if !ids.is_empty() => {
                    output.extend_from_slice(b" ALL ");
                    write_ranges(&ids, output);
                    None
                }
                _ => None,
            };
            if let Some(value) = value {
                output.extend_from_slice(format!(" {item} {value}").as_bytes());
            }
        }
        if has_modseq && highest_modseq > 0 {
            output.extend_from_slice(format!(" MODSEQ {highest_modseq}").as_bytes());
        }
        output.extend_from_slice(b"\r\n");
    }
    Ok(None)
}

fn write_ranges(mut ids: &[u32], output: &mut Output) {
    let mut separator = "";
    while let Some((&first, rest)) = ids.split_first() {
        let mut last = first;
        let count = rest
            .iter()
            .take_while(|&&next| {
                if last.checked_add(1) == Some(next) {
                    last = next;
                    true
                } else {
                    false
                }
            })
            .count();
        output.extend_from_slice(format!("{separator}{first}").as_bytes());
        if first != last {
            output.extend_from_slice(format!(":{last}").as_bytes());
        }
        separator = ",";
        ids = &rest[count..];
    }
}
