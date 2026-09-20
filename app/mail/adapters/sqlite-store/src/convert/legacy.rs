//! The two `state` shapes `41ce37e61` wrote: the oldest embeds `raw` bodies
//! and a single `mailbox`/`uid`; the later one carries `size` and a
//! `mailboxes` map with bodies in `message_bodies`. Ambiguous mixtures are
//! refused, as the retired lazy decoder did.
use mail_kernel::{Error, IdentitySettings, VacationSettings};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
pub struct LegacyAccount {
    pub id: String,
    pub tenant: String,
    pub owner: String,
    pub address: String,
    pub revision: u64,
    #[serde(default, deserialize_with = "present")]
    pub mail_modseq: Option<u64>,
    #[serde(default)]
    pub identity: IdentitySettings,
    #[serde(default)]
    pub identity_revision: u64,
    #[serde(default)]
    pub vacation: VacationSettings,
    #[serde(default)]
    pub vacation_revision: u64,
    pub quota_bytes: usize,
    pub mailboxes: Vec<LegacyMailbox>,
    pub messages: Vec<LegacyMessage>,
}

#[derive(Debug, Deserialize)]
pub struct LegacyMailbox {
    pub id: String,
    pub name: String,
    pub role: Option<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub sort_order: u32,
    #[serde(default = "yes")]
    pub is_subscribed: bool,
    pub uid_next: u32,
    pub uid_validity: u32,
}
fn yes() -> bool {
    true
}
/// An absent watermark is legacy; a present malformed one is refused.
fn present<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<u64>, D::Error> {
    u64::deserialize(deserializer).map(Some)
}

#[derive(Debug, Deserialize)]
pub struct LegacyMessage {
    pub id: String,
    #[serde(default)]
    pub modseq: Option<u64>,
    #[serde(default)]
    pub thread: Option<String>,
    #[serde(default)]
    pub email_identity: Option<String>,
    #[serde(default)]
    pub thread_identity: Option<String>,
    #[serde(default)]
    pub mailboxes: Option<BTreeMap<String, u32>>,
    #[serde(default)]
    pub mailbox: Option<String>,
    #[serde(default)]
    pub uid: Option<u32>,
    #[serde(default)]
    pub size: Option<usize>,
    #[serde(default)]
    pub raw: Option<Vec<u8>>,
    pub keywords: Vec<String>,
    #[serde(default)]
    pub received_at: i64,
}

impl LegacyMessage {
    /// Mailbox links in either encoding; a row carrying both is refused.
    pub fn links(&self) -> Result<BTreeMap<String, u32>, Error> {
        match (&self.mailboxes, &self.mailbox, self.uid) {
            (Some(map), None, None) => Ok(map.clone()),
            (None, Some(mailbox), Some(uid)) if uid > 0 => {
                Ok(BTreeMap::from([(mailbox.clone(), uid)]))
            }
            _ => Err(Error::Unavailable),
        }
    }
    /// Size and embedded body; a row carrying both is refused.
    pub fn content(&self) -> Result<(usize, Option<&[u8]>), Error> {
        match (self.size, &self.raw) {
            (Some(size), None) => Ok((size, None)),
            (None, Some(raw)) => Ok((raw.len(), Some(raw))),
            _ => Err(Error::Unavailable),
        }
    }
}

impl LegacyAccount {
    pub fn decode(state: &str) -> Result<Self, Error> {
        let account: Self = serde_json::from_str(state).map_err(|_| Error::Unavailable)?;
        for message in &account.messages {
            message.links()?;
            message.content()?;
        }
        Ok(account)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn state(message: serde_json::Value) -> String {
        json!({"id":"a","tenant":"t","owner":"alice","address":"alice@example.org","revision":3,
            "quota_bytes":1000,"mailboxes":[{"id":"inbox","name":"INBOX","role":"inbox","uid_next":2,"uid_validity":1}],
            "messages":[message]})
        .to_string()
    }

    #[test]
    fn both_shapes_decode_and_ambiguous_encodings_are_refused() {
        let oldest =
            state(json!({"id":"e1","mailbox":"inbox","uid":1,"raw":[0,255],"keywords":[]}));
        let account = LegacyAccount::decode(&oldest).unwrap();
        assert_eq!(account.messages[0].links().unwrap()["inbox"], 1);
        assert_eq!(
            account.messages[0].content().unwrap(),
            (2, Some(&[0u8, 255][..]))
        );
        assert_eq!(account.mail_modseq, None);
        let newer = state(
            json!({"id":"e1","modseq":4,"mailboxes":{"inbox":1},"size":2,"keywords":["$seen"],"received_at":5}),
        );
        let account = LegacyAccount::decode(&newer).unwrap();
        assert_eq!(account.messages[0].content().unwrap(), (2, None));
        assert_eq!(account.messages[0].modseq, Some(4));
        for ambiguous in [
            json!({"id":"e1","mailboxes":{"inbox":1},"mailbox":"other","uid":42,"size":2,"keywords":[]}),
            json!({"id":"e1","mailbox":"inbox","uid":1,"raw":[255],"size":1,"keywords":[]}),
            json!({"id":"e1","mailbox":"inbox","uid":0,"raw":[255],"keywords":[]}),
        ] {
            assert_eq!(
                LegacyAccount::decode(&state(ambiguous)).map(|_| ()),
                Err(Error::Unavailable)
            );
        }
    }

    #[test]
    fn missing_identity_defaults_and_invalid_present_fields_fail_closed() {
        let base = state(json!({"id":"e1","mailboxes":{"inbox":1},"size":2,"keywords":[]}));
        assert_eq!(
            LegacyAccount::decode(&base).unwrap().identity,
            IdentitySettings::default()
        );
        for (field, invalid) in [
            ("identity", json!(null)),
            ("identity", json!({"name":null})),
            ("identity", json!({"unexpected":true})),
            ("identity_revision", json!(-1)),
            ("identity_revision", json!("0")),
            ("mail_modseq", json!("0")),
            ("mail_modseq", json!(-1)),
        ] {
            let mut value: serde_json::Value = serde_json::from_str(&base).unwrap();
            value[field] = invalid;
            assert!(
                LegacyAccount::decode(&value.to_string()).is_err(),
                "{field}"
            );
        }
    }
}
