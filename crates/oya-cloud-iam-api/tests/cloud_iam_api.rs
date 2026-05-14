use oya_cloud_iam_api::{
    create_cloud_iam_role_from_api, issue_cloud_iam_sts_token_from_api, CloudIamApiAuthorization,
    CloudIamApiBoundaryContext, CloudIamApiError, CloudIamApiPrincipal, CloudIamPrincipalRef,
    CloudIamRoleCreateApiRequest, CloudIamRoleCreateApiStatus, CloudIamRoleCreateIdempotencyLedger,
    CloudIamRoleCreateRequest, CloudIamScopeRef, CloudIamStsTokenApiRequest,
    CloudIamStsTokenApiStatus, CloudIamStsTokenIdempotencyLedger, CloudIamStsTokenRequest,
    CLOUD_IAM_ROLE_CREATE_SURFACE, CLOUD_IAM_STS_TOKEN_SURFACE,
};
use oya_cloud_iam_domain::{
    CloudIamError, IamDirectory, IamPrincipalCreate, IamPrincipalKind, IamRoleCreate, MfaState,
};
use oya_data_boundary_kernel::DataClass;

fn boundary_for(request_id: &str, idempotency_key: &str) -> CloudIamApiBoundaryContext {
    CloudIamApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_kr".to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal_for(principal_id: &str) -> CloudIamApiPrincipal {
    CloudIamApiPrincipal {
        tenant_id: "ten_kr".to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization_for(principal_id: &str, surfaces: &[&str]) -> CloudIamApiAuthorization {
    CloudIamApiAuthorization {
        tenant_id: "ten_kr".to_string(),
        principal_id: principal_id.to_string(),
        decision_id: format!("authz_decision_{principal_id}"),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
    }
}

fn service_principal_create() -> IamPrincipalCreate {
    IamPrincipalCreate {
        id: "sp_cloud_provisioner".to_string(),
        tenant_id: "ten_kr".to_string(),
        kind: IamPrincipalKind::ServiceAccount,
        display_name: "cloud provisioner".to_string(),
        external_subject: None,
        identity_provider_id: None,
        region_pack: "oya-pack-kr".to_string(),
        mfa_state: MfaState::NotRequired,
        last_authenticated_at_epoch_seconds: None,
        created_at_epoch_seconds: 1_700_000_001,
    }
}

fn user_principal_create() -> IamPrincipalCreate {
    IamPrincipalCreate {
        id: "usr_alice".to_string(),
        tenant_id: "ten_kr".to_string(),
        kind: IamPrincipalKind::User,
        display_name: "Alice".to_string(),
        external_subject: None,
        identity_provider_id: None,
        region_pack: "oya-pack-kr".to_string(),
        mfa_state: MfaState::Verified,
        last_authenticated_at_epoch_seconds: Some(1_700_000_002),
        created_at_epoch_seconds: 1_700_000_001,
    }
}

fn unverified_user_principal_create() -> IamPrincipalCreate {
    IamPrincipalCreate {
        id: "usr_bob".to_string(),
        display_name: "Bob".to_string(),
        mfa_state: MfaState::Enrolled,
        ..user_principal_create()
    }
}

fn role_create() -> IamRoleCreate {
    IamRoleCreate {
        id: "role_compute_admin".to_string(),
        tenant_id: "ten_kr".to_string(),
        region: "kr-seoul".to_string(),
        name: "compute-admin".to_string(),
        cedar_policy_id: "pol_cloud_compute_admin".to_string(),
        cedar_policy_version: "1.0.0".to_string(),
        assumable_by: vec!["sp_cloud_provisioner".to_string(), "usr_alice".to_string()],
        max_session_duration_sec: 900,
        data_class: DataClass::Public,
        created_at_epoch_seconds: 1_700_000_003,
    }
}

fn directory_with_principals() -> IamDirectory {
    let mut directory = IamDirectory::default();
    directory
        .create_principal(service_principal_create())
        .expect("service principal registers");
    directory
        .create_principal(user_principal_create())
        .expect("user principal registers");
    directory
}

fn directory_with_role() -> IamDirectory {
    let mut directory = directory_with_principals();
    directory
        .create_role(role_create())
        .expect("role registers");
    directory
}

fn role_api_request(request_id: &str, idempotency_key: &str) -> CloudIamRoleCreateApiRequest {
    CloudIamRoleCreateApiRequest {
        path_role_id: "role_compute_admin".to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_cloud_provisioner"),
        authorization: authorization_for("sp_cloud_provisioner", &[CLOUD_IAM_ROLE_CREATE_SURFACE]),
        body: CloudIamRoleCreateRequest {
            tenant_id: "ten_kr".to_string(),
            role_id: "role_compute_admin".to_string(),
            region: "kr-seoul".to_string(),
            name: "compute-admin".to_string(),
            cedar_policy_id: "pol_cloud_compute_admin".to_string(),
            cedar_policy_version: "1.0.0".to_string(),
            assumable_by: vec![
                CloudIamPrincipalRef {
                    value: "sp_cloud_provisioner".to_string(),
                },
                CloudIamPrincipalRef {
                    value: "usr_alice".to_string(),
                },
            ],
            max_session_duration_sec: 900,
            data_class: "PUBLIC".to_string(),
            created_at_epoch_seconds: 1_700_000_003,
        },
    }
}

fn sts_api_request(request_id: &str, idempotency_key: &str) -> CloudIamStsTokenApiRequest {
    CloudIamStsTokenApiRequest {
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_cloud_provisioner"),
        authorization: authorization_for("sp_cloud_provisioner", &[CLOUD_IAM_STS_TOKEN_SURFACE]),
        body: CloudIamStsTokenRequest {
            tenant_id: "ten_kr".to_string(),
            session_id: "sts_compute_admin_001".to_string(),
            role_id: "role_compute_admin".to_string(),
            assumed_by: "sp_cloud_provisioner".to_string(),
            external_id: None,
            requested_duration_sec: 600,
            scopes: vec![
                CloudIamScopeRef {
                    value: "cloud.compute.write".to_string(),
                },
                CloudIamScopeRef {
                    value: "cloud.iam.read".to_string(),
                },
            ],
            issued_at_epoch_seconds: 1_700_000_100,
        },
    }
}

#[test]
fn role_create_api_rejects_unauthorized_same_tenant_principal_before_ledger() {
    let mut directory = directory_with_principals();
    let mut ledger = CloudIamRoleCreateIdempotencyLedger::default();
    let mut request = role_api_request("req-role-authz-deny", "idem-role-authz-deny");
    request.authorization.allowed_surfaces = vec![CLOUD_IAM_STS_TOKEN_SURFACE.to_string()];

    let error = create_cloud_iam_role_from_api(&mut directory, &mut ledger, request)
        .expect_err("authorization decision does not allow role creation");

    assert_eq!(
        error,
        CloudIamApiError::AuthorizationDenied {
            surface: CLOUD_IAM_ROLE_CREATE_SURFACE.to_string(),
        }
    );
    assert_eq!(error.role_create_status_code(), 403);
    assert!(ledger.is_empty());
    directory
        .create_role(role_create())
        .expect("authorization denial happened before directory mutation");
}

#[test]
fn role_create_api_rejects_path_body_role_drift_before_directory_mutation() {
    let mut directory = directory_with_principals();
    let mut ledger = CloudIamRoleCreateIdempotencyLedger::default();
    let mut request = role_api_request("req-role-drift", "idem-role-drift");
    request.body.role_id = "role_other".to_string();

    let result = create_cloud_iam_role_from_api(&mut directory, &mut ledger, request);

    assert_eq!(
        result,
        Err(CloudIamApiError::RoleIdMismatch {
            path_role_id: "role_compute_admin".to_string(),
            body_role_id: "role_other".to_string(),
        })
    );
    assert!(ledger.is_empty());
    assert!(directory
        .create_role(role_create())
        .expect("directory stayed mutable")
        .id
        .value
        .value
        .starts_with("role_"));
}

#[test]
fn role_create_api_rejects_required_header_and_tenant_drift_before_ledger() {
    let mut directory = directory_with_principals();
    let mut ledger = CloudIamRoleCreateIdempotencyLedger::default();
    let mut empty_request = role_api_request(" ", "idem-empty-header");
    assert_eq!(
        create_cloud_iam_role_from_api(&mut directory, &mut ledger, empty_request.clone()),
        Err(CloudIamApiError::EmptyRequestId)
    );

    empty_request.boundary.request_id = "req-tenant-drift".to_string();
    empty_request.boundary.tenant_id = "ten_other".to_string();
    assert_eq!(
        create_cloud_iam_role_from_api(&mut directory, &mut ledger, empty_request),
        Err(CloudIamApiError::TenantMismatch {
            header_tenant_id: "ten_other".to_string(),
            principal_tenant_id: "ten_kr".to_string(),
            body_tenant_id: "ten_kr".to_string(),
        })
    );
    assert!(ledger.is_empty());
}

#[test]
fn role_create_api_creates_role_once_and_replays_same_idempotent_result() {
    let mut directory = directory_with_principals();
    let mut ledger = CloudIamRoleCreateIdempotencyLedger::default();
    let request = role_api_request("req-role-create", "idem-role-create");

    let first = create_cloud_iam_role_from_api(&mut directory, &mut ledger, request.clone())
        .expect("role creation succeeds");
    let second = create_cloud_iam_role_from_api(&mut directory, &mut ledger, request)
        .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(first.data.role_id, "role_compute_admin");
    assert_eq!(first.data.region, "kr-seoul");
    assert_eq!(first.metadata.request_id, "req-role-create");
    assert_eq!(CLOUD_IAM_ROLE_CREATE_SURFACE, "cloud.iam.role.create");
    assert_eq!(CloudIamRoleCreateApiStatus::Created.code(), 201);
    assert_eq!(CloudIamRoleCreateApiStatus::BadRequest.code(), 400);
    assert_eq!(CloudIamRoleCreateApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudIamRoleCreateApiStatus::Conflict.code(), 409);
    assert_eq!(CloudIamRoleCreateApiStatus::UnprocessableEntity.code(), 422);
}

#[test]
fn role_create_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut directory = directory_with_principals();
    let mut ledger = CloudIamRoleCreateIdempotencyLedger::default();
    let request = role_api_request("req-role-create", "idem-role-create");
    create_cloud_iam_role_from_api(&mut directory, &mut ledger, request.clone())
        .expect("initial create succeeds");

    let mut drifted = request;
    drifted.body.name = "network-admin".to_string();
    assert_eq!(
        create_cloud_iam_role_from_api(&mut directory, &mut ledger, drifted),
        Err(CloudIamApiError::IdempotencyKeyReused {
            idempotency_key: "idem-role-create".to_string(),
        })
    );
}

#[test]
fn role_create_api_maps_kernel_duplicate_and_invalid_data_class() {
    let mut directory = directory_with_role();
    let mut ledger = CloudIamRoleCreateIdempotencyLedger::default();
    let duplicate = create_cloud_iam_role_from_api(
        &mut directory,
        &mut ledger,
        role_api_request("req-role-duplicate", "idem-role-duplicate"),
    )
    .expect_err("duplicate role is a conflict");
    assert_eq!(duplicate.role_create_status_code(), 409);
    assert_eq!(
        duplicate,
        CloudIamApiError::Iam(CloudIamError::DuplicateRole)
    );

    let mut directory = directory_with_principals();
    let mut ledger = CloudIamRoleCreateIdempotencyLedger::default();
    let mut invalid_class = role_api_request("req-bad-class", "idem-bad-class");
    invalid_class.body.data_class = "SECRET".to_string();
    assert_eq!(
        create_cloud_iam_role_from_api(&mut directory, &mut ledger, invalid_class),
        Err(CloudIamApiError::InvalidDataClassLabel {
            data_class: "SECRET".to_string(),
        })
    );
}

#[test]
fn sts_token_api_rejects_authorization_principal_drift_before_ledger() {
    let mut directory = directory_with_role();
    let mut ledger = CloudIamStsTokenIdempotencyLedger::default();
    let mut request = sts_api_request("req-sts-authz-drift", "idem-sts-authz-drift");
    request.authorization.principal_id = "usr_alice".to_string();

    let error = issue_cloud_iam_sts_token_from_api(&mut directory, &mut ledger, request)
        .expect_err("authorization decision principal drift is rejected");

    assert_eq!(
        error,
        CloudIamApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: "usr_alice".to_string(),
            principal_id: "sp_cloud_provisioner".to_string(),
        }
    );
    assert_eq!(error.sts_token_status_code(), 403);
    assert!(ledger.is_empty());
}

