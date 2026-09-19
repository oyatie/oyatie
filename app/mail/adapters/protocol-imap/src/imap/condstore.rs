use super::{
    response::Output,
    syntax::{Token, tokens},
};
use mail_kernel::{Account, HistoryEntry};
use mail_service::MailService;
use std::collections::BTreeSet;

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
            // IMAP4rev2 is not advertised, so it is not enabled either
            // (RFC 5161 §3.1: unknown capabilities are ignored, OK stays).
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

/// What the history log says about one mailbox after MODSEQ `since`.
#[derive(Default)]
pub(super) struct Vanished {
    /// UIDs whose link to the mailbox was removed (expunge, MOVE out,
    /// mailbox-set change) in a commit newer than `since`.
    pub uids: Vec<u32>,
    /// `since` predates the retained history (RFC 7162 §3.2.5.2): every
    /// requested message is reported and every unknown UID has vanished.
    pub below_floor: bool,
}

/// History rows are read in pages of whole revisions; a window that needs
/// more than this many pages is refused rather than scanned unboundedly.
const PAGES: usize = 64;

pub(super) fn vanished(
    service: &MailService,
    token: &str,
    account: &Account,
    mailbox: &str,
    since: u64,
) -> Result<Vanished, &'static str> {
    let highest = account
        .mailboxes
        .iter()
        .find(|m| m.id == mailbox)
        .map_or(account.mail_modseq, |m| m.highest_modseq);
    let mut result = Vanished::default();
    if since >= highest {
        return Ok(result);
    }
    let mut cursor = since;
    // A UID linked after `since` was never seen by the client, so its later
    // removal is not reported (Stalwart cancels create+delete in the window).
    let mut added = BTreeSet::new();
    for _ in 0..PAGES {
        let page = service
            .history(token, &account.id, cursor, 10_000)
            .map_err(super::retry::status)?;
        result.below_floor |= page.below_floor();
        for (_, entry) in &page.rows {
            match entry {
                HistoryEntry::Added {
                    mailbox: m, uid, ..
                } if m == mailbox => {
                    added.insert(*uid);
                }
                HistoryEntry::Removed {
                    mailbox: m, uid, ..
                } if m == mailbox && !added.contains(uid) => result.uids.push(*uid),
                _ => {}
            }
        }
        if !page.has_more {
            return Ok(result);
        }
        cursor = page.revision;
    }
    Err("NO")
}

/// UIDs in `set` (clamped to the mailbox's allocated range) that are not
/// `present`: the VANISHED (EARLIER) answer when history is unavailable.
pub(super) fn missing(set: &[(u32, u32)], uid_next: u32, present: &BTreeSet<u32>) -> Vec<u32> {
    let last = uid_next.saturating_sub(1);
    set.iter()
        .flat_map(|(a, b)| (*a).max(1)..=(*b).min(last))
        .filter(|uid| !present.contains(uid))
        .collect()
}
