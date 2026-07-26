//! `EventSinkConfig` — the machine event sink endpoint.
//!
//! Mirrors `pkg/machinery/config/types/runtime`. Talos streams machine events
//! (sequence transitions, service state changes, etc.) to a single gRPC sink.
//! This is a singleton document carrying the sink `endpoint` (`host:port`).

use crate::document::{ConfigDocument, DocId, DocKind};
use os_kernel::error::{Error, Result};

/// The `EventSinkConfig` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventSinkConfig {
    /// The gRPC sink endpoint in `host:port` form.
    pub endpoint: String,
}

impl EventSinkConfig {
    /// Construct from an endpoint.
    pub fn new(endpoint: impl Into<String>) -> Self {
        EventSinkConfig {
            endpoint: endpoint.into(),
        }
    }

    /// The host portion of the endpoint, if parseable.
    #[must_use]
    pub fn host(&self) -> Option<&str> {
        split_host_port(&self.endpoint).map(|(h, _)| h)
    }

    /// The port portion of the endpoint, if parseable.
    #[must_use]
    pub fn port(&self) -> Option<u16> {
        split_host_port(&self.endpoint).and_then(|(_, p)| p.parse().ok())
    }
}

/// Split a `host:port` endpoint, supporting bracketed IPv6 literals.
fn split_host_port(s: &str) -> Option<(&str, &str)> {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix('[') {
        // [::1]:8080
        let (host, after) = rest.split_once(']')?;
        let port = after.strip_prefix(':')?;
        Some((host, port))
    } else {
        let (host, port) = s.rsplit_once(':')?;
        if host.contains(':') {
            // bare IPv6 without brackets — ambiguous, reject
            return None;
        }
        Some((host, port))
    }
}

impl ConfigDocument for EventSinkConfig {
    fn kind(&self) -> DocKind {
        DocKind::EventSink
    }

    fn id(&self) -> DocId {
        DocId::singleton(DocKind::EventSink)
    }

    fn validate(&self) -> Result<()> {
        if self.endpoint.trim().is_empty() {
            return Err(Error::invalid("EventSinkConfig: endpoint is required"));
        }
        let (host, port) = split_host_port(&self.endpoint).ok_or_else(|| {
            Error::invalid(format!(
                "EventSinkConfig: endpoint '{}' must be host:port",
                self.endpoint
            ))
        })?;
        if host.is_empty() {
            return Err(Error::invalid("EventSinkConfig: endpoint host is empty"));
        }
        let port: u32 = port
            .parse()
            .map_err(|_| Error::invalid(format!("EventSinkConfig: invalid port '{port}'")))?;
        if port == 0 || port > 65535 {
            return Err(Error::invalid(format!(
                "EventSinkConfig: port {port} out of range"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_ipv4_endpoint() {
        let c = EventSinkConfig::new("10.0.0.1:4242");
        assert!(c.validate().is_ok());
        assert_eq!(c.host(), Some("10.0.0.1"));
        assert_eq!(c.port(), Some(4242));
        assert!(!c.allows_multiple());
    }

    #[test]
    fn valid_ipv6_endpoint() {
        let c = EventSinkConfig::new("[fd00::1]:514");
        assert!(c.validate().is_ok());
        assert_eq!(c.host(), Some("fd00::1"));
        assert_eq!(c.port(), Some(514));
    }

    #[test]
    fn empty_rejected() {
        assert!(EventSinkConfig::new("  ").validate().is_err());
    }

    #[test]
    fn missing_port_rejected() {
        assert!(EventSinkConfig::new("host-only").validate().is_err());
    }

    #[test]
    fn port_out_of_range_rejected() {
        assert!(EventSinkConfig::new("host:70000").validate().is_err());
        assert!(EventSinkConfig::new("host:0").validate().is_err());
    }

    #[test]
    fn bare_ipv6_rejected() {
        assert!(EventSinkConfig::new("fd00::1:514").validate().is_err());
    }
}
