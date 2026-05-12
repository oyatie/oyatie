use oya_cloud_network_dns_api::{
    create_cloud_network_dns_zone_from_api, CloudNetworkDnsApiAuthorization,
    CloudNetworkDnsApiBoundaryContext, CloudNetworkDnsApiError, CloudNetworkDnsApiPrincipal,
    CloudNetworkDnsZoneCreateApiRequest, CloudNetworkDnsZoneCreateApiStatus,
    CloudNetworkDnsZoneCreateIdempotencyLedger, CloudNetworkDnsZoneCreateRequest,
    CLOUD_NETWORK_DNS_ZONE_CREATE_SURFACE,
};
use oya_cloud_network_kernel::{
    CloudNetworkCatalog, CloudNetworkError, IpProtocol, NetworkRepo, RouteCreate, RouteNextHopKind,
    RouteTableCreate, RuleDirection, SecurityGroupCreate, SecurityRule, VpcCreate, VpcState,
};
use oya_platform_data_boundary_kernel::DataClass;
use oya_platform_residency_kernel::ResidencyClass;

const PUBLIC_ZONE_ID: &str = "oya:cloud:kr-seoul:ten_kr:dns-zone:example-com";
const PRIVATE_ZONE_ID: &str = "oya:cloud:kr-seoul:ten_kr:dns-zone:internal-example";
const VPC_ID: &str = "oya:cloud:kr-seoul:ten_kr:vpc:prod";

fn boundary_for(request_id: &str, idempotency_key: &str) -> CloudNetworkDnsApiBoundaryContext {
    CloudNetworkDnsApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_kr".to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal_for(principal_id: &str) -> CloudNetworkDnsApiPrincipal {
    CloudNetworkDnsApiPrincipal {
        tenant_id: "ten_kr".to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization_for(principal_id: &str, surfaces: &[&str]) -> CloudNetworkDnsApiAuthorization {
    CloudNetworkDnsApiAuthorization {
        tenant_id: "ten_kr".to_string(),
        principal_id: principal_id.to_string(),
        decision_id: format!("authz_decision_{principal_id}"),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
    }
}

fn public_zone_body(resource_id: &str) -> CloudNetworkDnsZoneCreateRequest {
    CloudNetworkDnsZoneCreateRequest {
        resource_id: resource_id.to_string(),
        tenant_id: "ten_kr".to_string(),
        region: "kr-seoul".to_string(),
        name: "example.com".to_string(),
        kind: "public".to_string(),
        vpc_id: None,
        dnssec_key_ref: Some("dnssec/kr-seoul/ten_kr/example-com".to_string()),
        data_class: "PUBLIC".to_string(),
        created_at_epoch_seconds: 1_700_000_030,
    }
}

fn private_zone_body(resource_id: &str) -> CloudNetworkDnsZoneCreateRequest {
    CloudNetworkDnsZoneCreateRequest {
        resource_id: resource_id.to_string(),
        tenant_id: "ten_kr".to_string(),
        region: "kr-seoul".to_string(),
        name: "internal.example".to_string(),
        kind: "private".to_string(),
        vpc_id: Some(VPC_ID.to_string()),
        dnssec_key_ref: None,
        data_class: "PUBLIC".to_string(),
        created_at_epoch_seconds: 1_700_000_031,
    }
}

fn create_public_request(
    request_id: &str,
    idempotency_key: &str,
) -> CloudNetworkDnsZoneCreateApiRequest {
    CloudNetworkDnsZoneCreateApiRequest {
        path_zone_id: PUBLIC_ZONE_ID.to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_network_dns_admin"),
        authorization: authorization_for(
            "sp_network_dns_admin",
            &[CLOUD_NETWORK_DNS_ZONE_CREATE_SURFACE],
        ),
        body: public_zone_body(PUBLIC_ZONE_ID),
    }
}

fn create_private_request(
    request_id: &str,
    idempotency_key: &str,
) -> CloudNetworkDnsZoneCreateApiRequest {
    CloudNetworkDnsZoneCreateApiRequest {
        path_zone_id: PRIVATE_ZONE_ID.to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_network_dns_admin"),
        authorization: authorization_for(
            "sp_network_dns_admin",
            &[CLOUD_NETWORK_DNS_ZONE_CREATE_SURFACE],
        ),
        body: private_zone_body(PRIVATE_ZONE_ID),
    }
}

fn seed_vpc(catalog: &mut CloudNetworkCatalog) {
    catalog
        .create_vpc(VpcCreate {
            resource_id: VPC_ID.to_string(),
            tenant_id: "ten_kr".to_string(),
            region: "kr-seoul".to_string(),
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
                    cidr: oya_cloud_network_kernel::RouteDestination::new("10.42.0.0/16")
                        .expect("valid CIDR"),
                    description: "tenant https ingress".to_string(),
                }],
            }],
            residency: ResidencyClass::StrictKr,
            state: VpcState::Creating,
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_000_000,
        })
        .expect("seed VPC");
}

