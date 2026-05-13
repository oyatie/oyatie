//! `oya-check-statelessness` — M02 implementation pending.
//!
//! Verifies that `application`, `worker`, and presentation-layer crates
//! contain no module-level mutable state (`static mut`, `lazy_static!`,
//! `once_cell::sync::Lazy` with interior mutability).
//!
//! Running in `--report-only` mode until M02 substrate phase completes.
