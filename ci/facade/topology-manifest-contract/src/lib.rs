//! CELL-001R cell-topology manifest contract gate marker.
//!
//! The executable validation contract is pinned by the Rust integration test in
//! `tests/cell_topology_manifest_contract.rs` so this crate can be wired through
//! cloud-ci/Buck without extending the retired local gate CLI authority.
#![forbid(unsafe_code)]

/// Cloud-ci gate id for the CELL-001R manifest-contract validation slice.
pub const GATE_ID: &str = "oya-cloud-ci-cell-topology-manifest-contract";
