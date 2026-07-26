//! The SideroLink manager controller and link state machine.
//!
//! Mirrors the `ManagerController` in
//! `internal/app/machined/pkg/controllers/siderolink`: it consumes the parsed
//! [`Config`], drives the provision handshake through a [`ProvisionService`],
//! programs the resulting WireGuard interface through the [`WireguardLink`] OS
//! boundary, and exposes the link's lifecycle as an explicit state machine.
//!
//! ```text
//!   Disabled ──configure──▶ Configured ──provision──▶ Provisioned
//!       ▲                                                  │
//!       │                                            bring_up
//!       └────────── reset ◀── Up ◀──handshake────────────┘
//! ```

use crate::config::Config;
use crate::provision::{ProvisionRequest, ProvisionResponse, ProvisionService};
use crate::tunnel::WireguardPublicKey;
use alloc::string::String;
use alloc::vec::Vec;
use os_kernel::address::NodeAddress;
use os_kernel::error::{Error, Result};

/// The lifecycle state of the SideroLink link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    /// No SideroLink config present; the feature is inactive.
    Disabled,
    /// Config parsed; ready to provision.
    Configured,
    /// Provision handshake completed; address + server peer known.
    Provisioned,
    /// WireGuard interface programmed and a handshake has been observed.
    Up,
}

/// The WireGuard interface configuration the manager programs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkSpec {
    /// The node's own WireGuard public key.
    pub node_public_key: WireguardPublicKey,
    /// The address assigned to this node on the link (`/128`).
    pub node_address: NodeAddress,
    /// The single peer: the management-plane server.
    pub peer_public_key: WireguardPublicKey,
    /// The peer's WireGuard endpoint `host:port`.
    pub peer_endpoint: String,
    /// The server-side link address.
    pub server_address: NodeAddress,
}

impl LinkSpec {
    fn from_response(node_public_key: WireguardPublicKey, resp: &ProvisionResponse) -> Self {
        LinkSpec {
            node_public_key,
            node_address: resp.node_address,
            peer_public_key: resp.server_public_key.clone(),
            peer_endpoint: resp.server_wg_endpoint(),
            server_address: resp.server_address,
        }
    }
}

/// The WireGuard interface OS boundary (netlink / wgctrl in production).
pub trait WireguardLink {
    /// Program (create or update) the SideroLink WireGuard interface.
    fn configure(&mut self, spec: &LinkSpec) -> Result<()>;
    /// Tear the interface down.
    fn shutdown(&mut self) -> Result<()>;
}

/// An in-memory [`WireguardLink`] recording the last programmed spec.
#[derive(Debug, Default)]
pub struct InMemoryWireguardLink {
    current: Option<LinkSpec>,
    configure_calls: usize,
    shutdown_calls: usize,
}

impl InMemoryWireguardLink {
    /// A fresh, unconfigured link device.
    pub fn new() -> Self {
        Self::default()
    }

    /// The currently programmed spec, if any.
    pub fn current(&self) -> Option<&LinkSpec> {
        self.current.as_ref()
    }

    /// How many times `configure` was invoked.
    pub fn configure_calls(&self) -> usize {
        self.configure_calls
    }

    /// How many times `shutdown` was invoked.
    pub fn shutdown_calls(&self) -> usize {
        self.shutdown_calls
    }
}

impl WireguardLink for InMemoryWireguardLink {
    fn configure(&mut self, spec: &LinkSpec) -> Result<()> {
        self.configure_calls += 1;
        self.current = Some(spec.clone());
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        self.shutdown_calls += 1;
        self.current = None;
        Ok(())
    }
}

/// The node's stable identity used when provisioning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeIdentity {
    /// Hardware UUID.
    pub uuid: String,
    /// The node's WireGuard public key.
    pub public_key: WireguardPublicKey,
    /// Talos version string.
    pub talos_version: String,
}

impl NodeIdentity {
    /// Build a node identity, validating the UUID and version are non-empty.
    pub fn new(
        uuid: impl Into<String>,
        public_key: WireguardPublicKey,
        talos_version: impl Into<String>,
    ) -> Result<Self> {
        let uuid = uuid.into();
        let talos_version = talos_version.into();
        if uuid.is_empty() {
            return Err(Error::invalid("node uuid is empty"));
        }
        if talos_version.is_empty() {
            return Err(Error::invalid("talos version is empty"));
        }
        Ok(NodeIdentity {
            uuid,
            public_key,
            talos_version,
        })
    }
}

/// The SideroLink manager controller.
///
/// Owns the link state machine and coordinates the provision handshake and
/// WireGuard programming. Transitions are explicit and reject out-of-order calls
/// so callers (the reconcile loop) cannot skip steps.
#[derive(Debug)]
pub struct SiderolinkManager {
    identity: NodeIdentity,
    config: Option<Config>,
    state: LinkState,
    response: Option<ProvisionResponse>,
    spec: Option<LinkSpec>,
    events: Vec<LinkState>,
}

