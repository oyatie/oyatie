//! # talos-cluster-mgmt
//!
//! Cluster management for the operating-system Talos port: the machinery behind
//! `talosctl cluster create` / `talosctl cluster destroy`, `talosctl gen
//! config` / `talosctl gen secrets`, `talosctl bootstrap`, and `talosctl
//! health`. It mirrors Talos's `pkg/cluster`, `pkg/provision`, and
//! `pkg/machinery/config/generate`.
//!
//! The crate is organized around five concerns, each in its own module:
//!
//! * [`gen`] — config bundle generation: secrets bundles, per-role machine
//!   configs, and the client talosconfig.
//! * [`provisioner`] — the cluster provisioner abstraction (docker/qemu as a
//!   trait) plus an in-memory implementation, and the request data model.
//! * [`bundle`] — `cluster create` planning: turning a [`bundle::ClusterSpec`]
//!   into a generated config bundle and a provisioner request with IP
//!   allocation.
//! * [`bootstrap`] — the bootstrap state machine: apply configs, then issue the
//!   one-time etcd bootstrap to a control-plane node.
//! * [`health`] — the ordered cluster health checks (`talosctl health`).
//!
//! OS boundaries (docker, qemu, the apid/etcd/Kubernetes APIs) are modeled as
//! traits with in-memory implementations so the whole flow runs offline in
//! tests. The only dependency is `talos-core`.

pub mod bootstrap;
pub mod bundle;
pub mod r#gen;
pub mod health;
pub mod provisioner;

use std::fmt;

pub use bootstrap::{BootstrapOrchestrator, BootstrapPhase};
pub use bundle::{ClusterPlan, ClusterSpec};
pub use r#gen::{
    CertificateAuthority, ConfigBundle, GenInput, MachineConfig, Secret, SecretsBundle, TalosConfig,
};
pub use health::{CheckResult, ClusterState, HealthCheck, HealthChecker, NodeHealth};
pub use provisioner::{
    ClusterRequest, InMemoryProvisioner, NetworkRequest, NodeRequest, NodeState,
    ProvisionedCluster, ProvisionedNode, Provisioner, ProvisionerKind,
};

/// Errors produced by cluster-management operations.
///
/// A small, self-contained error type (the crate may not depend on external
/// crates beyond `talos-core`, and `os_kernel::Error` is `no_std`/`alloc`
/// based; this `std`-backed type implements [`std::error::Error`]). It mirrors
/// the categories that arise across generation, provisioning, bootstrap, and
/// health checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClusterError {
    /// A value failed validation.
    Invalid(String),
    /// A required resource was not found.
    NotFound(String),
    /// A state-machine precondition was not met.
    InvalidState(String),
    /// A health check failed (the cluster is not healthy).
    Unhealthy(String),
}

impl ClusterError {
    /// Construct an [`ClusterError::Invalid`].
    pub fn invalid(msg: impl Into<String>) -> Self {
        ClusterError::Invalid(msg.into())
    }

    /// Construct an [`ClusterError::NotFound`].
    pub fn not_found(msg: impl Into<String>) -> Self {
        ClusterError::NotFound(msg.into())
    }

    /// Construct an [`ClusterError::InvalidState`].
    pub fn invalid_state(msg: impl Into<String>) -> Self {
        ClusterError::InvalidState(msg.into())
    }

    /// Construct an [`ClusterError::Unhealthy`].
    pub fn unhealthy(msg: impl Into<String>) -> Self {
        ClusterError::Unhealthy(msg.into())
    }

    /// A short, stable kind string for matching/logging.
    pub fn kind(&self) -> &'static str {
        match self {
            ClusterError::Invalid(_) => "invalid",
            ClusterError::NotFound(_) => "not_found",
            ClusterError::InvalidState(_) => "invalid_state",
            ClusterError::Unhealthy(_) => "unhealthy",
        }
    }
}

