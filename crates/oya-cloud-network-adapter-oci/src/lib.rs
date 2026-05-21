//! OCI network adapter boundary for Cloud Network.
//!
//! This crate keeps OCI compartment, region, VCN path, and evidence refs outside
//! provider-neutral Cloud Network domain/API crates while implementing the VPC
//! provider port contract. It builds deterministic request shapes only;
//! credentialed live smoke remains a separate promotion gate.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use oya_cloud_network_domain::{
    NetworkProviderKind, NetworkProviderVpcCreateRequest, NetworkProviderVpcError,
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

fn validate_endpoint(value: &str) -> Result<(), OciVcnAdapterConfigError> {
    if value.starts_with("https://") && no_space_or_control(value) {
        Ok(())
    } else {
        Err(OciVcnAdapterConfigError::InvalidEndpoint)
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
    use oya_cloud_network_domain::{
        CloudNetworkError, IpProtocol, Ipv4Cidr, NetworkProviderVpcPort, RouteCreate,
        RouteDestination, RouteNextHopKind, RouteTableCreate, RuleDirection, SecurityGroupCreate,
        SecurityRule, VpcCreate, VpcState,
    };
    use oya_data_boundary_kernel::DataClass;
    use oya_residency_domain::ResidencyClass;

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
