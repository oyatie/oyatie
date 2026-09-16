use crate::Error;
use crate::conversation::{ReactionChange, ReactionSummary};
use serde::{Deserialize, Serialize};

pub const MAX_MESSAGE_BYTES: usize = 32_768;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Decryption {
    Decrypted,
    MissingKey,
    Plaintext,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attachment {
    pub filename: String,
    pub mime: String,
    pub size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaFile {
    pub filename: String,
    pub mime: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub sender: String,
    pub timestamp: u64,
    pub body: String,
    pub decryption: Decryption,
    #[serde(default)]
    pub attachment: Option<Attachment>,
    #[serde(default)]
    pub reply_to: Option<String>,
    #[serde(default)]
    pub edit_of: Option<String>,
    #[serde(default)]
    pub redacted: bool,
    #[serde(default)]
    pub thread_root: Option<String>,
    #[serde(default)]
    pub reaction: Option<ReactionChange>,
    #[serde(default)]
    pub reactions: Vec<ReactionSummary>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomDelta {
    pub room: String,
    pub messages: Vec<Message>,
    pub limited: bool,
    pub previous: Option<String>,
    pub redacted: Vec<String>,
    #[serde(default)]
    pub typing: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SyncUpdate {
    #[serde(default)]
    pub call_keys: Vec<crate::CallKey>,
    pub rooms: Vec<RoomDelta>,
    pub keys_changed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessagePage {
    pub messages: Vec<Message>,
    pub next: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutgoingMessage {
    pub transaction: String,
    pub body: String,
    pub failed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_detail: Option<String>,
}

pub fn message_body(body: &str) -> Result<&str, Error> {
    let body = body.trim();
    if body.is_empty() || body.len() > MAX_MESSAGE_BYTES {
        return Err(Error::Invalid(
            "message must contain 1–32768 UTF-8 bytes".into(),
        ));
    }
    Ok(body)
}

pub fn merge_messages(current: &mut Vec<Message>, incoming: Vec<Message>, redacted: &[String]) {
    let removed: std::collections::BTreeSet<_> = current
        .iter()
        .filter(|m| m.redacted)
        .map(|m| m.id.clone())
        .chain(redacted.iter().cloned())
        .collect();
    let mut merged: std::collections::BTreeMap<_, _> =
        current.drain(..).map(|m| (m.id.clone(), m)).collect();
    for message in incoming {
        merged.insert(message.id.clone(), message);
    }
    for id in removed {
        if let Some(message) = merged.get_mut(&id) {
            message.redacted = true;
            message.body = "Message removed".into();
            message.attachment = None;
        }
    }
    *current = merged.into_values().collect();
    current.sort_by(|a, b| (a.timestamp, &a.id).cmp(&(b.timestamp, &b.id)));
}
