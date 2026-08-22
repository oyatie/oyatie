// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use iam_cloud_api::{
    CLOUD_IAM_DEFAULT_PUBLIC_API_VERSION, CLOUD_IAM_IDENTITY_PROVIDER_CREATE_SURFACE,
    CLOUD_IAM_IDENTITY_PROVIDER_DELETE_SURFACE, CLOUD_IAM_ROLE_CREATE_SURFACE,
    CLOUD_IAM_STS_TOKEN_SURFACE, CLOUD_IAM_SUPPORTED_PUBLIC_API_VERSIONS, CloudIamApiAuthorization,
    CloudIamApiBoundaryContext, CloudIamApiError, CloudIamApiPlacementBoundary,
    CloudIamApiPrincipal, CloudIamApiReadBoundaryContext, CloudIamBoundaryCellId,
    CloudIamBoundaryRegionId, CloudIamBoundaryTenantId, CloudIamIdentityProviderCreateApiRequest,
    CloudIamIdentityProviderCreateApiStatus, CloudIamIdentityProviderCreateIdempotencyLedger,
    CloudIamIdentityProviderCreateRequest, CloudIamIdentityProviderDeleteApiRequest,
    CloudIamIdentityProviderDeleteApiStatus, CloudIamIdentityProviderDeleteIdempotencyLedger,
    CloudIamIdentityProviderKind, CloudIamIdentityProviderListApiRequest,
    CloudIamIdentityProviderListApiStatus, CloudIamIdentityProviderUpdateApiRequest,
    CloudIamIdentityProviderUpdateApiStatus, CloudIamIdentityProviderUpdateIdempotencyLedger,
    CloudIamIdentityProviderUpdateRequest, CloudIamPrincipalRef, CloudIamRoleCreateApiRequest,
    CloudIamRoleCreateApiStatus, CloudIamRoleCreateIdempotencyLedger, CloudIamRoleCreateRequest,
    CloudIamScopeRef, CloudIamStsTokenApiRequest, CloudIamStsTokenApiStatus,
    CloudIamStsTokenIdempotencyLedger, CloudIamStsTokenRequest,
    create_cloud_iam_identity_provider_from_api, create_cloud_iam_role_from_api,
    delete_cloud_iam_identity_provider_from_api, issue_cloud_iam_sts_token_from_api,
    list_cloud_iam_identity_providers_from_api, update_cloud_iam_identity_provider_from_api,
};
use iam_cloud_domain::{
    CloudIamError, IamDirectory, IamPrincipalCreate, IamPrincipalKind, IamRoleCreate,
    IdentityProviderCreate, IdentityProviderKind, MfaState,
};
use data_boundary_kernel::DataClass;

fn boundary_for(request_id: &str, idempotency_key: &str) -> CloudIamApiBoundaryContext {
    CloudIamApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_alpha".to_string(),
        idempotency_key: idempotency_key.to_string(),
        oyatie_version: CLOUD_IAM_DEFAULT_PUBLIC_API_VERSION.to_string(),
        placement: placement_for("ten_alpha", "cell-alpha-region-a-001", "region-home"),
    }
}

fn read_boundary_for(request_id: &str) -> CloudIamApiReadBoundaryContext {
    CloudIamApiReadBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_alpha".to_string(),
        oyatie_version: CLOUD_IAM_DEFAULT_PUBLIC_API_VERSION.to_string(),
        placement: placement_for("ten_alpha", "cell-alpha-region-a-001", "region-home"),
    }
}

fn placement_for(tenant_id: &str, cell_id: &str, region_id: &str) -> CloudIamApiPlacementBoundary {
    CloudIamApiPlacementBoundary {
        tenant_id: CloudIamBoundaryTenantId {
            value: tenant_id.to_string(),
        },
        cell_id: CloudIamBoundaryCellId {
            value: cell_id.to_string(),
        },
        region_id: CloudIamBoundaryRegionId {
            value: region_id.to_string(),
        },
    }
}

