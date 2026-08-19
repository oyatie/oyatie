//! This crate's own sources, embedded so the engine-identity axis can hash them.
//!
//! GENERATED; regenerate rather than edit. Only files this crate OWNS are listed — the facade
//! assembles the whole-engine digest, because the facade is the one place the whole engine is
//! legitimately visible without inverting the dependency direction.

/// Crate-relative path and contents of each source this crate owns, sorted by path.
pub const CRATE_SOURCES: &[(&str, &str)] = &[
    ("cli/mod.rs", include_str!("cli/mod.rs")),
    ("cli/pipeline.rs", include_str!("cli/pipeline.rs")),
    ("cli/seams.rs", include_str!("cli/seams.rs")),
    ("driver/mod.rs", include_str!("driver/mod.rs")),
    ("driver/pipeline.rs", include_str!("driver/pipeline.rs")),
    ("driver/port_any.rs", include_str!("driver/port_any.rs")),
    ("driver/port_go.rs", include_str!("driver/port_go.rs")),
    ("driver/probe.rs", include_str!("driver/probe.rs")),
    ("driver/report.rs", include_str!("driver/report.rs")),
    ("driver/smoke.rs", include_str!("driver/smoke.rs")),
    ("engine.rs", include_str!("engine.rs")),
    ("lib.rs", include_str!("lib.rs")),
    ("main.rs", include_str!("main.rs")),
    ("receipt_codec.rs", include_str!("receipt_codec.rs")),
    ("receipt_e2e.rs", include_str!("receipt_e2e.rs")),
    ("sources.rs", include_str!("sources.rs")),
];
