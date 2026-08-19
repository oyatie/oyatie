//! A package variable nothing writes is a `static`, when its value is a constant expression.
//!
//! The pack defers `var` because the source's package variable is a MUTABLE global and the target
//! has no immediate counterpart: `static` is immutable, `static mut` is unsafe, and `OnceLock` and
//! `Mutex` each pick a synchronization policy the source never stated. That argument is sound, and
//! it bites only for a variable something actually assigns to — across the surveyed corpora most
//! package variables are never written anywhere in their own package.
//!
//! For the unwritten ones the form is decided here, and `static` rather than `const` is the whole
//! of that decision. A source package variable has ONE storage location and one address for the
//! life of the program; taking its address gives the same pointer every time. A target `const` is
//! materialised afresh at every use and has no stable address, so `&X` would differ per use — an
//! observable difference for a variable whose address the source can take. A target `static` has
//! exactly the source variable's storage identity, and being immutable it raises no
//! synchronization question at all: there is nothing to synchronize when nothing writes.
//!
//! The price is that a `static`'s initialiser must be a CONSTANT EXPRESSION. That is not a
//! heuristic standing in for something else — it is the target's own rule, and an initialiser that
//! fails it fails to compile rather than meaning something different. So the test is exact, the
//! shapes that pass are closed, and everything else refuses by name.

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustExpr, RustItem, RustType};

use crate::body::Body;
use crate::body_expr::expression;
use crate::docs::docs_of;
use crate::error::TransformError;
use crate::naming::{to_screaming_snake, visibility};
use crate::resolve::Resolver;
use crate::vocabulary::{
    ATTR_REF, CONSTRUCTION_RUST_STATIC, FLAG_REBOUND, FORM_WRITTEN_PACKAGE_VAR, KIND_COMPOSITE,
    KIND_IDENT, KIND_KEYED, KIND_LITERAL, KIND_ZERO, REF_CONST, SOURCE_STRING, TARGET_STR,
};

