//! The `machine.network` sub-tree, mirroring `NetworkConfig`, `Device`,
//! `DeviceVlan`, `Route`, `Bond`, and DNS/hostname settings in
//! `pkg/machinery/config/types/v1alpha1`.

use crate::defaults;
use crate::validation::{
    ValidationError, ValidationMode, ValidationReport, Validator, is_cidr, is_hostname, is_ip,
};

/// A static route attached to an interface (`machine.network.interfaces[].routes`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Route {
    /// Destination network in CIDR form (empty = default route).
    pub network: String,
    /// Next-hop gateway IP.
    pub gateway: String,
    /// Optional source address.
    pub source: String,
    /// Optional metric.
    pub metric: u32,
}

impl Route {
    /// Whether this is the default route (`0.0.0.0/0` or empty network).
    pub fn is_default(&self) -> bool {
        self.network.is_empty() || self.network == "0.0.0.0/0" || self.network == "::/0"
    }
}

impl Validator for Route {
    fn validate_into(&self, _mode: ValidationMode, report: &mut ValidationReport) {
        if !self.network.is_empty() && !is_cidr(&self.network) {
            report.push(ValidationError::invalid(
                "machine.network.interfaces[].routes[].network",
                format!("'{}' is not a CIDR", self.network),
            ));
        }
        if !self.gateway.is_empty() && !is_ip(&self.gateway) {
            report.push(ValidationError::invalid(
                "machine.network.interfaces[].routes[].gateway",
                format!("'{}' is not an IP", self.gateway),
            ));
        }
    }
}

/// DHCP behavior attached to a legacy v1alpha1 interface or VLAN.
///
/// Mirrors upstream `DHCPOptions` in
/// `pkg/machinery/config/types/v1alpha1/v1alpha1_types.go`: route metric `0`
/// means "use the network operator default", IPv4 defaults to enabled, IPv6
/// defaults to disabled, and `duidv6` carries a raw DHCPv6 DUID hex string.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DhcpOptions {
    /// Route metric override (`0` delegates to the operator default).
    pub route_metric: u32,
    /// Optional IPv4 DHCP toggle; absent means enabled.
    pub ipv4: Option<bool>,
    /// Optional IPv6 DHCP toggle; absent means disabled.
    pub ipv6: Option<bool>,
    /// Raw DHCPv6 DUID encoded as hexadecimal text.
    pub duid_v6: String,
}

impl DhcpOptions {
    /// Whether DHCPv4 should be configured.
    pub fn ipv4(&self) -> bool {
        self.ipv4.unwrap_or(true)
    }

    /// Whether DHCPv6 should be configured.
    pub fn ipv6(&self) -> bool {
        self.ipv6.unwrap_or(false)
    }

    /// Raw route metric value from config (`0` means use caller default).
    pub fn route_metric(&self) -> u32 {
        self.route_metric
    }

    /// Effective route metric after applying a caller-provided Talos default.
    pub fn route_metric_or(&self, default_metric: u32) -> u32 {
        if self.route_metric == 0 {
            default_metric
        } else {
            self.route_metric
        }
    }

    /// The configured DHCPv6 DUID hex string.
    pub fn duid_v6(&self) -> &str {
        &self.duid_v6
    }

    /// Whether a custom DHCPv6 DUID was configured.
    pub fn has_duid_v6(&self) -> bool {
        !self.duid_v6.is_empty()
    }
}

impl Validator for DhcpOptions {
    fn validate_into(&self, _mode: ValidationMode, report: &mut ValidationReport) {
        if !self.duid_v6.is_empty()
            && (!self.duid_v6.len().is_multiple_of(2) || !self.duid_v6.bytes().all(|b| b.is_ascii_hexdigit())) {
                report.push(ValidationError::invalid(
                    "machine.network.interfaces[].dhcpOptions.duidv6",
                    "must be an even-length hexadecimal string",
                ));
            }
    }
}

