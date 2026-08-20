//! Where the SOURCE's type system is wider than the target's, and the difference is not a spelling.
//!
//! Two constructs so far, and they have the same shape: the source admits something the target does
//! not, and translating the parts without the whole emits a crate that does not build. Both refuse
//! by name, and both name the rule that would replace the refusal.
//!
//! Kept together rather than in the files that call them because that is what they are about — not
//! statements, not operators, but the places where a faithful translation needs a construct the
//! target lacks and the engine must say so rather than approximate.

use std::collections::BTreeSet;

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustExpr, RustStmt};

use crate::naming::to_snake_case;

use crate::body::Body;
use crate::error::TransformError;

/// The BINDINGS a signature's named results are, which the body may assign to before returning.
///
/// The source lets a signature name its results, and those names are variables: zero-initialised at
/// entry, assigned during the body, and returned by a bare `return`. The target has no such thing —
/// a result is a value the body produces, and a name in the signature binds nothing.
///
/// So the names are bound at the top of the body, at the zero value the source gives them, and the
/// rest of the translation needs no special case: an assignment to one is an ordinary assignment,
/// and a `return sec, nsec` is an ordinary return. Translating those assignments WITHOUT this
/// emitted a body naming variables that do not exist — the same dangling-name defect
/// self-containment refuses everywhere else, arrived at from the signature rather than from a call.
///
/// `mut` only where the body actually assigns, because a binding declared mutable and never written
/// is a warning the compile proof denies — and the whole reason to bind these is that the body
/// writes them.
///
/// # Errors
/// [`TransformError::Unsupported`] when a result's zero value is one the pack does not give.
pub(crate) fn named_result_bindings(
    declaration: &Declaration,
    resolver: &crate::resolve::Resolver<'_>,
) -> Result<Vec<RustStmt>, TransformError> {
    let mut written = BTreeSet::new();
    for child in &declaration.children {
        collect_assigned(child, resolver, &mut written);
    }
    let mut bindings = Vec::new();
    for result in declaration.children_of_kind(crate::vocabulary::CHILD_RESULT) {
        if result.name.is_empty() {
            continue;
        }
        let Some(zero) = resolver.zero_value(&result.type_ref) else {
            return Err(TransformError::Unsupported {
                name: declaration.name.clone(),
                detail: format!(
                    "its result `{}` is NAMED, which makes it a binding the body may assign to \
                     before returning, and the pack gives no zero value for its type — the source \
                     starts such a binding at one and the target has to spell it",
                    result.name
                ),
            });
        };
        bindings.push(RustStmt::Let {
            name: to_snake_case(&result.name),
            mutable: written.contains(&result.name),
            ty: Some(resolver.resolve_in(
                &result.type_ref,
                &declaration.name,
                crate::vocabulary::POSITION_PARAM,
            )?),
            value: Some(RustExpr::Literal(zero)),
        });
    }
    Ok(bindings)
}

/// Names this subtree ASSIGNS to, so a binding is declared mutable only where it is written.
fn collect_assigned(
    node: &Declaration,
    resolver: &crate::resolve::Resolver<'_>,
    into: &mut BTreeSet<String>,
) {
    // A call that WRITES INTO an argument assigns to it. The source spells the write as a call and
    // the target spells it as a mutation, so a binding that only ever appears as such an argument is
    // never seen assigned — and comes out immutable, and does not compile. Which callees write is
    // the pack's to say; it already names them, because they are the same ones it maps.
    if node.kind == crate::vocabulary::KIND_CALL
        && crate::body_bytes::writes_into_first_argument(node, resolver)
        && let Some(destination) = node.children.get(1)
    {
        into.extend(root_name(destination));
    }
    if node.kind == "assign"
        && let Some(target) = node.children.first()
        && target.kind == crate::vocabulary::KIND_IDENT
    {
        into.insert(target.name.clone());
    }
    if node.kind == "assign_tuple" {
        for place in node.children_of_kind(crate::vocabulary::CHILD_PLACE) {
            if let Some(target) = place.children.first()
                && target.kind == crate::vocabulary::KIND_IDENT
            {
                into.insert(target.name.clone());
            }
        }
    }
    for child in &node.children {
        collect_assigned(child, resolver, into);
    }
}