#[test]
fn sts_token_api_rejects_principal_body_drift_before_directory_mutation() {
    let mut directory = directory_with_role();
    let mut ledger = CloudIamStsTokenIdempotencyLedger::default();
    let mut request = sts_api_request("req-sts-drift", "idem-sts-drift");
    request.body.assumed_by = "usr_alice".to_string();

    let result = issue_cloud_iam_sts_token_from_api(&mut directory, &mut ledger, request);

    assert_eq!(
        result,
        Err(CloudIamApiError::PrincipalMismatch {
            principal_tenant_id: "ten_kr".to_string(),
            principal_id: "sp_cloud_provisioner".to_string(),
            body_tenant_id: "ten_kr".to_string(),
            assumed_by: "usr_alice".to_string(),
        })
    );
    assert!(ledger.is_empty());
}

#[test]
fn sts_token_api_maps_invalid_duration_issue_without_generic_masking() {
    let mut directory = directory_with_role();
    let mut ledger = CloudIamStsTokenIdempotencyLedger::default();
    let mut request = sts_api_request("req-sts-duration", "idem-sts-duration");
    request.body.session_id = "sts_bad_duration".to_string();
    request.body.requested_duration_sec = 0;

    let error = issue_cloud_iam_sts_token_from_api(&mut directory, &mut ledger, request)
        .expect_err("kernel rejects zero-duration STS token");

    assert_eq!(
        error,
        CloudIamApiError::Iam(CloudIamError::InvalidSessionDuration)
    );
    assert_eq!(error.sts_token_status_code(), 400);
    assert_eq!(
        error
            .error_response("req-sts-duration")
            .error
            .details
            .first()
            .expect("cloud IAM error detail")
            .issue,
        "session duration must be >0 and <= role/platform limit"
    );
}

