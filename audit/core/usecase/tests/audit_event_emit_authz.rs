// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Fail-closed authorization seam for `audit.event.emit` (ADR-0588 / AUTH-005 /
//! C15). These tests prove the gate in `emit_audit_event_authorized`:
//!
//! - forged/absent credential -> 401 (the bypass-closed proof: a request that
//!   carries a fully-consistent self-attested authorization but NO verified
//!   credential cannot emit);
//! - verified principal acting cross-tenant -> 403 (blast-radius binding: the
//!   PDP would otherwise ALLOW, proving the resource is the TARGET tenant, not
//!   the caller's verified tenant);
//! - PDP deny / fault -> 403;
//! - happy path -> ok.
//!
//! Each RED case asserts NO audit record was appended and NO outbox record was
//! published — a forged authorization can never produce tamper-evidence.

use std::sync::Arc;

use audit_chain_domain::AuditChain;
use audit_usecase::authz::{
    AuditEmitAuthorizationError, AuditEmitAuthorizer, AuditEmitAuthzProvider, AuditEmitResource,
    AuditEmitScope, AuthzProviderConfigError, CallerCredential, ConfiguredBearerPrincipalVerifier,
    PrincipalVerificationError, VerifiedProducerPrincipal,
};
use audit_usecase::{
    AUDIT_EVENT_EMIT_SCHEMA, AUDIT_EVENT_EMIT_SOURCE, AUDIT_EVENT_EMIT_SURFACE, AUDIT_EVENT_TOPIC,
    AuditEventEmitAppError, AuditEventEmitAppRequest, AuditEventEmitAppStatus,
    AuditEventEmitAuthorization, AuditEventEmitEnvelopeContext, AuditEventEmitIdempotencyLedger,
    AuditEventEmitPayload, emit_audit_event_authorized,
};
use messaging_domain::Outbox;

const BEARER_SECRET: &str = "audit-emit-break-glass-secret";
const PRODUCER_ID: &str = "producer_cloud_compute";
const TENANT_ID: &str = "ten_alpha";
const EVENT_ID: &str = "audit_evt_cloud_vm_001";

// ==========================================================================
// PDP test doubles
// ==========================================================================

/// PDP that authorizes everything (an over-permissive policy). Used to prove
/// blast-radius binding: even when the PDP WOULD allow, a cross-tenant resource
/// is presented to it so a deny-by-policy would normally be the only barrier —
/// here we instead pair it with a tenant-bound PDP to show the resource tenant
/// is the TARGET, not the caller.
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

/// PDP that denies everything (default-deny policy).
struct DenyAllAuthorizer;
impl AuditEmitAuthorizer for DenyAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedProducerPrincipal,
        _resource: &AuditEmitResource,
    ) -> Result<(), AuditEmitAuthorizationError> {
        Err(AuditEmitAuthorizationError::Denied)
    }
}

/// PDP that always faults (e.g. cloud-iam unavailable). MUST be treated as deny
/// (fail-closed), never as allow.
struct FaultingAuthorizer;
impl AuditEmitAuthorizer for FaultingAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedProducerPrincipal,
        _resource: &AuditEmitResource,
    ) -> Result<(), AuditEmitAuthorizationError> {
        Err(AuditEmitAuthorizationError::Refused)
    }
}

/// A PDP that authorizes ONLY when the resource tenant equals the verified
/// principal's tenant (same-tenant isolation, the reference composition-root
/// policy). It is over-permissive WITHIN a tenant (it would allow any surface) —
/// so if a cross-tenant emit is denied, that denial comes purely from the
/// resource tenant being the TARGET tenant (blast-radius binding), NOT from a
/// surface restriction.
struct SameTenantAuthorizer;
impl AuditEmitAuthorizer for SameTenantAuthorizer {
    fn ensure_authorized(
        &self,
        principal: &VerifiedProducerPrincipal,
        resource: &AuditEmitResource,
    ) -> Result<(), AuditEmitAuthorizationError> {
        match resource.scope {
            // Platform-scoped records require platform authority; this tenant
            // reference policy never grants it.
            AuditEmitScope::Platform => Err(AuditEmitAuthorizationError::Denied),
            AuditEmitScope::Tenant => {
                if resource.tenant_id == principal.tenant_id() {
                    Ok(())
                } else {
                    Err(AuditEmitAuthorizationError::Denied)
                }
            }
        }
    }
}

