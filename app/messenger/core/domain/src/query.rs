use crate::Error;
use crate::authority::{AuthorityEvent, AuthorityRoomDelta, AuthoritySync, valid_user};
use crate::record::AuthorityRecord;

impl AuthorityRecord {
    pub fn sync(&self, user: &str, since: Option<&str>) -> Result<AuthoritySync, Error> {
        if !valid_user(user) {
            return Err(Error::Invalid("invalid user".into()));
        }
        let after = match since {
            None | Some("") => 0,
            Some(token) => token
                .strip_prefix('s')
                .and_then(|n| n.parse::<u64>().ok())
                .ok_or_else(|| Error::Invalid("invalid sync token".into()))?,
        };
        let mut rooms = Vec::new();
        let mut newest = after;
        for (room_id, room) in &self.rooms {
            if room.membership(user) != Some("join") {
                continue;
            }
            let events: Vec<_> = room
                .events
                .iter()
                .filter(|stored| stored.seq > after)
                .map(|stored| {
                    newest = newest.max(stored.seq);
                    stored.event.clone()
                })
                .collect();
            rooms.push(AuthorityRoomDelta {
                room: room_id.clone(),
                membership: "join".into(),
                events,
            });
        }
        if newest == after {
            newest = self.seq;
        }
        Ok(AuthoritySync {
            next_batch: format!("s{newest}"),
            rooms,
        })
    }

    pub fn state(&self, room: &str, event_type: &str, state_key: &str) -> Option<AuthorityEvent> {
        self.rooms.get(room).and_then(|room| {
            room.events.iter().rev().find_map(|stored| {
                if stored.event.event_type == event_type
                    && stored.event.state_key.as_deref() == Some(state_key)
                {
                    Some(stored.event.clone())
                } else {
                    None
                }
            })
        })
    }
}
