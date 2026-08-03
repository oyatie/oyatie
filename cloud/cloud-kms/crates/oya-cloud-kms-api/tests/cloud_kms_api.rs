// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_cloud_kms_api::{
    CLOUD_KMS_DECRYPT_SURFACE, CLOUD_KMS_DEFAULT_PUBLIC_API_VERSION, CLOUD_KMS_ENCRYPT_SURFACE,
    CLOUD_KMS_SCHEDULE_KEY_DELETION_SURFACE, CLOUD_KMS_SUPPORTED_PUBLIC_API_VERSIONS,
    CloudKmsApiAuthorization, CloudKmsApiBoundaryContext, CloudKmsApiError, CloudKmsApiPrincipal,
    CloudKmsCryptoApiStatus, CloudKmsCryptoIdempotencyLedger, CloudKmsDecryptApiRequest,
    CloudKmsDecryptRequest, CloudKmsEncryptApiRequest, CloudKmsEncryptRequest,
    CloudKmsKeyDeletionIdempotencyLedger, CloudKmsScheduleKeyDeletionApiRequest,
    CloudKmsScheduleKeyDeletionRequest, authorize_cloud_kms_decrypt_from_api,
    authorize_cloud_kms_encrypt_from_api, schedule_cloud_kms_key_deletion_from_api,
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

