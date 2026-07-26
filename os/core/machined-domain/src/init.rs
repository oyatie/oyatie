//! The init process (PID1).
//!
//! Mirrors `siderolabs/talos` `internal/app/machined`'s `main`/`run`: PID1 owns
//! the machine lifecycle. It wires together the v1alpha1 [`V1Alpha1Runtime`],
//! the controller [`ControllerRuntime`], and the service [`Supervisor`], and
//! exposes the top-level lifecycle operations the sequencer triggers:
//! `boot`, `reboot`, `shutdown`.
//!
//! The reboot/poweroff syscall boundary is modeled by the [`Rebooter`] trait so
//! tests can assert the action without halting the host.

use crate::controllers::ControllerRuntime;
use crate::error::{MachinedError, Result};
use crate::events::{EventKind, EventStream};
use crate::runtime::RuntimeMode;
use crate::supervisor::{ServiceLauncher, Supervisor};
use crate::v1alpha1_runtime::{V1Alpha1Runtime, V1Alpha1State};
use os_block_domain::{VolumeConfigController, VolumeConfigRuntimeMode};
use os_kernel::MachineType;
use os_runtime_cri_domain::RegistriesConfigController;

/// The host power action requested at the end of a terminal sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction {
    /// Reboot the kernel.
    Reboot,
    /// Power the machine off.
    PowerOff,
}

/// The syscall boundary for `reboot(2)` / `poweroff`.
///
/// In production this issues the real syscall; tests record the action.
pub trait Rebooter {
    /// Perform the power action. Returns an error if it is not possible in the
    /// current environment (e.g. rebooting a container).
    fn power(&mut self, action: PowerAction) -> Result<()>;
}

/// A no-op rebooter that records the last action, for tests and container mode.
#[derive(Debug, Default)]
pub struct RecordingRebooter {
    /// The last power action requested.
    pub last: Option<PowerAction>,
}

impl Rebooter for RecordingRebooter {
    fn power(&mut self, action: PowerAction) -> Result<()> {
        self.last = Some(action);
        Ok(())
    }
}

/// PID1: the machine init process.
pub struct Init {
    runtime: V1Alpha1Runtime,
    controllers: ControllerRuntime,
    supervisor: Supervisor,
    events: EventStream,
    staged_upgrade: Option<String>,
}

impl Init {
    /// Construct PID1 for a machine of the given mode and role.
    pub fn new(mode: RuntimeMode, machine_type: MachineType) -> Self {
        Init {
            runtime: V1Alpha1Runtime::new(mode, machine_type),
            controllers: ControllerRuntime::new(),
            supervisor: Supervisor::new(machine_type),
            events: EventStream::default(),
            staged_upgrade: None,
        }
    }

    /// The PID1 event stream (machine status-change events).
    pub fn events(&self) -> &EventStream {
        &self.events
    }

    /// The pending staged upgrade image, if one was staged for next boot.
    pub fn staged_upgrade(&self) -> Option<&str> {
        self.staged_upgrade.as_deref()
    }

    /// Access the v1alpha1 runtime.
    pub fn runtime(&self) -> &V1Alpha1Runtime {
        &self.runtime
    }

    /// Mutable access to the controller runtime (to register controllers).
    pub fn controllers_mut(&mut self) -> &mut ControllerRuntime {
        &mut self.controllers
    }

    /// Register the ported source-shaped block `VolumeConfigController`.
    ///
    /// Source Talos registers `block.VolumeConfigController` during v1alpha2
    /// startup with `V1Alpha1Mode` set from the current platform runtime mode.
    pub fn register_block_volume_config_controller(&mut self) -> Result<()> {
        if self
            .controllers
            .get(&crate::controllers::ControllerId::from(
                os_block_domain::VOLUME_CONFIG_CONTROLLER_NAME,
            ))
            .is_some()
        {
            return Ok(());
        }

        let controller = VolumeConfigController::new_for_runtime_mode(volume_config_runtime_mode(
            self.runtime.mode(),
        ));
        self.controllers
            .register(crate::controllers::MachinedController::from_cosi_controller(&controller))
    }

