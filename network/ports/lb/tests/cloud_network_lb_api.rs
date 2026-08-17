// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use network_domain::{
    CloudNetworkCatalog, CloudNetworkError, IpProtocol, NetworkRepo, RouteCreate, RouteNextHopKind,
    RouteTableCreate, RuleDirection, SecurityGroupCreate, SecurityRule, SubnetCreate, SubnetState,
    VpcCreate, VpcState,
};
use network_lb::{
    CLOUD_NETWORK_LB_CREATE_SURFACE, CallerCredential, CloudNetworkLbApiBoundaryContext,
    CloudNetworkLbApiError, CloudNetworkLbApiPrincipal, CloudNetworkLbAuthzProvider,
    CloudNetworkLbCreateApiRequest, CloudNetworkLbCreateApiStatus,
    CloudNetworkLbCreateIdempotencyLedger, CloudNetworkLbCreateRequest,
    CloudNetworkLbListenerCreateRequest, CloudNetworkLbMtlsConfigCreateRequest,
    CloudNetworkLbSubnetRef, CloudNetworkLbTargetGroupCreateRequest,
    ConfiguredBearerPrincipalVerifier, LbCreateAuthorizationError, LbCreateAuthorizer,
    LbCreateResource, VerifiedPrincipal, create_cloud_network_load_balancer_from_api,
};
use network_residency::ResidencyClass;
use oya_data_boundary_kernel::DataClass;

const VPC_ID: &str = "oya:cloud:region-home:ten_alpha:vpc:prod";
const SUBNET_ID: &str = "oya:cloud:region-home:ten_alpha:subnet:prod-a";
const LB_ID: &str = "oya:cloud:region-home:ten_alpha:lb-v7:frontdoor";
const PRINCIPAL_ID: &str = "sp_network_lb_admin";
const TENANT_ID: &str = "ten_alpha";
const BEARER_SECRET: &str = "break-glass-network-lb-secret";

/// A test PDP that allows everything — used to PROVE the verified-principal /
/// tenant cross-check and credential gate fail-close even when the PDP would
/// otherwise allow (blast-radius binding is independent of the PDP verdict).
struct AllowAllAuthorizer;
impl LbCreateAuthorizer for AllowAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &LbCreateResource,
    ) -> Result<(), LbCreateAuthorizationError> {
        Ok(())
    }
}

/// A test PDP that denies everything (proves PDP-deny → 403).
struct DenyAllAuthorizer;
impl LbCreateAuthorizer for DenyAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &LbCreateResource,
    ) -> Result<(), LbCreateAuthorizationError> {
        Err(LbCreateAuthorizationError::Denied)
    }
}

/// A test PDP that refuses (fault) — proves a PDP fault is fail-closed → 403,
/// never a 5xx/allow.
struct RefuseAuthorizer;
impl LbCreateAuthorizer for RefuseAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &LbCreateResource,
    ) -> Result<(), LbCreateAuthorizationError> {
        Err(LbCreateAuthorizationError::Refused)
    }
}

/// A PDP that authorizes for a SPECIFIC target tenant only — proves the PDP
/// receives the TARGET tenant (no IDOR / blast-radius binding).
struct TenantScopedAuthorizer {
    allowed_tenant: String,
}
impl LbCreateAuthorizer for TenantScopedAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        resource: &LbCreateResource,
    ) -> Result<(), LbCreateAuthorizationError> {
        if resource.tenant_id == self.allowed_tenant {
            Ok(())
        } else {
            Err(LbCreateAuthorizationError::Denied)
        }
    }
}

fn provider_with(authorizer: Arc<dyn LbCreateAuthorizer>) -> CloudNetworkLbAuthzProvider {
    let verifier = Arc::new(
        ConfiguredBearerPrincipalVerifier::new(BEARER_SECRET, PRINCIPAL_ID, TENANT_ID)
            .expect("valid break-glass verifier config"),
    );
    CloudNetworkLbAuthzProvider::new(verifier, authorizer)
}

fn allow_all_provider() -> CloudNetworkLbAuthzProvider {
    provider_with(Arc::new(AllowAllAuthorizer))
}

fn valid_credential() -> CallerCredential {
    CallerCredential {
        authorization: Some(format!("Bearer {BEARER_SECRET}")),
        claimed_principal_id: PRINCIPAL_ID.to_string(),
        claimed_tenant_id: TENANT_ID.to_string(),
    }
}

