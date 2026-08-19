//! This crate's own sources, embedded so the engine-identity axis can hash them.
//!
//! GENERATED; regenerate rather than edit. Only files this crate OWNS are listed — the facade
//! assembles the whole-engine digest, because the facade is the one place the whole engine is
//! legitimately visible without inverting the dependency direction.

/// Crate-relative path and contents of each source this crate owns, sorted by path.
pub const CRATE_SOURCES: &[(&str, &str)] = &[
    ("expr.rs", include_str!("expr.rs")),
    ("item.rs", include_str!("item.rs")),
    ("item_parts.rs", include_str!("item_parts.rs")),
    ("item_types.rs", include_str!("item_types.rs")),
    ("lib.rs", include_str!("lib.rs")),
    ("lower.rs", include_str!("lower.rs")),
    ("lower_body.rs", include_str!("lower_body.rs")),
    ("lower_expr.rs", include_str!("lower_expr.rs")),
    ("lower_parts.rs", include_str!("lower_parts.rs")),
    ("lower_precedence.rs", include_str!("lower_precedence.rs")),
    ("lower_sentinel.rs", include_str!("lower_sentinel.rs")),
    ("ops.rs", include_str!("ops.rs")),
    ("render.rs", include_str!("render.rs")),
    ("sources.rs", include_str!("sources.rs")),
    ("stmt.rs", include_str!("stmt.rs")),
    ("ty.rs", include_str!("ty.rs")),
];
