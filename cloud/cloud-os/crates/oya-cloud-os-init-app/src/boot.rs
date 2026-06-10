//! The early-userspace boot sequence as a state machine.
//!
//! Talos' `cmd/init` runs a fixed sequence of phases before handing PID 1 off to
//! `machined`. We model that sequence explicitly so it can be unit-tested as a
//! whole: each phase transitions the [`BootState`], and [`BootPlan::run`] drives
//! the phases against pluggable trait objects (mounter, waiter, rootfs) so the
//! entire boot is exercised on the host with fakes.
//!
//! Phases (matching upstream ordering):
//!   1. SetupConsole   — wire stdio to `/dev/console`.
//!   2. MountEssential — proc/sys/dev/run/...
//!   3. ReadCmdline    — parse `/proc/cmdline`.
//!   4. ApplyConfig    — read machine config, set hostname/type.
//!   5. ReapInitial    — drain any pre-existing zombies.
//!   6. SwitchRoot     — pivot to real rootfs and exec machined.
//!
//! If switch_root is disabled (e.g. the minimal "run as PID1 then poweroff"
//! mode the original binary implements), the sequence ends in `PowerOff`.

use crate::cmdline::{CmdLine, ShutdownMode};
use crate::config::{EarlyConfig, try_early_config};
use crate::mount::{MountResult, Mounter, all_ok, mount_essential};
use crate::reaper::{ChildWaiter, ReapStats, reap_all};
use crate::switch_root::{RootFs, SwitchRootError, SwitchRootPlan, switch_root};

/// The discrete phases of early boot, in order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    SetupConsole,
    MountEssential,
    ReadCmdline,
    ApplyConfig,
    ReapInitial,
    SwitchRoot,
    PowerOff,
    Done,
}

impl Phase {
    /// The next phase in the linear sequence (when not short-circuiting).
    pub fn next(self) -> Phase {
        match self {
            Phase::SetupConsole => Phase::MountEssential,
            Phase::MountEssential => Phase::ReadCmdline,
            Phase::ReadCmdline => Phase::ApplyConfig,
            Phase::ApplyConfig => Phase::ReapInitial,
            Phase::ReapInitial => Phase::SwitchRoot,
            Phase::SwitchRoot => Phase::Done,
            Phase::PowerOff => Phase::Done,
            Phase::Done => Phase::Done,
        }
    }
}

/// Mutable boot state threaded through the phases. Captures what each phase
/// learned, for assertions and for later phases to consume.
#[derive(Default, Debug)]
pub struct BootState {
    pub phase: Option<Phase>,
    pub completed: Vec<Phase>,
    pub mount_results: Vec<MountResult>,
    pub cmdline: Option<CmdLine>,
    pub early_config: EarlyConfig,
    pub hostname_applied: Option<String>,
    pub reap_stats: ReapStats,
    pub switch_root_error: Option<SwitchRootError>,
    pub powered_off: bool,
    pub shutdown_mode: ShutdownMode,
}

impl BootState {
    pub fn new() -> Self {
        BootState {
            shutdown_mode: ShutdownMode::PowerOff,
            ..Default::default()
        }
    }

    fn complete(&mut self, phase: Phase) {
        self.phase = Some(phase);
        self.completed.push(phase);
    }

    /// True if every essential mount succeeded.
    pub fn mounts_ok(&self) -> bool {
        all_ok(&self.mount_results)
    }
}

/// A fatal boot error. Most phase failures are *non*-fatal (logged, continue);
/// only switch_root failure is fatal and forces an emergency poweroff.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BootError {
    Config(String),
    SwitchRoot(SwitchRootError),
}

impl std::fmt::Display for BootError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootError::Config(e) => write!(f, "machine config failed: {e}"),
            BootError::SwitchRoot(e) => write!(f, "switch_root failed: {e}"),
        }
    }
}

