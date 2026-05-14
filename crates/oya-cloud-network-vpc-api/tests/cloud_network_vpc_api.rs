use oya_cloud_network_domain::{CloudNetworkCatalog, CloudNetworkError};
use oya_cloud_network_vpc_api::{
    create_cloud_network_vpc_from_api, CloudNetworkVpcApiAuthorization,
    CloudNetworkVpcApiBoundaryContext, CloudNetworkVpcApiError, CloudNetworkVpcApiPrincipal,
    CloudNetworkVpcCreateApiRequest, CloudNetworkVpcCreateApiStatus,
    CloudNetworkVpcCreateIdempotencyLedger, CloudNetworkVpcCreateRequest,
    CloudNetworkVpcRouteCreateRequest, CloudNetworkVpcRouteTableCreateRequest,
    CloudNetworkVpcSecurityGroupCreateRequest, CloudNetworkVpcSecurityRuleCreateRequest,
    CLOUD_NETWORK_VPC_CREATE_SURFACE,
};

const VPC_ID: &str = "oya:cloud:kr-seoul:ten_kr:vpc:prod";

fn boundary_for(request_id: &str, idempotency_key: &str) -> CloudNetworkVpcApiBoundaryContext {
    CloudNetworkVpcApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_kr".to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal_for(principal_id: &str) -> CloudNetworkVpcApiPrincipal {
    CloudNetworkVpcApiPrincipal {
        tenant_id: "ten_kr".to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization_for(principal_id: &str, surfaces: &[&str]) -> CloudNetworkVpcApiAuthorization {
    CloudNetworkVpcApiAuthorization {
        tenant_id: "ten_kr".to_string(),
        principal_id: principal_id.to_string(),
        decision_id: format!("authz_decision_{principal_id}"),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
    }
}

fn create_body(resource_id: &str) -> CloudNetworkVpcCreateRequest {
    CloudNetworkVpcCreateRequest {
        resource_id: resource_id.to_string(),
        tenant_id: "ten_kr".to_string(),
        region: "kr-seoul".to_string(),
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
        residency: "strict_kr".to_string(),
        data_class: "PUBLIC".to_string(),
        created_at_epoch_seconds: 1_700_000_000,
    }
}

fn create_request(request_id: &str, idempotency_key: &str) -> CloudNetworkVpcCreateApiRequest {
    CloudNetworkVpcCreateApiRequest {
        path_vpc_id: VPC_ID.to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_network_admin"),
        authorization: authorization_for("sp_network_admin", &[CLOUD_NETWORK_VPC_CREATE_SURFACE]),
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

#[test]
fn vpc_create_api_creates_vpc_once_and_replays_same_idempotent_result() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let request = create_request("req-network-vpc-create", "idem-network-vpc-create");

    let first = create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, request.clone())
        .expect("authorized VPC create succeeds");
    let second = create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, request)
        .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.vpcs().count(), 1);
    assert_eq!(first.metadata.request_id, "req-network-vpc-create");
    assert_eq!(first.data.resource_id, VPC_ID);
    assert_eq!(first.data.tenant_id, "ten_kr");
    assert_eq!(first.data.region, "kr-seoul");
    assert_eq!(first.data.cidr_v4, "10.42.0.0/16");
    assert_eq!(first.data.cidr_v6, "2001:db8:42::/56");
    assert!(first.data.flow_logs_enabled);
    assert_eq!(first.data.route_count, 2);
    assert_eq!(first.data.security_group_count, 1);
    assert_eq!(first.data.residency, "strict_kr");
    assert_eq!(first.data.data_class, "PUBLIC");
    assert_eq!(first.data.state, "creating");
    assert_eq!(first.data.schema_version, 1);
}

#[test]
fn vpc_create_api_rejects_path_body_vpc_drift_before_catalog_mutation() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let mut request = create_request("req-network-vpc-drift", "idem-network-vpc-drift");
    request.body.resource_id = "oya:cloud:kr-seoul:ten_kr:vpc:other".to_string();

    let error = create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, request)
        .expect_err("path/body VPC drift is rejected");

    assert_eq!(
        error,
        CloudNetworkVpcApiError::VpcIdMismatch {
            path_vpc_id: VPC_ID.to_string(),
            body_resource_id: "oya:cloud:kr-seoul:ten_kr:vpc:other".to_string(),
        }
    );
    assert_eq!(error.vpc_create_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(catalog.vpcs().count(), 0);
}

