// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use network_dns::{
    CLOUD_NETWORK_DNS_ZONE_CREATE_SURFACE, CallerCredential, CloudNetworkDnsApiBoundaryContext,
    CloudNetworkDnsApiError, CloudNetworkDnsApiPrincipal, CloudNetworkDnsAuthzProvider,
    CloudNetworkDnsZoneCreateApiRequest, CloudNetworkDnsZoneCreateApiStatus,
    CloudNetworkDnsZoneCreateIdempotencyLedger, CloudNetworkDnsZoneCreateRequest,
    ConfiguredBearerPrincipalVerifier, DnsZoneCreateAuthorizationError, DnsZoneCreateAuthorizer,
    DnsZoneCreateResource, VerifiedPrincipal, create_cloud_network_dns_zone_from_api,
};
use network_domain::{
    CloudNetworkCatalog, CloudNetworkError, IpProtocol, NetworkRepo, RouteCreate, RouteNextHopKind,
    RouteTableCreate, RuleDirection, SecurityGroupCreate, SecurityRule, VpcCreate, VpcState,
};
use network_residency::ResidencyClass;
use oya_data_boundary_kernel::DataClass;

const PUBLIC_ZONE_ID: &str = "oya:cloud:region-home:ten_alpha:dns-zone:example-com";
const PRIVATE_ZONE_ID: &str = "oya:cloud:region-home:ten_alpha:dns-zone:internal-example";
const VPC_ID: &str = "oya:cloud:region-home:ten_alpha:vpc:prod";
const PRINCIPAL_ID: &str = "sp_network_dns_admin";
const TENANT_ID: &str = "ten_alpha";
const BEARER_SECRET: &str = "break-glass-network-dns-secret";

/// A test PDP that allows everything — used to PROVE the verified-principal /
/// tenant cross-check and credential gate fail-close even when the PDP would
/// otherwise allow (blast-radius binding is independent of the PDP verdict).
struct AllowAllAuthorizer;
impl DnsZoneCreateAuthorizer for AllowAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &DnsZoneCreateResource,
    ) -> Result<(), DnsZoneCreateAuthorizationError> {
        Ok(())
    }
}

/// A test PDP that denies everything (proves PDP-deny → 403).
struct DenyAllAuthorizer;
impl DnsZoneCreateAuthorizer for DenyAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &DnsZoneCreateResource,
    ) -> Result<(), DnsZoneCreateAuthorizationError> {
        Err(DnsZoneCreateAuthorizationError::Denied)
    }
}

/// A test PDP that refuses (fault) — proves a PDP fault is fail-closed → 403.
struct RefuseAuthorizer;
impl DnsZoneCreateAuthorizer for RefuseAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &DnsZoneCreateResource,
    ) -> Result<(), DnsZoneCreateAuthorizationError> {
        Err(DnsZoneCreateAuthorizationError::Refused)
    }
}

/// A PDP that authorizes for a SPECIFIC target tenant only — proves the PDP
/// receives the TARGET tenant (no IDOR / blast-radius binding).
struct TenantScopedAuthorizer {
    allowed_tenant: String,
}
impl DnsZoneCreateAuthorizer for TenantScopedAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        resource: &DnsZoneCreateResource,
    ) -> Result<(), DnsZoneCreateAuthorizationError> {
        if resource.tenant_id == self.allowed_tenant {
            Ok(())
        } else {
            Err(DnsZoneCreateAuthorizationError::Denied)
        }
    }
}

fn provider_with(authorizer: Arc<dyn DnsZoneCreateAuthorizer>) -> CloudNetworkDnsAuthzProvider {
    let verifier = Arc::new(
        ConfiguredBearerPrincipalVerifier::new(BEARER_SECRET, PRINCIPAL_ID, TENANT_ID)
            .expect("valid break-glass verifier config"),
    );
    CloudNetworkDnsAuthzProvider::new(verifier, authorizer)
}

fn allow_all_provider() -> CloudNetworkDnsAuthzProvider {
    provider_with(Arc::new(AllowAllAuthorizer))
}

