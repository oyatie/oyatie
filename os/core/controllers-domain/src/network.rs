//! Network operator COSI bridge.
//!
//! Mirrors Talos' network operator controllers at the COSI boundary: an
//! [`OperatorSpecResource`] declares a DHCP/static operator on a link, an
//! [`OperatorResultResource`] carries the deterministic lease/result observed by
//! that operator, and [`OperatorResultBridgeController`] materializes the operator's
//! produced [`AddressSpecResource`], [`RouteSpecResource`],
//! [`HostnameSpecResource`] and [`ResolverSpecResource`] outputs in the
//! intermediate `network-config` namespace.
//!
//! The pure network translation remains in `talos-network`; this module only
//! gives those specs COSI metadata, lifecycle cleanup and runtime wiring.

use crate::config::MachineConfigDocument;
use crate::reconcile::{
    Controller, Input, Output, ReconcileContext, ReconcileError, ReconcileResult,
};
use std::collections::{BTreeMap, BTreeSet};
use os_kernel::Result as CoreResult;
use os_kernel::{NodeAddress, ResourceId};
use os_cosi_domain::resource::ResourceKind;
use os_cosi_domain::{Metadata, Resource};
use os_machine_config_domain::yaml::Yaml;
use os_machine_config_domain::{
    DhcpOptions, DhcpV4ClientIdentifier, DhcpV6ClientIdentifier, LinkFields, LinkRouteConfig,
    ResolverDnsProtocol, VlanMode, dhcpv4_configs, dhcpv6_configs, link_configs,
    load_from_bytes_with, resolver_config, vlan_configs,
};
use os_machine_config_domain::{KindSpec, Registry, decode_documents};
use os_network_domain::{
    AddressFamily, AddressFlags, AddressSpec, BondMode, ClientIdentifierSpec, ConfigLayer,
    HostnameSpec, LinkKind, LinkSpec, LinkStatus, LinkType, OperatorKind, OperatorResult,
    OperatorSpec, ResolverSpec, RouteProtocol, RouteSpec, RouteTable, Scope, VlanProtocol,
    merge_addresses, merge_hostname, merge_links, merge_operators, merge_routes, vlan_link_name,
};

const NETWORK_NS: &str = "network";
const NETWORK_CONFIG_NS: &str = "network-config";
const OPERATOR_SPEC_KIND: &str = "OperatorSpecs.net.talos.dev";
const OPERATOR_RESULT_KIND: &str = "OperatorResults.net.talos.dev";
const ADDRESS_SPEC_KIND: &str = "AddressSpecs.net.talos.dev";
const ROUTE_SPEC_KIND: &str = "RouteSpecs.net.talos.dev";
const LINK_SPEC_KIND: &str = "LinkSpecs.net.talos.dev";
const LINK_STATUS_KIND: &str = "LinkStatuses.net.talos.dev";
const HOSTNAME_SPEC_KIND: &str = "HostnameSpecs.net.talos.dev";
const RESOLVER_SPEC_KIND: &str = "ResolverSpecs.net.talos.dev";
const LAYER2_VIP_CONFIG_KIND: &str = "Layer2VIPConfig";

/// Desired operator config as a COSI resource.
#[derive(Debug, Clone)]
pub struct OperatorSpecResource {
    meta: Metadata,
    /// Operator configuration.
    pub spec: OperatorSpec,
}

impl OperatorSpecResource {
    /// Build a resource using the operator's stable `<kind>/<link>` id.
    pub fn new(spec: OperatorSpec) -> Self {
        let id = spec.id();
        OperatorSpecResource {
            meta: Metadata::new(NETWORK_NS, OPERATOR_SPEC_KIND, ResourceId::new(id).unwrap()),
            spec,
        }
    }

    /// Resource kind descriptor.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(NETWORK_NS, OPERATOR_SPEC_KIND)
    }
}

/// Source-layer operator config before merge.
///
/// Mirrors Talos' `network-config/OperatorSpecs.net.talos.dev/<layer>/<operator>`
/// resources written by `network.OperatorConfigController`. A later merge pass
/// collapses these layer-qualified resources into final `network` namespace
/// [`OperatorSpecResource`] values.
#[derive(Debug, Clone)]
pub struct OperatorConfigSpecResource {
    meta: Metadata,
    /// Operator configuration.
    pub spec: OperatorSpec,
}

impl OperatorConfigSpecResource {
    /// Build a source-layer resource using `LayeredID(layer, OperatorID(spec))`.
    pub fn new(spec: OperatorSpec) -> Self {
        let id = layered_operator_id(&spec);
        OperatorConfigSpecResource {
            meta: Metadata::new(
                NETWORK_CONFIG_NS,
                OPERATOR_SPEC_KIND,
                ResourceId::new(id).unwrap(),
            ),
            spec,
        }
    }

    /// Resource kind descriptor.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(NETWORK_CONFIG_NS, OPERATOR_SPEC_KIND)
    }
}

impl Resource for OperatorConfigSpecResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        operator_spec_fingerprint(&self.spec)
    }

    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

impl Resource for OperatorSpecResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        operator_spec_fingerprint(&self.spec)
    }

    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// Source-layer link config before merge.
///
/// Mirrors Talos' `network-config/LinkSpecs.net.talos.dev/<source>`
/// resources written by `network.LinkConfigController`.
#[derive(Debug, Clone)]
pub struct LinkSpecResource {
    meta: Metadata,
    /// Source document or producer that emitted this spec.
    pub source: String,
    /// Link configuration.
    pub spec: LinkSpec,
}

impl LinkSpecResource {
    /// Build a source-layer link resource using the deterministic source id.
    pub fn new(source: impl Into<String>, spec: LinkSpec) -> Self {
        let source = source.into();
        let id = layered_link_id(&source, &spec);
        LinkSpecResource {
            meta: Metadata::new(
                NETWORK_CONFIG_NS,
                LINK_SPEC_KIND,
                ResourceId::new(id).unwrap(),
            ),
            source,
            spec,
        }
    }

    /// Resource kind descriptor.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(NETWORK_CONFIG_NS, LINK_SPEC_KIND)
    }
}

impl Resource for LinkSpecResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        format!(
            "source={};{}",
            self.source,
            link_spec_fingerprint(&self.spec)
        )
    }

    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// Observed link status from the kernel/network namespace.
///
/// Mirrors Talos `network/LinkStatuses.net.talos.dev/<link>` inputs consumed by
/// the link and operator config controllers for source-shaped default DHCP.
#[derive(Debug, Clone)]
pub struct LinkStatusResource {
    meta: Metadata,
    /// Observed link status.
    pub status: LinkStatus,
}

impl LinkStatusResource {
    /// Build a link status resource using the link name as the resource ID.
    pub fn new(status: LinkStatus) -> CoreResult<Self> {
        let id = status.name.clone();
        let id = ResourceId::new(id).map_err(|e| {
            os_kernel::Error::invalid(format!(
                "link status name '{}' is not a valid resource id: {e}",
                status.name
            ))
        })?;
        Ok(LinkStatusResource {
            meta: Metadata::new(NETWORK_NS, LINK_STATUS_KIND, id),
            status,
        })
    }

    /// Resource kind descriptor.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(NETWORK_NS, LINK_STATUS_KIND)
    }
}

impl Resource for LinkStatusResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        link_status_fingerprint(&self.status)
    }

    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// Final merged link spec consumed from the `network` namespace.
#[derive(Debug, Clone)]
pub struct MergedLinkSpecResource {
    meta: Metadata,
    /// Merged link spec.
    pub spec: LinkSpec,
}

impl MergedLinkSpecResource {
    /// Build a final link spec resource using the spec's logical id.
    pub fn new(spec: LinkSpec) -> Self {
        let id = spec.id();
        MergedLinkSpecResource {
            meta: Metadata::new(NETWORK_NS, LINK_SPEC_KIND, ResourceId::new(id).unwrap()),
            spec,
        }
    }

    /// Resource kind descriptor.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(NETWORK_NS, LINK_SPEC_KIND)
    }
}

impl Resource for MergedLinkSpecResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        link_spec_fingerprint(&self.spec)
    }

    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// Observed operator lease/result as a COSI resource.
#[derive(Debug, Clone)]
pub struct OperatorResultResource {
    meta: Metadata,
    /// Lease/result produced by the operator.
    pub result: OperatorResult,
}

impl OperatorResultResource {
    /// Build a result resource for `operator_id` (the matching
    /// [`OperatorSpec::id`]).
    pub fn new(operator_id: impl Into<String>, result: OperatorResult) -> Self {
        OperatorResultResource {
            meta: Metadata::new(
                NETWORK_NS,
                OPERATOR_RESULT_KIND,
                ResourceId::new(operator_id.into()).unwrap(),
            ),
            result,
        }
    }

    /// Resource kind descriptor.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(NETWORK_NS, OPERATOR_RESULT_KIND)
    }
}

impl Resource for OperatorResultResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        format!(
            "address={};prefix={};gateway={};dns={};hostname={};search={}",
            self.result.address,
            self.result.prefix_len,
            option_address(self.result.gateway).unwrap_or_default(),
            join_addresses(&self.result.dns_servers),
            self.result.hostname.as_deref().unwrap_or_default(),
            self.result.search_domains.join(","),
        )
    }

    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// Address output produced by a network operator.
#[derive(Debug, Clone)]
pub struct AddressSpecResource {
    meta: Metadata,
    /// Operator id that produced this spec.
    pub source_operator: String,
    /// Address spec.
    pub spec: AddressSpec,
}

impl AddressSpecResource {
    /// Build a COSI resource for an address spec.
    pub fn new(source_operator: impl Into<String>, spec: AddressSpec) -> Self {
        let source_operator = source_operator.into();
        let id = format!("{source_operator}/{}", spec.id());
        AddressSpecResource {
            meta: Metadata::new(
                NETWORK_CONFIG_NS,
                ADDRESS_SPEC_KIND,
                ResourceId::new(id).unwrap(),
            ),
            source_operator,
            spec,
        }
    }

    /// Resource kind descriptor.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(NETWORK_CONFIG_NS, ADDRESS_SPEC_KIND)
    }
}

/// Final merged address spec consumed from the `network` namespace.
#[derive(Debug, Clone)]
pub struct MergedAddressSpecResource {
    meta: Metadata,
    /// Merged address spec.
    pub spec: AddressSpec,
}

impl MergedAddressSpecResource {
    /// Build a final address spec resource using the spec's logical id.
    pub fn new(spec: AddressSpec) -> Self {
        let id = spec.id();
        MergedAddressSpecResource {
            meta: Metadata::new(NETWORK_NS, ADDRESS_SPEC_KIND, ResourceId::new(id).unwrap()),
            spec,
        }
    }

    /// Resource kind descriptor.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(NETWORK_NS, ADDRESS_SPEC_KIND)
    }
}

impl Resource for MergedAddressSpecResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        address_spec_fingerprint(&self.spec)
    }

    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

impl Resource for AddressSpecResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        format!(
            "source={};{}",
            self.source_operator,
            address_spec_fingerprint(&self.spec),
        )
    }

    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// Route output produced by a network operator.
#[derive(Debug, Clone)]
pub struct RouteSpecResource {
    meta: Metadata,
    /// Operator id that produced this spec.
    pub source_operator: String,
    /// Route spec.
    pub spec: RouteSpec,
}

impl RouteSpecResource {
    /// Build a COSI resource for a route spec.
    pub fn new(source_operator: impl Into<String>, spec: RouteSpec) -> Self {
        let source_operator = source_operator.into();
        let id = operator_route_resource_id(&source_operator, &spec);
        RouteSpecResource {
            meta: Metadata::new(
                NETWORK_CONFIG_NS,
                ROUTE_SPEC_KIND,
                ResourceId::new(id).unwrap(),
            ),
            source_operator,
            spec,
        }
    }

    /// Resource kind descriptor.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(NETWORK_CONFIG_NS, ROUTE_SPEC_KIND)
    }
}

/// Final merged route spec consumed from the `network` namespace.
#[derive(Debug, Clone)]
pub struct MergedRouteSpecResource {
    meta: Metadata,
    /// Merged route spec.
    pub spec: RouteSpec,
}

impl MergedRouteSpecResource {
    /// Build a final route spec resource using the spec's logical id.
    pub fn new(spec: RouteSpec) -> Self {
        let id = spec.id();
        MergedRouteSpecResource {
            meta: Metadata::new(NETWORK_NS, ROUTE_SPEC_KIND, ResourceId::new(id).unwrap()),
            spec,
        }
    }

    /// Resource kind descriptor.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(NETWORK_NS, ROUTE_SPEC_KIND)
    }
}

impl Resource for MergedRouteSpecResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        route_spec_fingerprint(&self.spec)
    }

    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

impl Resource for RouteSpecResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        format!(
            "source={};{}",
            self.source_operator,
            route_spec_fingerprint(&self.spec),
        )
    }

    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// Hostname output produced by a network operator.
#[derive(Debug, Clone)]
pub struct HostnameSpecResource {
    meta: Metadata,
    /// Operator id that produced this spec.
    pub source_operator: String,
    /// Hostname spec.
    pub spec: HostnameSpec,
}

impl HostnameSpecResource {
    /// Build a COSI resource for a hostname spec.
    pub fn new(source_operator: impl Into<String>, spec: HostnameSpec) -> Self {
        let source_operator = source_operator.into();
        let id = format!("{source_operator}/hostname");
        HostnameSpecResource {
            meta: Metadata::new(
                NETWORK_CONFIG_NS,
                HOSTNAME_SPEC_KIND,
                ResourceId::new(id).unwrap(),
            ),
            source_operator,
            spec,
        }
    }

    /// Resource kind descriptor.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(NETWORK_CONFIG_NS, HOSTNAME_SPEC_KIND)
    }
}

/// Final merged hostname spec consumed from the `network` namespace.
#[derive(Debug, Clone)]
pub struct MergedHostnameSpecResource {
    meta: Metadata,
    /// Merged hostname spec.
    pub spec: HostnameSpec,
}

impl MergedHostnameSpecResource {
    /// Build the singleton final hostname spec resource.
    pub fn new(spec: HostnameSpec) -> Self {
        MergedHostnameSpecResource {
            meta: Metadata::new(
                NETWORK_NS,
                HOSTNAME_SPEC_KIND,
                ResourceId::new("hostname").unwrap(),
            ),
            spec,
        }
    }

    /// Resource kind descriptor.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(NETWORK_NS, HOSTNAME_SPEC_KIND)
    }
}

impl Resource for MergedHostnameSpecResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        hostname_spec_fingerprint(&self.spec)
    }

    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

impl Resource for HostnameSpecResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        format!(
            "source={};{}",
            self.source_operator,
            hostname_spec_fingerprint(&self.spec),
        )
    }

    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// Resolver output produced by a network operator.
#[derive(Debug, Clone)]
pub struct ResolverSpecResource {
    meta: Metadata,
    /// Operator id that produced this spec.
    pub source_operator: String,
    /// Resolver spec.
    pub spec: ResolverSpec,
}

impl ResolverSpecResource {
    /// Build a COSI resource for a resolver spec.
    pub fn new(source_operator: impl Into<String>, spec: ResolverSpec) -> Self {
        let source_operator = source_operator.into();
        let id = format!("{source_operator}/resolvers");
        ResolverSpecResource {
            meta: Metadata::new(
                NETWORK_CONFIG_NS,
                RESOLVER_SPEC_KIND,
                ResourceId::new(id).unwrap(),
            ),
            source_operator,
            spec,
        }
    }

    /// Resource kind descriptor.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(NETWORK_CONFIG_NS, RESOLVER_SPEC_KIND)
    }
}

/// Final merged resolver spec consumed from the `network` namespace.
#[derive(Debug, Clone)]
pub struct MergedResolverSpecResource {
    meta: Metadata,
    /// Merged resolver spec.
    pub spec: ResolverSpec,
}

impl MergedResolverSpecResource {
    /// Build the singleton final resolver spec resource.
    pub fn new(spec: ResolverSpec) -> Self {
        MergedResolverSpecResource {
            meta: Metadata::new(
                NETWORK_NS,
                RESOLVER_SPEC_KIND,
                ResourceId::new("resolvers").unwrap(),
            ),
            spec,
        }
    }

    /// Resource kind descriptor.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(NETWORK_NS, RESOLVER_SPEC_KIND)
    }
}

impl Resource for MergedResolverSpecResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        resolver_spec_fingerprint(&self.spec)
    }

    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

impl Resource for ResolverSpecResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        format!(
            "source={};{}",
            self.source_operator,
            resolver_spec_fingerprint(&self.spec),
        )
    }

    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// Controller that snapshots observed kernel links into COSI
/// [`LinkStatusResource`] values.
///
/// On Linux this calls `talos-network`'s live rtnetlink source. On non-Linux
/// hosts the default constructor is a no-op source so registry smoke tests stay
/// host-portable; tests can inject deterministic snapshots with
/// [`LinkStatusSourceController::new_with_source`].
pub struct LinkStatusSourceController {
    source: Box<dyn FnMut() -> CoreResult<Vec<LinkStatus>>>,
}

impl std::fmt::Debug for LinkStatusSourceController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinkStatusSourceController")
            .field("source", &"<link-status-source>")
            .finish()
    }
}

impl Default for LinkStatusSourceController {
    fn default() -> Self {
        Self::new()
    }
}

impl LinkStatusSourceController {
    /// Construct the controller with the platform live source.
    pub fn new() -> Self {
        #[cfg(target_os = "linux")]
        {
            Self::new_with_source(os_network_domain::list_link_statuses)
        }
        #[cfg(not(target_os = "linux"))]
        {
            Self::new_with_source(|| Ok(Vec::new()))
        }
    }

    /// Construct the controller with an injected source for deterministic tests.
    pub fn new_with_source(source: impl FnMut() -> CoreResult<Vec<LinkStatus>> + 'static) -> Self {
        LinkStatusSourceController {
            source: Box::new(source),
        }
    }
}

impl Controller for LinkStatusSourceController {
    fn name(&self) -> &str {
        "network.LinkStatusSourceController"
    }

    fn inputs(&self) -> Vec<Input> {
        Vec::new()
    }

    fn outputs(&self) -> Vec<Output> {
        vec![Output::new(LinkStatusResource::kind())]
    }

    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult<()> {
        let statuses = (self.source)().map_err(|e| {
            ReconcileError::Invalid(format!("link status source snapshot failed: {e}"))
        })?;

        let mut desired_keys = BTreeSet::new();
        for status in statuses {
            if status.name.is_empty() {
                return Err(ReconcileError::Invalid(
                    "link status source emitted empty link name".to_string(),
                ));
            }
            let resource = LinkStatusResource::new(status).map_err(|e| {
                ReconcileError::Invalid(format!("invalid link status resource: {e}"))
            })?;
            desired_keys.insert(resource.metadata().key());
            ctx.write(Box::new(resource))?;
        }

        cleanup_owned_outputs(ctx, &LinkStatusResource::kind(), &desired_keys)
    }
}

/// Controller that merges source-layer link specs into final network namespace
/// resources using Talos config-layer precedence.
#[derive(Debug, Default)]
pub struct LinkMergeController;

impl LinkMergeController {
    /// Construct the controller.
    pub fn new() -> Self {
        LinkMergeController
    }
}

impl Controller for LinkMergeController {
    fn name(&self) -> &str {
        "network.LinkMergeController"
    }

    fn inputs(&self) -> Vec<Input> {
        vec![Input::weak(LinkSpecResource::kind())]
    }

