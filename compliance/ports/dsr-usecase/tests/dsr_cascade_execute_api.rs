// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use compliance_dsr_usecase::{
    CallerCredential, ConfiguredBearerPrincipalVerifier, DsrCascadeAuthorizationError,
    DsrCascadeAuthorizer, DsrCascadeAuthzProvider, DsrCascadeResource, DsrCascadeScope,
    PLATFORM_DSR_CASCADE_EXECUTE_SURFACE, PLATFORM_DSR_OPENAPI_CONTRACT,
    PlatformDsrApiAuthorizationCorrelation, PlatformDsrApiError, PlatformDsrApiPrincipal,
    PlatformDsrCascadeBoundaryContext, PlatformDsrCascadeDirectory,
    PlatformDsrCascadeExecuteApiRequest, PlatformDsrCascadeExecuteApiStatus,
    PlatformDsrCascadeExecuteIdempotencyLedger, PlatformDsrCascadeExecuteRequest,
    PlatformDsrCascadeMetadata, PlatformDsrCascadeTargetRequest, PlatformDsrCompletionRecord,
    PrincipalVerificationError, PrincipalVerifier, VerifiedDsrPrincipal,
    execute_dsr_cascade_from_api,
};

const DSR_ID: &str = "dsr_001";
const TENANT_ID: &str = "ten_privacy_kr";
const SUBJECT_REF: &str = "subject:user@example.com";
const PRINCIPAL_ID: &str = "privacy-officer:kr";
const BEARER_SECRET: &str = "dsr-cascade-break-glass-token";

// ── Authz test doubles ─────────────────────────────────────────────────────

/// PDP double that authorizes ONLY a specific (principal, tenant, scope). This
/// proves blast-radius binding: a verified principal who would be allowed for
/// their OWN tenant is still denied for a DIFFERENT target tenant.
struct ScopedAuthorizer {
    allow_principal_id: String,
    allow_tenant_id: String,
}

impl DsrCascadeAuthorizer for ScopedAuthorizer {
    fn ensure_authorized(
        &self,
        principal: &VerifiedDsrPrincipal,
        resource: &DsrCascadeResource,
    ) -> Result<(), DsrCascadeAuthorizationError> {
        // Platform-scoped cascades require platform-admin authority — this
        // tenant-scoped authorizer never grants them (true blast radius).
        if resource.scope == DsrCascadeScope::Platform {
            return Err(DsrCascadeAuthorizationError::Denied);
        }
        if principal.principal_id() == self.allow_principal_id
            && resource.tenant_id == self.allow_tenant_id
        {
            Ok(())
        } else {
            Err(DsrCascadeAuthorizationError::Denied)
        }
    }
}

/// PDP double that ALWAYS allows — used to prove that the *cross-tenant guard /
/// resource binding* (not just the PDP) denies a cross-tenant target. If the
/// resource were flattened to the caller's own tenant this authorizer would let
/// a cross-tenant escalation through; the test asserts it does NOT.
struct AllowAllAuthorizer;
impl DsrCascadeAuthorizer for AllowAllAuthorizer {
    fn ensure_authorized(
        &self,
        _p: &VerifiedDsrPrincipal,
        _r: &DsrCascadeResource,
    ) -> Result<(), DsrCascadeAuthorizationError> {
        Ok(())
    }
}

struct DenyAuthorizer;
impl DsrCascadeAuthorizer for DenyAuthorizer {
    fn ensure_authorized(
        &self,
        _p: &VerifiedDsrPrincipal,
        _r: &DsrCascadeResource,
    ) -> Result<(), DsrCascadeAuthorizationError> {
        Err(DsrCascadeAuthorizationError::Denied)
    }
}

struct FaultAuthorizer;
impl DsrCascadeAuthorizer for FaultAuthorizer {
    fn ensure_authorized(
        &self,
        _p: &VerifiedDsrPrincipal,
        _r: &DsrCascadeResource,
    ) -> Result<(), DsrCascadeAuthorizationError> {
        Err(DsrCascadeAuthorizationError::Refused)
    }
}

