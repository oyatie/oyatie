use messenger_archive_api::{MAX_ARCHIVE_PAGE, validate_capture, validate_page};
use messenger_domain::{ArchiveEvent, Decryption, Error};
use serde_json::json;

const ROOM: &str = "!room:messenger.test";
const ALICE: &str = "@alice:messenger.test";

fn text_event() -> ArchiveEvent {
    ArchiveEvent {
        id: "$event".into(),
        sender: ALICE.into(),
        timestamp: 1,
        event_type: "m.room.message".into(),
        content: json!({"body": "hello", "msgtype": "m.text"}),
        decryption: Decryption::Decrypted,
    }
}

fn encrypted_file(size: u64) -> serde_json::Value {
    json!({
        "msgtype": "m.file",
        "body": "a.bin",
        "file": {
            "v": "v2",
            "url": "mxc://messenger.test/media",
            "key": {"kty": "oct", "k": "key"},
            "iv": "iv",
            "hashes": {"sha256": "hash"}
        },
        "info": {"size": size}
    })
}

#[test]
fn text_capture_is_admitted() {
    assert!(validate_capture(ROOM, &text_event()).is_ok());
}

#[test]
fn invalid_room_and_page_size_are_rejected() {
    assert!(matches!(
        validate_capture("room", &text_event()),
        Err(Error::Invalid(_))
    ));
    assert!(validate_page(ROOM, 1).is_ok());
    assert!(matches!(validate_page(ROOM, 0), Err(Error::Invalid(_))));
    assert!(matches!(
        validate_page(ROOM, MAX_ARCHIVE_PAGE + 1),
        Err(Error::Invalid(_))
    ));
}

#[test]
fn unencrypted_and_unready_media_are_refused() {
    let mut event = text_event();
    event.content = json!({"msgtype": "m.image", "url": "mxc://messenger.test/x"});
    assert!(matches!(
        validate_capture(ROOM, &event),
        Err(Error::Unencrypted)
    ));
    event.content = json!({"msgtype": "m.image"});
    assert!(matches!(
        validate_capture(ROOM, &event),
        Err(Error::ArchiveNotReady)
    ));
}

#[test]
fn oversized_encrypted_attachment_is_invalid() {
    let mut event = text_event();
    event.content = encrypted_file(20 * 1024 * 1024 + 1);
    assert!(matches!(
        validate_capture(ROOM, &event),
        Err(Error::Invalid(_))
    ));
    event.content = encrypted_file(20);
    assert!(validate_capture(ROOM, &event).is_ok());
}
