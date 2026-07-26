//! Cluster and node identity.
//!
//! Mirrors `cluster.Identity` in Talos (`pkg/machinery/resources/cluster`). The
//! node identity is a random, stable, base62-ish opaque string used to address a
//! node in the discovery service independent of its IPs or hostname. The cluster
//! identity is derived from the cluster CA / id.

use os_kernel::error::{Error, Result};
use os_kernel::id::Fingerprint;

/// The length, in characters, of a canonical node identity string.
///
/// Talos uses a 32-byte random value rendered base62; we model the rendered
/// identity as a fixed-width string and validate its length/charset.
pub const NODE_IDENTITY_LEN: usize = 43;

/// A stable, opaque node identity used to key affiliates in the discovery
/// service and the cluster membership resources.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identity {
    value: String,
}

impl Identity {
    /// Validate and wrap an identity string.
    ///
    /// Identities are non-empty and restricted to the URL-safe base64 / base62
    /// alphabet (`[A-Za-z0-9_-]`) so they round-trip safely through protobuf and
    /// certificate fields.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value: String = value.into();
        if value.is_empty() {
            return Err(Error::invalid("node identity is empty"));
        }
        if value.len() > 128 {
            return Err(Error::invalid("node identity too long"));
        }
        for c in value.chars() {
            if !(c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                return Err(Error::invalid("node identity has invalid character"));
            }
        }
        Ok(Identity { value })
    }

    /// Derive a deterministic identity from a seed (used in tests and to derive a
    /// stable identity from a machine token). The real implementation draws from
    /// the platform CSPRNG; here we expand a seed via the fingerprint primitive.
    pub fn derive_from_seed(seed: &str) -> Self {
        let a = Fingerprint::of_str(seed).to_hex();
        let b = Fingerprint::of_str(&format!("{seed}:salt")).to_hex();
        let c = Fingerprint::of_str(&format!("salt:{seed}")).to_hex();
        // 16 + 16 + 11 = 43 chars, matching NODE_IDENTITY_LEN.
        let mut s = String::with_capacity(NODE_IDENTITY_LEN);
        s.push_str(&a);
        s.push_str(&b);
        s.push_str(&c[..NODE_IDENTITY_LEN - 32]);
        Identity { value: s }
    }

    /// The identity as a string slice.
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Whether this identity has the canonical full length.
    pub fn is_canonical_len(&self) -> bool {
        self.value.len() == NODE_IDENTITY_LEN
    }
}

impl core::fmt::Display for Identity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.value)
    }
}

/// The cluster-wide identity: a shared cluster id plus the local node's
/// identity. Mirrors the way Talos stores both on the `Identity` resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterIdentity {
    cluster_id: String,
    node: Identity,
}

impl ClusterIdentity {
    /// Construct a cluster identity, validating the cluster id is non-empty.
    pub fn new(cluster_id: impl Into<String>, node: Identity) -> Result<Self> {
        let cluster_id = cluster_id.into();
        if cluster_id.is_empty() {
            return Err(Error::invalid("cluster id is empty"));
        }
        Ok(ClusterIdentity { cluster_id, node })
    }

    /// The shared cluster id.
    pub fn cluster_id(&self) -> &str {
        &self.cluster_id
    }

    /// The local node identity.
    pub fn node(&self) -> &Identity {
        &self.node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_validation() {
        assert!(Identity::new("abcDEF_-123").is_ok());
        assert!(Identity::new("").is_err());
        assert!(Identity::new("has space").is_err());
        assert!(Identity::new("has/slash").is_err());
    }

    #[test]
    fn derive_is_deterministic_and_canonical_length() {
        let a = Identity::derive_from_seed("node-token-1");
        let b = Identity::derive_from_seed("node-token-1");
        let c = Identity::derive_from_seed("node-token-2");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(a.is_canonical_len());
        assert_eq!(a.as_str().len(), NODE_IDENTITY_LEN);
    }

    #[test]
    fn cluster_identity_requires_id() {
        let node = Identity::new("nodeid").unwrap();
        assert!(ClusterIdentity::new("", node.clone()).is_err());
        let ci = ClusterIdentity::new("cluster-xyz", node.clone()).unwrap();
        assert_eq!(ci.cluster_id(), "cluster-xyz");
        assert_eq!(ci.node(), &node);
    }
}
