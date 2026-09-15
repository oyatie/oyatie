use messenger_authority_memory::MemoryAuthority;
use messenger_conversation_api::RoomAuthority;
use messenger_domain::{AdmitCommand, Error, send_endpoint};
use serde_json::json;

const ALICE: &str = "@alice:messenger.test";
const BOB: &str = "@bob:messenger.test";

fn send(room: &str, sender: &str, device: &str, txn: &str, body: &str) -> AdmitCommand {
    AdmitCommand {
        room: room.into(),
        sender: sender.into(),
        device: device.into(),
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
        device: "DEVICE".into(),
        event_type: "m.room.member".into(),
        state_key: Some(target.into()),
        content: json!({"membership": membership}),
        txn: None,
        endpoint: format!("POST /_matrix/client/v3/rooms/{room}/{membership}"),
    }
}

async fn public_room() -> (std::sync::Arc<MemoryAuthority>, String) {
    let store = MemoryAuthority::new("messenger.test");
    let room = store.create_room(ALICE, "public").await.unwrap();
    store.admit(member(&room, BOB, BOB, "join")).await.unwrap();
    (store, room)
}

async fn bodies(store: &MemoryAuthority, user: &str, room: &str) -> Vec<String> {
    store
        .sync(user, "DEVICE", None)
        .await
        .unwrap()
        .rooms
        .into_iter()
        .find(|delta| delta.room == room)
        .unwrap()
        .events
        .into_iter()
        .filter(|event| event.event_type == "m.room.message")
        .map(|event| event.content["body"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn lost_ack_retries_return_the_same_event_and_do_not_duplicate() {
    let (store, room) = public_room().await;
    let first = store
        .admit(send(&room, ALICE, "DEV", "txn-1", "hello"))
        .await
        .unwrap();
    let retry = store
        .admit(send(&room, ALICE, "DEV", "txn-1", "hello"))
        .await
        .unwrap();
    assert_eq!(first.event.event_id, retry.event.event_id);
    assert!(retry.reused);
    assert_eq!(bodies(&store, ALICE, &room).await, vec!["hello"]);
}

#[tokio::test]
async fn changed_matrix_retry_keeps_the_original_committed_event() {
    let (store, room) = public_room().await;
    let first = store
        .admit(send(&room, ALICE, "DEV", "txn-1", "original"))
        .await
        .unwrap();
    let retry = store
        .admit(send(&room, ALICE, "DEV", "txn-1", "changed"))
        .await
        .unwrap();
    assert_eq!(first.event.event_id, retry.event.event_id);
    assert!(retry.reused);
    assert_eq!(bodies(&store, ALICE, &room).await, vec!["original"]);
}

#[tokio::test]
async fn token_refresh_is_outside_transaction_scope_same_device_reuses_txn() {
    let (store, room) = public_room().await;
    let first = store
        .admit(send(&room, ALICE, "DEV", "txn-1", "hello"))
        .await
        .unwrap();
    let retry = store
        .admit(send(&room, ALICE, "DEV", "txn-1", "hello"))
        .await
        .unwrap();
    assert_eq!(first.event.event_id, retry.event.event_id);
}

#[tokio::test]
async fn concurrent_send_and_leave_never_accepts_a_message_from_a_left_member() {
    let (store, room) = public_room().await;
    let send_cmd = send(&room, BOB, "DEV", "txn-send", "maybe");
    let leave_cmd = member(&room, BOB, BOB, "leave");
    let (sent, left) = tokio::join!(store.admit(send_cmd), store.admit(leave_cmd));
    left.expect("leave must commit");
    match sent {
        Ok(admission) => {
            let events = store
                .sync(ALICE, "DEVICE", None)
                .await
                .unwrap()
                .rooms
                .into_iter()
                .find(|delta| delta.room == room)
                .unwrap()
                .events;
            let pos = events
                .iter()
                .position(|event| event.event_id == admission.event.event_id)
                .unwrap();
            let left_at = events
                .iter()
                .position(|event| {
                    event.event_type == "m.room.member"
                        && event.state_key.as_deref() == Some(BOB)
                        && event.content["membership"] == "leave"
                })
                .unwrap();
            assert!(pos < left_at, "accepted send must precede leave");
        }
        Err(Error::Denied) => {}
        Err(other) => panic!("unexpected send failure {other}"),
    }
    assert!(
        store
            .admit(send(&room, BOB, "DEV", "txn-after", "too late"))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn concurrent_send_and_ban_keeps_a_single_coherent_membership() {
    let (store, room) = public_room().await;
    let (sent, banned) = tokio::join!(
        store.admit(send(&room, BOB, "DEV", "txn-b", "hello")),
        store.admit(member(&room, ALICE, BOB, "ban"))
    );
    banned.unwrap();
    match sent {
        Ok(_) | Err(Error::Denied) => {}
        Err(other) => panic!("unexpected {other}"),
    }
    assert!(store.admit(member(&room, BOB, BOB, "join")).await.is_err());
}

#[tokio::test]
async fn unknown_commit_retry_stores_one_event() {
    let (store, room) = public_room().await;
    let first = store
        .admit(send(&room, ALICE, "DEV", "lost-ack", "once"))
        .await
        .unwrap();
    let retry = store
        .admit(send(&room, ALICE, "DEV", "lost-ack", "once"))
        .await
        .unwrap();
    assert_eq!(first.event.event_id, retry.event.event_id);
    assert_eq!(bodies(&store, ALICE, &room).await, vec!["once"]);
}

#[tokio::test]
async fn outbox_identity_is_the_transaction_result_not_a_second_write() {
    let (store, room) = public_room().await;
    store
        .admit(send(&room, ALICE, "DEV", "outbox", "work"))
        .await
        .unwrap();
    store
        .admit(send(&room, ALICE, "DEV", "outbox", "work"))
        .await
        .unwrap();
    assert_eq!(bodies(&store, ALICE, &room).await.len(), 1);
}