fn valid_credential() -> CallerCredential {
    CallerCredential {
        authorization: Some(format!("Bearer {BEARER_SECRET}")),
        claimed_principal_id: PRINCIPAL_ID.to_string(),
        claimed_tenant_id: TENANT_ID.to_string(),
    }
}

fn boundary_for(request_id: &str, idempotency_key: &str) -> CloudNetworkDnsApiBoundaryContext {
    CloudNetworkDnsApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: TENANT_ID.to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal_for(principal_id: &str) -> CloudNetworkDnsApiPrincipal {
    CloudNetworkDnsApiPrincipal {
        tenant_id: TENANT_ID.to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn public_zone_body(resource_id: &str) -> CloudNetworkDnsZoneCreateRequest {
    CloudNetworkDnsZoneCreateRequest {
        resource_id: resource_id.to_string(),
        tenant_id: TENANT_ID.to_string(),
        region: "region-home".to_string(),
        name: "example.com".to_string(),
        kind: "public".to_string(),
        vpc_id: None,
        dnssec_key_ref: Some("dnssec/region-home/ten_alpha/example-com".to_string()),
        data_class: "PUBLIC".to_string(),
        created_at_epoch_seconds: 1_700_000_030,
    }
}

fn private_zone_body(resource_id: &str) -> CloudNetworkDnsZoneCreateRequest {
    CloudNetworkDnsZoneCreateRequest {
        resource_id: resource_id.to_string(),
        tenant_id: TENANT_ID.to_string(),
        region: "region-home".to_string(),
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
        principal: principal_for(PRINCIPAL_ID),
        credential: valid_credential(),
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
        principal: principal_for(PRINCIPAL_ID),
        credential: valid_credential(),
        body: private_zone_body(PRIVATE_ZONE_ID),
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

// ===========================================================================
// FAIL-CLOSED AUTHZ SEAM (C11 / ADR-0587) — RED/GREEN tests that MUST fail if
// the verified-principal + PDP gate is removed.
// ===========================================================================

#[test]
fn dns_zone_create_api_rejects_absent_credential_as_401() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();
    let provider = allow_all_provider(); // PDP would allow — the credential gate must still block.
    let mut request = create_public_request("req-dns-no-cred", "idem-dns-no-cred");
    request.credential.authorization = None;

    let error =
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, &provider, request)
            .expect_err("absent credential must be rejected");

    assert_eq!(error, CloudNetworkDnsApiError::CallerUnauthenticated);
    assert_eq!(error.dns_zone_create_status_code(), 401);
    assert!(ledger.is_empty());
    assert_eq!(catalog.dns_zones().count(), 0);
}

#[test]
fn dns_zone_create_api_rejects_forged_bearer_as_401() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let mut request = create_public_request("req-dns-forged", "idem-dns-forged");
    request.credential.authorization = Some("Bearer not-the-real-secret".to_string());

    let error =
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, &provider, request)
            .expect_err("forged bearer must be rejected");

    assert_eq!(error, CloudNetworkDnsApiError::CallerUnauthenticated);
    assert_eq!(error.dns_zone_create_status_code(), 401);
    assert!(ledger.is_empty());
    assert_eq!(catalog.dns_zones().count(), 0);
}

#[test]
fn dns_zone_create_api_rejects_request_principal_not_matching_verified_identity_as_403() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let mut request = create_public_request("req-dns-princ", "idem-dns-princ");
    request.principal.principal_id = "sp_someone_else".to_string();

    let error =
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, &provider, request)
            .expect_err("request principal must match verified identity");

    assert_eq!(
        error,
        CloudNetworkDnsApiError::VerifiedPrincipalMismatch {
            verified_principal_id: PRINCIPAL_ID.to_string(),
            request_principal_id: "sp_someone_else".to_string(),
        }
    );
    assert_eq!(error.dns_zone_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.dns_zones().count(), 0);
}

