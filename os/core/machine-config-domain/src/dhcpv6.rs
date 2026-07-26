//! Typed accessors for the `DHCPv6Config` multi-document config.
//!
//! Talos v1.12 introduced name-keyed DHCP documents for dynamic addressing. A
//! `DHCPv6Config` document enables DHCPv6 on the link named by `name:` and
//! carries the same optional DHCP knobs as the network operator model:
//! `routeMetric`, `ignoreHostname`, `clientIdentifier`, and `duidRaw`.

use crate::container::Config;
use crate::yaml::{self, Yaml};
use std::collections::BTreeSet;
use os_kernel::error::{Error, Result};

/// Canonical Talos document kind.
pub const DHCPV6_CONFIG_KIND: &str = "DHCPv6Config";

/// DHCP client identifier policy accepted by `DHCPv6Config`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DhcpV6ClientIdentifier {
    /// Disable the explicit client identifier override; the DHCPv6 client uses
    /// its default DUID-LLT behavior.
    None,
    /// Use the link MAC address as a DUID-LL identifier. This is the Talos
    /// `DHCPv6Config` default.
    Mac,
    /// Use `duidRaw` bytes verbatim.
    Duid,
}

impl DhcpV6ClientIdentifier {
    /// Parse the YAML string value.
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "none" => Ok(Self::None),
            "mac" => Ok(Self::Mac),
            "duid" => Ok(Self::Duid),
            other => Err(Error::parse(format!(
                "DHCPv6Config: unknown clientIdentifier '{other}'"
            ))),
        }
    }

    /// Canonical string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Mac => "mac",
            Self::Duid => "duid",
        }
    }
}

/// Parsed `DHCPv6Config` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DhcpV6Config {
    /// Link/interface name.
    pub name: String,
    /// Route metric override (`0` means use the operator default).
    pub route_metric: u32,
    /// Whether to ignore hostname/FQDN supplied by the DHCP server.
    pub ignore_hostname: bool,
    /// Client identifier policy.
    pub client_identifier: DhcpV6ClientIdentifier,
    /// Raw DUID bytes, used only when [`DhcpV6ClientIdentifier::Duid`] is set.
    pub duid_raw: Vec<u8>,
}

impl DhcpV6Config {
    /// Build a minimal DHCPv6 config for `name`.
    pub fn new(name: impl Into<String>) -> Self {
        DhcpV6Config {
            name: name.into(),
            route_metric: 0,
            ignore_hostname: false,
            client_identifier: DhcpV6ClientIdentifier::Mac,
            duid_raw: Vec::new(),
        }
    }

    /// Effective route metric after applying a caller-provided default.
    pub fn route_metric_or(&self, default_metric: u32) -> u32 {
        if self.route_metric == 0 {
            default_metric
        } else {
            self.route_metric
        }
    }

    /// Validate the DHCPv6 document in isolation.
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::invalid("DHCPv6Config: name is required"));
        }
        match self.client_identifier {
            DhcpV6ClientIdentifier::Duid if self.duid_raw.is_empty() => Err(Error::invalid(
                "DHCPv6Config: duidRaw must be set if clientIdentifier is 'duid'",
            )),
            DhcpV6ClientIdentifier::Duid => Ok(()),
            _ if !self.duid_raw.is_empty() => Err(Error::invalid(
                "DHCPv6Config: duidRaw can only be set if clientIdentifier is 'duid'",
            )),
            _ => Ok(()),
        }
    }
}

/// Decode and validate one `DHCPv6Config` document body.
pub fn decode_dhcpv6_config_body(body: &str) -> Result<DhcpV6Config> {
    let root = yaml::parse(body).map_err(|e| Error::parse(e.to_string()))?;
    if let Some(kind) = root.get_str("kind")
        && kind != DHCPV6_CONFIG_KIND
    {
        return Err(Error::invalid(format!(
            "DHCPv6Config: unexpected kind '{kind}'"
        )));
    }

    let name = root
        .get_str("name")
        .ok_or_else(|| Error::invalid("DHCPv6Config: name is required"))?
        .trim()
        .to_string();
    let route_metric =
        optional_u32(root.get("routeMetric"), "DHCPv6Config.routeMetric")?.unwrap_or_default();
    let ignore_hostname =
        optional_bool(root.get("ignoreHostname"), "DHCPv6Config.ignoreHostname")?.unwrap_or(false);
    let client_identifier = match root.get_str("clientIdentifier") {
        Some(value) if !value.trim().is_empty() => DhcpV6ClientIdentifier::parse(value)?,
        _ => DhcpV6ClientIdentifier::Mac,
    };
    let duid_raw = match root.get_str("duidRaw") {
        Some(value) if !value.trim().is_empty() => parse_hardware_addr(value)?,
        _ => Vec::new(),
    };

    let config = DhcpV6Config {
        name,
        route_metric,
        ignore_hostname,
        client_identifier,
        duid_raw,
    };
    config.validate()?;
    Ok(config)
}

/// Extract all `DHCPv6Config` docs from a loaded config, rejecting duplicate
/// link names.
pub fn dhcpv6_configs(config: &Config) -> Result<Vec<DhcpV6Config>> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for doc in config
        .documents()
        .iter()
        .filter(|doc| doc.meta.kind == DHCPV6_CONFIG_KIND)
    {
        let parsed = decode_dhcpv6_config_body(&doc.body)?;
        if !seen.insert(parsed.name.clone()) {
            return Err(Error::invalid(format!(
                "duplicate DHCPv6Config document for link '{}'",
                parsed.name
            )));
        }
        out.push(parsed);
    }
    Ok(out)
}