    /// Register the ported source-shaped CRI `RegistriesConfigController`.
    ///
    /// Source Talos registers `cri.RegistriesConfigController` during v1alpha2
    /// startup. Its MachineConfig/ImageCacheConfig inputs are weak watches, so
    /// they should not block machined startup dependency ordering.
    pub fn register_cri_registries_config_controller(&mut self) -> Result<()> {
        if self
            .controllers
            .get(&crate::controllers::ControllerId::from(
                os_runtime_cri_domain::REGISTRIES_CONFIG_CONTROLLER_NAME,
            ))
            .is_some()
        {
            return Ok(());
        }

        let controller = RegistriesConfigController::new();
        self.controllers
            .register(crate::controllers::MachinedController::from_cosi_controller(&controller))
    }

    /// Register source v1alpha2 startup controllers ported into this crate.
    fn register_startup_controllers(&mut self) -> Result<()> {
        self.register_block_volume_config_controller()?;
        self.register_cri_registries_config_controller()
    }

    /// Mutable access to the supervisor (to register services).
    pub fn supervisor_mut(&mut self) -> &mut Supervisor {
        &mut self.supervisor
    }

    /// The current runtime status.
    pub fn state(&self) -> V1Alpha1State {
        self.runtime.state()
    }

    /// First step of PID1: enter maintenance mode and await config.
    pub fn enter_maintenance(&mut self) -> Result<()> {
        self.runtime.enter_maintenance()
    }

    /// Apply machine config and propagate the configured flag to the supervisor.
    pub fn apply_config(&mut self, config: impl Into<String>) -> Result<()> {
        self.runtime.set_config(config)?;
        self.supervisor.set_configured(true);
        Ok(())
    }

    /// Boot the machine: validate the controller graph, mark the network ready,
    /// transition the runtime to booting, start all services, then mark running.
    ///
    /// Returns the number of services that came up.
    pub fn boot(&mut self, launcher: &mut dyn ServiceLauncher) -> Result<usize> {
        self.register_startup_controllers()?;

        // Controllers must form a valid, acyclic, fully-satisfied graph.
        self.controllers.startup_order()?;

        self.runtime.begin_boot()?;
        self.events.publish(EventKind::Message {
            body: "booting".into(),
        });
        self.supervisor.set_network_ready(true);
        let started = self.supervisor.start_all(launcher)?;

        // Every registered service must have come up for the boot to succeed.
        if started < self.supervisor.len() {
            return Err(MachinedError::service_error(
                "boot",
                format!(
                    "only {}/{} services came up",
                    started,
                    self.supervisor.len()
                ),
            ));
        }
        self.runtime.mark_running()?;
        self.events.publish(EventKind::Message {
            body: "running".into(),
        });
        Ok(started)
    }

    /// Stage an upgrade to a new image, to be applied on the next boot.
    ///
    /// Mirrors Talos `StageUpgrade`: the new installer image is recorded (the
    /// real implementation drops an `upgrade` marker on disk) and a reboot is
    /// expected to follow. Requires config and disk-backed runtime.
    pub fn stage_upgrade(&mut self, image: impl Into<String>) -> Result<()> {
        if !self.runtime.is_configured() {
            return Err(MachinedError::sequence_not_allowed(
                "cannot stage upgrade before config is applied",
            ));
        }
        if !self.runtime.mode().has_disks() {
            return Err(MachinedError::sequence_not_allowed(format!(
                "{} mode has no disk to stage an upgrade",
                self.runtime.mode().as_str()
            )));
        }
        let image = image.into();
        self.events.publish(EventKind::Message {
            body: format!("staged upgrade: {image}"),
        });
        self.staged_upgrade = Some(image);
        Ok(())
    }

