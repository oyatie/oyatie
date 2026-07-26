//! etcd configuration: the bootstrap-vs-join decision, PKI wiring, advertised
//! addresses, and the rendered etcd command-line / static-pod spec.
//!
//! Mirrors Talos's `EtcdConfig`/`EtcdSpec` resources produced by the etcd
//! config controllers and the `pkg/etcd` PKI setup.

use std::collections::BTreeMap;

use os_kernel::{Error, Result};

/// How this node should bring up etcd.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapMode {
    /// First control-plane node: initialize a brand new single-member cluster
    /// (`--initial-cluster-state=new`).
    Bootstrap,
    /// Subsequent node: join an existing cluster as a learner
    /// (`--initial-cluster-state=existing`).
    Join,
    /// Restore from a snapshot, then come up as a new single-member cluster.
    RestoreFromSnapshot,
}

impl BootstrapMode {
    /// The etcd `--initial-cluster-state` value implied by this mode.
    pub fn initial_cluster_state(self) -> &'static str {
        match self {
            BootstrapMode::Bootstrap | BootstrapMode::RestoreFromSnapshot => "new",
            BootstrapMode::Join => "existing",
        }
    }
}

/// PKI material references for etcd (peer + client TLS and the shared CA).
///
/// In Talos these come from the machine config's `cluster.etcd.ca` and the
/// generated node certs; we model them as opaque PEM-ish strings so the logic
/// (presence/consistency checks) can be tested without real crypto.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EtcdPki {
    /// CA certificate shared by all members (PEM).
    pub ca_cert: String,
    /// CA private key (only present on nodes that mint member certs; PEM).
    pub ca_key: String,
    /// This member's server/peer certificate (PEM).
    pub cert: String,
    /// This member's private key (PEM).
    pub key: String,
}

impl EtcdPki {
    /// Validate that the PKI is internally consistent for serving.
    ///
    /// A serving member needs the CA cert plus its own cert/key. The CA *key*
    /// is only required when this node will sign new members' certs.
    pub fn validate(&self, needs_ca_key: bool) -> Result<()> {
        if self.ca_cert.trim().is_empty() {
            return Err(Error::invalid("etcd PKI missing CA certificate"));
        }
        if self.cert.trim().is_empty() || self.key.trim().is_empty() {
            return Err(Error::invalid("etcd PKI missing member cert/key"));
        }
        if needs_ca_key && self.ca_key.trim().is_empty() {
            return Err(Error::invalid(
                "etcd PKI missing CA key (required to sign members)",
            ));
        }
        Ok(())
    }

    /// Whether this node can act as a certificate authority for new members.
    pub fn can_sign(&self) -> bool {
        !self.ca_cert.trim().is_empty() && !self.ca_key.trim().is_empty()
    }
}

/// Default ports etcd listens on.
pub const DEFAULT_PEER_PORT: u16 = 2380;
/// Default client port.
pub const DEFAULT_CLIENT_PORT: u16 = 2379;

/// The desired etcd configuration for this node.
///
/// Equivalent to Talos's `EtcdSpec`: it captures the inputs needed to render
/// the etcd process arguments and to drive the lifecycle controllers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EtcdConfig {
    /// Member name (the node hostname).
    pub name: String,
    /// Data directory for the etcd member store.
    pub data_dir: String,
    /// IP/host to advertise to peers.
    pub advertised_ip: String,
    /// IP/host to listen on for peers/clients (often `0.0.0.0` or the node IP).
    pub listen_ip: String,
    /// Peer port.
    pub peer_port: u16,
    /// Client port.
    pub client_port: u16,
    /// Bootstrap mode.
    pub bootstrap: BootstrapMode,
    /// Initial cluster peers (name -> peer URL) for `--initial-cluster`.
    pub initial_cluster: BTreeMap<String, String>,
    /// PKI material.
    pub pki: EtcdPki,
    /// Extra etcd args (`extraArgs` in the machine config), highest precedence.
    pub extra_args: BTreeMap<String, String>,
}

impl EtcdConfig {
    /// A minimally valid bootstrap config for a single node.
    pub fn bootstrap(name: impl Into<String>, ip: impl Into<String>, pki: EtcdPki) -> Self {
        let name = name.into();
        let ip = ip.into();
        let mut initial = BTreeMap::new();
        initial.insert(name.clone(), format!("https://{ip}:{DEFAULT_PEER_PORT}"));
        EtcdConfig {
            name,
            data_dir: "/var/lib/etcd".to_string(),
            advertised_ip: ip.clone(),
            listen_ip: "0.0.0.0".to_string(),
            peer_port: DEFAULT_PEER_PORT,
            client_port: DEFAULT_CLIENT_PORT,
            bootstrap: BootstrapMode::Bootstrap,
            initial_cluster: initial,
            pki,
            extra_args: BTreeMap::new(),
        }
    }

    /// This member's advertised peer URL.
    pub fn peer_url(&self) -> String {
        format!("https://{}:{}", self.advertised_ip, self.peer_port)
    }

    /// This member's advertised client URL.
    pub fn client_url(&self) -> String {
        format!("https://{}:{}", self.advertised_ip, self.client_port)
    }