fn optional_bool(value: Option<&Yaml>, field: &str) -> Result<Option<bool>> {
    let Some(value) = value else {
        return Ok(None);
    };
    value
        .as_bool()
        .map(Some)
        .ok_or_else(|| Error::parse(format!("{field} must be a boolean")))
}

fn optional_u32(value: Option<&Yaml>, field: &str) -> Result<Option<u32>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let Some(raw) = value.as_str() else {
        return Err(Error::parse(format!("{field} must be an integer")));
    };
    raw.parse::<u32>()
        .map(Some)
        .map_err(|_| Error::parse(format!("{field} must be an unsigned 32-bit integer")))
}

fn parse_hardware_addr(raw: &str) -> Result<Vec<u8>> {
    let compact: String = raw.chars().filter(|c| *c != ':').collect();
    if compact.is_empty() {
        return Ok(Vec::new());
    }
    if !compact.len().is_multiple_of(2) {
        return Err(Error::invalid(
            "DHCPv6Config.duidRaw must be an even-length hexadecimal hardware address",
        ));
    }
    let mut out = Vec::with_capacity(compact.len() / 2);
    for idx in (0..compact.len()).step_by(2) {
        let byte = u8::from_str_radix(&compact[idx..idx + 2], 16)
            .map_err(|_| Error::invalid("DHCPv6Config.duidRaw must be hexadecimal"))?;
        out.push(byte);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load::load_from_bytes;

    const BASE: &str = "\
version: v1alpha1
machine:
  type: worker
";

    fn multidoc(doc: &str) -> String {
        format!("{BASE}---\n{doc}")
    }

    #[test]
    fn dhcpv6_config_decodes_named_doc_with_defaults() {
        let doc = "\
apiVersion: v1alpha1
kind: DHCPv6Config
name: eth0
";
        let parsed = decode_dhcpv6_config_body(doc).unwrap();
        assert_eq!(parsed.name, "eth0");
        assert_eq!(parsed.route_metric, 0);
        assert_eq!(parsed.route_metric_or(1024), 1024);
        assert!(!parsed.ignore_hostname);
        assert_eq!(parsed.client_identifier, DhcpV6ClientIdentifier::Mac);
        assert!(parsed.duid_raw.is_empty());
    }

    #[test]
    fn dhcpv6_config_decodes_all_operator_fields() {
        let doc = "\
apiVersion: v1alpha1
kind: DHCPv6Config
name: eth0
routeMetric: 2048
ignoreHostname: true
clientIdentifier: duid
duidRaw: 00:03:00:01:aa:bb:cc:dd:ee:ff
";
        let parsed = decode_dhcpv6_config_body(doc).unwrap();
        assert_eq!(parsed.route_metric_or(1024), 2048);
        assert!(parsed.ignore_hostname);
        assert_eq!(parsed.client_identifier, DhcpV6ClientIdentifier::Duid);
        assert_eq!(
            parsed.duid_raw,
            vec![0, 3, 0, 1, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]
        );
    }

    #[test]
    fn dhcpv6_config_rejects_empty_name() {
        let err =
            decode_dhcpv6_config_body("apiVersion: v1alpha1\nkind: DHCPv6Config\nname: \"\"\n")
                .unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    #[test]
    fn dhcpv6_config_validates_duid_shape() {
        assert!(
            decode_dhcpv6_config_body(
                "apiVersion: v1alpha1\nkind: DHCPv6Config\nname: eth0\nclientIdentifier: duid\n",
            )
            .is_err()
        );
        assert!(
            decode_dhcpv6_config_body(
                "apiVersion: v1alpha1\nkind: DHCPv6Config\nname: eth0\nclientIdentifier: mac\nduidRaw: 00:01\n",
            )
            .is_err()
        );
        assert!(
            decode_dhcpv6_config_body(
                "apiVersion: v1alpha1\nkind: DHCPv6Config\nname: eth0\nclientIdentifier: duid\nduidRaw: abc\n",
            )
            .is_err()
        );
        assert!(
            decode_dhcpv6_config_body(
                "apiVersion: v1alpha1\nkind: DHCPv6Config\nname: eth0\nclientIdentifier: duid\nduidRaw: 00:zz\n",
            )
            .is_err()
        );
    }

    #[test]
    fn dhcpv6_configs_extracts_multiple_named_documents() {
        let cfg = multidoc(
            "\
apiVersion: v1alpha1
kind: DHCPv6Config
name: eth0
---
apiVersion: v1alpha1
kind: DHCPv6Config
name: eth1
routeMetric: 2048
",
        );
        let container = load_from_bytes(&cfg).unwrap();
        let docs = dhcpv6_configs(&container).unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].name, "eth0");
        assert_eq!(docs[1].name, "eth1");
        assert_eq!(docs[1].route_metric, 2048);
    }

    #[test]
    fn dhcpv6_config_load_rejects_duplicate_names() {
        let cfg = multidoc(
            "\
apiVersion: v1alpha1
kind: DHCPv6Config
name: eth0
---
apiVersion: v1alpha1
kind: DHCPv6Config
name: eth0
",
        );
        let err = load_from_bytes(&cfg).unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }
}
