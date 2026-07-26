//! Nodename controller.
//!
//! Mirrors `internal/app/machined/pkg/controllers/k8s.NodenameController`: it
//! derives the Kubernetes node name from the machine's hostname (optionally
//! overridden by config), normalizing it to a valid DNS-1123 subdomain. The
//! kubelet registers under this name and the rest of the k8s controllers key
//! off it.

use os_kernel::address::Hostname;
use os_kernel::error::{Error, Result};

/// The maximum length of a Kubernetes node name (DNS-1123 subdomain).
pub const MAX_NODENAME_LEN: usize = 253;

/// Inputs the nodename controller reconciles.
///
/// Talos prefers, in order: an explicit `machine.kubelet.nodeIP`-adjacent
/// `nodename` override, otherwise the host's FQDN or short hostname depending on
/// whether `kubernetes.registerWithFQDN` is set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodenameSpec {
    /// The machine's hostname resource (already validated).
    pub hostname: Hostname,
    /// An explicit nodename override from config, if any.
    pub override_name: Option<String>,
    /// Whether to register with the fully-qualified domain name.
    pub register_with_fqdn: bool,
}

impl NodenameSpec {
    /// Reconcile the node name resource from the spec.
    ///
    /// Returns the lowercased, validated node name the kubelet should use. The
    /// short hostname is used unless `register_with_fqdn` is set, in which case
    /// the full hostname is used. An explicit override always wins (after
    /// normalization).
    pub fn reconcile(&self) -> Result<Nodename> {
        let raw = match &self.override_name {
            Some(o) => o.clone(),
            None => {
                if self.register_with_fqdn {
                    self.hostname.as_str().to_string()
                } else {
                    self.hostname.short().to_string()
                }
            }
        };
        Nodename::new(raw)
    }
}

/// A validated Kubernetes node name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Nodename(String);

impl Nodename {
    /// Validate and normalize a node name to a DNS-1123 subdomain.
    ///
    /// Kubernetes node names must be lowercase, made of `[a-z0-9.-]`, with each
    /// dot-separated label non-empty, starting and ending alphanumeric, and at
    /// most 253 characters total.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into().to_ascii_lowercase();
        if s.is_empty() {
            return Err(Error::invalid("node name is empty"));
        }
        if s.len() > MAX_NODENAME_LEN {
            return Err(Error::invalid(format!(
                "node name exceeds {MAX_NODENAME_LEN} characters"
            )));
        }
        for label in s.split('.') {
            Self::validate_label(label)?;
        }
        Ok(Nodename(s))
    }

    fn validate_label(label: &str) -> Result<()> {
        if label.is_empty() {
            return Err(Error::invalid("node name has an empty label"));
        }
        let bytes = label.as_bytes();
        let first = bytes[0] as char;
        let last = bytes[bytes.len() - 1] as char;
        if !(first.is_ascii_alphanumeric()) || !(last.is_ascii_alphanumeric()) {
            return Err(Error::invalid(format!(
                "node name label '{label}' must start and end alphanumeric"
            )));
        }
        for c in label.chars() {
            if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
                return Err(Error::invalid(format!("invalid node name character '{c}'")));
            }
        }
        Ok(())
    }

    /// The node name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for Nodename {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host(s: &str) -> Hostname {
        Hostname::new(s).unwrap()
    }

    #[test]
    fn short_hostname_is_default_nodename() {
        let spec = NodenameSpec {
            hostname: host("worker-1.cluster.local"),
            override_name: None,
            register_with_fqdn: false,
        };
        assert_eq!(spec.reconcile().unwrap().as_str(), "worker-1");
    }

    #[test]
    fn fqdn_registration_uses_full_hostname() {
        let spec = NodenameSpec {
            hostname: host("worker-1.cluster.local"),
            override_name: None,
            register_with_fqdn: true,
        };
        assert_eq!(spec.reconcile().unwrap().as_str(), "worker-1.cluster.local");
    }

    #[test]
    fn override_wins_and_is_normalized() {
        let spec = NodenameSpec {
            hostname: host("worker-1"),
            override_name: Some("Custom-Node".to_string()),
            register_with_fqdn: true,
        };
        assert_eq!(spec.reconcile().unwrap().as_str(), "custom-node");
    }

    #[test]
    fn invalid_nodenames_rejected() {
        assert!(Nodename::new("").is_err());
        assert!(Nodename::new("-bad").is_err());
        assert!(Nodename::new("bad-").is_err());
        assert!(Nodename::new("under_score").is_err());
        assert!(Nodename::new("a..b").is_err());
        assert!(Nodename::new("x".repeat(254)).is_err());
    }

    #[test]
    fn valid_nodename_with_dots_and_digits() {
        assert_eq!(
            Nodename::new("Node-7.EXAMPLE.com").unwrap().as_str(),
            "node-7.example.com"
        );
    }
}
