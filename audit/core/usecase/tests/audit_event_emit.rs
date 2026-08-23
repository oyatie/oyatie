// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use audit_chain_domain::AuditChain;
use audit_usecase::authz::{
    AuditEmitAuthorizationError, AuditEmitAuthorizer, AuditEmitAuthzProvider, AuditEmitResource,
    CallerCredential, ConfiguredBearerPrincipalVerifier, VerifiedProducerPrincipal,
};
use audit_usecase::{
    AUDIT_EVENT_ASYNCAPI_CONTRACT, AUDIT_EVENT_EMIT_SCHEMA, AUDIT_EVENT_EMIT_SOURCE,
    AUDIT_EVENT_EMIT_SURFACE, AUDIT_EVENT_PROTO_CONTRACT, AUDIT_EVENT_TOPIC,
    AuditEventEmitAppError, AuditEventEmitAppRequest, AuditEventEmitAppStatus,
    AuditEventEmitAuthorization, AuditEventEmitEnvelopeContext, AuditEventEmitIdempotencyLedger,
    AuditEventEmitPayload, emit_audit_event_authorized,
};
use messaging_domain::Outbox;

const EVENT_ID: &str = "audit_evt_cloud_vm_001";
const IDEMPOTENCY_KEY: &str = "idem_audit_emit_cloud_vm_001";
const DECISION: &str = "ALLOW";
const BEARER_SECRET: &str = "audit-emit-break-glass-secret";
const PRODUCER_ID: &str = "producer_cloud_compute";
const TENANT_ID: &str = "ten_alpha";

/// A permissive PDP test double that authorizes everything. Used by the
/// pre-existing behavioural suite so it exercises the append path with the gate
/// open; the dedicated authz suite uses deny/allow doubles to prove the seam.
struct AllowAllAuthorizer;
impl AuditEmitAuthorizer for AllowAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedProducerPrincipal,
        _resource: &AuditEmitResource,
    ) -> Result<(), AuditEmitAuthorizationError> {
        Ok(())
    }
}

/// Build an authz provider whose verifier accepts the break-glass bearer for
/// `(PRODUCER_ID, TENANT_ID)` and whose PDP authorizes everything.
fn allow_all_provider() -> AuditEmitAuthzProvider {
    let verifier = Arc::new(
        ConfiguredBearerPrincipalVerifier::new(BEARER_SECRET, PRODUCER_ID, TENANT_ID)
            .expect("non-empty break-glass credential"),
    );
    AuditEmitAuthzProvider::new(verifier, Arc::new(AllowAllAuthorizer))
}

/// Verify the break-glass bearer into the authoritative producer principal.
fn verified_break_glass(provider: &AuditEmitAuthzProvider) -> VerifiedProducerPrincipal {
    provider
        .verify_principal(&CallerCredential {
            authorization: Some(format!("Bearer {BEARER_SECRET}")),
            claimed_producer_id: PRODUCER_ID.to_string(),
            claimed_tenant_id: TENANT_ID.to_string(),
        })
        .expect("valid break-glass bearer verifies")
}

#[test]
fn audit_event_contract_runtime_constants_are_covered() {
    assert_eq!(AUDIT_EVENT_EMIT_SURFACE, "audit.event.emit");
    assert_eq!(AUDIT_EVENT_TOPIC, "oyatie.platform.audit");
    assert_eq!(AUDIT_EVENT_EMIT_SCHEMA, "audit.event.emit.v1");
    assert_eq!(AUDIT_EVENT_EMIT_SOURCE, "oyatie://platform/audit-chain");
    assert_eq!(
        AUDIT_EVENT_ASYNCAPI_CONTRACT,
        "contracts/asyncapi/platform/audit-events-v1.yaml"
    );
    assert_eq!(
        AUDIT_EVENT_PROTO_CONTRACT,
        "contracts/proto/platform/audit/v1/audit-event-v1.proto"
    );
    assert_eq!(AuditEventEmitAppStatus::Accepted.code(), 202);
    assert_eq!(AuditEventEmitAppStatus::BadRequest.code(), 400);
    assert_eq!(AuditEventEmitAppStatus::Unauthorized.code(), 401);
    assert_eq!(AuditEventEmitAppStatus::Forbidden.code(), 403);
    assert_eq!(AuditEventEmitAppStatus::UnprocessableEntity.code(), 422);
}

