use mail_kernel::{EnvelopeAddress, SubmissionEnvelope};
use mail_service::{SubmitEmail, SubmitEmailError};
use serde_json::Value;
use std::collections::BTreeMap;

pub(super) fn request(value: &Value) -> Result<SubmitEmail, &'static str> {
    let object = value.as_object().ok_or("invalidProperties")?;
    if object
        .keys()
        .any(|key| !matches!(key.as_str(), "identityId" | "emailId" | "envelope"))
    {
        return Err("invalidProperties");
    }
    let identity_id = value["identityId"]
        .as_str()
        .ok_or("invalidProperties")?
        .to_owned();
    let email_id = value["emailId"]
        .as_str()
        .ok_or("invalidProperties")?
        .to_owned();
    let envelope = if value["envelope"].is_null() {
        None
    } else {
        let envelope = &value["envelope"];
        if envelope.as_object().is_none_or(|o| {
            o.keys()
                .any(|k| !matches!(k.as_str(), "mailFrom" | "rcptTo"))
        }) {
            return Err("invalidProperties");
        }
        let mail_from = address(&envelope["mailFrom"], true)?;
        let recipients = envelope["rcptTo"].as_array().ok_or("invalidProperties")?;
        if recipients.len() > 100 {
            return Err("tooManyRecipients");
        }
        let rcpt_to = recipients
            .iter()
            .map(|r| address(r, false))
            .collect::<Result<_, _>>()?;
        Some(SubmissionEnvelope { mail_from, rcpt_to })
    };
    Ok(SubmitEmail {
        identity_id,
        email_id,
        envelope,
    })
}

fn address(value: &Value, sender: bool) -> Result<EnvelopeAddress, &'static str> {
    let object = value.as_object().ok_or("invalidProperties")?;
    if object
        .keys()
        .any(|k| !matches!(k.as_str(), "email" | "parameters"))
    {
        return Err("invalidProperties");
    }
    let email = value["email"]
        .as_str()
        .ok_or("invalidProperties")?
        .trim()
        .to_owned();
    if !mail_kernel::valid_address(&email) {
        return Err("invalidProperties");
    }
    let mut parameters = BTreeMap::new();
    if !value["parameters"].is_null() {
        for (key, value) in value["parameters"].as_object().ok_or("invalidProperties")? {
            let key = key.to_ascii_uppercase();
            // Only extensions actually honored by the queue are accepted.
            if !sender || !matches!(key.as_str(), "HOLDFOR" | "HOLDUNTIL") {
                return Err("invalidProperties");
            }
            let value = value.as_str().ok_or("invalidProperties")?;
            if value.len() > 64
                || value.bytes().any(|b| !b.is_ascii_graphic())
                || parameters.insert(key, Some(value.to_owned())).is_some()
            {
                return Err("invalidProperties");
            }
        }
    }
    Ok(EnvelopeAddress { email, parameters })
}

pub(super) fn error(error: SubmitEmailError) -> &'static str {
    match error {
        SubmitEmailError::Storage(error) => super::method::error(error),
        SubmitEmailError::IdentityNotFound => "invalidProperties",
        SubmitEmailError::EmailNotFound => "emailNotFound",
        SubmitEmailError::NoRecipients => "noRecipients",
        SubmitEmailError::InvalidRecipients => "invalidRecipients",
        SubmitEmailError::ForbiddenFrom => "forbiddenFrom",
        SubmitEmailError::InvalidEmail => "invalidEmail",
        SubmitEmailError::InvalidEnvelope => "invalidProperties",
    }
}