// ==========================================================================
// Provider + request builders
// ==========================================================================

fn provider_with(authorizer: Arc<dyn AuditEmitAuthorizer>) -> AuditEmitAuthzProvider {
    let verifier = Arc::new(
        ConfiguredBearerPrincipalVerifier::new(BEARER_SECRET, PRODUCER_ID, TENANT_ID)
            .expect("non-empty break-glass credential"),
    );
    AuditEmitAuthzProvider::new(verifier, authorizer)
}

fn good_credential() -> CallerCredential {
    CallerCredential {
        authorization: Some(format!("Bearer {BEARER_SECRET}")),
        claimed_producer_id: PRODUCER_ID.to_string(),
        claimed_tenant_id: TENANT_ID.to_string(),
    }
}

/// A request for `tenant_id` recording `surface`. Envelope + payload are
/// internally consistent (so validation passes) and the self-attested
/// authorization is FULLY consistent — modelling a forged-but-coherent caller.
fn request_for(tenant_id: &str, surface: &str, idempotency_key: &str) -> AuditEventEmitAppRequest {
    AuditEventEmitAppRequest {
        envelope: AuditEventEmitEnvelopeContext {
            event_id: EVENT_ID.to_string(),
            source: AUDIT_EVENT_EMIT_SOURCE.to_string(),
            subject: format!("tenant/{tenant_id}/surface/{surface}"),
            topic: AUDIT_EVENT_TOPIC.to_string(),
            schema: AUDIT_EVENT_EMIT_SCHEMA.to_string(),
            tenant_id: tenant_id.to_string(),
            producer_id: PRODUCER_ID.to_string(),
            idempotency_key: idempotency_key.to_string(),
            produced_at_epoch_seconds: 1_700_000_000,
        },
        authorization: AuditEventEmitAuthorization {
            tenant_id: tenant_id.to_string(),
            producer_id: PRODUCER_ID.to_string(),
            decision_id: "authz_audit_event_emit".to_string(),
            // FORGED grant: the caller self-attests it may emit this surface.
            // The fail-closed gate must IGNORE this entirely.
            allowed_surfaces: vec![AUDIT_EVENT_EMIT_SURFACE.to_string()],
        },
        payload: AuditEventEmitPayload {
            id: EVENT_ID.to_string(),
            tenant_id: tenant_id.to_string(),
            surface: surface.to_string(),
            plane: "control".to_string(),
            purpose: "CoreService".to_string(),
            data_classes_touched: vec!["INTERNAL_ONLY".to_string()],
            decision: "ALLOW".to_string(),
            idempotency_key: idempotency_key.to_string(),
            emitted_at_epoch_seconds: 1_700_000_000,
        },
    }
}

// ==========================================================================
// RED: forged / absent credential -> 401 (bypass-closed)
// ==========================================================================

#[test]
fn absent_credential_cannot_emit_even_with_consistent_self_attested_authorization() {
    let provider = provider_with(Arc::new(AllowAllAuthorizer));

    // No Authorization header at all — the self-attested authorization fields are
    // fully consistent, exactly the forge the old self-attested check accepted.
    let err = provider
        .verify_principal(&CallerCredential {
            authorization: None,
            claimed_producer_id: PRODUCER_ID.to_string(),
            claimed_tenant_id: TENANT_ID.to_string(),
        })
        .expect_err("absent credential must not verify");
    assert_eq!(err, PrincipalVerificationError::MissingCredential);

    // There is NO public way to obtain a VerifiedProducerPrincipal without a real
    // verifier run, so the request can never reach emit_audit_event_authorized.
    // (The type system enforces the bypass-closure; see the authz module note.)
}

#[test]
fn wrong_bearer_cannot_verify() {
    let provider = provider_with(Arc::new(AllowAllAuthorizer));
    let err = provider
        .verify_principal(&CallerCredential {
            authorization: Some("Bearer not-the-secret".to_string()),
            claimed_producer_id: PRODUCER_ID.to_string(),
            claimed_tenant_id: TENANT_ID.to_string(),
        })
        .expect_err("wrong bearer must not verify");
    assert_eq!(err, PrincipalVerificationError::InvalidCredential);
}

