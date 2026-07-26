//! `DHCPv6Config` — DHCPv6 client configuration for one network link.
//!
//! Mirrors Talos' network config document: `name:` is the link/interface name,
//! `routeMetric` optionally overrides DHCP route metrics, `ignoreHostname`
//! suppresses DHCP hostname/FQDN output, and `clientIdentifier` selects the
//! DHCPv6 client identifier mode (`mac` by default).

use crate::document::{ConfigDocument, DocId, DocKind};
use os_kernel::error::{Error, Result};

/// DHCP client identifier policy accepted by `DHCPv6Config`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DhcpClientIdentifier {
    /// No explicit override; the DHCPv6 client emits its default DUID-LLT.
    None,
    /// Link MAC based DUID-LL. This is the document default.
    Mac,
    /// Use `duid_raw` bytes verbatim.
    Duid,
}

impl DhcpClientIdentifier {
    /// Canonical config string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Mac => "mac",
            Self::Duid => "duid",
        }
    }

    /// Parse a config string.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.trim().to_ascii_lowercase().as_str() {
            "none" => Self::None,
            "mac" => Self::Mac,
            "duid" => Self::Duid,
            _ => return None,
        })
    }
}

/// The `DHCPv6Config` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DhcpV6Config {
    /// Link/interface name (document key).
    pub name: String,
    /// Optional route metric override (`None`/`0` delegates to the operator
    /// default).
    pub route_metric: Option<u32>,
    /// Optional hostname/FQDN ignore switch.
    pub ignore_hostname: Option<bool>,
    /// Client identifier policy. Talos defaults this field to `mac`.
    pub client_identifier: DhcpClientIdentifier,
    /// Raw DUID bytes for [`DhcpClientIdentifier::Duid`].
    pub duid_raw: Vec<u8>,
}

impl DhcpV6Config {
    /// Construct a minimal DHCPv6 config for `name`.
    pub fn new(name: impl Into<String>) -> Self {
        DhcpV6Config {
            name: name.into(),
            route_metric: None,
            ignore_hostname: None,
            client_identifier: DhcpClientIdentifier::Mac,
            duid_raw: Vec::new(),
        }
    }

    /// Builder: route metric.
    #[must_use]
    pub fn with_route_metric(mut self, route_metric: u32) -> Self {
        self.route_metric = Some(route_metric);
        self
    }

    /// Builder: ignore hostname/FQDN supplied by DHCP.
    #[must_use]
    pub fn with_ignore_hostname(mut self, ignore_hostname: bool) -> Self {
        self.ignore_hostname = Some(ignore_hostname);
        self
    }

    /// Builder: client identifier mode plus raw DUID bytes.
    #[must_use]
    pub fn with_client_identifier(
        mut self,
        client_identifier: DhcpClientIdentifier,
        duid_raw: impl Into<Vec<u8>>,
    ) -> Self {
        self.client_identifier = client_identifier;
        self.duid_raw = duid_raw.into();
        self
    }
}

impl ConfigDocument for DhcpV6Config {
    fn kind(&self) -> DocKind {
        DocKind::DhcpV6
    }

    fn id(&self) -> DocId {
        DocId::keyed(DocKind::DhcpV6, self.name.clone())
    }

    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::invalid("DHCPv6Config: name is required"));
        }
        match self.client_identifier {
            DhcpClientIdentifier::Duid if self.duid_raw.is_empty() => Err(Error::invalid(
                "DHCPv6Config: duidRaw must be set if clientIdentifier is 'duid'",
            )),
            DhcpClientIdentifier::Duid => Ok(()),
            _ if !self.duid_raw.is_empty() => Err(Error::invalid(
                "DHCPv6Config: duidRaw can only be set if clientIdentifier is 'duid'",
            )),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dhcpv6_config_validates_minimal_named_doc() {
        let doc = DhcpV6Config::new("enp0s2");
        assert!(doc.validate().is_ok());
        assert_eq!(doc.kind(), DocKind::DhcpV6);
        assert_eq!(doc.id(), DocId::keyed(DocKind::DhcpV6, "enp0s2"));
        assert!(doc.allows_multiple());
        assert_eq!(doc.client_identifier, DhcpClientIdentifier::Mac);
    }

    #[test]
    fn dhcpv6_config_maps_client_identifier_modes() {
        assert_eq!(
            DhcpClientIdentifier::parse("none"),
            Some(DhcpClientIdentifier::None)
        );
        assert_eq!(
            DhcpClientIdentifier::parse("MAC"),
            Some(DhcpClientIdentifier::Mac)
        );
        assert_eq!(
            DhcpClientIdentifier::parse("duid"),
            Some(DhcpClientIdentifier::Duid)
        );
        assert_eq!(DhcpClientIdentifier::parse("off"), None);
    }

    #[test]
    fn dhcpv6_config_rejects_empty_name() {
        assert!(DhcpV6Config::new("").validate().is_err());
    }

    #[test]
    fn dhcpv6_config_rejects_duid_identifier_without_duid_raw() {
        let doc = DhcpV6Config::new("eth0")
            .with_client_identifier(DhcpClientIdentifier::Duid, Vec::new());
        assert!(doc.validate().is_err());
    }

    #[test]
    fn dhcpv6_config_rejects_duid_raw_without_duid_identifier() {
        let doc =
            DhcpV6Config::new("eth0").with_client_identifier(DhcpClientIdentifier::Mac, vec![0, 1]);
        assert!(doc.validate().is_err());
    }

    #[test]
    fn dhcpv6_config_accepts_raw_duid() {
        let doc = DhcpV6Config::new("eth0")
            .with_route_metric(2048)
            .with_ignore_hostname(true)
            .with_client_identifier(DhcpClientIdentifier::Duid, vec![0, 1, 2, 3]);
        assert!(doc.validate().is_ok());
        assert_eq!(doc.route_metric, Some(2048));
        assert_eq!(doc.ignore_hostname, Some(true));
    }
}