    /// Apply a maintenance-mode upgrade: validate the new image and reboot into
    /// it directly from maintenance, without a running cluster to drain.
    ///
    /// Mirrors Talos `MaintenanceUpgrade`. Requires config to be present.
    pub fn maintenance_upgrade(
        &mut self,
        image: impl Into<String>,
        launcher: &mut dyn ServiceLauncher,
        rebooter: &mut dyn Rebooter,
    ) -> Result<()> {
        if !self.runtime.is_configured() {
            return Err(MachinedError::sequence_not_allowed(
                "cannot upgrade before config is applied",
            ));
        }
        if !self.runtime.mode().has_disks() {
            return Err(MachinedError::sequence_not_allowed(format!(
                "{} mode cannot be upgraded",
                self.runtime.mode().as_str()
            )));
        }
        let image = image.into();
        self.events.publish(EventKind::Message {
            body: format!("maintenance upgrade: {image}"),
        });
        // Stop anything running, install, then reboot into the new image.
        self.supervisor.stop_all(launcher)?;
        self.staged_upgrade = Some(image);
        self.runtime.shutdown()?;
        rebooter.power(PowerAction::Reboot)
    }

    /// Reboot: stop services and request a kernel reboot via the [`Rebooter`].
    /// Refused in modes that cannot reboot.
    pub fn reboot(
        &mut self,
        launcher: &mut dyn ServiceLauncher,
        rebooter: &mut dyn Rebooter,
    ) -> Result<()> {
        if !self.runtime.mode().can_reboot() {
            return Err(MachinedError::sequence_not_allowed(format!(
                "{} mode cannot reboot",
                self.runtime.mode().as_str()
            )));
        }
        self.supervisor.stop_all(launcher)?;
        self.runtime.shutdown()?;
        rebooter.power(PowerAction::Reboot)
    }

    /// Shutdown: stop services and power the machine off.
    pub fn shutdown(
        &mut self,
        launcher: &mut dyn ServiceLauncher,
        rebooter: &mut dyn Rebooter,
    ) -> Result<()> {
        self.supervisor.stop_all(launcher)?;
        self.runtime.shutdown()?;
        rebooter.power(PowerAction::PowerOff)
    }
}

