//! Acceptance: the full meet room/session lifecycle end-to-end through the
//! public usecase API over an in-memory store fake (cloud/persistence adapters
//! DEFERRED behind the port). Proves: fail-closed authz, host-owned room
//! creation, attendee join, host-driven close, and the room-opened protocol
//! event envelope parity — without any media/SFU or infra coupling.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use comms_meet_api::{
    AuthorizedMeetContext, CloseSessionRequest, JoinSessionRequest, MeetApiError, MeetSessionStore,
    OpenRoomRequest, meet_room_opened_event_envelope,
};
use comms_meet_domain::MeetSession;
use comms_meet_usecase::{MeetUsecaseError, close_session, join_session, open_room};

#[derive(Default)]
struct InMemoryMeetStore {
    sessions: BTreeMap<(String, String), MeetSession>,
}

impl MeetSessionStore for InMemoryMeetStore {
    type Session = MeetSession;

    fn put_session(&mut self, session: MeetSession) -> Result<(), MeetApiError> {
        let key = (session.tenant_id.value.clone(), session.id.value.clone());
        if self.sessions.contains_key(&key) {
            return Err(MeetApiError::Invalid);
        }
        self.sessions.insert(key, session);
        Ok(())
    }

    fn load_session(
        &self,
        tenant_scope_ref: &str,
        session_id: &str,
    ) -> Result<MeetSession, MeetApiError> {
        self.sessions
            .get(&(tenant_scope_ref.to_owned(), session_id.to_owned()))
            .cloned()
            .ok_or(MeetApiError::SessionNotFound)
    }

    fn update_session(&mut self, session: MeetSession) -> Result<(), MeetApiError> {
        let key = (session.tenant_id.value.clone(), session.id.value.clone());
        if !self.sessions.contains_key(&key) {
            return Err(MeetApiError::SessionNotFound);
        }
        self.sessions.insert(key, session);
        Ok(())
    }
}

fn ctx(principal: &str, decision: &str) -> AuthorizedMeetContext {
    AuthorizedMeetContext {
        tenant_scope_ref: "tenant:acme".into(),
        principal_ref: principal.into(),
        idempotency_key: "idem-acc-1".into(),
        policy_decision_ref: decision.into(),
        audit_correlation_id: "audit-acc-1".into(),
    }
}

#[test]
fn open_join_close_lifecycle_round_trip() {
    let mut store = InMemoryMeetStore::default();
    let host = ctx("user:host@acme.example", "cedar:allow:meet-open-room");

    let (opened, open_receipt) = open_room(
        &mut store,
        &host,
        OpenRoomRequest {
            room_id: "room-acc-1".into(),
            region: "region-eu1".into(),
            cell_id: "cell-eu1-a".into(),
            sfu_pool_id: "sfu-eu1-1".into(),
            host_actor_ref: "user:host@acme.example".into(),
            host_display_name: Some("Host".into()),
            started_at_epoch_seconds: 1_700_000_000,
        },
    )
    .unwrap();
    assert_eq!(opened.participants.value.len(), 1);
    assert_eq!(open_receipt.event_type, "meet.room.opened");

    // The room-opened receipt produces a parity-bound event envelope.
    let envelope = meet_room_opened_event_envelope(&host, &open_receipt).unwrap();
    assert_eq!(envelope.tenant_scope_ref, "tenant:acme");
    assert_eq!(envelope.aggregate_id, "room-acc-1");
    assert_eq!(envelope.policy_decision_ref, "cedar:allow:meet-open-room");

    let attendee = ctx("user:guest@acme.example", "cedar:allow:meet-join");
    let (joined, join_receipt) = join_session(
        &mut store,
        &attendee,
        JoinSessionRequest {
            session_id: "room-acc-1".into(),
            actor_ref: "user:guest@acme.example".into(),
            display_name: Some("Guest".into()),
            joined_at_epoch_seconds: 1_700_000_030,
        },
    )
    .unwrap();
    assert_eq!(joined.participants.value.len(), 2);
    assert_eq!(join_receipt.event_type, "meet.session.joined");

    let (closed, close_receipt) = close_session(
        &mut store,
        &host,
        CloseSessionRequest {
            session_id: "room-acc-1".into(),
            ended_at_epoch_seconds: 1_700_003_600,
        },
    )
    .unwrap();
    assert_eq!(closed.ended_at_epoch_seconds.value, Some(1_700_003_600));
    assert_eq!(close_receipt.event_type, "meet.session.closed");
}

#[test]
fn cross_tenant_load_is_isolated() {
    let mut store = InMemoryMeetStore::default();
    let host = ctx("user:host@acme.example", "cedar:allow:meet-open-room");
    open_room(
        &mut store,
        &host,
        OpenRoomRequest {
            room_id: "room-acc-2".into(),
            region: "region-eu1".into(),
            cell_id: "cell-eu1-a".into(),
            sfu_pool_id: "sfu-eu1-1".into(),
            host_actor_ref: "user:host@acme.example".into(),
            host_display_name: None,
            started_at_epoch_seconds: 1_700_000_000,
        },
    )
    .unwrap();

    // A different tenant scope cannot reach acme's session.
    let mut other = ctx("user:host@acme.example", "cedar:allow:meet-join");
    other.tenant_scope_ref = "tenant:globex".into();
    let res = join_session(
        &mut store,
        &other,
        JoinSessionRequest {
            session_id: "room-acc-2".into(),
            actor_ref: "user:host@acme.example".into(),
            display_name: None,
            joined_at_epoch_seconds: 1_700_000_030,
        },
    );
    assert_eq!(
        res,
        Err(MeetUsecaseError::Api(MeetApiError::SessionNotFound))
    );
}
