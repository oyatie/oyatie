use messenger_domain::{AdmitCommand, AuthorityEvent, Error};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone)]
pub(crate) struct Stored {
    pub seq: u64,
    pub event: AuthorityEvent,
}

#[derive(Clone)]
pub(crate) struct Room {
    pub creator: String,
    pub join_rule: String,
    pub members: BTreeMap<String, String>,
    pub events: Vec<Stored>,
}

#[derive(Clone, Default)]
pub(crate) struct Inner {
    pub generation: u64,
    pub seq: u64,
    pub rooms: BTreeMap<String, Room>,
    pub txns: BTreeMap<String, TxnResult>,
}

#[derive(Clone)]
pub(crate) struct TxnResult {
    pub event_id: String,
    pub digest: [u8; 32],
}

pub(crate) fn digest(content: &Value) -> Result<[u8; 32], Error> {
    let bytes =
        serde_json::to_vec(content).map_err(|_| Error::Invalid("invalid event content".into()))?;
    Ok(Sha256::digest(bytes).into())
}

pub(crate) fn txn_key(command: &AdmitCommand) -> Option<String> {
    command.txn.as_ref().map(|txn| {
        format!(
            "{}\0{}\0{}\0{txn}",
            command.sender, command.device, command.endpoint
        )
    })
}

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(1)
        .max(1)
}

pub(crate) fn event_id(event: &Value) -> Result<String, Error> {
    let bytes =
        serde_json::to_vec(event).map_err(|_| Error::Invalid("invalid event content".into()))?;
    Ok(format!("${}", hex_lower(&Sha256::digest(bytes))))
}

pub(crate) fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

pub(crate) fn membership<'a>(room: &'a Room, user: &str) -> Option<&'a str> {
    room.members.get(user).map(String::as_str)
}