fn schedule_key_deletion_api_request(
    request_id: &str,
    idempotency_key: &str,
) -> CloudKmsScheduleKeyDeletionApiRequest {
    CloudKmsScheduleKeyDeletionApiRequest {
        path_key_id: "kms/region-home/ten_alpha/object-key".to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_tenant_offboarding"),
        authorization: authorization_for(
            "sp_tenant_offboarding",
            &[CLOUD_KMS_SCHEDULE_KEY_DELETION_SURFACE],
        ),
        body: CloudKmsScheduleKeyDeletionRequest {
            key_id: "kms/region-home/ten_alpha/object-key".to_string(),
            tenant_id: "ten_alpha".to_string(),
            actor: "sp_tenant_offboarding".to_string(),
            schedule_proof_ref: "kproof_tenant_offboard_schedule_001".to_string(),
            authorization_policy_version: "cedar-policy-v3".to_string(),
            required_approvals: 2,
            approver_principal_ids: vec!["usr_security_a".to_string(), "usr_privacy_b".to_string()],
            requested_at_epoch_seconds: 1_700_000_100,
            scheduled_deletion_at_epoch_seconds: 1_700_604_900,
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
fn schedule_key_deletion_requires_cedar_quorum_decision_and_emits_audit_evidence() {
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsKeyDeletionIdempotencyLedger::default();
    let request =
        schedule_key_deletion_api_request("req-kms-schedule-delete", "idem-kms-schedule-delete");

    let response =
        schedule_cloud_kms_key_deletion_from_api(&mut directory, &mut ledger, request.clone())
            .expect("Cedar-authorized quorum schedule emits evidence");
    let replay = schedule_cloud_kms_key_deletion_from_api(&mut directory, &mut ledger, request)
        .expect("same ScheduleKeyDeletion request replays idempotently");

    assert_eq!(response, replay);
    assert_eq!(ledger.len(), 1);
    assert_eq!(directory.deletion_schedule_receipts().count(), 1);
    assert_eq!(response.data.key_state, "pending_deletion");
    assert_eq!(
        response.data.authorization_decision_id,
        "authz_decision_sp_tenant_offboarding"
    );
    assert_eq!(
        response.data.authorization_policy_version,
        "cedar-policy-v3"
    );
    assert_eq!(response.data.required_approvals, 2);
    assert_eq!(
        response.data.approver_principal_ids,
        vec!["usr_privacy_b".to_string(), "usr_security_a".to_string()]
    );
    assert_eq!(response.evidence.operation, "schedule_key_deletion");
    assert_eq!(
        response.evidence.evidence_ref,
        "kproof_tenant_offboard_schedule_001"
    );
    assert_eq!(response.evidence.actor, "sp_tenant_offboarding");
    assert_eq!(
        directory
            .keys()
            .next()
            .expect("key remains indexed")
            .state
            .value,
        KmsKeyState::PendingDeletion
    );
}

#[test]
fn schedule_key_deletion_rejects_missing_quorum_before_state_or_ledger_mutation() {
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsKeyDeletionIdempotencyLedger::default();
    let mut request = schedule_key_deletion_api_request(
        "req-kms-schedule-delete-no-quorum",
        "idem-kms-schedule-delete-no-quorum",
    );
    request.body.approver_principal_ids = vec!["usr_security_a".to_string()];

    let error = schedule_cloud_kms_key_deletion_from_api(&mut directory, &mut ledger, request)
        .expect_err("ScheduleKeyDeletion requires Cedar quorum evidence");

    assert_eq!(
        error,
        CloudKmsApiError::Kms(CloudKmsError::KeyDeletionQuorumNotReached)
    );
    assert!(ledger.is_empty());
    assert_eq!(directory.deletion_schedule_receipts().count(), 0);
    assert_eq!(
        directory
            .keys()
            .next()
            .expect("key remains indexed")
            .state
            .value,
        KmsKeyState::Enabled
    );
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
fn kms_api_rejects_missing_or_unsupported_oyatie_version_before_ledger() {
    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let mut missing_encrypt =
        encrypt_api_request("req-kms-version-missing", "idem-kms-version-missing");
    missing_encrypt.boundary.oyatie_version = " ".to_string();
    missing_encrypt.authorization.allowed_surfaces.clear();

    assert_eq!(
        authorize_cloud_kms_encrypt_from_api(&mut directory, &mut ledger, missing_encrypt),
        Err(CloudKmsApiError::MissingPublicApiVersion)
    );
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);

    let mut unsupported_encrypt = encrypt_api_request(
        "req-kms-version-unsupported",
        "idem-kms-version-unsupported",
    );
    unsupported_encrypt.boundary.oyatie_version = "2026-01-01".to_string();
    unsupported_encrypt.authorization.allowed_surfaces.clear();

    assert_eq!(
        authorize_cloud_kms_encrypt_from_api(&mut directory, &mut ledger, unsupported_encrypt),
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
    missing_decrypt.authorization.allowed_surfaces.clear();

    assert_eq!(
        authorize_cloud_kms_decrypt_from_api(&mut directory, &mut ledger, missing_decrypt),
        Err(CloudKmsApiError::MissingPublicApiVersion)
    );
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);

    let mut unsupported_decrypt = decrypt_api_request(
        "req-kms-decrypt-version-unsupported",
        "idem-kms-decrypt-version-unsupported",
    );
    unsupported_decrypt.boundary.oyatie_version = "not-a-date".to_string();
    unsupported_decrypt.authorization.allowed_surfaces.clear();

    assert_eq!(
        authorize_cloud_kms_decrypt_from_api(&mut directory, &mut ledger, unsupported_decrypt),
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
    missing_region.authorization.allowed_surfaces.clear();
    assert_eq!(
        authorize_cloud_kms_encrypt_from_api(&mut directory, &mut ledger, missing_region),
        Err(CloudKmsApiError::EmptyRegionHeader)
    );
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);

    let mut missing_cell = decrypt_api_request("req-kms-cell-missing", "idem-kms-cell-missing");
    missing_cell.boundary.cell_id = "\t".to_string();
    missing_cell.authorization.allowed_surfaces.clear();
    assert_eq!(
        authorize_cloud_kms_decrypt_from_api(&mut directory, &mut ledger, missing_cell),
        Err(CloudKmsApiError::EmptyCellHeader)
    );
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);

    let mut region_drift = encrypt_api_request("req-kms-region-drift", "idem-kms-region-drift");
    region_drift.boundary.region = "region-away".to_string();
    region_drift.boundary.cell_id = "cell-region-away-a-001".to_string();
    assert_eq!(
        authorize_cloud_kms_encrypt_from_api(&mut directory, &mut ledger, region_drift),
        Err(CloudKmsApiError::Kms(CloudKmsError::ResourceRegionMismatch))
    );
    assert!(ledger.is_empty());
    assert_eq!(directory.receipts().count(), 0);

    let mut cell_drift = decrypt_api_request("req-kms-cell-drift", "idem-kms-cell-drift");
    cell_drift.boundary.cell_id = "cell-region-home-b-001".to_string();
    assert_eq!(
        authorize_cloud_kms_decrypt_from_api(&mut directory, &mut ledger, cell_drift),
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

        let response = authorize_cloud_kms_encrypt_from_api(&mut directory, &mut ledger, request)
            .expect("manifest-declared Cloud KMS public API version is accepted");

        assert_eq!(response.data.key_id, "kms/region-home/ten_alpha/object-key");
        assert_eq!(ledger.len(), 1);
        assert_eq!(directory.receipts().count(), 1);
    }

    let mut directory = directory_with_key();
    let mut ledger = CloudKmsCryptoIdempotencyLedger::default();
    let request = encrypt_api_request("req-kms-version-default", "idem-kms-version-key");
    authorize_cloud_kms_encrypt_from_api(&mut directory, &mut ledger, request.clone())
        .expect("default public API version succeeds");

    let mut version_drifted = request;
    version_drifted.boundary.oyatie_version = "2026-02-21".to_string();
    assert_eq!(
        authorize_cloud_kms_encrypt_from_api(&mut directory, &mut ledger, version_drifted),
        Err(CloudKmsApiError::IdempotencyKeyReused {
            idempotency_key: "idem-kms-version-key".to_string(),
        })
    );
    assert_eq!(ledger.len(), 1);
    assert_eq!(directory.receipts().count(), 1);
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