/// A tagged VLAN on top of an interface (`...interfaces[].vlans`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Vlan {
    /// VLAN id (1..=4094).
    pub id: u16,
    /// Static addresses in CIDR form.
    pub addresses: Vec<String>,
    /// Whether to use DHCP for this VLAN.
    pub dhcp: bool,
    /// DHCP behavior (`dhcpOptions`).
    pub dhcp_options: DhcpOptions,
    /// Routes scoped to this VLAN.
    pub routes: Vec<Route>,
}

impl Validator for Vlan {
    fn validate_into(&self, mode: ValidationMode, report: &mut ValidationReport) {
        if self.id == 0 || self.id > 4094 {
            report.push(ValidationError::invalid(
                "machine.network.interfaces[].vlans[].vlanId",
                format!("VLAN id {} out of range 1..=4094", self.id),
            ));
        }
        for addr in &self.addresses {
            if !is_cidr(addr) {
                report.push(ValidationError::invalid(
                    "machine.network.interfaces[].vlans[].addresses",
                    format!("'{addr}' is not a CIDR"),
                ));
            }
        }
        self.dhcp_options.validate_into(mode, report);
        for r in &self.routes {
            r.validate_into(mode, report);
        }
    }
}

/// Bond mode for a bonded interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BondMode {
    RoundRobin,
    ActiveBackup,
    Xor,
    Broadcast,
    Lacp8023ad,
    TlbTransmit,
    AlbAdaptive,
}

impl BondMode {
    /// The kernel bonding mode string.
    pub fn as_str(self) -> &'static str {
        match self {
            BondMode::RoundRobin => "balance-rr",
            BondMode::ActiveBackup => "active-backup",
            BondMode::Xor => "balance-xor",
            BondMode::Broadcast => "broadcast",
            BondMode::Lacp8023ad => "802.3ad",
            BondMode::TlbTransmit => "balance-tlb",
            BondMode::AlbAdaptive => "balance-alb",
        }
    }
}

/// Bonding configuration for an interface (`...interfaces[].bond`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bond {
    /// Member interface names.
    pub interfaces: Vec<String>,
    /// Bond mode.
    pub mode: BondMode,
}

impl Bond {
    /// New active-backup bond over `members`.
    pub fn active_backup(members: Vec<String>) -> Self {
        Bond {
            interfaces: members,
            mode: BondMode::ActiveBackup,
        }
    }
}

/// A physical or virtual network interface (`machine.network.interfaces[]`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Device {
    /// Interface name (`eth0`) or device selector predicate.
    pub interface: String,
    /// Static addresses in CIDR form.
    pub addresses: Vec<String>,
    /// Whether DHCP is enabled.
    pub dhcp: bool,
    /// DHCP behavior (`dhcpOptions`).
    pub dhcp_options: DhcpOptions,
    /// Whether this is the primary interface used for the node address.
    pub ignore: bool,
    /// Interface MTU.
    pub mtu: u32,
    /// Static routes.
    pub routes: Vec<Route>,
    /// VLANs riding on this interface.
    pub vlans: Vec<Vlan>,
    /// Optional bonding.
    pub bond: Option<Bond>,
}

impl Device {
    /// A DHCP interface by name.
    pub fn dhcp(interface: impl Into<String>) -> Self {
        Device {
            interface: interface.into(),
            dhcp: true,
            ..Default::default()
        }
    }

    /// A static interface with a single CIDR address.
    pub fn static_addr(interface: impl Into<String>, cidr: impl Into<String>) -> Self {
        Device {
            interface: interface.into(),
            addresses: vec![cidr.into()],
            ..Default::default()
        }
    }

    /// Whether this device has any addressing configured.
    pub fn is_configured(&self) -> bool {
        self.dhcp || !self.addresses.is_empty() || self.bond.is_some()
    }
}

