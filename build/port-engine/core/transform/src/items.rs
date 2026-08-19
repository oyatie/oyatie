//! The declaration-level construction builders: what each construction emits, as IR items.

use port_engine_api::Declaration;
use port_engine_rust_ir::{
    RustExpr, RustField, RustFn, RustItem, RustStmt, RustType, StructShape, Visibility,
};

use crate::error::TransformError;
use crate::impls::trait_impls;
use crate::naming::{to_pascal_case, to_screaming_snake, to_snake_case, visibility};
use crate::params::{params, results};
use crate::resolve::Resolver;
use crate::items_self::rename_own_type;
use crate::signature::{Body, inherent_methods, trait_methods};
use crate::vocabulary::{
    ATTR_VALUE, CHILD_BODY, CHILD_EMBEDS, CHILD_FIELD, CHILD_RESULT, CONSTRUCTION_RUST_CONST, CONSTRUCTION_RUST_FN, CONSTRUCTION_RUST_FN_BODY, CONSTRUCTION_RUST_NEWTYPE, CONSTRUCTION_RUST_STATIC, CONSTRUCTION_RUST_STRUCT, CONSTRUCTION_RUST_STRUCT_BODY, CONSTRUCTION_RUST_TRAIT, CONSTRUCTION_RUST_TYPE_ALIAS, CONSTRUCTOR_PREFIX, IDIOM_SELF_IN_IMPL, POSITION_FIELD, POSITION_SUPERTRAIT, TYPE_POINTER,
};
use crate::{body, docs::docs_of};

