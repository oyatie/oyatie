//! Typed parser/accessor for the `ResolverConfig` multi-document config.
//!
//! This is a bounded source-guided path from upstream Talos
//! `pkg/machinery/config/types/network/resolver.go`: it parses and validates
//! v1alpha1 `nameservers`, `searchDomains`, and `hostDNS` fields while the
//! loader continues to preserve the original document body (including any
//! unsupported fields) in [`crate::container::AuxDocument`].

use crate::container::Config;
use crate::yaml::{self, Yaml};
use os_kernel::error::{Error, Result};
use std::net::IpAddr;

/// Canonical Talos document kind.
pub const RESOLVER_CONFIG_KIND: &str = "ResolverConfig";

/// DNS protocol used for a resolver nameserver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DnsProtocol {
    /// Plain DNS over UDP/TCP port 53 (Talos `Do53` / default).
    #[default]
    Do53,
    /// DNS over TLS (Talos `DoT`).
    DoT,
    /// DNS over HTTPS (Talos `DoH`).
    DoH,
}

impl DnsProtocol {
    /// Parse the YAML string value.
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim() {
            "" | "Do53" => Ok(Self::Do53),
            "DoT" => Ok(Self::DoT),
            "DoH" => Ok(Self::DoH),
            other => Err(Error::parse(format!(
                "ResolverConfig: unknown DNS protocol '{other}'"
            ))),
        }
    }

    /// Canonical Talos config string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Do53 => "Do53",
            Self::DoT => "DoT",
            Self::DoH => "DoH",
        }
    }

    fn is_encrypted(self) -> bool {
        matches!(self, Self::DoT | Self::DoH)
    }
}

/// One `nameservers[]` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameserverConfig {
    /// Nameserver IP address.
    pub address: String,
    /// DNS protocol. Defaults to [`DnsProtocol::Do53`].
    pub protocol: DnsProtocol,
    /// TLS server name / SNI for DoT and DoH.
    pub tls_server_name: String,
}

impl NameserverConfig {
    /// Build a plain-DNS nameserver.
    pub fn new(address: impl Into<String>) -> Self {
        NameserverConfig {
            address: address.into(),
            protocol: DnsProtocol::Do53,
            tls_server_name: String::new(),
        }
    }
}

/// `searchDomains` resolver settings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchDomainsConfig {
    /// Search domain list.
    pub domains: Vec<String>,
    /// Disable deriving default search domains from the hostname FQDN.
    pub disable_default: Option<bool>,
}

/// `hostDNS` resolver settings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostDnsConfig {
    /// Enable the local host DNS caching resolver.
    pub enabled: Option<bool>,
    /// Route Kubernetes CoreDNS upstreams through host DNS.
    pub forward_kube_dns_to_host: Option<bool>,
    /// Resolve member/node names through host DNS.
    pub resolve_member_names: Option<bool>,
}

impl HostDnsConfig {
    /// True when no `hostDNS` sub-field was configured.
    pub fn is_zero(&self) -> bool {
        self.enabled.is_none()
            && self.forward_kube_dns_to_host.is_none()
            && self.resolve_member_names.is_none()
    }

    /// Effective `enabled` value with Talos pointer-default semantics.
    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or(false)
    }

    /// Effective `forwardKubeDNSToHost` value.
    pub fn forward_kube_dns_to_host(&self) -> bool {
        self.forward_kube_dns_to_host.unwrap_or(false)
    }

    /// Effective `resolveMemberNames` value.
    pub fn resolve_member_names(&self) -> bool {
        self.resolve_member_names.unwrap_or(false)
    }
}

/// Parsed `ResolverConfig` document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolverConfig {
    /// Explicit nameserver configuration.
    pub nameservers: Vec<NameserverConfig>,
    /// Search-domain configuration.
    pub search_domains: SearchDomainsConfig,
    /// Host DNS resolver configuration.
    pub host_dns: HostDnsConfig,
}

impl ResolverConfig {
    /// Build an empty resolver config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Source-compatible validation warning for all-encrypted nameserver sets.
    pub fn warnings(&self) -> Vec<String> {
        if !self.nameservers.is_empty()
            && self.nameservers.iter().all(|ns| ns.protocol.is_encrypted())
        {
            vec!["all configured nameservers use encrypted DNS (DoT or DoH): validating certificates requires a correct system clock, so boot may stall when NTP servers are configured by hostname; consider keeping at least one plain-DNS fallback or configuring NTP servers by IP address".to_string()]
        } else {
            Vec::new()
        }
    }

    /// Validate that the parsed document can be projected into the current
    /// Rust resolver-spec model without silently dropping source semantics.
    pub fn validate_projection_supported(&self) -> Result<()> {
        if self.search_domains.disable_default == Some(true) {
            return Err(Error::invalid(
                "ResolverConfig searchDomains.disableDefault is parsed but not yet projectable to ResolverSpec",
            ));
        }
        if !self.host_dns.is_zero() {
            return Err(Error::invalid(
                "ResolverConfig hostDNS is parsed but not yet projectable to ResolverSpec",
            ));
        }
        Ok(())
    }

