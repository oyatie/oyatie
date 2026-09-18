use super::set::set;
use mail_kernel::Error;
use mail_service::{Budget, MailService};
use serde_json::{Value, json};

pub(super) fn error(error: Error) -> &'static str {
    match error {
        Error::NotFound => "notFound",
        Error::Forbidden => "forbidden",
        Error::Conflict => "stateMismatch",
        Error::OverQuota => "overQuota",
        Error::Invalid => "invalidArguments",
        Error::Unavailable => "serverFail",
        // Re-run by the request loop; `serverFail` once the budget is spent.
        Error::Busy => super::retry::BUSY,
    }
}

pub(super) fn get_ids(args: &Value) -> Result<Option<Vec<&str>>, &'static str> {
    if args["ids"].is_null() {
        return Ok(None);
    }
    let ids = args["ids"].as_array().ok_or("invalidArguments")?;
    if ids.len() > 256 {
        return Err("tooManyObjects");
    }
    Ok(Some(
        ids.iter()
            .map(|v| v.as_str().ok_or("invalidArguments"))
            .collect::<Result<_, _>>()?,
    ))
}

pub(super) fn get_result(
    account: &str,
    revision: u64,
    args: &Value,
    mut values: Vec<Value>,
) -> Result<Value, &'static str> {
    let ids = get_ids(args)?;
    let mut not_found = vec![];
    if let Some(ids) = ids {
        for id in &ids {
            if !values.iter().any(|v| v["id"] == *id) {
                not_found.push(*id);
            }
        }
        values.retain(|v| ids.iter().any(|id| v["id"] == *id));
    }
    if !args["properties"].is_null() {
        let properties = args["properties"].as_array().ok_or("invalidArguments")?;
        if properties.iter().any(|p| !p.is_string()) {
            return Err("invalidArguments");
        }
        for value in &mut values {
            if let Some(object) = value.as_object_mut() {
                object.retain(|k, _| k == "id" || properties.iter().any(|p| p == k));
            }
        }
    }
    Ok(json!({"accountId":account,"state":revision.to_string(),"list":values,"notFound":not_found}))
}

pub(super) fn method(
    service: &MailService,
    token: &str,
    name: &str,
    args: &Value,
    response_limit: usize,
    implicit: &mut Option<Value>,
    budget: &Budget,
) -> Result<Value, &'static str> {
    if name == "Core/echo" {
        return Ok(args.clone());
    }
    if name.starts_with("EmailSubmission/") {
        let account = args["accountId"].as_str().ok_or("invalidArguments")?;
        service
            .authorize(token, account, mail_api::Action::Read)
            .map_err(|e| {
                if e == Error::Unavailable {
                    "serverFail"
                } else {
                    "accountNotFound"
                }
            })?;
        return if name == "EmailSubmission/set" {
            super::submission_set::set(service, token, args, implicit)
        } else {
            super::submission_read::method(service, token, name, args, response_limit)
        };
    }
    if name == "Blob/copy" {
        return super::blob::copy(service, token, args);
    }
    if name == "SearchSnippet/get" {
        return super::snippet::get(service, token, args, response_limit);
    }
    if name.starts_with("VacationResponse/") {
        return super::vacation::method(service, token, name, args, budget);
    }
    if name == "Email/get" {
        return super::inspect::get(service, token, args, response_limit);
    }
    if name == "Email/parse" {
        return super::inspect::parse(service, token, args, response_limit);
    }
    if !matches!(
        name,
        "Mailbox/get"
            | "Mailbox/set"
            | "Mailbox/changes"
            | "Mailbox/query"
            | "Mailbox/queryChanges"
            | "Email/get"
            | "Email/query"
            | "Email/set"
            | "Email/import"
            | "Email/copy"
            | "Email/changes"
            | "Email/queryChanges"
            | "Identity/get"
            | "Identity/set"
            | "Identity/changes"
            | "Thread/get"
            | "Thread/changes"
    ) {
        return Err("unknownMethod");
    }
    let id = args["accountId"].as_str().ok_or("invalidArguments")?;
    let account = service.read(token, id).map_err(|e| {
        if e == Error::Unavailable {
            "serverFail"
        } else {
            "accountNotFound"
        }
    })?;
    match name {
        "Identity/get" => super::identity::get(&account, args),
        "Identity/set" => super::identity::set(service, token, &account, args, budget),
        "Identity/changes" => super::identity::changes(&account, args),
        "Thread/get" => super::thread::get(&account, args),
        "Thread/changes" => super::thread::changes(service, token, &account, args),
        "Mailbox/query" => super::mailbox_query::query(&account, args),
        "Mailbox/queryChanges" => super::mailbox_query::changes(service, token, &account, args),
        "Email/copy" => super::copy::copy(service, token, &account, args, budget),
        "Email/changes" => super::changes::changes(service, token, &account, args),
        "Email/queryChanges" => super::query::changes(service, token, &account, args),
        "Email/import" => super::email::import(service, token, &account, args, budget),
        "Mailbox/get" => {
            let write = super::mailbox::writable(service, token, id)?;
            get_result(
                &account.id,
                account.revision,
                args,
                account
                    .mailbox_states()
                    .iter()
                    .map(|m| super::mailbox::value(m, write))
                    .collect(),
            )
        }
        "Email/query" => super::query::query(service, token, &account, args),
        "Mailbox/changes" => super::changes::mailbox(service, token, &account, args),
        "Mailbox/set" | "Email/set" => set(service, token, &account, name, args, budget),
        _ => Err("unknownMethod"),
    }
}
