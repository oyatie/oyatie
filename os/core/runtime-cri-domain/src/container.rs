//! Static container metadata.
//!
//! Mirrors containerd's `Container` record: an id, the image it was created
//! from, the OCI runtime spec, the labels containerd indexes on, and the
//! snapshotter/runtime it was created with. A `Container` is inert metadata; a
//! [`crate::task::Task`] is the live process created from it.

use crate::image::ImageRef;
use crate::oci_spec::OciSpec;
use std::collections::BTreeMap;
use os_kernel::error::{Error, Result};

/// High-level status of a container record, derived from whether a task has
/// been created from it and that task's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerStatus {
    /// Metadata exists, no task created yet.
    Created,
    /// A task has been created and started.
    Running,
    /// The task exited (or was killed).
    Stopped,
}

/// A containerd container record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    /// Unique id within its namespace.
    pub id: String,
    /// The image reference this container was created from.
    pub image: ImageRef,
    /// The OCI runtime spec (`config.json`).
    pub spec: OciSpec,
    /// The snapshotter used for the rootfs (Talos uses `overlayfs`).
    pub snapshotter: String,
    /// The runtime shim (Talos uses `io.containerd.runc.v2`).
    pub runtime: String,
    /// Labels containerd indexes the container by.
    pub labels: BTreeMap<String, String>,
    /// Current high-level status.
    pub status: ContainerStatus,
}

impl Container {
    /// The default snapshotter Talos uses.
    pub const DEFAULT_SNAPSHOTTER: &'static str = "overlayfs";
    /// The default runtime shim Talos uses.
    pub const DEFAULT_RUNTIME: &'static str = "io.containerd.runc.v2";

    /// Create a container record. The id must be non-empty and the spec must
    /// validate. The image is required to be runnable-resolvable (any parsed
    /// reference); state-of-image checks live in the client.
    pub fn new(id: impl Into<String>, image: ImageRef, spec: OciSpec) -> Result<Self> {
        let id = id.into();
        if id.is_empty() {
            return Err(Error::invalid("container id must not be empty"));
        }
        // containerd ids are restricted to a safe charset.
        if !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        {
            return Err(Error::invalid("container id has invalid characters"));
        }
        spec.validate()?;
        Ok(Container {
            id,
            image,
            spec,
            snapshotter: Self::DEFAULT_SNAPSHOTTER.to_string(),
            runtime: Self::DEFAULT_RUNTIME.to_string(),
            labels: BTreeMap::new(),
            status: ContainerStatus::Created,
        })
    }

    /// Builder: attach a label.
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Builder: override the snapshotter.
    pub fn with_snapshotter(mut self, snapshotter: impl Into<String>) -> Self {
        self.snapshotter = snapshotter.into();
        self
    }

    /// Look up a label value.
    pub fn label(&self, key: &str) -> Option<&str> {
        self.labels.get(key).map(String::as_str)
    }

    /// Whether the container is currently considered running.
    pub fn is_running(&self) -> bool {
        self.status == ContainerStatus::Running
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oci_spec::OciSpec;

    fn spec() -> OciSpec {
        OciSpec::new(vec!["/usr/local/bin/etcd".to_string()]).unwrap()
    }

    #[test]
    fn create_sets_defaults() {
        let c = Container::new(
            "etcd",
            ImageRef::parse("registry.k8s.io/etcd:3.5").unwrap(),
            spec(),
        )
        .unwrap();
        assert_eq!(c.snapshotter, "overlayfs");
        assert_eq!(c.runtime, "io.containerd.runc.v2");
        assert_eq!(c.status, ContainerStatus::Created);
        assert!(!c.is_running());
    }

    #[test]
    fn empty_id_rejected() {
        assert!(Container::new("", ImageRef::parse("a/b:1").unwrap(), spec()).is_err());
    }

    #[test]
    fn invalid_id_chars_rejected() {
        assert!(Container::new("bad id", ImageRef::parse("a/b:1").unwrap(), spec()).is_err());
    }

    #[test]
    fn invalid_spec_propagates() {
        let bad = OciSpec::new(vec!["relative".to_string()]).unwrap();
        assert!(Container::new("c", ImageRef::parse("a/b:1").unwrap(), bad).is_err());
    }

    #[test]
    fn labels_round_trip() {
        let c = Container::new("c", ImageRef::parse("a/b:1").unwrap(), spec())
            .unwrap()
            .with_label("io.kubernetes.pod", "kube-system/etcd")
            .with_snapshotter("native");
        assert_eq!(c.label("io.kubernetes.pod"), Some("kube-system/etcd"));
        assert_eq!(c.snapshotter, "native");
        assert_eq!(c.label("missing"), None);
    }
}
