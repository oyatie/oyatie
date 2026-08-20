//! IDIOM rules: spellings the target prefers for something the source says another way.
//!
//! Separated from the expression walk because an idiom is a different KIND of rule from everything
//! around it. Every other decision here changes what the emitted program does or refuses to; an
//! idiom changes only how it reads, and an idiom that alters meaning is not an idiom but a bug.
//!
//! Each rule is pack data carrying its seed provenance, because
//! `specs/k8s-port/licensing.json` rejects a rust-skills-derived rule without `seed_source`,
//! `seed_license` and `seed_commit` — a rule whose derivation cannot be re-checked is a rule
//! nobody can audit.

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustExpr, UnaryOp};

use crate::body::Body;
use crate::body_expr::expression;
use crate::error::TransformError;
use crate::vocabulary::{ATTR_LIT_KIND, ATTR_VALUE, IDIOM_EMPTY_STRING, KIND_LITERAL, LIT_KIND_STRING};

/// `x == ""` and `x != ""`, which the target spells as a method.
///
/// Exactly equivalent — both are true precisely when the value has length zero — so this changes
/// the spelling and not the program, which is what makes it an idiom rather than a translation.
/// `clippy::style` flags the comparison form, and a comment a reviewer would make on hand-written
/// code is a defect in code held to that bar.
///
/// Either operand may be the literal, because the source permits `"" == x` too.
///
/// # Errors
/// [`TransformError`] from translating the non-literal operand.
pub(crate) fn emptiness_test(
    spelling: &str,
    lhs: &Declaration,
    rhs: &Declaration,
    cx: &Body<'_>,
) -> Result<Option<RustExpr>, TransformError> {
    let Some(method) = cx.resolver.idiom_method(IDIOM_EMPTY_STRING) else {
        return Ok(None);
    };
    let negated = match spelling {
        "==" => false,
        "!=" => true,
        _ => return Ok(None),
    };
    let subject = match (is_empty_string(lhs), is_empty_string(rhs)) {
        (true, false) => rhs,
        (false, true) => lhs,
        // Both literals is a comparison of two constants, and neither is a subject to ask about
        // emptiness; neither is not this shape at all.
        _ => return Ok(None),
    };

    let call = RustExpr::MethodCall {
        // A PLACE, because a method receiver is borrowed and not consumed. Built as a value it
        // CLONES — a field read of a non-copying type moves in the target, so the value position
        // asks for a copy — and `self.name.clone().is_empty()` allocates a string to ask whether
        // it is empty and drops it again. Both review gates named that allocation.
        receiver: Box::new(crate::body_expr::in_position(
            subject,
            cx,
            crate::body_expr::Position::Place,
        )?),
        method: method.to_owned(),
        args: Vec::new(),
    };
    Ok(Some(match negated {
        false => call,
        true => RustExpr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(call),
        },
    }))
}

/// Whether this node is the empty string literal the source compares against.
fn is_empty_string(node: &Declaration) -> bool {
    node.kind == KIND_LITERAL
        && node.attr(ATTR_LIT_KIND) == Some(LIT_KIND_STRING)
        && node.attr(ATTR_VALUE) == Some("\"\"")
}

/// `len(x) == 0` and `len(x) > 0`, which the target spells as a method on the sequence.
///
/// Exactly equivalent, and the same idiom the empty-STRING comparison above already is: a length
/// is zero precisely when the sequence is empty. The source has no such method and must compare,
/// which is why the comparison survives a mechanical port and reads as one — and here it reads
/// doubly so, because the length's own mapping adds a conversion first: `(buf.len() as i64) == 0`.
///
/// The literal must be ZERO and the operator one that tests against it. `len(x) > 1` is a different
/// question with no method, and `len(x) >= 0` is always true — neither is this shape.
pub(crate) fn emptiness_of_length(
    spelling: &str,
    lhs: &Declaration,
    rhs: &Declaration,
    cx: &Body<'_>,
) -> Result<Option<RustExpr>, TransformError> {
    let Some(method) = cx.resolver.idiom_method(IDIOM_EMPTY_STRING) else {
        return Ok(None);
    };
    // The LENGTH on one side and the literal zero on the other, in either order — with the operator
    // read from the length's side, so `0 < len(x)` and `len(x) > 0` reach the same answer.
    let (length, negated) = match (is_zero(lhs), is_zero(rhs)) {
        (true, false) => (
            rhs,
            match spelling {
                "==" => false,
                "!=" | "<" => true,
                _ => return Ok(None),
            },
        ),
        (false, true) => (
            lhs,
            match spelling {
                "==" => false,
                "!=" | ">" => true,
                _ => return Ok(None),
            },
        ),
        _ => return Ok(None),
    };
    if length.kind != crate::vocabulary::KIND_CALL
        || !length
            .attr(crate::vocabulary::ATTR_CALLEE)
            .is_some_and(|callee| cx.resolver.length_functions.contains(callee))
    {
        return Ok(None);
    }
    let [_, sequence] = length.children.as_slice() else {
        return Ok(None);
    };
    let call = RustExpr::MethodCall {
        // A PLACE: the method borrows, and a value position would copy the sequence to ask whether
        // it has anything in it.
        receiver: Box::new(crate::body_index::unwrapped_in(
            sequence,
            cx,
            crate::body_expr::Position::Place,
        )?),
        method: method.to_owned(),
        args: Vec::new(),
    };
    Ok(Some(match negated {
        true => RustExpr::Unary {
            op: port_engine_rust_ir::UnaryOp::Not,
            operand: Box::new(call),
        },
        false => call,
    }))
}

/// Whether this operand is the literal zero.
fn is_zero(node: &Declaration) -> bool {
    node.kind == KIND_LITERAL && node.attr(ATTR_VALUE) == Some("0")
}
