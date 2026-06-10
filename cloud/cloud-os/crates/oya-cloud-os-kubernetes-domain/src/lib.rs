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
