//! This crate's own sources, embedded so the engine-identity axis can hash them.
//!
//! GENERATED; regenerate rather than edit. Only files this crate OWNS are listed — the facade
//! assembles the whole-engine digest, because the facade is the one place the whole engine is
//! legitimately visible without inverting the dependency direction.

/// Crate-relative path and contents of each source this crate owns, sorted by path.
pub const CRATE_SOURCES: &[(&str, &str)] = &[
    ("expr.rs", include_str!("expr.rs")),
    ("item.rs", include_str!("item.rs")),
    ("lib.rs", include_str!("lib.rs")),
    ("lower.rs", include_str!("lower.rs")),
    ("lower_body.rs", include_str!("lower_body.rs")),
    ("lower_parts.rs", include_str!("lower_parts.rs")),
    ("ops.rs", include_str!("ops.rs")),
    ("render.rs", include_str!("render.rs")),
    ("sources.rs", include_str!("sources.rs")),
    ("ty.rs", include_str!("ty.rs")),
];
