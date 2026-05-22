//! Cloud network aggregate kernel.
//!
//! This crate owns the VPC, subnet, load-balancer, DNS-zone, CDN, interconnect,
//! DDoS, and mesh invariants for `cloud.network.*` surfaces. It keeps the
//! AWS/GCP/Azure-shaped primitives explicit while staying adapter-free:
//! OVN/OVS/BGP/CoreDNS/Envoy implementations consume these typed contracts
//! later.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use oya_cloud_region_domain::{AzCode, CellId, RegionCode};
use oya_cloud_resource_domain::{
    CloudResourceError, LbProtocol, PrincipalId, ResourceId, ResourceKind,
};
use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use oya_residency_domain::{ResidencyClass, residency_class_allows_home_region_label};

const NETWORK_SCHEMA_VERSION: u32 = 1;
const TENANT_ID_PREFIX: &str = "ten_";
const ROUTE_TABLE_ID_PREFIX: &str = "rtb_";
const SECURITY_GROUP_ID_PREFIX: &str = "sg_";
const TARGET_GROUP_ID_PREFIX: &str = "tg_";
const WAF_POLICY_ID_PREFIX: &str = "waf_";
const CERT_REF_PREFIX: &str = "cert/";
const DNSSEC_KEY_REF_PREFIX: &str = "dnssec/";
const FLOW_ANOMALY_ID_PREFIX: &str = "flowanom_";
const INTERCONNECT_PARTNER_ID_PREFIX: &str = "ixp_";
const INTERCONNECT_PORT_ID_PREFIX: &str = "icp_";
const BGP_SESSION_ID_PREFIX: &str = "bgp_";
const MESH_ID_PREFIX: &str = "mesh_";
const RUNBOOK_REF_PREFIX: &str = "runbook/";
const ONCALL_GROUP_REF_PREFIX: &str = "oncall/";
const CEDAR_POLICY_REF_PREFIX: &str = "cedar/";
const AUDIT_STREAM_REF_PREFIX: &str = "audit/";
const HEALTH_ALARM_REF_PREFIX: &str = "alarm/";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Ipv4Cidr {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Ipv6Cidr {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RouteTableId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SecurityGroupId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TargetGroupId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WafPolicyId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CertificateRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DnsName {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DnssecKeyRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FlowAnomalyId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct InterconnectPartnerId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct InterconnectPortId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BgpSessionId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PeeringLocation {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RunbookRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OnCallGroupRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MeshId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CedarPolicyRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct AuditStreamRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct HealthAlarmRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MeshNamespace {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum VpcState {
    Creating,
    Active,
    Suspended,
    Deleting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SubnetState {
    Creating,
    Active,
    Draining,
    Deleting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RouteNextHopKind {
    Local,
    InternetGateway,
    NatGateway,
    VpcPeering,
    TransitGateway,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RouteDestination {
    Ipv4(Ipv4Cidr),
    Ipv6(Ipv6Cidr),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Route {
    pub destination: RouteDestination, // data_class: PUBLIC
    pub next_hop: RouteNextHopKind,    // data_class: PUBLIC
    pub target_ref: Option<String>,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteCreate {
    pub destination: String,        // data_class: PUBLIC
    pub next_hop: RouteNextHopKind, // data_class: PUBLIC
    pub target_ref: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteTableCreate {
    pub id: String,               // data_class: INTERNAL_ONLY
    pub routes: Vec<RouteCreate>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteTable {
    pub id: RouteTableId,   // data_class: INTERNAL_ONLY
    pub routes: Vec<Route>, // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RuleDirection {
    Ingress,
    Egress,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IpProtocol {
    Tcp,
    Udp,
    Icmp,
    Any,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityRule {
    pub direction: RuleDirection,       // data_class: PUBLIC
    pub protocol: IpProtocol,           // data_class: PUBLIC
    pub port_range: Option<(u16, u16)>, // data_class: PUBLIC
    pub cidr: RouteDestination,         // data_class: PUBLIC
    pub description: String,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityGroupCreate {
    pub id: String,               // data_class: INTERNAL_ONLY
    pub rules: Vec<SecurityRule>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityGroup {
    pub id: SecurityGroupId,      // data_class: INTERNAL_ONLY
    pub rules: Vec<SecurityRule>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VpcCreate {
    pub resource_id: String,                       // data_class: INTERNAL_ONLY
    pub tenant_id: String,                         // data_class: INTERNAL_ONLY
    pub region: String,                            // data_class: PUBLIC
    pub cidr_v4: String,                           // data_class: PUBLIC
    pub cidr_v6: String,                           // data_class: PUBLIC
    pub flow_logs_enabled: bool,                   // data_class: PUBLIC
    pub route_table: RouteTableCreate,             // data_class: INTERNAL_ONLY
    pub security_groups: Vec<SecurityGroupCreate>, // data_class: INTERNAL_ONLY
    pub residency: ResidencyClass,                 // data_class: INTERNAL_ONLY
    pub state: VpcState,                           // data_class: PUBLIC
    pub data_class: DataClass,                     // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vpc {
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub cidr_v4: Classified<Ipv4Cidr>,       // data_class: PUBLIC
    pub cidr_v6: Classified<Ipv6Cidr>,       // data_class: PUBLIC
    pub flow_logs_enabled: Classified<bool>, // data_class: PUBLIC
    pub route_table: Classified<RouteTable>, // data_class: INTERNAL_ONLY
    pub security_groups: Classified<Vec<SecurityGroup>>, // data_class: INTERNAL_ONLY
    pub residency: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub state: Classified<VpcState>,         // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubnetCreate {
    pub resource_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub vpc_id: String,                // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: PUBLIC
    pub az: String,                    // data_class: PUBLIC
    pub cidr_v4: String,               // data_class: PUBLIC
    pub cidr_v6: String,               // data_class: PUBLIC
    pub public_ip_on_launch: bool,     // data_class: PUBLIC
    pub state: SubnetState,            // data_class: PUBLIC
    pub data_class: DataClass,         // data_class: PUBLIC
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subnet {
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub vpc_id: Classified<ResourceId>,      // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub az: Classified<AzCode>,              // data_class: PUBLIC
    pub cidr_v4: Classified<Ipv4Cidr>,       // data_class: PUBLIC
    pub cidr_v6: Classified<Ipv6Cidr>,       // data_class: PUBLIC
    pub public_ip_on_launch: Classified<bool>, // data_class: PUBLIC
    pub state: Classified<SubnetState>,      // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LbKind {
    L4Tcp,
    L4Udp,
    L7Http,
    L7Grpc,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LbState {
    Creating,
    Active,
    Draining,
    Deleting,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Listener {
    pub port: u16,                               // data_class: PUBLIC
    pub target_group_id: TargetGroupId,          // data_class: INTERNAL_ONLY
    pub tls_certificate: Option<CertificateRef>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetGroup {
    pub id: TargetGroupId,                 // data_class: INTERNAL_ONLY
    pub subnet_ids: Vec<ResourceId>,       // data_class: INTERNAL_ONLY
    pub health_check_path: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MtlsClientPolicy {
    RequireVerifiedClientCert,
    ForwardVerifiedIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MtlsConfig {
    pub ca_bundle_ref: CertificateRef,   // data_class: INTERNAL_ONLY
    pub client_policy: MtlsClientPolicy, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListenerCreate {
    pub port: u16,                       // data_class: PUBLIC
    pub target_group_id: String,         // data_class: INTERNAL_ONLY
    pub tls_certificate: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetGroupCreate {
    pub id: String,                        // data_class: INTERNAL_ONLY
    pub subnet_ids: Vec<String>,           // data_class: INTERNAL_ONLY
    pub health_check_path: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MtlsConfigCreate {
    pub ca_bundle_ref: String,           // data_class: INTERNAL_ONLY
    pub client_policy: MtlsClientPolicy, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadBalancerCreate {
    pub resource_id: String,                   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub vpc_id: String,                        // data_class: INTERNAL_ONLY
    pub region: String,                        // data_class: PUBLIC
    pub kind: LbKind,                          // data_class: PUBLIC
    pub listeners: Vec<ListenerCreate>,        // data_class: INTERNAL_ONLY
    pub target_groups: Vec<TargetGroupCreate>, // data_class: INTERNAL_ONLY
    pub mtls: Option<MtlsConfigCreate>,        // data_class: INTERNAL_ONLY
    pub waf_policy: Option<String>,            // data_class: INTERNAL_ONLY
    pub state: LbState,                        // data_class: PUBLIC
    pub data_class: DataClass,                 // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadBalancer {
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub vpc_id: Classified<ResourceId>,      // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub kind: Classified<LbKind>,            // data_class: PUBLIC
    pub listeners: Classified<Vec<Listener>>, // data_class: INTERNAL_ONLY
    pub target_groups: Classified<Vec<TargetGroup>>, // data_class: INTERNAL_ONLY
    pub mtls: Classified<Option<MtlsConfig>>, // data_class: INTERNAL_ONLY
    pub waf_policy: Classified<Option<WafPolicyId>>, // data_class: INTERNAL_ONLY
    pub state: Classified<LbState>,          // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DnsZoneKind {
    Public,
    Private,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DnsZoneState {
    Creating,
    Active,
    Suspended,
    Deleting,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DnsZoneCreate {
    pub resource_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub region: String,                 // data_class: PUBLIC
    pub name: String,                   // data_class: PUBLIC
    pub kind: DnsZoneKind,              // data_class: PUBLIC
    pub vpc_id: Option<String>,         // data_class: INTERNAL_ONLY
    pub dnssec_key_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub state: DnsZoneState,            // data_class: PUBLIC
    pub data_class: DataClass,          // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DnsZone {
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub name: Classified<DnsName>,           // data_class: PUBLIC
    pub kind: Classified<DnsZoneKind>,       // data_class: PUBLIC
    pub vpc_id: Classified<Option<ResourceId>>, // data_class: INTERNAL_ONLY
    pub dnssec_key_ref: Classified<Option<DnssecKeyRef>>, // data_class: INTERNAL_ONLY
    pub state: Classified<DnsZoneState>,     // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CdnOriginKind {
    LoadBalancer,
    DnsZone,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CdnCacheMode {
    StaticAssets,
    ConsoleAssets,
    ApiEdge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CdnState {
    Creating,
    Active,
    Suspended,
    Deleting,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CdnOriginCreate {
    pub resource_id: String, // data_class: INTERNAL_ONLY
    pub kind: CdnOriginKind, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CdnOrigin {
    pub resource_id: ResourceId, // data_class: INTERNAL_ONLY
    pub kind: CdnOriginKind,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CdnDistributionCreate {
    pub resource_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: PUBLIC
    pub hostnames: Vec<String>,        // data_class: PUBLIC
    pub origins: Vec<CdnOriginCreate>, // data_class: INTERNAL_ONLY
    pub tls_certificate: String,       // data_class: INTERNAL_ONLY
    pub waf_policy: String,            // data_class: INTERNAL_ONLY
    pub cache_mode: CdnCacheMode,      // data_class: PUBLIC
    pub state: CdnState,               // data_class: PUBLIC
    pub data_class: DataClass,         // data_class: PUBLIC
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CdnDistribution {
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub hostnames: Classified<Vec<DnsName>>, // data_class: PUBLIC
    pub origins: Classified<Vec<CdnOrigin>>, // data_class: INTERNAL_ONLY
    pub tls_certificate: Classified<CertificateRef>, // data_class: INTERNAL_ONLY
    pub waf_policy: Classified<WafPolicyId>, // data_class: INTERNAL_ONLY
    pub cache_mode: Classified<CdnCacheMode>, // data_class: PUBLIC
    pub state: Classified<CdnState>,         // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DirectInterconnectState {
    Creating,
    Provisioned,
    Suspended,
    Deleting,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterconnectPartnerCreate {
    pub id: String,                     // data_class: INTERNAL_ONLY
    pub name: String,                   // data_class: PUBLIC
    pub region: String,                 // data_class: PUBLIC
    pub peering_locations: Vec<String>, // data_class: PUBLIC
    pub per_link_sla_basis_points: u16, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterconnectPartner {
    pub id: Classified<InterconnectPartnerId>, // data_class: INTERNAL_ONLY
    pub name: Classified<String>,              // data_class: PUBLIC
    pub region: Classified<RegionCode>,        // data_class: PUBLIC
    pub peering_locations: Classified<Vec<PeeringLocation>>, // data_class: PUBLIC
    pub per_link_sla_basis_points: Classified<u16>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BgpSessionCreate {
    pub id: String,            // data_class: INTERNAL_ONLY
    pub local_asn: u32,        // data_class: INTERNAL_ONLY
    pub peer_asn: u32,         // data_class: INTERNAL_ONLY
    pub local_address: String, // data_class: INTERNAL_ONLY
    pub peer_address: String,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BgpSession {
    pub id: BgpSessionId,      // data_class: INTERNAL_ONLY
    pub local_asn: u32,        // data_class: INTERNAL_ONLY
    pub peer_asn: u32,         // data_class: INTERNAL_ONLY
    pub local_address: IpAddr, // data_class: INTERNAL_ONLY
    pub peer_address: IpAddr,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectInterconnectCreate {
    pub resource_id: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub region: String,                      // data_class: PUBLIC
    pub partner_id: String,                  // data_class: INTERNAL_ONLY
    pub peering_location: String,            // data_class: PUBLIC
    pub physical_port_id: String,            // data_class: INTERNAL_ONLY
    pub vlan_tag: u16,                       // data_class: INTERNAL_ONLY
    pub bandwidth_mbps: u32,                 // data_class: PUBLIC
    pub redundant_port_count: u8,            // data_class: PUBLIC
    pub bgp_sessions: Vec<BgpSessionCreate>, // data_class: INTERNAL_ONLY
    pub advertised_prefixes: Vec<String>,    // data_class: INTERNAL_ONLY
    pub per_link_sla_basis_points: u16,      // data_class: PUBLIC
    pub state: DirectInterconnectState,      // data_class: PUBLIC
    pub data_class: DataClass,               // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectInterconnect {
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub partner_id: Classified<InterconnectPartnerId>, // data_class: INTERNAL_ONLY
    pub peering_location: Classified<PeeringLocation>, // data_class: PUBLIC
    pub physical_port_id: Classified<InterconnectPortId>, // data_class: INTERNAL_ONLY
    pub vlan_tag: Classified<u16>,           // data_class: INTERNAL_ONLY
    pub bandwidth_mbps: Classified<u32>,     // data_class: PUBLIC
    pub redundant_port_count: Classified<u8>, // data_class: PUBLIC
    pub bgp_sessions: Classified<Vec<BgpSession>>, // data_class: INTERNAL_ONLY
    pub advertised_prefixes: Classified<Vec<RouteDestination>>, // data_class: INTERNAL_ONLY
    pub per_link_sla_basis_points: Classified<u16>, // data_class: PUBLIC
    pub state: Classified<DirectInterconnectState>, // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DdosProtectionState {
    Creating,
    Active,
    Suspended,
    Deleting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ProtectedResourceKind {
    Vpc,
    LoadBalancer,
    DnsZone,
    CdnDistribution,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedResourceRef {
    pub resource_id: ResourceId,     // data_class: INTERNAL_ONLY
    pub kind: ProtectedResourceKind, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DdosProtectionCreate {
    pub resource_id: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub region: String,                      // data_class: PUBLIC
    pub protected_resource_ids: Vec<String>, // data_class: INTERNAL_ONLY
    pub scrubbing_regions: Vec<String>,      // data_class: PUBLIC
    pub line_rate_scrubbing: bool,           // data_class: PUBLIC
    pub always_on: bool,                     // data_class: PUBLIC
    pub mitigation_runbook_ref: String,      // data_class: INTERNAL_ONLY
    pub oncall_group_ref: String,            // data_class: INTERNAL_ONLY
    pub state: DdosProtectionState,          // data_class: PUBLIC
    pub data_class: DataClass,               // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DdosProtection {
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub protected_resources: Classified<Vec<ProtectedResourceRef>>, // data_class: INTERNAL_ONLY
    pub scrubbing_regions: Classified<Vec<RegionCode>>, // data_class: PUBLIC
    pub line_rate_scrubbing: Classified<bool>, // data_class: PUBLIC
    pub always_on: Classified<bool>,         // data_class: PUBLIC
    pub mitigation_runbook_ref: Classified<RunbookRef>, // data_class: INTERNAL_ONLY
    pub oncall_group_ref: Classified<OnCallGroupRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<DdosProtectionState>, // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ServiceMeshMode {
    IstioAmbient,
    Sidecar,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MeshGatewayKind {
    Envoy,
    Nginx,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ServiceMeshState {
    Creating,
    Active,
    Degraded,
    Deleting,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceMeshCellCreate {
    pub mesh_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: PUBLIC
    pub cell_id: String,               // data_class: PUBLIC
    pub vpc_id: String,                // data_class: INTERNAL_ONLY
    pub namespace: String,             // data_class: INTERNAL_ONLY
    pub mode: ServiceMeshMode,         // data_class: PUBLIC
    pub edge_gateway: MeshGatewayKind, // data_class: PUBLIC
    pub mtls_everywhere: bool,         // data_class: PUBLIC
    pub ext_authz_enabled: bool,       // data_class: PUBLIC
    pub cross_cell_policy_ref: String, // data_class: INTERNAL_ONLY
    pub audit_stream_ref: String,      // data_class: INTERNAL_ONLY
    pub health_alarm_ref: String,      // data_class: INTERNAL_ONLY
    pub control_plane_replicas: u8,    // data_class: PUBLIC
    pub quarterly_upgrade_drill: bool, // data_class: PUBLIC
    pub state: ServiceMeshState,       // data_class: PUBLIC
    pub data_class: DataClass,         // data_class: PUBLIC
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceMeshCell {
    pub mesh_id: Classified<MeshId>,          // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,        // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,       // data_class: PUBLIC
    pub cell_id: Classified<CellId>,          // data_class: PUBLIC
    pub vpc_id: Classified<ResourceId>,       // data_class: INTERNAL_ONLY
    pub namespace: Classified<MeshNamespace>, // data_class: INTERNAL_ONLY
    pub mode: Classified<ServiceMeshMode>,    // data_class: PUBLIC
    pub edge_gateway: Classified<MeshGatewayKind>, // data_class: PUBLIC
    pub mtls_everywhere: Classified<bool>,    // data_class: PUBLIC
    pub ext_authz_enabled: Classified<bool>,  // data_class: PUBLIC
    pub cross_cell_policy_ref: Classified<CedarPolicyRef>, // data_class: INTERNAL_ONLY
    pub audit_stream_ref: Classified<AuditStreamRef>, // data_class: INTERNAL_ONLY
    pub health_alarm_ref: Classified<HealthAlarmRef>, // data_class: INTERNAL_ONLY
    pub control_plane_replicas: Classified<u8>, // data_class: PUBLIC
    pub quarterly_upgrade_drill: Classified<bool>, // data_class: PUBLIC
    pub state: Classified<ServiceMeshState>,  // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,      // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FlowAnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowAnomalyEvent {
    pub id: Classified<FlowAnomalyId>,  // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub vpc_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub severity: Classified<FlowAnomalySeverity>, // data_class: PUBLIC
    pub flow_pattern: Classified<String>, // data_class: INTERNAL_ONLY
    pub detected_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum NetworkProviderKind {
    OciVcn,
    OciLoadBalancer,
    OciDnsZone,
    OciFastConnect,
    SelfHostedColoVpc,
    SelfHostedColoDnsZone,
}

impl NetworkProviderKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::OciVcn => "oci_vcn",
            Self::OciLoadBalancer => "oci_load_balancer",
            Self::OciDnsZone => "oci_dns_zone",
            Self::OciFastConnect => "oci_fast_connect",
            Self::SelfHostedColoVpc => "selfhosted_colo_vpc",
            Self::SelfHostedColoDnsZone => "selfhosted_colo_dns_zone",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum NetworkProviderVpcOperation {
    CreateVpc,
}

impl NetworkProviderVpcOperation {
    pub const fn label(self) -> &'static str {
        match self {
            Self::CreateVpc => "create_vpc",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum NetworkProviderLoadBalancerOperation {
    CreateLoadBalancer,
}

impl NetworkProviderLoadBalancerOperation {
    pub const fn label(self) -> &'static str {
        match self {
            Self::CreateLoadBalancer => "create_load_balancer",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum NetworkProviderDnsZoneOperation {
    CreateDnsZone,
}

impl NetworkProviderDnsZoneOperation {
    pub const fn label(self) -> &'static str {
        match self {
            Self::CreateDnsZone => "create_dns_zone",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum NetworkProviderDirectInterconnectOperation {
    CreateDirectInterconnect,
}

impl NetworkProviderDirectInterconnectOperation {
    pub const fn label(self) -> &'static str {
        match self {
            Self::CreateDirectInterconnect => "create_direct_interconnect",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkProviderVpcCreateRequest {
    pub request_id: String,              // data_class: INTERNAL_ONLY
    pub provider_vcn_ref: String,        // data_class: INTERNAL_ONLY
    pub vpc: VpcCreate,                  // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub idempotency_key: String,         // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkProviderVpcReceipt {
    pub provider: NetworkProviderKind,          // data_class: PUBLIC
    pub operation: NetworkProviderVpcOperation, // data_class: PUBLIC
    pub request_id: String,                     // data_class: INTERNAL_ONLY
    pub provider_request_id: String,            // data_class: INTERNAL_ONLY
    pub provider_vcn_ref: String,               // data_class: INTERNAL_ONLY
    pub resource_id: String,                    // data_class: INTERNAL_ONLY
    pub tenant_id: String,                      // data_class: INTERNAL_ONLY
    pub region: String,                         // data_class: PUBLIC
    pub cidr_v4: String,                        // data_class: PUBLIC
    pub cidr_v6: String,                        // data_class: PUBLIC
    pub flow_logs_enabled: bool,                // data_class: PUBLIC
    pub actor: String,                          // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String,          // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
    pub schema_version: u32,                    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NetworkProviderVpcError {
    InvalidProviderVcnRef,
    InvalidProviderRequestId,
    InvalidProviderEvidenceRef,
    InvalidIdempotencyKey,
    InvalidActorRef,
    InvalidRequestShape(CloudNetworkError),
    ProviderRejected {
        provider: NetworkProviderKind, // data_class: PUBLIC
        reason: String,                // data_class: INTERNAL_ONLY
    },
    ProviderUnavailable {
        provider: NetworkProviderKind, // data_class: PUBLIC
        reason: String,                // data_class: INTERNAL_ONLY
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkProviderLoadBalancerCreateRequest {
    pub request_id: String,                 // data_class: INTERNAL_ONLY
    pub provider_load_balancer_ref: String, // data_class: INTERNAL_ONLY
    pub vpc: VpcCreate,                     // data_class: INTERNAL_ONLY
    pub subnets: Vec<SubnetCreate>,         // data_class: INTERNAL_ONLY
    pub load_balancer: LoadBalancerCreate,  // data_class: INTERNAL_ONLY
    pub actor: String,                      // data_class: INTERNAL_ONLY
    pub idempotency_key: String,            // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkProviderLoadBalancerReceipt {
    pub provider: NetworkProviderKind, // data_class: PUBLIC
    pub operation: NetworkProviderLoadBalancerOperation, // data_class: PUBLIC
    pub request_id: String,            // data_class: INTERNAL_ONLY
    pub provider_request_id: String,   // data_class: INTERNAL_ONLY
    pub provider_load_balancer_ref: String, // data_class: INTERNAL_ONLY
    pub resource_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub vpc_id: String,                // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: PUBLIC
    pub kind: LbKind,                  // data_class: PUBLIC
    pub listener_count: usize,         // data_class: PUBLIC
    pub target_group_count: usize,     // data_class: PUBLIC
    pub mtls_enabled: bool,            // data_class: PUBLIC
    pub actor: String,                 // data_class: INTERNAL_ONLY
    pub idempotency_key: String,       // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NetworkProviderLoadBalancerError {
    InvalidProviderLoadBalancerRef,
    InvalidProviderRequestId,
    InvalidProviderEvidenceRef,
    InvalidIdempotencyKey,
    InvalidActorRef,
    InvalidRequestShape(CloudNetworkError),
    ProviderRejected {
        provider: NetworkProviderKind, // data_class: PUBLIC
        reason: String,                // data_class: INTERNAL_ONLY
    },
    ProviderUnavailable {
        provider: NetworkProviderKind, // data_class: PUBLIC
        reason: String,                // data_class: INTERNAL_ONLY
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkProviderDnsZoneCreateRequest {
    pub request_id: String,              // data_class: INTERNAL_ONLY
    pub provider_dns_zone_ref: String,   // data_class: INTERNAL_ONLY
    pub vpc: Option<VpcCreate>,          // data_class: INTERNAL_ONLY
    pub dns_zone: DnsZoneCreate,         // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub idempotency_key: String,         // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkProviderDnsZoneReceipt {
    pub provider: NetworkProviderKind, // data_class: PUBLIC
    pub operation: NetworkProviderDnsZoneOperation, // data_class: PUBLIC
    pub request_id: String,            // data_class: INTERNAL_ONLY
    pub provider_request_id: String,   // data_class: INTERNAL_ONLY
    pub provider_dns_zone_ref: String, // data_class: INTERNAL_ONLY
    pub resource_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: PUBLIC
    pub name: String,                  // data_class: PUBLIC
    pub kind: DnsZoneKind,             // data_class: PUBLIC
    pub vpc_id: Option<String>,        // data_class: INTERNAL_ONLY
    pub dnssec_enabled: bool,          // data_class: PUBLIC
    pub actor: String,                 // data_class: INTERNAL_ONLY
    pub idempotency_key: String,       // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NetworkProviderDnsZoneError {
    InvalidProviderDnsZoneRef,
    InvalidProviderRequestId,
    InvalidProviderEvidenceRef,
    InvalidIdempotencyKey,
    InvalidActorRef,
    InvalidRequestShape(CloudNetworkError),
    ProviderRejected {
        provider: NetworkProviderKind, // data_class: PUBLIC
        reason: String,                // data_class: INTERNAL_ONLY
    },
    ProviderUnavailable {
        provider: NetworkProviderKind, // data_class: PUBLIC
        reason: String,                // data_class: INTERNAL_ONLY
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkProviderDirectInterconnectCreateRequest {
    pub request_id: String,                   // data_class: INTERNAL_ONLY
    pub provider_virtual_circuit_ref: String, // data_class: INTERNAL_ONLY
    pub interconnect_partners: Vec<InterconnectPartnerCreate>, // data_class: INTERNAL_ONLY
    pub direct_interconnect: DirectInterconnectCreate, // data_class: INTERNAL_ONLY
    pub actor: String,                        // data_class: INTERNAL_ONLY
    pub idempotency_key: String,              // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkProviderDirectInterconnectReceipt {
    pub provider: NetworkProviderKind, // data_class: PUBLIC
    pub operation: NetworkProviderDirectInterconnectOperation, // data_class: PUBLIC
    pub request_id: String,            // data_class: INTERNAL_ONLY
    pub provider_request_id: String,   // data_class: INTERNAL_ONLY
    pub provider_virtual_circuit_ref: String, // data_class: INTERNAL_ONLY
    pub resource_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: PUBLIC
    pub partner_id: String,            // data_class: INTERNAL_ONLY
    pub peering_location: String,      // data_class: PUBLIC
    pub physical_port_id: String,      // data_class: INTERNAL_ONLY
    pub vlan_tag: u16,                 // data_class: INTERNAL_ONLY
    pub bandwidth_mbps: u32,           // data_class: PUBLIC
    pub redundant_port_count: u8,      // data_class: PUBLIC
    pub bgp_session_count: usize,      // data_class: PUBLIC
    pub advertised_prefix_count: usize, // data_class: PUBLIC
    pub actor: String,                 // data_class: INTERNAL_ONLY
    pub idempotency_key: String,       // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NetworkProviderDirectInterconnectError {
    InvalidProviderVirtualCircuitRef,
    InvalidProviderRequestId,
    InvalidProviderEvidenceRef,
    InvalidIdempotencyKey,
    InvalidActorRef,
    InvalidRequestShape(CloudNetworkError),
    ProviderRejected {
        provider: NetworkProviderKind, // data_class: PUBLIC
        reason: String,                // data_class: INTERNAL_ONLY
    },
    ProviderUnavailable {
        provider: NetworkProviderKind, // data_class: PUBLIC
        reason: String,                // data_class: INTERNAL_ONLY
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudNetworkError {
    InvalidTenantId,
    InvalidResourceId,
    ResourceTenantMismatch,
    ResourceRegionMismatch,
    ResourceKindMismatch,
    InvalidDataClass,
    InvalidVpcState,
    InvalidSubnetState,
    InvalidLbState,
    InvalidDnsZoneState,
    InvalidIpv4Cidr,
    InvalidIpv6Cidr,
    Ipv6Required,
    InvalidRouteTableId,
    InvalidRoute,
    DuplicateRoute,
    InvalidSecurityGroupId,
    InvalidSecurityRule,
    DuplicateSecurityGroup,
    InvalidAzCode,
    AzRegionMismatch,
    SubnetOutsideVpc,
    OverlappingSubnet,
    DuplicateSubnet,
    UnknownVpc,
    UnknownSubnet,
    InvalidTargetGroupId,
    DuplicateTargetGroup,
    InvalidListener,
    DuplicateListenerPort,
    ListenerTargetGroupMissing,
    L7RequiresTls,
    GrpcRequiresMtls,
    InvalidCertificateRef,
    InvalidWafPolicyId,
    InvalidDnsName,
    InvalidDnssecKeyRef,
    DnssecRequired,
    PrivateZoneRequiresVpc,
    PublicZoneMustNotBindVpc,
    InvalidCdnState,
    InvalidCdnOrigin,
    DuplicateCdnOrigin,
    UnknownLoadBalancer,
    UnknownDnsZone,
    CdnWafRequired,
    CdnTlsRequired,
    DuplicateCdnHostname,
    InvalidInterconnectPartnerId,
    InvalidInterconnectPortId,
    InvalidPeeringLocation,
    InvalidInterconnectState,
    InvalidBandwidth,
    InvalidVlanTag,
    InvalidBgpSessionId,
    InvalidBgpSession,
    DuplicateBgpSession,
    InvalidAsn,
    InterconnectRedundancyRequired,
    InterconnectSlaRequired,
    RegionalInterconnectDiversityRequired,
    UnknownInterconnectPartner,
    DuplicateInterconnectPartner,
    DuplicateDirectInterconnect,
    InvalidDdosState,
    UnknownProtectedResource,
    DuplicateProtectedResource,
    ScrubbingRegionRequired,
    LineRateScrubbingRequired,
    DdosAlwaysOnRequired,
    InvalidRunbookRef,
    InvalidOnCallGroupRef,
    DuplicateDdosProtection,
    InvalidMeshId,
    InvalidCellId,
    InvalidMeshNamespace,
    InvalidMeshState,
    InvalidMeshMode,
    InvalidMeshGateway,
    MeshMtlsRequired,
    MeshExtAuthzRequired,
    InvalidCedarPolicyRef,
    InvalidAuditStreamRef,
    InvalidHealthAlarmRef,
    MeshControlPlaneReplicasRequired,
    MeshUpgradeDrillRequired,
    DuplicateServiceMesh,
    InvalidFlowAnomalyId,
    FlowLogsRequired,
    DuplicateFlowAnomaly,
    DuplicateVpc,
    DuplicateLoadBalancer,
    DuplicateDnsZone,
    DuplicateCdnDistribution,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudNetworkCatalog {
    vpcs: BTreeMap<ResourceId, Vpc>,
    subnets: BTreeMap<ResourceId, Subnet>,
    load_balancers: BTreeMap<ResourceId, LoadBalancer>,
    dns_zones: BTreeMap<ResourceId, DnsZone>,
    cdn_distributions: BTreeMap<ResourceId, CdnDistribution>,
    interconnect_partners: BTreeMap<InterconnectPartnerId, InterconnectPartner>,
    direct_interconnects: BTreeMap<ResourceId, DirectInterconnect>,
    ddos_protections: BTreeMap<ResourceId, DdosProtection>,
    service_meshes: BTreeMap<MeshId, ServiceMeshCell>,
    anomalies: BTreeMap<FlowAnomalyId, FlowAnomalyEvent>,
}

pub trait NetworkProviderVpcPort {
    fn provider_kind(&self) -> NetworkProviderKind;

    fn create_vpc(
        &self,
        input: NetworkProviderVpcCreateRequest,
    ) -> Result<NetworkProviderVpcReceipt, NetworkProviderVpcError>;
}

pub trait NetworkProviderLoadBalancerPort {
    fn provider_kind(&self) -> NetworkProviderKind;

    fn create_load_balancer(
        &self,
        input: NetworkProviderLoadBalancerCreateRequest,
    ) -> Result<NetworkProviderLoadBalancerReceipt, NetworkProviderLoadBalancerError>;
}

pub trait NetworkProviderDnsZonePort {
    fn provider_kind(&self) -> NetworkProviderKind;

    fn create_dns_zone(
        &self,
        input: NetworkProviderDnsZoneCreateRequest,
    ) -> Result<NetworkProviderDnsZoneReceipt, NetworkProviderDnsZoneError>;
}

pub trait NetworkProviderDirectInterconnectPort {
    fn provider_kind(&self) -> NetworkProviderKind;

    fn create_direct_interconnect(
        &self,
        input: NetworkProviderDirectInterconnectCreateRequest,
    ) -> Result<NetworkProviderDirectInterconnectReceipt, NetworkProviderDirectInterconnectError>;
}

pub trait NetworkRepo {
    fn create_vpc(&mut self, input: VpcCreate) -> Result<Vpc, CloudNetworkError>;
    fn add_subnet(&mut self, input: SubnetCreate) -> Result<Subnet, CloudNetworkError>;
    fn create_load_balancer(
        &mut self,
        input: LoadBalancerCreate,
    ) -> Result<LoadBalancer, CloudNetworkError>;
    fn create_dns_zone(&mut self, input: DnsZoneCreate) -> Result<DnsZone, CloudNetworkError>;
    fn create_cdn_distribution(
        &mut self,
        input: CdnDistributionCreate,
    ) -> Result<CdnDistribution, CloudNetworkError>;
    fn add_interconnect_partner(
        &mut self,
        input: InterconnectPartnerCreate,
    ) -> Result<InterconnectPartner, CloudNetworkError>;
    fn create_direct_interconnect(
        &mut self,
        input: DirectInterconnectCreate,
    ) -> Result<DirectInterconnect, CloudNetworkError>;
    fn create_ddos_protection(
        &mut self,
        input: DdosProtectionCreate,
    ) -> Result<DdosProtection, CloudNetworkError>;
    fn create_service_mesh_cell(
        &mut self,
        input: ServiceMeshCellCreate,
    ) -> Result<ServiceMeshCell, CloudNetworkError>;
    fn record_flow_anomaly(
        &mut self,
        id: String,
        vpc_id: String,
        severity: FlowAnomalySeverity,
        flow_pattern: String,
        detected_at_epoch_seconds: u64,
    ) -> Result<FlowAnomalyEvent, CloudNetworkError>;
}

impl NetworkProviderVpcCreateRequest {
    pub fn validate(&self) -> Result<(), NetworkProviderVpcError> {
        validate_network_provider_ref(
            &self.request_id,
            NetworkProviderVpcError::InvalidProviderRequestId,
        )?;
        validate_network_provider_ref(
            &self.provider_vcn_ref,
            NetworkProviderVpcError::InvalidProviderVcnRef,
        )?;
        validate_network_provider_ref(
            &self.idempotency_key,
            NetworkProviderVpcError::InvalidIdempotencyKey,
        )?;
        Vpc::new(self.vpc.clone()).map_err(NetworkProviderVpcError::InvalidRequestShape)?;
        PrincipalId::new(self.actor.clone())
            .map_err(|_| NetworkProviderVpcError::InvalidActorRef)?;
        Ok(())
    }
}

impl NetworkProviderVpcReceipt {
    pub fn create_vpc(
        provider: NetworkProviderKind,
        input: NetworkProviderVpcCreateRequest,
        provider_request_id: impl Into<String>,
        provider_evidence_ref: impl Into<String>,
    ) -> Result<Self, NetworkProviderVpcError> {
        input.validate()?;
        let provider_request_id = provider_request_id.into();
        let provider_evidence_ref = provider_evidence_ref.into();
        validate_network_provider_ref(
            &provider_request_id,
            NetworkProviderVpcError::InvalidProviderRequestId,
        )?;
        validate_network_provider_ref(
            &provider_evidence_ref,
            NetworkProviderVpcError::InvalidProviderEvidenceRef,
        )?;
        Ok(Self {
            provider,
            operation: NetworkProviderVpcOperation::CreateVpc,
            request_id: input.request_id,
            provider_request_id,
            provider_vcn_ref: input.provider_vcn_ref,
            resource_id: input.vpc.resource_id,
            tenant_id: input.vpc.tenant_id,
            region: input.vpc.region,
            cidr_v4: input.vpc.cidr_v4,
            cidr_v6: input.vpc.cidr_v6,
            flow_logs_enabled: input.vpc.flow_logs_enabled,
            actor: input.actor,
            idempotency_key: input.idempotency_key,
            provider_evidence_ref,
            occurred_at_epoch_seconds: input.requested_at_epoch_seconds,
            schema_version: NETWORK_SCHEMA_VERSION,
        })
    }
}

impl NetworkProviderLoadBalancerCreateRequest {
    pub fn validate(&self) -> Result<(), NetworkProviderLoadBalancerError> {
        validate_network_provider_load_balancer_ref(
            &self.request_id,
            NetworkProviderLoadBalancerError::InvalidProviderRequestId,
        )?;
        validate_network_provider_load_balancer_ref(
            &self.provider_load_balancer_ref,
            NetworkProviderLoadBalancerError::InvalidProviderLoadBalancerRef,
        )?;
        validate_network_provider_load_balancer_ref(
            &self.idempotency_key,
            NetworkProviderLoadBalancerError::InvalidIdempotencyKey,
        )?;
        let vpc = Vpc::new(self.vpc.clone())
            .map_err(NetworkProviderLoadBalancerError::InvalidRequestShape)?;
        let mut known_subnets = BTreeMap::new();
        for subnet_input in self.subnets.clone() {
            let subnet = Subnet::new(&vpc, subnet_input)
                .map_err(NetworkProviderLoadBalancerError::InvalidRequestShape)?;
            if known_subnets
                .insert(subnet.resource_id.value.clone(), subnet)
                .is_some()
            {
                return Err(NetworkProviderLoadBalancerError::InvalidRequestShape(
                    CloudNetworkError::DuplicateSubnet,
                ));
            }
        }
        LoadBalancer::new(&vpc, &known_subnets, self.load_balancer.clone())
            .map_err(NetworkProviderLoadBalancerError::InvalidRequestShape)?;
        PrincipalId::new(self.actor.clone())
            .map_err(|_| NetworkProviderLoadBalancerError::InvalidActorRef)?;
        Ok(())
    }
}

impl NetworkProviderLoadBalancerReceipt {
    pub fn create_load_balancer(
        provider: NetworkProviderKind,
        input: NetworkProviderLoadBalancerCreateRequest,
        provider_request_id: impl Into<String>,
        provider_evidence_ref: impl Into<String>,
    ) -> Result<Self, NetworkProviderLoadBalancerError> {
        input.validate()?;
        let provider_request_id = provider_request_id.into();
        let provider_evidence_ref = provider_evidence_ref.into();
        validate_network_provider_load_balancer_ref(
            &provider_request_id,
            NetworkProviderLoadBalancerError::InvalidProviderRequestId,
        )?;
        validate_network_provider_load_balancer_ref(
            &provider_evidence_ref,
            NetworkProviderLoadBalancerError::InvalidProviderEvidenceRef,
        )?;
        Ok(Self {
            provider,
            operation: NetworkProviderLoadBalancerOperation::CreateLoadBalancer,
            request_id: input.request_id,
            provider_request_id,
            provider_load_balancer_ref: input.provider_load_balancer_ref,
            resource_id: input.load_balancer.resource_id,
            tenant_id: input.load_balancer.tenant_id,
            vpc_id: input.load_balancer.vpc_id,
            region: input.load_balancer.region,
            kind: input.load_balancer.kind,
            listener_count: input.load_balancer.listeners.len(),
            target_group_count: input.load_balancer.target_groups.len(),
            mtls_enabled: input.load_balancer.mtls.is_some(),
            actor: input.actor,
            idempotency_key: input.idempotency_key,
            provider_evidence_ref,
            occurred_at_epoch_seconds: input.requested_at_epoch_seconds,
            schema_version: NETWORK_SCHEMA_VERSION,
        })
    }
}

impl NetworkProviderDnsZoneCreateRequest {
    pub fn validate(&self) -> Result<(), NetworkProviderDnsZoneError> {
        validate_network_provider_dns_zone_ref(
            &self.request_id,
            NetworkProviderDnsZoneError::InvalidProviderRequestId,
        )?;
        validate_network_provider_dns_zone_ref(
            &self.provider_dns_zone_ref,
            NetworkProviderDnsZoneError::InvalidProviderDnsZoneRef,
        )?;
        validate_network_provider_dns_zone_ref(
            &self.idempotency_key,
            NetworkProviderDnsZoneError::InvalidIdempotencyKey,
        )?;
        let vpc = self
            .vpc
            .clone()
            .map(Vpc::new)
            .transpose()
            .map_err(NetworkProviderDnsZoneError::InvalidRequestShape)?;
        DnsZone::new(vpc.as_ref(), self.dns_zone.clone())
            .map_err(NetworkProviderDnsZoneError::InvalidRequestShape)?;
        PrincipalId::new(self.actor.clone())
            .map_err(|_| NetworkProviderDnsZoneError::InvalidActorRef)?;
        Ok(())
    }
}

impl NetworkProviderDnsZoneReceipt {
    pub fn create_dns_zone(
        provider: NetworkProviderKind,
        input: NetworkProviderDnsZoneCreateRequest,
        provider_request_id: impl Into<String>,
        provider_evidence_ref: impl Into<String>,
    ) -> Result<Self, NetworkProviderDnsZoneError> {
        input.validate()?;
        let provider_request_id = provider_request_id.into();
        let provider_evidence_ref = provider_evidence_ref.into();
        validate_network_provider_dns_zone_ref(
            &provider_request_id,
            NetworkProviderDnsZoneError::InvalidProviderRequestId,
        )?;
        validate_network_provider_dns_zone_ref(
            &provider_evidence_ref,
            NetworkProviderDnsZoneError::InvalidProviderEvidenceRef,
        )?;
        Ok(Self {
            provider,
            operation: NetworkProviderDnsZoneOperation::CreateDnsZone,
            request_id: input.request_id,
            provider_request_id,
            provider_dns_zone_ref: input.provider_dns_zone_ref,
            resource_id: input.dns_zone.resource_id,
            tenant_id: input.dns_zone.tenant_id,
            region: input.dns_zone.region,
            name: input.dns_zone.name,
            kind: input.dns_zone.kind,
            vpc_id: input.dns_zone.vpc_id,
            dnssec_enabled: input.dns_zone.dnssec_key_ref.is_some(),
            actor: input.actor,
            idempotency_key: input.idempotency_key,
            provider_evidence_ref,
            occurred_at_epoch_seconds: input.requested_at_epoch_seconds,
            schema_version: NETWORK_SCHEMA_VERSION,
        })
    }
}

impl NetworkProviderDirectInterconnectCreateRequest {
    pub fn validate(&self) -> Result<(), NetworkProviderDirectInterconnectError> {
        validate_network_provider_direct_interconnect_ref(
            &self.request_id,
            NetworkProviderDirectInterconnectError::InvalidProviderRequestId,
        )?;
        validate_network_provider_direct_interconnect_ref(
            &self.provider_virtual_circuit_ref,
            NetworkProviderDirectInterconnectError::InvalidProviderVirtualCircuitRef,
        )?;
        validate_network_provider_direct_interconnect_ref(
            &self.idempotency_key,
            NetworkProviderDirectInterconnectError::InvalidIdempotencyKey,
        )?;
        let mut known_partners = BTreeMap::new();
        for partner_input in self.interconnect_partners.clone() {
            let partner = InterconnectPartner::new(partner_input)
                .map_err(NetworkProviderDirectInterconnectError::InvalidRequestShape)?;
            if known_partners
                .insert(partner.id.value.clone(), partner)
                .is_some()
            {
                return Err(NetworkProviderDirectInterconnectError::InvalidRequestShape(
                    CloudNetworkError::DuplicateInterconnectPartner,
                ));
            }
        }
        DirectInterconnect::new(&known_partners, self.direct_interconnect.clone())
            .map_err(NetworkProviderDirectInterconnectError::InvalidRequestShape)?;
        PrincipalId::new(self.actor.clone())
            .map_err(|_| NetworkProviderDirectInterconnectError::InvalidActorRef)?;
        Ok(())
    }
}

impl NetworkProviderDirectInterconnectReceipt {
    pub fn create_direct_interconnect(
        provider: NetworkProviderKind,
        input: NetworkProviderDirectInterconnectCreateRequest,
        provider_request_id: impl Into<String>,
        provider_evidence_ref: impl Into<String>,
    ) -> Result<Self, NetworkProviderDirectInterconnectError> {
        input.validate()?;
        let provider_request_id = provider_request_id.into();
        let provider_evidence_ref = provider_evidence_ref.into();
        validate_network_provider_direct_interconnect_ref(
            &provider_request_id,
            NetworkProviderDirectInterconnectError::InvalidProviderRequestId,
        )?;
        validate_network_provider_direct_interconnect_ref(
            &provider_evidence_ref,
            NetworkProviderDirectInterconnectError::InvalidProviderEvidenceRef,
        )?;
        Ok(Self {
            provider,
            operation: NetworkProviderDirectInterconnectOperation::CreateDirectInterconnect,
            request_id: input.request_id,
            provider_request_id,
            provider_virtual_circuit_ref: input.provider_virtual_circuit_ref,
            resource_id: input.direct_interconnect.resource_id,
            tenant_id: input.direct_interconnect.tenant_id,
            region: input.direct_interconnect.region,
            partner_id: input.direct_interconnect.partner_id,
            peering_location: input.direct_interconnect.peering_location,
            physical_port_id: input.direct_interconnect.physical_port_id,
            vlan_tag: input.direct_interconnect.vlan_tag,
            bandwidth_mbps: input.direct_interconnect.bandwidth_mbps,
            redundant_port_count: input.direct_interconnect.redundant_port_count,
            bgp_session_count: input.direct_interconnect.bgp_sessions.len(),
            advertised_prefix_count: input.direct_interconnect.advertised_prefixes.len(),
            actor: input.actor,
            idempotency_key: input.idempotency_key,
            provider_evidence_ref,
            occurred_at_epoch_seconds: input.requested_at_epoch_seconds,
            schema_version: NETWORK_SCHEMA_VERSION,
        })
    }
}

impl LbKind {
    pub const fn resource_protocol(self) -> LbProtocol {
        match self {
            Self::L4Tcp | Self::L4Udp => LbProtocol::L4,
            Self::L7Http | Self::L7Grpc => LbProtocol::L7,
        }
    }

    pub const fn requires_tls(self) -> bool {
        matches!(self, Self::L7Http | Self::L7Grpc)
    }

    pub const fn requires_mtls(self) -> bool {
        matches!(self, Self::L7Grpc)
    }
}

impl Ipv4Cidr {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        let value = value.into();
        parse_ipv4_cidr(&value)?;
        Ok(Self { value })
    }
}

impl Ipv6Cidr {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        let value = value.into();
        parse_ipv6_cidr(&value)?;
        Ok(Self { value })
    }
}

impl RouteTableId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        prefixed_id(
            value.into(),
            ROUTE_TABLE_ID_PREFIX,
            CloudNetworkError::InvalidRouteTableId,
        )
        .map(|value| Self { value })
    }
}

impl SecurityGroupId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        prefixed_id(
            value.into(),
            SECURITY_GROUP_ID_PREFIX,
            CloudNetworkError::InvalidSecurityGroupId,
        )
        .map(|value| Self { value })
    }
}

impl TargetGroupId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        prefixed_id(
            value.into(),
            TARGET_GROUP_ID_PREFIX,
            CloudNetworkError::InvalidTargetGroupId,
        )
        .map(|value| Self { value })
    }
}

impl WafPolicyId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        prefixed_id(
            value.into(),
            WAF_POLICY_ID_PREFIX,
            CloudNetworkError::InvalidWafPolicyId,
        )
        .map(|value| Self { value })
    }
}

impl CertificateRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        prefixed_ref(
            value.into(),
            CERT_REF_PREFIX,
            CloudNetworkError::InvalidCertificateRef,
        )
        .map(|value| Self { value })
    }
}

impl DnssecKeyRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        prefixed_ref(
            value.into(),
            DNSSEC_KEY_REF_PREFIX,
            CloudNetworkError::InvalidDnssecKeyRef,
        )
        .map(|value| Self { value })
    }
}

impl DnsName {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        let value = value.into();
        let trimmed = value.trim_end_matches('.').to_ascii_lowercase();
        if trimmed.len() < 3
            || trimmed.len() > 253
            || trimmed.split('.').any(|label| {
                label.is_empty()
                    || label.len() > 63
                    || label.starts_with('-')
                    || label.ends_with('-')
                    || !label.bytes().all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'
                    })
            })
        {
            return Err(CloudNetworkError::InvalidDnsName);
        }
        Ok(Self { value: trimmed })
    }
}

impl FlowAnomalyId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        let value = value.into();
        if value.starts_with(FLOW_ANOMALY_ID_PREFIX) && value.len() > FLOW_ANOMALY_ID_PREFIX.len() {
            Ok(Self { value })
        } else {
            Err(CloudNetworkError::InvalidFlowAnomalyId)
        }
    }
}

impl InterconnectPartnerId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        prefixed_id(
            value.into(),
            INTERCONNECT_PARTNER_ID_PREFIX,
            CloudNetworkError::InvalidInterconnectPartnerId,
        )
        .map(|value| Self { value })
    }
}

impl InterconnectPortId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        prefixed_id(
            value.into(),
            INTERCONNECT_PORT_ID_PREFIX,
            CloudNetworkError::InvalidInterconnectPortId,
        )
        .map(|value| Self { value })
    }
}

impl BgpSessionId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        prefixed_id(
            value.into(),
            BGP_SESSION_ID_PREFIX,
            CloudNetworkError::InvalidBgpSessionId,
        )
        .map(|value| Self { value })
    }
}

impl PeeringLocation {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        canonical_location(value.into()).map(|value| Self { value })
    }
}

impl RunbookRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        prefixed_ref(
            value.into(),
            RUNBOOK_REF_PREFIX,
            CloudNetworkError::InvalidRunbookRef,
        )
        .map(|value| Self { value })
    }
}

impl OnCallGroupRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        prefixed_ref(
            value.into(),
            ONCALL_GROUP_REF_PREFIX,
            CloudNetworkError::InvalidOnCallGroupRef,
        )
        .map(|value| Self { value })
    }
}

impl MeshId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        prefixed_id(
            value.into(),
            MESH_ID_PREFIX,
            CloudNetworkError::InvalidMeshId,
        )
        .map(|value| Self { value })
    }
}

impl CedarPolicyRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        prefixed_ref(
            value.into(),
            CEDAR_POLICY_REF_PREFIX,
            CloudNetworkError::InvalidCedarPolicyRef,
        )
        .map(|value| Self { value })
    }
}

impl AuditStreamRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        prefixed_ref(
            value.into(),
            AUDIT_STREAM_REF_PREFIX,
            CloudNetworkError::InvalidAuditStreamRef,
        )
        .map(|value| Self { value })
    }
}

impl HealthAlarmRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        prefixed_ref(
            value.into(),
            HEALTH_ALARM_REF_PREFIX,
            CloudNetworkError::InvalidHealthAlarmRef,
        )
        .map(|value| Self { value })
    }
}

impl MeshNamespace {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        let value = value.into();
        if value.starts_with("mesh-")
            && value.len() > "mesh-".len()
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            && !value.ends_with('-')
        {
            Ok(Self { value })
        } else {
            Err(CloudNetworkError::InvalidMeshNamespace)
        }
    }
}

impl RouteDestination {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudNetworkError> {
        let value = value.into();
        if value.contains(':') {
            Ok(Self::Ipv6(Ipv6Cidr::new(value)?))
        } else {
            Ok(Self::Ipv4(Ipv4Cidr::new(value)?))
        }
    }
}

impl RouteTable {
    pub fn new(input: RouteTableCreate) -> Result<Self, CloudNetworkError> {
        let id = RouteTableId::new(input.id)?;
        let mut seen = BTreeSet::new();
        let mut routes = Vec::with_capacity(input.routes.len());
        for route in input.routes {
            let destination = RouteDestination::new(route.destination)?;
            if !seen.insert(destination.clone()) {
                return Err(CloudNetworkError::DuplicateRoute);
            }
            if matches!(route.next_hop, RouteNextHopKind::Local) && route.target_ref.is_some() {
                return Err(CloudNetworkError::InvalidRoute);
            }
            if !matches!(route.next_hop, RouteNextHopKind::Local) && route.target_ref.is_none() {
                return Err(CloudNetworkError::InvalidRoute);
            }
            routes.push(Route {
                destination,
                next_hop: route.next_hop,
                target_ref: route.target_ref,
            });
        }
        Ok(Self { id, routes })
    }
}

impl SecurityGroup {
    pub fn new(input: SecurityGroupCreate) -> Result<Self, CloudNetworkError> {
        let id = SecurityGroupId::new(input.id)?;
        for rule in &input.rules {
            validate_security_rule(rule)?;
        }
        Ok(Self {
            id,
            rules: input.rules,
        })
    }
}

impl Vpc {
    pub fn new(input: VpcCreate) -> Result<Self, CloudNetworkError> {
        validate_tenant_id(&input.tenant_id)?;
        if input.state != VpcState::Creating {
            return Err(CloudNetworkError::InvalidVpcState);
        }
        if !input.flow_logs_enabled {
            return Err(CloudNetworkError::FlowLogsRequired);
        }
        let region =
            RegionCode::new(input.region).map_err(|_| CloudNetworkError::InvalidResourceId)?;
        if !residency_class_allows_home_region_label(&input.residency, &region.value) {
            return Err(CloudNetworkError::ResourceRegionMismatch);
        }
        let resource_id = resource_id_for(
            &input.resource_id,
            &input.tenant_id,
            &region,
            ResourceKind::Vpc,
        )?;
        let cidr_v4 = Ipv4Cidr::new(input.cidr_v4)?;
        let cidr_v6 = Ipv6Cidr::new(input.cidr_v6)?;
        let route_table = RouteTable::new(input.route_table)?;
        let security_groups = security_groups(input.security_groups)?;
        Ok(Self {
            resource_id: internal(resource_id),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            cidr_v4: public(cidr_v4),
            cidr_v6: public(cidr_v6),
            flow_logs_enabled: public(input.flow_logs_enabled),
            route_table: internal(route_table),
            security_groups: internal(security_groups),
            residency: internal(input.residency),
            state: public(input.state),
            data_class: public(public_metadata_class(input.data_class)?),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(NETWORK_SCHEMA_VERSION),
        })
    }
}

impl Subnet {
    pub fn new(vpc: &Vpc, input: SubnetCreate) -> Result<Self, CloudNetworkError> {
        validate_tenant_id(&input.tenant_id)?;
        if input.state != SubnetState::Creating {
            return Err(CloudNetworkError::InvalidSubnetState);
        }
        let vpc_id =
            ResourceId::new(input.vpc_id).map_err(|_| CloudNetworkError::InvalidResourceId)?;
        if vpc_id != vpc.resource_id.value {
            return Err(CloudNetworkError::UnknownVpc);
        }
        if input.tenant_id != vpc.tenant_id.value {
            return Err(CloudNetworkError::ResourceTenantMismatch);
        }
        let region =
            RegionCode::new(input.region).map_err(|_| CloudNetworkError::InvalidResourceId)?;
        if region != vpc.region.value {
            return Err(CloudNetworkError::ResourceRegionMismatch);
        }
        let az = AzCode::new(input.az).map_err(|_| CloudNetworkError::InvalidAzCode)?;
        validate_az_region(&az, &region)?;
        let resource_id = resource_id_for(
            &input.resource_id,
            &input.tenant_id,
            &region,
            ResourceKind::Subnet,
        )?;
        let cidr_v4 = Ipv4Cidr::new(input.cidr_v4)?;
        let cidr_v6 = Ipv6Cidr::new(input.cidr_v6)?;
        if !ipv4_contains(&vpc.cidr_v4.value, &cidr_v4)?
            || !ipv6_contains(&vpc.cidr_v6.value, &cidr_v6)?
        {
            return Err(CloudNetworkError::SubnetOutsideVpc);
        }
        Ok(Self {
            resource_id: internal(resource_id),
            tenant_id: internal(input.tenant_id),
            vpc_id: internal(vpc_id),
            region: public(region),
            az: public(az),
            cidr_v4: public(cidr_v4),
            cidr_v6: public(cidr_v6),
            public_ip_on_launch: public(input.public_ip_on_launch),
            state: public(input.state),
            data_class: public(public_metadata_class(input.data_class)?),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(NETWORK_SCHEMA_VERSION),
        })
    }
}

impl LoadBalancer {
    pub fn new(
        vpc: &Vpc,
        known_subnets: &BTreeMap<ResourceId, Subnet>,
        input: LoadBalancerCreate,
    ) -> Result<Self, CloudNetworkError> {
        validate_tenant_id(&input.tenant_id)?;
        if input.state != LbState::Creating {
            return Err(CloudNetworkError::InvalidLbState);
        }
        let vpc_id =
            ResourceId::new(input.vpc_id).map_err(|_| CloudNetworkError::InvalidResourceId)?;
        if vpc_id != vpc.resource_id.value {
            return Err(CloudNetworkError::UnknownVpc);
        }
        if input.tenant_id != vpc.tenant_id.value {
            return Err(CloudNetworkError::ResourceTenantMismatch);
        }
        let region =
            RegionCode::new(input.region).map_err(|_| CloudNetworkError::InvalidResourceId)?;
        if region != vpc.region.value {
            return Err(CloudNetworkError::ResourceRegionMismatch);
        }
        let resource_id = resource_id_for(
            &input.resource_id,
            &input.tenant_id,
            &region,
            ResourceKind::LoadBalancer(input.kind.resource_protocol()),
        )?;
        if input.kind.requires_mtls() && input.mtls.is_none() {
            return Err(CloudNetworkError::GrpcRequiresMtls);
        }
        let target_groups =
            target_groups(input.target_groups, known_subnets, &vpc.resource_id.value)?;
        let listeners = listeners(input.listeners, &target_groups, input.kind)?;
        let mtls = input
            .mtls
            .map(|mtls| {
                Ok(MtlsConfig {
                    ca_bundle_ref: CertificateRef::new(mtls.ca_bundle_ref)?,
                    client_policy: mtls.client_policy,
                })
            })
            .transpose()?;
        let waf_policy = input.waf_policy.map(WafPolicyId::new).transpose()?;
        Ok(Self {
            resource_id: internal(resource_id),
            tenant_id: internal(input.tenant_id),
            vpc_id: internal(vpc_id),
            region: public(region),
            kind: public(input.kind),
            listeners: internal(listeners),
            target_groups: internal(target_groups),
            mtls: internal(mtls),
            waf_policy: internal(waf_policy),
            state: public(input.state),
            data_class: public(public_metadata_class(input.data_class)?),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(NETWORK_SCHEMA_VERSION),
        })
    }
}

impl DnsZone {
    pub fn new(vpc: Option<&Vpc>, input: DnsZoneCreate) -> Result<Self, CloudNetworkError> {
        validate_tenant_id(&input.tenant_id)?;
        if input.state != DnsZoneState::Creating {
            return Err(CloudNetworkError::InvalidDnsZoneState);
        }
        let region =
            RegionCode::new(input.region).map_err(|_| CloudNetworkError::InvalidResourceId)?;
        let resource_id = resource_id_for(
            &input.resource_id,
            &input.tenant_id,
            &region,
            ResourceKind::DnsZone,
        )?;
        let vpc_id = match (input.kind, input.vpc_id) {
            (DnsZoneKind::Private, Some(vpc_id)) => {
                let vpc_id =
                    ResourceId::new(vpc_id).map_err(|_| CloudNetworkError::InvalidResourceId)?;
                let Some(vpc) = vpc else {
                    return Err(CloudNetworkError::UnknownVpc);
                };
                if vpc_id != vpc.resource_id.value {
                    return Err(CloudNetworkError::UnknownVpc);
                }
                if vpc.tenant_id.value != input.tenant_id {
                    return Err(CloudNetworkError::ResourceTenantMismatch);
                }
                if vpc.region.value != region {
                    return Err(CloudNetworkError::ResourceRegionMismatch);
                }
                Some(vpc_id)
            }
            (DnsZoneKind::Private, None) => return Err(CloudNetworkError::PrivateZoneRequiresVpc),
            (DnsZoneKind::Public, Some(_)) => {
                return Err(CloudNetworkError::PublicZoneMustNotBindVpc);
            }
            (DnsZoneKind::Public, None) => None,
        };
        let dnssec_key_ref = input.dnssec_key_ref.map(DnssecKeyRef::new).transpose()?;
        if matches!(input.kind, DnsZoneKind::Public) && dnssec_key_ref.is_none() {
            return Err(CloudNetworkError::DnssecRequired);
        }
        Ok(Self {
            resource_id: internal(resource_id),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            name: public(DnsName::new(input.name)?),
            kind: public(input.kind),
            vpc_id: internal(vpc_id),
            dnssec_key_ref: internal(dnssec_key_ref),
            state: public(input.state),
            data_class: public(public_metadata_class(input.data_class)?),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(NETWORK_SCHEMA_VERSION),
        })
    }
}

impl CdnDistribution {
    pub fn new(
        known_load_balancers: &BTreeMap<ResourceId, LoadBalancer>,
        known_dns_zones: &BTreeMap<ResourceId, DnsZone>,
        input: CdnDistributionCreate,
    ) -> Result<Self, CloudNetworkError> {
        validate_tenant_id(&input.tenant_id)?;
        if input.state != CdnState::Creating {
            return Err(CloudNetworkError::InvalidCdnState);
        }
        if input.tls_certificate.trim().is_empty() {
            return Err(CloudNetworkError::CdnTlsRequired);
        }
        if input.waf_policy.trim().is_empty() {
            return Err(CloudNetworkError::CdnWafRequired);
        }
        let region =
            RegionCode::new(input.region).map_err(|_| CloudNetworkError::InvalidResourceId)?;
        let resource_id = resource_id_for(
            &input.resource_id,
            &input.tenant_id,
            &region,
            ResourceKind::CdnDistribution,
        )?;
        let hostnames = dns_names(input.hostnames)?;
        let origins = cdn_origins(
            input.origins,
            known_load_balancers,
            known_dns_zones,
            &input.tenant_id,
            &region,
        )?;
        Ok(Self {
            resource_id: internal(resource_id),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            hostnames: public(hostnames),
            origins: internal(origins),
            tls_certificate: internal(CertificateRef::new(input.tls_certificate)?),
            waf_policy: internal(WafPolicyId::new(input.waf_policy)?),
            cache_mode: public(input.cache_mode),
            state: public(input.state),
            data_class: public(public_metadata_class(input.data_class)?),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(NETWORK_SCHEMA_VERSION),
        })
    }
}

impl InterconnectPartner {
    pub fn new(input: InterconnectPartnerCreate) -> Result<Self, CloudNetworkError> {
        let id = InterconnectPartnerId::new(input.id)?;
        let region =
            RegionCode::new(input.region).map_err(|_| CloudNetworkError::InvalidResourceId)?;
        validate_interconnect_sla(input.per_link_sla_basis_points)?;
        if input.name.trim().is_empty() {
            return Err(CloudNetworkError::InvalidInterconnectPartnerId);
        }
        let peering_locations = peering_locations(input.peering_locations)?;
        Ok(Self {
            id: internal(id),
            name: public(input.name),
            region: public(region),
            peering_locations: public(peering_locations),
            per_link_sla_basis_points: public(input.per_link_sla_basis_points),
            schema_version: public(NETWORK_SCHEMA_VERSION),
        })
    }
}

impl DirectInterconnect {
    pub fn new(
        known_partners: &BTreeMap<InterconnectPartnerId, InterconnectPartner>,
        input: DirectInterconnectCreate,
    ) -> Result<Self, CloudNetworkError> {
        validate_tenant_id(&input.tenant_id)?;
        if input.state != DirectInterconnectState::Creating {
            return Err(CloudNetworkError::InvalidInterconnectState);
        }
        let region =
            RegionCode::new(input.region).map_err(|_| CloudNetworkError::InvalidResourceId)?;
        let resource_id = resource_id_for(
            &input.resource_id,
            &input.tenant_id,
            &region,
            ResourceKind::DirectInterconnect,
        )?;
        let partner_id = InterconnectPartnerId::new(input.partner_id)?;
        let partner = known_partners
            .get(&partner_id)
            .ok_or(CloudNetworkError::UnknownInterconnectPartner)?;
        if partner.region.value != region {
            return Err(CloudNetworkError::ResourceRegionMismatch);
        }
        validate_regional_interconnect_diversity(known_partners, &region)?;
        let peering_location = PeeringLocation::new(input.peering_location)?;
        if !partner.peering_locations.value.contains(&peering_location) {
            return Err(CloudNetworkError::InvalidPeeringLocation);
        }
        validate_interconnect_sla(input.per_link_sla_basis_points)?;
        if input.bandwidth_mbps == 0 {
            return Err(CloudNetworkError::InvalidBandwidth);
        }
        if input.redundant_port_count < 2 {
            return Err(CloudNetworkError::InterconnectRedundancyRequired);
        }
        if !(1..=4094).contains(&input.vlan_tag) {
            return Err(CloudNetworkError::InvalidVlanTag);
        }
        Ok(Self {
            resource_id: internal(resource_id),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            partner_id: internal(partner_id),
            peering_location: public(peering_location),
            physical_port_id: internal(InterconnectPortId::new(input.physical_port_id)?),
            vlan_tag: internal(input.vlan_tag),
            bandwidth_mbps: public(input.bandwidth_mbps),
            redundant_port_count: public(input.redundant_port_count),
            bgp_sessions: internal(bgp_sessions(input.bgp_sessions)?),
            advertised_prefixes: internal(advertised_prefixes(input.advertised_prefixes)?),
            per_link_sla_basis_points: public(input.per_link_sla_basis_points),
            state: public(input.state),
            data_class: public(public_metadata_class(input.data_class)?),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(NETWORK_SCHEMA_VERSION),
        })
    }
}

impl DdosProtection {
    pub fn new(
        vpcs: &BTreeMap<ResourceId, Vpc>,
        load_balancers: &BTreeMap<ResourceId, LoadBalancer>,
        dns_zones: &BTreeMap<ResourceId, DnsZone>,
        cdn_distributions: &BTreeMap<ResourceId, CdnDistribution>,
        input: DdosProtectionCreate,
    ) -> Result<Self, CloudNetworkError> {
        validate_tenant_id(&input.tenant_id)?;
        if input.state != DdosProtectionState::Creating {
            return Err(CloudNetworkError::InvalidDdosState);
        }
        if !input.line_rate_scrubbing {
            return Err(CloudNetworkError::LineRateScrubbingRequired);
        }
        if !input.always_on {
            return Err(CloudNetworkError::DdosAlwaysOnRequired);
        }
        let region =
            RegionCode::new(input.region).map_err(|_| CloudNetworkError::InvalidResourceId)?;
        let resource_id = resource_id_for(
            &input.resource_id,
            &input.tenant_id,
            &region,
            ResourceKind::DdosProtection,
        )?;
        let protected_resources = protected_resources(
            input.protected_resource_ids,
            vpcs,
            load_balancers,
            dns_zones,
            cdn_distributions,
            &input.tenant_id,
            &region,
        )?;
        let scrubbing_regions = scrubbing_regions(input.scrubbing_regions, &region)?;
        Ok(Self {
            resource_id: internal(resource_id),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            protected_resources: internal(protected_resources),
            scrubbing_regions: public(scrubbing_regions),
            line_rate_scrubbing: public(input.line_rate_scrubbing),
            always_on: public(input.always_on),
            mitigation_runbook_ref: internal(RunbookRef::new(input.mitigation_runbook_ref)?),
            oncall_group_ref: internal(OnCallGroupRef::new(input.oncall_group_ref)?),
            state: public(input.state),
            data_class: public(public_metadata_class(input.data_class)?),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(NETWORK_SCHEMA_VERSION),
        })
    }
}

impl ServiceMeshCell {
    pub fn new(
        vpcs: &BTreeMap<ResourceId, Vpc>,
        input: ServiceMeshCellCreate,
    ) -> Result<Self, CloudNetworkError> {
        validate_tenant_id(&input.tenant_id)?;
        if input.state != ServiceMeshState::Creating {
            return Err(CloudNetworkError::InvalidMeshState);
        }
        if input.mode != ServiceMeshMode::IstioAmbient {
            return Err(CloudNetworkError::InvalidMeshMode);
        }
        if input.edge_gateway != MeshGatewayKind::Envoy {
            return Err(CloudNetworkError::InvalidMeshGateway);
        }
        if !input.mtls_everywhere {
            return Err(CloudNetworkError::MeshMtlsRequired);
        }
        if !input.ext_authz_enabled {
            return Err(CloudNetworkError::MeshExtAuthzRequired);
        }
        if input.control_plane_replicas < 3 {
            return Err(CloudNetworkError::MeshControlPlaneReplicasRequired);
        }
        if !input.quarterly_upgrade_drill {
            return Err(CloudNetworkError::MeshUpgradeDrillRequired);
        }
        let region =
            RegionCode::new(input.region).map_err(|_| CloudNetworkError::InvalidResourceId)?;
        let vpc_id =
            ResourceId::new(input.vpc_id).map_err(|_| CloudNetworkError::InvalidResourceId)?;
        let vpc = vpcs.get(&vpc_id).ok_or(CloudNetworkError::UnknownVpc)?;
        if vpc.tenant_id.value != input.tenant_id {
            return Err(CloudNetworkError::ResourceTenantMismatch);
        }
        if vpc.region.value != region {
            return Err(CloudNetworkError::ResourceRegionMismatch);
        }
        let cell_id = CellId::new(input.cell_id).map_err(|_| CloudNetworkError::InvalidCellId)?;
        validate_cell_region(&cell_id, &region)?;
        let namespace = MeshNamespace::new(input.namespace)?;
        validate_mesh_namespace_cell(&namespace, &cell_id)?;
        Ok(Self {
            mesh_id: internal(MeshId::new(input.mesh_id)?),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            cell_id: public(cell_id),
            vpc_id: internal(vpc_id),
            namespace: internal(namespace),
            mode: public(input.mode),
            edge_gateway: public(input.edge_gateway),
            mtls_everywhere: public(input.mtls_everywhere),
            ext_authz_enabled: public(input.ext_authz_enabled),
            cross_cell_policy_ref: internal(CedarPolicyRef::new(input.cross_cell_policy_ref)?),
            audit_stream_ref: internal(AuditStreamRef::new(input.audit_stream_ref)?),
            health_alarm_ref: internal(HealthAlarmRef::new(input.health_alarm_ref)?),
            control_plane_replicas: public(input.control_plane_replicas),
            quarterly_upgrade_drill: public(input.quarterly_upgrade_drill),
            state: public(input.state),
            data_class: public(public_metadata_class(input.data_class)?),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(NETWORK_SCHEMA_VERSION),
        })
    }
}

impl NetworkRepo for CloudNetworkCatalog {
    fn create_vpc(&mut self, input: VpcCreate) -> Result<Vpc, CloudNetworkError> {
        let vpc = Vpc::new(input)?;
        if self.vpcs.contains_key(&vpc.resource_id.value) {
            return Err(CloudNetworkError::DuplicateVpc);
        }
        self.vpcs.insert(vpc.resource_id.value.clone(), vpc.clone());
        Ok(vpc)
    }

    fn add_subnet(&mut self, input: SubnetCreate) -> Result<Subnet, CloudNetworkError> {
        let vpc_id = ResourceId::new(input.vpc_id.clone())
            .map_err(|_| CloudNetworkError::InvalidResourceId)?;
        let vpc = self
            .vpcs
            .get(&vpc_id)
            .ok_or(CloudNetworkError::UnknownVpc)?;
        let subnet = Subnet::new(vpc, input)?;
        if self.subnets.contains_key(&subnet.resource_id.value) {
            return Err(CloudNetworkError::DuplicateSubnet);
        }
        for existing in self
            .subnets
            .values()
            .filter(|existing| existing.vpc_id.value == vpc_id)
        {
            if ipv4_overlaps(&existing.cidr_v4.value, &subnet.cidr_v4.value)?
                || ipv6_overlaps(&existing.cidr_v6.value, &subnet.cidr_v6.value)?
            {
                return Err(CloudNetworkError::OverlappingSubnet);
            }
        }
        self.subnets
            .insert(subnet.resource_id.value.clone(), subnet.clone());
        Ok(subnet)
    }

    fn create_load_balancer(
        &mut self,
        input: LoadBalancerCreate,
    ) -> Result<LoadBalancer, CloudNetworkError> {
        let vpc_id = ResourceId::new(input.vpc_id.clone())
            .map_err(|_| CloudNetworkError::InvalidResourceId)?;
        let vpc = self
            .vpcs
            .get(&vpc_id)
            .ok_or(CloudNetworkError::UnknownVpc)?;
        let load_balancer = LoadBalancer::new(vpc, &self.subnets, input)?;
        if self
            .load_balancers
            .contains_key(&load_balancer.resource_id.value)
        {
            return Err(CloudNetworkError::DuplicateLoadBalancer);
        }
        self.load_balancers.insert(
            load_balancer.resource_id.value.clone(),
            load_balancer.clone(),
        );
        Ok(load_balancer)
    }

    fn create_dns_zone(&mut self, input: DnsZoneCreate) -> Result<DnsZone, CloudNetworkError> {
        let vpc = input
            .vpc_id
            .as_ref()
            .and_then(|id| ResourceId::new(id.clone()).ok())
            .and_then(|id| self.vpcs.get(&id));
        let zone = DnsZone::new(vpc, input)?;
        if self.dns_zones.contains_key(&zone.resource_id.value) {
            return Err(CloudNetworkError::DuplicateDnsZone);
        }
        self.dns_zones
            .insert(zone.resource_id.value.clone(), zone.clone());
        Ok(zone)
    }

    fn create_cdn_distribution(
        &mut self,
        input: CdnDistributionCreate,
    ) -> Result<CdnDistribution, CloudNetworkError> {
        let distribution = CdnDistribution::new(&self.load_balancers, &self.dns_zones, input)?;
        if self
            .cdn_distributions
            .contains_key(&distribution.resource_id.value)
        {
            return Err(CloudNetworkError::DuplicateCdnDistribution);
        }
        self.cdn_distributions
            .insert(distribution.resource_id.value.clone(), distribution.clone());
        Ok(distribution)
    }

    fn add_interconnect_partner(
        &mut self,
        input: InterconnectPartnerCreate,
    ) -> Result<InterconnectPartner, CloudNetworkError> {
        let partner = InterconnectPartner::new(input)?;
        if self.interconnect_partners.contains_key(&partner.id.value) {
            return Err(CloudNetworkError::DuplicateInterconnectPartner);
        }
        self.interconnect_partners
            .insert(partner.id.value.clone(), partner.clone());
        Ok(partner)
    }

    fn create_direct_interconnect(
        &mut self,
        input: DirectInterconnectCreate,
    ) -> Result<DirectInterconnect, CloudNetworkError> {
        let interconnect = DirectInterconnect::new(&self.interconnect_partners, input)?;
        if self
            .direct_interconnects
            .contains_key(&interconnect.resource_id.value)
        {
            return Err(CloudNetworkError::DuplicateDirectInterconnect);
        }
        self.direct_interconnects
            .insert(interconnect.resource_id.value.clone(), interconnect.clone());
        Ok(interconnect)
    }

    fn create_ddos_protection(
        &mut self,
        input: DdosProtectionCreate,
    ) -> Result<DdosProtection, CloudNetworkError> {
        let protection = DdosProtection::new(
            &self.vpcs,
            &self.load_balancers,
            &self.dns_zones,
            &self.cdn_distributions,
            input,
        )?;
        if self
            .ddos_protections
            .contains_key(&protection.resource_id.value)
        {
            return Err(CloudNetworkError::DuplicateDdosProtection);
        }
        self.ddos_protections
            .insert(protection.resource_id.value.clone(), protection.clone());
        Ok(protection)
    }

    fn create_service_mesh_cell(
        &mut self,
        input: ServiceMeshCellCreate,
    ) -> Result<ServiceMeshCell, CloudNetworkError> {
        let mesh = ServiceMeshCell::new(&self.vpcs, input)?;
        if self.service_meshes.contains_key(&mesh.mesh_id.value) {
            return Err(CloudNetworkError::DuplicateServiceMesh);
        }
        self.service_meshes
            .insert(mesh.mesh_id.value.clone(), mesh.clone());
        Ok(mesh)
    }

    fn record_flow_anomaly(
        &mut self,
        id: String,
        vpc_id: String,
        severity: FlowAnomalySeverity,
        flow_pattern: String,
        detected_at_epoch_seconds: u64,
    ) -> Result<FlowAnomalyEvent, CloudNetworkError> {
        let id = FlowAnomalyId::new(id)?;
        let vpc_id = ResourceId::new(vpc_id).map_err(|_| CloudNetworkError::InvalidResourceId)?;
        let vpc = self
            .vpcs
            .get(&vpc_id)
            .ok_or(CloudNetworkError::UnknownVpc)?;
        if !vpc.flow_logs_enabled.value {
            return Err(CloudNetworkError::FlowLogsRequired);
        }
        if self.anomalies.contains_key(&id) {
            return Err(CloudNetworkError::DuplicateFlowAnomaly);
        }
        if flow_pattern.trim().is_empty() {
            return Err(CloudNetworkError::InvalidRoute);
        }
        let event = FlowAnomalyEvent {
            id: internal(id.clone()),
            tenant_id: internal(vpc.tenant_id.value.clone()),
            vpc_id: internal(vpc_id),
            severity: public(severity),
            flow_pattern: internal(flow_pattern),
            detected_at_epoch_seconds: internal(detected_at_epoch_seconds),
            schema_version: public(NETWORK_SCHEMA_VERSION),
        };
        self.anomalies.insert(id, event.clone());
        Ok(event)
    }
}

impl CloudNetworkCatalog {
    pub fn vpcs(&self) -> impl Iterator<Item = &Vpc> {
        self.vpcs.values()
    }

    pub fn subnets(&self) -> impl Iterator<Item = &Subnet> {
        self.subnets.values()
    }

    pub fn load_balancers(&self) -> impl Iterator<Item = &LoadBalancer> {
        self.load_balancers.values()
    }

    pub fn dns_zones(&self) -> impl Iterator<Item = &DnsZone> {
        self.dns_zones.values()
    }

    pub fn cdn_distributions(&self) -> impl Iterator<Item = &CdnDistribution> {
        self.cdn_distributions.values()
    }

    pub fn interconnect_partners(&self) -> impl Iterator<Item = &InterconnectPartner> {
        self.interconnect_partners.values()
    }

    pub fn direct_interconnects(&self) -> impl Iterator<Item = &DirectInterconnect> {
        self.direct_interconnects.values()
    }

    pub fn ddos_protections(&self) -> impl Iterator<Item = &DdosProtection> {
        self.ddos_protections.values()
    }

    pub fn service_meshes(&self) -> impl Iterator<Item = &ServiceMeshCell> {
        self.service_meshes.values()
    }

    pub fn anomalies(&self) -> impl Iterator<Item = &FlowAnomalyEvent> {
        self.anomalies.values()
    }
}

fn resource_id_for(
    value: &str,
    tenant_id: &str,
    region: &RegionCode,
    kind: ResourceKind,
) -> Result<ResourceId, CloudNetworkError> {
    let id = ResourceId::new(value.to_string()).map_err(map_resource_error)?;
    if id.tenant_id().map_err(map_resource_error)? != tenant_id {
        return Err(CloudNetworkError::ResourceTenantMismatch);
    }
    if id.region().map_err(map_resource_error)? != *region {
        return Err(CloudNetworkError::ResourceRegionMismatch);
    }
    if id.kind_label().map_err(map_resource_error)? != kind.type_label() {
        return Err(CloudNetworkError::ResourceKindMismatch);
    }
    Ok(id)
}

fn security_groups(
    input: Vec<SecurityGroupCreate>,
) -> Result<Vec<SecurityGroup>, CloudNetworkError> {
    let mut seen = BTreeSet::new();
    let mut groups = Vec::with_capacity(input.len());
    for group in input {
        let group = SecurityGroup::new(group)?;
        if !seen.insert(group.id.clone()) {
            return Err(CloudNetworkError::DuplicateSecurityGroup);
        }
        groups.push(group);
    }
    Ok(groups)
}

fn target_groups(
    input: Vec<TargetGroupCreate>,
    known_subnets: &BTreeMap<ResourceId, Subnet>,
    vpc_id: &ResourceId,
) -> Result<Vec<TargetGroup>, CloudNetworkError> {
    if input.is_empty() {
        return Err(CloudNetworkError::InvalidTargetGroupId);
    }
    let mut seen = BTreeSet::new();
    let mut groups = Vec::with_capacity(input.len());
    for group in input {
        let id = TargetGroupId::new(group.id)?;
        if !seen.insert(id.clone()) {
            return Err(CloudNetworkError::DuplicateTargetGroup);
        }
        let mut subnet_ids = Vec::with_capacity(group.subnet_ids.len());
        if group.subnet_ids.is_empty() {
            return Err(CloudNetworkError::UnknownSubnet);
        }
        for subnet_id in group.subnet_ids {
            let subnet_id =
                ResourceId::new(subnet_id).map_err(|_| CloudNetworkError::InvalidResourceId)?;
            let subnet = known_subnets
                .get(&subnet_id)
                .ok_or(CloudNetworkError::UnknownSubnet)?;
            if &subnet.vpc_id.value != vpc_id {
                return Err(CloudNetworkError::UnknownSubnet);
            }
            subnet_ids.push(subnet_id);
        }
        if group
            .health_check_path
            .as_ref()
            .is_some_and(|path| !path.starts_with('/'))
        {
            return Err(CloudNetworkError::InvalidTargetGroupId);
        }
        groups.push(TargetGroup {
            id,
            subnet_ids,
            health_check_path: group.health_check_path,
        });
    }
    Ok(groups)
}

fn listeners(
    input: Vec<ListenerCreate>,
    target_groups: &[TargetGroup],
    kind: LbKind,
) -> Result<Vec<Listener>, CloudNetworkError> {
    if input.is_empty() {
        return Err(CloudNetworkError::InvalidListener);
    }
    let target_ids: BTreeSet<_> = target_groups.iter().map(|group| group.id.clone()).collect();
    let mut seen_ports = BTreeSet::new();
    let mut listeners = Vec::with_capacity(input.len());
    for listener in input {
        if listener.port == 0 || !seen_ports.insert(listener.port) {
            return Err(CloudNetworkError::DuplicateListenerPort);
        }
        let target_group_id = TargetGroupId::new(listener.target_group_id)?;
        if !target_ids.contains(&target_group_id) {
            return Err(CloudNetworkError::ListenerTargetGroupMissing);
        }
        let tls_certificate = listener
            .tls_certificate
            .map(CertificateRef::new)
            .transpose()?;
        if kind.requires_tls() && tls_certificate.is_none() {
            return Err(CloudNetworkError::L7RequiresTls);
        }
        listeners.push(Listener {
            port: listener.port,
            target_group_id,
            tls_certificate,
        });
    }
    Ok(listeners)
}

fn dns_names(input: Vec<String>) -> Result<Vec<DnsName>, CloudNetworkError> {
    if input.is_empty() {
        return Err(CloudNetworkError::InvalidDnsName);
    }
    let mut seen = BTreeSet::new();
    let mut names = Vec::with_capacity(input.len());
    for hostname in input {
        let name = DnsName::new(hostname)?;
        if !seen.insert(name.clone()) {
            return Err(CloudNetworkError::DuplicateCdnHostname);
        }
        names.push(name);
    }
    Ok(names)
}

fn cdn_origins(
    input: Vec<CdnOriginCreate>,
    known_load_balancers: &BTreeMap<ResourceId, LoadBalancer>,
    known_dns_zones: &BTreeMap<ResourceId, DnsZone>,
    tenant_id: &str,
    region: &RegionCode,
) -> Result<Vec<CdnOrigin>, CloudNetworkError> {
    if input.is_empty() {
        return Err(CloudNetworkError::InvalidCdnOrigin);
    }
    let mut seen = BTreeSet::new();
    let mut origins = Vec::with_capacity(input.len());
    for origin in input {
        let resource_id = ResourceId::new(origin.resource_id)
            .map_err(|_| CloudNetworkError::InvalidResourceId)?;
        if !seen.insert(resource_id.clone()) {
            return Err(CloudNetworkError::DuplicateCdnOrigin);
        }
        validate_resource_scope(&resource_id, tenant_id, region)?;
        match origin.kind {
            CdnOriginKind::LoadBalancer => {
                known_load_balancers
                    .get(&resource_id)
                    .ok_or(CloudNetworkError::UnknownLoadBalancer)?;
            }
            CdnOriginKind::DnsZone => {
                let zone = known_dns_zones
                    .get(&resource_id)
                    .ok_or(CloudNetworkError::UnknownDnsZone)?;
                if zone.kind.value != DnsZoneKind::Public {
                    return Err(CloudNetworkError::InvalidCdnOrigin);
                }
            }
        }
        origins.push(CdnOrigin {
            resource_id,
            kind: origin.kind,
        });
    }
    Ok(origins)
}

fn peering_locations(input: Vec<String>) -> Result<Vec<PeeringLocation>, CloudNetworkError> {
    if input.is_empty() {
        return Err(CloudNetworkError::InvalidPeeringLocation);
    }
    let mut seen = BTreeSet::new();
    let mut locations = Vec::with_capacity(input.len());
    for location in input {
        let location = PeeringLocation::new(location)?;
        if !seen.insert(location.clone()) {
            return Err(CloudNetworkError::InvalidPeeringLocation);
        }
        locations.push(location);
    }
    Ok(locations)
}

fn validate_interconnect_sla(value: u16) -> Result<(), CloudNetworkError> {
    if (9_999..=10_000).contains(&value) {
        Ok(())
    } else {
        Err(CloudNetworkError::InterconnectSlaRequired)
    }
}

fn validate_regional_interconnect_diversity(
    known_partners: &BTreeMap<InterconnectPartnerId, InterconnectPartner>,
    region: &RegionCode,
) -> Result<(), CloudNetworkError> {
    let regional_partners = known_partners
        .values()
        .filter(|partner| partner.region.value == *region)
        .count();
    if regional_partners >= 2 {
        Ok(())
    } else {
        Err(CloudNetworkError::RegionalInterconnectDiversityRequired)
    }
}

fn bgp_sessions(input: Vec<BgpSessionCreate>) -> Result<Vec<BgpSession>, CloudNetworkError> {
    if input.len() < 2 {
        return Err(CloudNetworkError::InterconnectRedundancyRequired);
    }
    let mut seen = BTreeSet::new();
    let mut sessions = Vec::with_capacity(input.len());
    for session in input {
        let id = BgpSessionId::new(session.id)?;
        if !seen.insert(id.clone()) {
            return Err(CloudNetworkError::DuplicateBgpSession);
        }
        validate_asn(session.local_asn)?;
        validate_asn(session.peer_asn)?;
        let local_address = IpAddr::from_str(&session.local_address)
            .map_err(|_| CloudNetworkError::InvalidBgpSession)?;
        let peer_address = IpAddr::from_str(&session.peer_address)
            .map_err(|_| CloudNetworkError::InvalidBgpSession)?;
        if local_address == peer_address || local_address.is_ipv4() != peer_address.is_ipv4() {
            return Err(CloudNetworkError::InvalidBgpSession);
        }
        sessions.push(BgpSession {
            id,
            local_asn: session.local_asn,
            peer_asn: session.peer_asn,
            local_address,
            peer_address,
        });
    }
    Ok(sessions)
}

fn validate_asn(value: u32) -> Result<(), CloudNetworkError> {
    if value == 0 || value == 23_456 {
        Err(CloudNetworkError::InvalidAsn)
    } else {
        Ok(())
    }
}

fn advertised_prefixes(input: Vec<String>) -> Result<Vec<RouteDestination>, CloudNetworkError> {
    if input.is_empty() {
        return Err(CloudNetworkError::InvalidRoute);
    }
    let mut seen = BTreeSet::new();
    let mut prefixes = Vec::with_capacity(input.len());
    for prefix in input {
        let prefix = RouteDestination::new(prefix)?;
        if !seen.insert(prefix.clone()) {
            return Err(CloudNetworkError::DuplicateRoute);
        }
        prefixes.push(prefix);
    }
    Ok(prefixes)
}

fn protected_resources(
    input: Vec<String>,
    vpcs: &BTreeMap<ResourceId, Vpc>,
    load_balancers: &BTreeMap<ResourceId, LoadBalancer>,
    dns_zones: &BTreeMap<ResourceId, DnsZone>,
    cdn_distributions: &BTreeMap<ResourceId, CdnDistribution>,
    tenant_id: &str,
    region: &RegionCode,
) -> Result<Vec<ProtectedResourceRef>, CloudNetworkError> {
    if input.is_empty() {
        return Err(CloudNetworkError::UnknownProtectedResource);
    }
    let mut seen = BTreeSet::new();
    let mut protected = Vec::with_capacity(input.len());
    for resource in input {
        let resource_id =
            ResourceId::new(resource).map_err(|_| CloudNetworkError::InvalidResourceId)?;
        if !seen.insert(resource_id.clone()) {
            return Err(CloudNetworkError::DuplicateProtectedResource);
        }
        validate_resource_scope(&resource_id, tenant_id, region)?;
        let kind = match resource_id
            .kind_label()
            .map_err(map_resource_error)?
            .as_str()
        {
            "vpc" => {
                vpcs.get(&resource_id)
                    .ok_or(CloudNetworkError::UnknownProtectedResource)?;
                ProtectedResourceKind::Vpc
            }
            "lb-v4" | "lb-v7" => {
                load_balancers
                    .get(&resource_id)
                    .ok_or(CloudNetworkError::UnknownProtectedResource)?;
                ProtectedResourceKind::LoadBalancer
            }
            "dns-zone" => {
                dns_zones
                    .get(&resource_id)
                    .ok_or(CloudNetworkError::UnknownProtectedResource)?;
                ProtectedResourceKind::DnsZone
            }
            "cdn-distribution" => {
                cdn_distributions
                    .get(&resource_id)
                    .ok_or(CloudNetworkError::UnknownProtectedResource)?;
                ProtectedResourceKind::CdnDistribution
            }
            _ => return Err(CloudNetworkError::UnknownProtectedResource),
        };
        protected.push(ProtectedResourceRef { resource_id, kind });
    }
    Ok(protected)
}

fn scrubbing_regions(
    input: Vec<String>,
    home_region: &RegionCode,
) -> Result<Vec<RegionCode>, CloudNetworkError> {
    if input.is_empty() {
        return Err(CloudNetworkError::ScrubbingRegionRequired);
    }
    let mut seen = BTreeSet::new();
    let mut regions = Vec::with_capacity(input.len());
    for region in input {
        let region = RegionCode::new(region).map_err(|_| CloudNetworkError::InvalidResourceId)?;
        if !seen.insert(region.clone()) {
            return Err(CloudNetworkError::ScrubbingRegionRequired);
        }
        regions.push(region);
    }
    if !seen.contains(home_region) {
        return Err(CloudNetworkError::ScrubbingRegionRequired);
    }
    Ok(regions)
}

fn validate_resource_scope(
    resource_id: &ResourceId,
    tenant_id: &str,
    region: &RegionCode,
) -> Result<(), CloudNetworkError> {
    if resource_id.tenant_id().map_err(map_resource_error)? != tenant_id {
        return Err(CloudNetworkError::ResourceTenantMismatch);
    }
    if resource_id.region().map_err(map_resource_error)? != *region {
        return Err(CloudNetworkError::ResourceRegionMismatch);
    }
    Ok(())
}

fn validate_cell_region(cell_id: &CellId, region: &RegionCode) -> Result<(), CloudNetworkError> {
    let expected_prefix = format!("cell-{}-", region.value);
    if cell_id.value.starts_with(&expected_prefix) {
        Ok(())
    } else {
        Err(CloudNetworkError::InvalidCellId)
    }
}

fn validate_mesh_namespace_cell(
    namespace: &MeshNamespace,
    cell_id: &CellId,
) -> Result<(), CloudNetworkError> {
    if namespace.value.contains(&cell_id.value) {
        Ok(())
    } else {
        Err(CloudNetworkError::InvalidMeshNamespace)
    }
}

fn validate_security_rule(rule: &SecurityRule) -> Result<(), CloudNetworkError> {
    if let Some((start, end)) = rule.port_range
        && (start == 0 || end == 0 || start > end || matches!(rule.protocol, IpProtocol::Icmp))
    {
        return Err(CloudNetworkError::InvalidSecurityRule);
    }
    if rule.description.trim().is_empty() || rule.description.len() > 128 {
        return Err(CloudNetworkError::InvalidSecurityRule);
    }
    Ok(())
}

fn validate_network_provider_ref(
    value: &str,
    error: NetworkProviderVpcError,
) -> Result<(), NetworkProviderVpcError> {
    if value.trim().is_empty()
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_network_provider_load_balancer_ref(
    value: &str,
    error: NetworkProviderLoadBalancerError,
) -> Result<(), NetworkProviderLoadBalancerError> {
    if value.trim().is_empty()
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_network_provider_dns_zone_ref(
    value: &str,
    error: NetworkProviderDnsZoneError,
) -> Result<(), NetworkProviderDnsZoneError> {
    if value.trim().is_empty()
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_network_provider_direct_interconnect_ref(
    value: &str,
    error: NetworkProviderDirectInterconnectError,
) -> Result<(), NetworkProviderDirectInterconnectError> {
    if value.trim().is_empty()
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Err(error)
    } else {
        Ok(())
    }
}

fn public_metadata_class(data_class: DataClass) -> Result<PrivacyDataClass, CloudNetworkError> {
    if data_class != DataClass::Public {
        return Err(CloudNetworkError::InvalidDataClass);
    }
    PrivacyDataClass::new(data_class).map_err(|_| CloudNetworkError::InvalidDataClass)
}

fn validate_tenant_id(value: &str) -> Result<(), CloudNetworkError> {
    if value.starts_with(TENANT_ID_PREFIX) && value.len() > TENANT_ID_PREFIX.len() {
        Ok(())
    } else {
        Err(CloudNetworkError::InvalidTenantId)
    }
}

fn validate_az_region(az: &AzCode, region: &RegionCode) -> Result<(), CloudNetworkError> {
    if az.value == region.value
        || az
            .value
            .strip_prefix(&region.value)
            .is_some_and(|suffix| suffix.starts_with('-') && suffix.len() > 1)
    {
        Ok(())
    } else {
        Err(CloudNetworkError::AzRegionMismatch)
    }
}

fn prefixed_id(
    value: String,
    prefix: &str,
    error: CloudNetworkError,
) -> Result<String, CloudNetworkError> {
    if value.starts_with(prefix)
        && value.len() > prefix.len()
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Ok(value)
    } else {
        Err(error)
    }
}

fn prefixed_ref(
    value: String,
    prefix: &str,
    error: CloudNetworkError,
) -> Result<String, CloudNetworkError> {
    if value.starts_with(prefix)
        && value.len() > prefix.len()
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Ok(value)
    } else {
        Err(error)
    }
}

fn canonical_location(value: String) -> Result<String, CloudNetworkError> {
    if (3..=96).contains(&value.len())
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        Ok(value)
    } else {
        Err(CloudNetworkError::InvalidPeeringLocation)
    }
}

fn parse_ipv4_cidr(value: &str) -> Result<(u32, u8), CloudNetworkError> {
    let (addr, prefix) = value
        .split_once('/')
        .ok_or(CloudNetworkError::InvalidIpv4Cidr)?;
    let addr = Ipv4Addr::from_str(addr).map_err(|_| CloudNetworkError::InvalidIpv4Cidr)?;
    let prefix: u8 = prefix
        .parse()
        .map_err(|_| CloudNetworkError::InvalidIpv4Cidr)?;
    if prefix > 32 {
        return Err(CloudNetworkError::InvalidIpv4Cidr);
    }
    Ok((u32::from(addr), prefix))
}

fn parse_ipv6_cidr(value: &str) -> Result<(u128, u8), CloudNetworkError> {
    let (addr, prefix) = value
        .split_once('/')
        .ok_or(CloudNetworkError::InvalidIpv6Cidr)?;
    let addr = Ipv6Addr::from_str(addr).map_err(|_| CloudNetworkError::InvalidIpv6Cidr)?;
    let prefix: u8 = prefix
        .parse()
        .map_err(|_| CloudNetworkError::InvalidIpv6Cidr)?;
    if prefix > 128 {
        return Err(CloudNetworkError::InvalidIpv6Cidr);
    }
    Ok((u128::from(addr), prefix))
}

fn ipv4_contains(parent: &Ipv4Cidr, child: &Ipv4Cidr) -> Result<bool, CloudNetworkError> {
    let (parent_addr, parent_prefix) = parse_ipv4_cidr(&parent.value)?;
    let (child_addr, child_prefix) = parse_ipv4_cidr(&child.value)?;
    if child_prefix < parent_prefix {
        return Ok(false);
    }
    let mask = if parent_prefix == 0 {
        0
    } else {
        u32::MAX << (32 - parent_prefix)
    };
    Ok((parent_addr & mask) == (child_addr & mask))
}

fn ipv6_contains(parent: &Ipv6Cidr, child: &Ipv6Cidr) -> Result<bool, CloudNetworkError> {
    let (parent_addr, parent_prefix) = parse_ipv6_cidr(&parent.value)?;
    let (child_addr, child_prefix) = parse_ipv6_cidr(&child.value)?;
    if child_prefix < parent_prefix {
        return Ok(false);
    }
    let mask = if parent_prefix == 0 {
        0
    } else {
        u128::MAX << (128 - parent_prefix)
    };
    Ok((parent_addr & mask) == (child_addr & mask))
}

fn ipv4_overlaps(left: &Ipv4Cidr, right: &Ipv4Cidr) -> Result<bool, CloudNetworkError> {
    let (left_addr, left_prefix) = parse_ipv4_cidr(&left.value)?;
    let (right_addr, right_prefix) = parse_ipv4_cidr(&right.value)?;
    let prefix = left_prefix.min(right_prefix);
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    Ok((left_addr & mask) == (right_addr & mask))
}

fn ipv6_overlaps(left: &Ipv6Cidr, right: &Ipv6Cidr) -> Result<bool, CloudNetworkError> {
    let (left_addr, left_prefix) = parse_ipv6_cidr(&left.value)?;
    let (right_addr, right_prefix) = parse_ipv6_cidr(&right.value)?;
    let prefix = left_prefix.min(right_prefix);
    let mask = if prefix == 0 {
        0
    } else {
        u128::MAX << (128 - prefix)
    };
    Ok((left_addr & mask) == (right_addr & mask))
}

fn map_resource_error(error: CloudResourceError) -> CloudNetworkError {
    match error {
        CloudResourceError::InvalidResourceId => CloudNetworkError::InvalidResourceId,
        CloudResourceError::ResourceIdTenantMismatch => CloudNetworkError::ResourceTenantMismatch,
        CloudResourceError::ResourceIdRegionMismatch => CloudNetworkError::ResourceRegionMismatch,
        CloudResourceError::ResourceIdKindMismatch => CloudNetworkError::ResourceKindMismatch,
        CloudResourceError::InvalidTenantId => CloudNetworkError::InvalidTenantId,
        _ => CloudNetworkError::InvalidResourceId,
    }
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_residency_domain::{
        PerPackResidency, PerPackResidencyCreate, RegulatorOverlay, RegulatorOverlayCreate,
    };

    fn residency_class() -> ResidencyClass {
        ResidencyClass::PerPack(Box::new(
            PerPackResidency::new(PerPackResidencyCreate {
                allowed_primary_regions: vec!["region-alpha1".to_string()],
                allowed_replica_regions: vec!["region-beta1".to_string()],
                forbidden_regions: vec!["region-gamma1".to_string()],
                regulator_overlay: RegulatorOverlay::new(RegulatorOverlayCreate {
                    regulator_refs: vec!["regulator/cloud-network".to_string()],
                    evidence_ref: "evidence/residency/cloud-network".to_string(),
                })
                .expect("regulator overlay fixture is valid"),
            })
            .expect("per-pack residency fixture is valid"),
        ))
    }

    fn route_table_create() -> RouteTableCreate {
        RouteTableCreate {
            id: "rtb_main".to_string(),
            routes: vec![
                RouteCreate {
                    destination: "10.42.0.0/16".to_string(),
                    next_hop: RouteNextHopKind::Local,
                    target_ref: None,
                },
                RouteCreate {
                    destination: "2001:db8:42::/56".to_string(),
                    next_hop: RouteNextHopKind::Local,
                    target_ref: None,
                },
            ],
        }
    }

    fn security_group_create() -> SecurityGroupCreate {
        SecurityGroupCreate {
            id: "sg_web".to_string(),
            rules: vec![SecurityRule {
                direction: RuleDirection::Ingress,
                protocol: IpProtocol::Tcp,
                port_range: Some((443, 443)),
                cidr: RouteDestination::Ipv4(Ipv4Cidr::new("10.42.0.0/16").unwrap()),
                description: "tenant https ingress".to_string(),
            }],
        }
    }

    fn vpc_create() -> VpcCreate {
        VpcCreate {
            resource_id: "oya:cloud:region-alpha1:ten_alpha:vpc:prod".to_string(),
            tenant_id: "ten_alpha".to_string(),
            region: "region-alpha1".to_string(),
            cidr_v4: "10.42.0.0/16".to_string(),
            cidr_v6: "2001:db8:42::/56".to_string(),
            flow_logs_enabled: true,
            route_table: route_table_create(),
            security_groups: vec![security_group_create()],
            residency: residency_class(),
            state: VpcState::Creating,
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_000_000,
        }
    }

    fn subnet_create() -> SubnetCreate {
        SubnetCreate {
            resource_id: "oya:cloud:region-alpha1:ten_alpha:subnet:prod-a".to_string(),
            tenant_id: "ten_alpha".to_string(),
            vpc_id: "oya:cloud:region-alpha1:ten_alpha:vpc:prod".to_string(),
            region: "region-alpha1".to_string(),
            az: "region-alpha1-a".to_string(),
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
            resource_id: "oya:cloud:region-alpha1:ten_alpha:lb-v7:frontdoor".to_string(),
            tenant_id: "ten_alpha".to_string(),
            vpc_id: "oya:cloud:region-alpha1:ten_alpha:vpc:prod".to_string(),
            region: "region-alpha1".to_string(),
            kind: LbKind::L7Grpc,
            listeners: vec![ListenerCreate {
                port: 443,
                target_group_id: "tg_api".to_string(),
                tls_certificate: Some("cert/region-alpha1/ten_alpha/frontdoor".to_string()),
            }],
            target_groups: vec![TargetGroupCreate {
                id: "tg_api".to_string(),
                subnet_ids: vec!["oya:cloud:region-alpha1:ten_alpha:subnet:prod-a".to_string()],
                health_check_path: Some("/healthz".to_string()),
            }],
            mtls: Some(MtlsConfigCreate {
                ca_bundle_ref: "cert/region-alpha1/ten_alpha/mesh-ca".to_string(),
                client_policy: MtlsClientPolicy::RequireVerifiedClientCert,
            }),
            waf_policy: Some("waf_cloud_frontdoor".to_string()),
            state: LbState::Creating,
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_000_020,
        }
    }

    fn public_dns_create() -> DnsZoneCreate {
        DnsZoneCreate {
            resource_id: "oya:cloud:region-alpha1:ten_alpha:dns-zone:example-com".to_string(),
            tenant_id: "ten_alpha".to_string(),
            region: "region-alpha1".to_string(),
            name: "example.com".to_string(),
            kind: DnsZoneKind::Public,
            vpc_id: None,
            dnssec_key_ref: Some("dnssec/region-alpha1/ten_alpha/example-com".to_string()),
            state: DnsZoneState::Creating,
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_000_030,
        }
    }

    fn seeded_catalog() -> CloudNetworkCatalog {
        let mut catalog = CloudNetworkCatalog::default();
        catalog.create_vpc(vpc_create()).expect("vpc create");
        catalog.add_subnet(subnet_create()).expect("subnet create");
        catalog
            .create_load_balancer(lb_create())
            .expect("load balancer create");
        catalog
            .create_dns_zone(public_dns_create())
            .expect("public dns create");
        catalog
    }

    fn cdn_create() -> CdnDistributionCreate {
        CdnDistributionCreate {
            resource_id: "oya:cloud:region-alpha1:ten_alpha:cdn-distribution:console".to_string(),
            tenant_id: "ten_alpha".to_string(),
            region: "region-alpha1".to_string(),
            hostnames: vec!["console.oyatie.example".to_string()],
            origins: vec![CdnOriginCreate {
                resource_id: "oya:cloud:region-alpha1:ten_alpha:lb-v7:frontdoor".to_string(),
                kind: CdnOriginKind::LoadBalancer,
            }],
            tls_certificate: "cert/region-alpha1/ten_alpha/console-edge".to_string(),
            waf_policy: "waf_console_edge".to_string(),
            cache_mode: CdnCacheMode::ConsoleAssets,
            state: CdnState::Creating,
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_000_050,
        }
    }

    fn interconnect_partner_create(id: &str, location: &str) -> InterconnectPartnerCreate {
        InterconnectPartnerCreate {
            id: id.to_string(),
            name: id.trim_start_matches("ixp_").to_ascii_uppercase(),
            region: "region-alpha1".to_string(),
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
            resource_id: "oya:cloud:region-alpha1:ten_alpha:direct-interconnect:fabric-a-primary"
                .to_string(),
            tenant_id: "ten_alpha".to_string(),
            region: "region-alpha1".to_string(),
            partner_id: "ixp_alpha".to_string(),
            peering_location: "region-alpha1-fabric-a".to_string(),
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

    fn interconnect_partner_creates() -> Vec<InterconnectPartnerCreate> {
        vec![
            interconnect_partner_create("ixp_alpha", "region-alpha1-fabric-a"),
            interconnect_partner_create("ixp_beta", "region-alpha1-fabric-b"),
        ]
    }

    fn provider_direct_interconnect_create_request()
    -> NetworkProviderDirectInterconnectCreateRequest {
        NetworkProviderDirectInterconnectCreateRequest {
            request_id: "networkprov_req_interconnect_create_001".to_string(),
            provider_virtual_circuit_ref: "oci-fast-connect://ocid1.compartment.oc1..cloud/ap-chuncheon-1/oya:cloud:region-alpha1:ten_alpha:direct-interconnect:fabric-a-primary".to_string(),
            interconnect_partners: interconnect_partner_creates(),
            direct_interconnect: direct_interconnect_create(),
            actor: "sp_network".to_string(),
            idempotency_key: "idem-network-interconnect-create".to_string(),
            requested_at_epoch_seconds: 1_700_000_060,
        }
    }

    fn ddos_create() -> DdosProtectionCreate {
        DdosProtectionCreate {
            resource_id: "oya:cloud:region-alpha1:ten_alpha:ddos-protection:frontdoor".to_string(),
            tenant_id: "ten_alpha".to_string(),
            region: "region-alpha1".to_string(),
            protected_resource_ids: vec![
                "oya:cloud:region-alpha1:ten_alpha:lb-v7:frontdoor".to_string(),
                "oya:cloud:region-alpha1:ten_alpha:cdn-distribution:console".to_string(),
            ],
            scrubbing_regions: vec!["region-alpha1".to_string(), "region-beta1".to_string()],
            line_rate_scrubbing: true,
            always_on: true,
            mitigation_runbook_ref: "runbook/network/ddos/frontdoor".to_string(),
            oncall_group_ref: "oncall/network-sre".to_string(),
            state: DdosProtectionState::Creating,
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_000_070,
        }
    }

    fn mesh_create() -> ServiceMeshCellCreate {
        ServiceMeshCellCreate {
            mesh_id: "mesh_prod_alpha".to_string(),
            tenant_id: "ten_alpha".to_string(),
            region: "region-alpha1".to_string(),
            cell_id: "cell-region-alpha1-a-001".to_string(),
            vpc_id: "oya:cloud:region-alpha1:ten_alpha:vpc:prod".to_string(),
            namespace: "mesh-cell-region-alpha1-a-001".to_string(),
            mode: ServiceMeshMode::IstioAmbient,
            edge_gateway: MeshGatewayKind::Envoy,
            mtls_everywhere: true,
            ext_authz_enabled: true,
            cross_cell_policy_ref: "cedar/network/cross-cell".to_string(),
            audit_stream_ref: "audit/network/mesh".to_string(),
            health_alarm_ref: "alarm/network/mesh-control-plane".to_string(),
            control_plane_replicas: 3,
            quarterly_upgrade_drill: true,
            state: ServiceMeshState::Creating,
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_000_080,
        }
    }

    fn provider_vpc_create_request() -> NetworkProviderVpcCreateRequest {
        NetworkProviderVpcCreateRequest {
            request_id: "networkprov_req_vpc_create_001".to_string(),
            provider_vcn_ref: "oci-vcn://ocid1.compartment.oc1..cloud/ap-chuncheon-1/oya:cloud:region-alpha1:ten_alpha:vpc:prod".to_string(),
            vpc: vpc_create(),
            actor: "sp_network".to_string(),
            idempotency_key: "idem-network-vpc-create".to_string(),
            requested_at_epoch_seconds: 1_700_000_010,
        }
    }

    fn provider_load_balancer_create_request() -> NetworkProviderLoadBalancerCreateRequest {
        NetworkProviderLoadBalancerCreateRequest {
            request_id: "networkprov_req_lb_create_001".to_string(),
            provider_load_balancer_ref: "oci-lb://ocid1.compartment.oc1..cloud/ap-chuncheon-1/oya:cloud:region-alpha1:ten_alpha:lb-v7:frontdoor".to_string(),
            vpc: vpc_create(),
            subnets: vec![subnet_create()],
            load_balancer: lb_create(),
            actor: "sp_network".to_string(),
            idempotency_key: "idem-network-lb-create".to_string(),
            requested_at_epoch_seconds: 1_700_000_030,
        }
    }

    fn provider_dns_zone_create_request() -> NetworkProviderDnsZoneCreateRequest {
        NetworkProviderDnsZoneCreateRequest {
            request_id: "networkprov_req_dns_create_001".to_string(),
            provider_dns_zone_ref: "oci-dns-zone://ocid1.compartment.oc1..cloud/ap-chuncheon-1/oya:cloud:region-alpha1:ten_alpha:dns-zone:example-com".to_string(),
            vpc: None,
            dns_zone: public_dns_create(),
            actor: "sp_network".to_string(),
            idempotency_key: "idem-network-dns-zone-create".to_string(),
            requested_at_epoch_seconds: 1_700_000_040,
        }
    }

    #[test]
    fn creates_vpc_with_ipv6_flow_logs_route_table_and_security_groups() {
        let vpc = Vpc::new(vpc_create()).expect("vpc is valid");

        assert_eq!(vpc.resource_id.value.kind_label().unwrap(), "vpc");
        assert_eq!(vpc.cidr_v4.value.value, "10.42.0.0/16");
        assert_eq!(vpc.cidr_v6.value.value, "2001:db8:42::/56");
        assert!(vpc.flow_logs_enabled.value);
        assert_eq!(vpc.route_table.value.routes.len(), 2);
        assert_eq!(vpc.security_groups.value.len(), 1);
        assert_eq!(
            vpc.route_table.data_class.compatibility_data_class(),
            DataClass::InternalOnly
        );
        assert_eq!(
            vpc.security_groups.data_class.compatibility_data_class(),
            DataClass::InternalOnly
        );
        assert_eq!(vpc.schema_version.value, NETWORK_SCHEMA_VERSION);
    }

    #[test]
    fn rejects_vpc_without_ipv6_flow_logs_or_public_metadata_class() {
        let ipv6_error = Vpc::new(VpcCreate {
            cidr_v6: "not-a-cidr".to_string(),
            ..vpc_create()
        })
        .expect_err("IPv6 is required from day one");
        assert_eq!(ipv6_error, CloudNetworkError::InvalidIpv6Cidr);

        let flow_error = Vpc::new(VpcCreate {
            flow_logs_enabled: false,
            ..vpc_create()
        })
        .expect_err("audit-grade flow logs are mandatory");
        assert_eq!(flow_error, CloudNetworkError::FlowLogsRequired);

        let class_error = Vpc::new(VpcCreate {
            data_class: DataClass::InternalOnly,
            ..vpc_create()
        })
        .expect_err("network metadata is public-only");
        assert_eq!(class_error, CloudNetworkError::InvalidDataClass);
    }

    #[test]
    fn network_provider_vpc_requests_validate_refs_shape_and_actor() {
        provider_vpc_create_request()
            .validate()
            .expect("provider VPC request is valid");

        let mut bad_provider_ref = provider_vpc_create_request();
        bad_provider_ref.provider_vcn_ref = " ".to_string();
        assert_eq!(
            bad_provider_ref.validate(),
            Err(NetworkProviderVpcError::InvalidProviderVcnRef)
        );

        let mut bad_vpc_shape = provider_vpc_create_request();
        bad_vpc_shape.vpc.flow_logs_enabled = false;
        assert_eq!(
            bad_vpc_shape.validate(),
            Err(NetworkProviderVpcError::InvalidRequestShape(
                CloudNetworkError::FlowLogsRequired,
            ))
        );

        let mut bad_actor = provider_vpc_create_request();
        bad_actor.actor = "network".to_string();
        assert_eq!(
            bad_actor.validate(),
            Err(NetworkProviderVpcError::InvalidActorRef)
        );
    }

    #[test]
    fn network_provider_vpc_receipts_keep_refs_without_provider_credentials() {
        let receipt = NetworkProviderVpcReceipt::create_vpc(
            NetworkProviderKind::OciVcn,
            provider_vpc_create_request(),
            "oci-vcn-1700000000-networkprov_req_vpc_create_001",
            "oci-vcn://ocid1.compartment.oc1..cloud/ap-chuncheon-1/oya:cloud:region-alpha1:ten_alpha:vpc:prod/networkprov_req_vpc_create_001",
        )
        .expect("VPC receipt keeps provider references only");

        assert_eq!(receipt.provider.label(), "oci_vcn");
        assert_eq!(receipt.operation.label(), "create_vpc");
        assert_eq!(
            receipt.resource_id,
            "oya:cloud:region-alpha1:ten_alpha:vpc:prod"
        );
        assert_eq!(receipt.cidr_v4, "10.42.0.0/16");
        assert_eq!(receipt.cidr_v6, "2001:db8:42::/56");
        assert!(receipt.flow_logs_enabled);
        assert_eq!(receipt.actor, "sp_network");
        assert_eq!(receipt.schema_version, NETWORK_SCHEMA_VERSION);
    }

    #[test]
    fn network_provider_load_balancer_requests_validate_context_shape_and_actor() {
        provider_load_balancer_create_request()
            .validate()
            .expect("provider load balancer request is valid");

        let mut bad_provider_ref = provider_load_balancer_create_request();
        bad_provider_ref.provider_load_balancer_ref = " ".to_string();
        assert_eq!(
            bad_provider_ref.validate(),
            Err(NetworkProviderLoadBalancerError::InvalidProviderLoadBalancerRef)
        );

        let mut bad_lb_shape = provider_load_balancer_create_request();
        bad_lb_shape.load_balancer.mtls = None;
        assert_eq!(
            bad_lb_shape.validate(),
            Err(NetworkProviderLoadBalancerError::InvalidRequestShape(
                CloudNetworkError::GrpcRequiresMtls,
            ))
        );

        let mut bad_actor = provider_load_balancer_create_request();
        bad_actor.actor = "network".to_string();
        assert_eq!(
            bad_actor.validate(),
            Err(NetworkProviderLoadBalancerError::InvalidActorRef)
        );
    }

    #[test]
    fn network_provider_load_balancer_receipts_keep_refs_without_provider_credentials() {
        let receipt = NetworkProviderLoadBalancerReceipt::create_load_balancer(
            NetworkProviderKind::OciLoadBalancer,
            provider_load_balancer_create_request(),
            "oci-lb-1700000000-networkprov_req_lb_create_001",
            "oci-lb://ocid1.compartment.oc1..cloud/ap-chuncheon-1/oya:cloud:region-alpha1:ten_alpha:lb-v7:frontdoor/networkprov_req_lb_create_001",
        )
        .expect("load balancer receipt keeps provider references only");

        assert_eq!(receipt.provider.label(), "oci_load_balancer");
        assert_eq!(receipt.operation.label(), "create_load_balancer");
        assert_eq!(
            receipt.resource_id,
            "oya:cloud:region-alpha1:ten_alpha:lb-v7:frontdoor"
        );
        assert_eq!(receipt.vpc_id, "oya:cloud:region-alpha1:ten_alpha:vpc:prod");
        assert_eq!(receipt.kind, LbKind::L7Grpc);
        assert_eq!(receipt.listener_count, 1);
        assert_eq!(receipt.target_group_count, 1);
        assert!(receipt.mtls_enabled);
        assert_eq!(receipt.actor, "sp_network");
        assert_eq!(receipt.schema_version, NETWORK_SCHEMA_VERSION);
    }

    #[test]
    fn network_provider_dns_zone_requests_validate_context_shape_and_actor() {
        provider_dns_zone_create_request()
            .validate()
            .expect("provider DNS zone request is valid");

        let mut bad_provider_ref = provider_dns_zone_create_request();
        bad_provider_ref.provider_dns_zone_ref = " ".to_string();
        assert_eq!(
            bad_provider_ref.validate(),
            Err(NetworkProviderDnsZoneError::InvalidProviderDnsZoneRef)
        );

        let mut bad_zone_shape = provider_dns_zone_create_request();
        bad_zone_shape.dns_zone.dnssec_key_ref = None;
        assert_eq!(
            bad_zone_shape.validate(),
            Err(NetworkProviderDnsZoneError::InvalidRequestShape(
                CloudNetworkError::DnssecRequired,
            ))
        );

        let mut bad_actor = provider_dns_zone_create_request();
        bad_actor.actor = "network".to_string();
        assert_eq!(
            bad_actor.validate(),
            Err(NetworkProviderDnsZoneError::InvalidActorRef)
        );
    }

    #[test]
    fn network_provider_dns_zone_receipts_keep_refs_without_provider_credentials() {
        let receipt = NetworkProviderDnsZoneReceipt::create_dns_zone(
            NetworkProviderKind::OciDnsZone,
            provider_dns_zone_create_request(),
            "oci-dns-zone-1700000000-networkprov_req_dns_create_001",
            "oci-dns-zone://ocid1.compartment.oc1..cloud/ap-chuncheon-1/oya:cloud:region-alpha1:ten_alpha:dns-zone:example-com/networkprov_req_dns_create_001",
        )
        .expect("DNS zone receipt keeps provider references only");

        assert_eq!(receipt.provider.label(), "oci_dns_zone");
        assert_eq!(receipt.operation.label(), "create_dns_zone");
        assert_eq!(
            receipt.resource_id,
            "oya:cloud:region-alpha1:ten_alpha:dns-zone:example-com"
        );
        assert_eq!(receipt.name, "example.com");
        assert_eq!(receipt.kind, DnsZoneKind::Public);
        assert_eq!(receipt.vpc_id, None);
        assert!(receipt.dnssec_enabled);
        assert_eq!(receipt.actor, "sp_network");
        assert_eq!(receipt.schema_version, NETWORK_SCHEMA_VERSION);
    }

    #[test]
    fn network_provider_direct_interconnect_requests_validate_context_shape_and_actor() {
        provider_direct_interconnect_create_request()
            .validate()
            .expect("provider direct interconnect request is valid");

        let mut bad_provider_ref = provider_direct_interconnect_create_request();
        bad_provider_ref.provider_virtual_circuit_ref = " ".to_string();
        assert_eq!(
            bad_provider_ref.validate(),
            Err(NetworkProviderDirectInterconnectError::InvalidProviderVirtualCircuitRef)
        );

        let mut bad_shape = provider_direct_interconnect_create_request();
        bad_shape.direct_interconnect.redundant_port_count = 1;
        assert_eq!(
            bad_shape.validate(),
            Err(NetworkProviderDirectInterconnectError::InvalidRequestShape(
                CloudNetworkError::InterconnectRedundancyRequired,
            ))
        );

        let mut bad_actor = provider_direct_interconnect_create_request();
        bad_actor.actor = "network".to_string();
        assert_eq!(
            bad_actor.validate(),
            Err(NetworkProviderDirectInterconnectError::InvalidActorRef)
        );
    }

    #[test]
    fn network_provider_direct_interconnect_receipts_keep_refs_without_provider_credentials() {
        let receipt = NetworkProviderDirectInterconnectReceipt::create_direct_interconnect(
            NetworkProviderKind::OciFastConnect,
            provider_direct_interconnect_create_request(),
            "oci-fast-connect-1700000000-networkprov_req_interconnect_create_001",
            "oci-fast-connect://ocid1.compartment.oc1..cloud/ap-chuncheon-1/oya:cloud:region-alpha1:ten_alpha:direct-interconnect:fabric-a-primary/networkprov_req_interconnect_create_001",
        )
        .expect("direct interconnect receipt keeps provider references only");

        assert_eq!(receipt.provider.label(), "oci_fast_connect");
        assert_eq!(receipt.operation.label(), "create_direct_interconnect");
        assert_eq!(
            receipt.resource_id,
            "oya:cloud:region-alpha1:ten_alpha:direct-interconnect:fabric-a-primary"
        );
        assert_eq!(receipt.partner_id, "ixp_alpha");
        assert_eq!(receipt.peering_location, "region-alpha1-fabric-a");
        assert_eq!(receipt.physical_port_id, "icp_alpha_001");
        assert_eq!(receipt.vlan_tag, 101);
        assert_eq!(receipt.bandwidth_mbps, 10_000);
        assert_eq!(receipt.redundant_port_count, 2);
        assert_eq!(receipt.bgp_session_count, 2);
        assert_eq!(receipt.advertised_prefix_count, 2);
        assert_eq!(receipt.actor, "sp_network");
        assert_eq!(receipt.schema_version, NETWORK_SCHEMA_VERSION);
    }

    #[test]
    fn create_contracts_reject_caller_forged_runtime_state() {
        let vpc_error = Vpc::new(VpcCreate {
            state: VpcState::Active,
            ..vpc_create()
        })
        .expect_err("vpc create begins in Creating");
        assert_eq!(vpc_error, CloudNetworkError::InvalidVpcState);

        let vpc = Vpc::new(vpc_create()).expect("vpc fixture is valid");
        let subnet_error = Subnet::new(
            &vpc,
            SubnetCreate {
                state: SubnetState::Active,
                ..subnet_create()
            },
        )
        .expect_err("subnet create begins in Creating");
        assert_eq!(subnet_error, CloudNetworkError::InvalidSubnetState);

        let lb_error = LoadBalancer::new(
            &vpc,
            &BTreeMap::new(),
            LoadBalancerCreate {
                state: LbState::Active,
                ..lb_create()
            },
        )
        .expect_err("load-balancer create begins in Creating");
        assert_eq!(lb_error, CloudNetworkError::InvalidLbState);

        let zone_error = DnsZone::new(
            None,
            DnsZoneCreate {
                resource_id: "oya:cloud:region-alpha1:ten_alpha:dns-zone:example-com".to_string(),
                tenant_id: "ten_alpha".to_string(),
                region: "region-alpha1".to_string(),
                name: "example.com".to_string(),
                kind: DnsZoneKind::Public,
                vpc_id: None,
                dnssec_key_ref: Some("dnssec/region-alpha1/ten_alpha/example-com".to_string()),
                state: DnsZoneState::Active,
                data_class: DataClass::Public,
                created_at_epoch_seconds: 1_700_000_030,
            },
        )
        .expect_err("dns-zone create begins in Creating");
        assert_eq!(zone_error, CloudNetworkError::InvalidDnsZoneState);
    }

    #[test]
    fn creates_subnet_only_inside_parent_vpc_and_region_az() {
        let vpc = Vpc::new(vpc_create()).expect("vpc is valid");
        let subnet = Subnet::new(&vpc, subnet_create()).expect("subnet is valid");

        assert_eq!(subnet.resource_id.value.kind_label().unwrap(), "subnet");
        assert_eq!(subnet.az.value.value, "region-alpha1-a");
        assert_eq!(subnet.cidr_v4.value.value, "10.42.1.0/24");

        let outside = Subnet::new(
            &vpc,
            SubnetCreate {
                cidr_v4: "10.99.1.0/24".to_string(),
                ..subnet_create()
            },
        )
        .expect_err("subnet CIDR must be inside VPC CIDR");
        assert_eq!(outside, CloudNetworkError::SubnetOutsideVpc);

        let az_error = Subnet::new(
            &vpc,
            SubnetCreate {
                az: "region-gamma1-a".to_string(),
                ..subnet_create()
            },
        )
        .expect_err("subnet AZ must belong to region");
        assert_eq!(az_error, CloudNetworkError::AzRegionMismatch);
    }

    #[test]
    fn catalog_rejects_overlapping_subnets_per_vpc() {
        let mut catalog = CloudNetworkCatalog::default();
        catalog.create_vpc(vpc_create()).expect("vpc create");
        catalog.add_subnet(subnet_create()).expect("first subnet");

        let overlap = catalog
            .add_subnet(SubnetCreate {
                resource_id: "oya:cloud:region-alpha1:ten_alpha:subnet:prod-a-overlap".to_string(),
                cidr_v4: "10.42.1.128/25".to_string(),
                cidr_v6: "2001:db8:42:1::/65".to_string(),
                ..subnet_create()
            })
            .expect_err("same VPC CIDR ranges must not overlap");
        assert_eq!(overlap, CloudNetworkError::OverlappingSubnet);

        let adjacent = catalog
            .add_subnet(SubnetCreate {
                resource_id: "oya:cloud:region-alpha1:ten_alpha:subnet:prod-b".to_string(),
                az: "region-alpha1-b".to_string(),
                cidr_v4: "10.42.2.0/24".to_string(),
                cidr_v6: "2001:db8:42:2::/64".to_string(),
                ..subnet_create()
            })
            .expect("non-overlapping same-VPC subnet is valid");
        assert_eq!(adjacent.cidr_v4.value.value, "10.42.2.0/24");
    }

    #[test]
    fn creates_l7_grpc_load_balancer_with_tls_mtls_and_known_target_group() {
        let mut catalog = CloudNetworkCatalog::default();
        catalog.create_vpc(vpc_create()).expect("vpc create");
        catalog.add_subnet(subnet_create()).expect("subnet create");
        let lb = catalog
            .create_load_balancer(lb_create())
            .expect("load balancer is valid");

        assert_eq!(lb.resource_id.value.kind_label().unwrap(), "lb-v7");
        assert_eq!(lb.listeners.value[0].port, 443);
        assert!(lb.mtls.value.is_some());
        assert_eq!(lb.target_groups.value[0].subnet_ids.len(), 1);
        assert_eq!(
            lb.listeners.data_class.compatibility_data_class(),
            DataClass::InternalOnly
        );
        assert_eq!(
            lb.target_groups.data_class.compatibility_data_class(),
            DataClass::InternalOnly
        );
        assert_eq!(
            lb.mtls.data_class.compatibility_data_class(),
            DataClass::InternalOnly
        );
    }

    #[test]
    fn rejects_l7_without_tls_grpc_without_mtls_and_unknown_subnet_targets() {
        let mut catalog = CloudNetworkCatalog::default();
        catalog.create_vpc(vpc_create()).expect("vpc create");
        catalog.add_subnet(subnet_create()).expect("subnet create");

        let tls_error = catalog
            .create_load_balancer(LoadBalancerCreate {
                listeners: vec![ListenerCreate {
                    port: 443,
                    target_group_id: "tg_api".to_string(),
                    tls_certificate: None,
                }],
                ..lb_create()
            })
            .expect_err("L7 listeners need TLS certificates");
        assert_eq!(tls_error, CloudNetworkError::L7RequiresTls);

        let mtls_error = catalog
            .create_load_balancer(LoadBalancerCreate {
                mtls: None,
                ..lb_create()
            })
            .expect_err("gRPC front doors require mTLS config");
        assert_eq!(mtls_error, CloudNetworkError::GrpcRequiresMtls);

        let subnet_error = catalog
            .create_load_balancer(LoadBalancerCreate {
                target_groups: vec![TargetGroupCreate {
                    id: "tg_api".to_string(),
                    subnet_ids: vec![
                        "oya:cloud:region-alpha1:ten_alpha:subnet:missing".to_string(),
                    ],
                    health_check_path: Some("/healthz".to_string()),
                }],
                ..lb_create()
            })
            .expect_err("LB target subnets must exist");
        assert_eq!(subnet_error, CloudNetworkError::UnknownSubnet);
    }

    #[test]
    fn creates_dns_zones_with_dnssec_and_private_zone_vpc_binding() {
        let mut catalog = CloudNetworkCatalog::default();
        catalog.create_vpc(vpc_create()).expect("vpc create");
        let public = catalog
            .create_dns_zone(DnsZoneCreate {
                resource_id: "oya:cloud:region-alpha1:ten_alpha:dns-zone:example-com".to_string(),
                tenant_id: "ten_alpha".to_string(),
                region: "region-alpha1".to_string(),
                name: "example.com".to_string(),
                kind: DnsZoneKind::Public,
                vpc_id: None,
                dnssec_key_ref: Some("dnssec/region-alpha1/ten_alpha/example-com".to_string()),
                state: DnsZoneState::Creating,
                data_class: DataClass::Public,
                created_at_epoch_seconds: 1_700_000_030,
            })
            .expect("public zone is valid");
        assert_eq!(public.name.value.value, "example.com");

        let private = catalog
            .create_dns_zone(DnsZoneCreate {
                resource_id: "oya:cloud:region-alpha1:ten_alpha:dns-zone:internal-example"
                    .to_string(),
                tenant_id: "ten_alpha".to_string(),
                region: "region-alpha1".to_string(),
                name: "internal.example".to_string(),
                kind: DnsZoneKind::Private,
                vpc_id: Some("oya:cloud:region-alpha1:ten_alpha:vpc:prod".to_string()),
                dnssec_key_ref: None,
                state: DnsZoneState::Creating,
                data_class: DataClass::Public,
                created_at_epoch_seconds: 1_700_000_031,
            })
            .expect("private zone binds known vpc");
        assert!(private.vpc_id.value.is_some());
    }

    #[test]
    fn rejects_public_dns_without_dnssec_and_private_dns_without_vpc() {
        let dnssec_error = DnsZone::new(
            None,
            DnsZoneCreate {
                resource_id: "oya:cloud:region-alpha1:ten_alpha:dns-zone:example-com".to_string(),
                tenant_id: "ten_alpha".to_string(),
                region: "region-alpha1".to_string(),
                name: "example.com".to_string(),
                kind: DnsZoneKind::Public,
                vpc_id: None,
                dnssec_key_ref: None,
                state: DnsZoneState::Creating,
                data_class: DataClass::Public,
                created_at_epoch_seconds: 1_700_000_030,
            },
        )
        .expect_err("public zones require DNSSEC");
        assert_eq!(dnssec_error, CloudNetworkError::DnssecRequired);

        let private_error = DnsZone::new(
            None,
            DnsZoneCreate {
                resource_id: "oya:cloud:region-alpha1:ten_alpha:dns-zone:internal".to_string(),
                tenant_id: "ten_alpha".to_string(),
                region: "region-alpha1".to_string(),
                name: "internal.example".to_string(),
                kind: DnsZoneKind::Private,
                vpc_id: None,
                dnssec_key_ref: None,
                state: DnsZoneState::Creating,
                data_class: DataClass::Public,
                created_at_epoch_seconds: 1_700_000_031,
            },
        )
        .expect_err("private zones require VPC binding");
        assert_eq!(private_error, CloudNetworkError::PrivateZoneRequiresVpc);
    }

    #[test]
    fn records_flow_anomaly_only_for_flow_logged_vpc() {
        let mut catalog = CloudNetworkCatalog::default();
        catalog.create_vpc(vpc_create()).expect("vpc create");
        let event = catalog
            .record_flow_anomaly(
                "flowanom_001".to_string(),
                "oya:cloud:region-alpha1:ten_alpha:vpc:prod".to_string(),
                FlowAnomalySeverity::High,
                "egress spike to undeclared cidr".to_string(),
                1_700_000_040,
            )
            .expect("flow anomaly event is valid");
        assert_eq!(event.severity.value, FlowAnomalySeverity::High);
        assert_eq!(catalog.anomalies().count(), 1);

        let duplicate = catalog
            .record_flow_anomaly(
                "flowanom_001".to_string(),
                "oya:cloud:region-alpha1:ten_alpha:vpc:prod".to_string(),
                FlowAnomalySeverity::Critical,
                "duplicate evidence id".to_string(),
                1_700_000_041,
            )
            .expect_err("flow anomaly IDs are immutable evidence keys");
        assert_eq!(duplicate, CloudNetworkError::DuplicateFlowAnomaly);
    }

    #[test]
    fn creates_cdn_distribution_with_tls_waf_and_known_origin() {
        let mut catalog = seeded_catalog();
        let distribution = catalog
            .create_cdn_distribution(cdn_create())
            .expect("cdn distribution is valid");

        assert_eq!(
            distribution.resource_id.value.kind_label().unwrap(),
            "cdn-distribution"
        );
        assert_eq!(
            distribution.hostnames.value[0].value,
            "console.oyatie.example"
        );
        assert_eq!(
            distribution.origins.value[0].kind,
            CdnOriginKind::LoadBalancer
        );
        assert_eq!(
            distribution.origins.data_class.compatibility_data_class(),
            DataClass::InternalOnly
        );

        let duplicate_hostname = catalog
            .create_cdn_distribution(CdnDistributionCreate {
                resource_id: "oya:cloud:region-alpha1:ten_alpha:cdn-distribution:dup-host"
                    .to_string(),
                hostnames: vec![
                    "console.oyatie.example".to_string(),
                    "Console.Oyatie.Example.".to_string(),
                ],
                ..cdn_create()
            })
            .expect_err("hostnames must be unique after normalization");
        assert_eq!(duplicate_hostname, CloudNetworkError::DuplicateCdnHostname);
    }

    #[test]
    fn rejects_cdn_without_waf_tls_known_origin_or_create_state() {
        let mut catalog = seeded_catalog();

        let waf_error = catalog
            .create_cdn_distribution(CdnDistributionCreate {
                waf_policy: String::new(),
                ..cdn_create()
            })
            .expect_err("cdn must bind a WAF policy");
        assert_eq!(waf_error, CloudNetworkError::CdnWafRequired);

        let tls_error = catalog
            .create_cdn_distribution(CdnDistributionCreate {
                tls_certificate: String::new(),
                ..cdn_create()
            })
            .expect_err("cdn must bind an edge certificate");
        assert_eq!(tls_error, CloudNetworkError::CdnTlsRequired);

        let origin_error = catalog
            .create_cdn_distribution(CdnDistributionCreate {
                origins: vec![CdnOriginCreate {
                    resource_id: "oya:cloud:region-alpha1:ten_alpha:lb-v7:missing".to_string(),
                    kind: CdnOriginKind::LoadBalancer,
                }],
                ..cdn_create()
            })
            .expect_err("cdn origins must be known network resources");
        assert_eq!(origin_error, CloudNetworkError::UnknownLoadBalancer);

        let state_error = catalog
            .create_cdn_distribution(CdnDistributionCreate {
                state: CdnState::Active,
                ..cdn_create()
            })
            .expect_err("cdn create begins in Creating");
        assert_eq!(state_error, CloudNetworkError::InvalidCdnState);
    }

    #[test]
    fn creates_direct_interconnect_with_multi_ixp_bgp_and_sla() {
        let mut catalog = CloudNetworkCatalog::default();
        catalog
            .add_interconnect_partner(interconnect_partner_create(
                "ixp_alpha",
                "region-alpha1-fabric-a",
            ))
            .expect("primary interconnect partner");
        let diversity_error = catalog
            .create_direct_interconnect(direct_interconnect_create())
            .expect_err("major regions require at least two interconnect partners");
        assert_eq!(
            diversity_error,
            CloudNetworkError::RegionalInterconnectDiversityRequired
        );

        catalog
            .add_interconnect_partner(interconnect_partner_create(
                "ixp_beta",
                "region-alpha1-fabric-b",
            ))
            .expect("second interconnect partner");
        let interconnect = catalog
            .create_direct_interconnect(direct_interconnect_create())
            .expect("direct interconnect is valid");

        assert_eq!(
            interconnect.resource_id.value.kind_label().unwrap(),
            "direct-interconnect"
        );
        assert_eq!(interconnect.bgp_sessions.value.len(), 2);
        assert_eq!(interconnect.per_link_sla_basis_points.value, 9_999);
    }

    #[test]
    fn rejects_direct_interconnect_without_redundant_bgp_vlan_or_sla() {
        let mut catalog = CloudNetworkCatalog::default();
        catalog
            .add_interconnect_partner(interconnect_partner_create(
                "ixp_alpha",
                "region-alpha1-fabric-a",
            ))
            .expect("primary interconnect partner");
        catalog
            .add_interconnect_partner(interconnect_partner_create(
                "ixp_beta",
                "region-alpha1-fabric-b",
            ))
            .expect("second interconnect partner");

        let bgp_error = catalog
            .create_direct_interconnect(DirectInterconnectCreate {
                bgp_sessions: vec![bgp_session_create(
                    "bgp_alpha_1",
                    "169.254.10.1",
                    "169.254.10.2",
                )],
                ..direct_interconnect_create()
            })
            .expect_err("interconnect needs redundant BGP sessions");
        assert_eq!(bgp_error, CloudNetworkError::InterconnectRedundancyRequired);

        let vlan_error = catalog
            .create_direct_interconnect(DirectInterconnectCreate {
                vlan_tag: 0,
                ..direct_interconnect_create()
            })
            .expect_err("802.1Q VLAN tag must be in range");
        assert_eq!(vlan_error, CloudNetworkError::InvalidVlanTag);

        let sla_error = catalog
            .create_direct_interconnect(DirectInterconnectCreate {
                per_link_sla_basis_points: 9_990,
                ..direct_interconnect_create()
            })
            .expect_err("direct interconnect requires 99.99% per-link SLA");
        assert_eq!(sla_error, CloudNetworkError::InterconnectSlaRequired);
    }

    #[test]
    fn creates_ddos_protection_for_known_edge_resources() {
        let mut catalog = seeded_catalog();
        catalog
            .create_cdn_distribution(cdn_create())
            .expect("cdn distribution");
        let protection = catalog
            .create_ddos_protection(ddos_create())
            .expect("ddos protection is valid");

        assert_eq!(
            protection.resource_id.value.kind_label().unwrap(),
            "ddos-protection"
        );
        assert_eq!(protection.protected_resources.value.len(), 2);
        assert!(protection.line_rate_scrubbing.value);
        assert!(protection.always_on.value);
    }

    #[test]
    fn rejects_ddos_without_known_resources_scrubbing_or_always_on_posture() {
        let mut catalog = seeded_catalog();
        catalog
            .create_cdn_distribution(cdn_create())
            .expect("cdn distribution");

        let resource_error = catalog
            .create_ddos_protection(DdosProtectionCreate {
                protected_resource_ids: vec![
                    "oya:cloud:region-alpha1:ten_alpha:cdn-distribution:missing".to_string(),
                ],
                ..ddos_create()
            })
            .expect_err("ddos binds only known protected resources");
        assert_eq!(resource_error, CloudNetworkError::UnknownProtectedResource);

        let always_on_error = catalog
            .create_ddos_protection(DdosProtectionCreate {
                always_on: false,
                ..ddos_create()
            })
            .expect_err("ddos protection must be always-on");
        assert_eq!(always_on_error, CloudNetworkError::DdosAlwaysOnRequired);

        let scrubbing_error = catalog
            .create_ddos_protection(DdosProtectionCreate {
                scrubbing_regions: vec!["region-beta1".to_string()],
                ..ddos_create()
            })
            .expect_err("home region must be in scrubbing set");
        assert_eq!(scrubbing_error, CloudNetworkError::ScrubbingRegionRequired);
    }

    #[test]
    fn creates_service_mesh_cell_with_ambient_envoy_policy_and_audit_chain() {
        let mut catalog = CloudNetworkCatalog::default();
        catalog.create_vpc(vpc_create()).expect("vpc create");
        let mesh = catalog
            .create_service_mesh_cell(mesh_create())
            .expect("service mesh cell is valid");

        assert_eq!(mesh.mode.value, ServiceMeshMode::IstioAmbient);
        assert_eq!(mesh.edge_gateway.value, MeshGatewayKind::Envoy);
        assert!(mesh.mtls_everywhere.value);
        assert!(mesh.ext_authz_enabled.value);
        assert_eq!(mesh.control_plane_replicas.value, 3);
        assert_eq!(
            mesh.cross_cell_policy_ref
                .data_class
                .compatibility_data_class(),
            DataClass::InternalOnly
        );
    }

    #[test]
    fn rejects_service_mesh_without_ambient_envoy_mtls_policy_or_cell_scope() {
        let mut catalog = CloudNetworkCatalog::default();
        catalog.create_vpc(vpc_create()).expect("vpc create");

        let mode_error = catalog
            .create_service_mesh_cell(ServiceMeshCellCreate {
                mode: ServiceMeshMode::Sidecar,
                ..mesh_create()
            })
            .expect_err("mesh mode is fixed to ambient");
        assert_eq!(mode_error, CloudNetworkError::InvalidMeshMode);

        let gateway_error = catalog
            .create_service_mesh_cell(ServiceMeshCellCreate {
                edge_gateway: MeshGatewayKind::Nginx,
                ..mesh_create()
            })
            .expect_err("edge gateway is fixed to Envoy");
        assert_eq!(gateway_error, CloudNetworkError::InvalidMeshGateway);

        let mtls_error = catalog
            .create_service_mesh_cell(ServiceMeshCellCreate {
                mtls_everywhere: false,
                ..mesh_create()
            })
            .expect_err("mesh requires mTLS everywhere");
        assert_eq!(mtls_error, CloudNetworkError::MeshMtlsRequired);

        let cell_error = catalog
            .create_service_mesh_cell(ServiceMeshCellCreate {
                cell_id: "cell-region-gamma1-a-001".to_string(),
                namespace: "mesh-cell-region-gamma1-a-001".to_string(),
                ..mesh_create()
            })
            .expect_err("mesh cell must belong to region");
        assert_eq!(cell_error, CloudNetworkError::InvalidCellId);
    }
}
