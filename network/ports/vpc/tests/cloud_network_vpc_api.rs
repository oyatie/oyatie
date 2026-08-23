// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use network_domain::{CloudNetworkCatalog, CloudNetworkError};
use network_vpc::{
    CLOUD_NETWORK_VPC_CREATE_SURFACE, CallerCredential, CloudNetworkVpcApiBoundaryContext,
    CloudNetworkVpcApiError, CloudNetworkVpcApiPrincipal, CloudNetworkVpcAuthzProvider,
    CloudNetworkVpcCreateApiRequest, CloudNetworkVpcCreateApiStatus,
    CloudNetworkVpcCreateIdempotencyLedger, CloudNetworkVpcCreateRequest,
    CloudNetworkVpcRouteCreateRequest, CloudNetworkVpcRouteTableCreateRequest,
    CloudNetworkVpcSecurityGroupCreateRequest, CloudNetworkVpcSecurityRuleCreateRequest,
    ConfiguredBearerPrincipalVerifier, VerifiedPrincipal, VpcCreateAuthorizationError,
    VpcCreateAuthorizer, VpcCreateResource, create_cloud_network_vpc_from_api,
};

const VPC_ID: &str = "oyatie:cloud:region-home:ten_alpha:vpc:prod";
const PRINCIPAL_ID: &str = "sp_network_admin";
const TENANT_ID: &str = "ten_alpha";
const BEARER_SECRET: &str = "break-glass-network-vpc-secret";

/// A test PDP that allows everything — used to PROVE the verified-principal /
/// tenant cross-check and credential gate fail-close even when the PDP would
/// otherwise allow (blast-radius binding is independent of the PDP verdict).
struct AllowAllAuthorizer;
impl VpcCreateAuthorizer for AllowAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &VpcCreateResource,
    ) -> Result<(), VpcCreateAuthorizationError> {
        Ok(())
    }
}

/// A test PDP that denies everything (proves PDP-deny → 403).
struct DenyAllAuthorizer;
impl VpcCreateAuthorizer for DenyAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &VpcCreateResource,
    ) -> Result<(), VpcCreateAuthorizationError> {
        Err(VpcCreateAuthorizationError::Denied)
    }
}

/// A test PDP that refuses (fault) — proves a PDP fault is fail-closed → 403.
struct RefuseAuthorizer;
impl VpcCreateAuthorizer for RefuseAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &VpcCreateResource,
    ) -> Result<(), VpcCreateAuthorizationError> {
        Err(VpcCreateAuthorizationError::Refused)
    }
}

/// A PDP that authorizes for a SPECIFIC target tenant only — proves the PDP
/// receives the TARGET tenant (no IDOR / blast-radius binding).
struct TenantScopedAuthorizer {
    allowed_tenant: String,
}
impl VpcCreateAuthorizer for TenantScopedAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        resource: &VpcCreateResource,
    ) -> Result<(), VpcCreateAuthorizationError> {
        if resource.tenant_id == self.allowed_tenant {
            Ok(())
        } else {
            Err(VpcCreateAuthorizationError::Denied)
        }
    }
}

fn provider_with(authorizer: Arc<dyn VpcCreateAuthorizer>) -> CloudNetworkVpcAuthzProvider {
    let verifier = Arc::new(
        ConfiguredBearerPrincipalVerifier::new(BEARER_SECRET, PRINCIPAL_ID, TENANT_ID)
            .expect("valid break-glass verifier config"),
    );
    CloudNetworkVpcAuthzProvider::new(verifier, authorizer)
}

fn allow_all_provider() -> CloudNetworkVpcAuthzProvider {
    provider_with(Arc::new(AllowAllAuthorizer))
}

fn valid_credential() -> CallerCredential {
    CallerCredential {
        authorization: Some(format!("Bearer {BEARER_SECRET}")),
        claimed_principal_id: PRINCIPAL_ID.to_string(),
        claimed_tenant_id: TENANT_ID.to_string(),
    }
}

