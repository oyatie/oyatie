use crate::Error;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorityEvent {
    pub event_id: String,
    pub room: String,
    pub sender: String,
    pub origin_server_ts: u64,
    pub event_type: String,
    pub state_key: Option<String>,
    pub content: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmitCommand {
    pub room: String,
    pub sender: String,
    pub device: String,
    pub event_type: String,
    pub state_key: Option<String>,
    pub content: serde_json::Value,
    pub txn: Option<String>,
    pub endpoint: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Admission {
    pub event: AuthorityEvent,
    pub reused: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthoritySync {
    pub next_batch: String,
    pub rooms: Vec<AuthorityRoomDelta>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorityRoomDelta {
    pub room: String,
    pub membership: String,
    pub events: Vec<AuthorityEvent>,
}

pub fn member_can_send(membership: &str) -> bool {
    membership == "join"
}

pub fn member_can_leave(membership: &str) -> bool {
    matches!(membership, "join" | "invite")
}

pub fn join_allowed(current: Option<&str>, join_rule: &str) -> Result<(), Error> {
    match current {
        Some("join") => Ok(()),
        Some("ban") => Err(Error::Denied),
        Some("invite") => Ok(()),
        _ if join_rule == "public" => Ok(()),
        _ => Err(Error::Denied),
    }
}

pub fn ban_allowed(actor_is_creator: bool, target_is_creator: bool) -> Result<(), Error> {
    if actor_is_creator && !target_is_creator {
        Ok(())
    } else {
        Err(Error::Denied)
    }
}

pub fn send_endpoint(room: &str, event_type: &str) -> String {
    format!("PUT /_matrix/client/v3/rooms/{room}/send/{event_type}")
}

pub fn valid_user(id: &str) -> bool {
    let Some((local, server)) = id.strip_prefix('@').and_then(|rest| rest.split_once(':')) else {
        return false;
    };
    !local.is_empty()
        && !server.is_empty()
        && id.len() <= 255
        && !id.chars().any(|c| c.is_control() || c.is_whitespace())
}

pub fn valid_room(id: &str) -> bool {
    let Some((opaque, server)) = id.strip_prefix('!').and_then(|rest| rest.split_once(':')) else {
        return false;
    };
    !opaque.is_empty()
        && !server.is_empty()
        && id.len() <= 255
        && !id.chars().any(|c| c.is_control() || c.is_whitespace())
}

pub fn valid_txn(id: &str) -> bool {
    !id.is_empty() && id.len() <= 255 && !id.chars().any(char::is_control)
}

pub fn validate_admit(command: &AdmitCommand) -> Result<(), Error> {
    if !valid_room(&command.room)
        || !valid_user(&command.sender)
        || command.device.is_empty()
        || command.device.len() > 255
        || command
            .device
            .chars()
            .any(|c| c.is_control() || c.is_whitespace())
        || command.event_type.is_empty()
        || command.event_type.len() > 255
        || command.event_type.chars().any(char::is_control)
        || command.endpoint.is_empty()
        || command.endpoint.len() > 512
        || !command.content.is_object()
    {
        return Err(Error::Invalid("invalid admission command".into()));
    }
    if let Some(txn) = &command.txn
        && !valid_txn(txn)
    {
        return Err(Error::Invalid("invalid transaction id".into()));
    }
    if let Some(state_key) = &command.state_key
        && (state_key.len() > 255 || state_key.chars().any(char::is_control))
    {
        return Err(Error::Invalid("invalid state key".into()));
    }
    let encoded = serde_json::to_vec(&command.content)
        .map_err(|_| Error::Invalid("invalid event content".into()))?;
    if encoded.len() > 65_536 {
        return Err(Error::Invalid("event content exceeds 65536 bytes".into()));
    }
    Ok(())
}