impl Validator for Device {
    fn validate_into(&self, mode: ValidationMode, report: &mut ValidationReport) {
        if self.interface.is_empty() && self.bond.is_none() {
            report.push(ValidationError::missing(
                "machine.network.interfaces[].interface",
            ));
        }
        for addr in &self.addresses {
            if !is_cidr(addr) {
                report.push(ValidationError::invalid(
                    "machine.network.interfaces[].addresses",
                    format!("'{addr}' must be a CIDR (with prefix length)"),
                ));
            }
        }
        if self.mtu != 0 && (self.mtu < 576 || self.mtu > 65535) {
            report.push(ValidationError::invalid(
                "machine.network.interfaces[].mtu",
                format!("MTU {} out of range", self.mtu),
            ));
        }
        // DHCP and static addresses both set on the same device is allowed by
        // Talos but a bond with members AND addresses on the member is not the
        // job of this check; routes/vlans recurse below.
        self.dhcp_options.validate_into(mode, report);
        if let Some(bond) = &self.bond
            && bond.interfaces.len() < 2 {
                report.push(ValidationError::invalid(
                    "machine.network.interfaces[].bond.interfaces",
                    "a bond requires at least two member interfaces",
                ));
            }
        for r in &self.routes {
            r.validate_into(mode, report);
        }
        for v in &self.vlans {
            v.validate_into(mode, report);
        }
    }
}

/// The `machine.network` sub-tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkConfig {
    /// Static hostname (`machine.network.hostname`).
    pub hostname: String,
    /// Interfaces.
    pub interfaces: Vec<Device>,
    /// Nameservers (`machine.network.nameservers`).
    pub nameservers: Vec<String>,
    /// Static `/etc/hosts` entries: (ip, [aliases]).
    pub extra_hosts: Vec<(String, Vec<String>)>,
    /// Whether to disable kube-proxy-managed search domains, etc. (`disableSearchDomain`).
    pub disable_search_domain: bool,
}

impl NetworkConfig {
    /// Apply Talos defaults (currently a no-op placeholder kept for symmetry;
    /// hostname/MTU defaults are derived at runtime, not in config).
    pub fn apply_defaults(&mut self) {
        for dev in &mut self.interfaces {
            if dev.mtu == 0 {
                dev.mtu = defaults::DEFAULT_MTU;
            }
        }
    }

    /// Find the first non-ignored, configured interface (the one Talos would
    /// pick as the node's primary address source).
    pub fn primary_interface(&self) -> Option<&Device> {
        self.interfaces
            .iter()
            .find(|d| !d.ignore && d.is_configured())
    }
}