    /// Validate the configuration before rendering.
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::invalid("etcd config: empty member name"));
        }
        if self.advertised_ip.trim().is_empty() {
            return Err(Error::invalid("etcd config: empty advertised IP"));
        }
        if self.data_dir.trim().is_empty() {
            return Err(Error::invalid("etcd config: empty data dir"));
        }
        if self.peer_port == 0 || self.client_port == 0 {
            return Err(Error::invalid("etcd config: ports must be non-zero"));
        }
        if self.peer_port == self.client_port {
            return Err(Error::invalid(
                "etcd config: peer and client ports must differ",
            ));
        }
        // The CA key is needed when bootstrapping (this node mints the first
        // certs and signs subsequent joiners), but not when merely joining.
        let needs_ca_key = matches!(
            self.bootstrap,
            BootstrapMode::Bootstrap | BootstrapMode::RestoreFromSnapshot
        );
        self.pki.validate(needs_ca_key)?;
        if matches!(self.bootstrap, BootstrapMode::Join) && self.initial_cluster.is_empty() {
            return Err(Error::invalid(
                "etcd config: join mode requires a non-empty initial cluster",
            ));
        }
        Ok(())
    }

    /// Render the ordered etcd command-line arguments.
    ///
    /// `extra_args` override any computed value with the same key, matching
    /// Talos's `extraArgs` precedence.
    pub fn render_args(&self) -> Result<Vec<String>> {
        self.validate()?;
        let mut args: BTreeMap<String, String> = BTreeMap::new();
        args.insert("name".into(), self.name.clone());
        args.insert("data-dir".into(), self.data_dir.clone());
        args.insert(
            "listen-peer-urls".into(),
            format!("https://{}:{}", self.listen_ip, self.peer_port),
        );
        args.insert(
            "listen-client-urls".into(),
            format!("https://{}:{}", self.listen_ip, self.client_port),
        );
        args.insert("initial-advertise-peer-urls".into(), self.peer_url());
        args.insert("advertise-client-urls".into(), self.client_url());
        args.insert(
            "initial-cluster-state".into(),
            self.bootstrap.initial_cluster_state().into(),
        );
        let initial_cluster = self
            .initial_cluster
            .iter()
            .map(|(n, u)| format!("{n}={u}"))
            .collect::<Vec<_>>()
            .join(",");
        args.insert("initial-cluster".into(), initial_cluster);
        // TLS flags.
        args.insert("client-cert-auth".into(), "true".into());
        args.insert("peer-client-cert-auth".into(), "true".into());

        // Apply overrides last so they win.
        for (k, v) in &self.extra_args {
            args.insert(k.clone(), v.clone());
        }

        Ok(args
            .into_iter()
            .map(|(k, v)| format!("--{k}={v}"))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pki(with_ca_key: bool) -> EtcdPki {
        EtcdPki {
            ca_cert: "CA".into(),
            ca_key: if with_ca_key {
                "CAKEY".into()
            } else {
                String::new()
            },
            cert: "CERT".into(),
            key: "KEY".into(),
        }
    }

    #[test]
    fn bootstrap_config_is_valid_and_renders() {
        let cfg = EtcdConfig::bootstrap("cp1", "10.0.0.1", pki(true));
        assert!(cfg.validate().is_ok());
        let args = cfg.render_args().unwrap();
        assert!(args.iter().any(|a| a == "--initial-cluster-state=new"));
        assert!(
            args.iter()
                .any(|a| a == "--initial-advertise-peer-urls=https://10.0.0.1:2380")
        );
        assert!(args.iter().any(|a| a.starts_with("--initial-cluster=cp1=")));
    }

    #[test]
    fn join_requires_initial_cluster_and_no_ca_key() {
        let mut cfg = EtcdConfig::bootstrap("cp2", "10.0.0.2", pki(false));
        cfg.bootstrap = BootstrapMode::Join;
        // initial_cluster already has cp2; add a peer to be realistic.
        cfg.initial_cluster
            .insert("cp1".into(), "https://10.0.0.1:2380".into());
        assert!(cfg.validate().is_ok());
        let args = cfg.render_args().unwrap();
        assert!(args.iter().any(|a| a == "--initial-cluster-state=existing"));

        cfg.initial_cluster.clear();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn bootstrap_needs_ca_key() {
        let cfg = EtcdConfig::bootstrap("cp1", "10.0.0.1", pki(false));
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn extra_args_override() {
        let mut cfg = EtcdConfig::bootstrap("cp1", "10.0.0.1", pki(true));
        cfg.extra_args.insert("data-dir".into(), "/custom".into());
        let args = cfg.render_args().unwrap();
        assert!(args.iter().any(|a| a == "--data-dir=/custom"));
        assert!(!args.iter().any(|a| a == "--data-dir=/var/lib/etcd"));
    }

    #[test]
    fn same_ports_rejected() {
        let mut cfg = EtcdConfig::bootstrap("cp1", "10.0.0.1", pki(true));
        cfg.client_port = cfg.peer_port;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn pki_can_sign() {
        assert!(pki(true).can_sign());
        assert!(!pki(false).can_sign());
    }
}
