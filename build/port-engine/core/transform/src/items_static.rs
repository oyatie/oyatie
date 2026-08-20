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
use port_engine_rust_ir::{RustExpr, RustItem, RustType, SentinelVariant, Visibility};

use crate::body::Body;
use crate::body_expr::expression;
use crate::docs::docs_of;
use crate::error::TransformError;
use crate::naming::{to_pascal_case, to_screaming_snake, visibility};
use crate::resolve::Resolver;
use crate::vocabulary::{
    ATTR_REF, CONSTRUCTION_RUST_STATIC, FLAG_EXPORTED, FLAG_INIT_WRITTEN, FLAG_REBOUND, FORM_INIT_WRITTEN_PACKAGE_VAR,
    FORM_EXPORTED_PACKAGE_VAR, FORM_WRITTEN_PACKAGE_VAR, KIND_COMPOSITE, KIND_IDENT, KIND_KEYED, KIND_LITERAL, KIND_ZERO, REF_CONST, SOURCE_STRING, TARGET_STR,
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
        // WHERE the writes are decides WHICH question is open. A variable the package initialiser
        // alone writes has no synchronization question — it is computed once before anything runs —
        // and what it lacks is the initialising expression rather than a decision. Naming both with
        // one reason told a reader the engine was weighing a concurrency policy for a compiled
        // constant, which is not what is missing.
        let form = match declaration.flags.iter().any(|flag| flag == FLAG_INIT_WRITTEN) {
            true => FORM_INIT_WRITTEN_PACKAGE_VAR,
            false => FORM_WRITTEN_PACKAGE_VAR,
        };
        return Err(TransformError::UndecidedForm {
            form: form.to_owned(),
            name: declaration.name.clone(),
            reason: resolver
                .undecided_forms
                .get(form)
                .cloned()
                .unwrap_or_default(),
        });
    }
    // A SENTINEL is its message. The initialiser is a call, which is not a constant expression, so
    // this arm exists before the constant test rather than inside it — what makes the value usable
    // is not that the call can be evaluated early but that the call is unnecessary until a return
    // needs one. See `sentinel.rs` for what this costs.
    if let Some(message) = resolver.scope.sentinels.get(&declaration.name) {
        // ASKED FIRST, and for its refusal rather than its value. A declaration whose prose names
        // something the crate does not contain refuses whatever it emits — and a sentinel that
        // emits nothing of its own would otherwise skip the check entirely, carrying a dangling
        // reference into the grouped enum's variant where nothing was watching.
        let docs = docs_of(declaration, resolver)?;
        // GROUPED, and then the FIRST sentinel in source order carries the whole enum and every
        // other one emits nothing. Built once rather than once per sentinel, and here rather than
        // as a unit-level item because this is where a resolver for the unit exists — the variants
        // need the same name and doc rewriting every other declaration gets.
        //
        // The ones that emit nothing still TRANSLATED: what they become is in the crate, in the
        // region the first sentinel owns rather than the one they own.
        if resolver.sentinel_enum_name().is_some() {
            // The first EMITTED one, not the first declared. A sentinel that refused is not in the
            // crate, and building the enum on it would build nothing at all; carrying it as a
            // variant would put a failure in the type that no return can produce.
            let first = resolver
                .scope
                .sentinel_order
                .iter()
                .find(|(name, _)| resolver.emitted.contains(name))
                .is_some_and(|(name, _)| name == &declaration.name);
            return Ok(match first {
                true => crate::items_sentinels::grouped_sentinels(resolver),
                false => RustItem::Nothing,
            });
        }
        return Ok(RustItem::SentinelError {
            docs,
            vis: visibility(declaration),
            // A TYPE, so its name is a type's — and without the source's `Err` prefix, which is
            // a convention for a namespacing problem the target does not have. Decided by the
            // resolver so the return and the identity test spell the same name.
            name: resolver.sentinel_type_name(&declaration.name),
            // The message names TYPES sometimes, and a source type name in it is a name the
            // emitted crate does not have. See `docs::rename_types_in_text` for why this one
            // rewrite reaches text the program emits, when no other does.
            message: crate::docs::rename_types_in_text(message, resolver.prose_type_names),
        });
    }
    // A SENTINEL is exempt, and the exemption is the sentinel decision rather than a second one:
    // it becomes its MESSAGE, and the message is constant however the variable is reassigned.
    // Reassignment changes which failure value the NAME holds, which is identity — and identity is
    // what the sentinel decision already records as lost, with its cost written down.
    //
    // EXPORTED is written by anyone, and the engine cannot see them. "Never written" is a fact
    // about THIS package, and an exported package variable is part of the source's API: a consumer
    // writes `semver.CoerceNewVersion = false` and the package's own documentation says to. Making
    // one a constant deletes a feature and keeps the prose describing it, which a reviewer reading
    // a real ported package found and called a translation that preserved syntax and dropped
    // semantics. They were right, and the engine had no way to see it — so the rule now asks
    // whether anyone COULD write it rather than whether this package does.
    if declaration.flags.iter().any(|flag| flag == FLAG_EXPORTED) {
        return Err(TransformError::UndecidedForm {
            // ITS OWN FORM. The decision is the same shape as the written one and it is not the
            // same decision: nothing in this package assigns to `Nil`, and calling it
            // `written_package_var` told every reader of the refusal that something does.
            form: FORM_EXPORTED_PACKAGE_VAR.to_owned(),
            name: declaration.name.clone(),
            reason: format!(
                "`{}` is EXPORTED, so anything that imports this package may write it — which is \
                 the same mutable global the undecided form is about, arrived at from outside \
                 rather than from within. Nothing in this package writes it, and nothing here can \
                 observe the writes that could come from outside. {}",
                declaration.name,
                resolver
                    .undecided_forms
                    .get(FORM_WRITTEN_PACKAGE_VAR)
                    .cloned()
                    .unwrap_or_default()
            ),
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
                newtype_parameters: std::collections::BTreeSet::new(),
                owner: &declaration.name,
                resolver,
                fallible: false,
                borrowed: std::collections::BTreeSet::new(),
                result_is_owned_string: false,
            result_is_owned_sequence: std::collections::BTreeSet::new(),
            drops_absent_failure: false,
            named_results: Vec::new(),
                results: crate::returns::ResultFacts::none(),
                usize_counters: std::collections::BTreeSet::new(),
                walked: None,
                receiver_type: None,
            };
            // AN ARRAY where the declared type is one. `expression` builds the pack's growable
            // sequence form, because that is the right answer everywhere a body builds a value;
            // here the type says otherwise, and the two must agree or the constant does not
            // compile. Built from the SAME element translation rather than by re-rendering, so an
            // element in an array constant and the same element in a body cannot differ.
            // A sequence of BYTE LITERALS is a byte string, however the type is spelled. Same type,
            // same bytes, and the target has a form a reader takes in at a glance where sixteen
            // comma-separated byte literals have to be counted. `clippy::byte_char_slices` refuses
            // the long form under the deny-warnings policy this engine is held to.
            //
            // Keyed on the ELEMENTS rather than on the declared type, because the source spells a
            // fixed-size sequence two ways — `[16]byte` and `[...]byte{..}` — and they arrive here
            // as different type kinds while being the same thing to a reader.
            if let Some(bytes) = byte_string(initialiser, &body) {
                return Ok(RustItem::PackageValue {
                    docs: docs_of(declaration, resolver)?,
                    vis: visibility(declaration),
                    name: to_screaming_snake(&declaration.name),
                    ty,
                    value: bytes,
                });
            }
            match &ty {
                RustType::Array { .. } => RustExpr::ArrayLiteral(
                    initialiser
                        .children
                        .iter()
                        .map(|element| expression(element, &body))
                        .collect::<Result<Vec<_>, _>>()?,
                ),
                _ => expression(initialiser, &body)?,
            }
        }
    };
    Ok(RustItem::PackageValue {
        docs: docs_of(declaration, resolver)?,
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
    if literal_string {
        return Ok(borrowed_str());
    }
    // A SEQUENCE OF LITERALS is an ARRAY here, not a growable one. The pack maps the source's slice
    // to the target's growable sequence, which is right for a value a body builds and wrong for a
    // constant: `vec![..]` ALLOCATES, and a constant's initialiser has to be evaluable before the
    // program runs. The length is known — it is how many elements the source wrote — so the array
    // is available and the growable one is not.
    //
    // This is the hole in the constant-expression test, which admitted "a COMPOSITE literal ...
    // because a struct, tuple and array constructor are all const in the target". True of all
    // three, and the source's slice was becoming none of them.
    if let Some(elements) = constant_sequence(declaration) {
        let inner = match declaration.type_ref.args.first() {
            // The element is the source's STRING and every element is a literal, so each is the
            // target's borrowed string for exactly the reason a scalar string constant is: a
            // literal is already static storage, and owning it would allocate per element at a
            // point where nothing may allocate at all.
            Some(element) if element.name == SOURCE_STRING => borrowed_str(),
            Some(element) => resolver.resolve(element, &declaration.name)?,
            None => {
                return Err(TransformError::Unsupported {
                    name: declaration.name.clone(),
                    detail: "a sequence constant whose element type the front end did not record \
                             has no array type to declare, and the growable sequence the pack maps \
                             its type to cannot stand in a constant"
                        .to_owned(),
                });
            }
        };
        return Ok(RustType::Array {
            inner: Box::new(inner),
            len: elements,
        });
    }
    resolver.resolve(&declaration.type_ref, &declaration.name)
}

/// The target's borrowed string, which is what a string LITERAL already is.
fn borrowed_str() -> RustType {
    RustType::Reference {
        mutable: false,
        inner: Box::new(RustType::Path(TARGET_STR.to_owned())),
    }
}

/// How many elements, when this declaration is a sequence of literals and nothing else.
///
/// Every element must be a LITERAL. A sequence holding a call or a name is not one this turns into
/// an array — the constant-expression proof answers that separately, and guessing here would
/// declare an array type for an initialiser that then refuses.
fn constant_sequence(declaration: &Declaration) -> Option<usize> {
    if declaration.type_ref.kind != "slice" || declaration.children.is_empty() {
        return None;
    }
    let [composite] = declaration.children.as_slice() else {
        return None;
    };
    if composite.kind != KIND_COMPOSITE || composite.children.is_empty() {
        return None;
    }
    composite
        .children
        .iter()
        .all(|element| element.kind == KIND_LITERAL)
        .then_some(composite.children.len())
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

/// A sequence of BYTE LITERALS as the target's byte-string literal.
///
/// Every element must be a byte literal in the printable ASCII range and none may need escaping:
/// `b"..."` carries its bytes literally, so a quote, a backslash or anything unprintable would have
/// to be escaped by rules this does not implement, and getting one wrong changes a byte.
fn byte_string(initialiser: &Declaration, body: &Body<'_>) -> Option<RustExpr> {
    let mut bytes = String::new();
    for element in &initialiser.children {
        let RustExpr::Literal(spelled) = expression(element, body).ok()? else {
            return None;
        };
        let inner = spelled.strip_prefix("b'")?.strip_suffix('\'')?;
        let [byte] = inner.as_bytes() else {
            return None;
        };
        if !byte.is_ascii_graphic() || *byte == b'"' || *byte == b'\\' {
            return None;
        }
        bytes.push(char::from(*byte));
    }
    match bytes.is_empty() {
        true => None,
        // DEREFERENCED, because `b"..."` has type `&[u8; N]` and the declared type is `[u8; N]`.
        false => Some(RustExpr::Deref(Box::new(RustExpr::Literal(format!(
            "b\"{bytes}\""
        ))))),
    }
}
