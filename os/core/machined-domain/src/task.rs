//! Tasks: the atomic unit of work inside a phase.
//!
//! Mirrors Talos `runtime.TaskExecutionFunc` / `runtime.TaskSetupFunc`. A
//! task receives a [`TaskContext`] describing the sequence being run and the
//! machine runtime, and returns a [`TaskOutcome`]. Tasks are deliberately
//! side-effect-light here: real Talos tasks mount filesystems, write configs,
//! etc.; we model that boundary so it can be driven in tests.

use crate::error::Result;
use crate::runtime::RuntimeMode;
use crate::sequence::Sequence;
use os_kernel::MachineType;

/// The result of running a [`Task`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskOutcome {
    /// The task ran and changed state.
    Done,
    /// The task's postcondition already held; nothing was done. Used by
    /// idempotent tasks on re-run (e.g. config already written).
    Skipped,
    /// The task requests the sequencer abort the remaining phases but treat
    /// the sequence as successful (e.g. a reboot was triggered).
    RebootRequested,
}

/// Read-only context handed to a [`Task`] when it runs.
///
/// Captures the cross-cutting facts a task needs without borrowing the whole
/// runtime mutably: the sequence in flight, the platform mode, the machine
/// role, and whether config has been applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskContext {
    sequence: Sequence,
    mode: RuntimeMode,
    machine_type: MachineType,
    configured: bool,
}

impl TaskContext {
    /// Build a task context.
    pub fn new(
        sequence: Sequence,
        mode: RuntimeMode,
        machine_type: MachineType,
        configured: bool,
    ) -> Self {
        TaskContext {
            sequence,
            mode,
            machine_type,
            configured,
        }
    }

    /// The sequence currently being run.
    pub fn sequence(&self) -> Sequence {
        self.sequence
    }

    /// The runtime/platform mode.
    pub fn mode(&self) -> RuntimeMode {
        self.mode
    }

    /// The machine role.
    pub fn machine_type(&self) -> MachineType {
        self.machine_type
    }

    /// Whether config has been applied.
    pub fn is_configured(&self) -> bool {
        self.configured
    }
}

/// A single unit of work in a boot/upgrade/reset sequence.
///
/// Mirrors a Talos sequencer task. `run` performs the work and reports a
/// [`TaskOutcome`]; `should_run` lets a task opt out for the current context
/// (for example, install tasks skip themselves in `Container` mode).
pub trait Task {
    /// Stable task name (used in logs and error reporting).
    fn name(&self) -> &str;

    /// Whether this task applies in the given context. Default: always.
    fn should_run(&self, _ctx: &TaskContext) -> bool {
        true
    }

    /// Execute the task.
    fn run(&mut self, ctx: &TaskContext) -> Result<TaskOutcome>;
}

/// A trivial task that records that it ran. Useful as a building block and in
/// tests; mirrors how Talos composes small named tasks into phases.
pub struct NamedTask<F>
where
    F: FnMut(&TaskContext) -> Result<TaskOutcome>,
{
    name: String,
    f: F,
    gate: Option<fn(&TaskContext) -> bool>,
}

impl<F> NamedTask<F>
where
    F: FnMut(&TaskContext) -> Result<TaskOutcome>,
{
    /// Build a named task from a closure.
    pub fn new(name: impl Into<String>, f: F) -> Self {
        NamedTask {
            name: name.into(),
            f,
            gate: None,
        }
    }

    /// Add a predicate controlling whether the task runs in a given context.
    pub fn with_gate(mut self, gate: fn(&TaskContext) -> bool) -> Self {
        self.gate = Some(gate);
        self
    }
}

impl<F> Task for NamedTask<F>
where
    F: FnMut(&TaskContext) -> Result<TaskOutcome>,
{
    fn name(&self) -> &str {
        &self.name
    }
    fn should_run(&self, ctx: &TaskContext) -> bool {
        match self.gate {
            Some(g) => g(ctx),
            None => true,
        }
    }
    fn run(&mut self, ctx: &TaskContext) -> Result<TaskOutcome> {
        (self.f)(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::MachinedError;

    fn ctx(seq: Sequence, mode: RuntimeMode) -> TaskContext {
        TaskContext::new(seq, mode, MachineType::Worker, true)
    }

    #[test]
    fn named_task_runs() {
        let mut t = NamedTask::new("writeConfig", |_| Ok(TaskOutcome::Done));
        let c = ctx(Sequence::Boot, RuntimeMode::Metal);
        assert_eq!(t.name(), "writeConfig");
        assert!(t.should_run(&c));
        assert_eq!(t.run(&c).unwrap(), TaskOutcome::Done);
    }

    #[test]
    fn gate_skips_container_install() {
        let install_gate: fn(&TaskContext) -> bool = |c| c.mode().has_disks();
        let t = NamedTask::new("install", |_| Ok(TaskOutcome::Done)).with_gate(install_gate);
        assert!(t.should_run(&ctx(Sequence::Install, RuntimeMode::Metal)));
        assert!(!t.should_run(&ctx(Sequence::Install, RuntimeMode::Container)));
    }

    #[test]
    fn task_can_fail() {
        let mut t = NamedTask::new("mount", |_| {
            Err(MachinedError::task_failed("mount", "EBUSY"))
        });
        let c = ctx(Sequence::Boot, RuntimeMode::Metal);
        assert_eq!(t.run(&c).unwrap_err().kind(), "task_failed");
    }

    #[test]
    fn context_exposes_fields() {
        let c = TaskContext::new(
            Sequence::Upgrade,
            RuntimeMode::Cloud,
            MachineType::ControlPlane,
            false,
        );
        assert_eq!(c.sequence(), Sequence::Upgrade);
        assert_eq!(c.mode(), RuntimeMode::Cloud);
        assert!(c.machine_type().is_control_plane());
        assert!(!c.is_configured());
    }
}
