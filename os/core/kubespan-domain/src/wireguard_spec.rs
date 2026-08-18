//! WireGuard device, key, and peer-config primitives for KubeSpan.
//!
//! Mirrors `pkg/machinery/resources/network` WireGuard specs and the device
//! configuration assembled by the KubeSpan manager in
//! `internal/app/machined/pkg/controllers/kubespan`. We model the *rendered*
//! key form (32-byte Curve25519 values as 44-char base64) and validate
//! length/charset/padding rather than implementing the cipher, keeping the
//! crate free of external dependencies.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use os_kernel::address::NodeAddress;
use os_kernel::error::{Error, Result};

/// Length in raw bytes of a WireGuard Curve25519 key.
pub const KEY_BYTES: usize = 32;
/// Length in characters of a base64-encoded WireGuard key (`ceil(32/3)*4`).
pub const KEY_B64_LEN: usize = 44;
/// The canonical WireGuard UDP listen port used by KubeSpan.
pub const DEFAULT_LISTEN_PORT: u16 = 51820;
/// The KubeSpan WireGuard firewall mark, used to keep KubeSpan traffic from
/// looping back through the tunnel (`network.WireguardFirewallMark`).
pub const FIREWALL_MARK: u32 = 0x51820;

/// A WireGuard key (public, private, or preshared) in canonical base64 form.
///
/// Validation enforces the 44-char length, the base64 alphabet, and the single
/// trailing `=` pad that any 32-byte value produces.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    /// Derive a deterministic, well-formed key from a seed.
    ///
    /// This is NOT a real Curve25519 key; it produces a stable, valid-looking
    /// key for tests and for modeling derivation from a node seed.
    pub fn derive_from_seed(seed: &str) -> Self {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut state = os_kernel::id::Fingerprint::of_str(seed).value();
        let mut out = String::with_capacity(KEY_B64_LEN);
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

    /// Whether two keys are equal (used for peer matching).
    pub fn matches(&self, other: &WireguardKey) -> bool {
        self == other
    }
}

impl fmt::Display for WireguardKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encoded)
    }
}

impl fmt::Debug for WireguardKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Avoid leaking full key material into debug logs; show a short prefix.
        let prefix: String = self.encoded.chars().take(8).collect();
        write!(f, "WireguardKey({prefix}…)")
    }
}

/// An `address/prefix` CIDR used as a WireGuard `AllowedIP`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AllowedIp {
    addr: NodeAddress,
    prefix: u8,
}

impl Ord for AllowedIp {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        addr_sort_key(&self.addr)
            .cmp(&addr_sort_key(&other.addr))
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
    pub fn new(addr: NodeAddress, prefix: u8) -> Result<Self> {
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
        let addr = NodeAddress::parse_v4(host)?;
        let prefix: u8 = prefix
            .parse()
            .map_err(|_| Error::parse("invalid allowed-ip prefix"))?;
        Self::new(addr, prefix)
    }

    /// A host route (`/32` or `/128`) for a single node address.
    pub fn host_route(addr: NodeAddress) -> Self {
        let prefix = if addr.is_v4() { 32 } else { 128 };
        AllowedIp { addr, prefix }
    }

    /// The network address.
    pub fn addr(&self) -> NodeAddress {
        self.addr
    }

    /// The prefix length.
    pub fn prefix(&self) -> u8 {
        self.prefix
    }
}

impl fmt::Display for AllowedIp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.addr, self.prefix)
    }
}

/// Stable sort key for addresses: IPv4 before IPv6, then by raw octets.
pub(crate) fn addr_sort_key(addr: &NodeAddress) -> ([u8; 17], ()) {
    let mut key = [0u8; 17];
    match addr {
        NodeAddress::V4(o) => {
            key[0] = 0;
            key[1..5].copy_from_slice(o);
        }
        NodeAddress::V6(g) => {
            key[0] = 1;
            for (i, seg) in g.iter().enumerate() {
                key[1 + i * 2] = (seg >> 8) as u8;
                key[2 + i * 2] = (*seg & 0xff) as u8;
            }
        }
    }
    (key, ())
}