#[test]
fn openapi_runtime_binding_contracts_are_covered() {
    assert_eq!(
        CLOUD_NETWORK_DNS_ZONE_CREATE_SURFACE,
        "cloud.network.dns.zone.create"
    );
    assert_eq!(CloudNetworkDnsZoneCreateApiStatus::Created.code(), 201);
    assert_eq!(CloudNetworkDnsZoneCreateApiStatus::BadRequest.code(), 400);
    assert_eq!(CloudNetworkDnsZoneCreateApiStatus::Unauthorized.code(), 401);
    assert_eq!(CloudNetworkDnsZoneCreateApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudNetworkDnsZoneCreateApiStatus::NotFound.code(), 404);
    assert_eq!(CloudNetworkDnsZoneCreateApiStatus::Conflict.code(), 409);
    assert_eq!(
        CloudNetworkDnsZoneCreateApiStatus::UnprocessableEntity.code(),
        422
    );
}

#[test]
fn dns_zone_create_api_creates_public_zone_once_and_replays_same_idempotent_result() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();
    let request = create_public_request("req-network-dns-create", "idem-network-dns-create");

    let first = create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, request.clone())
        .expect("authorized DNS zone create succeeds");
    let second = create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, request)
        .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.dns_zones().count(), 1);
    assert_eq!(first.metadata.request_id, "req-network-dns-create");
    assert_eq!(first.data.resource_id, PUBLIC_ZONE_ID);
    assert_eq!(first.data.tenant_id, "ten_kr");
    assert_eq!(first.data.region, "kr-seoul");
    assert_eq!(first.data.name, "example.com");
    assert_eq!(first.data.kind, "public");
    assert_eq!(first.data.vpc_id, None);
    assert_eq!(
        first.data.dnssec_key_ref,
        Some("dnssec/kr-seoul/ten_kr/example-com".to_string())
    );
    assert_eq!(first.data.data_class, "PUBLIC");
    assert_eq!(first.data.state, "creating");
    assert_eq!(first.data.schema_version, 1);
}

#[test]
fn dns_zone_create_api_creates_private_zone_bound_to_known_vpc() {
    let mut catalog = CloudNetworkCatalog::default();
    seed_vpc(&mut catalog);
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();

    let response = create_cloud_network_dns_zone_from_api(
        &mut catalog,
        &mut ledger,
        create_private_request("req-network-dns-private", "idem-network-dns-private"),
    )
    .expect("private zone binds a known same-tenant VPC");

    assert_eq!(response.data.kind, "private");
    assert_eq!(response.data.vpc_id, Some(VPC_ID.to_string()));
    assert_eq!(response.data.dnssec_key_ref, None);
    assert_eq!(catalog.dns_zones().count(), 1);
}

#[test]
fn dns_zone_create_api_rejects_path_body_zone_drift_before_catalog_mutation() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();
    let mut request = create_public_request("req-network-dns-drift", "idem-network-dns-drift");
    request.body.resource_id = "oya:cloud:kr-seoul:ten_kr:dns-zone:other".to_string();

    let error = create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, request)
        .expect_err("path/body DNS zone drift is rejected");

    assert_eq!(
        error,
        CloudNetworkDnsApiError::ZoneIdMismatch {
            path_zone_id: PUBLIC_ZONE_ID.to_string(),
            body_resource_id: "oya:cloud:kr-seoul:ten_kr:dns-zone:other".to_string(),
        }
    );
    assert_eq!(error.dns_zone_create_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(catalog.dns_zones().count(), 0);
}

#[test]
fn dns_zone_create_api_separates_missing_authentication_from_denied_authorization() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();
    let mut unauthenticated =
        create_public_request("req-network-dns-authn", "idem-network-dns-authn");
    unauthenticated.principal.principal_id = " ".to_string();

    let authn_error =
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, unauthenticated)
            .expect_err("missing principal is authentication failure");

    assert_eq!(authn_error, CloudNetworkDnsApiError::EmptyPrincipalId);
    assert_eq!(authn_error.dns_zone_create_status_code(), 401);

    let mut denied = create_public_request("req-network-dns-authz", "idem-network-dns-authz");
    denied.authorization.allowed_surfaces = vec!["cloud.network.vpc.create".to_string()];
    let authz_error = create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, denied)
        .expect_err("authorization decision excludes DNS zone create");

    assert_eq!(
        authz_error,
        CloudNetworkDnsApiError::AuthorizationDenied {
            surface: CLOUD_NETWORK_DNS_ZONE_CREATE_SURFACE.to_string(),
        }
    );
    assert_eq!(authz_error.dns_zone_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.dns_zones().count(), 0);
}

