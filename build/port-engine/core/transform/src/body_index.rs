//! Indexing and slicing.
//!
//! One module because they are the same question asked twice: what a subscript MEANS in the target.
//! The source indexes and slices the same backing array; the target distinguishes an element from a
//! borrowed view of several, and getting that wrong is a copy where the source had none.

use port_engine_api::Declaration;
use port_engine_rust_ir::RustExpr;

use crate::body::Body;
use crate::body_expr::expression;
use crate::error::TransformError;

/// An index operand, converted to what the target insists on.
///
/// The source indexes with its `int`; the target indexes with `usize`, and that mismatch has to
/// land somewhere. It used to land on `len`, which was mapped so its value would type as the
/// source's `int` — right for `return len(s)` and wrong for the counter of every indexed loop,
/// whose body then would not compile.
///
/// Here it lands at the only place the target actually insists: a LITERAL needs nothing, because
/// the target infers it, and every other operand converts.
///
/// The trade, recorded rather than discovered: a negative index panics in both languages for
/// different reasons — the source bounds-checks a negative, the target wraps it to an enormous
/// `usize` and bounds-checks that. Same outcome, different message.
pub(crate) fn index_operand(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let operand = expression(node, cx)?;
    if node.kind == "literal" {
        return Ok(operand);
    }
    Ok(RustExpr::Cast {
        expr: Box::new(operand),
        ty: port_engine_rust_ir::RustType::path("usize"),
    })
}

/// `s[lo:hi]` — a BORROWED subrange, with either bound optional.
///
/// The bounds arrive positionally with an explicit `absent` node for the ones the source left out,
/// so `s[:hi]` and `s[lo:]` stay distinguishable. Reconstructing that from arity would be guessing
/// which end was missing.
pub(crate) fn slice(node: &Declaration, cx: &Body<'_>) -> Result<RustExpr, TransformError> {
    let [base, low, high] = node.children.as_slice() else {
        return Err(TransformError::Unsupported {
            name: cx.owner.to_owned(),
            detail: format!(
                "a slice expression needs a base and both bounds, got {} operands",
                node.children.len()
            ),
        });
    };
    // A slice bound is an index like any other, and needs the same conversion for the same reason.
    let bound = |operand: &Declaration| -> Result<Option<Box<RustExpr>>, TransformError> {
        if operand.kind == "absent" {
            return Ok(None);
        }
        Ok(Some(Box::new(index_operand(operand, cx)?)))
    };
    Ok(RustExpr::Slice {
        base: Box::new(expression(base, cx)?),
        low: bound(low)?,
        high: bound(high)?,
    })
}