fn principal_for(principal_id: &str) -> CloudIamApiPrincipal {
    CloudIamApiPrincipal {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization_for(principal_id: &str, surfaces: &[&str]) -> CloudIamApiAuthorization {
    CloudIamApiAuthorization {
        tenant_id: "ten_alpha".to_string(),
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
        tenant_id: "ten_alpha".to_string(),
        kind: IamPrincipalKind::ServiceAccount,
        display_name: "cloud provisioner".to_string(),
        external_subject: None,
        identity_provider_id: None,
        region_pack: "pack-alpha".to_string(),
        mfa_state: MfaState::NotRequired,
        last_authenticated_at_epoch_seconds: None,
        created_at_epoch_seconds: 1_700_000_001,
    }
}

fn user_principal_create() -> IamPrincipalCreate {
    IamPrincipalCreate {
        id: "usr_alice".to_string(),
        tenant_id: "ten_alpha".to_string(),
        kind: IamPrincipalKind::User,
        display_name: "Alice".to_string(),
        external_subject: None,
        identity_provider_id: None,
        region_pack: "pack-alpha".to_string(),
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

fn external_oidc_provider_create() -> IdentityProviderCreate {
    IdentityProviderCreate {
        id: "idp_partner_oidc".to_string(),
        tenant_id: "ten_alpha".to_string(),
        region_pack: "pack-alpha".to_string(),
        kind: IdentityProviderKind::Oidc,
        issuer_uri: "https://partner.example/oidc".to_string(),
        audience: "urn:oyatie:cloud".to_string(),
        verification_material_ref: "jwks/partner".to_string(),
        created_at_epoch_seconds: 1_700_000_020,
    }
}

fn external_saml_provider_create() -> IdentityProviderCreate {
    IdentityProviderCreate {
        id: "idp_alpha_saml".to_string(),
        tenant_id: "ten_alpha".to_string(),
        region_pack: "pack-alpha".to_string(),
        kind: IdentityProviderKind::Saml,
        issuer_uri: "https://partner.example/saml".to_string(),
        audience: "urn:oyatie:cloud:saml".to_string(),
        verification_material_ref: "cert/partner".to_string(),
        created_at_epoch_seconds: 1_700_000_021,
    }
}

fn beta_oidc_provider_create() -> IdentityProviderCreate {
    IdentityProviderCreate {
        id: "idp_beta_oidc".to_string(),
        tenant_id: "ten_beta".to_string(),
        region_pack: "pack-beta".to_string(),
        kind: IdentityProviderKind::Oidc,
        issuer_uri: "https://beta.example/oidc".to_string(),
        audience: "urn:oyatie:cloud:beta".to_string(),
        verification_material_ref: "jwks/beta".to_string(),
        created_at_epoch_seconds: 1_700_000_022,
    }
}

fn external_principal_create() -> IamPrincipalCreate {
    IamPrincipalCreate {
        id: "sp_external_partner".to_string(),
        tenant_id: "ten_alpha".to_string(),
        kind: IamPrincipalKind::External,
        display_name: "Partner".to_string(),
        external_subject: Some("oidc://partner.example/sub-1".to_string()),
        identity_provider_id: Some("idp_partner_oidc".to_string()),
        region_pack: "pack-alpha".to_string(),
        mfa_state: MfaState::Verified,
        last_authenticated_at_epoch_seconds: Some(1_700_000_050),
        created_at_epoch_seconds: 1_700_000_040,
    }
}

fn role_create() -> IamRoleCreate {
    IamRoleCreate {
        id: "role_compute_admin".to_string(),
        tenant_id: "ten_alpha".to_string(),
        region: "region-home".to_string(),
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

fn directory_with_external_role() -> IamDirectory {
    let mut directory = IamDirectory::default();
    directory
        .register_identity_provider(external_oidc_provider_create())
        .expect("OIDC provider registers");
    directory
        .create_principal(external_principal_create())
        .expect("external principal registers");
    directory
        .create_role(IamRoleCreate {
            assumable_by: vec!["sp_external_partner".to_string()],
            ..role_create()
        })
        .expect("role trusts external principal");
    directory
}

fn role_api_request(request_id: &str, idempotency_key: &str) -> CloudIamRoleCreateApiRequest {
    CloudIamRoleCreateApiRequest {
        path_role_id: "role_compute_admin".to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_cloud_provisioner"),
        authorization: authorization_for("sp_cloud_provisioner", &[CLOUD_IAM_ROLE_CREATE_SURFACE]),
        body: CloudIamRoleCreateRequest {
            tenant_id: "ten_alpha".to_string(),
            role_id: "role_compute_admin".to_string(),
            region: "region-home".to_string(),
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
            tenant_id: "ten_alpha".to_string(),
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

fn external_sts_api_request(request_id: &str, idempotency_key: &str) -> CloudIamStsTokenApiRequest {
    CloudIamStsTokenApiRequest {
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_external_partner"),
        authorization: authorization_for("sp_external_partner", &[CLOUD_IAM_STS_TOKEN_SURFACE]),
        body: CloudIamStsTokenRequest {
            tenant_id: "ten_alpha".to_string(),
            session_id: "sts_external_partner_001".to_string(),
            role_id: "role_compute_admin".to_string(),
            assumed_by: "sp_external_partner".to_string(),
            external_id: Some("external-customer-alpha".to_string()),
            requested_duration_sec: 300,
            scopes: vec![CloudIamScopeRef {
                value: "cloud.iam.read".to_string(),
            }],
            issued_at_epoch_seconds: 1_700_000_100,
        },
    }
}

fn identity_provider_api_request(
    request_id: &str,
    idempotency_key: &str,
) -> CloudIamIdentityProviderCreateApiRequest {
    CloudIamIdentityProviderCreateApiRequest {
        path_identity_provider_id: "idp_partner_oidc".to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_cloud_provisioner"),
        authorization: authorization_for(
            "sp_cloud_provisioner",
            &[CLOUD_IAM_IDENTITY_PROVIDER_CREATE_SURFACE],
        ),
        body: CloudIamIdentityProviderCreateRequest {
            tenant_id: "ten_alpha".to_string(),
            identity_provider_id: "idp_partner_oidc".to_string(),
            region_pack: "pack-alpha".to_string(),
            kind: CloudIamIdentityProviderKind::Oidc,
            issuer_uri: "https://partner.example/oidc".to_string(),
            audience: "urn:oyatie:cloud".to_string(),
            verification_material_ref: "jwks/partner".to_string(),
            created_at_epoch_seconds: 1_700_000_020,
        },
    }
}

fn identity_provider_list_api_request(request_id: &str) -> CloudIamIdentityProviderListApiRequest {
    CloudIamIdentityProviderListApiRequest {
        boundary: read_boundary_for(request_id),
        principal: principal_for("sp_cloud_provisioner"),
        authorization: authorization_for(
            "sp_cloud_provisioner",
            &[iam_cloud_api::CLOUD_IAM_IDENTITY_PROVIDER_LIST_SURFACE],
        ),
    }
}

fn identity_provider_update_api_request(
    request_id: &str,
    idempotency_key: &str,
) -> CloudIamIdentityProviderUpdateApiRequest {
    CloudIamIdentityProviderUpdateApiRequest {
        path_identity_provider_id: "idp_partner_oidc".to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_cloud_provisioner"),
        authorization: authorization_for(
            "sp_cloud_provisioner",
            &[iam_cloud_api::CLOUD_IAM_IDENTITY_PROVIDER_UPDATE_SURFACE],
        ),
        body: CloudIamIdentityProviderUpdateRequest {
            tenant_id: "ten_alpha".to_string(),
            identity_provider_id: "idp_partner_oidc".to_string(),
            region_pack: "pack-alpha".to_string(),
            kind: CloudIamIdentityProviderKind::Saml,
            issuer_uri: "https://partner.example/saml/v2".to_string(),
            audience: "urn:oyatie:cloud:saml:v2".to_string(),
            verification_material_ref: "cert/partner-rotated".to_string(),
        },
    }
}

fn identity_provider_delete_api_request(
    request_id: &str,
    idempotency_key: &str,
) -> CloudIamIdentityProviderDeleteApiRequest {
    CloudIamIdentityProviderDeleteApiRequest {
        path_identity_provider_id: "idp_partner_oidc".to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_cloud_provisioner"),
        authorization: authorization_for(
            "sp_cloud_provisioner",
            &[CLOUD_IAM_IDENTITY_PROVIDER_DELETE_SURFACE],
        ),
        tenant_id: "ten_alpha".to_string(),
    }
}

#[test]
fn identity_provider_list_api_returns_tenant_scoped_providers_in_deterministic_order() {
    let mut directory = IamDirectory::default();
    directory
        .register_identity_provider(external_oidc_provider_create())
        .expect("OIDC provider registers");
    directory
        .register_identity_provider(beta_oidc_provider_create())
        .expect("beta provider registers");
    directory
        .register_identity_provider(external_saml_provider_create())
        .expect("SAML provider registers");

    let response = list_cloud_iam_identity_providers_from_api(
        &directory,
        identity_provider_list_api_request("req-idp-list"),
    )
    .expect("tenant-scoped provider list succeeds");

    assert_eq!(response.metadata.request_id, "req-idp-list");
    assert_eq!(
        response
            .data
            .iter()
            .map(|provider| provider.identity_provider_id.as_str())
            .collect::<Vec<_>>(),
        vec!["idp_alpha_saml", "idp_partner_oidc"]
    );
    assert_eq!(response.data[0].kind, CloudIamIdentityProviderKind::Saml);
    assert_eq!(response.data[1].kind, CloudIamIdentityProviderKind::Oidc);
    assert_eq!(CloudIamIdentityProviderListApiStatus::Ok.code(), 200);
    assert_eq!(
        CloudIamIdentityProviderListApiStatus::BadRequest.code(),
        400
    );
    assert_eq!(CloudIamIdentityProviderListApiStatus::Forbidden.code(), 403);
}

#[test]
fn identity_provider_list_api_rejects_cross_tenant_or_unauthorized_reads() {
    let directory = IamDirectory::default();
    let mut cross_tenant = identity_provider_list_api_request("req-idp-list-cross-tenant");
    cross_tenant.boundary.tenant_id = "ten_beta".to_string();
    cross_tenant.boundary.placement.tenant_id.value = "ten_beta".to_string();

    assert_eq!(
        list_cloud_iam_identity_providers_from_api(&directory, cross_tenant),
        Err(CloudIamApiError::TenantMismatch {
            header_tenant_id: "ten_beta".to_string(),
            principal_tenant_id: "ten_alpha".to_string(),
            body_tenant_id: "ten_beta".to_string(),
        })
    );

    let unauthorized = CloudIamIdentityProviderListApiRequest {
        authorization: authorization_for("sp_cloud_provisioner", &[CLOUD_IAM_STS_TOKEN_SURFACE]),
        ..identity_provider_list_api_request("req-idp-list-denied")
    };
    assert_eq!(
        list_cloud_iam_identity_providers_from_api(&directory, unauthorized),
        Err(CloudIamApiError::AuthorizationDenied {
            surface: iam_cloud_api::CLOUD_IAM_IDENTITY_PROVIDER_LIST_SURFACE.to_string(),
        })
    );
}

#[test]
fn identity_provider_delete_api_deletes_existing_provider_with_idempotency_and_tenant_binding() {
    let mut directory = IamDirectory::default();
    directory
        .register_identity_provider(external_oidc_provider_create())
        .expect("OIDC provider registers");
    let mut ledger = CloudIamIdentityProviderDeleteIdempotencyLedger::default();
    let request = identity_provider_delete_api_request("req-idp-delete", "idem-idp-delete");

    let first =
        delete_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, request.clone())
            .expect("provider-managed IdP deletes through API");
    let second = delete_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, request)
        .expect("same IdP delete request replays idempotently");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(first.metadata.request_id, "req-idp-delete");
    assert_eq!(first.data.identity_provider_id, "idp_partner_oidc");
    assert_eq!(first.data.tenant_id, "ten_alpha");
    assert_eq!(first.data.kind, CloudIamIdentityProviderKind::Oidc);
    assert_eq!(
        CLOUD_IAM_IDENTITY_PROVIDER_DELETE_SURFACE,
        "cloud.iam.identity_provider.delete"
    );
    assert_eq!(CloudIamIdentityProviderDeleteApiStatus::Ok.code(), 200);
    assert_eq!(
        CloudIamIdentityProviderDeleteApiStatus::BadRequest.code(),
        400
    );
    assert_eq!(
        CloudIamIdentityProviderDeleteApiStatus::Forbidden.code(),
        403
    );
    assert_eq!(
        CloudIamIdentityProviderDeleteApiStatus::Conflict.code(),
        409
    );
    assert_eq!(
        CloudIamIdentityProviderDeleteApiStatus::UnprocessableEntity.code(),
        422
    );

    let listed = list_cloud_iam_identity_providers_from_api(
        &directory,
        identity_provider_list_api_request("req-idp-delete-list"),
    )
    .expect("tenant-scoped provider list remains available after delete");
    assert!(listed.data.is_empty());
}

#[test]
fn identity_provider_delete_api_rejects_missing_cross_tenant_unauthorized_or_in_use_deletes() {
    let mut directory = IamDirectory::default();
    directory
        .register_identity_provider(external_oidc_provider_create())
        .expect("OIDC provider registers");
    directory
        .register_identity_provider(beta_oidc_provider_create())
        .expect("beta provider registers");
    let mut ledger = CloudIamIdentityProviderDeleteIdempotencyLedger::default();

    let mut missing =
        identity_provider_delete_api_request("req-idp-delete-missing", "idem-delete-missing");
    missing.path_identity_provider_id = "idp_missing_oidc".to_string();
    let missing_error =
        delete_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, missing)
            .expect_err("missing provider cannot be deleted");
    assert_eq!(
        missing_error,
        CloudIamApiError::Iam(CloudIamError::UnknownProvider)
    );
    assert_eq!(missing_error.identity_provider_delete_status_code(), 400);

    let mut cross_tenant =
        identity_provider_delete_api_request("req-idp-delete-cross", "idem-delete-cross");
    cross_tenant.path_identity_provider_id = "idp_beta_oidc".to_string();
    let cross_tenant_error =
        delete_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, cross_tenant)
            .expect_err("cross-tenant provider cannot be deleted by alpha principal");
    assert_eq!(
        cross_tenant_error,
        CloudIamApiError::Iam(CloudIamError::ProviderTenantMismatch)
    );
    assert_eq!(
        cross_tenant_error.identity_provider_delete_status_code(),
        403
    );

    let mut unauthorized =
        identity_provider_delete_api_request("req-idp-delete-denied", "idem-delete-denied");
    unauthorized.authorization = authorization_for(
        "sp_cloud_provisioner",
        &[iam_cloud_api::CLOUD_IAM_IDENTITY_PROVIDER_LIST_SURFACE],
    );
    let before_denied_ledger_len = ledger.len();
    assert_eq!(
        delete_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, unauthorized),
        Err(CloudIamApiError::AuthorizationDenied {
            surface: CLOUD_IAM_IDENTITY_PROVIDER_DELETE_SURFACE.to_string(),
        })
    );
    assert_eq!(ledger.len(), before_denied_ledger_len);

    let mut drifted =
        identity_provider_delete_api_request("req-idp-delete-tenant-drift", "idem-delete-drift");
    drifted.tenant_id = "ten_beta".to_string();
    assert_eq!(
        delete_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, drifted),
        Err(CloudIamApiError::TenantMismatch {
            header_tenant_id: "ten_alpha".to_string(),
            principal_tenant_id: "ten_alpha".to_string(),
            body_tenant_id: "ten_beta".to_string(),
        })
    );

    directory
        .create_principal(external_principal_create())
        .expect("external principal binds provider");
    let in_use = delete_cloud_iam_identity_provider_from_api(
        &mut directory,
        &mut ledger,
        identity_provider_delete_api_request("req-idp-delete-in-use", "idem-delete-in-use"),
    )
    .expect_err("provider with bound principals cannot be deleted");
    assert_eq!(in_use, CloudIamApiError::Iam(CloudIamError::ProviderInUse));
    assert_eq!(in_use.identity_provider_delete_status_code(), 409);
}

#[test]
fn identity_provider_update_api_updates_existing_provider_with_idempotency_and_tenant_binding() {
    let mut directory = IamDirectory::default();
    directory
        .register_identity_provider(external_oidc_provider_create())
        .expect("OIDC provider registers");
    let mut ledger = CloudIamIdentityProviderUpdateIdempotencyLedger::default();
    let request = identity_provider_update_api_request("req-idp-update", "idem-idp-update");

    let first =
        update_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, request.clone())
            .expect("provider-managed IdP updates through API");
    let second = update_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, request)
        .expect("same IdP update request replays idempotently");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(first.metadata.request_id, "req-idp-update");
    assert_eq!(first.data.identity_provider_id, "idp_partner_oidc");
    assert_eq!(first.data.tenant_id, "ten_alpha");
    assert_eq!(first.data.kind, CloudIamIdentityProviderKind::Saml);
    assert_eq!(first.data.issuer_uri, "https://partner.example/saml/v2");
    assert_eq!(first.data.verification_material_ref, "cert/partner-rotated");
    assert_eq!(first.data.created_at_epoch_seconds, 1_700_000_020);
    assert_eq!(
        iam_cloud_api::CLOUD_IAM_IDENTITY_PROVIDER_UPDATE_SURFACE,
        "cloud.iam.identity_provider.update"
    );
    assert_eq!(CloudIamIdentityProviderUpdateApiStatus::Ok.code(), 200);
    assert_eq!(
        CloudIamIdentityProviderUpdateApiStatus::BadRequest.code(),
        400
    );
    assert_eq!(
        CloudIamIdentityProviderUpdateApiStatus::Forbidden.code(),
        403
    );
    assert_eq!(
        CloudIamIdentityProviderUpdateApiStatus::Conflict.code(),
        409
    );
    assert_eq!(
        CloudIamIdentityProviderUpdateApiStatus::UnprocessableEntity.code(),
        422
    );

    let listed = list_cloud_iam_identity_providers_from_api(
        &directory,
        identity_provider_list_api_request("req-idp-update-list"),
    )
    .expect("updated provider remains tenant-scoped and listable");
    assert_eq!(listed.data.len(), 1);
    assert_eq!(listed.data[0].kind, CloudIamIdentityProviderKind::Saml);
    assert_eq!(
        listed.data[0].verification_material_ref,
        "cert/partner-rotated"
    );
}

#[test]
fn identity_provider_update_api_rejects_missing_cross_tenant_or_unauthorized_updates() {
    let mut directory = IamDirectory::default();
    directory
        .register_identity_provider(external_oidc_provider_create())
        .expect("OIDC provider registers");
    directory
        .register_identity_provider(beta_oidc_provider_create())
        .expect("beta provider registers");
    let mut ledger = CloudIamIdentityProviderUpdateIdempotencyLedger::default();

    let mut missing =
        identity_provider_update_api_request("req-idp-update-missing", "idem-missing");
    missing.path_identity_provider_id = "idp_missing_oidc".to_string();
    missing.body.identity_provider_id = "idp_missing_oidc".to_string();
    let missing_error =
        update_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, missing)
            .expect_err("missing provider cannot be updated");
    assert_eq!(
        missing_error,
        CloudIamApiError::Iam(CloudIamError::UnknownProvider)
    );
    assert_eq!(missing_error.identity_provider_update_status_code(), 400);

    let mut cross_tenant =
        identity_provider_update_api_request("req-idp-update-cross", "idem-cross");
    cross_tenant.path_identity_provider_id = "idp_beta_oidc".to_string();
    cross_tenant.body.identity_provider_id = "idp_beta_oidc".to_string();
    let cross_tenant_error =
        update_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, cross_tenant)
            .expect_err("cross-tenant provider cannot be updated by alpha principal");
    assert_eq!(
        cross_tenant_error,
        CloudIamApiError::Iam(CloudIamError::ProviderTenantMismatch)
    );
    assert_eq!(
        cross_tenant_error.identity_provider_update_status_code(),
        403
    );

    let mut unauthorized =
        identity_provider_update_api_request("req-idp-update-denied", "idem-denied");
    unauthorized.authorization = authorization_for(
        "sp_cloud_provisioner",
        &[iam_cloud_api::CLOUD_IAM_IDENTITY_PROVIDER_LIST_SURFACE],
    );
    let before_denied_ledger_len = ledger.len();
    assert_eq!(
        update_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, unauthorized),
        Err(CloudIamApiError::AuthorizationDenied {
            surface: iam_cloud_api::CLOUD_IAM_IDENTITY_PROVIDER_UPDATE_SURFACE.to_string(),
        })
    );
    assert_eq!(ledger.len(), before_denied_ledger_len);

    let mut drifted = identity_provider_update_api_request("req-idp-update-drift", "idem-drift");
    drifted.body.identity_provider_id = "idp_other_oidc".to_string();
    assert_eq!(
        update_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, drifted),
        Err(CloudIamApiError::ProviderIdMismatch {
            path_identity_provider_id: "idp_partner_oidc".to_string(),
            body_identity_provider_id: "idp_other_oidc".to_string(),
        })
    );
}

