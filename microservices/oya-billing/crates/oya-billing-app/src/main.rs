//! Billing composition-root binary entry point (ADR-0478).
//!
//! Reads config from environment, wires adapters, and starts the HTTP listener.
//!
//! ## Honest-claims note
//!
//! non_claim: HTTP listener start-up deferred to ADR-0478 D3-D4. Binary exits
//! 0 after boot validation to enable smoke-test CI runs.
#![forbid(unsafe_code)]

fn main() {
    // TODO: implement per ADR-0478 D1-D5
}
