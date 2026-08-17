// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use compliance_dsr_usecase::authz::{
    ConfiguredBearerDsrPrincipalVerifier, DsrCallerCredential, DsrCascadeAuthorizationError,
    DsrCascadeAuthorizer, DsrCascadeAuthzProvider, DsrCascadePrincipalVerifier, DsrCascadeResource,
    DsrPrincipalVerificationError, VerifiedDsrPrincipal,
};
use compliance_dsr_usecase::{
    PLATFORM_DSR_CASCADE_EXECUTE_SURFACE, PLATFORM_DSR_OPENAPI_CONTRACT,
    PlatformDsrApiAuthorization, PlatformDsrApiError, PlatformDsrApiPrincipal,
    PlatformDsrCascadeBoundaryContext, PlatformDsrCascadeDirectory,
    PlatformDsrCascadeExecuteApiRequest, PlatformDsrCascadeExecuteApiStatus,
    PlatformDsrCascadeExecuteIdempotencyLedger, PlatformDsrCascadeExecuteRequest,
    PlatformDsrCascadeMetadata, PlatformDsrCascadeTargetRequest, PlatformDsrCompletionRecord,
    execute_dsr_cascade_from_api,
};

const DSR_ID: &str = "dsr_001";
const TENANT_ID: &str = "ten_privacy_kr";
const SUBJECT_REF: &str = "subject:user@example.com";
const PRINCIPAL_ID: &str = "privacy-officer:kr";
const BEARER_SECRET: &str = "dsr-break-glass";

/// Mint a [`VerifiedDsrPrincipal`] for the privacy officer by running the REAL
/// [`ConfiguredBearerDsrPrincipalVerifier`] the composition root uses. There is
/// no public constructor for `VerifiedDsrPrincipal`, so this is the ONLY way an
/// external crate can obtain one — proving the type is unforgeable.
fn verified_principal() -> VerifiedDsrPrincipal {
    let verifier =
        ConfiguredBearerDsrPrincipalVerifier::new(BEARER_SECRET, PRINCIPAL_ID, TENANT_ID)
            .expect("verifier construction");
    verifier
        .verify_principal(&DsrCallerCredential {
            authorization: Some(format!("Bearer {BEARER_SECRET}")),
            claimed_principal_id: PRINCIPAL_ID.to_string(),
            claimed_tenant_id: TENANT_ID.to_string(),
        })
        .expect("bearer verifies")
}

/// A PDP authorizer that allows everything (GREEN-path stand-in for the cloud-iam
/// Cedar PDP client).
struct AllowAllAuthorizer;
impl DsrCascadeAuthorizer for AllowAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedDsrPrincipal,
        _resource: &DsrCascadeResource,
    ) -> Result<(), DsrCascadeAuthorizationError> {
        Ok(())
    }
}

/// A PDP authorizer that denies everything (proves the server-side PDP seam — a
/// deny is 403, NOT the retired caller-supplied `allowed_surfaces` blob).
struct DenyAllAuthorizer;
impl DsrCascadeAuthorizer for DenyAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedDsrPrincipal,
        _resource: &DsrCascadeResource,
    ) -> Result<(), DsrCascadeAuthorizationError> {
        Err(DsrCascadeAuthorizationError::Denied)
    }
}

/// A PDP authorizer that returns a fault (timeout/unavailability stand-in).
/// Proves PDP-FAULT-DENIES: a refusal maps to 403, not allow.
struct FaultAuthorizer;
impl DsrCascadeAuthorizer for FaultAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedDsrPrincipal,
        _resource: &DsrCascadeResource,
    ) -> Result<(), DsrCascadeAuthorizationError> {
        Err(DsrCascadeAuthorizationError::Refused)
    }
}

