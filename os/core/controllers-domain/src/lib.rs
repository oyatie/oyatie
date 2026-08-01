//! # talos-controllers
//!
//! The COSI controller runtime and the controller domains owned by machined
//! that are not delegated to a dedicated crate. Mirrors Talos's
//! `internal/app/machined/pkg/controllers` together with the
//! `cosi-project/runtime` `Controller`/`Runtime` abstractions.
//!
//! ## Layout
//!
//! - [`reconcile`]: the [`Controller`](reconcile::Controller) trait, declared
//!   [`Input`](reconcile::Input)/[`Output`](reconcile::Output)s, and the
//!   [`ReconcileContext`](reconcile::ReconcileContext) through which controllers
//!   read inputs and write outputs into the shared COSI store.
//! - [`runtime`]: the [`ControllerRuntime`](runtime::ControllerRuntime) that
//!   owns the store, routes input changes to dependent controllers, and drives
//!   reconcile ticks to a stable point.
//! - [`machine_status`]: the runtime `MachineStatus` aggregation controller.
//! - [`kernel_param`]: the runtime sysctl/sysfs kernel-parameter controllers,
//!   with the [`KernelWriter`](kernel_param::KernelWriter) OS boundary.
//! - [`config`]: config acquire/apply and the derived machine/cluster config
//!   views.
//! - [`registry`]: wiring of all domains into a runtime via
//!   [`RegistryBuilder`](registry::RegistryBuilder).
//!
//! OS boundaries (writing kernel parameters) are modeled as traits with
//! in-memory implementations so the whole crate is testable offline.

// This crate is dominated by small COSI resource/controller types with trivial
// pure accessors and `Result`-returning reconcile entry points. The following
// pedantic lints would require annotating dozens of such methods without making
// the API clearer, so we opt out crate-wide (matching `talos-core`) rather than
// littering per-item attributes:
//   - `must_use_candidate` / `return_self_not_must_use`: pure accessors and
//     builders where ignoring the result is already an obvious no-op.
//   - `missing_errors_doc` / `missing_panics_doc`: the fallible functions
//     document their failure modes inline, and the `unwrap`s are on
//     statically-valid resource ids; separate sections add only noise here.
#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]

pub mod config;
pub mod kernel_param;
pub mod machine_status;
pub mod network;
pub mod reconcile;
pub mod registry;
pub mod runtime;

pub use config::{
    AcquiredConfig, ClusterConfig, ClusterConfigController, ConfigAcquireController, ConfigSource,
    MachineConfig, MachineConfigDocument, MachineConfigSpec,
};
pub use kernel_param::{
    InMemoryKernel, KernelParamConfig, KernelParamController, KernelParamStatus, KernelWriter,
    ParamKind,
};
pub use machine_status::{
    MachineStage, MachineStatus, MachineStatusController, StageReport, UnmetCondition,
};
pub use network::{
    AddressMergeController, AddressSpecResource, HostnameMergeController, HostnameSpecResource,
    LinkConfigController, LinkMergeController, LinkSpecResource, LinkStatusSourceController,
    MergedAddressSpecResource, MergedHostnameSpecResource, MergedLinkSpecResource,
    MergedResolverSpecResource, MergedRouteSpecResource, OperatorConfigController,
    OperatorConfigSpecResource, OperatorMergeController, OperatorResultBridgeController,
    OperatorResultResource, OperatorSpecResource, ResolverConfigController,
    ResolverMergeController, ResolverSpecResource, RouteMergeController, RouteSpecResource,
    default_dhcp_link_status_projection_fingerprint, machine_config_link_projection_fingerprint,
};
pub use reconcile::{
    Controller, Input, InputKind, Output, ReconcileContext, ReconcileError, ReconcileResult,
};
pub use registry::{Domain, RegistryBuilder};
pub use runtime::{ControllerRuntime, TickReport};

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::MachineType;

    /// A whole-machine smoke test: register every domain and drive a boot from
    /// raw config to a ready machine status.
    #[test]
    fn full_runtime_boot_to_ready() {
        let kernel = InMemoryKernel::new().with_path("/proc/sys/net/ipv4/ip_forward", "0");
        let mut rt = RegistryBuilder::with_kernel(kernel)
            .with_link_status_source(|| Ok(Vec::new()))
            .all_domains()
            .build();
        assert_eq!(rt.controller_count(), 15);

        // Seed acquired config, a kernel param, and a running stage report.
        let spec = MachineConfigSpec {
            machine_type: MachineType::ControlPlane,
            cluster_name: "talos".into(),
            control_plane_endpoint: "https://10.0.0.1:6443".into(),
            kubernetes_version: "1.30.0".into(),
        };
        rt.state_mut()
            .create(Box::new(AcquiredConfig::new(ConfigSource::Disk, spec)))
            .unwrap();
        rt.state_mut()
            .create(Box::new(
                KernelParamConfig::sysctl("net.ipv4.ip_forward", "1").unwrap(),
            ))
            .unwrap();
        rt.state_mut()
            .create(Box::new(StageReport::new(MachineStage::Running, "booted")))
            .unwrap();

        rt.run_until_stable(8).unwrap();

        assert!(rt.state().contains("config/MachineConfig/v1alpha1"));
        assert!(rt.state().contains("config/ClusterConfig/cluster"));
        assert!(
            rt.state()
                .contains("runtime/KernelParamStatus/net.ipv4.ip_forward")
        );
        let ms = rt.state().get("runtime/MachineStatus/machine").unwrap();
        assert_eq!(ms.spec_fingerprint(), "stage=running;ready=true;unmet=[]");
    }

    #[test]
    fn unmet_condition_keeps_machine_not_ready() {
        let mut rt = RegistryBuilder::in_memory()
            .domain(Domain::RuntimeMachineStatus)
            .build();
        rt.state_mut()
            .create(Box::new(StageReport::new(MachineStage::Running, "booted")))
            .unwrap();
        rt.state_mut()
            .create(Box::new(UnmetCondition::new("etcd", "not a leader yet")))
            .unwrap();
        rt.run_until_stable(3).unwrap();
        let ms = rt.state().get("runtime/MachineStatus/machine").unwrap();
        assert_eq!(
            ms.spec_fingerprint(),
            "stage=running;ready=false;unmet=[etcd]"
        );
    }
}