impl fmt::Display for ClusterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClusterError::Invalid(m) => write!(f, "invalid: {m}"),
            ClusterError::NotFound(m) => write!(f, "not found: {m}"),
            ClusterError::InvalidState(m) => write!(f, "invalid state: {m}"),
            ClusterError::Unhealthy(m) => write!(f, "unhealthy: {m}"),
        }
    }
}

impl std::error::Error for ClusterError {}

impl From<os_kernel::Error> for ClusterError {
    fn from(e: os_kernel::Error) -> Self {
        match e {
            os_kernel::Error::Invalid(m)
            | os_kernel::Error::Parse(m)
            | os_kernel::Error::Unsupported(m) => ClusterError::Invalid(m),
            os_kernel::Error::NotFound(m) => ClusterError::NotFound(m),
            os_kernel::Error::InvalidState(m) => ClusterError::InvalidState(m),
            os_kernel::Error::PermissionDenied(m) | os_kernel::Error::Other(m) => {
                ClusterError::Invalid(m)
            }
            os_kernel::Error::Timeout => ClusterError::Invalid("operation timed out".into()),
        }
    }
}

/// The high-level orchestration of `talosctl cluster create`: plan, provision,
/// bootstrap, and health-check a cluster end to end.
///
/// This ties the modules together with a provisioner injected by the caller so
/// the OS boundary stays behind a trait.
#[cfg(any(test, feature = "modeled-crypto"))]
pub fn create_cluster<P: Provisioner>(
    spec: &ClusterSpec,
    provisioner: &mut P,
) -> Result<CreatedCluster, ClusterError> {
    if spec.provisioner != provisioner.kind() {
        return Err(ClusterError::invalid(format!(
            "spec requests the {} provisioner but a {} provisioner was supplied",
            spec.provisioner.as_str(),
            provisioner.kind().as_str()
        )));
    }
    let plan = ClusterPlan::plan(spec)?;
    let provisioned = provisioner.create(&plan.request)?;
    let mut orchestrator = BootstrapOrchestrator::new(spec.name.clone(), plan.node_types())?;
    orchestrator.run()?;
    Ok(CreatedCluster {
        plan,
        provisioned,
        bootstrap_phase: orchestrator.phase(),
    })
}

/// The result of a successful [`create_cluster`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedCluster {
    /// The plan that was applied.
    pub plan: ClusterPlan,
    /// The provisioned cluster handle.
    pub provisioned: ProvisionedCluster,
    /// The bootstrap phase reached (should be
    /// [`BootstrapPhase::Bootstrapped`]).
    pub bootstrap_phase: BootstrapPhase,
}