/// Inputs the boot needs that would otherwise come from the kernel/initramfs.
pub struct BootInputs<'a> {
    /// Raw `/proc/cmdline` contents.
    pub cmdline: &'a str,
    /// Raw machine config contents (may be empty if unavailable).
    pub machine_config: &'a str,
    /// Default hostname if config provides none.
    pub default_hostname: &'a str,
    /// If `Some`, switch_root to this plan; if `None`, end in PowerOff (the
    /// minimal init mode).
    pub switch_root: Option<SwitchRootPlan>,
}

/// The boot orchestrator: holds the OS-abstraction trait objects and drives all
/// phases.
pub struct BootPlan<'a> {
    pub mounter: &'a mut dyn Mounter,
    pub waiter: &'a mut dyn ChildWaiter,
    pub rootfs: &'a mut dyn RootFs,
}

impl BootPlan<'_> {
    /// Run the full early-boot sequence. Returns the final [`BootState`] on
    /// success, or `(state, BootError)` if switch_root failed (the caller then
    /// powers off).
    // The error variant deliberately carries the full `BootState` so the caller
    // can inspect everything the boot accomplished before failing; boxing it
    // would obscure that contract.
    #[allow(clippy::result_large_err)]
    pub fn run(&mut self, inputs: &BootInputs<'_>) -> Result<BootState, (BootState, BootError)> {
        let mut state = BootState::new();

        // 1. Console (modeled as a no-op success here; the Linux binary dups fds).
        state.complete(Phase::SetupConsole);

        // 2. Mount essential filesystems.
        let table = crate::mount::essential_mounts();
        state.mount_results = mount_essential(self.mounter, &table);
        state.complete(Phase::MountEssential);

        // 3. Read & parse kernel cmdline.
        let cmdline = CmdLine::parse(inputs.cmdline);
        state.shutdown_mode = cmdline.shutdown_mode();
        state.cmdline = Some(cmdline);
        state.complete(Phase::ReadCmdline);

        // 4. Apply machine config: extract early config + resolve hostname.
        state.early_config = match try_early_config(inputs.machine_config) {
            Ok(config) => config,
            Err(e) => {
                state.early_config = crate::config::early_config(inputs.machine_config);
                state.powered_off = true;
                return Err((state, BootError::Config(e.to_string())));
            }
        };
        let hostname = state
            .early_config
            .hostname
            .clone()
            // cmdline override beats config? Talos: config wins for hostname, but
            // a cmdline talos.hostname is honored when config has none.
            .or_else(|| {
                state
                    .cmdline
                    .as_ref()
                    .and_then(|c| c.hostname().map(String::from))
            })
            .unwrap_or_else(|| inputs.default_hostname.to_string());
        state.hostname_applied = Some(hostname);
        state.complete(Phase::ApplyConfig);

        // 5. Reap any pre-existing zombies.
        let reaped = reap_all(self.waiter);
        state.reap_stats.record(&reaped);
        state.complete(Phase::ReapInitial);

        // 6. Switch root (or power off in minimal mode).
        let Some(plan) = &inputs.switch_root else {
            // Minimal mode: success path ends in poweroff.
            state.powered_off = true;
            state.complete(Phase::PowerOff);
            return Ok(state);
        };
        match switch_root(plan, self.rootfs) {
            Ok(_done) => {
                state.complete(Phase::SwitchRoot);
                Ok(state)
            }
            Err(e) => {
                state.switch_root_error = Some(e.clone());
                state.complete(Phase::SwitchRoot);
                // Fatal: caller powers off.
                state.powered_off = true;
                Err((state, BootError::SwitchRoot(e)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mount::RecordingMounter;
    use crate::reaper::FakeWaiter;
    use crate::switch_root::FakeRootFs;

    fn worker_config() -> &'static str {
        "version: v1alpha1\nmachine:\n  type: worker\n  network:\n    hostname: node-x\n"
    }

    #[test]
    fn phase_next_is_linear() {
        assert_eq!(Phase::SetupConsole.next(), Phase::MountEssential);
        assert_eq!(Phase::ReapInitial.next(), Phase::SwitchRoot);
        assert_eq!(Phase::Done.next(), Phase::Done);
    }

    #[test]
    fn full_boot_with_switch_root_succeeds() {
        let mut mounter = RecordingMounter::new();
        let mut waiter = FakeWaiter::from_statuses(&[(10, 0), (11, 0)]);
        let mut rootfs = FakeRootFs::healthy("/root", "/root/sbin/machined", 100);
        let mut plan = BootPlan {
            mounter: &mut mounter,
            waiter: &mut waiter,
            rootfs: &mut rootfs,
        };
        let inputs = BootInputs {
            cmdline: "console=ttyS0,115200n8 talos.platform=metal",
            machine_config: worker_config(),
            default_hostname: "talos-rust",
            switch_root: Some(SwitchRootPlan::to_machined("/root")),
        };
        let state = plan.run(&inputs).expect("boot should succeed");
        assert!(state.mounts_ok());
        assert_eq!(state.hostname_applied.as_deref(), Some("node-x"));
        assert_eq!(state.reap_stats.total, 2);
        assert!(state.completed.contains(&Phase::SwitchRoot));
        assert!(state.switch_root_error.is_none());
        // The machined exec was recorded.
        assert!(rootfs.execed.is_some());
    }

    #[test]
    fn minimal_mode_ends_in_poweroff() {
        let mut mounter = RecordingMounter::new();
        let mut waiter = FakeWaiter::new(vec![]);
        let mut rootfs = FakeRootFs::default();
        let mut plan = BootPlan {
            mounter: &mut mounter,
            waiter: &mut waiter,
            rootfs: &mut rootfs,
        };
        let inputs = BootInputs {
            cmdline: "quiet",
            machine_config: worker_config(),
            default_hostname: "talos-rust",
            switch_root: None,
        };
        let state = plan.run(&inputs).unwrap();
        assert!(state.powered_off);
        assert!(state.completed.contains(&Phase::PowerOff));
        assert!(!state.completed.contains(&Phase::SwitchRoot));
        // No exec happened.
        assert!(rootfs.execed.is_none());
    }

    #[test]
    fn switch_root_failure_is_fatal_and_powers_off() {
        let mut mounter = RecordingMounter::new();
        let mut waiter = FakeWaiter::new(vec![]);
        // Missing init -> validate fails.
        let mut rootfs = FakeRootFs {
            mount_points: vec!["/root".to_string()],
            files: vec![],
            ..Default::default()
        };
        let mut plan = BootPlan {
            mounter: &mut mounter,
            waiter: &mut waiter,
            rootfs: &mut rootfs,
        };
        let inputs = BootInputs {
            cmdline: "",
            machine_config: worker_config(),
            default_hostname: "talos-rust",
            switch_root: Some(SwitchRootPlan::to_machined("/root")),
        };
        let (state, err) = plan.run(&inputs).unwrap_err();
        assert!(state.powered_off);
        assert!(matches!(
            err,
            BootError::SwitchRoot(SwitchRootError::InitMissing(_))
        ));
        assert!(state.switch_root_error.is_some());
    }

    #[test]
    fn malformed_machine_config_fails_apply_config_visibly() {
        let mut mounter = RecordingMounter::new();
        let mut waiter = FakeWaiter::new(vec![]);
        let mut rootfs = FakeRootFs::default();
        let mut plan = BootPlan {
            mounter: &mut mounter,
            waiter: &mut waiter,
            rootfs: &mut rootfs,
        };
        let inputs = BootInputs {
            cmdline: "",
            machine_config: "version: v1alpha1\nmachine:\n  type: worker\n---\napiVersion: v1alpha1\nkind: DHCPv4Config\nname: eth0\nclientIdentifier: duid\nduidRaw: not-hex\n",
            default_hostname: "talos-rust",
            switch_root: None,
        };
        let (state, err) = plan.run(&inputs).unwrap_err();
        assert!(state.powered_off);
        assert!(state.completed.contains(&Phase::ReadCmdline));
        assert!(!state.completed.contains(&Phase::ApplyConfig));
        assert!(matches!(err, BootError::Config(_)));
        assert!(!state.early_config.config_errors.is_empty());
        assert!(err.to_string().contains("duidRaw"));
    }

    #[test]
    fn hostname_falls_back_to_cmdline_then_default() {
        let mut mounter = RecordingMounter::new();
        let mut waiter = FakeWaiter::new(vec![]);
        let mut rootfs = FakeRootFs::default();
        let mut plan = BootPlan {
            mounter: &mut mounter,
            waiter: &mut waiter,
            rootfs: &mut rootfs,
        };
        // Config has no hostname; cmdline supplies one.
        let inputs = BootInputs {
            cmdline: "talos.hostname=from-cmdline",
            machine_config: "version: v1alpha1\nmachine:\n  type: worker\n",
            default_hostname: "talos-rust",
            switch_root: None,
        };
        let state = plan.run(&inputs).unwrap();
        assert_eq!(state.hostname_applied.as_deref(), Some("from-cmdline"));
    }

    #[test]
    fn hostname_defaults_when_nothing_provides_it() {
        let mut mounter = RecordingMounter::new();
        let mut waiter = FakeWaiter::new(vec![]);
        let mut rootfs = FakeRootFs::default();
        let mut plan = BootPlan {
            mounter: &mut mounter,
            waiter: &mut waiter,
            rootfs: &mut rootfs,
        };
        let inputs = BootInputs {
            cmdline: "",
            machine_config: "",
            default_hostname: "talos-rust",
            switch_root: None,
        };
        let state = plan.run(&inputs).unwrap();
        assert_eq!(state.hostname_applied.as_deref(), Some("talos-rust"));
    }

    #[test]
    fn shutdown_mode_read_from_cmdline() {
        let mut mounter = RecordingMounter::new();
        let mut waiter = FakeWaiter::new(vec![]);
        let mut rootfs = FakeRootFs::default();
        let mut plan = BootPlan {
            mounter: &mut mounter,
            waiter: &mut waiter,
            rootfs: &mut rootfs,
        };
        let inputs = BootInputs {
            cmdline: "talos.shutdown=halt",
            machine_config: "",
            default_hostname: "talos-rust",
            switch_root: None,
        };
        let state = plan.run(&inputs).unwrap();
        assert_eq!(state.shutdown_mode, ShutdownMode::Halt);
    }

    #[test]
    fn phases_complete_in_order() {
        let mut mounter = RecordingMounter::new();
        let mut waiter = FakeWaiter::new(vec![]);
        let mut rootfs = FakeRootFs::healthy("/root", "/root/sbin/machined", 0);
        let mut plan = BootPlan {
            mounter: &mut mounter,
            waiter: &mut waiter,
            rootfs: &mut rootfs,
        };
        let inputs = BootInputs {
            cmdline: "",
            machine_config: worker_config(),
            default_hostname: "talos-rust",
            switch_root: Some(SwitchRootPlan::to_machined("/root")),
        };
        let state = plan.run(&inputs).unwrap();
        assert_eq!(
            state.completed,
            vec![
                Phase::SetupConsole,
                Phase::MountEssential,
                Phase::ReadCmdline,
                Phase::ApplyConfig,
                Phase::ReapInitial,
                Phase::SwitchRoot,
            ]
        );
    }

    #[test]
    fn boot_error_display() {
        let e = BootError::SwitchRoot(SwitchRootError::ExecFailed("x".into()));
        assert!(e.to_string().contains("switch_root failed"));
        let e = BootError::Config("bad".into());
        assert!(e.to_string().contains("machine config failed"));
    }
}
