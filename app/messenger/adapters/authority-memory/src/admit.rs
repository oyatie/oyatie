use crate::types::{
    Inner, Room, Stored, TxnResult, digest, event_id, hex_lower, membership, now_ms, txn_key,
};
use messenger_domain::{
    Admission, AdmitCommand, AuthorityEvent, Error, ban_allowed, join_allowed, member_can_leave,
    member_can_send, valid_user, validate_admit,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub(crate) fn append(
    inner: &mut Inner,
    room_id: &str,
    sender: &str,
    event_type: &str,
    state_key: Option<String>,
    content: Value,
) -> Result<AuthorityEvent, Error> {
    let room = inner
        .rooms
        .get_mut(room_id)
        .ok_or_else(|| Error::Invalid("unknown room".into()))?;
    inner.seq = inner.seq.saturating_add(1);
    let origin_server_ts = now_ms();
    let event = json!({
        "room_id": room_id,
        "sender": sender,
        "type": event_type,
        "state_key": state_key,
        "content": content,
        "origin_server_ts": origin_server_ts,
        "seq": inner.seq,
    });
    let stored = AuthorityEvent {
        event_id: event_id(&event)?,
        room: room_id.into(),
        sender: sender.into(),
        origin_server_ts,
        event_type: event_type.into(),
        state_key,
        content,
    };
    room.events.push(Stored {
        seq: inner.seq,
        event: stored.clone(),
    });
    Ok(stored)
}

pub(crate) fn admit_into(
    inner: &Inner,
    command: AdmitCommand,
) -> Result<(Inner, Admission), Error> {
    validate_admit(&command)?;
    let mut next = inner.clone();
    if let Some(key) = txn_key(&command)
        && let Some(existing) = next.txns.get(&key)
    {
        let event = next
            .rooms
            .values()
            .flat_map(|room| room.events.iter())
            .find(|stored| stored.event.event_id == existing.event_id)
            .map(|stored| stored.event.clone())
            .ok_or_else(|| Error::Unavailable("transaction result missing".into()))?;
        let _changed = existing.digest != digest(&command.content)?;
        return Ok((
            next,
            Admission {
                event,
                reused: true,
            },
        ));
    }

    match command.event_type.as_str() {
        "m.room.member" => admit_member(&mut next, &command)?,
        _ if command.state_key.is_some() => return Err(Error::Denied),
        _ => admit_message(&mut next, &command)?,
    }

    let event = next
        .rooms
        .get(&command.room)
        .and_then(|room| room.events.last())
        .map(|stored| stored.event.clone())
        .ok_or_else(|| Error::Unavailable("admission did not persist".into()))?;
    if let Some(key) = txn_key(&command) {
        next.txns.insert(
            key,
            TxnResult {
                event_id: event.event_id.clone(),
                digest: digest(&command.content)?,
            },
        );
    }
    Ok((
        next,
        Admission {
            event,
            reused: false,
        },
    ))
}

pub(crate) fn create_room_state(
    inner: &Inner,
    server_name: &str,
    creator: &str,
    join_rule: &str,
) -> Result<(Inner, String), Error> {
    if !valid_user(creator) || !matches!(join_rule, "public" | "invite") {
        return Err(Error::Invalid("invalid room creation".into()));
    }
    let mut next = inner.clone();
    next.seq = next.seq.saturating_add(1);
    let opaque = hex_lower(&Sha256::digest(format!("{}:{}", server_name, next.seq)));
    let room_id = format!("!{opaque}:{server_name}");
    next.rooms.insert(
        room_id.clone(),
        Room {
            creator: creator.into(),
            join_rule: join_rule.into(),
            members: std::collections::BTreeMap::new(),
            events: Vec::new(),
        },
    );
    append(
        &mut next,
        &room_id,
        creator,
        "m.room.create",
        Some(String::new()),
        json!({"creator": creator, "room_version": "11"}),
    )?;
    append(
        &mut next,
        &room_id,
        creator,
        "m.room.member",
        Some(creator.into()),
        json!({"membership": "join"}),
    )?;
    append(
        &mut next,
        &room_id,
        creator,
        "m.room.join_rules",
        Some(String::new()),
        json!({"join_rule": join_rule}),
    )?;
    let room = next
        .rooms
        .get_mut(&room_id)
        .ok_or_else(|| Error::Unavailable("room missing after create".into()))?;
    room.members.insert(creator.into(), "join".into());
    Ok((next, room_id))
}

fn admit_message(inner: &mut Inner, command: &AdmitCommand) -> Result<(), Error> {
    let membership = inner
        .rooms
        .get(&command.room)
        .and_then(|room| membership(room, &command.sender).map(str::to_owned));
    if !member_can_send(membership.as_deref().unwrap_or("")) {
        return Err(Error::Denied);
    }
    append(
        inner,
        &command.room,
        &command.sender,
        &command.event_type,
        None,
        command.content.clone(),
    )?;
    Ok(())
}

fn admit_member(inner: &mut Inner, command: &AdmitCommand) -> Result<(), Error> {
    let target = command
        .state_key
        .as_deref()
        .filter(|id| valid_user(id))
        .ok_or_else(|| Error::Invalid("member events require a user state key".into()))?;
    let next_membership = command.content["membership"]
        .as_str()
        .ok_or_else(|| Error::Invalid("membership required".into()))?;
    let room = inner
        .rooms
        .get(&command.room)
        .ok_or_else(|| Error::Invalid("unknown room".into()))?;
    let current = membership(room, target).map(str::to_owned);
    let actor_membership = membership(room, &command.sender).map(str::to_owned);
    match next_membership {
        "join" => {
            if command.sender != target {
                return Err(Error::Denied);
            }
            join_allowed(current.as_deref(), &room.join_rule)?;
        }
        "leave" => {
            if command.sender == target {
                if !member_can_leave(current.as_deref().unwrap_or("")) {
                    return Err(Error::Denied);
                }
            } else if !member_can_send(actor_membership.as_deref().unwrap_or(""))
                || ban_allowed(room.creator == command.sender, room.creator == target).is_err()
            {
                return Err(Error::Denied);
            }
        }
        "invite" => {
            if !member_can_send(actor_membership.as_deref().unwrap_or(""))
                || matches!(current.as_deref(), Some("ban" | "join"))
            {
                return Err(Error::Denied);
            }
        }
        "ban" => {
            if !member_can_send(actor_membership.as_deref().unwrap_or("")) {
                return Err(Error::Denied);
            }
            ban_allowed(room.creator == command.sender, room.creator == target)?;
        }
        _ => return Err(Error::Invalid("unsupported membership".into())),
    }
    append(
        inner,
        &command.room,
        &command.sender,
        "m.room.member",
        Some(target.into()),
        command.content.clone(),
    )?;
    let room = inner
        .rooms
        .get_mut(&command.room)
        .ok_or_else(|| Error::Unavailable("room missing during membership".into()))?;
    room.members.insert(target.into(), next_membership.into());
    Ok(())
}
