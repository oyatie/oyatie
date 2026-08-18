//! SideroLink configuration, derived from kernel command-line arguments.
//!
//! Mirrors `internal/app/machined/pkg/controllers/siderolink`'s `ConfigController`
//! plus the `siderolink.Config` resource in `pkg/machinery/resources/siderolink`.
//!
//! Talos discovers the SideroLink management endpoint from the kernel argument
//! `siderolink.api=<url>`. The URL carries the API host:port, an optional join
//! token (`?jointoken=...`), and TLS / insecure / grpc-tunnel hints. This module
//! parses that argument into a validated [`Config`] resource exactly the way the
//! controller does, without performing any I/O.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use os_kernel::address::Port;
use os_kernel::error::{Error, Result};

/// The kernel command-line argument key carrying the SideroLink API endpoint.
pub const KERNEL_ARG_API: &str = "siderolink.api";

/// The default gRPC port a SideroLink server listens on when none is given.
pub const DEFAULT_API_PORT: u16 = 443;

/// The query-string parameter carrying the join token.
pub const PARAM_JOIN_TOKEN: &str = "jointoken";

/// The query-string parameter requesting that traffic be tunnelled over gRPC
/// instead of raw WireGuard UDP (used when only the TCP API port is reachable).
pub const PARAM_GRPC_TUNNEL: &str = "grpc_tunnel";

/// Transport security mode negotiated with the SideroLink API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiScheme {
    /// Plain HTTP/2 (`http://` or `grpc://`) — no TLS.
    Insecure,
    /// TLS-protected HTTP/2 (`https://` or `grpcs://`).
    Tls,
}

impl ApiScheme {
    /// Parse a URL scheme into a transport mode.
    pub fn parse(scheme: &str) -> Result<Self> {
        match scheme {
            "http" | "grpc" => Ok(ApiScheme::Insecure),
            "https" | "grpcs" => Ok(ApiScheme::Tls),
            other => Err(Error::parse(alloc::format!(
                "unsupported siderolink scheme '{other}'"
            ))),
        }
    }

    /// Whether the transport is TLS-protected.
    pub fn is_secure(self) -> bool {
        matches!(self, ApiScheme::Tls)
    }
}

impl fmt::Display for ApiScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiScheme::Insecure => f.write_str("http"),
            ApiScheme::Tls => f.write_str("https"),
        }
    }
}

/// The validated SideroLink configuration resource.
///
/// Equivalent to the `siderolink.Config` COSI resource Talos materialises from
/// the kernel command line. It is the single input the provision and manager
/// controllers consume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    scheme: ApiScheme,
    host: String,
    port: Port,
    join_token: Option<String>,
    grpc_tunnel: bool,
}

impl Config {
    /// Build a config from its parts, validating the host.
    pub fn new(
        scheme: ApiScheme,
        host: impl Into<String>,
        port: Port,
        join_token: Option<String>,
        grpc_tunnel: bool,
    ) -> Result<Self> {
        let host: String = host.into();
        validate_host(&host)?;
        if let Some(tok) = &join_token
            && tok.is_empty()
        {
            return Err(Error::invalid("siderolink join token is empty"));
        }
        Ok(Config {
            scheme,
            host,
            port,
            join_token,
            grpc_tunnel,
        })
    }

