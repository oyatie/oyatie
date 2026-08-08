//! Real Linux network configuration over `rtnetlink` (and a `/sys` cross-check).
//!
//! This module is the **real** implementation of the machined sequencer network
//! task — it replaces the in-memory [`crate::netlink::InMemoryNetlink`] fake at
//! boot. It talks to the kernel exactly the way upstream Talos's
//! `internal/app/machined/pkg/controllers/network/{link_spec,address_spec}.go`
//! controllers do: by hand-assembling `rtnetlink` messages on an `AF_NETLINK`
//! socket (upstream uses the `jsimonetti/rtnetlink` Go library; here we build the
//! wire format ourselves with `libc` only).
//!
//! ## What this provides
//!
//! - [`set_link_up`]   — bring an interface UP (`RTM_NEWLINK`, `ifinfomsg` with
//!   `IFF_UP` set in both `flags` and `change`).
//! - [`add_ipv4`]      — assign an IPv4 address (`RTM_NEWADDR`, `ifaddrmsg` plus
//!   `IFA_LOCAL` / `IFA_ADDRESS` rtattrs).
//! - [`add_ipv6`]      — assign an IPv6 address using the same `RTM_NEWADDR`
//!   primitive with `AF_INET6` and 16-byte address attributes.
//! - [`query_addrs`]   — read the IPv4 addresses back from the kernel
//!   (`RTM_GETADDR` dump, parsing `ifaddrmsg` + `IFA_ADDRESS`). This is the
//!   verification path.
//! - [`list_link_statuses`] — read observed links from the kernel
//!   (`RTM_GETLINK` dump, parsing `ifinfomsg` + `IFLA_LINKINFO/IFLA_INFO_KIND`).
//! - [`get_operstate`] — read `/sys/class/net/<ifname>/operstate`, a *different*
//!   kernel surface, for cross-confirmation.
//!
//! ## Wire layout (Linux uapi)
//!
//! All structures are little/host-endian, naturally aligned, and padded to
//! 4-byte (`NLMSG_ALIGN` / `RTA_ALIGN`) boundaries. We reproduce them byte for
//! byte. Sizes below are for the LP64 Linux ABI (the only target this module is
//! compiled for).
//!
//! ```text
//! struct nlmsghdr {            // 16 bytes, <linux/netlink.h>
//!     __u32 nlmsg_len;         //  0: total length incl. header + payload + attrs
//!     __u16 nlmsg_type;        //  4: RTM_NEWLINK / RTM_NEWADDR / RTM_GETADDR ...
//!     __u16 nlmsg_flags;       //  6: NLM_F_REQUEST | NLM_F_ACK | NLM_F_CREATE...
//!     __u32 nlmsg_seq;         //  8: sequence number (echoed in the reply)
//!     __u32 nlmsg_pid;         // 12: port id (0 == kernel assigns)
//! };
//!
//! struct ifinfomsg {          // 16 bytes, <linux/rtnetlink.h>
//!     __u8   ifi_family;       //  0: AF_UNSPEC
//!     __u8   __ifi_pad;        //  1: padding
//!     __u16  ifi_type;         //  2: ARPHRD_* (0 on request)
//!     __s32  ifi_index;        //  4: interface index
//!     __u32  ifi_flags;        //  8: IFF_* device flags (e.g. IFF_UP)
//!     __u32  ifi_change;       // 12: change mask (which flags to apply)
//! };
//!
//! struct ifaddrmsg {          // 8 bytes, <linux/if_addr.h>
//!     __u8  ifa_family;        //  0: AF_INET / AF_INET6
//!     __u8  ifa_prefixlen;     //  1: e.g. 24
//!     __u8  ifa_flags;         //  2: IFA_F_* (0 here)
//!     __u8  ifa_scope;         //  3: RT_SCOPE_UNIVERSE (0)
//!     __u32 ifa_index;         //  4: interface index
//! };
//!
//! struct rtattr {             // 4-byte header, <linux/rtnetlink.h>
//!     __u16 rta_len;           //  0: length of header + payload (NOT padded)
//!     __u16 rta_type;          //  2: IFA_LOCAL / IFA_ADDRESS / ...
//!     // payload follows, padded to RTA_ALIGN (4)
//! };
//! ```
//!
//! Only the *pure* builders/parsers (everything that does not touch a socket)
//! are compiled on the host so they can be unit-tested; the syscall paths are
//! gated to `target_os = "linux"` and exercised for real at boot.

use crate::{LinkStatus, LinkType, OperState};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::net::Ipv6Addr;
use os_kernel::error::{Error, Result};

// ---------------------------------------------------------------------------
// uapi constants (from <linux/netlink.h>, <linux/rtnetlink.h>, <linux/if_addr.h>,
// <linux/if.h>). Re-declared here so the pure builders compile on any host.
// ---------------------------------------------------------------------------

/// `NETLINK_ROUTE` protocol number for `socket(AF_NETLINK, SOCK_RAW, ...)`.
pub const NETLINK_ROUTE: i32 = 0;

/// `RTM_NEWLINK` — create/modify a link.
pub const RTM_NEWLINK: u16 = 16;
/// `RTM_GETLINK` — dump/query links.
pub const RTM_GETLINK: u16 = 18;
/// `RTM_NEWADDR` — add an address.
pub const RTM_NEWADDR: u16 = 20;
/// `RTM_GETADDR` — dump addresses.
pub const RTM_GETADDR: u16 = 22;
/// `RTM_NEWROUTE` — add a route.
pub const RTM_NEWROUTE: u16 = 24;
/// `NLMSG_ERROR` — error / ack message type.
pub const NLMSG_ERROR: u16 = 2;
/// `NLMSG_DONE` — end of a multipart dump.
pub const NLMSG_DONE: u16 = 3;

/// `NLM_F_REQUEST` — this is a request.
pub const NLM_F_REQUEST: u16 = 0x01;
/// `NLM_F_ACK` — request an ack/error reply.
pub const NLM_F_ACK: u16 = 0x04;
/// `NLM_F_EXCL` — fail if it already exists (with CREATE).
pub const NLM_F_EXCL: u16 = 0x200;
/// `NLM_F_CREATE` — create if it does not exist.
pub const NLM_F_CREATE: u16 = 0x400;
/// `NLM_F_DUMP` — return a list (ROOT | MATCH).
pub const NLM_F_DUMP: u16 = 0x100 | 0x200;

/// `IFA_ADDRESS` rtattr type (interface address / peer).
pub const IFA_ADDRESS: u16 = 1;
/// `IFA_LOCAL` rtattr type (local address).
pub const IFA_LOCAL: u16 = 2;
/// `RTA_DST` rtattr type (route destination prefix).
pub const RTA_DST: u16 = 1;
/// `RTA_OIF` rtattr type (route output interface index).
pub const RTA_OIF: u16 = 4;
/// `RTA_GATEWAY` rtattr type (route next hop).
pub const RTA_GATEWAY: u16 = 5;
/// `RTA_PRIORITY` rtattr type (route metric).
pub const RTA_PRIORITY: u16 = 6;

/// `IFLA_ADDRESS` link rtattr type (hardware address).
pub const IFLA_ADDRESS: u16 = 1;
/// `IFLA_IFNAME` link rtattr type (interface name).
pub const IFLA_IFNAME: u16 = 3;
/// `IFLA_MTU` link rtattr type.
pub const IFLA_MTU: u16 = 4;
/// `IFLA_OPERSTATE` link rtattr type.
pub const IFLA_OPERSTATE: u16 = 16;
/// `IFLA_LINKINFO` nested link rtattr type.
pub const IFLA_LINKINFO: u16 = 18;
/// `IFLA_IFALIAS` link rtattr type.
pub const IFLA_IFALIAS: u16 = 20;
/// `IFLA_CARRIER` link rtattr type.
pub const IFLA_CARRIER: u16 = 33;
/// `IFLA_PROP_LIST` nested link rtattr type.
pub const IFLA_PROP_LIST: u16 = 52;
/// `IFLA_ALT_IFNAME` link rtattr type.
pub const IFLA_ALT_IFNAME: u16 = 53;
/// `IFLA_INFO_KIND` nested `IFLA_LINKINFO` rtattr type.
pub const IFLA_INFO_KIND: u16 = 1;

/// Mask for stripping `NLA_F_NESTED`/`NLA_F_NET_BYTEORDER` from attr types.
pub const NLA_TYPE_MASK: u16 = 0x3fff;

/// `IFF_UP` device flag.
pub const IFF_UP: u32 = 0x1;
/// `IFF_RUNNING` device flag.
pub const IFF_RUNNING: u32 = 0x40;

/// `AF_UNSPEC` address family.
pub const AF_UNSPEC: u8 = 0;
/// `AF_INET` address family.
pub const AF_INET: u8 = 2;
/// `AF_INET6` address family.
pub const AF_INET6: u8 = 10;

/// Header size of `struct nlmsghdr`.
pub const NLMSGHDR_LEN: usize = 16;
/// Size of `struct ifinfomsg`.
pub const IFINFOMSG_LEN: usize = 16;
/// Size of `struct ifaddrmsg`.
pub const IFADDRMSG_LEN: usize = 8;
/// Size of `struct rtmsg`.
pub const RTMSG_LEN: usize = 12;
/// Header size of `struct rtattr`.
pub const RTA_HDR_LEN: usize = 4;

/// `NLMSG_ALIGNTO` / `RTA_ALIGNTO` — both are 4.
const ALIGN_TO: usize = 4;

/// Round `len` up to the netlink 4-byte alignment (`NLMSG_ALIGN`/`RTA_ALIGN`).
#[inline]
pub fn nl_align(len: usize) -> usize {
    (len + ALIGN_TO - 1) & !(ALIGN_TO - 1)
}

// ---------------------------------------------------------------------------
// Pure helpers: address / netmask math.
// ---------------------------------------------------------------------------

