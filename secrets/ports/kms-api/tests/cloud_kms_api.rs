// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use data_boundary_kernel::DataClass;
use network_residency::ResidencyClass;
use secrets_kms_api::authz::{
    CallerCredential, ConfiguredBearerPrincipalVerifier, KmsCryptoAuthorizationError,
    KmsCryptoAuthorizer, KmsCryptoAuthzProvider, KmsCryptoResource, PrincipalVerifier,
    VerifiedKmsPrincipal,
};
use secrets_kms_api::{
    CLOUD_KMS_DECRYPT_SURFACE, CLOUD_KMS_DEFAULT_PUBLIC_API_VERSION, CLOUD_KMS_ENCRYPT_SURFACE,
    CLOUD_KMS_SUPPORTED_PUBLIC_API_VERSIONS, CloudKmsApiAuthorizationCorrelation,
    CloudKmsApiBoundaryContext, CloudKmsApiError, CloudKmsApiPrincipal, CloudKmsCryptoApiStatus,
    CloudKmsCryptoIdempotencyLedger, CloudKmsDecryptApiRequest, CloudKmsDecryptRequest,
    CloudKmsEncryptApiRequest, CloudKmsEncryptRequest, authorize_cloud_kms_decrypt_from_api,
    authorize_cloud_kms_encrypt_from_api,
};
use secrets_kms_domain::{
    CloudKmsDirectory, CloudKmsError, HsmValidation, KmsKeyCreate, KmsKeyOrigin, KmsKeyState,
    KmsKeyUsage, KmsRepo,
};

const TEST_BEARER_SECRET: &str = "test-kms-break-glass-secret";

/// Mint a [`VerifiedKmsPrincipal`] for `sp_storage` / `ten_alpha` by running the
/// REAL [`ConfiguredBearerPrincipalVerifier`] path — the same path production
/// uses. This proves an external crate CANNOT forge a `VerifiedKmsPrincipal` by
/// struct literal; it must run a verifier.
fn verified_storage_principal() -> VerifiedKmsPrincipal {
    ConfiguredBearerPrincipalVerifier::new(TEST_BEARER_SECRET, "sp_storage", "ten_alpha")
        .expect("bearer verifier constructs with a non-empty secret")
        .verify_principal(&CallerCredential {
            authorization: Some(format!("Bearer {TEST_BEARER_SECRET}")),
            claimed_principal_id: "sp_storage".to_string(),
            claimed_tenant_id: "ten_alpha".to_string(),
        })
        .expect("a valid bearer verifies into a principal")
}

/// A PDP authorizer that allows every crypto op (so non-authz tests exercise the
/// request-shape and kernel paths exactly as before, but through the new gate).
struct AllowAllAuthorizer;
impl KmsCryptoAuthorizer for AllowAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedKmsPrincipal,
        _resource: &KmsCryptoResource,
    ) -> Result<(), KmsCryptoAuthorizationError> {
        Ok(())
    }
}

/// A PDP authorizer that denies every crypto op.
struct DenyAllAuthorizer;
impl KmsCryptoAuthorizer for DenyAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedKmsPrincipal,
        _resource: &KmsCryptoResource,
    ) -> Result<(), KmsCryptoAuthorizationError> {
        Err(KmsCryptoAuthorizationError::Denied)
    }
}

/// A PDP authorizer that panics (proves the fail-closed catch maps panic → 403,
/// not a 500/process-crash, under the test `panic = unwind` profile).
struct PanicAuthorizer;
impl KmsCryptoAuthorizer for PanicAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedKmsPrincipal,
        _resource: &KmsCryptoResource,
    ) -> Result<(), KmsCryptoAuthorizationError> {
        panic!("PDP adapter panicked");
    }
}

/// A PDP authorizer that allows ONLY when the resource tenant equals the
/// verified principal tenant — i.e. it would otherwise allow same-tenant access,
/// so a cross-tenant 403 proves the resource tenant is parsed from the target
/// key path, not caller input.
struct SameTenantAuthorizer;
impl KmsCryptoAuthorizer for SameTenantAuthorizer {
    fn ensure_authorized(
        &self,
        principal: &VerifiedKmsPrincipal,
        resource: &KmsCryptoResource,
    ) -> Result<(), KmsCryptoAuthorizationError> {
        if resource.tenant_id == principal.tenant_id() {
            Ok(())
        } else {
            Err(KmsCryptoAuthorizationError::Denied)
        }
    }
}

