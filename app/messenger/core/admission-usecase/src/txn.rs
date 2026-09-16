use messenger_domain::{Admission, AdmitCommand, AuthorityRecord, Error, validate_admit};
use std::time::{SystemTime, UNIX_EPOCH};

fn txn_key(command: &AdmitCommand) -> Option<String> {
    command.txn.as_ref().map(|txn| {
        format!(
            "{}\0{}\0{}\0{txn}",
            command.sender, command.device, command.endpoint
        )
    })
}

pub(super) fn admit_into(
    inner: &AuthorityRecord,
    command: &AdmitCommand,
) -> Result<(AuthorityRecord, Admission), Error> {
    validate_admit(command)?;
    let mut next = inner.clone();
    if let Some(key) = txn_key(command)
        && let Some(event_id) = next.txns.get(&key)
    {
        let event = next
            .event(event_id)
            .ok_or_else(|| Error::Unavailable("transaction result missing".into()))?;
        return Ok((
            next,
            Admission {
                event,
                reused: true,
            },
        ));
    }

    match command.event_type.as_str() {
        "m.room.member" => next.apply_member(command, now_ms())?,
        _ if command.state_key.is_some() => return Err(Error::Denied),
        _ => next.apply_message(command, now_ms())?,
    }

    let event = next
        .last_event(&command.room)
        .ok_or_else(|| Error::Unavailable("admission did not persist".into()))?;
    if let Some(key) = txn_key(command) {
        next.txns.insert(key, event.event_id.clone());
    }
    Ok((
        next,
        Admission {
            event,
            reused: false,
        },
    ))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(1)
        .max(1)
}
