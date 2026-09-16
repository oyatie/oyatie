use super::{
    response::Output,
    syntax::{Token, tokens},
};
use mail_kernel::Account;
use mail_service::MailService;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn enable(
    session: &mut super::Session,
    parts: &[String],
    output: &mut Output,
) -> Result<(), &'static str> {
    // Validate the complete request before enabling any extension. These are
    // the capability names accepted by the maintained upstream ENABLE parser.
    for value in &parts[2..] {
        match value.to_ascii_uppercase().as_str() {
            "OBJECTID+" | "UTF8=ACCEPT" | "CONDSTORE" | "QRESYNC" | "STARTTLS"
            | "LOGINDISABLED" | "IMAP4REV2" | "UIDONLY" => {}
            _ => return Err("BAD"),
        }
    }
    if parts[2..]
        .iter()
        .any(|v| v.eq_ignore_ascii_case("IMAP4rev2"))
    {
        return Err("NO");
    }
    output.extend_from_slice(b"* ENABLED");
    for value in &parts[2..] {
        match value.to_ascii_uppercase().as_str() {
            "UIDONLY" => {
                session.uidonly = true;
                output.uidonly = true;
                output.extend_from_slice(b" UIDONLY");
            }
            "OBJECTID+" => {
                session.objectid = true;
                output.objectid = true;
                output.extend_from_slice(b" OBJECTID+");
            }
            "UTF8=ACCEPT" => {
                session.utf8 = true;
                output.extend_from_slice(b" UTF8=ACCEPT");
            }
            "CONDSTORE" => {
                session.condstore = true;
                output.extend_from_slice(b" CONDSTORE");
            }
            "QRESYNC" => {
                session.qresync = true;
                session.condstore = true;
                output.extend_from_slice(b" QRESYNC");
            }
            _ => {}
        }
    }
    if let Some(selected) = &mut session.selected {
        selected.condstore = session.condstore;
        selected.qresync = session.qresync;
    }
    output.condstore = session.condstore;
    output.qresync = session.qresync;
    output.extend_from_slice(b"\r\n");
    Ok(())
}

pub(super) fn number(value: &str) -> Result<u64, &'static str> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err("BAD");
    }
    value.parse().map_err(|_| "BAD")
}

pub(super) fn ranges(ids: impl IntoIterator<Item = u32>) -> String {
    let ids: BTreeSet<_> = ids.into_iter().collect();
    let mut result = Vec::new();
    let mut values = ids.into_iter().peekable();
    while let Some(first) = values.next() {
        let mut last = first;
        while values
            .peek()
            .is_some_and(|n| last.checked_add(1) == Some(*n))
        {
            last = values.next().unwrap();
        }
        result.push(if first == last {
            first.to_string()
        } else {
            format!("{first}:{last}")
        });
    }
    result.join(",")
}

pub(super) fn fetch_args(input: &str) -> Result<(&str, Option<u64>, bool), &'static str> {
    // Parse the suffix only after the complete FETCH item list. Item parsing
    // still validates every preceding byte, including quoted/literal headers.
    let Some(end) = input.trim_end().strip_suffix(')') else {
        return Ok((input, None, false));
    };
    let Some(start) = end.rfind('(') else {
        return Ok((input, None, false));
    };
    let suffix = &end[start + 1..];
    if !suffix.split_ascii_whitespace().next().is_some_and(|s| {
        s.eq_ignore_ascii_case("CHANGEDSINCE") || s.eq_ignore_ascii_case("VANISHED")
    }) {
        return Ok((input, None, false));
    }
    let items = tokens(suffix).ok_or("BAD")?;
    let mut items = items.iter();
    let mut since = None;
    let mut vanished = false;
    while let Some(item) = items.next() {
        match item {
            Token::Word(word) if word.eq_ignore_ascii_case("CHANGEDSINCE") && since.is_none() => {
                let Some(Token::Word(value)) = items.next() else {
                    return Err("BAD");
                };
                since = Some(number(value)?);
            }
            Token::Word(word) if word.eq_ignore_ascii_case("VANISHED") && !vanished => {
                vanished = true
            }
            _ => return Err("BAD"),
        }
    }
    if since.is_none() {
        return Err("BAD");
    }
    Ok((input[..start].trim_end(), since, vanished))
}

pub(super) fn vanished(
    service: &MailService,
    token: &str,
    account: &Account,
    mailbox: &str,
    since: u64,
) -> Result<Vec<u32>, &'static str> {
    if since >= account.mail_modseq {
        return Ok(vec![]);
    }
    let changes = service
        .changes_after(
            token,
            &account.id,
            since.saturating_sub(1),
            account.revision,
        )
        .map_err(|_| "NO")?;
    let mut merged = BTreeMap::new();
    let mut deleted = Vec::new();
    for change in changes {
        if change.before.as_ref().is_some_and(|m| {
            m.mailboxes.iter().any(|id| id == mailbox) && !m.uids.contains_key(mailbox)
        }) {
            return Err("NO");
        }
        if let Some(before) = &change.before
            && let Some(uid) = before.uids.get(mailbox)
            && change.after.as_ref().and_then(|a| a.uids.get(mailbox)) != Some(uid)
        {
            deleted.push(*uid);
        }
        let entry = merged
            .entry(change.id)
            .or_insert_with(|| (change.before, None));
        entry.1 = change.after;
    }
    // Stalwart scans tombstones only if the merged global log contains an
    // update or deletion. A creation followed by deletion cancels in that log.
    if !merged
        .values()
        .any(|(before, after)| before.is_some() && before != after)
    {
        deleted.clear();
    }
    Ok(deleted)
}