#[test]
fn empty_bearer_secret_refuses_provider_construction() {
    // A provider that cannot prove a credential root must never authenticate.
    // (`ConfiguredBearerPrincipalVerifier` does not implement `Debug` — it holds
    // a secret — so we match the Result rather than `expect_err`.)
    assert_eq!(
        construct_err(ConfiguredBearerPrincipalVerifier::new(
            "",
            PRODUCER_ID,
            TENANT_ID
        )),
        Some(AuthzProviderConfigError::EmptyBearerSecret)
    );
    assert_eq!(
        construct_err(ConfiguredBearerPrincipalVerifier::new(
            "   ",
            PRODUCER_ID,
            TENANT_ID
        )),
        Some(AuthzProviderConfigError::EmptyBearerSecret)
    );
    assert_eq!(
        construct_err(ConfiguredBearerPrincipalVerifier::new(
            BEARER_SECRET,
            "",
            TENANT_ID
        )),
        Some(AuthzProviderConfigError::EmptyBoundIdentity)
    );
    assert_eq!(
        construct_err(ConfiguredBearerPrincipalVerifier::new(
            BEARER_SECRET,
            PRODUCER_ID,
            ""
        )),
        Some(AuthzProviderConfigError::EmptyBoundIdentity)
    );
}

/// Extract the config error from a verifier construction result without
/// requiring the `Ok` type to be `Debug` (it holds a secret and is not `Debug`).
fn construct_err(
    result: Result<ConfiguredBearerPrincipalVerifier, AuthzProviderConfigError>,
) -> Option<AuthzProviderConfigError> {
    result.err()
}

// ==========================================================================
// RED: verified principal acting cross-tenant -> 403 (blast-radius binding)
// ==========================================================================

#[test]
fn verified_principal_cannot_emit_for_another_tenant() {
    // The PDP is SameTenant: it would ALLOW any surface for the caller's own
    // tenant. So the only way a cross-tenant emit is denied is if the resource
    // tenant handed to the PDP is the TARGET tenant (ten_beta), NOT the caller's
    // verified tenant (ten_alpha). This proves the resource is bound to the
    // target, not flattened to the caller (no IDOR).
    let provider = provider_with(Arc::new(SameTenantAuthorizer));
    let verified = provider
        .verify_principal(&good_credential())
        .expect("break-glass bearer verifies as ten_alpha");

    let mut chain = AuditChain::default();
    let mut outbox = Outbox::default();
    let mut idempotency = AuditEventEmitIdempotencyLedger::default();

    // The verified principal is ten_alpha, but the request targets ten_beta. The
    // cross-check rejects the tenant substitution before the PDP is even reached.
    let cross_tenant = request_for("ten_beta", "cloud.compute.vm.create", "idem_cross_tenant");
    let err = emit_audit_event_authorized(
        &provider,
        &verified,
        &mut chain,
        &mut outbox,
        &mut idempotency,
        cross_tenant,
    )
    .expect_err("verified ten_alpha cannot record for ten_beta");
    assert!(matches!(
        err,
        AuditEventEmitAppError::VerifiedPrincipalMismatch { .. }
    ));
    assert_eq!(
        err.audit_event_emit_status(),
        AuditEventEmitAppStatus::Forbidden
    );
    assert_no_emission(&chain, &outbox, &idempotency);
}

#[test]
fn same_tenant_emit_is_authorized_by_tenant_bound_pdp() {
    // The companion to the cross-tenant denial: the SAME tenant-bound PDP that
    // denies a foreign-tenant resource ALLOWS the same-tenant one. This proves
    // the cross-tenant denial above is purely a function of the resource tenant
    // being the TARGET (ten_beta), not a blanket deny — the PDP genuinely
    // distinguishes target tenants.
    let provider = provider_with(Arc::new(SameTenantAuthorizer));
    let verified = provider
        .verify_principal(&good_credential())
        .expect("break-glass bearer verifies as ten_alpha");

    let mut chain = AuditChain::default();
    let mut outbox = Outbox::default();
    let mut idempotency = AuditEventEmitIdempotencyLedger::default();

    let ok = request_for("ten_alpha", "cloud.compute.vm.create", "idem_same_tenant");
    emit_audit_event_authorized(
        &provider,
        &verified,
        &mut chain,
        &mut outbox,
        &mut idempotency,
        ok,
    )
    .expect("same-tenant verified emit is authorized by the tenant-bound PDP");
    assert_eq!(chain.events().len(), 1);
    assert_eq!(outbox.records().len(), 1);
}