#[test]
fn identity_provider_create_api_rejects_path_body_provider_drift_before_ledger() {
    let mut directory = IamDirectory::default();
    let mut ledger = CloudIamIdentityProviderCreateIdempotencyLedger::default();
    let mut request = identity_provider_api_request("req-idp-drift", "idem-idp-drift");
    request.body.identity_provider_id = "idp_other_oidc".to_string();

    let result = create_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, request);

    assert_eq!(
        result,
        Err(CloudIamApiError::ProviderIdMismatch {
            path_identity_provider_id: "idp_partner_oidc".to_string(),
            body_identity_provider_id: "idp_other_oidc".to_string(),
        })
    );
    assert!(ledger.is_empty());
    directory
        .register_identity_provider(external_oidc_provider_create())
        .expect("path/body denial happened before directory mutation");
}

#[test]
fn identity_provider_mutation_api_rejects_missing_or_unsupported_oyatie_version_before_ledger() {
    let mut directory = IamDirectory::default();
    let mut create_ledger = CloudIamIdentityProviderCreateIdempotencyLedger::default();
    let mut missing_create =
        identity_provider_api_request("req-idp-version-missing", "idem-idp-version-missing");
    missing_create.boundary.oyatie_version = " ".to_string();
    missing_create.authorization.allowed_surfaces.clear();

    assert_eq!(
        create_cloud_iam_identity_provider_from_api(
            &mut directory,
            &mut create_ledger,
            missing_create
        ),
        Err(CloudIamApiError::MissingPublicApiVersion)
    );
    assert!(create_ledger.is_empty());
    directory
        .register_identity_provider(external_oidc_provider_create())
        .expect("missing version rejection happened before directory mutation");

    let mut directory = IamDirectory::default();
    let mut create_ledger = CloudIamIdentityProviderCreateIdempotencyLedger::default();
    let mut unsupported_create =
        identity_provider_api_request("req-idp-version-unsupported", "idem-idp-version-bad");
    unsupported_create.boundary.oyatie_version = "2026-01-01".to_string();
    unsupported_create.authorization.allowed_surfaces.clear();

    assert_eq!(
        create_cloud_iam_identity_provider_from_api(
            &mut directory,
            &mut create_ledger,
            unsupported_create
        ),
        Err(CloudIamApiError::UnsupportedPublicApiVersion {
            oyatie_version: "2026-01-01".to_string(),
        })
    );
    assert!(create_ledger.is_empty());
    directory
        .register_identity_provider(external_oidc_provider_create())
        .expect("unsupported version rejection happened before directory mutation");

    let mut directory = IamDirectory::default();
    let mut update_ledger = CloudIamIdentityProviderUpdateIdempotencyLedger::default();
    let mut missing_update = identity_provider_update_api_request(
        "req-idp-update-version-missing",
        "idem-update-missing",
    );
    missing_update.boundary.oyatie_version = String::new();
    missing_update.authorization.allowed_surfaces.clear();
    assert_eq!(
        update_cloud_iam_identity_provider_from_api(
            &mut directory,
            &mut update_ledger,
            missing_update
        ),
        Err(CloudIamApiError::MissingPublicApiVersion)
    );
    assert!(update_ledger.is_empty());

    let mut unsupported_update = identity_provider_update_api_request(
        "req-idp-update-version-unsupported",
        "idem-update-unsupported",
    );
    unsupported_update.boundary.oyatie_version = "not-a-date".to_string();
    unsupported_update.authorization.allowed_surfaces.clear();
    assert_eq!(
        update_cloud_iam_identity_provider_from_api(
            &mut directory,
            &mut update_ledger,
            unsupported_update
        ),
        Err(CloudIamApiError::UnsupportedPublicApiVersion {
            oyatie_version: "not-a-date".to_string(),
        })
    );
    assert!(update_ledger.is_empty());

    let mut delete_ledger = CloudIamIdentityProviderDeleteIdempotencyLedger::default();
    let mut missing_delete = identity_provider_delete_api_request(
        "req-idp-delete-version-missing",
        "idem-delete-missing",
    );
    missing_delete.boundary.oyatie_version = "\t".to_string();
    missing_delete.authorization.allowed_surfaces.clear();
    assert_eq!(
        delete_cloud_iam_identity_provider_from_api(
            &mut directory,
            &mut delete_ledger,
            missing_delete
        ),
        Err(CloudIamApiError::MissingPublicApiVersion)
    );
    assert!(delete_ledger.is_empty());

    let mut unsupported_delete = identity_provider_delete_api_request(
        "req-idp-delete-version-unsupported",
        "idem-delete-unsupported",
    );
    unsupported_delete.boundary.oyatie_version = "2026-01-01".to_string();
    unsupported_delete.authorization.allowed_surfaces.clear();
    assert_eq!(
        delete_cloud_iam_identity_provider_from_api(
            &mut directory,
            &mut delete_ledger,
            unsupported_delete
        ),
        Err(CloudIamApiError::UnsupportedPublicApiVersion {
            oyatie_version: "2026-01-01".to_string(),
        })
    );
    assert!(delete_ledger.is_empty());
}