    /// Validate the resolver document in isolation.
    pub fn validate(&self) -> Result<()> {
        if !self.host_dns.is_zero() && !self.host_dns.enabled() {
            if self.host_dns.forward_kube_dns_to_host() {
                return Err(Error::invalid(
                    "hostDNS.forwardKubeDNSToHost cannot be enabled when hostDNS.enabled is false",
                ));
            }
            if self.host_dns.resolve_member_names() {
                return Err(Error::invalid(
                    "hostDNS.resolveMemberNames cannot be enabled when hostDNS.enabled is false",
                ));
            }
        }

        let mut errs = Vec::new();
        for (idx, ns) in self.nameservers.iter().enumerate() {
            match ns.protocol {
                DnsProtocol::DoT if ns.tls_server_name.is_empty() => errs.push(format!(
                    "tlsServerName must be set when protocol is DoT: entry {idx}"
                )),
                DnsProtocol::DoH if ns.tls_server_name.is_empty() => errs.push(format!(
                    "tlsServerName must be set when protocol is DoH: entry {idx}"
                )),
                DnsProtocol::Do53 if !ns.tls_server_name.is_empty() => errs.push(format!(
                    "tlsServerName must be empty when protocol is Do53: entry {idx}"
                )),
                _ => {}
            }
            if ns.address.parse::<IpAddr>().is_err() {
                errs.push(format!(
                    "nameserver address must be a valid IP: entry {idx}"
                ));
            }
        }

        if errs.is_empty() {
            Ok(())
        } else {
            Err(Error::invalid(errs.join("\n")))
        }
    }
}

/// Decode and validate one `ResolverConfig` document body.
pub fn decode_resolver_config_body(body: &str) -> Result<ResolverConfig> {
    let root = yaml::parse(body).map_err(|e| Error::parse(e.to_string()))?;
    if let Some(kind) = root.get_str("kind")
        && kind != RESOLVER_CONFIG_KIND
    {
        return Err(Error::invalid(format!(
            "ResolverConfig: unexpected kind '{kind}'"
        )));
    }

    let nameservers = match root.get("nameservers") {
        Some(value) => decode_nameservers(value)?,
        None => Vec::new(),
    };
    let search_domains = match root.get("searchDomains") {
        Some(value) => decode_search_domains(value)?,
        None => SearchDomainsConfig::default(),
    };
    let host_dns = match root.get("hostDNS") {
        Some(value) => decode_host_dns(value)?,
        None => HostDnsConfig::default(),
    };

    let config = ResolverConfig {
        nameservers,
        search_domains,
        host_dns,
    };
    config.validate()?;
    Ok(config)
}

/// Extract the singleton `ResolverConfig` from a loaded config if present.
pub fn resolver_config(config: &Config) -> Result<Option<ResolverConfig>> {
    config
        .document(RESOLVER_CONFIG_KIND)
        .map(|doc| decode_resolver_config_body(&doc.body))
        .transpose()
}

fn decode_nameservers(value: &Yaml) -> Result<Vec<NameserverConfig>> {
    let Some(items) = value.as_sequence() else {
        return Err(Error::parse(
            "ResolverConfig.nameservers must be a sequence",
        ));
    };
    let mut out = Vec::new();
    for (idx, item) in items.iter().enumerate() {
        if item.as_mapping().is_none() {
            return Err(Error::parse(format!(
                "ResolverConfig.nameservers[{idx}] must be a mapping"
            )));
        }
        let address = item.get_str("address").unwrap_or("").trim().to_string();
        let protocol = match item.get_str("protocol") {
            Some(value) => DnsProtocol::parse(value)?,
            None => DnsProtocol::Do53,
        };
        let tls_server_name = item.get_str("tlsServerName").unwrap_or("").to_string();
        out.push(NameserverConfig {
            address,
            protocol,
            tls_server_name,
        });
    }
    Ok(out)
}

fn decode_search_domains(value: &Yaml) -> Result<SearchDomainsConfig> {
    if value.as_mapping().is_none() {
        return Err(Error::parse(
            "ResolverConfig.searchDomains must be a mapping",
        ));
    }
    let domains = match value.get("domains") {
        Some(domains) => string_sequence(domains, "ResolverConfig.searchDomains.domains")?,
        None => Vec::new(),
    };
    let disable_default = optional_bool(
        value.get("disableDefault"),
        "ResolverConfig.searchDomains.disableDefault",
    )?;
    Ok(SearchDomainsConfig {
        domains,
        disable_default,
    })
}

