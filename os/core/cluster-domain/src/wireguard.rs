//! WireGuard key and peer primitives shared by KubeSpan and SideroLink.
//!
//! Mirrors `pkg/machinery/resources/network/wireguard` and the KubeSpan peer
//! handling in `internal/app/machined/pkg/controllers/kubespan`. Keys are the
//! standard 32-byte Curve25519 values rendered as 44-character base64. We model
//! the rendered key form and validate length/charset rather than implementing
//! the cipher, keeping the crate free of external dependencies.

use os_kernel::error::{Error, Result};

/// Length in raw bytes of a WireGuard Curve25519 key.
pub const KEY_BYTES: usize = 32;
/// Length in characters of a base64-encoded WireGuard key (`ceil(32/3)*4`).
pub const KEY_B64_LEN: usize = 44;
/// The canonical WireGuard UDP listen port used by KubeSpan.
pub const DEFAULT_LISTEN_PORT: u16 = 51820;

/// A WireGuard key (public or preshared) in its canonical base64 string form.
///
/// Validation enforces the 44-char length, the base64 alphabet, and the single
/// trailing `=` pad that a 32-byte value always produces.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WireguardKey {
    encoded: String,
}

impl WireguardKey {
    /// Validate and wrap a base64 key string.
    pub fn parse(s: impl Into<String>) -> Result<Self> {
        let encoded: String = s.into();
        if encoded.len() != KEY_B64_LEN {
            return Err(Error::invalid("wireguard key must be 44 base64 characters"));
        }
        if !encoded.ends_with('=') {
            return Err(Error::invalid("wireguard key must end with base64 padding"));
        }
        for (i, c) in encoded.char_indices() {
            let is_b64 = c.is_ascii_alphanumeric() || c == '+' || c == '/';
            let is_pad = c == '=' && i == KEY_B64_LEN - 1;
            if !(is_b64 || is_pad) {
                return Err(Error::invalid("wireguard key has invalid base64 character"));
            }
        }
        Ok(WireguardKey { encoded })
    }

    /// Derive a deterministic, well-formed key from a seed. This is NOT a real
    /// Curve25519 key; it produces a stable, valid-looking key for tests and for
    /// modeling derivation from a node seed.
    pub fn derive_from_seed(seed: &str) -> Self {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut state = os_kernel::id::Fingerprint::of_str(seed).value();
        let mut out = String::with_capacity(KEY_B64_LEN);
        // 43 data chars + 1 pad.
        for _ in 0..KEY_B64_LEN - 1 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            out.push(ALPHABET[(state % 64) as usize] as char);
        }
        out.push('=');
        WireguardKey { encoded: out }
    }

    /// The encoded key string.
    pub fn as_str(&self) -> &str {
        &self.encoded
    }

    /// The corresponding public key of a private key is not derivable without
    /// the cipher; this returns whether two keys are equal (peer matching).
    pub fn matches(&self, other: &WireguardKey) -> bool {
        self == other
    }
}

impl core::fmt::Display for WireguardKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.encoded)
    }
}

/// An `address/prefix` CIDR used as a WireGuard `AllowedIP`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AllowedIp {
    addr: os_kernel::address::NodeAddress,
    prefix: u8,
}

impl Ord for AllowedIp {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        crate::addr_sort_key(&self.addr)
            .cmp(&crate::addr_sort_key(&other.addr))
            .then(self.prefix.cmp(&other.prefix))
    }
}

impl PartialOrd for AllowedIp {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl AllowedIp {
    /// Construct an allowed-ip, validating the prefix length against the family.
    pub fn new(addr: os_kernel::address::NodeAddress, prefix: u8) -> Result<Self> {
        let max = if addr.is_v4() { 32 } else { 128 };
        if prefix > max {
            return Err(Error::invalid("allowed-ip prefix out of range"));
        }
        Ok(AllowedIp { addr, prefix })
    }

    /// Parse an `a.b.c.d/n` IPv4 CIDR.
    pub fn parse_v4(s: &str) -> Result<Self> {
        let (host, prefix) = s
            .split_once('/')
            .ok_or_else(|| Error::parse("allowed-ip missing '/prefix'"))?;
        let addr = os_kernel::address::NodeAddress::parse_v4(host)?;
        let prefix: u8 = prefix
            .parse()
            .map_err(|_| Error::parse("invalid allowed-ip prefix"))?;
        Self::new(addr, prefix)
    }

    /// The network prefix length.
    pub fn prefix(&self) -> u8 {
        self.prefix
    }

