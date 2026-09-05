//! Qualified policy publication and verified embedded-engine composition.
#![forbid(unsafe_code)]

mod command;
mod engine;
mod qualification;
mod source;

pub use command::{CommandError, CommandOutput, qualify_json};
pub use engine::{EngineLoadError, PolicyEngine};
pub use qualification::{PreparedPolicy, QualificationError, QualificationReport};
pub use source::{DecisionExpectation, PolicyCase, PolicyProject, PolicySource};
