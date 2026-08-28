//! Structural home for the pure dependency-declaration reconciliation kernel.
//!
//! Wave S establishes module discovery only. Reconciliation behavior lands in
//! the separately reviewed kernel wave.

#![forbid(unsafe_code)]

include!(concat!(env!("OUT_DIR"), "/items.generated.rs"));
