//! Data Use Boundary policy-gate slice.
//!
//! This module keeps the first PRIVACY-001 gate fixture close to the DUB value
//! types without claiming a full Cedar/runtime integration. It is a fixed
//! compatibility namespace, not an item inventory: membership is the sorted
//! `src/policy_gate_items/` directory rendered by `build.rs`.

use crate::{
    ClassificationLevel, ConsentScope, DataClass, DataClassification, DataUseAttributes,
    DataUseDenialReason, PrivacyDataClass, Purpose,
};

include!(concat!(env!("OUT_DIR"), "/policy_gate.generated.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AgeBand, DataClass, DataClassification, OperationalDataClass, PrivacyDataClass, Purpose,
        SubjectClass,
    };

    include!(concat!(env!("OUT_DIR"), "/policy_gate_tests.generated.rs"));
}