#[test]
fn audit_event_emit_appends_chain_and_outbox_once_and_replays_idempotently() {
    let mut chain = AuditChain::default();
    let mut outbox = Outbox::default();
    let mut idempotency = AuditEventEmitIdempotencyLedger::default();
    let provider = allow_all_provider();
    let verified = verified_break_glass(&provider);
    let request = emit_request(EVENT_ID, IDEMPOTENCY_KEY);

    let first = emit_audit_event_authorized(
        &provider,
        &verified,
        &mut chain,
        &mut outbox,
        &mut idempotency,
        request.clone(),
    )
    .expect("first emit appends audit chain and outbox");
    let second = emit_audit_event_authorized(
        &provider,
        &verified,
        &mut chain,
        &mut outbox,
        &mut idempotency,
        request,
    )
    .expect("same request fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(idempotency.len(), 1);
    assert_eq!(chain.events().len(), 1);
    assert_eq!(outbox.records().len(), 1);
    assert!(chain.verify());
    assert_eq!(first.data.sequence, 0);
    assert_eq!(first.data.tenant_id, "ten_alpha");
    assert_eq!(first.data.surface, "cloud.compute.vm.create");
    assert_eq!(first.data.plane, "control");
    assert_eq!(first.data.purpose, "CoreService");
    assert_eq!(first.data.data_classes, vec!["INTERNAL_ONLY", "PUBLIC"]);
    assert_eq!(first.data.decision, DECISION);
    assert_eq!(first.data.previous_hash, "GENESIS");
    assert!(first.data.hash.starts_with("sha256:"));
    assert_eq!(first.data.outbox_topic, AUDIT_EVENT_TOPIC);
    assert_eq!(
        first.data.outbox_payload_ref,
        "audit-events/audit_evt_cloud_vm_001"
    );
    assert!(!first.data.outbox_published);
    assert_eq!(first.metadata.event_id, EVENT_ID);
    assert_eq!(first.metadata.schema, AUDIT_EVENT_EMIT_SCHEMA);
}

#[test]
fn audit_event_emit_rejects_envelope_payload_drift_before_mutation() {
    let mut chain = AuditChain::default();
    let mut outbox = Outbox::default();
    let mut idempotency = AuditEventEmitIdempotencyLedger::default();
    let provider = allow_all_provider();
    let verified = verified_break_glass(&provider);
    let mut tenant_drift = emit_request(EVENT_ID, "idem_audit_emit_drift_tenant");
    tenant_drift.payload.tenant_id = "ten_other".to_string();

    let tenant_error = emit_audit_event_authorized(
        &provider,
        &verified,
        &mut chain,
        &mut outbox,
        &mut idempotency,
        tenant_drift,
    )
    .expect_err("tenant drift is rejected before mutation");
    assert!(matches!(
        tenant_error,
        AuditEventEmitAppError::EnvelopePayloadTenantMismatch { .. }
    ));
    assert_eq!(
        tenant_error.audit_event_emit_status(),
        AuditEventEmitAppStatus::Forbidden
    );

    let mut idempotency_drift = emit_request(EVENT_ID, "idem_audit_emit_drift_idempotency");
    idempotency_drift.payload.idempotency_key = "idem_payload_other".to_string();
    let idempotency_error = emit_audit_event_authorized(
        &provider,
        &verified,
        &mut chain,
        &mut outbox,
        &mut idempotency,
        idempotency_drift,
    )
    .expect_err("idempotency drift is rejected before mutation");
    assert!(matches!(
        idempotency_error,
        AuditEventEmitAppError::EnvelopePayloadIdempotencyMismatch { .. }
    ));
    assert_eq!(idempotency_error.audit_event_emit_status_code(), 400);

    assert!(idempotency.is_empty());
    assert!(chain.events().is_empty());
    assert!(outbox.records().is_empty());
}

#[test]
fn audit_event_emit_separates_missing_producer_from_denied_authorization() {
    let mut chain = AuditChain::default();
    let mut outbox = Outbox::default();
    let mut idempotency = AuditEventEmitIdempotencyLedger::default();
    let provider = allow_all_provider();
    let verified = verified_break_glass(&provider);
    let mut unauthenticated = emit_request(EVENT_ID, "idem_audit_emit_authn");
    unauthenticated.envelope.producer_id.clear();

    let authn_error = emit_audit_event_authorized(
        &provider,
        &verified,
        &mut chain,
        &mut outbox,
        &mut idempotency,
        unauthenticated,
    )
    .expect_err("missing producer is authentication failure");
    assert_eq!(
        authn_error.audit_event_emit_status(),
        AuditEventEmitAppStatus::Unauthorized
    );

    let mut denied = emit_request(EVENT_ID, "idem_audit_emit_authz");
    denied.authorization.allowed_surfaces = vec!["metering.event.ingest".to_string()];
    let authz_error = emit_audit_event_authorized(
        &provider,
        &verified,
        &mut chain,
        &mut outbox,
        &mut idempotency,
        denied,
    )
    .expect_err("missing surface grant is authorization failure");
    assert!(matches!(
        authz_error,
        AuditEventEmitAppError::AuthorizationDenied { ref surface }
            if surface == AUDIT_EVENT_EMIT_SURFACE
    ));
    assert_eq!(
        authz_error.audit_event_emit_status(),
        AuditEventEmitAppStatus::Forbidden
    );
    assert!(idempotency.is_empty());
    assert!(chain.events().is_empty());
    assert!(outbox.records().is_empty());
}

#[test]
fn audit_event_emit_rejects_invalid_labels_before_mutation() {
    let mut chain = AuditChain::default();
    let mut outbox = Outbox::default();
    let mut idempotency = AuditEventEmitIdempotencyLedger::default();
    let provider = allow_all_provider();
    let verified = verified_break_glass(&provider);

    let mut invalid_plane = emit_request(EVENT_ID, "idem_audit_emit_bad_plane");
    invalid_plane.payload.plane = "compute".to_string();
    assert!(matches!(
        emit_audit_event_authorized(
            &provider,
            &verified,
            &mut chain,
            &mut outbox,
            &mut idempotency,
            invalid_plane
        ),
        Err(AuditEventEmitAppError::InvalidPlane { .. })
    ));

    let mut invalid_purpose = emit_request(EVENT_ID, "idem_audit_emit_bad_purpose");
    invalid_purpose.payload.purpose = "Banana".to_string();
    assert!(matches!(
        emit_audit_event_authorized(
            &provider,
            &verified,
            &mut chain,
            &mut outbox,
            &mut idempotency,
            invalid_purpose
        ),
        Err(AuditEventEmitAppError::InvalidPurpose { .. })
    ));

    let mut invalid_class = emit_request(EVENT_ID, "idem_audit_emit_bad_class");
    invalid_class.payload.data_classes_touched = vec!["SECRET".to_string()];
    let class_error = emit_audit_event_authorized(
        &provider,
        &verified,
        &mut chain,
        &mut outbox,
        &mut idempotency,
        invalid_class,
    )
    .expect_err("audit payload accepts privacy-program data classes, not operational labels");
    assert!(matches!(
        class_error,
        AuditEventEmitAppError::InvalidDataClassLabel { .. }
    ));
    assert_eq!(class_error.audit_event_emit_status_code(), 400);

    assert!(idempotency.is_empty());
    assert!(chain.events().is_empty());
    assert!(outbox.records().is_empty());
}

#[test]
fn audit_event_emit_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut chain = AuditChain::default();
    let mut outbox = Outbox::default();
    let mut idempotency = AuditEventEmitIdempotencyLedger::default();
    let provider = allow_all_provider();
    let verified = verified_break_glass(&provider);
    let mut request = emit_request(EVENT_ID, "idem_audit_emit_reused");

    emit_audit_event_authorized(
        &provider,
        &verified,
        &mut chain,
        &mut outbox,
        &mut idempotency,
        request.clone(),
    )
    .expect("first idempotent request succeeds");

    request.payload.decision = "DENY".to_string();
    let error = emit_audit_event_authorized(
        &provider,
        &verified,
        &mut chain,
        &mut outbox,
        &mut idempotency,
        request,
    )
    .expect_err("same idempotency key with changed payload is rejected");

    assert_eq!(
        error,
        AuditEventEmitAppError::IdempotencyKeyReused {
            idempotency_key: "idem_audit_emit_reused".to_string()
        }
    );
    assert_eq!(
        error.audit_event_emit_status(),
        AuditEventEmitAppStatus::UnprocessableEntity
    );
    assert_eq!(idempotency.len(), 1);
    assert_eq!(chain.events().len(), 1);
    assert_eq!(outbox.records().len(), 1);
}

