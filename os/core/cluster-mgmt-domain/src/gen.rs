//! Config bundle generation: `talosctl gen config` / `talosctl gen secrets`.
//!
//! Mirrors Talos `pkg/machinery/config/generate` and the `talosctl gen`
//! command family. A [`SecretsBundle`] holds the cluster-wide secrets (CA
//! certificates modeled as opaque fingerprints, the bootstrap token, and
//! the etcd/aggregator material). A [`ConfigBundle`] derives per-role machine
//! configs and a talosconfig (client config) from a [`GenInput`].
//!
//! Crypto is *modeled*, not performed: certificates are represented by stable,
//! deterministic fingerprints derived from the cluster name and a counter, so
//! tests can assert that the same input yields the same bundle and that
//! control-plane and worker configs share the same CA.
//!
//! That is not a doc-comment promise. [`Secret`] has a private field and
//! [`Secret::derive`] is its only constructor, and `derive` — plus every
//! function that transitively reaches it — is behind
//! `cfg(any(test, feature = "modeled-crypto"))`. The feature is non-default and
//! no production target enables it, so a production build cannot construct a
//! [`Secret`], and therefore cannot construct a [`CertificateAuthority`],
//! [`SecretsBundle`], [`ConfigBundle`], [`crate::ClusterPlan`], or call
//! [`crate::create_cluster`]. Misuse is a link/compile error, not a review miss.

use crate::ClusterError;
use os_kernel::machine_type::MachineType;
use os_kernel::version::Version;

/// A deterministically-derived opaque secret/certificate value.
///
/// Real Talos generates RSA/ECDSA key material; here we model a secret as a
/// hex fingerprint so the generation logic (which material flows into which
/// config) can be exercised without a crypto dependency.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Secret(String);

impl Secret {
    /// Derive a secret deterministically from a seed string.
    #[cfg(any(test, feature = "modeled-crypto"))]
    pub fn derive(seed: &str) -> Self {
        Secret(hex_hash(seed.as_bytes()))
    }

    /// The hex string form.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A cluster CA certificate + key pair (modeled as fingerprints).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificateAuthority {
    /// Fingerprint of the CA certificate.
    pub cert: Secret,
    /// Fingerprint of the CA private key.
    pub key: Secret,
}

impl CertificateAuthority {
    #[cfg(any(test, feature = "modeled-crypto"))]
    fn derive(cluster: &str, kind: &str) -> Self {
        CertificateAuthority {
            cert: Secret::derive(&format!("{cluster}:{kind}:ca:cert")),
            key: Secret::derive(&format!("{cluster}:{kind}:ca:key")),
        }
    }
}

/// The cluster-wide secrets bundle (`talosctl gen secrets`).
///
/// Mirrors the Talos `SecretsBundle`: the various CAs, the bootstrap token,
/// the cluster id/secret, and the etcd material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretsBundle {
    /// Cluster name the bundle was generated for.
    pub cluster_name: String,
    /// The Talos OS API CA (used by apid/trustd).
    pub os_ca: CertificateAuthority,
    /// The Kubernetes API CA.
    pub k8s_ca: CertificateAuthority,
    /// The etcd CA.
    pub etcd_ca: CertificateAuthority,
    /// The aggregator CA (for the Kubernetes aggregation layer).
    pub aggregator_ca: CertificateAuthority,
    /// The kubelet-facing service account signing key.
    pub service_account_key: Secret,
    /// The bootstrap token (`<id>.<secret>`), used to join nodes.
    pub bootstrap_token: String,
    /// The unique cluster id.
    pub cluster_id: Secret,
    /// The shared cluster secret (used by discovery encryption).
    pub cluster_secret: Secret,
}