#[test]
fn identity_provider_create_api_accepts_manifest_public_versions_and_keys_by_version() {
    for (index, oyatie_version) in CLOUD_IAM_SUPPORTED_PUBLIC_API_VERSIONS
        .iter()
        .copied()
        .enumerate()
    {
        let mut directory = IamDirectory::default();
        let mut ledger = CloudIamIdentityProviderCreateIdempotencyLedger::default();
        let mut request = identity_provider_api_request(
            &format!("req-idp-version-supported-{index}"),
            &format!("idem-idp-version-supported-{index}"),
        );
        request.boundary.oyatie_version = oyatie_version.to_string();

        let response =
            create_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, request)
                .expect("manifest-declared public API version is accepted");

        assert_eq!(response.data.identity_provider_id, "idp_partner_oidc");
        assert_eq!(ledger.len(), 1);
    }

    let mut directory = IamDirectory::default();
    let mut ledger = CloudIamIdentityProviderCreateIdempotencyLedger::default();
    let request = identity_provider_api_request("req-idp-version-default", "idem-idp-version-key");
    create_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, request.clone())
        .expect("default public API version succeeds");

    let mut version_drifted = request;
    version_drifted.boundary.oyatie_version = "2026-02-21".to_string();
    assert_eq!(
        create_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, version_drifted),
        Err(CloudIamApiError::IdempotencyKeyReused {
            idempotency_key: "idem-idp-version-key".to_string(),
        })
    );
    assert_eq!(ledger.len(), 1);
}

