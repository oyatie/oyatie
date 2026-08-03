//! OCI network adapter boundary for Cloud Network.
//!
//! This crate keeps OCI compartment, region, VCN path, and evidence refs outside
//! provider-neutral Cloud Network domain/API crates while implementing the VPC
//! provider port contract. It builds deterministic request shapes only;
//! credentialed live smoke remains a separate promotion gate.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use network_domain::{
    NetworkProviderDirectInterconnectCreateRequest, NetworkProviderDirectInterconnectError,
    NetworkProviderDirectInterconnectPort, NetworkProviderDirectInterconnectReceipt,
    NetworkProviderDnsZoneCreateRequest, NetworkProviderDnsZoneError, NetworkProviderDnsZonePort,
    NetworkProviderDnsZoneReceipt, NetworkProviderKind, NetworkProviderLoadBalancerCreateRequest,
    NetworkProviderLoadBalancerError, NetworkProviderLoadBalancerPort,
    NetworkProviderLoadBalancerReceipt, NetworkProviderVpcCreateRequest, NetworkProviderVpcError,
    NetworkProviderVpcPort, NetworkProviderVpcReceipt,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OciVcnAdapterConfigError {
    InvalidEndpoint,
    InvalidCompartmentRef,
    InvalidRegion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciVcnAdapter {
    endpoint_origin: String,  // data_class: INTERNAL_ONLY
    compartment_ref: String,  // data_class: INTERNAL_ONLY
    region: String,           // data_class: PUBLIC
    clock_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciVcnCommand {
    pub operation: &'static str,       // data_class: PUBLIC
    pub method: &'static str,          // data_class: PUBLIC
    pub endpoint_origin: String,       // data_class: INTERNAL_ONLY
    pub path: String,                  // data_class: INTERNAL_ONLY
    pub body_canonical: String,        // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OciLoadBalancerAdapterConfigError {
    InvalidEndpoint,
    InvalidCompartmentRef,
    InvalidRegion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciLoadBalancerAdapter {
    endpoint_origin: String,  // data_class: INTERNAL_ONLY
    compartment_ref: String,  // data_class: INTERNAL_ONLY
    region: String,           // data_class: PUBLIC
    clock_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciLoadBalancerCommand {
    pub operation: &'static str,       // data_class: PUBLIC
    pub method: &'static str,          // data_class: PUBLIC
    pub endpoint_origin: String,       // data_class: INTERNAL_ONLY
    pub path: String,                  // data_class: INTERNAL_ONLY
    pub body_canonical: String,        // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OciDnsZoneAdapterConfigError {
    InvalidEndpoint,
    InvalidCompartmentRef,
    InvalidRegion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciDnsZoneAdapter {
    endpoint_origin: String,  // data_class: INTERNAL_ONLY
    compartment_ref: String,  // data_class: INTERNAL_ONLY
    region: String,           // data_class: PUBLIC
    clock_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciDnsZoneCommand {
    pub operation: &'static str,       // data_class: PUBLIC
    pub method: &'static str,          // data_class: PUBLIC
    pub endpoint_origin: String,       // data_class: INTERNAL_ONLY
    pub path: String,                  // data_class: INTERNAL_ONLY
    pub body_canonical: String,        // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OciDirectInterconnectAdapterConfigError {
    InvalidEndpoint,
    InvalidCompartmentRef,
    InvalidRegion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciDirectInterconnectAdapter {
    endpoint_origin: String,  // data_class: INTERNAL_ONLY
    compartment_ref: String,  // data_class: INTERNAL_ONLY
    region: String,           // data_class: PUBLIC
    clock_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciDirectInterconnectCommand {
    pub operation: &'static str,       // data_class: PUBLIC
    pub method: &'static str,          // data_class: PUBLIC
    pub endpoint_origin: String,       // data_class: INTERNAL_ONLY
    pub path: String,                  // data_class: INTERNAL_ONLY
    pub body_canonical: String,        // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String, // data_class: INTERNAL_ONLY
}

impl OciVcnAdapter {
    pub fn new(
        endpoint_origin: impl Into<String>,
        compartment_ref: impl Into<String>,
        region: impl Into<String>,
    ) -> Result<Self, OciVcnAdapterConfigError> {
        let endpoint_origin = endpoint_origin.into();
        let compartment_ref = compartment_ref.into();
        let region = region.into();
        validate_endpoint(&endpoint_origin)?;
        validate_segment(
            &compartment_ref,
            OciVcnAdapterConfigError::InvalidCompartmentRef,
        )?;
        validate_segment(&region, OciVcnAdapterConfigError::InvalidRegion)?;
        Ok(Self {
            endpoint_origin,
            compartment_ref,
            region,
            clock_epoch_seconds: 0,
        })
    }

    pub fn with_clock(mut self, clock_epoch_seconds: u64) -> Self {
        self.clock_epoch_seconds = clock_epoch_seconds;
        self
    }

    pub fn provider_vcn_ref(&self, vpc_resource_id: &str) -> String {
        format!(
            "oci-vcn://{}/{}/{}",
            self.compartment_ref, self.region, vpc_resource_id
        )
    }

    pub fn create_vcn_command(
        &self,
        request: &NetworkProviderVpcCreateRequest,
    ) -> Result<OciVcnCommand, NetworkProviderVpcError> {
        request.validate()?;
        self.ensure_provider_vcn(&request.provider_vcn_ref, &request.vpc.resource_id)?;
        let flow_logs_enabled = request.vpc.flow_logs_enabled.to_string();
        let security_group_count = request.vpc.security_groups.len().to_string();
        let route_count = request.vpc.route_table.routes.len().to_string();
        Ok(OciVcnCommand {
            operation: "CreateVcn",
            method: "POST",
            endpoint_origin: self.endpoint_origin.clone(),
            path: "/20160918/vcns".to_string(),
            body_canonical: canonical_body(&[
                ("compartment_ref", self.compartment_ref.as_str()),
                ("region", self.region.as_str()),
                ("resource_id", request.vpc.resource_id.as_str()),
                ("tenant_id", request.vpc.tenant_id.as_str()),
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
                "oci-vcn://{}/{}/{}/{}",
                self.compartment_ref, self.region, request.vpc.resource_id, request.request_id
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
                provider: NetworkProviderKind::OciVcn,
                reason: "provider_vcn_ref does not match configured OCI VCN target".to_string(),
            })
        }
    }

    fn provider_request_id(&self, request_id: &str) -> String {
        format!("oci-vcn-{}-{request_id}", self.clock_epoch_seconds)
    }
}

impl NetworkProviderVpcPort for OciVcnAdapter {
    fn provider_kind(&self) -> NetworkProviderKind {
        NetworkProviderKind::OciVcn
    }

    fn create_vpc(
        &self,
        input: NetworkProviderVpcCreateRequest,
    ) -> Result<NetworkProviderVpcReceipt, NetworkProviderVpcError> {
        let command = self.create_vcn_command(&input)?;
        NetworkProviderVpcReceipt::create_vpc(
            self.provider_kind(),
            input.clone(),
            self.provider_request_id(&input.request_id),
            command.provider_evidence_ref,
        )
    }
}

impl OciLoadBalancerAdapter {
    pub fn new(
        endpoint_origin: impl Into<String>,
        compartment_ref: impl Into<String>,
        region: impl Into<String>,
    ) -> Result<Self, OciLoadBalancerAdapterConfigError> {
        let endpoint_origin = endpoint_origin.into();
        let compartment_ref = compartment_ref.into();
        let region = region.into();
        validate_lb_endpoint(&endpoint_origin)?;
        validate_lb_segment(
            &compartment_ref,
            OciLoadBalancerAdapterConfigError::InvalidCompartmentRef,
        )?;
        validate_lb_segment(&region, OciLoadBalancerAdapterConfigError::InvalidRegion)?;
        Ok(Self {
            endpoint_origin,
            compartment_ref,
            region,
            clock_epoch_seconds: 0,
        })
    }

    pub fn with_clock(mut self, clock_epoch_seconds: u64) -> Self {
        self.clock_epoch_seconds = clock_epoch_seconds;
        self
    }

    pub fn provider_load_balancer_ref(&self, load_balancer_resource_id: &str) -> String {
        format!(
            "oci-lb://{}/{}/{}",
            self.compartment_ref, self.region, load_balancer_resource_id
        )
    }

    pub fn create_load_balancer_command(
        &self,
        request: &NetworkProviderLoadBalancerCreateRequest,
    ) -> Result<OciLoadBalancerCommand, NetworkProviderLoadBalancerError> {
        request.validate()?;
        self.ensure_provider_load_balancer(
            &request.provider_load_balancer_ref,
            &request.load_balancer.resource_id,
        )?;
        let listener_count = request.load_balancer.listeners.len().to_string();
        let target_group_count = request.load_balancer.target_groups.len().to_string();
        let mtls_enabled = request.load_balancer.mtls.is_some().to_string();
        let waf_policy = request
            .load_balancer
            .waf_policy
            .as_deref()
            .unwrap_or("none");
        Ok(OciLoadBalancerCommand {
            operation: "CreateLoadBalancer",
            method: "POST",
            endpoint_origin: self.endpoint_origin.clone(),
            path: "/20170115/loadBalancers".to_string(),
            body_canonical: canonical_body(&[
                ("compartment_ref", self.compartment_ref.as_str()),
                ("region", self.region.as_str()),
                ("resource_id", request.load_balancer.resource_id.as_str()),
                ("tenant_id", request.load_balancer.tenant_id.as_str()),
                ("vpc_id", request.load_balancer.vpc_id.as_str()),
                ("kind", lb_kind_label(request.load_balancer.kind)),
                ("listener_count", listener_count.as_str()),
                ("target_group_count", target_group_count.as_str()),
                ("mtls_enabled", mtls_enabled.as_str()),
                ("waf_policy", waf_policy),
                ("actor", request.actor.as_str()),
                ("idempotency_key", request.idempotency_key.as_str()),
            ]),
            provider_evidence_ref: format!(
                "oci-lb://{}/{}/{}/{}",
                self.compartment_ref,
                self.region,
                request.load_balancer.resource_id,
                request.request_id
            ),
        })
    }

    fn ensure_provider_load_balancer(
        &self,
        provider_load_balancer_ref: &str,
        load_balancer_resource_id: &str,
    ) -> Result<(), NetworkProviderLoadBalancerError> {
        let expected = self.provider_load_balancer_ref(load_balancer_resource_id);
        if provider_load_balancer_ref == expected {
            Ok(())
        } else {
            Err(NetworkProviderLoadBalancerError::ProviderRejected {
                provider: NetworkProviderKind::OciLoadBalancer,
                reason:
                    "provider_load_balancer_ref does not match configured OCI load balancer target"
                        .to_string(),
            })
        }
    }

    fn provider_request_id(&self, request_id: &str) -> String {
        format!("oci-lb-{}-{request_id}", self.clock_epoch_seconds)
    }
}

impl NetworkProviderLoadBalancerPort for OciLoadBalancerAdapter {
    fn provider_kind(&self) -> NetworkProviderKind {
        NetworkProviderKind::OciLoadBalancer
    }

    fn create_load_balancer(
        &self,
        input: NetworkProviderLoadBalancerCreateRequest,
    ) -> Result<NetworkProviderLoadBalancerReceipt, NetworkProviderLoadBalancerError> {
        let command = self.create_load_balancer_command(&input)?;
        NetworkProviderLoadBalancerReceipt::create_load_balancer(
            self.provider_kind(),
            input.clone(),
            self.provider_request_id(&input.request_id),
            command.provider_evidence_ref,
        )
    }
}

impl OciDnsZoneAdapter {
    pub fn new(
        endpoint_origin: impl Into<String>,
        compartment_ref: impl Into<String>,
        region: impl Into<String>,
    ) -> Result<Self, OciDnsZoneAdapterConfigError> {
        let endpoint_origin = endpoint_origin.into();
        let compartment_ref = compartment_ref.into();
        let region = region.into();
        validate_dns_endpoint(&endpoint_origin)?;
        validate_dns_segment(
            &compartment_ref,
            OciDnsZoneAdapterConfigError::InvalidCompartmentRef,
        )?;
        validate_dns_segment(&region, OciDnsZoneAdapterConfigError::InvalidRegion)?;
        Ok(Self {
            endpoint_origin,
            compartment_ref,
            region,
            clock_epoch_seconds: 0,
        })
    }

    pub fn with_clock(mut self, clock_epoch_seconds: u64) -> Self {
        self.clock_epoch_seconds = clock_epoch_seconds;
        self
    }

    pub fn provider_dns_zone_ref(&self, dns_zone_resource_id: &str) -> String {
        format!(
            "oci-dns-zone://{}/{}/{}",
            self.compartment_ref, self.region, dns_zone_resource_id
        )
    }

    pub fn create_dns_zone_command(
        &self,
        request: &NetworkProviderDnsZoneCreateRequest,
    ) -> Result<OciDnsZoneCommand, NetworkProviderDnsZoneError> {
        request.validate()?;
        self.ensure_provider_dns_zone(
            &request.provider_dns_zone_ref,
            &request.dns_zone.resource_id,
        )?;
        let vpc_id = request.dns_zone.vpc_id.as_deref().unwrap_or("none");
        let dnssec_enabled = request.dns_zone.dnssec_key_ref.is_some().to_string();
        let dnssec_key_ref = request.dns_zone.dnssec_key_ref.as_deref().unwrap_or("none");
        Ok(OciDnsZoneCommand {
            operation: "CreateZone",
            method: "POST",
            endpoint_origin: self.endpoint_origin.clone(),
            path: "/20180115/zones".to_string(),
            body_canonical: canonical_body(&[
                ("compartment_ref", self.compartment_ref.as_str()),
                ("region", self.region.as_str()),
                ("resource_id", request.dns_zone.resource_id.as_str()),
                ("tenant_id", request.dns_zone.tenant_id.as_str()),
                ("name", request.dns_zone.name.as_str()),
                ("kind", dns_zone_kind_label(request.dns_zone.kind)),
                ("scope", dns_zone_scope_label(request.dns_zone.kind)),
                ("zone_type", "PRIMARY"),
                ("vpc_id", vpc_id),
                ("dnssec_enabled", dnssec_enabled.as_str()),
                ("dnssec_key_ref", dnssec_key_ref),
                ("actor", request.actor.as_str()),
                ("idempotency_key", request.idempotency_key.as_str()),
            ]),
            provider_evidence_ref: format!(
                "oci-dns-zone://{}/{}/{}/{}",
                self.compartment_ref, self.region, request.dns_zone.resource_id, request.request_id
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
                provider: NetworkProviderKind::OciDnsZone,
                reason: "provider_dns_zone_ref does not match configured OCI DNS zone target"
                    .to_string(),
            })
        }
    }

    fn provider_request_id(&self, request_id: &str) -> String {
        format!("oci-dns-zone-{}-{request_id}", self.clock_epoch_seconds)
    }
}

impl NetworkProviderDnsZonePort for OciDnsZoneAdapter {
    fn provider_kind(&self) -> NetworkProviderKind {
        NetworkProviderKind::OciDnsZone
    }

    fn create_dns_zone(
        &self,
        input: NetworkProviderDnsZoneCreateRequest,
    ) -> Result<NetworkProviderDnsZoneReceipt, NetworkProviderDnsZoneError> {
        let command = self.create_dns_zone_command(&input)?;
        NetworkProviderDnsZoneReceipt::create_dns_zone(
            self.provider_kind(),
            input.clone(),
            self.provider_request_id(&input.request_id),
            command.provider_evidence_ref,
        )
    }
}

impl OciDirectInterconnectAdapter {
    pub fn new(
        endpoint_origin: impl Into<String>,
        compartment_ref: impl Into<String>,
        region: impl Into<String>,
    ) -> Result<Self, OciDirectInterconnectAdapterConfigError> {
        let endpoint_origin = endpoint_origin.into();
        let compartment_ref = compartment_ref.into();
        let region = region.into();
        validate_interconnect_endpoint(&endpoint_origin)?;
        validate_interconnect_segment(
            &compartment_ref,
            OciDirectInterconnectAdapterConfigError::InvalidCompartmentRef,
        )?;
        validate_interconnect_segment(
            &region,
            OciDirectInterconnectAdapterConfigError::InvalidRegion,
        )?;
        Ok(Self {
            endpoint_origin,
            compartment_ref,
            region,
            clock_epoch_seconds: 0,
        })
    }

    pub fn with_clock(mut self, clock_epoch_seconds: u64) -> Self {
        self.clock_epoch_seconds = clock_epoch_seconds;
        self
    }

    pub fn provider_virtual_circuit_ref(&self, direct_interconnect_resource_id: &str) -> String {
        format!(
            "oci-fast-connect://{}/{}/{}",
            self.compartment_ref, self.region, direct_interconnect_resource_id
        )
    }

    pub fn create_virtual_circuit_command(
        &self,
        request: &NetworkProviderDirectInterconnectCreateRequest,
    ) -> Result<OciDirectInterconnectCommand, NetworkProviderDirectInterconnectError> {
        request.validate()?;
        self.ensure_provider_virtual_circuit(
            &request.provider_virtual_circuit_ref,
            &request.direct_interconnect.resource_id,
        )?;
        let bandwidth_mbps = request.direct_interconnect.bandwidth_mbps.to_string();
        let vlan_tag = request.direct_interconnect.vlan_tag.to_string();
        let redundant_port_count = request.direct_interconnect.redundant_port_count.to_string();
        let bgp_session_count = request.direct_interconnect.bgp_sessions.len().to_string();
        let advertised_prefix_count = request
            .direct_interconnect
            .advertised_prefixes
            .len()
            .to_string();
        Ok(OciDirectInterconnectCommand {
            operation: "CreateVirtualCircuit",
            method: "POST",
            endpoint_origin: self.endpoint_origin.clone(),
            path: "/20160918/virtualCircuits".to_string(),
            body_canonical: canonical_body(&[
                ("compartment_ref", self.compartment_ref.as_str()),
                ("region", self.region.as_str()),
                (
                    "resource_id",
                    request.direct_interconnect.resource_id.as_str(),
                ),
                ("tenant_id", request.direct_interconnect.tenant_id.as_str()),
                ("virtual_circuit_type", "PRIVATE"),
                (
                    "partner_id",
                    request.direct_interconnect.partner_id.as_str(),
                ),
                (
                    "peering_location",
                    request.direct_interconnect.peering_location.as_str(),
                ),
                (
                    "physical_port_id",
                    request.direct_interconnect.physical_port_id.as_str(),
                ),
                ("vlan_tag", vlan_tag.as_str()),
                ("bandwidth_mbps", bandwidth_mbps.as_str()),
                ("redundant_port_count", redundant_port_count.as_str()),
                ("bgp_session_count", bgp_session_count.as_str()),
                ("advertised_prefix_count", advertised_prefix_count.as_str()),
                ("actor", request.actor.as_str()),
                ("idempotency_key", request.idempotency_key.as_str()),
            ]),
            provider_evidence_ref: format!(
                "oci-fast-connect://{}/{}/{}/{}",
                self.compartment_ref,
                self.region,
                request.direct_interconnect.resource_id,
                request.request_id
            ),
        })
    }

    fn ensure_provider_virtual_circuit(
        &self,
        provider_virtual_circuit_ref: &str,
        direct_interconnect_resource_id: &str,
    ) -> Result<(), NetworkProviderDirectInterconnectError> {
        let expected = self.provider_virtual_circuit_ref(direct_interconnect_resource_id);
        if provider_virtual_circuit_ref == expected {
            Ok(())
        } else {
            Err(NetworkProviderDirectInterconnectError::ProviderRejected {
                provider: NetworkProviderKind::OciFastConnect,
                reason: "provider_virtual_circuit_ref does not match configured OCI Fasttarget"
                    .to_string(),
            })
        }
    }

    fn provider_request_id(&self, request_id: &str) -> String {
        format!("oci-fast-connect-{}-{request_id}", self.clock_epoch_seconds)
    }
}

impl NetworkProviderDirectInterconnectPort for OciDirectInterconnectAdapter {
    fn provider_kind(&self) -> NetworkProviderKind {
        NetworkProviderKind::OciFastConnect
    }

    fn create_direct_interconnect(
        &self,
        input: NetworkProviderDirectInterconnectCreateRequest,
    ) -> Result<NetworkProviderDirectInterconnectReceipt, NetworkProviderDirectInterconnectError>
    {
        let command = self.create_virtual_circuit_command(&input)?;
        NetworkProviderDirectInterconnectReceipt::create_direct_interconnect(
            self.provider_kind(),
            input.clone(),
            self.provider_request_id(&input.request_id),
            command.provider_evidence_ref,
        )
    }
}

fn validate_endpoint(value: &str) -> Result<(), OciVcnAdapterConfigError> {
    if value.starts_with("https://") && no_space_or_control(value) {
        Ok(())
    } else {
        Err(OciVcnAdapterConfigError::InvalidEndpoint)
    }
}

fn validate_lb_endpoint(value: &str) -> Result<(), OciLoadBalancerAdapterConfigError> {
    if value.starts_with("https://") && no_space_or_control(value) {
        Ok(())
    } else {
        Err(OciLoadBalancerAdapterConfigError::InvalidEndpoint)
    }
}

fn validate_dns_endpoint(value: &str) -> Result<(), OciDnsZoneAdapterConfigError> {
    if value.starts_with("https://") && no_space_or_control(value) {
        Ok(())
    } else {
        Err(OciDnsZoneAdapterConfigError::InvalidEndpoint)
    }
}

fn validate_interconnect_endpoint(
    value: &str,
) -> Result<(), OciDirectInterconnectAdapterConfigError> {
    if value.starts_with("https://") && no_space_or_control(value) {
        Ok(())
    } else {
        Err(OciDirectInterconnectAdapterConfigError::InvalidEndpoint)
    }
}

fn validate_segment(
    value: &str,
    error: OciVcnAdapterConfigError,
) -> Result<(), OciVcnAdapterConfigError> {
    if value.trim().is_empty() || value.contains('/') || !no_space_or_control(value) {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_lb_segment(
    value: &str,
    error: OciLoadBalancerAdapterConfigError,
) -> Result<(), OciLoadBalancerAdapterConfigError> {
    if value.trim().is_empty() || value.contains('/') || !no_space_or_control(value) {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_dns_segment(
    value: &str,
    error: OciDnsZoneAdapterConfigError,
) -> Result<(), OciDnsZoneAdapterConfigError> {
    if value.trim().is_empty() || value.contains('/') || !no_space_or_control(value) {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_interconnect_segment(
    value: &str,
    error: OciDirectInterconnectAdapterConfigError,
) -> Result<(), OciDirectInterconnectAdapterConfigError> {
    if value.trim().is_empty() || value.contains('/') || !no_space_or_control(value) {
        Err(error)
    } else {
        Ok(())
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

fn lb_kind_label(kind: network_domain::LbKind) -> &'static str {
    match kind {
        network_domain::LbKind::L4Tcp => "l4_tcp",
        network_domain::LbKind::L4Udp => "l4_udp",
        network_domain::LbKind::L7Http => "l7_http",
        network_domain::LbKind::L7Grpc => "l7_grpc",
    }
}

fn dns_zone_kind_label(kind: network_domain::DnsZoneKind) -> &'static str {
    match kind {
        network_domain::DnsZoneKind::Public => "public",
        network_domain::DnsZoneKind::Private => "private",
    }
}

fn dns_zone_scope_label(kind: network_domain::DnsZoneKind) -> &'static str {
    match kind {
        network_domain::DnsZoneKind::Public => "GLOBAL",
        network_domain::DnsZoneKind::Private => "PRIVATE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use network_domain::{
        BgpSessionCreate, CloudNetworkError, DirectInterconnectCreate, DirectInterconnectState,
        DnsZoneCreate, DnsZoneKind, DnsZoneState, InterconnectPartnerCreate, IpProtocol, Ipv4Cidr,
        LbKind, LbState, ListenerCreate, LoadBalancerCreate, MtlsClientPolicy, MtlsConfigCreate,
        NetworkProviderDirectInterconnectCreateRequest, NetworkProviderDirectInterconnectError,
        NetworkProviderDirectInterconnectPort, NetworkProviderDnsZoneCreateRequest,
        NetworkProviderDnsZoneError, NetworkProviderDnsZonePort, NetworkProviderLoadBalancerPort,
        NetworkProviderVpcPort, RouteCreate, RouteDestination, RouteNextHopKind, RouteTableCreate,
        RuleDirection, SecurityGroupCreate, SecurityRule, SubnetCreate, SubnetState,
        TargetGroupCreate, VpcCreate, VpcState,
    };
    use data_boundary_kernel::DataClass;
    use network_residency::ResidencyClass;

    const COMPARTMENT_REF: &str = "ocid1.compartment.oc1..cloud";
    const REGION: &str = "ap-chuncheon-1";
    const VPC_ID: &str = "oya:cloud:alpha-region:ten_alpha:vpc:prod";

    fn adapter() -> OciVcnAdapter {
        OciVcnAdapter::new(
            "https://iaas.ap-chuncheon-1.oraclecloud.com",
            COMPARTMENT_REF,
            REGION,
        )
        .unwrap()
        .with_clock(1_700_000_000)
    }

    fn lb_adapter() -> OciLoadBalancerAdapter {
        OciLoadBalancerAdapter::new(
            "https://iaas.ap-chuncheon-1.oraclecloud.com",
            COMPARTMENT_REF,
            REGION,
        )
        .unwrap()
        .with_clock(1_700_000_000)
    }

    fn dns_adapter() -> OciDnsZoneAdapter {
        OciDnsZoneAdapter::new(
            "https://dns.ap-chuncheon-1.oraclecloud.com",
            COMPARTMENT_REF,
            REGION,
        )
        .unwrap()
        .with_clock(1_700_000_000)
    }

    fn interconnect_adapter() -> OciDirectInterconnectAdapter {
        OciDirectInterconnectAdapter::new(
            "https://iaas.ap-chuncheon-1.oraclecloud.com",
            COMPARTMENT_REF,
            REGION,
        )
        .unwrap()
        .with_clock(1_700_000_000)
    }

    fn request() -> NetworkProviderVpcCreateRequest {
        NetworkProviderVpcCreateRequest {
            request_id: "networkprov_req_vpc_create_001".to_string(),
            provider_vcn_ref: format!("oci-vcn://{COMPARTMENT_REF}/{REGION}/{VPC_ID}"),
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
            idempotency_key: "idem-network-vpc-create".to_string(),
            requested_at_epoch_seconds: 1_700_000_010,
        }
    }

    fn subnet_create() -> SubnetCreate {
        SubnetCreate {
            resource_id: "oya:cloud:alpha-region:ten_alpha:subnet:prod-a".to_string(),
            tenant_id: "ten_alpha".to_string(),
            vpc_id: VPC_ID.to_string(),
            region: "alpha-region".to_string(),
            az: "alpha-region-a".to_string(),
            cidr_v4: "10.42.1.0/24".to_string(),
            cidr_v6: "2001:db8:42:1::/64".to_string(),
            public_ip_on_launch: false,
            state: SubnetState::Creating,
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_000_010,
        }
    }

    fn lb_create() -> LoadBalancerCreate {
        LoadBalancerCreate {
            resource_id: "oya:cloud:alpha-region:ten_alpha:lb-v7:frontdoor".to_string(),
            tenant_id: "ten_alpha".to_string(),
            vpc_id: VPC_ID.to_string(),
            region: "alpha-region".to_string(),
            kind: LbKind::L7Grpc,
            listeners: vec![ListenerCreate {
                port: 443,
                target_group_id: "tg_api".to_string(),
                tls_certificate: Some("cert/alpha-region/ten_alpha/frontdoor".to_string()),
            }],
            target_groups: vec![TargetGroupCreate {
                id: "tg_api".to_string(),
                subnet_ids: vec!["oya:cloud:alpha-region:ten_alpha:subnet:prod-a".to_string()],
                health_check_path: Some("/healthz".to_string()),
            }],
            mtls: Some(MtlsConfigCreate {
                ca_bundle_ref: "cert/alpha-region/ten_alpha/mesh-ca".to_string(),
                client_policy: MtlsClientPolicy::RequireVerifiedClientCert,
            }),
            waf_policy: Some("waf_cloud_frontdoor".to_string()),
            state: LbState::Creating,
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_000_020,
        }
    }

    fn lb_request() -> NetworkProviderLoadBalancerCreateRequest {
        NetworkProviderLoadBalancerCreateRequest {
            request_id: "networkprov_req_lb_create_001".to_string(),
            provider_load_balancer_ref: format!(
                "oci-lb://{COMPARTMENT_REF}/{REGION}/oya:cloud:alpha-region:ten_alpha:lb-v7:frontdoor"
            ),
            vpc: request().vpc,
            subnets: vec![subnet_create()],
            load_balancer: lb_create(),
            actor: "sp_network".to_string(),
            idempotency_key: "idem-network-lb-create".to_string(),
            requested_at_epoch_seconds: 1_700_000_030,
        }
    }

    fn dns_zone_create() -> DnsZoneCreate {
        DnsZoneCreate {
            resource_id: "oya:cloud:alpha-region:ten_alpha:dns-zone:example-com".to_string(),
            tenant_id: "ten_alpha".to_string(),
            region: "alpha-region".to_string(),
            name: "example.com".to_string(),
            kind: DnsZoneKind::Public,
            vpc_id: None,
            dnssec_key_ref: Some("dnssec/alpha-region/ten_alpha/example-com".to_string()),
            state: DnsZoneState::Creating,
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_000_040,
        }
    }

    fn dns_request() -> NetworkProviderDnsZoneCreateRequest {
        NetworkProviderDnsZoneCreateRequest {
            request_id: "networkprov_req_dns_create_001".to_string(),
            provider_dns_zone_ref: format!(
                "oci-dns-zone://{COMPARTMENT_REF}/{REGION}/oya:cloud:alpha-region:ten_alpha:dns-zone:example-com"
            ),
            vpc: None,
            dns_zone: dns_zone_create(),
            actor: "sp_network".to_string(),
            idempotency_key: "idem-network-dns-zone-create".to_string(),
            requested_at_epoch_seconds: 1_700_000_040,
        }
    }

    fn interconnect_partner_create(id: &str, location: &str) -> InterconnectPartnerCreate {
        InterconnectPartnerCreate {
            id: id.to_string(),
            name: id.trim_start_matches("ixp_").to_ascii_uppercase(),
            region: "alpha-region".to_string(),
            peering_locations: vec![location.to_string()],
            per_link_sla_basis_points: 9_999,
        }
    }

    fn bgp_session_create(id: &str, local: &str, peer: &str) -> BgpSessionCreate {
        BgpSessionCreate {
            id: id.to_string(),
            local_asn: 64_512,
            peer_asn: 64_520,
            local_address: local.to_string(),
            peer_address: peer.to_string(),
        }
    }

    fn direct_interconnect_create() -> DirectInterconnectCreate {
        DirectInterconnectCreate {
            resource_id: "oya:cloud:alpha-region:ten_alpha:direct-interconnect:fabric-a-primary"
                .to_string(),
            tenant_id: "ten_alpha".to_string(),
            region: "alpha-region".to_string(),
            partner_id: "ixp_alpha".to_string(),
            peering_location: "alpha-region-fabric-a".to_string(),
            physical_port_id: "icp_alpha_001".to_string(),
            vlan_tag: 101,
            bandwidth_mbps: 10_000,
            redundant_port_count: 2,
            bgp_sessions: vec![
                bgp_session_create("bgp_alpha_1", "169.254.10.1", "169.254.10.2"),
                bgp_session_create("bgp_alpha_2", "169.254.10.5", "169.254.10.6"),
            ],
            advertised_prefixes: vec!["10.42.0.0/16".to_string(), "2001:db8:42::/56".to_string()],
            per_link_sla_basis_points: 9_999,
            state: DirectInterconnectState::Creating,
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_000_060,
        }
    }

    fn interconnect_request() -> NetworkProviderDirectInterconnectCreateRequest {
        NetworkProviderDirectInterconnectCreateRequest {
            request_id: "networkprov_req_interconnect_create_001".to_string(),
            provider_virtual_circuit_ref: format!(
                "oci-fast-connect://{COMPARTMENT_REF}/{REGION}/oya:cloud:alpha-region:ten_alpha:direct-interconnect:fabric-a-primary"
            ),
            interconnect_partners: vec![
                interconnect_partner_create("ixp_alpha", "alpha-region-fabric-a"),
                interconnect_partner_create("ixp_beta", "alpha-region-fabric-b"),
            ],
            direct_interconnect: direct_interconnect_create(),
            actor: "sp_network".to_string(),
            idempotency_key: "idem-network-interconnect-create".to_string(),
            requested_at_epoch_seconds: 1_700_000_060,
        }
    }

    #[test]
    fn create_vcn_command_uses_oci_path_and_reference_only_body() {
        let command = adapter()
            .create_vcn_command(&request())
            .expect("valid VPC request becomes deterministic OCI VCN command");

        assert_eq!(command.operation, "CreateVcn");
        assert_eq!(command.method, "POST");
        assert_eq!(
            command.endpoint_origin,
            "https://iaas.ap-chuncheon-1.oraclecloud.com"
        );
        assert_eq!(command.path, "/20160918/vcns");
        assert!(command.body_canonical.contains("compartment_ref=ocid1."));
        assert!(command.body_canonical.contains("cidr_v4=10.42.0.0/16"));
        assert!(command.body_canonical.contains("cidr_v6=2001:db8:42::/56"));
        assert!(command.body_canonical.contains("flow_logs_enabled=true"));
        assert!(command.body_canonical.contains("route_table_id=rtb_main"));
        assert!(!command.body_canonical.contains("private_key"));
        assert_eq!(
            command.provider_evidence_ref,
            format!("oci-vcn://{COMPARTMENT_REF}/{REGION}/{VPC_ID}/networkprov_req_vpc_create_001")
        );
    }

    #[test]
    fn vpc_port_receipts_preserve_refs_without_provider_credentials() {
        let receipt = adapter()
            .create_vpc(request())
            .expect("VPC receipt is generated");

        assert_eq!(receipt.provider, NetworkProviderKind::OciVcn);
        assert_eq!(
            receipt.provider_request_id,
            "oci-vcn-1700000000-networkprov_req_vpc_create_001"
        );
        assert_eq!(
            receipt.provider_vcn_ref,
            format!("oci-vcn://{COMPARTMENT_REF}/{REGION}/{VPC_ID}")
        );
        assert_eq!(receipt.resource_id, VPC_ID);
        assert_eq!(receipt.actor, "sp_network");
    }

    #[test]
    fn rejects_provider_vcn_drift_and_bad_vpc_shape() {
        let mut drifted = request();
        drifted.provider_vcn_ref = "oci-vcn://other/ap-chuncheon-1/vpc".to_string();
        assert!(matches!(
            adapter().create_vcn_command(&drifted),
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
    fn create_load_balancer_command_uses_oci_path_and_reference_only_body() {
        let command = lb_adapter()
            .create_load_balancer_command(&lb_request())
            .expect("valid load balancer request becomes deterministic OCI command");

        assert_eq!(command.operation, "CreateLoadBalancer");
        assert_eq!(command.method, "POST");
        assert_eq!(command.path, "/20170115/loadBalancers");
        assert!(command.body_canonical.contains("compartment_ref=ocid1."));
        assert!(
            command
                .body_canonical
                .contains("resource_id=oya:cloud:alpha-region:ten_alpha:lb-v7:frontdoor")
        );
        assert!(command.body_canonical.contains("kind=l7_grpc"));
        assert!(command.body_canonical.contains("listener_count=1"));
        assert!(command.body_canonical.contains("target_group_count=1"));
        assert!(command.body_canonical.contains("mtls_enabled=true"));
        assert!(!command.body_canonical.contains("private_key"));
        assert_eq!(
            command.provider_evidence_ref,
            format!(
                "oci-lb://{COMPARTMENT_REF}/{REGION}/oya:cloud:alpha-region:ten_alpha:lb-v7:frontdoor/networkprov_req_lb_create_001"
            )
        );
    }

    #[test]
    fn load_balancer_port_receipts_preserve_refs_without_provider_credentials() {
        let receipt = lb_adapter()
            .create_load_balancer(lb_request())
            .expect("load balancer receipt is generated");

        assert_eq!(receipt.provider, NetworkProviderKind::OciLoadBalancer);
        assert_eq!(
            receipt.provider_request_id,
            "oci-lb-1700000000-networkprov_req_lb_create_001"
        );
        assert_eq!(
            receipt.provider_load_balancer_ref,
            format!(
                "oci-lb://{COMPARTMENT_REF}/{REGION}/oya:cloud:alpha-region:ten_alpha:lb-v7:frontdoor"
            )
        );
        assert_eq!(
            receipt.resource_id,
            "oya:cloud:alpha-region:ten_alpha:lb-v7:frontdoor"
        );
        assert_eq!(receipt.actor, "sp_network");
        assert_eq!(receipt.listener_count, 1);
        assert_eq!(receipt.target_group_count, 1);
    }

    #[test]
    fn rejects_provider_load_balancer_drift_and_bad_lb_shape() {
        let mut drifted = lb_request();
        drifted.provider_load_balancer_ref = "oci-lb://other/ap-chuncheon-1/lb".to_string();
        assert!(matches!(
            lb_adapter().create_load_balancer_command(&drifted),
            Err(NetworkProviderLoadBalancerError::ProviderRejected { .. })
        ));

        let mut bad_lb = lb_request();
        bad_lb.load_balancer.mtls = None;
        assert_eq!(
            bad_lb.validate(),
            Err(NetworkProviderLoadBalancerError::InvalidRequestShape(
                CloudNetworkError::GrpcRequiresMtls,
            ))
        );
    }

    #[test]
    fn create_dns_zone_command_uses_oci_path_and_reference_only_body() {
        let command = dns_adapter()
            .create_dns_zone_command(&dns_request())
            .expect("valid DNS zone request becomes deterministic OCI command");

        assert_eq!(command.operation, "CreateZone");
        assert_eq!(command.method, "POST");
        assert_eq!(command.path, "/20180115/zones");
        assert!(command.body_canonical.contains("compartment_ref=ocid1."));
        assert!(
            command
                .body_canonical
                .contains("resource_id=oya:cloud:alpha-region:ten_alpha:dns-zone:example-com")
        );
        assert!(command.body_canonical.contains("name=example.com"));
        assert!(command.body_canonical.contains("scope=GLOBAL"));
        assert!(command.body_canonical.contains("zone_type=PRIMARY"));
        assert!(command.body_canonical.contains("dnssec_enabled=true"));
        assert!(!command.body_canonical.contains("private_key"));
        assert_eq!(
            command.provider_evidence_ref,
            format!(
                "oci-dns-zone://{COMPARTMENT_REF}/{REGION}/oya:cloud:alpha-region:ten_alpha:dns-zone:example-com/networkprov_req_dns_create_001"
            )
        );
    }

    #[test]
    fn dns_zone_port_receipts_preserve_refs_without_provider_credentials() {
        let receipt = dns_adapter()
            .create_dns_zone(dns_request())
            .expect("DNS zone receipt is generated");

        assert_eq!(receipt.provider, NetworkProviderKind::OciDnsZone);
        assert_eq!(
            receipt.provider_request_id,
            "oci-dns-zone-1700000000-networkprov_req_dns_create_001"
        );
        assert_eq!(
            receipt.provider_dns_zone_ref,
            format!(
                "oci-dns-zone://{COMPARTMENT_REF}/{REGION}/oya:cloud:alpha-region:ten_alpha:dns-zone:example-com"
            )
        );
        assert_eq!(
            receipt.resource_id,
            "oya:cloud:alpha-region:ten_alpha:dns-zone:example-com"
        );
        assert_eq!(receipt.actor, "sp_network");
        assert_eq!(receipt.name, "example.com");
        assert!(receipt.dnssec_enabled);
    }

    #[test]
    fn rejects_provider_dns_zone_drift_and_bad_zone_shape() {
        let mut drifted = dns_request();
        drifted.provider_dns_zone_ref = "oci-dns-zone://other/ap-chuncheon-1/zone".to_string();
        assert!(matches!(
            dns_adapter().create_dns_zone_command(&drifted),
            Err(NetworkProviderDnsZoneError::ProviderRejected { .. })
        ));

        let mut bad_zone = dns_request();
        bad_zone.dns_zone.dnssec_key_ref = None;
        assert_eq!(
            bad_zone.validate(),
            Err(NetworkProviderDnsZoneError::InvalidRequestShape(
                CloudNetworkError::DnssecRequired,
            ))
        );
    }

    #[test]
    fn create_direct_interconnect_command_uses_oci_path_and_reference_only_body() {
        let command = interconnect_adapter()
            .create_virtual_circuit_command(&interconnect_request())
            .expect("valid direct interconnect request becomes deterministic OCI command");

        assert_eq!(command.operation, "CreateVirtualCircuit");
        assert_eq!(command.method, "POST");
        assert_eq!(command.path, "/20160918/virtualCircuits");
        assert!(command.body_canonical.contains("compartment_ref=ocid1."));
        assert!(command.body_canonical.contains(
            "resource_id=oya:cloud:alpha-region:ten_alpha:direct-interconnect:fabric-a-primary"
        ));
        assert!(
            command
                .body_canonical
                .contains("virtual_circuit_type=PRIVATE")
        );
        assert!(command.body_canonical.contains("partner_id=ixp_alpha"));
        assert!(command.body_canonical.contains("bandwidth_mbps=10000"));
        assert!(command.body_canonical.contains("redundant_port_count=2"));
        assert!(command.body_canonical.contains("bgp_session_count=2"));
        assert!(!command.body_canonical.contains("private_key"));
        assert_eq!(
            command.provider_evidence_ref,
            format!(
                "oci-fast-connect://{COMPARTMENT_REF}/{REGION}/oya:cloud:alpha-region:ten_alpha:direct-interconnect:fabric-a-primary/networkprov_req_interconnect_create_001"
            )
        );
    }

    #[test]
    fn direct_interconnect_port_receipts_preserve_refs_without_provider_credentials() {
        let receipt = interconnect_adapter()
            .create_direct_interconnect(interconnect_request())
            .expect("direct interconnect receipt is generated");

        assert_eq!(receipt.provider, NetworkProviderKind::OciFastConnect);
        assert_eq!(
            receipt.provider_request_id,
            "oci-fast-connect-1700000000-networkprov_req_interconnect_create_001"
        );
        assert_eq!(
            receipt.provider_virtual_circuit_ref,
            format!(
                "oci-fast-connect://{COMPARTMENT_REF}/{REGION}/oya:cloud:alpha-region:ten_alpha:direct-interconnect:fabric-a-primary"
            )
        );
        assert_eq!(
            receipt.resource_id,
            "oya:cloud:alpha-region:ten_alpha:direct-interconnect:fabric-a-primary"
        );
        assert_eq!(receipt.actor, "sp_network");
        assert_eq!(receipt.bgp_session_count, 2);
        assert_eq!(receipt.advertised_prefix_count, 2);
    }

    #[test]
    fn rejects_provider_interconnect_drift_and_bad_shape() {
        let mut drifted = interconnect_request();
        drifted.provider_virtual_circuit_ref =
            "oci-fast-connect://other/ap-chuncheon-1/circuit".to_string();
        assert!(matches!(
            interconnect_adapter().create_virtual_circuit_command(&drifted),
            Err(NetworkProviderDirectInterconnectError::ProviderRejected { .. })
        ));

        let mut bad_interconnect = interconnect_request();
        bad_interconnect.direct_interconnect.redundant_port_count = 1;
        assert_eq!(
            bad_interconnect.validate(),
            Err(NetworkProviderDirectInterconnectError::InvalidRequestShape(
                CloudNetworkError::InterconnectRedundancyRequired,
            ))
        );
    }

    #[test]
    fn rejects_invalid_oci_direct_interconnect_adapter_config() {
        assert_eq!(
            OciDirectInterconnectAdapter::new("http://iaas", COMPARTMENT_REF, REGION),
            Err(OciDirectInterconnectAdapterConfigError::InvalidEndpoint)
        );
        assert_eq!(
            OciDirectInterconnectAdapter::new(
                "https://iaas.ap-chuncheon-1.oraclecloud.com",
                "bad compartment",
                REGION,
            ),
            Err(OciDirectInterconnectAdapterConfigError::InvalidCompartmentRef)
        );
        assert_eq!(
            OciDirectInterconnectAdapter::new(
                "https://iaas.ap-chuncheon-1.oraclecloud.com",
                COMPARTMENT_REF,
                "bad region",
            ),
            Err(OciDirectInterconnectAdapterConfigError::InvalidRegion)
        );
    }

    #[test]
    fn rejects_invalid_oci_load_balancer_adapter_config() {
        assert_eq!(
            OciLoadBalancerAdapter::new("http://iaas", COMPARTMENT_REF, REGION),
            Err(OciLoadBalancerAdapterConfigError::InvalidEndpoint)
        );
        assert_eq!(
            OciLoadBalancerAdapter::new(
                "https://iaas.ap-chuncheon-1.oraclecloud.com",
                "bad compartment",
                REGION,
            ),
            Err(OciLoadBalancerAdapterConfigError::InvalidCompartmentRef)
        );
        assert_eq!(
            OciLoadBalancerAdapter::new(
                "https://iaas.ap-chuncheon-1.oraclecloud.com",
                COMPARTMENT_REF,
                "bad region",
            ),
            Err(OciLoadBalancerAdapterConfigError::InvalidRegion)
        );
    }

    #[test]
    fn rejects_invalid_oci_dns_zone_adapter_config() {
        assert_eq!(
            OciDnsZoneAdapter::new("http://dns", COMPARTMENT_REF, REGION),
            Err(OciDnsZoneAdapterConfigError::InvalidEndpoint)
        );
        assert_eq!(
            OciDnsZoneAdapter::new(
                "https://dns.ap-chuncheon-1.oraclecloud.com",
                "bad compartment",
                REGION,
            ),
            Err(OciDnsZoneAdapterConfigError::InvalidCompartmentRef)
        );
        assert_eq!(
            OciDnsZoneAdapter::new(
                "https://dns.ap-chuncheon-1.oraclecloud.com",
                COMPARTMENT_REF,
                "bad region",
            ),
            Err(OciDnsZoneAdapterConfigError::InvalidRegion)
        );
    }

    #[test]
    fn rejects_invalid_oci_vcn_adapter_config() {
        assert_eq!(
            OciVcnAdapter::new("http://iaas", COMPARTMENT_REF, REGION),
            Err(OciVcnAdapterConfigError::InvalidEndpoint)
        );
        assert_eq!(
            OciVcnAdapter::new(
                "https://iaas.ap-chuncheon-1.oraclecloud.com",
                "bad compartment",
                REGION,
            ),
            Err(OciVcnAdapterConfigError::InvalidCompartmentRef)
        );
        assert_eq!(
            OciVcnAdapter::new(
                "https://iaas.ap-chuncheon-1.oraclecloud.com",
                COMPARTMENT_REF,
                "bad region",
            ),
            Err(OciVcnAdapterConfigError::InvalidRegion)
        );
    }
}
