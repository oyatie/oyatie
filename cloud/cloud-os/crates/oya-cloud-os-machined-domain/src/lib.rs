//! # talos-machined
//!
//! PID1 for a Talos node: the init process and the machine sequencer.
//!
//! This crate mirrors `internal/app/machined` in `siderolabs/talos`. It models:
//!
//! - the [`Init`] process (PID1) that owns the machine lifecycle;
//! - the [`Sequencer`] that drives ordered [`Sequence`]s (Boot, Install,
//!   Upgrade, Reset, Shutdown, Reboot) built from [`Phase`]s of [`Task`]s;
//! - the service [`supervisor`] that runs and restarts long-lived
//!   [`Service`]s through a [`ServiceRunner`] state machine
//!   ([`ServiceState`]);
//! - the COSI [`controllers`] wiring for machined; and
//! - the v1alpha1 [`MachineRuntime`] / [`RuntimeMode`] runtime state machine.
//!
//! Where the real subsystem performs syscalls (`reboot(2)`, `mount(2)`),
//! talks to containerd, or hits the network, the boundary is modeled as a
//! trait so it can be driven by an in-memory implementation in tests.

pub mod boot;
pub mod controllers;
pub mod error;
pub mod events;
pub mod init;
pub mod phase;
pub mod runtime;
pub mod sequence;
pub mod sequencer;
pub mod service;
pub mod state_machine;
pub mod supervisor;
pub mod task;
pub mod task_catalog;
pub mod v1alpha1_runtime;

pub use boot::{
    BootPhase, BootPhaseId, BootReport, BootRuntimeMode, BootSequencer, BootService, BootTask,
    FailPolicy, FakeRuntime, FnTask, MountRequest, NullLogger, PhaseTally, PlatformOps,
    ProgressLogger, RecordingLogger, RestartPolicy, Runtime, ServiceStatus,
    TaskOutcome as BootTaskOutcome, detect_runtime_mode, mount_skip_reason,
    privileged_op_skip_reason, standard_boot_phases, standard_services, sysctl_skip_reason,
};
pub use controllers::{ControllerId, ControllerRuntime, MachinedController};
pub use error::{MachinedError, Result};
pub use events::{Event, EventKind, EventStream};
pub use init::Init;
pub use phase::Phase;
pub use runtime::{InMemoryRuntime, MachineRuntime, RuntimeMode};
pub use sequence::Sequence;
pub use sequencer::Sequencer;
pub use service::{Service, ServiceCondition, ServiceState};
pub use state_machine::{MachineState, StateMachine};
pub use supervisor::{
    ServiceLauncher, ServiceRunner, Supervisor, SupervisorRegistrydServiceManager,
};
pub use task::{NamedTask, Task, TaskContext, TaskOutcome};
pub use task_catalog::{SideEffects, TaskCatalog};
pub use v1alpha1_runtime::{V1Alpha1Runtime, V1Alpha1State};
