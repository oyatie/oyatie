//! # talos-kubernetes
//!
//! A port of the Talos Kubernetes subsystem, mirroring
//! `internal/app/machined/pkg/controllers/k8s` in `siderolabs/talos`.
//!
//! It models the data and logic that machined uses to bring up and supervise a
//! Kubernetes node:
//!
//! * [`kubelet`] — kubelet configuration and the kubelet service spec.
//! * [`static_pod`] — static pod manifest generation for control-plane
//!   components.
//! * [`control_plane`] — the desired configuration of the control plane
//!   (apiserver / controller-manager / scheduler) and per-component argument
//!   rendering.
//! * [`etcd`] — the etcd member spec used to bootstrap and join the cluster's
//!   datastore.
//! * [`manifests`] — individual cluster manifests (CNI, `CoreDNS`, kube-proxy,
//!   bootstrap RBAC, ...).
//! * [`bootstrap`] — the ordered collection of bootstrap manifests applied
//!   exactly once when the cluster is first brought up.
//! * [`config`] — node-level Kubernetes configuration shared by the controllers.
//! * [`secrets`] — the cryptographic material the control plane depends on.
//! * [`rendered`] — the rendered output of a controller (files written to disk),
//!   modeled behind a trait so the syscall boundary stays testable and
//!   dependency-free.
//!
//! Everything here is pure logic: where the real subsystem talks to containerd,
//! the API server, or the filesystem, we model the boundary as a trait with an
//! in-memory implementation used by the tests.

// These pedantic lints fire pervasively on this crate's data-modeling API
// surface and the documentation suggestions they push add noise without
// improving clarity, so they are allowed crate-wide.
#![allow(
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::return_self_not_must_use
)]

pub mod bootstrap;
pub mod config;
pub mod control_plane;
pub mod cri_static_pod;
pub mod encoding;
pub mod error;
pub mod etcd;
pub mod kubeconfig;
pub mod kubelet;
pub mod manifests;
pub mod pki;
pub mod provision;
pub mod rendered;
pub mod secrets;
pub mod static_pod;
pub mod templates;

pub use bootstrap::BootstrapManifests;
pub use config::{ClusterEndpoint, K8sConfig, NodeName};
pub use control_plane::{ControlPlaneConfig, K8sComponent};
pub use cri_static_pod::{
    StaticPodLaunchReport, launch_static_pod_on_cri, launch_static_pods_on_cri,
};
pub use encoding::{base64_standard, kubeconfig_data};
pub use error::{K8sError, Result};
pub use etcd::{EtcdMemberState, EtcdSpec};
pub use kubeconfig::{AuthInfo, KubeConfig};
pub use kubelet::{KubeletConfig, KubeletSpec};
pub use manifests::{Manifest, ManifestKind};
pub use pki::{
    CertAuthority, KeyUsage, LeafCert, SubjectAltName, apiserver_sans, control_plane_leaves,
};
pub use provision::provision_control_plane;
pub use rendered::{FileMode, InMemoryFileSink, RenderedFile, RenderedOutput};
pub use secrets::K8sSecrets;
pub use static_pod::{StaticPod, StaticPodPhase};
pub use templates::default_bootstrap_manifests;

/// The default Kubernetes version Talos targets when none is configured.
pub const DEFAULT_KUBERNETES_VERSION: &str = "1.30.0";

/// The directory kubelet watches for static pod manifests.
pub const STATIC_POD_PATH: &str = "/etc/kubernetes/manifests";

/// The namespace control-plane static pods run in.
pub const CONTROL_PLANE_NAMESPACE: &str = "kube-system";

#[cfg(test)]
mod tests {
    /// The modeled PKI crate must stay a DEV dependency of this one.
    ///
    /// This is the consumer half of the `os-secrets-domain` barrier, and it was
    /// the last unguarded route to the model. That crate's own guards cover its
    /// crate-root `cfg`, its `[features]` section and its BUCK targets; the
    /// modeled buck2 target's enumerated `visibility` names this package, and
    /// only the `rust_test` here depends on it. None of that constrains *this*
    /// manifest. Moving the entry below from `[dev-dependencies]` to
    /// `[dependencies]` puts the model in the production `os-kubernetes-domain`
    /// rlib for every `cargo build`, and buck2 stays green through it because
    /// buck2 deps come from BUCK and never consult `Cargo.toml` — the same
    /// blindness that made the `default`-feature hole invisible.
    ///
    /// The assertion is over the SECTION each declaration appears in, not over
    /// the presence of a string, which makes it anti-vacuous in both
    /// directions: deleting the dependency, renaming the section, or declaring
    /// it twice all produce a vector that is not exactly `["dev-dependencies"]`.
    ///
    /// Proven to fire, by mutation: moving the entry to `[dependencies]` gives
    ///
    /// ```text
    /// assertion `left == right` failed: os-secrets-domain models Talos PKI and
    /// must be declared ONLY under [dev-dependencies]
    ///   left: ["dependencies"]
    ///  right: ["dev-dependencies"]
    /// ```
    #[test]
    fn modeled_crate_is_declared_only_as_a_dev_dependency() {
        let mut section = "";
        let mut declared_in: Vec<&str> = Vec::new();

        for line in include_str!("../Cargo.toml").lines() {
            let line = line.trim();
            if line.starts_with('#') {
                continue;
            }
            match line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                Some(header) => section = header,
                None if line.starts_with("os-secrets-domain") => declared_in.push(section),
                None => {}
            }
        }

        assert_eq!(
            declared_in,
            ["dev-dependencies"],
            "os-secrets-domain models Talos PKI and must be declared ONLY under \
             [dev-dependencies]: every use of it in this crate is inside a \
             `#[cfg(test)] mod tests`, and a normal dependency would link the model \
             into the production rlib on every cargo build while buck2 stayed green"
        );
    }
}
