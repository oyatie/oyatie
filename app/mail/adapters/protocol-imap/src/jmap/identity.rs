use mail_kernel::{Account, Command, IdentitySettings};
use mail_service::{Budget, MailService};
use serde_json::{Value, json};

pub(super) fn get(account: &Account, args: &Value) -> Result<Value, &'static str> {
    if let Some(properties) = args["properties"].as_array()
        && properties.iter().any(|p| {
            !matches!(
                p.as_str(),
                Some(
                    "id" | "name"
                        | "email"
                        | "replyTo"
                        | "bcc"
                        | "textSignature"
                        | "htmlSignature"
                        | "mayDelete"
                )
            )
        })
    {
        return Err("invalidArguments");
    }
    let mut identity = serde_json::to_value(&account.identity).map_err(|_| "serverFail")?;
    identity["id"] = json!(account.id);
    identity["email"] = json!(account.address);
    identity["mayDelete"] = json!(false);
    super::method::get_result(&account.id, account.identity_revision, args, vec![identity])
}

pub(super) fn changes(account: &Account, args: &Value) -> Result<Value, &'static str> {
    super::changes::limit(args)?;
    let state = args["sinceState"].as_str().ok_or("invalidArguments")?;
    let since: u64 = state.parse().map_err(|_| "cannotCalculateChanges")?;
    if since > account.identity_revision || since.to_string() != state {
        return Err("cannotCalculateChanges");
    }
    let updated: Vec<_> = (since < account.identity_revision)
        .then_some(&account.id)
        .into_iter()
        .collect();
    Ok(
        json!({"accountId":account.id,"oldState":state,"newState":account.identity_revision.to_string(),"hasMoreChanges":false,"created":[],"updated":updated,"destroyed":[]}),
    )
}

fn patch(account: &Account, patch: &Value) -> Result<IdentitySettings, &'static str> {
    let object = patch.as_object().ok_or("invalidProperties")?;
    let mut value = serde_json::to_value(&account.identity).map_err(|_| "serverFail")?;
    for (property, update) in object {
        if !matches!(
            property.as_str(),
            "name" | "textSignature" | "htmlSignature" | "replyTo" | "bcc"
        ) {
            return Err("invalidProperties");
        }
        value[property] = update.clone();
    }
    let settings: IdentitySettings =
        serde_json::from_value(value).map_err(|_| "invalidProperties")?;
    settings.validate().map_err(|_| "invalidProperties")?;
    Ok(settings)
}

pub(super) fn set(
    service: &MailService,
    token: &str,
    original: &Account,
    args: &Value,
    budget: &Budget,
) -> Result<Value, &'static str> {
    if !args["ifInState"].is_null() && !args["ifInState"].is_string() {
        return Err("invalidArguments");
    }
    if args["ifInState"]
        .as_str()
        .is_some_and(|s| s != original.identity_revision.to_string())
    {
        return Err("stateMismatch");
    }
    for field in ["create", "update"] {
        if !args[field].is_null() && !args[field].is_object() {
            return Err("invalidArguments");
        }
    }
    if !args["destroy"].is_null()
        && args["destroy"]
            .as_array()
            .is_none_or(|ids| ids.iter().any(|id| !id.is_string()))
    {
        return Err("invalidArguments");
    }
    let count = args["create"].as_object().map_or(0, |v| v.len())
        + args["update"].as_object().map_or(0, |v| v.len())
        + args["destroy"].as_array().map_or(0, |v| v.len());
    if count > 256 {
        return Err("tooManyObjects");
    }
    let mut result = json!({"accountId":original.id,"oldState":original.identity_revision.to_string(),"newState":original.identity_revision.to_string(),"created":{},"updated":{},"destroyed":[],"notCreated":{},"notUpdated":{},"notDestroyed":{}});
    if let Some(create) = args["create"].as_object() {
        for id in create.keys() {
            result["notCreated"][id] = json!({"type":"forbidden"});
        }
    }
    if let Some(update) = args["update"].as_object() {
        for (id, value) in update {
            let updated = if args["destroy"]
                .as_array()
                .is_some_and(|ids| ids.contains(&json!(id)))
            {
                Err("willDestroy")
            } else if id != &original.id {
                Err("notFound")
            } else {
                patch(original, value).and_then(|settings| {
                    super::retry::commit(
                        service,
                        token,
                        original,
                        args["ifInState"].is_string(),
                        vec![Command::SetIdentity { settings }],
                        budget,
                    )
                    .map_err(super::method::error)
                })
            };
            match updated {
                Ok((_, account)) => {
                    result["updated"][id] = Value::Null;
                    result["newState"] = json!(account.identity_revision.to_string());
                }
                Err(kind) => {
                    result["notUpdated"][id] = json!({"type":super::retry::object(kind, false)?})
                }
            }
        }
    }
    if let Some(destroy) = args["destroy"].as_array() {
        for id in destroy.iter().filter_map(Value::as_str) {
            result["notDestroyed"][id] =
                json!({"type":if id == original.id { "forbidden" } else { "notFound" }});
        }
    }
    Ok(result)
}
