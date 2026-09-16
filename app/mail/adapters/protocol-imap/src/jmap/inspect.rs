use super::{headers, message, method::error};
use mail_api::Action;
use mail_kernel::{Error, Message};
use mail_service::MailService;
use serde_json::{Value, json};

pub(super) fn get(
    service: &MailService,
    token: &str,
    args: &Value,
    response_limit: usize,
) -> Result<Value, &'static str> {
    validate(args)?;
    let account = args["accountId"].as_str().ok_or("invalidArguments")?;
    let ids = super::method::get_ids(args)?
        .ok_or("invalidArguments")?
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let selected = service.messages(token, account, &ids).map_err(|e| {
        if e == Error::Unavailable {
            "serverFail"
        } else {
            "accountNotFound"
        }
    })?;
    let needs_content = args["properties"].as_array().is_none_or(|properties| {
        properties.iter().any(|p| {
            !matches!(
                p.as_str(),
                Some(
                    "id" | "blobId"
                        | "threadId"
                        | "mailboxIds"
                        | "keywords"
                        | "size"
                        | "receivedAt"
                )
            )
        })
    });
    let mut remaining = response_limit;
    let emails = selected
        .messages
        .iter()
        .map(|m| {
            let raw = needs_content
                .then(|| service.download(token, account, &m.id))
                .transpose()
                .map_err(|e| {
                    if e == Error::NotFound {
                        "serverFail"
                    } else {
                        error(e)
                    }
                })?;
            let mut email = message::email(m, raw.as_deref(), args, remaining)?;
            if let Some(properties) = args["properties"].as_array() {
                email
                    .as_object_mut()
                    .unwrap()
                    .retain(|k, _| k == "id" || properties.iter().any(|v| v == k));
            }
            super::limits::charge(&email, &mut remaining)?;
            Ok(email)
        })
        .collect::<Result<Vec<_>, &'static str>>()?;
    super::method::get_result(account, selected.revision, args, emails)
}

pub(super) fn validate(args: &Value) -> Result<(), &'static str> {
    for field in ["properties", "bodyProperties"] {
        if args[field].is_null() {
            continue;
        }
        for p in args[field].as_array().ok_or("invalidArguments")? {
            let p = p.as_str().ok_or("invalidArguments")?;
            let supported = if field == "bodyProperties" {
                [
                    "partId",
                    "blobId",
                    "size",
                    "name",
                    "type",
                    "charset",
                    "disposition",
                    "cid",
                    "language",
                    "location",
                    "subParts",
                ]
                .contains(&p)
            } else {
                [
                    "id",
                    "blobId",
                    "threadId",
                    "mailboxIds",
                    "keywords",
                    "size",
                    "receivedAt",
                    "messageId",
                    "inReplyTo",
                    "references",
                    "sender",
                    "from",
                    "to",
                    "cc",
                    "bcc",
                    "replyTo",
                    "subject",
                    "sentAt",
                    "hasAttachment",
                    "preview",
                    "bodyStructure",
                    "textBody",
                    "htmlBody",
                    "attachments",
                    "bodyValues",
                    "headers",
                ]
                .contains(&p)
                    || p.starts_with("header:") && headers::property(p).is_ok()
            };
            if !supported {
                return Err("invalidArguments");
            }
        }
    }
    for field in [
        "fetchTextBodyValues",
        "fetchHTMLBodyValues",
        "fetchAllBodyValues",
    ] {
        if !args[field].is_null() && !args[field].is_boolean() {
            return Err("invalidArguments");
        }
    }
    if !args["maxBodyValueBytes"].is_null() && args["maxBodyValueBytes"].as_u64().is_none() {
        return Err("invalidArguments");
    }
    Ok(())
}

pub(super) fn parse(
    service: &MailService,
    token: &str,
    args: &Value,
    response_limit: usize,
) -> Result<Value, &'static str> {
    validate(args)?;
    let account = args["accountId"].as_str().ok_or("invalidArguments")?;
    service
        .authorize(token, account, Action::Read)
        .map_err(|e| {
            if e == Error::Unavailable {
                "serverFail"
            } else {
                "accountNotFound"
            }
        })?;
    let ids = args["blobIds"].as_array().ok_or("invalidArguments")?;
    if ids.len() > 256 {
        return Err("tooManyObjects");
    }
    if ids.iter().any(|id| !id.is_string()) {
        return Err("invalidArguments");
    }
    let mut remaining = response_limit;
    let mut result = json!({"accountId":account,"parsed":{},"notFound":[],"notParsable":[]});
    for id in ids.iter().filter_map(Value::as_str) {
        let raw = match message::blob(service, token, account, id) {
            Ok(raw) => raw,
            Err(Error::NotFound) => {
                result["notFound"].as_array_mut().unwrap().push(json!(id));
                continue;
            }
            Err(e) => return Err(error(e)),
        };
        if !message::valid(&raw) {
            result["notParsable"]
                .as_array_mut()
                .unwrap()
                .push(json!(id));
            continue;
        }
        let metadata = Message {
            modseq: 1,
            id: id.into(),
            thread: None,
            email_identity: None,
            thread_identity: None,
            mailboxes: Default::default(),
            size: raw.len(),
            keywords: vec![],
            received_at: 0,
        };
        let mut email = match message::email(&metadata, Some(&raw), args, remaining) {
            Ok(email) => email,
            Err("limit") => return Err("limit"),
            Err(_) => {
                result["notParsable"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!(id));
                continue;
            }
        };
        for property in ["id", "threadId", "mailboxIds", "keywords", "receivedAt"] {
            email[property] = Value::Null;
        }
        if let Some(properties) = args["properties"].as_array() {
            email
                .as_object_mut()
                .unwrap()
                .retain(|k, _| properties.iter().any(|v| v == k));
        }
        super::limits::charge(&email, &mut remaining)?;
        result["parsed"][id] = email;
    }
    Ok(result)
}
