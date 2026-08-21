//! Platform Audit Chain app boundary.
//!
//! This crate owns CloudEvents envelope normalization, producer authorization,
//! request fingerprint idempotency, platform audit-chain append, and eventing
//! outbox publication for `audit.event.emit`.
//!
//! ## Fail-closed authorization (AUTH-005 / C15; ADR-0588)
//!
//! Emitting an audit record is a tamper-evidence-critical mutation. The ONLY
//! public emit path is [`emit_audit_event_authorized`], which requires a
//! verified producer principal ([`authz::VerifiedProducerPrincipal`], unforgeable
//! by construction) and a server-side PDP authorization decision via the
//! [`authz::AuditEmitAuthzProvider`] (default-deny / fail-closed). The
//! caller-supplied [`AuditEventEmitAuthorization`] DTO is NO LONGER an
//! authorization grant — its `decision_id` is demoted to a non-authoritative
//! correlation hint and its other fields are cross-checked against the verified
//! identity, never trusted to authorize. See [`authz`] for the full doctrine.

pub mod authz;

use std::collections::BTreeMap;

use audit_chain_domain::{AuditChain, AuditChainError, AuditEvent, Plane};
use messaging_domain::{EventingError, Outbox, OutboxRecord};
use data_boundary_kernel::{
    DataClassification, Purpose, parse_data_class_label, parse_purpose_pascal_label,
};

use authz::{
    AuditEmitAuthorizationError, AuditEmitAuthzProvider, AuditEmitResource, AuditEmitScope,
    VerifiedProducerPrincipal,
};

pub const AUDIT_EVENT_EMIT_SURFACE: &str = "audit.event.emit";
pub const AUDIT_EVENT_TOPIC: &str = "oya.platform.audit";
pub const AUDIT_EVENT_EMIT_SCHEMA: &str = "audit.event.emit.v1";
pub const AUDIT_EVENT_EMIT_SOURCE: &str = "oyatie://platform/audit-chain";
pub const AUDIT_EVENT_ASYNCAPI_CONTRACT: &str = "contracts/asyncapi/platform/audit-events-v1.yaml";
pub const AUDIT_EVENT_PROTO_CONTRACT: &str =
    "contracts/proto/platform/audit/v1/audit-event-v1.proto";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditEventEmitAppStatus {
    Accepted,
    BadRequest,
    Unauthorized,
    Forbidden,
    UnprocessableEntity,
}