/// The identifier a place expression is rooted at — `out` for `out`, `out[:]`, `out[1:2]`.
fn root_name(node: &Declaration) -> Option<String> {
    match node.kind.as_str() {
        crate::vocabulary::KIND_IDENT => Some(node.name.clone()),
        _ => node.children.first().and_then(root_name),
    }
}

/// Refuse an operation the target's NEWTYPE does not support and the source's defined type did.
///
/// The source's `type Version byte` is transparent: it compares against an untyped constant, it
/// formats as a number, it does arithmetic. The target's newtype is opaque — a distinct type with
/// none of its underlying type's operators — and that opacity is the whole reason the newtype is
/// the faithful shape, because it keeps the distinction the source declared.
///
/// So the two disagree exactly here, and the disagreement is invisible until the operand is used:
/// `if v > 15` in the source became `if self > 15` on a `&Version`, which does not compile, and
/// `%d` of one became `format!("{}", self)` on a type with no `Display`.
///
/// The rule this wants is `self.0` — a defined type over a numeric underlying is its wrapped value
/// wherever the underlying type is what the operation needs. Refused by name until it is written,
/// because the alternative is not a wrong program but a crate that does not build, and because
/// unwrapping without proving the underlying type is numeric would silently reach inside a newtype
/// declared to stop exactly that.
///
/// # Errors
/// [`TransformError::Unsupported`] naming the declaration and the type whose opacity blocks it.
pub(crate) fn refuse_opaque_newtype(
    left: &Declaration,
    right: &Declaration,
    spelling: &str,
    cx: &Body<'_>,
) -> Result<(), TransformError> {
    for (operand, other) in [(left, right), (right, left)] {
        let Some(name) = local_named_type(operand, cx) else {
            continue;
        };
        // The SAME defined type on both sides is what the newtype supports through its derives —
        // two `Version`s compare because the type derives the comparison. Anything else asks for an
        // operator the newtype does not have.
        if local_named_type(other, cx).as_deref() == Some(name.as_str()) {
            continue;
        }
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "`{spelling}` is applied to `{name}`, a type this unit DEFINES, and to something \
                 that is not one — the source's defined type carries its underlying type's \
                 operators and the target's newtype carries none, which is what keeps the \
                 distinction the source declared"
            ),
        });
    }
    Ok(())
}

/// The name of a locally declared defined type this operand has, if it has one.
fn local_named_type(operand: &Declaration, cx: &Body<'_>) -> Option<String> {
    // The RECEIVER carries no type of its own — it is not a child of the method and the front end
    // records nothing for it — so what `self` IS comes from the signature, which is the only place
    // that knows. Without this the check saw every method on a defined type as untyped and let
    // `self - X` through, which does not compile.
    if crate::body_ops::is_receiver(operand) {
        return cx
            .receiver_type
            .and_then(|owner| cx.resolver.scope.types.get(owner))
            .cloned();
    }
    let type_ref = &operand.type_ref;
    if type_ref.kind != "named" {
        return None;
    }
    // NOT filtered by what is emitted. The fixpoint that decides emittability only ever SHRINKS,
    // and that is what makes it terminate; a refusal that consults it would stop firing as the set
    // lost members, un-refusing a declaration and growing the set again. It hung on the first
    // package that had one. Whether a type is a newtype is a fact about the source, so ask the
    // source.
    cx.resolver.scope.types.get(&type_ref.name).cloned()
}

/// The NAMED results a bare return hands back, as one expression.
///
/// The source binds named results at entry and `return` with no operands returns whatever they hold.
/// The target has no such binding, so the names have to be spelled — one of them directly, several as
/// the tuple the signature renders them as.
///
/// `None` where the enclosing declaration has no named results, which is when a bare return really
/// does return nothing.
pub(crate) fn named_results(cx: &crate::body::Body<'_>) -> Option<port_engine_rust_ir::RustExpr> {
    let names = cx.named_results.clone();
    match names.len() {
        0 => None,
        1 => names
            .into_iter()
            .next()
            .map(port_engine_rust_ir::RustExpr::Path),
        _ => Some(port_engine_rust_ir::RustExpr::Tuple(
            names.into_iter().map(port_engine_rust_ir::RustExpr::Path).collect(),
        )),
    }
}
