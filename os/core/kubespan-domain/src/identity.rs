//! KubeSpan node identity.
//!
//! Mirrors `pkg/machinery/resources/kubespan.Identity` and the
//! `IdentityController` in `internal/app/machined/pkg/controllers/kubespan`.
//!
//! A node's KubeSpan identity is a WireGuard key pair plus a stable ULA IPv6
//! address derived from the cluster id and the node's public key. The address
//! lives in the `fd...::/64` unique-local range Talos reserves for KubeSpan and
//! is used as the WireGuard `AllowedIP` and overlay address for the node.

use crate::wireguard_spec::WireguardKey;
use alloc::string::String;
use os_kernel::address::NodeAddress;
use os_kernel::error::{Error, Result};

/// A KubeSpan identity: the node's WireGuard key pair and derived overlay
/// address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubeSpanIdentity {
    private_key: WireguardKey,
    public_key: WireguardKey,
    address: NodeAddress,
}

impl KubeSpanIdentity {
    /// Build an identity from explicit key material and cluster id.
    ///
    /// The overlay address is derived deterministically from `cluster_id` and
    /// the public key so every node in the cluster computes a consistent
    /// address for any peer.
    pub fn new(
        cluster_id: &str,
        private_key: WireguardKey,
        public_key: WireguardKey,
    ) -> Result<Self> {
        if cluster_id.is_empty() {
            return Err(Error::invalid("cluster id is empty"));
        }
        let address = derive_address(cluster_id, &public_key);
        Ok(KubeSpanIdentity {
            private_key,
            public_key,
            address,
        })
    }

    /// Deterministically generate an identity from a node seed and cluster id.
    ///
    /// In real Talos the key pair comes from Curve25519 key generation; here we
    /// derive both keys from the seed so tests are reproducible. The public key
    /// is derived from a distinct seed so it differs from the private key.
    pub fn generate(cluster_id: &str, node_seed: &str) -> Result<Self> {
        if node_seed.is_empty() {
            return Err(Error::invalid("node seed is empty"));
        }
        let private_key = WireguardKey::derive_from_seed(node_seed);
        let public_key = WireguardKey::derive_from_seed(&alloc::format!("{node_seed}:public"));
        Self::new(cluster_id, private_key, public_key)
    }

    /// The node's WireGuard private key.
    pub fn private_key(&self) -> &WireguardKey {
        &self.private_key
    }

    /// The node's WireGuard public key.
    pub fn public_key(&self) -> &WireguardKey {
        &self.public_key
    }

    /// The node's KubeSpan overlay (ULA IPv6) address.
    pub fn address(&self) -> NodeAddress {
        self.address
    }

    /// Validate the identity: keys must round-trip and address must be a ULA.
    pub fn validate(&self) -> Result<()> {
        WireguardKey::parse(self.private_key.as_str())?;
        WireguardKey::parse(self.public_key.as_str())?;
        if !self.address.is_private() {
            return Err(Error::invalid_state(
                "kubespan address is not in the unique-local range",
            ));
        }
        Ok(())
    }
}

/// Derive a `fd00::/8` ULA IPv6 address from the cluster id and public key.
///
/// The first byte is `0xfd` (a locally-assigned ULA), the next 5 bytes are
/// hashed from the cluster id (the ULA global id / subnet id), and the low
/// 64 bits are hashed from the public key (the interface identifier).
fn derive_address(cluster_id: &str, public_key: &WireguardKey) -> NodeAddress {
    let net_hash = os_kernel::id::Fingerprint::of_str(cluster_id).value();
    let host_hash = os_kernel::id::Fingerprint::of_str(public_key.as_str()).value();

    let mut groups = [0u16; 8];
    // fd + 7 bytes of network/subnet id (cluster-scoped).
    groups[0] = 0xfd00 | ((net_hash >> 56) as u16 & 0x00ff);
    groups[1] = (net_hash >> 40) as u16;
    groups[2] = (net_hash >> 24) as u16;
    groups[3] = (net_hash >> 8) as u16;
    // interface identifier derived from the public key.
    groups[4] = (host_hash >> 48) as u16;
    groups[5] = (host_hash >> 32) as u16;
    groups[6] = (host_hash >> 16) as u16;
    groups[7] = host_hash as u16;
    NodeAddress::V6(groups)
}

/// Render an identity's address as a `/128` overlay string for diagnostics.
pub fn overlay_cidr(identity: &KubeSpanIdentity) -> String {
    alloc::format!("{}/128", identity.address())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_identity_is_deterministic() {
        let a = KubeSpanIdentity::generate("cluster-1", "node-a").unwrap();
        let b = KubeSpanIdentity::generate("cluster-1", "node-a").unwrap();
        assert_eq!(a, b);
        assert_eq!(a.address(), b.address());
    }

    #[test]
    fn distinct_nodes_get_distinct_addresses() {
        let a = KubeSpanIdentity::generate("cluster-1", "node-a").unwrap();
        let b = KubeSpanIdentity::generate("cluster-1", "node-b").unwrap();
        assert_ne!(a.public_key(), b.public_key());
        assert_ne!(a.address(), b.address());
    }

    #[test]
    fn cluster_id_affects_network_portion() {
        let a = KubeSpanIdentity::generate("cluster-1", "node-a").unwrap();
        let b = KubeSpanIdentity::generate("cluster-2", "node-a").unwrap();
        // same node seed -> same keys, but different cluster -> different addr.
        assert_eq!(a.public_key(), b.public_key());
        assert_ne!(a.address(), b.address());
    }

    #[test]
    fn address_is_unique_local() {
        let id = KubeSpanIdentity::generate("cluster-1", "node-a").unwrap();
        assert!(id.address().is_private());
        match id.address() {
            NodeAddress::V6(g) => assert_eq!(g[0] & 0xff00, 0xfd00),
            _ => panic!("expected IPv6"),
        }
        assert!(id.validate().is_ok());
    }

    #[test]
    fn private_and_public_keys_differ() {
        let id = KubeSpanIdentity::generate("cluster-1", "node-a").unwrap();
        assert_ne!(id.private_key(), id.public_key());
    }

    #[test]
    fn empty_inputs_rejected() {
        assert!(KubeSpanIdentity::generate("", "node-a").is_err());
        assert!(KubeSpanIdentity::generate("cluster-1", "").is_err());
    }

    #[test]
    fn overlay_cidr_formats_address() {
        let id = KubeSpanIdentity::generate("cluster-1", "node-a").unwrap();
        let cidr = overlay_cidr(&id);
        assert!(cidr.ends_with("/128"));
        assert!(cidr.starts_with("fd"));
    }
}