#[test]
fn iam_api_rejects_missing_typed_cell_or_region_boundary_before_ledger() {
    let mut directory = IamDirectory::default();
    let mut ledger = CloudIamIdentityProviderCreateIdempotencyLedger::default();
    let mut missing_cell =
        identity_provider_api_request("req-idp-cell-missing", "idem-idp-cell-missing");
    missing_cell.boundary.placement.cell_id.value = " ".to_string();
    missing_cell.authorization.allowed_surfaces.clear();

    assert_eq!(
        create_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, missing_cell),
        Err(CloudIamApiError::EmptyBoundaryCell)
    );
    assert!(ledger.is_empty());
    directory
        .register_identity_provider(external_oidc_provider_create())
        .expect("missing typed cell boundary rejected before directory mutation");

    let mut directory = directory_with_principals();
    let mut ledger = CloudIamRoleCreateIdempotencyLedger::default();
    let mut missing_region =
        role_api_request("req-role-region-missing", "idem-role-region-missing");
    missing_region.boundary.placement.region_id.value = String::new();
    missing_region.authorization.allowed_surfaces.clear();

    assert_eq!(
        create_cloud_iam_role_from_api(&mut directory, &mut ledger, missing_region),
        Err(CloudIamApiError::EmptyBoundaryRegion)
    );
    assert!(ledger.is_empty());
    directory
        .create_role(role_create())
        .expect("missing typed region boundary rejected before directory mutation");
}

