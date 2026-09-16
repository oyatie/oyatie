use mail_kernel::{
    SubmissionFilter as Filter, SubmissionQuery, SubmissionSortField as Field, UndoStatus,
};
use mail_service::MailService;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

fn filter(value: &Value, depth: usize, budget: &mut usize) -> Result<Filter, &'static str> {
    if depth > 16 || *budget == 0 {
        return Err("unsupportedFilter");
    }
    *budget -= 1;
    if value.is_null() {
        return Ok(Filter::All);
    }
    let object = value.as_object().ok_or("invalidArguments")?;
    if let Some(operator) = object.get("operator") {
        if object
            .keys()
            .any(|key| !["operator", "conditions"].contains(&key.as_str()))
        {
            return Err("unsupportedFilter");
        }
        let conditions = value["conditions"].as_array().ok_or("invalidArguments")?;
        if conditions.len() > 256 {
            return Err("unsupportedFilter");
        }
        let conditions = conditions
            .iter()
            .map(|value| filter(value, depth + 1, budget))
            .collect::<Result<Vec<_>, _>>()?;
        return match operator.as_str().ok_or("invalidArguments")? {
            "AND" => Ok(Filter::And(conditions)),
            "OR" => Ok(Filter::Or(conditions)),
            "NOT" => Ok(Filter::Not(conditions)),
            _ => Err("invalidArguments"),
        };
    }
    let mut conditions = Vec::new();
    for (key, value) in object {
        conditions.push(match key.as_str() {
            "identityIds" | "emailIds" | "threadIds" => {
                let ids = super::submission_read::strings(value, 256)?;
                match key.as_str() {
                    "identityIds" => Filter::IdentityIds(ids),
                    "emailIds" => Filter::EmailIds(ids),
                    _ => Filter::ThreadIds(ids),
                }
            }
            "undoStatus" => Filter::UndoStatus(match value.as_str().ok_or("invalidArguments")? {
                "pending" => UndoStatus::Pending,
                "final" => UndoStatus::Final,
                "canceled" => UndoStatus::Canceled,
                _ => return Err("invalidArguments"),
            }),
            "before" | "after" => {
                if !value.is_string() {
                    return Err("invalidArguments");
                }
                let date = super::email::utc_date(value).map_err(|_| "invalidArguments")?;
                if key == "before" {
                    Filter::Before(date)
                } else {
                    Filter::After(date)
                }
            }
            _ => return Err("unsupportedFilter"),
        });
    }
    Ok(match conditions.len() {
        0 => Filter::All,
        1 => conditions.remove(0),
        _ => Filter::And(conditions),
    })
}

fn parse(account: &str, args: &Value) -> Result<(SubmissionQuery, String), &'static str> {
    let filter = filter(&args["filter"], 0, &mut 256)?;
    let mut sort = Vec::new();
    if !args["sort"].is_null() {
        let values = args["sort"].as_array().ok_or("invalidArguments")?;
        if values.len() > 3 {
            return Err("unsupportedSort");
        }
        for value in values {
            let object = value.as_object().ok_or("invalidArguments")?;
            if object
                .keys()
                .any(|key| !["property", "isAscending", "collation"].contains(&key.as_str()))
                || (!value["collation"].is_null() && value["collation"] != "")
            {
                return Err("unsupportedSort");
            }
            let field = match value["property"].as_str().ok_or("invalidArguments")? {
                "emailId" => Field::EmailId,
                "threadId" => Field::ThreadId,
                "sentAt" => Field::SentAt,
                _ => return Err("unsupportedSort"),
            };
            let ascending = if value["isAscending"].is_null() {
                true
            } else {
                value["isAscending"].as_bool().ok_or("invalidArguments")?
            };
            if sort.iter().any(|(existing, _)| *existing == field) {
                return Err("unsupportedSort");
            }
            sort.push((field, ascending));
        }
    }
    if sort.is_empty() {
        sort.push((Field::SentAt, true));
    }
    let fingerprint = format!(
        "{:x}",
        Sha256::digest(format!("{account:?}:{filter:?}:{sort:?}"))
    );
    let position = integer(args, "position")?;
    let anchor_offset = integer(args, "anchorOffset")?;
    let anchor = if args["anchor"].is_null() {
        None
    } else {
        Some(
            args["anchor"]
                .as_str()
                .ok_or("invalidArguments")?
                .to_owned(),
        )
    };
    let limit = if args["limit"].is_null() {
        256
    } else {
        args["limit"].as_u64().ok_or("invalidArguments")?.min(256) as usize
    };
    boolean(args, "calculateTotal")?;
    Ok((
        SubmissionQuery {
            filter,
            sort,
            position,
            anchor,
            anchor_offset,
            limit,
        },
        fingerprint,
    ))
}

