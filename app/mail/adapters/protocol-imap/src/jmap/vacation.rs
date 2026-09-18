use mail_kernel::{Account, Command, Error, VacationSettings};
use mail_service::{Budget, MailService};
use serde_json::{Value, json};

const ID: &str = "singleton";

pub(super) fn method(
    service: &MailService,
    token: &str,
    name: &str,
    args: &Value,
    budget: &Budget,
) -> Result<Value, &'static str> {
    let id = args["accountId"].as_str().ok_or("invalidArguments")?;
    let account = service.read(token, id).map_err(|error| {
        if error == Error::Unavailable {
            "serverFail"
        } else {
            "accountNotFound"
        }
    })?;
    match name {
        "VacationResponse/get" => get(&account, args),
        "VacationResponse/set" => set(service, token, &account, args, budget),
        _ => Err("unknownMethod"),
    }
}

fn get(account: &Account, args: &Value) -> Result<Value, &'static str> {
    if let Some(properties) = args["properties"].as_array()
        && properties.iter().any(|p| {
            !matches!(
                p.as_str(),
                Some(
                    "id" | "isEnabled"
                        | "fromDate"
                        | "toDate"
                        | "subject"
                        | "textBody"
                        | "htmlBody"
                )
            )
        })
    {
        return Err("invalidArguments");
    }
    let mut value = serde_json::to_value(&account.vacation).map_err(|_| "serverFail")?;
    value["id"] = json!(ID);
    super::method::get_result(&account.id, account.vacation_revision, args, vec![value])
}

fn patch(account: &Account, patch: &Value) -> Result<VacationSettings, &'static str> {
    let object = patch.as_object().ok_or("invalidProperties")?;
    let mut value = serde_json::to_value(&account.vacation).map_err(|_| "serverFail")?;
    for (property, update) in object {
        match property.as_str() {
            "isEnabled" | "fromDate" | "toDate" | "subject" | "textBody" | "htmlBody" => {
                value[property] = update.clone();
            }
            "id" if update.as_str() == Some(ID) => {}
            _ => return Err("invalidProperties"),
        }
    }
    if value["isEnabled"].is_null() {
        value["isEnabled"] = json!(false);
    }
    let settings: VacationSettings =
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
        .is_some_and(|s| s != original.vacation_revision.to_string())
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
    let created = args["create"].as_object().map_or(0, |v| v.len());
    let updated = args["update"].as_object().map_or(0, |v| v.len());
    if created + updated + args["destroy"].as_array().map_or(0, |v| v.len()) > 256 {
        return Err("tooManyObjects");
    }
    if created > 0 && updated > 0 {
        return Err("invalidArguments");
    }
    let mut result = json!({"accountId":original.id,"oldState":original.vacation_revision.to_string(),"newState":original.vacation_revision.to_string(),"created":{},"updated":{},"destroyed":[],"notCreated":{},"notUpdated":{},"notDestroyed":{}});
    if let Some(create) = args["create"].as_object() {
        for id in create.keys() {
            result["notCreated"][id] = json!({"type":"singleton"});
        }
    }
    if let Some(update) = args["update"].as_object() {
        for (id, value) in update {
            let updated = if id != ID {
                Err("notFound")
            } else {
                patch(original, value).and_then(|settings| {
                    super::retry::commit(
                        service,
                        token,
                        original,
                        args["ifInState"].is_string(),
                        vec![Command::SetVacation { settings }],
                        budget,
                    )
                    .map_err(super::method::error)
                })
            };
            match updated {
                Ok((_, account)) => {
                    result["updated"][id] = Value::Null;
                    result["newState"] = json!(account.vacation_revision.to_string());
                }
                Err(kind) => {
                    result["notUpdated"][id] = json!({"type":super::retry::object(kind, false)?})
                }
            }
        }
    }
    if let Some(destroy) = args["destroy"].as_array() {
        for id in destroy.iter().filter_map(Value::as_str) {
            result["notDestroyed"][id] = json!({"type":"singleton"});
        }
    }
    Ok(result)
}
