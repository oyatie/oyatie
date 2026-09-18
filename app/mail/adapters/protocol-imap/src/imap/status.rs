use super::{
    folders, object_id,
    response::Output,
    syntax::{Token, tokens},
};
use mail_kernel::{Account, Mailbox};

pub(super) fn execute(
    account: &Account,
    parts: &[String],
    output: &mut Output,
) -> Result<Option<String>, &'static str> {
    let name = parts.get(2).ok_or("BAD")?;
    let args = parts.get(3..).ok_or("BAD")?.join(" ");
    let tokens = tokens(&args).ok_or("BAD")?;
    let [Token::Open, inner @ .., Token::Close] = tokens.as_slice() else {
        return Err("BAD");
    };
    let items = items(inner)?;
    if items.iter().any(|i| i == "OBJECTID") {
        super::objectid::activate(output);
    }
    let folder = folders::find(account, name).ok_or("NO [NONEXISTENT]")?;
    report(account, folder, &items, output);
    Ok(None)
}

/// The status data item names inside the parenthesised list, upper-cased and
/// in request order except SIZE, which upstream computes after the cached
/// counters and therefore reports last.
pub(super) fn items(tokens: &[Token]) -> Result<Vec<String>, &'static str> {
    let mut items = tokens
        .iter()
        .map(|t| match t {
            Token::Word(item)
                if [
                    "MESSAGES",
                    "RECENT",
                    "UIDNEXT",
                    "UIDVALIDITY",
                    "UNSEEN",
                    "MAILBOXID",
                    "OBJECTID",
                    "SIZE",
                    "HIGHESTMODSEQ",
                    "DELETED",
                ]
                .iter()
                .any(|i| item.eq_ignore_ascii_case(i)) =>
            {
                Ok(item.to_ascii_uppercase())
            }
            _ => Err("BAD"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    if items.is_empty() {
        return Err("BAD");
    }
    items.sort_by_key(|item| item == "SIZE");
    Ok(items)
}

/// One `* STATUS` line in the requested item order; RECENT is always 0.
pub(super) fn report(account: &Account, folder: &Mailbox, items: &[String], output: &mut Output) {
    let messages = super::selected::messages_in(account, &folder.id);
    let Some(path) = account.mailbox_path(&folder.id) else {
        return;
    };
    let mailbox = super::mailboxes::quote(&path, output.utf8);
    output.extend_from_slice(format!("* STATUS {mailbox} (").as_bytes());
    for (i, item) in items.iter().enumerate() {
        let value = match item.as_str() {
            "HIGHESTMODSEQ" => folder.highest_modseq.to_string(),
            "MESSAGES" => folder.total_emails.to_string(),
            "RECENT" => "0".into(),
            "UIDNEXT" => folder.uid_next.to_string(),
            "UIDVALIDITY" => folder.uid_validity.to_string(),
            "UNSEEN" => folder.unread_emails.to_string(),
            "DELETED" => messages
                .iter()
                .filter(|m| m.keywords.iter().any(|k| k == "$deleted"))
                .count()
                .to_string(),
            "SIZE" => folder.size_bytes.to_string(),
            "OBJECTID" => super::objectid::compound(account, folder),
            "MAILBOXID" => format!("({})", object_id("F", &account.id, &folder.id)),
            _ => unreachable!(),
        };
        output.extend_from_slice(
            format!("{}{item} {value}", if i == 0 { "" } else { " " }).as_bytes(),
        );
    }
    output.extend_from_slice(b")\r\n");
}
