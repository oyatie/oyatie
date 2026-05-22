//! Payments charge-BC kernel — sealed port traits, value objects, and
//! domain-event envelopes for the `charge` bounded context.
//!
//! Wave 15-IMPL-truth-up scaffold; full implementation in IP-001.
//!
//! Per ADR-0056 (12-layer enum, port-in-kernel) and ADR-0212 (buildability
//! doctrine): the kernel holds neutral value types and port trait shapes
//! consumed by domain/adapter/usecase layers. No I/O, no tokio, no HTTP.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ChargeId(pub String);

/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct PaymentMethodId(pub String);

/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct IdempotencyKey(pub String);

/// ISO 4217 currency code stub; full validation lands in IP-001.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Currency(pub String);

/// Amount in minor units (e.g. cents).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct AmountMinor(pub i64);

/// Charge state machine. Transitions enforced in `oya-payments-charge-domain`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ChargeState {
    Authorized,
    Captured,
    Voided,
    Declined,
    Errored,
}

/// Audience taxonomy from ADR-0292 / ADR-0244.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum AudienceType {
    B2bTenant,
    B2cConsumer,
    PartnerAgency,
}
