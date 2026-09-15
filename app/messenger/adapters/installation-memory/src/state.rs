use messenger_domain::{Delivery, Error, Installation};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    time::{Duration, SystemTime},
};

pub(crate) const LEASE: Duration = Duration::from_secs(30);
pub(crate) const QUEUE_LIMIT: usize = 1000;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
    Pending,
    Leased,
    Complete,
    Cancelled,
}

pub(crate) struct Command {
    pub digest: [u8; 32],
    pub result: Option<Installation>,
}

pub(crate) struct DeliveryRow {
    pub delivery: Delivery,
    pub digest: [u8; 32],
    pub state: State,
    pub lease: Option<String>,
    pub expires: Option<SystemTime>,
    pub available: SystemTime,
    pub attempts: u32,
    pub receipt_digest: Option<[u8; 32]>,
}

#[derive(Default)]
pub(crate) struct Inner {
    pub tenant: String,
    pub clock_offset: Duration,
    pub leases: u64,
    pub installations: BTreeMap<(String, String), Installation>,
    pub commands: BTreeMap<(String, String), Command>,
    pub deliveries: BTreeMap<(String, String, String), DeliveryRow>,
}

impl Inner {
    pub(crate) fn now(&self) -> SystemTime {
        SystemTime::now()
            .checked_add(self.clock_offset)
            .unwrap_or_else(SystemTime::now)
    }

    pub(crate) fn elapse(&mut self, duration: Duration) {
        self.clock_offset = self.clock_offset.saturating_add(duration);
    }
}

pub(crate) fn failed() -> Error {
    Error::Unavailable("integration store unavailable".into())
}

pub(crate) fn digest(value: &impl Serialize) -> Result<[u8; 32], Error> {
    let bytes = serde_json::to_vec(value).map_err(|_| failed())?;
    Ok(Sha256::digest(bytes).into())
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

pub(crate) fn command_id(value: &str) -> bool {
    !value.is_empty()
        && value != "00000000-0000-0000-0000-000000000000"
        && value.len() <= 36
        && !value.chars().any(char::is_whitespace)
}

pub(crate) fn locked(inner: &Inner, room: &str, id: &str) -> Result<Installation, Error> {
    let value = inner
        .installations
        .get(&(room.to_owned(), id.to_owned()))
        .cloned()
        .ok_or(Error::Denied)?;
    if value.spec.room != room || value.spec.id != id {
        return Err(Error::Denied);
    }
    Ok(value)
}

pub(crate) fn replay(
    inner: &mut Inner,
    room: &str,
    command: &str,
    hash: [u8; 32],
) -> Result<Option<Installation>, Error> {
    if !command_id(command) {
        return Err(Error::Invalid("command ID required".into()));
    }
    let key = (room.to_owned(), command.to_owned());
    match inner.commands.get(&key) {
        None => {
            inner.commands.insert(
                key,
                Command {
                    digest: hash,
                    result: None,
                },
            );
            Ok(None)
        }
        Some(existing) if existing.digest != hash => Err(Error::Denied),
        Some(existing) => Ok(existing.result.clone()),
    }
}

pub(crate) fn save_command(
    inner: &mut Inner,
    command: &str,
    value: &Installation,
) -> Result<(), Error> {
    let stored = inner
        .commands
        .get_mut(&(value.spec.room.clone(), command.to_owned()))
        .ok_or_else(failed)?;
    stored.result = Some(value.clone());
    Ok(())
}

pub(crate) fn save_installation(inner: &mut Inner, value: &Installation) {
    let room = value.spec.room.clone();
    let id = value.spec.id.clone();
    inner
        .installations
        .insert((room.clone(), id.clone()), value.clone());
    for ((row_room, installation, _), row) in &mut inner.deliveries {
        if row_room == &room
            && installation == &id
            && row.state == State::Pending
            && row.delivery.generation != value.generation
        {
            row.state = State::Cancelled;
        }
    }
}