    fn outputs(&self) -> Vec<Output> {
        vec![Output::new(MergedLinkSpecResource::kind())]
    }

    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult<()> {
        let mut source_specs = Vec::new();
        for resource in ctx.list(&LinkSpecResource::kind()) {
            let fp = resource.spec_fingerprint();
            let spec = parse_link_spec(&fp)?;
            let fp_fields = fields(&fp);
            let source = required(&fp_fields, "source")?.to_string();
            let expected_id = layered_link_id(&source, &spec);
            let actual_id = resource.metadata().id().as_str();
            if actual_id != expected_id {
                return Err(ReconcileError::Invalid(format!(
                    "link spec id '{actual_id}' does not match layered id '{expected_id}'"
                )));
            }
            source_specs.push(spec);
        }

        let mut desired_keys = BTreeSet::new();
        for spec in merge_links(&source_specs) {
            let resource = MergedLinkSpecResource::new(spec);
            desired_keys.insert(resource.metadata().key());
            ctx.write(Box::new(resource))?;
        }

        cleanup_owned_outputs(ctx, &MergedLinkSpecResource::kind(), &desired_keys)
    }
}

/// Controller that merges source-layer address specs into final network
/// namespace resources using Talos config-layer precedence.
#[derive(Debug, Default)]
pub struct AddressMergeController;

impl AddressMergeController {
    /// Construct the controller.
    pub fn new() -> Self {
        AddressMergeController
    }
}

impl Controller for AddressMergeController {
    fn name(&self) -> &str {
        "network.AddressMergeController"
    }

    fn inputs(&self) -> Vec<Input> {
        vec![Input::weak(AddressSpecResource::kind())]
    }

    fn outputs(&self) -> Vec<Output> {
        vec![Output::new(MergedAddressSpecResource::kind())]
    }

    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult<()> {
        let mut specs = Vec::new();
        for resource in ctx.list(&AddressSpecResource::kind()) {
            specs.push(parse_address_spec(&resource.spec_fingerprint())?);
        }

        let mut desired_keys = BTreeSet::new();
        for spec in merge_addresses(&specs) {
            let resource = MergedAddressSpecResource::new(spec);
            desired_keys.insert(resource.metadata().key());
            ctx.write(Box::new(resource))?;
        }

        cleanup_owned_outputs(ctx, &MergedAddressSpecResource::kind(), &desired_keys)
    }
}

/// Controller that merges source-layer route specs into final network
/// namespace resources using Talos config-layer precedence.
#[derive(Debug, Default)]
pub struct RouteMergeController;

impl RouteMergeController {
    /// Construct the controller.
    pub fn new() -> Self {
        RouteMergeController
    }
}

impl Controller for RouteMergeController {
    fn name(&self) -> &str {
        "network.RouteMergeController"
    }

    fn inputs(&self) -> Vec<Input> {
        vec![Input::weak(RouteSpecResource::kind())]
    }

    fn outputs(&self) -> Vec<Output> {
        vec![Output::new(MergedRouteSpecResource::kind())]
    }

    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult<()> {
        let mut specs = Vec::new();
        for resource in ctx.list(&RouteSpecResource::kind()) {
            specs.push(parse_route_spec(&resource.spec_fingerprint())?);
        }

        let mut desired_keys = BTreeSet::new();
        for spec in merge_routes(&specs) {
            let resource = MergedRouteSpecResource::new(spec);
            desired_keys.insert(resource.metadata().key());
            ctx.write(Box::new(resource))?;
        }

        cleanup_owned_outputs(ctx, &MergedRouteSpecResource::kind(), &desired_keys)
    }
}

/// Controller that merges source-layer hostname specs into the final network
/// namespace singleton using Talos config-layer precedence.
#[derive(Debug, Default)]
pub struct HostnameMergeController;

impl HostnameMergeController {
    /// Construct the controller.
    pub fn new() -> Self {
        HostnameMergeController
    }
}

impl Controller for HostnameMergeController {
    fn name(&self) -> &str {
        "network.HostnameMergeController"
    }

    fn inputs(&self) -> Vec<Input> {
        vec![Input::weak(HostnameSpecResource::kind())]
    }

    fn outputs(&self) -> Vec<Output> {
        vec![Output::new(MergedHostnameSpecResource::kind())]
    }

    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult<()> {
        let mut specs = Vec::new();
        for resource in ctx.list(&HostnameSpecResource::kind()) {
            specs.push(parse_hostname_spec(&resource.spec_fingerprint())?);
        }

        let mut desired_keys = BTreeSet::new();
        if let Some(spec) = select_merged_hostname_spec(&specs) {
            let resource = MergedHostnameSpecResource::new(spec);
            desired_keys.insert(resource.metadata().key());
            ctx.write(Box::new(resource))?;
        }

        cleanup_owned_outputs(ctx, &MergedHostnameSpecResource::kind(), &desired_keys)
    }
}

/// Controller that merges source-layer resolver specs into the final network
/// namespace using Talos config-layer precedence.
#[derive(Debug, Default)]
pub struct ResolverMergeController;

impl ResolverMergeController {
    /// Construct the controller.
    pub fn new() -> Self {
        ResolverMergeController
    }
}

impl Controller for ResolverMergeController {
    fn name(&self) -> &str {
        "network.ResolverMergeController"
    }

    fn inputs(&self) -> Vec<Input> {
        vec![Input::weak(ResolverSpecResource::kind())]
    }

    fn outputs(&self) -> Vec<Output> {
        vec![Output::new(MergedResolverSpecResource::kind())]
    }

    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult<()> {
        let mut specs = Vec::new();
        for resource in ctx.list(&ResolverSpecResource::kind()) {
            specs.push(parse_resolver_spec(&resource.spec_fingerprint())?);
        }

        let mut desired_keys = BTreeSet::new();
        if let Some(spec) = merge_resolver_specs(&specs) {
            let resource = MergedResolverSpecResource::new(spec);
            desired_keys.insert(resource.metadata().key());
            ctx.write(Box::new(resource))?;
        }

        cleanup_owned_outputs(ctx, &MergedResolverSpecResource::kind(), &desired_keys)
    }
}

/// Controller that seeds source-layer resolver specs from `ResolverConfig`.
#[derive(Debug, Default)]
pub struct ResolverConfigController;

impl ResolverConfigController {
    /// Construct the controller.
    pub fn new() -> Self {
        ResolverConfigController
    }
}

impl Controller for ResolverConfigController {
    fn name(&self) -> &str {
        "network.ResolverConfigController"
    }

    fn inputs(&self) -> Vec<Input> {
        vec![Input::weak(MachineConfigDocument::kind())]
    }

    fn outputs(&self) -> Vec<Output> {
        vec![Output::new(ResolverSpecResource::kind())]
    }

    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult<()> {
        let specs = match ctx.get(&MachineConfigDocument::active_key()) {
            Some(resource) => {
                let contents = parse_machine_config_contents(&resource.spec_fingerprint())?;
                machine_config_resolver_specs(&contents)?
            }
            None => Vec::new(),
        };

        let mut desired_keys = BTreeSet::new();
        for spec in specs {
            let resource = ResolverSpecResource::new("configuration/resolverconfig", spec);
            desired_keys.insert(resource.metadata().key());
            ctx.write(Box::new(resource))?;
        }

        cleanup_owned_outputs(ctx, &ResolverSpecResource::kind(), &desired_keys)
    }
}

/// Controller that seeds source-layer link, address and route specs from
/// `LinkConfig` and `VLANConfig` machine config documents.
#[derive(Debug, Default)]
pub struct LinkConfigController;

impl LinkConfigController {
    /// Construct the controller.
    pub fn new() -> Self {
        LinkConfigController
    }
}

impl Controller for LinkConfigController {
    fn name(&self) -> &str {
        "network.LinkConfigController"
    }

    fn inputs(&self) -> Vec<Input> {
        vec![
            Input::weak(MachineConfigDocument::kind()),
            Input::weak(LinkStatusResource::kind()),
            Input::weak(LinkSpecResource::kind()),
        ]
    }

    fn outputs(&self) -> Vec<Output> {
        vec![
            Output::new(LinkSpecResource::kind()),
            Output::new(AddressSpecResource::kind()),
            Output::new(RouteSpecResource::kind()),
        ]
    }

    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult<()> {
        let machine_config = match ctx.get(&MachineConfigDocument::active_key()) {
            Some(resource) => {
                let contents = parse_machine_config_contents(&resource.spec_fingerprint())?;
                Some(contents)
            }
            None => None,
        };
        let mut projection = match machine_config.as_deref() {
            Some(contents) => machine_config_link_projection(contents)?,
            None => LinkProjection::default(),
        };
        if machine_config_run_default_dhcp_operators(machine_config.as_deref())? {
            let mut configured_links = machine_config_configured_links(machine_config.as_deref())?;
            configured_links.extend(projection.links.iter().map(|(_, spec)| spec.name.clone()));
            for resource in ctx.list(&LinkSpecResource::kind()) {
                let spec = parse_link_spec(&resource.spec_fingerprint())?;
                match spec.layer {
                    ConfigLayer::Cmdline | ConfigLayer::Configuration | ConfigLayer::Platform => {
                        configured_links.insert(spec.name);
                    }
                    ConfigLayer::Default | ConfigLayer::Operator => {}
                }
            }

            for status in parse_link_status_resources(ctx)? {
                if status.physical() && !link_status_matches_any(&status, &configured_links) {
                    let mut spec = LinkSpec::physical(status.name.clone(), ConfigLayer::Default);
                    spec.mtu = 0;
                    projection
                        .links
                        .push((format!("default/{}", status.name), spec));
                }
            }
        }

        let mut link_keys = BTreeSet::new();
        for (source, spec) in projection.links {
            spec.validate()
                .map_err(|e| ReconcileError::Invalid(e.to_string()))?;
            let resource = LinkSpecResource::new(source, spec);
            link_keys.insert(resource.metadata().key());
            ctx.write(Box::new(resource))?;
        }

        let mut address_keys = BTreeSet::new();
        for (source, spec) in projection.addresses {
            let resource = AddressSpecResource::new(source, spec);
            address_keys.insert(resource.metadata().key());
            ctx.write(Box::new(resource))?;
        }

        let mut route_keys = BTreeSet::new();
        for (source, spec) in projection.routes {
            let resource = RouteSpecResource::new(source, spec);
            route_keys.insert(resource.metadata().key());
            ctx.write(Box::new(resource))?;
        }

        cleanup_owned_outputs(ctx, &LinkSpecResource::kind(), &link_keys)?;
        cleanup_owned_outputs(ctx, &AddressSpecResource::kind(), &address_keys)?;
        cleanup_owned_outputs(ctx, &RouteSpecResource::kind(), &route_keys)
    }
}

/// Controller that seeds source-layer network operator specs from the active
/// machine config document.
#[derive(Debug, Default)]
pub struct OperatorConfigController;

impl OperatorConfigController {
    /// Construct the controller.
    pub fn new() -> Self {
        OperatorConfigController
    }
}

impl Controller for OperatorConfigController {
    fn name(&self) -> &str {
        "network.OperatorConfigController"
    }

    fn inputs(&self) -> Vec<Input> {
        vec![
            Input::weak(MachineConfigDocument::kind()),
            Input::weak(LinkStatusResource::kind()),
            Input::weak(LinkSpecResource::kind()),
        ]
    }

    fn outputs(&self) -> Vec<Output> {
        vec![Output::new(OperatorConfigSpecResource::kind())]
    }

    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult<()> {
        let machine_config = match ctx.get(&MachineConfigDocument::active_key()) {
            Some(resource) => {
                let contents = parse_machine_config_contents(&resource.spec_fingerprint())?;
                Some(contents)
            }
            None => None,
        };
        let mut specs = match machine_config.as_deref() {
            Some(contents) => machine_config_operator_specs(contents)?,
            None => Vec::new(),
        };

        if machine_config_run_default_dhcp_operators(machine_config.as_deref())? {
            let mut configured_links = machine_config_configured_links(machine_config.as_deref())?;
            for resource in ctx.list(&LinkSpecResource::kind()) {
                let spec = parse_link_spec(&resource.spec_fingerprint())?;
                match spec.layer {
                    ConfigLayer::Cmdline | ConfigLayer::Configuration | ConfigLayer::Platform => {
                        configured_links.insert(spec.name);
                    }
                    ConfigLayer::Default | ConfigLayer::Operator => {}
                }
            }

            for status in parse_link_status_resources(ctx)? {
                if status.default_dhcp4_candidate()
                    && !link_status_matches_any(&status, &configured_links)
                {
                    specs.push(default_dhcp4_operator_spec(&status.name));
                }
            }
        }

        let mut desired_keys = BTreeSet::new();
        for spec in specs {
            spec.validate()
                .map_err(|e| ReconcileError::Invalid(e.to_string()))?;
            let resource = OperatorConfigSpecResource::new(spec);
            desired_keys.insert(resource.metadata().key());
            ctx.write(Box::new(resource))?;
        }

        cleanup_owned_outputs(ctx, &OperatorConfigSpecResource::kind(), &desired_keys)
    }
}

/// Controller that merges source-layer operator specs into final network
/// operator specs using Talos config-layer precedence.
#[derive(Debug, Default)]
pub struct OperatorMergeController;

impl OperatorMergeController {
    /// Construct the controller.
    pub fn new() -> Self {
        OperatorMergeController
    }
}

impl Controller for OperatorMergeController {
    fn name(&self) -> &str {
        "network.OperatorMergeController"
    }

    fn inputs(&self) -> Vec<Input> {
        vec![Input::weak(OperatorConfigSpecResource::kind())]
    }

    fn outputs(&self) -> Vec<Output> {
        vec![Output::new(OperatorSpecResource::kind())]
    }

    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult<()> {
        let mut source_specs = Vec::new();
        for resource in ctx.list(&OperatorConfigSpecResource::kind()) {
            let spec = parse_operator_spec(&resource.spec_fingerprint())?;
            let expected_id = layered_operator_id(&spec);
            let actual_id = resource.metadata().id().as_str();
            if actual_id != expected_id {
                return Err(ReconcileError::Invalid(format!(
                    "operator config spec id '{actual_id}' does not match layered id '{expected_id}'"
                )));
            }
            source_specs.push(spec);
        }

        let mut desired_keys = BTreeSet::new();
        for spec in merge_operators(&source_specs) {
            let resource = OperatorSpecResource::new(spec);
            desired_keys.insert(resource.metadata().key());
            ctx.write(Box::new(resource))?;
        }

        cleanup_owned_outputs(ctx, &OperatorSpecResource::kind(), &desired_keys)
    }
}

/// Controller that materializes operator results into network spec resources.
#[derive(Debug, Default)]
pub struct OperatorResultBridgeController;

impl OperatorResultBridgeController {
    /// Construct the controller.
    pub fn new() -> Self {
        OperatorResultBridgeController
    }
}

impl Controller for OperatorResultBridgeController {
    fn name(&self) -> &str {
        "network.OperatorResultBridgeController"
    }

    fn inputs(&self) -> Vec<Input> {
        vec![
            Input::weak(OperatorSpecResource::kind()),
            Input::weak(OperatorResultResource::kind()),
        ]
    }

    fn outputs(&self) -> Vec<Output> {
        vec![
            Output::new(AddressSpecResource::kind()),
            Output::new(RouteSpecResource::kind()),
            Output::new(HostnameSpecResource::kind()),
            Output::new(ResolverSpecResource::kind()),
        ]
    }

    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult<()> {
        let specs = parse_operator_specs(ctx)?;
        let results = parse_operator_results(ctx)?;
        let mut desired_keys = BTreeSet::new();

        for (operator_id, spec) in &specs {
            let Some(result) = results.get(operator_id) else {
                continue;
            };
            spec.validate()
                .map_err(|e| ReconcileError::Invalid(e.to_string()))?;
            let output = spec
                .apply_result(result)
                .map_err(|e| ReconcileError::Invalid(e.to_string()))?;

            for address in output.addresses {
                let resource = AddressSpecResource::new(operator_id.clone(), address);
                desired_keys.insert(resource.metadata().key());
                ctx.write(Box::new(resource))?;
            }
            for route in output.routes {
                let resource = RouteSpecResource::new(operator_id.clone(), route);
                desired_keys.insert(resource.metadata().key());
                ctx.write(Box::new(resource))?;
            }
            if let Some(hostname) = output.hostname {
                let resource = HostnameSpecResource::new(operator_id.clone(), hostname);
                desired_keys.insert(resource.metadata().key());
                ctx.write(Box::new(resource))?;
            }
            if let Some(resolver) = output.resolver {
                let resource = ResolverSpecResource::new(operator_id.clone(), resolver);
                desired_keys.insert(resource.metadata().key());
                ctx.write(Box::new(resource))?;
            }
        }

        cleanup_stale_outputs(ctx, &desired_keys, &specs, &results)
    }
}

fn parse_operator_specs(
    ctx: &ReconcileContext<'_>,
) -> ReconcileResult<BTreeMap<String, OperatorSpec>> {
    let mut specs = BTreeMap::new();
    for resource in ctx.list(&OperatorSpecResource::kind()) {
        let spec = parse_operator_spec(&resource.spec_fingerprint())?;
        let operator_id = resource.metadata().id().as_str().to_string();
        if operator_id != spec.id() {
            return Err(ReconcileError::Invalid(format!(
                "operator spec id '{operator_id}' does not match spec id '{}'",
                spec.id()
            )));
        }
        specs.insert(operator_id, spec);
    }
    Ok(specs)
}

fn parse_operator_results(
    ctx: &ReconcileContext<'_>,
) -> ReconcileResult<BTreeMap<String, OperatorResult>> {
    let mut results = BTreeMap::new();
    for resource in ctx.list(&OperatorResultResource::kind()) {
        let result = parse_operator_result(&resource.spec_fingerprint())?;
        results.insert(resource.metadata().id().as_str().to_string(), result);
    }
    Ok(results)
}

fn parse_link_status_resources(ctx: &ReconcileContext<'_>) -> ReconcileResult<Vec<LinkStatus>> {
    let mut statuses = Vec::new();
    for resource in ctx.list(&LinkStatusResource::kind()) {
        let status = parse_link_status(&resource.spec_fingerprint())?;
        let actual_id = resource.metadata().id().as_str();
        if actual_id != status.name {
            return Err(ReconcileError::Invalid(format!(
                "link status id '{actual_id}' does not match status name '{}'",
                status.name
            )));
        }
        statuses.push(status);
    }
    Ok(statuses)
}

fn machine_config_run_default_dhcp_operators(contents: Option<&str>) -> ReconcileResult<bool> {
    let Some(contents) = contents else {
        return Ok(true);
    };
    if contents.trim().is_empty() {
        return Ok(true);
    }
    try_machine_config_run_default_dhcp_operators(contents)
        .map_err(|e| ReconcileError::Invalid(e.to_string()))
}

fn try_machine_config_run_default_dhcp_operators(contents: &str) -> CoreResult<bool> {
    let container = load_network_config(contents)?;
    Ok(link_configs(&container)?.is_empty()
        && vlan_configs(&container)?.is_empty()
        && dhcpv4_configs(&container)?.is_empty()
        && dhcpv6_configs(&container)?.is_empty())
}

fn machine_config_configured_links(contents: Option<&str>) -> ReconcileResult<BTreeSet<String>> {
    let Some(contents) = contents else {
        return Ok(BTreeSet::new());
    };
    if contents.trim().is_empty() {
        return Ok(BTreeSet::new());
    }
    try_machine_config_configured_links(contents)
        .map_err(|e| ReconcileError::Invalid(e.to_string()))
}