/// A WireGuard peer entry within a device spec.
///
/// Mirrors `network.WireguardPeer`: a public key, optional preshared key, a
/// persistent-keepalive interval, an endpoint (`ip:port`) and the set of
/// allowed-ips routed to this peer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireguardPeerSpec {
    pub public_key: WireguardKey,
    pub preshared_key: Option<WireguardKey>,
    pub endpoint: Option<(NodeAddress, u16)>,
    pub persistent_keepalive_secs: u32,
    pub allowed_ips: Vec<AllowedIp>,
}

impl WireguardPeerSpec {
    /// Construct a peer spec with no endpoint, no PSK and no keepalive.
    pub fn new(public_key: WireguardKey) -> Self {
        WireguardPeerSpec {
            public_key,
            preshared_key: None,
            endpoint: None,
            persistent_keepalive_secs: 0,
            allowed_ips: Vec::new(),
        }
    }

    /// Set the endpoint.
    pub fn with_endpoint(mut self, addr: NodeAddress, port: u16) -> Self {
        self.endpoint = Some((addr, port));
        self
    }

    /// Set the persistent-keepalive interval (seconds).
    pub fn with_keepalive(mut self, secs: u32) -> Self {
        self.persistent_keepalive_secs = secs;
        self
    }

    /// Add an allowed-ip, keeping the list sorted and deduplicated.
    pub fn add_allowed_ip(&mut self, ip: AllowedIp) {
        if let Err(pos) = self.allowed_ips.binary_search(&ip) {
            self.allowed_ips.insert(pos, ip);
        }
    }

    /// Validate the peer config for use in a device spec.
    pub fn validate(&self) -> Result<()> {
        if let Some((_, port)) = self.endpoint
            && port == 0
        {
            return Err(Error::invalid("peer endpoint port must be non-zero"));
        }
        Ok(())
    }
}

/// A WireGuard device spec: the local interface key material plus the peer set.
///
/// Mirrors `network.WireguardSpec` as assembled by the KubeSpan manager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireguardDeviceSpec {
    pub private_key: WireguardKey,
    pub listen_port: u16,
    pub firewall_mark: u32,
    pub peers: Vec<WireguardPeerSpec>,
}

impl WireguardDeviceSpec {
    /// Construct a device spec with the KubeSpan defaults and no peers.
    pub fn new(private_key: WireguardKey) -> Self {
        WireguardDeviceSpec {
            private_key,
            listen_port: DEFAULT_LISTEN_PORT,
            firewall_mark: FIREWALL_MARK,
            peers: Vec::new(),
        }
    }

    /// Replace the peer set, ordering peers by public key for a stable spec.
    pub fn set_peers(&mut self, mut peers: Vec<WireguardPeerSpec>) {
        peers.sort_by(|a, b| a.public_key.cmp(&b.public_key));
        self.peers = peers;
    }

    /// Look up a peer by public key.
    pub fn peer(&self, key: &WireguardKey) -> Option<&WireguardPeerSpec> {
        self.peers.iter().find(|p| &p.public_key == key)
    }

    /// Validate the device and all peers.
    pub fn validate(&self) -> Result<()> {
        if self.listen_port == 0 {
            return Err(Error::invalid("device listen port must be non-zero"));
        }
        for p in &self.peers {
            p.validate()?;
        }
        Ok(())
    }

