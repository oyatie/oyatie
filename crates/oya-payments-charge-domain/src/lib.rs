//! Payments charge-BC domain — `Charge` aggregate, invariants, and
//! `ChargeRepository` port.
//!
//! Wave 15-IMPL-truth-up scaffold; full state-machine + COPPA invariant
//! implementation in IP-002. Zero I/O. Domain events emitted through the
//! `DomainEventEnvelope` shape declared here.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use oya_payments_charge_kernel::{ChargeId, ChargeState};

/// Charge aggregate root. State-machine guards land in IP-002.
#[allow(dead_code)]
pub struct Charge {
    id: ChargeId,
    state: ChargeState,
}

/// Repository port. Adapter impls land in IP-004 / IP-018 lanes.
pub trait ChargeRepository {
    type Error;
    fn save(&self, charge: &Charge) -> Result<(), Self::Error>;
}