#[test]
fn dns_zone_create_api_rejects_required_header_and_tenant_drift_before_ledger() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();
    let mut request = create_public_request(" ", "idem-network-dns-empty-header");
    assert_eq!(
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, request.clone()),
        Err(CloudNetworkDnsApiError::EmptyRequestId)
    );

    request.boundary.request_id = "req-network-dns-tenant-drift".to_string();
    request.boundary.tenant_id = "ten_other".to_string();
    assert_eq!(
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, request),
        Err(CloudNetworkDnsApiError::TenantMismatch {
            header_tenant_id: "ten_other".to_string(),
            principal_tenant_id: "ten_kr".to_string(),
            body_tenant_id: "ten_kr".to_string(),
        })
    );
    assert!(ledger.is_empty());
    assert_eq!(catalog.dns_zones().count(), 0);
}

#[test]
fn dns_zone_create_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();
    let request = create_public_request("req-network-dns-idem", "idem-network-dns-idem");
    create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, request.clone())
        .expect("initial create succeeds");

    let mut drifted = request;
    drifted.body.name = "drift.example".to_string();
    assert_eq!(
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, drifted),
        Err(CloudNetworkDnsApiError::IdempotencyKeyReused {
            idempotency_key: "idem-network-dns-idem".to_string(),
        })
    );
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.dns_zones().count(), 1);
}

#[test]
fn dns_zone_create_api_maps_duplicate_zone_to_conflict() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();
    create_cloud_network_dns_zone_from_api(
        &mut catalog,
        &mut ledger,
        create_public_request("req-network-dns-dup-1", "idem-network-dns-dup-1"),
    )
    .expect("first DNS zone create succeeds");

    let error = create_cloud_network_dns_zone_from_api(
        &mut catalog,
        &mut ledger,
        create_public_request("req-network-dns-dup-2", "idem-network-dns-dup-2"),
    )
    .expect_err("same zone id through a new idempotency key conflicts");

    assert_eq!(
        error,
        CloudNetworkDnsApiError::Network(CloudNetworkError::DuplicateDnsZone)
    );
    assert_eq!(error.dns_zone_create_status_code(), 409);
    assert_eq!(catalog.dns_zones().count(), 1);
}

#[test]
fn dns_zone_create_api_maps_dnssec_private_vpc_and_unknown_vpc_invariants() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();
    let mut no_dnssec = create_public_request("req-network-dns-dnssec", "idem-network-dns-dnssec");
    no_dnssec.body.dnssec_key_ref = None;

    let dnssec_error = create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, no_dnssec)
        .expect_err("public DNS zones require DNSSEC");

    assert_eq!(
        dnssec_error,
        CloudNetworkDnsApiError::Network(CloudNetworkError::DnssecRequired)
    );
    assert_eq!(dnssec_error.dns_zone_create_status_code(), 400);
    assert_eq!(catalog.dns_zones().count(), 0);

    let mut missing_vpc = create_private_request("req-network-dns-vpc", "idem-network-dns-vpc");
    missing_vpc.body.vpc_id = None;
    let private_error =
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, missing_vpc)
            .expect_err("private DNS zones require a VPC binding");

    assert_eq!(
        private_error,
        CloudNetworkDnsApiError::Network(CloudNetworkError::PrivateZoneRequiresVpc)
    );
    assert_eq!(private_error.dns_zone_create_status_code(), 403);

    let unknown_vpc = create_private_request("req-network-dns-unknown", "idem-network-dns-unknown");
    let unknown_error =
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, unknown_vpc)
            .expect_err("private DNS zones require a known VPC binding");

    assert_eq!(
        unknown_error,
        CloudNetworkDnsApiError::Network(CloudNetworkError::UnknownVpc)
    );
    assert_eq!(unknown_error.dns_zone_create_status_code(), 404);
    assert_eq!(catalog.dns_zones().count(), 0);
}

#[test]
fn dns_zone_create_api_rejects_invalid_kind_and_data_class_without_catalog_mutation() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();
    let mut invalid_kind = create_public_request("req-network-dns-kind", "idem-network-dns-kind");
    invalid_kind.body.kind = "split_horizon".to_string();

    let kind_error =
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, invalid_kind)
            .expect_err("unknown DNS zone kind is rejected before catalog mutation");

    assert_eq!(
        kind_error,
        CloudNetworkDnsApiError::InvalidZoneKindLabel {
            kind: "split_horizon".to_string(),
        }
    );
    assert_eq!(kind_error.dns_zone_create_status_code(), 400);

    let mut invalid_class =
        create_public_request("req-network-dns-class", "idem-network-dns-class");
    invalid_class.body.data_class = "INTERNAL_ONLY".to_string();
    let class_error =
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, invalid_class)
            .expect_err("non-public DNS zone metadata is rejected by the kernel");

    assert_eq!(
        class_error,
        CloudNetworkDnsApiError::Network(CloudNetworkError::InvalidDataClass)
    );
    assert_eq!(class_error.dns_zone_create_status_code(), 400);
    assert_eq!(catalog.dns_zones().count(), 0);
}