fn verifier() -> ConfiguredBearerPrincipalVerifier {
    ConfiguredBearerPrincipalVerifier::new(BEARER_SECRET, PRINCIPAL_ID, TENANT_ID)
        .expect("verifier configures")
}

/// Obtain a REAL verified principal by running the bearer verifier — integration
/// tests (external crate) cannot mint a `VerifiedDsrPrincipal` any other way,
/// which is the unforgeability guarantee in action.
fn verified_principal() -> VerifiedDsrPrincipal {
    verifier()
        .verify_principal(&CallerCredential {
            authorization: Some(format!("Bearer {BEARER_SECRET}")),
            claimed_principal_id: PRINCIPAL_ID.to_string(),
            claimed_tenant_id: TENANT_ID.to_string(),
        })
        .expect("valid bearer verifies")
}

fn provider(authorizer: Arc<dyn DsrCascadeAuthorizer>) -> DsrCascadeAuthzProvider {
    DsrCascadeAuthzProvider::new(Arc::new(verifier()), authorizer)
}

fn allow_provider() -> DsrCascadeAuthzProvider {
    provider(Arc::new(ScopedAuthorizer {
        allow_principal_id: PRINCIPAL_ID.to_string(),
        allow_tenant_id: TENANT_ID.to_string(),
    }))
}