/// `static NAME: T = value;` for a package variable nothing writes.
///
/// # Errors
/// [`TransformError::UndecidedForm`] when something writes the variable — the synchronization
/// policy is a decision nobody has made, and the pack's own words say so.
/// [`TransformError::NotConstantExpression`] when the initialiser is not one the target can
/// evaluate at compile time.
pub(crate) fn build_static(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<RustItem, TransformError> {
    // WRITTEN is the case the deferral's argument is actually about, and the reason comes from the
    // pack so the refusal a reader sees and the reason the digest carries are one text.
    if declaration.flags.iter().any(|flag| flag == FLAG_REBOUND) {
        return Err(TransformError::UndecidedForm {
            form: FORM_WRITTEN_PACKAGE_VAR.to_owned(),
            name: declaration.name.clone(),
            reason: resolver
                .undecided_forms
                .get(FORM_WRITTEN_PACKAGE_VAR)
                .cloned()
                .unwrap_or_default(),
        });
    }
    // A SENTINEL is its message. The initialiser is a call, which is not a constant expression, so
    // this arm exists before the constant test rather than inside it — what makes the value usable
    // is not that the call can be evaluated early but that the call is unnecessary until a return
    // needs one. See `sentinel.rs` for what this costs.
    if let Some(message) = resolver.scope.sentinels.get(&declaration.name) {
        return Ok(RustItem::PackageValue {
            docs: docs_of(declaration, resolver),
            vis: visibility(declaration),
            name: to_screaming_snake(&declaration.name),
            ty: RustType::Reference {
                mutable: false,
                inner: Box::new(RustType::Path(TARGET_STR.to_owned())),
            },
            value: RustExpr::Literal(message.clone()),
        });
    }
    let ty = static_type(declaration, resolver)?;
    let value = match declaration.children.first() {
        // No initialiser at all: the source guarantees the zero value, and there is no work whose
        // timing could be in question, because there is no work.
        None => resolver
            .zero_value(&declaration.type_ref)
            .map(RustExpr::Literal)
            .ok_or_else(|| TransformError::NotConstantExpression {
                name: declaration.name.clone(),
                detail: format!(
                    "the variable has no initialiser, so its value is the zero of `{}` — for which \
                     the pack declares no target form",
                    declaration.type_ref.kind
                ),
            })?,
        Some(initialiser) => {
            prove_constant(initialiser, declaration, resolver)?;
            // Translated by the SAME translator a body uses, so a value in a static and the same
            // value in a function cannot come out differently. What is decided here is only
            // whether the target can evaluate it before the program runs.
            let body = Body {
                owner: &declaration.name,
                resolver,
                fallible: false,
                borrowed: std::collections::BTreeSet::new(),
                result_is_owned_string: false,
                results: crate::returns::ResultFacts::none(),
                usize_counters: std::collections::BTreeSet::new(),
                walked: None,
            };
            expression(initialiser, &body)?
        }
    };
    Ok(RustItem::PackageValue {
        docs: docs_of(declaration, resolver),
        vis: visibility(declaration),
        name: to_screaming_snake(&declaration.name),
        ty,
        value,
    })
}

/// The type the static declares, which is the type its constant expression actually has.
///
/// One case differs from resolving the declaration's type on its own, and the target forces it: a
/// STRING LITERAL is a borrow. `"id-"` has type `&'static str`, and the owned `String` cannot be
/// built by a constant expression at all — so the owned form here is not merely less idiomatic,
/// it does not exist. It is also what every reader wants: this rule applies only to a variable
/// nothing writes, so the value is shared read-only data and a borrow is exactly that.
///
/// Nothing else moves. A map or slice initialised to its ZERO takes the owned form, because the
/// pack's zero for those is a `const fn` that builds an empty owned container — and an owned empty
/// container is what the source's zero map and zero slice are.
///
/// # Errors
/// [`TransformError`] when the declaration's own type does not resolve.
fn static_type(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<RustType, TransformError> {
    let literal_string = declaration.type_ref.name == SOURCE_STRING
        && declaration
            .children
            .first()
            .is_some_and(|child| child.kind == KIND_LITERAL);
    match literal_string {
        true => Ok(RustType::Reference {
            mutable: false,
            inner: Box::new(RustType::Path(TARGET_STR.to_owned())),
        }),
        false => resolver.resolve(&declaration.type_ref, &declaration.name),
    }
}

/// Whether the target can evaluate this initialiser before the program runs, or a refusal.
///
/// A SEPARATE question from what the initialiser translates to, and separate on purpose: one
/// translator produces the expression, and this decides only whether it may stand in a `static`.
///
/// CLOSED, and each shape admitted because the target's own rule admits it:
///
/// - a LITERAL of a basic type is a constant expression by construction;
/// - an IDENT referring to a CONSTANT is one, because the target's counterpart is a `const` — a
///   name this unit declares resolves to that declaration, and a PREDECLARED one through the
///   pack's constant map. An identifier referring to anything else refuses, because nothing here
///   says what it becomes;
/// - a COMPOSITE literal is one when every element is, because a struct, tuple and array
///   constructor are all const in the target.
///
/// Everything else refuses. A CALL is the common one and worth stating: `errors.New("..")` and
/// `regexp.MustCompile("..")` allocate, so neither is a constant expression, and reaching for
/// `LazyLock` instead would run the initialiser on FIRST USE rather than before it — the same
/// when-does-the-work-happen question that defers `package_init`, and not one this rule may answer.
fn prove_constant(
    node: &Declaration,
    owner: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<(), TransformError> {
    let refuse = |detail: String| TransformError::NotConstantExpression {
        name: owner.name.clone(),
        detail,
    };
    match node.kind.as_str() {
        // A ZERO is what the source supplies for an omitted field, and the pack's form for it is
        // a literal or a `const fn` in every case the table declares — a value, never work.
        KIND_LITERAL | KIND_ZERO => Ok(()),
        KIND_IDENT if names_a_constant(node, resolver) => Ok(()),
        KIND_IDENT => Err(refuse(format!(
            "the initialiser reads `{}`, and a static's initialiser must be a constant \
             expression: this unit declares no constant of that name, and the pack's constant map \
             does not list it either",
            node.name
        ))),
        KIND_COMPOSITE | KIND_KEYED => {
            for child in &node.children {
                prove_constant(child, owner, resolver)?;
            }
            Ok(())
        }
        other => Err(refuse(format!(
            "the initialiser is a `{other}`, which the target cannot evaluate before the program \
             runs — a static's initialiser must be a constant expression, and running it lazily \
             instead would move the work from before first use to at it"
        ))),
    }
}

/// Whether this identifier names a constant the emitted crate will have.
///
/// Two homes, and neither overlaps the other: a constant this unit DECLARES is emitted by this
/// crate under its own cased name, and a PREDECLARED one belongs to the source language and is
/// spelled by the pack. A name in neither is not this rule's to invent.
fn names_a_constant(node: &Declaration, resolver: &Resolver<'_>) -> bool {
    node.attr(ATTR_REF) == Some(REF_CONST)
        && (resolver.scope.contains(&node.name) || resolver.constant_map.contains_key(&node.name))
}

/// The construction this module answers for.
pub(crate) const CONSTRUCTION: &str = CONSTRUCTION_RUST_STATIC;