fn allow_all_provider() -> KmsCryptoAuthzProvider {
    KmsCryptoAuthzProvider::new(
        Arc::new(
            ConfiguredBearerPrincipalVerifier::new(TEST_BEARER_SECRET, "sp_storage", "ten_alpha")
                .expect("verifier constructs"),
        ),
        Arc::new(AllowAllAuthorizer),
    )
}

fn provider_with(authorizer: Arc<dyn KmsCryptoAuthorizer>) -> KmsCryptoAuthzProvider {
    KmsCryptoAuthzProvider::new(
        Arc::new(
            ConfiguredBearerPrincipalVerifier::new(TEST_BEARER_SECRET, "sp_storage", "ten_alpha")
                .expect("verifier constructs"),
        ),
        authorizer,
    )
}

fn boundary_for(request_id: &str, idempotency_key: &str) -> CloudKmsApiBoundaryContext {
    CloudKmsApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_alpha".to_string(),
        region: "region-home".to_string(),
        cell_id: "cell-region-home-a-001".to_string(),
        idempotency_key: idempotency_key.to_string(),
        oyatie_version: CLOUD_KMS_DEFAULT_PUBLIC_API_VERSION.to_string(),
    }
}

fn principal_for(principal_id: &str) -> CloudKmsApiPrincipal {
    CloudKmsApiPrincipal {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn correlation_for(principal_id: &str) -> CloudKmsApiAuthorizationCorrelation {
    CloudKmsApiAuthorizationCorrelation {
        decision_id: format!("authz_decision_{principal_id}"),
    }
}

fn key_create() -> KmsKeyCreate {
    KmsKeyCreate {
        resource_id: "oya:cloud:region-home:ten_alpha:kms-key:object-key".to_string(),
        key_id: "kms/region-home/ten_alpha/object-key".to_string(),
        tenant_id: "ten_alpha".to_string(),
        region: "region-home".to_string(),
        cell_id: "cell-region-home-a-001".to_string(),
        hsm_partition_ref: "hsm/region-home/cell-region-home-a-001".to_string(),
        origin: KmsKeyOrigin::OyatieManaged,
        usage: KmsKeyUsage::EncryptDecrypt,
        hsm_validation: HsmValidation::PackEnhancedFips1403Level3,
        residency: ResidencyClass::StrictHomeRegion,
        data_class: DataClass::PiiIdentifying,
        state: KmsKeyState::Enabled,
        rotation_period_days: Some(90),
        created_at_epoch_seconds: 1_700_000_000,
    }
}

fn directory_with_key() -> CloudKmsDirectory {
    let mut directory = CloudKmsDirectory::default();
    directory
        .create_key(key_create())
        .expect("KMS key registers");
    directory
}

fn encrypt_api_request(request_id: &str, idempotency_key: &str) -> CloudKmsEncryptApiRequest {
    CloudKmsEncryptApiRequest {
        path_key_id: "kms/region-home/ten_alpha/object-key".to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_storage"),
        correlation: correlation_for("sp_storage"),
        body: CloudKmsEncryptRequest {
            event_id: "kmsuse_encrypt_001".to_string(),
            key_id: "kms/region-home/ten_alpha/object-key".to_string(),
            tenant_id: "ten_alpha".to_string(),
            plaintext_ref: "matref/ten_alpha/object/001".to_string(),
            ciphertext_ref: "ct/ten_alpha/object/001".to_string(),
            data_class: "PII_IDENTIFYING".to_string(),
            purpose: "cloud_object_storage".to_string(),
            actor: "sp_storage".to_string(),
            aad_fingerprint: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .to_string(),
            requested_at_epoch_seconds: 1_700_000_010,
        },
    }
}

fn decrypt_api_request(request_id: &str, idempotency_key: &str) -> CloudKmsDecryptApiRequest {
    CloudKmsDecryptApiRequest {
        path_key_id: "kms/region-home/ten_alpha/object-key".to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_storage"),
        correlation: correlation_for("sp_storage"),
        body: CloudKmsDecryptRequest {
            event_id: "kmsuse_decrypt_001".to_string(),
            key_id: "kms/region-home/ten_alpha/object-key".to_string(),
            tenant_id: "ten_alpha".to_string(),
            ciphertext_ref: "ct/ten_alpha/object/001".to_string(),
            data_class: "PII_IDENTIFYING".to_string(),
            purpose: "cloud_object_storage".to_string(),
            actor: "sp_storage".to_string(),
            requested_at_epoch_seconds: 1_700_000_020,
        },
    }
}

#[test]
fn decrypt_api_rejects_malformed_path_key_before_pdp_ledger_and_receipt() {
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let malformed_key_id = "kms/region-home/ten_alpha/object-key/extra";
    let mut request = decrypt_api_request("req-kms-malformed-key", "idem-kms-malformed-key");
    request.path_key_id = malformed_key_id.to_string();
    request.body.key_id = malformed_key_id.to_string();

    let error = authorize_cloud_kms_decrypt_from_api(
        &verified_storage_principal(),
        &provider_with(Arc::new(DenyAllAuthorizer)),
        &mut directory,
        &mut ledger,
        request,
    )
    .expect_err("malformed key path is rejected before PDP authorization");

    assert_eq!(
        error,
        CloudKmsApiError::MalformedPathKeyId {
            path_key_id: malformed_key_id.to_string(),
        }
    );
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);
}

#[test]
fn encrypt_api_rejects_path_body_key_drift_before_receipt_mutation() {
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let mut request = encrypt_api_request("req-kms-key-drift", "idem-kms-key-drift");
    request.body.key_id = "kms/region-home/ten_alpha/other-key".to_string();

    let error = authorize_cloud_kms_encrypt_from_api(
        &verified_storage_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        request,
    )
    .expect_err("path/body key drift is rejected");

    assert_eq!(
        error,
        CloudKmsApiError::KeyIdMismatch {
            path_key_id: "kms/region-home/ten_alpha/object-key".to_string(),
            body_key_id: "kms/region-home/ten_alpha/other-key".to_string(),
        }
    );
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);
}

