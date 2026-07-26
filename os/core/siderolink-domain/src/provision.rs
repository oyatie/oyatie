//! The SideroLink provision handshake.
//!
//! Mirrors the `provision.proto` gRPC service (`pkg/machinery/api/siderolink`)
//! and Talos's client side in
//! `internal/app/machined/pkg/controllers/siderolink`. A node sends a
//! [`ProvisionRequest`] carrying its UUID, WireGuard public key and Talos
//! version; the management plane (Sidero / Omni) replies with a
//! [`ProvisionResponse`] giving the server's public key, its WireGuard
//! endpoint, and the unique IPv6 ULA address the node must use on the link.
//!
//! The gRPC transport is modelled as the [`ProvisionService`] trait; an
//! in-memory [`InMemoryProvisionServer`] implements it for tests and allocates
//! deterministic per-node addresses from a configured `/64`.

use crate::config::Config;
use crate::tunnel::WireguardPublicKey;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use os_kernel::address::{Cidr, NodeAddress};
use os_kernel::error::{Error, Result};

/// The WireGuard listen port the server hands back when none is otherwise set.
pub const DEFAULT_WG_PORT: u16 = 51820;

/// The provision request a node sends to the SideroLink API.
///
/// Equivalent to `siderolink.ProvisionRequest`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvisionRequest {
    /// The node's hardware UUID (`/sys/class/dmi/id/product_uuid`).
    pub node_uuid: String,
    /// The node's WireGuard public key.
    pub node_public_key: WireguardPublicKey,
    /// The Talos version string the node is running (e.g. `v1.7.0`).
    pub talos_version: String,
    /// The join token presented for authorization, if the config carried one.
    pub join_token: Option<String>,
    /// Whether the node requests gRPC tunnelling rather than raw WireGuard.
    pub grpc_tunnel: bool,
}

impl ProvisionRequest {
    /// Build a request, validating the UUID and version are non-empty.
    pub fn new(
        node_uuid: impl Into<String>,
        node_public_key: WireguardPublicKey,
        talos_version: impl Into<String>,
        join_token: Option<String>,
        grpc_tunnel: bool,
    ) -> Result<Self> {
        let node_uuid = node_uuid.into();
        let talos_version = talos_version.into();
        if node_uuid.is_empty() {
            return Err(Error::invalid("provision request node uuid is empty"));
        }
        if talos_version.is_empty() {
            return Err(Error::invalid("provision request talos version is empty"));
        }
        Ok(ProvisionRequest {
            node_uuid,
            node_public_key,
            talos_version,
            join_token,
            grpc_tunnel,
        })
    }

    /// Assemble a provision request from a [`Config`] and node identity.
    pub fn from_config(
        config: &Config,
        node_uuid: impl Into<String>,
        node_public_key: WireguardPublicKey,
        talos_version: impl Into<String>,
    ) -> Result<Self> {
        Self::new(
            node_uuid,
            node_public_key,
            talos_version,
            config.join_token().map(str::to_string),
            config.grpc_tunnel(),
        )
    }
}

/// The provision response from the SideroLink API.
///
/// Equivalent to `siderolink.ProvisionResponse`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvisionResponse {
    /// The server's WireGuard public key (the single peer the node adds).
    pub server_public_key: WireguardPublicKey,
    /// The server's WireGuard endpoint host (IP or DNS name).
    pub server_endpoint: String,
    /// The server's WireGuard endpoint UDP port.
    pub server_port: u16,
    /// The IPv6 ULA address assigned to this node on the link (a `/128`).
    pub node_address: NodeAddress,
    /// The IPv6 ULA address of the server side of the link.
    pub server_address: NodeAddress,
}

impl ProvisionResponse {
    /// The node's link address formatted as `addr/128`.
    pub fn node_address_cidr(&self) -> Result<Cidr> {
        Cidr::new(self.node_address, 128)
    }

    /// The server WireGuard endpoint as a `host:port` authority.
    pub fn server_wg_endpoint(&self) -> String {
        match self.server_address {
            NodeAddress::V6(_) if self.server_endpoint.contains(':') => {
                alloc::format!("[{}]:{}", self.server_endpoint, self.server_port)
            }
            _ => alloc::format!("{}:{}", self.server_endpoint, self.server_port),
        }
    }
}