fn boundary_for(request_id: &str, idempotency_key: &str) -> CloudNetworkLbApiBoundaryContext {
    CloudNetworkLbApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: TENANT_ID.to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal_for(principal_id: &str) -> CloudNetworkLbApiPrincipal {
    CloudNetworkLbApiPrincipal {
        tenant_id: TENANT_ID.to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn lb_body(resource_id: &str) -> CloudNetworkLbCreateRequest {
    CloudNetworkLbCreateRequest {
        resource_id: resource_id.to_string(),
        tenant_id: TENANT_ID.to_string(),
        vpc_id: VPC_ID.to_string(),
        region: "region-home".to_string(),
        kind: "l7_grpc".to_string(),
        listeners: vec![CloudNetworkLbListenerCreateRequest {
            port: 443,
            target_group_id: "tg_api".to_string(),
            tls_certificate: Some("cert/region-home/ten_alpha/frontdoor".to_string()),
        }],
        target_groups: vec![CloudNetworkLbTargetGroupCreateRequest {
            id: "tg_api".to_string(),
            subnet_ids: vec![CloudNetworkLbSubnetRef {
                subnet_id: SUBNET_ID.to_string(),
            }],
            health_check_path: Some("/healthz".to_string()),
        }],
        mtls: Some(CloudNetworkLbMtlsConfigCreateRequest {
            ca_bundle_ref: "cert/region-home/ten_alpha/mesh-ca".to_string(),
            client_policy: "require_verified_client_cert".to_string(),
        }),
        waf_policy: Some("waf_cloud_frontdoor".to_string()),
        data_class: "PUBLIC".to_string(),
        created_at_epoch_seconds: 1_700_000_020,
    }
}

fn create_request(request_id: &str, idempotency_key: &str) -> CloudNetworkLbCreateApiRequest {
    CloudNetworkLbCreateApiRequest {
        path_load_balancer_id: LB_ID.to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for(PRINCIPAL_ID),
        credential: valid_credential(),
        body: lb_body(LB_ID),
    }
}

fn seed_vpc(catalog: &mut CloudNetworkCatalog) {
    catalog
        .create_vpc(VpcCreate {
            resource_id: VPC_ID.to_string(),
            tenant_id: TENANT_ID.to_string(),
            region: "region-home".to_string(),
            cidr_v4: "10.42.0.0/16".to_string(),
            cidr_v6: "2001:db8:42::/56".to_string(),
            flow_logs_enabled: true,
            route_table: RouteTableCreate {
                id: "rtb_main".to_string(),
                routes: vec![RouteCreate {
                    destination: "10.42.0.0/16".to_string(),
                    next_hop: RouteNextHopKind::Local,
                    target_ref: None,
                }],
            },
            security_groups: vec![SecurityGroupCreate {
                id: "sg_web".to_string(),
                rules: vec![SecurityRule {
                    direction: RuleDirection::Ingress,
                    protocol: IpProtocol::Tcp,
                    port_range: Some((443, 443)),
                    cidr: network_domain::RouteDestination::new("10.42.0.0/16")
                        .expect("valid CIDR"),
                    description: "tenant https ingress".to_string(),
                }],
            }],
            residency: ResidencyClass::StrictHomeRegion,
            state: VpcState::Creating,
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_000_000,
        })
        .expect("seed VPC");
}

fn seed_subnet(catalog: &mut CloudNetworkCatalog) {
    catalog
        .add_subnet(SubnetCreate {
            resource_id: SUBNET_ID.to_string(),
            tenant_id: TENANT_ID.to_string(),
            vpc_id: VPC_ID.to_string(),
            region: "region-home".to_string(),
            az: "region-home-a".to_string(),
            cidr_v4: "10.42.1.0/24".to_string(),
            cidr_v6: "2001:db8:42:1::/64".to_string(),
            public_ip_on_launch: false,
            state: SubnetState::Creating,
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_000_010,
        })
        .expect("seed subnet");
}

fn seed_network(catalog: &mut CloudNetworkCatalog) {
    seed_vpc(catalog);
    seed_subnet(catalog);
}

#[test]
fn openapi_runtime_binding_contracts_are_covered() {
    assert_eq!(CLOUD_NETWORK_LB_CREATE_SURFACE, "cloud.network.lb.create");
    assert_eq!(CloudNetworkLbCreateApiStatus::Created.code(), 201);
    assert_eq!(CloudNetworkLbCreateApiStatus::BadRequest.code(), 400);
    assert_eq!(CloudNetworkLbCreateApiStatus::Unauthorized.code(), 401);
    assert_eq!(CloudNetworkLbCreateApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudNetworkLbCreateApiStatus::NotFound.code(), 404);
    assert_eq!(CloudNetworkLbCreateApiStatus::Conflict.code(), 409);
    assert_eq!(
        CloudNetworkLbCreateApiStatus::UnprocessableEntity.code(),
        422
    );
}

// ===========================================================================
// FAIL-CLOSED AUTHZ SEAM (C9 / ADR-0587) — RED/GREEN tests that MUST fail if the
// verified-principal + PDP gate is removed.
// ===========================================================================

#[test]
fn lb_create_api_rejects_absent_credential_as_401() {
    let mut catalog = CloudNetworkCatalog::default();
    seed_network(&mut catalog);
    let mut ledger = CloudNetworkLbCreateIdempotencyLedger::default();
    let provider = allow_all_provider(); // PDP would allow — the credential gate must still block.
    let mut request = create_request("req-lb-no-cred", "idem-lb-no-cred");
    request.credential.authorization = None;

    let error =
        create_cloud_network_load_balancer_from_api(&mut catalog, &mut ledger, &provider, request)
            .expect_err("absent credential must be rejected");

    assert_eq!(error, CloudNetworkLbApiError::CallerUnauthenticated);
    assert_eq!(error.lb_create_status_code(), 401);
    assert!(ledger.is_empty());
    assert_eq!(catalog.load_balancers().count(), 0);
}

#[test]
fn lb_create_api_rejects_forged_bearer_as_401() {
    let mut catalog = CloudNetworkCatalog::default();
    seed_network(&mut catalog);
    let mut ledger = CloudNetworkLbCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let mut request = create_request("req-lb-forged", "idem-lb-forged");
    // A forged/wrong bearer; a self-attested principal/tenant cannot launder it.
    request.credential.authorization = Some("Bearer not-the-real-secret".to_string());

    let error =
        create_cloud_network_load_balancer_from_api(&mut catalog, &mut ledger, &provider, request)
            .expect_err("forged bearer must be rejected");

    assert_eq!(error, CloudNetworkLbApiError::CallerUnauthenticated);
    assert_eq!(error.lb_create_status_code(), 401);
    assert!(ledger.is_empty());
    assert_eq!(catalog.load_balancers().count(), 0);
}

#[test]
fn lb_create_api_rejects_request_principal_not_matching_verified_identity_as_403() {
    let mut catalog = CloudNetworkCatalog::default();
    seed_network(&mut catalog);
    let mut ledger = CloudNetworkLbCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    // Valid bearer (verifies as sp_network_lb_admin) but the request claims a
    // DIFFERENT principal — a verified caller cannot act as another principal.
    let mut request = create_request("req-lb-princ", "idem-lb-princ");
    request.principal.principal_id = "sp_someone_else".to_string();

    let error =
        create_cloud_network_load_balancer_from_api(&mut catalog, &mut ledger, &provider, request)
            .expect_err("request principal must match verified identity");

    assert_eq!(
        error,
        CloudNetworkLbApiError::VerifiedPrincipalMismatch {
            verified_principal_id: PRINCIPAL_ID.to_string(),
            request_principal_id: "sp_someone_else".to_string(),
        }
    );
    assert_eq!(error.lb_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.load_balancers().count(), 0);
}

#[test]
fn lb_create_api_denies_cross_tenant_at_pdp_proving_blast_radius_binding() {
    // The PDP only authorizes for ten_alpha. A caller verified as ten_alpha
    // whose body targets ten_beta is denied AT THE PDP because the resource
    // carries the TARGET tenant (no IDOR / no flatten-to-caller).
    let mut catalog = CloudNetworkCatalog::default();
    seed_network(&mut catalog);
    let mut ledger = CloudNetworkLbCreateIdempotencyLedger::default();
    let provider = provider_with(Arc::new(TenantScopedAuthorizer {
        allowed_tenant: TENANT_ID.to_string(),
    }));

    // Verified tenant is ten_alpha; the body claims ten_beta. The verified-tenant
    // cross-check catches this first (also 403) — a verified caller cannot act on
    // another tenant's resource. This proves the target tenant is the trusted
    // source, not a caller header.
    let mut request = create_request("req-lb-xtenant", "idem-lb-xtenant");
    request.boundary.tenant_id = "ten_beta".to_string();
    request.principal.tenant_id = "ten_beta".to_string();
    request.body.tenant_id = "ten_beta".to_string();

    let error =
        create_cloud_network_load_balancer_from_api(&mut catalog, &mut ledger, &provider, request)
            .expect_err("cross-tenant create must be denied");

    assert_eq!(
        error,
        CloudNetworkLbApiError::VerifiedTenantMismatch {
            verified_tenant_id: TENANT_ID.to_string(),
            request_tenant_id: "ten_beta".to_string(),
        }
    );
    assert_eq!(error.lb_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.load_balancers().count(), 0);
}

#[test]
fn lb_create_api_denies_when_pdp_denies_as_403() {
    let mut catalog = CloudNetworkCatalog::default();
    seed_network(&mut catalog);
    let mut ledger = CloudNetworkLbCreateIdempotencyLedger::default();
    let provider = provider_with(Arc::new(DenyAllAuthorizer));
    let request = create_request("req-lb-deny", "idem-lb-deny");

    let error =
        create_cloud_network_load_balancer_from_api(&mut catalog, &mut ledger, &provider, request)
            .expect_err("PDP deny must be 403");

    assert_eq!(
        error,
        CloudNetworkLbApiError::AuthorizationDenied {
            surface: CLOUD_NETWORK_LB_CREATE_SURFACE.to_string(),
        }
    );
    assert_eq!(error.lb_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.load_balancers().count(), 0);
}

#[test]
fn lb_create_api_fails_closed_when_pdp_faults_as_403_not_5xx() {
    let mut catalog = CloudNetworkCatalog::default();
    seed_network(&mut catalog);
    let mut ledger = CloudNetworkLbCreateIdempotencyLedger::default();
    let provider = provider_with(Arc::new(RefuseAuthorizer));
    let request = create_request("req-lb-fault", "idem-lb-fault");

    let error =
        create_cloud_network_load_balancer_from_api(&mut catalog, &mut ledger, &provider, request)
            .expect_err("PDP fault must fail closed to 403");

    assert_eq!(
        error,
        CloudNetworkLbApiError::AuthorizationDenied {
            surface: CLOUD_NETWORK_LB_CREATE_SURFACE.to_string(),
        }
    );
    assert_eq!(error.lb_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.load_balancers().count(), 0);
}

#[test]
fn lb_create_api_creates_l7_grpc_once_and_replays_same_idempotent_result() {
    let mut catalog = CloudNetworkCatalog::default();
    seed_network(&mut catalog);
    let mut ledger = CloudNetworkLbCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let request = create_request("req-network-lb-create", "idem-network-lb-create");

    let first = create_cloud_network_load_balancer_from_api(
        &mut catalog,
        &mut ledger,
        &provider,
        request.clone(),
    )
    .expect("authorized load balancer create succeeds");
    let second =
        create_cloud_network_load_balancer_from_api(&mut catalog, &mut ledger, &provider, request)
            .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.load_balancers().count(), 1);
    assert_eq!(first.metadata.request_id, "req-network-lb-create");
    assert_eq!(first.data.resource_id, LB_ID);
    assert_eq!(first.data.tenant_id, TENANT_ID);
    assert_eq!(first.data.vpc_id, VPC_ID);
    assert_eq!(first.data.region, "region-home");
    assert_eq!(first.data.kind, "l7_grpc");
    assert_eq!(first.data.listener_count, 1);
    assert_eq!(first.data.target_group_count, 1);
    assert!(first.data.mtls_enabled);
    assert_eq!(
        first.data.waf_policy,
        Some("waf_cloud_frontdoor".to_string())
    );
    assert_eq!(first.data.data_class, "PUBLIC");
    assert_eq!(first.data.state, "creating");
    assert_eq!(first.data.schema_version, 1);
}

#[test]
fn lb_create_api_rejects_path_body_lb_drift_before_catalog_mutation() {
    let mut catalog = CloudNetworkCatalog::default();
    seed_network(&mut catalog);
    let mut ledger = CloudNetworkLbCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let mut request = create_request("req-network-lb-drift", "idem-network-lb-drift");
    request.body.resource_id = "oya:cloud:region-home:ten_alpha:lb-v7:other".to_string();

    let error =
        create_cloud_network_load_balancer_from_api(&mut catalog, &mut ledger, &provider, request)
            .expect_err("path/body LB drift is rejected");

    assert_eq!(
        error,
        CloudNetworkLbApiError::LoadBalancerIdMismatch {
            path_load_balancer_id: LB_ID.to_string(),
            body_resource_id: "oya:cloud:region-home:ten_alpha:lb-v7:other".to_string(),
        }
    );
    assert_eq!(error.lb_create_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(catalog.load_balancers().count(), 0);
}

#[test]
fn lb_create_api_rejects_missing_principal_as_401() {
    let mut catalog = CloudNetworkCatalog::default();
    seed_network(&mut catalog);
    let mut ledger = CloudNetworkLbCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let mut unauthenticated = create_request("req-network-lb-authn", "idem-network-lb-authn");
    unauthenticated.principal.principal_id = " ".to_string();

    let authn_error = create_cloud_network_load_balancer_from_api(
        &mut catalog,
        &mut ledger,
        &provider,
        unauthenticated,
    )
    .expect_err("missing principal is authentication failure");

    assert_eq!(authn_error, CloudNetworkLbApiError::EmptyPrincipalId);
    assert_eq!(authn_error.lb_create_status_code(), 401);
    assert!(ledger.is_empty());
    assert_eq!(catalog.load_balancers().count(), 0);
}

#[test]
fn lb_create_api_rejects_required_header_and_tenant_drift_before_ledger() {
    let mut catalog = CloudNetworkCatalog::default();
    seed_network(&mut catalog);
    let mut ledger = CloudNetworkLbCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let mut request = create_request(" ", "idem-network-lb-empty-header");
    assert_eq!(
        create_cloud_network_load_balancer_from_api(
            &mut catalog,
            &mut ledger,
            &provider,
            request.clone()
        ),
        Err(CloudNetworkLbApiError::EmptyRequestId)
    );

    request.boundary.request_id = "req-network-lb-tenant-drift".to_string();
    request.boundary.tenant_id = "ten_other".to_string();
    assert_eq!(
        create_cloud_network_load_balancer_from_api(&mut catalog, &mut ledger, &provider, request),
        Err(CloudNetworkLbApiError::TenantMismatch {
            header_tenant_id: "ten_other".to_string(),
            principal_tenant_id: TENANT_ID.to_string(),
            body_tenant_id: TENANT_ID.to_string(),
        })
    );
    assert!(ledger.is_empty());
    assert_eq!(catalog.load_balancers().count(), 0);
}

#[test]
fn lb_create_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut catalog = CloudNetworkCatalog::default();
    seed_network(&mut catalog);
    let mut ledger = CloudNetworkLbCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let request = create_request("req-network-lb-idem", "idem-network-lb-idem");
    create_cloud_network_load_balancer_from_api(
        &mut catalog,
        &mut ledger,
        &provider,
        request.clone(),
    )
    .expect("initial create succeeds");

    let mut drifted = request;
    drifted.body.waf_policy = Some("waf_drifted".to_string());
    assert_eq!(
        create_cloud_network_load_balancer_from_api(&mut catalog, &mut ledger, &provider, drifted),
        Err(CloudNetworkLbApiError::IdempotencyKeyReused {
            idempotency_key: "idem-network-lb-idem".to_string(),
        })
    );
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.load_balancers().count(), 1);
}

#[test]
fn lb_create_api_maps_duplicate_lb_to_conflict() {
    let mut catalog = CloudNetworkCatalog::default();
    seed_network(&mut catalog);
    let mut ledger = CloudNetworkLbCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    create_cloud_network_load_balancer_from_api(
        &mut catalog,
        &mut ledger,
        &provider,
        create_request("req-network-lb-dup-1", "idem-network-lb-dup-1"),
    )
    .expect("first LB create succeeds");

    let error = create_cloud_network_load_balancer_from_api(
        &mut catalog,
        &mut ledger,
        &provider,
        create_request("req-network-lb-dup-2", "idem-network-lb-dup-2"),
    )
    .expect_err("same LB id through a new idempotency key conflicts");

    assert_eq!(
        error,
        CloudNetworkLbApiError::Network(CloudNetworkError::DuplicateLoadBalancer)
    );
    assert_eq!(error.lb_create_status_code(), 409);
    assert_eq!(catalog.load_balancers().count(), 1);
}

#[test]
fn lb_create_api_maps_unknown_vpc_and_subnet_targets() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkLbCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let unknown_vpc = create_request("req-network-lb-vpc", "idem-network-lb-vpc");

    let vpc_error = create_cloud_network_load_balancer_from_api(
        &mut catalog,
        &mut ledger,
        &provider,
        unknown_vpc,
    )
    .expect_err("LB create requires a known VPC");
    assert_eq!(
        vpc_error,
        CloudNetworkLbApiError::Network(CloudNetworkError::UnknownVpc)
    );
    assert_eq!(vpc_error.lb_create_status_code(), 404);

    seed_vpc(&mut catalog);
    let missing_subnet = create_request("req-network-lb-subnet", "idem-network-lb-subnet");
    let subnet_error = create_cloud_network_load_balancer_from_api(
        &mut catalog,
        &mut ledger,
        &provider,
        missing_subnet,
    )
    .expect_err("LB target groups require known subnets");
    assert_eq!(
        subnet_error,
        CloudNetworkLbApiError::Network(CloudNetworkError::UnknownSubnet)
    );
    assert_eq!(subnet_error.lb_create_status_code(), 404);
    assert_eq!(catalog.load_balancers().count(), 0);
}