#[test]
fn encrypt_api_rejects_required_header_and_tenant_drift_before_ledger() {
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let mut empty_request = encrypt_api_request(" ", "idem-kms-empty-header");
    assert_eq!(
        authorize_cloud_kms_encrypt_from_api(
            &verified_storage_principal(),
            &allow_all_provider(),
            &mut directory,
            &mut ledger,
            empty_request.clone()
        ),
        Err(CloudKmsApiError::EmptyRequestId)
    );

    empty_request.boundary.request_id = "req-kms-tenant-drift".to_string();
    empty_request.boundary.tenant_id = "ten_other".to_string();
    assert_eq!(
        authorize_cloud_kms_encrypt_from_api(
            &verified_storage_principal(),
            &allow_all_provider(),
            &mut directory,
            &mut ledger,
            empty_request
        ),
        Err(CloudKmsApiError::TenantMismatch {
            header_tenant_id: "ten_other".to_string(),
            principal_tenant_id: "ten_alpha".to_string(),
            body_tenant_id: "ten_alpha".to_string(),
        })
    );
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);
}

#[test]
fn kms_api_rejects_missing_or_unsupported_oyatie_version_before_ledger() {
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let mut missing_encrypt =
        encrypt_api_request("req-kms-version-missing", "idem-kms-version-missing");
    missing_encrypt.boundary.oyatie_version = " ".to_string();

    assert_eq!(
        authorize_cloud_kms_encrypt_from_api(
            &verified_storage_principal(),
            &allow_all_provider(),
            &mut directory,
            &mut ledger,
            missing_encrypt
        ),
        Err(CloudKmsApiError::MissingPublicApiVersion)
    );
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);

    let mut unsupported_encrypt = encrypt_api_request(
        "req-kms-version-unsupported",
        "idem-kms-version-unsupported",
    );
    unsupported_encrypt.boundary.oyatie_version = "2026-01-01".to_string();

    assert_eq!(
        authorize_cloud_kms_encrypt_from_api(
            &verified_storage_principal(),
            &allow_all_provider(),
            &mut directory,
            &mut ledger,
            unsupported_encrypt
        ),
        Err(CloudKmsApiError::UnsupportedPublicApiVersion {
            oyatie_version: "2026-01-01".to_string(),
        })
    );
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);

    let mut missing_decrypt = decrypt_api_request(
        "req-kms-decrypt-version-missing",
        "idem-kms-decrypt-version-missing",
    );
    missing_decrypt.boundary.oyatie_version = "\t".to_string();

    assert_eq!(
        authorize_cloud_kms_decrypt_from_api(
            &verified_storage_principal(),
            &allow_all_provider(),
            &mut directory,
            &mut ledger,
            missing_decrypt
        ),
        Err(CloudKmsApiError::MissingPublicApiVersion)
    );
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);

    let mut unsupported_decrypt = decrypt_api_request(
        "req-kms-decrypt-version-unsupported",
        "idem-kms-decrypt-version-unsupported",
    );
    unsupported_decrypt.boundary.oyatie_version = "not-a-date".to_string();

    assert_eq!(
        authorize_cloud_kms_decrypt_from_api(
            &verified_storage_principal(),
            &allow_all_provider(),
            &mut directory,
            &mut ledger,
            unsupported_decrypt
        ),
        Err(CloudKmsApiError::UnsupportedPublicApiVersion {
            oyatie_version: "not-a-date".to_string(),
        })
    );
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);
}

