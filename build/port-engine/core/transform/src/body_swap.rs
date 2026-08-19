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
use port_engine_rust_ir::RustExpr;

use crate::body::Body;
use crate::body_expr::expression;
use crate::error::TransformError;
use crate::naming::to_snake_case;
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
