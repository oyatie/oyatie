// This crate exposes many small accessors and fallible constructors that mirror
// talos-core's conventions. The following pedantic lints would require
// annotating dozens of trivial methods without making the API clearer, so we
// opt out crate-wide rather than littering per-item attributes:
//   - `must_use_candidate`: pure accessors and builders where ignoring the
//     result is already an obvious no-op.
//   - `missing_errors_doc`: the `Result`-returning functions document their
//     failure modes inline; a separate `# Errors` section adds noise here.
#![allow(clippy::must_use_candidate, clippy::missing_errors_doc)]

//! talos-kubelet
//!
//! Models the kubelet domain as Talos runs it, mirroring the
//! `internal/app/machined/pkg/controllers/k8s` kubelet controllers and
//! `pkg/kubelet`:
//!
//! - [`config`]: the validated kubelet configuration (cluster DNS, cluster
//!   domain, cgroup driver, extra args/mounts, credential providers) that backs
//!   the kubelet's `config.yaml`.
//! - [`nodename`]: the nodename controller, deriving the registration node name
//!   from the host's hostname (or an override).
//! - [`node_ip`]: node IP selection through `validSubnets`, plus node labels and
//!   taints.
//! - [`spec`]: the `KubeletSpecController` rendering of the process command line
//!   and config file.
//! - [`service`]: the kubelet lifecycle and its CSR-approval bootstrap, with OS
//!   boundaries (process supervision, the CSR API) modeled as traits.
//!
//! OS boundaries are traits with in-memory implementations used by the tests, so
//! the whole crate builds and tests fully offline with no external crates.

pub mod config;
pub mod node_ip;
pub mod nodename;
pub mod service;
pub mod spec;

pub use config::{
    CgroupDriver, CredentialProvider, DEFAULT_CGROUP_DRIVER, DEFAULT_CLUSTER_DOMAIN, ExtraMount,
    KubeletConfig, PROTECTED_ARGS,
};
pub use node_ip::{NodeIpSpec, NodeLabel, NodeTaint, SubnetFilter, TaintEffect};
pub use nodename::{Nodename, NodenameSpec};
pub use service::{
    BootstrapPhase, CsrApprover, CsrState, InMemoryCsrApprover, InMemorySupervisor, KubeletService,
    ProcessSupervisor,
};
pub use spec::KubeletSpec;

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::address::{Hostname, NodeAddress};
    use os_kernel::traits::Runnable;

    /// End-to-end: hostname -> nodename -> node IP -> spec -> service bootstrap.
    #[test]
    fn end_to_end_control_plane_bootstrap() {
        let nodename = NodenameSpec {
            hostname: Hostname::new("cp-1.cluster.local").unwrap(),
            override_name: None,
            register_with_fqdn: false,
        }
        .reconcile()
        .unwrap();
        assert_eq!(nodename.as_str(), "cp-1");

        let node_ips = NodeIpSpec {
            candidates: vec![
                NodeAddress::parse_v4("127.0.0.1").unwrap(),
                NodeAddress::parse_v4("10.0.0.10").unwrap(),
            ],
            valid_subnets: vec![SubnetFilter::parse("10.0.0.0/8").unwrap()],
        }
        .reconcile()
        .unwrap();
        assert_eq!(node_ips, vec![NodeAddress::parse_v4("10.0.0.10").unwrap()]);

        let cfg = KubeletConfig::with_dns_from_service_cidr("10.96.0.0/12")
            .unwrap()
            .with_extra_arg("max-pods", "220")
            .unwrap();

        let spec =
            KubeletSpec::render(&cfg, &nodename, &node_ips, &[NodeTaint::control_plane()]).unwrap();
        assert_eq!(spec.flag_value("node-ip"), Some("10.0.0.10"));
        assert!(spec.has_flag("register-with-taints"));

        let mut svc = KubeletService::new(
            spec,
            InMemorySupervisor::default(),
            InMemoryCsrApprover::new(true),
        );
        svc.start().unwrap();
        assert!(svc.is_ready());
    }

    #[test]
    fn worker_bootstrap_without_taints_pending_until_approved() {
        let nodename = Nodename::new("worker-9").unwrap();
        let cfg = KubeletConfig::with_dns_from_service_cidr("10.96.0.0/12").unwrap();
        let spec = KubeletSpec::render(
            &cfg,
            &nodename,
            &[NodeAddress::parse_v4("10.0.0.9").unwrap()],
            &[],
        )
        .unwrap();
        assert!(!spec.has_flag("register-with-taints"));

        let mut svc = KubeletService::new(
            spec,
            InMemorySupervisor::default(),
            InMemoryCsrApprover::new(false),
        );
        svc.bootstrap().unwrap();
        assert_eq!(svc.phase(), BootstrapPhase::AwaitingApproval);
        let csr = svc.csr_id().unwrap().to_string();
        svc.approver_mut().approve(&csr).unwrap();
        svc.reconcile_bootstrap();
        assert!(svc.is_ready());
    }
}
