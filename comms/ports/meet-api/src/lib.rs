//! Workspace meet port surface (room/session lifecycle).
//!
//! Cloud-agnostic control-plane port for the W-Workspace-Stable Meet surface
//! (`docs/products/workspace/PRD.md`, ADR-0029). This crate owns the typed
//! lifecycle commands, the authorized request context, and the protocol-parity
//! binding for the room/session lifecycle on top of the
//! `comms-meet-domain` kernel.
//!
//! Clean-arch posture: this is a `ports` crate. It defines the seam — the
//! `MeetSessionStore` repository trait and the typed lifecycle requests — that
//! the cloud/persistence/identity adapters implement BEHIND. Media/SFU routing,
//! transcription engines, and durable archive storage remain DEFERRED adapter
//! concerns; this slice models only the room/session lifecycle.
//!
//! Authz is FAIL-CLOSED (founder new-HTTP-surface doctrine): every request
//! carries a verified principal, a tenant scope, an explicit policy-decision
//! ref (cloud-iam PDP decision), an idempotency key, and an audit-correlation
//! id. [`AuthorizedMeetContext::validate`] is the single default-deny gate; a
//! usecase that does not call it cannot construct a lifecycle effect.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use shared_protocol_parity_kernel::{
    ProtocolEventEnvelope, ProtocolParityBinding, ProtocolParityBindingSpec, ProtocolParityError,
    require_receipt_event_type,
};

/// Port-surface validation + lookup errors. Fail-closed: a missing authz fact
/// is a distinct, named refusal, never a silent allow.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MeetApiError {
    Invalid,
    MissingTenantScope,
    MissingPrincipal,
    MissingPolicyDecision,
    MissingIdempotencyKey,
    MissingAuditCorrelation,
    InvalidRoomId,
    InvalidSessionId,
    EmptyParticipantSet,
    SessionNotFound,
}

/// A request context whose authz facts have been resolved upstream (verified
/// principal + cloud-iam PDP decision). Construction does NOT imply
/// authorization; [`AuthorizedMeetContext::validate`] is the default-deny gate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizedMeetContext {
    pub tenant_scope_ref: String,
    pub principal_ref: String,
    pub idempotency_key: String,
    pub policy_decision_ref: String,
    pub audit_correlation_id: String,
}

/// Open-room command. The room is the durable lifecycle aggregate; a session is
/// the live instantiation of a room. Region/cell/sfu_pool are placement hints
/// the cloud adapter resolves; the port stays cloud-agnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenRoomRequest {
    pub room_id: String,
    pub region: String,
    pub cell_id: String,
    pub sfu_pool_id: String,
    pub host_actor_ref: String,
    pub host_display_name: Option<String>,
    pub started_at_epoch_seconds: u64,
}

/// Join-session command — admit a participant to an open room's live session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JoinSessionRequest {
    pub session_id: String,
    pub actor_ref: String,
    pub display_name: Option<String>,
    pub joined_at_epoch_seconds: u64,
}

/// Close-session command — end the live session at `ended_at`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloseSessionRequest {
    pub session_id: String,
    pub ended_at_epoch_seconds: u64,
}

/// Lifecycle receipt — carries the policy/audit/idempotency provenance the
/// event envelope and the audit log require.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeetLifecycleReceipt {
    pub session_id: String,
    pub event_type: &'static str,
    pub audit_correlation_id: String,
    pub idempotency_key: String,
    pub policy_decision_ref: String,
}

/// Repository port for meet session lifecycle persistence. DEFINED in the port
/// (clean-arch ports-in-core); the durable Postgres/cloud adapter IMPLEMENTS it
/// and is DEFERRED. The in-memory test fake in the usecase crate proves the
/// lifecycle without coupling the build to any infra.
pub trait MeetSessionStore {
    type Session;

    /// Persist a freshly opened session keyed by its id. Fail-closed: the
    /// implementor must reject a duplicate id rather than silently overwrite.
    fn put_session(&mut self, session: Self::Session) -> Result<(), MeetApiError>;

    /// Load a session by `(tenant_scope_ref, session_id)`. Cross-tenant reads
    /// are an isolation defect; the durable adapter enforces RLS behind this.
    fn load_session(
        &self,
        tenant_scope_ref: &str,
        session_id: &str,
    ) -> Result<Self::Session, MeetApiError>;

    /// Replace an existing session (join/close mutate the aggregate).
    fn update_session(&mut self, session: Self::Session) -> Result<(), MeetApiError>;
}

/// Protocol schema version for the meet room/session lifecycle events.
pub const MEET_ROOM_OPENED_PROTOCOL_SCHEMA_VERSION: &str = "1.0.0";

/// REST/AsyncAPI/proto parity binding for the room-opened lifecycle event,
/// mirroring messenger's binding discipline so the facade surfaces stay in
/// parity across the three transports.
pub fn meet_room_opened_protocol_binding() -> Result<ProtocolParityBinding, ProtocolParityError> {
    ProtocolParityBinding::new(ProtocolParityBindingSpec {
        rest_operation_id: "openRoom",
        asyncapi_operation_id: "emitMeetRoomOpened",
        asyncapi_channel_address: "workflow-events/meet.room.opened",
        asyncapi_message_name: "MeetRoomOpened",
        asyncapi_event_kind: "oya.meet.room.opened.v1",
        receipt_event_type: "meet.room.opened",
        proto_package: "oya.meet.v1",
        proto_service: "MeetRoomLifecycle",
        proto_rpc: "OpenRoom",
    })
}

