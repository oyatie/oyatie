use messenger_admission_usecase::admit;
use messenger_authority_memory::MemoryAuthority;
use messenger_conversation_api::RoomAuthority;
use messenger_domain::{
    AdmitCommand, AuthorityEvent, AuthorityRecord, AuthoritySync, Error, send_endpoint,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

const ALICE: &str = "@alice:messenger.test";
const BOB: &str = "@bob:messenger.test";

struct Contending {
    inner: Arc<MemoryAuthority>,
    fail_commits: Mutex<u32>,
}

impl Contending {
    fn new(inner: Arc<MemoryAuthority>, fail_commits: u32) -> Arc<Self> {
        Arc::new(Self {
            inner,
            fail_commits: Mutex::new(fail_commits),
        })
    }
}

impl RoomAuthority for Contending {
    async fn create_room(&self, creator: &str, join_rule: &str) -> Result<String, Error> {
        self.inner.create_room(creator, join_rule).await
    }

    async fn snapshot(&self) -> Result<AuthorityRecord, Error> {
        self.inner.snapshot().await
    }

    async fn commit(&self, expected_generation: u64, record: AuthorityRecord) -> Result<(), Error> {
        let mut remaining = self.fail_commits.lock().await;
        if *remaining > 0 {
            *remaining -= 1;
            return Err(Error::Unavailable("injected contention".into()));
        }
        drop(remaining);
        self.inner.commit(expected_generation, record).await
    }

    async fn sync(
        &self,
        user: &str,
        device: &str,
        since: Option<&str>,
    ) -> Result<AuthoritySync, Error> {
        self.inner.sync(user, device, since).await
    }

    async fn state(
        &self,
        room: &str,
        event_type: &str,
        state_key: &str,
    ) -> Result<Option<AuthorityEvent>, Error> {
        self.inner.state(room, event_type, state_key).await
    }
}

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

async fn public_room() -> (Arc<MemoryAuthority>, String) {
    let store = MemoryAuthority::new("messenger.test");
    let room = store.create_room(ALICE, "public").await.unwrap();
    admit(&*store, &member(&room, BOB, BOB, "join"))
        .await
        .unwrap();
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
    let first = admit(&*store, &send(&room, ALICE, "DEV", "txn-1", "hello"))
        .await
        .unwrap();
    let retry = admit(&*store, &send(&room, ALICE, "DEV", "txn-1", "hello"))
        .await
        .unwrap();
    assert_eq!(first.event.event_id, retry.event.event_id);
    assert!(retry.reused);
    assert_eq!(bodies(&store, ALICE, &room).await, vec!["hello"]);
}

#[tokio::test]
async fn changed_matrix_retry_keeps_the_original_committed_event() {
    let (store, room) = public_room().await;
    let first = admit(&*store, &send(&room, ALICE, "DEV", "txn-1", "original"))
        .await
        .unwrap();
    let retry = admit(&*store, &send(&room, ALICE, "DEV", "txn-1", "changed"))
        .await
        .unwrap();
    assert_eq!(first.event.event_id, retry.event.event_id);
    assert!(retry.reused);
    assert_eq!(bodies(&store, ALICE, &room).await, vec!["original"]);
}

#[tokio::test]
async fn token_refresh_is_outside_transaction_scope_same_device_reuses_txn() {
    let (store, room) = public_room().await;
    let first = admit(&*store, &send(&room, ALICE, "DEV", "txn-1", "hello"))
        .await
        .unwrap();
    let retry = admit(&*store, &send(&room, ALICE, "DEV", "txn-1", "hello"))
        .await
        .unwrap();
    assert_eq!(first.event.event_id, retry.event.event_id);
}

#[tokio::test]
async fn concurrent_send_and_leave_never_accepts_a_message_from_a_left_member() {
    let (store, room) = public_room().await;
    let send_cmd = send(&room, BOB, "DEV", "txn-send", "maybe");
    let leave_cmd = member(&room, BOB, BOB, "leave");
    let (sent, left) = tokio::join!(admit(&*store, &send_cmd), admit(&*store, &leave_cmd));
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
        admit(&*store, &send(&room, BOB, "DEV", "txn-after", "too late"))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn concurrent_send_and_ban_keeps_a_single_coherent_membership() {
    let (store, room) = public_room().await;
    let send_cmd = send(&room, BOB, "DEV", "txn-b", "hello");
    let ban_cmd = member(&room, ALICE, BOB, "ban");
    let (sent, banned) = tokio::join!(admit(&*store, &send_cmd), admit(&*store, &ban_cmd));
    banned.unwrap();
    match sent {
        Ok(_) | Err(Error::Denied) => {}
        Err(other) => panic!("unexpected {other}"),
    }
    assert!(
        admit(&*store, &member(&room, BOB, BOB, "join"))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn serializable_conflict_retries_and_commits_once() {
    let (store, room) = public_room().await;
    let command = send(&room, ALICE, "DEV", "retry", "once");
    let snapshot = store.snapshot().await.unwrap();
    let first = admit(&*store, &command).await.unwrap();
    assert!(store.commit(snapshot.generation, snapshot).await.is_err());
    let retry = admit(&*store, &command).await.unwrap();
    assert_eq!(first.event.event_id, retry.event.event_id);
    assert!(retry.reused);
    assert_eq!(bodies(&store, ALICE, &room).await, vec!["once"]);
}

#[tokio::test]
async fn admit_retries_serializable_conflicts_then_commits() {
    let (store, room) = public_room().await;
    let flaky = Contending::new(store.clone(), 1);
    let admission = admit(&*flaky, &send(&room, ALICE, "DEV", "retry-id", "once"))
        .await
        .unwrap();
    assert!(!admission.reused);
    assert_eq!(bodies(&store, ALICE, &room).await, vec!["once"]);
}

#[tokio::test]
async fn invalid_commands_are_not_retried() {
    let store = MemoryAuthority::new("messenger.test");
    let mut command = send("!room:messenger.test", ALICE, "DEV", "txn", "hello");
    command.sender = "alice".into();
    assert!(matches!(
        admit(&*store, &command).await,
        Err(Error::Invalid(_))
    ));
}
