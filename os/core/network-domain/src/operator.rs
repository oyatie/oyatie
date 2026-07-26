//! Dynamic network operators (DHCP4/6, static).
//!
//! Mirrors `network.OperatorSpec`, `OperatorConfigController`,
//! `OperatorMergeController` and the operator implementations under
//! `internal/app/machined/pkg/controllers/network/operator/*`. An operator runs
//! against a single link and produces address/route/hostname/resolver specs at
//! the [`ConfigLayer::Operator`] layer. Where the real operators speak DHCP on
//! the wire, this models the lease/result they yield so the downstream merge
//! and reconcile logic can be exercised deterministically.

use crate::address::AddressSpec;
use crate::config_layer::ConfigLayer;
use crate::dhcp6::Dhcp6ClientIdentifier;
use crate::hostname::HostnameSpec;
use crate::nethelpers::ClientIdentifier;
use crate::resolver::ResolverSpec;
use crate::route::RouteSpec;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use os_kernel::address::NodeAddress;
use os_kernel::error::{Error, Result};

/// Default metric for DHCP-provided routes.
///
/// Talos uses the same 1024 default as systemd-networkd; machine-config
/// `routeMetric: 0` means "unspecified" and falls back to this value.
pub const DEFAULT_ROUTE_METRIC: u32 = 1024;

/// The kind of operator bound to a link.
///
/// Equivalent to `network.Operator` enum (`OperatorDHCP4`, `OperatorDHCP6`,
/// `OperatorVIP`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorKind {
    /// DHCPv4 client.
    Dhcp4,
    /// DHCPv6 client.
    Dhcp6,
    /// Static (config-driven) operator, used for a virtual IP.
    Vip,
}

impl OperatorKind {
    /// The stable lowercase identifier used in operator resource ids.
    pub fn as_str(self) -> &'static str {
        match self {
            OperatorKind::Dhcp4 => "dhcp4",
            OperatorKind::Dhcp6 => "dhcp6",
            OperatorKind::Vip => "vip",
        }
    }
}

/// Shared DHCP client-identifier configuration.
///
/// Mirrors upstream `network.ClientIdentifierSpec`: `client_identifier` selects
/// the policy (`none`, `mac`, or `duid`) and `duid_raw` carries the raw DUID
/// bytes used when the policy is `duid`. The zero value is intentionally
/// meaningful: Talos' DHCPv6 operator passes no modifier to `nclient6`, whose
/// `NewSolicit` path still emits the upstream default DUID-LLT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientIdentifierSpec {
    /// Client identifier policy.
    pub client_identifier: ClientIdentifier,
    /// Raw DUID bytes, used only with [`ClientIdentifier::Duid`].
    pub duid_raw: Vec<u8>,
}

impl Default for ClientIdentifierSpec {
    fn default() -> Self {
        ClientIdentifierSpec {
            client_identifier: ClientIdentifier::None,
            duid_raw: Vec::new(),
        }
    }
}

impl ClientIdentifierSpec {
    /// Explicitly request the default/none policy.
    pub fn none() -> Self {
        Self::default()
    }

    /// Request link-MAC based client identifiers.
    pub fn mac() -> Self {
        ClientIdentifierSpec {
            client_identifier: ClientIdentifier::Mac,
            duid_raw: Vec::new(),
        }
    }

    /// Request a caller-provided raw DUID.
    pub fn duid(raw: impl Into<Vec<u8>>) -> Self {
        ClientIdentifierSpec {
            client_identifier: ClientIdentifier::Duid,
            duid_raw: raw.into(),
        }
    }

    /// Validate the upstream DUID constraints.
    pub fn validate(&self) -> Result<()> {
        match self.client_identifier {
            ClientIdentifier::Duid if self.duid_raw.is_empty() => Err(Error::invalid(
                "duidRaw must be set if clientIdentifier is 'duid'",
            )),
            ClientIdentifier::Duid => Ok(()),
            _ if !self.duid_raw.is_empty() => Err(Error::invalid(
                "duidRaw can only be set if clientIdentifier is 'duid'",
            )),
            _ => Ok(()),
        }
    }

