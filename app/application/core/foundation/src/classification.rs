//! Data-classification projections for audit, evidence, and capability records.

use crate::*;

use data_boundary_kernel::{DataClass, DataClassification, OperationalDataClass};
use intelligence_capability_domain::Capability;

// Audit-chain storage remains hash-compatible with legacy `DataClass` payloads;
// foundation call sites express audit markers through `DataClassification` so
// new code does not construct operational markers as privacy data classes.
pub(crate) fn internal_audit_classifications() -> [DataClassification; 2] {
    [
        DataClassification::from(DataClass::InternalOnly),
        DataClassification::from(OperationalDataClass::Audit),
    ]
}

// Audit, evidence, run, step, telemetry, and MCP records still persist the
// shared `DataClass` vocabulary. Enforcement reads the typed privacy classes
// from `Capability`; these helpers make each record-facing projection explicit.
pub(crate) fn capability_record_classifications(
    capability: &Capability,
) -> Vec<DataClassification> {
    capability
        .touched_privacy_data_classes()
        .iter()
        .copied()
        .map(DataClassification::from)
        .collect()
}

pub(crate) fn capability_record_data_class_labels(capability: &Capability) -> String {
    capability
        .touched_privacy_data_classes()
        .iter()
        .map(|data_class| data_class.label())
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn behavioral_audit_classifications() -> [DataClassification; 2] {
    [
        DataClassification::from(DataClass::BehavioralTenantProduct),
        DataClassification::from(OperationalDataClass::Audit),
    ]
}
