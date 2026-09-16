use super::method::error;
use mail_kernel::{Account, Command};
use mail_service::MailService;
use serde_json::{Value, json};

pub(super) fn set(
    service: &MailService,
    token: &str,
    original: &Account,
    name: &str,
    args: &Value,
) -> Result<Value, &'static str> {
    if !args["ifInState"].is_null() && !args["ifInState"].is_string() {
        return Err("invalidArguments");
    }
    if args["ifInState"]
        .as_str()
        .is_some_and(|s| s != original.revision.to_string())
    {
        return Err("stateMismatch");
    }
    for key in ["create", "update"] {
        if !args[key].is_null() && !args[key].is_object() {
            return Err("invalidArguments");
        }
    }
    if !args["destroy"].is_null()
        && (!args["destroy"].is_array()
            || args["destroy"]
                .as_array()
                .is_some_and(|a| a.iter().any(|v| !v.is_string())))
    {
        return Err("invalidArguments");
    }
    let count = args["create"].as_object().map_or(0, |a| a.len())
        + args["update"].as_object().map_or(0, |a| a.len())
        + args["destroy"].as_array().map_or(0, |a| a.len());
    if count > 256 {
        return Err("tooManyObjects");
    }
    if name == "Mailbox/set"
        && !args["onDestroyRemoveEmails"].is_null()
        && !args["onDestroyRemoveEmails"].is_boolean()
    {
        return Err("invalidArguments");
    }
    let mut account = original.clone();
    let mut created_ids = std::collections::BTreeMap::new();
    let mut result = json!({"accountId":original.id,"oldState":original.revision.to_string(),"created":{},"updated":{},"destroyed":[],"notCreated":{},"notUpdated":{},"notDestroyed":{}});
    for (operation, success, failure) in [
        ("create", "created", "notCreated"),
        ("update", "updated", "notUpdated"),
    ] {
        if let Some(objects) = args[operation].as_object() {
            let entries = if name == "Mailbox/set" && operation == "create" {
                super::references::mailbox_creations(objects)
            } else {
                objects.iter().collect()
            };
            for (key, value) in entries {
                let mut resolved = value.clone();
                super::references::created(&mut resolved, &created_ids);
                let value = &resolved;
                if name == "Email/set" && operation == "create" {
                    match super::compose::create(service, token, &account, value) {
                        Ok(new) => {
                            result[success][key] =
                                super::email::created(new.messages.last().ok_or("serverFail")?);
                            account = new;
                        }
                        Err(kind) => result[failure][key] = json!({"type":kind}),
                    }
                    continue;
                }
                if name == "Email/set" && operation == "update" {
                    match super::email::update(service, token, &account, key, value) {
                        Ok(new) => {
                            result[success][key] = Value::Null;
                            account = new;
                        }
                        Err(kind) => result[failure][key] = json!({"type":kind}),
                    }
                    continue;
                }
                let command = super::mailbox::command(
                    &account,
                    if operation == "create" {
                        None
                    } else {
                        Some(key)
                    },
                    value,
                );
                match command.and_then(|c| {
                    service
                        .execute(token, &account.id, account.revision, vec![c])
                        .map_err(super::mailbox::error)
                }) {
                    Ok(new) => {
                        result[success][key] = if operation == "create" {
                            created_ids.insert(key.clone(), format!("m{}", new.revision));
                            json!({"id":format!("m{}",new.revision)})
                        } else {
                            Value::Null
                        };
                        account = new;
                    }
                    Err(e) => {
                        result[failure][key] = json!({"type":e});
                    }
                }
            }
        }
    }
    if let Some(ids) = args["destroy"].as_array() {
        let mut ids: Vec<_> = ids
            .iter()
            .map(|id| id.as_str().ok_or("invalidArguments"))
            .collect::<Result<std::collections::BTreeSet<_>, _>>()?
            .into_iter()
            .collect();
        if name == "Mailbox/set" {
            ids.sort_by_key(|id| {
                std::cmp::Reverse(
                    account
                        .mailbox_path(id)
                        .map_or(0, |path| path.split('/').count()),
                )
            });
        }
        for id in ids {
            let command = if name == "Mailbox/set" {
                super::mailbox::destroy(&account, id, args["onDestroyRemoveEmails"] == true)
            } else {
                Ok(Command::Destroy { id: id.into() })
            };
            match command.and_then(|command| {
                service
                    .execute(token, &account.id, account.revision, vec![command])
                    .map_err(error)
            }) {
                Ok(new) => {
                    result["destroyed"]
                        .as_array_mut()
                        .ok_or("serverFail")?
                        .push(json!(id));
                    account = new;
                }
                Err(e) => {
                    result["notDestroyed"][id] = json!({"type":e});
                }
            }
        }
    }
    result["newState"] = json!(account.revision.to_string());
    Ok(result)
}
