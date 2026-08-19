//! The comparison LADDER, which is the target's `cmp` written out.
//!
//! `if v < o { -1 } if v > o { 1 } 0` is how the source spells a three-way comparison, and it
//! spells it that way because it has no such method — its sort and its comparison interfaces are
//! defined in terms of a signed integer. A reviewer reading a real ported package put it exactly:
//! the ladder is NECESSARY there and dead here.
//!
//! Split from `returns.rs` because that file decides what a result IS and this decides what a body
//! that produces one can be replaced by.

use port_engine_api::Declaration;

use crate::vocabulary::{ATTR_OP, ATTR_VALUE, KIND_IDENT, KIND_RETURN, KIND_UNARY};

/// The two operands a three-way comparison LADDER compares, if the body is exactly one.
///
/// `if v < o { -1 } if v > o { 1 } 0` is how the source spells `cmp`, and it spells it that way
/// because it has no such method. Recognised structurally and strictly: two `if`s over the SAME
/// pair in the same order, one `<` and one `>`, each returning the matching extreme, and a trailing
/// equal. Anything else is a comparison that does something of its own and keeps its branches.
pub(crate) fn comparison_ladder_of(
    statements: &[Declaration],
    _cx: &crate::body::Body<'_>,
) -> Option<(String, String)> {
    let [less, greater, equal] = statements else {
        return None;
    };
    if !returns_literal(equal, "0") {
        return None;
    }
    let (a, b) = branch_operands(less, "<", "-1")?;
    let (c, d) = branch_operands(greater, ">", "1")?;
    match a == c && b == d {
        true => Some((a, b)),
        false => None,
    }
}

/// The two names an `if <a> <op> <b> { return <literal> }` compares, when that is all it is.
fn branch_operands(node: &Declaration, op: &str, literal: &str) -> Option<(String, String)> {
    if node.kind != "if" || !node.children_of_kind("else").is_empty() {
        return None;
    }
    let conditions = node.children_of_kind("cond");
    let test = conditions.first()?.children.first()?;
    if test.kind != "binary" || test.attr(ATTR_OP) != Some(op) {
        return None;
    }
    let [left, right] = test.children.as_slice() else {
        return None;
    };
    if left.kind != KIND_IDENT || right.kind != KIND_IDENT {
        return None;
    }
    let branches = node.children_of_kind("then");
    let then = branches.first()?;
    let [only] = then.children.as_slice() else {
        return None;
    };
    match returns_literal(only, literal) {
        true => Some((left.name.clone(), right.name.clone())),
        false => None,
    }
}

/// Whether this statement returns exactly the given integer literal, negative sign included.
fn returns_literal(node: &Declaration, literal: &str) -> bool {
    if node.kind != KIND_RETURN {
        return false;
    }
    let [only] = node.children.as_slice() else {
        return false;
    };
    match literal.strip_prefix('-') {
        Some(magnitude) => {
            only.kind == KIND_UNARY
                && only.attr(ATTR_OP) == Some("-")
                && only
                    .children
                    .first()
                    .is_some_and(|inner| inner.attr(ATTR_VALUE) == Some(magnitude))
        }
        None => only.attr(ATTR_VALUE) == Some(literal),
    }
}
