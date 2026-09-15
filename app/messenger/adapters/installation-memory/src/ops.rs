use crate::state::{
    DeliveryRow, Inner, QUEUE_LIMIT, State, digest, locked, replay, save_command, save_installation,
};
use messenger_domain::{Delivery, Error, Installation, InstallationSpec};
use std::collections::btree_map::Entry;

pub(crate) fn get(inner: &Inner, room: &str, id: &str) -> Result<Option<Installation>, Error> {
    let Some(value) = inner.installations.get(&(room.to_owned(), id.to_owned())) else {
        return Ok(None);
    };
    if value.spec.room != room || value.spec.id != id {
        return Err(Error::Denied);
    }
    Ok(Some(value.clone()))
}

pub(crate) fn list(
    inner: &Inner,
    room: &str,
    after: Option<&str>,
) -> Result<Vec<Installation>, Error> {
    let mut values: Vec<_> = inner
        .installations
        .values()
        .filter(|value| {
            value.spec.room == room && after.is_none_or(|cursor| value.spec.id.as_str() > cursor)
        })
        .cloned()
        .collect();
    values.sort_by(|left, right| left.spec.id.cmp(&right.spec.id));
    values.truncate(100);
    Ok(values)
}

pub(crate) fn install(
    inner: &mut Inner,
    spec: &InstallationSpec,
    expected: u64,
    command: &str,
) -> Result<Installation, Error> {
    spec.validate()?;
    if expected >= i64::MAX as u64 {
        return Err(Error::Invalid("installation generation exhausted".into()));
    }
    if let Some(value) = replay(
        inner,
        &spec.room,
        command,
        digest(&("install", spec, expected))?,
    )? {
        return Ok(value);
    }
    let key = (spec.room.clone(), spec.id.clone());
    let inserted = if expected == 0 {
        match inner.installations.entry(key) {
            Entry::Vacant(slot) => {
                slot.insert(Installation {
                    spec: spec.clone(),
                    generation: 1,
                    enabled: true,
                });
                true
            }
            Entry::Occupied(_) => false,
        }
    } else {
        false
    };
    let current = locked(inner, &spec.room, &spec.id)?;
    if !inserted && current.generation != expected {
        return Err(Error::Denied);
    }
    let value = Installation {
        spec: spec.clone(),
        generation: expected + 1,
        enabled: true,
    };
    save_installation(inner, &value);
    save_command(inner, command, &value)?;
    Ok(value)
}

pub(crate) fn revoke(
    inner: &mut Inner,
    room: &str,
    id: &str,
    expected: u64,
    command: &str,
) -> Result<Installation, Error> {
    if expected == 0 || expected >= i64::MAX as u64 {
        return Err(Error::Invalid("invalid installation generation".into()));
    }
    if let Some(value) = replay(
        inner,
        room,
        command,
        digest(&("revoke", room, id, expected))?,
    )? {
        return Ok(value);
    }
    let mut current = locked(inner, room, id)?;
    if current.generation != expected {
        return Err(Error::Denied);
    }
    current.generation += 1;
    current.enabled = false;
    save_installation(inner, &current);
    save_command(inner, command, &current)?;
    Ok(current)
}

pub(crate) fn enqueue(inner: &mut Inner, room: &str, delivery: &Delivery) -> Result<(), Error> {
    delivery.validate()?;
    let current = locked(inner, room, &delivery.installation)?;
    delivery.authorize(&current.spec, current.generation, current.enabled)?;
    let hash = digest(delivery)?;
    let key = (
        room.to_owned(),
        delivery.installation.clone(),
        delivery.event.clone(),
    );
    if let Some(existing) = inner.deliveries.get(&key) {
        return if existing.digest == hash {
            Ok(())
        } else {
            Err(Error::Denied)
        };
    }
    let inflight = inner
        .deliveries
        .iter()
        .filter(|((row_room, installation, _), row)| {
            row_room == room
                && installation == &delivery.installation
                && row.delivery.generation == current.generation
                && matches!(row.state, State::Pending | State::Leased)
        })
        .count();
    if inflight >= QUEUE_LIMIT {
        return Err(Error::Unavailable(
            "integration queue full; retry original request".into(),
        ));
    }
    inner.deliveries.insert(
        key,
        DeliveryRow {
            delivery: delivery.clone(),
            digest: hash,
            state: State::Pending,
            lease: None,
            expires: None,
            available: inner.now(),
            attempts: 0,
            receipt_digest: None,
        },
    );
    Ok(())
}
