// ADR-0083 Tier 3: integration tests assert invariants with panic helpers.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

include!(concat!(env!("OUT_DIR"), "/integration.generated.rs"));