fn try_machine_config_configured_links(contents: &str) -> CoreResult<BTreeSet<String>> {
    let container = load_network_config(contents)?;
    let mut links = BTreeSet::new();

    for interface in &container.core().machine.network.interfaces {
        if !interface.interface.is_empty() {
            links.insert(interface.interface.clone());
            for vlan in &interface.vlans {
                if vlan.vlan_id != 0 {
                    links.insert(vlan_link_name(&interface.interface, vlan.vlan_id));
                }
            }
        }
    }
    for config in link_configs(&container)? {
        links.insert(config.name);
    }
    for config in vlan_configs(&container)? {
        links.insert(config.name);
    }
    for config in dhcpv4_configs(&container)? {
        links.insert(config.name);
    }
    for config in dhcpv6_configs(&container)? {
        links.insert(config.name);
    }

    Ok(links)
}

fn link_status_matches_any(status: &LinkStatus, names: &BTreeSet<String>) -> bool {
    status.all_names().any(|name| names.contains(name))
}

fn cleanup_owned_outputs(
    ctx: &mut ReconcileContext<'_>,
    kind: &ResourceKind,
    desired_keys: &BTreeSet<String>,
) -> ReconcileResult<()> {
    for resource in ctx.list(kind) {
        if resource.metadata().owner() != ctx.owner() {
            continue;
        }
        let key = resource.metadata().key();
        if !desired_keys.contains(&key) {
            ctx.destroy(&key)?;
        }
    }
    Ok(())
}

fn cleanup_stale_outputs(
    ctx: &mut ReconcileContext<'_>,
    desired_keys: &BTreeSet<String>,
    specs: &BTreeMap<String, OperatorSpec>,
    results: &BTreeMap<String, OperatorResult>,
) -> ReconcileResult<()> {
    let live_sources: BTreeSet<&str> = specs
        .keys()
        .filter(|operator_id| results.contains_key(*operator_id))
        .map(String::as_str)
        .collect();

    for kind in [
        AddressSpecResource::kind(),
        RouteSpecResource::kind(),
        HostnameSpecResource::kind(),
        ResolverSpecResource::kind(),
    ] {
        for resource in ctx.list(&kind) {
            if resource.metadata().owner() != ctx.owner() {
                continue;
            }
            let Some(source) = fingerprint_field(&resource.spec_fingerprint(), "source") else {
                continue;
            };
            let key = resource.metadata().key();
            if !live_sources.contains(source.as_str()) || !desired_keys.contains(&key) {
                ctx.destroy(&key)?;
            }
        }
    }
    Ok(())
}

fn operator_spec_fingerprint(spec: &OperatorSpec) -> String {
    format!(
        "kind={};link={};require_up={};route_metric={};skip_hostname={};client={};duid={};layer={}",
        spec.kind.as_str(),
        spec.link_name,
        spec.require_up,
        spec.route_metric,
        spec.skip_hostname_request,
        client_identifier_as_str(&spec.client_identifier),
        hex_bytes(&spec.client_identifier.duid_raw),
        spec.layer.as_str(),
    )
}

fn link_spec_fingerprint(spec: &LinkSpec) -> String {
    let (kind, members, mode, parent, vlan_id, vlan_protocol) = match &spec.kind {
        LinkKind::Physical => (
            "physical",
            String::new(),
            String::new(),
            String::new(),
            0,
            String::new(),
        ),
        LinkKind::Bond { members, mode } => (
            "bond",
            members.join(","),
            bond_mode_as_str(*mode).to_string(),
            String::new(),
            0,
            String::new(),
        ),
        LinkKind::Bridge { members } => (
            "bridge",
            members.join(","),
            String::new(),
            String::new(),
            0,
            String::new(),
        ),
        LinkKind::Vlan {
            parent,
            vlan_id,
            protocol,
        } => (
            "vlan",
            String::new(),
            String::new(),
            parent.clone(),
            *vlan_id,
            protocol.to_str().to_string(),
        ),
        LinkKind::Dummy => (
            "dummy",
            String::new(),
            String::new(),
            String::new(),
            0,
            String::new(),
        ),
    };
    let multicast = spec
        .multicast
        .map(|value| value.to_string())
        .unwrap_or_default();
    format!(
        "name={};up={};mtu={};multicast={multicast};kind={kind};members={members};mode={mode};parent={parent};vlan_id={vlan_id};vlan_protocol={vlan_protocol};layer={}",
        spec.name,
        spec.up,
        spec.mtu,
        spec.layer.as_str(),
    )
}

fn link_status_fingerprint(status: &LinkStatus) -> String {
    let aliases = if status.aliases.is_empty() {
        "-".to_string()
    } else {
        status.aliases.join(",")
    };
    format!(
        "name={};type={};kind={};aliases={aliases};admin_up={};oper_state={};carrier={};mac={};mtu={}",
        status.name,
        status.link_type.as_str(),
        status.kind,
        status.admin_up,
        oper_state_as_str(status.oper_state),
        status.carrier,
        status.mac_string(),
        status.mtu,
    )
}

fn address_spec_fingerprint(spec: &AddressSpec) -> String {
    format!(
        "address={};prefix={};link={};family={};scope={};flags={};priority={};layer={}",
        spec.address,
        spec.prefix_len,
        spec.link_name,
        address_family_as_str(spec.family),
        scope_as_str(spec.scope),
        address_flags_as_str(spec.flags),
        spec.priority,
        spec.layer.as_str(),
    )
}

fn route_spec_fingerprint(spec: &RouteSpec) -> String {
    format!(
        "destination={};prefix={};source={};gateway={};out_link={};family={};metric={};mtu={};table={};protocol={};layer={}",
        option_address(spec.destination).unwrap_or_else(|| "default".to_string()),
        spec.prefix_len,
        option_address(spec.source).unwrap_or_default(),
        option_address(spec.gateway).unwrap_or_default(),
        spec.out_link,
        address_family_as_str(spec.family),
        spec.metric,
        spec.mtu,
        route_table_as_u32(spec.table),
        route_protocol_as_str(spec.protocol),
        spec.layer.as_str(),
    )
}

fn hostname_spec_fingerprint(spec: &HostnameSpec) -> String {
    format!(
        "hostname={};domain={};layer={}",
        spec.hostname.as_str(),
        spec.domainname.as_deref().unwrap_or_default(),
        spec.layer.as_str(),
    )
}

fn resolver_spec_fingerprint(spec: &ResolverSpec) -> String {
    format!(
        "servers={};search={};layer={}",
        join_addresses(&spec.servers),
        spec.search_domains.join(","),
        spec.layer.as_str(),
    )
}

fn select_merged_hostname_spec(specs: &[HostnameSpec]) -> Option<HostnameSpec> {
    let status = merge_hostname(specs)?;
    specs
        .iter()
        .rev()
        .find(|spec| {
            spec.hostname.as_str() == status.hostname
                && spec.domainname == status.domainname
                && specs
                    .iter()
                    .all(|candidate| spec.layer.precedence() >= candidate.layer.precedence())
        })
        .cloned()
}

fn merge_resolver_specs(specs: &[ResolverSpec]) -> Option<ResolverSpec> {
    let mut best: Option<&ResolverSpec> = None;
    for spec in specs {
        if spec.servers.is_empty() && spec.search_domains.is_empty() {
            continue;
        }
        match best {
            Some(b) if b.layer.precedence() > spec.layer.precedence() => {}
            _ => best = Some(spec),
        }
    }

    best.map(|spec| ResolverSpec {
        servers: spec.effective_servers(),
        search_domains: spec.search_domains.clone(),
        layer: spec.layer,
    })
}

fn layered_operator_id(spec: &OperatorSpec) -> String {
    format!("{}/{}", spec.layer.as_str(), spec.id())
}

fn layered_link_id(source: &str, _spec: &LinkSpec) -> String {
    source.to_string()
}

fn parse_machine_config_contents(fp: &str) -> ReconcileResult<String> {
    let fields = fields(fp);
    let contents_hex = required(&fields, "contents")?;
    let bytes = parse_hex_bytes(contents_hex)?;
    String::from_utf8(bytes).map_err(|e| ReconcileError::Invalid(e.to_string()))
}

fn machine_config_operator_specs(contents: &str) -> ReconcileResult<Vec<OperatorSpec>> {
    if contents.trim().is_empty() {
        return Ok(Vec::new());
    }

    try_machine_config_operator_specs(contents).map_err(|e| ReconcileError::Invalid(e.to_string()))
}

fn try_machine_config_operator_specs(contents: &str) -> CoreResult<Vec<OperatorSpec>> {
    let container = load_network_config(contents)?;
    let mut out = Vec::new();

    for iface in &container.core().machine.network.interfaces {
        if iface.interface.is_empty() || iface.ignore {
            continue;
        }
        if iface.dhcp {
            push_dhcp_operator_specs(&mut out, &iface.interface, &iface.dhcp_options)?;
        }

        for vlan in &iface.vlans {
            if !vlan.dhcp {
                if vlan.vlan_id != 0 {
                    push_legacy_vlan_vip_operator_specs(
                        &mut out,
                        contents,
                        &iface.interface,
                        vlan.vlan_id,
                    )?;
                }
                continue;
            }
            let vlan_link = vlan_link_name(&iface.interface, vlan.vlan_id);
            push_dhcp_operator_specs(&mut out, &vlan_link, &vlan.dhcp_options)?;
            push_legacy_vlan_vip_operator_specs(
                &mut out,
                contents,
                &iface.interface,
                vlan.vlan_id,
            )?;
        }
        push_legacy_interface_vip_operator_specs(&mut out, contents, &iface.interface)?;
    }

    for config in dhcpv4_configs(&container)? {
        let mut op = OperatorSpec::dhcp4(&config.name);
        op.route_metric = config.route_metric_or(os_network_domain::DEFAULT_ROUTE_METRIC);
        op.skip_hostname_request = config.ignore_hostname;
        op.client_identifier = match config.client_identifier {
            DhcpV4ClientIdentifier::None => ClientIdentifierSpec::none(),
            DhcpV4ClientIdentifier::Mac => ClientIdentifierSpec::mac(),
            DhcpV4ClientIdentifier::Duid => ClientIdentifierSpec::duid(config.duid_raw),
        };
        op.validate()?;
        out.push(op);
    }

    for config in dhcpv6_configs(&container)? {
        let mut op = OperatorSpec::dhcp6(&config.name);
        op.route_metric = config.route_metric_or(os_network_domain::DEFAULT_ROUTE_METRIC);
        op.skip_hostname_request = config.ignore_hostname;
        op.client_identifier = match config.client_identifier {
            DhcpV6ClientIdentifier::None => ClientIdentifierSpec::none(),
            DhcpV6ClientIdentifier::Mac => ClientIdentifierSpec::mac(),
            DhcpV6ClientIdentifier::Duid => ClientIdentifierSpec::duid(config.duid_raw),
        };
        op.validate()?;
        out.push(op);
    }

    push_config_doc_vip_operator_specs(&mut out, &container)?;

    Ok(out)
}

fn load_network_config(contents: &str) -> CoreResult<os_machine_config_domain::Config> {
    load_from_bytes_with(contents, &network_operator_config_registry())
}

fn network_operator_config_registry() -> Registry {
    let mut registry = Registry::with_builtins();
    registry.register_or_replace(KindSpec::multiple(LAYER2_VIP_CONFIG_KIND));
    registry
}

fn push_legacy_interface_vip_operator_specs(
    out: &mut Vec<OperatorSpec>,
    contents: &str,
    link_name: &str,
) -> CoreResult<()> {
    let Some(ip) = legacy_interface_vip_ip(contents, link_name)? else {
        return Ok(());
    };
    validate_vip_ip(&ip)?;
    out.push(vip_operator_spec(link_name));
    Ok(())
}

fn push_legacy_vlan_vip_operator_specs(
    out: &mut Vec<OperatorSpec>,
    contents: &str,
    parent: &str,
    vlan_id: u16,
) -> CoreResult<()> {
    let Some(ip) = legacy_vlan_vip_ip(contents, parent, vlan_id)? else {
        return Ok(());
    };
    validate_vip_ip(&ip)?;
    out.push(vip_operator_spec(vlan_link_name(parent, vlan_id)));
    Ok(())
}

fn push_config_doc_vip_operator_specs(
    out: &mut Vec<OperatorSpec>,
    container: &os_machine_config_domain::Config,
) -> CoreResult<()> {
    for doc in container.documents() {
        if doc.meta.kind != LAYER2_VIP_CONFIG_KIND {
            continue;
        }
        let root = os_machine_config_domain::yaml::parse(&doc.body)
            .map_err(|e| os_kernel::Error::parse(e.to_string()))?;
        let name = yaml_required_str(&root, "name")?;
        let link = yaml_required_str(&root, "link")?;
        validate_vip_ip(name)?;
        out.push(vip_operator_spec(link));
    }
    Ok(())
}

fn vip_operator_spec(link_name: impl Into<String>) -> OperatorSpec {
    OperatorSpec {
        kind: OperatorKind::Vip,
        link_name: link_name.into(),
        require_up: true,
        route_metric: os_network_domain::DEFAULT_ROUTE_METRIC,
        skip_hostname_request: false,
        client_identifier: ClientIdentifierSpec::none(),
        layer: ConfigLayer::Configuration,
    }
}

fn validate_vip_ip(ip: &str) -> CoreResult<()> {
    NodeAddress::parse(ip).map(|_| ())
}

fn legacy_interface_vip_ip(contents: &str, link_name: &str) -> CoreResult<Option<String>> {
    for doc in decode_documents(contents)? {
        if doc.meta.kind != "v1alpha1" {
            continue;
        }
        let root = os_machine_config_domain::yaml::parse(&doc.body)
            .map_err(|e| os_kernel::Error::parse(e.to_string()))?;
        let Some(interfaces) = root
            .get("machine")
            .and_then(|m| m.get("network"))
            .and_then(|n| n.get("interfaces"))
            .and_then(Yaml::as_sequence)
        else {
            return Ok(None);
        };
        for iface in interfaces {
            if iface.get_str("interface") != Some(link_name) {
                continue;
            }
            if let Some(vip) = iface.get("vip") {
                reject_unsupported_legacy_vip_cloud(vip)?;
                let Some(ip) = vip.get_str("ip").or_else(|| vip.get_str("sharedIP")) else {
                    continue;
                };
                return Ok(Some(ip.to_string()));
            }
        }
    }
    Ok(None)
}

fn legacy_vlan_vip_ip(contents: &str, parent: &str, vlan_id: u16) -> CoreResult<Option<String>> {
    for doc in decode_documents(contents)? {
        if doc.meta.kind != "v1alpha1" {
            continue;
        }
        let root = os_machine_config_domain::yaml::parse(&doc.body)
            .map_err(|e| os_kernel::Error::parse(e.to_string()))?;
        let Some(interfaces) = root
            .get("machine")
            .and_then(|m| m.get("network"))
            .and_then(|n| n.get("interfaces"))
            .and_then(Yaml::as_sequence)
        else {
            return Ok(None);
        };
        for iface in interfaces {
            if iface.get_str("interface") != Some(parent) {
                continue;
            }
            let Some(vlans) = iface.get("vlans").and_then(Yaml::as_sequence) else {
                continue;
            };
            for vlan in vlans {
                if parse_yaml_u16(vlan.get_str("vlanId"), "vlanId")? != Some(vlan_id) {
                    continue;
                }
                if let Some(vip) = vlan.get("vip") {
                    reject_unsupported_legacy_vip_cloud(vip)?;
                    let Some(ip) = vip.get_str("ip").or_else(|| vip.get_str("sharedIP")) else {
                        continue;
                    };
                    return Ok(Some(ip.to_string()));
                }
            }
        }
    }
    Ok(None)
}

fn yaml_required_str<'a>(root: &'a Yaml, key: &str) -> CoreResult<&'a str> {
    root.get_str(key)
        .ok_or_else(|| os_kernel::Error::invalid(format!("{key} must be specified")))
}

fn reject_unsupported_legacy_vip_cloud(vip: &Yaml) -> CoreResult<()> {
    if vip.get("equinixMetal").is_some() || vip.get("hcloud").is_some() {
        return Err(os_kernel::Error::invalid(
            "cloud VIP operator specs are not projectable by the current static VIP model",
        ));
    }
    Ok(())
}

fn parse_yaml_u16(value: Option<&str>, field: &str) -> CoreResult<Option<u16>> {
    let Some(value) = value else {
        return Ok(None);
    };
    value
        .parse::<u16>()
        .map(Some)
        .map_err(|_| os_kernel::Error::invalid(format!("{field} must be a u16")))
}

fn machine_config_resolver_specs(contents: &str) -> ReconcileResult<Vec<ResolverSpec>> {
    if contents.trim().is_empty() {
        return Ok(Vec::new());
    }

    try_machine_config_resolver_specs(contents).map_err(|e| ReconcileError::Invalid(e.to_string()))
}

fn try_machine_config_resolver_specs(contents: &str) -> CoreResult<Vec<ResolverSpec>> {
    let container = load_network_config(contents)?;
    let Some(config) = resolver_config(&container)? else {
        return Ok(Vec::new());
    };
    config.validate_projection_supported()?;

    let mut servers = Vec::new();
    for ns in &config.nameservers {
        if ns.protocol != ResolverDnsProtocol::Do53 || !ns.tls_server_name.is_empty() {
            return Err(os_kernel::Error::invalid(
                "ResolverConfig encrypted nameservers are parsed but not yet projectable to ResolverSpec",
            ));
        }
        servers.push(NodeAddress::parse(&ns.address)?);
    }

    if servers.is_empty() && config.search_domains.domains.is_empty() {
        return Ok(Vec::new());
    }

    Ok(vec![ResolverSpec::new_with_search(
        servers,
        config.search_domains.domains,
        ConfigLayer::Configuration,
    )?])
}

#[derive(Debug, Default)]
struct LinkProjection {
    links: Vec<(String, LinkSpec)>,
    addresses: Vec<(String, AddressSpec)>,
    routes: Vec<(String, RouteSpec)>,
}

fn machine_config_link_projection(contents: &str) -> ReconcileResult<LinkProjection> {
    if contents.trim().is_empty() {
        return Ok(LinkProjection::default());
    }

    try_machine_config_link_projection(contents).map_err(|e| ReconcileError::Invalid(e.to_string()))
}

fn try_machine_config_link_projection(contents: &str) -> CoreResult<LinkProjection> {
    let container = load_network_config(contents)?;
    let mut projection = LinkProjection::default();

    for config in link_configs(&container)? {
        let source = link_config_source(&config.name);
        let mut spec = LinkSpec::physical(&config.name, ConfigLayer::Configuration);
        spec.up = config.link.up_or_default();
        spec.mtu = config.link.mtu;
        spec.multicast = config.link.multicast;
        spec.validate()?;
        project_static_link_fields(&mut projection, &source, &config.name, &config.link)?;
        projection.links.push((source, spec));
    }

    for config in vlan_configs(&container)? {
        let source = vlan_config_source(&config.name);
        let spec = LinkSpec {
            name: config.name.clone(),
            up: config.link.up_or_default(),
            mtu: config.link.mtu,
            multicast: config.link.multicast,
            kind: LinkKind::Vlan {
                parent: config.parent.clone(),
                vlan_id: config.vlan_id,
                protocol: vlan_mode_protocol(config.vlan_mode_or_default()),
            },
            layer: ConfigLayer::Configuration,
        };
        spec.validate()?;
        project_static_link_fields(&mut projection, &source, &config.name, &config.link)?;
        projection.links.push((source, spec));
    }

    Ok(projection)
}

fn vlan_mode_protocol(mode: VlanMode) -> VlanProtocol {
    match mode {
        VlanMode::Dot1Q => VlanProtocol::Ieee8021q,
        VlanMode::Dot1Ad => VlanProtocol::Ieee8021ad,
    }
}

