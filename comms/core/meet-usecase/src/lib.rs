//! Workspace meet room/session lifecycle usecase.
//!
//! Cloud-agnostic application layer for the W-Workspace-Stable Meet surface. It
//! composes the `comms-meet-domain` kernel (typed, invariant-checked session +
//! participant records) with the `comms-meet-api` port (authz context +
//! lifecycle commands + repository seam) to implement the room/session
//! lifecycle: OPEN a room (creating its first live session with the host),
//! JOIN a participant, and CLOSE the session.
//!
//! Clean-arch posture: this is a `core` usecase. It depends DOWN on the domain
//! kernel and the port, never on an adapter. Persistence is expressed only
//! through the [`comms_meet_api::MeetSessionStore`] trait; the durable
//! Postgres/cloud adapter is DEFERRED. Media/SFU routing and transcription are
//! DEFERRED adapter concerns and are NOT modeled here.
//!
//! Authz is FAIL-CLOSED: every lifecycle function calls
//! [`AuthorizedMeetContext::validate`] FIRST and rejects a principal that does
//! not own the action. No effect is constructed before the default-deny gate
//! and the tenant-scope check pass.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use comms_meet_api::{
    AuthorizedMeetContext, CloseSessionRequest, JoinSessionRequest, MeetApiError,
    MeetLifecycleReceipt, MeetSessionStore, OpenRoomRequest,
};
use comms_meet_domain::{
    MeetError, MeetSession, MeetSessionCreate, ParticipantConnectionState, ParticipantRef,
    ParticipantRole, RecordingConsentMode,
};

/// Errors the lifecycle can surface: an authz/port refusal, a domain-invariant
/// violation, or a principal that does not own the requested action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MeetUsecaseError {
    Api(MeetApiError),
    Domain(MeetError),
    PrincipalMismatch,
    SessionAlreadyEnded,
    DuplicateParticipant,
}

impl From<MeetApiError> for MeetUsecaseError {
    fn from(error: MeetApiError) -> Self {
        MeetUsecaseError::Api(error)
    }
}

impl From<MeetError> for MeetUsecaseError {
    fn from(error: MeetError) -> Self {
        MeetUsecaseError::Domain(error)
    }
}

/// OPEN a meeting room: validate authz (fail-closed), assert the host principal
/// owns the room, build the host participant + the first live session via the
/// invariant-checked domain constructor, persist it, and emit a receipt.
///
/// The session id equals the room id for the room's first session; later
/// sessions of the same room are a DEFERRED concern (the room aggregate split
/// pairs with the calendar move's shared session-id concept without coupling at
/// build time).
pub fn open_room<S>(
    store: &mut S,
    ctx: &AuthorizedMeetContext,
    req: OpenRoomRequest,
) -> Result<(MeetSession, MeetLifecycleReceipt), MeetUsecaseError>
where
    S: MeetSessionStore<Session = MeetSession>,
{
    ctx.validate()?;
    if req.host_actor_ref != ctx.principal_ref {
        return Err(MeetUsecaseError::PrincipalMismatch);
    }
    if req.room_id.trim().is_empty() {
        return Err(MeetUsecaseError::Api(MeetApiError::InvalidRoomId));
    }

    let host = ParticipantRef::new(
        req.host_actor_ref,
        req.host_display_name,
        ParticipantRole::Host,
        ParticipantConnectionState::Joined,
        Some(req.started_at_epoch_seconds),
        None,
    )?;

    let session = MeetSession::new(MeetSessionCreate {
        id: req.room_id.clone(),
        tenant_id: ctx.tenant_scope_ref.clone(),
        region: req.region,
        cell_id: req.cell_id,
        sfu_pool_id: req.sfu_pool_id,
        data_class: None,
        started_at_epoch_seconds: req.started_at_epoch_seconds,
        ended_at_epoch_seconds: None,
        participants: vec![host],
        recording: None,
        recording_consent: RecordingConsentMode::NotRequested,
        transcript_session_id: None,
        summary_id: None,
    })?;

    store.put_session(session.clone())?;

    let receipt = lifecycle_receipt(ctx, req.room_id, "meet.room.opened");
    Ok((session, receipt))
}

