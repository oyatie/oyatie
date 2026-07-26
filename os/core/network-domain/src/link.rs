//! Link specs and link state.
//!
//! Mirrors `network.LinkSpec`, `LinkConfigController` and `LinkStatusController`.
//! A [`LinkSpec`] is the desired state of a network interface (admin up/down,
//! MTU, kind, bonding/bridge/vlan parameters); link state tracks the observed
//! operational state.

use crate::config_layer::ConfigLayer;
use crate::nethelpers::VlanProtocol;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use os_kernel::error::{Error, Result};

const MAX_LINK_NAME_LENGTH: usize = 15;

/// Address family for addresses and routes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressFamily {
    /// IPv4.
    Inet4,
    /// IPv6.
    Inet6,
}

impl AddressFamily {
    /// Maximum CIDR prefix length for the family.
    pub fn max_prefix_len(self) -> u8 {
        match self {
            AddressFamily::Inet4 => 32,
            AddressFamily::Inet6 => 128,
        }
    }
}

/// Return Talos' VLAN child link name for a base link and VLAN id.
///
/// Mirrors `pkg/machinery/nethelpers.VLANLinkName`: short bases use
/// `<base>.<vlan>`, while long bases keep the first four bytes and append the
/// first three SHA-256 bytes of the base name as six lowercase hex characters.
/// The upstream helper deliberately budgets for the longest `.4095` suffix even
/// when a shorter VLAN id is supplied; matching that behavior keeps generated
/// DHCP/link specs source-compatible.
pub fn vlan_link_name(base: &str, vlan_id: u16) -> String {
    if base.len() + 5 <= MAX_LINK_NAME_LENGTH {
        return alloc::format!("{base}.{vlan_id}");
    }

    let base_bytes = base.as_bytes();
    let prefix_len = 4;
    let hash_bytes_count = (MAX_LINK_NAME_LENGTH - prefix_len - 5) / 2;
    let digest = sha256(base_bytes);
    let hex = b"0123456789abcdef";

    let mut name = String::with_capacity(prefix_len + (hash_bytes_count * 2) + 5);
    for &byte in &base_bytes[..prefix_len] {
        name.push(char::from(byte));
    }
    for &byte in &digest[..hash_bytes_count] {
        name.push(char::from(hex[usize::from(byte >> 4)]));
        name.push(char::from(hex[usize::from(byte & 0x0f)]));
    }
    name.push('.');
    name.push_str(&vlan_id.to_string());
    name
}

/// The kind of a link, mirroring the subset of `rtnl` link kinds Talos
/// configures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkKind {
    /// A physical interface (no special kind).
    Physical,
    /// A bond aggregating member interfaces.
    Bond {
        members: Vec<String>,
        mode: BondMode,
    },
    /// A bridge with member interfaces.
    Bridge { members: Vec<String> },
    /// A VLAN sub-interface of a parent link.
    Vlan {
        parent: String,
        vlan_id: u16,
        protocol: VlanProtocol,
    },
    /// A dummy interface.
    Dummy,
}

impl LinkKind {
    /// The kind string as the kernel/netlink uses it.
    pub fn kind_str(&self) -> &'static str {
        match self {
            LinkKind::Physical => "",
            LinkKind::Bond { .. } => "bond",
            LinkKind::Bridge { .. } => "bridge",
            LinkKind::Vlan { .. } => "vlan",
            LinkKind::Dummy => "dummy",
        }
    }

    /// Whether this link is virtual (created by Talos rather than discovered).
    pub fn is_virtual(&self) -> bool {
        !matches!(self, LinkKind::Physical)
    }
}

/// Bonding mode (subset of Linux bonding modes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BondMode {
    /// active-backup
    ActiveBackup,
    /// balance-rr (round robin)
    BalanceRr,
    /// 802.3ad LACP
    Lacp,
}

/// Operational state of a link (mirrors `IF_OPER_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperState {
    /// Operationally up.
    Up,
    /// Operationally down.
    Down,
    /// State unknown / not reported.
    Unknown,
}

/// Observed kernel link type.
///
/// Talos' `LinkStatus.Physical()` treats only `ether` links with an empty
/// rtnetlink kind as physical; virtual links such as VLANs and bonds usually
/// still have type `ether` but carry a non-empty kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkType {
    /// Ethernet link type.
    Ether,
    /// Any other kernel link type, preserved fail-visibly for fingerprints.
    Other(String),
}

