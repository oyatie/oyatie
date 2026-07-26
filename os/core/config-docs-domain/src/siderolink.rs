//! `SideroLinkConfig` — the `SideroLink` join configuration document.
//!
//! Mirrors `pkg/machinery/config/types/siderolink`. `SideroLink` establishes a
//! management wireguard tunnel back to a SideroLink/Omni endpoint. The single
//! required field is the API URL, which may carry a `jointoken` query
//! parameter used to authenticate the node on first contact.

use crate::document::{ConfigDocument, DocId, DocKind};
use os_kernel::error::{Error, Result};

/// The `SideroLinkConfig` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideroLinkConfig {
    /// The `SideroLink` API URL (e.g. `https://siderolink.example/?jointoken=…`).
    pub api_url: String,
    /// Optional unique token used to deduplicate the node on the server. When
    /// omitted, the server derives identity from the node's UUID.
    pub unique_token: Option<String>,
}

impl SideroLinkConfig {
    /// Construct from an API URL.
    pub fn new(api_url: impl Into<String>) -> Self {
        SideroLinkConfig {
            api_url: api_url.into(),
            unique_token: None,
        }
    }

    /// Builder: set the unique token.
    pub fn with_unique_token(mut self, token: impl Into<String>) -> Self {
        self.unique_token = Some(token.into());
        self
    }

    /// The scheme component of the API URL (`https`, `grpc`, …), if present.
    #[must_use]
    pub fn scheme(&self) -> Option<&str> {
        self.api_url.split_once("://").map(|(s, _)| s)
    }

    /// Extract the `jointoken` query parameter, if present.
    #[must_use]
    pub fn join_token(&self) -> Option<&str> {
        let query = self.api_url.split_once('?').map(|(_, q)| q)?;
        for pair in query.split('&') {
            if let Some(("jointoken", value)) = pair.split_once('=')
                && !value.is_empty() {
                    return Some(value);
                }
        }
        None
    }
}

impl ConfigDocument for SideroLinkConfig {
    fn kind(&self) -> DocKind {
        DocKind::SideroLink
    }

    fn id(&self) -> DocId {
        DocId::singleton(DocKind::SideroLink)
    }

    fn validate(&self) -> Result<()> {
        let url = self.api_url.trim();
        if url.is_empty() {
            return Err(Error::invalid("SideroLinkConfig: apiUrl is required"));
        }
        let scheme = self
            .scheme()
            .ok_or_else(|| Error::invalid("SideroLinkConfig: apiUrl must include a scheme"))?;
        match scheme {
            "https" | "http" | "grpc" => {}
            other => {
                return Err(Error::invalid(format!(
                    "SideroLinkConfig: unsupported apiUrl scheme '{other}'"
                )));
            }
        }
        // Host must be non-empty after the scheme.
        let rest = &url[scheme.len() + 3..];
        let host = rest.split(['/', '?']).next().unwrap_or("");
        if host.is_empty() {
            return Err(Error::invalid("SideroLinkConfig: apiUrl is missing a host"));
        }
        if let Some(tok) = &self.unique_token
            && tok.trim().is_empty() {
                return Err(Error::invalid(
                    "SideroLinkConfig: uniqueToken, if set, must be non-empty",
                ));
            }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_https_with_token() {
        let c = SideroLinkConfig::new("https://siderolink.example/?jointoken=abc123");
        assert!(c.validate().is_ok());
        assert_eq!(c.scheme(), Some("https"));
        assert_eq!(c.join_token(), Some("abc123"));
    }

    #[test]
    fn join_token_absent() {
        let c = SideroLinkConfig::new("https://siderolink.example/");
        assert_eq!(c.join_token(), None);
        assert!(c.validate().is_ok());
    }

    #[test]
    fn empty_url_rejected() {
        let c = SideroLinkConfig::new("   ");
        assert!(c.validate().is_err());
    }

    #[test]
    fn missing_scheme_rejected() {
        let c = SideroLinkConfig::new("siderolink.example/path");
        assert!(c.validate().is_err());
    }

    #[test]
    fn unsupported_scheme_rejected() {
        let c = SideroLinkConfig::new("ftp://host/");
        assert!(c.validate().is_err());
    }

    #[test]
    fn missing_host_rejected() {
        let c = SideroLinkConfig::new("https:///path");
        assert!(c.validate().is_err());
    }

    #[test]
    fn empty_unique_token_rejected() {
        let c = SideroLinkConfig::new("grpc://host:443").with_unique_token("  ");
        assert!(c.validate().is_err());
    }

    #[test]
    fn is_singleton() {
        let c = SideroLinkConfig::new("https://host/");
        assert!(!c.allows_multiple());
        assert_eq!(c.id(), DocId::singleton(DocKind::SideroLink));
    }
}
