#![cfg_attr(not(test), no_std)]
//! # talos-network
//!
//! Network configuration subsystem for the operating-system Talos migration.
//!
//! Mirrors the `internal/app/machined/pkg/controllers/network` package in
//! `siderolabs/talos`: network specs (links, addresses, routes) are produced by
//! several controllers, each tagging its output with a [`ConfigLayer`]. Merge
//! controllers fold specs from every layer by priority so that, for any logical
//! resource, the highest-precedence source wins.
//!
//! Modules:
//! - [`config_layer`]: the provenance/priority ordering shared by every spec.
//! - [`link`]: link specs and observed link status (interfaces, bonds, vlans).
//! - [`address`]: address specs, merging, and the node-address aggregation.
//! - [`route`]: route specs, merging, and route-table selection.
//! - [`hostname`]: hostname/domain specs, merging, and cmdline parsing.
//! - [`resolver`]: DNS resolver specs, merging, and `resolv.conf` rendering.
//! - [`operator`]: dynamic operators (DHCP4/6, static/VIP) and their output.
//! - [`netlink`]: the kernel boundary as a trait with an in-memory kernel and
//!   address/route reconcilers.
//! - [`linux_net`]: the REAL Linux rtnetlink/ioctl boundary (libc-only,
//!   `target_os = "linux"`) that replaces the in-memory fake at boot.
//!
//! The crate is `no_std` for real builds and only uses the `alloc` crate, plus
//! an internal path dependency on `talos-core`. Under `cargo test` it links
//! against `std` on the host.

// Pedantic lints intentionally allowed crate-wide: these are documentation- and
// annotation-only nags that do not change behavior or improve idiom for this
// internal subsystem. Suppressing them keeps the signal-to-noise ratio of
// `clippy::pedantic` useful for the lints that do matter.
#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::doc_markdown,
    clippy::module_name_repetitions
)]

extern crate alloc;

pub mod address;
pub mod config_layer;
pub mod dhcp4;
pub mod dhcp6;
pub mod hostname;
pub mod link;
pub mod linux_net;
pub mod nethelpers;
pub mod netlink;
pub mod operator;
pub mod resolver;
pub mod route;

pub use address::{AddressFlags, AddressSpec, NodeAddressSpec, Scope, merge_addresses};
pub use config_layer::ConfigLayer;
pub use dhcp4::{
    Dhcp4ClasslessRoute, Dhcp4ClientConfig, Dhcp4ClientIdentifier, Dhcp4Lease, Dhcp4Offer,
    Dhcp4Outbound, Dhcp4RequestResponse, Dhcp4SendTarget, Dhcp4WireAction, Dhcp4WireState,
    Dhcp4WireTransaction, build_dhcp4_discover, build_dhcp4_reboot_request,
    build_dhcp4_renew_request, build_dhcp4_request_from_offer, dhcp4_elapsed_secs_for_attempt,
    parse_dhcp4_ack, parse_dhcp4_offer, parse_dhcp4_request_response,
};
pub use dhcp6::{
    Dhcp6ClientConfig, Dhcp6ClientIdentifier, Dhcp6IaAddress, Dhcp6Lease, Dhcp6Outbound,
    Dhcp6SendTarget, Dhcp6WireAction, Dhcp6WireState, Dhcp6WireTransaction,
    build_dhcp6_rapid_solicit, build_dhcp6_request, build_dhcp6_request_with_ia_na,
    parse_dhcp6_reply,
};
pub use hostname::{HostnameSpec, HostnameStatus, merge_hostname};
pub use link::{
    AddressFamily, BondMode, LinkKind, LinkSpec, LinkStatus, LinkType, OperState, merge_links,
    vlan_link_name,
};
#[cfg(target_os = "linux")]
pub use linux_net::{
    LinuxNet, add_ipv4, add_ipv4_route, add_ipv6, get_operstate, list_link_statuses, query_addrs,
    set_link_up,
};
pub use nethelpers::VlanProtocol;
pub use netlink::{InMemoryNetlink, Netlink, reconcile_addresses, reconcile_routes};
pub use operator::{
    ClientIdentifierSpec, DEFAULT_ROUTE_METRIC, OperatorKind, OperatorOutput, OperatorResult,
    OperatorSpec, merge_operators,
};
pub use resolver::{ResolverSpec, ResolverStatus, merge_resolvers};
pub use route::{RouteProtocol, RouteSpec, RouteTable, merge_routes};