impl LinkType {
    /// Stable string form used in status fingerprints.
    pub fn as_str(&self) -> &str {
        match self {
            LinkType::Ether => "ether",
            LinkType::Other(value) => value.as_str(),
        }
    }
}

/// Desired configuration for a network link.
///
/// Equivalent to `network.LinkSpecSpec`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkSpec {
    /// Interface name (e.g. `eth0`, `bond0`).
    pub name: String,
    /// Whether the interface should be administratively up.
    pub up: bool,
    /// MTU in bytes.
    ///
    /// A value of `0` means "unspecified" and mirrors source-layer Talos specs
    /// before kernel/default materialization.
    pub mtu: u32,
    /// Optional multicast setting; `None` means the source did not express it.
    pub multicast: Option<bool>,
    /// Link kind and its parameters.
    pub kind: LinkKind,
    /// Provenance / priority.
    pub layer: ConfigLayer,
}

impl LinkSpec {
    /// A physical link brought up with a default MTU.
    pub fn physical(name: impl Into<String>, layer: ConfigLayer) -> Self {
        LinkSpec {
            name: name.into(),
            up: true,
            mtu: 1500,
            multicast: None,
            kind: LinkKind::Physical,
            layer,
        }
    }

    /// Stable logical id for this link spec.
    ///
    /// Source-layer specs use the pure link name as their identity; the layer
    /// only decides precedence during merging and is deliberately not part of
    /// the id.
    pub fn id(&self) -> String {
        self.name.clone()
    }

