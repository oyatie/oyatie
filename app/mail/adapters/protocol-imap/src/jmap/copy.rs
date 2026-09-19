use super::{email, method::error, retry};
use mail_kernel::{Account, Command, Error};
use mail_service::{Budget, MailService};
use serde_json::{Value, json};

pub(super) fn copy(
    service: &MailService,
    token: &str,
    target: &Account,
    args: &Value,
    budget: &Budget,
) -> Result<Value, &'static str> {
    let source_id = args["fromAccountId"].as_str().ok_or("invalidArguments")?;
    if source_id == target.id {
        return Err("invalidArguments");
    }
    for key in ["ifFromInState", "ifInState", "destroyFromIfInState"] {
        if !args[key].is_null() && !args[key].is_string() {
            return Err("invalidArguments");
        }
    }
    if !args["onSuccessDestroyOriginal"].is_null() && !args["onSuccessDestroyOriginal"].is_boolean()
    {
        return Err("invalidArguments");
    }
    let source = service.read(token, source_id).map_err(|e| match e {
        Error::Unavailable => "serverFail",
        _ => "fromAccountNotFound",
    })?;
    for (key, revision) in [
        ("ifFromInState", source.revision),
        ("ifInState", target.revision),
    ] {
        if args[key]
            .as_str()
            .is_some_and(|s| s != revision.to_string())
        {
            return Err("stateMismatch");
        }
    }
    let objects = args["create"].as_object().ok_or("invalidArguments")?;
    if objects.len() > 256 {
        return Err("tooManyObjects");
    }
    let conditional = args["ifInState"].is_string();
    let mut current = target.clone();
    let mut committed = false;
    let mut result = json!({"fromAccountId":source.id,"accountId":target.id,
        "oldState":target.revision.to_string(),"created":{},"notCreated":{}});
    for (key, value) in objects {
        let copied = (|| {
            let properties = value.as_object().ok_or("invalidProperties")?;
            if properties
                .keys()
                .any(|k| !["id", "mailboxIds", "keywords", "receivedAt"].contains(&k.as_str()))
            {
                return Err("invalidProperties");
            }
            let id = value["id"].as_str().ok_or("invalidProperties")?;
            let message = source
                .messages
                .iter()
                .find(|m| m.id == id)
                .ok_or("notFound")?;
            let mailboxes = email::true_keys(&value["mailboxIds"])?;
            let keywords = if value["keywords"].is_null() {
                message.keywords.clone()
            } else {
                email::true_keys(&value["keywords"])?
            };
            let received_at = if value["receivedAt"].is_null() {
                message.received_at
            } else {
                email::utc_date(&value["receivedAt"])?
            };
            retry::commit(
                service,
                token,
                &current,
                conditional,
                vec![Command::Append {
                    mailboxes,
                    keywords,
                    received_at,
                    raw: service
                        .download(token, source_id, &message.id)
                        .map_err(error)?,
                }],
                budget,
            )
            .map_err(|e| match e {
                Error::Invalid | Error::NotFound => "invalidProperties",
                _ => error(e),
            })
        })();
        match copied {
            Ok((execution, account)) => {
                result["created"][key] = email::created(email::find(&account, &execution)?);
                current = account;
                committed = true;
            }
            Err(kind) => {
                result["notCreated"][key] = json!({"type":retry::object(kind, committed)?})
            }
        }
    }
    result["newState"] = json!(current.revision.to_string());
    Ok(result)
}

pub(super) fn destroy_args(args: &Value, result: &Value) -> Option<Value> {
    if args["onSuccessDestroyOriginal"] != true {
        return None;
    }
    let ids: std::collections::BTreeSet<_> = result["created"]
        .as_object()?
        .keys()
        .filter_map(|key| args["create"][key]["id"].as_str())
        .collect();
    Some(
        json!({"accountId":args["fromAccountId"],"destroy":ids,"ifInState":args["destroyFromIfInState"]}),
    )
}
