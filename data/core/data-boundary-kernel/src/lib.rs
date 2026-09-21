//! Data Use Boundary kernel.
//!
//! Pure value types for classifying fields and checking purpose-bound use.
//! The classification vocabulary is defined by `data-classification` and
//! re-exported here under its historical paths; purpose, consent and
//! evaluator items are the sorted `src/items/` directory rendered by
//! `build.rs`, so adding, renaming or removing an item never edits this root.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod policy_gate;
pub mod retention_policy;

pub use data_classification::{
    Classified, DataClass, DataClassification, NonPrivacyDataClass, OperationalDataClass,
    PRIVACY_PROGRAM_DATA_CLASS_LABELS, PrivacyDataClass, SubjectDataMarker,
    data_classes_from_privacy_data_classes, most_restrictive_privacy_data_class,
    parse_data_class_label, parse_data_class_pascal_label, parse_operational_data_class_label,
    parse_subject_data_marker_label, privacy_data_classes_from,
};
pub use retention_policy::{ClassificationLevel, DataClassMatcher, PurgeAction, RetentionPolicy};

include!(concat!(env!("OUT_DIR"), "/boundary.generated.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    include!(concat!(env!("OUT_DIR"), "/boundary_tests.generated.rs"));
}
