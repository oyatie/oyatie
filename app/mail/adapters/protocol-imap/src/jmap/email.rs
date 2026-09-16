use super::method::error;
use mail_kernel::{Account, Command, Error, Message};
use mail_service::MailService;
use serde_json::{Value, json};

pub(super) fn utc_date(value: &Value) -> Result<i64, &'static str> {
    if value.is_null() {
        return std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "serverFail")?
            .as_secs()
            .try_into()
            .map_err(|_| "serverFail");
    }
    let value = value.as_str().ok_or("invalidProperties")?;
    let date = mail_parser::DateTime::parse_rfc3339(value)
        .filter(|d| d.is_valid())
        .ok_or("invalidProperties")?;
    if value.len() != 20
        || !value.ends_with('Z')
        || mail_parser::DateTime::from_timestamp(date.to_timestamp()).to_rfc3339() != value
    {
        return Err("invalidProperties");
    }
    Ok(date.to_timestamp())
}

pub(super) fn true_keys(value: &Value) -> Result<Vec<String>, &'static str> {
    let object = value.as_object().ok_or("invalidProperties")?;
    if object.values().any(|v| v != true) {
        return Err("invalidProperties");
    }
    Ok(object.keys().cloned().collect())
}

pub(super) fn created(message: &Message) -> Value {
    json!({"id":message.id,"blobId":message.id,"threadId":message.thread_id(),"size":message.size})
}

pub(super) fn import(
    service: &MailService,
    token: &str,
    account: &Account,
    args: &Value,
) -> Result<Value, &'static str> {
    if !args["ifInState"].is_null() {
        let state = args["ifInState"].as_str().ok_or("invalidArguments")?;
        if state != account.revision.to_string() {
            return Err("stateMismatch");
        }
    }
    let emails = args["emails"].as_object().ok_or("invalidArguments")?;
    if emails.len() > 256 {
        return Err("tooManyObjects");
    }
    let mut current = account.clone();
    let mut response = json!({"accountId":account.id,"oldState":account.revision.to_string(),"created":{},"notCreated":{}});
    for (key, value) in emails {
        let result = (|| {
            let properties = value.as_object().ok_or("invalidProperties")?;
            if properties
                .keys()
                .any(|k| !["blobId", "mailboxIds", "keywords", "receivedAt"].contains(&k.as_str()))
            {
                return Err("invalidProperties");
            }
            let blob = value["blobId"].as_str().ok_or("invalidProperties")?;
            let raw = service.download(token, &account.id, blob).map_err(|e| {
                if e == Error::NotFound {
                    "invalidProperties"
                } else {
                    error(e)
                }
            })?;
            if !super::message::valid(&raw) {
                return Err("invalidEmail");
            }
            let mailboxes = true_keys(&value["mailboxIds"])?;
            let keywords = if value["keywords"].is_null() {
                vec![]
            } else {
                true_keys(&value["keywords"])?
            };
            let received_at = utc_date(&value["receivedAt"])?;
            service
                .execute(
                    token,
                    &current.id,
                    current.revision,
                    vec![Command::Append {
                        mailboxes,
                        raw,
                        keywords,
                        received_at,
                    }],
                )
                .map_err(|e| match e {
                    Error::Invalid | Error::NotFound => "invalidProperties",
                    _ => error(e),
                })
        })();
        match result {
            Ok(account) => {
                response["created"][key] = created(account.messages.last().ok_or("serverFail")?);
                current = account;
            }
            Err(kind) => response["notCreated"][key] = json!({"type":kind}),
        }
    }
    response["newState"] = json!(current.revision.to_string());
    Ok(response)
}

fn patch_keys(
    root: &str,
    original: Vec<String>,
    patch: &serde_json::Map<String, Value>,
) -> Result<Vec<String>, &'static str> {
    let prefix = format!("{root}/");
    if let Some(value) = patch.get(root) {
        if patch.keys().any(|k| k.starts_with(&prefix)) {
            return Err("invalidPatch");
        }
        return true_keys(value);
    }
    let mut keys: std::collections::BTreeSet<_> = original.into_iter().collect();
    for (path, value) in patch {
        if let Some(path) = path.strip_prefix(&prefix) {
            if path.is_empty() || path.contains('/') {
                return Err("invalidPatch");
            }
            let mut decoded = String::new();
            let mut chars = path.chars();
            while let Some(c) = chars.next() {
                decoded.push(if c == '~' {
                    match chars.next() {
                        Some('0') => '~',
                        Some('1') => '/',
                        _ => return Err("invalidPatch"),
                    }
                } else {
                    c
                });
            }
            if value.is_null() {
                keys.remove(&decoded);
            } else if value == true {
                keys.insert(decoded);
            } else {
                return Err("invalidProperties");
            }
        }
    }
    Ok(keys.into_iter().collect())
}

pub(super) fn update(
    service: &MailService,
    token: &str,
    account: &Account,
    id: &str,
    value: &Value,
) -> Result<Account, &'static str> {
    let message = account
        .messages
        .iter()
        .find(|m| m.id == id)
        .ok_or("notFound")?;
    let patch = value.as_object().ok_or("invalidPatch")?;
    if patch.keys().any(|k| {
        !matches!(k.as_str(), "keywords" | "mailboxIds")
            && !k.starts_with("keywords/")
            && !k.starts_with("mailboxIds/")
    }) {
        return Err("invalidProperties");
    }
    let keywords = patch_keys("keywords", message.keywords.clone(), patch)?;
    let mailboxes = patch_keys(
        "mailboxIds",
        message.mailboxes.keys().cloned().collect(),
        patch,
    )?;
    if mailboxes.is_empty()
        || mailboxes
            .iter()
            .any(|id| !account.mailboxes.iter().any(|m| m.id == *id))
    {
        return Err("invalidProperties");
    }
    let mut commands = vec![];
    if keywords != message.keywords {
        commands.push(Command::Keywords {
            id: id.into(),
            keywords,
        });
    }
    if mailboxes != message.mailboxes.keys().cloned().collect::<Vec<_>>() {
        commands.push(Command::SetMailboxes {
            id: id.into(),
            mailboxes,
        });
    }
    service
        .execute(token, &account.id, account.revision, commands)
        .map_err(|e| {
            if e == Error::Invalid {
                "invalidProperties"
            } else {
                error(e)
            }
        })
}
