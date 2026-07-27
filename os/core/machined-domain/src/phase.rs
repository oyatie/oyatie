//! A phase: an ordered, named group of tasks run by the sequencer.
//!
//! Mirrors `siderolabs/talos` `runtime.Phase`. Within a sequence the phases
//! run strictly in order; the tasks inside a single phase are logically run
//! together (Talos runs them concurrently, we run them in registration order
//! since our tasks are in-memory). A failing task aborts the phase and the
//! sequence.

use crate::error::Result;
use crate::task::{Task, TaskContext, TaskOutcome};

/// An ordered group of tasks identified by a stable name.
///
/// `Phase` owns its tasks as boxed trait objects so heterogeneous task types
/// can be composed, exactly like the Talos sequencer composes named task funcs.
pub struct Phase {
    name: String,
    tasks: Vec<Box<dyn Task>>,
}

impl Phase {
    /// Create an empty phase with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Phase {
            name: name.into(),
            tasks: Vec::new(),
        }
    }

    /// Append a task to the phase, returning `self` for builder-style use.
    pub fn with_task(mut self, task: Box<dyn Task>) -> Self {
        self.tasks.push(task);
        self
    }

    /// Append a task to the phase in place.
    pub fn push(&mut self, task: Box<dyn Task>) {
        self.tasks.push(task);
    }

    /// The phase's stable name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Number of tasks registered in the phase.
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Whether the phase has no tasks.
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Run every applicable task in order.
    ///
    /// Tasks whose [`Task::should_run`] is false for the context are skipped.
    /// Returns the aggregate outcome:
    ///
    /// - [`TaskOutcome::RebootRequested`] if any task requested a reboot (the
    ///   sequencer should stop running later phases but treat the sequence as
    ///   successful);
    /// - [`TaskOutcome::Done`] if at least one task did work;
    /// - [`TaskOutcome::Skipped`] if every applicable task reported `Skipped`
    ///   (or there were no applicable tasks).
    ///
    /// The first failing task aborts the phase and propagates the error.
    pub fn run(&mut self, ctx: &TaskContext) -> Result<TaskOutcome> {
        let mut aggregate = TaskOutcome::Skipped;
        for task in &mut self.tasks {
            if !task.should_run(ctx) {
                continue;
            }
            match task.run(ctx)? {
                TaskOutcome::RebootRequested => return Ok(TaskOutcome::RebootRequested),
                TaskOutcome::Done => aggregate = TaskOutcome::Done,
                TaskOutcome::Skipped => {}
            }
        }
        Ok(aggregate)
    }
}

impl core::fmt::Debug for Phase {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Phase")
            .field("name", &self.name)
            .field("tasks", &self.tasks.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::MachinedError;
    use crate::runtime::RuntimeMode;
    use crate::sequence::Sequence;
    use crate::task::NamedTask;
    use std::cell::Cell;
    use std::rc::Rc;
    use os_kernel::MachineType;

    fn ctx() -> TaskContext {
        TaskContext::new(
            Sequence::Boot,
            RuntimeMode::Metal,
            MachineType::Worker,
            true,
        )
    }

    #[test]
    fn empty_phase_skips() {
        let mut p = Phase::new("empty");
        assert!(p.is_empty());
        assert_eq!(p.run(&ctx()).unwrap(), TaskOutcome::Skipped);
    }

    #[test]
    fn runs_tasks_in_order() {
        let log = Rc::new(Cell::new(0u32));
        let l1 = log.clone();
        let l2 = log.clone();
        let mut p = Phase::new("boot")
            .with_task(Box::new(NamedTask::new("a", move |_| {
                l1.set(l1.get() * 10 + 1);
                Ok(TaskOutcome::Done)
            })))
            .with_task(Box::new(NamedTask::new("b", move |_| {
                l2.set(l2.get() * 10 + 2);
                Ok(TaskOutcome::Done)
            })));
        assert_eq!(p.len(), 2);
        assert_eq!(p.run(&ctx()).unwrap(), TaskOutcome::Done);
        assert_eq!(log.get(), 12);
    }

    #[test]
    fn gated_task_is_skipped() {
        let install_gate: fn(&TaskContext) -> bool = |c| c.mode().has_disks();
        let mut p = Phase::new("install").with_task(Box::new(
            NamedTask::new("wipe", |_| Ok(TaskOutcome::Done)).with_gate(install_gate),
        ));
        let container_ctx = TaskContext::new(
            Sequence::Install,
            RuntimeMode::Container,
            MachineType::Worker,
            true,
        );
        assert_eq!(p.run(&container_ctx).unwrap(), TaskOutcome::Skipped);
    }

    #[test]
    fn failing_task_aborts_phase() {
        let after = Rc::new(Cell::new(false));
        let a = after.clone();
        let mut p = Phase::new("boot")
            .with_task(Box::new(NamedTask::new("boom", |_| {
                Err(MachinedError::task_failed("boom", "nope"))
            })))
            .with_task(Box::new(NamedTask::new("after", move |_| {
                a.set(true);
                Ok(TaskOutcome::Done)
            })));
        assert_eq!(p.run(&ctx()).unwrap_err().kind(), "task_failed");
        assert!(!after.get(), "later task must not run after a failure");
    }

    #[test]
    fn reboot_request_short_circuits() {
        let after = Rc::new(Cell::new(false));
        let a = after.clone();
        let mut p = Phase::new("reboot")
            .with_task(Box::new(NamedTask::new("reboot", |_| {
                Ok(TaskOutcome::RebootRequested)
            })))
            .with_task(Box::new(NamedTask::new("after", move |_| {
                a.set(true);
                Ok(TaskOutcome::Done)
            })));
        assert_eq!(p.run(&ctx()).unwrap(), TaskOutcome::RebootRequested);
        assert!(!after.get());
    }
}