    /// Whether this allowed-ip is a single-host route.
    pub fn is_host_route(&self) -> bool {
        (self.addr.is_v4() && self.prefix == 32) || (!self.addr.is_v4() && self.prefix == 128)
    }
}

impl core::fmt::Display for AllowedIp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}/{}", self.addr, self.prefix)
    }
}

/// A configured WireGuard peer: the public key, its endpoints, persistent
/// keepalive, and the set of allowed IPs routed to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireguardPeer {
    public_key: WireguardKey,
    endpoint: Option<crate::endpoint::ClusterEndpoint>,
    persistent_keepalive_secs: u16,
    allowed_ips: Vec<AllowedIp>,
}

impl WireguardPeer {
    /// Construct a peer with no endpoint and no keepalive.
    pub fn new(public_key: WireguardKey) -> Self {
        WireguardPeer {
            public_key,
            endpoint: None,
            persistent_keepalive_secs: 0,
            allowed_ips: Vec::new(),
        }
    }

    /// Set the peer endpoint (builder style).
    pub fn with_endpoint(mut self, ep: crate::endpoint::ClusterEndpoint) -> Self {
        self.endpoint = Some(ep);
        self
    }

    /// Set persistent keepalive in seconds (builder style). Talos uses 25s by
    /// default for NAT traversal in KubeSpan.
    pub fn with_keepalive(mut self, secs: u16) -> Self {
        self.persistent_keepalive_secs = secs;
        self
    }

    /// Add an allowed-ip route to this peer.
    pub fn add_allowed_ip(&mut self, ip: AllowedIp) {
        if !self.allowed_ips.contains(&ip) {
            self.allowed_ips.push(ip);
        }
    }

    /// The peer's public key.
    pub fn public_key(&self) -> &WireguardKey {
        &self.public_key
    }

    /// The peer's endpoint, if known.
    pub fn endpoint(&self) -> Option<crate::endpoint::ClusterEndpoint> {
        self.endpoint
    }

    /// The peer's allowed IPs.
    pub fn allowed_ips(&self) -> &[AllowedIp] {
        &self.allowed_ips
    }

    /// The keepalive interval in seconds (0 == disabled).
    pub fn keepalive_secs(&self) -> u16 {
        self.persistent_keepalive_secs
    }

    /// Whether the peer is fully usable: has a routable endpoint and at least one
    /// allowed-ip. KubeSpan only programs peers that satisfy this.
    pub fn is_programmable(&self) -> bool {
        self.endpoint.is_some_and(|e| e.is_routable()) && !self.allowed_ips.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endpoint::ClusterEndpoint;

    #[test]
    fn key_validation() {
        let k = WireguardKey::derive_from_seed("node-a");
        assert!(WireguardKey::parse(k.as_str()).is_ok());
        assert_eq!(k.as_str().len(), KEY_B64_LEN);
        assert!(WireguardKey::parse("short").is_err());
        // wrong length but ends with =
        assert!(WireguardKey::parse("abc=").is_err());
    }

    #[test]
    fn derived_keys_are_deterministic_and_distinct() {
        assert_eq!(
            WireguardKey::derive_from_seed("x"),
            WireguardKey::derive_from_seed("x")
        );
        assert_ne!(
            WireguardKey::derive_from_seed("x"),
            WireguardKey::derive_from_seed("y")
        );
    }

    #[test]
    fn allowed_ip_prefix_validation() {
        assert!(AllowedIp::parse_v4("10.0.0.0/8").unwrap().prefix() == 8);
        assert!(AllowedIp::parse_v4("10.0.0.1/32").unwrap().is_host_route());
        assert!(AllowedIp::parse_v4("10.0.0.0/33").is_err());
        assert!(AllowedIp::parse_v4("10.0.0.0").is_err());
    }

    #[test]
    fn peer_programmable_requires_endpoint_and_route() {
        let key = WireguardKey::derive_from_seed("peer");
        let mut peer = WireguardPeer::new(key)
            .with_endpoint(ClusterEndpoint::parse_v4("192.168.1.2:51820").unwrap())
            .with_keepalive(25);
        assert!(!peer.is_programmable());
        peer.add_allowed_ip(AllowedIp::parse_v4("10.244.0.0/24").unwrap());
        assert!(peer.is_programmable());
        assert_eq!(peer.keepalive_secs(), 25);

        // loopback endpoint is not routable.
        let key2 = WireguardKey::derive_from_seed("peer2");
        let mut peer2 = WireguardPeer::new(key2)
            .with_endpoint(ClusterEndpoint::parse_v4("127.0.0.1:51820").unwrap());
        peer2.add_allowed_ip(AllowedIp::parse_v4("10.244.1.0/24").unwrap());
        assert!(!peer2.is_programmable());
    }
}
