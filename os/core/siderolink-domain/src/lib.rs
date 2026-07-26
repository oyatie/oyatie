#![cfg_attr(not(test), no_std)]
//! # talos-siderolink
//!
//! Models SideroLink, the point-to-point WireGuard link Talos establishes to a
//! management plane (Sidero / Omni). Mirrors the `siderolink` controllers in
//! `internal/app/machined/pkg/controllers/siderolink` and the
//! `pkg/machinery/api/siderolink` provision API:
//!
//! - [`config`]: the `ConfigController` — parses the `siderolink.api=` kernel
//!   argument (scheme, host:port, join token, gRPC-tunnel hint) into a
//!   validated [`Config`] resource.
//! - [`provision`]: the gRPC provision handshake ([`ProvisionRequest`] /
//!   [`ProvisionResponse`]) and an in-memory server that allocates a stable,
//!   per-node IPv6 ULA address out of a `/64`.
//! - [`tunnel`]: the [`WireguardPublicKey`] type plus the kmsg / event sinks
//!   tunnelled over the link, with down-buffering and ordered replay.
//! - [`manager`]: the `ManagerController` link state machine
//!   (`Disabled → Configured → Provisioned → Up`) that drives provisioning and
//!   programs the WireGuard interface through the [`manager::WireguardLink`] OS
//!   boundary.
//!
//! OS boundaries (the gRPC provision RPC, the WireGuard interface, the telemetry
//! sinks) are modelled as traits with in-memory implementations used by the
//! tests. The crate is `no_std` for real builds and only uses the `alloc` crate
//! plus an internal path dependency on `talos-core`; under `cargo test` it links
//! against `std` on the host.

extern crate alloc;

pub mod config;
pub mod manager;
pub mod provision;
pub mod tunnel;

pub use config::{ApiScheme, Config, DEFAULT_API_PORT, KERNEL_ARG_API};
pub use manager::{
    InMemoryWireguardLink, LinkSpec, LinkState, NodeIdentity, SiderolinkManager, WireguardLink,
};
pub use provision::{
    DEFAULT_WG_PORT, InMemoryProvisionServer, ProvisionRequest, ProvisionResponse, ProvisionService,
};
pub use tunnel::{
    InMemorySink, KEY_B64_LEN, Sink, SinkKind, Tunnel, TunnelEntry, WireguardPublicKey,
};

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::address::{Cidr, NodeAddress};

    /// End-to-end: a node boots, parses its kernel arg, provisions against the
    /// management plane, brings up the WireGuard link, and then tunnels kmsg and
    /// event telemetry that was buffered during boot.
    #[test]
    fn boot_to_tunnel_end_to_end() {
        // 1. ConfigController parses the kernel command line.
        let cmdline = "console=ttyS0 \
            siderolink.api=https://omni.example.com:443?jointoken=cluster-secret \
            quiet";
        let config = Config::from_kernel_cmdline(cmdline).unwrap().unwrap();
        assert_eq!(config.host(), "omni.example.com");

        // 2. The management plane stands up a provision server on a ULA /64,
        //    accepting the cluster's join token.
        let mut server = InMemoryProvisionServer::new(
            WireguardPublicKey::derive_from_seed("omni-server"),
            "omni.example.com",
            DEFAULT_WG_PORT,
            Cidr::parse("fdae:41e4:649b:9303::/64").unwrap(),
        )
        .unwrap()
        .require_token("cluster-secret");

        // 3. The node's manager drives the link from config to Up.
        let identity = NodeIdentity::new(
            "0000-1111-2222-3333",
            WireguardPublicKey::derive_from_seed("this-node"),
            "v1.7.0",
        )
        .unwrap();
        let mut manager = SiderolinkManager::new(identity);
        let mut link = InMemoryWireguardLink::new();

        // Telemetry buffered before the link is up.
        let mut tunnel = Tunnel::new(64);
        tunnel.push_kmsg("kernel: Linux version 6.x");
        tunnel.push_event("event: SequenceBoot");

        manager.reconcile(config, &mut server, &mut link).unwrap();
        assert_eq!(manager.state(), LinkState::Up);

        // The node got an address inside the server's prefix and the link was
        // programmed with the server as its single peer.
        let prefix = Cidr::parse("fdae:41e4:649b:9303::/64").unwrap();
        let node_addr = manager.node_address().unwrap();
        assert!(matches!(node_addr, NodeAddress::V6(_)));
        assert!(prefix.contains(&node_addr));
        assert_eq!(
            link.current().unwrap().peer_public_key,
            WireguardPublicKey::derive_from_seed("omni-server")
        );

        // 4. With the link up, telemetry flushes in order to the management
        //    plane's sink.
        tunnel.mark_up();
        let mut sink = InMemorySink::new();
        let delivered = tunnel.flush(&mut sink);
        assert_eq!(delivered, 2);
        assert_eq!(
            sink.payloads_of(SinkKind::KernelLog),
            ["kernel: Linux version 6.x"]
        );
        assert_eq!(sink.payloads_of(SinkKind::Event), ["event: SequenceBoot"]);
    }

    /// A node with no `siderolink.api=` argument never leaves [`LinkState::Disabled`].
    #[test]
    fn siderolink_absent_is_inactive() {
        let config = Config::from_kernel_cmdline("console=ttyS0 quiet").unwrap();
        assert!(config.is_none());

        let manager = SiderolinkManager::new(
            NodeIdentity::new("u", WireguardPublicKey::derive_from_seed("n"), "v1").unwrap(),
        );
        assert_eq!(manager.state(), LinkState::Disabled);
    }

    /// A wrong join token blocks the node from provisioning.
    #[test]
    fn wrong_token_blocks_provision() {
        let mut server = InMemoryProvisionServer::new(
            WireguardPublicKey::derive_from_seed("server"),
            "host",
            DEFAULT_WG_PORT,
            Cidr::parse("fd00:1234:5678:9abc::/64").unwrap(),
        )
        .unwrap()
        .require_token("right");
        let config = Config::parse_api_arg("https://host:443?jointoken=wrong").unwrap();
        let mut manager = SiderolinkManager::new(
            NodeIdentity::new("u", WireguardPublicKey::derive_from_seed("n"), "v1").unwrap(),
        );
        let mut link = InMemoryWireguardLink::new();
        assert!(manager.reconcile(config, &mut server, &mut link).is_err());
        // Failed mid-flow: the config moved it to Configured but provision failed.
        assert_ne!(manager.state(), LinkState::Up);
    }
}
