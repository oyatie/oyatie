//! A parallel assignment that EXCHANGES two elements of one sequence.
//!
//! `a[i], a[j] = a[j], a[i]` is the shape, and the target's sequence has a method for exactly it.
//! The parallel assignment the engine already emits is faithful — both sides are evaluated before
//! either is written, which is what makes it an exchange rather than two writes — so this changes
//! the spelling and not the program.
//!
//! Recognised from the SOURCE nodes rather than from the rendered places, so the match cannot
//! depend on how they printed: a change to how an index renders would otherwise silently stop the
//! idiom firing, and nothing would say so.

use port_engine_api::Declaration;
use port_engine_rust_ir::{MatchArm, RustExpr, RustStmt};

use crate::body::Body;
use crate::body_call::render_operand;
use crate::body_expr::expression;
use crate::error::TransformError;
use crate::naming::{to_pascal_case, to_snake_case};
use crate::vocabulary::{CHILD_PLACE, CHILD_VALUE, IDIOM_SWAP, KIND_IDENT, KIND_INDEX};

/// The exchange this parallel assignment is, if it is one.
///
/// Requires exactly two places and two values, every one an index into the SAME sequence by a plain
/// name, and the values naming the two indices in the opposite order. Anything else is a parallel
/// assignment that happens to have two of each and keeps its own form.
///
/// # Errors
/// [`TransformError`] from translating the sequence or either index.
pub(crate) fn exchange(
    node: &Declaration,
    cx: &Body<'_>,
) -> Result<Option<RustExpr>, TransformError> {
    let places = node.children_of_kind(CHILD_PLACE);
    let values = node.children_of_kind(CHILD_VALUE);
    let ([first, second], [third, fourth]) = (places.as_slice(), values.as_slice()) else {
        return Ok(None);
    };
    let (Some(a), Some(b), Some(c), Some(d)) = (
        indexed(first),
        indexed(second),
        indexed(third),
        indexed(fourth),
    ) else {
        return Ok(None);
    };
    // One sequence, and the values are its two indices the other way round. Comparing the index
    // NODES by name is what makes this an exchange rather than two coincidentally shaped writes.
    if a.0 != b.0 || a.0 != c.0 || a.0 != d.0 || a.1 != d.1 || b.1 != c.1 || a.1 == b.1 {
        return Ok(None);
    }
    let method = cx
        .resolver
        .idiom_method(IDIOM_SWAP)
        .unwrap_or("swap")
        .to_owned();
    Ok(Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Path(to_snake_case(a.0))),
        method,
        args: vec![
            place_index(first, cx)?,
            place_index(second, cx)?,
        ],
    }))
}

/// The `(sequence, index)` names this place indexes, when both are plain names.
///
/// A base or an index that is anything else — a field, a call, an expression — is not a name this
/// can compare, and comparing rendered text instead would make the match depend on spelling.
fn indexed(place: &Declaration) -> Option<(&str, &str)> {
    let node = index_node(place);
    if node.kind != KIND_INDEX {
        return None;
    }
    let [base, operand] = node.children.as_slice() else {
        return None;
    };
    match base.kind == KIND_IDENT && operand.kind == KIND_IDENT {
        true => Some((base.name.as_str(), operand.name.as_str())),
        false => None,
    }
}

/// The expression inside a `place` or `value` wrapper.
fn index_node(wrapper: &Declaration) -> &Declaration {
    wrapper.children.first().unwrap_or(wrapper)
}

/// The index operand of a place, translated as the target's own index wants it.
///
/// # Errors
/// [`TransformError`] from translating the operand.
fn place_index(place: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let node = index_node(place);
    match node.children.get(1) {
        Some(operand) => crate::body_index::index_operand(operand, cx),
        None => expression(node, cx),
    }
}

