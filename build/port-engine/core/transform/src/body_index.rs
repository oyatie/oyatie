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
    let bound = |operand: &Declaration| -> Result<Option<Box<RustExpr>>, TransformError> {
        if operand.kind == "absent" {
            return Ok(None);
        }
        Ok(Some(Box::new(expression(operand, cx)?)))
    };
    Ok(RustExpr::Slice {
        base: Box::new(expression(base, cx)?),
        low: bound(low)?,
        high: bound(high)?,
    })
}
