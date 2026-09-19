//! CREATE, RENAME, DELETE, SUBSCRIBE and UNSUBSCRIBE.
use super::{folders, response::Output};
use mail_kernel::{Account, Command};
use mail_service::MailService;

type Completion = Result<Option<String>, &'static str>;

fn objectid(output: &Output, account: &Account, mailbox: &mail_kernel::Mailbox) -> Option<String> {
    output
        .objectid
        .then(|| format!("[OBJECTID {}]", super::objectid::compound(account, mailbox)))
}

/// RFC 6154 §5.3: `CREATE name (USE (\Attr ...))`; a single role is stored.
fn special_use(parameters: &str) -> Result<Option<&'static str>, &'static str> {
    if parameters.is_empty() {
        return Ok(None);
    }
    fn list(value: &str) -> Result<&str, &'static str> {
        value
            .trim_matches(' ')
            .strip_prefix('(')
            .and_then(|v| v.strip_suffix(')'))
            .ok_or("BAD")
    }
    let (name, attributes) = list(parameters)?
        .trim_start_matches(' ')
        .split_once(' ')
        .ok_or("BAD")?;
    let attributes: Vec<_> = list(attributes)?
        .split(' ')
        .filter(|a| !a.is_empty())
        .collect();
    if !name.eq_ignore_ascii_case("USE") || attributes.is_empty() {
        return Err("BAD");
    }
    let mut role = None;
    for attribute in attributes {
        if attribute.contains(['(', ')', '"', '{']) {
            return Err("BAD");
        }
        let value = folders::role(attribute).ok_or("NO [USEATTR]")?;
        if role.is_some_and(|r| r != value) {
            return Err("NO [USEATTR]");
        }
        role = Some(value);
    }
    Ok(role)
}

pub(super) fn create(
    service: &MailService,
    token: &str,
    account: &Account,
    parts: &[String],
    output: &Output,
) -> Completion {
    let role = special_use(&parts[3])?;
    let mut parents = folders::segments(&parts[2]);
    let name = parents.pop().ok_or("NO")?;
    if let Some(role) = role
        && account
            .mailboxes
            .iter()
            .any(|m| m.role.as_deref() == Some(role))
    {
        return Err("NO [USEATTR]");
    }
    let (account, parent_id) = folders::ensure_parents(service, token, account, &parents)?;
    let mut properties = folders::leaf(name.clone(), parent_id, None);
    properties.role = role.map(str::to_owned);
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
    parents.push(name);
    let folder = folders::find(&updated, &parents.join("/")).ok_or("NO")?;
    Ok(objectid(output, &updated, folder))
}

pub(super) fn rename(
    service: &MailService,
    token: &str,
    account: &Account,
    parts: &[String],
    output: &Output,
) -> Completion {
    let mailbox = folders::find(account, &parts[2]).ok_or("NO [NONEXISTENT]")?;
    let mut parents = folders::segments(&parts[3]);
    let name = parents.pop().ok_or("NO")?;
    let (refreshed, parent_id) = folders::ensure_parents(service, token, account, &parents)?;
    service
        .execute(
            token,
            &refreshed.id,
            refreshed.revision,
            vec![Command::SetMailbox {
                id: Some(mailbox.id.clone()),
                properties: folders::leaf(name, parent_id, Some(mailbox)),
            }],
        )
        .map_err(|_| "NO")?;
    Ok(objectid(output, account, mailbox))
}

/// A mailbox with inferiors is refused (RFC 3501 §6.3.4).
pub(super) fn delete(
    service: &MailService,
    token: &str,
    account: &Account,
    parts: &[String],
) -> Completion {
    let mailbox = folders::find(account, &parts[2]).ok_or("NO [NONEXISTENT]")?;
    service
        .execute(
            token,
            &account.id,
            account.revision,
            vec![Command::RemoveMailbox {
                id: mailbox.id.clone(),
                remove_emails: true,
            }],
        )
        .map(|_| None)
        .map_err(|_| "NO")
}

pub(super) fn subscribe(
    service: &MailService,
    token: &str,
    account: &Account,
    parts: &[String],
) -> Completion {
    let mailbox = folders::find(account, &parts[2]).ok_or("NO [NONEXISTENT]")?;
    let subscribed = parts[1].eq_ignore_ascii_case("SUBSCRIBE");
    if mailbox.is_subscribed == subscribed {
        return Ok(None);
    }
    let mut properties = mailbox.properties();
    properties.is_subscribed = subscribed;
    service
        .execute(
            token,
            &account.id,
            account.revision,
            vec![Command::SetMailbox {
                id: Some(mailbox.id.clone()),
                properties,
            }],
        )
        .map(|_| None)
        .map_err(|_| "NO")
}