#[test]
fn iam_api_rejects_typed_boundary_tenant_drift_before_authorization_or_domain() {
    let mut directory = IamDirectory::default();
    let mut ledger = CloudIamIdentityProviderCreateIdempotencyLedger::default();
    let mut request =
        identity_provider_api_request("req-idp-placement-tenant", "idem-idp-placement-tenant");
    request.boundary.placement.tenant_id.value = "ten_beta".to_string();
    request.authorization.allowed_surfaces.clear();

    assert_eq!(
        create_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, request),
        Err(CloudIamApiError::PlacementTenantMismatch {
            header_tenant_id: "ten_alpha".to_string(),
            placement_tenant_id: "ten_beta".to_string(),
        })
    );
    assert!(ledger.is_empty());
    directory
        .register_identity_provider(external_oidc_provider_create())
        .expect("tenant placement drift rejected before directory mutation");
}

#[test]
fn iam_api_keys_idempotency_fingerprints_by_typed_placement_boundary() {
    let mut directory = IamDirectory::default();
    let mut ledger = CloudIamIdentityProviderCreateIdempotencyLedger::default();
    let request = identity_provider_api_request("req-idp-placement-key", "idem-idp-placement-key");
    create_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, request.clone())
        .expect("typed placement boundary succeeds");

    let mut cell_drifted = request;
    cell_drifted.boundary.placement.cell_id.value = "cell-alpha-region-a-002".to_string();
    assert_eq!(
        create_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, cell_drifted),
        Err(CloudIamApiError::IdempotencyKeyReused {
            idempotency_key: "idem-idp-placement-key".to_string(),
        })
    );
    assert_eq!(ledger.len(), 1);
}