fn integer(args: &Value, name: &str) -> Result<i64, &'static str> {
    if args[name].is_null() {
        Ok(0)
    } else {
        args[name].as_i64().ok_or("invalidArguments")
    }
}
fn boolean(args: &Value, name: &str) -> Result<bool, &'static str> {
    if args[name].is_null() {
        Ok(false)
    } else {
        args[name].as_bool().ok_or("invalidArguments")
    }
}
fn state(revision: u64, fingerprint: &str) -> String {
    format!("{revision}:{fingerprint}")
}

pub(super) fn query(
    service: &MailService,
    account: &str,
    args: &Value,
) -> Result<Value, &'static str> {
    let (query, fingerprint) = parse(account, args)?;
    let page = service
        .store
        .query_submissions(account, None, &query)
        .map_err(super::submission_read::failure)?;
    if page.ids.len() > query.limit || page.ids.len() > page.total.saturating_sub(page.position) {
        return Err("serverFail");
    }
    let mut value = json!({"accountId":account,"queryState":state(page.revision, &fingerprint),"canCalculateChanges":true,"position":page.position,"ids":page.ids});
    if boolean(args, "calculateTotal")? {
        value["total"] = json!(page.total);
    }
    Ok(value)
}

pub(super) fn changes(
    service: &MailService,
    account: &str,
    args: &Value,
) -> Result<Value, &'static str> {
    let (mut query, fingerprint) = parse(account, args)?;
    let old_state = args["sinceQueryState"].as_str().ok_or("invalidArguments")?;
    let (revision, previous_fingerprint) =
        old_state.split_once(':').ok_or("cannotCalculateChanges")?;
    if fingerprint != previous_fingerprint {
        return Err("cannotCalculateChanges");
    }
    let revision = revision
        .parse::<u64>()
        .map_err(|_| "cannotCalculateChanges")?;
    let max = super::changes::limit(args)?;
    let up_to = if args["upToId"].is_null() {
        None
    } else {
        Some(args["upToId"].as_str().ok_or("invalidArguments")?)
    };
    query.position = 0;
    query.anchor = None;
    query.anchor_offset = 0;
    query.limit = 10000;
    let current = service
        .store
        .query_submissions(account, None, &query)
        .map_err(history_failure)?;
    if revision > current.revision {
        return Err("cannotCalculateChanges");
    }
    let before = service
        .store
        .query_submissions(account, Some(revision), &query)
        .map_err(history_failure)?;
    if before.revision != revision
        || current.total > 10000
        || before.total > 10000
        || current.ids.len() != current.total
        || before.ids.len() != before.total
    {
        return Err("cannotCalculateChanges");
    }
    let before_set: BTreeSet<_> = before.ids.iter().collect();
    let after_set: BTreeSet<_> = current.ids.iter().collect();
    let removed: Vec<_> = before
        .ids
        .iter()
        .filter(|id| !after_set.contains(id))
        .collect();
    let mut added = Vec::new();
    // All supported sort fields are immutable, so retained IDs keep relative order.
    for (index, id) in current.ids.iter().enumerate() {
        if !before_set.contains(id) {
            added.push(json!({"id":id,"index":index}));
        }
        if up_to == Some(id.as_str()) {
            break;
        }
    }
    if removed.len() + added.len() > max {
        return Err("tooManyChanges");
    }
    let mut value = json!({"accountId":account,"oldQueryState":old_state,"newQueryState":state(current.revision, &fingerprint),"removed":removed,"added":added});
    if boolean(args, "calculateTotal")? {
        value["total"] = json!(current.total);
    }
    Ok(value)
}

fn history_failure(error: mail_api::SubmissionFailure) -> &'static str {
    match error {
        mail_api::SubmissionFailure::Storage(error) => super::submission_read::history_error(error),
        _ => "cannotCalculateChanges",
    }
}
