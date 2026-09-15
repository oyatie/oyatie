#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

mod admit;
mod types;

use admit::{admit_into, create_room_state};
use messenger_conversation_api::RoomAuthority;
use messenger_domain::{
    Admission, AdmitCommand, AuthorityEvent, AuthorityRoomDelta, AuthoritySync, Error, valid_user,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use types::{Inner, membership};

pub struct MemoryAuthority {
    server_name: String,
    inner: Mutex<Inner>,
}

impl MemoryAuthority {
    pub fn new(server_name: impl Into<String>) -> Arc<Self> {
        Arc::new(Self {
            server_name: server_name.into(),
            inner: Mutex::new(Inner::default()),
        })
    }

    async fn transact<R>(
        &self,
        apply: impl Fn(&Inner) -> Result<(Inner, R), Error>,
    ) -> Result<R, Error> {
        for _ in 0..32 {
            let snapshot = self.inner.lock().await.clone();
            let generation = snapshot.generation;
            let (mut next, result) = apply(&snapshot)?;
            let mut guard = self.inner.lock().await;
            if guard.generation != generation {
                continue;
            }
            next.generation = generation.saturating_add(1);
            *guard = next;
            return Ok(result);
        }
        Err(Error::Unavailable("room contention".into()))
    }
}

impl RoomAuthority for MemoryAuthority {
    async fn create_room(&self, creator: &str, join_rule: &str) -> Result<String, Error> {
        let server = self.server_name.clone();
        let creator = creator.to_owned();
        let join_rule = join_rule.to_owned();
        self.transact(move |inner| create_room_state(inner, &server, &creator, &join_rule))
            .await
    }

    async fn admit(&self, command: AdmitCommand) -> Result<Admission, Error> {
        self.transact(|inner| admit_into(inner, command.clone()))
            .await
    }

    async fn sync(
        &self,
        user: &str,
        _device: &str,
        since: Option<&str>,
    ) -> Result<AuthoritySync, Error> {
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
        let inner = self.inner.lock().await;
        let mut rooms = Vec::new();
        let mut newest = after;
        for (room_id, room) in &inner.rooms {
            if membership(room, user) != Some("join") {
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
            newest = inner.seq;
        }
        Ok(AuthoritySync {
            next_batch: format!("s{newest}"),
            rooms,
        })
    }

    async fn state(
        &self,
        room: &str,
        event_type: &str,
        state_key: &str,
    ) -> Result<Option<AuthorityEvent>, Error> {
        let inner = self.inner.lock().await;
        let Some(room) = inner.rooms.get(room) else {
            return Ok(None);
        };
        Ok(room
            .events
            .iter()
            .rev()
            .find(|stored| {
                stored.event.event_type == event_type
                    && stored.event.state_key.as_deref() == Some(state_key)
            })
            .map(|stored| stored.event.clone()))
    }
}
