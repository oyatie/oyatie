#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_shared_presence_kernel::{
    CursorPosition, LoroPresenceTracker, ParticipantId, PresenceState, PresenceTracker,
    RoomKey, TenantId,
};

fn rk(tenant: &str, room: &str) -> RoomKey {
    RoomKey::new(TenantId::new(tenant).unwrap(), room.to_owned()).unwrap()
}

#[test]
fn empty_room_returns_empty_participant_list() {
    let t = LoroPresenceTracker::new();
    let room = rk("tenant_a", "ghost-room");
    assert_eq!(t.participants(&room).len(), 0);
}

#[test]
fn prune_does_not_remove_recent_participants() {
    let mut t = LoroPresenceTracker::new();
    let room = rk("tenant_a", "canvas-1");
    t.join(
        &room,
        PresenceState {
            participant_id: ParticipantId::new("u_alice").unwrap(),
            cursor: Some(CursorPosition::new(5.0, 5.0).unwrap()),
            selection_anchor: Some("node_42".into()),
            last_seen_unix_ms: 9_500,
        },
    )
    .unwrap();
    let pruned = t.prune_stale(10_000, 3_000); // cutoff 7000 → alice 9500 stays
    assert_eq!(pruned, 0);
    assert_eq!(t.participants(&room).len(), 1);
}

#[test]
fn leave_unknown_room_yields_error() {
    let mut t = LoroPresenceTracker::new();
    let room = rk("tenant_a", "never-joined");
    let pid = ParticipantId::new("u_alice").unwrap();
    assert!(t.leave(&room, &pid).is_err());
}

#[test]
fn presence_state_carries_selection_anchor() {
    let s = PresenceState {
        participant_id: ParticipantId::new("u_alice").unwrap(),
        cursor: None,
        selection_anchor: Some("step_3.out".into()),
        last_seen_unix_ms: 100,
    };
    assert_eq!(s.selection_anchor.as_deref(), Some("step_3.out"));
}

#[test]
fn multi_room_prune_yields_correct_total() {
    let mut t = LoroPresenceTracker::new();
    let r1 = rk("tenant_a", "room-1");
    let r2 = rk("tenant_a", "room-2");
    let stale = PresenceState {
        participant_id: ParticipantId::new("u_stale").unwrap(),
        cursor: None,
        selection_anchor: None,
        last_seen_unix_ms: 100,
    };
    t.join(&r1, stale.clone()).unwrap();
    t.join(&r2, stale).unwrap();
    let pruned = t.prune_stale(10_000, 3_000);
    assert_eq!(pruned, 2);
    assert_eq!(t.participants(&r1).len(), 0);
    assert_eq!(t.participants(&r2).len(), 0);
}