    /// Convert this operator spec into the DHCPv6 wire identity to use.
    ///
    /// This models upstream `GetDHCPv6ClientIdentifier` plus `nclient6`'s
    /// `NewSolicit` default: `none` means no overriding modifier, so the
    /// default DUID-LLT derived from link MAC and DHCPv6 time is still sent.
    pub fn to_dhcp6_client_identifier(
        &self,
        mac: [u8; 6],
        seconds_since_2000: u32,
    ) -> Result<Dhcp6ClientIdentifier> {
        self.validate()?;
        match self.client_identifier {
            ClientIdentifier::None => Ok(Dhcp6ClientIdentifier::DuidLlt {
                mac,
                seconds_since_2000,
            }),
            ClientIdentifier::Mac => Ok(Dhcp6ClientIdentifier::Mac(mac)),
            ClientIdentifier::Duid => Ok(Dhcp6ClientIdentifier::Duid(self.duid_raw.clone())),
        }
    }
}

/// Declarative configuration that an operator should run on a link.
///
/// Equivalent to `network.OperatorSpecSpec`. The `route_metric` influences the
/// metric assigned to operator-installed routes (DHCP routes carry a higher
/// metric than static config routes by default).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorSpec {
    /// The operator kind.
    pub kind: OperatorKind,
    /// The link the operator runs on.
    pub link_name: String,
    /// Whether this operator requires the link to be up first.
    pub require_up: bool,
    /// Metric for routes the operator installs.
    pub route_metric: u32,
    /// Whether DHCP hostname/FQDN output should be ignored.
    pub skip_hostname_request: bool,
    /// DHCP client identifier override carried by the operator spec.
    pub client_identifier: ClientIdentifierSpec,
    /// Provenance / priority (operators are themselves configured at some
    /// layer, even though their *output* is at the operator layer).
    pub layer: ConfigLayer,
}

impl OperatorSpec {
    /// A DHCPv4 operator on a link, with Talos' default DHCP route metric.
    pub fn dhcp4(link_name: impl Into<String>) -> Self {
        OperatorSpec {
            kind: OperatorKind::Dhcp4,
            link_name: link_name.into(),
            require_up: true,
            route_metric: DEFAULT_ROUTE_METRIC,
            skip_hostname_request: false,
            client_identifier: ClientIdentifierSpec::default(),
            layer: ConfigLayer::Configuration,
        }
    }

    /// A DHCPv6 operator on a link.
    pub fn dhcp6(link_name: impl Into<String>) -> Self {
        OperatorSpec {
            kind: OperatorKind::Dhcp6,
            link_name: link_name.into(),
            require_up: true,
            route_metric: DEFAULT_ROUTE_METRIC,
            skip_hostname_request: false,
            client_identifier: ClientIdentifierSpec::default(),
            layer: ConfigLayer::Configuration,
        }
    }

    /// Validate the operator spec.
    pub fn validate(&self) -> Result<()> {
        if self.link_name.is_empty() {
            return Err(Error::invalid("operator spec has empty link name"));
        }
        if self.route_metric == 0 {
            return Err(Error::invalid("operator route metric must be non-zero"));
        }
        self.client_identifier.validate()?;
        Ok(())
    }

    /// Return a copy with the DHCP hostname/FQDN request suppressed.
    pub fn with_skip_hostname_request(mut self, skip: bool) -> Self {
        self.skip_hostname_request = skip;
        self
    }

    /// Return a copy with a DHCP client identifier override.
    pub fn with_client_identifier(mut self, spec: ClientIdentifierSpec) -> Self {
        self.client_identifier = spec;
        self
    }