fn project_static_link_fields(
    projection: &mut LinkProjection,
    source: &str,
    link_name: &str,
    fields: &LinkFields,
) -> CoreResult<()> {
    for address in &fields.addresses {
        let (addr, prefix_len) = parse_cidr_core(&address.address)?;
        let mut spec = AddressSpec::new(addr, prefix_len, link_name, ConfigLayer::Configuration)?;
        spec.layer = ConfigLayer::Configuration;
        spec.priority = address.route_priority;
        projection.addresses.push((source.to_string(), spec));
    }

    for route in &fields.routes {
        let spec = static_route_spec(route, link_name)?;
        projection.routes.push((source.to_string(), spec));
    }

    Ok(())
}

fn static_route_spec(route: &LinkRouteConfig, link_name: &str) -> CoreResult<RouteSpec> {
    let (mut destination, prefix_len) = if route.destination.trim().is_empty() {
        (None, 0)
    } else {
        let (address, prefix) = parse_cidr_core(&route.destination)?;
        (Some(address), prefix)
    };
    let mut normalized_family = None;
    if let Some(address) = destination
        && prefix_len == 0 && address.is_unspecified() {
            normalized_family = Some(address_family(address));
            destination = None;
        }
    let gateway = if route.gateway.trim().is_empty() {
        None
    } else {
        Some(NodeAddress::parse(&route.gateway)?)
    };
    let mut gateway = gateway;
    if let Some(address) = gateway
        && address.is_unspecified() {
            normalized_family = Some(address_family(address));
            gateway = None;
        }
    let source = if route.source.trim().is_empty() {
        None
    } else {
        Some(NodeAddress::parse(&route.source)?)
    };
    let mut source = source;
    if let Some(address) = source
        && address.is_unspecified() {
            normalized_family = Some(address_family(address));
            source = None;
        }
    let family = match (destination, gateway, source, normalized_family) {
        (Some(address), _, _, _) => address_family(address),
        (None, Some(address), _, _) => address_family(address),
        (None, None, Some(address), _) => address_family(address),
        (None, None, None, Some(family)) => family,
        (None, None, None, None) => {
            return Err(os_kernel::Error::invalid(
                "static route must set destination or gateway",
            ));
        }
    };
    let table = route_table_from_config(route.table);
    let spec = RouteSpec {
        destination,
        prefix_len,
        source,
        gateway,
        out_link: link_name.to_string(),
        family,
        metric: if route.metric == 0 {
            os_network_domain::DEFAULT_ROUTE_METRIC
        } else {
            route.metric
        },
        mtu: route.mtu,
        table,
        protocol: RouteProtocol::Static,
        layer: ConfigLayer::Configuration,
    };
    spec.validate()?;
    Ok(spec)
}

fn route_table_from_config(table: u32) -> RouteTable {
    match table {
        0 | 254 => RouteTable::Main,
        255 => RouteTable::Local,
        other => RouteTable::Custom(other),
    }
}

/// Deterministic LinkConfig/VLANConfig projection fingerprint for differential
/// tests.
///
/// This intentionally exposes only a string contract rather than the private
/// controller projection type, so difftests can prove source-visible fields are
/// either represented in Rust resources or rejected fail-visible.
pub fn machine_config_link_projection_fingerprint(contents: &str) -> Result<String, String> {
    try_machine_config_link_projection(contents)
        .map(render_link_projection_fingerprint)
        .map_err(|e| e.to_string())
}

/// Deterministic machine-config operator-spec projection fingerprint for
/// differential tests.
pub fn machine_config_operator_specs_fingerprint(contents: &str) -> Result<String, String> {
    try_machine_config_operator_specs(contents)
        .map(|specs| {
            let mut operators = specs
                .iter()
                .map(operator_spec_fingerprint)
                .collect::<Vec<_>>();
            operators.sort();
            projection_section(operators)
        })
        .map_err(|e| e.to_string())
}

/// Deterministic fingerprint of the source-guided default LinkStatus projection.
///
/// This is a difftest-friendly pure surface for the default LinkConfig and
/// default DHCPv4 operator synthesis that the controllers perform from observed
/// link status resources. It intentionally reports only default-produced
/// outputs; explicit machine-config projections are covered by their own
/// difftests.
pub fn default_dhcp_link_status_projection_fingerprint(
    contents: Option<&str>,
    statuses: &[LinkStatus],
) -> Result<String, String> {
    try_default_dhcp_link_status_projection_fingerprint(contents, statuses)
        .map_err(|e| e.to_string())
}

fn try_default_dhcp_link_status_projection_fingerprint(
    contents: Option<&str>,
    statuses: &[LinkStatus],
) -> ReconcileResult<String> {
    let mut links = Vec::new();
    let mut operators = Vec::new();

    if machine_config_run_default_dhcp_operators(contents)? {
        let configured_links = machine_config_configured_links(contents)?;
        for status in statuses {
            if status.physical() && !link_status_matches_any(status, &configured_links) {
                let mut link = LinkSpec::physical(status.name.clone(), ConfigLayer::Default);
                link.mtu = 0;
                links.push(format!(
                    "source=default/{},{}",
                    status.name,
                    link_spec_fingerprint(&link)
                ));
            }
            if status.default_dhcp4_candidate()
                && !link_status_matches_any(status, &configured_links)
            {
                operators.push(operator_spec_fingerprint(&default_dhcp4_operator_spec(
                    &status.name,
                )));
            }
        }
    }

    links.sort();
    operators.sort();
    Ok(format!(
        "links={};operators={}",
        if links.is_empty() {
            "-".to_string()
        } else {
            links.join("|")
        },
        if operators.is_empty() {
            "-".to_string()
        } else {
            operators.join("|")
        },
    ))
}

fn render_link_projection_fingerprint(projection: LinkProjection) -> String {
    let mut links = projection
        .links
        .into_iter()
        .map(|(source, spec)| format!("source={source},{}", link_spec_fingerprint(&spec)))
        .collect::<Vec<_>>();
    links.sort();

    let mut addresses = projection
        .addresses
        .into_iter()
        .map(|(source, spec)| format!("source={source},{}", address_spec_fingerprint(&spec)))
        .collect::<Vec<_>>();
    addresses.sort();

    let mut routes = projection
        .routes
        .into_iter()
        .map(|(source, spec)| format!("source={source},{}", route_spec_fingerprint(&spec)))
        .collect::<Vec<_>>();
    routes.sort();

    format!(
        "links={};addresses={};routes={}",
        projection_section(links),
        projection_section(addresses),
        projection_section(routes),
    )
}

fn projection_section(items: Vec<String>) -> String {
    if items.is_empty() {
        "-".to_string()
    } else {
        items.join("|")
    }
}

fn parse_cidr_core(cidr: &str) -> CoreResult<(NodeAddress, u8)> {
    let (address, prefix) = cidr
        .trim()
        .split_once('/')
        .ok_or_else(|| os_kernel::Error::invalid("CIDR value is missing '/'"))?;
    let address = NodeAddress::parse(address.trim())?;
    let prefix_len = prefix
        .trim()
        .parse::<u8>()
        .map_err(|_| os_kernel::Error::invalid(format!("invalid CIDR prefix '{prefix}'")))?;
    Ok((address, prefix_len))
}

fn address_family(address: NodeAddress) -> AddressFamily {
    if address.is_v4() {
        AddressFamily::Inet4
    } else {
        AddressFamily::Inet6
    }
}

fn link_config_source(name: &str) -> String {
    format!("configuration/linkconfig/{name}")
}

fn vlan_config_source(name: &str) -> String {
    format!("configuration/vlanconfig/{name}")
}

fn push_dhcp_operator_specs(
    out: &mut Vec<OperatorSpec>,
    link_name: &str,
    options: &DhcpOptions,
) -> CoreResult<()> {
    if options.ipv4() {
        let mut op = OperatorSpec::dhcp4(link_name);
        op.route_metric = options.route_metric_or(os_network_domain::DEFAULT_ROUTE_METRIC);
        out.push(op);
    }

    if options.ipv6() {
        let mut op = OperatorSpec::dhcp6(link_name);
        op.route_metric = options.route_metric_or(os_network_domain::DEFAULT_ROUTE_METRIC);
        if !options.duid_v6.is_empty() {
            op = op.with_client_identifier(ClientIdentifierSpec::duid(parse_config_hex_bytes(
                &options.duid_v6,
            )?));
        }
        op.validate()?;
        out.push(op);
    }

    Ok(())
}

fn default_dhcp4_operator_spec(link_name: &str) -> OperatorSpec {
    // Source-guided helper for Talos' default DHCPv4 path. Callers must prove a
    // physical LinkStatus exists and the link is not explicitly configured.
    let mut op = OperatorSpec::dhcp4(link_name);
    op.layer = ConfigLayer::Default;
    op.client_identifier = ClientIdentifierSpec::mac();
    op
}

fn parse_config_hex_bytes(s: &str) -> CoreResult<Vec<u8>> {
    parse_hex_bytes(s).map_err(|e| os_kernel::Error::invalid(e.to_string()))
}

fn operator_route_resource_id(source_operator: &str, spec: &RouteSpec) -> String {
    format!("{source_operator}/{}", spec.id())
}

fn parse_operator_spec(fp: &str) -> ReconcileResult<OperatorSpec> {
    let fields = fields(fp);
    let kind = parse_operator_kind(required(&fields, "kind")?)?;
    let link = required(&fields, "link")?.to_string();
    let mut spec = match kind {
        OperatorKind::Dhcp4 => OperatorSpec::dhcp4(link),
        OperatorKind::Dhcp6 => OperatorSpec::dhcp6(link),
        OperatorKind::Vip => OperatorSpec {
            kind: OperatorKind::Vip,
            link_name: link,
            require_up: true,
            route_metric: os_network_domain::DEFAULT_ROUTE_METRIC,
            skip_hostname_request: false,
            client_identifier: ClientIdentifierSpec::none(),
            layer: ConfigLayer::Configuration,
        },
    };
    spec.require_up = parse_bool(required(&fields, "require_up")?)?;
    spec.route_metric = parse_u32(required(&fields, "route_metric")?, "route_metric")?;
    spec.skip_hostname_request = parse_bool(required(&fields, "skip_hostname")?)?;
    spec.client_identifier = parse_client_identifier(
        required(&fields, "client")?,
        fields.get("duid").map_or("", String::as_str),
    )?;
    spec.layer = parse_layer(required(&fields, "layer")?)?;
    Ok(spec)
}

fn parse_link_spec(fp: &str) -> ReconcileResult<LinkSpec> {
    let fields = fields(fp);
    let name = required(&fields, "name")?.to_string();
    let up = parse_bool(required(&fields, "up")?)?;
    let mtu = parse_u32(required(&fields, "mtu")?, "mtu")?;
    let multicast = match fields.get("multicast").map(String::as_str).unwrap_or("") {
        "" => None,
        value => Some(parse_bool(value)?),
    };
    let layer = parse_layer(required(&fields, "layer")?)?;
    let kind = match required(&fields, "kind")? {
        "physical" => LinkKind::Physical,
        "bond" => LinkKind::Bond {
            members: parse_string_list(fields.get("members").map_or("", String::as_str)),
            mode: parse_bond_mode(fields.get("mode").map_or("", String::as_str))?,
        },
        "bridge" => LinkKind::Bridge {
            members: parse_string_list(fields.get("members").map_or("", String::as_str)),
        },
        "vlan" => LinkKind::Vlan {
            parent: required(&fields, "parent")?.to_string(),
            vlan_id: parse_u16(required(&fields, "vlan_id")?, "vlan_id")?,
            protocol: parse_vlan_protocol(required(&fields, "vlan_protocol")?)?,
        },
        "dummy" => LinkKind::Dummy,
        other => {
            return Err(ReconcileError::Invalid(format!(
                "unknown link kind '{other}'"
            )));
        }
    };
    let spec = LinkSpec {
        name,
        up,
        mtu,
        multicast,
        kind,
        layer,
    };
    spec.validate()
        .map_err(|e| ReconcileError::Invalid(e.to_string()))?;
    Ok(spec)
}

fn parse_link_status(fp: &str) -> ReconcileResult<LinkStatus> {
    let fields = fields(fp);
    let link_type = match required(&fields, "type")? {
        "ether" => LinkType::Ether,
        other => LinkType::Other(other.to_string()),
    };
    let status = LinkStatus {
        name: required(&fields, "name")?.to_string(),
        link_type,
        kind: required(&fields, "kind")?.to_string(),
        aliases: parse_link_aliases(required(&fields, "aliases")?)?,
        admin_up: parse_bool(required(&fields, "admin_up")?)?,
        oper_state: parse_oper_state(required(&fields, "oper_state")?)?,
        carrier: parse_bool(required(&fields, "carrier")?)?,
        hardware_addr: parse_mac(required(&fields, "mac")?)?,
        mtu: parse_u32(required(&fields, "mtu")?, "mtu")?,
    };
    Ok(status)
}

fn parse_link_aliases(s: &str) -> ReconcileResult<Vec<String>> {
    if s.is_empty() || s == "-" {
        return Ok(Vec::new());
    }

    let mut aliases = Vec::new();
    for alias in s.split(',') {
        let alias = alias.trim();
        if alias.is_empty() {
            return Err(ReconcileError::Invalid(
                "link status aliases must not contain empty entries".to_string(),
            ));
        }
        if alias.contains(';') || alias.contains('=') {
            return Err(ReconcileError::Invalid(format!(
                "invalid link status alias '{alias}'"
            )));
        }
        aliases.push(alias.to_string());
    }
    Ok(aliases)
}

fn parse_oper_state(s: &str) -> ReconcileResult<os_network_domain::OperState> {
    match s {
        "up" => Ok(os_network_domain::OperState::Up),
        "down" => Ok(os_network_domain::OperState::Down),
        "unknown" => Ok(os_network_domain::OperState::Unknown),
        other => Err(ReconcileError::Invalid(format!(
            "unknown link oper_state '{other}'"
        ))),
    }
}

fn parse_mac(s: &str) -> ReconcileResult<[u8; 6]> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 6 {
        return Err(ReconcileError::Invalid(format!(
            "invalid MAC address '{s}'"
        )));
    }
    let mut out = [0u8; 6];
    for (idx, part) in parts.iter().enumerate() {
        if part.len() != 2 {
            return Err(ReconcileError::Invalid(format!(
                "invalid MAC address '{s}'"
            )));
        }
        out[idx] = u8::from_str_radix(part, 16)
            .map_err(|_| ReconcileError::Invalid(format!("invalid MAC address '{s}'")))?;
    }
    Ok(out)
}

fn parse_vlan_protocol(s: &str) -> ReconcileResult<VlanProtocol> {
    match s {
        "802.1q" => Ok(VlanProtocol::Ieee8021q),
        "802.1ad" => Ok(VlanProtocol::Ieee8021ad),
        "" => Err(ReconcileError::Invalid(
            "missing VLAN protocol for vlan link spec".to_string(),
        )),
        other => Err(ReconcileError::Invalid(format!(
            "unknown VLAN protocol '{other}'"
        ))),
    }
}

fn parse_operator_result(fp: &str) -> ReconcileResult<OperatorResult> {
    let fields = fields(fp);
    let address = parse_address(required(&fields, "address")?)?;
    let prefix_len = parse_u8(required(&fields, "prefix")?, "prefix")?;
    let gateway = match fields
        .get("gateway")
        .map(String::as_str)
        .unwrap_or_default()
    {
        "" => None,
        s => Some(parse_address(s)?),
    };
    let dns_servers = parse_address_list(fields.get("dns").map_or("", String::as_str))?;
    let hostname = match fields
        .get("hostname")
        .map(String::as_str)
        .unwrap_or_default()
    {
        "" => None,
        s => Some(s.to_string()),
    };
    let search_domains = parse_string_list(fields.get("search").map_or("", String::as_str));
    Ok(OperatorResult {
        address,
        prefix_len,
        gateway,
        dns_servers,
        hostname,
        search_domains,
    })
}

fn parse_address_spec(fp: &str) -> ReconcileResult<AddressSpec> {
    let fields = fields(fp);
    let address = parse_address(required(&fields, "address")?)?;
    let prefix_len = parse_u8(required(&fields, "prefix")?, "prefix")?;
    let link_name = required(&fields, "link")?.to_string();
    let family = parse_address_family(required(&fields, "family")?)?;
    let scope = parse_scope(required(&fields, "scope")?)?;
    let flags = parse_address_flags(required(&fields, "flags")?)?;
    let priority = parse_u32(required(&fields, "priority")?, "priority")?;
    let layer = parse_layer(required(&fields, "layer")?)?;
    let spec = AddressSpec {
        address,
        prefix_len,
        link_name,
        family,
        scope,
        flags,
        priority,
        layer,
    };
    spec.validate()
        .map_err(|e| ReconcileError::Invalid(e.to_string()))?;
    Ok(spec)
}

fn parse_route_spec(fp: &str) -> ReconcileResult<RouteSpec> {
    let fields = fields(fp);
    let destination = match required(&fields, "destination")? {
        "default" | "" => None,
        s => Some(parse_address(s)?),
    };
    let prefix_len = parse_u8(required(&fields, "prefix")?, "prefix")?;
    let source = match fields.get("source").map(String::as_str).unwrap_or_default() {
        "" => None,
        s => Some(parse_address(s)?),
    };
    let gateway = match fields
        .get("gateway")
        .map(String::as_str)
        .unwrap_or_default()
    {
        "" => None,
        s => Some(parse_address(s)?),
    };
    let out_link = required(&fields, "out_link")?.to_string();
    let family = parse_address_family(required(&fields, "family")?)?;
    let metric = parse_u32(required(&fields, "metric")?, "metric")?;
    let mtu = parse_u32(required(&fields, "mtu")?, "mtu")?;
    let table = parse_route_table(required(&fields, "table")?)?;
    let protocol = parse_route_protocol(required(&fields, "protocol")?)?;
    let layer = parse_layer(required(&fields, "layer")?)?;
    let spec = RouteSpec {
        destination,
        prefix_len,
        source,
        gateway,
        out_link,
        family,
        metric,
        mtu,
        table,
        protocol,
        layer,
    };
    spec.validate()
        .map_err(|e| ReconcileError::Invalid(e.to_string()))?;
    Ok(spec)
}

fn parse_hostname_spec(fp: &str) -> ReconcileResult<HostnameSpec> {
    let fields = fields(fp);
    let hostname = required(&fields, "hostname")?;
    let domain = fields.get("domain").map(String::as_str).unwrap_or_default();
    let layer = parse_layer(required(&fields, "layer")?)?;
    let spec = if domain.is_empty() {
        HostnameSpec::new(hostname, layer)
    } else {
        HostnameSpec::with_domain(hostname, domain, layer)
    };
    spec.map_err(|e| ReconcileError::Invalid(e.to_string()))
}

fn parse_resolver_spec(fp: &str) -> ReconcileResult<ResolverSpec> {
    let fields = fields(fp);
    let servers = parse_address_list(fields.get("servers").map_or("", String::as_str))?;
    let search_domains = parse_string_list(fields.get("search").map_or("", String::as_str));
    let layer = parse_layer(required(&fields, "layer")?)?;
    ResolverSpec::new_with_search(servers, search_domains, layer)
        .map_err(|e| ReconcileError::Invalid(e.to_string()))
}

fn fields(fp: &str) -> BTreeMap<String, String> {
    fp.split(';')
        .filter_map(|part| {
            let (k, v) = part.split_once('=')?;
            Some((k.to_string(), v.to_string()))
        })
        .collect()
}

