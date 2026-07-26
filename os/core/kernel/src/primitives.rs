//! Composite primitives built from the smaller newtypes: a node identity
//! aggregate and a label map, mirroring common Talos machinery values.

use crate::address::{Hostname, NodeAddress};
use crate::error::{Error, Result};
use crate::machine_type::MachineType;
use crate::platform::Platform;
use crate::version::Version;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

/// A small, validated key/value label map (Kubernetes-style).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Labels {
    inner: BTreeMap<String, String>,
}

impl Labels {
    /// An empty label map.
    pub fn new() -> Self {
        Labels {
            inner: BTreeMap::new(),
        }
    }

    /// Insert a label after validating its key. Keys must be non-empty and
    /// contain only `[A-Za-z0-9._/-]`.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) -> Result<()> {
        let key = key.into();
        if key.is_empty() {
            return Err(Error::invalid("label key is empty"));
        }
        for c in key.chars() {
            if !(c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '/' | '-')) {
                return Err(Error::invalid(alloc::format!(
                    "invalid label key character '{c}'"
                )));
            }
        }
        self.inner.insert(key, value.into());
        Ok(())
    }

    /// Look up a label value.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(String::as_str)
    }

    /// Number of labels.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether the map is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Whether every key/value in `selector` is present and equal here.
    pub fn matches(&self, selector: &Labels) -> bool {
        selector
            .inner
            .iter()
            .all(|(k, v)| self.inner.get(k) == Some(v))
    }

    /// Render as a stable, sorted `k=v` comma-joined string.
    pub fn to_selector_string(&self) -> String {
        let parts: Vec<String> = self
            .inner
            .iter()
            .map(|(k, v)| alloc::format!("{k}={v}"))
            .collect();
        parts.join(",")
    }
}

/// An aggregate describing a single node's identity, combining several
/// primitive newtypes. This is the value most subsystems thread through their
/// APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeIdentity {
    /// The node's hostname.
    pub hostname: Hostname,
    /// Its primary address.
    pub address: NodeAddress,
    /// Whether it is a control-plane or worker node.
    pub machine_type: MachineType,
    /// The platform it runs on.
    pub platform: Platform,
    /// The Talos OS version it reports.
    pub os_version: Version,
    /// Arbitrary labels.
    pub labels: Labels,
}

impl NodeIdentity {
    /// Construct a node identity with empty labels.
    pub fn new(
        hostname: Hostname,
        address: NodeAddress,
        machine_type: MachineType,
        platform: Platform,
        os_version: Version,
    ) -> Self {
        NodeIdentity {
            hostname,
            address,
            machine_type,
            platform,
            os_version,
            labels: Labels::new(),
        }
    }

    /// Validate cross-field invariants. For example, an `Unknown` machine type
    /// is never a valid joined node identity.
    pub fn validate(&self) -> Result<()> {
        if self.machine_type == MachineType::Unknown {
            return Err(Error::invalid_state("node has unknown machine type"));
        }
        if self.platform == Platform::Unknown {
            return Err(Error::invalid_state("node has unknown platform"));
        }
        Ok(())
    }

    /// Whether this node is eligible to run control-plane components.
    pub fn is_control_plane(&self) -> bool {
        self.machine_type.is_control_plane()
    }

    /// A stable, human-friendly identity string.
    pub fn display_name(&self) -> String {
        alloc::format!(
            "{}/{}/{}",
            self.hostname.as_str(),
            self.machine_type.as_str(),
            self.address
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_identity(mt: MachineType, plat: Platform) -> NodeIdentity {
        NodeIdentity::new(
            Hostname::new("cp-1.cluster.local").unwrap(),
            NodeAddress::parse_v4("10.0.0.5").unwrap(),
            mt,
            plat,
            Version::new(1, 8, 0),
        )
    }

    #[test]
    fn labels_validate_and_match() {
        let mut l = Labels::new();
        l.insert("node-role.kubernetes.io/control-plane", "")
            .unwrap();
        l.insert("topology.kubernetes.io/zone", "us-east-1a")
            .unwrap();
        assert_eq!(l.len(), 2);
        assert!(l.insert("bad key", "x").is_err());

        let mut sel = Labels::new();
        sel.insert("topology.kubernetes.io/zone", "us-east-1a")
            .unwrap();
        assert!(l.matches(&sel));

        sel.insert("topology.kubernetes.io/zone", "other").unwrap();
        assert!(!l.matches(&sel));
    }

    #[test]
    fn selector_string_is_sorted() {
        let mut l = Labels::new();
        l.insert("z", "1").unwrap();
        l.insert("a", "2").unwrap();
        assert_eq!(l.to_selector_string(), "a=2,z=1");
    }

    #[test]
    fn node_identity_validation() {
        let good = sample_identity(MachineType::ControlPlane, Platform::Aws);
        assert!(good.validate().is_ok());
        assert!(good.is_control_plane());

        let bad_type = sample_identity(MachineType::Unknown, Platform::Aws);
        assert_eq!(bad_type.validate().unwrap_err().kind(), "invalid_state");

        let bad_plat = sample_identity(MachineType::Worker, Platform::Unknown);
        assert!(bad_plat.validate().is_err());
        assert!(!bad_plat.is_control_plane());
    }

    #[test]
    fn display_name_format() {
        let id = sample_identity(MachineType::ControlPlane, Platform::Metal);
        assert_eq!(
            id.display_name(),
            "cp-1.cluster.local/controlplane/10.0.0.5"
        );
    }
}
