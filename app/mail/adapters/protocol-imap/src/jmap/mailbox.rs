use mail_kernel::{Account, Command, Error, MailboxProperties, MailboxState};
use serde_json::{Value, json};

pub(super) fn writable(
    service: &mail_service::MailService,
    token: &str,
    account: &str,
) -> Result<bool, &'static str> {
    match service.authorize(token, account, mail_api::Action::Write) {
        Ok(_) => Ok(true),
        Err(Error::Forbidden) => Ok(false),
        Err(error) => Err(super::method::error(error)),
    }
}

pub(super) fn value(state: &MailboxState, write: bool) -> Value {
    let m = &state.mailbox;
    json!({"id":m.id,"name":m.name,"parentId":m.parent_id,"role":m.role,"sortOrder":m.sort_order,
        "isSubscribed":m.is_subscribed,"totalEmails":state.total_emails,"unreadEmails":state.unread_emails,
        "totalThreads":state.total_threads,"unreadThreads":state.unread_threads,
        "myRights":{"mayReadItems":true,"mayAddItems":write,"mayRemoveItems":write,"maySetSeen":write,
            "maySetKeywords":write,"mayCreateChild":write,"mayRename":write && m.role.as_deref()!=Some("inbox"),
            "mayDelete":write && m.role.as_deref()!=Some("inbox"),"maySubmit":false}})
}

pub(super) fn command(
    account: &Account,
    id: Option<&str>,
    value: &Value,
) -> Result<Command, &'static str> {
    let object = value.as_object().ok_or("invalidProperties")?;
    if object
        .keys()
        .any(|k| !["name", "parentId", "role", "sortOrder", "isSubscribed"].contains(&k.as_str()))
    {
        return Err("invalidProperties");
    }
    let mut properties = match id {
        Some(id) => account
            .mailboxes
            .iter()
            .find(|m| m.id == id)
            .ok_or("notFound")?
            .properties(),
        None => MailboxProperties::named(String::new()),
    };
    if let Some(value) = object.get("name") {
        properties.name = value.as_str().ok_or("invalidProperties")?.to_owned();
    }
    for (key, property) in [
        ("parentId", &mut properties.parent_id),
        ("role", &mut properties.role),
    ] {
        if let Some(value) = object.get(key) {
            *property = if value.is_null() {
                None
            } else {
                Some(value.as_str().ok_or("invalidProperties")?.to_owned())
            };
        }
    }
    if let Some(value) = object.get("sortOrder") {
        properties.sort_order = value
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("invalidProperties")?;
    }
    if let Some(value) = object.get("isSubscribed") {
        properties.is_subscribed = value.as_bool().ok_or("invalidProperties")?;
    }
    if account.mailboxes.iter().any(|m| {
        Some(m.id.as_str()) != id
            && m.name == properties.name
            && m.parent_id == properties.parent_id
    }) {
        return Err("alreadyExists");
    }
    if properties.role.is_some()
        && account
            .mailboxes
            .iter()
            .any(|m| Some(m.id.as_str()) != id && m.role == properties.role)
    {
        return Err("invalidProperties");
    }
    Ok(Command::SetMailbox {
        id: id.map(str::to_owned),
        properties,
    })
}

pub(super) fn destroy(
    account: &Account,
    id: &str,
    remove_emails: bool,
) -> Result<Command, &'static str> {
    if !account.mailboxes.iter().any(|m| m.id == id) {
        return Err("notFound");
    }
    if account
        .mailboxes
        .iter()
        .any(|m| m.parent_id.as_deref() == Some(id))
    {
        return Err("mailboxHasChild");
    }
    if !remove_emails
        && account
            .messages
            .iter()
            .any(|m| m.mailboxes.contains_key(id))
    {
        return Err("mailboxHasEmail");
    }
    Ok(Command::RemoveMailbox {
        id: id.into(),
        remove_emails,
    })
}

pub(super) fn error(error: Error) -> &'static str {
    match error {
        Error::Invalid | Error::NotFound => "invalidProperties",
        other => super::method::error(other),
    }
}
