use crate::state::{Inner, LEASE, State, digest, failed, hex_lower, locked};
use messenger_domain::{Error, Installation};
use messenger_installation_api::Dispatch;
use serde_json::Value;
use std::cmp::Ordering;
use std::time::{Duration, SystemTime};

fn ready(inner: &Inner, room: &str, id: &str, generation: u64, now: SystemTime) -> Option<String> {
    let mut chosen: Option<(SystemTime, &str)> = None;
    for ((row_room, installation, event), row) in &inner.deliveries {
        if row_room != room || installation != id || row.delivery.generation != generation {
            continue;
        }
        let claimable = match row.state {
            State::Pending if row.available <= now => true,
            State::Leased if row.expires.is_some_and(|expires| expires <= now) => true,
            _ => false,
        };
        if !claimable {
            continue;
        }
        let better = match &chosen {
            None => true,
            Some((available, current_event)) => {
                match row
                    .available
                    .partial_cmp(available)
                    .unwrap_or(Ordering::Equal)
                {
                    Ordering::Less => true,
                    Ordering::Equal => event.as_str() < *current_event,
                    Ordering::Greater => false,
                }
            }
        };
        if better {
            chosen = Some((row.available, event.as_str()));
        }
    }
    chosen.map(|(_, event)| event.to_owned())
}

pub(crate) fn claim(inner: &mut Inner, expected: &Installation) -> Result<Option<Dispatch>, Error> {
    let room = expected.spec.room.as_str();
    let current = locked(inner, room, &expected.spec.id)?;
    if &current != expected || !current.enabled {
        return Err(Error::Denied);
    }
    let now = inner.now();
    let Some(event) = ready(inner, room, &current.spec.id, current.generation, now) else {
        return Ok(None);
    };
    let key = (room.to_owned(), current.spec.id.clone(), event.clone());
    let delivery = {
        let row = inner.deliveries.get(&key).ok_or_else(failed)?;
        row.delivery
            .authorize(&current.spec, current.generation, current.enabled)?;
        if row.delivery.event != event || digest(&row.delivery)? != row.digest {
            return Err(Error::Denied);
        }
        row.delivery.clone()
    };
    inner.leases = inner.leases.saturating_add(1);
    let lease = format!("{:032x}", inner.leases);
    let idempotency_key = hex_lower(&digest(&(
        "oyatie.messenger.integration.v1",
        &inner.tenant,
        room,
        &current.spec.id,
        current.generation,
        &event,
        "dispatch",
    ))?);
    let row = inner.deliveries.get_mut(&key).ok_or_else(failed)?;
    row.state = State::Leased;
    row.lease = Some(lease.clone());
    row.expires = Some(now.checked_add(LEASE).ok_or_else(failed)?);
    row.attempts = row.attempts.min(1_000_000) + 1;
    Ok(Some(Dispatch {
        installation: current,
        delivery,
        lease,
        idempotency_key,
    }))
}

pub(crate) fn complete(
    inner: &mut Inner,
    dispatch: &Dispatch,
    receipt: &Value,
) -> Result<(), Error> {
    if !receipt.is_object() {
        return Err(Error::Invalid("object receipt required".into()));
    }
    let delivery = &dispatch.delivery;
    delivery.authorize(
        &dispatch.installation.spec,
        dispatch.installation.generation,
        dispatch.installation.enabled,
    )?;
    let key = (
        dispatch.installation.spec.room.clone(),
        delivery.installation.clone(),
        delivery.event.clone(),
    );
    let row = inner.deliveries.get_mut(&key).ok_or(Error::Denied)?;
    if row.delivery.generation != delivery.generation
        || row.lease.as_deref() != Some(dispatch.lease.as_str())
        || row.digest != digest(delivery)?
    {
        return Err(Error::Denied);
    }
    let hash = digest(receipt)?;
    if row.state == State::Complete && row.receipt_digest == Some(hash) {
        return Ok(());
    }
    if row.state != State::Leased {
        return Err(Error::Denied);
    }
    row.state = State::Complete;
    row.receipt_digest = Some(hash);
    Ok(())
}

pub(crate) fn retry(inner: &mut Inner, dispatch: &Dispatch) -> Result<(), Error> {
    let room = dispatch.installation.spec.room.as_str();
    let delivery = &dispatch.delivery;
    let current = locked(inner, room, &delivery.installation)?;
    let active = current == dispatch.installation && current.enabled;
    let available_at = inner.now();
    let key = (
        room.to_owned(),
        delivery.installation.clone(),
        delivery.event.clone(),
    );
    let row = inner.deliveries.get_mut(&key).ok_or(Error::Denied)?;
    if row.state != State::Leased
        || row.lease.as_deref() != Some(dispatch.lease.as_str())
        || row.digest != digest(delivery)?
        || row.delivery.generation != delivery.generation
    {
        return Err(Error::Denied);
    }
    let delay = 2u64.pow(row.attempts.min(8)).min(300);
    row.state = if active {
        State::Pending
    } else {
        State::Cancelled
    };
    row.lease = None;
    row.expires = None;
    row.available = available_at
        .checked_add(Duration::from_secs(delay))
        .ok_or_else(failed)?;
    if active { Ok(()) } else { Err(Error::Denied) }
}
