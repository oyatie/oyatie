// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_cloud_kms_api::{
    CLOUD_KMS_DECRYPT_SURFACE, CLOUD_KMS_ENCRYPT_SURFACE, CloudKmsApiAuthorization,
    CloudKmsApiBoundaryContext, CloudKmsApiError, CloudKmsApiPrincipal, CloudKmsCryptoApiStatus,
    CloudKmsCryptoIdempotencyLedger, CloudKmsDecryptApiRequest, CloudKmsDecryptRequest,
    CloudKmsEncryptApiRequest, CloudKmsEncryptRequest, authorize_cloud_kms_decrypt_from_api,
    authorize_cloud_kms_encrypt_from_api,
};
use oya_cloud_kms_domain::{
    CloudKmsDirectory, CloudKmsError, HsmValidation, KmsKeyCreate, KmsKeyOrigin, KmsKeyState,
    KmsKeyUsage, KmsRepo,
};
use oya_data_boundary_kernel::DataClass;
use oya_residency_domain::ResidencyClass;

fn boundary_for(request_id: &str, idempotency_key: &str) -> CloudKmsApiBoundaryContext {
    CloudKmsApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_alpha".to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal_for(principal_id: &str) -> CloudKmsApiPrincipal {
    CloudKmsApiPrincipal {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization_for(principal_id: &str, surfaces: &[&str]) -> CloudKmsApiAuthorization {
    CloudKmsApiAuthorization {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
        decision_id: format!("authz_decision_{principal_id}"),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
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
        authorization: authorization_for("sp_storage", &[CLOUD_KMS_ENCRYPT_SURFACE]),
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
        authorization: authorization_for("sp_storage", &[CLOUD_KMS_DECRYPT_SURFACE]),
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
fn encrypt_api_rejects_path_body_key_drift_before_receipt_mutation() {
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let mut request = encrypt_api_request("req-kms-key-drift", "idem-kms-key-drift");
    request.body.key_id = "kms/region-home/ten_alpha/other-key".to_string();

    let error = authorize_cloud_kms_encrypt_from_api(&mut directory, &mut ledger, request)
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
        authorize_cloud_kms_encrypt_from_api(&mut directory, &mut ledger, empty_request.clone()),
        Err(CloudKmsApiError::EmptyRequestId)
    );

    empty_request.boundary.request_id = "req-kms-tenant-drift".to_string();
    empty_request.boundary.tenant_id = "ten_other".to_string();
    assert_eq!(
        authorize_cloud_kms_encrypt_from_api(&mut directory, &mut ledger, empty_request),
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
fn encrypt_api_rejects_unauthorized_same_tenant_principal_before_ledger() {
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let mut request = encrypt_api_request("req-kms-authz-deny", "idem-kms-authz-deny");
    request.authorization.allowed_surfaces = vec![CLOUD_KMS_DECRYPT_SURFACE.to_string()];

    let error = authorize_cloud_kms_encrypt_from_api(&mut directory, &mut ledger, request)
        .expect_err("authorization decision does not allow encrypt");

    assert_eq!(
        error,
        CloudKmsApiError::AuthorizationDenied {
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

    let error = authorize_cloud_kms_decrypt_from_api(&mut directory, &mut ledger, request)
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

    let first = authorize_cloud_kms_encrypt_from_api(&mut directory, &mut ledger, request.clone())
        .expect("encrypt authorization succeeds");
    let second = authorize_cloud_kms_encrypt_from_api(&mut directory, &mut ledger, request)
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

    let first = authorize_cloud_kms_decrypt_from_api(&mut directory, &mut ledger, request.clone())
        .expect("decrypt authorization succeeds");
    let second = authorize_cloud_kms_decrypt_from_api(&mut directory, &mut ledger, request)
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

#[test]
fn encrypt_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let request = encrypt_api_request("req-kms-encrypt", "idem-kms-encrypt");
    authorize_cloud_kms_encrypt_from_api(&mut directory, &mut ledger, request.clone())
        .expect("initial encrypt succeeds");

    let mut drifted = request;
    drifted.body.ciphertext_ref = "ct/ten_alpha/object/002".to_string();
    assert_eq!(
        authorize_cloud_kms_encrypt_from_api(&mut directory, &mut ledger, drifted),
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
    let missing = authorize_cloud_kms_encrypt_from_api(&mut directory, &mut ledger, unknown_key)
        .expect_err("unknown key maps to not found");
    assert_eq!(missing.crypto_status_code(), 404);
    assert_eq!(missing, CloudKmsApiError::Kms(CloudKmsError::UnknownKey));

    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    authorize_cloud_kms_encrypt_from_api(
        &mut directory,
        &mut ledger,
        encrypt_api_request("req-kms-dup-1", "idem-kms-dup-1"),
    )
    .expect("first event succeeds");
    let duplicate = authorize_cloud_kms_encrypt_from_api(
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

    let error = authorize_cloud_kms_encrypt_from_api(&mut directory, &mut ledger, request)
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

    let error = authorize_cloud_kms_encrypt_from_api(&mut directory, &mut ledger, request)
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
