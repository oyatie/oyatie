//! Retention policy types for data-boundary classification.
//!
//! [`ClassificationLevel`] is an ordered severity tier that maps the raw
//! [`DataClass`] vocabulary to a 4-level operational sensitivity scale.
//! [`DataClassMatcher`] provides predicate logic for testing membership in
//! named class sets without importing the full privacy-evaluation graph.
//! [`RetentionPolicy`] captures the declared retention window and mandatory
//! purge action for a classified data object. This is a fixed compatibility
//! namespace, not an item inventory: membership is the sorted
//! `src/retention_policy_items/` directory rendered by `build.rs`.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use crate::DataClass;
use std::time::Duration;

include!(concat!(env!("OUT_DIR"), "/retention_policy.generated.rs"));

#[cfg(test)]
mod tests {
    use super::{ClassificationLevel, DataClassMatcher, PurgeAction, RetentionPolicy};
    use crate::DataClass;

    include!(concat!(
        env!("OUT_DIR"),
        "/retention_policy_tests.generated.rs"
    ));
}
