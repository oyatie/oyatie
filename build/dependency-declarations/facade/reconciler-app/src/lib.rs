//! Structural dependency-declaration reconciliation composition root.
//!
//! Wave S is deliberately not a serving implementation. The process remains
//! fail-closed until the separately reviewed facade wave lands.

#![forbid(unsafe_code)]

use std::process::ExitCode;

include!(concat!(env!("OUT_DIR"), "/items.generated.rs"));

/// Refuses execution while this crate is structural scaffolding only.
pub fn structural_not_ready() -> ExitCode {
    ExitCode::FAILURE
}
