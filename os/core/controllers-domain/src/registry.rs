//! Controller registry wiring all domains into a [`ControllerRuntime`].
//!
//! Mirrors Talos `internal/app/machined/pkg/controllers`' top-level wiring,
//! where machined registers every controller domain (runtime, config, k8s,
//! network, ...) into the single COSI controller runtime. Here we register the
//! domains this crate owns and expose helpers to build a ready-to-run runtime.

use crate::config::{ClusterConfigController, ConfigAcquireController};
use crate::kernel_param::{InMemoryKernel, KernelParamController, KernelWriter};
use crate::machine_status::MachineStatusController;
use crate::network::{
    AddressMergeController, HostnameMergeController, LinkConfigController, LinkMergeController,
    LinkStatusSourceController, OperatorConfigController, OperatorMergeController,
    OperatorResultBridgeController, ResolverConfigController, ResolverMergeController,
    RouteMergeController,
};
use crate::runtime::ControllerRuntime;

/// The controller domains this crate provides, used to select what to register.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    /// `MachineStatus` aggregation.
    RuntimeMachineStatus,
    /// sysctl/sysfs kernel parameters.
    RuntimeKernelParam,
    /// Config acquire/apply.
    ConfigAcquire,
    /// Cluster-config derivation.
    ConfigCluster,
    /// Live kernel link-status source.
    NetworkLinkStatusSource,
    /// Config-derived network operator spec seeding.
    NetworkOperatorConfig,
    /// Network operator source-layer merge.
    NetworkOperatorMerge,
    /// Network address source-layer merge.
    NetworkAddressMerge,
    /// Network route source-layer merge.
    NetworkRouteMerge,
    /// Network hostname source-layer merge.
    NetworkHostnameMerge,
    /// Config-derived link/address/route spec seeding from LinkConfig/VLANConfig.
    NetworkLinkConfig,
    /// Network link source-layer merge.
    NetworkLinkMerge,
    /// Config-derived resolver spec seeding.
    NetworkResolverConfig,
    /// Network resolver source-layer merge.
    NetworkResolverMerge,
    /// Network operator-result bridge.
    NetworkOperatorBridge,
}

impl Domain {
    /// Every domain provided by this crate, in registration order.
    pub fn all() -> &'static [Domain] {
        &[
            Domain::ConfigAcquire,
            Domain::ConfigCluster,
            Domain::NetworkLinkStatusSource,
            Domain::NetworkOperatorConfig,
            Domain::NetworkOperatorMerge,
            Domain::NetworkOperatorBridge,
            Domain::NetworkAddressMerge,
            Domain::NetworkRouteMerge,
            Domain::NetworkHostnameMerge,
            Domain::NetworkLinkConfig,
            Domain::NetworkLinkMerge,
            Domain::NetworkResolverConfig,
            Domain::NetworkResolverMerge,
            Domain::RuntimeMachineStatus,
            Domain::RuntimeKernelParam,
        ]
    }

    /// The controller name a domain registers under.
    pub fn controller_name(&self) -> &'static str {
        match self {
            Domain::RuntimeMachineStatus => "runtime.MachineStatusController",
            Domain::RuntimeKernelParam => "runtime.KernelParamController",
            Domain::ConfigAcquire => "config.ConfigAcquireController",
            Domain::ConfigCluster => "config.ClusterConfigController",
            Domain::NetworkLinkStatusSource => "network.LinkStatusSourceController",
            Domain::NetworkOperatorConfig => "network.OperatorConfigController",
            Domain::NetworkOperatorMerge => "network.OperatorMergeController",
            Domain::NetworkAddressMerge => "network.AddressMergeController",
            Domain::NetworkRouteMerge => "network.RouteMergeController",
            Domain::NetworkHostnameMerge => "network.HostnameMergeController",
            Domain::NetworkLinkConfig => "network.LinkConfigController",
            Domain::NetworkLinkMerge => "network.LinkMergeController",
            Domain::NetworkResolverConfig => "network.ResolverConfigController",
            Domain::NetworkResolverMerge => "network.ResolverMergeController",
            Domain::NetworkOperatorBridge => "network.OperatorResultBridgeController",
        }
    }
}

/// Builder that assembles a [`ControllerRuntime`] from selected domains.
///
/// The kernel-param controller needs an OS write boundary; the builder takes a
/// [`KernelWriter`] so tests can inject an [`InMemoryKernel`].
pub struct RegistryBuilder<W: KernelWriter + 'static> {
    domains: Vec<Domain>,
    kernel: Option<W>,
}

