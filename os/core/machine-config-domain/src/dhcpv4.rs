//! Typed accessors for the `DHCPv4Config` multi-document config.
//!
//! Talos v1.12 introduced name-keyed DHCP documents for dynamic addressing. A
//! `DHCPv4Config` document enables DHCPv4 on the link named by `name:` and
//! carries the same optional DHCP knobs as the network operator model:
//! `routeMetric`, `ignoreHostname`, `clientIdentifier`, and `duidRaw`.

use crate::container::Config;
use crate::yaml::{self, Yaml};
use os_kernel::error::{Error, Result};
use std::collections::BTreeSet;

/// Canonical Talos document kind.
pub const DHCPV4_CONFIG_KIND: &str = "DHCPv4Config";

/// DHCP client identifier policy accepted by `DHCPv4Config`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DhcpV4ClientIdentifier {
    /// Disable the explicit client identifier override; the DHCPv4 client uses
    /// no option 61 override.
    None,
    /// Use the link MAC address as the client identifier. This is the Talos
    /// `DHCPv4Config` default.
    Mac,
    /// Use `duidRaw` bytes verbatim.
    Duid,
}

impl DhcpV4ClientIdentifier {
    /// Parse the YAML string value.
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "none" => Ok(Self::None),
            "mac" => Ok(Self::Mac),
            "duid" => Ok(Self::Duid),
            other => Err(Error::parse(format!(
                "DHCPv4Config: unknown clientIdentifier '{other}'"
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

/// Parsed `DHCPv4Config` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DhcpV4Config {
    /// Link/interface name.
    pub name: String,
    /// Route metric override (`0` means use the operator default).
    pub route_metric: u32,
    /// Whether to ignore hostname/FQDN supplied by the DHCP server.
    pub ignore_hostname: bool,
    /// Client identifier policy.
    pub client_identifier: DhcpV4ClientIdentifier,
    /// Raw client-identifier bytes, used only when [`DhcpV4ClientIdentifier::Duid`] is set.
    pub duid_raw: Vec<u8>,
}

impl DhcpV4Config {
    /// Build a minimal DHCPv4 config for `name`.
    pub fn new(name: impl Into<String>) -> Self {
        DhcpV4Config {
            name: name.into(),
            route_metric: 0,
            ignore_hostname: false,
            client_identifier: DhcpV4ClientIdentifier::Mac,
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

    /// Validate the DHCPv4 document in isolation.
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::invalid("DHCPv4Config: name is required"));
        }
        match self.client_identifier {
            DhcpV4ClientIdentifier::Duid if self.duid_raw.is_empty() => Err(Error::invalid(
                "DHCPv4Config: duidRaw must be set if clientIdentifier is 'duid'",
            )),
            DhcpV4ClientIdentifier::Duid => Ok(()),
            _ if !self.duid_raw.is_empty() => Err(Error::invalid(
                "DHCPv4Config: duidRaw can only be set if clientIdentifier is 'duid'",
            )),
            _ => Ok(()),
        }
    }
}

/// Decode and validate one `DHCPv4Config` document body.
pub fn decode_dhcpv4_config_body(body: &str) -> Result<DhcpV4Config> {
    let root = yaml::parse(body).map_err(|e| Error::parse(e.to_string()))?;
    if let Some(kind) = root.get_str("kind")
        && kind != DHCPV4_CONFIG_KIND
    {
        return Err(Error::invalid(format!(
            "DHCPv4Config: unexpected kind '{kind}'"
        )));
    }

    let name = root
        .get_str("name")
        .ok_or_else(|| Error::invalid("DHCPv4Config: name is required"))?
        .trim()
        .to_string();
    let route_metric =
        optional_u32(root.get("routeMetric"), "DHCPv4Config.routeMetric")?.unwrap_or_default();
    let ignore_hostname =
        optional_bool(root.get("ignoreHostname"), "DHCPv4Config.ignoreHostname")?.unwrap_or(false);
    let client_identifier = match root.get_str("clientIdentifier") {
        Some(value) if !value.trim().is_empty() => DhcpV4ClientIdentifier::parse(value)?,
        _ => DhcpV4ClientIdentifier::Mac,
    };
    let duid_raw = match root.get_str("duidRaw") {
        Some(value) if !value.trim().is_empty() => parse_hardware_addr(value)?,
        _ => Vec::new(),
    };

    let config = DhcpV4Config {
        name,
        route_metric,
        ignore_hostname,
        client_identifier,
        duid_raw,
    };
    config.validate()?;
    Ok(config)
}

/// Extract all `DHCPv4Config` docs from a loaded config, rejecting duplicate
/// link names.
pub fn dhcpv4_configs(config: &Config) -> Result<Vec<DhcpV4Config>> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for doc in config
        .documents()
        .iter()
        .filter(|doc| doc.meta.kind == DHCPV4_CONFIG_KIND)
    {
        let parsed = decode_dhcpv4_config_body(&doc.body)?;
        if !seen.insert(parsed.name.clone()) {
            return Err(Error::invalid(format!(
                "duplicate DHCPv4Config document for link '{}'",
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
            "DHCPv4Config.duidRaw must be an even-length hexadecimal hardware address",
        ));
    }
    let mut out = Vec::with_capacity(compact.len() / 2);
    for idx in (0..compact.len()).step_by(2) {
        let byte = u8::from_str_radix(&compact[idx..idx + 2], 16)
            .map_err(|_| Error::invalid("DHCPv4Config.duidRaw must be hexadecimal"))?;
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
    fn dhcpv4_config_decodes_named_doc_with_defaults() {
        let doc = "\
apiVersion: v1alpha1
kind: DHCPv4Config
name: eth0
";
        let parsed = decode_dhcpv4_config_body(doc).unwrap();
        assert_eq!(parsed.name, "eth0");
        assert_eq!(parsed.route_metric, 0);
        assert_eq!(parsed.route_metric_or(1024), 1024);
        assert!(!parsed.ignore_hostname);
        assert_eq!(parsed.client_identifier, DhcpV4ClientIdentifier::Mac);
        assert!(parsed.duid_raw.is_empty());
    }

    #[test]
    fn dhcpv4_config_decodes_all_operator_fields() {
        let doc = "\
apiVersion: v1alpha1
kind: DHCPv4Config
name: eth0
routeMetric: 2048
ignoreHostname: true
clientIdentifier: duid
duidRaw: 00:03:00:01:aa:bb:cc:dd:ee:ff
";
        let parsed = decode_dhcpv4_config_body(doc).unwrap();
        assert_eq!(parsed.route_metric_or(1024), 2048);
        assert!(parsed.ignore_hostname);
        assert_eq!(parsed.client_identifier, DhcpV4ClientIdentifier::Duid);
        assert_eq!(
            parsed.duid_raw,
            vec![0, 3, 0, 1, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]
        );
    }

    #[test]
    fn dhcpv4_config_rejects_empty_name() {
        let err =
            decode_dhcpv4_config_body("apiVersion: v1alpha1\nkind: DHCPv4Config\nname: \"\"\n")
                .unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    #[test]
    fn dhcpv4_config_validates_duid_shape() {
        assert!(
            decode_dhcpv4_config_body(
                "apiVersion: v1alpha1\nkind: DHCPv4Config\nname: eth0\nclientIdentifier: duid\n",
            )
            .is_err()
        );
        assert!(
            decode_dhcpv4_config_body(
                "apiVersion: v1alpha1\nkind: DHCPv4Config\nname: eth0\nclientIdentifier: mac\nduidRaw: 00:01\n",
            )
            .is_err()
        );
        assert!(
            decode_dhcpv4_config_body(
                "apiVersion: v1alpha1\nkind: DHCPv4Config\nname: eth0\nclientIdentifier: duid\nduidRaw: abc\n",
            )
            .is_err()
        );
        assert!(
            decode_dhcpv4_config_body(
                "apiVersion: v1alpha1\nkind: DHCPv4Config\nname: eth0\nclientIdentifier: duid\nduidRaw: 00:zz\n",
            )
            .is_err()
        );
    }

    #[test]
    fn dhcpv4_configs_extracts_multiple_named_documents() {
        let cfg = multidoc(
            "\
apiVersion: v1alpha1
kind: DHCPv4Config
name: eth0
---
apiVersion: v1alpha1
kind: DHCPv4Config
name: eth1
routeMetric: 2048
",
        );
        let container = load_from_bytes(&cfg).unwrap();
        let docs = dhcpv4_configs(&container).unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].name, "eth0");
        assert_eq!(docs[1].name, "eth1");
        assert_eq!(docs[1].route_metric, 2048);
    }

    #[test]
    fn dhcpv4_config_load_rejects_duplicate_names() {
        let cfg = multidoc(
            "\
apiVersion: v1alpha1
kind: DHCPv4Config
name: eth0
---
apiVersion: v1alpha1
kind: DHCPv4Config
name: eth0
",
        );
        let err = load_from_bytes(&cfg).unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }
}