impl Validator for NetworkConfig {
    fn validate_into(&self, mode: ValidationMode, report: &mut ValidationReport) {
        if !self.hostname.is_empty() && !is_hostname(&self.hostname) {
            report.push(ValidationError::invalid(
                "machine.network.hostname",
                format!("'{}' is not a valid hostname", self.hostname),
            ));
        }
        for ns in &self.nameservers {
            if !is_ip(ns) {
                report.push(ValidationError::invalid(
                    "machine.network.nameservers",
                    format!("'{ns}' is not an IP"),
                ));
            }
        }
        for (ip, _aliases) in &self.extra_hosts {
            if !is_ip(ip) {
                report.push(ValidationError::invalid(
                    "machine.network.extraHostEntries[].ip",
                    format!("'{ip}' is not an IP"),
                ));
            }
        }
        for dev in &self.interfaces {
            dev.validate_into(mode, report);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dhcp_device_is_configured() {
        let d = Device::dhcp("eth0");
        assert!(d.is_configured());
        assert!(d.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn static_address_must_be_cidr() {
        let mut d = Device::static_addr("eth0", "10.0.0.5");
        // Missing prefix -> not a CIDR.
        assert!(d.validate(ValidationMode::Metal).is_err());
        d.addresses = vec!["10.0.0.5/24".to_string()];
        assert!(d.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn vlan_id_range_enforced() {
        let mut v = Vlan {
            id: 0,
            ..Default::default()
        };
        assert!(v.validate(ValidationMode::Metal).is_err());
        v.id = 4095;
        assert!(v.validate(ValidationMode::Metal).is_err());
        v.id = 100;
        assert!(v.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn bond_needs_two_members() {
        let mut d = Device {
            interface: "bond0".to_string(),
            bond: Some(Bond::active_backup(vec!["eth0".to_string()])),
            ..Device::default()
        };
        assert!(d.validate(ValidationMode::Metal).is_err());
        d.bond = Some(Bond::active_backup(vec![
            "eth0".to_string(),
            "eth1".to_string(),
        ]));
        assert!(d.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn default_route_detection() {
        let r = Route {
            network: "0.0.0.0/0".to_string(),
            gateway: "10.0.0.1".to_string(),
            ..Default::default()
        };
        assert!(r.is_default());
        let r2 = Route {
            network: "192.168.0.0/24".to_string(),
            ..Default::default()
        };
        assert!(!r2.is_default());
    }

    #[test]
    fn network_validates_hostname_and_nameservers() {
        let mut n = NetworkConfig {
            hostname: "bad_host!".to_string(),
            nameservers: vec!["notanip".to_string()],
            ..NetworkConfig::default()
        };
        assert!(n.validate(ValidationMode::Metal).is_err());
        n.hostname = "node-1".to_string();
        n.nameservers = vec!["1.1.1.1".to_string()];
        assert!(n.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn primary_interface_skips_ignored() {
        let mut n = NetworkConfig::default();
        let mut ignored = Device::dhcp("eth0");
        ignored.ignore = true;
        n.interfaces.push(ignored);
        n.interfaces.push(Device::dhcp("eth1"));
        assert_eq!(n.primary_interface().unwrap().interface, "eth1");
    }

    #[test]
    fn apply_defaults_sets_mtu() {
        let mut n = NetworkConfig::default();
        n.interfaces.push(Device::dhcp("eth0"));
        n.apply_defaults();
        assert_eq!(n.interfaces[0].mtu, defaults::DEFAULT_MTU);
    }

    #[test]
    fn bond_mode_strings() {
        assert_eq!(BondMode::Lacp8023ad.as_str(), "802.3ad");
        assert_eq!(BondMode::ActiveBackup.as_str(), "active-backup");
    }

    #[test]
    fn dhcp_options_match_legacy_defaults() {
        let opts = DhcpOptions::default();

        assert!(opts.ipv4());
        assert!(!opts.ipv6());
        assert_eq!(opts.route_metric(), 0);
        assert_eq!(opts.route_metric_or(1024), 1024);
        assert_eq!(opts.duid_v6(), "");
        assert!(!opts.has_duid_v6());
        assert!(opts.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn device_dhcp_options_drive_ipv6_operator_inputs() {
        let d = Device {
            interface: "eth0".to_string(),
            dhcp: true,
            dhcp_options: DhcpOptions {
                route_metric: 512,
                ipv4: Some(false),
                ipv6: Some(true),
                duid_v6: "00030001aabbccddeeff".to_string(),
            },
            ..Device::default()
        };

        assert!(d.is_configured());
        assert!(!d.dhcp_options.ipv4());
        assert!(d.dhcp_options.ipv6());
        assert_eq!(d.dhcp_options.route_metric_or(1024), 512);
        assert_eq!(d.dhcp_options.duid_v6(), "00030001aabbccddeeff");
        assert!(d.dhcp_options.has_duid_v6());
        assert!(d.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn vlan_dhcp_options_are_modelled() {
        let v = Vlan {
            id: 4094,
            dhcp: true,
            dhcp_options: DhcpOptions {
                route_metric: 2048,
                ipv4: Some(false),
                ipv6: Some(true),
                duid_v6: String::new(),
            },
            ..Vlan::default()
        };

        assert!(!v.dhcp_options.ipv4());
        assert!(v.dhcp_options.ipv6());
        assert_eq!(v.dhcp_options.route_metric_or(1024), 2048);
        assert!(v.validate(ValidationMode::Metal).is_ok());
    }

    #[test]
    fn dhcp_options_reject_bad_duid_hex() {
        let opts = DhcpOptions {
            duid_v6: "not-hex".to_string(),
            ..DhcpOptions::default()
        };

        assert!(opts.validate(ValidationMode::Metal).is_err());
    }
}
