//! Structural home for qualified filesystem publication.
//!
//! Wave S establishes module discovery only. Filesystem behavior lands in the
//! separately reviewed adapter wave.

#![forbid(unsafe_code)]

include!(concat!(env!("OUT_DIR"), "/items.generated.rs"));
