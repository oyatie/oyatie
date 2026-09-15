#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_conversation_api::RoomAuthority;
use messenger_domain::{AuthorityEvent, AuthorityRecord, AuthoritySync, Error, valid_user};
use sha2::{Digest, Sha256};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;

pub struct MemoryAuthority {
    server_name: String,
    inner: Mutex<AuthorityRecord>,
}

impl MemoryAuthority {
    pub fn new(server_name: impl Into<String>) -> Arc<Self> {
        Arc::new(Self {
            server_name: server_name.into(),
            inner: Mutex::new(AuthorityRecord::default()),
        })
    }

    async fn cas(&self, expected: u64, mut next: AuthorityRecord) -> Result<(), Error> {
        let mut guard = self.inner.lock().await;
        if guard.generation != expected {
            return Err(Error::Unavailable("room contention".into()));
        }
        next.generation = expected.saturating_add(1);
        *guard = next;
        Ok(())
    }
}

impl RoomAuthority for MemoryAuthority {
    async fn create_room(&self, creator: &str, join_rule: &str) -> Result<String, Error> {
        if !valid_user(creator) || !matches!(join_rule, "public" | "invite") {
            return Err(Error::Invalid("invalid room creation".into()));
        }
        for _ in 0..32 {
            let snapshot = self.inner.lock().await.clone();
            let expected = snapshot.generation;
            let mut next = snapshot;
            next.seq = next.seq.saturating_add(1);
            let opaque = hex_lower(&Sha256::digest(format!(
                "{}:{}",
                self.server_name, next.seq
            )));
            let room_id = format!("!{opaque}:{}", self.server_name);
            let room_id = next.create_room(room_id, creator, join_rule, now_ms())?;
            match self.cas(expected, next).await {
                Ok(()) => return Ok(room_id),
                Err(Error::Unavailable(_)) => continue,
                Err(error) => return Err(error),
            }
        }
        Err(Error::Unavailable("room contention".into()))
    }

    async fn snapshot(&self) -> Result<AuthorityRecord, Error> {
        Ok(self.inner.lock().await.clone())
    }

    async fn commit(&self, expected_generation: u64, record: AuthorityRecord) -> Result<(), Error> {
        self.cas(expected_generation, record).await
    }

    async fn sync(
        &self,
        user: &str,
        _device: &str,
        since: Option<&str>,
    ) -> Result<AuthoritySync, Error> {
        self.inner.lock().await.sync(user, since)
    }

    async fn state(
        &self,
        room: &str,
        event_type: &str,
        state_key: &str,
    ) -> Result<Option<AuthorityEvent>, Error> {
        Ok(self.inner.lock().await.state(room, event_type, state_key))
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(1)
        .max(1)
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
