//! AWS config source, mirroring
//! `internal/app/machined/pkg/runtime/v1alpha1/platform/aws/{aws,metadata}.go`.
//!
//! On AWS the machine config is the EC2 **user-data** served by the Instance
//! Metadata Service (IMDS). Upstream uses the AWS SDK `imds.GetUserData`, which
//! targets `GET /latest/user-data` on the IMDS endpoint. The SDK races an IPv4
//! and an IPv6 endpoint (see `probeIMDS`), so both are modeled here:
//!
//! - IPv4: `http://169.254.169.254/latest/user-data`
//! - IPv6: `http://[fd00:ec2::254]/latest/user-data`
//!
//! IMDSv2 requires a session token presented via the `X-aws-ec2-metadata-token`
//! header (the SDK fetches it from `PUT /latest/api/token`); we encode that
//! header on the request as data.

use crate::source::{ConfigSource, Header};
use crate::{Mode, Platform};
use alloc::vec;
use alloc::vec::Vec;

/// IMDS IPv4 base address (`169.254.169.254`), per
/// AWS `instancedata-data-retrieval` docs.
pub const IMDS_IPV4_BASE: &str = "http://169.254.169.254";

/// IMDS IPv6 base address (`[fd00:ec2::254]`), the SDK's
/// `EndpointModeStateIPv6` endpoint.
pub const IMDS_IPV6_BASE: &str = "http://[fd00:ec2::254]";

/// IMDS user-data path (`GetUserData` → `/latest/user-data`).
pub const USER_DATA_PATH: &str = "/latest/user-data";

/// IMDSv2 session-token request header name.
pub const TOKEN_HEADER: &str = "X-aws-ec2-metadata-token";

/// Primary AWS interface name used by Talos platform network bootstrap.
pub const INTERFACE_NAME: &str = "eth0";

/// Talos default route metric for DHCP-provided routes.
pub const DEFAULT_DHCP_ROUTE_METRIC: u32 = 1024;

/// AWS pre-IMDS network bootstrap.
///
/// Upstream publishes this before fetching full metadata so IPv4-only,
/// IPv6-only, and dual-stack instances can all reach IMDS. It brings `eth0` up
/// and starts both DHCP families; the later metadata-derived config narrows the
/// final operators to the address families actually present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapNetworkConfig {
    /// Primary interface to bring up.
    pub interface: &'static str,
    /// Whether to start a DHCPv4 operator.
    pub dhcp4: bool,
    /// Whether to start a DHCPv6 operator.
    pub dhcp6: bool,
    /// Whether the DHCP operator requires the link to be up first.
    pub require_up: bool,
    /// Route metric used by DHCP operators.
    pub route_metric: u32,
}

/// AWS platform config source.
///
/// Mirrors `aws.AWS`. [`Mode`] is [`Mode::Cloud`].
#[derive(Debug, Default, Clone, Copy)]
pub struct Aws {
    /// IMDSv2 token to present via [`TOKEN_HEADER`], if known.
    token: Option<&'static str>,
}

impl Aws {
    /// Construct the AWS source with no session token (IMDSv1-style request).
    pub fn new() -> Self {
        Aws { token: None }
    }

    /// Construct with an IMDSv2 session token.
    pub fn with_token(token: &'static str) -> Self {
        Aws { token: Some(token) }
    }

    fn headers(self) -> Vec<Header> {
        match self.token {
            Some(tok) => vec![Header::new(TOKEN_HEADER, tok)],
            None => Vec::new(),
        }
    }

    /// Bootstrap network config emitted before IMDS metadata is available.
    pub fn bootstrap_network_config(self) -> BootstrapNetworkConfig {
        BootstrapNetworkConfig {
            interface: INTERFACE_NAME,
            dhcp4: true,
            dhcp6: true,
            require_up: true,
            route_metric: DEFAULT_DHCP_ROUTE_METRIC,
        }
    }
}