/// Build the room-opened event envelope from an authorized context + receipt,
/// carrying the policy/audit/idempotency provenance.
pub fn meet_room_opened_event_envelope(
    context: &AuthorizedMeetContext,
    receipt: &MeetLifecycleReceipt,
) -> Result<ProtocolEventEnvelope, ProtocolParityError> {
    let binding = meet_room_opened_protocol_binding()?;
    require_receipt_event_type(&binding, receipt.event_type)?;
    ProtocolEventEnvelope::new(
        binding,
        MEET_ROOM_OPENED_PROTOCOL_SCHEMA_VERSION,
        context.tenant_scope_ref.clone(),
        receipt.session_id.clone(),
        receipt.audit_correlation_id.clone(),
        Some(receipt.idempotency_key.clone()),
        receipt.policy_decision_ref.clone(),
    )
}

impl AuthorizedMeetContext {
    /// Default-deny validation gate. Every authz fact must be present and
    /// non-blank; the tenant scope must be tenant-scoped. A usecase calls this
    /// FIRST or it cannot proceed.
    pub fn validate(&self) -> Result<(), MeetApiError> {
        if self.tenant_scope_ref.trim().is_empty() {
            return Err(MeetApiError::MissingTenantScope);
        }
        if !self.tenant_scope_ref.starts_with("tenant:") {
            return Err(MeetApiError::MissingTenantScope);
        }
        if self.principal_ref.trim().is_empty() {
            return Err(MeetApiError::MissingPrincipal);
        }
        if self.idempotency_key.trim().is_empty() {
            return Err(MeetApiError::MissingIdempotencyKey);
        }
        if self.policy_decision_ref.trim().is_empty() {
            return Err(MeetApiError::MissingPolicyDecision);
        }
        if self.audit_correlation_id.trim().is_empty() {
            return Err(MeetApiError::MissingAuditCorrelation);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn authorized() -> AuthorizedMeetContext {
        AuthorizedMeetContext {
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: "user:host@example.com".into(),
            idempotency_key: "idem-1".into(),
            policy_decision_ref: "cedar:allow:meet-open-room".into(),
            audit_correlation_id: "audit-1".into(),
        }
    }

    #[test]
    fn validate_requires_tenant_scope() {
        let mut ctx = authorized();
        ctx.tenant_scope_ref = "person:u".into();
        assert_eq!(ctx.validate(), Err(MeetApiError::MissingTenantScope));

        let mut ctx = authorized();
        ctx.tenant_scope_ref = "".into();
        assert_eq!(ctx.validate(), Err(MeetApiError::MissingTenantScope));
    }

    #[test]
    fn validate_requires_principal_idempotency_policy_and_audit() {
        let mut ctx = authorized();
        ctx.principal_ref = "  ".into();
        assert_eq!(ctx.validate(), Err(MeetApiError::MissingPrincipal));

        let mut ctx = authorized();
        ctx.idempotency_key = "".into();
        assert_eq!(ctx.validate(), Err(MeetApiError::MissingIdempotencyKey));

        let mut ctx = authorized();
        ctx.policy_decision_ref = "".into();
        assert_eq!(ctx.validate(), Err(MeetApiError::MissingPolicyDecision));

        let mut ctx = authorized();
        ctx.audit_correlation_id = "".into();
        assert_eq!(ctx.validate(), Err(MeetApiError::MissingAuditCorrelation));
    }

    #[test]
    fn fully_authorized_context_validates() {
        assert_eq!(authorized().validate(), Ok(()));
    }

    #[test]
    fn room_opened_binding_matches_asyncapi_and_proto_contracts() {
        let binding = meet_room_opened_protocol_binding().unwrap();
        assert_eq!(binding.rest_operation_id, "openRoom");
        assert_eq!(binding.asyncapi_operation_id, "emitMeetRoomOpened");
        assert_eq!(
            binding.asyncapi_channel_address,
            "workflow-events/meet.room.opened"
        );
        assert_eq!(binding.asyncapi_message_name, "MeetRoomOpened");
        assert_eq!(binding.asyncapi_event_kind, "oya.meet.room.opened.v1");
        assert_eq!(binding.receipt_event_type, "meet.room.opened");
        assert_eq!(binding.proto_package, "oya.meet.v1");
        assert_eq!(binding.proto_service, "MeetRoomLifecycle");
        assert_eq!(binding.proto_rpc, "OpenRoom");
    }

    #[test]
    fn room_opened_envelope_carries_policy_audit_and_idempotency_refs() {
        let context = authorized();
        let receipt = MeetLifecycleReceipt {
            session_id: "session:s".into(),
            event_type: "meet.room.opened",
            audit_correlation_id: "audit-1".into(),
            idempotency_key: "idem-1".into(),
            policy_decision_ref: "cedar:allow:meet-open-room".into(),
        };
        let envelope = meet_room_opened_event_envelope(&context, &receipt).unwrap();
        assert_eq!(envelope.schema_version, "1.0.0");
        assert_eq!(envelope.tenant_scope_ref, "tenant:t");
        assert_eq!(envelope.aggregate_id, "session:s");
        assert_eq!(envelope.audit_correlation_id, "audit-1");
        assert_eq!(envelope.idempotency_key, Some("idem-1".into()));
        assert_eq!(envelope.policy_decision_ref, "cedar:allow:meet-open-room");
    }

    #[test]
    fn room_opened_envelope_rejects_mismatched_receipt_event_type() {
        let context = authorized();
        let receipt = MeetLifecycleReceipt {
            session_id: "session:s".into(),
            event_type: "meet.room.closed",
            audit_correlation_id: "audit-1".into(),
            idempotency_key: "idem-1".into(),
            policy_decision_ref: "cedar:allow:meet-open-room".into(),
        };
        assert!(meet_room_opened_event_envelope(&context, &receipt).is_err());
    }
}
