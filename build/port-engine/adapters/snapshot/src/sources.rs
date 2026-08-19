//! This crate's own sources, embedded so the engine-identity axis can hash them.
//!
//! GENERATED; regenerate rather than edit. Only files this crate OWNS are listed — the facade
//! assembles the whole-engine digest, because the facade is the one place the whole engine is
//! legitimately visible without inverting the dependency direction.

/// Crate-relative path and contents of each source this crate owns, sorted by path.
pub const CRATE_SOURCES: &[(&str, &str)] = &[
    ("admit.rs", include_str!("admit.rs")),
    ("admitted.rs", include_str!("admitted.rs")),
    ("error.rs", include_str!("error.rs")),
    ("lib.rs", include_str!("lib.rs")),
    ("preimage.rs", include_str!("preimage.rs")),
    ("sources.rs", include_str!("sources.rs")),
];
