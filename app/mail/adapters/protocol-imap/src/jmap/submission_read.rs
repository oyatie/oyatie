use mail_api::SubmissionFailure;
use mail_kernel::{EnvelopeAddress, Error, SubmissionRecord, UndoStatus};
use mail_service::MailService;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const PROPERTIES: &[&str] = &[
    "id",
    "identityId",
    "emailId",
    "threadId",
    "envelope",
    "sendAt",
    "undoStatus",
    "deliveryStatus",
    "dsnBlobIds",
    "mdnBlobIds",
];

pub(super) fn failure(error: SubmissionFailure) -> &'static str {
    match error {
        SubmissionFailure::Storage(error) => super::method::error(error),
        SubmissionFailure::CannotUnsend => "cannotUnsend",
        SubmissionFailure::AnchorNotFound => "anchorNotFound",
    }
}

fn address(address: &EnvelopeAddress) -> Value {
    json!({"email":address.email,"parameters":if address.parameters.is_empty() { Value::Null } else { json!(address.parameters) }})
}

pub(super) fn value(record: &SubmissionRecord) -> Value {
    let delivery: BTreeMap<_, _> = record
        .delivery_status
        .iter()
        .map(|(recipient, status)| {
            let delivered = match status.delivered {
                mail_kernel::SubmissionDelivered::Unknown => "unknown",
                mail_kernel::SubmissionDelivered::Queued => "queued",
                mail_kernel::SubmissionDelivered::Yes => "yes",
                mail_kernel::SubmissionDelivered::No => "no",
            };
            (
                recipient,
                json!({"smtpReply":status.smtp_reply,"delivered":delivered,"displayed":"unknown"}),
            )
        })
        .collect();
    json!({
        "id":record.id,"identityId":record.identity_id,"emailId":record.email_id,
        "threadId":record.thread_id,
        "envelope":{"mailFrom":address(&record.envelope.mail_from),"rcptTo":record.envelope.rcpt_to.iter().map(address).collect::<Vec<_>>()},
        "sendAt":mail_parser::DateTime::from_timestamp(record.send_at).to_rfc3339(),
        "undoStatus":match record.undo_status { UndoStatus::Pending=>"pending",UndoStatus::Final=>"final",UndoStatus::Canceled=>"canceled" },
        // Nothing in the tree produces an MDN, so mdnBlobIds has no source.
        "deliveryStatus":delivery,"dsnBlobIds":record.dsn_blob_ids,"mdnBlobIds":[]
    })
}

pub(super) fn method(
    service: &MailService,
    _token: &str,
    name: &str,
    args: &Value,
    mut response_limit: usize,
) -> Result<Value, &'static str> {
    let account = args["accountId"].as_str().ok_or("invalidArguments")?;
    let result = match name {
        "EmailSubmission/get" => get(service, account, args),
        "EmailSubmission/changes" => changes(service, account, args),
        "EmailSubmission/query" => super::submission_query::query(service, account, args),
        "EmailSubmission/queryChanges" => super::submission_query::changes(service, account, args),
        _ => Err("unknownMethod"),
    }?;
    super::limits::charge(&result, &mut response_limit)?;
    Ok(result)
}

pub(super) fn strings(value: &Value, max: usize) -> Result<Vec<String>, &'static str> {
    let values = value.as_array().ok_or("invalidArguments")?;
    if values.len() > max {
        return Err("tooManyObjects");
    }
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .filter(|s| !s.is_empty() && s.len() <= 256)
                .map(str::to_owned)
                .ok_or("invalidArguments")
        })
        .collect()
}

fn get(service: &MailService, account: &str, args: &Value) -> Result<Value, &'static str> {
    let ids = if args["ids"].is_null() {
        None
    } else {
        Some(strings(&args["ids"], 256)?)
    };
    let properties = if args["properties"].is_null() {
        None
    } else {
        let names = strings(&args["properties"], PROPERTIES.len())?;
        if names
            .iter()
            .any(|name| !PROPERTIES.contains(&name.as_str()))
        {
            return Err("invalidArguments");
        }
        Some(names)
    };
    let selection = service
        .store
        .submissions(account, ids.as_deref())
        .map_err(|error| {
            if error == Error::OverQuota {
                "tooManyObjects"
            } else {
                super::method::error(error)
            }
        })?;
    if selection.records.len() > 256 {
        return Err("tooManyObjects");
    }
    let mut records: BTreeMap<_, _> = selection
        .records
        .iter()
        .map(|record| (record.id.as_str(), record))
        .collect();
    let order = ids.unwrap_or_else(|| records.keys().map(|id| (*id).to_owned()).collect());
    let mut list = Vec::new();
    let mut missing = Vec::new();
    let mut seen = BTreeSet::new();
    for id in order {
        if !seen.insert(id.clone()) {
            continue;
        }
        match records.remove(id.as_str()) {
            Some(record) => {
                let mut object = value(record);
                if let Some(properties) = &properties {
                    object
                        .as_object_mut()
                        .ok_or("serverFail")?
                        .retain(|key, _| key == "id" || properties.contains(key));
                }
                list.push(object);
            }
            None => missing.push(id),
        }
    }
    Ok(
        json!({"accountId":account,"state":selection.revision.to_string(),"list":list,"notFound":missing}),
    )
}

pub(super) fn history_error(error: Error) -> &'static str {
    match error {
        Error::Conflict | Error::OverQuota => "cannotCalculateChanges",
        _ => super::method::error(error),
    }
}

fn changes(service: &MailService, account: &str, args: &Value) -> Result<Value, &'static str> {
    let state = args["sinceState"].as_str().ok_or("invalidArguments")?;
    let since = state.parse::<u64>().map_err(|_| "cannotCalculateChanges")?;
    let max = super::changes::limit(args)?;
    let history = service
        .store
        .submission_changes(account, since, max)
        .map_err(history_error)?;
    if history.revision < since {
        return Err("cannotCalculateChanges");
    }
    let mut net = BTreeMap::new();
    for change in &history.changes {
        if change.revision <= since || change.revision > history.revision {
            return Err("cannotCalculateChanges");
        }
        let pair = net
            .entry(&change.id)
            .or_insert((change.before.is_some(), false));
        pair.1 = change.after.is_some();
    }
    let (mut created, mut updated, mut destroyed) = (vec![], vec![], vec![]);
    for (id, (before, after)) in net {
        match (before, after) {
            (false, true) => created.push(id),
            (true, true) => updated.push(id),
            (true, false) => destroyed.push(id),
            (false, false) => {}
        }
    }
    if created.len() + updated.len() + destroyed.len() > max
        || (history.has_more && history.revision == since)
    {
        return Err("cannotCalculateChanges");
    }
    Ok(
        json!({"accountId":account,"oldState":state,"newState":history.revision.to_string(),
        "hasMoreChanges":history.has_more,"created":created,"updated":updated,"destroyed":destroyed}),
    )
}
