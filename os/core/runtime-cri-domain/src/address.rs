//! containerd connection addresses and dialer modeling.
//!
//! In the real system, machined and the CRI plugin dial containerd over a Unix
//! domain socket (`/run/containerd/containerd.sock` for the CRI/k8s namespace,
//! and a dedicated `/run/system/containerd.sock` for the Talos system services).
//! The gRPC client is configured with that address plus a default namespace.
//!
//! This module models the address parsing/validation and a dial registry so the
//! ordering invariant — "you must be able to reach the daemon at an address
//! before you can talk to it" — is testable offline.

use os_kernel::error::{Error, Result};
use std::collections::HashMap;

/// The well-known containerd socket Talos uses for the CRI (`k8s.io`) namespace.
pub const CRI_CONTAINERD_ADDRESS: &str = "/run/containerd/containerd.sock";
/// The dedicated containerd socket Talos runs for `system` services.
pub const SYSTEM_CONTAINERD_ADDRESS: &str = "/run/system/containerd.sock";

/// The transport scheme of a containerd address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    /// A Unix domain socket (`unix://` or a bare absolute path).
    Unix,
    /// A TCP endpoint (`tcp://host:port`).
    Tcp,
}

/// A parsed containerd dial address.
///
/// Mirrors containerd's `dialer` address handling: an explicit `unix://` /
/// `tcp://` scheme, or a bare absolute path treated as a Unix socket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerdAddress {
    /// The transport scheme.
    pub scheme: Scheme,
    /// For Unix: the socket path. For TCP: the `host:port` authority.
    pub endpoint: String,
}

impl ContainerdAddress {
    /// Parse a containerd address string.
    ///
    /// * `unix:///run/containerd/containerd.sock` -> Unix socket.
    /// * `/run/containerd/containerd.sock` (bare absolute path) -> Unix socket.
    /// * `tcp://127.0.0.1:1234` -> TCP endpoint.
    pub fn parse(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Err(Error::parse("empty containerd address"));
        }
        if let Some(rest) = s.strip_prefix("unix://") {
            if !rest.starts_with('/') {
                return Err(Error::parse("unix socket path must be absolute"));
            }
            return Ok(ContainerdAddress {
                scheme: Scheme::Unix,
                endpoint: rest.to_string(),
            });
        }
        if let Some(rest) = s.strip_prefix("tcp://") {
            // Require a host:port authority.
            match rest.rsplit_once(':') {
                Some((host, port)) if !host.is_empty() && port.parse::<u16>().is_ok() => {
                    Ok(ContainerdAddress {
                        scheme: Scheme::Tcp,
                        endpoint: rest.to_string(),
                    })
                }
                _ => Err(Error::parse("tcp address must be host:port")),
            }
        } else if s.starts_with('/') {
            Ok(ContainerdAddress {
                scheme: Scheme::Unix,
                endpoint: s.to_string(),
            })
        } else {
            Err(Error::parse("unrecognized containerd address scheme"))
        }
    }

    /// The default CRI (`k8s.io`) containerd socket.
    pub fn cri() -> Self {
        ContainerdAddress {
            scheme: Scheme::Unix,
            endpoint: CRI_CONTAINERD_ADDRESS.to_string(),
        }
    }

    /// The dedicated `system` services containerd socket.
    pub fn system() -> Self {
        ContainerdAddress {
            scheme: Scheme::Unix,
            endpoint: SYSTEM_CONTAINERD_ADDRESS.to_string(),
        }
    }

    /// Whether this address is a Unix domain socket.
    pub fn is_unix(&self) -> bool {
        self.scheme == Scheme::Unix
    }

    /// The fully-qualified dial string (with scheme prefix restored).
    pub fn dial_string(&self) -> String {
        match self.scheme {
            Scheme::Unix => format!("unix://{}", self.endpoint),
            Scheme::Tcp => format!("tcp://{}", self.endpoint),
        }
    }
}

