use mail_api::Execution;
use mail_builder::{MessageBuilder, headers::address::Address, mime::MimePart};
use mail_kernel::{Account, Command, MAX_MESSAGE_BYTES, valid_address};
use mail_service::{Budget, MailService};
use serde_json::Value;

fn address(value: &Value) -> Result<Address<'static>, &'static str> {
    let values = value.as_array().ok_or("invalidProperties")?;
    let addresses = values
        .iter()
        .map(|v| {
            let email = v["email"]
                .as_str()
                .filter(|s| valid_address(s))
                .ok_or("invalidProperties")?;
            let name = if v["name"].is_null() {
                None
            } else {
                Some(v["name"].as_str().ok_or("invalidProperties")?.to_owned())
            };
            Ok(Address::new_address(name, email.to_owned()))
        })
        .collect::<Result<Vec<_>, &'static str>>()?;
    Ok(Address::new_list(addresses))
}

fn part(
    service: &MailService,
    token: &str,
    account: &str,
    value: &Value,
    bodies: &Value,
    depth: usize,
    budget: &mut (usize, usize),
) -> Result<MimePart<'static>, &'static str> {
    if depth > 32 || budget.0 == 0 {
        return Err("invalidProperties");
    }
    budget.0 -= 1;
    let typ = value["type"].as_str().ok_or("invalidProperties")?;
    if !super::blob::media_type(typ) {
        return Err("invalidProperties");
    }
    let mut result = if typ.starts_with("multipart/") {
        if !value["blobId"].is_null() || !value["partId"].is_null() {
            return Err("invalidProperties");
        }
        let values = value["subParts"]
            .as_array()
            .filter(|v| !v.is_empty())
            .ok_or("invalidProperties")?;
        let parts = values
            .iter()
            .map(|v| part(service, token, account, v, bodies, depth + 1, budget))
            .collect::<Result<Vec<_>, _>>()?;
        MimePart::new(typ.to_owned(), parts)
    } else if let Some(blob) = value["blobId"].as_str() {
        if !value["partId"].is_null() || !value["subParts"].is_null() {
            return Err("invalidProperties");
        }
        let content =
            super::message::blob(service, token, account, blob).map_err(|_| "blobNotFound")?;
        budget.1 = budget.1.checked_sub(content.len()).ok_or("tooLarge")?;
        MimePart::new(typ.to_owned(), content)
    } else {
        if !value["subParts"].is_null() {
            return Err("invalidProperties");
        }
        let id = value["partId"].as_str().ok_or("invalidProperties")?;
        let body = bodies[id]["value"].as_str().ok_or("invalidProperties")?;
        budget.1 = budget.1.checked_sub(body.len()).ok_or("tooLarge")?;
        MimePart::new(typ.to_owned(), body.to_owned())
    };
    if let Some(name) = value["name"].as_str() {
        result = result.attachment(name.to_owned());
    }
    if value["disposition"] == "inline" {
        result = result.inline();
    }
    if let Some(cid) = value["cid"].as_str() {
        result = result.cid(cid.to_owned());
    }
    Ok(result)
}

pub(super) fn create(
    service: &MailService,
    token: &str,
    account: &Account,
    value: &Value,
    conditional: bool,
    budget: &Budget,
) -> Result<(Execution, Account), &'static str> {
    let object = value.as_object().ok_or("invalidProperties")?;
    if object.keys().any(|k| {
        ![
            "mailboxIds",
            "keywords",
            "receivedAt",
            "from",
            "to",
            "cc",
            "bcc",
            "replyTo",
            "sender",
            "subject",
            "sentAt",
            "bodyStructure",
            "bodyValues",
        ]
        .contains(&k.as_str())
    }) {
        return Err("invalidProperties");
    }
    let mailboxes = super::email::true_keys(&value["mailboxIds"])?;
    let keywords = if value["keywords"].is_null() {
        vec![]
    } else {
        super::email::true_keys(&value["keywords"])?
    };
    let received_at = super::email::utc_date(&value["receivedAt"])?;
    let mut builder = MessageBuilder::new();
    for (name, header) in [
        ("from", "From"),
        ("to", "To"),
        ("cc", "Cc"),
        ("bcc", "Bcc"),
        ("replyTo", "Reply-To"),
        ("sender", "Sender"),
    ] {
        if !value[name].is_null() {
            builder = builder.header(header, address(&value[name])?);
        }
    }
    if !value["subject"].is_null() {
        builder = builder.subject(
            value["subject"]
                .as_str()
                .ok_or("invalidProperties")?
                .to_owned(),
        );
    }
    if !value["sentAt"].is_null() {
        builder = builder.date(super::email::utc_date(&value["sentAt"])?);
    }
    let body = part(
        service,
        token,
        &account.id,
        &value["bodyStructure"],
        &value["bodyValues"],
        0,
        &mut (1000, MAX_MESSAGE_BYTES),
    )?;
    let mut buffer = super::limits::Buffer::new(MAX_MESSAGE_BYTES);
    builder
        .body(body)
        .write_to(&mut buffer)
        .map_err(|_| "tooLarge")?;
    let raw = buffer.bytes;
    let append = service
        .append(token, &account.id, mailboxes, &raw, keywords, received_at)
        .map_err(|_| "serverUnavailable")?;
    super::retry::commit(service, token, account, conditional, vec![append], budget).map_err(|e| {
        if matches!(
            e,
            mail_kernel::Error::Invalid | mail_kernel::Error::NotFound
        ) {
            "invalidProperties"
        } else {
            super::method::error(e)
        }
    })
}
