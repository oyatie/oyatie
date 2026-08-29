//! Pure dependency-declaration reconciliation over immutable typed values.
//!
//! Generation, independent syntax projection, validation, and publication are
//! composed through ports; filesystem, process, network, SCM, and campaigns stay outside.

#![forbid(unsafe_code)]

include!(concat!(env!("OUT_DIR"), "/items.generated.rs"));
