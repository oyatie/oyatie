//! `ResolverConfig` — host DNS resolver configuration.
//!
//! Source-guided by Talos `pkg/machinery/config/types/network/resolver.go`:
//! `nameservers[].{address,protocol,tlsServerName}`,
//! `searchDomains.{domains,disableDefault}`, and
//! `hostDNS.{enabled,forwardKubeDNSToHost,resolveMemberNames}`.

use crate::document::{ConfigDocument, DocId, DocKind};
use os_kernel::error::{Error, Result};
use std::net::IpAddr;

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
    /// Canonical Talos config string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Do53 => "Do53",
            Self::DoT => "DoT",
            Self::DoH => "DoH",
        }
    }

    /// Parse a Talos config string.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.trim() {
            "" | "Do53" => Self::Do53,
            "DoT" => Self::DoT,
            "DoH" => Self::DoH,
            _ => return None,
        })
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

    /// Build an encrypted nameserver.
    pub fn encrypted(
        address: impl Into<String>,
        protocol: DnsProtocol,
        tls_server_name: impl Into<String>,
    ) -> Self {
        NameserverConfig {
            address: address.into(),
            protocol,
            tls_server_name: tls_server_name.into(),
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
    fn is_zero(&self) -> bool {
        self.enabled.is_none()
            && self.forward_kube_dns_to_host.is_none()
            && self.resolve_member_names.is_none()
    }

    /// Effective `enabled` value with Talos pointer-default semantics.
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or(false)
    }

    /// Effective `forwardKubeDNSToHost` value.
    #[must_use]
    pub fn forward_kube_dns_to_host(&self) -> bool {
        self.forward_kube_dns_to_host.unwrap_or(false)
    }

    /// Effective `resolveMemberNames` value.
    #[must_use]
    pub fn resolve_member_names(&self) -> bool {
        self.resolve_member_names.unwrap_or(false)
    }
}

/// The `ResolverConfig` document.
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
    #[must_use]
    pub fn warnings(&self) -> Vec<String> {
        if !self.nameservers.is_empty()
            && self.nameservers.iter().all(|ns| ns.protocol.is_encrypted())
        {
            vec!["all configured nameservers use encrypted DNS (DoT or DoH): validating certificates requires a correct system clock, so boot may stall when NTP servers are configured by hostname; consider keeping at least one plain-DNS fallback or configuring NTP servers by IP address".to_string()]
        } else {
            Vec::new()
        }
    }
}

impl ConfigDocument for ResolverConfig {
    fn kind(&self) -> DocKind {
        DocKind::Resolver
    }

    fn id(&self) -> DocId {
        DocId::singleton(DocKind::Resolver)
    }

    fn validate(&self) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolver_config_validates_minimal_doc() {
        let doc = ResolverConfig::new();
        assert!(doc.validate().is_ok());
        assert_eq!(doc.kind(), DocKind::Resolver);
        assert_eq!(doc.id(), DocId::singleton(DocKind::Resolver));
        assert!(!doc.allows_multiple());
    }

    #[test]
    fn dns_protocol_maps_talos_values() {
        assert_eq!(DnsProtocol::parse(""), Some(DnsProtocol::Do53));
        assert_eq!(DnsProtocol::parse("Do53"), Some(DnsProtocol::Do53));
        assert_eq!(DnsProtocol::parse("DoT"), Some(DnsProtocol::DoT));
        assert_eq!(DnsProtocol::parse("DoH"), Some(DnsProtocol::DoH));
        assert_eq!(DnsProtocol::parse("dns-over-quic"), None);
    }

    #[test]
    fn resolver_config_validates_tls_rules() {
        let mut doc = ResolverConfig::new();
        doc.nameservers = vec![NameserverConfig::encrypted(
            "9.9.9.9",
            DnsProtocol::DoT,
            "dns.quad9.net",
        )];
        assert!(doc.validate().is_ok());
        assert_eq!(doc.warnings().len(), 1);

        doc.nameservers[0].tls_server_name.clear();
        let err = doc.validate().unwrap_err();
        assert!(err.to_string().contains("tlsServerName must be set"));
    }

    #[test]
    fn resolver_config_rejects_hostdns_dependents_when_disabled() {
        let doc = ResolverConfig {
            host_dns: HostDnsConfig {
                enabled: Some(false),
                forward_kube_dns_to_host: Some(true),
                resolve_member_names: None,
            },
            ..ResolverConfig::new()
        };
        assert!(doc.validate().is_err());
    }
}