impl SecretsBundle {
    /// Generate a fresh secrets bundle for `cluster_name`.
    #[cfg(any(test, feature = "modeled-crypto"))]
    pub fn generate(cluster_name: &str) -> Result<Self, ClusterError> {
        if cluster_name.trim().is_empty() {
            return Err(ClusterError::invalid("cluster name is empty"));
        }
        let token_id = &hex_hash(format!("{cluster_name}:token:id").as_bytes())[..6];
        let token_secret = &hex_hash(format!("{cluster_name}:token:secret").as_bytes())[..16];
        Ok(SecretsBundle {
            cluster_name: cluster_name.to_string(),
            os_ca: CertificateAuthority::derive(cluster_name, "os"),
            k8s_ca: CertificateAuthority::derive(cluster_name, "k8s"),
            etcd_ca: CertificateAuthority::derive(cluster_name, "etcd"),
            aggregator_ca: CertificateAuthority::derive(cluster_name, "aggregator"),
            service_account_key: Secret::derive(&format!("{cluster_name}:sa:key")),
            bootstrap_token: format!("{token_id}.{token_secret}"),
            cluster_id: Secret::derive(&format!("{cluster_name}:id")),
            cluster_secret: Secret::derive(&format!("{cluster_name}:secret")),
        })
    }

    /// Validate structural invariants of the bundle.
    pub fn validate(&self) -> Result<(), ClusterError> {
        if self.cluster_name.trim().is_empty() {
            return Err(ClusterError::invalid("cluster name is empty"));
        }
        let parts: Vec<&str> = self.bootstrap_token.split('.').collect();
        if parts.len() != 2 || parts[0].len() != 6 || parts[1].len() != 16 {
            return Err(ClusterError::invalid(
                "bootstrap token must be of form <6 chars>.<16 chars>",
            ));
        }
        Ok(())
    }
}

/// Input for generating a config bundle (`talosctl gen config`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenInput {
    /// Cluster name.
    pub cluster_name: String,
    /// The Kubernetes control-plane endpoint, e.g. `https://10.0.0.1:6443`.
    pub control_plane_endpoint: String,
    /// Kubernetes version to deploy.
    pub kubernetes_version: Version,
    /// Talos version this config targets.
    pub talos_version: Version,
    /// The cluster DNS domain.
    pub dns_domain: String,
    /// Pod subnet CIDRs.
    pub pod_subnets: Vec<String>,
    /// Service subnet CIDRs.
    pub service_subnets: Vec<String>,
    /// Additional SANs to embed in the API server / talosconfig certs.
    pub additional_sans: Vec<String>,
}

impl GenInput {
    /// Construct a [`GenInput`] with Talos's default subnets and DNS domain.
    pub fn new(
        cluster_name: impl Into<String>,
        control_plane_endpoint: impl Into<String>,
        kubernetes_version: Version,
        talos_version: Version,
    ) -> Self {
        GenInput {
            cluster_name: cluster_name.into(),
            control_plane_endpoint: control_plane_endpoint.into(),
            kubernetes_version,
            talos_version,
            dns_domain: "cluster.local".to_string(),
            pod_subnets: vec!["10.244.0.0/16".to_string()],
            service_subnets: vec!["10.96.0.0/12".to_string()],
            additional_sans: Vec::new(),
        }
    }

    /// Validate the generation input.
    pub fn validate(&self) -> Result<(), ClusterError> {
        if self.cluster_name.trim().is_empty() {
            return Err(ClusterError::invalid("cluster name is empty"));
        }
        if !self.control_plane_endpoint.starts_with("https://") {
            return Err(ClusterError::invalid(
                "control-plane endpoint must be an https:// URL",
            ));
        }
        if self.pod_subnets.is_empty() {
            return Err(ClusterError::invalid("at least one pod subnet is required"));
        }
        if self.service_subnets.is_empty() {
            return Err(ClusterError::invalid(
                "at least one service subnet is required",
            ));
        }
        if self.dns_domain.trim().is_empty() {
            return Err(ClusterError::invalid("dns domain is empty"));
        }
        Ok(())
    }
}

/// A generated machine config for a single role.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MachineConfig {
    /// The machine type this config targets.
    pub machine_type: MachineType,
    /// Cluster name.
    pub cluster_name: String,
    /// Control-plane endpoint this node points at.
    pub control_plane_endpoint: String,
    /// Talos version.
    pub talos_version: Version,
    /// Kubernetes version.
    pub kubernetes_version: Version,
    /// The OS API CA cert this node trusts (shared across the cluster).
    pub os_ca_cert: Secret,
    /// The Kubernetes CA cert (only embedded fully on control-plane nodes; the
    /// cert fingerprint is shared so nodes agree on the same CA).
    pub k8s_ca_cert: Secret,
    /// The bootstrap token used to join.
    pub bootstrap_token: String,
    /// Whether the etcd CA *key* is included (control-plane only).
    pub includes_etcd_ca_key: bool,
}