fn required<'a>(fields: &'a BTreeMap<String, String>, key: &str) -> ReconcileResult<&'a str> {
    fields
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| ReconcileError::Invalid(format!("missing operator field '{key}'")))
}

fn fingerprint_field(fp: &str, key: &str) -> Option<String> {
    fp.split(';').find_map(|part| {
        let (field_key, value) = part.split_once('=')?;
        (field_key == key).then(|| value.to_string())
    })
}

fn parse_operator_kind(s: &str) -> ReconcileResult<OperatorKind> {
    match s {
        "dhcp4" => Ok(OperatorKind::Dhcp4),
        "dhcp6" => Ok(OperatorKind::Dhcp6),
        "vip" => Ok(OperatorKind::Vip),
        other => Err(ReconcileError::Invalid(format!(
            "unknown operator kind '{other}'"
        ))),
    }
}

fn parse_layer(s: &str) -> ReconcileResult<ConfigLayer> {
    match s {
        "default" => Ok(ConfigLayer::Default),
        "cmdline" => Ok(ConfigLayer::Cmdline),
        "platform" => Ok(ConfigLayer::Platform),
        "operator" => Ok(ConfigLayer::Operator),
        "configuration" => Ok(ConfigLayer::Configuration),
        other => Err(ReconcileError::Invalid(format!(
            "unknown config layer '{other}'"
        ))),
    }
}

fn parse_client_identifier(policy: &str, duid_hex: &str) -> ReconcileResult<ClientIdentifierSpec> {
    match policy {
        "none" => Ok(ClientIdentifierSpec::none()),
        "mac" => Ok(ClientIdentifierSpec::mac()),
        "duid" => Ok(ClientIdentifierSpec::duid(parse_hex_bytes(duid_hex)?)),
        other => Err(ReconcileError::Invalid(format!(
            "unknown client identifier '{other}'"
        ))),
    }
}

fn parse_bond_mode(s: &str) -> ReconcileResult<BondMode> {
    match s {
        "active-backup" => Ok(BondMode::ActiveBackup),
        "balance-rr" => Ok(BondMode::BalanceRr),
        "802.3ad" => Ok(BondMode::Lacp),
        "" => Err(ReconcileError::Invalid("missing bond mode".into())),
        other => Err(ReconcileError::Invalid(format!(
            "unknown bond mode '{other}'"
        ))),
    }
}

fn parse_address_family(s: &str) -> ReconcileResult<AddressFamily> {
    match s {
        "inet4" => Ok(AddressFamily::Inet4),
        "inet6" => Ok(AddressFamily::Inet6),
        other => Err(ReconcileError::Invalid(format!(
            "unknown address family '{other}'"
        ))),
    }
}

fn parse_scope(s: &str) -> ReconcileResult<Scope> {
    match s {
        "global" => Ok(Scope::Global),
        "link" => Ok(Scope::Link),
        "host" => Ok(Scope::Host),
        other => Err(ReconcileError::Invalid(format!(
            "unknown address scope '{other}'"
        ))),
    }
}

fn parse_address_flags(s: &str) -> ReconcileResult<AddressFlags> {
    match s {
        "permanent" => Ok(AddressFlags::permanent()),
        "temporary" => Ok(AddressFlags {
            temporary: true,
            ..Default::default()
        }),
        "tentative" => Ok(AddressFlags {
            tentative: true,
            ..Default::default()
        }),
        "none" => Ok(AddressFlags::default()),
        "mixed" => Err(ReconcileError::Invalid(
            "mixed address flags cannot be parsed from compact fingerprint".into(),
        )),
        other => Err(ReconcileError::Invalid(format!(
            "unknown address flags '{other}'"
        ))),
    }
}

fn parse_route_table(s: &str) -> ReconcileResult<RouteTable> {
    match parse_u32(s, "table")? {
        254 => Ok(RouteTable::Main),
        255 => Ok(RouteTable::Local),
        id => Ok(RouteTable::Custom(id)),
    }
}

fn parse_route_protocol(s: &str) -> ReconcileResult<RouteProtocol> {
    match s {
        "static" => Ok(RouteProtocol::Static),
        "boot" => Ok(RouteProtocol::Boot),
        "kernel" => Ok(RouteProtocol::Kernel),
        "dhcp" => Ok(RouteProtocol::Dhcp),
        other => Err(ReconcileError::Invalid(format!(
            "unknown route protocol '{other}'"
        ))),
    }
}

fn parse_bool(s: &str) -> ReconcileResult<bool> {
    match s {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(ReconcileError::Invalid(format!("invalid bool '{other}'"))),
    }
}

fn parse_u32(s: &str, field: &str) -> ReconcileResult<u32> {
    s.parse()
        .map_err(|_| ReconcileError::Invalid(format!("invalid {field} '{s}'")))
}

fn parse_u8(s: &str, field: &str) -> ReconcileResult<u8> {
    s.parse()
        .map_err(|_| ReconcileError::Invalid(format!("invalid {field} '{s}'")))
}

fn parse_u16(s: &str, field: &str) -> ReconcileResult<u16> {
    s.parse()
        .map_err(|_| ReconcileError::Invalid(format!("invalid {field} '{s}'")))
}

fn parse_address(s: &str) -> ReconcileResult<NodeAddress> {
    NodeAddress::parse(s).map_err(|e| ReconcileError::Invalid(e.to_string()))
}

fn parse_address_list(s: &str) -> ReconcileResult<Vec<NodeAddress>> {
    if s.is_empty() {
        return Ok(Vec::new());
    }
    s.split(',').map(parse_address).collect()
}

fn parse_string_list(s: &str) -> Vec<String> {
    if s.is_empty() {
        Vec::new()
    } else {
        s.split(',').map(str::to_string).collect()
    }
}

fn parse_hex_bytes(s: &str) -> ReconcileResult<Vec<u8>> {
    if s.is_empty() {
        return Ok(Vec::new());
    }
    if !s.len().is_multiple_of(2) {
        return Err(ReconcileError::Invalid(
            "hex byte string has odd length".into(),
        ));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| {
                ReconcileError::Invalid(format!("invalid hex byte '{}';", &s[i..i + 2]))
            })
        })
        .collect()
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
}

fn client_identifier_as_str(spec: &ClientIdentifierSpec) -> &'static str {
    match spec.client_identifier {
        os_network_domain::nethelpers::ClientIdentifier::None => "none",
        os_network_domain::nethelpers::ClientIdentifier::Mac => "mac",
        os_network_domain::nethelpers::ClientIdentifier::Duid => "duid",
    }
}

fn bond_mode_as_str(mode: BondMode) -> &'static str {
    match mode {
        BondMode::ActiveBackup => "active-backup",
        BondMode::BalanceRr => "balance-rr",
        BondMode::Lacp => "802.3ad",
    }
}

fn oper_state_as_str(state: os_network_domain::OperState) -> &'static str {
    match state {
        os_network_domain::OperState::Up => "up",
        os_network_domain::OperState::Down => "down",
        os_network_domain::OperState::Unknown => "unknown",
    }
}

fn option_address(address: Option<NodeAddress>) -> Option<String> {
    address.map(|addr| addr.to_string())
}

fn join_addresses(addresses: &[NodeAddress]) -> String {
    addresses
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn address_family_as_str(family: AddressFamily) -> &'static str {
    match family {
        AddressFamily::Inet4 => "inet4",
        AddressFamily::Inet6 => "inet6",
    }
}

fn scope_as_str(scope: Scope) -> &'static str {
    match scope {
        Scope::Global => "global",
        Scope::Link => "link",
        Scope::Host => "host",
    }
}

fn address_flags_as_str(flags: AddressFlags) -> &'static str {
    match (flags.permanent, flags.temporary, flags.tentative) {
        (true, false, false) => "permanent",
        (false, true, false) => "temporary",
        (false, false, true) => "tentative",
        (false, false, false) => "none",
        _ => "mixed",
    }
}

fn route_table_as_u32(table: RouteTable) -> u32 {
    table.table_id()
}

fn route_protocol_as_str(protocol: RouteProtocol) -> &'static str {
    match protocol {
        RouteProtocol::Static => "static",
        RouteProtocol::Boot => "boot",
        RouteProtocol::Kernel => "kernel",
        RouteProtocol::Dhcp => "dhcp",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::ControllerRuntime;
    use std::cell::RefCell;
    use std::rc::Rc;
    use os_cosi_domain::State;

    fn v4(s: &str) -> NodeAddress {
        NodeAddress::parse_v4(s).unwrap()
    }

    fn v6(s: &str) -> NodeAddress {
        NodeAddress::parse_v6(s).unwrap()
    }

    fn machine_config_with_dhcpv4(metric: u32) -> String {
        format!(
            "\
version: v1alpha1
machine:
  type: worker
  token: tok
  network:
    interfaces:
      - interface: eth0
        dhcp: true
        dhcpOptions:
          routeMetric: {metric}
          ipv4: true
          ipv6: false
cluster:
  controlPlane:
    endpoint: https://10.0.0.1:6443
"
        )
    }

    fn machine_config_dual_stack() -> &'static str {
        "\
version: v1alpha1
machine:
  type: worker
  token: tok
  network:
    interfaces:
      - interface: eth0
        dhcp: true
        dhcpOptions:
          routeMetric: 2048
          ipv4: true
          ipv6: false
---
apiVersion: v1alpha1
kind: DHCPv6Config
name: eth0
routeMetric: 4096
ignoreHostname: true
clientIdentifier: duid
duidRaw: 00:03:00:01:aa:bb:cc:dd:ee:ff
"
    }

    fn full_v6_result(address: &str) -> OperatorResult {
        OperatorResult {
            address: v6(address),
            prefix_len: 64,
            gateway: Some(v6("2001:db8::1")),
            dns_servers: vec![v6("2001:4860:4860::8888")],
            hostname: Some("leased-v6.example.com".to_string()),
            search_domains: vec!["example.com".to_string()],
        }
    }

    fn full_result(address: &str) -> OperatorResult {
        OperatorResult {
            address: v4(address),
            prefix_len: 24,
            gateway: Some(v4("10.0.0.1")),
            dns_servers: vec![v4("10.0.0.2"), v4("8.8.8.8")],
            hostname: Some("leased-host.example.com".to_string()),
            search_domains: vec!["example.com".to_string()],
        }
    }

    fn resolver_source(
        source: &str,
        servers: Vec<NodeAddress>,
        layer: ConfigLayer,
    ) -> Box<dyn Resource> {
        Box::new(ResolverSpecResource::new(
            source,
            ResolverSpec::new(servers, layer).unwrap(),
        ))
    }

    fn resolver_source_with_search(
        source: &str,
        servers: Vec<NodeAddress>,
        search_domains: Vec<String>,
        layer: ConfigLayer,
    ) -> Box<dyn Resource> {
        Box::new(ResolverSpecResource::new(
            source,
            ResolverSpec::new_with_search(servers, search_domains, layer).unwrap(),
        ))
    }

    fn address_source(
        source: &str,
        address: &str,
        prefix_len: u8,
        link: &str,
        layer: ConfigLayer,
    ) -> Box<dyn Resource> {
        Box::new(AddressSpecResource::new(
            source,
            AddressSpec::new(v4(address), prefix_len, link, layer).unwrap(),
        ))
    }

    fn route_source(
        source: &str,
        gateway: &str,
        layer: ConfigLayer,
        metric: u32,
    ) -> Box<dyn Resource> {
        let mut route = RouteSpec::default_via(v4(gateway), "eth0", layer).unwrap();
        route.metric = metric;
        route.protocol = RouteProtocol::Dhcp;
        Box::new(RouteSpecResource::new(source, route))
    }

    fn hostname_source(
        source: &str,
        hostname: &str,
        domain: Option<&str>,
        layer: ConfigLayer,
    ) -> Box<dyn Resource> {
        let spec = match domain {
            Some(domain) => HostnameSpec::with_domain(hostname, domain, layer).unwrap(),
            None => HostnameSpec::new(hostname, layer).unwrap(),
        };
        Box::new(HostnameSpecResource::new(source, spec))
    }

    fn link_source(source: &str, name: &str, mtu: u32, layer: ConfigLayer) -> Box<dyn Resource> {
        let mut spec = LinkSpec::physical(name, layer);
        spec.mtu = mtu;
        Box::new(LinkSpecResource::new(source, spec))
    }

    fn physical_status(name: &str) -> LinkStatus {
        LinkStatus {
            name: name.to_string(),
            link_type: LinkType::Ether,
            kind: String::new(),
            aliases: Vec::new(),
            admin_up: false,
            oper_state: os_network_domain::OperState::Down,
            carrier: false,
            hardware_addr: [0x02, 0, 0, 0, 0, 1],
            mtu: 1500,
        }
    }

    fn nonphysical_status(name: &str, kind: &str) -> LinkStatus {
        LinkStatus {
            kind: kind.to_string(),
            aliases: Vec::new(),
            ..physical_status(name)
        }
    }

    #[derive(Debug, Clone)]
    struct RawTestResource {
        meta: Metadata,
        fingerprint: String,
    }

    impl RawTestResource {
        fn link_status(id: &str, fingerprint: &str) -> Self {
            RawTestResource {
                meta: Metadata::new(NETWORK_NS, LINK_STATUS_KIND, ResourceId::new(id).unwrap()),
                fingerprint: fingerprint.to_string(),
            }
        }
    }

    impl Resource for RawTestResource {
        fn metadata(&self) -> &Metadata {
            &self.meta
        }

        fn metadata_mut(&mut self) -> &mut Metadata {
            &mut self.meta
        }

        fn spec_fingerprint(&self) -> String {
            self.fingerprint.clone()
        }

        fn clone_box(&self) -> Box<dyn Resource> {
            Box::new(self.clone())
        }
    }

    fn destroy_resource(rt: &mut ControllerRuntime, key: &str) {
        let version = rt.state().get(key).unwrap().metadata().version();
        let teardown = rt.state_mut().teardown(key, version).unwrap();
        rt.state_mut().destroy(key, teardown).unwrap();
    }

    fn runtime_with_addresses(resources: Vec<Box<dyn Resource>>) -> ControllerRuntime {
        let mut rt = ControllerRuntime::new();
        for resource in resources {
            rt.state_mut().create(resource).unwrap();
        }
        rt.register(Box::new(AddressMergeController::new()));
        rt
    }

    fn runtime_with_routes(resources: Vec<Box<dyn Resource>>) -> ControllerRuntime {
        let mut rt = ControllerRuntime::new();
        for resource in resources {
            rt.state_mut().create(resource).unwrap();
        }
        rt.register(Box::new(RouteMergeController::new()));
        rt
    }

    fn runtime_with_hostnames(resources: Vec<Box<dyn Resource>>) -> ControllerRuntime {
        let mut rt = ControllerRuntime::new();
        for resource in resources {
            rt.state_mut().create(resource).unwrap();
        }
        rt.register(Box::new(HostnameMergeController::new()));
        rt
    }

    fn runtime_with_links(resources: Vec<Box<dyn Resource>>) -> ControllerRuntime {
        let mut rt = ControllerRuntime::new();
        for resource in resources {
            rt.state_mut().create(resource).unwrap();
        }
        rt.register(Box::new(LinkMergeController::new()));
        rt
    }

    fn runtime_with_resolvers(resources: Vec<Box<dyn Resource>>) -> ControllerRuntime {
        let mut rt = ControllerRuntime::new();
        for resource in resources {
            rt.state_mut().create(resource).unwrap();
        }
        rt.register(Box::new(ResolverMergeController::new()));
        rt
    }

    fn runtime_with_operator(spec: OperatorSpec, result: OperatorResult) -> ControllerRuntime {
        let operator_id = spec.id();
        let mut rt = ControllerRuntime::new();
        rt.state_mut()
            .create(Box::new(OperatorSpecResource::new(spec)))
            .unwrap();
        rt.state_mut()
            .create(Box::new(OperatorResultResource::new(operator_id, result)))
            .unwrap();
        rt.register(Box::new(OperatorResultBridgeController::new()));
        rt
    }

    fn runtime_with_machine_config(contents: &str) -> ControllerRuntime {
        let mut rt = ControllerRuntime::new();
        rt.state_mut()
            .create(Box::new(MachineConfigDocument::new(contents)))
            .unwrap();
        rt.register(Box::new(OperatorConfigController::new()));
        rt.register(Box::new(OperatorMergeController::new()));
        rt
    }

    fn runtime_with_default_dhcp_inputs(
        machine_config: Option<&str>,
        statuses: Vec<LinkStatus>,
        link_specs: Vec<Box<dyn Resource>>,
    ) -> ControllerRuntime {
        let mut rt = ControllerRuntime::new();
        if let Some(contents) = machine_config {
            rt.state_mut()
                .create(Box::new(MachineConfigDocument::new(contents)))
                .unwrap();
        }
        for status in statuses {
            rt.state_mut()
                .create(Box::new(LinkStatusResource::new(status).unwrap()))
                .unwrap();
        }
        for resource in link_specs {
            rt.state_mut().create(resource).unwrap();
        }
        rt.register(Box::new(OperatorConfigController::new()));
        rt.register(Box::new(OperatorMergeController::new()));
        rt
    }

    fn runtime_with_link_status_source(statuses: Vec<LinkStatus>) -> ControllerRuntime {
        let mut rt = ControllerRuntime::new();
        rt.register(Box::new(LinkStatusSourceController::new_with_source(
            move || Ok(statuses.clone()),
        )));
        rt.register(Box::new(LinkConfigController::new()));
        rt.register(Box::new(LinkMergeController::new()));
        rt.register(Box::new(OperatorConfigController::new()));
        rt.register(Box::new(OperatorMergeController::new()));
        rt
    }

    fn runtime_with_link_status_source_snapshots(
        snapshots: Vec<Vec<LinkStatus>>,
    ) -> ControllerRuntime {
        let snapshots = Rc::new(RefCell::new(snapshots));
        let mut rt = ControllerRuntime::new();
        rt.register(Box::new(LinkStatusSourceController::new_with_source({
            let snapshots = Rc::clone(&snapshots);
            move || {
                let mut snapshots = snapshots.borrow_mut();
                if snapshots.is_empty() {
                    Ok(Vec::new())
                } else {
                    Ok(snapshots.remove(0))
                }
            }
        })));
        rt
    }

    fn runtime_with_link_status_config(
        machine_config: Option<&str>,
        statuses: Vec<LinkStatus>,
    ) -> ControllerRuntime {
        let mut rt = ControllerRuntime::new();
        if let Some(contents) = machine_config {
            rt.state_mut()
                .create(Box::new(MachineConfigDocument::new(contents)))
                .unwrap();
        }
        for status in statuses {
            rt.state_mut()
                .create(Box::new(LinkStatusResource::new(status).unwrap()))
                .unwrap();
        }
        rt.register(Box::new(LinkConfigController::new()));
        rt.register(Box::new(LinkMergeController::new()));
        rt
    }

    fn runtime_with_link_status_config_and_links(
        machine_config: Option<&str>,
        statuses: Vec<LinkStatus>,
        link_specs: Vec<Box<dyn Resource>>,
    ) -> ControllerRuntime {
        let mut rt = ControllerRuntime::new();
        if let Some(contents) = machine_config {
            rt.state_mut()
                .create(Box::new(MachineConfigDocument::new(contents)))
                .unwrap();
        }
        for status in statuses {
            rt.state_mut()
                .create(Box::new(LinkStatusResource::new(status).unwrap()))
                .unwrap();
        }
        for resource in link_specs {
            rt.state_mut().create(resource).unwrap();
        }
        rt.register(Box::new(LinkConfigController::new()));
        rt.register(Box::new(LinkMergeController::new()));
        rt
    }

    fn runtime_with_resolver_config_doc(contents: &str) -> ControllerRuntime {
        let mut rt = ControllerRuntime::new();
        rt.state_mut()
            .create(Box::new(MachineConfigDocument::new(contents)))
            .unwrap();
        rt.register(Box::new(ResolverConfigController::new()));
        rt.register(Box::new(ResolverMergeController::new()));
        rt
    }

    fn runtime_with_link_config_doc(contents: &str) -> ControllerRuntime {
        let mut rt = ControllerRuntime::new();
        rt.state_mut()
            .create(Box::new(MachineConfigDocument::new(contents)))
            .unwrap();
        rt.register(Box::new(LinkConfigController::new()));
        rt.register(Box::new(LinkMergeController::new()));
        rt.register(Box::new(AddressMergeController::new()));
        rt.register(Box::new(RouteMergeController::new()));
        rt
    }

    #[test]
    fn resource_fingerprints_round_trip_without_downcast() {
        let mut spec = OperatorSpec::dhcp6("eth0")
            .with_skip_hostname_request(true)
            .with_client_identifier(ClientIdentifierSpec::duid(vec![0, 1, 2, 3]));
        spec.route_metric = 2048;
        let resource = OperatorSpecResource::new(spec.clone());
        let parsed = parse_operator_spec(&resource.spec_fingerprint()).unwrap();
        assert_eq!(parsed, spec);

        let result = full_result("10.0.0.50");
        let result_resource = OperatorResultResource::new("dhcp6/eth0", result.clone());
        let parsed = parse_operator_result(&result_resource.spec_fingerprint()).unwrap();
        assert_eq!(parsed, result);

        let mut link = LinkSpec::physical("eth0", ConfigLayer::Configuration);
        link.up = false;
        link.mtu = 9000;
        let resource = LinkSpecResource::new("configuration/linkconfig/eth0", link.clone());
        let parsed = parse_link_spec(&resource.spec_fingerprint()).unwrap();
        assert_eq!(parsed, link);

        let status = physical_status("eth0");
        let resource = LinkStatusResource::new(status.clone()).unwrap();
        let parsed = parse_link_status(&resource.spec_fingerprint()).unwrap();
        assert_eq!(parsed, status);
    }

    #[test]
    fn link_status_publication_controller_publishes_live_kernel_status_resources() {
        let mut status = physical_status("eth0");
        status.admin_up = true;
        status.oper_state = os_network_domain::OperState::Up;
        status.carrier = true;
        status.aliases = vec!["lan0".to_string()];
        let mut rt = runtime_with_link_status_source(vec![status]);

        rt.run_until_stable(5).unwrap();

        let source = rt
            .state()
            .get("network/LinkStatuses.net.talos.dev/eth0")
            .unwrap();
        assert_eq!(
            source.metadata().owner(),
            "network.LinkStatusSourceController"
        );
        let fingerprint = source.spec_fingerprint();
        assert!(fingerprint.contains("name=eth0;"));
        assert!(fingerprint.contains("aliases=lan0;"));
        assert!(fingerprint.contains("oper_state=up;"));
        assert!(fingerprint.contains("carrier=true;"));
    }

    #[test]
    fn link_status_publication_controller_removes_stale_status_when_kernel_link_disappears() {
        let mut rt =
            runtime_with_link_status_source_snapshots(vec![vec![physical_status("eth0")], vec![]]);

        rt.tick().unwrap();
        assert!(
            rt.state()
                .contains("network/LinkStatuses.net.talos.dev/eth0")
        );

        rt.tick().unwrap();
        assert!(
            !rt.state()
                .contains("network/LinkStatuses.net.talos.dev/eth0")
        );
    }

    #[test]
    fn link_status_publication_controller_surfaces_malformed_live_status_without_silent_drop() {
        let mut rt = ControllerRuntime::new();
        rt.register(Box::new(LinkStatusSourceController::new_with_source(
            || {
                Err(os_kernel::Error::invalid(
                    "empty link-status source fixture",
                ))
            },
        )));

        let err = rt.tick().unwrap_err();

        assert!(matches!(err, ReconcileError::Invalid(_)));
        assert!(
            err.to_string()
                .contains("link status source snapshot failed")
        );
    }

    #[test]
    fn link_status_publication_controller_rejects_resource_id_unsafe_link_names() {
        let mut status = physical_status("veth@peer");
        status.aliases = vec!["safe-alias".to_string()];
        let mut rt = runtime_with_link_status_source(vec![status]);

        let err = rt.run_until_stable(5).unwrap_err();

        assert!(matches!(err, ReconcileError::Invalid(_)));
        assert!(err.to_string().contains("invalid link status resource"));
        assert!(
            !rt.state()
                .contains("network/LinkStatuses.net.talos.dev/veth@peer")
        );
    }

    #[test]
    fn live_link_status_drives_default_network_from_publication() {
        let mut status = physical_status("eth0");
        status.admin_up = true;
        status.oper_state = os_network_domain::OperState::Up;
        status.carrier = true;
        let mut rt = runtime_with_link_status_source(vec![status]);

        rt.run_until_stable(5).unwrap();

        assert!(
            rt.state()
                .contains("network/LinkStatuses.net.talos.dev/eth0")
        );
        assert!(
            rt.state()
                .contains("network-config/LinkSpecs.net.talos.dev/default/eth0")
        );
        assert!(rt.state().contains("network/LinkSpecs.net.talos.dev/eth0"));
        assert!(
            rt.state()
                .contains("network-config/OperatorSpecs.net.talos.dev/default/dhcp4/eth0")
        );
        assert!(
            rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/eth0")
        );
    }

    #[test]
    fn live_link_status_alias_suppresses_default_dhcp_for_configured_alias() {
        let mut status = physical_status("enp0s1");
        status.aliases = vec!["eth0".to_string()];
        let mut rt = ControllerRuntime::new();
        rt.state_mut()
            .create(Box::new(MachineConfigDocument::new(
                "\
version: v1alpha1
machine:
  type: worker
  token: tok
  network:
    interfaces:
      - interface: eth0
        dhcp: false
cluster:
  controlPlane:
    endpoint: https://10.0.0.1:6443
",
            )))
            .unwrap();
        rt.register(Box::new(LinkStatusSourceController::new_with_source(
            move || Ok(vec![status.clone()]),
        )));
        rt.register(Box::new(OperatorConfigController::new()));
        rt.register(Box::new(OperatorMergeController::new()));

        rt.run_until_stable(5).unwrap();

        assert!(
            rt.state()
                .contains("network/LinkStatuses.net.talos.dev/enp0s1")
        );
        assert!(
            !rt.state()
                .contains("network-config/OperatorSpecs.net.talos.dev/default/dhcp4/enp0s1")
        );
        assert!(
            !rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/enp0s1")
        );
    }

    #[test]
    fn link_status_fingerprint_requires_explicit_kind() {
        let err = parse_link_status(
            "name=eth0;type=ether;admin_up=false;oper_state=down;carrier=false;mac=02:00:00:00:00:01;mtu=1500",
        )
        .unwrap_err();

        assert!(matches!(err, ReconcileError::Invalid(_)));
        assert!(err.to_string().contains("kind"));
    }

    #[test]
    fn malformed_link_status_without_kind_does_not_synthesize_defaults() {
        let mut rt = ControllerRuntime::new();
        rt.state_mut()
            .create(Box::new(RawTestResource::link_status(
                "eth0",
                "name=eth0;type=ether;admin_up=false;oper_state=down;carrier=false;mac=02:00:00:00:00:01;mtu=1500",
            )))
            .unwrap();
        rt.register(Box::new(OperatorConfigController::new()));
        rt.register(Box::new(OperatorMergeController::new()));

        let err = rt.run_until_stable(4).unwrap_err();

        assert!(matches!(err, ReconcileError::Invalid(_)));
        assert!(err.to_string().contains("kind"));
        assert!(
            !rt.state()
                .contains("network-config/OperatorSpecs.net.talos.dev/default/dhcp4/eth0")
        );
        assert!(
            !rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/eth0")
        );
    }

    #[test]
    fn operator_config_controller_seeds_machine_config_operator_specs() {
        let mut rt = runtime_with_machine_config(machine_config_dual_stack());

        rt.run_until_stable(4).unwrap();

        let raw_dhcp4 = rt
            .state()
            .get("network-config/OperatorSpecs.net.talos.dev/configuration/dhcp4/eth0")
            .unwrap();
        assert_eq!(
            raw_dhcp4.spec_fingerprint(),
            "kind=dhcp4;link=eth0;require_up=true;route_metric=2048;skip_hostname=false;client=none;duid=;layer=configuration"
        );

        let raw_dhcp6 = rt
            .state()
            .get("network-config/OperatorSpecs.net.talos.dev/configuration/dhcp6/eth0")
            .unwrap();
        assert_eq!(
            raw_dhcp6.spec_fingerprint(),
            "kind=dhcp6;link=eth0;require_up=true;route_metric=4096;skip_hostname=true;client=duid;duid=00030001aabbccddeeff;layer=configuration"
        );

        assert!(
            rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/eth0")
        );
        assert!(
            rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp6/eth0")
        );
    }

    #[test]
    fn operator_config_controller_rejects_malformed_dhcp_config() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
