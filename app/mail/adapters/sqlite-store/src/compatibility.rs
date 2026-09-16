use mail_kernel::{Account, Error};
use serde_json::{Value, json};

// The legacy row remains byte-for-byte intact until an actual account write.
// This read adapter accepts old snapshots without a destructive startup rewrite.
pub(super) fn decode(state: &str) -> Result<Account, Error> {
    decode_all(state).map(|(account, _)| account)
}

type LegacyBodies = Vec<(String, Vec<u8>)>;
pub(super) fn decode_all(state: &str) -> Result<(Account, LegacyBodies), Error> {
    if let Ok(account) = serde_json::from_str(state) {
        return Ok((account, vec![]));
    }
    let mut value: Value = serde_json::from_str(state).map_err(|_| Error::Unavailable)?;
    let mut bodies = vec![];
    for message in value["messages"].as_array_mut().ok_or(Error::Unavailable)? {
        let object = message.as_object_mut().ok_or(Error::Unavailable)?;
        if let Some(raw) = object.remove("raw") {
            if object.contains_key("size") {
                return Err(Error::Unavailable);
            }
            let raw: Vec<u8> = serde_json::from_value(raw).map_err(|_| Error::Unavailable)?;
            let id = object
                .get("id")
                .and_then(Value::as_str)
                .ok_or(Error::Unavailable)?
                .to_owned();
            object.insert("size".into(), json!(raw.len()));
            bodies.push((id, raw));
        }
        if object.contains_key("mailboxes") {
            if object.contains_key("mailbox") || object.contains_key("uid") {
                return Err(Error::Unavailable);
            }
            continue;
        }
        let mailbox = object.remove("mailbox").ok_or(Error::Unavailable)?;
        let mailbox = mailbox.as_str().ok_or(Error::Unavailable)?;
        let uid = object
            .remove("uid")
            .and_then(|v| v.as_u64())
            .filter(|n| *n > 0 && *n <= u32::MAX as u64)
            .ok_or(Error::Unavailable)?;
        object.insert("mailboxes".into(), json!({mailbox:uid}));
    }
    Ok((
        serde_json::from_value(value).map_err(|_| Error::Unavailable)?,
        bodies,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambiguous_membership_encodings_are_refused() {
        let mut account = Account::new("a", "t", "alice", "alice@example.org").unwrap();
        account
            .apply(mail_kernel::Command::Append {
                mailboxes: vec!["inbox".into()],
                raw: vec![0, 255],
                keywords: vec![],
                received_at: 0,
            })
            .unwrap();
        let mut value = serde_json::to_value(account).unwrap();
        value["messages"][0]["mailbox"] = json!("other");
        value["messages"][0]["uid"] = json!(42);
        assert_eq!(decode(&value.to_string()), Err(Error::Unavailable));
    }

    #[test]
    fn ambiguous_content_encodings_are_refused() {
        let mut account = Account::new("a", "t", "alice", "alice@example.org").unwrap();
        account
            .apply(mail_kernel::Command::Append {
                mailboxes: vec!["inbox".into()],
                raw: vec![255],
                keywords: vec![],
                received_at: 0,
            })
            .unwrap();
        let mut value = serde_json::to_value(account).unwrap();
        value["messages"][0]["raw"] = json!([255]);
        assert_eq!(decode(&value.to_string()), Err(Error::Unavailable));
    }
}