/// The gRPC provision service boundary.
///
/// Implemented by the real gRPC client in production and by
/// [`InMemoryProvisionServer`] in tests.
pub trait ProvisionService {
    /// Perform the provision RPC, returning the server's response.
    fn provision(&mut self, request: &ProvisionRequest) -> Result<ProvisionResponse>;
}

/// An in-memory SideroLink provision server.
///
/// Allocates a deterministic, stable IPv6 address out of a configured ULA `/64`
/// per node UUID (first-seen wins, re-provisioning is idempotent), validates the
/// join token against an allowed set, and tracks every provisioned node.
#[derive(Debug, Clone)]
pub struct InMemoryProvisionServer {
    server_public_key: WireguardPublicKey,
    server_endpoint: String,
    server_port: u16,
    prefix: Cidr,
    server_address: NodeAddress,
    /// Tokens accepted; empty means "no token required".
    accepted_tokens: alloc::vec::Vec<String>,
    /// Stable assignments keyed by node UUID, plus the next free host index.
    assigned: BTreeMap<String, NodeAddress>,
    next_host: u16,
}

impl InMemoryProvisionServer {
    /// Construct a server bound to a `/64` ULA prefix.
    ///
    /// The server claims `<prefix>::1`; nodes are handed `<prefix>::1000+` to
    /// keep them clear of reserved low addresses.
    pub fn new(
        server_public_key: WireguardPublicKey,
        server_endpoint: impl Into<String>,
        server_port: u16,
        prefix: Cidr,
    ) -> Result<Self> {
        if prefix.prefix_len() != 64 {
            return Err(Error::invalid("siderolink prefix must be a /64"));
        }
        if !matches!(prefix.base(), NodeAddress::V6(_)) {
            return Err(Error::invalid("siderolink prefix must be IPv6"));
        }
        let server_address = host_in_prefix(&prefix, 1);
        Ok(InMemoryProvisionServer {
            server_public_key,
            server_endpoint: server_endpoint.into(),
            server_port,
            prefix,
            server_address,
            accepted_tokens: alloc::vec::Vec::new(),
            assigned: BTreeMap::new(),
            next_host: 0x1000,
        })
    }

    /// Restrict provisioning to requests carrying one of these join tokens.
    pub fn require_token(mut self, token: impl Into<String>) -> Self {
        self.accepted_tokens.push(token.into());
        self
    }

    /// The number of nodes provisioned so far.
    pub fn provisioned_count(&self) -> usize {
        self.assigned.len()
    }

    /// The address currently assigned to a node UUID, if any.
    pub fn address_for(&self, node_uuid: &str) -> Option<NodeAddress> {
        self.assigned.get(node_uuid).copied()
    }

    fn authorize(&self, request: &ProvisionRequest) -> Result<()> {
        if self.accepted_tokens.is_empty() {
            return Ok(());
        }
        match &request.join_token {
            Some(tok) if self.accepted_tokens.iter().any(|t| t == tok) => Ok(()),
            Some(_) => Err(Error::permission_denied("siderolink join token rejected")),
            None => Err(Error::permission_denied("siderolink join token required")),
        }
    }
}

impl ProvisionService for InMemoryProvisionServer {
    fn provision(&mut self, request: &ProvisionRequest) -> Result<ProvisionResponse> {
        self.authorize(request)?;

        // Idempotent: re-provisioning a known UUID returns the same address.
        let node_address = if let Some(addr) = self.assigned.get(&request.node_uuid) {
            *addr
        } else {
            if self.next_host == u16::MAX {
                return Err(Error::invalid_state("siderolink address pool exhausted"));
            }
            let addr = host_in_prefix(&self.prefix, self.next_host);
            self.next_host += 1;
            self.assigned.insert(request.node_uuid.clone(), addr);
            addr
        };

        Ok(ProvisionResponse {
            server_public_key: self.server_public_key.clone(),
            server_endpoint: self.server_endpoint.clone(),
            server_port: self.server_port,
            node_address,
            server_address: self.server_address,
        })
    }
}

