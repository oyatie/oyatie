//! # registry-drift
//!
//! The committed==regenerated enforcement target (PHASE-0-FIREWALL-PLAN §5.3). The
//! contract lives in `tests/ci_inventory_registry_drift.rs`: re-run the producer in a sandbox and
//! byte-diff against the committed `accounting-registry.generated.json`. A hand-edit to
//! any generated face fails the test, making drift structurally impossible.
//!
//! This lib is intentionally empty — there is no runtime logic, only the gate test.
#![forbid(unsafe_code)]