#[test]
fn identity_provider_create_api_registers_oidc_provider_once_and_replays() {
    let mut directory = IamDirectory::default();
    directory
        .create_principal(service_principal_create())
        .expect("provisioner principal exists");
    let mut ledger = CloudIamIdentityProviderCreateIdempotencyLedger::default();
    let request = identity_provider_api_request("req-idp-create", "idem-idp-create");

    let first =
        create_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, request.clone())
            .expect("provider-managed OIDC IdP registers through API");
    let second = create_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, request)
        .expect("same IdP create request replays idempotently");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(first.data.identity_provider_id, "idp_partner_oidc");
    assert_eq!(first.data.kind, CloudIamIdentityProviderKind::Oidc);
    assert_eq!(first.data.verification_material_ref, "jwks/partner");
    assert_eq!(first.metadata.request_id, "req-idp-create");
    assert_eq!(
        CLOUD_IAM_IDENTITY_PROVIDER_CREATE_SURFACE,
        "cloud.iam.identity_provider.create"
    );
    assert_eq!(CloudIamIdentityProviderCreateApiStatus::Created.code(), 201);
    assert_eq!(
        CloudIamIdentityProviderCreateApiStatus::BadRequest.code(),
        400
    );
    assert_eq!(
        CloudIamIdentityProviderCreateApiStatus::Forbidden.code(),
        403
    );
    assert_eq!(
        CloudIamIdentityProviderCreateApiStatus::Conflict.code(),
        409
    );
    assert_eq!(
        CloudIamIdentityProviderCreateApiStatus::UnprocessableEntity.code(),
        422
    );

    directory
        .create_principal(external_principal_create())
        .expect("registered provider can bind external principal");
}

