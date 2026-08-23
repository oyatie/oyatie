//! Self-hosted / colo VPC adapter boundary for Cloud Network.
//!
//! This crate keeps on-prem and colo control-plane endpoint, site, cell, and
//! fabric references outside the provider-neutral Cloud Network domain/API
//! crates while implementing the shared VPC provider port contract. It builds
//! deterministic request shapes only; credentialed live smoke remains a
//! separate promotion gate.
//! ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
//! `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use network_domain::{
    NetworkProviderDnsZoneCreateRequest, NetworkProviderDnsZoneError, NetworkProviderDnsZonePort,
    NetworkProviderDnsZoneReceipt, NetworkProviderKind, NetworkProviderVpcCreateRequest,
    NetworkProviderVpcError, NetworkProviderVpcPort, NetworkProviderVpcReceipt,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelfHostedColoVpcAdapterConfigError {
    InvalidEndpoint,
    InvalidSiteRef,
    InvalidCellRef,
    InvalidFabricRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelfHostedColoDnsZoneAdapterConfigError {
    InvalidEndpoint,
    InvalidSiteRef,
    InvalidCellRef,
    InvalidFabricRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelfHostedColoVpcAdapter {
    endpoint_origin: String,  // data_class: INTERNAL_ONLY
    site_ref: String,         // data_class: PUBLIC
    cell_ref: String,         // data_class: PUBLIC
    fabric_ref: String,       // data_class: INTERNAL_ONLY
    clock_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelfHostedColoVpcCommand {
    pub operation: &'static str,       // data_class: PUBLIC
    pub method: &'static str,          // data_class: PUBLIC
    pub endpoint_origin: String,       // data_class: INTERNAL_ONLY
    pub path: String,                  // data_class: INTERNAL_ONLY
    pub body_canonical: String,        // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelfHostedColoDnsZoneAdapter {
    endpoint_origin: String,  // data_class: INTERNAL_ONLY
    site_ref: String,         // data_class: PUBLIC
    cell_ref: String,         // data_class: PUBLIC
    fabric_ref: String,       // data_class: INTERNAL_ONLY
    clock_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelfHostedColoDnsZoneCommand {
    pub operation: &'static str,       // data_class: PUBLIC
    pub method: &'static str,          // data_class: PUBLIC
    pub endpoint_origin: String,       // data_class: INTERNAL_ONLY
    pub path: String,                  // data_class: INTERNAL_ONLY
    pub body_canonical: String,        // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String, // data_class: INTERNAL_ONLY
}

impl SelfHostedColoVpcAdapter {
    pub fn new(
        endpoint_origin: impl Into<String>,
        site_ref: impl Into<String>,
        cell_ref: impl Into<String>,
        fabric_ref: impl Into<String>,
    ) -> Result<Self, SelfHostedColoVpcAdapterConfigError> {
        let endpoint_origin = endpoint_origin.into();
        let site_ref = site_ref.into();
        let cell_ref = cell_ref.into();
        let fabric_ref = fabric_ref.into();
        validate_endpoint(&endpoint_origin)?;
        validate_segment(
            &site_ref,
            SelfHostedColoVpcAdapterConfigError::InvalidSiteRef,
        )?;
        validate_segment(
            &cell_ref,
            SelfHostedColoVpcAdapterConfigError::InvalidCellRef,
        )?;
        validate_segment(
            &fabric_ref,
            SelfHostedColoVpcAdapterConfigError::InvalidFabricRef,
        )?;
        Ok(Self {
            endpoint_origin,
            site_ref,
            cell_ref,
            fabric_ref,
            clock_epoch_seconds: 0,
        })
    }

    pub fn with_clock(mut self, clock_epoch_seconds: u64) -> Self {
        self.clock_epoch_seconds = clock_epoch_seconds;
        self
    }

    pub fn provider_vcn_ref(&self, vpc_resource_id: &str) -> String {
        format!(
            "selfhosted-vpc://{}/{}/{}/{}",
            self.site_ref, self.cell_ref, self.fabric_ref, vpc_resource_id
        )
    }

    pub fn create_network_segment_command(
        &self,
        request: &NetworkProviderVpcCreateRequest,
    ) -> Result<SelfHostedColoVpcCommand, NetworkProviderVpcError> {
        request.validate()?;
        self.ensure_provider_vcn(&request.provider_vcn_ref, &request.vpc.resource_id)?;
        let flow_logs_enabled = request.vpc.flow_logs_enabled.to_string();
        let route_count = request.vpc.route_table.routes.len().to_string();
        let security_group_count = request.vpc.security_groups.len().to_string();
        Ok(SelfHostedColoVpcCommand {
            operation: "CreateTenantNetworkSegment",
            method: "POST",
            endpoint_origin: self.endpoint_origin.clone(),
            path: format!(
                "/v1/sites/{}/cells/{}/network-segments",
                self.site_ref, self.cell_ref
            ),
            body_canonical: canonical_body(&[
                ("site_ref", self.site_ref.as_str()),
                ("cell_ref", self.cell_ref.as_str()),
                ("fabric_ref", self.fabric_ref.as_str()),
                ("resource_id", request.vpc.resource_id.as_str()),
                ("tenant_id", request.vpc.tenant_id.as_str()),
                ("region", request.vpc.region.as_str()),
                ("cidr_v4", request.vpc.cidr_v4.as_str()),
                ("cidr_v6", request.vpc.cidr_v6.as_str()),
                ("flow_logs_enabled", flow_logs_enabled.as_str()),
                ("route_table_id", request.vpc.route_table.id.as_str()),
                ("route_count", route_count.as_str()),
                ("security_group_count", security_group_count.as_str()),
                (
                    "residency",
                    request.vpc.residency.label().unwrap_or("per_pack"),
                ),
                ("data_class", request.vpc.data_class.label()),
                ("actor", request.actor.as_str()),
                ("idempotency_key", request.idempotency_key.as_str()),
            ]),
            provider_evidence_ref: format!(
                "selfhosted-vpc://{}/{}/{}/{}/{}",
                self.site_ref,
                self.cell_ref,
                self.fabric_ref,
                request.vpc.resource_id,
                request.request_id
            ),
        })
    }

    fn ensure_provider_vcn(
        &self,
        provider_vcn_ref: &str,
        vpc_resource_id: &str,
    ) -> Result<(), NetworkProviderVpcError> {
        let expected = self.provider_vcn_ref(vpc_resource_id);
        if provider_vcn_ref == expected {
            Ok(())
        } else {
            Err(NetworkProviderVpcError::ProviderRejected {
                provider: NetworkProviderKind::SelfHostedColoVpc,
                reason: "provider_vcn_ref does not match configured self-hosted colo VPC target"
                    .to_string(),
            })
        }
    }

    fn provider_request_id(&self, request_id: &str) -> String {
        format!("selfhosted-vpc-{}-{request_id}", self.clock_epoch_seconds)
    }
}

impl SelfHostedColoDnsZoneAdapter {
    pub fn new(
        endpoint_origin: impl Into<String>,
        site_ref: impl Into<String>,
        cell_ref: impl Into<String>,
        fabric_ref: impl Into<String>,
    ) -> Result<Self, SelfHostedColoDnsZoneAdapterConfigError> {
        let endpoint_origin = endpoint_origin.into();
        let site_ref = site_ref.into();
        let cell_ref = cell_ref.into();
        let fabric_ref = fabric_ref.into();
        validate_dns_endpoint(&endpoint_origin)?;
        validate_dns_segment(
            &site_ref,
            SelfHostedColoDnsZoneAdapterConfigError::InvalidSiteRef,
        )?;
        validate_dns_segment(
            &cell_ref,
            SelfHostedColoDnsZoneAdapterConfigError::InvalidCellRef,
        )?;
        validate_dns_segment(
            &fabric_ref,
            SelfHostedColoDnsZoneAdapterConfigError::InvalidFabricRef,
        )?;
        Ok(Self {
            endpoint_origin,
            site_ref,
            cell_ref,
            fabric_ref,
            clock_epoch_seconds: 0,
        })
    }

    pub fn with_clock(mut self, clock_epoch_seconds: u64) -> Self {
        self.clock_epoch_seconds = clock_epoch_seconds;
        self
    }

    pub fn provider_dns_zone_ref(&self, dns_zone_resource_id: &str) -> String {
        format!(
            "selfhosted-dns-zone://{}/{}/{}/{}",
            self.site_ref, self.cell_ref, self.fabric_ref, dns_zone_resource_id
        )
    }

    pub fn create_authoritative_zone_command(
        &self,
        request: &NetworkProviderDnsZoneCreateRequest,
    ) -> Result<SelfHostedColoDnsZoneCommand, NetworkProviderDnsZoneError> {
        request.validate()?;
        self.ensure_provider_dns_zone(
            &request.provider_dns_zone_ref,
            &request.dns_zone.resource_id,
        )?;
        let vpc_id = request.dns_zone.vpc_id.as_deref().unwrap_or("none");
        let dnssec_enabled = request.dns_zone.dnssec_key_ref.is_some().to_string();
        let dnssec_key_ref = request.dns_zone.dnssec_key_ref.as_deref().unwrap_or("none");
        Ok(SelfHostedColoDnsZoneCommand {
            operation: "CreateAuthoritativeDnsZone",
            method: "POST",
            endpoint_origin: self.endpoint_origin.clone(),
            path: format!(
                "/v1/sites/{}/cells/{}/dns/zones",
                self.site_ref, self.cell_ref
            ),
            body_canonical: canonical_body(&[
                ("site_ref", self.site_ref.as_str()),
                ("cell_ref", self.cell_ref.as_str()),
                ("fabric_ref", self.fabric_ref.as_str()),
                ("resource_id", request.dns_zone.resource_id.as_str()),
                ("tenant_id", request.dns_zone.tenant_id.as_str()),
                ("region", request.dns_zone.region.as_str()),
                ("name", request.dns_zone.name.as_str()),
                ("kind", dns_zone_kind_label(request.dns_zone.kind)),
                ("vpc_id", vpc_id),
                ("dnssec_enabled", dnssec_enabled.as_str()),
                ("dnssec_key_ref", dnssec_key_ref),
                ("actor", request.actor.as_str()),
                ("idempotency_key", request.idempotency_key.as_str()),
            ]),
            provider_evidence_ref: format!(
                "selfhosted-dns-zone://{}/{}/{}/{}/{}",
                self.site_ref,
                self.cell_ref,
                self.fabric_ref,
                request.dns_zone.resource_id,
                request.request_id
            ),
        })
    }

    fn ensure_provider_dns_zone(
        &self,
        provider_dns_zone_ref: &str,
        dns_zone_resource_id: &str,
    ) -> Result<(), NetworkProviderDnsZoneError> {
        let expected = self.provider_dns_zone_ref(dns_zone_resource_id);
        if provider_dns_zone_ref == expected {
            Ok(())
        } else {
            Err(NetworkProviderDnsZoneError::ProviderRejected {
                provider: NetworkProviderKind::SelfHostedColoDnsZone,
                reason:
                    "provider_dns_zone_ref does not match configured self-hosted colo DNS target"
                        .to_string(),
            })
        }
    }

    fn provider_request_id(&self, request_id: &str) -> String {
        format!(
            "selfhosted-dns-zone-{}-{request_id}",
            self.clock_epoch_seconds
        )
    }
}

impl NetworkProviderVpcPort for SelfHostedColoVpcAdapter {
    fn provider_kind(&self) -> NetworkProviderKind {
        NetworkProviderKind::SelfHostedColoVpc
    }

    fn create_vpc(
        &self,
        input: NetworkProviderVpcCreateRequest,
    ) -> Result<NetworkProviderVpcReceipt, NetworkProviderVpcError> {
        let command = self.create_network_segment_command(&input)?;
        NetworkProviderVpcReceipt::create_vpc(
            self.provider_kind(),
            input.clone(),
            self.provider_request_id(&input.request_id),
            command.provider_evidence_ref,
        )
    }
}

impl NetworkProviderDnsZonePort for SelfHostedColoDnsZoneAdapter {
    fn provider_kind(&self) -> NetworkProviderKind {
        NetworkProviderKind::SelfHostedColoDnsZone
    }

    fn create_dns_zone(
        &self,
        input: NetworkProviderDnsZoneCreateRequest,
    ) -> Result<NetworkProviderDnsZoneReceipt, NetworkProviderDnsZoneError> {
        let command = self.create_authoritative_zone_command(&input)?;
        NetworkProviderDnsZoneReceipt::create_dns_zone(
            self.provider_kind(),
            input.clone(),
            self.provider_request_id(&input.request_id),
            command.provider_evidence_ref,
        )
    }
}

fn validate_endpoint(value: &str) -> Result<(), SelfHostedColoVpcAdapterConfigError> {
    if value.starts_with("https://") && no_space_or_control(value) {
        Ok(())
    } else {
        Err(SelfHostedColoVpcAdapterConfigError::InvalidEndpoint)
    }
}

fn validate_segment(
    value: &str,
    error: SelfHostedColoVpcAdapterConfigError,
) -> Result<(), SelfHostedColoVpcAdapterConfigError> {
    if value.trim().is_empty() || value.contains('/') || !no_space_or_control(value) {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_dns_endpoint(value: &str) -> Result<(), SelfHostedColoDnsZoneAdapterConfigError> {
    if value.starts_with("https://") && no_space_or_control(value) {
        Ok(())
    } else {
        Err(SelfHostedColoDnsZoneAdapterConfigError::InvalidEndpoint)
    }
}

fn validate_dns_segment(
    value: &str,
    error: SelfHostedColoDnsZoneAdapterConfigError,
) -> Result<(), SelfHostedColoDnsZoneAdapterConfigError> {
    if value.trim().is_empty() || value.contains('/') || !no_space_or_control(value) {
        Err(error)
    } else {
        Ok(())
    }
}

const fn dns_zone_kind_label(kind: network_domain::DnsZoneKind) -> &'static str {
    match kind {
        network_domain::DnsZoneKind::Public => "public",
        network_domain::DnsZoneKind::Private => "private",
    }
}

fn no_space_or_control(value: &str) -> bool {
    !value
        .bytes()
        .any(|byte| byte.is_ascii_control() || byte == b' ')
}

fn canonical_body(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_boundary_kernel::DataClass;
    use network_domain::{
        CloudNetworkError, DnsZoneCreate, DnsZoneKind, DnsZoneState, IpProtocol, Ipv4Cidr,
        NetworkProviderDnsZoneError, NetworkProviderDnsZonePort, NetworkProviderVpcPort,
        RouteCreate, RouteDestination, RouteNextHopKind, RouteTableCreate, RuleDirection,
        SecurityGroupCreate, SecurityRule, VpcCreate, VpcState,
    };
    use network_residency::ResidencyClass;

    const SITE_REF: &str = "kr-seoul-colo-a";
    const CELL_REF: &str = "cell-kr-seoul-a";
    const FABRIC_REF: &str = "fabric-ovn-frr-a";
    const VPC_ID: &str = "oya:cloud:alpha-region:ten_alpha:vpc:prod";
    const DNS_ZONE_ID: &str = "oya:cloud:alpha-region:ten_alpha:dns-zone:example-com";

    fn adapter() -> SelfHostedColoVpcAdapter {
        SelfHostedColoVpcAdapter::new(
            "https://network-control.kr-seoul-1.oyatie.local",
            SITE_REF,
            CELL_REF,
            FABRIC_REF,
        )
        .unwrap()
        .with_clock(1_700_000_000)
    }

    fn dns_adapter() -> SelfHostedColoDnsZoneAdapter {
        SelfHostedColoDnsZoneAdapter::new(
            "https://network-control.kr-seoul-1.oyatie.local",
            SITE_REF,
            CELL_REF,
            FABRIC_REF,
        )
        .unwrap()
        .with_clock(1_700_000_100)
    }

    fn request() -> NetworkProviderVpcCreateRequest {
        NetworkProviderVpcCreateRequest {
            request_id: "networkprov_req_selfhosted_vpc_create_001".to_string(),
            provider_vcn_ref: format!(
                "selfhosted-vpc://{SITE_REF}/{CELL_REF}/{FABRIC_REF}/{VPC_ID}"
            ),
            vpc: VpcCreate {
                resource_id: VPC_ID.to_string(),
                tenant_id: "ten_alpha".to_string(),
                region: "alpha-region".to_string(),
                cidr_v4: "10.42.0.0/16".to_string(),
                cidr_v6: "2001:db8:42::/56".to_string(),
                flow_logs_enabled: true,
                route_table: RouteTableCreate {
                    id: "rtb_main".to_string(),
                    routes: vec![RouteCreate {
                        destination: "0.0.0.0/0".to_string(),
                        next_hop: RouteNextHopKind::NatGateway,
                        target_ref: Some("nat/alpha-region/nonprod".to_string()),
                    }],
                },
                security_groups: vec![SecurityGroupCreate {
                    id: "sg_web".to_string(),
                    rules: vec![SecurityRule {
                        direction: RuleDirection::Ingress,
                        protocol: IpProtocol::Tcp,
                        port_range: Some((443, 443)),
                        cidr: RouteDestination::Ipv4(Ipv4Cidr::new("10.42.0.0/16").unwrap()),
                        description: "tenant https ingress".to_string(),
                    }],
                }],
                residency: ResidencyClass::Global,
                state: VpcState::Creating,
                data_class: DataClass::Public,
                created_at_epoch_seconds: 1_700_000_000,
            },
            actor: "sp_network".to_string(),
            idempotency_key: "idem-selfhosted-network-vpc-create".to_string(),
            requested_at_epoch_seconds: 1_700_000_010,
        }
    }

    fn dns_request() -> NetworkProviderDnsZoneCreateRequest {
        NetworkProviderDnsZoneCreateRequest {
            request_id: "networkprov_req_selfhosted_dns_create_001".to_string(),
            provider_dns_zone_ref: format!(
                "selfhosted-dns-zone://{SITE_REF}/{CELL_REF}/{FABRIC_REF}/{DNS_ZONE_ID}"
            ),
            vpc: None,
            dns_zone: DnsZoneCreate {
                resource_id: DNS_ZONE_ID.to_string(),
                tenant_id: "ten_alpha".to_string(),
                region: "alpha-region".to_string(),
                name: "example.com".to_string(),
                kind: DnsZoneKind::Public,
                vpc_id: None,
                dnssec_key_ref: Some("dnssec/alpha-region/ten_alpha/example-com".to_string()),
                state: DnsZoneState::Creating,
                data_class: DataClass::Public,
                created_at_epoch_seconds: 1_700_000_030,
            },
            actor: "sp_network".to_string(),
            idempotency_key: "idem-selfhosted-network-dns-zone-create".to_string(),
            requested_at_epoch_seconds: 1_700_000_110,
        }
    }

    #[test]
    fn create_network_segment_command_uses_self_hosted_path_and_reference_only_body() {
        let command = adapter()
            .create_network_segment_command(&request())
            .expect("valid VPC request becomes deterministic self-hosted VPC command");

        assert_eq!(command.operation, "CreateTenantNetworkSegment");
        assert_eq!(command.method, "POST");
        assert_eq!(
            command.endpoint_origin,
            "https://network-control.kr-seoul-1.oyatie.local"
        );
        assert_eq!(
            command.path,
            "/v1/sites/kr-seoul-colo-a/cells/cell-kr-seoul-a/network-segments"
        );
        assert!(command.body_canonical.contains("site_ref=kr-seoul-colo-a"));
        assert!(command.body_canonical.contains("cell_ref=cell-kr-seoul-a"));
        assert!(
            command
                .body_canonical
                .contains("fabric_ref=fabric-ovn-frr-a")
        );
        assert!(command.body_canonical.contains("cidr_v4=10.42.0.0/16"));
        assert!(command.body_canonical.contains("cidr_v6=2001:db8:42::/56"));
        assert!(command.body_canonical.contains("flow_logs_enabled=true"));
        assert!(!command.body_canonical.contains("private_key"));
        assert!(!command.body_canonical.contains("token"));
        assert_eq!(
            command.provider_evidence_ref,
            format!(
                "selfhosted-vpc://{SITE_REF}/{CELL_REF}/{FABRIC_REF}/{VPC_ID}/networkprov_req_selfhosted_vpc_create_001"
            )
        );
    }

    #[test]
    fn vpc_port_receipts_preserve_self_hosted_refs_without_provider_credentials() {
        let receipt = adapter()
            .create_vpc(request())
            .expect("self-hosted VPC receipt is generated");

        assert_eq!(receipt.provider, NetworkProviderKind::SelfHostedColoVpc);
        assert_eq!(receipt.provider.label(), "selfhosted_colo_vpc");
        assert_eq!(
            receipt.provider_request_id,
            "selfhosted-vpc-1700000000-networkprov_req_selfhosted_vpc_create_001"
        );
        assert_eq!(
            receipt.provider_vcn_ref,
            format!("selfhosted-vpc://{SITE_REF}/{CELL_REF}/{FABRIC_REF}/{VPC_ID}")
        );
        assert_eq!(receipt.resource_id, VPC_ID);
        assert_eq!(receipt.actor, "sp_network");
        assert!(receipt.flow_logs_enabled);
    }

    #[test]
    fn create_authoritative_zone_command_uses_self_hosted_dns_path_and_reference_only_body() {
        let command = dns_adapter()
            .create_authoritative_zone_command(&dns_request())
            .expect("valid DNS request becomes deterministic self-hosted DNS command");

        assert_eq!(command.operation, "CreateAuthoritativeDnsZone");
        assert_eq!(command.method, "POST");
        assert_eq!(
            command.endpoint_origin,
            "https://network-control.kr-seoul-1.oyatie.local"
        );
        assert_eq!(
            command.path,
            "/v1/sites/kr-seoul-colo-a/cells/cell-kr-seoul-a/dns/zones"
        );
        assert!(command.body_canonical.contains("site_ref=kr-seoul-colo-a"));
        assert!(command.body_canonical.contains("cell_ref=cell-kr-seoul-a"));
        assert!(
            command
                .body_canonical
                .contains("fabric_ref=fabric-ovn-frr-a")
        );
        assert!(command.body_canonical.contains("name=example.com"));
        assert!(command.body_canonical.contains("kind=public"));
        assert!(command.body_canonical.contains("vpc_id=none"));
        assert!(command.body_canonical.contains("dnssec_enabled=true"));
        assert!(!command.body_canonical.contains("private_key"));
        assert!(!command.body_canonical.contains("token"));
        assert_eq!(
            command.provider_evidence_ref,
            format!(
                "selfhosted-dns-zone://{SITE_REF}/{CELL_REF}/{FABRIC_REF}/{DNS_ZONE_ID}/networkprov_req_selfhosted_dns_create_001"
            )
        );
    }

    #[test]
    fn dns_zone_port_receipts_preserve_self_hosted_refs_without_provider_credentials() {
        let receipt = dns_adapter()
            .create_dns_zone(dns_request())
            .expect("self-hosted DNS zone receipt is generated");

        assert_eq!(receipt.provider, NetworkProviderKind::SelfHostedColoDnsZone);
        assert_eq!(receipt.provider.label(), "selfhosted_colo_dns_zone");
        assert_eq!(
            receipt.provider_request_id,
            "selfhosted-dns-zone-1700000100-networkprov_req_selfhosted_dns_create_001"
        );
        assert_eq!(
            receipt.provider_dns_zone_ref,
            format!("selfhosted-dns-zone://{SITE_REF}/{CELL_REF}/{FABRIC_REF}/{DNS_ZONE_ID}")
        );
        assert_eq!(receipt.resource_id, DNS_ZONE_ID);
        assert_eq!(receipt.name, "example.com");
        assert!(receipt.dnssec_enabled);
    }

    #[test]
    fn rejects_provider_vcn_drift_and_bad_vpc_shape() {
        let mut drifted = request();
        drifted.provider_vcn_ref =
            "selfhosted-vpc://other/cell-kr-seoul-a/fabric-ovn-frr-a/vpc".to_string();
        assert!(matches!(
            adapter().create_network_segment_command(&drifted),
            Err(NetworkProviderVpcError::ProviderRejected { .. })
        ));

        let mut bad_vpc = request();
        bad_vpc.vpc.flow_logs_enabled = false;
        assert_eq!(
            bad_vpc.validate(),
            Err(NetworkProviderVpcError::InvalidRequestShape(
                CloudNetworkError::FlowLogsRequired,
            ))
        );
    }

    #[test]
    fn rejects_provider_dns_zone_drift_and_bad_dns_shape() {
        let mut drifted = dns_request();
        drifted.provider_dns_zone_ref =
            "selfhosted-dns-zone://other/cell-kr-seoul-a/fabric-ovn-frr-a/dns".to_string();
        assert!(matches!(
            dns_adapter().create_authoritative_zone_command(&drifted),
            Err(NetworkProviderDnsZoneError::ProviderRejected { .. })
        ));

        let mut bad_dns = dns_request();
        bad_dns.dns_zone.dnssec_key_ref = None;
        assert_eq!(
            bad_dns.validate(),
            Err(NetworkProviderDnsZoneError::InvalidRequestShape(
                CloudNetworkError::DnssecRequired,
            ))
        );
    }

    #[test]
    fn rejects_invalid_self_hosted_adapter_config() {
        assert_eq!(
            SelfHostedColoVpcAdapter::new("http://network-control", SITE_REF, CELL_REF, FABRIC_REF),
            Err(SelfHostedColoVpcAdapterConfigError::InvalidEndpoint)
        );
        assert_eq!(
            SelfHostedColoVpcAdapter::new(
                "https://network-control.kr-seoul-1.oyatie.local",
                "kr/seoul",
                CELL_REF,
                FABRIC_REF,
            ),
            Err(SelfHostedColoVpcAdapterConfigError::InvalidSiteRef)
        );
        assert_eq!(
            SelfHostedColoVpcAdapter::new(
                "https://network-control.kr-seoul-1.oyatie.local",
                SITE_REF,
                "cell kr seoul",
                FABRIC_REF,
            ),
            Err(SelfHostedColoVpcAdapterConfigError::InvalidCellRef)
        );
        assert_eq!(
            SelfHostedColoVpcAdapter::new(
                "https://network-control.kr-seoul-1.oyatie.local",
                SITE_REF,
                CELL_REF,
                "",
            ),
            Err(SelfHostedColoVpcAdapterConfigError::InvalidFabricRef)
        );
        assert_eq!(
            SelfHostedColoDnsZoneAdapter::new(
                "http://network-control",
                SITE_REF,
                CELL_REF,
                FABRIC_REF,
            ),
            Err(SelfHostedColoDnsZoneAdapterConfigError::InvalidEndpoint)
        );
        assert_eq!(
            SelfHostedColoDnsZoneAdapter::new(
                "https://network-control.kr-seoul-1.oyatie.local",
                "kr/seoul",
                CELL_REF,
                FABRIC_REF,
            ),
            Err(SelfHostedColoDnsZoneAdapterConfigError::InvalidSiteRef)
        );
        assert_eq!(
            SelfHostedColoDnsZoneAdapter::new(
                "https://network-control.kr-seoul-1.oyatie.local",
                SITE_REF,
                "cell kr seoul",
                FABRIC_REF,
            ),
            Err(SelfHostedColoDnsZoneAdapterConfigError::InvalidCellRef)
        );
        assert_eq!(
            SelfHostedColoDnsZoneAdapter::new(
                "https://network-control.kr-seoul-1.oyatie.local",
                SITE_REF,
                CELL_REF,
                "",
            ),
            Err(SelfHostedColoDnsZoneAdapterConfigError::InvalidFabricRef)
        );
    }
}
