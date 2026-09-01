//! API-first dependency-declaration reconciliation composition.

#![forbid(unsafe_code)]

pub use dependency_declarations_reconcile::{
    DigestV1, ReconciliationRequestV1, ReconciliationResultV1,
};

include!(concat!(env!("OUT_DIR"), "/items.generated.rs"));