/// Convert an IPv4 prefix length (`0..=32`) to a big-endian netmask `[u8; 4]`.
///
/// e.g. `24 -> 255.255.255.0`, `0 -> 0.0.0.0`, `32 -> 255.255.255.255`.
pub fn prefix_to_netmask(prefix_len: u8) -> Result<[u8; 4]> {
    if prefix_len > 32 {
        return Err(Error::invalid(format!(
            "ipv4 prefix length {prefix_len} out of range 0..=32"
        )));
    }
    // Build the mask as a u32 then split into network-order bytes. Shifting by
    // 32 is UB, so special-case the full / empty masks.
    let mask: u32 = if prefix_len == 0 {
        0
    } else {
        u32::MAX << (32 - prefix_len)
    };
    Ok(mask.to_be_bytes())
}

/// Parse a dotted-quad IPv4 string into 4 octets.
pub fn parse_ipv4(addr: &str) -> Result<[u8; 4]> {
    let mut octets = [0u8; 4];
    let mut count = 0usize;
    for part in addr.split('.') {
        if count >= 4 {
            return Err(Error::parse(format!("invalid IPv4 address '{addr}'")));
        }
        let v: u32 = part
            .parse()
            .map_err(|_| Error::parse(format!("invalid IPv4 octet '{part}' in '{addr}'")))?;
        if v > 255 || part.is_empty() {
            return Err(Error::parse(format!(
                "invalid IPv4 octet '{part}' in '{addr}'"
            )));
        }
        octets[count] = v as u8;
        count += 1;
    }
    if count != 4 {
        return Err(Error::parse(format!("invalid IPv4 address '{addr}'")));
    }
    Ok(octets)
}

/// Render 4 octets as a dotted-quad string.
pub fn format_ipv4(octets: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])
}

/// Parse an IPv6 address string into 16 network-order octets.
pub fn parse_ipv6(addr: &str) -> Result<[u8; 16]> {
    addr.parse::<Ipv6Addr>()
        .map(|addr| addr.octets())
        .map_err(|_| Error::parse(format!("invalid IPv6 address '{addr}'")))
}

/// Render 16 network-order octets as a canonical IPv6 string.
pub fn format_ipv6(octets: [u8; 16]) -> String {
    Ipv6Addr::from(octets).to_string()
}

// ---------------------------------------------------------------------------
// Pure builders: hand-rolled netlink message serialization.
// ---------------------------------------------------------------------------

/// Append a `struct nlmsghdr` to `buf` with a placeholder length (patched by
/// [`patch_nlmsg_len`] once the body is complete).
fn push_nlmsghdr(buf: &mut Vec<u8>, msg_type: u16, flags: u16, seq: u32) {
    buf.extend_from_slice(&0u32.to_ne_bytes()); // nlmsg_len (placeholder)
    buf.extend_from_slice(&msg_type.to_ne_bytes()); // nlmsg_type
    buf.extend_from_slice(&flags.to_ne_bytes()); // nlmsg_flags
    buf.extend_from_slice(&seq.to_ne_bytes()); // nlmsg_seq
    buf.extend_from_slice(&0u32.to_ne_bytes()); // nlmsg_pid (kernel fills)
}

/// Patch the `nlmsg_len` field (first 4 bytes) to the buffer's final length.
fn patch_nlmsg_len(buf: &mut [u8]) {
    let len = buf.len() as u32;
    buf[0..4].copy_from_slice(&len.to_ne_bytes());
}

/// Append a `struct ifinfomsg`.
fn push_ifinfomsg(buf: &mut Vec<u8>, family: u8, index: i32, flags: u32, change: u32) {
    buf.push(family); // ifi_family
    buf.push(0); // __ifi_pad
    buf.extend_from_slice(&0u16.to_ne_bytes()); // ifi_type
    buf.extend_from_slice(&index.to_ne_bytes()); // ifi_index
    buf.extend_from_slice(&flags.to_ne_bytes()); // ifi_flags
    buf.extend_from_slice(&change.to_ne_bytes()); // ifi_change
}

/// Append a `struct ifaddrmsg`.
fn push_ifaddrmsg(buf: &mut Vec<u8>, family: u8, prefix_len: u8, scope: u8, index: u32) {
    buf.push(family); // ifa_family
    buf.push(prefix_len); // ifa_prefixlen
    buf.push(0); // ifa_flags
    buf.push(scope); // ifa_scope
    buf.extend_from_slice(&index.to_ne_bytes()); // ifa_index
}

/// Append a `struct rtmsg`.
fn push_rtmsg(
    buf: &mut Vec<u8>,
    family: u8,
    dst_len: u8,
    table: u8,
    protocol: u8,
    scope: u8,
    route_type: u8,
) {
    buf.push(family); // rtm_family
    buf.push(dst_len); // rtm_dst_len
    buf.push(0); // rtm_src_len
    buf.push(0); // rtm_tos
    buf.push(table); // rtm_table
    buf.push(protocol); // rtm_protocol
    buf.push(scope); // rtm_scope
    buf.push(route_type); // rtm_type
    buf.extend_from_slice(&0u32.to_ne_bytes()); // rtm_flags
}

/// Append one `struct rtattr` (header + payload) padded to `RTA_ALIGN`.
///
/// `rta_len` is the **unpadded** length of header + payload (the kernel uses it
/// to find the payload); the trailing pad bytes bring the cursor to the next
/// 4-byte boundary but are not counted in `rta_len`.
pub fn push_rtattr(buf: &mut Vec<u8>, rta_type: u16, payload: &[u8]) {
    let rta_len = RTA_HDR_LEN + payload.len();
    buf.extend_from_slice(&(rta_len as u16).to_ne_bytes()); // rta_len
    buf.extend_from_slice(&rta_type.to_ne_bytes()); // rta_type
    buf.extend_from_slice(payload); // payload
    let pad = nl_align(rta_len) - rta_len;
    for _ in 0..pad {
        buf.push(0);
    }
}

/// Build a complete `RTM_NEWLINK` request that sets `IFF_UP` on `index`.
///
/// Mirrors what `rtnetlink`'s `LinkSet`/`link_spec.go` issues: an `ifinfomsg`
/// with `ifi_flags = IFF_UP|IFF_RUNNING` and `ifi_change = IFF_UP|IFF_RUNNING`
/// so only those flags are touched. No rtattrs are needed to flip admin state.
pub fn build_set_link_up(index: i32, seq: u32) -> Vec<u8> {
    let mut buf = Vec::with_capacity(NLMSGHDR_LEN + IFINFOMSG_LEN);
    push_nlmsghdr(&mut buf, RTM_NEWLINK, NLM_F_REQUEST | NLM_F_ACK, seq);
    let flags = IFF_UP | IFF_RUNNING;
    push_ifinfomsg(&mut buf, AF_UNSPEC, index, flags, flags);
    patch_nlmsg_len(&mut buf);
    buf
}

/// Build a complete `RTM_NEWADDR` request assigning `addr/prefix_len` to
/// `index`. Includes both `IFA_LOCAL` and `IFA_ADDRESS` (equal for a non-PPP
/// link), exactly as the kernel and `rtnetlink` expect for IPv4.
pub fn build_add_ipv4(index: u32, addr: [u8; 4], prefix_len: u8, seq: u32) -> Vec<u8> {
    let mut buf = Vec::with_capacity(NLMSGHDR_LEN + IFADDRMSG_LEN + 2 * (RTA_HDR_LEN + 4));
    push_nlmsghdr(
        &mut buf,
        RTM_NEWADDR,
        NLM_F_REQUEST | NLM_F_ACK | NLM_F_CREATE | NLM_F_EXCL,
        seq,
    );
    // RT_SCOPE_UNIVERSE == 0.
    push_ifaddrmsg(&mut buf, AF_INET, prefix_len, 0, index);
    push_rtattr(&mut buf, IFA_LOCAL, &addr);
    push_rtattr(&mut buf, IFA_ADDRESS, &addr);
    patch_nlmsg_len(&mut buf);
    buf
}

/// Build a complete `RTM_NEWADDR` request assigning IPv6 `addr/prefix_len` to
/// `index`. Includes both `IFA_LOCAL` and `IFA_ADDRESS` with the same 16-byte
/// address, matching the IPv4 primitive and ordinary non-PPP links.
pub fn build_add_ipv6(index: u32, addr: [u8; 16], prefix_len: u8, seq: u32) -> Vec<u8> {
    let mut buf = Vec::with_capacity(NLMSGHDR_LEN + IFADDRMSG_LEN + 2 * (RTA_HDR_LEN + 16));
    push_nlmsghdr(
        &mut buf,
        RTM_NEWADDR,
        NLM_F_REQUEST | NLM_F_ACK | NLM_F_CREATE | NLM_F_EXCL,
        seq,
    );
    // RT_SCOPE_UNIVERSE == 0.
    push_ifaddrmsg(&mut buf, AF_INET6, prefix_len, 0, index);
    push_rtattr(&mut buf, IFA_LOCAL, &addr);
    push_rtattr(&mut buf, IFA_ADDRESS, &addr);
    patch_nlmsg_len(&mut buf);
    buf
}

/// Build a complete `RTM_NEWROUTE` request for an IPv4 route.
pub fn build_add_ipv4_route(
    index: u32,
    destination: Option<[u8; 4]>,
    prefix_len: u8,
    gateway: Option<[u8; 4]>,
    metric: u32,
    protocol: u8,
    seq: u32,
) -> Vec<u8> {
    let mut buf = Vec::with_capacity(NLMSGHDR_LEN + RTMSG_LEN + 32);
    push_nlmsghdr(
        &mut buf,
        RTM_NEWROUTE,
        NLM_F_REQUEST | NLM_F_ACK | NLM_F_CREATE | NLM_F_EXCL,
        seq,
    );
    // RT_TABLE_MAIN=254, RT_SCOPE_UNIVERSE=0, RT_SCOPE_LINK=253, RTN_UNICAST=1.
    let scope = if gateway.is_some() { 0 } else { 253 };
    push_rtmsg(&mut buf, AF_INET, prefix_len, 254, protocol, scope, 1);
    if let Some(destination) = destination {
        push_rtattr(&mut buf, RTA_DST, &destination);
    }
    if let Some(gateway) = gateway {
        push_rtattr(&mut buf, RTA_GATEWAY, &gateway);
    }
    push_rtattr(&mut buf, RTA_OIF, &index.to_ne_bytes());
    push_rtattr(&mut buf, RTA_PRIORITY, &metric.to_ne_bytes());
    patch_nlmsg_len(&mut buf);
    buf
}