    /// Resolve the DHCPv6 client identifier for a concrete link MAC/time.
    pub fn dhcp6_client_identifier(
        &self,
        mac: [u8; 6],
        seconds_since_2000: u32,
    ) -> Result<Dhcp6ClientIdentifier> {
        self.client_identifier
            .to_dhcp6_client_identifier(mac, seconds_since_2000)
    }

    /// Whether the operator should accept hostname/FQDN data from a lease.
    pub fn uses_hostname(&self) -> bool {
        !self.skip_hostname_request
    }

    /// Stable operator id: `<kind>/<link>`.
    pub fn id(&self) -> String {
        alloc::format!("{}/{}", self.kind.as_str(), self.link_name)
    }
}

/// The lease/result an operator produces after it runs, mirroring a parsed
/// DHCP `OFFER`/`ACK`. All produced specs are emitted at the operator layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorResult {
    /// Address to assign (with prefix length).
    pub address: NodeAddress,
    /// Prefix length for the assigned address.
    pub prefix_len: u8,
    /// Default gateway, if the lease carries a router option.
    pub gateway: Option<NodeAddress>,
    /// DNS servers from the lease.
    pub dns_servers: Vec<NodeAddress>,
    /// Hostname offered by the lease (DHCP option 12), if any.
    pub hostname: Option<String>,
    /// Search domains (DHCP option 119 / domain-name), if any.
    pub search_domains: Vec<String>,
}

impl OperatorResult {
    /// A minimal lease with just an address and prefix.
    pub fn address_only(address: NodeAddress, prefix_len: u8) -> Self {
        OperatorResult {
            address,
            prefix_len,
            gateway: None,
            dns_servers: Vec::new(),
            hostname: None,
            search_domains: Vec::new(),
        }
    }
}

/// The full set of specs an operator contributes after a successful run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OperatorOutput {
    /// Addresses to assign.
    pub addresses: Vec<AddressSpec>,
    /// Routes to install.
    pub routes: Vec<RouteSpec>,
    /// Hostname candidate, if the lease carried one.
    pub hostname: Option<HostnameSpec>,
    /// Resolver candidate, if the lease carried DNS servers.
    pub resolver: Option<ResolverSpec>,
    /// NTP/time servers contributed by DHCP operator output.
    pub time_servers: Vec<String>,
}

impl OperatorSpec {
    /// Translate an operator [`OperatorResult`] (lease) into the full set of
    /// specs it contributes. Mirrors how the DHCP operators publish addresses,
    /// a default route, a hostname and resolver at the operator layer.
    pub fn apply_result(&self, result: &OperatorResult) -> Result<OperatorOutput> {
        let mut out = OperatorOutput::default();

        let addr = AddressSpec::new(
            result.address,
            result.prefix_len,
            self.link_name.clone(),
            ConfigLayer::Operator,
        )?;
        out.addresses.push(addr);

        if let Some(gw) = result.gateway {
            let mut route =
                RouteSpec::default_via(gw, self.link_name.clone(), ConfigLayer::Operator)?;
            route.metric = self.route_metric;
            route.protocol = crate::route::RouteProtocol::Dhcp;
            out.routes.push(route);
        }

        if self.uses_hostname()
            && let Some(h) = &result.hostname {
                // A lease hostname may be an FQDN; split into host/domain.
                let spec = match h.split_once('.') {
                    Some((host, domain)) => {
                        HostnameSpec::with_domain(host, domain, ConfigLayer::Operator)?
                    }
                    None => HostnameSpec::new(h.clone(), ConfigLayer::Operator)?,
                };
                out.hostname = Some(spec);
            }

        if !result.dns_servers.is_empty() {
            let mut res = ResolverSpec::new(result.dns_servers.clone(), ConfigLayer::Operator)?;
            if !result.search_domains.is_empty() {
                res = res.with_search(result.search_domains.clone())?;
            }
            out.resolver = Some(res);
        }

        Ok(out)
    }
}