impl SiderolinkManager {
    /// Create a manager for a node identity, starting in [`LinkState::Disabled`].
    pub fn new(identity: NodeIdentity) -> Self {
        SiderolinkManager {
            identity,
            config: None,
            state: LinkState::Disabled,
            response: None,
            spec: None,
            events: Vec::new(),
        }
    }

    /// The current link state.
    pub fn state(&self) -> LinkState {
        self.state
    }

    /// The ordered history of states this manager has entered.
    pub fn transitions(&self) -> &[LinkState] {
        &self.events
    }

    /// The programmed link spec, available once provisioned.
    pub fn spec(&self) -> Option<&LinkSpec> {
        self.spec.as_ref()
    }

    /// The node's assigned link address, once provisioned.
    pub fn node_address(&self) -> Option<NodeAddress> {
        self.response.as_ref().map(|r| r.node_address)
    }

    fn enter(&mut self, state: LinkState) {
        self.state = state;
        self.events.push(state);
    }

    /// Apply a parsed [`Config`], moving `Disabled -> Configured`.
    ///
    /// Re-applying config is allowed only before provisioning; afterwards the
    /// link must be reset first.
    pub fn configure(&mut self, config: Config) -> Result<()> {
        match self.state {
            LinkState::Disabled | LinkState::Configured => {
                self.config = Some(config);
                self.enter(LinkState::Configured);
                Ok(())
            }
            _ => Err(Error::invalid_state(
                "cannot reconfigure SideroLink after provisioning; reset first",
            )),
        }
    }

    /// Run the provision handshake against `service`, moving
    /// `Configured -> Provisioned`.
    pub fn provision<S: ProvisionService>(&mut self, service: &mut S) -> Result<()> {
        if self.state != LinkState::Configured {
            return Err(Error::invalid_state(
                "provision requires the SideroLink to be Configured",
            ));
        }
        let config = self
            .config
            .as_ref()
            .ok_or_else(|| Error::invalid_state("no SideroLink config"))?;

        let request = ProvisionRequest::from_config(
            config,
            self.identity.uuid.clone(),
            self.identity.public_key.clone(),
            self.identity.talos_version.clone(),
        )?;
        let response = service.provision(&request)?;

        // Sanity: the assigned address must be IPv6 (a ULA link address).
        if !matches!(response.node_address, NodeAddress::V6(_)) {
            return Err(Error::invalid("provision returned a non-IPv6 node address"));
        }

        self.spec = Some(LinkSpec::from_response(
            self.identity.public_key.clone(),
            &response,
        ));
        self.response = Some(response);
        self.enter(LinkState::Provisioned);
        Ok(())
    }

    /// Program the WireGuard interface, moving `Provisioned -> Up`.
    ///
    /// In Talos, the link is considered up once a handshake is observed; here we
    /// treat a successful interface programming as the up transition.
    pub fn bring_up<L: WireguardLink>(&mut self, link: &mut L) -> Result<()> {
        if self.state != LinkState::Provisioned {
            return Err(Error::invalid_state(
                "bring_up requires the SideroLink to be Provisioned",
            ));
        }
        let spec = self
            .spec
            .as_ref()
            .ok_or_else(|| Error::invalid_state("no link spec to program"))?;
        link.configure(spec)?;
        self.enter(LinkState::Up);
        Ok(())
    }

    /// Tear the link down and return to [`LinkState::Disabled`], clearing the
    /// provision result so a fresh handshake is required.
    pub fn reset<L: WireguardLink>(&mut self, link: &mut L) -> Result<()> {
        if self.state == LinkState::Up {
            link.shutdown()?;
        }
        self.response = None;
        self.spec = None;
        self.config = None;
        self.enter(LinkState::Disabled);
        Ok(())
    }