/// Build an `RTM_GETADDR` dump request for the whole address table.
///
/// The reply is multipart; the caller filters by `ifa_index` while parsing.
pub fn build_dump_addrs(seq: u32) -> Vec<u8> {
    let mut buf = Vec::with_capacity(NLMSGHDR_LEN + IFADDRMSG_LEN);
    push_nlmsghdr(&mut buf, RTM_GETADDR, NLM_F_REQUEST | NLM_F_DUMP, seq);
    // ifaddrmsg with family AF_INET to scope the dump to IPv4.
    push_ifaddrmsg(&mut buf, AF_INET, 0, 0, 0);
    patch_nlmsg_len(&mut buf);
    buf
}

/// Build an `RTM_GETLINK` dump request for the whole link table.
///
/// The reply is multipart; each `RTM_NEWLINK` body carries an `ifinfomsg` plus
/// link rtattrs such as `IFLA_IFNAME`, `IFLA_MTU`, `IFLA_ADDRESS`,
/// `IFLA_OPERSTATE`, `IFLA_CARRIER`, and nested
/// `IFLA_LINKINFO/IFLA_INFO_KIND`.
pub fn build_dump_links(seq: u32) -> Vec<u8> {
    let mut buf = Vec::with_capacity(NLMSGHDR_LEN + IFINFOMSG_LEN);
    push_nlmsghdr(&mut buf, RTM_GETLINK, NLM_F_REQUEST | NLM_F_DUMP, seq);
    push_ifinfomsg(&mut buf, AF_UNSPEC, 0, 0, 0);
    patch_nlmsg_len(&mut buf);
    buf
}

// ---------------------------------------------------------------------------
// Pure parsers: walk a netlink dump reply.
// ---------------------------------------------------------------------------

/// One IPv4 address parsed out of an `RTM_NEWADDR` dump entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAddr {
    /// Interface index the address belongs to.
    pub index: u32,
    /// The address octets.
    pub addr: [u8; 4],
    /// The prefix length.
    pub prefix_len: u8,
}

fn parse_rtattrs(attrs: &[u8], mut visit: impl FnMut(u16, &[u8]) -> Result<()>) -> Result<()> {
    let mut off = 0usize;
    while off + RTA_HDR_LEN <= attrs.len() {
        let rta_len = read_u16(attrs, off)
            .ok_or_else(|| Error::parse("truncated rtnetlink attribute length"))?
            as usize;
        let rta_type = read_u16(attrs, off + 2)
            .ok_or_else(|| Error::parse("truncated rtnetlink attribute type"))?
            & NLA_TYPE_MASK;
        if rta_len < RTA_HDR_LEN || off + rta_len > attrs.len() {
            return Err(Error::parse(format!(
                "malformed rtnetlink attribute: len {rta_len} at offset {off} in {} bytes",
                attrs.len()
            )));
        }
        visit(rta_type, &attrs[off + RTA_HDR_LEN..off + rta_len])?;
        off += nl_align(rta_len);
    }
    if off < attrs.len() && attrs[off..].iter().any(|&byte| byte != 0) {
        return Err(Error::parse(format!(
            "malformed rtnetlink attribute tail: {} trailing bytes",
            attrs.len() - off
        )));
    }
    Ok(())
}

fn parse_attr_string(payload: &[u8], field: &str) -> Result<String> {
    let end = payload
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(payload.len());
    let value = core::str::from_utf8(&payload[..end])
        .map_err(|_| Error::parse(format!("invalid UTF-8 in {field}")))?
        .trim();
    if value.is_empty() {
        return Err(Error::parse(format!("empty {field}")));
    }
    Ok(value.to_string())
}

fn push_unique_alias(aliases: &mut Vec<String>, alias: String) {
    if !aliases.iter().any(|existing| existing == &alias) {
        aliases.push(alias);
    }
}

fn parse_alt_ifname(payload: &[u8]) -> Result<String> {
    let alias = parse_attr_string(payload, "IFLA_ALT_IFNAME")?;
    validate_iface_name(&alias)?;
    if alias.contains([',', ';', '=']) {
        return Err(Error::invalid(format!(
            "invalid alternate interface name '{alias}'"
        )));
    }
    Ok(alias)
}

fn parse_linkinfo_kind(payload: &[u8]) -> Result<String> {
    let mut kind = String::new();
    parse_rtattrs(payload, |rta_type, payload| {
        if rta_type == IFLA_INFO_KIND {
            kind = parse_attr_string(payload, "IFLA_INFO_KIND")?;
        }
        Ok(())
    })?;
    Ok(kind)
}

fn parse_prop_list_aliases(payload: &[u8], aliases: &mut Vec<String>) -> Result<()> {
    parse_rtattrs(payload, |rta_type, payload| {
        if rta_type == IFLA_ALT_IFNAME {
            push_unique_alias(aliases, parse_alt_ifname(payload)?);
        }
        Ok(())
    })
}

fn parse_oper_state(value: u8) -> OperState {
    match value {
        // IF_OPER_UP
        6 => OperState::Up,
        // IF_OPER_DOWN / IF_OPER_LOWERLAYERDOWN
        2 | 3 => OperState::Down,
        _ => OperState::Unknown,
    }
}

fn parse_link_type(value: u16) -> LinkType {
    match value {
        // ARPHRD_ETHER
        1 => LinkType::Ether,
        other => LinkType::Other(other.to_string()),
    }
}

