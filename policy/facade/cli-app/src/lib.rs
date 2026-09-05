//! Offline policy qualification command core.
#![forbid(unsafe_code)]

mod command;

pub use command::{CommandError, CommandOutput, qualify_json};
