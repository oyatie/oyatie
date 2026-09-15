use super::{
    folders, object_id,
    response::Output,
    syntax::{Token, tokens},
};
use mail_kernel::Account;

pub(super) fn execute(
    account: &Account,
    parts: &[String],
    output: &mut Output,
) -> Result<Option<String>, &'static str> {
    let name = parts.get(2).ok_or("BAD")?;
    let args = parts.get(3..).ok_or("BAD")?.join(" ");
    let items = tokens(
        args.strip_prefix('(')
            .and_then(|a| a.strip_suffix(')'))
            .ok_or("BAD")?,
    )
    .ok_or("BAD")?;
    let items = items
        .into_iter()
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
    if items.iter().any(|i| i == "OBJECTID") {
        super::objectid::activate(output);
    }
    let folder = folders::find(account, name).ok_or("NO")?;
    let messages = super::selected::messages_in(account, &folder.id);
    let mailbox =
        super::mailboxes::quote(&account.mailbox_path(&folder.id).ok_or("NO")?, output.utf8);
    output.extend_from_slice(format!("* STATUS {mailbox} (").as_bytes());
    for (i, item) in items.iter().enumerate() {
        let value = match item.as_str() {
            "HIGHESTMODSEQ" => account.mail_modseq.to_string(),
            "MESSAGES" => messages.len().to_string(),
            "RECENT" => "0".into(),
            "UIDNEXT" => folder.uid_next.to_string(),
            "UIDVALIDITY" => folder.uid_validity.to_string(),
            "UNSEEN" => messages
                .iter()
                .filter(|m| !m.keywords.iter().any(|k| k == "$seen"))
                .count()
                .to_string(),
            "DELETED" => messages
                .iter()
                .filter(|m| m.keywords.iter().any(|k| k == "$deleted"))
                .count()
                .to_string(),
            "SIZE" => messages
                .iter()
                .map(|m| m.size as u64)
                .sum::<u64>()
                .to_string(),
            "OBJECTID" => super::objectid::compound(account, folder),
            "MAILBOXID" => format!("({})", object_id("F", &account.id, &folder.id)),
            _ => unreachable!(),
        };
        output.extend_from_slice(
            format!("{}{item} {value}", if i == 0 { "" } else { " " }).as_bytes(),
        );
    }
    output.extend_from_slice(b")\r\n");
    Ok(None)
}