fn volume_config_runtime_mode(mode: RuntimeMode) -> VolumeConfigRuntimeMode {
    match mode {
        RuntimeMode::Cloud => VolumeConfigRuntimeMode::Cloud,
        RuntimeMode::Container => VolumeConfigRuntimeMode::Container,
        RuntimeMode::Metal => VolumeConfigRuntimeMode::Metal,
        RuntimeMode::MetalAgent => VolumeConfigRuntimeMode::MetalAgent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controllers::MachinedController;
    use crate::service::{Service, ServiceCondition};
    use std::collections::HashSet;

    struct OkLauncher {
        stopped: HashSet<String>,
    }
    impl OkLauncher {
        fn new() -> Self {
            OkLauncher {
                stopped: HashSet::new(),
            }
        }
    }
    impl ServiceLauncher for OkLauncher {
        fn launch(&mut self, _id: &str) -> Result<bool> {
            Ok(true)
        }
        fn stop(&mut self, id: &str) -> Result<()> {
            self.stopped.insert(id.to_string());
            Ok(())
        }
    }

    fn cp_init() -> Init {
        let mut init = Init::new(RuntimeMode::Metal, MachineType::ControlPlane);
        init.controllers_mut()
            .register(MachinedController::new(
                "config",
                vec![],
                vec!["MachineConfig"],
            ))
            .unwrap();
        init.controllers_mut()
            .register(MachinedController::new(
                "kubelet",
                vec!["MachineConfig"],
                vec!["KubeletStatus"],
            ))
            .unwrap();
        init.supervisor_mut()
            .register(Service::new("etcd", vec![ServiceCondition::ConfigPresent]));
        init.supervisor_mut().register(Service::new(
            "kubelet",
            vec![ServiceCondition::ServiceHealthy("etcd".to_string())],
        ));
        init
    }

    #[test]
    fn full_boot_brings_services_up() {
        let mut init = cp_init();
        init.enter_maintenance().unwrap();
        init.apply_config("version: v1alpha1").unwrap();
        let mut l = OkLauncher::new();
        let up = init.boot(&mut l).unwrap();
        assert_eq!(up, 2);
        assert!(init.state().is_ready());
    }

    #[test]
    fn boot_without_config_fails() {
        let mut init = cp_init();
        init.enter_maintenance().unwrap();
        let mut l = OkLauncher::new();
        let err = init.boot(&mut l).unwrap_err();
        assert_eq!(err.kind(), "sequence_not_allowed");
    }

    #[test]
    fn reboot_stops_services_and_powers() {
        let mut init = cp_init();
        init.enter_maintenance().unwrap();
        init.apply_config("cfg").unwrap();
        let mut l = OkLauncher::new();
        init.boot(&mut l).unwrap();
        let mut r = RecordingRebooter::default();
        init.reboot(&mut l, &mut r).unwrap();
        assert_eq!(r.last, Some(PowerAction::Reboot));
        assert!(l.stopped.contains("etcd"));
        assert_eq!(init.state(), V1Alpha1State::Shutdown);
    }

    #[test]
    fn container_cannot_reboot() {
        let mut init = Init::new(RuntimeMode::Container, MachineType::Worker);
        init.enter_maintenance().unwrap();
        init.apply_config("cfg").unwrap();
        let mut l = OkLauncher::new();
        let mut r = RecordingRebooter::default();
        let err = init.reboot(&mut l, &mut r).unwrap_err();
        assert_eq!(err.kind(), "sequence_not_allowed");
        assert_eq!(r.last, None);
    }

    #[test]
    fn boot_emits_status_events() {
        let mut init = cp_init();
        init.enter_maintenance().unwrap();
        init.apply_config("cfg").unwrap();
        let mut l = OkLauncher::new();
        init.boot(&mut l).unwrap();
        let msgs: Vec<String> = init
            .events()
            .of_type("message")
            .into_iter()
            .filter_map(|e| match &e.kind {
                crate::events::EventKind::Message { body } => Some(body.clone()),
                _ => None,
            })
            .collect();
        assert!(msgs.contains(&"booting".to_string()));
        assert!(msgs.contains(&"running".to_string()));
    }

    #[test]
    fn stage_upgrade_records_image() {
        let mut init = cp_init();
        init.enter_maintenance().unwrap();
        init.apply_config("cfg").unwrap();
        init.stage_upgrade("ghcr.io/siderolabs/installer:v1.7.0")
            .unwrap();
        assert_eq!(
            init.staged_upgrade(),
            Some("ghcr.io/siderolabs/installer:v1.7.0")
        );
    }

    #[test]
    fn stage_upgrade_rejected_without_config() {
        let mut init = cp_init();
        init.enter_maintenance().unwrap();
        let err = init.stage_upgrade("img").unwrap_err();
        assert_eq!(err.kind(), "sequence_not_allowed");
    }

    #[test]
    fn stage_upgrade_rejected_in_container() {
        let mut init = Init::new(RuntimeMode::Container, MachineType::Worker);
        init.enter_maintenance().unwrap();
        init.apply_config("cfg").unwrap();
        let err = init.stage_upgrade("img").unwrap_err();
        assert_eq!(err.kind(), "sequence_not_allowed");
    }

    #[test]
    fn maintenance_upgrade_reboots_into_image() {
        let mut init = cp_init();
        init.enter_maintenance().unwrap();
        init.apply_config("cfg").unwrap();
        let mut l = OkLauncher::new();
        let mut r = RecordingRebooter::default();
        init.maintenance_upgrade("installer:v2", &mut l, &mut r)
            .unwrap();
        assert_eq!(r.last, Some(PowerAction::Reboot));
        assert_eq!(init.staged_upgrade(), Some("installer:v2"));
        assert_eq!(init.state(), V1Alpha1State::Shutdown);
    }

    #[test]
    fn invalid_controller_graph_blocks_boot() {
        let mut init = Init::new(RuntimeMode::Metal, MachineType::Worker);
        // kubelet needs MachineConfig but nothing produces it.
        init.controllers_mut()
            .register(MachinedController::new(
                "kubelet",
                vec!["MachineConfig"],
                vec!["KubeletStatus"],
            ))
            .unwrap();
        init.enter_maintenance().unwrap();
        init.apply_config("cfg").unwrap();
        let mut l = OkLauncher::new();
        let err = init.boot(&mut l).unwrap_err();
        assert_eq!(err.kind(), "dependency_unmet");
    }

    #[test]
    fn volume_config_startup_registers_block_controller_from_runtime_mode() {
        let mut init = Init::new(RuntimeMode::MetalAgent, MachineType::Worker);

        init.register_block_volume_config_controller().unwrap();

        let controller = init
            .controllers_mut()
            .get(&crate::controllers::ControllerId::from(
                os_block_domain::VOLUME_CONFIG_CONTROLLER_NAME,
            ))
            .expect("block volume config controller is registered");
        assert!(
            controller.inputs().is_empty(),
            "weak/destroy-ready source inputs must not become blocking startup dependencies"
        );
        assert_eq!(
            controller.outputs(),
            &[
                "runtime/VolumeConfigs.block.talos.dev".to_string(),
                "runtime/VolumeMountRequests.block.talos.dev".to_string()
            ]
        );
        assert!(init.controllers_mut().startup_order().is_ok());
    }

    #[test]
    fn volume_config_startup_registers_block_controller_during_boot() {
        let mut init = Init::new(RuntimeMode::MetalAgent, MachineType::Worker);
        let block_controller_id =
            crate::controllers::ControllerId::from(os_block_domain::VOLUME_CONFIG_CONTROLLER_NAME);
        assert!(init.controllers_mut().get(&block_controller_id).is_none());

        init.enter_maintenance().unwrap();
        init.apply_config("cfg").unwrap();
        let mut launcher = OkLauncher::new();
        assert_eq!(init.boot(&mut launcher).unwrap(), 0);

        let controller = init
            .controllers_mut()
            .get(&block_controller_id)
            .expect("block volume config controller is registered during startup");
        assert!(
            controller.inputs().is_empty(),
            "weak/destroy-ready source inputs must not become blocking startup dependencies"
        );
        assert_eq!(
            controller.outputs(),
            &[
                "runtime/VolumeConfigs.block.talos.dev".to_string(),
                "runtime/VolumeMountRequests.block.talos.dev".to_string()
            ]
        );
    }

    #[test]
    fn cri_registries_config_startup_registers_controller_during_boot() {
        let mut init = Init::new(RuntimeMode::Metal, MachineType::Worker);
        let registries_controller_id = crate::controllers::ControllerId::from(
            os_runtime_cri_domain::REGISTRIES_CONFIG_CONTROLLER_NAME,
        );
        assert!(
            init.controllers_mut()
                .get(&registries_controller_id)
                .is_none()
        );

        init.enter_maintenance().unwrap();
        init.apply_config("cfg").unwrap();
        let mut launcher = OkLauncher::new();
        assert_eq!(init.boot(&mut launcher).unwrap(), 0);

        let controller = init
            .controllers_mut()
            .get(&registries_controller_id)
            .expect("CRI registries config controller is registered during startup");
        assert!(
            controller.inputs().is_empty(),
            "weak source inputs must not become blocking startup dependencies"
        );
        assert_eq!(
            controller.outputs(),
            &[os_runtime_cri_domain::RegistriesConfigResource::kind().to_string()]
        );
    }
}