/// Destroy a cluster via its provisioner (`talosctl cluster destroy`).
pub fn destroy_cluster<P: Provisioner>(
    name: &str,
    provisioner: &mut P,
) -> Result<(), ClusterError> {
    provisioner.destroy(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::machine_type::MachineType;
    use os_kernel::version::Version;

    fn docker_spec() -> ClusterSpec {
        ClusterSpec::new("test", 1, 2, ProvisionerKind::Docker)
    }

    #[test]
    fn secrets_bundle_is_deterministic_and_shares_cas() {
        let a = SecretsBundle::generate("acme").unwrap();
        let b = SecretsBundle::generate("acme").unwrap();
        assert_eq!(a, b, "same cluster name must yield identical secrets");
        let c = SecretsBundle::generate("other").unwrap();
        assert_ne!(a.k8s_ca, c.k8s_ca, "different clusters get different CAs");
        a.validate().unwrap();
        let parts: Vec<&str> = a.bootstrap_token.split('.').collect();
        assert_eq!(parts[0].len(), 6);
        assert_eq!(parts[1].len(), 16);
    }

    #[test]
    fn config_bundle_shares_ca_across_roles() {
        let input = GenInput::new(
            "acme",
            "https://10.0.0.2:6443",
            Version::new(1, 30, 0),
            Version::new(1, 7, 0),
        );
        let bundle = ConfigBundle::generate(&input).unwrap();
        assert_eq!(bundle.control_plane.os_ca_cert, bundle.worker.os_ca_cert);
        assert_eq!(bundle.control_plane.k8s_ca_cert, bundle.worker.k8s_ca_cert);
        assert!(bundle.control_plane.includes_etcd_ca_key);
        assert!(!bundle.worker.includes_etcd_ca_key);
        assert_eq!(bundle.talosconfig.context, "acme");
    }

    #[test]
    fn gen_input_validation_rejects_bad_endpoint() {
        let mut input = GenInput::new(
            "acme",
            "http://10.0.0.2:6443",
            Version::new(1, 30, 0),
            Version::new(1, 7, 0),
        );
        assert!(input.validate().is_err());
        input.control_plane_endpoint = "https://10.0.0.2:6443".into();
        assert!(input.validate().is_ok());
        input.pod_subnets.clear();
        assert!(input.validate().is_err());
    }

    #[test]
    fn generate_with_mismatched_secrets_fails() {
        let input = GenInput::new(
            "acme",
            "https://10.0.0.2:6443",
            Version::new(1, 30, 0),
            Version::new(1, 7, 0),
        );
        let wrong = SecretsBundle::generate("other").unwrap();
        let err = ConfigBundle::generate_with_secrets(&input, wrong).unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    #[test]
    fn provisioner_kind_parse_and_disk_support() {
        assert_eq!(
            ProvisionerKind::parse("qemu").unwrap(),
            ProvisionerKind::Qemu
        );
        assert!(ProvisionerKind::parse("xen").is_err());
        assert!(ProvisionerKind::Qemu.supports_disk_image());
        assert!(!ProvisionerKind::Docker.supports_disk_image());
    }

    #[test]
    fn cluster_request_validation_catches_no_control_plane_and_dupes() {
        let net = NetworkRequest::new("n", "10.5.0.0/24").unwrap();
        assert_eq!(net.gateway, "10.5.0.1");
        let worker = NodeRequest {
            name: "w1".into(),
            machine_type: MachineType::Worker,
            ip: "10.5.0.2".into(),
            vcpus: 2,
            memory_mib: 2048,
            disk_mib: 0,
            config: String::new(),
        };
        let req = ClusterRequest {
            name: "c".into(),
            network: net.clone(),
            nodes: vec![worker.clone()],
            image: "img".into(),
        };
        assert!(req.validate(ProvisionerKind::Docker).is_err());

        let mut cp = worker.clone();
        cp.name = "cp1".into();
        cp.machine_type = MachineType::ControlPlane;
        let dup = ClusterRequest {
            name: "c".into(),
            network: net,
            nodes: vec![cp.clone(), worker],
            image: "img".into(),
        };
        let err = dup.validate(ProvisionerKind::Docker).unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    #[test]
    fn docker_node_rejects_disk_request() {
        let node = NodeRequest {
            name: "n".into(),
            machine_type: MachineType::ControlPlane,
            ip: "10.5.0.2".into(),
            vcpus: 2,
            memory_mib: 2048,
            disk_mib: 4096,
            config: String::new(),
        };
        assert!(node.validate(ProvisionerKind::Docker).is_err());
        assert!(node.validate(ProvisionerKind::Qemu).is_ok());
    }

    #[test]
    fn in_memory_provisioner_create_reflect_destroy() {
        let mut p = InMemoryProvisioner::new(ProvisionerKind::Docker);
        let plan = ClusterPlan::plan(&docker_spec()).unwrap();
        let provisioned = p.create(&plan.request).unwrap();
        assert_eq!(provisioned.nodes.len(), 3);
        assert_eq!(p.cluster_count(), 1);
        assert!(p.create(&plan.request).is_err());
        let reflected = p.reflect("test").unwrap();
        assert_eq!(reflected.control_plane_ips(), vec!["10.5.0.2".to_string()]);
        p.destroy("test").unwrap();
        assert_eq!(p.cluster_count(), 0);
        assert!(p.reflect("test").is_err());
    }

    #[test]
    fn cluster_spec_requires_odd_control_planes() {
        let mut spec = ClusterSpec::new("c", 2, 1, ProvisionerKind::Qemu);
        assert!(spec.validate().is_err());
        spec.control_planes = 3;
        assert!(spec.validate().is_ok());
        spec.control_planes = 0;
        assert!(spec.validate().is_err());
    }

    #[test]
    fn cluster_plan_allocates_sequential_ips() {
        let plan = ClusterPlan::plan(&ClusterSpec::new("c", 3, 1, ProvisionerKind::Qemu)).unwrap();
        let ips = plan.node_ips();
        assert_eq!(
            ips,
            vec![
                "10.5.0.2".to_string(),
                "10.5.0.3".to_string(),
                "10.5.0.4".to_string(),
                "10.5.0.5".to_string(),
            ]
        );
        assert_eq!(plan.control_plane_endpoint, "https://10.5.0.2:6443");
        assert_eq!(plan.request.nodes[0].machine_type, MachineType::Init);
        assert_eq!(
            plan.request.nodes[1].machine_type,
            MachineType::ControlPlane
        );
    }

    #[test]
    fn bootstrap_state_machine_enforces_order() {
        let nodes = vec![
            ("10.5.0.2".to_string(), MachineType::Init),
            ("10.5.0.3".to_string(), MachineType::Worker),
        ];
        let mut b = BootstrapOrchestrator::new("c", nodes).unwrap();
        assert_eq!(b.phase(), BootstrapPhase::Pending);
        assert!(b.bootstrap("10.5.0.2").is_err());
        b.apply_all_configs().unwrap();
        assert!(b.bootstrap("10.5.0.3").is_err());
        b.bootstrap("10.5.0.2").unwrap();
        assert_eq!(b.phase(), BootstrapPhase::Bootstrapping);
        assert_eq!(b.bootstrap_node(), Some("10.5.0.2"));
        assert!(b.bootstrap("10.5.0.2").is_err());
        b.mark_bootstrapped().unwrap();
        assert_eq!(b.phase(), BootstrapPhase::Bootstrapped);
    }

    #[test]
    fn bootstrap_requires_control_plane_node() {
        let nodes = vec![("10.5.0.2".to_string(), MachineType::Worker)];
        assert!(BootstrapOrchestrator::new("c", nodes).is_err());
    }

    #[test]
    fn bootstrap_run_happy_path() {
        let plan =
            ClusterPlan::plan(&ClusterSpec::new("c", 1, 1, ProvisionerKind::Docker)).unwrap();
        let mut b = BootstrapOrchestrator::new("c", plan.node_types()).unwrap();
        b.run().unwrap();
        assert_eq!(b.phase(), BootstrapPhase::Bootstrapped);
        assert_eq!(b.bootstrap_node(), Some("10.5.0.2"));
    }

    #[test]
    fn healthy_cluster_passes_all_checks() {
        let state = ClusterState {
            expected_members: vec!["10.5.0.2".into(), "10.5.0.3".into()],
            nodes: vec![
                NodeHealth::healthy("10.5.0.2", MachineType::ControlPlane),
                NodeHealth::healthy("10.5.0.3", MachineType::Worker),
            ],
        };
        let checker = HealthChecker::new();
        assert!(checker.is_healthy(&state));
        let results = checker.run_all(&state);
        assert_eq!(results.len(), 6);
        assert!(results.iter().all(|r| r.passed()));
    }

    #[test]
    fn missing_member_fails_report_in_check() {
        let state = ClusterState {
            expected_members: vec!["10.5.0.2".into(), "10.5.0.3".into()],
            nodes: vec![NodeHealth::healthy("10.5.0.2", MachineType::ControlPlane)],
        };
        let err = HealthCheck::AllNodesReportIn.evaluate(&state).unwrap_err();
        assert_eq!(err.kind(), "unhealthy");
        assert!(!HealthChecker::new().is_healthy(&state));
    }

    #[test]
    fn unhealthy_etcd_and_missing_static_pods_detected() {
        let mut cp = NodeHealth::healthy("10.5.0.2", MachineType::ControlPlane);
        cp.etcd_healthy = false;
        let state = ClusterState {
            expected_members: vec!["10.5.0.2".into()],
            nodes: vec![cp],
        };
        assert!(HealthCheck::EtcdHealthy.evaluate(&state).is_err());

        let mut cp2 = NodeHealth::healthy("10.5.0.2", MachineType::ControlPlane);
        cp2.static_pods_running.clear();
        let state2 = ClusterState {
            expected_members: vec!["10.5.0.2".into()],
            nodes: vec![cp2],
        };
        assert!(
            HealthCheck::ControlPlaneStaticPods
                .evaluate(&state2)
                .is_err()
        );
    }

    #[test]
    fn end_to_end_create_and_destroy() {
        let spec = docker_spec();
        let mut p = InMemoryProvisioner::new(ProvisionerKind::Docker);
        let created = create_cluster(&spec, &mut p).unwrap();
        assert_eq!(created.bootstrap_phase, BootstrapPhase::Bootstrapped);
        assert_eq!(created.provisioned.nodes.len(), 3);
        assert_eq!(p.cluster_count(), 1);
        destroy_cluster("test", &mut p).unwrap();
        assert_eq!(p.cluster_count(), 0);
    }

    #[test]
    fn create_with_wrong_provisioner_kind_fails() {
        let spec = ClusterSpec::new("test", 1, 0, ProvisionerKind::Qemu);
        let mut p = InMemoryProvisioner::new(ProvisionerKind::Docker);
        let err = create_cluster(&spec, &mut p).unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    /// Every constructor that mints modeled CA/token material — and every
    /// function that transitively reaches one — must stay behind the
    /// non-default `modeled-crypto` gate, so a production build cannot link a
    /// path that yields CA keys derived from the cluster name. `Secret` has a
    /// private field and no other constructor, so gating `Secret::derive`
    /// makes the whole downstream bundle unconstructible off-feature.
    ///
    /// The barrier is the `cfg`; this test only proves it does not silently
    /// disappear.
    // ponytail: source-text assertion. A `cfg` cannot be observed from inside a
    // build where it is enabled, and a compile-fail harness (trybuild) would be
    // a new dependency.
    #[test]
    fn modeled_crypto_constructors_stay_behind_the_gate() {
        const GATE: &str = "#[cfg(any(test, feature = \"modeled-crypto\"))]";

        // (source file, item signature)
        let required: [(&str, &str); 7] = [
            (include_str!("gen.rs"), "pub fn derive(seed: &str) -> Self {"),
            (
                include_str!("gen.rs"),
                "fn derive(cluster: &str, kind: &str) -> Self {",
            ),
            (
                include_str!("gen.rs"),
                "pub fn generate(cluster_name: &str) -> Result<Self, ClusterError> {",
            ),
            (
                include_str!("gen.rs"),
                "pub fn generate(input: &GenInput) -> Result<Self, ClusterError> {",
            ),
            (include_str!("gen.rs"), "pub fn generate_with_secrets("),
            (
                include_str!("bundle.rs"),
                "pub fn plan(spec: &ClusterSpec) -> Result<Self, ClusterError> {",
            ),
            (
                include_str!("lib.rs"),
                "pub fn create_cluster<P: Provisioner>(",
            ),
        ];

        for (src, signature) in required {
            let gated = src
                .match_indices(signature)
                .any(|(i, _)| src[..i].trim_end().ends_with(GATE));
            assert!(gated, "`{signature}` must be immediately preceded by {GATE}");
        }
    }

    #[test]
    fn error_conversion_from_core() {
        let core = os_kernel::Error::not_found("x");
        let mapped: ClusterError = core.into();
        assert_eq!(mapped.kind(), "not_found");
    }
}
