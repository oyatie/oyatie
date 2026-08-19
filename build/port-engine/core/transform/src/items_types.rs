//! What a source TYPE declaration becomes.
//!
//! Split from `items.rs` because a type and a function are answered from different data: a type's
//! shape comes from its fields and the derives they earn, and a function's from its signature and
//! its body. That file keeps the dispatch and the callable builders.

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustField, RustItem, RustType, StructShape};

use crate::docs::docs_of;
use crate::error::TransformError;
use crate::naming::{to_pascal_case, to_snake_case, visibility};
use crate::resolve::Resolver;
use crate::signature::{Body, inherent_methods, trait_methods};
use crate::vocabulary::{CHILD_EMBEDS, CHILD_FIELD, POSITION_FIELD, POSITION_SUPERTRAIT};

pub(crate) fn build_type_alias(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<RustItem, TransformError> {
    Ok(RustItem::TypeAlias {
        docs: docs_of(declaration, resolver),
        vis: visibility(declaration),
        name: to_pascal_case(&declaration.name),
        ty: resolver.resolve(&declaration.type_ref, &declaration.name)?,
    })
}

/// A defined type over an underlying type becomes a newtype, never an alias.
///
/// The distinction is the whole point of the source construct: a defined type is a DISTINCT type
/// that does not interchange with its underlying one, and rendering it as an alias would erase
/// exactly the property it was declared for. A newtype keeps the distinction in the target's own
/// type system.
pub(crate) fn build_newtype(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<RustItem, TransformError> {
    let vis = visibility(declaration);
    Ok(RustItem::Struct {
        docs: docs_of(declaration, resolver),
        vis,
        name: to_pascal_case(&declaration.name),
        shape: StructShape::Tuple(vec![RustField {
            docs: Vec::new(),
            vis,
            name: String::new(),
            ty: resolver.resolve(&declaration.type_ref, &declaration.name)?,
        }]),
        // A newtype's one field is the source type itself, so the same rule answers for it.
        derives: resolver.derives_for(std::slice::from_ref(&declaration.type_ref)),
        methods: inherent_methods(declaration, resolver, Body::Stub)?,
    })
}

pub(crate) fn build_struct(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
    body: Body,
) -> Result<RustItem, TransformError> {
    let mut fields = Vec::new();
    // The SOURCE types, kept alongside the resolved ones: a derive is decided by what the source
    // guarantees about a field, and the target spelling has already lost that.
    let mut field_types = Vec::new();
    for field in declaration.children_of_kind(CHILD_FIELD) {
        field_types.push(field.type_ref.clone());
        fields.push(RustField {
            docs: docs_of(field, resolver),
            vis: visibility(field),
            name: to_snake_case(&field.name),
            ty: resolver.resolve_in(&field.type_ref, &field.name, POSITION_FIELD)?,
        });
    }

    Ok(RustItem::Struct {
        docs: docs_of(declaration, resolver),
        vis: visibility(declaration),
        name: to_pascal_case(&declaration.name),
        shape: if fields.is_empty() {
            StructShape::Unit
        } else {
            StructShape::Named(fields)
        },
        derives: resolver.derives_for(&field_types),
        methods: inherent_methods(declaration, resolver, body)?,
    })
}

pub(crate) fn build_trait(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<RustItem, TransformError> {
    Ok(RustItem::Trait {
        docs: docs_of(declaration, resolver),
        vis: visibility(declaration),
        name: to_pascal_case(&declaration.name),
        supertraits: supertraits(declaration, resolver)?,
        methods: trait_methods(declaration, resolver)?,
    })
}

/// The blanket impl a pure SUPERTRAIT BUNDLE earns, if this interface is one.
///
/// A source interface that embeds others and declares no method of its own is satisfied
/// STRUCTURALLY: a type with both method sets has it, and there is nothing to declare. The target
/// is nominal, so saying the same thing takes `impl<T: A + B> Job for T {}`.
///
/// That is not merely tidier than one empty impl per observed type — it is what the source MEANS.
/// The per-type form gives the trait only to types the engine saw asserted, and the source gives it
/// to every type that qualifies. A caller writing a generic function over `Job` would find their
/// own type rejected under the per-type form and accepted under this one, which is the difference
/// between a translation and an approximation.
///
/// `None` for an interface that declares any method of its own: a blanket impl would have to supply
/// bodies for them, and there are none to supply.
pub(crate) fn blanket_impl(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<Option<RustItem>, TransformError> {
    let bounds = supertraits(declaration, resolver)?;
    if bounds.is_empty() || !trait_methods(declaration, resolver)?.is_empty() {
        return Ok(None);
    }
    Ok(Some(RustItem::BlanketImpl {
        name: to_pascal_case(&declaration.name),
        bounds,
    }))
}

/// The traits an interface's embedded interfaces require.
///
/// A supertrait is a REQUIREMENT: an implementor must implement these too. Flattening them into the
/// outer trait's method list would compile and would mean something weaker — a type could satisfy
/// the outer trait without satisfying the embedded ones, which the source does not allow.
pub(crate) fn supertraits(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<Vec<RustType>, TransformError> {
    declaration
        .children_of_kind(CHILD_EMBEDS)
        .into_iter()
        .map(|embed| resolver.resolve_in(&embed.type_ref, &declaration.name, POSITION_SUPERTRAIT))
        .collect()
}
