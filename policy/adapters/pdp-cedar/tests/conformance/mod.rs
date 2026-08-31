//! Shared substrate for the G004 Cedar conformance suite.
//!
//! The suite was one 1,537-line file. It is split by authorization concern so
//! each file fits the repository budget; the fixtures those concerns share
//! live here. The split changed no assertion and no test name.
#![allow(dead_code)]

pub mod fixtures;
pub mod overlays;
pub mod seeds;

pub use fixtures::*;
pub use overlays::*;
pub use seeds::*;
