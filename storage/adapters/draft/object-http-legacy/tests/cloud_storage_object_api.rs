// Compatibility tests intentionally use panic helpers to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

include!(concat!(env!("OUT_DIR"), "/integration.generated.rs"));