fn boundary_for(request_id: &str, idempotency_key: &str) -> CloudNetworkVpcApiBoundaryContext {
    CloudNetworkVpcApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: TENANT_ID.to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal_for(principal_id: &str) -> CloudNetworkVpcApiPrincipal {
    CloudNetworkVpcApiPrincipal {
        tenant_id: TENANT_ID.to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn create_body(resource_id: &str) -> CloudNetworkVpcCreateRequest {
    CloudNetworkVpcCreateRequest {
        resource_id: resource_id.to_string(),
        tenant_id: TENANT_ID.to_string(),
        region: "region-home".to_string(),
        cidr_v4: "10.42.0.0/16".to_string(),
        cidr_v6: "2001:db8:42::/56".to_string(),
        flow_logs_enabled: true,
        route_table: CloudNetworkVpcRouteTableCreateRequest {
            id: "rtb_main".to_string(),
            routes: vec![
                CloudNetworkVpcRouteCreateRequest {
                    destination: "10.42.0.0/16".to_string(),
                    next_hop: "local".to_string(),
                    target_ref: None,
                },
                CloudNetworkVpcRouteCreateRequest {
                    destination: "0.0.0.0/0".to_string(),
                    next_hop: "internet_gateway".to_string(),
                    target_ref: Some("igw/prod".to_string()),
                },
            ],
        },
        security_groups: vec![CloudNetworkVpcSecurityGroupCreateRequest {
            id: "sg_web".to_string(),
            rules: vec![CloudNetworkVpcSecurityRuleCreateRequest {
                direction: "ingress".to_string(),
                protocol: "tcp".to_string(),
                port_start: Some(443),
                port_end: Some(443),
                cidr: "10.42.0.0/16".to_string(),
                description: "tenant https ingress".to_string(),
            }],
        }],
        residency: "strict_home_region".to_string(),
        data_class: "PUBLIC".to_string(),
        created_at_epoch_seconds: 1_700_000_000,
    }
}

fn create_request(request_id: &str, idempotency_key: &str) -> CloudNetworkVpcCreateApiRequest {
    CloudNetworkVpcCreateApiRequest {
        path_vpc_id: VPC_ID.to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for(PRINCIPAL_ID),
        credential: valid_credential(),
        body: create_body(VPC_ID),
    }
}

#[test]
fn openapi_runtime_binding_contracts_are_covered() {
    assert_eq!(CLOUD_NETWORK_VPC_CREATE_SURFACE, "cloud.network.vpc.create");
    assert_eq!(CloudNetworkVpcCreateApiStatus::Created.code(), 201);
    assert_eq!(CloudNetworkVpcCreateApiStatus::BadRequest.code(), 400);
    assert_eq!(CloudNetworkVpcCreateApiStatus::Unauthorized.code(), 401);
    assert_eq!(CloudNetworkVpcCreateApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudNetworkVpcCreateApiStatus::NotFound.code(), 404);
    assert_eq!(CloudNetworkVpcCreateApiStatus::Conflict.code(), 409);
    assert_eq!(
        CloudNetworkVpcCreateApiStatus::UnprocessableEntity.code(),
        422
    );
}

// ===========================================================================
// FAIL-CLOSED AUTHZ SEAM (C10 / ADR-0587) — RED/GREEN tests that MUST fail if
// the verified-principal + PDP gate is removed.
// ===========================================================================

#[test]
fn vpc_create_api_rejects_absent_credential_as_401() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let provider = allow_all_provider(); // PDP would allow — the credential gate must still block.
    let mut request = create_request("req-vpc-no-cred", "idem-vpc-no-cred");
    request.credential.authorization = None;

    let error = create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, &provider, request)
        .expect_err("absent credential must be rejected");

    assert_eq!(error, CloudNetworkVpcApiError::CallerUnauthenticated);
    assert_eq!(error.vpc_create_status_code(), 401);
    assert!(ledger.is_empty());
    assert_eq!(catalog.vpcs().count(), 0);
}

#[test]
fn vpc_create_api_rejects_forged_bearer_as_401() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let mut request = create_request("req-vpc-forged", "idem-vpc-forged");
    request.credential.authorization = Some("Bearer not-the-real-secret".to_string());

    let error = create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, &provider, request)
        .expect_err("forged bearer must be rejected");

    assert_eq!(error, CloudNetworkVpcApiError::CallerUnauthenticated);
    assert_eq!(error.vpc_create_status_code(), 401);
    assert!(ledger.is_empty());
    assert_eq!(catalog.vpcs().count(), 0);
}

#[test]
fn vpc_create_api_rejects_request_principal_not_matching_verified_identity_as_403() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let mut request = create_request("req-vpc-princ", "idem-vpc-princ");
    request.principal.principal_id = "sp_someone_else".to_string();

    let error = create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, &provider, request)
        .expect_err("request principal must match verified identity");

    assert_eq!(
        error,
        CloudNetworkVpcApiError::VerifiedPrincipalMismatch {
            verified_principal_id: PRINCIPAL_ID.to_string(),
            request_principal_id: "sp_someone_else".to_string(),
        }
    );
    assert_eq!(error.vpc_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.vpcs().count(), 0);
}

