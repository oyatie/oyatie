#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_archive_api::Archive;
use messenger_domain::{ArchiveEvent, ArchiveEventPage, Error};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;

type EventKey = (u64, String);
type RoomEvents = BTreeMap<EventKey, ArchiveEvent>;
type ArchiveRooms = BTreeMap<String, RoomEvents>;

pub struct MemoryArchive {
    inner: Mutex<ArchiveRooms>,
}

impl MemoryArchive {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(BTreeMap::new()),
        })
    }
}

fn decode_cursor(cursor: &str) -> Result<(u64, String), Error> {
    let (timestamp, id) = cursor
        .split_once(':')
        .ok_or_else(|| Error::Invalid("invalid archive cursor".into()))?;
    if id.is_empty() {
        return Err(Error::Invalid("invalid archive cursor".into()));
    }
    let timestamp = timestamp
        .parse::<u64>()
        .map_err(|_| Error::Invalid("invalid archive cursor".into()))?;
    Ok((timestamp, id.to_owned()))
}

impl Archive for MemoryArchive {
    async fn capture(&self, room: &str, event: ArchiveEvent) -> Result<(), Error> {
        event.validate_capture(room)?;
        let mut inner = self.inner.lock().await;
        let room_events = inner.entry(room.to_owned()).or_default();
        if let Some(stored) = room_events.values().find(|stored| stored.id == event.id) {
            return if stored == &event {
                Ok(())
            } else {
                Err(Error::Invalid("archive event conflict".into()))
            };
        }
        room_events.insert((event.timestamp, event.id.clone()), event);
        Ok(())
    }

    async fn page(
        &self,
        room: &str,
        after: Option<&str>,
        limit: u16,
    ) -> Result<ArchiveEventPage, Error> {
        ArchiveEventPage::validate_page(room, limit)?;
        let after = match after {
            Some(cursor) => decode_cursor(cursor)?,
            None => (0, String::new()),
        };
        let inner = self.inner.lock().await;
        let mut events: Vec<_> = inner
            .get(room)
            .into_iter()
            .flatten()
            .filter(|(key, _)| **key > after)
            .map(|(_, event)| event.clone())
            .take(usize::from(limit) + 1)
            .collect();
        let next = (events.len() > usize::from(limit)).then(|| {
            events.truncate(usize::from(limit));
            events
                .last()
                .map(|event| format!("{}:{}", event.timestamp, event.id))
        });
        Ok(ArchiveEventPage {
            events,
            next: next.flatten(),
        })
    }
}