impl MachineConfig {
    /// True if this config carries control-plane-only secret material.
    pub fn is_control_plane(&self) -> bool {
        self.machine_type.is_control_plane()
    }
}

/// The Talos client config (`talosconfig`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TalosConfig {
    /// Context name (usually the cluster name).
    pub context: String,
    /// CA cert the client trusts.
    pub ca: Secret,
    /// Client cert fingerprint.
    pub client_cert: Secret,
    /// Client key fingerprint.
    pub client_key: Secret,
    /// Endpoints the client may talk to.
    pub endpoints: Vec<String>,
}

/// A complete generated config bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigBundle {
    /// The secrets the bundle was generated from.
    pub secrets: SecretsBundle,
    /// The control-plane machine config.
    pub control_plane: MachineConfig,
    /// The worker machine config.
    pub worker: MachineConfig,
    /// The init machine config (legacy single-bootstrap node).
    pub init: MachineConfig,
    /// The client talosconfig.
    pub talosconfig: TalosConfig,
}

impl ConfigBundle {
    /// Generate a config bundle from input, deriving a fresh secrets bundle.
    #[cfg(any(test, feature = "modeled-crypto"))]
    pub fn generate(input: &GenInput) -> Result<Self, ClusterError> {
        let secrets = SecretsBundle::generate(&input.cluster_name)?;
        Self::generate_with_secrets(input, secrets)
    }

    /// Generate a config bundle from input reusing an existing secrets bundle
    /// (mirrors `talosctl gen config --with-secrets`).
    #[cfg(any(test, feature = "modeled-crypto"))]
    pub fn generate_with_secrets(
        input: &GenInput,
        secrets: SecretsBundle,
    ) -> Result<Self, ClusterError> {
        input.validate()?;
        secrets.validate()?;
        if secrets.cluster_name != input.cluster_name {
            return Err(ClusterError::invalid(
                "secrets bundle cluster name does not match generation input",
            ));
        }

        let mk = |machine_type: MachineType| MachineConfig {
            machine_type,
            cluster_name: input.cluster_name.clone(),
            control_plane_endpoint: input.control_plane_endpoint.clone(),
            talos_version: input.talos_version.clone(),
            kubernetes_version: input.kubernetes_version.clone(),
            os_ca_cert: secrets.os_ca.cert.clone(),
            k8s_ca_cert: secrets.k8s_ca.cert.clone(),
            bootstrap_token: secrets.bootstrap_token.clone(),
            includes_etcd_ca_key: machine_type.is_control_plane(),
        };

        let talosconfig = TalosConfig {
            context: input.cluster_name.clone(),
            ca: secrets.os_ca.cert.clone(),
            client_cert: Secret::derive(&format!("{}:admin:cert", input.cluster_name)),
            client_key: Secret::derive(&format!("{}:admin:key", input.cluster_name)),
            endpoints: vec![input.control_plane_endpoint.clone()],
        };

        Ok(ConfigBundle {
            control_plane: mk(MachineType::ControlPlane),
            worker: mk(MachineType::Worker),
            init: mk(MachineType::Init),
            talosconfig,
            secrets,
        })
    }

    /// Look up the machine config for a given machine type.
    pub fn config_for(&self, machine_type: MachineType) -> Result<&MachineConfig, ClusterError> {
        match machine_type {
            MachineType::ControlPlane => Ok(&self.control_plane),
            MachineType::Worker => Ok(&self.worker),
            MachineType::Init => Ok(&self.init),
            MachineType::Unknown => Err(ClusterError::invalid(
                "cannot produce a config for an unknown machine type",
            )),
        }
    }
}

/// A tiny, dependency-free deterministic hash rendered as 32 hex chars.
///
/// FNV-1a (64-bit) over the input, mixed and rendered to 16 bytes of hex by
/// hashing twice with different seeds. Sufficient for stable, collision-rare
/// fingerprints in tests; not cryptographic.
fn hex_hash(bytes: &[u8]) -> String {
    fn fnv(seed: u64, bytes: &[u8]) -> u64 {
        let mut h: u64 = seed;
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    }
    let a = fnv(0xcbf2_9ce4_8422_2325, bytes);
    let b = fnv(0x1000_0000_0000_0001 ^ a, bytes);
    format!("{a:016x}{b:016x}")
}
