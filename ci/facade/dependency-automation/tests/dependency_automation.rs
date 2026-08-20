// ADR-0535 dependency automation gate: live-tree GREEN plus RED fixtures proving the gate rejects
// missing policy, closed-schema drift, Rust pin split-brain, and external updater config residue.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

#[path = "dependency_automation/contract.rs"]
mod contract;
#[path = "dependency_automation/declared_paths.rs"]
mod declared_paths;
#[path = "dependency_automation/helpers.rs"]
mod helpers;
#[path = "dependency_automation/overlay.rs"]
mod overlay;
