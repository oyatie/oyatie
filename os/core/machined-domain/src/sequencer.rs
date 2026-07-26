//! The sequencer: builds the ordered list of [`Phase`]s for a [`Sequence`] and
//! runs them, mirroring `siderolabs/talos` `runtime.Sequencer`.
//!
//! The real Talos sequencer assembles platform/role-specific task lists. Here
//! the [`Sequencer`] validates a requested sequence against the runtime and the
//! machine [`StateMachine`], runs each phase's tasks via a [`TaskContext`], and
//! reports a [`SequenceReport`].

use crate::error::{MachinedError, Result};
use crate::events::{EventKind, EventStream};
use crate::phase::Phase;
use crate::runtime::MachineRuntime;
use crate::sequence::Sequence;
use crate::state_machine::{MachineState, StateMachine};
use crate::task::{TaskContext, TaskOutcome};

/// The outcome of running a whole sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceReport {
    /// The sequence that ran.
    pub sequence: Sequence,
    /// The phases that were executed (in order), with their aggregate outcome.
    pub phases: Vec<(String, TaskOutcome)>,
    /// The machine state after the sequence completed.
    pub final_state: MachineState,
    /// Whether the sequence ended early because a reboot was requested.
    pub rebooted: bool,
}

impl SequenceReport {
    /// Number of phases run.
    pub fn phase_count(&self) -> usize {
        self.phases.len()
    }
}

/// Drives sequences against a runtime and a machine state machine.
pub struct Sequencer<R: MachineRuntime> {
    runtime: R,
    machine: StateMachine,
    events: EventStream,
}

impl<R: MachineRuntime> Sequencer<R> {
    /// Build a sequencer over the given runtime, starting from the initial
    /// machine state.
    pub fn new(runtime: R) -> Self {
        Sequencer {
            runtime,
            machine: StateMachine::new(),
            events: EventStream::default(),
        }
    }

    /// Borrow the underlying runtime.
    pub fn runtime(&self) -> &R {
        &self.runtime
    }

    /// The sequencer's event stream (sequence/phase/machine-state events).
    pub fn events(&self) -> &EventStream {
        &self.events
    }

    /// The current machine state.
    pub fn state(&self) -> MachineState {
        self.machine.state()
    }

    /// Validate that the requested sequence may run right now: it must be legal
    /// in the runtime mode, satisfy config preconditions, and be allowed by the
    /// machine state machine.
    pub fn validate(&self, seq: Sequence) -> Result<()> {
        let mode = self.runtime.mode();
        if !seq.allowed_in(mode) {
            return Err(MachinedError::sequence_not_allowed(format!(
                "{} not allowed in {} mode",
                seq.as_str(),
                mode.as_str()
            )));
        }
        if seq.requires_config() && !self.runtime.is_configured() {
            return Err(MachinedError::sequence_not_allowed(format!(
                "{} requires machine config",
                seq.as_str()
            )));
        }
        self.machine.validate_start(seq)?;
        Ok(())
    }

    /// Build the [`TaskContext`] for the current runtime and a given sequence.
    fn context(&self, seq: Sequence) -> TaskContext {
        TaskContext::new(
            seq,
            self.runtime.mode(),
            self.runtime.machine_type(),
            self.runtime.is_configured(),
        )
    }