fn decode_host_dns(value: &Yaml) -> Result<HostDnsConfig> {
    if value.as_mapping().is_none() {
        return Err(Error::parse("ResolverConfig.hostDNS must be a mapping"));
    }
    Ok(HostDnsConfig {
        enabled: optional_bool(value.get("enabled"), "ResolverConfig.hostDNS.enabled")?,
        forward_kube_dns_to_host: optional_bool(
            value.get("forwardKubeDNSToHost"),
            "ResolverConfig.hostDNS.forwardKubeDNSToHost",
        )?,
        resolve_member_names: optional_bool(
            value.get("resolveMemberNames"),
            "ResolverConfig.hostDNS.resolveMemberNames",
        )?,
    })
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

fn string_sequence(value: &Yaml, field: &str) -> Result<Vec<String>> {
    let Some(items) = value.as_sequence() else {
        return Err(Error::parse(format!("{field} must be a sequence")));
    };
    let mut out = Vec::new();
    for (idx, item) in items.iter().enumerate() {
        let Some(raw) = item.as_str() else {
            return Err(Error::parse(format!("{field}[{idx}] must be a string")));
        };
        out.push(raw.to_string());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load::{load_from_bytes, save_to_bytes};

    const BASE: &str = "\
version: v1alpha1
machine:
  type: worker
";

    fn multidoc(doc: &str) -> String {
        format!("{BASE}---\n{doc}")
    }

    #[test]
    fn resolver_config_decodes_upstream_fields() {
        let doc = "\
apiVersion: v1alpha1
kind: ResolverConfig
nameservers:
  - address: 10.0.0.1
  - address: 2001:4860:4860::8888
    protocol: DoT
    tlsServerName: dns.google
searchDomains:
  domains:
    - example.org
    - example.com
  disableDefault: false
hostDNS:
  enabled: true
  forwardKubeDNSToHost: true
  resolveMemberNames: false
";
        let parsed = decode_resolver_config_body(doc).unwrap();
        assert_eq!(parsed.nameservers.len(), 2);
        assert_eq!(parsed.nameservers[0], NameserverConfig::new("10.0.0.1"));
        assert_eq!(parsed.nameservers[1].protocol, DnsProtocol::DoT);
        assert_eq!(parsed.nameservers[1].tls_server_name, "dns.google");
        assert_eq!(
            parsed.search_domains.domains,
            vec!["example.org", "example.com"]
        );
        assert_eq!(parsed.search_domains.disable_default, Some(false));
        assert_eq!(parsed.host_dns.enabled, Some(true));
        assert_eq!(parsed.host_dns.forward_kube_dns_to_host, Some(true));
        assert_eq!(parsed.host_dns.resolve_member_names, Some(false));
    }

    #[test]
    fn resolver_config_loads_as_singleton_and_preserves_raw_body() {
        let cfg = multidoc(
            "\
apiVersion: v1alpha1
kind: ResolverConfig
nameservers:
  - address: 1.1.1.1
unsupportedFutureField:
  nested: preserved
",
        );
        let container = load_from_bytes(&cfg).unwrap();
        let parsed = resolver_config(&container).unwrap().unwrap();
        assert_eq!(parsed.nameservers, vec![NameserverConfig::new("1.1.1.1")]);
        let encoded = save_to_bytes(&container);
        assert!(encoded.contains("unsupportedFutureField:"));
        assert!(encoded.contains("nested: preserved"));
    }

    #[test]
    fn resolver_config_rejects_invalid_tls_and_address_rules() {
        let err = decode_resolver_config_body(
            "\
apiVersion: v1alpha1
kind: ResolverConfig
nameservers:
  - address: not-an-ip
    tlsServerName: dns.example
",
        )
        .unwrap_err();
        assert_eq!(err.kind(), "invalid");
        assert!(err.to_string().contains("tlsServerName must be empty"));
        assert!(
            err.to_string()
                .contains("nameserver address must be a valid IP")
        );

        let err = decode_resolver_config_body(
            "\
apiVersion: v1alpha1
kind: ResolverConfig
nameservers:
  - address: 9.9.9.9
    protocol: DoT
",
        )
        .unwrap_err();
        assert!(err.to_string().contains("tlsServerName must be set"));
    }

    #[test]
    fn resolver_config_rejects_invalid_hostdns_dependencies() {
        let err = decode_resolver_config_body(
            "\
apiVersion: v1alpha1
kind: ResolverConfig
hostDNS:
  enabled: false
  forwardKubeDNSToHost: true
",
        )
        .unwrap_err();
        assert_eq!(err.kind(), "invalid");
        assert!(
            err.to_string()
                .contains("forwardKubeDNSToHost cannot be enabled")
        );
    }

    #[test]
    fn resolver_config_warns_when_all_nameservers_are_encrypted() {
        let parsed = decode_resolver_config_body(
            "\
apiVersion: v1alpha1
kind: ResolverConfig
nameservers:
  - address: 9.9.9.9
    protocol: DoT
    tlsServerName: dns.quad9.net
  - address: 1.1.1.1
    protocol: DoH
    tlsServerName: cloudflare-dns.com
",
        )
        .unwrap();
        assert_eq!(parsed.warnings().len(), 1);
    }

    #[test]
    fn duplicate_resolver_config_rejected_on_load() {
        let cfg = multidoc(
            "\
apiVersion: v1alpha1
kind: ResolverConfig
---
apiVersion: v1alpha1
kind: ResolverConfig
",
        );
        let err = load_from_bytes(&cfg).unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }
}
