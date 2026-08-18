// ADR-0535 dependency automation gate: live-tree GREEN plus RED fixtures proving the gate rejects
// missing policy, closed-schema drift, Rust pin split-brain, and external updater config residue.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod contract;
mod declared_paths;
mod helpers;
mod overlay;
