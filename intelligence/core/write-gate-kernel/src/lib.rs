//! M02-P04-IP-003 — Foundry write-gate kernel.
//!
//! State machine: Proposed → Reviewed → Approved → Executed (linear).
//! Any state may transition to Rejected. Executed and Rejected are terminal.
//! Default-deny: a fresh `WriteGate` starts in `Proposed` and refuses to
//! transition into `Executed` without passing through Reviewed and Approved.
//! Reviewer, Approver, and Executor are separated principals (ADR-0091).
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WriteGateState {
    Proposed,
    Reviewed { reviewer: String },
    Approved { approver: String },
    Executed,
    Rejected { reason: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovalEnvelope {
    pub proposer: String,         // data_class: INTERNAL_ONLY
    pub reviewer: Option<String>, // data_class: INTERNAL_ONLY
    pub approver: Option<String>, // data_class: INTERNAL_ONLY
    pub executor: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WriteGateError {
    /// Default-deny: transition not permitted from current state.
    Denied { from: String, attempted: String },
    /// Reviewer/approver/executor MUST be distinct principals.
    SamePrincipal { role: String, principal: String },
    /// Terminal state cannot transition.
    Terminal { state: String },
}

impl fmt::Display for WriteGateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteGateError::Denied { from, attempted } => {
                write!(f, "write-gate denied: {from} -> {attempted}")
            }
            WriteGateError::SamePrincipal { role, principal } => {
                write!(f, "write-gate same-principal {role}={principal}")
            }
            WriteGateError::Terminal { state } => {
                write!(f, "write-gate terminal: {state}")
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WriteGate {
    state: WriteGateState,
    envelope: ApprovalEnvelope,
}

impl WriteGate {
    /// Default-deny: every new gate starts Proposed.
    pub fn propose(proposer: String) -> Self {
        Self {
            state: WriteGateState::Proposed,
            envelope: ApprovalEnvelope {
                proposer,
                reviewer: None,
                approver: None,
                executor: None,
            },
        }
    }

    pub fn state(&self) -> &WriteGateState {
        &self.state
    }

    pub fn envelope(&self) -> &ApprovalEnvelope {
        &self.envelope
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.state,
            WriteGateState::Executed | WriteGateState::Rejected { .. }
        )
    }

    pub fn review(&mut self, reviewer: String) -> Result<(), WriteGateError> {
        if self.envelope.proposer == reviewer {
            return Err(WriteGateError::SamePrincipal {
                role: "reviewer".into(),
                principal: reviewer,
            });
        }
        match &self.state {
            WriteGateState::Proposed => {
                self.envelope.reviewer = Some(reviewer.clone());
                self.state = WriteGateState::Reviewed { reviewer };
                Ok(())
            }
            other => Err(WriteGateError::Denied {
                from: state_name(other).into(),
                attempted: "review".into(),
            }),
        }
    }

    pub fn approve(&mut self, approver: String) -> Result<(), WriteGateError> {
        if self.envelope.proposer == approver
            || self.envelope.reviewer.as_deref() == Some(approver.as_str())
        {
            return Err(WriteGateError::SamePrincipal {
                role: "approver".into(),
                principal: approver,
            });
        }
        match &self.state {
            WriteGateState::Reviewed { .. } => {
                self.envelope.approver = Some(approver.clone());
                self.state = WriteGateState::Approved { approver };
                Ok(())
            }
            other => Err(WriteGateError::Denied {
                from: state_name(other).into(),
                attempted: "approve".into(),
            }),
        }
    }

    pub fn execute(&mut self, executor: String) -> Result<(), WriteGateError> {
        if self.envelope.proposer == executor
            || self.envelope.reviewer.as_deref() == Some(executor.as_str())
            || self.envelope.approver.as_deref() == Some(executor.as_str())
        {
            return Err(WriteGateError::SamePrincipal {
                role: "executor".into(),
                principal: executor,
            });
        }
        match &self.state {
            WriteGateState::Approved { .. } => {
                self.envelope.executor = Some(executor);
                self.state = WriteGateState::Executed;
                Ok(())
            }
            other => Err(WriteGateError::Denied {
                from: state_name(other).into(),
                attempted: "execute".into(),
            }),
        }
    }

    pub fn reject(&mut self, reason: String) -> Result<(), WriteGateError> {
        if self.is_terminal() {
            return Err(WriteGateError::Terminal {
                state: state_name(&self.state).into(),
            });
        }
        self.state = WriteGateState::Rejected { reason };
        Ok(())
    }
}

fn state_name(s: &WriteGateState) -> &'static str {
    match s {
        WriteGateState::Proposed => "Proposed",
        WriteGateState::Reviewed { .. } => "Reviewed",
        WriteGateState::Approved { .. } => "Approved",
        WriteGateState::Executed => "Executed",
        WriteGateState::Rejected { .. } => "Rejected",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_path_proposed_to_executed() {
        let mut g = WriteGate::propose("alice".into());
        g.review("bob".into()).unwrap();
        g.approve("carol".into()).unwrap();
        g.execute("dave".into()).unwrap();
        assert_eq!(g.state(), &WriteGateState::Executed);
        assert!(g.is_terminal());
    }

    #[test]
    fn deny_by_default_skip_review() {
        let mut g = WriteGate::propose("alice".into());
        let err = g.approve("carol".into()).unwrap_err();
        assert!(matches!(err, WriteGateError::Denied { .. }));
    }

    #[test]
    fn deny_by_default_skip_approve() {
        let mut g = WriteGate::propose("alice".into());
        g.review("bob".into()).unwrap();
        let err = g.execute("dave".into()).unwrap_err();
        assert!(matches!(err, WriteGateError::Denied { .. }));
    }

    #[test]
    fn reject_from_any_non_terminal() {
        let mut g = WriteGate::propose("alice".into());
        g.review("bob".into()).unwrap();
        g.reject("did not pass policy".into()).unwrap();
        assert!(matches!(g.state(), WriteGateState::Rejected { .. }));
        assert!(g.is_terminal());
    }

    #[test]
    fn executed_is_terminal() {
        let mut g = WriteGate::propose("alice".into());
        g.review("bob".into()).unwrap();
        g.approve("carol".into()).unwrap();
        g.execute("dave".into()).unwrap();
        let err = g.reject("late".into()).unwrap_err();
        assert!(matches!(err, WriteGateError::Terminal { .. }));
    }

    #[test]
    fn separation_of_duties_reviewer_distinct() {
        let mut g = WriteGate::propose("alice".into());
        let err = g.review("alice".into()).unwrap_err();
        assert!(matches!(err, WriteGateError::SamePrincipal { .. }));
    }

    #[test]
    fn separation_of_duties_approver_distinct() {
        let mut g = WriteGate::propose("alice".into());
        g.review("bob".into()).unwrap();
        let err = g.approve("bob".into()).unwrap_err();
        assert!(matches!(err, WriteGateError::SamePrincipal { .. }));
    }

    #[test]
    fn separation_of_duties_executor_distinct() {
        let mut g = WriteGate::propose("alice".into());
        g.review("bob".into()).unwrap();
        g.approve("carol".into()).unwrap();
        let err = g.execute("carol".into()).unwrap_err();
        assert!(matches!(err, WriteGateError::SamePrincipal { .. }));
    }
}
