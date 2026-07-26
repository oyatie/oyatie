//! `KmsgLogConfig` — kernel log (kmsg) delivery destinations.
//!
//! Mirrors `pkg/machinery/config/types/runtime`. Each document is keyed by
//! `name:` and declares a sink URL to which Talos ships kernel/service logs.
//! Supported schemes are `tcp://` and `udp://` (the syslog-ish stream sinks)
//! and `fluentd://`.

use crate::document::{ConfigDocument, DocId, DocKind};
use os_kernel::error::{Error, Result};

/// The `KmsgLogConfig` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KmsgLogConfig {
    /// Sink name (document key).
    pub name: String,
    /// Sink URL (`tcp://host:port`, `udp://host:port`, `fluentd://host:port`).
    pub url: String,
}

/// Supported kmsg sink schemes.
const SCHEMES: &[&str] = &["tcp", "udp", "fluentd"];

impl KmsgLogConfig {
    /// Construct a kmsg sink.
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        KmsgLogConfig {
            name: name.into(),
            url: url.into(),
        }
    }

    /// The URL scheme, if present.
    #[must_use]
    pub fn scheme(&self) -> Option<&str> {
        self.url.split_once("://").map(|(s, _)| s)
    }

    /// The `host:port` authority portion of the URL, if present.
    #[must_use]
    pub fn authority(&self) -> Option<&str> {
        let (_, rest) = self.url.split_once("://")?;
        Some(rest.split(['/', '?']).next().unwrap_or(rest))
    }
}

impl ConfigDocument for KmsgLogConfig {
    fn kind(&self) -> DocKind {
        DocKind::KmsgLog
    }

    fn id(&self) -> DocId {
        DocId::keyed(DocKind::KmsgLog, self.name.clone())
    }

    fn as_kmsg_log(&self) -> Option<&KmsgLogConfig> {
        Some(self)
    }

    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::invalid("KmsgLogConfig: name is required"));
        }
        let scheme = self
            .scheme()
            .ok_or_else(|| Error::invalid("KmsgLogConfig: url must include a scheme"))?;
        if !SCHEMES.contains(&scheme) {
            return Err(Error::invalid(format!(
                "KmsgLogConfig: unsupported url scheme '{scheme}' (expected tcp/udp/fluentd)"
            )));
        }
        let authority = self
            .authority()
            .filter(|a| !a.is_empty())
            .ok_or_else(|| Error::invalid("KmsgLogConfig: url is missing host:port"))?;
        // Require an explicit port for stream sinks.
        let (host, port) = authority.rsplit_once(':').ok_or_else(|| {
            Error::invalid(format!(
                "KmsgLogConfig: url authority '{authority}' must be host:port"
            ))
        })?;
        if host.is_empty() {
            return Err(Error::invalid("KmsgLogConfig: url host is empty"));
        }
        let port: u32 = port
            .parse()
            .map_err(|_| Error::invalid(format!("KmsgLogConfig: invalid port '{port}'")))?;
        if port == 0 || port > 65535 {
            return Err(Error::invalid(format!(
                "KmsgLogConfig: port {port} out of range"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_tcp_sink() {
        let c = KmsgLogConfig::new("remote", "tcp://10.0.0.1:514");
        assert!(c.validate().is_ok());
        assert_eq!(c.scheme(), Some("tcp"));
        assert_eq!(c.authority(), Some("10.0.0.1:514"));
        assert!(c.allows_multiple());
    }

    #[test]
    fn valid_udp_sink() {
        assert!(
            KmsgLogConfig::new("u", "udp://host:5514")
                .validate()
                .is_ok()
        );
    }

    #[test]
    fn empty_name_rejected() {
        assert!(KmsgLogConfig::new("", "tcp://h:1").validate().is_err());
    }

    #[test]
    fn missing_scheme_rejected() {
        assert!(KmsgLogConfig::new("n", "host:514").validate().is_err());
    }

    #[test]
    fn bad_scheme_rejected() {
        assert!(
            KmsgLogConfig::new("n", "https://host:443")
                .validate()
                .is_err()
        );
    }

    #[test]
    fn missing_port_rejected() {
        assert!(
            KmsgLogConfig::new("n", "tcp://hostonly")
                .validate()
                .is_err()
        );
    }

    #[test]
    fn port_out_of_range_rejected() {
        assert!(KmsgLogConfig::new("n", "tcp://host:0").validate().is_err());
        assert!(
            KmsgLogConfig::new("n", "tcp://host:99999")
                .validate()
                .is_err()
        );
    }
}