    /// Drive the link from a config all the way to [`LinkState::Up`] in one call,
    /// the common reconcile path. Requires the manager to be `Disabled`.
    pub fn reconcile<S: ProvisionService, L: WireguardLink>(
        &mut self,
        config: Config,
        service: &mut S,
        link: &mut L,
    ) -> Result<()> {
        if self.state != LinkState::Disabled {
            return Err(Error::invalid_state(
                "reconcile requires the SideroLink to be Disabled",
            ));
        }
        self.configure(config)?;
        self.provision(service)?;
        self.bring_up(link)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provision::{DEFAULT_WG_PORT, InMemoryProvisionServer};
    use os_kernel::address::Cidr;

    fn identity(seed: &str) -> NodeIdentity {
        NodeIdentity::new(
            alloc::format!("uuid-{seed}"),
            WireguardPublicKey::derive_from_seed(seed),
            "v1.7.0",
        )
        .unwrap()
    }

    fn server() -> InMemoryProvisionServer {
        InMemoryProvisionServer::new(
            WireguardPublicKey::derive_from_seed("server"),
            "siderolink.example.com",
            DEFAULT_WG_PORT,
            Cidr::parse("fdae:41e4:649b:9303::/64").unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn full_lifecycle_reaches_up() {
        let mut mgr = SiderolinkManager::new(identity("a"));
        let mut srv = server();
        let mut link = InMemoryWireguardLink::new();
        let cfg = Config::parse_api_arg("https://siderolink.example.com:443?jointoken=t").unwrap();

        mgr.configure(cfg).unwrap();
        assert_eq!(mgr.state(), LinkState::Configured);
        mgr.provision(&mut srv).unwrap();
        assert_eq!(mgr.state(), LinkState::Provisioned);
        mgr.bring_up(&mut link).unwrap();
        assert_eq!(mgr.state(), LinkState::Up);

        assert_eq!(link.configure_calls(), 1);
        let spec = mgr.spec().unwrap();
        assert!(matches!(spec.node_address, NodeAddress::V6(_)));
        assert_eq!(
            spec.peer_public_key,
            WireguardPublicKey::derive_from_seed("server")
        );
        assert_eq!(
            mgr.transitions(),
            &[LinkState::Configured, LinkState::Provisioned, LinkState::Up]
        );
    }

    #[test]
    fn reconcile_one_shot() {
        let mut mgr = SiderolinkManager::new(identity("b"));
        let mut srv = server();
        let mut link = InMemoryWireguardLink::new();
        let cfg = Config::parse_api_arg("grpc://10.0.0.1").unwrap();
        mgr.reconcile(cfg, &mut srv, &mut link).unwrap();
        assert_eq!(mgr.state(), LinkState::Up);
        assert!(mgr.node_address().is_some());
    }

    #[test]
    fn out_of_order_transitions_rejected() {
        let mut mgr = SiderolinkManager::new(identity("c"));
        let mut srv = server();
        let mut link = InMemoryWireguardLink::new();
        // Cannot provision before configure.
        assert!(mgr.provision(&mut srv).is_err());
        // Cannot bring up before provision.
        assert!(mgr.bring_up(&mut link).is_err());
    }

    #[test]
    fn cannot_reconfigure_after_provisioning() {
        let mut mgr = SiderolinkManager::new(identity("d"));
        let mut srv = server();
        let cfg = Config::parse_api_arg("grpc://10.0.0.1").unwrap();
        mgr.configure(cfg).unwrap();
        mgr.provision(&mut srv).unwrap();
        let again = Config::parse_api_arg("grpc://10.0.0.2").unwrap();
        assert!(mgr.configure(again).is_err());
    }

    #[test]
    fn reset_tears_down_and_allows_reprovision() {
        let mut mgr = SiderolinkManager::new(identity("e"));
        let mut srv = server();
        let mut link = InMemoryWireguardLink::new();
        let cfg = Config::parse_api_arg("grpc://10.0.0.1").unwrap();
        mgr.reconcile(cfg, &mut srv, &mut link).unwrap();
        assert_eq!(mgr.state(), LinkState::Up);

        mgr.reset(&mut link).unwrap();
        assert_eq!(mgr.state(), LinkState::Disabled);
        assert_eq!(link.shutdown_calls(), 1);
        assert!(link.current().is_none());
        assert!(mgr.node_address().is_none());

        // Can run the whole flow again.
        let cfg2 = Config::parse_api_arg("grpc://10.0.0.1").unwrap();
        mgr.reconcile(cfg2, &mut srv, &mut link).unwrap();
        assert_eq!(mgr.state(), LinkState::Up);
    }

    #[test]
    fn reconcile_requires_disabled_state() {
        let mut mgr = SiderolinkManager::new(identity("f"));
        let mut srv = server();
        let mut link = InMemoryWireguardLink::new();
        let cfg = Config::parse_api_arg("grpc://10.0.0.1").unwrap();
        mgr.configure(cfg).unwrap();
        let cfg2 = Config::parse_api_arg("grpc://10.0.0.1").unwrap();
        assert!(mgr.reconcile(cfg2, &mut srv, &mut link).is_err());
    }

    #[test]
    fn node_identity_validation() {
        assert!(NodeIdentity::new("", WireguardPublicKey::derive_from_seed("x"), "v1").is_err());
        assert!(NodeIdentity::new("uuid", WireguardPublicKey::derive_from_seed("x"), "").is_err());
    }
}