/// Parse the rtattrs of a single `RTM_NEWLINK` body into a [`LinkStatus`].
///
/// This parser deliberately takes `kind` from nested
/// `IFLA_LINKINFO/IFLA_INFO_KIND`, matching rtnetlink's link kind surface for
/// virtual devices. Sysfs `DEVTYPE` is not used for Talos physical-link
/// classification because it is a different uevent field, not rtnetlink kind.
pub fn parse_link_message(body: &[u8]) -> Result<LinkStatus> {
    if body.len() < IFINFOMSG_LEN {
        return Err(Error::parse("short RTM_NEWLINK ifinfomsg"));
    }
    let ifi_type = read_u16(body, 2).ok_or_else(|| Error::parse("missing ifi_type"))?;
    let flags = read_u32(body, 8).ok_or_else(|| Error::parse("missing ifi_flags"))?;

    let mut name: Option<String> = None;
    let mut kind = String::new();
    let mut aliases = Vec::new();
    let mut hardware_addr: Option<[u8; 6]> = None;
    let mut mtu: Option<u32> = None;
    let mut oper_state = OperState::Unknown;
    let mut carrier: Option<bool> = None;

    parse_rtattrs(&body[IFINFOMSG_LEN..], |rta_type, payload| {
        match rta_type {
            IFLA_IFNAME => {
                name = Some(parse_attr_string(payload, "IFLA_IFNAME")?);
            }
            IFLA_LINKINFO => {
                kind = parse_linkinfo_kind(payload)?;
            }
            // IFLA_IFALIAS is a free-form user description, not a kernel alternate
            // interface name. Only IFLA_ALT_IFNAME participates in Talos name
            // matching so descriptive aliases cannot suppress DHCP/link defaults.
            IFLA_IFALIAS => {}
            IFLA_ALT_IFNAME => {
                push_unique_alias(&mut aliases, parse_alt_ifname(payload)?);
            }
            IFLA_PROP_LIST => {
                parse_prop_list_aliases(payload, &mut aliases)?;
            }
            IFLA_ADDRESS => {
                if payload.len() < 6 {
                    return Err(Error::parse("short IFLA_ADDRESS hardware address"));
                }
                hardware_addr = Some([
                    payload[0], payload[1], payload[2], payload[3], payload[4], payload[5],
                ]);
            }
            IFLA_MTU => {
                let value = payload
                    .get(..4)
                    .map(|bytes| u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
                    .ok_or_else(|| Error::parse("short IFLA_MTU value"))?;
                mtu = Some(value);
            }
            IFLA_OPERSTATE => {
                let value = payload
                    .first()
                    .copied()
                    .ok_or_else(|| Error::parse("short IFLA_OPERSTATE value"))?;
                oper_state = parse_oper_state(value);
            }
            IFLA_CARRIER => {
                let value = payload
                    .first()
                    .copied()
                    .ok_or_else(|| Error::parse("short IFLA_CARRIER value"))?;
                carrier = Some(value != 0);
            }
            _ => {}
        }
        Ok(())
    })?;

    let name = name.ok_or_else(|| Error::parse("RTM_NEWLINK missing IFLA_IFNAME"))?;
    validate_iface_name(&name)?;
    let mtu = mtu.ok_or_else(|| Error::parse(format!("RTM_NEWLINK {name} missing IFLA_MTU")))?;
    let link_type = parse_link_type(ifi_type);
    let hardware_addr = match (matches!(link_type, LinkType::Ether), hardware_addr) {
        (true, Some(mac)) | (false, Some(mac)) => mac,
        (true, None) => {
            return Err(Error::parse(format!(
                "RTM_NEWLINK {name} ethernet missing IFLA_ADDRESS"
            )));
        }
        (false, None) => [0; 6],
    };

    Ok(LinkStatus {
        name,
        link_type,
        kind,
        aliases,
        admin_up: flags & IFF_UP != 0,
        oper_state,
        carrier: carrier.unwrap_or(flags & IFF_RUNNING != 0),
        hardware_addr,
        mtu,
    })
}

/// Read a host-endian `u32` from `buf[off..off+4]`, if in range.
fn read_u32(buf: &[u8], off: usize) -> Option<u32> {
    buf.get(off..off + 4)
        .map(|b| u32::from_ne_bytes([b[0], b[1], b[2], b[3]]))
}

/// Read a host-endian `u16` from `buf[off..off+2]`, if in range.
fn read_u16(buf: &[u8], off: usize) -> Option<u16> {
    buf.get(off..off + 2)
        .map(|b| u16::from_ne_bytes([b[0], b[1]]))
}

/// Parse the rtattrs of a single `RTM_NEWADDR` message body into a [`ParsedAddr`].
///
/// `body` is the message payload *after* the `nlmsghdr` (i.e. starts at the
/// `ifaddrmsg`). Returns `None` if it is not an IPv4 entry or is malformed.
pub fn parse_addr_message(body: &[u8]) -> Option<ParsedAddr> {
    if body.len() < IFADDRMSG_LEN {
        return None;
    }
    let family = body[0];
    if family != AF_INET {
        return None;
    }
    let prefix_len = body[1];
    let index = read_u32(body, 4)?;

    // Walk rtattrs after the ifaddrmsg. Prefer IFA_ADDRESS, fall back to
    // IFA_LOCAL (they are equal for ordinary links).
    let mut addr: Option<[u8; 4]> = None;
    let mut local: Option<[u8; 4]> = None;
    let mut off = IFADDRMSG_LEN;
    while off + RTA_HDR_LEN <= body.len() {
        let rta_len = read_u16(body, off)? as usize;
        let rta_type = read_u16(body, off + 2)?;
        if rta_len < RTA_HDR_LEN || off + rta_len > body.len() {
            break;
        }
        let payload = &body[off + RTA_HDR_LEN..off + rta_len];
        if payload.len() >= 4 {
            let octets = [payload[0], payload[1], payload[2], payload[3]];
            match rta_type {
                IFA_ADDRESS => addr = Some(octets),
                IFA_LOCAL => local = Some(octets),
                _ => {}
            }
        }
        off += nl_align(rta_len);
    }

    let chosen = addr.or(local)?;
    Some(ParsedAddr {
        index,
        addr: chosen,
        prefix_len,
    })
}

/// Walk a (possibly multipart) `RTM_GETADDR` dump buffer, returning every IPv4
/// [`ParsedAddr`] found. Stops at `NLMSG_DONE`; surfaces `NLMSG_ERROR` as an
/// error. This is the pure core of [`query_addrs`].
pub fn parse_addr_dump(buf: &[u8]) -> Result<Vec<ParsedAddr>> {
    let mut out = Vec::new();
    let mut off = 0usize;
    while off + NLMSGHDR_LEN <= buf.len() {
        let nlmsg_len = read_u32(buf, off)
            .ok_or_else(|| Error::Other("truncated nlmsghdr".to_string()))?
            as usize;
        let nlmsg_type =
            read_u16(buf, off + 4).ok_or_else(|| Error::Other("truncated nlmsghdr".to_string()))?;
        if nlmsg_len < NLMSGHDR_LEN || off + nlmsg_len > buf.len() {
            break;
        }
        match nlmsg_type {
            NLMSG_DONE => break,
            NLMSG_ERROR => {
                // Body starts with an i32 errno (negative on error).
                let errno = read_u32(buf, off + NLMSGHDR_LEN)
                    .map(|v| v as i32)
                    .unwrap_or(0);
                if errno != 0 {
                    return Err(Error::Other(format!(
                        "netlink dump error: errno {}",
                        -errno
                    )));
                }
            }
            RTM_NEWADDR => {
                let body = &buf[off + NLMSGHDR_LEN..off + nlmsg_len];
                if let Some(p) = parse_addr_message(body) {
                    out.push(p);
                }
            }
            _ => {}
        }
        off += nl_align(nlmsg_len);
    }
    Ok(out)
}

/// Walk a (possibly multipart) `RTM_GETLINK` dump buffer, returning every
/// [`LinkStatus`] found. Stops at `NLMSG_DONE`; surfaces `NLMSG_ERROR` and
/// malformed `RTM_NEWLINK` bodies as errors instead of silently dropping an
/// interface.
pub fn parse_link_dump(buf: &[u8]) -> Result<Vec<LinkStatus>> {
    let mut out = Vec::new();
    let mut off = 0usize;
    while off + NLMSGHDR_LEN <= buf.len() {
        let nlmsg_len = read_u32(buf, off)
            .ok_or_else(|| Error::Other("truncated nlmsghdr".to_string()))?
            as usize;
        let nlmsg_type =
            read_u16(buf, off + 4).ok_or_else(|| Error::Other("truncated nlmsghdr".to_string()))?;
        if nlmsg_len < NLMSGHDR_LEN || off + nlmsg_len > buf.len() {
            return Err(Error::parse(format!(
                "malformed netlink message: len {nlmsg_len} at offset {off} in {} bytes",
                buf.len()
            )));
        }
        match nlmsg_type {
            NLMSG_DONE => break,
            NLMSG_ERROR => {
                let errno = read_u32(buf, off + NLMSGHDR_LEN)
                    .map(|v| v as i32)
                    .unwrap_or(0);
                if errno != 0 {
                    return Err(Error::Other(format!(
                        "netlink dump error: errno {}",
                        -errno
                    )));
                }
            }
            RTM_NEWLINK => {
                let body = &buf[off + NLMSGHDR_LEN..off + nlmsg_len];
                out.push(parse_link_message(body)?);
            }
            _ => {}
        }
        off += nl_align(nlmsg_len);
    }
    Ok(out)
}

/// Inspect one rtnetlink dump datagram before the receive loop waits for more.
///
/// A dump normally terminates with `NLMSG_DONE`, but the kernel may instead
/// return a standalone `NLMSG_ERROR`. Treating that as "not done" can block the
/// boot controller path forever, so live receive loops fail immediately on dump
/// errors while still accumulating ordinary multipart chunks.
pub fn dump_chunk_done_or_error(buf: &[u8]) -> Result<bool> {
    let mut off = 0usize;
    while off + NLMSGHDR_LEN <= buf.len() {
        let nlmsg_len = read_u32(buf, off)
            .ok_or_else(|| Error::Other("truncated nlmsghdr".to_string()))?
            as usize;
        let nlmsg_type =
            read_u16(buf, off + 4).ok_or_else(|| Error::Other("truncated nlmsghdr".to_string()))?;
        if nlmsg_len < NLMSGHDR_LEN || off + nlmsg_len > buf.len() {
            return Err(Error::parse(format!(
                "malformed netlink message: len {nlmsg_len} at offset {off} in {} bytes",
                buf.len()
            )));
        }
        match nlmsg_type {
            NLMSG_DONE => return Ok(true),
            NLMSG_ERROR => {
                let errno = read_u32(buf, off + NLMSGHDR_LEN)
                    .map(|v| v as i32)
                    .unwrap_or(0);
                if errno != 0 {
                    return Err(Error::Other(format!(
                        "netlink dump error: errno {}",
                        -errno
                    )));
                }
            }
            _ => {}
        }
        off += nl_align(nlmsg_len);
    }
    Ok(false)
}

// ---------------------------------------------------------------------------
// Pure sysfs/procfs parsers retained for cross-check fixtures.
// ---------------------------------------------------------------------------

/// Parse interface names from `/proc/net/dev`.
///
/// This is used by the Linux status producer to avoid a dependency on `std::fs`
/// directory iteration in the boot binary. The first two header lines are
/// ignored; every later line is `<ifname>: counters...`.
pub fn parse_proc_net_dev_ifaces(contents: &str) -> Vec<String> {
    contents
        .lines()
        .skip(2)
        .filter_map(|line| {
            let (name, _) = line.split_once(':')?;
            let name = name.trim();
            (!name.is_empty()).then(|| name.to_string())
        })
        .collect()
}

fn validate_iface_name(name: &str) -> Result<()> {
    if name.is_empty() || name.contains('/') || name.contains("..") || name.bytes().any(|b| b == 0)
    {
        return Err(Error::invalid(format!("invalid interface name '{name}'")));
    }
    Ok(())
}

fn parse_sysfs_u32(value: &str, field: &str) -> Result<u32> {
    value
        .trim()
        .parse::<u32>()
        .map_err(|_| Error::parse(format!("invalid {field} value '{}'", value.trim())))
}

fn parse_sysfs_flags(value: &str) -> Result<u32> {
    let value = value.trim();
    let hex = value.strip_prefix("0x").unwrap_or(value);
    u32::from_str_radix(hex, 16).map_err(|_| Error::parse(format!("invalid flags value '{value}'")))
}

fn parse_sysfs_oper_state(value: &str) -> OperState {
    match value.trim() {
        "up" => OperState::Up,
        "down" => OperState::Down,
        _ => OperState::Unknown,
    }
}

fn parse_sysfs_carrier(value: &str) -> bool {
    value.trim() == "1" || value.trim().eq_ignore_ascii_case("true")
}

fn parse_sysfs_mac(value: &str) -> Result<[u8; 6]> {
    let mut out = [0u8; 6];
    let mut count = 0usize;
    for part in value.trim().split(':') {
        if count >= 6 || part.len() != 2 {
            return Err(Error::parse(format!(
                "invalid MAC address '{}'",
                value.trim()
            )));
        }
        out[count] = u8::from_str_radix(part, 16)
            .map_err(|_| Error::parse(format!("invalid MAC address '{}'", value.trim())))?;
        count += 1;
    }
    if count != 6 {
        return Err(Error::parse(format!(
            "invalid MAC address '{}'",
            value.trim()
        )));
    }
    Ok(out)
}

fn sysfs_link_type(sysfs_type: &str) -> Result<LinkType> {
    match parse_sysfs_u32(sysfs_type, "type")? {
        1 => Ok(LinkType::Ether),
        other => Ok(LinkType::Other(other.to_string())),
    }
}

/// Build a source-shaped [`LinkStatus`] from explicit Linux field fixtures.
///
/// Required inputs correspond to the same value domains exposed by Linux:
/// `type`, already-decoded rtnetlink `kind`, `flags`, `operstate`, `carrier`,
/// `address`, and `mtu`. Production code obtains `kind` from
/// `IFLA_LINKINFO/IFLA_INFO_KIND`; these fixtures keep the older sysfs-shaped
/// tests host-compilable without treating sysfs `DEVTYPE` as rtnetlink kind.
#[allow(clippy::too_many_arguments)] // arity mirrors the Linux sysfs/rtnetlink field set
pub fn link_status_from_sysfs_fields(
    name: &str,
    sysfs_type: &str,
    link_kind: &str,
    flags: &str,
    operstate: &str,
    carrier: &str,
    address: &str,
    mtu: &str,
) -> Result<LinkStatus> {
    validate_iface_name(name)?;
    let flags = parse_sysfs_flags(flags)?;
    Ok(LinkStatus {
        name: name.to_string(),
        link_type: sysfs_link_type(sysfs_type)?,
        kind: link_kind.trim().to_string(),
        aliases: Vec::new(),
        admin_up: flags & IFF_UP != 0,
        oper_state: parse_sysfs_oper_state(operstate),
        carrier: parse_sysfs_carrier(carrier),
        hardware_addr: parse_sysfs_mac(address)?,
        mtu: parse_sysfs_u32(mtu, "mtu")?,
    })
}

// ---------------------------------------------------------------------------
// The real syscall paths (Linux only). Compiled out on non-Linux hosts so the
// pure code above still builds and tests there; exercised for real at boot.
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
pub use linux_impl::{
    LinuxNet, add_ipv4, add_ipv4_route, add_ipv6, get_operstate, list_link_statuses, query_addrs,
    set_link_up,
};

#[cfg(target_os = "linux")]
mod linux_impl {
    use super::*;
    use core::mem;

    /// Translate the last OS error into a crate [`Error`].
    fn last_os_error(ctx: &str) -> Error {
        // SAFETY: reading the thread-local errno is always valid.
        let errno = unsafe { *libc_errno_location() };
        Error::Other(format!("{ctx}: errno {errno}"))
    }

    /// `__errno_location()` shim (libc exposes it as `__errno_location` on glibc
    /// and musl). `libc::__errno_location` is provided by the crate.
    unsafe fn libc_errno_location() -> *mut i32 {
        // SAFETY: caller upholds the shim's unsafe contract; Rust 2024 still
        // requires the unsafe libc call to be explicit inside an unsafe fn.
        unsafe { libc::__errno_location() }
    }

    /// Resolve an interface name to its kernel index via `if_nametoindex(3)`.
    fn if_index(ifname: &str) -> Result<u32> {
        let cname = cstr(ifname)?;
        // SAFETY: cname is a valid NUL-terminated C string.
        // `.cast()` adapts *const u8 -> *const c_char, whose signedness is
        // arch-dependent (i8 on x86_64, u8 on aarch64).
        let idx = unsafe { libc::if_nametoindex(cname.as_ptr().cast()) };
        if idx == 0 {
            return Err(Error::not_found(format!("interface '{ifname}' not found")));
        }
        Ok(idx)
    }

    /// Build a NUL-terminated C string from `s` (rejects embedded NULs).
    fn cstr(s: &str) -> Result<Vec<u8>> {
        if s.bytes().any(|b| b == 0) {
            return Err(Error::invalid(format!("string '{s}' contains NUL")));
        }
        let mut v = Vec::with_capacity(s.len() + 1);
        v.extend_from_slice(s.as_bytes());
        v.push(0);
        Ok(v)
    }

    /// An open `AF_NETLINK`/`NETLINK_ROUTE` socket, closed on drop.
    struct NetlinkSocket {
        fd: i32,
    }

    impl NetlinkSocket {
        /// Open and bind a routing-netlink socket.
        fn open() -> Result<Self> {
            // SAFETY: plain socket(2) FFI; the fd is validated before use.
            let fd = unsafe {
                libc::socket(
                    libc::AF_NETLINK,
                    libc::SOCK_RAW | libc::SOCK_CLOEXEC,
                    NETLINK_ROUTE,
                )
            };
            if fd < 0 {
                return Err(last_os_error("socket(AF_NETLINK)"));
            }
            let sock = NetlinkSocket { fd };
            // Bind with pid 0 so the kernel assigns a unique port id.
            let mut addr: libc::sockaddr_nl = unsafe { mem::zeroed() };
            addr.nl_family = libc::AF_NETLINK as u16;
            // SAFETY: addr is a valid sockaddr_nl of the given length.
            let rc = unsafe {
                libc::bind(
                    sock.fd,
                    core::ptr::addr_of!(addr).cast(),
                    mem::size_of::<libc::sockaddr_nl>() as u32,
                )
            };
            if rc < 0 {
                return Err(last_os_error("bind(AF_NETLINK)"));
            }
            Ok(sock)
        }

        /// Send a fully-formed netlink message to the kernel (pid 0).
        fn send(&self, msg: &[u8]) -> Result<()> {
            let mut addr: libc::sockaddr_nl = unsafe { mem::zeroed() };
            addr.nl_family = libc::AF_NETLINK as u16;
            // SAFETY: msg points to len valid bytes; addr is a valid kernel addr.
            let n = unsafe {
                libc::sendto(
                    self.fd,
                    msg.as_ptr().cast(),
                    msg.len(),
                    0,
                    core::ptr::addr_of!(addr).cast(),
                    mem::size_of::<libc::sockaddr_nl>() as u32,
                )
            };
            if n < 0 {
                return Err(last_os_error("sendto(netlink)"));
            }
            Ok(())
        }

        /// Receive one datagram into a fresh buffer.
        fn recv(&self) -> Result<Vec<u8>> {
            let mut buf = alloc::vec![0u8; 8192];
            // SAFETY: buf is a valid writable region of buf.len() bytes.
            let n = unsafe { libc::recv(self.fd, buf.as_mut_ptr().cast(), buf.len(), 0) };
            if n < 0 {
                return Err(last_os_error("recv(netlink)"));
            }
            buf.truncate(n as usize);
            Ok(buf)
        }

        /// Send `msg` then read the `NLM_F_ACK` reply, returning an error if the
        /// kernel reported a non-zero errno.
        fn request_ack(&self, msg: &[u8]) -> Result<()> {
            self.send(msg)?;
            let reply = self.recv()?;
            check_ack(&reply)
        }
    }

    impl Drop for NetlinkSocket {
        fn drop(&mut self) {
            // SAFETY: fd was opened by us and is not used after close.
            unsafe {
                libc::close(self.fd);
            }
        }
    }

    /// Inspect an ACK reply: `NLMSG_ERROR` with errno 0 means success.
    fn check_ack(reply: &[u8]) -> Result<()> {
        if reply.len() < NLMSGHDR_LEN {
            return Err(Error::Other("short netlink ack".to_string()));
        }
        let nlmsg_type = read_u16(reply, 4).unwrap_or(0);
        if nlmsg_type == NLMSG_ERROR {
            let errno = read_u32(reply, NLMSGHDR_LEN).map(|v| v as i32).unwrap_or(0);
            if errno != 0 {
                return Err(Error::Other(format!(
                    "netlink request failed: errno {}",
                    -errno
                )));
            }
        }
        Ok(())
    }

    /// A monotonically-increasing sequence number per process.
    fn next_seq() -> u32 {
        use core::sync::atomic::{AtomicU32, Ordering};
        static SEQ: AtomicU32 = AtomicU32::new(1);
        SEQ.fetch_add(1, Ordering::Relaxed)
    }

    /// Bring `ifname` administratively UP via `RTM_NEWLINK` (IFF_UP).
    pub fn set_link_up(ifname: &str) -> Result<()> {
        let index = if_index(ifname)? as i32;
        let sock = NetlinkSocket::open()?;
        let msg = build_set_link_up(index, next_seq());
        sock.request_ack(&msg)
    }

    /// Assign IPv4 `addr/prefix_len` to `ifname` via `RTM_NEWADDR`.
    pub fn add_ipv4(ifname: &str, addr: &str, prefix_len: u8) -> Result<()> {
        if prefix_len > 32 {
            return Err(Error::invalid(format!(
                "ipv4 prefix length {prefix_len} out of range 0..=32"
            )));
        }
        let index = if_index(ifname)?;
        let octets = parse_ipv4(addr)?;
        let sock = NetlinkSocket::open()?;
        let msg = build_add_ipv4(index, octets, prefix_len, next_seq());
        sock.request_ack(&msg)
    }

    /// Assign IPv6 `addr/prefix_len` to `ifname` via `RTM_NEWADDR`.
    pub fn add_ipv6(ifname: &str, addr: &str, prefix_len: u8) -> Result<()> {
        if prefix_len > 128 {
            return Err(Error::invalid(format!(
                "ipv6 prefix length {prefix_len} out of range 0..=128"
            )));
        }
        let index = if_index(ifname)?;
        let octets = parse_ipv6(addr)?;
        let sock = NetlinkSocket::open()?;
        let msg = build_add_ipv6(index, octets, prefix_len, next_seq());
        sock.request_ack(&msg)
    }

    /// Install an IPv4 route in the main table via `RTM_NEWROUTE`.
    pub fn add_ipv4_route(
        ifname: &str,
        destination: Option<[u8; 4]>,
        prefix_len: u8,
        gateway: Option<[u8; 4]>,
        metric: u32,
        protocol: u8,
    ) -> Result<()> {
        if prefix_len > 32 {
            return Err(Error::invalid(format!(
                "ipv4 route prefix length {prefix_len} out of range 0..=32"
            )));
        }
        if prefix_len > 0 && destination.is_none() {
            return Err(Error::invalid("non-default IPv4 route needs destination"));
        }
        let index = if_index(ifname)?;
        let sock = NetlinkSocket::open()?;
        let msg = build_add_ipv4_route(
            index,
            destination,
            prefix_len,
            gateway,
            metric,
            protocol,
            next_seq(),
        );
        sock.request_ack(&msg)
    }

    /// Read back the IPv4 addresses assigned to `ifname` via `RTM_GETADDR`.
    ///
    /// Returns CIDR strings like `"10.0.0.5/24"`. This is the verification path.
    pub fn query_addrs(ifname: &str) -> Result<Vec<String>> {
        let index = if_index(ifname)?;
        let sock = NetlinkSocket::open()?;
        sock.send(&build_dump_addrs(next_seq()))?;

        // A dump can span several datagrams; concatenate until NLMSG_DONE.
        // NLMSG_ERROR is terminal too; fail instead of blocking forever waiting
        // for a DONE marker that will not follow a dump error.
        let mut all = Vec::new();
        loop {
            let chunk = sock.recv()?;
            let done = dump_chunk_done_or_error(&chunk)?;
            all.extend_from_slice(&chunk);
            if done {
                break;
            }
        }

        let parsed = parse_addr_dump(&all)?;
        Ok(parsed
            .into_iter()
            .filter(|p| p.index == index)
            .map(|p| format!("{}/{}", format_ipv4(p.addr), p.prefix_len))
            .collect())
    }

    /// Read `/sys/class/net/<ifname>/operstate` (a *different* kernel surface)
    /// for cross-confirmation. Returns the trimmed value (e.g. `"up"`, `"down"`,
    /// `"unknown"`).
    pub fn get_operstate(ifname: &str) -> Result<String> {
        if ifname.contains('/') || ifname.contains("..") {
            return Err(Error::invalid(format!("invalid interface name '{ifname}'")));
        }
        let path = format!("/sys/class/net/{ifname}/operstate");
        let bytes = read_file(&path)?;
        let s = String::from_utf8_lossy(&bytes);
        Ok(s.trim().to_string())
    }

    /// Read observed link statuses from Linux `RTM_GETLINK`.
    ///
    /// The dump is intentionally read-only, but not best-effort per interface:
    /// malformed `RTM_NEWLINK` entries surface as errors so the boot/default-DHCP
    /// pipeline cannot silently lose a kernel link.
    pub fn list_link_statuses() -> Result<Vec<LinkStatus>> {
        let sock = NetlinkSocket::open()?;
        sock.send(&build_dump_links(next_seq()))?;

        let mut all = Vec::new();
        loop {
            let chunk = sock.recv()?;
            let done = dump_chunk_done_or_error(&chunk)?;
            all.extend_from_slice(&chunk);
            if done {
                break;
            }
        }
        parse_link_dump(&all)
    }

    /// Read a whole (small) file via raw `open`/`read`/`close` (no std fs needed).
    fn read_file(path: &str) -> Result<Vec<u8>> {
        let cpath = cstr(path)?;
        // SAFETY: cpath is a valid NUL-terminated path string.
        let fd = unsafe { libc::open(cpath.as_ptr().cast(), libc::O_RDONLY | libc::O_CLOEXEC) };
        if fd < 0 {
            return Err(last_os_error(&format!("open({path})")));
        }
        let mut out = Vec::new();
        let mut tmp = [0u8; 256];
        loop {
            // SAFETY: tmp is a valid writable buffer of tmp.len() bytes.
            let n = unsafe { libc::read(fd, tmp.as_mut_ptr().cast(), tmp.len()) };
            if n < 0 {
                let e = last_os_error(&format!("read({path})"));
                // SAFETY: fd is open and owned here.
                unsafe { libc::close(fd) };
                return Err(e);
            }
            if n == 0 {
                break;
            }
            out.extend_from_slice(&tmp[..n as usize]);
        }
        // SAFETY: fd is open and owned here.
        unsafe { libc::close(fd) };
        Ok(out)
    }

    /// Thin adapter the machined sequencer network task calls.
    ///
    /// This is the real stand-in for [`crate::netlink::InMemoryNetlink`] at boot:
    /// `talos-init`'s `Runtime::link_up` can delegate to [`LinuxNet::set_link_up`],
    /// and the address task to [`LinuxNet::add_ipv4`] / [`LinuxNet::query_addrs`].
    #[derive(Debug, Default, Clone, Copy)]
    pub struct LinuxNet;

    impl LinuxNet {
        /// Construct the adapter (stateless).
        pub fn new() -> Self {
            LinuxNet
        }

        /// Bring an interface UP. See [`set_link_up`].
        pub fn set_link_up(&self, ifname: &str) -> Result<()> {
            set_link_up(ifname)
        }

        /// Assign an IPv4 address. See [`add_ipv4`].
        pub fn add_ipv4(&self, ifname: &str, addr: &str, prefix_len: u8) -> Result<()> {
            add_ipv4(ifname, addr, prefix_len)
        }

        /// Assign an IPv6 address. See [`add_ipv6`].
        pub fn add_ipv6(&self, ifname: &str, addr: &str, prefix_len: u8) -> Result<()> {
            add_ipv6(ifname, addr, prefix_len)
        }

        /// Install an IPv4 route. See [`add_ipv4_route`].
        pub fn add_ipv4_route(
            &self,
            ifname: &str,
            destination: Option<[u8; 4]>,
            prefix_len: u8,
            gateway: Option<[u8; 4]>,
            metric: u32,
            protocol: u8,
        ) -> Result<()> {
            add_ipv4_route(ifname, destination, prefix_len, gateway, metric, protocol)
        }

        /// Read assigned IPv4 addresses back from the kernel. See [`query_addrs`].
        pub fn query_addrs(&self, ifname: &str) -> Result<Vec<String>> {
            query_addrs(ifname)
        }

        /// Read `operstate` from `/sys`. See [`get_operstate`].
        pub fn get_operstate(&self, ifname: &str) -> Result<String> {
            get_operstate(ifname)
        }

        /// Read observed link statuses. See [`list_link_statuses`].
        pub fn list_link_statuses(&self) -> Result<Vec<LinkStatus>> {
            list_link_statuses()
        }
    }

    /// `LinuxNet` is the Linux **adapter** for the kernel-ABI port. Callers
    /// depend on the trait, so swapping the kernel substrate replaces this impl
    /// and nothing else. Every method here is pure delegation: the encoding
    /// work (netlink message layout, `RTPROT_*` numbers, the `/sys` path) stays
    /// in this module, which is where Linux belongs.
    impl os_kernel_abi::KernelNet for LinuxNet {
        fn set_link_up(&self, iface: &str) -> Result<()> {
            set_link_up(iface)
        }

        fn add_ipv4_address(&self, iface: &str, addr: &str, prefix_len: u8) -> Result<()> {
            add_ipv4(iface, addr, prefix_len)
        }

        fn add_ipv6_address(&self, iface: &str, addr: &str, prefix_len: u8) -> Result<()> {
            add_ipv6(iface, addr, prefix_len)
        }

        fn add_ipv4_route(
            &self,
            iface: &str,
            destination: Option<[u8; 4]>,
            prefix_len: u8,
            gateway: Option<[u8; 4]>,
            metric: u32,
            origin: os_kernel_abi::RouteOrigin,
        ) -> Result<()> {
            add_ipv4_route(
                iface,
                destination,
                prefix_len,
                gateway,
                metric,
                crate::route::RouteProtocol::from(origin).protocol_id(),
            )
        }

        fn ipv4_addresses(&self, iface: &str) -> Result<Vec<String>> {
            query_addrs(iface)
        }

        fn link_oper_state(&self, iface: &str) -> Result<String> {
            get_operstate(iface)
        }
    }
}

// ---------------------------------------------------------------------------
// Host-compilable unit tests for the pure parts.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn netmask_math() {
        assert_eq!(prefix_to_netmask(0).unwrap(), [0, 0, 0, 0]);
        assert_eq!(prefix_to_netmask(8).unwrap(), [255, 0, 0, 0]);
        assert_eq!(prefix_to_netmask(24).unwrap(), [255, 255, 255, 0]);
        assert_eq!(prefix_to_netmask(25).unwrap(), [255, 255, 255, 128]);
        assert_eq!(prefix_to_netmask(32).unwrap(), [255, 255, 255, 255]);
        assert!(prefix_to_netmask(33).is_err());
    }

    #[test]
    fn ipv4_parse_roundtrip() {
        assert_eq!(parse_ipv4("10.0.0.5").unwrap(), [10, 0, 0, 5]);
        assert_eq!(parse_ipv4("255.255.255.255").unwrap(), [255, 255, 255, 255]);
        assert_eq!(format_ipv4([192, 168, 1, 1]), "192.168.1.1");
        assert!(parse_ipv4("10.0.0").is_err());
        assert!(parse_ipv4("10.0.0.256").is_err());
        assert!(parse_ipv4("10.0.0.1.2").is_err());
        assert!(parse_ipv4("10..0.1").is_err());
    }

    #[test]
    fn ipv6_parse_roundtrip() {
        assert_eq!(
            parse_ipv6("2001:db8::1").unwrap(),
            [0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
        );
        assert_eq!(
            parse_ipv6("::1").unwrap(),
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
        );
        assert_eq!(
            format_ipv6([0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
            "2001:db8::1"
        );
        assert!(parse_ipv6("2001:db8:::1").is_err());
        assert!(parse_ipv6("10.0.0.1").is_err());
    }

    #[test]
    fn align_rounds_to_four() {
        assert_eq!(nl_align(0), 0);
        assert_eq!(nl_align(1), 4);
        assert_eq!(nl_align(4), 4);
        assert_eq!(nl_align(5), 8);
        assert_eq!(nl_align(16), 16);
    }

    #[test]
    fn rtattr_encoding_pads_payload() {
        // A 4-byte payload: rta_len = 4 + 4 = 8, no padding needed.
        let mut buf = Vec::new();
        push_rtattr(&mut buf, IFA_LOCAL, &[10, 0, 0, 5]);
        assert_eq!(buf.len(), 8);
        assert_eq!(read_u16(&buf, 0).unwrap(), 8); // rta_len
        assert_eq!(read_u16(&buf, 2).unwrap(), IFA_LOCAL); // rta_type
        assert_eq!(&buf[4..8], &[10, 0, 0, 5]);

        // A 1-byte payload: rta_len = 5, padded to 8 total.
        let mut buf2 = Vec::new();
        push_rtattr(&mut buf2, 7, &[0xAB]);
        assert_eq!(read_u16(&buf2, 0).unwrap(), 5); // unpadded rta_len
        assert_eq!(buf2.len(), 8); // padded to alignment
        assert_eq!(buf2[4], 0xAB);
        assert_eq!(&buf2[5..8], &[0, 0, 0]); // pad bytes are zero
    }

    #[test]
    fn set_link_up_message_layout() {
        let buf = build_set_link_up(3, 42);
        assert_eq!(buf.len(), NLMSGHDR_LEN + IFINFOMSG_LEN);
        // nlmsg_len matches the buffer length.
        assert_eq!(read_u32(&buf, 0).unwrap() as usize, buf.len());
        assert_eq!(read_u16(&buf, 4).unwrap(), RTM_NEWLINK);
        assert_eq!(read_u16(&buf, 6).unwrap(), NLM_F_REQUEST | NLM_F_ACK);
        assert_eq!(read_u32(&buf, 8).unwrap(), 42); // seq
        // ifinfomsg starts at NLMSGHDR_LEN.
        let body = NLMSGHDR_LEN;
        assert_eq!(buf[body], AF_UNSPEC); // ifi_family
        assert_eq!(read_u32(&buf, body + 4).unwrap() as i32, 3); // ifi_index
        assert_eq!(read_u32(&buf, body + 8).unwrap(), IFF_UP | IFF_RUNNING); // flags
        assert_eq!(read_u32(&buf, body + 12).unwrap(), IFF_UP | IFF_RUNNING); // change
    }

    #[test]
    fn add_ipv4_message_layout() {
        let buf = build_add_ipv4(3, [10, 0, 0, 5], 24, 7);
        // header + ifaddrmsg + two rtattrs (each 8 bytes for a v4 addr).
        assert_eq!(buf.len(), NLMSGHDR_LEN + IFADDRMSG_LEN + 8 + 8);
        assert_eq!(read_u32(&buf, 0).unwrap() as usize, buf.len());
        assert_eq!(read_u16(&buf, 4).unwrap(), RTM_NEWADDR);
        assert_eq!(
            read_u16(&buf, 6).unwrap(),
            NLM_F_REQUEST | NLM_F_ACK | NLM_F_CREATE | NLM_F_EXCL
        );
        // ifaddrmsg.
        let body = NLMSGHDR_LEN;
        assert_eq!(buf[body], AF_INET); // ifa_family
        assert_eq!(buf[body + 1], 24); // ifa_prefixlen
        assert_eq!(read_u32(&buf, body + 4).unwrap(), 3); // ifa_index
        // first rtattr is IFA_LOCAL.
        let rta = body + IFADDRMSG_LEN;
        assert_eq!(read_u16(&buf, rta).unwrap(), 8);
        assert_eq!(read_u16(&buf, rta + 2).unwrap(), IFA_LOCAL);
        assert_eq!(&buf[rta + 4..rta + 8], &[10, 0, 0, 5]);
        // second rtattr is IFA_ADDRESS.
        assert_eq!(read_u16(&buf, rta + 8 + 2).unwrap(), IFA_ADDRESS);
    }

    #[test]
    fn add_ipv6_message_layout() {
        let addr = [0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        let buf = build_add_ipv6(7, addr, 64, 11);
        // header + ifaddrmsg + two rtattrs (each 20 bytes for a v6 addr).
        assert_eq!(buf.len(), NLMSGHDR_LEN + IFADDRMSG_LEN + 20 + 20);
        assert_eq!(read_u32(&buf, 0).unwrap() as usize, buf.len());
        assert_eq!(read_u16(&buf, 4).unwrap(), RTM_NEWADDR);
        assert_eq!(
            read_u16(&buf, 6).unwrap(),
            NLM_F_REQUEST | NLM_F_ACK | NLM_F_CREATE | NLM_F_EXCL
        );
        assert_eq!(read_u32(&buf, 8).unwrap(), 11);

        // ifaddrmsg.
        let body = NLMSGHDR_LEN;
        assert_eq!(buf[body], AF_INET6); // ifa_family
        assert_eq!(buf[body + 1], 64); // ifa_prefixlen
        assert_eq!(buf[body + 2], 0); // ifa_flags
        assert_eq!(buf[body + 3], 0); // ifa_scope
        assert_eq!(read_u32(&buf, body + 4).unwrap(), 7); // ifa_index

        // first rtattr is IFA_LOCAL.
        let local = body + IFADDRMSG_LEN;
        assert_eq!(read_u16(&buf, local).unwrap(), 20);
        assert_eq!(read_u16(&buf, local + 2).unwrap(), IFA_LOCAL);
        assert_eq!(&buf[local + 4..local + 20], &addr);

        // second rtattr is IFA_ADDRESS.
        let address = local + 20;
        assert_eq!(read_u16(&buf, address).unwrap(), 20);
        assert_eq!(read_u16(&buf, address + 2).unwrap(), IFA_ADDRESS);
        assert_eq!(&buf[address + 4..address + 20], &addr);
    }

    #[test]
    fn add_ipv4_route_message_layout() {
        let buf = build_add_ipv4_route(3, None, 0, Some([10, 0, 0, 1]), 1024, 3, 8);
        assert_eq!(read_u32(&buf, 0).unwrap() as usize, buf.len());
        assert_eq!(read_u16(&buf, 4).unwrap(), RTM_NEWROUTE);
        assert_eq!(
            read_u16(&buf, 6).unwrap(),
            NLM_F_REQUEST | NLM_F_ACK | NLM_F_CREATE | NLM_F_EXCL
        );
        let body = NLMSGHDR_LEN;
        assert_eq!(buf[body], AF_INET);
        assert_eq!(buf[body + 1], 0); // default route prefix
        assert_eq!(buf[body + 4], 254); // main table
        assert_eq!(buf[body + 5], 3); // RTPROT_BOOT
        assert_eq!(buf[body + 6], 0); // universe scope because gateway exists
        assert_eq!(buf[body + 7], 1); // unicast

        let gateway = body + RTMSG_LEN;
        assert_eq!(read_u16(&buf, gateway).unwrap(), 8);
        assert_eq!(read_u16(&buf, gateway + 2).unwrap(), RTA_GATEWAY);
        assert_eq!(&buf[gateway + 4..gateway + 8], &[10, 0, 0, 1]);

        let oif = gateway + 8;
        assert_eq!(read_u16(&buf, oif + 2).unwrap(), RTA_OIF);
        assert_eq!(read_u32(&buf, oif + 4).unwrap(), 3);

        let metric = oif + 8;
        assert_eq!(read_u16(&buf, metric + 2).unwrap(), RTA_PRIORITY);
        assert_eq!(read_u32(&buf, metric + 4).unwrap(), 1024);

        let onlink = build_add_ipv4_route(3, Some([169, 254, 0, 1]), 32, None, 1024, 3, 9);
        assert_eq!(onlink[NLMSGHDR_LEN + 6], 253); // link scope without gateway
        let dst = NLMSGHDR_LEN + RTMSG_LEN;
        assert_eq!(read_u16(&onlink, dst + 2).unwrap(), RTA_DST);
        assert_eq!(&onlink[dst + 4..dst + 8], &[169, 254, 0, 1]);
    }

    #[test]
    fn dump_request_layout() {
        let buf = build_dump_addrs(99);
        assert_eq!(buf.len(), NLMSGHDR_LEN + IFADDRMSG_LEN);
        assert_eq!(read_u16(&buf, 4).unwrap(), RTM_GETADDR);
        assert_eq!(read_u16(&buf, 6).unwrap(), NLM_F_REQUEST | NLM_F_DUMP);
        assert_eq!(buf[NLMSGHDR_LEN], AF_INET); // scoped to IPv4

        let links = build_dump_links(100);
        assert_eq!(links.len(), NLMSGHDR_LEN + IFINFOMSG_LEN);
        assert_eq!(read_u16(&links, 4).unwrap(), RTM_GETLINK);
        assert_eq!(read_u16(&links, 6).unwrap(), NLM_F_REQUEST | NLM_F_DUMP);
        assert_eq!(links[NLMSGHDR_LEN], AF_UNSPEC);
    }

    /// Build a synthetic `RTM_NEWADDR` dump entry and parse it back — proves the
    /// builder and parser agree on the wire layout without touching a kernel.
    fn synth_newaddr(index: u32, addr: [u8; 4], prefix: u8) -> Vec<u8> {
        let mut body = Vec::new();
        push_ifaddrmsg(&mut body, AF_INET, prefix, 0, index);
        push_rtattr(&mut body, IFA_ADDRESS, &addr);
        push_rtattr(&mut body, IFA_LOCAL, &addr);

        let mut msg = Vec::new();
        push_nlmsghdr(&mut msg, RTM_NEWADDR, 0, 1);
        msg.extend_from_slice(&body);
        patch_nlmsg_len(&mut msg);
        msg
    }

    fn push_test_ifinfomsg(buf: &mut Vec<u8>, ifi_type: u16, flags: u32) {
        buf.push(AF_UNSPEC);
        buf.push(0);
        buf.extend_from_slice(&ifi_type.to_ne_bytes());
        buf.extend_from_slice(&7i32.to_ne_bytes());
        buf.extend_from_slice(&flags.to_ne_bytes());
        buf.extend_from_slice(&0u32.to_ne_bytes());
    }

    fn nul(value: &str) -> Vec<u8> {
        let mut out = value.as_bytes().to_vec();
        out.push(0);
        out
    }

    #[allow(clippy::too_many_arguments)] // arity mirrors the rtnetlink message field set
    fn synth_newlink(
        name: &str,
        ifi_type: u16,
        flags: u32,
        mtu: u32,
        mac: [u8; 6],
        operstate: u8,
        carrier: u8,
        kind: &str,
        aliases: &[&str],
    ) -> Vec<u8> {
        let mut body = Vec::new();
        push_test_ifinfomsg(&mut body, ifi_type, flags);
        push_rtattr(&mut body, IFLA_IFNAME, &nul(name));
        push_rtattr(&mut body, IFLA_MTU, &mtu.to_ne_bytes());
        push_rtattr(&mut body, IFLA_ADDRESS, &mac);
        push_rtattr(&mut body, IFLA_OPERSTATE, &[operstate]);
        push_rtattr(&mut body, IFLA_CARRIER, &[carrier]);
        if !kind.is_empty() {
            let mut linkinfo = Vec::new();
            push_rtattr(&mut linkinfo, IFLA_INFO_KIND, &nul(kind));
            push_rtattr(&mut body, IFLA_LINKINFO, &linkinfo);
        }
        for alias in aliases {
            push_rtattr(&mut body, IFLA_ALT_IFNAME, &nul(alias));
        }

        let mut msg = Vec::new();
        push_nlmsghdr(&mut msg, RTM_NEWLINK, 0, 1);
        msg.extend_from_slice(&body);
        patch_nlmsg_len(&mut msg);
        msg
    }

    #[test]
    fn parse_single_addr_message() {
        let msg = synth_newaddr(5, [192, 168, 1, 10], 24);
        let body = &msg[NLMSGHDR_LEN..];
        let p = parse_addr_message(body).unwrap();
        assert_eq!(p.index, 5);
        assert_eq!(p.addr, [192, 168, 1, 10]);
        assert_eq!(p.prefix_len, 24);
    }

    #[test]
    fn parse_dump_collects_all_and_stops_at_done() {
        let mut dump = Vec::new();
        dump.extend_from_slice(&synth_newaddr(1, [127, 0, 0, 1], 8));
        dump.extend_from_slice(&synth_newaddr(2, [10, 0, 0, 5], 24));
        // Append an NLMSG_DONE marker.
        let mut done = Vec::new();
        push_nlmsghdr(&mut done, NLMSG_DONE, 0, 1);
        patch_nlmsg_len(&mut done);
        dump.extend_from_slice(&done);
        // Anything after DONE must be ignored.
        dump.extend_from_slice(&synth_newaddr(3, [8, 8, 8, 8], 32));

        let parsed = parse_addr_dump(&dump).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].addr, [127, 0, 0, 1]);
        assert_eq!(parsed[1].addr, [10, 0, 0, 5]);
        assert_eq!(parsed[1].prefix_len, 24);
    }

    #[test]
    fn parse_dump_surfaces_error() {
        let mut buf = Vec::new();
        push_nlmsghdr(&mut buf, NLMSG_ERROR, 0, 1);
        // errno -1 (EPERM) as i32.
        buf.extend_from_slice(&(-1i32).to_ne_bytes());
        patch_nlmsg_len(&mut buf);
        let err = parse_addr_dump(&buf).unwrap_err();
        assert_eq!(err.kind(), "other");
    }

    #[test]
    fn kernel_link_status_dump_chunk_error_is_terminal_before_done() {
        let mut buf = Vec::new();
        push_nlmsghdr(&mut buf, NLMSG_ERROR, 0, 1);
        // errno -19 (ENODEV) as i32. No NLMSG_DONE follows this standalone
        // error, so the live receive loop must not keep waiting.
        buf.extend_from_slice(&(-19i32).to_ne_bytes());
        patch_nlmsg_len(&mut buf);

        let err = dump_chunk_done_or_error(&buf).unwrap_err();

        assert_eq!(err.kind(), "other");
        assert!(err.to_string().contains("errno 19"));
    }

    #[test]
    fn kernel_link_status_dump_chunk_done_is_terminal() {
        let mut buf = Vec::new();
        push_nlmsghdr(&mut buf, NLMSG_DONE, 0, 1);
        patch_nlmsg_len(&mut buf);

        assert!(dump_chunk_done_or_error(&buf).unwrap());
    }

    #[test]
    fn parse_rejects_non_ipv4_family() {
        let mut body = Vec::new();
        // AF_INET6 == 10.
        push_ifaddrmsg(&mut body, 10, 64, 0, 1);
        assert!(parse_addr_message(&body).is_none());
    }

    #[test]
    fn kernel_link_status_parse_rtnetlink_physical_ethernet() {
        let msg = synth_newlink(
            "eth0",
            1,
            IFF_UP | IFF_RUNNING,
            1500,
            [0x02, 0, 0, 0, 0, 1],
            6,
            1,
            "",
            &[],
        );
        let status = parse_link_message(&msg[NLMSGHDR_LEN..]).unwrap();

        assert_eq!(status.name, "eth0");
        assert_eq!(status.link_type, LinkType::Ether);
        assert_eq!(status.kind, "");
        assert!(status.aliases.is_empty());
        assert!(status.admin_up);
        assert_eq!(status.oper_state, OperState::Up);
        assert!(status.carrier);
        assert_eq!(status.hardware_addr, [0x02, 0, 0, 0, 0, 1]);
        assert_eq!(status.mtu, 1500);
        assert!(status.physical());
        assert!(status.default_dhcp4_candidate());
    }

    #[test]
    fn kernel_link_status_parse_rtnetlink_linkinfo_kind_as_non_physical() {
        let msg = synth_newlink(
            "eth0.100",
            1,
            IFF_UP | IFF_RUNNING,
            1500,
            [0x02, 0, 0, 0, 0, 0x64],
            6,
            1,
            "vlan",
            &[],
        );
        let status = parse_link_message(&msg[NLMSGHDR_LEN..]).unwrap();

        assert_eq!(status.link_type, LinkType::Ether);
        assert_eq!(status.kind, "vlan");
        assert!(!status.physical());
        assert!(!status.default_dhcp4_candidate());
    }

    #[test]
    fn kernel_link_status_parse_rtnetlink_aliases_from_top_level_and_prop_list() {
        let mut msg = synth_newlink(
            "enp0s1",
            1,
            IFF_UP,
            9000,
            [0x02, 0, 0, 0, 0, 2],
            2,
            0,
            "",
            &["eth0"],
        );
        let mut prop_list = Vec::new();
        push_rtattr(&mut prop_list, IFLA_ALT_IFNAME, &nul("lan0"));
        push_rtattr(&mut prop_list, IFLA_ALT_IFNAME, &nul("eth0"));
        let insert_at = read_u32(&msg, 0).unwrap() as usize;
        msg.truncate(insert_at);
        push_rtattr(&mut msg, IFLA_PROP_LIST, &prop_list);
        patch_nlmsg_len(&mut msg);

        let status = parse_link_message(&msg[NLMSGHDR_LEN..]).unwrap();
        assert_eq!(status.aliases, vec!["eth0".to_string(), "lan0".to_string()]);
        assert!(status.has_name("enp0s1"));
        assert!(status.has_name("eth0"));
        assert!(status.has_name("lan0"));
    }

    #[test]
    fn kernel_link_status_parse_rtnetlink_ignores_ifalias_as_alt_name() {
        let mut msg = synth_newlink(
            "enp0s1",
            1,
            IFF_UP,
            9000,
            [0x02, 0, 0, 0, 0, 2],
            2,
            0,
            "",
            &[],
        );
        let insert_at = read_u32(&msg, 0).unwrap() as usize;
        msg.truncate(insert_at);
        push_rtattr(&mut msg, IFLA_IFALIAS, &nul("eth0"));
        patch_nlmsg_len(&mut msg);

        let status = parse_link_message(&msg[NLMSGHDR_LEN..]).unwrap();
        assert!(status.aliases.is_empty());
        assert!(status.has_name("enp0s1"));
        assert!(!status.has_name("eth0"));
    }

    #[test]
    fn kernel_link_status_parse_rtnetlink_ether_requires_hardware_address() {
        let mut body = Vec::new();
        push_test_ifinfomsg(&mut body, 1, IFF_UP);
        push_rtattr(&mut body, IFLA_IFNAME, &nul("eth0"));
        push_rtattr(&mut body, IFLA_MTU, &1500u32.to_ne_bytes());
        push_rtattr(&mut body, IFLA_OPERSTATE, &[6]);
        push_rtattr(&mut body, IFLA_CARRIER, &[1]);

        let err = parse_link_message(&body).unwrap_err();

        assert_eq!(err.kind(), "parse");
        assert!(err.to_string().contains("IFLA_ADDRESS"));
    }

    #[test]
    fn kernel_link_status_parse_dump_surfaces_malformed_newlink() {
        let mut body = Vec::new();
        push_test_ifinfomsg(&mut body, 1, IFF_UP);
        push_rtattr(&mut body, IFLA_MTU, &1500u32.to_ne_bytes());

        let mut msg = Vec::new();
        push_nlmsghdr(&mut msg, RTM_NEWLINK, 0, 1);
        msg.extend_from_slice(&body);
        patch_nlmsg_len(&mut msg);

        let err = parse_link_dump(&msg).unwrap_err();
        assert_eq!(err.kind(), "parse");
    }

    #[test]
    fn kernel_link_status_parse_proc_net_dev_names() {
        let proc_net_dev = "\
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo: 100 1 0 0 0 0 0 0 100 1 0 0 0 0 0 0
  eth0: 200 2 0 0 0 0 0 0 200 2 0 0 0 0 0 0
";

        assert_eq!(
            parse_proc_net_dev_ifaces(proc_net_dev),
            vec!["lo".to_string(), "eth0".to_string()]
        );
    }

    #[test]
    fn kernel_link_status_sysfs_fields_populate_physical_ethernet() {
        let status = link_status_from_sysfs_fields(
            "eth0",
            "1\n",
            "",
            "0x1003\n",
            "up\n",
            "1\n",
            "02:00:00:00:00:01\n",
            "1500\n",
        )
        .unwrap();

        assert_eq!(status.name, "eth0");
        assert_eq!(status.link_type, LinkType::Ether);
        assert_eq!(status.kind, "");
        assert!(status.admin_up);
        assert_eq!(status.oper_state, OperState::Up);
        assert!(status.carrier);
        assert_eq!(status.hardware_addr, [0x02, 0, 0, 0, 0, 1]);
        assert_eq!(status.mtu, 1500);
        assert!(status.physical());
        assert!(status.default_dhcp4_candidate());
    }

    #[test]
    fn kernel_link_status_sysfs_fields_preserve_vlan_devtype_as_non_physical() {
        let status = link_status_from_sysfs_fields(
            "eth0.100",
            "1\n",
            "vlan\n",
            "0x1003\n",
            "up\n",
            "1\n",
            "02:00:00:00:00:64\n",
            "1500\n",
        )
        .unwrap();

        assert_eq!(status.link_type, LinkType::Ether);
        assert_eq!(status.kind, "vlan");
        assert!(!status.physical());
        assert!(!status.default_dhcp4_candidate());
    }

    #[test]
    fn kernel_link_status_sysfs_fields_reject_malformed_identity() {
        assert!(
            link_status_from_sysfs_fields(
                "../eth0",
                "1",
                "",
                "0x1003",
                "up",
                "1",
                "02:00:00:00:00:01",
                "1500",
            )
            .is_err()
        );
        assert!(
            link_status_from_sysfs_fields(
                "eth0",
                "1",
                "",
                "0x1003",
                "up",
                "1",
                "not-a-mac",
                "1500",
            )
            .is_err()
        );
    }
}
