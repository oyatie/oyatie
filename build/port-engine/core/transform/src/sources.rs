//! This crate's own sources, embedded so the engine-identity axis can hash them.
//!
//! GENERATED; regenerate rather than edit. Only files this crate OWNS are listed — the facade
//! assembles the whole-engine digest, because the facade is the one place the whole engine is
//! legitimately visible without inverting the dependency direction.

/// Crate-relative path and contents of each source this crate owns, sorted by path.
pub const CRATE_SOURCES: &[(&str, &str)] = &[
    ("apply.rs", include_str!("apply.rs")),
    ("body.rs", include_str!("body.rs")),
    ("body_call.rs", include_str!("body_call.rs")),
    ("body_cond.rs", include_str!("body_cond.rs")),
    ("body_expr.rs", include_str!("body_expr.rs")),
    ("body_failure.rs", include_str!("body_failure.rs")),
    ("body_idiom.rs", include_str!("body_idiom.rs")),
    ("body_index.rs", include_str!("body_index.rs")),
    ("body_literal.rs", include_str!("body_literal.rs")),
    ("body_loops.rs", include_str!("body_loops.rs")),
    ("body_ops.rs", include_str!("body_ops.rs")),
    ("body_parts.rs", include_str!("body_parts.rs")),
    ("counters.rs", include_str!("counters.rs")),
    ("docs.rs", include_str!("docs.rs")),
    ("error.rs", include_str!("error.rs")),
    ("failure.rs", include_str!("failure.rs")),
    ("impls.rs", include_str!("impls.rs")),
    ("items.rs", include_str!("items.rs")),
    ("items_self.rs", include_str!("items_self.rs")),
    ("items_static.rs", include_str!("items_static.rs")),
    ("lib.rs", include_str!("lib.rs")),
    ("naming.rs", include_str!("naming.rs")),
    ("ownership.rs", include_str!("ownership.rs")),
    ("params.rs", include_str!("params.rs")),
    ("promote.rs", include_str!("promote.rs")),
    ("resolve.rs", include_str!("resolve.rs")),
    ("resolve_policy.rs", include_str!("resolve_policy.rs")),
    ("resolve_tables.rs", include_str!("resolve_tables.rs")),
    ("resolve_types.rs", include_str!("resolve_types.rs")),
    ("returns.rs", include_str!("returns.rs")),
    ("sentinel.rs", include_str!("sentinel.rs")),
    ("signature.rs", include_str!("signature.rs")),
    ("signature_table.rs", include_str!("signature_table.rs")),
    ("sources.rs", include_str!("sources.rs")),
    ("survey.rs", include_str!("survey.rs")),
    ("vocabulary.rs", include_str!("vocabulary.rs")),
];