---
apiVersion: v1alpha1
kind: DHCPv4Config
name: eth0
clientIdentifier: duid
duidRaw: not-hex
";
        let mut rt = runtime_with_machine_config(cfg);

        let err = rt.run_until_stable(4).unwrap_err();

        assert!(matches!(err, ReconcileError::Invalid(_)));
        assert!(err.to_string().contains("duidRaw"));
        assert!(
            !rt.state()
                .contains("network-config/OperatorSpecs.net.talos.dev/configuration/dhcp4/eth0")
        );
    }

    #[test]
    fn operator_config_controller_seeds_vlan_dhcp_operator_specs() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
  network:
    interfaces:
      - interface: eth0
        dhcp: false
        vlans:
          - vlanId: 100
            dhcp: true
            dhcpOptions:
              routeMetric: 4096
              ipv4: false
              ipv6: true
              duidv6: 00030001aabbccddeeff
          - vlanId: 200
            dhcp: true
cluster:
  controlPlane:
    endpoint: https://10.0.0.1:6443
";
        let mut rt = runtime_with_machine_config(cfg);

        rt.run_until_stable(4).unwrap();

        let raw_dhcp6 = rt
            .state()
            .get("network-config/OperatorSpecs.net.talos.dev/configuration/dhcp6/eth0.100")
            .unwrap();
        assert_eq!(
            raw_dhcp6.spec_fingerprint(),
            "kind=dhcp6;link=eth0.100;require_up=true;route_metric=4096;skip_hostname=false;client=duid;duid=00030001aabbccddeeff;layer=configuration"
        );

        let raw_dhcp4 = rt
            .state()
            .get("network-config/OperatorSpecs.net.talos.dev/configuration/dhcp4/eth0.200")
            .unwrap();
        assert_eq!(
            raw_dhcp4.spec_fingerprint(),
            "kind=dhcp4;link=eth0.200;require_up=true;route_metric=1024;skip_hostname=false;client=none;duid=;layer=configuration"
        );

        assert!(
            rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp6/eth0.100")
        );
        assert!(
            rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/eth0.200")
        );
    }

    #[test]
    fn operator_config_controller_hashes_long_vlan_link_names_like_talos() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
  network:
    interfaces:
      - interface: enx12545f8c99ce
        dhcp: false
        vlans:
          - vlanId: 4095
            dhcp: true
";
        let mut rt = runtime_with_machine_config(cfg);

        rt.run_until_stable(4).unwrap();

        assert!(rt.state().contains(
            "network-config/OperatorSpecs.net.talos.dev/configuration/dhcp4/enx1ef972f.4095"
        ));
        assert!(
            rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/enx1ef972f.4095")
        );
    }

    #[test]
    fn operator_config_controller_seeds_dhcpv4_config_documents() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: DHCPv4Config
