use mail_parser::{Addr, Address, HeaderForm, HeaderValue, Message};
use serde_json::{Value, json};

fn address(value: &Addr<'_>) -> Value {
    json!({"name": value.name.as_deref().unwrap_or(""), "email": value.address})
}

fn convert(value: &HeaderValue<'_>, form: HeaderForm) -> Value {
    match value {
        HeaderValue::Text(text) if form == HeaderForm::MessageIds => json!([text]),
        HeaderValue::Text(text) => json!(text),
        HeaderValue::TextList(text) => json!(text),
        HeaderValue::DateTime(date) => json!(date.to_timezone(0).to_rfc3339()),
        HeaderValue::Address(values) => match form {
            HeaderForm::URLs => json!(values.iter().filter_map(|v| v.address.as_deref()).collect::<Vec<_>>()),
            HeaderForm::GroupedAddresses => match values {
                Address::List(values) => json!([{"name":null,"addresses":values.iter().map(address).collect::<Vec<_>>()}]),
                Address::Group(groups) => json!(groups.iter().map(|g| json!({"name":g.name,"addresses":g.addresses.iter().map(address).collect::<Vec<_>>()})).collect::<Vec<_>>()),
            },
            _ => json!(values.iter().map(address).collect::<Vec<_>>()),
        },
        _ => Value::Null,
    }
}

pub(super) fn property(property: &str) -> Result<(&str, HeaderForm, bool), &'static str> {
    let mut parts = property
        .strip_prefix("header:")
        .ok_or("invalidArguments")?
        .split(':');
    let name = parts.next().ok_or("invalidArguments")?;
    if name.is_empty() || !name.bytes().all(|b| (33..=126).contains(&b) && b != b':') {
        return Err("invalidArguments");
    }
    let (form, all) = match parts.next() {
        None => (HeaderForm::Raw, false),
        Some("all") => (HeaderForm::Raw, true),
        Some(value) => {
            let form = match value {
                "asRaw" => HeaderForm::Raw,
                "asText" => HeaderForm::Text,
                "asAddresses" => HeaderForm::Addresses,
                "asGroupedAddresses" => HeaderForm::GroupedAddresses,
                "asMessageIds" => HeaderForm::MessageIds,
                "asDate" => HeaderForm::Date,
                "asURLs" => HeaderForm::URLs,
                _ => return Err("invalidArguments"),
            };
            (
                form,
                match parts.next() {
                    None => false,
                    Some("all") => true,
                    _ => return Err("invalidArguments"),
                },
            )
        }
    };
    if parts.next().is_some() {
        return Err("invalidArguments");
    }
    Ok((name, form, all))
}

fn read(parsed: &Message<'_>, name: &str, form: HeaderForm, all: bool) -> Value {
    let mut values = parsed.header_as(name, form);
    if all {
        json!(values.iter().map(|v| convert(v, form)).collect::<Vec<_>>())
    } else {
        values.pop().map_or(Value::Null, |v| convert(&v, form))
    }
}

pub(super) fn render(
    parsed: &Message<'_>,
    args: &Value,
    result: &mut Value,
    remaining: &mut usize,
) -> Result<(), &'static str> {
    for (property, name, form) in [
        ("from", "From", HeaderForm::Addresses),
        ("to", "To", HeaderForm::Addresses),
        ("cc", "Cc", HeaderForm::Addresses),
        ("bcc", "Bcc", HeaderForm::Addresses),
        ("sender", "Sender", HeaderForm::Addresses),
        ("replyTo", "Reply-To", HeaderForm::Addresses),
        ("messageId", "Message-ID", HeaderForm::MessageIds),
        ("inReplyTo", "In-Reply-To", HeaderForm::MessageIds),
        ("references", "References", HeaderForm::MessageIds),
    ] {
        if super::message::requested(args, property) {
            let value = read(parsed, name, form, false);
            super::limits::charge(&value, remaining)?;
            result[property] = value;
        }
    }
    if super::message::requested(args, "headers") {
        let mut headers = vec![];
        for (name, value) in parsed.headers_raw() {
            let header = json!({"name":name, "value":value.trim_end_matches(['\r','\n'])});
            super::limits::charge(&header, remaining)?;
            headers.push(header);
        }
        result["headers"] = Value::Array(headers);
    }
    if let Some(properties) = args["properties"].as_array() {
        for p in properties.iter().filter_map(Value::as_str) {
            if let Ok((name, form, all)) = property(p) {
                let value = read(parsed, name, form, all);
                super::limits::charge(&value, remaining)?;
                result[p] = value;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projection_refusal_stops_before_retaining_more_headers() {
        let raw = format!("X-Long-Header: {}\r\n\r\nbody", "x".repeat(1024));
        let parsed = mail_parser::MessageParser::default().parse(&raw).unwrap();
        let args = json!({"properties":["header:X-Long-Header:asRaw", "header:x-long-header:asRaw", "header:x-LONG-header:asRaw"]});
        let mut result = json!({});
        assert_eq!(render(&parsed, &args, &mut result, &mut 1500), Err("limit"));
        assert_eq!(result.as_object().unwrap().len(), 1);
    }
}