fn emit_request(event_id: &str, idempotency_key: &str) -> AuditEventEmitAppRequest {
    AuditEventEmitAppRequest {
        envelope: AuditEventEmitEnvelopeContext {
            event_id: event_id.to_string(),
            source: AUDIT_EVENT_EMIT_SOURCE.to_string(),
            subject: "tenant/ten_alpha/surface/cloud.compute.vm.create".to_string(),
            topic: AUDIT_EVENT_TOPIC.to_string(),
            schema: AUDIT_EVENT_EMIT_SCHEMA.to_string(),
            tenant_id: "ten_alpha".to_string(),
            producer_id: "producer_cloud_compute".to_string(),
            idempotency_key: idempotency_key.to_string(),
            produced_at_epoch_seconds: 1_700_000_000,
        },
        authorization: AuditEventEmitAuthorization {
            tenant_id: "ten_alpha".to_string(),
            producer_id: "producer_cloud_compute".to_string(),
            decision_id: "authz_audit_event_emit".to_string(),
            allowed_surfaces: vec![AUDIT_EVENT_EMIT_SURFACE.to_string()],
        },
        payload: AuditEventEmitPayload {
            id: event_id.to_string(),
            tenant_id: "ten_alpha".to_string(),
            surface: "cloud.compute.vm.create".to_string(),
            plane: "control".to_string(),
            purpose: "CoreService".to_string(),
            data_classes_touched: vec!["INTERNAL_ONLY".to_string(), "PUBLIC".to_string()],
            decision: DECISION.to_string(),
            idempotency_key: idempotency_key.to_string(),
            emitted_at_epoch_seconds: 1_700_000_000,
        },
    }
}
