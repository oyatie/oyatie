use messenger_admission_usecase::{
    AdmitCommand, AuthorityRecord, admit_into, send_endpoint, txn_key,
};
use serde_json::json;

const ALICE: &str = "@alice:messenger.test";
const BOB: &str = "@bob:messenger.test";

fn send(room: &str, sender: &str, txn: &str, body: &str) -> AdmitCommand {
    AdmitCommand {
        room: room.into(),
        sender: sender.into(),
        device: "DEV".into(),
        event_type: "m.room.message".into(),
        state_key: None,
        content: json!({"body": body, "msgtype": "m.text"}),
        txn: Some(txn.into()),
        endpoint: send_endpoint(room, "m.room.message"),
    }
}

fn member(room: &str, sender: &str, target: &str, membership: &str) -> AdmitCommand {
    AdmitCommand {
        room: room.into(),
        sender: sender.into(),
        device: "DEV".into(),
        event_type: "m.room.member".into(),
        state_key: Some(target.into()),
        content: json!({"membership": membership}),
        txn: None,
        endpoint: format!("POST /_matrix/client/v3/rooms/{room}/{membership}"),
    }
}

fn public_room() -> (AuthorityRecord, String) {
    let mut state = AuthorityRecord::default();
    let room = state
        .create_room("!room:messenger.test".into(), ALICE, "public", 1)
        .unwrap();
    let (state, _) = admit_into(&state, &member(&room, BOB, BOB, "join")).unwrap();
    (state, room)
}

#[test]
fn lost_ack_reuses_the_committed_event_without_a_second_write() {
    let (state, room) = public_room();
    let command = send(&room, ALICE, "txn-1", "hello");
    let (state, first) = admit_into(&state, &command).unwrap();
    let (state, retry) = admit_into(&state, &command).unwrap();
    assert_eq!(first.event.event_id, retry.event.event_id);
    assert!(retry.reused);
    assert!(!first.reused);
    let bodies: Vec<_> = state
        .rooms
        .get(&room)
        .unwrap()
        .events
        .iter()
        .filter(|stored| stored.event.event_type == "m.room.message")
        .map(|stored| stored.event.content["body"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(bodies, vec!["hello"]);
}

#[test]
fn changed_retry_keeps_the_original_committed_event() {
    let (state, room) = public_room();
    let (state, first) = admit_into(&state, &send(&room, ALICE, "txn-1", "original")).unwrap();
    let (_, retry) = admit_into(&state, &send(&room, ALICE, "txn-1", "changed")).unwrap();
    assert_eq!(first.event.event_id, retry.event.event_id);
    assert!(retry.reused);
}

#[test]
fn txn_key_is_sender_device_endpoint_and_txn() {
    let command = send("!room:messenger.test", ALICE, "txn-1", "hello");
    let key = txn_key(&command).unwrap();
    assert!(key.contains(ALICE));
    assert!(key.contains("DEV"));
    assert!(key.contains("txn-1"));
    assert!(key.contains(&command.endpoint));
}

#[test]
fn outbox_identity_is_the_transaction_result() {
    let (state, room) = public_room();
    let command = send(&room, ALICE, "outbox", "work");
    let (state, _) = admit_into(&state, &command).unwrap();
    let (state, retry) = admit_into(&state, &command).unwrap();
    assert!(retry.reused);
    assert_eq!(
        state
            .rooms
            .get(&room)
            .unwrap()
            .events
            .iter()
            .filter(|stored| stored.event.event_type == "m.room.message")
            .count(),
        1
    );
}

#[test]
fn left_member_cannot_send() {
    let (state, room) = public_room();
    let (state, _) = admit_into(&state, &member(&room, BOB, BOB, "leave")).unwrap();
    assert!(admit_into(&state, &send(&room, BOB, "late", "no")).is_err());
}