    /// Parse the value of a `siderolink.api=` kernel argument.
    ///
    /// Accepts forms like `https://siderolink.example.com:443?jointoken=abc`,
    /// `grpc://10.0.0.1` (default port), and the `grpc_tunnel=true` hint.
    pub fn parse_api_arg(value: &str) -> Result<Self> {
        let value = value.trim();
        if value.is_empty() {
            return Err(Error::parse("empty siderolink.api value"));
        }

        let (scheme_str, rest) = value
            .split_once("://")
            .ok_or_else(|| Error::parse("siderolink.api missing scheme://"))?;
        let scheme = ApiScheme::parse(scheme_str)?;

        // Split authority from the optional query string.
        let (authority, query) = match rest.split_once('?') {
            Some((a, q)) => (a, Some(q)),
            None => (rest, None),
        };
        // Strip any path component; SideroLink uses host:port authorities only.
        let authority = authority.split('/').next().unwrap_or(authority);
        if authority.is_empty() {
            return Err(Error::parse("siderolink.api missing host"));
        }

        let (host, port) = parse_authority(authority)?;

        let mut join_token = None;
        let mut grpc_tunnel = false;
        if let Some(q) = query {
            for pair in q.split('&').filter(|p| !p.is_empty()) {
                let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
                match k {
                    PARAM_JOIN_TOKEN => {
                        if v.is_empty() {
                            return Err(Error::parse("empty siderolink jointoken"));
                        }
                        join_token = Some(v.to_string());
                    }
                    PARAM_GRPC_TUNNEL => {
                        grpc_tunnel = matches!(v, "true" | "1" | "yes" | "");
                    }
                    _ => {} // Unknown params ignored, matching Talos's leniency.
                }
            }
        }

        Config::new(scheme, host, port, join_token, grpc_tunnel)
    }

    /// Scan a full kernel command line for `siderolink.api=` and parse it.
    ///
    /// Returns `Ok(None)` when no SideroLink argument is present (SideroLink is
    /// optional), and an error only when the argument exists but is malformed.
    pub fn from_kernel_cmdline(cmdline: &str) -> Result<Option<Self>> {
        for tok in cmdline.split_whitespace() {
            if let Some(val) = tok.strip_prefix(&alloc::format!("{KERNEL_ARG_API}=")) {
                return Config::parse_api_arg(val).map(Some);
            }
        }
        Ok(None)
    }

    /// The transport security mode.
    pub fn scheme(&self) -> ApiScheme {
        self.scheme
    }

    /// The API host (DNS name or IP literal).
    pub fn host(&self) -> &str {
        &self.host
    }

    /// The API port.
    pub fn port(&self) -> Port {
        self.port
    }

    /// The join token, if one was supplied.
    pub fn join_token(&self) -> Option<&str> {
        self.join_token.as_deref()
    }

    /// Whether WireGuard traffic should be tunnelled over the gRPC API.
    pub fn grpc_tunnel(&self) -> bool {
        self.grpc_tunnel
    }

    /// The reassembled `host:port` API endpoint authority.
    pub fn endpoint(&self) -> String {
        alloc::format!("{}:{}", self.host, self.port)
    }

    /// The canonical URL form of this configuration (token redacted).
    pub fn redacted_url(&self) -> String {
        let mut s = alloc::format!("{}://{}", self.scheme, self.endpoint());
        let mut params: Vec<String> = Vec::new();
        if self.join_token.is_some() {
            params.push(alloc::format!("{PARAM_JOIN_TOKEN}=<redacted>"));
        }
        if self.grpc_tunnel {
            params.push(alloc::format!("{PARAM_GRPC_TUNNEL}=true"));
        }
        if !params.is_empty() {
            s.push('?');
            s.push_str(&params.join("&"));
        }
        s
    }
}

/// Validate a host: a non-empty DNS name or IP literal with a safe charset.
fn validate_host(host: &str) -> Result<()> {
    if host.is_empty() {
        return Err(Error::invalid("siderolink host is empty"));
    }
    for c in host.chars() {
        let ok = c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':');
        if !ok {
            return Err(Error::invalid(alloc::format!(
                "invalid host character '{c}'"
            )));
        }
    }
    Ok(())
}

