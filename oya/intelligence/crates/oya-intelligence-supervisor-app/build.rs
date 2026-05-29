//! build.rs — CLI capability probe + capability-seed TOML emitter (placeholder).
//!
//! Wave 2 will implement:
//!   - Probe each CLI's `--version` / `--show-config` to capture stop_hook_supported,
//!     hook event vocabulary, and config path (persisted as capability-seed TOML
//!     at `$OUT_DIR/supervisor-capability-seed.toml`)
//!   - Emit `cargo:rerun-if-changed` directives for each CLI binary path
//!   - Validate generated TOML parses back through the hand-rolled parser
//!     (no serde — HARD CONSTRAINT per v4 §B.0 L96)
//!   - Per v6 BLOCKER-7..12: generate fixture pair index for the meta-test
//!     `tests/conformance.rs::fixture_pair_registry_complete`

fn main() {
    // Placeholder: emit rerun directives so incremental builds work.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/main.rs");
}