fn boundary(request_id: &str, idempotency_key: &str) -> PlatformDsrCascadeBoundaryContext {
    PlatformDsrCascadeBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: TENANT_ID.to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal() -> PlatformDsrApiPrincipal {
    PlatformDsrApiPrincipal {
        tenant_id: TENANT_ID.to_string(),
        principal_id: PRINCIPAL_ID.to_string(),
    }
}

fn correlation() -> PlatformDsrApiAuthorizationCorrelation {
    PlatformDsrApiAuthorizationCorrelation {
        decision_id: Some("authz-dsr-cascade".to_string()),
    }
}

fn target(
    dispatch_id: &str,
    record_ref: &str,
    proof_method: &str,
) -> PlatformDsrCascadeTargetRequest {
    PlatformDsrCascadeTargetRequest {
        dispatch_id: dispatch_id.to_string(),
        dispatch_idempotency_key: format!("idem-{dispatch_id}"),
        ack_id: format!("ack-{dispatch_id}"),
        ack_status: "completed".to_string(),
        ack_reason: None,
        axis: "workspace".to_string(),
        store_kind: "workspace_object".to_string(),
        store_id: "workspace-drive".to_string(),
        region: "region-home".to_string(),
        cell_id: "cell-kr-001".to_string(),
        record_ref: record_ref.to_string(),
        data_class: "PII_IDENTIFYING".to_string(),
        proof_id: Some(format!("proof-{dispatch_id}")),
        proof_method: Some(proof_method.to_string()),
        evidence_hash: Some(format!("sha256:{dispatch_id:0<64}")),
        witness_ref: Some("retention-dsr-worker".to_string()),
        signer_ref: Some("sigstore:privacy".to_string()),
        signature_ref: Some(format!("sig-{dispatch_id}")),
        rekor_log_index: Some(17),
        processed_at_epoch_seconds: 1_700_000_020,
    }
}

fn body(dsr_id: &str) -> PlatformDsrCascadeExecuteRequest {
    PlatformDsrCascadeExecuteRequest {
        dsr_id: dsr_id.to_string(),
        tenant_id: TENANT_ID.to_string(),
        region: "region-home".to_string(),
        subject_ref: SUBJECT_REF.to_string(),
        action: "erase".to_string(),
        sla_tier: "preview".to_string(),
        data_classes: vec!["PII_IDENTIFYING".to_string()],
        received_at_epoch_seconds: 1_700_000_000,
        deadline_epoch_seconds: 1_700_000_000 + (30 * 86_400),
        completion_id: "completion-dsr-001".to_string(),
        aggregate_proof_hash:
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        signer_ref: "sigstore:privacy".to_string(),
        signature_ref: "sig-completion-dsr-001".to_string(),
        rekor_log_index: 99,
        completed_at_epoch_seconds: 1_700_000_030,
        targets: vec![target("dispatch-001", "drive/object/1", "kms_shred")],
    }
}

fn request(request_id: &str, idempotency_key: &str) -> PlatformDsrCascadeExecuteApiRequest {
    PlatformDsrCascadeExecuteApiRequest {
        path_dsr_id: DSR_ID.to_string(),
        boundary: boundary(request_id, idempotency_key),
        principal: principal(),
        authorization: correlation(),
        body: body(DSR_ID),
    }
}

#[test]
fn openapi_runtime_binding_contracts_are_covered() {
    assert_eq!(PLATFORM_DSR_CASCADE_EXECUTE_SURFACE, "dsr.cascade.execute");
    assert_eq!(
        PLATFORM_DSR_OPENAPI_CONTRACT,
        "contracts/openapi/platform/platform-dsr-v1.yaml"
    );
    assert_eq!(PlatformDsrCascadeExecuteApiStatus::Accepted.code(), 202);
    assert_eq!(PlatformDsrCascadeExecuteApiStatus::BadRequest.code(), 400);
    assert_eq!(PlatformDsrCascadeExecuteApiStatus::Forbidden.code(), 403);
    assert_eq!(PlatformDsrCascadeExecuteApiStatus::Conflict.code(), 409);
    assert_eq!(
        PlatformDsrCascadeExecuteApiStatus::UnprocessableEntity.code(),
        422
    );
}

// ── RED/GREEN: the fail-closed authz seam (ADR-0589, C16) ──────────────────

#[test]
fn forged_principal_without_verified_credential_cannot_trigger_erasure_cascade() {
    // The RED test for C16: a caller who has NOT presented a valid credential
    // cannot obtain a VerifiedDsrPrincipal at all. The bearer verifier refuses
    // a missing/wrong credential with a 401-class error, and there is NO public
    // way to construct a VerifiedDsrPrincipal — so the execute entry point is
    // simply unreachable without authentication. Proves: forged
    // {tenant,principal} request fields are worthless without a real credential.
    let absent = verifier().verify_principal(&CallerCredential {
        authorization: None,
        claimed_principal_id: "attacker".to_string(),
        claimed_tenant_id: TENANT_ID.to_string(),
    });
    assert_eq!(
        absent.unwrap_err(),
        PrincipalVerificationError::MissingCredential
    );

    let forged = verifier().verify_principal(&CallerCredential {
        authorization: Some("Bearer not-the-secret".to_string()),
        claimed_principal_id: PRINCIPAL_ID.to_string(),
        claimed_tenant_id: TENANT_ID.to_string(),
    });
    assert_eq!(
        forged.unwrap_err(),
        PrincipalVerificationError::InvalidCredential
    );

    // And the HTTP-edge mapping of an unauthenticated request is 401.
    assert_eq!(PlatformDsrApiError::Unauthenticated.status_code(), 401);
}

#[test]
fn verified_principal_substitution_in_request_body_is_rejected_403() {
    let mut directory = PlatformDsrCascadeDirectory::default();
    let mut ledger = PlatformDsrCascadeExecuteIdempotencyLedger::default();
    let verified = verified_principal();
    let provider = allow_provider();

    // Caller verified as PRINCIPAL_ID but tries to act AS a different principal
    // in the request body. The cross-check against the verified identity rejects.
    let mut spoofed = request("req-spoof-principal", "idem-spoof-principal");
    spoofed.principal.principal_id = "privacy-officer:someone-else".to_string();
    let error =
        execute_dsr_cascade_from_api(&verified, &provider, &mut directory, &mut ledger, spoofed)
            .expect_err("substituted principal rejected");
    assert_eq!(error.status_code(), 403);
    assert!(matches!(
        error,
        PlatformDsrApiError::AuthorizationPrincipalMismatch { .. }
    ));

    // Same for a substituted tenant in the request principal.
    let mut spoofed_tenant = request("req-spoof-tenant", "idem-spoof-tenant");
    spoofed_tenant.boundary.tenant_id = "ten_other".to_string();
    spoofed_tenant.principal.tenant_id = "ten_other".to_string();
    spoofed_tenant.body.tenant_id = "ten_other".to_string();
    let error = execute_dsr_cascade_from_api(
        &verified,
        &provider,
        &mut directory,
        &mut ledger,
        spoofed_tenant,
    )
    .expect_err("substituted tenant rejected");
    assert_eq!(error.status_code(), 403);
    assert!(matches!(
        error,
        PlatformDsrApiError::AuthorizationTenantMismatch { .. }
    ));
    assert!(directory.is_empty());
}

#[test]
fn cross_tenant_target_is_denied_even_with_an_allow_all_authorizer_blast_radius() {
    // BLAST-RADIUS proof: a principal verified for ten_privacy_kr targets a
    // DIFFERENT tenant. We use an authorizer that would otherwise ALLOW
    // anything — so the denial must come from the resource binding presenting
    // the TRUE target tenant, not from a permissive PDP. Here the request is
    // internally consistent for ten_evil (header/principal/body all agree) but
    // the VERIFIED principal belongs to ten_privacy_kr, so the cross-check
    // catches the substitution first.
    let mut directory = PlatformDsrCascadeDirectory::default();
    let mut ledger = PlatformDsrCascadeExecuteIdempotencyLedger::default();
    let verified = verified_principal(); // bound to ten_privacy_kr
    let allow_all = provider(Arc::new(AllowAllAuthorizer));

    let mut cross = request("req-cross", "idem-cross");
    cross.boundary.tenant_id = "ten_evil".to_string();
    cross.principal.tenant_id = "ten_evil".to_string();
    cross.body.tenant_id = "ten_evil".to_string();
    let error =
        execute_dsr_cascade_from_api(&verified, &allow_all, &mut directory, &mut ledger, cross)
            .expect_err("cross-tenant target denied even with allow-all PDP");
    assert_eq!(error.status_code(), 403);
    assert!(directory.is_empty());
    assert!(ledger.is_empty());
}

#[test]
fn scoped_authorizer_binds_decision_to_the_target_tenant() {
    // The ScopedAuthorizer allows ONLY (PRINCIPAL_ID, TENANT_ID, Tenant-scope).
    // The happy path (same tenant) is allowed; this is the positive control for
    // the blast-radius test above (proving the authorizer is not vacuous).
    let mut directory = PlatformDsrCascadeDirectory::default();
    let mut ledger = PlatformDsrCascadeExecuteIdempotencyLedger::default();
    let verified = verified_principal();
    let provider = allow_provider();

    let response = execute_dsr_cascade_from_api(
        &verified,
        &provider,
        &mut directory,
        &mut ledger,
        request("req-scoped-ok", "idem-scoped-ok"),
    )
    .expect("scoped authorizer allows the bound tenant");
    assert_eq!(response.data.tenant_id, TENANT_ID);
}

#[test]
fn pdp_deny_maps_to_403_before_any_mutation() {
    let mut directory = PlatformDsrCascadeDirectory::default();
    let mut ledger = PlatformDsrCascadeExecuteIdempotencyLedger::default();
    let verified = verified_principal();
    let deny = provider(Arc::new(DenyAuthorizer));

    let error = execute_dsr_cascade_from_api(
        &verified,
        &deny,
        &mut directory,
        &mut ledger,
        request("req-deny", "idem-deny"),
    )
    .expect_err("PDP deny rejected");
    assert_eq!(error.status_code(), 403);
    assert!(matches!(
        error,
        PlatformDsrApiError::AuthorizationDenied { .. }
    ));
    assert!(directory.is_empty());
    assert!(ledger.is_empty());
}

#[test]
fn pdp_fault_fails_closed_to_403_never_500() {
    let mut directory = PlatformDsrCascadeDirectory::default();
    let mut ledger = PlatformDsrCascadeExecuteIdempotencyLedger::default();
    let verified = verified_principal();
    let fault = provider(Arc::new(FaultAuthorizer));

    let error = execute_dsr_cascade_from_api(
        &verified,
        &fault,
        &mut directory,
        &mut ledger,
        request("req-fault", "idem-fault"),
    )
    .expect_err("PDP fault fails closed");
    assert_eq!(error.status_code(), 403);
    assert!(matches!(
        error,
        PlatformDsrApiError::AuthorizationFault { .. }
    ));
    assert!(directory.is_empty());
}

// ── Existing behavioral coverage, rewired to the authorized entry point ─────

#[test]
fn dsr_cascade_execute_completes_once_and_replays_same_idempotent_result() {
    let mut directory = PlatformDsrCascadeDirectory::default();
    let mut ledger = PlatformDsrCascadeExecuteIdempotencyLedger::default();
    let verified = verified_principal();
    let provider = allow_provider();
    let request = request("req-dsr-cascade", "idem-dsr-cascade");

    let first = execute_dsr_cascade_from_api(
        &verified,
        &provider,
        &mut directory,
        &mut ledger,
        request.clone(),
    )
    .expect("DSR cascade completes");
    let second =
        execute_dsr_cascade_from_api(&verified, &provider, &mut directory, &mut ledger, request)
            .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(directory.len(), 1);
    assert_eq!(first.metadata.request_id, "req-dsr-cascade");
    assert_eq!(first.metadata.surface, PLATFORM_DSR_CASCADE_EXECUTE_SURFACE);
    assert_eq!(first.data.dsr_id, DSR_ID);
    assert_eq!(first.data.completion_status, "completed");
    assert_eq!(first.data.sla_status, "within_sla");
    assert_eq!(first.data.dispatch_ids, vec!["dispatch-001".to_string()]);
    assert_eq!(first.data.proof_ids, vec!["proof-dispatch-001".to_string()]);
    assert_eq!(first.data.store_count, 1);
}

#[test]
fn dsr_cascade_execute_requires_proof_of_erasure_per_affected_store() {
    let mut directory = PlatformDsrCascadeDirectory::default();
    let mut ledger = PlatformDsrCascadeExecuteIdempotencyLedger::default();
    let verified = verified_principal();
    let provider = allow_provider();
    let mut cascade_request = request("req-dsr-multistore", "idem-dsr-multistore");
    cascade_request
        .body
        .targets
        .push(target("dispatch-002", "drive/object/2", "record_delete"));

    let response = execute_dsr_cascade_from_api(
        &verified,
        &provider,
        &mut directory,
        &mut ledger,
        cascade_request,
    )
    .expect("each affected store has terminal ack plus erasure proof");

    assert_eq!(response.data.store_count, 2);
    assert_eq!(
        response.data.dispatch_ids,
        vec!["dispatch-001".to_string(), "dispatch-002".to_string()]
    );
    assert_eq!(
        response.data.proof_ids,
        vec![
            "proof-dispatch-001".to_string(),
            "proof-dispatch-002".to_string()
        ]
    );

    let mut missing_directory = PlatformDsrCascadeDirectory::default();
    let mut missing_ledger = PlatformDsrCascadeExecuteIdempotencyLedger::default();
    let mut missing_proof = request("req-dsr-missing-proof", "idem-dsr-missing-proof");
    missing_proof.body.targets[0].proof_id = None;
    let error = execute_dsr_cascade_from_api(
        &verified,
        &provider,
        &mut missing_directory,
        &mut missing_ledger,
        missing_proof,
    )
    .expect_err("completed store ack without proof is rejected");
    assert_eq!(error.status_code(), 400);
    assert!(matches!(
        error,
        PlatformDsrApiError::MissingCompletedProofField { .. }
    ));
}

#[test]
fn dsr_cascade_execute_rejects_path_body_tenant_and_principal_drift_before_mutation() {
    let mut directory = PlatformDsrCascadeDirectory::default();
    let mut ledger = PlatformDsrCascadeExecuteIdempotencyLedger::default();
    let verified = verified_principal();
    let provider = allow_provider();
    let mut drifted = request("req-dsr-drift", "idem-dsr-drift");
    drifted.body.dsr_id = "dsr-other".to_string();

    let error =
        execute_dsr_cascade_from_api(&verified, &provider, &mut directory, &mut ledger, drifted)
            .expect_err("path/body DSR id drift rejected");
    assert_eq!(error.status_code(), 400);
    assert!(matches!(error, PlatformDsrApiError::DsrIdMismatch { .. }));
    assert!(ledger.is_empty());
    assert!(directory.is_empty());

    // Header tenant drift (header != verified principal tenant) -> 403.
    let mut tenant_drift = request("req-dsr-tenant", "idem-dsr-tenant");
    tenant_drift.boundary.tenant_id = "ten-other".to_string();
    let error = execute_dsr_cascade_from_api(
        &verified,
        &provider,
        &mut directory,
        &mut ledger,
        tenant_drift,
    )
    .expect_err("tenant drift rejected");
    assert_eq!(error.status_code(), 403);

    let mut principal_drift = request("req-dsr-principal", "idem-dsr-principal");
    principal_drift.principal.principal_id = "privacy-officer:other".to_string();
    let error = execute_dsr_cascade_from_api(
        &verified,
        &provider,
        &mut directory,
        &mut ledger,
        principal_drift,
    )
    .expect_err("principal drift rejected");
    assert_eq!(error.status_code(), 403);
    assert!(matches!(
        error,
        PlatformDsrApiError::AuthorizationPrincipalMismatch { .. }
    ));
    assert!(directory.is_empty());
}

#[test]
fn dsr_cascade_execute_rejects_reused_idempotency_key_and_duplicate_completion() {
    let mut directory = PlatformDsrCascadeDirectory::default();
    let mut ledger = PlatformDsrCascadeExecuteIdempotencyLedger::default();
    let verified = verified_principal();
    let provider = allow_provider();

    let first = request("req-dsr-idem", "idem-dsr-idem");
    execute_dsr_cascade_from_api(
        &verified,
        &provider,
        &mut directory,
        &mut ledger,
        first.clone(),
    )
    .expect("initial DSR cascade succeeds");
    let mut drift = first;
    drift.body.aggregate_proof_hash =
        "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string();
    assert_eq!(
        execute_dsr_cascade_from_api(&verified, &provider, &mut directory, &mut ledger, drift),
        Err(PlatformDsrApiError::IdempotencyKeyReused {
            idempotency_key: "idem-dsr-idem".to_string(),
        })
    );

    let duplicate = execute_dsr_cascade_from_api(
        &verified,
        &provider,
        &mut directory,
        &mut ledger,
        request("req-dsr-dup", "idem-dsr-dup"),
    )
    .expect_err("same dsr through new idempotency key conflicts");
    assert_eq!(duplicate.status_code(), 409);
    assert!(matches!(
        duplicate,
        PlatformDsrApiError::CascadeAlreadyCompleted { .. }
    ));
}

#[test]
fn dsr_cascade_execute_maps_invalid_labels_scope_and_duplicate_targets() {
    let mut directory = PlatformDsrCascadeDirectory::default();
    let mut ledger = PlatformDsrCascadeExecuteIdempotencyLedger::default();
    let verified = verified_principal();
    let provider = allow_provider();

    let mut invalid_action = request("req-dsr-action", "idem-dsr-action");
    invalid_action.body.action = "delete_everything".to_string();
    let error = execute_dsr_cascade_from_api(
        &verified,
        &provider,
        &mut directory,
        &mut ledger,
        invalid_action,
    )
    .expect_err("unknown action is rejected before kernel");
    assert_eq!(error.status_code(), 400);
    assert!(matches!(
        error,
        PlatformDsrApiError::InvalidActionLabel { .. }
    ));

    let mut invalid_class = request("req-dsr-class", "idem-dsr-class");
    invalid_class.body.data_classes = vec!["AUDIT".to_string()];
    let error = execute_dsr_cascade_from_api(
        &verified,
        &provider,
        &mut directory,
        &mut ledger,
        invalid_class,
    )
    .expect_err("operational data class is rejected");
    assert_eq!(error.status_code(), 400);
    assert!(matches!(
        error,
        PlatformDsrApiError::InvalidDataClassLabel { .. }
    ));

    let mut out_of_scope = request("req-dsr-scope", "idem-dsr-scope");
    out_of_scope.body.targets[0].region = "region-recovery".to_string();
    let error = execute_dsr_cascade_from_api(
        &verified,
        &provider,
        &mut directory,
        &mut ledger,
        out_of_scope,
    )
    .expect_err("out-of-scope store is rejected by kernel");
    assert_eq!(error.status_code(), 400);
    assert!(matches!(error, PlatformDsrApiError::Kernel(_)));

    let mut duplicate_target = request("req-dsr-duptarget", "idem-dsr-duptarget");
    duplicate_target
        .body
        .targets
        .push(duplicate_target.body.targets[0].clone());
    let error = execute_dsr_cascade_from_api(
        &verified,
        &provider,
        &mut directory,
        &mut ledger,
        duplicate_target,
    )
    .expect_err("duplicate dispatch/store coverage rejected");
    assert_eq!(error.status_code(), 400);
    assert!(matches!(error, PlatformDsrApiError::Kernel(_)));
}

#[test]
fn stable_error_response_shape_uses_request_id_and_field_details() {
    let error = PlatformDsrApiError::InvalidDataClassLabel {
        data_class: "AUDIT".to_string(),
    };

    let response = error.error_response("req-dsr-error");

    assert_eq!(response.error.code, "PLATFORM_DSR_DATA_CLASS_INVALID");
    assert_eq!(response.error.request_id, "req-dsr-error");
    assert_eq!(response.error.details[0].field, "body.data_classes");
    assert_eq!(response.error.retry_after_seconds, None);
}

#[test]
fn public_response_structs_keep_contract_names_stable() {
    let _metadata = PlatformDsrCascadeMetadata {
        request_id: "req-dsr-structs".to_string(),
        surface: PLATFORM_DSR_CASCADE_EXECUTE_SURFACE.to_string(),
        openapi_contract: PLATFORM_DSR_OPENAPI_CONTRACT.to_string(),
    };
    let _record = PlatformDsrCompletionRecord {
        dsr_id: DSR_ID.to_string(),
        tenant_id: TENANT_ID.to_string(),
        subject_ref: SUBJECT_REF.to_string(),
        action: "erase".to_string(),
        completion_id: "completion-dsr-001".to_string(),
        completion_status: "completed".to_string(),
        sla_status: "within_sla".to_string(),
        dispatch_ids: Vec::new(),
        ack_ids: Vec::new(),
        proof_ids: Vec::new(),
        aggregate_proof_hash:
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        signer_ref: "sigstore:privacy".to_string(),
        signature_ref: "sig-completion-dsr-001".to_string(),
        rekor_log_index: 99,
        completed_at_epoch_seconds: 1_700_000_030,
        schema_version: 1,
        store_count: 0,
    };
}