/// A modeled registry of containerd endpoints that can be "dialed".
///
/// Stands in for the OS socket layer: an address must be registered (the daemon
/// is listening) before a dial succeeds, mirroring `ENOENT`/connection-refused
/// errors that the real client surfaces when containerd is not up yet.
#[derive(Debug, Default)]
pub struct DialRegistry {
    listening: HashMap<String, bool>,
}

impl DialRegistry {
    /// An empty registry with nothing listening.
    pub fn new() -> Self {
        DialRegistry {
            listening: HashMap::new(),
        }
    }

    /// Mark an address as having a listening daemon.
    pub fn listen(&mut self, addr: &ContainerdAddress) {
        self.listening.insert(addr.dial_string(), true);
    }

    /// Stop listening on an address (daemon went down).
    pub fn shutdown(&mut self, addr: &ContainerdAddress) {
        self.listening.remove(&addr.dial_string());
    }

    /// Attempt to dial; succeeds only if something is listening.
    pub fn dial(&self, addr: &ContainerdAddress) -> Result<Connection> {
        if self
            .listening
            .get(&addr.dial_string())
            .copied()
            .unwrap_or(false)
        {
            Ok(Connection {
                address: addr.clone(),
            })
        } else {
            Err(Error::not_found(
                "containerd not listening at address (connection refused)",
            ))
        }
    }
}

/// A successful (modeled) connection to a containerd endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    /// The address that was dialed.
    pub address: ContainerdAddress,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bare_unix_path() {
        let a = ContainerdAddress::parse("/run/containerd/containerd.sock").unwrap();
        assert_eq!(a.scheme, Scheme::Unix);
        assert!(a.is_unix());
        assert_eq!(a.endpoint, "/run/containerd/containerd.sock");
        assert_eq!(a.dial_string(), "unix:///run/containerd/containerd.sock");
    }

    #[test]
    fn parse_unix_scheme() {
        let a = ContainerdAddress::parse("unix:///run/system/containerd.sock").unwrap();
        assert_eq!(a.scheme, Scheme::Unix);
        assert_eq!(a.endpoint, "/run/system/containerd.sock");
    }

    #[test]
    fn parse_tcp() {
        let a = ContainerdAddress::parse("tcp://127.0.0.1:3456").unwrap();
        assert_eq!(a.scheme, Scheme::Tcp);
        assert!(!a.is_unix());
        assert_eq!(a.dial_string(), "tcp://127.0.0.1:3456");
    }

    #[test]
    fn parse_rejects_bad() {
        assert!(ContainerdAddress::parse("").is_err());
        assert!(ContainerdAddress::parse("unix://relative").is_err());
        assert!(ContainerdAddress::parse("tcp://noport").is_err());
        assert!(ContainerdAddress::parse("tcp://host:notaport").is_err());
        assert!(ContainerdAddress::parse("http://x").is_err());
    }

    #[test]
    fn well_known_addresses() {
        assert_eq!(ContainerdAddress::cri().endpoint, CRI_CONTAINERD_ADDRESS);
        assert_eq!(
            ContainerdAddress::system().endpoint,
            SYSTEM_CONTAINERD_ADDRESS
        );
        assert_ne!(ContainerdAddress::cri(), ContainerdAddress::system());
    }

    #[test]
    fn dial_requires_listener() {
        let mut reg = DialRegistry::new();
        let addr = ContainerdAddress::system();
        assert_eq!(reg.dial(&addr).unwrap_err().kind(), "not_found");
        reg.listen(&addr);
        let conn = reg.dial(&addr).unwrap();
        assert_eq!(conn.address, addr);
        reg.shutdown(&addr);
        assert!(reg.dial(&addr).is_err());
    }

    #[test]
    fn dial_endpoints_are_isolated() {
        let mut reg = DialRegistry::new();
        reg.listen(&ContainerdAddress::cri());
        assert!(reg.dial(&ContainerdAddress::cri()).is_ok());
        assert!(reg.dial(&ContainerdAddress::system()).is_err());
    }
}
