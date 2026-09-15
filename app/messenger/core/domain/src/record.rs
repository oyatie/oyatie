use crate::Error;
use crate::authority::{AuthorityEvent, valid_user};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredEvent {
    pub seq: u64,
    pub event: AuthorityEvent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Room {
    pub creator: String,
    pub join_rule: String,
    pub members: BTreeMap<String, String>,
    pub events: Vec<StoredEvent>,
}

impl Room {
    pub fn membership(&self, user: &str) -> Option<&str> {
        self.members.get(user).map(String::as_str)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AuthorityRecord {
    pub generation: u64,
    pub seq: u64,
    pub rooms: BTreeMap<String, Room>,
    pub txns: BTreeMap<String, String>,
}

impl AuthorityRecord {
    pub fn event(&self, event_id: &str) -> Option<AuthorityEvent> {
        self.rooms
            .values()
            .flat_map(|room| room.events.iter())
            .find(|stored| stored.event.event_id == event_id)
            .map(|stored| stored.event.clone())
    }

    pub fn last_event(&self, room: &str) -> Option<AuthorityEvent> {
        self.rooms
            .get(room)
            .and_then(|room| room.events.last())
            .map(|stored| stored.event.clone())
    }

    pub fn append(
        &mut self,
        room_id: &str,
        sender: &str,
        event_type: &str,
        state_key: Option<String>,
        content: Value,
        origin_server_ts: u64,
    ) -> Result<AuthorityEvent, Error> {
        let room = self
            .rooms
            .get_mut(room_id)
            .ok_or_else(|| Error::Invalid("unknown room".into()))?;
        self.seq = self.seq.saturating_add(1);
        let event = AuthorityEvent {
            event_id: format!("${:x}:{room_id}", self.seq),
            room: room_id.into(),
            sender: sender.into(),
            origin_server_ts,
            event_type: event_type.into(),
            state_key,
            content,
        };
        room.events.push(StoredEvent {
            seq: self.seq,
            event: event.clone(),
        });
        Ok(event)
    }

    pub fn create_room(
        &mut self,
        room_id: String,
        creator: &str,
        join_rule: &str,
        origin_server_ts: u64,
    ) -> Result<String, Error> {
        if !valid_user(creator) || !matches!(join_rule, "public" | "invite") {
            return Err(Error::Invalid("invalid room creation".into()));
        }
        self.rooms.insert(
            room_id.clone(),
            Room {
                creator: creator.into(),
                join_rule: join_rule.into(),
                members: BTreeMap::new(),
                events: Vec::new(),
            },
        );
        self.append(
            &room_id,
            creator,
            "m.room.create",
            Some(String::new()),
            json!({"creator": creator, "room_version": "11"}),
            origin_server_ts,
        )?;
        self.append(
            &room_id,
            creator,
            "m.room.member",
            Some(creator.into()),
            json!({"membership": "join"}),
            origin_server_ts,
        )?;
        self.append(
            &room_id,
            creator,
            "m.room.join_rules",
            Some(String::new()),
            json!({"join_rule": join_rule}),
            origin_server_ts,
        )?;
        let room = self
            .rooms
            .get_mut(&room_id)
            .ok_or_else(|| Error::Unavailable("room missing after create".into()))?;
        room.members.insert(creator.into(), "join".into());
        Ok(room_id)
    }
}
