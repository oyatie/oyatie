use super::{
    condstore,
    response::Output,
    state::Selection,
    syntax::{flags, keyword, sequence_set},
};
use mail_kernel::{Account, Command, Message};
use mail_service::MailService;
use std::collections::BTreeSet;

pub(super) fn execute(
    service: &MailService,
    token: &str,
    account: &Account,
    selected: &mut Selection,
    chosen: Vec<(usize, &Message)>,
    parts: &[String],
    output: &mut Output,
) -> Result<Option<String>, &'static str> {
    if selected.readonly {
        return Err("NO");
    }
    let uid = parts[1].eq_ignore_ascii_case("UID");
    let offset = if uid { 3 } else { 2 };
    let args = parts.get(offset + 1..).ok_or("BAD")?.join(" ");
    let mut args = args.as_str();
    let mut since = None;
    if args.starts_with('(') {
        let (modifier, rest) = args.split_once(')').ok_or("BAD")?;
        let fields: Vec<_> = modifier[1..].split_ascii_whitespace().collect();
        if fields.len() != 2 || !fields[0].eq_ignore_ascii_case("UNCHANGEDSINCE") {
            return Err("BAD");
        }
        since = Some(condstore::number(fields[1])?);
        args = rest.strip_prefix(' ').ok_or("BAD")?.trim_start();
    }
    let (mode, flag_list) = args.split_once(' ').ok_or("BAD")?;
    let mode = mode.to_ascii_uppercase();
    if !matches!(
        mode.as_str(),
        "FLAGS" | "FLAGS.SILENT" | "+FLAGS" | "+FLAGS.SILENT" | "-FLAGS" | "-FLAGS.SILENT"
    ) {
        return Err("BAD");
    }
    let flag_list = flag_list.trim();
    let flag_list = if flag_list.starts_with('(') {
        flag_list
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .ok_or("BAD")?
    } else {
        flag_list
    };
    let requested = flag_list
        .split_ascii_whitespace()
        .map(keyword)
        .collect::<Option<Vec<_>>>()
        .ok_or("BAD")?;
    if !mail_kernel::valid_keywords(&requested) {
        return Err("BAD");
    }
    let chosen: Vec<_> = chosen
        .into_iter()
        .filter(|(_, m)| since.is_none_or(|s| m.modseq <= s))
        .collect();
    let accepted: BTreeSet<_> = chosen
        .iter()
        .map(|(i, m)| {
            if uid {
                m.uid_in(&selected.mailbox).unwrap()
            } else {
                *i as u32 + 1
            }
        })
        .collect();
    let modified = if since.is_some() {
        let requested = if parts[offset] == "$" {
            selected
                .saved
                .iter()
                .filter_map(|entry| {
                    let n = if uid {
                        entry.1
                    } else {
                        selected.ids.iter().position(|id| id == entry)? as u32 + 1
                    };
                    Some((n, n))
                })
                .collect()
        } else {
            let largest = if uid {
                selected.largest_uid(account)
            } else {
                selected.ids.len() as u32
            };
            sequence_set(&parts[offset], largest).ok_or("BAD")?
        };
        difference(requested, &accepted)
    } else {
        String::new()
    };
    let changes = chosen
        .iter()
        .map(|(_, m)| {
            let mut keys = if mode.starts_with(['+', '-']) {
                m.keywords.clone()
            } else {
                vec![]
            };
            if mode.starts_with('-') {
                keys.retain(|k| !requested.contains(k));
            } else {
                keys.extend(requested.clone());
            }
            Command::Keywords {
                id: m.id.clone(),
                keywords: keys,
            }
        })
        .collect();
    let updated = service
        .execute(token, &account.id, account.revision, changes)
        .map_err(|_| "NO")?;
    if since.is_some() {
        selected.condstore = true;
        output.condstore = true;
    }
    if !mode.ends_with(".SILENT") || selected.condstore {
        for (i, m) in chosen {
            let current = updated.messages.iter().find(|e| e.id == m.id).ok_or("NO")?;
            output.fetch_start(i + 1, m.uid_in(&selected.mailbox).ok_or("NO")?);
            if selected.condstore {
                output.extend_from_slice(format!(" MODSEQ ({})", current.modseq).as_bytes());
            }
            if !mode.ends_with(".SILENT") {
                output.extend_from_slice(format!(" FLAGS ({})", flags(current)).as_bytes());
            }
            output.extend_from_slice(b")\r\n");
        }
    }
    selected.modseq = updated.mail_modseq;
    Ok((!modified.is_empty()).then(|| format!("[MODIFIED {modified}]")))
}

fn difference(mut ranges: Vec<(u32, u32)>, accepted: &BTreeSet<u32>) -> String {
    ranges.sort_unstable();
    let mut merged: Vec<(u32, u32)> = vec![];
    for (a, b) in ranges {
        if let Some(last) = merged.last_mut()
            && a <= last.1.saturating_add(1)
        {
            last.1 = last.1.max(b);
        } else {
            merged.push((a, b));
        }
    }
    let mut result = Vec::new();
    for (a, b) in merged {
        let mut next = u64::from(a);
        for value in accepted.range(a..=b).copied().map(u64::from) {
            if next < value {
                result.push((next, value - 1));
            }
            next = value + 1;
        }
        if next <= u64::from(b) {
            result.push((next, u64::from(b)));
        }
    }
    result
        .into_iter()
        .map(|(a, b)| {
            if a == b {
                a.to_string()
            } else {
                format!("{a}:{b}")
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}
