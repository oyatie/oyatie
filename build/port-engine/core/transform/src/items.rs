//! The declaration-level construction builders: what each construction emits, as IR items.

use port_engine_api::Declaration;
use port_engine_rust_ir::{
    RustExpr, RustField, RustFn, RustItem, RustStmt, RustType, StructShape, Visibility,
};

use crate::error::TransformError;
use crate::impls::trait_impls;
use crate::naming::{to_pascal_case, to_screaming_snake, to_snake_case, visibility};
use crate::params::params;
use crate::results::results;
use crate::items_types::{
    blanket_impl, build_newtype, build_struct, build_trait, build_type_alias,
};
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
        CONSTRUCTION_RUST_NEWTYPE => build_newtype(declaration, resolver, Body::Translate),
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
    // A BINARY string is not the target's string. The source's string holds arbitrary bytes and the
    // target's is guaranteed UTF-8, so the ordinary mapping is right only for the ones that are
    // text — and a framing prefix typed as text is one escape away from a wire-format break.
    if let Some((ty, value)) = crate::items_value::binary_string(declaration, resolver) {
        return Ok(RustItem::Const {
            docs: docs_of(declaration, resolver)?,
            vis: visibility(declaration),
            name: to_screaming_snake(&declaration.name),
            ty,
            value,
        });
    }
    // A LENGTH constant is the target's index type. The source types it as its own integer, and
    // every guard then casts one side or the other — a cast per call site, each one a chance to get
    // the direction wrong, and public so the casts leak to every caller.
    let ty = match resolver.scope.length_constants.contains(&declaration.name) {
        true => RustType::path("usize"),
        false => resolver.resolve(&declaration.type_ref, &declaration.name)?,
    };
    // The author's DERIVATION where the target can spell it. Carried as an EXPRESSION rather than
    // as text: the derivation is a tree, and the one item shape that holds a constant's value as a
    // tree already exists for a package value.
    if let Some(value) = crate::items_value::authored_value(declaration, resolver, &ty) {
        return Ok(RustItem::PackageValue {
            docs: docs_of(declaration, resolver)?,
            vis: visibility(declaration),
            name: to_screaming_snake(&declaration.name),
            ty,
            value,
        });
    }
    Ok(RustItem::Const {
        docs: docs_of(declaration, resolver)?,
        vis: visibility(declaration),
        name: to_screaming_snake(&declaration.name),
        // A constant AT a defined type is CONSTRUCTED at it, not assigned to it. The source's
        // untyped literal takes whatever type the declaration names, so `const Person Domain = 0`
        // needs no conversion there; the target's newtype is a distinct type and `Domain = 0` does
        // not typecheck. This is the same operation a conversion to a defined type performs, and it
        // was missing here only because a constant reaches its type by declaration rather than by
        // call — nine of uuid's constants came out ill-typed for exactly that reason.
        // A BIT PATTERN is spelled as one. The source's base is not recorded and would not bind
        // the target anyway; what the target owes a reader is the spelling in which the value can
        // be checked against whatever defines it.
        value: match crate::items_value::constructs_at_type(declaration, resolver) {
            true => format!("{}({})", ty.spelling(), value),
            false => crate::items_value::bit_pattern(declaration, resolver)
                .or_else(|| crate::items_value::readable_literal(value, resolver))
                .map(|spelled| pointed_for(spelled, &ty))
                .unwrap_or_else(|| pointed_for(value.to_owned(), &ty)),
        },
        ty,
    })
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

    // The BODY first, and what it FOLDED with it: the signature below is built from what the fold
    // did rather than from a prediction of it, because a parameter it substituted away needs no
    // `mut` and one it could not needs its own.
    // A SHORTHAND emits nothing of its own. Every call to it became the call it wraps, so the
    // declaration has no callers left — and eight one-line wrappers around an intrinsic were, in
    // three reviewers' words, the clearest sign the source's structure had been carried rather than
    // its meaning. It still TRANSLATED: what it becomes is at each of its call sites.
    if resolver
        .signatures
        .eta(&format!("{}.{}", resolver.unit.0, declaration.name))
        .is_some()
    {
        return Ok(RustItem::Nothing);
    }
    let consumed;
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
        let (statements, folded) =
            body::statements(&source.children, declaration, resolver, body::ResultShape::Own, None)?;
        consumed = folded;
        statements
    } else {
        // A rung that does not translate bodies REFUSES the function it cannot write, for the same
        // reason a method does: a body that panics compiles, reads as success to every gate, and
        // aborts at the caller where the source computed something.
        return Err(TransformError::Unsupported {
            name: declaration.name.clone(),
            detail: format!(
                "`{}` is captured by a rule that does not translate function bodies, and emitting \
                 it without one would compile and panic where the source computed something",
                declaration.name
            ),
        });
    };

    let rendered = RustFn {
        docs: docs_of(declaration, resolver)?,
        vis: visibility(declaration),
        name: to_snake_case(&declaration.name),
        receiver: None,
        params: params(declaration, resolver, &declaration.name, &consumed)?,
        ret: results(declaration, resolver)?,
        attrs: crate::params::inline_attrs(Some(&body), visibility(declaration), resolver),
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
        // A canary carries none: it is the engine's own marker, not a ported declaration.
        attrs: Vec::new(),
        body: Some(Vec::new()),
    }))
}

/// A whole-number value spelled for a FLOATING-POINT constant.
///
/// The source's untyped constant carries no point and takes one from wherever it is used;
/// `const pushPullScaleThreshold = 32` is a float because every use of it is. The target's constant
/// states its type at the declaration, and `const T: f64 = 32` does not typecheck — a literal with
/// no point is an integer there.
///
/// Only a value that is entirely DIGITS. Anything else already carries a point, an exponent or a
/// sign it spelled for itself, and appending to it would produce something that is not a number.
fn pointed_for(spelled: String, ty: &RustType) -> String {
    let float = matches!(ty.spelling().as_str(), "f32" | "f64");
    let whole = !spelled.is_empty()
        && spelled
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'_');
    match float && whole {
        true => format!("{spelled}.0"),
        false => spelled,
    }
}
