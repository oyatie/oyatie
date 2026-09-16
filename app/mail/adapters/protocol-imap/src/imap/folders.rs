use mail_kernel::{Account, Mailbox, MailboxProperties};

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

pub(super) fn properties(
    account: &Account,
    path: &str,
    old: Option<&Mailbox>,
) -> Result<MailboxProperties, &'static str> {
    let path = canonical(path.trim_end_matches('/'));
    let (parent_id, name) = match path.rsplit_once('/') {
        Some((parent, name)) => (Some(find(account, parent).ok_or("NO")?.id.clone()), name),
        None => (None, path.as_str()),
    };
    let mut properties = old.map_or_else(
        || MailboxProperties::named(name.into()),
        Mailbox::properties,
    );
    properties.name = name.into();
    properties.parent_id = parent_id;
    Ok(properties)
}

pub(super) fn matches(pattern: &str, path: &str) -> bool {
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
