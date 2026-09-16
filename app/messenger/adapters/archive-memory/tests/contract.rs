use messenger_archive_api::Archive;
use messenger_archive_memory::MemoryArchive;
use messenger_domain::{ArchiveEvent, Decryption, Error};
use serde_json::json;
use std::sync::Arc;

const ROOM: &str = "!room:messenger.test";
const ALICE: &str = "@alice:messenger.test";

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

#[test]
fn memory_adapter_is_shared_like_authority_memory() {
    fn assert_archive<T: Archive>() {}
    assert_archive::<MemoryArchive>();
    let _: Arc<MemoryArchive> = MemoryArchive::new();
}