/// Derive the address at host index `host` within an IPv6 `/64` prefix by
/// setting the low 16 bits of the interface identifier.
fn host_in_prefix(prefix: &Cidr, host: u16) -> NodeAddress {
    match prefix.network() {
        NodeAddress::V6(mut groups) => {
            groups[7] = host;
            NodeAddress::V6(groups)
        }
        // Validated to be IPv6 at construction; unreachable in practice.
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server() -> InMemoryProvisionServer {
        let prefix = Cidr::parse("fdae:41e4:649b:9303::/64").unwrap();
        InMemoryProvisionServer::new(
            WireguardPublicKey::derive_from_seed("server"),
            "siderolink.example.com",
            DEFAULT_WG_PORT,
            prefix,
        )
        .unwrap()
    }

    fn request(uuid: &str, token: Option<&str>) -> ProvisionRequest {
        ProvisionRequest::new(
            uuid,
            WireguardPublicKey::derive_from_seed(uuid),
            "v1.7.0",
            token.map(str::to_string),
            false,
        )
        .unwrap()
    }

    #[test]
    fn provision_assigns_address_in_prefix() {
        let mut srv = server();
        let resp = srv.provision(&request("uuid-a", None)).unwrap();
        let prefix = Cidr::parse("fdae:41e4:649b:9303::/64").unwrap();
        assert!(prefix.contains(&resp.node_address));
        assert_eq!(resp.server_port, DEFAULT_WG_PORT);
        assert!(prefix.contains(&resp.server_address));
        assert_eq!(srv.provisioned_count(), 1);
    }

    #[test]
    fn provision_is_idempotent_per_uuid() {
        let mut srv = server();
        let a1 = srv.provision(&request("uuid-a", None)).unwrap();
        let a2 = srv.provision(&request("uuid-a", None)).unwrap();
        assert_eq!(a1.node_address, a2.node_address);
        assert_eq!(srv.provisioned_count(), 1);
    }

    #[test]
    fn distinct_nodes_get_distinct_addresses() {
        let mut srv = server();
        let a = srv.provision(&request("uuid-a", None)).unwrap();
        let b = srv.provision(&request("uuid-b", None)).unwrap();
        assert_ne!(a.node_address, b.node_address);
        assert_eq!(srv.provisioned_count(), 2);
    }

    #[test]
    fn token_enforcement() {
        let mut srv = server().require_token("good-token");
        assert!(srv.provision(&request("uuid-a", None)).is_err());
        assert!(srv.provision(&request("uuid-a", Some("bad"))).is_err());
        let ok = srv.provision(&request("uuid-a", Some("good-token")));
        assert!(ok.is_ok());
    }

    #[test]
    fn response_helpers() {
        let mut srv = server();
        let resp = srv.provision(&request("uuid-a", None)).unwrap();
        let cidr = resp.node_address_cidr().unwrap();
        assert_eq!(cidr.prefix_len(), 128);
        assert!(
            resp.server_wg_endpoint()
                .ends_with(&DEFAULT_WG_PORT.to_string())
        );
    }

    #[test]
    fn rejects_non_64_or_ipv4_prefix() {
        let key = WireguardPublicKey::derive_from_seed("s");
        let v4 = Cidr::parse("10.0.0.0/24").unwrap();
        assert!(InMemoryProvisionServer::new(key.clone(), "h", 51820, v4).is_err());
        let wrong = Cidr::parse("fd00::/48").unwrap();
        assert!(InMemoryProvisionServer::new(key, "h", 51820, wrong).is_err());
    }

    #[test]
    fn request_validation_and_from_config() {
        assert!(
            ProvisionRequest::new(
                "",
                WireguardPublicKey::derive_from_seed("k"),
                "v1.7.0",
                None,
                false
            )
            .is_err()
        );

        let cfg = Config::parse_api_arg("https://host:443?jointoken=tok").unwrap();
        let req = ProvisionRequest::from_config(
            &cfg,
            "uuid-x",
            WireguardPublicKey::derive_from_seed("k"),
            "v1.7.0",
        )
        .unwrap();
        assert_eq!(req.join_token.as_deref(), Some("tok"));
    }
}