/// Split an authority into host and port, applying the default port and
/// handling bracketed IPv6 literals (`[fd00::1]:443`).
fn parse_authority(authority: &str) -> Result<(String, Port)> {
    if let Some(rest) = authority.strip_prefix('[') {
        // Bracketed IPv6 literal.
        let (host, after) = rest
            .split_once(']')
            .ok_or_else(|| Error::parse("unterminated IPv6 literal in siderolink.api"))?;
        let port = match after.strip_prefix(':') {
            Some(p) => Port::parse(p)?,
            None if after.is_empty() => Port::new(DEFAULT_API_PORT)?,
            None => return Err(Error::parse("unexpected characters after IPv6 literal")),
        };
        return Ok((host.to_string(), port));
    }

    // For unbracketed authorities, a colon only delimits the port if there is a
    // single one (more than one colon implies a bare IPv6 literal, disallowed
    // here — those must be bracketed).
    match authority.rsplit_once(':') {
        Some((host, port)) if !host.contains(':') => Ok((host.to_string(), Port::parse(port)?)),
        _ => Ok((authority.to_string(), Port::new(DEFAULT_API_PORT)?)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_url_with_token() {
        let cfg =
            Config::parse_api_arg("https://siderolink.example.com:8443?jointoken=secret").unwrap();
        assert_eq!(cfg.scheme(), ApiScheme::Tls);
        assert!(cfg.scheme().is_secure());
        assert_eq!(cfg.host(), "siderolink.example.com");
        assert_eq!(cfg.port().value(), 8443);
        assert_eq!(cfg.join_token(), Some("secret"));
        assert!(!cfg.grpc_tunnel());
        assert_eq!(cfg.endpoint(), "siderolink.example.com:8443");
    }

    #[test]
    fn applies_default_port() {
        let cfg = Config::parse_api_arg("grpc://10.0.0.1").unwrap();
        assert_eq!(cfg.scheme(), ApiScheme::Insecure);
        assert_eq!(cfg.port().value(), DEFAULT_API_PORT);
        assert_eq!(cfg.host(), "10.0.0.1");
        assert!(cfg.join_token().is_none());
    }

    #[test]
    fn parses_grpc_tunnel_hint() {
        let cfg =
            Config::parse_api_arg("https://omni.example.com?jointoken=x&grpc_tunnel=true").unwrap();
        assert!(cfg.grpc_tunnel());
        assert_eq!(cfg.join_token(), Some("x"));
    }

    #[test]
    fn parses_bracketed_ipv6_authority() {
        let cfg = Config::parse_api_arg("grpcs://[fd00::1]:6443").unwrap();
        assert_eq!(cfg.host(), "fd00::1");
        assert_eq!(cfg.port().value(), 6443);
        assert!(cfg.scheme().is_secure());

        let default = Config::parse_api_arg("grpc://[fd00::2]").unwrap();
        assert_eq!(default.port().value(), DEFAULT_API_PORT);
    }

    #[test]
    fn rejects_malformed_inputs() {
        assert!(Config::parse_api_arg("").is_err());
        assert!(Config::parse_api_arg("siderolink.example.com").is_err()); // no scheme
        assert!(Config::parse_api_arg("ftp://host").is_err()); // bad scheme
        assert!(Config::parse_api_arg("https://host?jointoken=").is_err()); // empty token
        assert!(Config::parse_api_arg("https://host:0").is_err()); // bad port
        assert!(Config::parse_api_arg("https://host:notaport").is_err());
    }

    #[test]
    fn scans_kernel_cmdline() {
        let cmdline =
            "console=ttyS0 siderolink.api=https://omni.example.com:443?jointoken=abc quiet";
        let cfg = Config::from_kernel_cmdline(cmdline).unwrap().unwrap();
        assert_eq!(cfg.host(), "omni.example.com");
        assert_eq!(cfg.join_token(), Some("abc"));

        // Absent argument yields None, not an error.
        assert!(
            Config::from_kernel_cmdline("console=ttyS0 quiet")
                .unwrap()
                .is_none()
        );

        // Present but malformed yields an error.
        assert!(Config::from_kernel_cmdline("siderolink.api=bogus").is_err());
    }

    #[test]
    fn redacts_token_in_url() {
        let cfg =
            Config::parse_api_arg("https://host:443?jointoken=topsecret&grpc_tunnel=true").unwrap();
        let url = cfg.redacted_url();
        assert!(url.contains("<redacted>"));
        assert!(!url.contains("topsecret"));
        assert!(url.contains("grpc_tunnel=true"));
    }

    #[test]
    fn strips_path_component() {
        let cfg = Config::parse_api_arg("https://host:443/some/path?jointoken=t").unwrap();
        assert_eq!(cfg.host(), "host");
        assert_eq!(cfg.port().value(), 443);
    }
}
