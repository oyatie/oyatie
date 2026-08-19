//! This crate's own sources, embedded so the engine-identity axis can hash them.
//!
//! GENERATED; regenerate rather than edit. Only files this crate OWNS are listed — the facade
//! assembles the whole-engine digest, because the facade is the one place the whole engine is
//! legitimately visible without inverting the dependency direction.

/// Crate-relative path and contents of each source this crate owns, sorted by path.
pub const CRATE_SOURCES: &[(&str, &str)] = &[
    ("lib.rs", include_str!("lib.rs")),
    ("sources.rs", include_str!("sources.rs")),
];