impl AuditEventEmitAppStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Accepted => 202,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEventEmitEnvelopeContext {
    pub event_id: String,               // data_class: INTERNAL_ONLY
    pub source: String,                 // data_class: INTERNAL_ONLY
    pub subject: String,                // data_class: INTERNAL_ONLY
    pub topic: String,                  // data_class: INTERNAL_ONLY
    pub schema: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub producer_id: String,            // data_class: INTERNAL_ONLY
    pub idempotency_key: String,        // data_class: INTERNAL_ONLY
    pub produced_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

/// Caller-supplied authorization correlation fields.
///
/// ## ⚠ NOT an authorization grant (ADR-0588 / AUTH-005 / C15)
///
/// Every field here is CALLER-SUPPLIED and is therefore NON-AUTHORITATIVE. None
/// of `tenant_id`, `producer_id`, `decision_id`, or `allowed_surfaces` grants
/// permission to emit an audit record. Authorization is made server-side by the
/// [`authz::AuditEmitAuthzProvider`] in [`emit_audit_event_authorized`] from a
/// VERIFIED principal + a PDP decision. These fields are retained only for
/// fingerprint/correlation continuity:
/// - `tenant_id` / `producer_id` are CROSS-CHECKED against the verified identity
///   (a mismatch is rejected) but never confer authority on their own.
/// - `decision_id` is a non-authoritative correlation hint for log joins.
/// - `allowed_surfaces` is retained for request-fingerprint stability only; it
///   is NEVER consulted to decide whether the emit is permitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEventEmitAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub producer_id: String,           // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEventEmitPayload {
    pub id: String,                        // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub surface: String,                   // data_class: INTERNAL_ONLY
    pub plane: String,                     // data_class: INTERNAL_ONLY
    pub purpose: String,                   // data_class: INTERNAL_ONLY
    pub data_classes_touched: Vec<String>, // data_class: INTERNAL_ONLY
    pub decision: String,                  // data_class: INTERNAL_ONLY
    pub idempotency_key: String,           // data_class: INTERNAL_ONLY
    pub emitted_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEventEmitAppRequest {
    pub envelope: AuditEventEmitEnvelopeContext, // data_class: INTERNAL_ONLY
    pub authorization: AuditEventEmitAuthorization, // data_class: INTERNAL_ONLY
    pub payload: AuditEventEmitPayload,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AuditEventEmitIdempotencyLedger {
    entries: BTreeMap<AuditEventEmitIdempotencyLedgerKey, AuditEventEmitIdempotencyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl AuditEventEmitIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct AuditEventEmitIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    producer_id: String,     // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AuditEventEmitIdempotencyLedgerEntry {
    fingerprint: AuditEventEmitRequestFingerprint, // data_class: INTERNAL_ONLY
    result: AuditEventEmitAppResult,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AuditEventEmitRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type AuditEventEmitAppResult = Result<AuditEventEmitSuccessResponse, AuditEventEmitAppError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEventEmitSuccessResponse {
    pub data: AuditEventEmitRecord,       // data_class: INTERNAL_ONLY
    pub metadata: AuditEventEmitMetadata, // data_class: INTERNAL_ONLY
}

impl AuditEventEmitSuccessResponse {
    pub fn accepted(data: AuditEventEmitRecord, envelope: &AuditEventEmitEnvelopeContext) -> Self {
        Self {
            data,
            metadata: AuditEventEmitMetadata {
                event_id: envelope.event_id.clone(),
                tenant_id: envelope.tenant_id.clone(),
                producer_id: envelope.producer_id.clone(),
                topic: envelope.topic.clone(),
                schema: envelope.schema.clone(),
                produced_at_epoch_seconds: envelope.produced_at_epoch_seconds,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEventEmitMetadata {
    pub event_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub producer_id: String,            // data_class: INTERNAL_ONLY
    pub topic: String,                  // data_class: INTERNAL_ONLY
    pub schema: String,                 // data_class: INTERNAL_ONLY
    pub produced_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEventEmitRecord {
    pub sequence: u64,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub surface: String,                // data_class: INTERNAL_ONLY
    pub plane: String,                  // data_class: INTERNAL_ONLY
    pub purpose: String,                // data_class: INTERNAL_ONLY
    pub data_classes: Vec<String>,      // data_class: INTERNAL_ONLY
    pub decision: String,               // data_class: INTERNAL_ONLY
    pub previous_hash: String,          // data_class: INTERNAL_ONLY
    pub hash: String,                   // data_class: INTERNAL_ONLY
    pub audit_schema_version: u32,      // data_class: PUBLIC
    pub outbox_sequence: u64,           // data_class: INTERNAL_ONLY
    pub outbox_topic: String,           // data_class: INTERNAL_ONLY
    pub outbox_idempotency_key: String, // data_class: INTERNAL_ONLY
    pub outbox_payload_ref: String,     // data_class: INTERNAL_ONLY
    pub outbox_published: bool,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditEventEmitAppError {
    EmptyEventId,
    EmptySource,
    InvalidSource {
        source: String,
    },
    EmptySubject,
    SubjectMismatch {
        expected_subject: String,
        actual_subject: String,
    },
    InvalidTopic {
        topic: String,
    },
    InvalidSchema {
        schema: String,
    },
    EmptyTenantId,
    EmptyProducerId,
    EmptyIdempotencyKey,
    InvalidProducedAt,
    EmptyAuthorizationDecisionId,
    AuthorizationTenantMismatch {
        authorization_tenant_id: String,
        envelope_tenant_id: String,
    },
    AuthorizationProducerMismatch {
        authorization_producer_id: String,
        envelope_producer_id: String,
    },
    AuthorizationDenied {
        surface: String,
    },
    /// ADR-0588: the caller did not present a verifiable producer credential.
    /// Maps to 401 (Unauthorized) — fail-closed: the audit record is never
    /// emitted without a verified principal.
    PrincipalUnverified,
    /// ADR-0588: the verified principal's identity does not match the
    /// envelope/payload's self-attested tenant or producer (substitution
    /// attempt). Maps to 403 (Forbidden).
    VerifiedPrincipalMismatch {
        verified_id: String,
        claimed_id: String,
    },
    /// ADR-0588: the server-side PDP denied (or refused, fail-closed) the verified
    /// principal's request to emit for the target resource. Maps to 403
    /// (Forbidden).
    PdpAuthorizationDenied,
    EnvelopePayloadEventIdMismatch {
        envelope_event_id: String,
        payload_event_id: String,
    },
    EnvelopePayloadTenantMismatch {
        envelope_tenant_id: String,
        payload_tenant_id: String,
    },
    EnvelopePayloadIdempotencyMismatch {
        envelope_idempotency_key: String,
        payload_idempotency_key: String,
    },
    EmptySurface,
    InvalidPlane {
        plane: String,
    },
    InvalidPurpose {
        purpose: String,
    },
    MissingDataClassesTouched,
    InvalidDataClassLabel {
        data_class: String,
    },
    EmptyDecision,
    InvalidEmittedAt,
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    Eventing(EventingError),
    /// ADR-0083 amendment 2026-05-15: `append_classifications` is now Tier 1
    /// fallible and propagates `AuditChainError` variants (`EmptyTenantId`,
    /// `TenantShardMismatch`, etc.) through this app boundary.
    AuditChain(AuditChainError),
}

impl From<AuditChainError> for AuditEventEmitAppError {
    fn from(error: AuditChainError) -> Self {
        Self::AuditChain(error)
    }
}

impl AuditEventEmitAppError {
    pub fn audit_event_emit_status(&self) -> AuditEventEmitAppStatus {
        match self {
            Self::EmptyProducerId | Self::PrincipalUnverified => {
                AuditEventEmitAppStatus::Unauthorized
            }
            Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationProducerMismatch { .. }
            | Self::AuthorizationDenied { .. }
            | Self::VerifiedPrincipalMismatch { .. }
            | Self::PdpAuthorizationDenied
            | Self::EnvelopePayloadTenantMismatch { .. } => AuditEventEmitAppStatus::Forbidden,
            Self::IdempotencyKeyReused { .. }
            | Self::Eventing(EventingError::IdempotencyReplayMismatch)
            | Self::Eventing(EventingError::DuplicateTopic)
            | Self::Eventing(EventingError::TopicNotFound)
            | Self::Eventing(EventingError::OutboxRecordNotFound)
            | Self::Eventing(EventingError::InvalidOutboxHistory) => {
                AuditEventEmitAppStatus::UnprocessableEntity
            }
            Self::EmptyEventId
            | Self::EmptySource
            | Self::InvalidSource { .. }
            | Self::EmptySubject
            | Self::SubjectMismatch { .. }
            | Self::InvalidTopic { .. }
            | Self::InvalidSchema { .. }
            | Self::EmptyTenantId
            | Self::EmptyIdempotencyKey
            | Self::InvalidProducedAt
            | Self::EmptyAuthorizationDecisionId
            | Self::EnvelopePayloadEventIdMismatch { .. }
            | Self::EnvelopePayloadIdempotencyMismatch { .. }
            | Self::EmptySurface
            | Self::InvalidPlane { .. }
            | Self::InvalidPurpose { .. }
            | Self::MissingDataClassesTouched
            | Self::InvalidDataClassLabel { .. }
            | Self::EmptyDecision
            | Self::InvalidEmittedAt
            | Self::Eventing(EventingError::EmptyTopic)
            | Self::Eventing(EventingError::EmptyTopicAxis)
            | Self::Eventing(EventingError::EmptyTopicDescription)
            | Self::Eventing(EventingError::InvalidTopicName)
            | Self::Eventing(EventingError::EmptyIdempotencyKey)
            | Self::Eventing(EventingError::EmptyPayloadRef)
            | Self::AuditChain(_) => AuditEventEmitAppStatus::BadRequest,
        }
    }

    pub fn audit_event_emit_status_code(&self) -> u16 {
        self.audit_event_emit_status().code()
    }
}

pub fn validate_audit_event_emit_request(
    request: &AuditEventEmitAppRequest,
) -> Result<(), AuditEventEmitAppError> {
    validate_envelope(&request.envelope)?;
    validate_authorization(&request.envelope, &request.authorization)?;
    validate_payload_labels(&request.payload)?;
    validate_envelope_payload_binding(&request.envelope, &request.payload)
}

/// FAIL-CLOSED audit-event emit boundary (ADR-0588 / AUTH-005 / C15).
///
/// This is the ONLY public path that appends to the platform audit chain. It is
/// impossible to emit an audit record without:
///
/// 1. A **verified producer principal** — `verified` is an
///    [`authz::VerifiedProducerPrincipal`], an UNFORGEABLE token (private fields,
///    `pub(crate)` constructor) that external crates can only obtain by running a
///    real [`authz::PrincipalVerifier`]. The composition root verifies the
///    caller credential (bearer / mTLS-SVID) via
///    [`authz::AuditEmitAuthzProvider::verify_principal`] and passes the result
///    here. There is no code path to this function that skips verification.
/// 2. An **active cross-check** of the verified identity against the request's
///    self-attested envelope/payload tenant + producer. A mismatch means the
///    caller fabricated a different identity in the body than the verifier bound
///    — rejected as [`AuditEventEmitAppError::VerifiedPrincipalMismatch`] (403).
/// 3. A **server-side PDP decision** — `authz.ensure_authorized(verified,
///    resource)` for `action = audit.event.emit` on the TARGET
///    `{tenant, surface, scope}`. The resource tenant is derived from the
///    validated payload's TARGET tenant (a trusted source), NOT flattened to the
///    caller's verified tenant, so a cross-tenant emit is deniable at the PDP
///    (no IDOR). Default-deny: any deny or PDP fault maps to
///    [`AuditEventEmitAppError::PdpAuthorizationDenied`] (403).
///
/// The caller-supplied [`AuditEventEmitAuthorization`] fields NEVER authorize;
/// the audit record reflects the VERIFIED principal. The
/// [`AuditEmitAuthzProvider`] is a REQUIRED, non-optional argument — there is no
/// `Default` and no allow-all provider, so a boundary that cannot prove a
/// credential root and reach a PDP can never emit.
///
/// # Errors
/// - [`AuditEventEmitAppError::PrincipalUnverified`] — no/invalid credential (401).
/// - [`AuditEventEmitAppError::VerifiedPrincipalMismatch`] — identity substitution (403).
/// - [`AuditEventEmitAppError::PdpAuthorizationDenied`] — PDP deny / fault (403).
/// - the validation and emission errors from the inner append path.
pub fn emit_audit_event_authorized(
    authz: &AuditEmitAuthzProvider,
    verified: &VerifiedProducerPrincipal,
    chain: &mut AuditChain,
    outbox: &mut Outbox,
    idempotency_ledger: &mut AuditEventEmitIdempotencyLedger,
    request: AuditEventEmitAppRequest,
) -> Result<AuditEventEmitSuccessResponse, AuditEventEmitAppError> {
    // (1) Structural validation FIRST so malformed input is a 4xx before any
    //     authz work (and so the cross-check below sees normalized fields).
    validate_audit_event_emit_request(&request)?;

    // (2) Cross-check the verified identity against the request's self-attested
    //     envelope/payload tenant + producer. The verified identity is
    //     AUTHORITATIVE; a non-matching self-attested field is a substitution
    //     attempt → 403. (Empty fields are already rejected by
    //     validate_envelope, so these are non-empty here.)
    if request.envelope.producer_id != verified.producer_id() {
        return Err(AuditEventEmitAppError::VerifiedPrincipalMismatch {
            verified_id: verified.producer_id().to_string(),
            claimed_id: request.envelope.producer_id.clone(),
        });
    }
    if request.envelope.tenant_id != verified.tenant_id() {
        return Err(AuditEventEmitAppError::VerifiedPrincipalMismatch {
            verified_id: verified.tenant_id().to_string(),
            claimed_id: request.envelope.tenant_id.clone(),
        });
    }

    // (3) Server-side PDP authorization on the TARGET resource. The resource
    //     tenant + surface come from the validated payload (the TARGET), not the
    //     caller's verified tenant — so a cross-tenant emit (verified producer of
    //     tenant A recording for tenant B) is deniable AT THE PDP (no IDOR). The
    //     scope is explicit so the PDP sees the true blast radius: a
    //     platform-level audit record requires platform-audit authority, not mere
    //     tenant-producer authority.
    let resource = audit_emit_resource(&request.payload);
    authz
        .ensure_authorized(verified, &resource)
        .map_err(|err| match err {
            // Both an explicit deny and a fail-closed refusal map to 403; the
            // record is never emitted. (No catch_unwind: panic = "abort" defeats
            // it; the adapter contract forbids panics — see authz module docs.)
            AuditEmitAuthorizationError::Denied | AuditEmitAuthorizationError::Refused => {
                AuditEventEmitAppError::PdpAuthorizationDenied
            }
        })?;

    // (4) Gate passed — append. The append uses the VERIFIED principal's tenant
    //     for the record's tenant attribution (already cross-checked equal to the
    //     envelope/payload tenant above), so the recorded tenant is authoritative.
    emit_audit_event_unauthorized_inner(chain, outbox, idempotency_ledger, request)
}

/// Derive the PDP resource from the validated TARGET payload. The tenant + surface
/// are the event's TARGET (trusted, post-validation), never the caller's verified
/// tenant. An empty payload tenant (a platform-lineage event) is presented as
/// [`AuditEmitScope::Platform`] so the PDP requires platform-audit authority and a
/// tenant producer cannot forge a platform-level record.
fn audit_emit_resource(payload: &AuditEventEmitPayload) -> AuditEmitResource {
    if payload.tenant_id.trim().is_empty() {
        AuditEmitResource {
            tenant_id: String::new(),
            surface: payload.surface.clone(),
            scope: AuditEmitScope::Platform,
        }
    } else {
        AuditEmitResource {
            tenant_id: payload.tenant_id.clone(),
            surface: payload.surface.clone(),
            scope: AuditEmitScope::Tenant,
        }
    }
}

/// RAW audit-chain append — runs AFTER the fail-closed authz gate in
/// [`emit_audit_event_authorized`]. **`pub(crate)` only**: external crates cannot
/// reach the mutation without passing through the verified-principal + PDP gate.
/// The self-attested [`AuditEventEmitAuthorization`] is cross-checked for internal
/// consistency only (`validate_authorization`); it confers NO authority.
pub(crate) fn emit_audit_event_unauthorized_inner(
    chain: &mut AuditChain,
    outbox: &mut Outbox,
    idempotency_ledger: &mut AuditEventEmitIdempotencyLedger,
    request: AuditEventEmitAppRequest,
) -> Result<AuditEventEmitSuccessResponse, AuditEventEmitAppError> {
    validate_audit_event_emit_request(&request)?;
    let key = idempotency_key_for(&request.envelope, AUDIT_EVENT_EMIT_SURFACE);
    let fingerprint = audit_event_emit_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(AuditEventEmitAppError::IdempotencyKeyReused {
            idempotency_key: request.envelope.idempotency_key,
        });
    }

    let envelope = request.envelope.clone();
    let event_id = request.payload.id.clone();
    let result = audit_event_input(request.payload).and_then(|input| {
        let event = chain
            .append_classifications(
                input.tenant_id,
                input.surface,
                input.plane,
                input.purpose,
                input.data_classifications,
                input.decision,
            )
            .map_err(AuditEventEmitAppError::from)?
            .clone();
        let outbox_record = publish_outbox(outbox, &event, &event_id)?;
        Ok(AuditEventEmitSuccessResponse::accepted(
            audit_event_record(event, outbox_record),
            &envelope,
        ))
    });

    idempotency_ledger.entries.insert(
        key,
        AuditEventEmitIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AuditEventInput {
    tenant_id: String,                             // data_class: INTERNAL_ONLY
    surface: String,                               // data_class: INTERNAL_ONLY
    plane: Plane,                                  // data_class: INTERNAL_ONLY
    purpose: Purpose,                              // data_class: INTERNAL_ONLY
    data_classifications: Vec<DataClassification>, // data_class: INTERNAL_ONLY
    decision: String,                              // data_class: INTERNAL_ONLY
}

fn validate_envelope(
    envelope: &AuditEventEmitEnvelopeContext,
) -> Result<(), AuditEventEmitAppError> {
    if envelope.event_id.trim().is_empty() {
        return Err(AuditEventEmitAppError::EmptyEventId);
    }
    if envelope.source.trim().is_empty() {
        return Err(AuditEventEmitAppError::EmptySource);
    }
    if envelope.source != AUDIT_EVENT_EMIT_SOURCE {
        return Err(AuditEventEmitAppError::InvalidSource {
            source: envelope.source.clone(),
        });
    }
    if envelope.subject.trim().is_empty() {
        return Err(AuditEventEmitAppError::EmptySubject);
    }
    if envelope.topic != AUDIT_EVENT_TOPIC {
        return Err(AuditEventEmitAppError::InvalidTopic {
            topic: envelope.topic.clone(),
        });
    }
    if envelope.schema != AUDIT_EVENT_EMIT_SCHEMA {
        return Err(AuditEventEmitAppError::InvalidSchema {
            schema: envelope.schema.clone(),
        });
    }
    if envelope.tenant_id.trim().is_empty() {
        return Err(AuditEventEmitAppError::EmptyTenantId);
    }
    if envelope.producer_id.trim().is_empty() {
        return Err(AuditEventEmitAppError::EmptyProducerId);
    }
    if envelope.idempotency_key.trim().is_empty() {
        return Err(AuditEventEmitAppError::EmptyIdempotencyKey);
    }
    if envelope.produced_at_epoch_seconds == 0 {
        return Err(AuditEventEmitAppError::InvalidProducedAt);
    }
    Ok(())
}

fn validate_authorization(
    envelope: &AuditEventEmitEnvelopeContext,
    authorization: &AuditEventEmitAuthorization,
) -> Result<(), AuditEventEmitAppError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(AuditEventEmitAppError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != envelope.tenant_id {
        return Err(AuditEventEmitAppError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            envelope_tenant_id: envelope.tenant_id.clone(),
        });
    }
    if authorization.producer_id != envelope.producer_id {
        return Err(AuditEventEmitAppError::AuthorizationProducerMismatch {
            authorization_producer_id: authorization.producer_id.clone(),
            envelope_producer_id: envelope.producer_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|surface| surface == AUDIT_EVENT_EMIT_SURFACE)
    {
        return Err(AuditEventEmitAppError::AuthorizationDenied {
            surface: AUDIT_EVENT_EMIT_SURFACE.to_string(),
        });
    }
    Ok(())
}

fn validate_payload_labels(payload: &AuditEventEmitPayload) -> Result<(), AuditEventEmitAppError> {
    if payload.surface.trim().is_empty() {
        return Err(AuditEventEmitAppError::EmptySurface);
    }
    parse_plane(payload.plane.clone())?;
    parse_purpose(payload.purpose.clone())?;
    parse_data_classifications(&payload.data_classes_touched)?;
    if payload.decision.trim().is_empty() {
        return Err(AuditEventEmitAppError::EmptyDecision);
    }
    if payload.emitted_at_epoch_seconds == 0 {
        return Err(AuditEventEmitAppError::InvalidEmittedAt);
    }
    Ok(())
}

fn validate_envelope_payload_binding(
    envelope: &AuditEventEmitEnvelopeContext,
    payload: &AuditEventEmitPayload,
) -> Result<(), AuditEventEmitAppError> {
    if envelope.event_id != payload.id {
        return Err(AuditEventEmitAppError::EnvelopePayloadEventIdMismatch {
            envelope_event_id: envelope.event_id.clone(),
            payload_event_id: payload.id.clone(),
        });
    }
    if envelope.tenant_id != payload.tenant_id {
        return Err(AuditEventEmitAppError::EnvelopePayloadTenantMismatch {
            envelope_tenant_id: envelope.tenant_id.clone(),
            payload_tenant_id: payload.tenant_id.clone(),
        });
    }
    if envelope.idempotency_key != payload.idempotency_key {
        return Err(AuditEventEmitAppError::EnvelopePayloadIdempotencyMismatch {
            envelope_idempotency_key: envelope.idempotency_key.clone(),
            payload_idempotency_key: payload.idempotency_key.clone(),
        });
    }
    let expected_subject = expected_subject(payload);
    if envelope.subject != expected_subject {
        return Err(AuditEventEmitAppError::SubjectMismatch {
            expected_subject,
            actual_subject: envelope.subject.clone(),
        });
    }
    Ok(())
}

fn audit_event_input(
    payload: AuditEventEmitPayload,
) -> Result<AuditEventInput, AuditEventEmitAppError> {
    Ok(AuditEventInput {
        tenant_id: payload.tenant_id,
        surface: payload.surface,
        plane: parse_plane(payload.plane)?,
        purpose: parse_purpose(payload.purpose)?,
        data_classifications: parse_data_classifications(&payload.data_classes_touched)?,
        decision: payload.decision,
    })
}

fn parse_plane(label: String) -> Result<Plane, AuditEventEmitAppError> {
    match label.as_str() {
        "control" => Ok(Plane::Control),
        "data" => Ok(Plane::Data),
        "analytics" => Ok(Plane::Analytics),
        "audit" => Ok(Plane::Audit),
        _ => Err(AuditEventEmitAppError::InvalidPlane { plane: label }),
    }
}

fn parse_purpose(label: String) -> Result<Purpose, AuditEventEmitAppError> {
    parse_purpose_pascal_label(&label)
        .ok_or(AuditEventEmitAppError::InvalidPurpose { purpose: label })
}

fn parse_data_classifications(
    labels: &[String],
) -> Result<Vec<DataClassification>, AuditEventEmitAppError> {
    if labels.is_empty() {
        return Err(AuditEventEmitAppError::MissingDataClassesTouched);
    }
    labels
        .iter()
        .map(|label| {
            parse_data_class_label(label)
                .map(DataClassification::from)
                .ok_or_else(|| AuditEventEmitAppError::InvalidDataClassLabel {
                    data_class: label.clone(),
                })
        })
        .collect()
}

fn publish_outbox(
    outbox: &mut Outbox,
    event: &AuditEvent,
    event_id: &str,
) -> Result<OutboxRecord, AuditEventEmitAppError> {
    outbox
        .publish(
            event.tenant_id.clone(),
            AUDIT_EVENT_TOPIC.to_string(),
            event_id.to_string(),
            format!("audit-events/{event_id}"),
        )
        .map_err(AuditEventEmitAppError::Eventing)
}

fn idempotency_key_for(
    envelope: &AuditEventEmitEnvelopeContext,
    surface: &str,
) -> AuditEventEmitIdempotencyLedgerKey {
    AuditEventEmitIdempotencyLedgerKey {
        tenant_id: envelope.tenant_id.clone(),
        producer_id: envelope.producer_id.clone(),
        surface: surface.to_string(),
        idempotency_key: envelope.idempotency_key.clone(),
    }
}

fn audit_event_emit_fingerprint_for(
    request: &AuditEventEmitAppRequest,
) -> AuditEventEmitRequestFingerprint {
    AuditEventEmitRequestFingerprint {
        canonical: [
            format!("event.id={}", request.envelope.event_id),
            format!("event.source={}", request.envelope.source),
            format!("event.subject={}", request.envelope.subject),
            format!("event.topic={}", request.envelope.topic),
            format!("event.schema={}", request.envelope.schema),
            format!("event.tenant_id={}", request.envelope.tenant_id),
            format!("event.producer_id={}", request.envelope.producer_id),
            format!("event.idempotency_key={}", request.envelope.idempotency_key),
            format!(
                "event.produced_at_epoch_seconds={}",
                request.envelope.produced_at_epoch_seconds
            ),
            format!(
                "authorization.tenant_id={}",
                request.authorization.tenant_id
            ),
            format!(
                "authorization.producer_id={}",
                request.authorization.producer_id
            ),
            format!(
                "authorization.decision_id={}",
                request.authorization.decision_id
            ),
            format!(
                "authorization.allowed_surfaces={}",
                request.authorization.allowed_surfaces.join(",")
            ),
            format!("payload.id={}", request.payload.id),
            format!("payload.tenant_id={}", request.payload.tenant_id),
            format!("payload.surface={}", request.payload.surface),
            format!("payload.plane={}", request.payload.plane),
            format!("payload.purpose={}", request.payload.purpose),
            format!(
                "payload.data_classes_touched={}",
                request.payload.data_classes_touched.join(",")
            ),
            format!("payload.decision={}", request.payload.decision),
            format!(
                "payload.idempotency_key={}",
                request.payload.idempotency_key
            ),
            format!(
                "payload.emitted_at_epoch_seconds={}",
                request.payload.emitted_at_epoch_seconds
            ),
        ]
        .join("|"),
    }
}

fn audit_event_record(event: AuditEvent, outbox_record: OutboxRecord) -> AuditEventEmitRecord {
    AuditEventEmitRecord {
        sequence: event.sequence,
        tenant_id: event.tenant_id,
        surface: event.surface,
        plane: plane_label(event.plane).to_string(),
        purpose: event.purpose.pascal_label().to_string(),
        data_classes: event
            .data_classes
            .iter()
            .map(|data_class| data_class.label().to_string())
            .collect(),
        decision: event.decision,
        previous_hash: event.previous_hash,
        hash: event.hash,
        audit_schema_version: 1,
        outbox_sequence: outbox_record.sequence,
        outbox_topic: outbox_record.topic.value,
        outbox_idempotency_key: outbox_record.idempotency_key.value,
        outbox_payload_ref: outbox_record.payload_ref.value,
        outbox_published: outbox_record.published,
    }
}

fn expected_subject(payload: &AuditEventEmitPayload) -> String {
    format!("tenant/{}/surface/{}", payload.tenant_id, payload.surface)
}

fn plane_label(plane: Plane) -> &'static str {
    match plane {
        Plane::Control => "control",
        Plane::Data => "data",
        Plane::Audit => "audit",
        Plane::Analytics => "analytics",
    }
}
