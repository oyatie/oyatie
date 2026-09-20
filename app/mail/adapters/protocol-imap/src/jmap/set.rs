use super::{
    method::error,
    retry::{commit, object},
};
use mail_kernel::{Account, Command};
use mail_service::{Budget, MailService};
use serde_json::{Value, json};

pub(super) fn set(
    service: &MailService,
    token: &str,
    original: &Account,
    name: &str,
    args: &Value,
    budget: &Budget,
) -> Result<Value, &'static str> {
    if !args["ifInState"].is_null() && !args["ifInState"].is_string() {
        return Err("invalidArguments");
    }
    // Only a client-conditional set may fail with stateMismatch; every
    // other set is re-applied by the store over concurrent commits.
    let conditional = args["ifInState"].is_string();
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
    // Busy before the first commit re-runs the whole method; afterwards the
    // partial result must be reported, so later objects fail individually.
    let mut committed = false;
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
                let outcome = if name == "Email/set" && operation == "create" {
                    super::compose::create(service, token, &account, value, conditional, budget)
                        .map(|(execution, new)| {
                            let message = super::email::find(&new, &execution).ok();
                            (message.map(super::email::created), new)
                        })
                } else if name == "Email/set" && operation == "update" {
                    super::email::update(service, token, &account, key, value, conditional, budget)
                        .map(|new| (Some(Value::Null), new))
                } else {
                    let id = (operation == "update").then_some(key.as_str());
                    super::mailbox::command(&account, id, value).and_then(|c| {
                        commit(service, token, &account, conditional, vec![c], budget)
                            .map_err(super::mailbox::error)
                            .map(|(_, new)| {
                                // `Execution.ids` lists messages only; the new
                                // mailbox is the one the projection gained.
                                let value = if operation == "create" {
                                    let id = new
                                        .mailboxes
                                        .iter()
                                        .find(|m| !account.mailboxes.iter().any(|o| o.id == m.id))
                                        .map(|m| m.id.clone())
                                        .unwrap_or_default();
                                    created_ids.insert(key.clone(), id.clone());
                                    json!({ "id": id })
                                } else {
                                    Value::Null
                                };
                                (Some(value), new)
                            })
                    })
                };
                match outcome {
                    Ok((value, new)) => {
                        result[success][key] = value.ok_or("serverFail")?;
                        account = new;
                        committed = true;
                    }
                    Err(kind) => {
                        result[failure][key] = json!({"type":object(kind, committed)?});
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
                commit(service, token, &account, conditional, vec![command], budget).map_err(error)
            }) {
                Ok((_, new)) => {
                    result["destroyed"]
                        .as_array_mut()
                        .ok_or("serverFail")?
                        .push(json!(id));
                    account = new;
                    committed = true;
                }
                Err(kind) => {
                    result["notDestroyed"][id] = json!({"type":object(kind, committed)?});
                }
            }
        }
    }
    result["newState"] = json!(account.revision.to_string());
    Ok(result)
}