#[test]
fn vpc_create_api_denies_cross_tenant_proving_blast_radius_binding() {
    // The PDP only authorizes for ten_alpha. A caller verified as ten_alpha
    // whose body targets ten_beta is denied because the resource carries the
    // TARGET tenant (no IDOR / no flatten-to-caller).
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let provider = provider_with(Arc::new(TenantScopedAuthorizer {
        allowed_tenant: TENANT_ID.to_string(),
    }));
    let mut request = create_request("req-vpc-xtenant", "idem-vpc-xtenant");
    request.boundary.tenant_id = "ten_beta".to_string();
    request.principal.tenant_id = "ten_beta".to_string();
    request.body.tenant_id = "ten_beta".to_string();

    let error = create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, &provider, request)
        .expect_err("cross-tenant create must be denied");

    assert_eq!(
        error,
        CloudNetworkVpcApiError::VerifiedTenantMismatch {
            verified_tenant_id: TENANT_ID.to_string(),
            request_tenant_id: "ten_beta".to_string(),
        }
    );
    assert_eq!(error.vpc_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.vpcs().count(), 0);
}

#[test]
fn vpc_create_api_denies_when_pdp_denies_as_403() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let provider = provider_with(Arc::new(DenyAllAuthorizer));
    let request = create_request("req-vpc-deny", "idem-vpc-deny");

    let error = create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, &provider, request)
        .expect_err("PDP deny must be 403");

    assert_eq!(
        error,
        CloudNetworkVpcApiError::AuthorizationDenied {
            surface: CLOUD_NETWORK_VPC_CREATE_SURFACE.to_string(),
        }
    );
    assert_eq!(error.vpc_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.vpcs().count(), 0);
}

#[test]
fn vpc_create_api_fails_closed_when_pdp_faults_as_403_not_5xx() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let provider = provider_with(Arc::new(RefuseAuthorizer));
    let request = create_request("req-vpc-fault", "idem-vpc-fault");

    let error = create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, &provider, request)
        .expect_err("PDP fault must fail closed to 403");

    assert_eq!(
        error,
        CloudNetworkVpcApiError::AuthorizationDenied {
            surface: CLOUD_NETWORK_VPC_CREATE_SURFACE.to_string(),
        }
    );
    assert_eq!(error.vpc_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.vpcs().count(), 0);
}

#[test]
fn vpc_create_api_creates_vpc_once_and_replays_same_idempotent_result() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let request = create_request("req-network-vpc-create", "idem-network-vpc-create");

    let first =
        create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, &provider, request.clone())
            .expect("authorized VPC create succeeds");
    let second = create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, &provider, request)
        .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.vpcs().count(), 1);
    assert_eq!(first.metadata.request_id, "req-network-vpc-create");
    assert_eq!(first.data.resource_id, VPC_ID);
    assert_eq!(first.data.tenant_id, TENANT_ID);
    assert_eq!(first.data.region, "region-home");
    assert_eq!(first.data.cidr_v4, "10.42.0.0/16");
    assert_eq!(first.data.cidr_v6, "2001:db8:42::/56");
    assert!(first.data.flow_logs_enabled);
    assert_eq!(first.data.route_count, 2);
    assert_eq!(first.data.security_group_count, 1);
    assert_eq!(first.data.residency, "strict_home_region");
    assert_eq!(first.data.data_class, "PUBLIC");
    assert_eq!(first.data.state, "creating");
    assert_eq!(first.data.schema_version, 1);
}

#[test]
fn vpc_create_api_rejects_path_body_vpc_drift_before_catalog_mutation() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let mut request = create_request("req-network-vpc-drift", "idem-network-vpc-drift");
    request.body.resource_id = "oyatie:cloud:region-home:ten_alpha:vpc:other".to_string();

    let error = create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, &provider, request)
        .expect_err("path/body VPC drift is rejected");

    assert_eq!(
        error,
        CloudNetworkVpcApiError::VpcIdMismatch {
            path_vpc_id: VPC_ID.to_string(),
            body_resource_id: "oyatie:cloud:region-home:ten_alpha:vpc:other".to_string(),
        }
    );
    assert_eq!(error.vpc_create_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(catalog.vpcs().count(), 0);
}

#[test]
fn vpc_create_api_rejects_missing_principal_as_401() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let mut unauthenticated = create_request("req-network-vpc-authn", "idem-network-vpc-authn");
    unauthenticated.principal.principal_id = " ".to_string();

    let authn_error =
        create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, &provider, unauthenticated)
            .expect_err("missing principal is authentication failure");

    assert_eq!(authn_error, CloudNetworkVpcApiError::EmptyPrincipalId);
    assert_eq!(authn_error.vpc_create_status_code(), 401);
    assert!(ledger.is_empty());
    assert_eq!(catalog.vpcs().count(), 0);
}