#[test]
fn kms_api_rejects_missing_or_drifted_placement_before_ledger() {
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();

    let mut missing_region =
        encrypt_api_request("req-kms-region-missing", "idem-kms-region-missing");
    missing_region.boundary.region = " ".to_string();
    assert_eq!(
        authorize_cloud_kms_encrypt_from_api(
            &verified_storage_principal(),
            &allow_all_provider(),
            &mut directory,
            &mut ledger,
            missing_region
        ),
        Err(CloudKmsApiError::EmptyRegionHeader)
    );
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);

    let mut missing_cell = decrypt_api_request("req-kms-cell-missing", "idem-kms-cell-missing");
    missing_cell.boundary.cell_id = "\t".to_string();
    assert_eq!(
        authorize_cloud_kms_decrypt_from_api(
            &verified_storage_principal(),
            &allow_all_provider(),
            &mut directory,
            &mut ledger,
            missing_cell
        ),
        Err(CloudKmsApiError::EmptyCellHeader)
    );
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);

    let mut region_drift = encrypt_api_request("req-kms-region-drift", "idem-kms-region-drift");
    region_drift.boundary.region = "region-away".to_string();
    region_drift.boundary.cell_id = "cell-region-away-a-001".to_string();
    assert_eq!(
        authorize_cloud_kms_encrypt_from_api(
            &verified_storage_principal(),
            &allow_all_provider(),
            &mut directory,
            &mut ledger,
            region_drift
        ),
        Err(CloudKmsApiError::Kms(CloudKmsError::ResourceRegionMismatch))
    );
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);

    let mut cell_drift = decrypt_api_request("req-kms-cell-drift", "idem-kms-cell-drift");
    cell_drift.boundary.cell_id = "cell-region-home-b-001".to_string();
    assert_eq!(
        authorize_cloud_kms_decrypt_from_api(
            &verified_storage_principal(),
            &allow_all_provider(),
            &mut directory,
            &mut ledger,
            cell_drift
        ),
        Err(CloudKmsApiError::Kms(CloudKmsError::CellPlacementMismatch))
    );
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);
}

#[test]
fn kms_api_accepts_manifest_public_versions_and_keys_idempotency_by_version() {
    for (index, oyatie_version) in CLOUD_KMS_SUPPORTED_PUBLIC_API_VERSIONS
        .iter()
        .copied()
        .enumerate()
    {
        let mut directory = directory_with_key();
        let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
        let mut request = encrypt_api_request(
            &format!("req-kms-version-supported-{index}"),
            &format!("idem-kms-version-supported-{index}"),
        );
        request.boundary.oyatie_version = oyatie_version.to_string();

        let response = authorize_cloud_kms_encrypt_from_api(
            &verified_storage_principal(),
            &allow_all_provider(),
            &mut directory,
            &mut ledger,
            request,
        )
        .expect("manifest-declared Cloud KMS public API version is accepted");

        assert_eq!(response.data.key_id, "kms/region-home/ten_alpha/object-key");
        assert_eq!(ledger.len(), 1);
        assert_eq!(directory.receipts().count(), 1);
    }

    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let request = encrypt_api_request("req-kms-version-default", "idem-kms-version-key");
    authorize_cloud_kms_encrypt_from_api(
        &verified_storage_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        request.clone(),
    )
    .expect("default public API version succeeds");

    let mut version_drifted = request;
    version_drifted.boundary.oyatie_version = "2026-02-21".to_string();
    assert_eq!(
        authorize_cloud_kms_encrypt_from_api(
            &verified_storage_principal(),
            &allow_all_provider(),
            &mut directory,
            &mut ledger,
            version_drifted
        ),
        Err(CloudKmsApiError::IdempotencyKeyReused {
            idempotency_key: "idem-kms-version-key".to_string(),
        })
    );
    assert_eq!(ledger.len(), 1);
    assert_eq!(directory.receipts().count(), 1);
}

#[test]
fn encrypt_api_pdp_deny_returns_403_before_ledger() {
    // GREEN: a verified, same-tenant principal whose PDP decision is DENY must
    // be rejected 403 with no state mutation. Proves the server-side PDP seam
    // (DenyAllAuthorizer) — NOT the retired caller-supplied authorization blob.
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let request = encrypt_api_request("req-kms-authz-deny", "idem-kms-authz-deny");
    let deny_provider = provider_with(Arc::new(DenyAllAuthorizer));

    let error = authorize_cloud_kms_encrypt_from_api(
        &verified_storage_principal(),
        &deny_provider,
        &mut directory,
        &mut ledger,
        request,
    )
    .expect_err("PDP deny rejects the crypto op");

    assert_eq!(
        error,
        CloudKmsApiError::CryptoAuthorizationDenied {
            surface: CLOUD_KMS_ENCRYPT_SURFACE.to_string(),
        }
    );
    assert_eq!(error.crypto_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);
}

