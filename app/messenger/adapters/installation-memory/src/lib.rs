#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_domain::{Error, Installation, InstallationSpec};
use messenger_installation_api::InstallationStore;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Clone, PartialEq, Eq)]
enum CommandBody {
    Install {
        spec: InstallationSpec,
        expected: u64,
    },
    Revoke {
        room: String,
        id: String,
        expected: u64,
    },
}

struct Command {
    body: CommandBody,
    result: Installation,
}

#[derive(Default)]
struct Inner {
    installations: BTreeMap<(String, String), Installation>,
    commands: BTreeMap<(String, String), Command>,
}

pub struct MemoryInstallations {
    inner: Mutex<Inner>,
}

impl MemoryInstallations {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(Inner::default()),
        })
    }
}

fn command_id(value: &str) -> bool {
    !value.is_empty()
        && value != "00000000-0000-0000-0000-000000000000"
        && value.len() <= 36
        && !value.chars().any(char::is_whitespace)
}

fn locked(inner: &Inner, room: &str, id: &str) -> Result<Installation, Error> {
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

fn replay(
    inner: &Inner,
    room: &str,
    command: &str,
    body: &CommandBody,
) -> Result<Option<Installation>, Error> {
    if !command_id(command) {
        return Err(Error::Invalid("command ID required".into()));
    }
    match inner.commands.get(&(room.to_owned(), command.to_owned())) {
        None => Ok(None),
        Some(existing) if existing.body == *body => Ok(Some(existing.result.clone())),
        Some(_) => Err(Error::Denied),
    }
}

fn occupy(inner: &mut Inner, room: &str, command: &str, body: CommandBody, result: &Installation) {
    inner.commands.insert(
        (room.to_owned(), command.to_owned()),
        Command {
            body,
            result: result.clone(),
        },
    );
}

fn get(inner: &Inner, room: &str, id: &str) -> Result<Option<Installation>, Error> {
    let Some(value) = inner.installations.get(&(room.to_owned(), id.to_owned())) else {
        return Ok(None);
    };
    if value.spec.room != room || value.spec.id != id {
        return Err(Error::Denied);
    }
    Ok(Some(value.clone()))
}

fn list(inner: &Inner, room: &str, after: Option<&str>) -> Result<Vec<Installation>, Error> {
    let mut values: Vec<_> = inner
        .installations
        .values()
        .filter(|value| {
            value.spec.room == room && after.is_none_or(|cursor| value.spec.id.as_str() > cursor)
        })
        .cloned()
        .collect();
    values.sort_by(|left, right| left.spec.id.cmp(&right.spec.id));
    Ok(values)
}

fn install(
    inner: &mut Inner,
    spec: &InstallationSpec,
    expected: u64,
    command: &str,
) -> Result<Installation, Error> {
    spec.validate()?;
    if expected >= i64::MAX as u64 {
        return Err(Error::Invalid("installation generation exhausted".into()));
    }
    let body = CommandBody::Install {
        spec: spec.clone(),
        expected,
    };
    if let Some(value) = replay(inner, &spec.room, command, &body)? {
        return Ok(value);
    }
    let key = (spec.room.clone(), spec.id.clone());
    let inserted = expected == 0 && !inner.installations.contains_key(&key);
    if inserted {
        inner.installations.insert(
            key,
            Installation {
                spec: spec.clone(),
                generation: 1,
                enabled: true,
            },
        );
    }
    let current = locked(inner, &spec.room, &spec.id)?;
    if !inserted && current.generation != expected {
        return Err(Error::Denied);
    }
    let value = Installation {
        spec: spec.clone(),
        generation: expected + 1,
        enabled: true,
    };
    inner.installations.insert(
        (value.spec.room.clone(), value.spec.id.clone()),
        value.clone(),
    );
    occupy(inner, &spec.room, command, body, &value);
    Ok(value)
}

fn revoke(
    inner: &mut Inner,
    room: &str,
    id: &str,
    expected: u64,
    command: &str,
) -> Result<Installation, Error> {
    if expected == 0 || expected >= i64::MAX as u64 {
        return Err(Error::Invalid("invalid installation generation".into()));
    }
    let body = CommandBody::Revoke {
        room: room.to_owned(),
        id: id.to_owned(),
        expected,
    };
    if let Some(value) = replay(inner, room, command, &body)? {
        return Ok(value);
    }
    let mut current = locked(inner, room, id)?;
    if current.generation != expected {
        return Err(Error::Denied);
    }
    current.generation += 1;
    current.enabled = false;
    inner.installations.insert(
        (current.spec.room.clone(), current.spec.id.clone()),
        current.clone(),
    );
    occupy(inner, room, command, body, &current);
    Ok(current)
}

impl InstallationStore for MemoryInstallations {
    async fn get(&self, room: &str, id: &str) -> Result<Option<Installation>, Error> {
        get(&*self.inner.lock().await, room, id)
    }

    async fn list(&self, room: &str, after: Option<&str>) -> Result<Vec<Installation>, Error> {
        list(&*self.inner.lock().await, room, after)
    }

    async fn install(
        &self,
        spec: &InstallationSpec,
        expected: u64,
        command: &str,
    ) -> Result<Installation, Error> {
        install(&mut *self.inner.lock().await, spec, expected, command)
    }

    async fn revoke(
        &self,
        room: &str,
        id: &str,
        expected: u64,
        command: &str,
    ) -> Result<Installation, Error> {
        revoke(&mut *self.inner.lock().await, room, id, expected, command)
    }
}