/// What one construction emits.
///
/// A LIST, because a declaration is not always one item: a type that satisfies an interface emits
/// the type and an `impl` per satisfaction. Folding those into the type's own construction would
/// make "which interfaces does this type satisfy" a question the struct builder answers, and a
/// type could not gain an impl without its construction changing.
pub(crate) fn build_item(
    construction: &str,
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<Vec<RustItem>, TransformError> {
    let item = match construction {
        CONSTRUCTION_RUST_CONST => build_const(declaration, resolver),
        CONSTRUCTION_RUST_STATIC => crate::items_static::build_static(declaration, resolver),
        CONSTRUCTION_RUST_TYPE_ALIAS => build_type_alias(declaration, resolver),
        CONSTRUCTION_RUST_NEWTYPE => build_newtype(declaration, resolver),
        CONSTRUCTION_RUST_STRUCT => build_struct(declaration, resolver, Body::Stub),
        CONSTRUCTION_RUST_STRUCT_BODY => build_struct(declaration, resolver, Body::Translate),
        CONSTRUCTION_RUST_TRAIT => build_trait(declaration, resolver),
        CONSTRUCTION_RUST_FN => build_fn(declaration, resolver, false),
        CONSTRUCTION_RUST_FN_BODY => build_fn(declaration, resolver, true),
        other => Err(TransformError::UnknownConstruction {
            rule: String::new(),
            construction: other.to_owned(),
        }),
    }?;

    let mut items = vec![item];
    if construction == CONSTRUCTION_RUST_TRAIT
        && let Some(blanket) = blanket_impl(declaration, resolver)?
    {
        items.push(blanket);
    }
    items.extend(trait_impls(declaration, resolver)?);
    Ok(items)
}

fn build_const(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<RustItem, TransformError> {
    let value = declaration
        .attr(ATTR_VALUE)
        .ok_or_else(|| TransformError::MissingDatum {
            construction: CONSTRUCTION_RUST_CONST.to_owned(),
            name: declaration.name.clone(),
            datum: ATTR_VALUE,
        })?;
    Ok(RustItem::Const {
        docs: docs_of(declaration, resolver.doc_convention),
        vis: visibility(declaration),
        name: to_screaming_snake(&declaration.name),
        ty: resolver.resolve(&declaration.type_ref, &declaration.name)?,
        value: value.to_owned(),
    })
}

fn build_type_alias(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<RustItem, TransformError> {
    Ok(RustItem::TypeAlias {
        docs: docs_of(declaration, resolver.doc_convention),
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
fn build_newtype(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<RustItem, TransformError> {
    let vis = visibility(declaration);
    Ok(RustItem::Struct {
        docs: docs_of(declaration, resolver.doc_convention),
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

fn build_struct(
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
            docs: docs_of(field, resolver.doc_convention),
            vis: visibility(field),
            name: to_snake_case(&field.name),
            ty: resolver.resolve_in(&field.type_ref, &field.name, POSITION_FIELD)?,
        });
    }

    Ok(RustItem::Struct {
        docs: docs_of(declaration, resolver.doc_convention),
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

fn build_trait(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<RustItem, TransformError> {
    Ok(RustItem::Trait {
        docs: docs_of(declaration, resolver.doc_convention),
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
fn blanket_impl(
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
fn supertraits(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<Vec<RustType>, TransformError> {
    declaration
        .children_of_kind(CHILD_EMBEDS)
        .into_iter()
        .map(|embed| resolver.resolve_in(&embed.type_ref, &declaration.name, POSITION_SUPERTRAIT))
        .collect()
}

fn build_fn(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
    translate_body: bool,
) -> Result<RustItem, TransformError> {
    if !declaration.children_of_kind(CHILD_FIELD).is_empty() {
        return Err(TransformError::ConstructionKindMismatch {
            construction: CONSTRUCTION_RUST_FN.to_owned(),
            kind: declaration.kind.clone(),
            name: declaration.name.clone(),
        });
    }

    let body = if translate_body {
        let source = declaration
            .children_of_kind(CHILD_BODY)
            .first()
            .copied()
            .ok_or_else(|| TransformError::MissingDatum {
                construction: CONSTRUCTION_RUST_FN_BODY.to_owned(),
                name: declaration.name.clone(),
                datum: "body",
            })?;
        body::statements(&source.children, declaration, resolver, body::ResultShape::Own)?
    } else {
        vec![RustStmt::Tail(RustExpr::Todo)]
    };

    let rendered = RustFn {
        docs: docs_of(declaration, resolver.doc_convention),
        vis: visibility(declaration),
        name: to_snake_case(&declaration.name),
        receiver: None,
        params: params(declaration, resolver, &declaration.name)?,
        ret: results(declaration, resolver)?,
        body: Some(body),
    };

    // A package-level CONSTRUCTOR belongs on the type, not beside it.
    match constructed_type(declaration, resolver) {
        Some(self_ty) => {
            // Inside the type's own impl block the target spells that type `Self`, which is not
            // merely shorter: it survives a rename, where the name written twice has two places to
            // miss. The source has no such spelling and always writes the name.
            let inside = match resolver.idiom_method(IDIOM_SELF_IN_IMPL) {
                Some(spelling) => rename_own_type(rendered, &self_ty, spelling),
                None => rendered,
            };
            Ok(RustItem::InherentImpl {
                docs: Vec::new(),
                self_ty: RustType::path(self_ty),
                methods: vec![RustFn {
                    name: "new".to_owned(),
                    ..inside
                }],
            })
        }
        None => Ok(RustItem::Function(rendered)),
    }
}

/// The type a package-level CONSTRUCTOR constructs, if this declaration is one.
///
/// Recognised by SHAPE, not by name alone: the source's explicit constructor convention is a
/// package-level function named `New` or `New<Type>` whose sole result is a type that same package
/// declares. Both halves are required. A function merely named `NewFoo` that returns something
/// else is not a constructor, and one returning a local type without the prefix is a factory the
/// source did not mark as one — neither is moved onto a type.
///
/// The target puts a constructor on the type: `Label::new` rather than a free `new_label`. A
/// reviewer reading the emitted crate called the free form the single most visible sign that
/// another language's structure had been carried across rather than translated.
fn constructed_type(declaration: &Declaration, resolver: &Resolver<'_>) -> Option<String> {
    let suffix = declaration.name.strip_prefix(CONSTRUCTOR_PREFIX)?;
    let results = declaration.children_of_kind(CHILD_RESULT);
    let [result] = results.as_slice() else {
        return None;
    };
    // `func New(..) *T` is the SAME convention as `func New(..) T`, and it is the commoner of the
    // two: the source allocates away from the caller's frame and hands the pointer back. What the
    // constructor constructs is the pointer's target either way, and it is that type the impl block
    // stands on — so the pointer is looked through rather than treated as a different result.
    let constructed = match result.type_ref.kind.as_str() {
        TYPE_POINTER => result.type_ref.args.first()?,
        _ => &result.type_ref,
    };
    // The result must be a type this unit DECLARES. A constructor for someone else's type is not
    // this package's to put a method on — the target's coherence rule forbids it outright.
    if !resolver.declares(constructed) {
        return None;
    }
    let target = to_pascal_case(&constructed.name);
    match suffix.is_empty() || to_pascal_case(suffix) == target {
        true => Some(target),
        false => None,
    }
}

/// The unit-level constructions: one region per unit, no declarations read.
pub(crate) fn build_unit_item(construction: &str, region: &str) -> Option<RustItem> {
    let name = match construction {
        crate::vocabulary::CONSTRUCTION_PASS_THROUGH => region.to_owned(),
        crate::vocabulary::CONSTRUCTION_EMPTY_CANARY => format!("{region}_canary"),
        _ => return None,
    };
    Some(RustItem::Function(RustFn {
        docs: Vec::new(),
        vis: Visibility::Public,
        name,
        receiver: None,
        params: Vec::new(),
        ret: None,
        body: Some(Vec::new()),
    }))
}