#[test]
fn decrypt_api_rejects_actor_drift_before_receipt_mutation() {
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let mut request = decrypt_api_request("req-kms-actor-drift", "idem-kms-actor-drift");
    request.body.actor = "usr_alice".to_string();

    let error = authorize_cloud_kms_decrypt_from_api(
        &verified_storage_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        request,
    )
    .expect_err("body actor must match authenticated principal");

    assert_eq!(
        error,
        CloudKmsApiError::PrincipalMismatch {
            principal_tenant_id: "ten_alpha".to_string(),
            principal_id: "sp_storage".to_string(),
            body_tenant_id: "ten_alpha".to_string(),
            actor: "usr_alice".to_string(),
        }
    );
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);
}

#[test]
fn encrypt_api_authorizes_once_and_replays_same_idempotent_receipt() {
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let request = encrypt_api_request("req-kms-encrypt", "idem-kms-encrypt");

    let first = authorize_cloud_kms_encrypt_from_api(
        &verified_storage_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        request.clone(),
    )
    .expect("encrypt authorization succeeds");
    let second = authorize_cloud_kms_encrypt_from_api(
        &verified_storage_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        request,
    )
    .expect("same encrypt idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(directory.receipts().count(), 1);
    assert_eq!(first.data.event_id, "kmsuse_encrypt_001");
    assert_eq!(first.data.operation, "encrypt");
    assert_eq!(
        first.data.material_ref.as_deref(),
        Some("matref/ten_alpha/object/001")
    );
    assert_eq!(first.data.key_version, 1);
    assert_eq!(first.metadata.request_id, "req-kms-encrypt");
    assert_eq!(CLOUD_KMS_ENCRYPT_SURFACE, "cloud.kms.encrypt");
    assert_eq!(CloudKmsCryptoApiStatus::Ok.code(), 200);
    assert_eq!(CloudKmsCryptoApiStatus::BadRequest.code(), 400);
    assert_eq!(CloudKmsCryptoApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudKmsCryptoApiStatus::NotFound.code(), 404);
    assert_eq!(CloudKmsCryptoApiStatus::Conflict.code(), 409);
    assert_eq!(CloudKmsCryptoApiStatus::UnprocessableEntity.code(), 422);
}

#[test]
fn decrypt_api_authorizes_once_and_replays_same_idempotent_receipt() {
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let request = decrypt_api_request("req-kms-decrypt", "idem-kms-decrypt");

    let first = authorize_cloud_kms_decrypt_from_api(
        &verified_storage_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        request.clone(),
    )
    .expect("decrypt authorization succeeds");
    let second = authorize_cloud_kms_decrypt_from_api(
        &verified_storage_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        request,
    )
    .expect("same decrypt idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(directory.receipts().count(), 1);
    assert_eq!(first.data.event_id, "kmsuse_decrypt_001");
    assert_eq!(first.data.operation, "decrypt");
    assert_eq!(first.data.material_ref, None);
    assert_eq!(first.metadata.request_id, "req-kms-decrypt");
    assert_eq!(CLOUD_KMS_DECRYPT_SURFACE, "cloud.kms.decrypt");
}

// ----------------------------------------------------------------------------
// AUTH-005 / C5 fail-closed crypto-gate tests (ADR-0573).
//
// Each test below FAILS if the fail-closed seam is removed: the crypto op must
// be unreachable without a verified principal (401) AND a passing PDP decision
// (403 on deny/refuse/cross-tenant). The forged caller-supplied authorization
// blob no longer exists and confers no authority.
// ----------------------------------------------------------------------------

/// Map a verification refusal to the boundary error the edge returns to the
/// caller. Models the edge: a request whose credential does not verify is
/// rejected 401 BEFORE any boundary function (which requires a
/// `VerifiedKmsPrincipal`) is ever called.
fn edge_verify_or_unauthenticated(
    provider: &KmsCryptoAuthzProvider,
    credential: &CallerCredential,
) -> Result<VerifiedKmsPrincipal, CloudKmsApiError> {
    provider
        .verify_principal(credential)
        .map_err(|_| CloudKmsApiError::PrincipalUnauthenticated)
}

#[test]
fn decrypt_forged_authz_blob_without_verified_credential_returns_401() {
    // RED-if-removed: a caller forges a consistent principal + correlation blob
    // but presents NO credential. The edge verification fails → 401. The
    // boundary function is never reached, so no crypto op occurs.
    let provider = allow_all_provider();
    let forged_no_credential = CallerCredential {
        authorization: None,
        claimed_principal_id: "sp_storage".to_string(),
        claimed_tenant_id: "ten_alpha".to_string(),
    };

    let error = edge_verify_or_unauthenticated(&provider, &forged_no_credential)
        .expect_err("absent credential must not authenticate");

    assert_eq!(error, CloudKmsApiError::PrincipalUnauthenticated);
    assert_eq!(error.crypto_status_code(), 401);
}

#[test]
fn decrypt_wrong_bearer_credential_returns_401() {
    // RED-if-removed: a present-but-WRONG bearer must not verify → 401.
    let provider = allow_all_provider();
    let wrong_bearer = CallerCredential {
        authorization: Some("Bearer not-the-secret".to_string()),
        claimed_principal_id: "sp_storage".to_string(),
        claimed_tenant_id: "ten_alpha".to_string(),
    };

    let error = edge_verify_or_unauthenticated(&provider, &wrong_bearer)
        .expect_err("wrong bearer must not authenticate");

    assert_eq!(error, CloudKmsApiError::PrincipalUnauthenticated);
    assert_eq!(error.crypto_status_code(), 401);
}

#[test]
fn decrypt_verified_principal_with_mismatched_claimed_tenant_returns_403() {
    // RED-if-removed: a VERIFIED principal (ten_alpha) whose request asserts a
    // different tenant in the body/principal is rejected 403 — the verified
    // identity is authoritative; caller-supplied fields cannot override it.
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let mut request = decrypt_api_request("req-kms-tenant-sub", "idem-kms-tenant-sub");
    // Caller substitutes a foreign tenant in the asserted principal + body.
    request.boundary.tenant_id = "ten_beta".to_string();
    request.principal.tenant_id = "ten_beta".to_string();
    request.body.tenant_id = "ten_beta".to_string();

    let error = authorize_cloud_kms_decrypt_from_api(
        &verified_storage_principal(), // verified tenant = ten_alpha
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        request,
    )
    .expect_err("verified tenant overrides caller-asserted tenant");

    assert_eq!(
        error,
        CloudKmsApiError::VerifiedTenantMismatch {
            verified_tenant_id: "ten_alpha".to_string(),
            claimed_tenant_id: "ten_beta".to_string(),
        }
    );
    assert_eq!(error.crypto_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);
}

#[test]
fn decrypt_cross_tenant_key_denied_by_pdp_returns_403() {
    // RED-if-removed (BLAST-RADIUS proof): a verified ten_alpha principal targets
    // a key owned by ten_beta. The PDP resource tenant is parsed from the TARGET
    // KEY path ("ten_beta"), not from the verified principal's tenant ("ten_alpha").
    // A SameTenantAuthorizer — which allows principal.tenant == resource.tenant —
    // DENIES because "ten_alpha" != "ten_beta". Denial fires at the AUTHZ layer,
    // before the kernel. This proves cross-tenant key access is blocked at the PDP.
    //
    // Register a ten_beta key in the directory.
    let mut directory = directory_with_key();
    directory
        .create_key(KmsKeyCreate {
            resource_id: "oya:cloud:region-home:ten_beta:kms-key:object-key".to_string(),
            key_id: "kms/region-home/ten_beta/object-key".to_string(),
            tenant_id: "ten_beta".to_string(),
            region: "region-home".to_string(),
            cell_id: "cell-region-home-a-001".to_string(),
            hsm_partition_ref: "hsm/region-home/cell-region-home-a-001".to_string(),
            origin: KmsKeyOrigin::OyatieManaged,
            usage: KmsKeyUsage::EncryptDecrypt,
            hsm_validation: HsmValidation::PackEnhancedFips1403Level3,
            residency: ResidencyClass::StrictHomeRegion,
            data_class: DataClass::PiiIdentifying,
            state: KmsKeyState::Enabled,
            rotation_period_days: Some(90),
            created_at_epoch_seconds: 1_700_000_000,
        })
        .expect("ten_beta key registers");
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();

    // The verified ten_alpha principal targets ten_beta's key. The body tenant is
    // ten_alpha so the verified-principal cross-check passes (the IDOR is in the
    // key target, not the tenant claim). The SameTenantAuthorizer ALLOWS when
    // resource.tenant_id == principal.tenant_id(), but now resource.tenant_id is
    // parsed from the TARGET key path ("ten_beta"), NOT the verified caller's
    // tenant ("ten_alpha"). So the PDP sees "may ten_alpha access ten_beta's key?"
    // and DENIES — the rejection happens at the AUTHZ layer, before the kernel.
    // RED-if-reverted: reverting ensure_crypto_authorized to use verified.tenant_id()
    // makes this test fail (the PDP would allow "ten_alpha/ten_alpha" and the
    // kernel ResourceTenantMismatch would fire instead).
    let request = CloudKmsDecryptApiRequest {
        path_key_id: "kms/region-home/ten_beta/object-key".to_string(),
        boundary: boundary_for("req-kms-idor", "idem-kms-idor"),
        principal: principal_for("sp_storage"),
        correlation: correlation_for("sp_storage"),
        body: CloudKmsDecryptRequest {
            event_id: "kmsuse_idor_001".to_string(),
            key_id: "kms/region-home/ten_beta/object-key".to_string(),
            tenant_id: "ten_alpha".to_string(),
            ciphertext_ref: "ct/ten_alpha/object/001".to_string(),
            data_class: "PII_IDENTIFYING".to_string(),
            purpose: "cloud_object_storage".to_string(),
            actor: "sp_storage".to_string(),
            requested_at_epoch_seconds: 1_700_000_020,
        },
    };

    let error = authorize_cloud_kms_decrypt_from_api(
        &verified_storage_principal(), // verified tenant = ten_alpha
        &provider_with(Arc::new(SameTenantAuthorizer)),
        &mut directory,
        &mut ledger,
        request,
    )
    .expect_err("cross-tenant key target must be denied at PDP layer");

    // The PDP (SameTenantAuthorizer) sees resource.tenant_id == "ten_beta" while
    // principal.tenant_id() == "ten_alpha" — DENIES. The boundary converts all
    // authz denials to CryptoAuthorizationDenied (403). The kernel is never reached.
    assert_eq!(error.crypto_status_code(), 403);
    assert_eq!(
        error,
        CloudKmsApiError::CryptoAuthorizationDenied {
            surface: "cloud.kms.decrypt".to_string(),
        }
    );
    // No receipt was produced — no plaintext-revealing decrypt occurred.
    assert_eq!(directory.receipts().count(), 0);
}

#[test]
fn decrypt_pdp_deny_returns_403_before_ledger() {
    // RED-if-removed: a verified, same-tenant, same-key principal whose PDP
    // decision is DENY is rejected 403 with no state mutation.
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let request = decrypt_api_request("req-kms-pdp-deny", "idem-kms-pdp-deny");

    let error = authorize_cloud_kms_decrypt_from_api(
        &verified_storage_principal(),
        &provider_with(Arc::new(DenyAllAuthorizer)),
        &mut directory,
        &mut ledger,
        request,
    )
    .expect_err("PDP deny rejects decrypt");

    assert_eq!(
        error,
        CloudKmsApiError::CryptoAuthorizationDenied {
            surface: CLOUD_KMS_DECRYPT_SURFACE.to_string(),
        }
    );
    assert_eq!(error.crypto_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);
}

#[test]
fn decrypt_pdp_fault_returns_403_not_500() {
    // RED-if-removed: a panicking PDP adapter maps to Refused → 403 (fail-closed),
    // NOT a 500/process-crash, under the test `panic = unwind` profile.
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let request = decrypt_api_request("req-kms-pdp-fault", "idem-kms-pdp-fault");

    let error = authorize_cloud_kms_decrypt_from_api(
        &verified_storage_principal(),
        &provider_with(Arc::new(PanicAuthorizer)),
        &mut directory,
        &mut ledger,
        request,
    )
    .expect_err("PDP fault is treated as deny");

    assert_eq!(
        error,
        CloudKmsApiError::CryptoAuthorizationDenied {
            surface: CLOUD_KMS_DECRYPT_SURFACE.to_string(),
        }
    );
    assert_eq!(error.crypto_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);
}

#[test]
fn decrypt_happy_path_verified_and_authorized_returns_ok() {
    // GREEN: a verified principal, same tenant + key, allowed by the PDP, with a
    // valid bearer, produces a receipt whose actor is the verified principal.
    let provider = allow_all_provider();
    let verified = edge_verify_or_unauthenticated(
        &provider,
        &CallerCredential {
            authorization: Some(format!("Bearer {TEST_BEARER_SECRET}")),
            claimed_principal_id: "sp_storage".to_string(),
            claimed_tenant_id: "ten_alpha".to_string(),
        },
    )
    .expect("valid bearer verifies");

    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let request = decrypt_api_request("req-kms-ok", "idem-kms-ok");

    let response = authorize_cloud_kms_decrypt_from_api(
        &verified,
        &provider,
        &mut directory,
        &mut ledger,
        request,
    )
    .expect("verified + authorized decrypt succeeds");

    // Audit/receipt actor reflects the VERIFIED principal, not a caller header.
    assert_eq!(response.data.actor, "sp_storage");
    assert_eq!(response.data.tenant_id, "ten_alpha");
    assert_eq!(response.data.operation, "decrypt");
    assert_eq!(directory.receipts().count(), 1);
    assert_eq!(CloudKmsCryptoApiStatus::Unauthorized.code(), 401);
}

#[test]
fn encrypt_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let request = encrypt_api_request("req-kms-encrypt", "idem-kms-encrypt");
    authorize_cloud_kms_encrypt_from_api(
        &verified_storage_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        request.clone(),
    )
    .expect("initial encrypt succeeds");

    let mut drifted = request;
    drifted.body.ciphertext_ref = "ct/ten_alpha/object/002".to_string();
    assert_eq!(
        authorize_cloud_kms_encrypt_from_api(
            &verified_storage_principal(),
            &allow_all_provider(),
            &mut directory,
            &mut ledger,
            drifted
        ),
        Err(CloudKmsApiError::IdempotencyKeyReused {
            idempotency_key: "idem-kms-encrypt".to_string(),
        })
    );
    assert_eq!(directory.receipts().count(), 1);
}

#[test]
fn encrypt_api_maps_unknown_key_and_duplicate_event() {
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let mut unknown_key = encrypt_api_request("req-kms-missing", "idem-kms-missing");
    unknown_key.path_key_id = "kms/region-home/ten_alpha/missing-key".to_string();
    unknown_key.body.key_id = "kms/region-home/ten_alpha/missing-key".to_string();
    let missing = authorize_cloud_kms_encrypt_from_api(
        &verified_storage_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        unknown_key,
    )
    .expect_err("unknown key maps to not found");
    assert_eq!(missing.crypto_status_code(), 404);
    assert_eq!(missing, CloudKmsApiError::Kms(CloudKmsError::UnknownKey));

    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    authorize_cloud_kms_encrypt_from_api(
        &verified_storage_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        encrypt_api_request("req-kms-dup-1", "idem-kms-dup-1"),
    )
    .expect("first event succeeds");
    let duplicate = authorize_cloud_kms_encrypt_from_api(
        &verified_storage_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        encrypt_api_request("req-kms-dup-2", "idem-kms-dup-2"),
    )
    .expect_err("same event id through new idempotency key is a conflict");
    assert_eq!(duplicate.crypto_status_code(), 409);
    assert_eq!(
        duplicate,
        CloudKmsApiError::Kms(CloudKmsError::DuplicateUseEvent)
    );
}

#[test]
fn encrypt_api_maps_invalid_aad_issue_without_generic_masking() {
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let mut request = encrypt_api_request("req-kms-aad", "idem-kms-aad");
    request.body.event_id = "kmsuse_bad_aad".to_string();
    request.body.aad_fingerprint = "not-a-digest".to_string();

    let error = authorize_cloud_kms_encrypt_from_api(
        &verified_storage_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        request,
    )
    .expect_err("kernel rejects malformed AAD fingerprint");

    assert_eq!(
        error,
        CloudKmsApiError::Kms(CloudKmsError::InvalidAadFingerprint)
    );
    assert_eq!(error.crypto_status_code(), 400);
    assert_eq!(
        error
            .error_response("req-kms-aad")
            .error
            .details
            .first()
            .expect("cloud KMS error detail")
            .issue,
        "aad_fingerprint must be a 64-character hexadecimal digest"
    );
}

#[test]
fn encrypt_api_rejects_unknown_data_class_label_before_receipt_mutation() {
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let mut request = encrypt_api_request("req-kms-class", "idem-kms-class");
    request.body.data_class = "SECRET".to_string();

    let error = authorize_cloud_kms_encrypt_from_api(
        &verified_storage_principal(),
        &allow_all_provider(),
        &mut directory,
        &mut ledger,
        request,
    )
    .expect_err("operational markers are not KMS API data classes");

    assert_eq!(
        error,
        CloudKmsApiError::InvalidDataClassLabel {
            data_class: "SECRET".to_string(),
        }
    );
    assert_eq!(error.crypto_status_code(), 400);
    assert_eq!(directory.receipts().count(), 0);
}