name: eth1
routeMetric: 2048
ignoreHostname: true
clientIdentifier: duid
duidRaw: 00:03:00:01:aa:bb:cc:dd:ee:ff
";
        let mut rt = runtime_with_machine_config(cfg);

        rt.run_until_stable(4).unwrap();

        let raw_dhcp4 = rt
            .state()
            .get("network-config/OperatorSpecs.net.talos.dev/configuration/dhcp4/eth1")
            .unwrap();
        assert_eq!(
            raw_dhcp4.spec_fingerprint(),
            "kind=dhcp4;link=eth1;require_up=true;route_metric=2048;skip_hostname=true;client=duid;duid=00030001aabbccddeeff;layer=configuration"
        );
        assert!(
            rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/eth1")
        );
    }

    #[test]
    fn operator_config_controller_seeds_legacy_vip_operator_specs() {
        let cfg = "\
version: v1alpha1
machine:
  type: controlplane
  token: tok
  network:
    interfaces:
      - interface: eth1
        vip:
          ip: 2.3.4.5
      - interface: eth2
        vlans:
          - vlanId: 26
            vip:
              ip: 5.5.4.4
cluster:
  controlPlane:
    endpoint: https://10.0.0.1:6443
";
        let mut rt = runtime_with_machine_config(cfg);

        rt.run_until_stable(4).unwrap();

        let raw_vip = rt
            .state()
            .get("network-config/OperatorSpecs.net.talos.dev/configuration/vip/eth1")
            .unwrap();
        assert_eq!(
            raw_vip.spec_fingerprint(),
            "kind=vip;link=eth1;require_up=true;route_metric=1024;skip_hostname=false;client=none;duid=;layer=configuration"
        );
        let raw_vlan_vip = rt
            .state()
            .get("network-config/OperatorSpecs.net.talos.dev/configuration/vip/eth2.26")
            .unwrap();
        assert_eq!(
            raw_vlan_vip.spec_fingerprint(),
            "kind=vip;link=eth2.26;require_up=true;route_metric=1024;skip_hostname=false;client=none;duid=;layer=configuration"
        );
        assert!(
            rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/vip/eth1")
        );
        assert!(
            rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/vip/eth2.26")
        );
    }

    #[test]
    fn operator_config_controller_seeds_layer2_vip_operator_specs() {
        let cfg = "\
version: v1alpha1
machine:
  type: controlplane
  token: tok
cluster:
  controlPlane:
    endpoint: https://10.0.0.1:6443
---
apiVersion: v1alpha1
kind: Layer2VIPConfig
name: fd7a:115c:a1e0:ab12:4843:cd96:6277:2302
link: eth5
";
        let mut rt = runtime_with_machine_config(cfg);

        rt.run_until_stable(4).unwrap();

        let raw_vip = rt
            .state()
            .get("network-config/OperatorSpecs.net.talos.dev/configuration/vip/eth5")
            .unwrap();
        assert_eq!(
            raw_vip.spec_fingerprint(),
            "kind=vip;link=eth5;require_up=true;route_metric=1024;skip_hostname=false;client=none;duid=;layer=configuration"
        );
        assert!(
            rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/vip/eth5")
        );
    }

    #[test]
    fn default_dhcp4_operator_spec_matches_source_defaults() {
        let op = default_dhcp4_operator_spec("eth0");

        assert_eq!(op.kind, OperatorKind::Dhcp4);
        assert_eq!(op.link_name, "eth0");
        assert_eq!(op.layer, ConfigLayer::Default);
        assert_eq!(op.route_metric, os_network_domain::DEFAULT_ROUTE_METRIC);
        assert_eq!(op.client_identifier, ClientIdentifierSpec::mac());
        op.validate().unwrap();
    }

    #[test]
    fn operator_config_controller_does_not_synthesize_default_dhcp_without_link_status_inputs() {
        // Source Talos only synthesizes default DHCP from observed LinkStatus;
        // without status inputs, Rust must not guess interface names.
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
cluster:
  controlPlane:
    endpoint: https://10.0.0.1:6443
";
        let mut rt = runtime_with_machine_config(cfg);

        rt.run_until_stable(4).unwrap();

        assert!(
            !rt.state()
                .contains("network-config/OperatorSpecs.net.talos.dev/default/dhcp4/eth0")
        );
        assert!(
            !rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/eth0")
        );
    }

    #[test]
    fn operator_config_controller_synthesizes_default_dhcp_for_unconfigured_physical_link_status() {
        let mut rt = runtime_with_default_dhcp_inputs(None, vec![physical_status("eth0")], vec![]);

        rt.run_until_stable(4).unwrap();

        let source = rt
            .state()
            .get("network-config/OperatorSpecs.net.talos.dev/default/dhcp4/eth0")
            .unwrap();
        assert_eq!(
            source.spec_fingerprint(),
            "kind=dhcp4;link=eth0;require_up=true;route_metric=1024;skip_hostname=false;client=mac;duid=;layer=default"
        );
        assert!(
            rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/eth0")
        );
    }

    #[test]
    fn operator_config_controller_suppresses_default_dhcp_for_configured_link_spec() {
        let mut rt = runtime_with_default_dhcp_inputs(
            None,
            vec![physical_status("eth0")],
            vec![link_source(
                "configuration/linkconfig/eth0",
                "eth0",
                1500,
                ConfigLayer::Configuration,
            )],
        );

        rt.run_until_stable(4).unwrap();

        assert!(
            !rt.state()
                .contains("network-config/OperatorSpecs.net.talos.dev/default/dhcp4/eth0")
        );
        assert!(
            !rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/eth0")
        );
    }

    #[test]
    fn operator_config_controller_suppresses_default_dhcp_for_configured_link_alias() {
        let mut status = physical_status("eth7");
        status.aliases.push("eth0".to_string());
        let mut rt = runtime_with_default_dhcp_inputs(
            None,
            vec![status],
            vec![link_source(
                "configuration/linkconfig/eth0",
                "eth0",
                1500,
                ConfigLayer::Configuration,
            )],
        );

        rt.run_until_stable(4).unwrap();

        assert!(
            !rt.state()
                .contains("network-config/OperatorSpecs.net.talos.dev/default/dhcp4/eth7")
        );
        assert!(
            !rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/eth7")
        );
    }

    #[test]
    fn operator_config_controller_suppresses_default_dhcp_for_ignored_legacy_link_alias() {
        let mut status = physical_status("eth7");
        status.aliases.push("eth0".to_string());
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
  network:
    interfaces:
      - interface: eth0
        dhcp: true
        ignore: true
cluster:
  controlPlane:
    endpoint: https://10.0.0.1:6443
";
        let mut rt = runtime_with_default_dhcp_inputs(Some(cfg), vec![status], vec![]);

        rt.run_until_stable(4).unwrap();

        assert!(
            !rt.state()
                .contains("network-config/OperatorSpecs.net.talos.dev/configuration/dhcp4/eth0")
        );
        assert!(
            !rt.state()
                .contains("network-config/OperatorSpecs.net.talos.dev/default/dhcp4/eth7")
        );
        assert!(
            !rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/eth0")
        );
        assert!(
            !rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/eth7")
        );
    }

    #[test]
    fn operator_config_controller_skips_default_dhcp_for_non_physical_link_status() {
        let mut rt = runtime_with_default_dhcp_inputs(
            None,
            vec![nonphysical_status("eth0.100", "vlan")],
            vec![],
        );

        rt.run_until_stable(4).unwrap();

        assert!(
            !rt.state()
                .contains("network-config/OperatorSpecs.net.talos.dev/default/dhcp4/eth0.100")
        );
        assert!(
            !rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/eth0.100")
        );
    }

    #[test]
    fn operator_config_controller_removes_stale_default_dhcp_when_link_disappears() {
        let mut rt = runtime_with_default_dhcp_inputs(None, vec![physical_status("eth0")], vec![]);

        rt.run_until_stable(4).unwrap();
        assert!(
            rt.state()
                .contains("network-config/OperatorSpecs.net.talos.dev/default/dhcp4/eth0")
        );

        destroy_resource(&mut rt, "network/LinkStatuses.net.talos.dev/eth0");
        rt.run_until_stable(4).unwrap();

        assert!(
            !rt.state()
                .contains("network-config/OperatorSpecs.net.talos.dev/default/dhcp4/eth0")
        );
        assert!(
            !rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/eth0")
        );
    }

    #[test]
    fn explicit_static_interface_suppresses_default_dhcp_operator_config() {
        // Talos docs say explicit link configuration disables default DHCP;
        // DHCP must be explicitly enabled on links that should run it.
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
  network:
    interfaces:
      - interface: eth0
        addresses:
          - 10.0.2.15/24
cluster:
  controlPlane:
    endpoint: https://10.0.0.1:6443
";
        let mut rt =
            runtime_with_default_dhcp_inputs(Some(cfg), vec![physical_status("eth0")], vec![]);

        rt.run_until_stable(4).unwrap();

        assert!(
            !rt.state()
                .contains("network-config/OperatorSpecs.net.talos.dev/default/dhcp4/eth0")
        );
        assert!(
            !rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/eth0")
        );
    }

    #[test]
    fn operator_config_update_removes_stale_source_and_merged_specs() {
        let mut rt = runtime_with_machine_config(machine_config_dual_stack());
        rt.run_until_stable(4).unwrap();
        assert!(
            rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp6/eth0")
        );

        let updated = machine_config_with_dhcpv4(1024);
        let resource = MachineConfigDocument::new(updated);
        let key = MachineConfigDocument::active_key();
        let version = rt.state().get(&key).unwrap().metadata().version();
        rt.state_mut().update(Box::new(resource), version).unwrap();
        rt.run_until_stable(4).unwrap();

        assert!(
            !rt.state()
                .contains("network-config/OperatorSpecs.net.talos.dev/configuration/dhcp6/eth0")
        );
        assert!(
            !rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp6/eth0")
        );
        assert!(
            rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/eth0")
        );
    }

    #[test]
    fn operator_config_reconcile_is_idempotent_for_unchanged_specs() {
        let mut rt = runtime_with_machine_config(&machine_config_with_dhcpv4(2048));
        rt.run_until_stable(4).unwrap();
        let key = "network/OperatorSpecs.net.talos.dev/dhcp4/eth0";
        let first_version = rt.state().get(key).unwrap().metadata().version();

        rt.run_until_stable(4).unwrap();
        let second_version = rt.state().get(key).unwrap().metadata().version();

        assert_eq!(first_version, second_version);
    }

    #[test]
    fn malformed_machine_config_errors_without_erasing_seeded_operator_specs() {
        let mut rt = runtime_with_machine_config(&machine_config_with_dhcpv4(1024));
        rt.run_until_stable(4).unwrap();
        assert!(
            rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/eth0")
        );

        let malformed = "\
version: v1alpha1
machine:
  network:
    interfaces:
      - interface: eth0
        dhcp: true
        dhcpOptions:
          routeMetric: not-a-number
";
        let resource = MachineConfigDocument::new(malformed);
        let key = MachineConfigDocument::active_key();
        let version = rt.state().get(&key).unwrap().metadata().version();
        rt.state_mut().update(Box::new(resource), version).unwrap();
        let err = rt.run_until_stable(4).unwrap_err();

        assert!(matches!(err, ReconcileError::Invalid(_)));
        assert!(err.to_string().contains("routeMetric"));
        assert!(
            rt.state()
                .contains("network-config/OperatorSpecs.net.talos.dev/configuration/dhcp4/eth0")
        );
        assert!(
            rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/eth0")
        );
    }

    #[test]
    fn seeded_operator_specs_drive_operator_result_bridge() {
        let mut rt = runtime_with_machine_config(&machine_config_with_dhcpv4(4242));
        rt.state_mut()
            .create(Box::new(OperatorResultResource::new(
                "dhcp4/eth0",
                full_result("10.0.0.50"),
            )))
            .unwrap();
        rt.register(Box::new(OperatorResultBridgeController::new()));

        rt.run_until_stable(6).unwrap();

        assert!(
            rt.state()
                .contains("network/OperatorSpecs.net.talos.dev/dhcp4/eth0")
        );
        assert!(
            rt.state()
                .contains("network-config/AddressSpecs.net.talos.dev/dhcp4/eth0/eth0/10.0.0.50/24")
        );
        let route = rt
            .state()
            .get("network-config/RouteSpecs.net.talos.dev/dhcp4/eth0/inet4/10.0.0.1//4242")
            .unwrap();
        assert_eq!(
            route.spec_fingerprint(),
            "source=dhcp4/eth0;destination=default;prefix=0;source=;gateway=10.0.0.1;out_link=eth0;family=inet4;metric=4242;mtu=0;table=254;protocol=dhcp;layer=operator"
        );
    }

    #[test]
    fn operator_merge_prefers_higher_config_layer() {
        let mut rt = ControllerRuntime::new();
        let mut default = OperatorSpec::dhcp4("eth0");
        default.layer = ConfigLayer::Default;
        default.route_metric = 1024;
        let mut configured = OperatorSpec::dhcp4("eth0");
        configured.layer = ConfigLayer::Configuration;
        configured.route_metric = 4096;
        rt.state_mut()
            .create(Box::new(OperatorConfigSpecResource::new(default)))
            .unwrap();
        rt.state_mut()
            .create(Box::new(OperatorConfigSpecResource::new(configured)))
            .unwrap();
        rt.register(Box::new(OperatorMergeController::new()));

        rt.run_until_stable(3).unwrap();

        let merged = rt
            .state()
            .get("network/OperatorSpecs.net.talos.dev/dhcp4/eth0")
            .unwrap();
        assert_eq!(
            merged.spec_fingerprint(),
            "kind=dhcp4;link=eth0;require_up=true;route_metric=4096;skip_hostname=false;client=none;duid=;layer=configuration"
        );
    }

    #[test]
    fn address_merge_prefers_higher_config_layer() {
        let mut rt = runtime_with_addresses(vec![
            address_source("default", "10.0.0.50", 24, "eth0", ConfigLayer::Default),
            address_source(
                "configuration",
                "10.0.0.50",
                24,
                "eth0",
                ConfigLayer::Configuration,
            ),
        ]);

        rt.run_until_stable(3).unwrap();

        let merged = rt
            .state()
            .get("network/AddressSpecs.net.talos.dev/eth0/10.0.0.50/24")
            .unwrap();
        assert_eq!(
            merged.spec_fingerprint(),
            "address=10.0.0.50;prefix=24;link=eth0;family=inet4;scope=global;flags=permanent;priority=0;layer=configuration"
        );
    }

    #[test]
    fn address_merge_removes_stale_final_specs() {
        let mut rt = runtime_with_addresses(vec![address_source(
            "operator",
            "10.0.0.50",
            24,
            "eth0",
            ConfigLayer::Operator,
        )]);
        rt.run_until_stable(3).unwrap();
        let source_key = "network-config/AddressSpecs.net.talos.dev/operator/eth0/10.0.0.50/24";
        let final_key = "network/AddressSpecs.net.talos.dev/eth0/10.0.0.50/24";
        assert!(rt.state().contains(final_key));

        destroy_resource(&mut rt, source_key);
        rt.run_until_stable(3).unwrap();

        assert!(!rt.state().contains(final_key));
    }

    #[test]
    fn address_merge_reconcile_is_idempotent() {
        let mut rt = runtime_with_addresses(vec![address_source(
            "configuration",
            "10.0.0.50",
            24,
            "eth0",
            ConfigLayer::Configuration,
        )]);
        rt.run_until_stable(3).unwrap();
        let key = "network/AddressSpecs.net.talos.dev/eth0/10.0.0.50/24";
        let first_version = rt.state().get(key).unwrap().metadata().version();

        rt.run_until_stable(3).unwrap();
        let second_version = rt.state().get(key).unwrap().metadata().version();

        assert_eq!(first_version, second_version);
    }

    #[test]
    fn route_merge_prefers_higher_config_layer() {
        let mut rt = runtime_with_routes(vec![
            route_source("default", "10.0.0.1", ConfigLayer::Default, 1024),
            route_source(
                "configuration",
                "10.0.0.1",
                ConfigLayer::Configuration,
                1024,
            ),
        ]);

        rt.run_until_stable(3).unwrap();

        let merged = rt
            .state()
            .get("network/RouteSpecs.net.talos.dev/inet4/10.0.0.1//1024")
            .unwrap();
        assert_eq!(
            merged.spec_fingerprint(),
            "destination=default;prefix=0;source=;gateway=10.0.0.1;out_link=eth0;family=inet4;metric=1024;mtu=0;table=254;protocol=dhcp;layer=configuration"
        );
    }

    #[test]
    fn route_merge_preserves_same_destination_with_distinct_gateways() {
        let mut rt = runtime_with_routes(vec![
            route_source("default", "10.0.0.1", ConfigLayer::Default, 1024),
            route_source(
                "configuration",
                "10.0.0.254",
                ConfigLayer::Configuration,
                4096,
            ),
        ]);

        rt.run_until_stable(3).unwrap();

        assert!(
            rt.state()
                .contains("network/RouteSpecs.net.talos.dev/inet4/10.0.0.1//1024")
        );
        assert!(
            rt.state()
                .contains("network/RouteSpecs.net.talos.dev/inet4/10.0.0.254//4096")
        );
    }

    #[test]
    fn route_merge_removes_stale_final_specs_and_is_idempotent() {
        let mut rt = runtime_with_routes(vec![route_source(
            "operator",
            "10.0.0.1",
            ConfigLayer::Operator,
            1024,
        )]);
        rt.run_until_stable(3).unwrap();
        let final_key = "network/RouteSpecs.net.talos.dev/inet4/10.0.0.1//1024";
        let source_key = "network-config/RouteSpecs.net.talos.dev/operator/inet4/10.0.0.1//1024";
        let first_version = rt.state().get(final_key).unwrap().metadata().version();

        rt.run_until_stable(3).unwrap();
        let second_version = rt.state().get(final_key).unwrap().metadata().version();
        assert_eq!(first_version, second_version);

        destroy_resource(&mut rt, source_key);
        rt.run_until_stable(3).unwrap();

        assert!(!rt.state().contains(final_key));
    }

    #[test]
    fn hostname_merge_prefers_higher_config_layer() {
        let mut rt = runtime_with_hostnames(vec![
            hostname_source(
                "operator",
                "lease-host",
                Some("example.com"),
                ConfigLayer::Operator,
            ),
            hostname_source(
                "configuration",
                "configured",
                Some("cluster.local"),
                ConfigLayer::Configuration,
            ),
        ]);

        rt.run_until_stable(3).unwrap();

        let merged = rt
            .state()
            .get("network/HostnameSpecs.net.talos.dev/hostname")
            .unwrap();
        assert_eq!(
            merged.spec_fingerprint(),
            "hostname=configured;domain=cluster.local;layer=configuration"
        );
    }

    #[test]
    fn hostname_merge_removes_stale_final_spec_and_is_idempotent() {
        let mut rt = runtime_with_hostnames(vec![hostname_source(
            "operator",
            "lease-host",
            Some("example.com"),
            ConfigLayer::Operator,
        )]);
        rt.run_until_stable(3).unwrap();
        let final_key = "network/HostnameSpecs.net.talos.dev/hostname";
        let source_key = "network-config/HostnameSpecs.net.talos.dev/operator/hostname";
        let first_version = rt.state().get(final_key).unwrap().metadata().version();

        rt.run_until_stable(3).unwrap();
        let second_version = rt.state().get(final_key).unwrap().metadata().version();
        assert_eq!(first_version, second_version);

        destroy_resource(&mut rt, source_key);
        rt.run_until_stable(3).unwrap();

        assert!(!rt.state().contains(final_key));
    }

    #[test]
    fn link_merge_prefers_higher_config_layer() {
        let mut rt = runtime_with_links(vec![
            link_source("operator/eth0", "eth0", 1500, ConfigLayer::Operator),
            link_source(
                "configuration/linkconfig/eth0",
                "eth0",
                9000,
                ConfigLayer::Configuration,
            ),
        ]);

        rt.run_until_stable(3).unwrap();

        let merged = rt
            .state()
            .get("network/LinkSpecs.net.talos.dev/eth0")
            .unwrap();
        assert_eq!(
            merged.spec_fingerprint(),
            "name=eth0;up=true;mtu=9000;multicast=;kind=physical;members=;mode=;parent=;vlan_id=0;vlan_protocol=;layer=configuration"
        );
    }

    #[test]
    fn link_merge_removes_stale_final_spec_and_is_idempotent() {
        let mut rt = runtime_with_links(vec![link_source(
            "configuration/linkconfig/eth0",
            "eth0",
            9000,
            ConfigLayer::Configuration,
        )]);
        rt.run_until_stable(3).unwrap();
        let final_key = "network/LinkSpecs.net.talos.dev/eth0";
        let source_key = "network-config/LinkSpecs.net.talos.dev/configuration/linkconfig/eth0";
        let first_version = rt.state().get(final_key).unwrap().metadata().version();

        rt.run_until_stable(3).unwrap();
        let second_version = rt.state().get(final_key).unwrap().metadata().version();
        assert_eq!(first_version, second_version);

        destroy_resource(&mut rt, source_key);
        rt.run_until_stable(3).unwrap();

        assert!(!rt.state().contains(final_key));
    }

    #[test]
    fn resolver_config_controller_seeds_and_merges_resolver_config_document() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
---
apiVersion: v1alpha1
kind: ResolverConfig
nameservers:
  - address: 1.1.1.1
  - address: 2606:4700:4700::1111
searchDomains:
  domains:
    - cluster.local
";
        let mut rt = runtime_with_resolver_config_doc(cfg);

        rt.run_until_stable(4).unwrap();

        let source = rt
            .state()
            .get(
                "network-config/ResolverSpecs.net.talos.dev/configuration/resolverconfig/resolvers",
            )
            .unwrap();
        assert_eq!(
            source.spec_fingerprint(),
            "source=configuration/resolverconfig;servers=1.1.1.1,2606:4700:4700:0:0:0:0:1111;search=cluster.local;layer=configuration"
        );

        let merged = rt
            .state()
            .get("network/ResolverSpecs.net.talos.dev/resolvers")
            .unwrap();
        assert_eq!(
            merged.spec_fingerprint(),
            "servers=1.1.1.1,2606:4700:4700:0:0:0:0:1111;search=cluster.local;layer=configuration"
        );
    }

    #[test]
    fn resolver_config_controller_rejects_encrypted_nameserver_without_downgrade() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
---
apiVersion: v1alpha1
kind: ResolverConfig
nameservers:
  - address: 9.9.9.9
    protocol: DoT
    tlsServerName: dns.quad9.net
";
        let mut rt = runtime_with_resolver_config_doc(cfg);

        let err = rt.run_until_stable(4).unwrap_err();

        assert!(matches!(err, ReconcileError::Invalid(_)));
        assert!(err.to_string().contains("not yet projectable"));
        assert!(
            !rt.state()
                .contains("network/ResolverSpecs.net.talos.dev/resolvers")
        );
    }

    #[test]
    fn resolver_config_controller_rejects_hostdns_without_downgrade() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
---
apiVersion: v1alpha1
kind: ResolverConfig
hostDNS:
  enabled: true
";
        let mut rt = runtime_with_resolver_config_doc(cfg);

        let err = rt.run_until_stable(4).unwrap_err();

        assert!(matches!(err, ReconcileError::Invalid(_)));
        assert!(err.to_string().contains("hostDNS"));
        assert!(
            !rt.state()
                .contains("network/ResolverSpecs.net.talos.dev/resolvers")
        );
    }

    #[test]
    fn resolver_config_controller_rejects_disable_default_without_downgrade() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
---
apiVersion: v1alpha1
kind: ResolverConfig
searchDomains:
  disableDefault: true
";
        let mut rt = runtime_with_resolver_config_doc(cfg);

        let err = rt.run_until_stable(4).unwrap_err();

        assert!(matches!(err, ReconcileError::Invalid(_)));
        assert!(err.to_string().contains("disableDefault"));
        assert!(
            !rt.state()
                .contains("network/ResolverSpecs.net.talos.dev/resolvers")
        );
    }

    #[test]
    fn link_config_controller_brings_up_unconfigured_physical_link_for_default_dhcp() {
        let mut rt = runtime_with_link_status_config(None, vec![physical_status("eth0")]);

        rt.run_until_stable(5).unwrap();

        let source = rt
            .state()
            .get("network-config/LinkSpecs.net.talos.dev/default/eth0")
            .unwrap();
        assert_eq!(
            source.spec_fingerprint(),
            "source=default/eth0;name=eth0;up=true;mtu=0;multicast=;kind=physical;members=;mode=;parent=;vlan_id=0;vlan_protocol=;layer=default"
        );
        let merged = rt
            .state()
            .get("network/LinkSpecs.net.talos.dev/eth0")
            .unwrap();
        assert_eq!(
            merged.spec_fingerprint(),
            "name=eth0;up=true;mtu=0;multicast=;kind=physical;members=;mode=;parent=;vlan_id=0;vlan_protocol=;layer=default"
        );
    }

    #[test]
    fn link_config_controller_skips_default_link_for_non_physical_status() {
        let mut rt =
            runtime_with_link_status_config(None, vec![nonphysical_status("eth0.100", "vlan")]);

        rt.run_until_stable(5).unwrap();

        assert!(
            !rt.state()
                .contains("network-config/LinkSpecs.net.talos.dev/default/eth0.100")
        );
        assert!(
            !rt.state()
                .contains("network/LinkSpecs.net.talos.dev/eth0.100")
        );
    }

    #[test]
    fn link_config_controller_suppresses_default_link_for_existing_source_link_spec() {
        let mut rt = runtime_with_link_status_config_and_links(
            None,
            vec![physical_status("eth0")],
            vec![link_source(
                "cmdline/eth0",
                "eth0",
                1500,
                ConfigLayer::Cmdline,
            )],
        );

        rt.run_until_stable(5).unwrap();

        assert!(
            !rt.state()
                .contains("network-config/LinkSpecs.net.talos.dev/default/eth0")
        );
        assert!(
            rt.state()
                .contains("network-config/LinkSpecs.net.talos.dev/cmdline/eth0")
        );
        let merged = rt
            .state()
            .get("network/LinkSpecs.net.talos.dev/eth0")
            .unwrap();
        assert_eq!(
            merged.spec_fingerprint(),
            "name=eth0;up=true;mtu=1500;multicast=;kind=physical;members=;mode=;parent=;vlan_id=0;vlan_protocol=;layer=cmdline"
        );
    }

    #[test]
    fn link_config_controller_suppresses_default_link_for_configured_link_alias() {
        let mut status = physical_status("eth7");
        status.aliases.push("eth0".to_string());
        let mut rt = runtime_with_link_status_config_and_links(
            None,
            vec![status],
            vec![link_source(
                "cmdline/eth0",
                "eth0",
                1500,
                ConfigLayer::Cmdline,
            )],
        );

        rt.run_until_stable(5).unwrap();

        assert!(
            !rt.state()
                .contains("network-config/LinkSpecs.net.talos.dev/default/eth7")
        );
        assert!(!rt.state().contains("network/LinkSpecs.net.talos.dev/eth7"));
        assert!(
            rt.state()
                .contains("network-config/LinkSpecs.net.talos.dev/cmdline/eth0")
        );
    }

    #[test]
    fn link_config_controller_suppresses_default_link_for_ignored_legacy_link_alias() {
        let mut status = physical_status("eth7");
        status.aliases.push("eth0".to_string());
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
  network:
    interfaces:
      - interface: eth0
        dhcp: true
        ignore: true
cluster:
  controlPlane:
    endpoint: https://10.0.0.1:6443
";
        let mut rt = runtime_with_link_status_config(Some(cfg), vec![status]);

        rt.run_until_stable(5).unwrap();

        assert!(
            !rt.state()
                .contains("network-config/LinkSpecs.net.talos.dev/default/eth7")
        );
        assert!(!rt.state().contains("network/LinkSpecs.net.talos.dev/eth7"));
    }

    #[test]
    fn link_config_controller_materializes_physical_link_document() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: LinkConfig
