use messenger_archive_api::{Archive, same_archive_event, validate_capture, validate_page};
use messenger_archive_postgres::PostgresArchive;
use messenger_domain::{ArchiveEvent, ArchiveEventPage, Decryption, Error};
use serde_json::json;
use std::collections::BTreeMap;
use tokio::sync::Mutex;

const ROOM: &str = "!room:messenger.test";
const ALICE: &str = "@alice:messenger.test";

struct MemoryArchive {
    inner: Mutex<BTreeMap<String, BTreeMap<(u64, String), ArchiveEvent>>>,
}

impl MemoryArchive {
    fn new() -> Self {
        Self {
            inner: Mutex::new(BTreeMap::new()),
        }
    }
}

impl Archive for MemoryArchive {
    async fn capture(&self, room: &str, event: ArchiveEvent) -> Result<(), Error> {
        validate_capture(room, &event)?;
        let mut inner = self.inner.lock().await;
        let room_events = inner.entry(room.to_owned()).or_default();
        if let Some(stored) = room_events.values().find(|stored| stored.id == event.id) {
            return if same_archive_event(stored, &event) {
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
        validate_page(room, limit)?;
        let after = match after {
            Some(cursor) => {
                let (timestamp, id) = cursor
                    .split_once(':')
                    .ok_or_else(|| Error::Invalid("invalid archive cursor".into()))?;
                (
                    timestamp
                        .parse::<u64>()
                        .map_err(|_| Error::Invalid("invalid archive cursor".into()))?,
                    id.to_owned(),
                )
            }
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

fn text_event(id: &str, timestamp: u64, body: &str) -> ArchiveEvent {
    ArchiveEvent {
        id: id.into(),
        sender: ALICE.into(),
        timestamp,
        event_type: "m.room.message".into(),
        content: json!({"body": body, "msgtype": "m.text"}),
        decryption: Decryption::Decrypted,
    }
}

async fn bodies(archive: &MemoryArchive, after: Option<&str>, limit: u16) -> Vec<String> {
    archive
        .page(ROOM, after, limit)
        .await
        .unwrap()
        .events
        .into_iter()
        .map(|event| event.content["body"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn capture_then_page_returns_the_event() {
    let archive = MemoryArchive::new();
    archive
        .capture(ROOM, text_event("$a", 1, "hello"))
        .await
        .unwrap();
    assert_eq!(bodies(&archive, None, 10).await, vec!["hello"]);
}

#[tokio::test]
async fn reused_event_id_is_idempotent_and_conflicts_on_change() {
    let archive = MemoryArchive::new();
    let event = text_event("$a", 1, "hello");
    archive.capture(ROOM, event.clone()).await.unwrap();
    archive.capture(ROOM, event).await.unwrap();
    assert_eq!(bodies(&archive, None, 10).await, vec!["hello"]);
    let conflict = archive
        .capture(ROOM, text_event("$a", 1, "changed"))
        .await
        .unwrap_err();
    assert!(matches!(conflict, Error::Invalid(_)));
}

#[tokio::test]
async fn page_cursor_returns_the_rest() {
    let archive = MemoryArchive::new();
    archive
        .capture(ROOM, text_event("$a", 1, "one"))
        .await
        .unwrap();
    archive
        .capture(ROOM, text_event("$b", 2, "two"))
        .await
        .unwrap();
    archive
        .capture(ROOM, text_event("$c", 3, "three"))
        .await
        .unwrap();
    let first = archive.page(ROOM, None, 2).await.unwrap();
    assert_eq!(
        first
            .events
            .iter()
            .map(|event| event.content["body"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["one", "two"]
    );
    let next = first.next.expect("next page");
    assert_eq!(bodies(&archive, Some(&next), 2).await, vec!["three"]);
}

#[test]
fn postgres_adapter_implements_the_archive_port() {
    fn assert_archive<T: Archive>() {}
    assert_archive::<PostgresArchive>();
}

#[tokio::test]
async fn unencrypted_media_is_not_stored() {
    let archive = MemoryArchive::new();
    let mut event = text_event("$a", 1, "pic");
    event.content = json!({"msgtype": "m.image", "url": "mxc://messenger.test/x"});
    assert!(matches!(
        archive.capture(ROOM, event).await,
        Err(Error::Unencrypted)
    ));
    assert!(bodies(&archive, None, 10).await.is_empty());
}
