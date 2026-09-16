use mail_api::Action;
use mail_service::MailService;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(super) fn set(
    service: &MailService,
    token: &str,
    args: &Value,
    implicit: &mut Option<Value>,
) -> Result<Value, &'static str> {
    validate(args)?;
    let account = args["accountId"].as_str().ok_or("invalidArguments")?;
    let mut revision = service
        .store
        .submissions(account, Some(&[]))
        .map_err(super::method::error)?
        .revision;
    if args["ifInState"]
        .as_str()
        .is_some_and(|s| s != revision.to_string())
    {
        return Err("stateMismatch");
    }
    let mut result = json!({"accountId":account,"oldState":revision.to_string(),"created":{},"updated":{},"destroyed":[],"notCreated":{},"notUpdated":{},"notDestroyed":{}});
    let mut emails = BTreeMap::new();
    if let Some(create) = args["create"].as_object() {
        for (key, value) in create {
            let accepted = super::submission_envelope::request(value).and_then(|request| {
                service
                    .submit_email(token, account, revision, request)
                    .map_err(super::submission_envelope::error)
            });
            match accepted {
                Ok(selection) => {
                    let record = selection.records.first().ok_or("serverFail")?;
                    let value = super::submission_read::value(record);
                    result["created"][key] = json!({"id":value["id"],"sendAt":value["sendAt"],"undoStatus":value["undoStatus"]});
                    emails.insert(key.clone(), record.email_id.clone());
                    revision = selection.revision;
                }
                Err(kind) => result["notCreated"][key] = json!({"type":kind}),
            }
        }
    }
    if let Some(update) = args["update"].as_object() {
        for (id, value) in update {
            let canceled = if args["destroy"]
                .as_array()
                .is_some_and(|ids| ids.iter().any(|v| v == id))
            {
                Err("willDestroy")
            } else if value
                .as_object()
                .is_none_or(|o| o.len() != 1 || value["undoStatus"] != "canceled")
            {
                Err("invalidProperties")
            } else {
                service
                    .authorize(token, account, Action::Submit)
                    .map_err(super::method::error)
                    .and_then(|_| {
                        service
                            .store
                            .cancel_submission(account, revision, id)
                            .map_err(super::submission_read::failure)
                    })
            };
            match canceled {
                Ok(selection) => {
                    result["updated"][id] = Value::Null;
                    revision = selection.revision;
                }
                Err(kind) => result["notUpdated"][id] = json!({"type":kind}),
            }
        }
    }
    if let Some(destroy) = args["destroy"].as_array() {
        for id in destroy
            .iter()
            .filter_map(Value::as_str)
            .collect::<std::collections::BTreeSet<_>>()
        {
            match service
                .authorize(token, account, Action::Submit)
                .and_then(|_| service.store.destroy_submission(account, revision, id))
            {
                Ok(next) => {
                    result["destroyed"]
                        .as_array_mut()
                        .ok_or("serverFail")?
                        .push(json!(id));
                    revision = next;
                }
                Err(error) => {
                    result["notDestroyed"][id] = json!({"type":super::method::error(error)})
                }
            }
        }
    }
    result["newState"] = json!(revision.to_string());
    if result["newState"] != result["oldState"] {
        *implicit = success(args, &emails);
    }
    Ok(result)
}

fn validate(args: &Value) -> Result<(), &'static str> {
    if !args["ifInState"].is_null() && !args["ifInState"].is_string() {
        return Err("invalidArguments");
    }
    for field in ["create", "update", "onSuccessUpdateEmail"] {
        if !args[field].is_null() && !args[field].is_object() {
            return Err("invalidArguments");
        }
    }
    for field in ["destroy", "onSuccessDestroyEmail"] {
        if !args[field].is_null()
            && args[field]
                .as_array()
                .is_none_or(|a| a.iter().any(|v| !v.is_string()))
        {
            return Err("invalidArguments");
        }
    }
    if args["onSuccessUpdateEmail"]
        .as_object()
        .is_some_and(|o| o.values().any(|v| !v.is_object()))
    {
        return Err("invalidArguments");
    }
    if args["create"].as_object().map_or(0, |o| o.len())
        + args["update"].as_object().map_or(0, |o| o.len())
        + args["destroy"].as_array().map_or(0, |a| a.len())
        > 256
        || args["onSuccessUpdateEmail"]
            .as_object()
            .map_or(0, |o| o.len())
            + args["onSuccessDestroyEmail"]
                .as_array()
                .map_or(0, |a| a.len())
            > 256
    {
        return Err("tooManyObjects");
    }
    Ok(())
}

fn success(args: &Value, emails: &BTreeMap<String, String>) -> Option<Value> {
    let resolve = |id: &str| match id.strip_prefix('#') {
        Some(key) => emails.get(key).cloned(),
        None => Some(id.to_owned()),
    };
    let mut update = serde_json::Map::new();
    if let Some(values) = args["onSuccessUpdateEmail"].as_object() {
        for (id, patch) in values {
            if let Some(id) = resolve(id) {
                update.insert(id, patch.clone());
            }
        }
    }
    let destroy: Vec<_> = args["onSuccessDestroyEmail"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter_map(resolve)
        .collect();
    (!update.is_empty() || !destroy.is_empty())
        .then(|| json!({"accountId":args["accountId"],"update":update,"destroy":destroy}))
}
