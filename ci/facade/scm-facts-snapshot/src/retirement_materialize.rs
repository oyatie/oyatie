//! Materialize history-only retirement facts from a Git object source.

use std::collections::BTreeSet;

use serde_json::{Value, json};

use super::{
    CONTROL_PLANE_PATH, PROTECTED_BASE_REF, RECEIPT_ROOT, ReceiptStage, RetirementControlPlane,
    RetirementMaterializationContext, RetirementObjectSource, build_equivalence_index,
    canonical_value_sha256, classify_stage, closure_preparation_link, control_entry_value,
    coverage_scope, entries_by_path, expected_receipt_paths, find_linked_preparation, input_fact,
    parse_closed_json, receipt_baseline, receipt_for_stage, receipt_root_inventory,
    require_predecessor_baseline, require_regular, sha256_digest, validate_control_plane,
    validate_event_identity, validate_oid, validate_predecessor_inputs, validate_receipt_identity,
    validate_receipt_population, validate_selector_coverage,
};


include!("retirement_materialize_a.rs");
include!("retirement_materialize_b.rs");