/// Build the operators implied by a static address configuration on a link.
/// Static config does not run a daemon, but Talos still models it uniformly:
/// this returns address/route specs at the configuration layer directly.
pub fn static_specs(
    link_name: &str,
    address: NodeAddress,
    prefix_len: u8,
    gateway: Option<NodeAddress>,
) -> Result<(AddressSpec, Option<RouteSpec>)> {
    let addr = AddressSpec::new(
        address,
        prefix_len,
        link_name.to_string(),
        ConfigLayer::Configuration,
    )?;
    let route = match gateway {
        Some(gw) => Some(RouteSpec::default_via(
            gw,
            link_name.to_string(),
            ConfigLayer::Configuration,
        )?),
        None => None,
    };
    Ok((addr, route))
}

/// Merge operator specs by their logical id, highest layer winning. Mirrors
/// `OperatorMergeController`.
pub fn merge_operators(specs: &[OperatorSpec]) -> Vec<OperatorSpec> {
    use alloc::collections::BTreeMap;
    let mut by_id: BTreeMap<String, OperatorSpec> = BTreeMap::new();
    for spec in specs {
        let id = spec.id();
        match by_id.get(&id) {
            Some(existing) if existing.layer.precedence() >= spec.layer.precedence() => {}
            _ => {
                by_id.insert(id, spec.clone());
            }
        }
    }
    by_id.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v4(s: &str) -> NodeAddress {
        NodeAddress::parse_v4(s).unwrap()
    }

    #[test]
    fn dhcp4_operator_id_and_validation() {
        let op = OperatorSpec::dhcp4("eth0");
        assert_eq!(op.id(), "dhcp4/eth0");
        assert!(op.validate().is_ok());

        let mut bad = op.clone();
        bad.route_metric = 0;
        assert!(bad.validate().is_err());
    }

    #[test]
    fn apply_full_lease() {
        let op = OperatorSpec::dhcp4("eth0");
        let result = OperatorResult {
            address: v4("10.0.0.50"),
            prefix_len: 24,
            gateway: Some(v4("10.0.0.1")),
            dns_servers: alloc::vec![v4("10.0.0.2"), v4("8.8.8.8")],
            hostname: Some("leased-host.example.com".to_string()),
            search_domains: alloc::vec!["example.com".to_string()],
        };
        let out = op.apply_result(&result).unwrap();

        assert_eq!(out.addresses.len(), 1);
        assert_eq!(out.addresses[0].layer, ConfigLayer::Operator);
        assert_eq!(out.addresses[0].id(), "eth0/10.0.0.50/24");

        let route = &out.routes[0];
        assert!(route.is_default());
        assert_eq!(route.protocol, crate::route::RouteProtocol::Dhcp);
        assert_eq!(route.metric, 1024);

        let hn = out.hostname.unwrap();
        assert_eq!(hn.hostname.as_str(), "leased-host");
        assert_eq!(hn.domainname.as_deref(), Some("example.com"));

        let res = out.resolver.unwrap();
        assert_eq!(res.servers.len(), 2);
        assert_eq!(res.search_domains, alloc::vec!["example.com".to_string()]);
    }

    #[test]
    fn apply_address_only_lease() {
        let op = OperatorSpec::dhcp6("eth0");
        let result = OperatorResult::address_only(v4("10.0.0.7"), 32);
        let out = op.apply_result(&result).unwrap();
        assert_eq!(out.addresses.len(), 1);
        assert!(out.routes.is_empty());
        assert!(out.hostname.is_none());
        assert!(out.resolver.is_none());
    }

    #[test]
    fn static_specs_with_and_without_gateway() {
        let (addr, route) =
            static_specs("eth0", v4("192.168.1.10"), 24, Some(v4("192.168.1.1"))).unwrap();
        assert_eq!(addr.layer, ConfigLayer::Configuration);
        assert!(route.unwrap().is_default());

        let (_, no_route) = static_specs("eth0", v4("192.168.1.10"), 24, None).unwrap();
        assert!(no_route.is_none());
    }

    #[test]
    fn merge_operators_dedup_by_id() {
        let a = OperatorSpec::dhcp4("eth0");
        let mut b = OperatorSpec::dhcp4("eth0");
        b.layer = ConfigLayer::Platform;
        let c = OperatorSpec::dhcp4("eth1");
        let merged = merge_operators(&[b, a.clone(), c]);
        // eth0 collapses to one (Configuration wins over Platform), eth1 stays
        assert_eq!(merged.len(), 2);
        let eth0: Vec<_> = merged.iter().filter(|o| o.link_name == "eth0").collect();
        assert_eq!(eth0.len(), 1);
        assert_eq!(eth0[0].layer, ConfigLayer::Configuration);
    }

    #[test]
    fn client_identifier_spec_maps_to_upstream_dhcp6_default() {
        let mac = [0x02, 0x00, 0x5e, 0x10, 0x20, 0x30];
        let spec = ClientIdentifierSpec::default();
        let config = crate::dhcp6::Dhcp6ClientConfig::new(
            [0x01, 0x02, 0x03],
            0x5e10_2030,
            spec.to_dhcp6_client_identifier(mac, 0x0102_0304).unwrap(),
        );

        assert_eq!(
            config.client_duid().unwrap().unwrap(),
            alloc::vec![
                0x00, 0x01, 0x00, 0x01, 0x01, 0x02, 0x03, 0x04, 0x02, 0x00, 0x5e, 0x10, 0x20, 0x30,
            ]
        );
    }

    #[test]
    fn client_identifier_spec_maps_mac_and_raw_duid() {
        let mac = [0x02, 0x00, 0x5e, 0x10, 0x20, 0x30];
        let mac_config = crate::dhcp6::Dhcp6ClientConfig::new(
            [0x01, 0x02, 0x03],
            0x5e10_2030,
            ClientIdentifierSpec::mac()
                .to_dhcp6_client_identifier(mac, 0)
                .unwrap(),
        );
        assert_eq!(
            mac_config.client_duid().unwrap().unwrap(),
            alloc::vec![0x00, 0x03, 0x00, 0x01, 0x02, 0x00, 0x5e, 0x10, 0x20, 0x30]
        );

        let raw = alloc::vec![0x00, 0x01, 0x00, 0x01, 0xaa, 0xbb, 0xcc, 0xdd];
        let raw_config = crate::dhcp6::Dhcp6ClientConfig::new(
            [0x01, 0x02, 0x03],
            0x5e10_2030,
            ClientIdentifierSpec::duid(raw.clone())
                .to_dhcp6_client_identifier(mac, 0)
                .unwrap(),
        );
        assert_eq!(raw_config.client_duid().unwrap().unwrap(), raw);
    }

    #[test]
    fn client_identifier_spec_rejects_invalid_duid_raw_shape() {
        assert!(ClientIdentifierSpec::duid(Vec::new()).validate().is_err());
        let invalid = ClientIdentifierSpec {
            client_identifier: ClientIdentifier::Mac,
            duid_raw: alloc::vec![1, 2, 3],
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn skip_hostname_request_suppresses_generic_operator_hostname() {
        let op = OperatorSpec::dhcp4("eth0").with_skip_hostname_request(true);
        let result = OperatorResult {
            address: v4("10.0.0.50"),
            prefix_len: 24,
            gateway: None,
            dns_servers: Vec::new(),
            hostname: Some("leased-host.example.com".to_string()),
            search_domains: Vec::new(),
        };
        let out = op.apply_result(&result).unwrap();
        assert!(out.hostname.is_none());
    }

    #[test]
    fn operator_kind_strings() {
        assert_eq!(OperatorKind::Dhcp4.as_str(), "dhcp4");
        assert_eq!(OperatorKind::Dhcp6.as_str(), "dhcp6");
        assert_eq!(OperatorKind::Vip.as_str(), "vip");
    }
}