#[test]
fn sts_token_api_issues_short_lived_session_and_replays_idempotent_result() {
    let mut directory = directory_with_role();
    let mut ledger = CloudIamStsTokenIdempotencyLedger::default();
    let request = sts_api_request("req-sts-token", "idem-sts-token");

    let first = issue_cloud_iam_sts_token_from_api(&mut directory, &mut ledger, request.clone())
        .expect("STS token succeeds");
    let second = issue_cloud_iam_sts_token_from_api(&mut directory, &mut ledger, request)
        .expect("same STS idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(first.data.session_id, "sts_compute_admin_001");
    assert_eq!(first.data.expires_at_epoch_seconds, 1_700_000_700);
    assert!(first.data.token_fingerprint.starts_with("sts1:"));
    assert_eq!(first.metadata.request_id, "req-sts-token");
    assert_eq!(CLOUD_IAM_STS_TOKEN_SURFACE, "cloud.iam.sts.token");
    assert_eq!(CloudIamStsTokenApiStatus::Ok.code(), 200);
    assert_eq!(CloudIamStsTokenApiStatus::BadRequest.code(), 400);
    assert_eq!(CloudIamStsTokenApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudIamStsTokenApiStatus::Conflict.code(), 409);
    assert_eq!(CloudIamStsTokenApiStatus::UnprocessableEntity.code(), 422);
}

#[test]
fn sts_token_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut directory = directory_with_role();
    let mut ledger = CloudIamStsTokenIdempotencyLedger::default();
    let request = sts_api_request("req-sts-token", "idem-sts-token");
    issue_cloud_iam_sts_token_from_api(&mut directory, &mut ledger, request.clone())
        .expect("initial STS token succeeds");

    let mut drifted = request;
    drifted.body.requested_duration_sec = 300;
    assert_eq!(
        issue_cloud_iam_sts_token_from_api(&mut directory, &mut ledger, drifted),
        Err(CloudIamApiError::IdempotencyKeyReused {
            idempotency_key: "idem-sts-token".to_string(),
        })
    );
}

#[test]
fn sts_token_api_maps_mfa_policy_denial_to_forbidden() {
    let mut directory = IamDirectory::default();
    directory
        .create_principal(unverified_user_principal_create())
        .expect("unverified user registers");
    directory
        .create_role(IamRoleCreate {
            assumable_by: vec!["usr_bob".to_string()],
            ..role_create()
        })
        .expect("role registers");
    let mut ledger = CloudIamStsTokenIdempotencyLedger::default();
    let mut request = sts_api_request("req-sts-mfa", "idem-sts-mfa");
    request.principal = principal_for("usr_bob");
    request.authorization = authorization_for("usr_bob", &[CLOUD_IAM_STS_TOKEN_SURFACE]);
    request.body.assumed_by = "usr_bob".to_string();
    request.body.session_id = "sts_user_mfa".to_string();
    request.body.scopes = vec![CloudIamScopeRef {
        value: "cloud.iam.read".to_string(),
    }];

    let error = issue_cloud_iam_sts_token_from_api(&mut directory, &mut ledger, request)
        .expect_err("MFA policy denies token issuance");

    assert_eq!(error.sts_token_status_code(), 403);
    assert_eq!(error, CloudIamApiError::Iam(CloudIamError::MfaNotVerified));
}