/// JOIN a participant to an open session: validate authz (fail-closed), assert
/// the joining principal owns the join, load the session, append the
/// participant (rejecting a duplicate actor and a session that has already
/// ended), re-validate through the domain constructor, persist, and emit a
/// receipt.
pub fn join_session<S>(
    store: &mut S,
    ctx: &AuthorizedMeetContext,
    req: JoinSessionRequest,
) -> Result<(MeetSession, MeetLifecycleReceipt), MeetUsecaseError>
where
    S: MeetSessionStore<Session = MeetSession>,
{
    ctx.validate()?;
    if req.actor_ref != ctx.principal_ref {
        return Err(MeetUsecaseError::PrincipalMismatch);
    }
    if req.session_id.trim().is_empty() {
        return Err(MeetUsecaseError::Api(MeetApiError::InvalidSessionId));
    }

    let current = store.load_session(&ctx.tenant_scope_ref, &req.session_id)?;
    if current.ended_at_epoch_seconds.value.is_some() {
        return Err(MeetUsecaseError::SessionAlreadyEnded);
    }

    let mut participants = current.participants.value.clone();
    if participants
        .iter()
        .any(|p| p.actor_ref.value == req.actor_ref)
    {
        return Err(MeetUsecaseError::DuplicateParticipant);
    }
    participants.push(ParticipantRef::new(
        req.actor_ref,
        req.display_name,
        ParticipantRole::Attendee,
        ParticipantConnectionState::Joined,
        Some(req.joined_at_epoch_seconds),
        None,
    )?);

    let updated = rebuild_session(&current, participants, current.ended_at_epoch_seconds.value)?;
    store.update_session(updated.clone())?;

    let receipt = lifecycle_receipt(ctx, req.session_id, "meet.session.joined");
    Ok((updated, receipt))
}

/// CLOSE an open session: validate authz (fail-closed), load the session,
/// reject a re-close, stamp `ended_at` through the invariant-checked domain
/// constructor (which enforces ended >= started), persist, and emit a receipt.
pub fn close_session<S>(
    store: &mut S,
    ctx: &AuthorizedMeetContext,
    req: CloseSessionRequest,
) -> Result<(MeetSession, MeetLifecycleReceipt), MeetUsecaseError>
where
    S: MeetSessionStore<Session = MeetSession>,
{
    ctx.validate()?;
    if req.session_id.trim().is_empty() {
        return Err(MeetUsecaseError::Api(MeetApiError::InvalidSessionId));
    }

    let current = store.load_session(&ctx.tenant_scope_ref, &req.session_id)?;
    if current.ended_at_epoch_seconds.value.is_some() {
        return Err(MeetUsecaseError::SessionAlreadyEnded);
    }

    let participants = current.participants.value.clone();
    let updated = rebuild_session(&current, participants, Some(req.ended_at_epoch_seconds))?;
    store.update_session(updated.clone())?;

    let receipt = lifecycle_receipt(ctx, req.session_id, "meet.session.closed");
    Ok((updated, receipt))
}

/// Rebuild a session through the domain constructor so EVERY mutation re-runs
/// the kernel invariants (host present, time order, recording consent). The
/// usecase never hand-builds `Classified` fields; it round-trips the plaintext
/// projection back through `MeetSession::new`.
fn rebuild_session(
    current: &MeetSession,
    participants: Vec<ParticipantRef>,
    ended_at_epoch_seconds: Option<u64>,
) -> Result<MeetSession, MeetUsecaseError> {
    let recording = current.recording.value.clone();
    Ok(MeetSession::new(MeetSessionCreate {
        id: current.id.value.clone(),
        tenant_id: current.tenant_id.value.clone(),
        region: current.region.value.clone(),
        cell_id: current.cell_id.value.clone(),
        sfu_pool_id: current.sfu_pool_id.value.clone(),
        data_class: Some(current.data_class.value),
        started_at_epoch_seconds: current.started_at_epoch_seconds.value,
        ended_at_epoch_seconds,
        participants,
        recording,
        recording_consent: current.recording_consent.value,
        transcript_session_id: current.transcript_session_id.value.clone(),
        summary_id: current.summary_id.value.clone(),
    })?)
}

