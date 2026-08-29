//! Exact-source Reindeer adapter for dependency declarations.
//!
//! One pinned provider profile produces a typed graph and rendered BUCK bytes.

#![forbid(unsafe_code)]

include!(concat!(env!("OUT_DIR"), "/items.generated.rs"));
