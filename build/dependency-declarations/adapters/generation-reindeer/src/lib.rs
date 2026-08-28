//! Structural home for the pinned Reindeer generation adapter.
//!
//! Wave S establishes module discovery only. Process behavior lands in the
//! separately reviewed adapter wave.

#![forbid(unsafe_code)]

include!(concat!(env!("OUT_DIR"), "/items.generated.rs"));
