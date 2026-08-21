//! The unit's sentinel failures as ONE type.
//!
//! Split from `items_static.rs` because the shape is different in kind: everything there builds one
//! item from one declaration, and this builds one item from the whole list. It has to — the variants
//! and the `Display` arms are two views of the same list, and building them per declaration is how a
//! message drifts from the sentinel it belongs to.

use port_engine_rust_ir::{RustItem, SentinelVariant, Visibility};

use crate::resolve::Resolver;

/// The unit's sentinels as ONE enum, in the order the source declares them.
///
/// Built from the whole list at once because the variants and the `Display` arms are two views of
/// it, and building them separately is how a message drifts from the sentinel it belongs to.
pub(crate) fn grouped_sentinels(resolver: &Resolver<'_>) -> RustItem {
    let name = resolver
        .sentinel_enum_name()
        .unwrap_or_else(|| unreachable!("the caller proved the unit groups its sentinels"))
        .to_owned();
    let variants = resolver
        .scope
        .sentinel_order
        .iter()
        .filter_map(|(source, block)| {
            // EMITTED only. A sentinel that refused — because its own prose named something the
            // crate does not contain, say — is not a failure this unit can produce, and a variant
            // for it would be a case no return ever constructs.
            if !resolver.emitted.contains(source) {
                return None;
            }
            let message = resolver.scope.sentinels.get(source)?;
            Some(SentinelVariant {
                docs: crate::docs::docs_from_block(block, source, resolver),
                name: resolver.sentinel_type_name(source),
                // The message names TYPES sometimes, and a source type name in it is a name the
                // emitted crate does not have. See `docs::rename_types_in_text` for why this one
                // rewrite reaches text the program emits, when no other does.
                message: crate::docs::rename_types_in_text(message, resolver.prose_type_names),
                // The CONSTANTS the message interpolates, as the target names them.
                arguments: resolver
                    .scope
                    .sentinel_arguments
                    .get(source)
                    .map(|names| {
                        names
                            .iter()
                            .map(|name| {
                                port_engine_rust_ir::RustExpr::Path(
                                    crate::naming::to_screaming_snake(name),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
            })
        })
        .collect();
    RustItem::SentinelEnum {
        // The type documents itself: it is this unit's failures, and each variant carries the
        // source's own words for the one it is.
        docs: Vec::new(),
        vis: Visibility::Public,
        name,
        exhaustive: resolver
            .failure
            .is_some_and(|convention| convention.sentinel_enum_exhaustive),
        variants,
    }
}
