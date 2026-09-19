use mail_kernel::{Account, Command, Mailbox, MailboxProperties};
use mail_service::MailService;
use std::borrow::Cow;

pub(super) fn canonical(path: &str) -> String {
    let (first, rest) = path.split_once('/').map_or((path, ""), |(a, b)| (a, b));
    if first.eq_ignore_ascii_case("inbox") {
        if rest.is_empty() {
            "INBOX".into()
        } else {
            format!("INBOX/{rest}")
        }
    } else {
        path.into()
    }
}

pub(super) fn find<'a>(account: &'a Account, path: &str) -> Option<&'a Mailbox> {
    let path = canonical(path);
    account
        .mailboxes
        .iter()
        .find(|m| account.mailbox_path(&m.id).as_deref() == Some(path.as_str()))
}

/// Splits a client-supplied name into trimmed segments. Empty segments from
/// leading, doubled or trailing delimiters are ignored; INBOX is canonical.
pub(super) fn segments(path: &str) -> Vec<String> {
    let mut segments: Vec<String> = path
        .split('/')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    if let Some(first) = segments.first_mut()
        && first.eq_ignore_ascii_case("inbox")
    {
        *first = "INBOX".into();
    }
    segments
}

fn child<'a>(account: &'a Account, parent: Option<&str>, name: &str) -> Option<&'a Mailbox> {
    account
        .mailboxes
        .iter()
        .find(|m| m.parent_id.as_deref() == parent && m.name == name)
}

/// Creates the missing ancestors named by `parents`, one committed revision
/// each, and returns the refreshed account with the innermost parent's id.
pub(super) fn ensure_parents<'a>(
    service: &MailService,
    token: &str,
    account: &'a Account,
    parents: &[String],
) -> Result<(Cow<'a, Account>, Option<String>), &'static str> {
    let mut account = Cow::Borrowed(account);
    let mut parent: Option<String> = None;
    for name in parents {
        if child(&account, parent.as_deref(), name).is_none() {
            let mut properties = MailboxProperties::named(name.clone());
            properties.parent_id = parent.clone();
            properties.is_subscribed = false;
            let updated = service
                .execute(
                    token,
                    &account.id,
                    account.revision,
                    vec![Command::SetMailbox {
                        id: None,
                        properties,
                    }],
                )
                .map_err(|_| "NO")?;
            account = Cow::Owned(updated);
        }
        parent = Some(
            child(&account, parent.as_deref(), name)
                .ok_or("NO")?
                .id
                .clone(),
        );
    }
    Ok((account, parent))
}

/// Properties for the leaf of a CREATE or RENAME. IMAP-created mailboxes are
/// not subscribed until the client asks (RFC 3501 §6.3.6); RENAME keeps the
/// role and subscription of the existing mailbox.
pub(super) fn leaf(
    name: String,
    parent_id: Option<String>,
    old: Option<&Mailbox>,
) -> MailboxProperties {
    let mut properties = old.map_or_else(
        || {
            let mut properties = MailboxProperties::named(String::new());
            properties.is_subscribed = false;
            properties
        },
        Mailbox::properties,
    );
    properties.name = name;
    properties.parent_id = parent_id;
    properties
}

/// RFC 6154 special-use attribute for a JMAP mailbox role.
pub(super) fn attribute(mailbox: &Mailbox) -> Option<&'static str> {
    Some(match mailbox.role.as_deref()? {
        "archive" => "\\Archive",
        "drafts" => "\\Drafts",
        "important" => "\\Important",
        "junk" => "\\Junk",
        "sent" => "\\Sent",
        "trash" => "\\Trash",
        _ => return None,
    })
}

/// JMAP role for an RFC 6154 use attribute; `\All` and `\Flagged` are
/// virtual views this server cannot create.
pub(super) fn role(attribute: &str) -> Option<&'static str> {
    Some(match attribute.to_ascii_lowercase().as_str() {
        "\\archive" => "archive",
        "\\drafts" => "drafts",
        "\\important" => "important",
        "\\junk" => "junk",
        "\\sent" => "sent",
        "\\trash" => "trash",
        _ => return None,
    })
}

/// LIST pattern matching: `*` spans hierarchy delimiters, `%` does not.
pub fn matches(pattern: &str, path: &str) -> bool {
    let text: Vec<_> = path.chars().collect();
    let mut reachable = vec![false; text.len() + 1];
    reachable[0] = true;
    for c in canonical(pattern).chars() {
        if c == '*' || c == '%' {
            for i in 1..=text.len() {
                reachable[i] |= reachable[i - 1] && (c == '*' || text[i - 1] != '/');
            }
        } else {
            for i in (1..=text.len()).rev() {
                reachable[i] = reachable[i - 1] && text[i - 1] == c;
            }
            reachable[0] = false;
        }
    }
    reachable[text.len()]
}