impl<W: KernelWriter + 'static> RegistryBuilder<W> {
    /// Start a builder with an explicit kernel writer and no domains.
    pub fn with_kernel(kernel: W) -> Self {
        RegistryBuilder {
            domains: Vec::new(),
            kernel: Some(kernel),
        }
    }

    /// Add a domain to register (idempotent on duplicates).
    pub fn domain(mut self, d: Domain) -> Self {
        if !self.domains.contains(&d) {
            self.domains.push(d);
        }
        self
    }

    /// Add every domain this crate provides.
    pub fn all_domains(mut self) -> Self {
        for d in Domain::all() {
            if !self.domains.contains(d) {
                self.domains.push(*d);
            }
        }
        self
    }

    /// Consume the builder and produce a wired [`ControllerRuntime`].
    pub fn build(mut self) -> ControllerRuntime {
        let mut rt = ControllerRuntime::new();
        // Kernel writer can only be consumed once; the kernel-param domain takes
        // ownership of it.
        for d in &self.domains {
            match d {
                Domain::ConfigAcquire => rt.register(Box::new(ConfigAcquireController::new())),
                Domain::ConfigCluster => rt.register(Box::new(ClusterConfigController::new())),
                Domain::NetworkLinkStatusSource => {
                    rt.register(Box::new(LinkStatusSourceController::new()));
                }
                Domain::NetworkOperatorConfig => {
                    rt.register(Box::new(OperatorConfigController::new()));
                }
                Domain::NetworkOperatorMerge => {
                    rt.register(Box::new(OperatorMergeController::new()));
                }
                Domain::NetworkAddressMerge => {
                    rt.register(Box::new(AddressMergeController::new()));
                }
                Domain::NetworkRouteMerge => {
                    rt.register(Box::new(RouteMergeController::new()));
                }
                Domain::NetworkHostnameMerge => {
                    rt.register(Box::new(HostnameMergeController::new()));
                }
                Domain::NetworkLinkConfig => {
                    rt.register(Box::new(LinkConfigController::new()));
                }
                Domain::NetworkLinkMerge => {
                    rt.register(Box::new(LinkMergeController::new()));
                }
                Domain::NetworkResolverConfig => {
                    rt.register(Box::new(ResolverConfigController::new()));
                }
                Domain::NetworkResolverMerge => {
                    rt.register(Box::new(ResolverMergeController::new()));
                }
                Domain::RuntimeMachineStatus => {
                    rt.register(Box::new(MachineStatusController::new()))
                }
                Domain::RuntimeKernelParam => {
                    if let Some(kernel) = self.kernel.take() {
                        rt.register(Box::new(KernelParamController::new(kernel)));
                    }
                }
                Domain::NetworkOperatorBridge => {
                    rt.register(Box::new(OperatorResultBridgeController::new()));
                }
            }
        }
        rt
    }
}