    /// A change-detection fingerprint over the rendered spec. The manager uses
    /// this to skip pushing an identical device spec to the kernel.
    pub fn fingerprint(&self) -> os_kernel::id::Fingerprint {
        let mut buf = String::new();
        buf.push_str(self.private_key.as_str());
        buf.push('|');
        buf.push_str(&self.listen_port.to_string());
        buf.push('|');
        buf.push_str(&self.firewall_mark.to_string());
        for p in &self.peers {
            buf.push('\n');
            buf.push_str(p.public_key.as_str());
            if let Some((addr, port)) = p.endpoint {
                buf.push('@');
                buf.push_str(&addr.to_string());
                buf.push(':');
                buf.push_str(&port.to_string());
            }
            buf.push_str(":k");
            buf.push_str(&p.persistent_keepalive_secs.to_string());
            for ip in &p.allowed_ips {
                buf.push(',');
                buf.push_str(&ip.to_string());
            }
        }
        os_kernel::id::Fingerprint::of_str(&buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_parse_validates_format() {
        let k = WireguardKey::derive_from_seed("node-a");
        assert_eq!(k.as_str().len(), KEY_B64_LEN);
        assert!(WireguardKey::parse(k.as_str()).is_ok());
        assert!(WireguardKey::parse("tooshort=").is_err());
        // 44 chars but no trailing pad.
        let nopad: String = "A".repeat(44);
        assert!(WireguardKey::parse(nopad).is_err());
        // invalid char.
        let bad: String = core::iter::repeat_n('A', 43)
            .chain(core::iter::once('='))
            .collect::<String>();
        assert!(WireguardKey::parse(bad).is_ok());
        let bad2: String = core::iter::repeat_n('!', 43)
            .chain(core::iter::once('='))
            .collect();
        assert!(WireguardKey::parse(bad2).is_err());
    }

    #[test]
    fn key_derivation_is_deterministic() {
        let a = WireguardKey::derive_from_seed("seed-x");
        let b = WireguardKey::derive_from_seed("seed-x");
        let c = WireguardKey::derive_from_seed("seed-y");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(a.matches(&b));
    }

    #[test]
    fn key_debug_does_not_leak_full_material() {
        let k = WireguardKey::derive_from_seed("secret");
        let dbg = alloc::format!("{k:?}");
        assert!(!dbg.contains(k.as_str()));
        assert!(dbg.starts_with("WireguardKey("));
    }

    #[test]
    fn allowed_ip_prefix_validation_and_sort() {
        assert!(AllowedIp::parse_v4("10.0.0.0/8").is_ok());
        assert!(AllowedIp::parse_v4("10.0.0.0/33").is_err());
        assert!(AllowedIp::parse_v4("10.0.0.0").is_err());
        let host = AllowedIp::host_route(NodeAddress::parse_v4("10.244.0.5").unwrap());
        assert_eq!(host.prefix(), 32);
    }

    #[test]
    fn peer_allowed_ips_are_sorted_and_deduped() {
        let mut p = WireguardPeerSpec::new(WireguardKey::derive_from_seed("p"));
        p.add_allowed_ip(AllowedIp::parse_v4("10.0.2.0/24").unwrap());
        p.add_allowed_ip(AllowedIp::parse_v4("10.0.1.0/24").unwrap());
        p.add_allowed_ip(AllowedIp::parse_v4("10.0.1.0/24").unwrap());
        assert_eq!(p.allowed_ips.len(), 2);
        assert_eq!(p.allowed_ips[0].to_string(), "10.0.1.0/24");
    }

    #[test]
    fn device_spec_defaults_and_fingerprint_changes() {
        let mut dev = WireguardDeviceSpec::new(WireguardKey::derive_from_seed("priv"));
        assert_eq!(dev.listen_port, DEFAULT_LISTEN_PORT);
        assert_eq!(dev.firewall_mark, FIREWALL_MARK);
        let fp0 = dev.fingerprint();
        dev.set_peers(alloc::vec![WireguardPeerSpec::new(
            WireguardKey::derive_from_seed("peer-1")
        )]);
        let fp1 = dev.fingerprint();
        assert_ne!(fp0, fp1);
        assert!(dev.validate().is_ok());
    }

    #[test]
    fn device_peer_lookup_and_ordering() {
        let mut dev = WireguardDeviceSpec::new(WireguardKey::derive_from_seed("priv"));
        let k1 = WireguardKey::derive_from_seed("zzz");
        let k2 = WireguardKey::derive_from_seed("aaa");
        dev.set_peers(alloc::vec![
            WireguardPeerSpec::new(k1.clone()),
            WireguardPeerSpec::new(k2.clone()),
        ]);
        // peers sorted by key, so ordering is deterministic regardless of input.
        assert!(dev.peers[0].public_key <= dev.peers[1].public_key);
        assert!(dev.peer(&k1).is_some());
        assert!(dev.peer(&k2).is_some());
    }
}
