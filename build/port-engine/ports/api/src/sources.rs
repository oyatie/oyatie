//! This crate's own sources, embedded so the engine-identity axis can hash them.
//!
//! GENERATED; regenerate rather than edit. Only files this crate OWNS are listed — the facade
//! assembles the whole-engine digest, because the facade is the one place the whole engine is
//! legitimately visible without inverting the dependency direction.

/// Crate-relative path and contents of each source this crate owns, sorted by path.
pub const CRATE_SOURCES: &[(&str, &str)] = &[
    ("declaration.rs", include_str!("declaration.rs")),
    ("error.rs", include_str!("error.rs")),
    ("failure.rs", include_str!("failure.rs")),
    ("identity.rs", include_str!("identity.rs")),
    ("lib.rs", include_str!("lib.rs")),
    ("ownership.rs", include_str!("ownership.rs")),
    ("plan.rs", include_str!("plan.rs")),
    ("receipt.rs", include_str!("receipt.rs")),
    ("seams.rs", include_str!("seams.rs")),
    ("sources.rs", include_str!("sources.rs")),
    ("type_ref.rs", include_str!("type_ref.rs")),
];