/// A match that only answers YES or NO, as the membership test it is.
///
/// `match x { "x" | "*" | "X" => true, _ => false }` is a test of whether `x` is one of three
/// things, and the target has a macro that is exactly that. The source has to spell it as a switch
/// because its switch is the only multi-pattern form it has.
///
/// Recognised from the ARMS AS BUILT rather than from the source shape, so it cannot fire on a match
/// that merely looks like one — what matters is what the arms yield, and that is known only after
/// they are translated.
///
/// `None` unless the default arm yields `false` and every other yields `true`. The reverse is a
/// NEGATED membership test and needs its own form; inverting it silently would be a different
/// expression wearing this one's shape.
pub(crate) fn membership(arms: &[MatchArm], scrutinee: &RustExpr) -> Option<RustExpr> {
    let (default, tested) = arms.split_last()?;
    if !default.patterns.is_empty() || !yields(&default.body, "false") {
        return None;
    }
    let mut patterns = Vec::new();
    for arm in tested {
        if arm.patterns.is_empty() || !yields(&arm.body, "true") {
            return None;
        }
        patterns.extend(arm.patterns.iter().map(render_operand).collect::<Option<Vec<_>>>()?);
    }
    if patterns.is_empty() {
        return None;
    }
    Some(RustExpr::Literal(format!(
        "matches!({}, {})",
        render_operand(scrutinee)?,
        patterns.join(" | ")
    )))
}

/// Whether this arm body is exactly the given boolean literal.
fn yields(body: &[RustStmt], literal: &str) -> bool {
    matches!(body, [RustStmt::Tail(RustExpr::Path(value) | RustExpr::Literal(value))]
        if value == literal)
}

/// `err == ErrSize` — is this failure that SENTINEL?
///
/// The source compares identity, because its sentinel is a pointer and every `return ErrSize` hands
/// back the same one. The target asks the trait object what concrete type it holds, which is true
/// in exactly the same cases.
///
/// Available only because the sentinel became a TYPE. While it was its message there was nothing to
/// compare — a fresh failure built from a shared string is equal to nothing — and this refused by
/// name, with the loss recorded as the cost of that decision.
///
/// `None` unless one side names a sentinel of this unit and the operator is an equality. A `!=` is
/// the same test negated, which is what the source means by it.
///
/// # Errors
/// [`TransformError`] from translating the other operand.
pub(crate) fn identity_test(
    spelling: &str,
    lhs: &Declaration,
    rhs: &Declaration,
    cx: &Body<'_>,
) -> Result<Option<RustExpr>, TransformError> {
    let negated = match spelling {
        "==" => false,
        "!=" => true,
        _ => return Ok(None),
    };
    let Some(convention) = cx.resolver.failure else {
        return Ok(None);
    };
    if convention.identity_test.is_empty() {
        return Ok(None);
    }
    // Through the RESOLVER, which is where the sentinel's type name is decided once. Casing it
    // here instead is how the declaration came out `Gone` and the test asking about it came out
    // `ErrGone` — the exact disagreement that decision exists to prevent, reintroduced by the one
    // site that did not ask.
    let sentinel_of = |node: &Declaration| {
        (node.kind == KIND_IDENT && cx.resolver.scope.sentinels.contains_key(&node.name))
            .then(|| cx.resolver.sentinel_type_name(&node.name))
    };
    let (sentinel, subject) = match (sentinel_of(lhs), sentinel_of(rhs)) {
        // BOTH sides a sentinel is a comparison of two known types, which the source can write and
        // which this form does not answer: it asks what a failure holds, and neither side is one.
        (Some(_), Some(_)) | (None, None) => return Ok(None),
        (Some(name), None) => (name, rhs),
        (None, Some(name)) => (name, lhs),
    };
    let Some(rendered) = render_operand(&expression(subject, cx)?) else {
        return Ok(None);
    };
    let test = convention
        .identity_test
        .replace("{0}", &rendered)
        .replace("{1}", &sentinel);
    Ok(Some(RustExpr::Literal(match negated {
        true => format!("!{test}"),
        false => test,
    })))
}
