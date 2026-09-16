use messenger_push_api::{Push, PushError, PushNotice, PushPriority};
use messenger_push_memory::MemoryPush;
use serde_json::json;

fn notice(app: &str, key: &str) -> PushNotice {
    serde_json::from_value(json!({
        "event_id": "$event",
        "room_id": "!room:example.org",
        "counts": {"unread": 0, "missed_calls": 1},
        "prio": "high",
        "devices": [{"app_id": app, "pushkey": key}],
        "sender": "SECRET SENDER",
        "room_name": "SECRET ROOM",
        "content": {"body": "SECRET MESSAGE"},
        "type": "m.room.encrypted"
    }))
    .unwrap()
}

#[tokio::test]
async fn notify_records_routing_ids_and_counts_and_drops_secret_fields() {
    let push = MemoryPush::new(["android"]);
    assert_eq!(
        push.notify(&notice("android", "device-token")).await,
        Ok(vec![])
    );
    let delivered = push.delivered();
    assert_eq!(delivered.len(), 1);
    assert_eq!(delivered[0].event_id.as_deref(), Some("$event"));
    assert_eq!(delivered[0].room_id.as_deref(), Some("!room:example.org"));
    assert_eq!(delivered[0].counts.unread, Some(0));
    assert_eq!(delivered[0].counts.missed_calls, Some(1));
    assert_eq!(delivered[0].prio, PushPriority::High);
    assert_eq!(delivered[0].app_id, "android");
    let dump = format!("{delivered:?}");
    assert!(!dump.contains("SECRET"));
    assert!(!dump.contains("device-token"));
}

#[tokio::test]
async fn unknown_app_keys_are_returned_and_not_delivered() {
    let push = MemoryPush::new(["android"]);
    assert_eq!(
        push.notify(&notice("ios", "gone")).await,
        Ok(vec!["gone".into()])
    );
    assert!(push.delivered().is_empty());
}

#[tokio::test]
async fn invalid_notice_does_not_contact_the_store() {
    let push = MemoryPush::new(["android"]);
    let mut invalid = notice("android", "key");
    invalid.room_id = None;
    assert_eq!(push.notify(&invalid).await, Err(PushError::Invalid));
    assert!(push.delivered().is_empty());
}

#[tokio::test]
async fn mixed_devices_reject_only_unknown_apps() {
    let push = MemoryPush::new(["android"]);
    let mut n = notice("android", "keep");
    n.devices.push(messenger_push_api::PushDevice {
        app_id: "ios".into(),
        pushkey: "drop".into(),
        pushkey_ts: None,
    });
    assert_eq!(push.notify(&n).await, Ok(vec!["drop".into()]));
    let delivered = push.delivered();
    assert_eq!(delivered.len(), 1);
    assert_eq!(delivered[0].app_id, "android");
}