#[test]
fn dns_zone_create_api_denies_cross_tenant_proving_blast_radius_binding() {
    // The PDP only authorizes for ten_alpha. A caller verified as ten_alpha
    // whose body targets ten_beta is denied because the resource carries the
    // TARGET tenant (no IDOR / no flatten-to-caller).
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();
    let provider = provider_with(Arc::new(TenantScopedAuthorizer {
        allowed_tenant: TENANT_ID.to_string(),
    }));
    let mut request = create_public_request("req-dns-xtenant", "idem-dns-xtenant");
    request.boundary.tenant_id = "ten_beta".to_string();
    request.principal.tenant_id = "ten_beta".to_string();
    request.body.tenant_id = "ten_beta".to_string();

    let error =
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, &provider, request)
            .expect_err("cross-tenant create must be denied");

    assert_eq!(
        error,
        CloudNetworkDnsApiError::VerifiedTenantMismatch {
            verified_tenant_id: TENANT_ID.to_string(),
            request_tenant_id: "ten_beta".to_string(),
        }
    );
    assert_eq!(error.dns_zone_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.dns_zones().count(), 0);
}

#[test]
fn dns_zone_create_api_denies_when_pdp_denies_as_403() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();
    let provider = provider_with(Arc::new(DenyAllAuthorizer));
    let request = create_public_request("req-dns-deny", "idem-dns-deny");

    let error =
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, &provider, request)
            .expect_err("PDP deny must be 403");

    assert_eq!(
        error,
        CloudNetworkDnsApiError::AuthorizationDenied {
            surface: CLOUD_NETWORK_DNS_ZONE_CREATE_SURFACE.to_string(),
        }
    );
    assert_eq!(error.dns_zone_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.dns_zones().count(), 0);
}

#[test]
fn dns_zone_create_api_fails_closed_when_pdp_faults_as_403_not_5xx() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();
    let provider = provider_with(Arc::new(RefuseAuthorizer));
    let request = create_public_request("req-dns-fault", "idem-dns-fault");

    let error =
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, &provider, request)
            .expect_err("PDP fault must fail closed to 403");

    assert_eq!(
        error,
        CloudNetworkDnsApiError::AuthorizationDenied {
            surface: CLOUD_NETWORK_DNS_ZONE_CREATE_SURFACE.to_string(),
        }
    );
    assert_eq!(error.dns_zone_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.dns_zones().count(), 0);
}

#[test]
fn dns_zone_create_api_creates_public_zone_once_and_replays_same_idempotent_result() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let request = create_public_request("req-network-dns-create", "idem-network-dns-create");

    let first = create_cloud_network_dns_zone_from_api(
        &mut catalog,
        &mut ledger,
        &provider,
        request.clone(),
    )
    .expect("authorized DNS zone create succeeds");
    let second =
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, &provider, request)
            .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.dns_zones().count(), 1);
    assert_eq!(first.metadata.request_id, "req-network-dns-create");
    assert_eq!(first.data.resource_id, PUBLIC_ZONE_ID);
    assert_eq!(first.data.tenant_id, TENANT_ID);
    assert_eq!(first.data.region, "region-home");
    assert_eq!(first.data.name, "example.com");
    assert_eq!(first.data.kind, "public");
    assert_eq!(first.data.vpc_id, None);
    assert_eq!(
        first.data.dnssec_key_ref,
        Some("dnssec/region-home/ten_alpha/example-com".to_string())
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
    let provider = allow_all_provider();

    let response = create_cloud_network_dns_zone_from_api(
        &mut catalog,
        &mut ledger,
        &provider,
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
    let provider = allow_all_provider();
    let mut request = create_public_request("req-network-dns-drift", "idem-network-dns-drift");
    request.body.resource_id = "oya:cloud:region-home:ten_alpha:dns-zone:other".to_string();

    let error =
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, &provider, request)
            .expect_err("path/body DNS zone drift is rejected");

    assert_eq!(
        error,
        CloudNetworkDnsApiError::ZoneIdMismatch {
            path_zone_id: PUBLIC_ZONE_ID.to_string(),
            body_resource_id: "oya:cloud:region-home:ten_alpha:dns-zone:other".to_string(),
        }
    );
    assert_eq!(error.dns_zone_create_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(catalog.dns_zones().count(), 0);
}

#[test]
fn dns_zone_create_api_rejects_missing_principal_as_401() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let mut unauthenticated =
        create_public_request("req-network-dns-authn", "idem-network-dns-authn");
    unauthenticated.principal.principal_id = " ".to_string();

    let authn_error = create_cloud_network_dns_zone_from_api(
        &mut catalog,
        &mut ledger,
        &provider,
        unauthenticated,
    )
    .expect_err("missing principal is authentication failure");

    assert_eq!(authn_error, CloudNetworkDnsApiError::EmptyPrincipalId);
    assert_eq!(authn_error.dns_zone_create_status_code(), 401);
    assert!(ledger.is_empty());
    assert_eq!(catalog.dns_zones().count(), 0);
}

