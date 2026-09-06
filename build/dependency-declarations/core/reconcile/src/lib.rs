//! Pure dependency-declaration reconciliation kernel.
//!
//! The build script assembles direct semantic items in deterministic byte order.

#![forbid(unsafe_code)]

include!(concat!(env!("OUT_DIR"), "/items.generated.rs"));
