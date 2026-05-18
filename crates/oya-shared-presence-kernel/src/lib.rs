//! Shared presence kernel — Loro CRDT awareness protocol (ADR-0145
//! Loro pin) at the kernel boundary.
//!
//! Presence (who is online, where they are pointing, what selection
//! they have) is a CRDT-replicated awareness map. The kernel models
//! the awareness state as a per-participant snapshot; the wire-level
//! Loro / yjs-protocol-compatible diff stream is an adapter concern.
//!
//! What the kernel enforces:
//!
//! 1. Per-tenant isolation — a presence state belongs to exactly one
//!    tenant; cross-tenant peers cannot enter the same room.
//! 2. Participant identifier shape — non-empty + ASCII-printable.
//! 3. Cursor coordinates are bounded scalars (no `NaN`, no infinity).
//! 4. Stale-entry pruning — last-seen timestamps drive eviction so a
//!    crashed client doesn't haunt the room.
//!
//! ADR-0083 Tier 3 test exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TenantId(String); // data_class: INTERNAL_ONLY

impl TenantId {
    pub fn new(id: &str) -> Result<Self, PresenceError> {
        if id.is_empty() {
            return Err(PresenceError::EmptyTenantId);
        }
        Ok(Self(id.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ParticipantId(String); // data_class: INTERNAL_ONLY

impl ParticipantId {
    pub fn new(id: &str) -> Result<Self, PresenceError> {
        if id.is_empty() {
            return Err(PresenceError::EmptyParticipantId);
        }
        if !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(PresenceError::MalformedParticipantId {
                participant_id: id.to_owned(),
            });
        }
        Ok(Self(id.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Room identifier — scoped under a tenant. The kernel-level room is
/// the unit of collab; clients subscribe to a single room.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RoomKey {
    pub tenant_id: TenantId,
    pub room_id: String,
}

impl RoomKey {
    pub fn new(tenant_id: TenantId, room_id: String) -> Result<Self, PresenceError> {
        if room_id.is_empty() {
            return Err(PresenceError::EmptyRoomId);
        }
        Ok(Self { tenant_id, room_id })
    }
}

/// Cursor coordinates — bounded scalars. Kernel rejects `NaN`/`Inf`
/// (which would replicate via the CRDT and break downstream renderers).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CursorPosition {
    pub x: f64,
    pub y: f64,
}

impl CursorPosition {
    pub fn new(x: f64, y: f64) -> Result<Self, PresenceError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(PresenceError::CursorNotFinite);
        }
        Ok(Self { x, y })
    }
}

/// A single participant's awareness state.
#[derive(Clone, Debug, PartialEq)]
pub struct PresenceState {
    pub participant_id: ParticipantId,    // data_class: INTERNAL_ONLY
    pub cursor: Option<CursorPosition>,   // data_class: INTERNAL_ONLY
    pub selection_anchor: Option<String>, // data_class: INTERNAL_ONLY
    pub last_seen_unix_ms: u64,           // data_class: INTERNAL_ONLY
}

/// Presence tracker trait — adapters implement (Loro awareness is the
/// real impl; kernel ships an in-memory default).
pub trait PresenceTracker {
    fn join(&mut self, room: &RoomKey, state: PresenceState) -> Result<(), PresenceError>;
    fn leave(&mut self, room: &RoomKey, participant: &ParticipantId) -> Result<(), PresenceError>;
    fn participants(&self, room: &RoomKey) -> Vec<PresenceState>;
    /// Evict participants whose last-seen is older than
    /// `now_unix_ms - max_idle_ms`.
    fn prune_stale(&mut self, now_unix_ms: u64, max_idle_ms: u64) -> usize;
}

/// In-kernel `BTreeMap`-backed default. Adapters bring real Loro
/// awareness; this exists so the trait + invariants are testable
/// without adapter dependencies.
#[derive(Clone, Debug, Default)]
pub struct LoroPresenceTracker {
    rooms: BTreeMap<RoomKey, BTreeMap<ParticipantId, PresenceState>>, // data_class: INTERNAL_ONLY
}

impl LoroPresenceTracker {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PresenceTracker for LoroPresenceTracker {
    fn join(&mut self, room: &RoomKey, state: PresenceState) -> Result<(), PresenceError> {
        // Cross-tenant guard: callers must not splice participants from
        // one tenant's room into another. The room key carries the
        // tenant id; the kernel-level invariant is "no implicit
        // promotion" — the join lookup is by full room key. Belt-and-
        // suspenders: a participant id collision across tenants is
        // tolerated (different rooms), but inside one room two states
        // for the same participant id is an upsert, not a duplicate.
        self.rooms
            .entry(room.clone())
            .or_default()
            .insert(state.participant_id.clone(), state);
        Ok(())
    }

    fn leave(&mut self, room: &RoomKey, participant: &ParticipantId) -> Result<(), PresenceError> {
        let Some(entry) = self.rooms.get_mut(room) else {
            return Err(PresenceError::UnknownRoom);
        };
        if entry.remove(participant).is_none() {
            return Err(PresenceError::UnknownParticipant);
        }
        if entry.is_empty() {
            self.rooms.remove(room);
        }
        Ok(())
    }

    fn participants(&self, room: &RoomKey) -> Vec<PresenceState> {
        self.rooms
            .get(room)
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default()
    }

    fn prune_stale(&mut self, now_unix_ms: u64, max_idle_ms: u64) -> usize {
        let cutoff = now_unix_ms.saturating_sub(max_idle_ms);
        let mut pruned = 0usize;
        let mut empty_rooms: Vec<RoomKey> = Vec::new();
        for (room, members) in self.rooms.iter_mut() {
            let before = members.len();
            members.retain(|_, state| state.last_seen_unix_ms >= cutoff);
            pruned += before - members.len();
            if members.is_empty() {
                empty_rooms.push(room.clone());
            }
        }
        for k in empty_rooms {
            self.rooms.remove(&k);
        }
        pruned
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PresenceError {
    EmptyTenantId,
    EmptyParticipantId,
    MalformedParticipantId { participant_id: String },
    EmptyRoomId,
    CursorNotFinite,
    UnknownRoom,
    UnknownParticipant,
}

impl PresenceError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyTenantId => "tenant id is empty".to_owned(),
            Self::EmptyParticipantId => "participant id is empty".to_owned(),
            Self::MalformedParticipantId { participant_id } => {
                format!("participant id malformed: {participant_id}")
            }
            Self::EmptyRoomId => "room id is empty".to_owned(),
            Self::CursorNotFinite => "cursor coordinates must be finite".to_owned(),
            Self::UnknownRoom => "no presence state for that room".to_owned(),
            Self::UnknownParticipant => "no such participant in room".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rk(tenant: &str, room: &str) -> RoomKey {
        RoomKey::new(TenantId::new(tenant).unwrap(), room.to_owned()).unwrap()
    }
    fn state(id: &str, ts: u64) -> PresenceState {
        PresenceState {
            participant_id: ParticipantId::new(id).unwrap(),
            cursor: Some(CursorPosition::new(0.0, 0.0).unwrap()),
            selection_anchor: None,
            last_seen_unix_ms: ts,
        }
    }

    #[test]
    fn join_and_list_yields_inserted_states() {
        let mut t = LoroPresenceTracker::new();
        let room = rk("tenant_a", "canvas-1");
        t.join(&room, state("u_alice", 1_000)).unwrap();
        t.join(&room, state("u_bob", 1_001)).unwrap();
        assert_eq!(t.participants(&room).len(), 2);
    }

    #[test]
    fn cross_tenant_rooms_are_disjoint() {
        let mut t = LoroPresenceTracker::new();
        let room_a = rk("tenant_a", "canvas-1");
        let room_b = rk("tenant_b", "canvas-1");
        t.join(&room_a, state("u_alice", 1_000)).unwrap();
        // Same room id "canvas-1" under different tenant must NOT see
        // tenant_a participants.
        assert_eq!(t.participants(&room_b).len(), 0);
        assert_eq!(t.participants(&room_a).len(), 1);
    }

    #[test]
    fn leave_removes_participant_and_empty_room() {
        let mut t = LoroPresenceTracker::new();
        let room = rk("tenant_a", "canvas-1");
        t.join(&room, state("u_alice", 1_000)).unwrap();
        t.leave(&room, &ParticipantId::new("u_alice").unwrap())
            .unwrap();
        assert_eq!(t.participants(&room).len(), 0);
        // Unknown participant fails
        assert_eq!(
            t.leave(&room, &ParticipantId::new("u_alice").unwrap()),
            Err(PresenceError::UnknownRoom)
        );
    }

    #[test]
    fn cursor_rejects_non_finite_coordinates() {
        assert!(matches!(
            CursorPosition::new(f64::NAN, 1.0),
            Err(PresenceError::CursorNotFinite)
        ));
        assert!(matches!(
            CursorPosition::new(1.0, f64::INFINITY),
            Err(PresenceError::CursorNotFinite)
        ));
        assert!(CursorPosition::new(1.0, 2.0).is_ok());
    }

    #[test]
    fn prune_stale_evicts_old_participants() {
        let mut t = LoroPresenceTracker::new();
        let room = rk("tenant_a", "canvas-1");
        t.join(&room, state("u_alice", 1_000)).unwrap();
        t.join(&room, state("u_bob", 8_500)).unwrap();
        // now = 10_000, max_idle = 3_000 → cutoff 7_000.
        // alice (1000 < 7000) evicted; bob (8500 >= 7000) kept.
        let pruned = t.prune_stale(10_000, 3_000);
        assert_eq!(pruned, 1);
        let names: Vec<String> = t
            .participants(&room)
            .into_iter()
            .map(|s| s.participant_id.as_str().to_owned())
            .collect();
        assert_eq!(names, vec!["u_bob"]);
    }

    #[test]
    fn malformed_participant_id_rejected() {
        assert!(matches!(
            ParticipantId::new(""),
            Err(PresenceError::EmptyParticipantId)
        ));
        assert!(matches!(
            ParticipantId::new("u alice"),
            Err(PresenceError::MalformedParticipantId { .. })
        ));
    }

    #[test]
    fn join_upserts_same_participant() {
        let mut t = LoroPresenceTracker::new();
        let room = rk("tenant_a", "canvas-1");
        t.join(&room, state("u_alice", 1_000)).unwrap();
        t.join(&room, state("u_alice", 2_000)).unwrap();
        let ps = t.participants(&room);
        assert_eq!(ps.len(), 1);
        assert_eq!(ps[0].last_seen_unix_ms, 2_000);
    }
}