#[test]
fn vpc_create_api_separates_missing_authentication_from_denied_authorization() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let mut unauthenticated = create_request("req-network-vpc-authn", "idem-network-vpc-authn");
    unauthenticated.principal.principal_id = " ".to_string();

    let authn_error = create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, unauthenticated)
        .expect_err("missing principal is authentication failure");

    assert_eq!(authn_error, CloudNetworkVpcApiError::EmptyPrincipalId);
    assert_eq!(authn_error.vpc_create_status_code(), 401);

    let mut denied = create_request("req-network-vpc-authz", "idem-network-vpc-authz");
    denied.authorization.allowed_surfaces = vec!["cloud.network.lb.create".to_string()];
    let authz_error = create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, denied)
        .expect_err("authorization decision excludes VPC create");

    assert_eq!(
        authz_error,
        CloudNetworkVpcApiError::AuthorizationDenied {
            surface: CLOUD_NETWORK_VPC_CREATE_SURFACE.to_string(),
        }
    );
    assert_eq!(authz_error.vpc_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.vpcs().count(), 0);
}

#[test]
fn vpc_create_api_rejects_required_header_and_tenant_drift_before_ledger() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let mut request = create_request(" ", "idem-network-vpc-empty-header");
    assert_eq!(
        create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, request.clone()),
        Err(CloudNetworkVpcApiError::EmptyRequestId)
    );

    request.boundary.request_id = "req-network-vpc-tenant-drift".to_string();
    request.boundary.tenant_id = "ten_other".to_string();
    assert_eq!(
        create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, request),
        Err(CloudNetworkVpcApiError::TenantMismatch {
            header_tenant_id: "ten_other".to_string(),
            principal_tenant_id: "ten_kr".to_string(),
            body_tenant_id: "ten_kr".to_string(),
        })
    );
    assert!(ledger.is_empty());
    assert_eq!(catalog.vpcs().count(), 0);
}

#[test]
fn vpc_create_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkVpcCreateIdempotencyLedger::default();
    let request = create_request("req-network-vpc-idem", "idem-network-vpc-idem");
    create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, request.clone())
        .expect("initial create succeeds");

    let mut drifted = request;
    drifted.body.cidr_v4 = "10.43.0.0/16".to_string();
    assert_eq!(
        create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, drifted),
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
    create_cloud_network_vpc_from_api(
        &mut catalog,
        &mut ledger,
        create_request("req-network-vpc-dup-1", "idem-network-vpc-dup-1"),
    )
    .expect("first VPC create succeeds");

    let error = create_cloud_network_vpc_from_api(
        &mut catalog,
        &mut ledger,
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
    let mut no_flow_logs = create_request("req-network-vpc-flow", "idem-network-vpc-flow");
    no_flow_logs.body.flow_logs_enabled = false;

    let flow_error = create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, no_flow_logs)
        .expect_err("flow logs are mandatory for VPC creation");

    assert_eq!(
        flow_error,
        CloudNetworkVpcApiError::Network(CloudNetworkError::FlowLogsRequired)
    );
    assert_eq!(flow_error.vpc_create_status_code(), 400);
    assert_eq!(catalog.vpcs().count(), 0);

    let mut residency_drift = create_request("req-network-vpc-res", "idem-network-vpc-res");
    residency_drift.body.region = "us-virginia".to_string();
    let residency_error =
        create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, residency_drift)
            .expect_err("strict KR residency cannot create a US VPC");

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
    let mut invalid_route = create_request("req-network-vpc-route", "idem-network-vpc-route");
    invalid_route.body.route_table.routes[0].next_hop = "sidecar".to_string();

    let route_error = create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, invalid_route)
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
    let rule_error = create_cloud_network_vpc_from_api(&mut catalog, &mut ledger, invalid_rule)
        .expect_err("invalid port interval is rejected before catalog mutation");

    assert_eq!(rule_error, CloudNetworkVpcApiError::InvalidPortRange);
    assert_eq!(rule_error.vpc_create_status_code(), 400);
    assert_eq!(catalog.vpcs().count(), 0);
}