#[test]
fn vpc_create_api_rejects_required_header_and_tenant_drift_before_ledger() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let mut request = create_request(" ", "idem-network-vpc-empty-header");
    assert_eq!(
        create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, &provider, request.clone()),
        Err(CloudNetworkVpcApiError::EmptyRequestId)
    );

    request.boundary.request_id = "req-network-vpc-tenant-drift".to_string();
    request.boundary.tenant_id = "ten_other".to_string();
    assert_eq!(
        create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, &provider, request),
        Err(CloudNetworkVpcApiError::TenantMismatch {
            header_tenant_id: "ten_other".to_string(),
            principal_tenant_id: TENANT_ID.to_string(),
            body_tenant_id: TENANT_ID.to_string(),
        })
    );
    assert!(ledger.is_empty());
    assert_eq!(catalog.vpcs().count(), 0);
}

#[test]
fn vpc_create_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let request = create_request("req-network-vpc-idem", "idem-network-vpc-idem");
    create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, &provider, request.clone())
        .expect("initial create succeeds");

    let mut drifted = request;
    drifted.body.cidr_v4 = "10.43.0.0/16".to_string();
    assert_eq!(
        create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, &provider, drifted),
        Err(CloudNetworkVpcApiError::IdempotencyKeyReused {
            idempotency_key: "idem-network-vpc-idem".to_string(),
        })
    );
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.vpcs().count(), 1);
}

#[test]
fn vpc_create_api_maps_duplicate_vpc_to_conflict() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    create_cloud_network_vpc_from_api(
        &mut catalog,
        &mut ledger,
        &provider,
        create_request("req-network-vpc-dup-1", "idem-network-vpc-dup-1"),
    )
    .expect("first VPC create succeeds");

    let error = create_cloud_network_vpc_from_api(
        &mut catalog,
        &mut ledger,
        &provider,
        create_request("req-network-vpc-dup-2", "idem-network-vpc-dup-2"),
    )
    .expect_err("same VPC id through a new idempotency key conflicts");

    assert_eq!(
        error,
        CloudNetworkVpcApiError::Network(CloudNetworkError::DuplicateVpc)
    );
    assert_eq!(error.vpc_create_status_code(), 409);
    assert_eq!(catalog.vpcs().count(), 1);
}

#[test]
fn vpc_create_api_maps_flow_log_and_residency_invariants() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let mut no_flow_logs = create_request("req-network-vpc-flow", "idem-network-vpc-flow");
    no_flow_logs.body.flow_logs_enabled = false;

    let flow_error =
        create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, &provider, no_flow_logs)
            .expect_err("flow logs are mandatory for VPC creation");

    assert_eq!(
        flow_error,
        CloudNetworkVpcApiError::Network(CloudNetworkError::FlowLogsRequired)
    );
    assert_eq!(flow_error.vpc_create_status_code(), 400);
    assert_eq!(catalog.vpcs().count(), 0);

    let mut residency_drift = create_request("req-network-vpc-res", "idem-network-vpc-res");
    residency_drift.body.region = "failover-region".to_string();
    let residency_error =
        create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, &provider, residency_drift)
            .expect_err("strict home-region residency cannot create a US VPC");

    assert_eq!(
        residency_error,
        CloudNetworkVpcApiError::Network(CloudNetworkError::ResourceRegionMismatch)
    );
    assert_eq!(residency_error.vpc_create_status_code(), 403);
    assert_eq!(catalog.vpcs().count(), 0);
}

#[test]
fn vpc_create_api_rejects_invalid_route_and_rule_labels_without_catalog_mutation() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let mut invalid_route = create_request("req-network-vpc-route", "idem-network-vpc-route");
    invalid_route.body.route_table.routes[0].next_hop = "sidecar".to_string();

    let route_error =
        create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, &provider, invalid_route)
            .expect_err("unknown next hop is rejected before catalog mutation");

    assert_eq!(
        route_error,
        CloudNetworkVpcApiError::InvalidRouteNextHopLabel {
            next_hop: "sidecar".to_string(),
        }
    );
    assert_eq!(route_error.vpc_create_status_code(), 400);

    let mut invalid_rule = create_request("req-network-vpc-rule", "idem-network-vpc-rule");
    invalid_rule.body.security_groups[0].rules[0].port_end = Some(80);
    let rule_error =
        create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, &provider, invalid_rule)
            .expect_err("invalid port interval is rejected before catalog mutation");

    assert_eq!(rule_error, CloudNetworkVpcApiError::InvalidPortRange);
    assert_eq!(rule_error.vpc_create_status_code(), 400);
    assert_eq!(catalog.vpcs().count(), 0);
}