name: eth0
up: false
mtu: 9000
multicast: true
addresses:
  - address: 192.168.1.10/24
routes:
  - gateway: 192.168.1.1
    metric: 100
";
        let mut rt = runtime_with_link_config_doc(cfg);

        rt.run_until_stable(5).unwrap();

        let source = rt
            .state()
            .get("network-config/LinkSpecs.net.talos.dev/configuration/linkconfig/eth0")
            .unwrap();
        assert_eq!(
            source.spec_fingerprint(),
            "source=configuration/linkconfig/eth0;name=eth0;up=false;mtu=9000;multicast=true;kind=physical;members=;mode=;parent=;vlan_id=0;vlan_protocol=;layer=configuration"
        );

        let merged = rt
            .state()
            .get("network/LinkSpecs.net.talos.dev/eth0")
            .unwrap();
        assert_eq!(
            merged.spec_fingerprint(),
            "name=eth0;up=false;mtu=9000;multicast=true;kind=physical;members=;mode=;parent=;vlan_id=0;vlan_protocol=;layer=configuration"
        );

        assert!(rt.state().contains(
            "network-config/AddressSpecs.net.talos.dev/configuration/linkconfig/eth0/eth0/192.168.1.10/24"
        ));
        let route = rt.state().get(
            "network-config/RouteSpecs.net.talos.dev/configuration/linkconfig/eth0/inet4/192.168.1.1//100",
        ).unwrap();
        assert_eq!(
            route.spec_fingerprint(),
            "source=configuration/linkconfig/eth0;destination=default;prefix=0;source=;gateway=192.168.1.1;out_link=eth0;family=inet4;metric=100;mtu=0;table=254;protocol=static;layer=configuration"
        );
        assert!(
            rt.state()
                .contains("network/AddressSpecs.net.talos.dev/eth0/192.168.1.10/24")
        );
        assert!(
            rt.state()
                .contains("network/RouteSpecs.net.talos.dev/inet4/192.168.1.1//100")
        );
    }

    #[test]
    fn link_config_controller_materializes_vlan_document() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: VLANConfig
name: eth0.100
vlanID: 100
parent: eth0
vlanMode: 802.1ad
up: true
mtu: 1500
multicast: false
addresses:
  - address: 10.100.0.2/24
routes:
  - destination: 10.200.0.0/16
    gateway: 10.100.0.1
    metric: 200
";
        let mut rt = runtime_with_link_config_doc(cfg);

        rt.run_until_stable(5).unwrap();

        let source = rt
            .state()
            .get("network-config/LinkSpecs.net.talos.dev/configuration/vlanconfig/eth0.100")
            .unwrap();
        assert_eq!(
            source.spec_fingerprint(),
            "source=configuration/vlanconfig/eth0.100;name=eth0.100;up=true;mtu=1500;multicast=false;kind=vlan;members=;mode=;parent=eth0;vlan_id=100;vlan_protocol=802.1ad;layer=configuration"
        );

        let merged = rt
            .state()
            .get("network/LinkSpecs.net.talos.dev/eth0.100")
            .unwrap();
        assert_eq!(
            merged.spec_fingerprint(),
            "name=eth0.100;up=true;mtu=1500;multicast=false;kind=vlan;members=;mode=;parent=eth0;vlan_id=100;vlan_protocol=802.1ad;layer=configuration"
        );
        assert!(rt.state().contains(
            "network-config/AddressSpecs.net.talos.dev/configuration/vlanconfig/eth0.100/eth0.100/10.100.0.2/24"
        ));
        assert!(
            rt.state().contains(
                "network-config/RouteSpecs.net.talos.dev/configuration/vlanconfig/eth0.100/inet4/10.100.0.1/10.200.0.0/16/200"
            )
        );
    }

    #[test]
    fn link_config_controller_projects_route_priority_without_downgrade() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: LinkConfig
name: eth0
addresses:
  - address: 192.168.1.10/24
    routePriority: 2048
";
        let mut rt = runtime_with_link_config_doc(cfg);

        rt.run_until_stable(5).unwrap();

        let address = rt
            .state()
            .get("network-config/AddressSpecs.net.talos.dev/configuration/linkconfig/eth0/eth0/192.168.1.10/24")
            .unwrap();
        assert_eq!(
            address.spec_fingerprint(),
            "source=configuration/linkconfig/eth0;address=192.168.1.10;prefix=24;link=eth0;family=inet4;scope=global;flags=permanent;priority=2048;layer=configuration"
        );
    }

    #[test]
    fn link_config_controller_projects_route_source_and_mtu_without_downgrade() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: LinkConfig
name: eth0
routes:
  - destination: 192.0.2.0/24
    source: 192.0.2.10
    mtu: 1400
";
        let mut rt = runtime_with_link_config_doc(cfg);

        rt.run_until_stable(5).unwrap();

        let route = rt
            .state()
            .get("network-config/RouteSpecs.net.talos.dev/configuration/linkconfig/eth0/inet4//192.0.2.0/24/1024")
            .unwrap();
        assert_eq!(
            route.spec_fingerprint(),
            "source=configuration/linkconfig/eth0;destination=192.0.2.0;prefix=24;source=192.0.2.10;gateway=;out_link=eth0;family=inet4;metric=1024;mtu=1400;table=254;protocol=static;layer=configuration"
        );
    }

    #[test]
    fn resolver_merge_prefers_higher_config_layer() {
        let mut rt = runtime_with_resolvers(vec![
            resolver_source(
                "platform/resolvers",
                vec![v4("8.8.8.8")],
                ConfigLayer::Platform,
            ),
            resolver_source_with_search(
                "configuration/resolvers",
                vec![v4("1.1.1.1"), v4("1.0.0.1")],
                vec!["cluster.local".to_string()],
                ConfigLayer::Configuration,
            ),
        ]);

        rt.run_until_stable(3).unwrap();

        let merged = rt
            .state()
            .get("network/ResolverSpecs.net.talos.dev/resolvers")
            .unwrap();
        assert_eq!(
            merged.spec_fingerprint(),
            "servers=1.1.1.1,1.0.0.1;search=cluster.local;layer=configuration"
        );
    }

    #[test]
    fn resolver_merge_removes_stale_final_spec() {
        let mut rt = runtime_with_resolvers(vec![resolver_source(
            "operator",
            vec![v4("10.0.0.2")],
            ConfigLayer::Operator,
        )]);
        rt.run_until_stable(3).unwrap();
        let final_key = "network/ResolverSpecs.net.talos.dev/resolvers";
        assert!(rt.state().contains(final_key));

        let version = rt
            .state()
            .get("network-config/ResolverSpecs.net.talos.dev/operator/resolvers")
            .unwrap()
            .metadata()
            .version();
        let teardown = rt
            .state_mut()
            .teardown(
                "network-config/ResolverSpecs.net.talos.dev/operator/resolvers",
                version,
            )
            .unwrap();
        rt.state_mut()
            .destroy(
                "network-config/ResolverSpecs.net.talos.dev/operator/resolvers",
                teardown,
            )
            .unwrap();
        rt.run_until_stable(3).unwrap();

        assert!(!rt.state().contains(final_key));
    }

    #[test]
    fn resolver_merge_reconcile_is_idempotent() {
        let mut rt = runtime_with_resolvers(vec![resolver_source(
            "configuration/resolvers",
            vec![v4("1.1.1.1")],
            ConfigLayer::Configuration,
        )]);
        rt.run_until_stable(3).unwrap();
        let key = "network/ResolverSpecs.net.talos.dev/resolvers";
        let first_version = rt.state().get(key).unwrap().metadata().version();

        rt.run_until_stable(3).unwrap();
        let second_version = rt.state().get(key).unwrap().metadata().version();

        assert_eq!(first_version, second_version);
    }

    #[test]
    fn resolver_merge_search_only_configuration_suppresses_lower_dns_servers() {
        let mut rt = runtime_with_resolvers(vec![
            resolver_source(
                "operator/resolvers",
                vec![v4("8.8.8.8")],
                ConfigLayer::Operator,
            ),
            resolver_source_with_search(
                "configuration/resolverconfig",
                Vec::new(),
                vec!["cluster.local".to_string()],
                ConfigLayer::Configuration,
            ),
        ]);

        rt.run_until_stable(3).unwrap();

        let merged = rt
            .state()
            .get("network/ResolverSpecs.net.talos.dev/resolvers")
            .unwrap();
        assert_eq!(
            merged.spec_fingerprint(),
            "servers=;search=cluster.local;layer=configuration"
        );
    }

    #[test]
    fn controller_materializes_operator_output_resources() {
        let mut spec = OperatorSpec::dhcp4("eth0");
        spec.route_metric = 4242;
        let mut rt = runtime_with_operator(spec, full_result("10.0.0.50"));

        rt.run_until_stable(3).unwrap();

        let address = rt
            .state()
            .get("network-config/AddressSpecs.net.talos.dev/dhcp4/eth0/eth0/10.0.0.50/24")
            .unwrap();
        assert_eq!(
            address.spec_fingerprint(),
            "source=dhcp4/eth0;address=10.0.0.50;prefix=24;link=eth0;family=inet4;scope=global;flags=permanent;priority=0;layer=operator"
        );

        let route = rt
            .state()
            .get("network-config/RouteSpecs.net.talos.dev/dhcp4/eth0/inet4/10.0.0.1//4242")
            .unwrap();
        assert_eq!(
            route.spec_fingerprint(),
            "source=dhcp4/eth0;destination=default;prefix=0;source=;gateway=10.0.0.1;out_link=eth0;family=inet4;metric=4242;mtu=0;table=254;protocol=dhcp;layer=operator"
        );

        let hostname = rt
            .state()
            .get("network-config/HostnameSpecs.net.talos.dev/dhcp4/eth0/hostname")
            .unwrap();
        assert_eq!(
            hostname.spec_fingerprint(),
            "source=dhcp4/eth0;hostname=leased-host;domain=example.com;layer=operator"
        );

        let resolver = rt
            .state()
            .get("network-config/ResolverSpecs.net.talos.dev/dhcp4/eth0/resolvers")
            .unwrap();
        assert_eq!(
            resolver.spec_fingerprint(),
            "source=dhcp4/eth0;servers=10.0.0.2,8.8.8.8;search=example.com;layer=operator"
        );
    }

    #[test]
    fn skip_hostname_request_removes_stale_hostname_output() {
        let spec = OperatorSpec::dhcp4("eth0");
        let mut rt = runtime_with_operator(spec.clone(), full_result("10.0.0.50"));
        rt.run_until_stable(3).unwrap();
        let hostname_key = "network-config/HostnameSpecs.net.talos.dev/dhcp4/eth0/hostname";
        assert!(rt.state().contains(hostname_key));

        let mut skip = spec.with_skip_hostname_request(true);
        skip.route_metric = 1024;
        let resource = OperatorSpecResource::new(skip);
        let version = rt
            .state()
            .get("network/OperatorSpecs.net.talos.dev/dhcp4/eth0")
            .unwrap()
            .metadata()
            .version();
        rt.state_mut().update(Box::new(resource), version).unwrap();
        rt.run_until_stable(3).unwrap();

        assert!(!rt.state().contains(hostname_key));
        assert!(
            rt.state()
                .contains("network-config/AddressSpecs.net.talos.dev/dhcp4/eth0/eth0/10.0.0.50/24")
        );
    }

    #[test]
    fn result_change_removes_stale_address_output() {
        let spec = OperatorSpec::dhcp4("eth0");
        let mut rt = runtime_with_operator(spec, full_result("10.0.0.50"));
        rt.run_until_stable(3).unwrap();
        let old_key = "network-config/AddressSpecs.net.talos.dev/dhcp4/eth0/eth0/10.0.0.50/24";
        let new_key = "network-config/AddressSpecs.net.talos.dev/dhcp4/eth0/eth0/10.0.0.77/24";
        assert!(rt.state().contains(old_key));

        let resource = OperatorResultResource::new("dhcp4/eth0", full_result("10.0.0.77"));
        let version = rt
            .state()
            .get("network/OperatorResults.net.talos.dev/dhcp4/eth0")
            .unwrap()
            .metadata()
            .version();
        rt.state_mut().update(Box::new(resource), version).unwrap();
        rt.run_until_stable(3).unwrap();

        assert!(!rt.state().contains(old_key));
        assert!(rt.state().contains(new_key));
    }

    #[test]
    fn missing_result_cleans_existing_operator_outputs() {
        let mut rt = runtime_with_operator(OperatorSpec::dhcp4("eth0"), full_result("10.0.0.50"));
        rt.run_until_stable(3).unwrap();
        assert!(
            rt.state()
                .contains("network-config/AddressSpecs.net.talos.dev/dhcp4/eth0/eth0/10.0.0.50/24")
        );

        let version = rt
            .state()
            .get("network/OperatorResults.net.talos.dev/dhcp4/eth0")
            .unwrap()
            .metadata()
            .version();
        let teardown = rt
            .state_mut()
            .teardown("network/OperatorResults.net.talos.dev/dhcp4/eth0", version)
            .unwrap();
        rt.state_mut()
            .destroy("network/OperatorResults.net.talos.dev/dhcp4/eth0", teardown)
            .unwrap();
        rt.run_until_stable(3).unwrap();

        assert!(
            rt.state()
                .list(&AddressSpecResource::kind(), None)
                .is_empty()
        );
        assert!(rt.state().list(&RouteSpecResource::kind(), None).is_empty());
        assert!(
            rt.state()
                .list(&HostnameSpecResource::kind(), None)
                .is_empty()
        );
        assert!(
            rt.state()
                .list(&ResolverSpecResource::kind(), None)
                .is_empty()
        );
    }

    #[test]
    fn controller_materializes_dhcpv6_operator_output_resources() {
        let mut spec = OperatorSpec::dhcp6("eth0");
        spec.route_metric = 2048;
        let mut rt = runtime_with_operator(spec, full_v6_result("2001:db8::50"));

        rt.run_until_stable(3).unwrap();

        let address_key =
            "network-config/AddressSpecs.net.talos.dev/dhcp6/eth0/eth0/2001:db8:0:0:0:0:0:50/64";
        let address = rt.state().get(address_key).unwrap();
        assert_eq!(
            address.spec_fingerprint(),
            "source=dhcp6/eth0;address=2001:db8:0:0:0:0:0:50;prefix=64;link=eth0;family=inet6;scope=global;flags=permanent;priority=0;layer=operator"
        );

        let route = rt
            .state()
            .get("network-config/RouteSpecs.net.talos.dev/dhcp6/eth0/eth0/inet6/2001:db8::1//2048")
            .unwrap();
        assert_eq!(
            route.spec_fingerprint(),
            "source=dhcp6/eth0;destination=default;prefix=0;source=;gateway=2001:db8:0:0:0:0:0:1;out_link=eth0;family=inet6;metric=2048;mtu=0;table=254;protocol=dhcp;layer=operator"
        );

        let resolver = rt
            .state()
            .get("network-config/ResolverSpecs.net.talos.dev/dhcp6/eth0/resolvers")
            .unwrap();
        assert_eq!(
            resolver.spec_fingerprint(),
            "source=dhcp6/eth0;servers=2001:4860:4860:0:0:0:0:8888;search=example.com;layer=operator"
        );
    }

    #[test]
    fn multiple_operators_only_materialize_matching_results() {
        let mut rt = ControllerRuntime::new();
        rt.state_mut()
            .create(Box::new(OperatorSpecResource::new(OperatorSpec::dhcp4(
                "eth0",
            ))))
            .unwrap();
        rt.state_mut()
            .create(Box::new(OperatorSpecResource::new(OperatorSpec::dhcp6(
                "eth0",
            ))))
            .unwrap();
        rt.state_mut()
            .create(Box::new(OperatorSpecResource::new(OperatorSpec::dhcp4(
                "eth1",
            ))))
            .unwrap();
        rt.state_mut()
            .create(Box::new(OperatorResultResource::new(
                "dhcp4/eth0",
                full_result("10.0.0.50"),
            )))
            .unwrap();
        rt.state_mut()
            .create(Box::new(OperatorResultResource::new(
                "dhcp6/eth0",
                full_v6_result("2001:db8::50"),
            )))
            .unwrap();
        rt.state_mut()
            .create(Box::new(OperatorResultResource::new(
                "dhcp4/missing",
                full_result("10.0.0.99"),
            )))
            .unwrap();
        rt.register(Box::new(OperatorResultBridgeController::new()));

        rt.run_until_stable(3).unwrap();

        assert!(
            rt.state()
                .contains("network-config/AddressSpecs.net.talos.dev/dhcp4/eth0/eth0/10.0.0.50/24")
        );
        assert!(rt.state().contains(
            "network-config/AddressSpecs.net.talos.dev/dhcp6/eth0/eth0/2001:db8:0:0:0:0:0:50/64"
        ));
        assert!(
            !rt.state().contains(
                "network-config/AddressSpecs.net.talos.dev/dhcp4/missing/eth0/10.0.0.99/24"
            )
        );
    }

    #[test]
    fn stale_cleanup_preserves_unowned_resources() {
        let mut rt = runtime_with_operator(OperatorSpec::dhcp4("eth0"), full_result("10.0.0.50"));
        let manual = AddressSpecResource::new(
            "dhcp4/eth0",
            AddressSpec::new(v4("10.0.0.99"), 24, "eth0", ConfigLayer::Operator).unwrap(),
        );
        let manual_key = manual.metadata().key();
        rt.state_mut().create(Box::new(manual)).unwrap();

        rt.run_until_stable(3).unwrap();

        assert!(rt.state().contains(&manual_key));
        assert!(
            rt.state()
                .contains("network-config/AddressSpecs.net.talos.dev/dhcp4/eth0/eth0/10.0.0.50/24")
        );
    }

    #[test]
    fn identical_reconcile_is_idempotent() {
        let mut rt = runtime_with_operator(OperatorSpec::dhcp4("eth0"), full_result("10.0.0.50"));
        rt.run_until_stable(3).unwrap();
        let key = "network-config/AddressSpecs.net.talos.dev/dhcp4/eth0/eth0/10.0.0.50/24";
        let first_version = rt.state().get(key).unwrap().metadata().version();

        rt.run_until_stable(3).unwrap();
        let second_version = rt.state().get(key).unwrap().metadata().version();

        assert_eq!(first_version, second_version);
    }

    #[test]
    fn undeclared_output_protection_still_applies() {
        let mut state = State::new();
        let mut ctx =
            ReconcileContext::new(&mut state, "network.OperatorResultBridgeController", vec![]);
        let resource = AddressSpecResource::new(
            "dhcp4/eth0",
            AddressSpec::new(v4("10.0.0.50"), 24, "eth0", ConfigLayer::Operator).unwrap(),
        );
        let err = ctx.write(Box::new(resource)).unwrap_err();
        assert!(matches!(err, ReconcileError::UndeclaredOutput(_)));
    }
}