    /// Run a fully-built sequence: validate, transition the state machine into
    /// the running state, run each phase in order, then complete.
    ///
    /// The `phases` are supplied by the caller (the real Talos sequencer builds
    /// these from platform/role; tests build them directly), keeping this
    /// engine decoupled from the concrete task catalog.
    pub fn run(&mut self, seq: Sequence, mut phases: Vec<Phase>) -> Result<SequenceReport> {
        self.validate(seq)?;
        self.machine.begin(seq)?;
        self.events
            .publish(EventKind::SequenceStart { sequence: seq });
        self.events.publish(EventKind::MachineStateChange {
            state: self.machine.state(),
        });

        let ctx = self.context(seq);
        let mut executed = Vec::with_capacity(phases.len());
        let mut rebooted = false;

        for phase in &mut phases {
            let name = phase.name().to_string();
            self.events.publish(EventKind::PhaseStart {
                phase: name.clone(),
            });
            let outcome = phase.run(&ctx)?;
            self.events.publish(EventKind::PhaseFinish {
                phase: name.clone(),
            });
            executed.push((name, outcome.clone()));
            if outcome == TaskOutcome::RebootRequested {
                rebooted = true;
                break;
            }
        }

        self.machine.complete(seq)?;
        self.events.publish(EventKind::MachineStateChange {
            state: self.machine.state(),
        });
        self.events.publish(EventKind::SequenceFinish {
            sequence: seq,
            rebooted,
        });

        Ok(SequenceReport {
            sequence: seq,
            phases: executed,
            final_state: self.machine.state(),
            rebooted,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::MachinedError;
    use crate::runtime::{InMemoryRuntime, RuntimeMode};
    use crate::task::NamedTask;
    use os_kernel::MachineType;

    fn boot_phases() -> Vec<Phase> {
        vec![
            Phase::new("mount").with_task(Box::new(NamedTask::new("mountRoot", |_| {
                Ok(TaskOutcome::Done)
            }))),
            Phase::new("services").with_task(Box::new(NamedTask::new("startServices", |_| {
                Ok(TaskOutcome::Done)
            }))),
        ]
    }

    #[test]
    fn boot_runs_all_phases() {
        let rt =
            InMemoryRuntime::new(RuntimeMode::Metal, MachineType::ControlPlane).with_config("cp-1");
        let mut seq = Sequencer::new(rt);
        let report = seq.run(Sequence::Boot, boot_phases()).unwrap();
        assert_eq!(report.phase_count(), 2);
        assert_eq!(report.final_state, MachineState::Running);
        assert!(!report.rebooted);
    }

    #[test]
    fn install_rejected_in_container() {
        let rt =
            InMemoryRuntime::new(RuntimeMode::Container, MachineType::Worker).with_config("c-1");
        let mut seq = Sequencer::new(rt);
        let err = seq.run(Sequence::Install, vec![]).unwrap_err();
        assert_eq!(err.kind(), "sequence_not_allowed");
    }

    #[test]
    fn boot_requires_config() {
        let rt = InMemoryRuntime::new(RuntimeMode::Metal, MachineType::Worker);
        let seq = Sequencer::new(rt);
        let err = seq.validate(Sequence::Boot).unwrap_err();
        assert_eq!(err.kind(), "sequence_not_allowed");
    }

    #[test]
    fn reboot_short_circuits_remaining_phases() {
        let rt = InMemoryRuntime::new(RuntimeMode::Metal, MachineType::Worker).with_config("w-1");
        let mut seq = Sequencer::new(rt);
        // Boot first so we are Running and may reboot.
        seq.run(Sequence::Boot, boot_phases()).unwrap();
        let phases = vec![
            Phase::new("stopServices")
                .with_task(Box::new(NamedTask::new("stop", |_| Ok(TaskOutcome::Done)))),
            Phase::new("reboot").with_task(Box::new(NamedTask::new("reboot", |_| {
                Ok(TaskOutcome::RebootRequested)
            }))),
            Phase::new("never").with_task(Box::new(NamedTask::new("never", |_| {
                panic!("must not run after reboot")
            }))),
        ];
        let report = seq.run(Sequence::Reboot, phases).unwrap();
        assert!(report.rebooted);
        assert_eq!(report.phase_count(), 2);
        assert_eq!(report.final_state, MachineState::Rebooting);
    }

    #[test]
    fn task_failure_aborts_sequence() {
        let rt = InMemoryRuntime::new(RuntimeMode::Metal, MachineType::Worker).with_config("w-1");
        let mut seq = Sequencer::new(rt);
        let phases = vec![
            Phase::new("boot").with_task(Box::new(NamedTask::new("mount", |_| {
                Err(MachinedError::task_failed("mount", "EBUSY"))
            }))),
        ];
        let err = seq.run(Sequence::Boot, phases).unwrap_err();
        assert_eq!(err.kind(), "task_failed");
        // State machine entered Booting but the sequence aborted before complete.
        assert_eq!(seq.state(), MachineState::Booting);
    }

    #[test]
    fn run_emits_sequence_and_phase_events() {
        let rt =
            InMemoryRuntime::new(RuntimeMode::Metal, MachineType::ControlPlane).with_config("cp-1");
        let mut seq = Sequencer::new(rt);
        seq.run(Sequence::Boot, boot_phases()).unwrap();
        assert_eq!(seq.events().of_type("sequence.start").len(), 1);
        assert_eq!(seq.events().of_type("sequence.finish").len(), 1);
        // Two phases => two phase.start and two phase.finish events.
        assert_eq!(seq.events().of_type("phase.start").len(), 2);
        assert_eq!(seq.events().of_type("phase.finish").len(), 2);
        // The start event precedes the first phase event.
        let start = seq.events().of_type("sequence.start")[0].id;
        let first_phase = seq.events().of_type("phase.start")[0].id;
        assert!(start < first_phase);
    }

    #[test]
    fn aborted_sequence_emits_no_finish() {
        let rt = InMemoryRuntime::new(RuntimeMode::Metal, MachineType::Worker).with_config("w-1");
        let mut seq = Sequencer::new(rt);
        let phases = vec![
            Phase::new("boot").with_task(Box::new(NamedTask::new("mount", |_| {
                Err(MachinedError::task_failed("mount", "EBUSY"))
            }))),
        ];
        let _ = seq.run(Sequence::Boot, phases).unwrap_err();
        assert_eq!(seq.events().of_type("sequence.start").len(), 1);
        // No finish event because the sequence aborted.
        assert_eq!(seq.events().of_type("sequence.finish").len(), 0);
    }

    #[test]
    fn double_boot_rejected_by_state_machine() {
        let rt = InMemoryRuntime::new(RuntimeMode::Metal, MachineType::Worker).with_config("w-1");
        let mut seq = Sequencer::new(rt);
        seq.run(Sequence::Boot, boot_phases()).unwrap();
        let err = seq.run(Sequence::Boot, boot_phases()).unwrap_err();
        assert_eq!(err.kind(), "sequence_not_allowed");
    }
}
