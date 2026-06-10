//! Cross-cutting traits used workspace-wide: runnable services and sequence
//! hooks, modeled on Talos `runtime` controller/service abstractions.

use crate::error::Result;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

/// Lifecycle state of a [`Runnable`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunState {
    /// Created but not started.
    Initialized,
    /// Preparing to run (pulling images, writing config, ...).
    Preparing,
    /// Actively running.
    Running,
    /// Stopped cleanly.
    Stopped,
    /// Stopped due to a failure.
    Failed,
}

impl RunState {
    /// Whether a transition from `self` to `next` is permitted.
    ///
    /// The lifecycle is: Initialized -> Preparing -> Running -> (Stopped |
    /// Failed). A Stopped or Failed runnable may be re-Initialized (restart).
    pub fn can_transition_to(self, next: RunState) -> bool {
        use RunState::{Failed, Initialized, Preparing, Running, Stopped};
        matches!(
            (self, next),
            (Initialized, Preparing)
                | (Preparing, Running | Failed)
                | (Running, Stopped | Failed)
                | (Stopped | Failed, Initialized)
        )
    }

    /// Whether this is a terminal-for-now state.
    pub fn is_terminal(self) -> bool {
        matches!(self, RunState::Stopped | RunState::Failed)
    }
}

/// A long-running component (a "service") that the runtime supervises.
///
/// Mirrors the Talos `system.Service`/`Runnable` notion: it has an identity,
/// can be started and stopped, and reports health.
pub trait Runnable {
    /// Stable identifier for this runnable (e.g. `"kubelet"`).
    fn id(&self) -> &str;

    /// Start the runnable. Should be idempotent if already running.
    fn start(&mut self) -> Result<()>;

    /// Stop the runnable. Should be idempotent if already stopped.
    fn stop(&mut self) -> Result<()>;

    /// Current lifecycle state.
    fn state(&self) -> RunState;

    /// Whether the runnable is healthy. Default: healthy iff running.
    fn is_healthy(&self) -> bool {
        self.state() == RunState::Running
    }
}

/// A discrete phase in a boot/upgrade/reset sequence.
///
/// Talos models machine lifecycle as ordered sequences (Boot, Upgrade, Reset,
/// Shutdown) composed of named phases each containing tasks. This is the
/// minimal hook surface other crates build on.
pub trait SequenceHook {
    /// Human-readable phase name.
    fn name(&self) -> &str;

    /// Run the phase. Returning `Err` aborts the sequence.
    fn run(&mut self) -> Result<()>;

    /// Whether this phase may be skipped when a previous run already satisfied
    /// its postconditions. Default: not skippable.
    fn is_idempotent(&self) -> bool {
        false
    }
}

/// Orders and runs a list of [`SequenceHook`] phases, recording the names of
/// phases that completed successfully.
#[derive(Default)]
pub struct Sequence {
    phases: Vec<Box<dyn SequenceHook>>,
    completed: Vec<String>,
}

impl Sequence {
    /// An empty sequence.
    pub fn new() -> Self {
        Sequence {
            phases: Vec::new(),
            completed: Vec::new(),
        }
    }

    /// Append a phase.
    pub fn push(&mut self, phase: Box<dyn SequenceHook>) {
        self.phases.push(phase);
    }

    /// Names of phases that completed, in order.
    pub fn completed(&self) -> &[String] {
        &self.completed
    }

    /// Run every phase in order, stopping at the first error. On error the
    /// error is returned and `completed()` reflects phases that succeeded.
    pub fn run_all(&mut self) -> Result<()> {
        for phase in &mut self.phases {
            phase.run()?;
            self.completed.push(String::from(phase.name()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    struct DummyService {
        id: String,
        state: RunState,
    }

    impl Runnable for DummyService {
        fn id(&self) -> &str {
            &self.id
        }
        fn start(&mut self) -> Result<()> {
            self.state = RunState::Running;
            Ok(())
        }
        fn stop(&mut self) -> Result<()> {
            self.state = RunState::Stopped;
            Ok(())
        }
        fn state(&self) -> RunState {
            self.state
        }
    }

    #[test]
    fn runnable_lifecycle() {
        let mut s = DummyService {
            id: String::from("kubelet"),
            state: RunState::Initialized,
        };
        assert_eq!(s.id(), "kubelet");
        assert!(!s.is_healthy());
        s.start().unwrap();
        assert!(s.is_healthy());
        s.stop().unwrap();
        assert_eq!(s.state(), RunState::Stopped);
    }

    #[test]
    fn state_transition_rules() {
        assert!(RunState::Initialized.can_transition_to(RunState::Preparing));
        assert!(RunState::Running.can_transition_to(RunState::Failed));
        assert!(RunState::Failed.can_transition_to(RunState::Initialized));
        assert!(!RunState::Initialized.can_transition_to(RunState::Running));
        assert!(!RunState::Stopped.can_transition_to(RunState::Running));
        assert!(RunState::Stopped.is_terminal());
        assert!(!RunState::Running.is_terminal());
    }

    struct OkPhase(String);
    impl SequenceHook for OkPhase {
        fn name(&self) -> &str {
            &self.0
        }
        fn run(&mut self) -> Result<()> {
            Ok(())
        }
    }

    struct FailPhase(String);
    impl SequenceHook for FailPhase {
        fn name(&self) -> &str {
            &self.0
        }
        fn run(&mut self) -> Result<()> {
            Err(Error::invalid_state("phase precondition failed"))
        }
    }

    #[test]
    fn sequence_runs_until_error() {
        let mut seq = Sequence::new();
        seq.push(Box::new(OkPhase(String::from("mount"))));
        seq.push(Box::new(OkPhase(String::from("network"))));
        seq.push(Box::new(FailPhase(String::from("kubelet"))));
        seq.push(Box::new(OkPhase(String::from("never"))));

        let err = seq.run_all().unwrap_err();
        assert_eq!(err.kind(), "invalid_state");
        assert_eq!(
            seq.completed(),
            &[String::from("mount"), String::from("network")]
        );
    }

    #[test]
    fn sequence_all_ok() {
        let mut seq = Sequence::new();
        seq.push(Box::new(OkPhase(String::from("a"))));
        seq.push(Box::new(OkPhase(String::from("b"))));
        assert!(seq.run_all().is_ok());
        assert_eq!(seq.completed().len(), 2);
    }
}