#[test]
fn identity_provider_create_api_maps_duplicate_and_reused_idempotency_key() {
    let mut directory = IamDirectory::default();
    directory
        .register_identity_provider(external_oidc_provider_create())
        .expect("provider exists");
    let mut ledger = CloudIamIdentityProviderCreateIdempotencyLedger::default();

    let duplicate = create_cloud_iam_identity_provider_from_api(
        &mut directory,
        &mut ledger,
        identity_provider_api_request("req-idp-duplicate", "idem-idp-duplicate"),
    )
    .expect_err("duplicate provider maps to conflict");
    assert_eq!(duplicate.identity_provider_create_status_code(), 409);
    assert_eq!(
        duplicate,
        CloudIamApiError::Iam(CloudIamError::DuplicateProvider)
    );

    let mut directory = IamDirectory::default();
    let request = identity_provider_api_request("req-idp-create", "idem-idp-create");
    create_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, request.clone())
        .expect("new provider registers");
    let mut drifted = request;
    drifted.body.audience = "urn:oyatie:changed".to_string();
    assert_eq!(
        create_cloud_iam_identity_provider_from_api(&mut directory, &mut ledger, drifted),
        Err(CloudIamApiError::IdempotencyKeyReused {
            idempotency_key: "idem-idp-create".to_string(),
        })
    );
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
    assert!(
        directory
            .create_role(role_create())
            .expect("directory stayed mutable")
            .id
            .value
            .value
            .starts_with("role_")
    );
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
    empty_request.boundary.placement.tenant_id.value = "ten_other".to_string();
    assert_eq!(
        create_cloud_iam_role_from_api(&mut directory, &mut ledger, empty_request),
        Err(CloudIamApiError::TenantMismatch {
            header_tenant_id: "ten_other".to_string(),
            principal_tenant_id: "ten_alpha".to_string(),
            body_tenant_id: "ten_alpha".to_string(),
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
    assert_eq!(first.data.region, "region-home");
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
            principal_tenant_id: "ten_alpha".to_string(),
            principal_id: "sp_cloud_provisioner".to_string(),
            body_tenant_id: "ten_alpha".to_string(),
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

#[test]
fn sts_token_api_issues_external_oidc_session_with_external_id() {
    let mut directory = directory_with_external_role();
    let mut ledger = CloudIamStsTokenIdempotencyLedger::default();
    let request = external_sts_api_request("req-sts-external", "idem-sts-external");

    let first = issue_cloud_iam_sts_token_from_api(&mut directory, &mut ledger, request.clone())
        .expect("external OIDC principal can receive scoped STS token");
    let second = issue_cloud_iam_sts_token_from_api(&mut directory, &mut ledger, request)
        .expect("same external STS request replays idempotently");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(first.data.session_id, "sts_external_partner_001");
    assert_eq!(first.data.assumed_by, "sp_external_partner");
    assert_eq!(
        first.data.external_id.as_deref(),
        Some("external-customer-alpha")
    );
    assert_eq!(first.data.expires_at_epoch_seconds, 1_700_000_400);
    assert_eq!(first.data.scopes[0].value, "cloud.iam.read");
    assert!(first.data.token_fingerprint.starts_with("sts1:"));
}

#[test]
fn sts_token_api_maps_external_id_policy_denial_to_forbidden() {
    let mut directory = directory_with_external_role();
    let mut ledger = CloudIamStsTokenIdempotencyLedger::default();
    let mut request = external_sts_api_request("req-sts-external-id", "idem-sts-external-id");
    request.body.session_id = "sts_external_partner_no_id".to_string();
    request.body.external_id = None;

    let error = issue_cloud_iam_sts_token_from_api(&mut directory, &mut ledger, request)
        .expect_err("external OIDC principal requires external_id");

    assert_eq!(error.sts_token_status_code(), 403);
    assert_eq!(
        error,
        CloudIamApiError::Iam(CloudIamError::ExternalIdRequired)
    );
    assert_eq!(ledger.len(), 1);
}