#[test]
fn dns_zone_create_api_rejects_required_header_and_tenant_drift_before_ledger() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let mut request = create_public_request(" ", "idem-network-dns-empty-header");
    assert_eq!(
        create_cloud_network_dns_zone_from_api(
            &mut catalog,
            &mut ledger,
            &provider,
            request.clone()
        ),
        Err(CloudNetworkDnsApiError::EmptyRequestId)
    );

    request.boundary.request_id = "req-network-dns-tenant-drift".to_string();
    request.boundary.tenant_id = "ten_other".to_string();
    assert_eq!(
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, &provider, request),
        Err(CloudNetworkDnsApiError::TenantMismatch {
            header_tenant_id: "ten_other".to_string(),
            principal_tenant_id: TENANT_ID.to_string(),
            body_tenant_id: TENANT_ID.to_string(),
        })
    );
    assert!(ledger.is_empty());
    assert_eq!(catalog.dns_zones().count(), 0);
}

#[test]
fn dns_zone_create_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut catalog = CloudNetworkCatalog::default();
    let mut ledger = CloudNetworkDnsZoneCreateIdempotencyLedger::default();
    let provider = allow_all_provider();
    let request = create_public_request("req-network-dns-idem", "idem-network-dns-idem");
    create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, &provider, request.clone())
        .expect("initial create succeeds");

    let mut drifted = request;
    drifted.body.name = "drift.example".to_string();
    assert_eq!(
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, &provider, drifted),
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
    let provider = allow_all_provider();
    create_cloud_network_dns_zone_from_api(
        &mut catalog,
        &mut ledger,
        &provider,
        create_public_request("req-network-dns-dup-1", "idem-network-dns-dup-1"),
    )
    .expect("first DNS zone create succeeds");

    let error = create_cloud_network_dns_zone_from_api(
        &mut catalog,
        &mut ledger,
        &provider,
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
    let provider = allow_all_provider();
    let mut no_dnssec = create_public_request("req-network-dns-dnssec", "idem-network-dns-dnssec");
    no_dnssec.body.dnssec_key_ref = None;

    let dnssec_error =
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, &provider, no_dnssec)
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
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, &provider, missing_vpc)
            .expect_err("private DNS zones require a VPC binding");

    assert_eq!(
        private_error,
        CloudNetworkDnsApiError::Network(CloudNetworkError::PrivateZoneRequiresVpc)
    );
    assert_eq!(private_error.dns_zone_create_status_code(), 403);

    let unknown_vpc = create_private_request("req-network-dns-unknown", "idem-network-dns-unknown");
    let unknown_error =
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, &provider, unknown_vpc)
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
    let provider = allow_all_provider();
    let mut invalid_kind = create_public_request("req-network-dns-kind", "idem-network-dns-kind");
    invalid_kind.body.kind = "split_horizon".to_string();

    let kind_error =
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, &provider, invalid_kind)
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
        create_cloud_network_dns_zone_from_api(&mut catalog, &mut ledger, &provider, invalid_class)
            .expect_err("non-public DNS zone metadata is rejected by the kernel");

    assert_eq!(
        class_error,
        CloudNetworkDnsApiError::Network(CloudNetworkError::InvalidDataClass)
    );
    assert_eq!(class_error.dns_zone_create_status_code(), 400);
    assert_eq!(catalog.dns_zones().count(), 0);
}