impl Platform for Aws {
    fn name(&self) -> &str {
        "aws"
    }

    fn mode(&self) -> Mode {
        Mode::Cloud
    }

    fn config_sources(&self) -> Vec<ConfigSource> {
        // IPv4 endpoint is preferred; IPv6 is the fallback the SDK races.
        vec![
            ConfigSource::http(
                alloc::format!("{IMDS_IPV4_BASE}{USER_DATA_PATH}"),
                self.headers(),
            ),
            ConfigSource::http(
                alloc::format!("{IMDS_IPV6_BASE}{USER_DATA_PATH}"),
                self.headers(),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{ConfigStore, MemoryStore};

    #[test]
    fn name_and_mode() {
        let p = Aws::new();
        assert_eq!(p.name(), "aws");
        assert_eq!(p.mode(), Mode::Cloud);
    }

    #[test]
    fn ipv4_user_data_endpoint_is_faithful() {
        let p = Aws::new();
        let src = &p.config_sources()[0];
        assert_eq!(src.url(), Some("http://169.254.169.254/latest/user-data"));
    }

    #[test]
    fn ipv6_fallback_endpoint_is_faithful() {
        let p = Aws::new();
        let src = &p.config_sources()[1];
        assert_eq!(src.url(), Some("http://[fd00:ec2::254]/latest/user-data"));
    }

    #[test]
    fn imdsv2_token_header_present() {
        let p = Aws::with_token("AQAE-token");
        let src = &p.config_sources()[0];
        assert_eq!(src.header("X-aws-ec2-metadata-token"), Some("AQAE-token"));
    }

    #[test]
    fn imdsv1_has_no_token_header() {
        let p = Aws::new();
        assert!(p.config_sources()[0].headers().is_empty());
    }

    #[test]
    fn bootstrap_network_config_brings_eth0_up_for_both_dhcp_families() {
        let bootstrap = Aws::new().bootstrap_network_config();
        assert_eq!(bootstrap.interface, "eth0");
        assert!(bootstrap.dhcp4);
        assert!(bootstrap.dhcp6);
        assert!(bootstrap.require_up);
        assert_eq!(bootstrap.route_metric, 1024);
    }

    #[test]
    fn aws_bootstrap_network_config_is_pre_imds_contract() {
        let bootstrap = Aws::new().bootstrap_network_config();
        assert_eq!(bootstrap.interface, INTERFACE_NAME);
        assert!(bootstrap.dhcp4, "AWS pre-IMDS bootstrap needs DHCPv4");
        assert!(bootstrap.dhcp6, "AWS pre-IMDS bootstrap needs DHCPv6");
        assert!(bootstrap.require_up);
        assert_eq!(bootstrap.route_metric, DEFAULT_DHCP_ROUTE_METRIC);
    }

    #[test]
    fn configuration_reads_user_data() {
        let p = Aws::new();
        let store = MemoryStore::new().with(
            &p.config_sources()[0],
            b"version: v1alpha1\nmachine: {}".to_vec(),
        );
        assert_eq!(
            p.configuration(&store as &dyn ConfigStore).unwrap(),
            b"version: v1alpha1\nmachine: {}".to_vec()
        );
    }

    #[test]
    fn configuration_falls_back_to_ipv6() {
        let p = Aws::new();
        // Only the IPv6 endpoint is reachable.
        let store = MemoryStore::new().with(&p.config_sources()[1], b"machine: ipv6".to_vec());
        assert_eq!(
            p.configuration(&store as &dyn ConfigStore).unwrap(),
            b"machine: ipv6".to_vec()
        );
    }

    #[test]
    fn configuration_errors_when_no_source() {
        let p = Aws::new();
        let store = MemoryStore::new();
        let err = p.configuration(&store as &dyn ConfigStore).unwrap_err();
        assert_eq!(err, crate::no_config_source());
    }
}
