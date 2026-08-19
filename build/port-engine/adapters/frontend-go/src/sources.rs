//! This crate's own sources, embedded so the engine-identity axis can hash them.
//!
//! GENERATED; regenerate rather than edit. Only files this crate OWNS are listed — the facade
//! assembles the whole-engine digest, because the facade is the one place the whole engine is
//! legitimately visible without inverting the dependency direction.

/// Crate-relative path and contents of each source this crate owns, sorted by path.
pub const CRATE_SOURCES: &[(&str, &str)] = &[
    ("convert.rs", include_str!("convert.rs")),
    ("error.rs", include_str!("error.rs")),
    ("lib.rs", include_str!("lib.rs")),
    ("model.rs", include_str!("model.rs")),
    ("sources.rs", include_str!("sources.rs")),
    ("vocabulary.rs", include_str!("vocabulary.rs")),
    ("wire.rs", include_str!("wire.rs")),
];