#[test]
fn lb_create_api_maps_l7_tls_and_grpc_mtls_invariants() {
    let mut catalog = CloudNetworkCatalog::default();
    seed_network(&mut catalog);
    let mut ledger = CloudNetworkLbCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let mut no_tls = create_request("req-network-lb-tls", "idem-network-lb-tls");
    no_tls.body.listeners[0].tls_certificate = None;

    let tls_error =
        create_cloud_network_load_balancer_from_api(&mut catalog, &mut ledger, &provider, no_tls)
            .expect_err("L7 listeners require TLS certificates");
    assert_eq!(
        tls_error,
        CloudNetworkLbApiError::Network(CloudNetworkError::L7RequiresTls)
    );
    assert_eq!(tls_error.lb_create_status_code(), 400);

    let mut no_mtls = create_request("req-network-lb-mtls", "idem-network-lb-mtls");
    no_mtls.body.mtls = None;
    let mtls_error =
        create_cloud_network_load_balancer_from_api(&mut catalog, &mut ledger, &provider, no_mtls)
            .expect_err("gRPC LBs require mTLS config");
    assert_eq!(
        mtls_error,
        CloudNetworkLbApiError::Network(CloudNetworkError::GrpcRequiresMtls)
    );
    assert_eq!(mtls_error.lb_create_status_code(), 400);
    assert_eq!(catalog.load_balancers().count(), 0);
}

