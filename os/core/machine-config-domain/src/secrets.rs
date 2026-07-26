//! The cluster [`Secrets`] bundle: PKI material, bootstrap tokens, and shared
//! secrets. Mirrors the Talos `cluster` secrets section plus
//! `machine.token`/`machine.ca`.

use os_kernel::error::{Error, Result};

/// A PEM-encoded certificate / key pair (modeled as opaque strings; no crypto
/// is performed in this `no_std` port).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CertKeyPair {
    /// PEM certificate bytes (base64/PEM text).
    pub crt: String,
    /// PEM private key bytes.
    pub key: String,
}

impl CertKeyPair {
    /// Build a pair from cert + key text.
    pub fn new(crt: impl Into<String>, key: impl Into<String>) -> Self {
        CertKeyPair {
            crt: crt.into(),
            key: key.into(),
        }
    }

    /// Whether both halves are present.
    pub fn is_complete(&self) -> bool {
        !self.crt.is_empty() && !self.key.is_empty()
    }
}

/// The full secret material shared across a cluster.
///
/// Mirrors the Talos `SecretsBundle`: the cluster CA, etcd CA, Kubernetes
/// aggregator/front-proxy CA, the bootstrap token, the cluster ID/secret, and
/// the machine join token.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Secrets {
    /// Kubernetes cluster CA.
    pub cluster_ca: CertKeyPair,
    /// etcd CA.
    pub etcd_ca: CertKeyPair,
    /// Aggregator (front-proxy) CA.
    pub aggregator_ca: CertKeyPair,
    /// Talos machine API CA (`machine.ca`).
    pub machine_ca: CertKeyPair,
    /// Bootstrap token used to join nodes to the Kubernetes control plane
    /// (`<id>.<secret>` form).
    pub bootstrap_token: String,
    /// Opaque cluster id.
    pub cluster_id: String,
    /// Shared cluster secret (`aescbcEncryptionSecret` / `secret`).
    pub cluster_secret: String,
    /// The machine join token (`machine.token`).
    pub machine_token: String,
}

impl Secrets {
    /// Validate the bootstrap token format `<6 chars>.<16 chars>`, mirroring the
    /// Kubernetes bootstrap-token regex used by Talos.
    pub fn validate_bootstrap_token(token: &str) -> Result<()> {
        let (id, secret) = token
            .split_once('.')
            .ok_or_else(|| Error::invalid("bootstrap token must be '<id>.<secret>'"))?;
        if id.len() != 6
            || !id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        {
            return Err(Error::invalid(
                "bootstrap token id must be 6 lowercase alnum chars",
            ));
        }
        if secret.len() != 16
            || !secret
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        {
            return Err(Error::invalid(
                "bootstrap token secret must be 16 lowercase alnum chars",
            ));
        }
        Ok(())
    }

    /// Whether the minimal control-plane secrets are present: cluster CA, etcd
    /// CA, a cluster id and secret.
    pub fn is_control_plane_complete(&self) -> bool {
        self.cluster_ca.is_complete()
            && self.etcd_ca.is_complete()
            && !self.cluster_id.is_empty()
            && !self.cluster_secret.is_empty()
    }

    /// Validate the secret bundle for a control-plane node.
    pub fn validate_control_plane(&self) -> Result<()> {
        if !self.cluster_ca.is_complete() {
            return Err(Error::invalid(
                "cluster CA cert+key required on control plane",
            ));
        }
        if !self.etcd_ca.is_complete() {
            return Err(Error::invalid("etcd CA cert+key required on control plane"));
        }
        if self.cluster_id.is_empty() {
            return Err(Error::invalid("cluster id is required"));
        }
        if self.cluster_secret.is_empty() {
            return Err(Error::invalid("cluster secret is required"));
        }
        Ok(())
    }

    /// Validate the secrets required on a worker node: just the machine token
    /// and the cluster CA certificate (no private key needed).
    pub fn validate_worker(&self) -> Result<()> {
        if self.machine_token.is_empty() {
            return Err(Error::invalid("machine token is required to join"));
        }
        if self.cluster_ca.crt.is_empty() {
            return Err(Error::invalid("cluster CA certificate is required to join"));
        }
        Ok(())
    }
}

/// Build a `.`-joined bootstrap token from id + secret.
pub fn make_bootstrap_token(id: &str, secret: &str) -> String {
    let mut s = id.to_string();
    s.push('.');
    s.push_str(secret);
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_token_validation() {
        assert!(Secrets::validate_bootstrap_token("abcdef.0123456789abcdef").is_ok());
        assert!(Secrets::validate_bootstrap_token("ABCDEF.0123456789abcdef").is_err()); // uppercase id
        assert!(Secrets::validate_bootstrap_token("abc.0123456789abcdef").is_err()); // short id
        assert!(Secrets::validate_bootstrap_token("abcdef.short").is_err()); // short secret
        assert!(Secrets::validate_bootstrap_token("nodot").is_err());
    }

    #[test]
    fn control_plane_completeness() {
        let mut s = Secrets::default();
        assert!(!s.is_control_plane_complete());
        assert!(s.validate_control_plane().is_err());

        s.cluster_ca = CertKeyPair::new("crt", "key");
        s.etcd_ca = CertKeyPair::new("crt", "key");
        s.cluster_id = "id".to_string();
        s.cluster_secret = "secret".to_string();
        assert!(s.is_control_plane_complete());
        assert!(s.validate_control_plane().is_ok());
    }

    #[test]
    fn worker_only_needs_token_and_ca_cert() {
        let mut s = Secrets::default();
        assert!(s.validate_worker().is_err());
        s.machine_token = "tok".to_string();
        s.cluster_ca.crt = "ca-cert".to_string();
        assert!(s.validate_worker().is_ok());
        // No private key required for worker.
        assert!(!s.cluster_ca.is_complete());
    }

    #[test]
    fn token_builder_roundtrips() {
        let t = make_bootstrap_token("abcdef", "0123456789abcdef");
        assert_eq!(t, "abcdef.0123456789abcdef");
        assert!(Secrets::validate_bootstrap_token(&t).is_ok());
    }
}