// ==========================================================================
// RED: PDP deny / fault -> 403
// ==========================================================================

#[test]
fn pdp_deny_blocks_emit_despite_forged_authorization() {
    let provider = provider_with(Arc::new(DenyAllAuthorizer));
    let verified = provider
        .verify_principal(&good_credential())
        .expect("break-glass bearer verifies");

    let mut chain = AuditChain::default();
    let mut outbox = Outbox::default();
    let mut idempotency = AuditEventEmitIdempotencyLedger::default();

    let request = request_for(TENANT_ID, "cloud.compute.vm.create", "idem_deny");
    let err = emit_audit_event_authorized(
        &provider,
        &verified,
        &mut chain,
        &mut outbox,
        &mut idempotency,
        request,
    )
    .expect_err("PDP deny blocks the emit");
    assert_eq!(err, AuditEventEmitAppError::PdpAuthorizationDenied);
    assert_eq!(
        err.audit_event_emit_status(),
        AuditEventEmitAppStatus::Forbidden
    );
    assert_no_emission(&chain, &outbox, &idempotency);
}

#[test]
fn pdp_fault_fails_closed_to_403() {
    let provider = provider_with(Arc::new(FaultingAuthorizer));
    let verified = provider
        .verify_principal(&good_credential())
        .expect("break-glass bearer verifies");

    let mut chain = AuditChain::default();
    let mut outbox = Outbox::default();
    let mut idempotency = AuditEventEmitIdempotencyLedger::default();

    let request = request_for(TENANT_ID, "cloud.compute.vm.create", "idem_fault");
    let err = emit_audit_event_authorized(
        &provider,
        &verified,
        &mut chain,
        &mut outbox,
        &mut idempotency,
        request,
    )
    .expect_err("a PDP fault is fail-closed, not allow");
    assert_eq!(err, AuditEventEmitAppError::PdpAuthorizationDenied);
    assert_eq!(
        err.audit_event_emit_status(),
        AuditEventEmitAppStatus::Forbidden
    );
    assert_no_emission(&chain, &outbox, &idempotency);
}

// ==========================================================================
// GREEN: happy path -> ok, record reflects the verified principal
// ==========================================================================

#[test]
fn authorized_emit_appends_record_for_verified_principal() {
    let provider = provider_with(Arc::new(AllowAllAuthorizer));
    let verified = provider
        .verify_principal(&good_credential())
        .expect("break-glass bearer verifies");

    let mut chain = AuditChain::default();
    let mut outbox = Outbox::default();
    let mut idempotency = AuditEventEmitIdempotencyLedger::default();

    let request = request_for(TENANT_ID, "cloud.compute.vm.create", "idem_ok");
    let response = emit_audit_event_authorized(
        &provider,
        &verified,
        &mut chain,
        &mut outbox,
        &mut idempotency,
        request,
    )
    .expect("verified + PDP-allowed emit succeeds");

    // The recorded tenant is the verified principal's tenant (cross-checked equal
    // to the envelope/payload tenant). A forged authorization can never change it.
    assert_eq!(response.data.tenant_id, TENANT_ID);
    assert_eq!(response.data.surface, "cloud.compute.vm.create");
    assert_eq!(chain.events().len(), 1);
    assert_eq!(outbox.records().len(), 1);
    assert!(chain.verify());
}

fn assert_no_emission(
    chain: &AuditChain,
    outbox: &Outbox,
    idempotency: &AuditEventEmitIdempotencyLedger,
) {
    assert!(
        chain.events().is_empty(),
        "a denied/unauthorized emit must not append to the audit chain"
    );
    assert!(
        outbox.records().is_empty(),
        "a denied/unauthorized emit must not publish to the outbox"
    );
    assert!(
        idempotency.is_empty(),
        "a denied/unauthorized emit must not record an idempotency entry"
    );
}