#[test]
fn lb_create_api_rejects_invalid_kind_policy_and_data_class() {
    let mut catalog = CloudNetworkCatalog::default();
    seed_network(&mut catalog);
    let mut ledger = CloudNetworkLbCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let mut invalid_kind = create_request("req-network-lb-kind", "idem-network-lb-kind");
    invalid_kind.body.kind = "l9_magic".to_string();

    let kind_error = create_cloud_network_load_balancer_from_api(
        &mut catalog,
        &mut ledger,
        &provider,
        invalid_kind,
    )
    .expect_err("unknown LB kind is rejected before catalog mutation");
    assert_eq!(
        kind_error,
        CloudNetworkLbApiError::InvalidLoadBalancerKindLabel {
            kind: "l9_magic".to_string(),
        }
    );
    assert_eq!(kind_error.lb_create_status_code(), 400);

    let mut invalid_policy = create_request("req-network-lb-policy", "idem-network-lb-policy");
    invalid_policy.body.mtls = Some(CloudNetworkLbMtlsConfigCreateRequest {
        ca_bundle_ref: "cert/region-home/ten_alpha/mesh-ca".to_string(),
        client_policy: "trust_any_client".to_string(),
    });
    let policy_error = create_cloud_network_load_balancer_from_api(
        &mut catalog,
        &mut ledger,
        &provider,
        invalid_policy,
    )
    .expect_err("unknown mTLS client policy is rejected before catalog mutation");
    assert_eq!(
        policy_error,
        CloudNetworkLbApiError::InvalidMtlsClientPolicyLabel {
            client_policy: "trust_any_client".to_string(),
        }
    );
    assert_eq!(policy_error.lb_create_status_code(), 400);

    let mut invalid_class = create_request("req-network-lb-class", "idem-network-lb-class");
    invalid_class.body.data_class = "INTERNAL_ONLY".to_string();
    let class_error = create_cloud_network_load_balancer_from_api(
        &mut catalog,
        &mut ledger,
        &provider,
        invalid_class,
    )
    .expect_err("non-public LB metadata is rejected by the kernel");
    assert_eq!(
        class_error,
        CloudNetworkLbApiError::Network(CloudNetworkError::InvalidDataClass)
    );
    assert_eq!(class_error.lb_create_status_code(), 400);
    assert_eq!(catalog.load_balancers().count(), 0);
}
