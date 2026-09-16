use messenger_authority_memory::MemoryAuthority;
use messenger_conversation_api::RoomAuthority;
use messenger_domain::Error;

const ALICE: &str = "@alice:messenger.test";
const BOB: &str = "@bob:messenger.test";

#[tokio::test]
async fn create_room_assigns_a_server_local_id_and_joins_the_creator() {
    let store = MemoryAuthority::new("messenger.test");
    let room = store.create_room(ALICE, "public").await.unwrap();
    assert!(room.starts_with('!'));
    assert!(room.ends_with(":messenger.test"));
    let create = store
        .state(&room, "m.room.create", "")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(create.sender, ALICE);
    let sync = store.sync(ALICE, "DEVICE", None).await.unwrap();
    assert_eq!(sync.rooms.len(), 1);
    assert_eq!(sync.rooms[0].room, room);
    assert_eq!(sync.rooms[0].membership, "join");
}

#[tokio::test]
async fn snapshot_commit_is_one_compare_and_swap() {
    let store = MemoryAuthority::new("messenger.test");
    store.create_room(ALICE, "invite").await.unwrap();
    let snapshot = store.snapshot().await.unwrap();
    let expected = snapshot.generation;
    let mut next = snapshot.clone();
    next.seq = next.seq.saturating_add(1);
    store.commit(expected, next).await.unwrap();
    assert!(
        store
            .commit(expected, snapshot)
            .await
            .is_err_and(|error| matches!(error, Error::Unavailable(_)))
    );
}

#[tokio::test]
async fn invalid_creator_or_join_rule_is_rejected() {
    let store = MemoryAuthority::new("messenger.test");
    assert!(store.create_room("alice", "public").await.is_err());
    assert!(store.create_room(ALICE, "knock").await.is_err());
}

#[tokio::test]
async fn sync_hides_rooms_the_user_has_not_joined() {
    let store = MemoryAuthority::new("messenger.test");
    store.create_room(ALICE, "invite").await.unwrap();
    let sync = store.sync(BOB, "DEVICE", None).await.unwrap();
    assert!(sync.rooms.is_empty());
}