impl RegistryBuilder<InMemoryKernel> {
    /// Convenience: a builder backed by an empty in-memory kernel.
    pub fn in_memory() -> Self {
        RegistryBuilder::with_kernel(InMemoryKernel::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AcquiredConfig, ConfigSource, MachineConfigSpec};
    use crate::machine_status::{MachineStage, StageReport};
    use crate::network::{OperatorResultResource, OperatorSpecResource};
    use os_kernel::{MachineType, NodeAddress};
    use os_network_domain::{OperatorResult, OperatorSpec};

    #[test]
    fn domain_names_are_stable() {
        assert_eq!(Domain::all().len(), 15);
        assert_eq!(
            Domain::ConfigAcquire.controller_name(),
            "config.ConfigAcquireController"
        );
        assert_eq!(
            Domain::NetworkLinkStatusSource.controller_name(),
            "network.LinkStatusSourceController"
        );
        assert_eq!(
            Domain::NetworkOperatorConfig.controller_name(),
            "network.OperatorConfigController"
        );
        assert_eq!(
            Domain::NetworkOperatorMerge.controller_name(),
            "network.OperatorMergeController"
        );
        assert_eq!(
            Domain::NetworkAddressMerge.controller_name(),
            "network.AddressMergeController"
        );
        assert_eq!(
            Domain::NetworkRouteMerge.controller_name(),
            "network.RouteMergeController"
        );
        assert_eq!(
            Domain::NetworkHostnameMerge.controller_name(),
            "network.HostnameMergeController"
        );
        assert_eq!(
            Domain::NetworkLinkConfig.controller_name(),
            "network.LinkConfigController"
        );
        assert_eq!(
            Domain::NetworkLinkMerge.controller_name(),
            "network.LinkMergeController"
        );
        assert_eq!(
            Domain::NetworkResolverConfig.controller_name(),
            "network.ResolverConfigController"
        );
        assert_eq!(
            Domain::NetworkResolverMerge.controller_name(),
            "network.ResolverMergeController"
        );
    }

    #[test]
    fn builder_registers_selected_domains() {
        let rt = RegistryBuilder::in_memory()
            .domain(Domain::ConfigAcquire)
            .domain(Domain::ConfigAcquire) // duplicate ignored
            .domain(Domain::ConfigCluster)
            .build();
        assert_eq!(rt.controller_count(), 2);
    }

    #[test]
    fn builder_all_domains_registers_everything() {
        let rt = RegistryBuilder::in_memory().all_domains().build();
        assert_eq!(rt.controller_count(), 15);
    }

    #[test]
    fn end_to_end_config_pipeline_through_runtime() {
        // Acquire -> MachineConfig -> ClusterConfig across two controllers.
        let mut rt = RegistryBuilder::in_memory()
            .domain(Domain::ConfigAcquire)
            .domain(Domain::ConfigCluster)
            .build();

        let spec = MachineConfigSpec {
            machine_type: MachineType::ControlPlane,
            cluster_name: "talos".into(),
            control_plane_endpoint: "https://10.0.0.1:6443".into(),
            kubernetes_version: "1.30.0".into(),
        };
        rt.state_mut()
            .create(Box::new(AcquiredConfig::new(ConfigSource::Disk, spec)))
            .unwrap();

        rt.run_until_stable(5).unwrap();

        assert!(rt.state().contains("config/MachineConfig/v1alpha1"));
        let cc = rt.state().get("config/ClusterConfig/cluster").unwrap();
        assert_eq!(
            cc.spec_fingerprint(),
            "cluster=talos;endpoint=https://10.0.0.1:6443;k8s=1.30.0;cp=true"
        );
    }

    #[test]
    fn kernel_param_domain_consumes_kernel() {
        use crate::kernel_param::KernelParamConfig;
        let kernel = InMemoryKernel::new().with_path("/proc/sys/net/ipv4/ip_forward", "0");
        let mut rt = RegistryBuilder::with_kernel(kernel)
            .domain(Domain::RuntimeKernelParam)
            .build();
        rt.state_mut()
            .create(Box::new(
                KernelParamConfig::sysctl("net.ipv4.ip_forward", "1").unwrap(),
            ))
            .unwrap();
        rt.run_until_stable(3).unwrap();
        let status = rt
            .state()
            .get("runtime/KernelParamStatus/net.ipv4.ip_forward")
            .unwrap();
        assert_eq!(status.spec_fingerprint(), "current=1;default=0");
    }

    #[test]
    fn registry_all_domains_merges_operator_result_bridge_outputs_before_stable() {
        let spec = OperatorSpec::dhcp4("eth0");
        let operator_id = spec.id();
        let result = OperatorResult {
            address: NodeAddress::parse_v4("10.0.0.50").unwrap(),
            prefix_len: 24,
            gateway: None,
            dns_servers: Vec::new(),
            hostname: None,
            search_domains: Vec::new(),
        };
        let mut rt = RegistryBuilder::in_memory().all_domains().build();
        rt.state_mut()
            .create(Box::new(StageReport::new(
                MachineStage::Running,
                "registry smoke",
            )))
            .unwrap();
        rt.state_mut()
            .create(Box::new(OperatorSpecResource::new(spec)))
            .unwrap();
        rt.state_mut()
            .create(Box::new(OperatorResultResource::new(operator_id, result)))
            .unwrap();

        rt.run_until_stable(5).unwrap();

        assert!(
            rt.state()
                .contains("network-config/AddressSpecs.net.talos.dev/dhcp4/eth0/eth0/10.0.0.50/24")
        );
        assert!(
            rt.state()
                .contains("network/AddressSpecs.net.talos.dev/eth0/10.0.0.50/24")
        );
    }
}