    /// Validate the link spec invariants.
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::invalid("link spec has empty name"));
        }
        if self.name.len() > 15 {
            // IFNAMSIZ - 1
            return Err(Error::invalid("link name exceeds 15 characters"));
        }
        if self.mtu != 0 && !(68..=65535).contains(&self.mtu) {
            return Err(Error::invalid(alloc::format!(
                "MTU {} out of range 68..=65535",
                self.mtu
            )));
        }
        match &self.kind {
            LinkKind::Bond { members, .. } if members.is_empty() => {
                Err(Error::invalid("bond link has no members"))
            }
            LinkKind::Bridge { members } if members.is_empty() => {
                Err(Error::invalid("bridge link has no members"))
            }
            LinkKind::Vlan {
                vlan_id, parent, ..
            } => {
                if *vlan_id == 0 || *vlan_id > 4094 {
                    return Err(Error::invalid("VLAN id must be in 1..=4094"));
                }
                if parent.is_empty() {
                    return Err(Error::invalid("VLAN has empty parent"));
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

/// In-memory merge of link specs by logical link id, applying layer priority.
///
/// Equivalent to `LinkMergeController`: for each logical link name the
/// highest-priority layer wins. Returns the merged specs sorted by id.
pub fn merge_links(specs: &[LinkSpec]) -> Vec<LinkSpec> {
    let mut by_id: BTreeMap<String, LinkSpec> = BTreeMap::new();
    for spec in specs {
        let id = spec.id();
        match by_id.get(&id) {
            Some(existing) if existing.layer.precedence() >= spec.layer.precedence() => {}
            _ => {
                by_id.insert(id, spec.clone());
            }
        }
    }
    by_id.into_values().collect()
}

/// Observed status of a link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkStatus {
    /// Interface name.
    pub name: String,
    /// Kernel link type (Talos uses `ether` for physical interface detection).
    pub link_type: LinkType,
    /// Rtnetlink kind. Physical Ethernet links have an empty kind.
    pub kind: String,
    /// Alternate names Talos also considers when matching configured links.
    pub aliases: Vec<String>,
    /// Whether the admin flag is up.
    pub admin_up: bool,
    /// Operational state.
    pub oper_state: OperState,
    /// Whether carrier (physical link) is present.
    pub carrier: bool,
    /// Hardware (MAC) address as six octets.
    pub hardware_addr: [u8; 6],
    /// Effective MTU.
    pub mtu: u32,
}

impl LinkStatus {
    /// Whether this status matches Talos' `Physical()` predicate.
    pub fn physical(&self) -> bool {
        self.link_type == LinkType::Ether && self.kind.is_empty()
    }

    /// Whether the link is eligible for Talos' default DHCPv4 operator.
    ///
    /// Source Talos gates default DHCP on `Physical()` plus controller-level
    /// configured/ignored-interface checks; carrier is intentionally not part
    /// of this predicate.
    pub fn default_dhcp4_candidate(&self) -> bool {
        self.physical()
    }

    /// Return the canonical link name plus aliases used by source Talos
    /// `AllLinkNames` matching for configured/default-DHCP suppression.
    pub fn all_names(&self) -> impl Iterator<Item = &str> {
        core::iter::once(self.name.as_str()).chain(self.aliases.iter().map(String::as_str))
    }

    /// Whether any canonical/alias link name matches `name`.
    pub fn has_name(&self, name: &str) -> bool {
        self.all_names().any(|candidate| candidate == name)
    }

    /// Whether the link is fully usable: admin up, oper up, and carrier present.
    pub fn is_operational(&self) -> bool {
        self.admin_up && self.oper_state == OperState::Up && self.carrier
    }

    /// Render the MAC address in canonical colon-separated lowercase hex.
    pub fn mac_string(&self) -> String {
        let h = &self.hardware_addr;
        alloc::format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            h[0],
            h[1],
            h[2],
            h[3],
            h[4],
            h[5]
        )
    }
}

fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a_2f98,
        0x7137_4491,
        0xb5c0_fbcf,
        0xe9b5_dba5,
        0x3956_c25b,
        0x59f1_11f1,
        0x923f_82a4,
        0xab1c_5ed5,
        0xd807_aa98,
        0x1283_5b01,
        0x2431_85be,
        0x550c_7dc3,
        0x72be_5d74,
        0x80de_b1fe,
        0x9bdc_06a7,
        0xc19b_f174,
        0xe49b_69c1,
        0xefbe_4786,
        0x0fc1_9dc6,
        0x240c_a1cc,
        0x2de9_2c6f,
        0x4a74_84aa,
        0x5cb0_a9dc,
        0x76f9_88da,
        0x983e_5152,
        0xa831_c66d,
        0xb003_27c8,
        0xbf59_7fc7,
        0xc6e0_0bf3,
        0xd5a7_9147,
        0x06ca_6351,
        0x1429_2967,
        0x27b7_0a85,
        0x2e1b_2138,
        0x4d2c_6dfc,
        0x5338_0d13,
        0x650a_7354,
        0x766a_0abb,
        0x81c2_c92e,
        0x9272_2c85,
        0xa2bf_e8a1,
        0xa81a_664b,
        0xc24b_8b70,
        0xc76c_51a3,
        0xd192_e819,
        0xd699_0624,
        0xf40e_3585,
        0x106a_a070,
        0x19a4_c116,
        0x1e37_6c08,
        0x2748_774c,
        0x34b0_bcb5,
        0x391c_0cb3,
        0x4ed8_aa4a,
        0x5b9c_ca4f,
        0x682e_6ff3,
        0x748f_82ee,
        0x78a5_636f,
        0x84c8_7814,
        0x8cc7_0208,
        0x90be_fffa,
        0xa450_6ceb,
        0xbef9_a3f7,
        0xc671_78f2,
    ];

    let mut h = [
        0x6a09_e667u32,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = Vec::with_capacity(((data.len() + 9).div_ceil(64)) * 64);
    msg.extend_from_slice(data);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (idx, word) in w.iter_mut().take(16).enumerate() {
            let offset = idx * 4;
            *word = u32::from_be_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }
        for idx in 16..64 {
            w[idx] = small_sigma1(w[idx - 2])
                .wrapping_add(w[idx - 7])
                .wrapping_add(small_sigma0(w[idx - 15]))
                .wrapping_add(w[idx - 16]);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for idx in 0..64 {
            let t1 = hh
                .wrapping_add(big_sigma1(e))
                .wrapping_add((e & f) ^ ((!e) & g))
                .wrapping_add(K[idx])
                .wrapping_add(w[idx]);
            let t2 = big_sigma0(a).wrapping_add((a & b) ^ (a & c) ^ (b & c));
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (idx, word) in h.iter().enumerate() {
        out[(idx * 4)..(idx * 4) + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

fn big_sigma0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

fn big_sigma1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

fn small_sigma0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

fn small_sigma1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_link_is_valid() {
        let l = LinkSpec::physical("eth0", ConfigLayer::Platform);
        assert!(l.validate().is_ok());
        assert_eq!(l.id(), "eth0");
        assert!(!l.kind.is_virtual());
        assert_eq!(l.kind.kind_str(), "");
    }

    #[test]
    fn link_name_and_mtu_validation() {
        let mut l = LinkSpec::physical("eth0", ConfigLayer::Default);
        l.mtu = 60;
        assert!(l.validate().is_err());
        l.mtu = 9000;
        assert!(l.validate().is_ok());

        let long = LinkSpec::physical("aninterfacenametoolong", ConfigLayer::Default);
        assert!(long.validate().is_err());
    }

    #[test]
    fn vlan_validation() {
        let good = LinkSpec {
            name: vlan_link_name("eth0", 100),
            up: true,
            mtu: 1500,
            multicast: None,
            kind: LinkKind::Vlan {
                parent: String::from("eth0"),
                vlan_id: 100,
                protocol: VlanProtocol::Ieee8021q,
            },
            layer: ConfigLayer::Configuration,
        };
        assert!(good.validate().is_ok());
        assert_eq!(good.id(), "eth0.100");
        assert!(good.kind.is_virtual());

        let bad = LinkSpec {
            kind: LinkKind::Vlan {
                parent: String::from("eth0"),
                vlan_id: 5000,
                protocol: VlanProtocol::Ieee8021q,
            },
            ..good.clone()
        };
        assert!(bad.validate().is_err());
    }

    #[test]
    fn vlan_link_name_matches_talos_source_cases() {
        assert_eq!(vlan_link_name("eth0", 100), "eth0.100");
        assert_eq!(vlan_link_name("enx12545f8c99cd", 25), "enx1ee6413.25");
        assert_eq!(vlan_link_name("enx12545f8c99ce", 4095), "enx1ef972f.4095");
    }

    #[test]
    fn sha256_matches_known_digest() {
        fn hex_digest(input: &[u8]) -> String {
            let mut out = String::new();
            for b in sha256(input) {
                out.push_str(&format!("{b:02x}"));
            }
            out
        }

        assert_eq!(
            hex_digest(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex_digest(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hex_digest(&[b'a'; 55]),
            "9f4390f8d30c2dd92ec9f095b65e2b9ae9b0a925a5258e241c9f1e910f734318"
        );
        assert_eq!(
            hex_digest(&[b'a'; 56]),
            "b35439a4ac6f0948b6d6f9e3c6af0f5f590ce20f1bde7090ef7970686ec6738a"
        );
        assert_eq!(
            hex_digest(&[b'a'; 64]),
            "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb"
        );
        assert_eq!(
            hex_digest(&[b'a'; 100]),
            "2816597888e4a0d3a36b82b83316ab32680eb8f00f8cd3b904d681246d285a0e"
        );
    }

    #[test]
    fn vlan_link_name_hashes_padding_boundary_long_bases() {
        assert_eq!(vlan_link_name(&"x".repeat(55), 7), "xxxxd5e285.7");
        assert_eq!(vlan_link_name(&"x".repeat(56), 7), "xxxx04c262.7");
        assert_eq!(vlan_link_name(&"x".repeat(64), 7), "xxxx7ce100.7");
    }

    #[test]
    fn bond_requires_members() {
        let empty = LinkSpec {
            name: String::from("bond0"),
            up: true,
            mtu: 1500,
            multicast: None,
            kind: LinkKind::Bond {
                members: Vec::new(),
                mode: BondMode::Lacp,
            },
            layer: ConfigLayer::Configuration,
        };
        assert!(empty.validate().is_err());
    }

    #[test]
    fn merge_links_keeps_highest_layer_per_logical_id() {
        let low = LinkSpec::physical("eth0", ConfigLayer::Cmdline);
        let mut high = LinkSpec::physical("eth0", ConfigLayer::Configuration);
        high.mtu = 9000;

        let merged = merge_links(&[low, high]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].id(), "eth0");
        assert_eq!(merged[0].layer, ConfigLayer::Configuration);
        assert_eq!(merged[0].mtu, 9000);
    }

    #[test]
    fn merge_links_preserves_distinct_logical_ids() {
        let eth0 = LinkSpec::physical("eth0", ConfigLayer::Cmdline);
        let eth1 = LinkSpec::physical("eth1", ConfigLayer::Platform);

        let merged = merge_links(&[eth1, eth0]);

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].id(), "eth0");
        assert_eq!(merged[1].id(), "eth1");
        assert_eq!(merged[0].layer, ConfigLayer::Cmdline);
        assert_eq!(merged[1].layer, ConfigLayer::Platform);
    }

    #[test]
    fn link_spec_allows_unspecified_source_mtu_and_multicast() {
        let mut link = LinkSpec::physical("eth0", ConfigLayer::Configuration);
        link.mtu = 0;
        link.multicast = Some(false);
        assert!(link.validate().is_ok());

        link.mtu = 1;
        assert!(link.validate().is_err());
    }

    #[test]
    fn merge_links_keeps_configuration_over_operator() {
        let mut operator = LinkSpec::physical("eth0", ConfigLayer::Operator);
        operator.mtu = 1500;
        let mut configuration = LinkSpec::physical("eth0", ConfigLayer::Configuration);
        configuration.mtu = 9000;
        configuration.multicast = Some(true);

        let merged = merge_links(&[operator, configuration]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].layer, ConfigLayer::Configuration);
        assert_eq!(merged[0].mtu, 9000);
        assert_eq!(merged[0].multicast, Some(true));
    }

    #[test]
    fn merge_links_tie_keeps_existing_spec_without_version_churn() {
        let mut first = LinkSpec::physical("eth0", ConfigLayer::Configuration);
        first.mtu = 1500;
        let mut second = LinkSpec::physical("eth0", ConfigLayer::Configuration);
        second.mtu = 9000;

        let merged = merge_links(&[first.clone(), second]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], first);
    }

    #[test]
    fn link_status_operational_and_mac() {
        let st = LinkStatus {
            name: String::from("eth0"),
            link_type: LinkType::Ether,
            kind: String::new(),
            aliases: Vec::new(),
            admin_up: true,
            oper_state: OperState::Up,
            carrier: true,
            hardware_addr: [0xde, 0xad, 0xbe, 0xef, 0x00, 0x01],
            mtu: 1500,
        };
        assert!(st.is_operational());
        assert!(st.physical());
        assert!(st.default_dhcp4_candidate());
        assert_eq!(st.mac_string(), "de:ad:be:ef:00:01");

        let down = LinkStatus {
            carrier: false,
            ..st
        };
        assert!(!down.is_operational());
    }

    #[test]
    fn default_dhcp_candidate_matches_talos_physical_type_and_kind() {
        let physical = LinkStatus {
            name: String::from("eth0"),
            link_type: LinkType::Ether,
            kind: String::new(),
            aliases: Vec::new(),
            admin_up: false,
            oper_state: OperState::Down,
            carrier: false,
            hardware_addr: [0; 6],
            mtu: 1500,
        };
        assert!(physical.physical());
        assert!(physical.default_dhcp4_candidate());

        let vlan = LinkStatus {
            name: String::from("eth0.100"),
            kind: "vlan".to_string(),
            aliases: Vec::new(),
            ..physical.clone()
        };
        assert!(!vlan.physical());
        assert!(!vlan.default_dhcp4_candidate());

        let loopback = LinkStatus {
            link_type: LinkType::Other("loopback".to_string()),
            kind: String::new(),
            aliases: Vec::new(),
            ..physical
        };
        assert!(!loopback.physical());
        assert!(!loopback.default_dhcp4_candidate());
    }

    #[test]
    fn link_status_all_names_include_aliases_after_canonical_name() {
        let status = LinkStatus {
            name: String::from("eth0"),
            link_type: LinkType::Ether,
            kind: String::new(),
            aliases: Vec::from([String::from("net0"), String::from("enx0001")]),
            admin_up: true,
            oper_state: OperState::Up,
            carrier: true,
            hardware_addr: [0; 6],
            mtu: 1500,
        };

        assert_eq!(
            status.all_names().collect::<Vec<_>>(),
            vec!["eth0", "net0", "enx0001"]
        );
        assert!(status.has_name("net0"));
        assert!(!status.has_name("eth1"));
    }
}
