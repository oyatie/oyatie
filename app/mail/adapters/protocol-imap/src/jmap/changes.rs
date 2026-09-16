use mail_api::{Change, MailboxChange, MessageChange};
use mail_kernel::{Account, Error};
use mail_service::MailService;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(super) fn history(
    service: &MailService,
    token: &str,
    account: &Account,
    since: u64,
) -> Result<Vec<MessageChange>, &'static str> {
    service
        .changes(token, &account.id, since, account.revision)
        .map_err(|e| match e {
            Error::Conflict => "cannotCalculateChanges",
            _ => super::method::error(e),
        })
}

pub(super) fn limit(args: &Value) -> Result<usize, &'static str> {
    if args["maxChanges"].is_null() {
        return Ok(10000);
    }
    let n = args["maxChanges"]
        .as_u64()
        .filter(|n| *n > 0)
        .ok_or("invalidArguments")?;
    Ok(n.min(10000) as usize)
}

fn classify<T>(changes: &[Change<T>]) -> (Vec<&str>, Vec<&str>, Vec<&str>) {
    let mut net = BTreeMap::new();
    for change in changes {
        let pair = net
            .entry(change.id.as_str())
            .or_insert((&change.before, &change.after));
        pair.1 = &change.after;
    }
    let (mut created, mut updated, mut destroyed) = (vec![], vec![], vec![]);
    for (id, (before, after)) in net {
        match (before, after) {
            (None, Some(_)) => created.push(id),
            (Some(_), None) => destroyed.push(id),
            (Some(_), Some(_)) => updated.push(id),
            (None, None) => {}
        }
    }
    (created, updated, destroyed)
}

pub(super) fn changes(
    service: &MailService,
    token: &str,
    account: &Account,
    args: &Value,
) -> Result<Value, &'static str> {
    let state = args["sinceState"].as_str().ok_or("invalidArguments")?;
    let since = state.parse().map_err(|_| "cannotCalculateChanges")?;
    let changes = history(service, token, account, since)?;
    response(account, args, &changes)
}

pub(super) fn mailbox_history(
    service: &MailService,
    token: &str,
    account: &Account,
    since: u64,
) -> Result<Vec<MailboxChange>, &'static str> {
    service
        .mailbox_changes(token, &account.id, since, account.revision)
        .map_err(|e| {
            if e == Error::Conflict {
                "cannotCalculateChanges"
            } else {
                super::method::error(e)
            }
        })
}

pub(super) fn mailbox(
    service: &MailService,
    token: &str,
    account: &Account,
    args: &Value,
) -> Result<Value, &'static str> {
    let since = args["sinceState"]
        .as_str()
        .ok_or("invalidArguments")?
        .parse()
        .map_err(|_| "cannotCalculateChanges")?;
    let changes = mailbox_history(service, token, account, since)?;
    let mut value = response(account, args, &changes)?;
    value["updatedProperties"] = Value::Null;
    Ok(value)
}

pub(super) fn response<T>(
    account: &Account,
    args: &Value,
    changes: &[Change<T>],
) -> Result<Value, &'static str> {
    let state = args["sinceState"].as_str().ok_or("invalidArguments")?;
    let max = limit(args)?;
    let mut end = changes.len();
    loop {
        let (created, updated, destroyed) = classify(&changes[..end]);
        if created.len() + updated.len() + destroyed.len() <= max {
            let more = end < changes.len();
            let next = if more {
                changes[end - 1].revision
            } else {
                account.revision
            };
            return Ok(
                json!({"accountId":account.id,"oldState":state,"newState":next.to_string(),
                "hasMoreChanges":more,"created":created,"updated":updated,"destroyed":destroyed}),
            );
        }
        let revision = changes[end - 1].revision;
        while end > 0 && changes[end - 1].revision == revision {
            end -= 1;
        }
        // An atomic transaction cannot be split into invented intermediate states.
        if end == 0 {
            return Err("cannotCalculateChanges");
        }
    }
}