/// A PDP authorizer that asserts the resource is bound to the TARGET tenant +
/// dsr from a trusted source — never a forged blob. Proves the BLAST-RADIUS
/// binding.
struct TargetBindingAssertingAuthorizer;
impl DsrCascadeAuthorizer for TargetBindingAssertingAuthorizer {
    fn ensure_authorized(
        &self,
        principal: &VerifiedDsrPrincipal,
        resource: &DsrCascadeResource,
    ) -> Result<(), DsrCascadeAuthorizationError> {
        assert_eq!(resource.tenant_id, principal.tenant_id());
        assert_eq!(resource.tenant_id, TENANT_ID);
        assert_eq!(resource.dsr_id, DSR_ID);
        assert_eq!(resource.surface, PLATFORM_DSR_CASCADE_EXECUTE_SURFACE);
        Ok(())
    }
}

fn provider_with(authorizer: Arc<dyn DsrCascadeAuthorizer>) -> DsrCascadeAuthzProvider {
    let verifier =
        ConfiguredBearerDsrPrincipalVerifier::new(BEARER_SECRET, PRINCIPAL_ID, TENANT_ID)
            .expect("verifier construction");
    DsrCascadeAuthzProvider::new(Arc::new(verifier), authorizer)
}

fn allow_all_provider() -> DsrCascadeAuthzProvider {
    provider_with(Arc::new(AllowAllAuthorizer))
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

fn correlation() -> PlatformDsrApiAuthorization {
    PlatformDsrApiAuthorization {
        decision_id: "authz-dsr-cascade".to_string(),
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
    assert_eq!(PlatformDsrCascadeExecuteApiStatus::Unauthorized.code(), 401);
    assert_eq!(PlatformDsrCascadeExecuteApiStatus::Forbidden.code(), 403);
    assert_eq!(PlatformDsrCascadeExecuteApiStatus::Conflict.code(), 409);
    assert_eq!(
        PlatformDsrCascadeExecuteApiStatus::UnprocessableEntity.code(),
        422
    );
}

// ====================================================================
// AUTH-005 / Wave-2b RED tests (ADR-0589): a forged principal or forged
// authorization blob cannot trigger an erasure cascade.
// ====================================================================

/// RED: the pre-fix CRITICAL. A caller who forges the request body alone — a
/// correlation blob with NO verified credential and NO PDP allow — must NOT be
/// able to execute the erasure cascade. With a DENY PDP it is 403 with no state
/// mutation. There is no longer ANY `allowed_surfaces` field for the caller to
/// fabricate.
#[test]
fn dsr_cascade_pdp_deny_returns_403_with_no_state_mutation() {
    let mut directory = PlatformDsrCascadeDirectory::default();
    let mut ledger = PlatformDsrCascadeExecuteIdempotencyLedger::default();
    let deny_provider = provider_with(Arc::new(DenyAllAuthorizer));

    let error = execute_dsr_cascade_from_api(
        &verified_principal(),
        &deny_provider,
        &mut directory,
        &mut ledger,
        request("req-dsr-deny", "idem-dsr-deny"),
    )
    .expect_err("a PDP deny must block the erasure cascade");

    assert_eq!(error.status_code(), 403);
    assert!(matches!(
        error,
        PlatformDsrApiError::CascadeAuthorizationDenied { .. }
    ));
    assert!(ledger.is_empty());
    assert!(directory.is_empty());
}

/// RED: PDP-FAULT-DENIES. A PDP refusal (timeout/network/unavailability) MUST
/// fail closed to 403 — never allow.
#[test]
fn dsr_cascade_pdp_fault_denies_with_403() {
    let mut directory = PlatformDsrCascadeDirectory::default();
    let mut ledger = PlatformDsrCascadeExecuteIdempotencyLedger::default();
    let fault_provider = provider_with(Arc::new(FaultAuthorizer));

    let error = execute_dsr_cascade_from_api(
        &verified_principal(),
        &fault_provider,
        &mut directory,
        &mut ledger,
        request("req-dsr-fault", "idem-dsr-fault"),
    )
    .expect_err("a PDP fault must fail closed");

    assert_eq!(error.status_code(), 403);
    assert!(matches!(
        error,
        PlatformDsrApiError::CascadeAuthorizationDenied { .. }
    ));
    assert!(directory.is_empty());
}

/// RED: a forged caller-asserted principal id (different from the verified
/// identity) cannot substitute itself onto the cascade. 403, no mutation.
#[test]
fn dsr_cascade_rejects_forged_caller_principal_against_verified() {
    let mut directory = PlatformDsrCascadeDirectory::default();
    let mut ledger = PlatformDsrCascadeExecuteIdempotencyLedger::default();
    let mut forged = request("req-dsr-forged-principal", "idem-dsr-forged-principal");
    forged.principal.principal_id = "attacker".to_string();

    let error = execute_dsr_cascade_from_api(
        &verified_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        forged,
    )
    .expect_err("forged caller principal cannot override the verified identity");

    assert_eq!(error.status_code(), 403);
    assert!(matches!(
        error,
        PlatformDsrApiError::VerifiedPrincipalMismatch { .. }
    ));
    assert!(directory.is_empty());
}

/// RED: a forged body tenant (claiming a different tenant than the verified
/// principal) cannot expand the blast radius. 403, no mutation.
#[test]
fn dsr_cascade_rejects_forged_body_tenant_against_verified() {
    let mut directory = PlatformDsrCascadeDirectory::default();
    let mut ledger = PlatformDsrCascadeExecuteIdempotencyLedger::default();
    let mut forged = request("req-dsr-forged-tenant", "idem-dsr-forged-tenant");
    forged.principal.tenant_id = "ten_victim".to_string();
    forged.boundary.tenant_id = "ten_victim".to_string();
    forged.body.tenant_id = "ten_victim".to_string();

    let error = execute_dsr_cascade_from_api(
        &verified_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        forged,
    )
    .expect_err("forged tenant cannot override the verified tenant");

    assert_eq!(error.status_code(), 403);
    assert!(matches!(
        error,
        PlatformDsrApiError::VerifiedTenantMismatch { .. }
    ));
    assert!(directory.is_empty());
}

/// A caller cannot forge a `VerifiedDsrPrincipal`: there is no public
/// constructor, so this integration crate can only obtain one by running the
/// real verifier with the right bearer. A wrong/missing bearer refuses (401
/// class) and the cascade is never reached.
#[test]
fn verifier_refuses_wrong_and_missing_credential() {
    let verifier =
        ConfiguredBearerDsrPrincipalVerifier::new(BEARER_SECRET, PRINCIPAL_ID, TENANT_ID)
            .expect("verifier construction");
    assert_eq!(
        verifier
            .verify_principal(&DsrCallerCredential {
                authorization: None,
                claimed_principal_id: PRINCIPAL_ID.to_string(),
                claimed_tenant_id: TENANT_ID.to_string(),
            })
            .unwrap_err(),
        DsrPrincipalVerificationError::MissingCredential
    );
    assert_eq!(
        verifier
            .verify_principal(&DsrCallerCredential {
                authorization: Some("Bearer wrong".to_string()),
                claimed_principal_id: PRINCIPAL_ID.to_string(),
                claimed_tenant_id: TENANT_ID.to_string(),
            })
            .unwrap_err(),
        DsrPrincipalVerificationError::InvalidCredential
    );
}

/// The PDP resource is bound to the TARGET tenant + dsr from a trusted source,
/// never a caller-forged surface list.
#[test]
fn dsr_cascade_binds_pdp_resource_to_target_tenant_and_dsr() {
    let mut directory = PlatformDsrCascadeDirectory::default();
    let mut ledger = PlatformDsrCascadeExecuteIdempotencyLedger::default();
    let provider = provider_with(Arc::new(TargetBindingAssertingAuthorizer));

    let response = execute_dsr_cascade_from_api(
        &verified_principal(),
        &provider,
        &mut directory,
        &mut ledger,
        request("req-dsr-binding", "idem-dsr-binding"),
    )
    .expect("target-binding authorizer allows and asserts the bound resource");
    assert_eq!(response.data.dsr_id, DSR_ID);
}

// ====================================================================
// GREEN-path behaviour (PDP allow) — preserved cascade semantics.
// ====================================================================

#[test]
fn dsr_cascade_execute_completes_once_and_replays_same_idempotent_result() {
    let mut directory = PlatformDsrCascadeDirectory::default();
    let mut ledger = PlatformDsrCascadeExecuteIdempotencyLedger::default();
    let request = request("req-dsr-cascade", "idem-dsr-cascade");

    let first = execute_dsr_cascade_from_api(
        &verified_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        request.clone(),
    )
    .expect("DSR cascade completes");
    let second = execute_dsr_cascade_from_api(
        &verified_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        request,
    )
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
    let mut cascade_request = request("req-dsr-multistore", "idem-dsr-multistore");
    cascade_request
        .body
        .targets
        .push(target("dispatch-002", "drive/object/2", "record_delete"));

    let response = execute_dsr_cascade_from_api(
        &verified_principal(),
        &allow_all_provider(),
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
        &verified_principal(),
        &allow_all_provider(),
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
    let mut drifted = request("req-dsr-drift", "idem-dsr-drift");
    drifted.body.dsr_id = "dsr-other".to_string();

    let error = execute_dsr_cascade_from_api(
        &verified_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        drifted,
    )
    .expect_err("path/body DSR id drift rejected");
    assert_eq!(error.status_code(), 400);
    assert!(matches!(error, PlatformDsrApiError::DsrIdMismatch { .. }));
    assert!(ledger.is_empty());
    assert!(directory.is_empty());

    let mut tenant_drift = request("req-dsr-tenant", "idem-dsr-tenant");
    tenant_drift.boundary.tenant_id = "ten-other".to_string();
    let error = execute_dsr_cascade_from_api(
        &verified_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        tenant_drift,
    )
    .expect_err("tenant drift rejected");
    assert_eq!(error.status_code(), 403);
    assert!(matches!(error, PlatformDsrApiError::TenantMismatch { .. }));
    assert!(directory.is_empty());
}

#[test]
fn dsr_cascade_execute_rejects_reused_idempotency_key_and_completed_conflict() {
    let mut directory = PlatformDsrCascadeDirectory::default();
    let mut ledger = PlatformDsrCascadeExecuteIdempotencyLedger::default();

    let first = request("req-dsr-idem", "idem-dsr-idem");
    execute_dsr_cascade_from_api(
        &verified_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        first.clone(),
    )
    .expect("initial DSR cascade succeeds");
    let mut drift = first;
    drift.body.aggregate_proof_hash =
        "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string();
    assert_eq!(
        execute_dsr_cascade_from_api(
            &verified_principal(),
            &allow_all_provider(),
            &mut directory,
            &mut ledger,
            drift,
        ),
        Err(PlatformDsrApiError::IdempotencyKeyReused {
            idempotency_key: "idem-dsr-idem".to_string(),
        })
    );

    let duplicate = execute_dsr_cascade_from_api(
        &verified_principal(),
        &allow_all_provider(),
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

    let mut invalid_action = request("req-dsr-action", "idem-dsr-action");
    invalid_action.body.action = "delete_everything".to_string();
    let error = execute_dsr_cascade_from_api(
        &verified_principal(),
        &allow_all_provider(),
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
        &verified_principal(),
        &allow_all_provider(),
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
        &verified_principal(),
        &allow_all_provider(),
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
        &verified_principal(),
        &allow_all_provider(),
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
fn cascade_authorization_denied_error_shape_is_stable() {
    let error = PlatformDsrApiError::CascadeAuthorizationDenied {
        surface: PLATFORM_DSR_CASCADE_EXECUTE_SURFACE.to_string(),
    };
    let response = error.error_response("req-dsr-denied");
    assert_eq!(error.status_code(), 403);
    assert_eq!(
        response.error.code,
        "PLATFORM_DSR_CASCADE_AUTHORIZATION_DENIED"
    );
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