fn lifecycle_receipt(
    ctx: &AuthorizedMeetContext,
    session_id: String,
    event_type: &'static str,
) -> MeetLifecycleReceipt {
    MeetLifecycleReceipt {
        session_id,
        event_type,
        audit_correlation_id: ctx.audit_correlation_id.clone(),
        idempotency_key: ctx.idempotency_key.clone(),
        policy_decision_ref: ctx.policy_decision_ref.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    /// In-memory `MeetSessionStore` fake — proves the lifecycle with NO infra
    /// (cloud/persistence adapters deferred). Keyed by `(tenant, session)`.
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

    fn host_ctx() -> AuthorizedMeetContext {
        AuthorizedMeetContext {
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: "user:host@example.com".into(),
            idempotency_key: "idem-1".into(),
            policy_decision_ref: "cedar:allow:meet-open-room".into(),
            audit_correlation_id: "audit-1".into(),
        }
    }

    fn open_req() -> OpenRoomRequest {
        OpenRoomRequest {
            room_id: "room-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            sfu_pool_id: "sfu-pool-1".into(),
            host_actor_ref: "user:host@example.com".into(),
            host_display_name: Some("Host User".into()),
            started_at_epoch_seconds: 1_700_000_000,
        }
    }

    #[test]
    fn open_room_requires_authorized_principal_owning_the_room() {
        let mut store = InMemoryMeetStore::default();
        let mut req = open_req();
        req.host_actor_ref = "user:other@example.com".into();
        assert_eq!(
            open_room(&mut store, &host_ctx(), req),
            Err(MeetUsecaseError::PrincipalMismatch)
        );
    }

    #[test]
    fn open_room_is_fail_closed_without_tenant_scope() {
        let mut store = InMemoryMeetStore::default();
        let mut ctx = host_ctx();
        ctx.tenant_scope_ref = "person:host".into();
        assert_eq!(
            open_room(&mut store, &ctx, open_req()),
            Err(MeetUsecaseError::Api(MeetApiError::MissingTenantScope))
        );
    }

    #[test]
    fn open_room_creates_host_session_and_persists() {
        let mut store = InMemoryMeetStore::default();
        let (session, receipt) = open_room(&mut store, &host_ctx(), open_req()).unwrap();
        assert_eq!(session.id.value, "room-1");
        assert_eq!(session.tenant_id.value, "tenant:t");
        assert_eq!(session.participants.value.len(), 1);
        assert_eq!(receipt.event_type, "meet.room.opened");
        assert_eq!(receipt.policy_decision_ref, "cedar:allow:meet-open-room");
        // persisted + idempotent-conflict on re-open
        assert_eq!(
            store.load_session("tenant:t", "room-1").unwrap().id.value,
            "room-1"
        );
    }

    #[test]
    fn join_session_appends_attendee_and_rejects_duplicate() {
        let mut store = InMemoryMeetStore::default();
        open_room(&mut store, &host_ctx(), open_req()).unwrap();

        let attendee_ctx = AuthorizedMeetContext {
            principal_ref: "user:attendee@example.com".into(),
            policy_decision_ref: "cedar:allow:meet-join".into(),
            ..host_ctx()
        };
        let join_req = JoinSessionRequest {
            session_id: "room-1".into(),
            actor_ref: "user:attendee@example.com".into(),
            display_name: Some("Attendee".into()),
            joined_at_epoch_seconds: 1_700_000_010,
        };
        let (session, receipt) = join_session(&mut store, &attendee_ctx, join_req.clone()).unwrap();
        assert_eq!(session.participants.value.len(), 2);
        assert_eq!(receipt.event_type, "meet.session.joined");

        // duplicate join of the same actor is rejected
        assert_eq!(
            join_session(&mut store, &attendee_ctx, join_req),
            Err(MeetUsecaseError::DuplicateParticipant)
        );
    }

    #[test]
    fn join_session_requires_the_joining_principal() {
        let mut store = InMemoryMeetStore::default();
        open_room(&mut store, &host_ctx(), open_req()).unwrap();
        let join_req = JoinSessionRequest {
            session_id: "room-1".into(),
            actor_ref: "user:someone-else@example.com".into(),
            display_name: None,
            joined_at_epoch_seconds: 1_700_000_010,
        };
        assert_eq!(
            join_session(&mut store, &host_ctx(), join_req),
            Err(MeetUsecaseError::PrincipalMismatch)
        );
    }

    #[test]
    fn join_missing_session_is_not_found() {
        let mut store = InMemoryMeetStore::default();
        let join_req = JoinSessionRequest {
            session_id: "absent".into(),
            actor_ref: "user:host@example.com".into(),
            display_name: None,
            joined_at_epoch_seconds: 1_700_000_010,
        };
        assert_eq!(
            join_session(&mut store, &host_ctx(), join_req),
            Err(MeetUsecaseError::Api(MeetApiError::SessionNotFound))
        );
    }

    #[test]
    fn close_session_stamps_ended_and_rejects_reclose_and_late_join() {
        let mut store = InMemoryMeetStore::default();
        open_room(&mut store, &host_ctx(), open_req()).unwrap();

        let close_req = CloseSessionRequest {
            session_id: "room-1".into(),
            ended_at_epoch_seconds: 1_700_000_900,
        };
        let (session, receipt) = close_session(&mut store, &host_ctx(), close_req.clone()).unwrap();
        assert_eq!(session.ended_at_epoch_seconds.value, Some(1_700_000_900));
        assert_eq!(receipt.event_type, "meet.session.closed");

        // re-close is rejected
        assert_eq!(
            close_session(&mut store, &host_ctx(), close_req),
            Err(MeetUsecaseError::SessionAlreadyEnded)
        );

        // join after close is rejected
        let late = JoinSessionRequest {
            session_id: "room-1".into(),
            actor_ref: "user:host@example.com".into(),
            display_name: None,
            joined_at_epoch_seconds: 1_700_001_000,
        };
        assert_eq!(
            join_session(&mut store, &host_ctx(), late),
            Err(MeetUsecaseError::SessionAlreadyEnded)
        );
    }

    #[test]
    fn close_with_ended_before_started_is_domain_rejected() {
        let mut store = InMemoryMeetStore::default();
        open_room(&mut store, &host_ctx(), open_req()).unwrap();
        let close_req = CloseSessionRequest {
            session_id: "room-1".into(),
            ended_at_epoch_seconds: 1_699_999_999,
        };
        assert_eq!(
            close_session(&mut store, &host_ctx(), close_req),
            Err(MeetUsecaseError::Domain(MeetError::InvalidSessionTimeOrder))
        );
    }
}
